#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_long, c_short, c_uchar};
use core::mem::transmute;
use core::ptr::addr_of;

use super::mhead_h::{DEC_INFO, MPEG_HEAD};
use super::mp3struct_h::{pMP3Stream, LP_MP3STREAM, SBT_FUNCTION};
use super::small_header_h::IN_OUT;

pub type AUDIO_DECODE_ROUTINE =
    Option<unsafe extern "C" fn(bs: *mut c_uchar, pcm: *mut c_short) -> IN_OUT>;

static steps: [c_long; 18] = [
    0, 3, 5, 7, 9, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383,
    32767, 65535,
];

/* ABCD_INDEX = lookqt[mode][sr_index][br_index]  */
/* -1 = invalid  */
static lookqt: [[[i8; 16]; 3]; 4] = [
    [
        [1, -1, -1, -1, 2, -1, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1], /*  44ks stereo */
        [0, -1, -1, -1, 2, -1, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1], /*  48ks */
        [1, -1, -1, -1, 3, -1, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1], /*  32ks */
    ],
    [
        [1, -1, -1, -1, 2, -1, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1], /*  44ks joint stereo */
        [0, -1, -1, -1, 2, -1, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1], /*  48ks */
        [1, -1, -1, -1, 3, -1, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1], /*  32ks */
    ],
    [
        [1, -1, -1, -1, 2, -1, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1], /*  44ks dual chan */
        [0, -1, -1, -1, 2, -1, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1], /*  48ks */
        [1, -1, -1, -1, 3, -1, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1], /*  32ks */
    ],
    // mono extended beyond legal br index
    //  1,2,2,0,0,0,1,1,1,1,1,1,1,1,1,-1,          /*  44ks single chan */
    //  0,2,2,0,0,0,0,0,0,0,0,0,0,0,0,-1,          /*  48ks */
    //  1,3,3,0,0,0,1,1,1,1,1,1,1,1,1,-1,          /*  32ks */
    // legal mono
    [
        [1, 2, 2, 0, 0, 0, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1], /*  44ks single chan */
        [0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, -1, -1], /*  48ks */
        [1, 3, 3, 0, 0, 0, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1], /*  32ks */
    ],
];

static sr_table: [c_long; 8] = [22050, 24000, 16000, 1, 44100, 48000, 32000, 1];

/* bit allocation table look up */
/* table per mpeg spec tables 3b2a/b/c/d  /e is mpeg2 */
/* look_bat[abcd_index][4][16]  */
static look_bat: [[[c_uchar; 16]; 4]; 5] = [
    /* LOOK_BATA */
    [
        [0, 1, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17],
        [0, 1, 2, 3, 4, 5, 6, 17, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 2, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    /* LOOK_BATB */
    [
        [0, 1, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 17],
        [0, 1, 2, 3, 4, 5, 6, 17, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 2, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    /* LOOK_BATC */
    [
        [0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 2, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    /* LOOK_BATD */
    [
        [0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 2, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
    /* LOOK_BATE */
    [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 2, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ],
];

/* look_nbat[abcd_index]][4] */
static look_nbat: [[c_uchar; 4]; 5] = [
    [3, 8, 12, 4],
    [3, 8, 12, 7],
    [2, 0, 6, 0],
    [2, 0, 10, 0],
    [4, 0, 7, 19],
];

unsafe extern "C" {
    pub static mut decinfo: DEC_INFO;
    pub static mut look_c_value: [f32; 18];
    pub static mut sf_table: [f32; 64];
    pub static mut sample: [f32; 2304 * 2];
    pub static mut group3_table: [[i8; 3]; 32];
    pub static mut group5_table: [[i8; 3]; 128];
    pub static mut group9_table: [[c_short; 3]; 1024];
    pub static mut audio_decode_routine: AUDIO_DECODE_ROUTINE;

    pub fn sbt_mono(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt_dual(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt_dual_mono(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt_dual_left(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt_dual_right(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt16_mono(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt16_dual(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt16_dual_mono(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt16_dual_left(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt16_dual_right(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt8_mono(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt8_dual(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt8_dual_mono(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt8_dual_left(sample: *mut f32, pcm: *mut c_short, n: c_int);
    pub fn sbt8_dual_right(sample: *mut f32, pcm: *mut c_short, n: c_int);

    /*--- 8 bit output ---*/
    pub fn sbtB_mono(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB_dual(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB_dual_mono(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB_dual_left(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB_dual_right(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB16_mono(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB16_dual(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB16_dual_mono(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB16_dual_left(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB16_dual_right(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB8_mono(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB8_dual(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB8_dual_mono(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB8_dual_left(sample: *mut f32, pcm: *mut c_uchar, n: c_int);
    pub fn sbtB8_dual_right(sample: *mut f32, pcm: *mut c_uchar, n: c_int);

    pub fn audio_decode_initL1(
        h: *mut MPEG_HEAD,
        framebytes_arg: c_int,
        reduction_code: c_int,
        transform_code: c_int,
        convert_code: c_int,
        freq_limit: c_int,
    ) -> c_int;
    pub fn sbt_init();

    pub fn L1audio_decode(bs: *mut c_uchar, pcm: *mut c_short) -> IN_OUT;
    pub fn L2audio_decode(bs: *mut c_uchar, pcm: *mut c_short) -> IN_OUT;
    pub fn L3audio_decode(bs: *mut c_uchar, pcm: *mut c_uchar) -> IN_OUT;

    pub fn L1audio_decode_init(
        h: *mut MPEG_HEAD,
        framebytes_arg: c_int,
        reduction_code: c_int,
        transform_code: c_int,
        convert_code: c_int,
        freq_limit: c_int,
    ) -> c_int;
    pub fn L3audio_decode_init(
        h: *mut MPEG_HEAD,
        framebytes_arg: c_int,
        reduction_code: c_int,
        transform_code: c_int,
        convert_code: c_int,
        freq_limit: c_int,
    ) -> c_int;
}

const unsafe fn sbt8_cast(
    f: unsafe extern "C" fn(sample: *mut f32, pcm: *mut c_uchar, n: c_int),
) -> unsafe extern "C" fn(sample: *mut f32, pcm: *mut c_short, n: c_int) {
    unsafe { transmute(f) }
}

const unsafe fn decode_cast(
    f: unsafe extern "C" fn(bs: *mut c_uchar, pcm: *mut c_uchar) -> IN_OUT,
) -> unsafe extern "C" fn(bs: *mut c_uchar, pcm: *mut c_short) -> IN_OUT {
    unsafe { transmute(f) }
}

static sbt_table: [[[SBT_FUNCTION; 5]; 3]; 2] = [
    [
        [
            Some(sbt_mono),
            Some(sbt_dual),
            Some(sbt_dual_mono),
            Some(sbt_dual_left),
            Some(sbt_dual_right),
        ],
        [
            Some(sbt16_mono),
            Some(sbt16_dual),
            Some(sbt16_dual_mono),
            Some(sbt16_dual_left),
            Some(sbt16_dual_right),
        ],
        [
            Some(sbt8_mono),
            Some(sbt8_dual),
            Some(sbt8_dual_mono),
            Some(sbt8_dual_left),
            Some(sbt8_dual_right),
        ],
    ],
    [
        [
            Some(unsafe { sbt8_cast(sbtB_mono) }),
            Some(unsafe { sbt8_cast(sbtB_dual) }),
            Some(unsafe { sbt8_cast(sbtB_dual_mono) }),
            Some(unsafe { sbt8_cast(sbtB_dual_left) }),
            Some(unsafe { sbt8_cast(sbtB_dual_right) }),
        ],
        [
            Some(unsafe { sbt8_cast(sbtB16_mono) }),
            Some(unsafe { sbt8_cast(sbtB16_dual) }),
            Some(unsafe { sbt8_cast(sbtB16_dual_mono) }),
            Some(unsafe { sbt8_cast(sbtB16_dual_left) }),
            Some(unsafe { sbt8_cast(sbtB16_dual_right) }),
        ],
        [
            Some(unsafe { sbt8_cast(sbtB8_mono) }),
            Some(unsafe { sbt8_cast(sbtB8_dual) }),
            Some(unsafe { sbt8_cast(sbtB8_dual_mono) }),
            Some(unsafe { sbt8_cast(sbtB8_dual_left) }),
            Some(unsafe { sbt8_cast(sbtB8_dual_right) }),
        ],
    ],
];

static out_chans: [c_int; 5] = [1, 2, 1, 1, 1];

static decode_routine_table: [AUDIO_DECODE_ROUTINE; 4] = [
    Some(L2audio_decode),
    Some(unsafe { decode_cast(L3audio_decode) }),
    Some(L2audio_decode),
    Some(L1audio_decode),
];

/*---------------------------------------------------------*/
unsafe fn table_init() {
    let mut i: c_int;
    let mut j: c_int;
    let mut code: c_int;
    static mut iOnceOnly: c_int = 0;

    if iOnceOnly == 0 {
        iOnceOnly += 1;
        /*--  c_values (dequant) --*/
        i = 1;
        while i < 18 {
            look_c_value[i as usize] = 2.0f32 / steps[i as usize] as f32;
            i += 1;
        }

        /*--  scale factor table, scale by 32768 for 16 pcm output  --*/
        i = 0;
        while i < 64 {
            sf_table[i as usize] = (32768.0f64 * 2.0f64 * 2.0f64.powf(-(i as f64) / 3.0f64)) as f32;
            i += 1;
        }

        /*--  grouped 3 level lookup table 5 bit token --*/
        i = 0;
        while i < 32 {
            code = i;
            j = 0;
            while j < 3 {
                group3_table[i as usize][j as usize] = ((code % 3) - 1) as i8;
                code /= 3;
                j += 1;
            }
            i += 1;
        }

        /*--  grouped 5 level lookup table 7 bit token --*/
        i = 0;
        while i < 128 {
            code = i;
            j = 0;
            while j < 3 {
                group5_table[i as usize][j as usize] = ((code % 5) - 2) as i8;
                code /= 5;
                j += 1;
            }
            i += 1;
        }

        /*--  grouped 9 level lookup table 10 bit token --*/
        i = 0;
        while i < 1024 {
            code = i;
            j = 0;
            while j < 3 {
                group9_table[i as usize][j as usize] = ((code % 9) - 4) as c_short;
                code /= 9;
                j += 1;
            }
            i += 1;
        }
    } else {
        iOnceOnly += 1;
    }
}

/*---------------------------------------------------------*/
/* mpeg_head defined in mhead.h  frame bytes is without pad */
pub unsafe extern "C" fn audio_decode_init(
    h: *mut MPEG_HEAD,
    framebytes_arg: c_int,
    mut reduction_code: c_int,
    mut transform_code: c_int,
    mut convert_code: c_int,
    mut freq_limit: c_int,
) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    static mut first_pass: c_int = 1;
    let abcd_index: c_int;
    let samprate: c_long;
    let mut limit: c_int;
    let mut bit_code: c_int;
    let stream: LP_MP3STREAM;

    if first_pass != 0 {
        table_init();
        first_pass = 0;
    }

    /* select decoder routine Layer I,II,III */
    audio_decode_routine = decode_routine_table[((*h).option & 3) as usize];

    if (*h).option == 3 {
        /* layer I */
        return L1audio_decode_init(
            h,
            framebytes_arg,
            reduction_code,
            transform_code,
            convert_code,
            freq_limit,
        );
    }

    if (*h).option == 1 {
        /* layer III */
        return L3audio_decode_init(
            h,
            framebytes_arg,
            reduction_code,
            transform_code,
            convert_code,
            freq_limit,
        );
    }

    transform_code = transform_code; /* not used, asm compatability */
    bit_code = 0;
    if (convert_code & 8) != 0 {
        bit_code = 1;
    }
    convert_code = convert_code & 3; /* higher bits used by dec8 freq cvt */
    if reduction_code < 0 {
        reduction_code = 0;
    }
    if reduction_code > 2 {
        reduction_code = 2;
    }
    if freq_limit < 1000 {
        freq_limit = 1000;
    }

    stream = pMP3Stream;
    (*stream).framebytes = framebytes_arg;
    /* check if code handles */
    if (*h).option != 2 {
        return 0; /* layer II only */
    }
    if (*h).sr_index == 3 {
        return 0; /* reserved */
    }

    /* compute abcd index for bit allo table selection */
    if (*h).id != 0 {
        /* mpeg 1 */
        abcd_index = lookqt[(*h).mode as usize][(*h).sr_index as usize][(*h).br_index as usize] as c_int;
    } else {
        abcd_index = 4; /* mpeg 2 */
    }

    if abcd_index < 0 {
        return 0; // fail invalid Layer II bit rate index
    }

    i = 0;
    while i < 4 {
        j = 0;
        while j < 16 {
            (*stream).u.L1_2.bat[i as usize][j as usize] =
                look_bat[abcd_index as usize][i as usize][j as usize] as c_int;
            j += 1;
        }
        i += 1;
    }
    i = 0;
    while i < 4 {
        (*stream).u.L1_2.nbat[i as usize] = look_nbat[abcd_index as usize][i as usize] as c_int;
        i += 1;
    }
    (*stream).u.L1_2.max_sb = (*stream).u.L1_2.nbat[0]
        + (*stream).u.L1_2.nbat[1]
        + (*stream).u.L1_2.nbat[2]
        + (*stream).u.L1_2.nbat[3];
    /*----- compute pMP3Stream->nsb_limit --------*/
    samprate = sr_table[(4 * (*h).id + (*h).sr_index) as usize];
    (*stream).nsb_limit = ((freq_limit as c_long * 64 + samprate / 2) / samprate) as c_int;
    /*- caller limit -*/
    /*---- limit = 0.94*(32>>reduction_code);  ----*/
    limit = 32 >> reduction_code;
    if limit > 8 {
        limit -= 1;
    }
    if (*stream).nsb_limit > limit {
        (*stream).nsb_limit = limit;
    }
    if (*stream).nsb_limit > (*stream).u.L1_2.max_sb {
        (*stream).nsb_limit = (*stream).u.L1_2.max_sb;
    }

    (*stream).outvalues = 1152 >> reduction_code;
    if (*h).mode != 3 {
        /* adjust for 2 channel modes */
        i = 0;
        while i < 4 {
            (*stream).u.L1_2.nbat[i as usize] *= 2;
            i += 1;
        }
        (*stream).u.L1_2.max_sb *= 2;
        (*stream).nsb_limit *= 2;
    }

    /* set sbt function */
    k = 1 + convert_code;
    if (*h).mode == 3 {
        k = 0;
    }
    (*stream).u.L1_2.sbt = sbt_table[bit_code as usize][reduction_code as usize][k as usize];
    (*stream).outvalues *= out_chans[k as usize];
    if bit_code != 0 {
        (*stream).outbytes = (*stream).outvalues;
    } else {
        (*stream).outbytes = core::mem::size_of::<c_short>() as c_int * (*stream).outvalues;
    }

    decinfo.channels = out_chans[k as usize];
    decinfo.outvalues = (*stream).outvalues;
    decinfo.samprate = samprate >> reduction_code;
    if bit_code != 0 {
        decinfo.bits = 8;
    } else {
        decinfo.bits = core::mem::size_of::<c_short>() as c_int * 8;
    }

    decinfo.framebytes = (*stream).framebytes;
    decinfo.r#type = 0;

    /* clear sample buffer, unused sub bands must be 0 */
    i = 0;
    while i < 2304 * 2 {
        // the *2 here was inserted by me just in case, since the array is now *2, because of stereo files unpacking at 4608 bytes per frame (which may or may not be relevant, but in any case I don't think we use the L1 versions of MP3 now anyway
        sample[i as usize] = 0.0f32;
        i += 1;
    }

    /* init sub-band transform */
    sbt_init();

    return 1;
}

/*---------------------------------------------------------*/
pub unsafe extern "C" fn audio_decode_info(info: *mut DEC_INFO) {
    core::ptr::copy_nonoverlapping(addr_of!(decinfo), info, 1); /* info return, call after init */
}

/*---------------------------------------------------------*/
pub unsafe extern "C" fn decode_table_init() {
    /* dummy for asm version compatability */
}
