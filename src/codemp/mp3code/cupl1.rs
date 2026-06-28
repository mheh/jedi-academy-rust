#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_long, c_short, c_uchar, c_ulong};
use core::mem::transmute;
use core::ptr::{addr_of, addr_of_mut};

use super::mhead_h::{DEC_INFO, MPEG_HEAD};
use super::mp3struct_h::{pMP3Stream, LP_MP3STREAM, SBT_FUNCTION};
use super::small_header_h::IN_OUT;

/*____________________________________________________________________________

	FreeAmp - The Free MP3 Player

        MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
        Corp.  http://www.xingtech.com

	Portions Copyright (C) 1998-1999 EMusic.com

	This program is free software; you can redistribute it and/or modify
	it under the terms of the GNU General Public License as published by
	the Free Software Foundation; either version 2 of the License, or
	(at your option) any later version.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with this program; if not, write to the Free Software
	Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

	$Id: cupL1.c,v 1.3 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cupL1.c  ***************************************************

MPEG audio decoder Layer I mpeg1 and mpeg2

include to clup.c


******************************************************************/
/*======================================================================*/
static bat_bit_masterL1: [c_int; 16] = [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
////@@@@static float *pMP3Stream->cs_factorL1 = &pMP3Stream->cs_factor[0];	// !!!!!!!!!!!!!!!!
static mut look_c_valueL1: [f32; 16] = [0.0; 16]; // effectively constant
////@@@@static int nbatL1 = 32;

static look_joint: [c_int; 16] = [
    64, 64, 64, 64, /* stereo */
    2 * 4, 2 * 8, 2 * 12, 2 * 16, /* joint */
    64, 64, 64, 64, /* dual */
    32, 32, 32, 32, /* mono */
];

static sr_table: [c_long; 8] = [22050, 24000, 16000, 1, 44100, 48000, 32000, 1];
static out_chans: [c_int; 5] = [1, 2, 1, 1, 1];

static mut ballo: [c_int; 64] = [0; 64]; /* set by unpack_ba */
static mut samp_dispatch: [c_int; 66] = [0; 66]; /* set by unpack_ba */
static mut c_value: [f32; 64] = [0.0; 64]; /* set by unpack_ba */

/* get bits from bitstream in endian independent way */
////@@@@ FINDME - this stuff doesn't appear to be used by any of our samples (phew)
static mut bs_ptr: *mut c_uchar = core::ptr::null_mut();
static mut bitbuf: c_ulong = 0;
static mut bits: c_int = 0;
static mut bitval: c_long = 0;

unsafe extern "C" {
    pub static mut decinfo: DEC_INFO;
    pub static mut sf_table: [f32; 64];
    pub static mut sample: [f32; 2304 * 2];

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

    pub fn sbt_init();
}

const unsafe fn sbt8_cast(
    f: unsafe extern "C" fn(sample: *mut f32, pcm: *mut c_uchar, n: c_int),
) -> unsafe extern "C" fn(sample: *mut f32, pcm: *mut c_short, n: c_int) {
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

/*------------- initialize bit getter -------------*/
unsafe fn load_init(buf: *mut c_uchar) {
    bs_ptr = buf;
    bits = 0;
    bitbuf = 0;
}

/*------------- get n bits from bitstream -------------*/
unsafe fn load(n: c_int) -> c_long {
    let x: c_ulong;

    if bits < n {
        /* refill bit buf if necessary */
        while bits <= 24 {
            bitbuf = (bitbuf << 8) | *bs_ptr as c_ulong;
            bs_ptr = bs_ptr.add(1);
            bits += 8;
        }
    }
    bits -= n;
    x = bitbuf >> bits;
    bitbuf -= x << bits;
    return x as c_long;
}

/*------------- skip over n bits in bitstream -------------*/
unsafe fn skip(mut n: c_int) {
    let k: c_int;

    if bits < n {
        n -= bits;
        k = n >> 3;
        /*--- bytes = n/8 --*/
        bs_ptr = bs_ptr.add(k as usize);
        n -= k << 3;
        bitbuf = *bs_ptr as c_ulong;
        bs_ptr = bs_ptr.add(1);
        bits = 8;
    }
    bits -= n;
    bitbuf -= (bitbuf >> bits) << bits;
}

#[inline]
unsafe fn mac_load_check(n: c_int) {
    if bits < n {
        while bits <= 24 {
            bitbuf = (bitbuf << 8) | *bs_ptr as c_ulong;
            bs_ptr = bs_ptr.add(1);
            bits += 8;
        }
    }
}

#[inline]
unsafe fn mac_load(n: c_int) -> c_long {
    bits -= n;
    bitval = (bitbuf >> bits) as c_long;
    bitbuf -= (bitval as c_ulong) << bits;
    return bitval;
}

#[inline]
unsafe fn stream_l1_2() -> *mut super::mp3struct_h::MP3STREAM_L1_2 {
    addr_of_mut!((*pMP3Stream).u.L1_2)
}

/*======================================================================*/
unsafe fn unpack_baL1() {
    let mut j: c_int;
    let mut nstereo: c_int;
    let stream = stream_l1_2();
    let ballo_p = addr_of_mut!(ballo) as *mut c_int;
    let samp_dispatch_p = addr_of_mut!(samp_dispatch) as *mut c_int;
    let c_value_p = addr_of_mut!(c_value) as *mut f32;

    (*stream).bit_skip = 0;
    nstereo = (*stream).stereo_sb;

    j = 0;
    while j < (*stream).nbatL1 {
        mac_load_check(4);
        *samp_dispatch_p.add(j as usize) = mac_load(4) as c_int;
        *ballo_p.add(j as usize) = *samp_dispatch_p.add(j as usize);
        if j >= (*pMP3Stream).nsb_limit {
            (*stream).bit_skip += bat_bit_masterL1[*samp_dispatch_p.add(j as usize) as usize];
        }
        *c_value_p.add(j as usize) =
            *addr_of!(look_c_valueL1).cast::<f32>().add(*samp_dispatch_p.add(j as usize) as usize);
        nstereo -= 1;
        if nstereo < 0 {
            *ballo_p.add((j + 1) as usize) = *ballo_p.add(j as usize);
            *samp_dispatch_p.add(j as usize) += 15; /* flag as joint */
            *samp_dispatch_p.add((j + 1) as usize) = *samp_dispatch_p.add(j as usize); /* flag for sf */
            *c_value_p.add((j + 1) as usize) = *c_value_p.add(j as usize);
            j += 1;
        }
        j += 1;
    }
    /*-- terminate with bit skip and end --*/
    *samp_dispatch_p.add((*pMP3Stream).nsb_limit as usize) = 31;
    *samp_dispatch_p.add(j as usize) = 30;
}

/*-------------------------------------------------------------------------*/
unsafe fn unpack_sfL1() {
    /* unpack scale factor */
    /* combine dequant and scale factors */
    let mut i: c_int;
    let stream = stream_l1_2();
    let ballo_p = addr_of!(ballo) as *const c_int;
    let c_value_p = addr_of!(c_value) as *const f32;
    let sf_table_p = addr_of!(sf_table) as *const f32;

    i = 0;
    while i < (*stream).nbatL1 {
        if *ballo_p.add(i as usize) != 0 {
            mac_load_check(6);
            *(*stream).cs_factorL1.add(i as usize) =
                *c_value_p.add(i as usize) * *sf_table_p.add(mac_load(6) as usize);
        }
        i += 1;
    }
    /*-- done --*/
}

/*-------------------------------------------------------------------------*/
#[inline]
unsafe fn UNPACKL1_N(s: *mut f32, k: c_int, n: c_int) {
    *s.add(k as usize) =
        *(*stream_l1_2()).cs_factorL1.add(k as usize) * (load(n) - ((1 << (n - 1)) - 1)) as f32;
}

#[inline]
unsafe fn UNPACKL1J_N(s: *mut f32, k: *mut c_int, n: c_int) {
    let tmp: c_long;

    tmp = load(n) - ((1 << (n - 1)) - 1);
    *s.add(*k as usize) = *(*stream_l1_2()).cs_factorL1.add(*k as usize) * tmp as f32;
    *s.add((*k + 1) as usize) = *(*stream_l1_2()).cs_factorL1.add((*k + 1) as usize) * tmp as f32;
    *k += 1;
}

/*-------------------------------------------------------------------------*/
unsafe fn unpack_sampL1() {
    /* unpack samples */
    let mut j: c_int;
    let mut k: c_int;
    let mut s: *mut f32;
    let samp_dispatch_p = addr_of!(samp_dispatch) as *const c_int;

    s = addr_of_mut!(sample) as *mut f32;
    j = 0;
    while j < 12 {
        k = -1;
        loop {
            k += 1;
            match *samp_dispatch_p.add(k as usize) {
                0 => {
                    *s.add(k as usize) = 0.0f32;
                }
                1 => {
                    UNPACKL1_N(s, k, 2); /*  3 levels */
                }
                2 => {
                    UNPACKL1_N(s, k, 3); /*  7 levels */
                }
                3 => {
                    UNPACKL1_N(s, k, 4); /* 15 levels */
                }
                4 => {
                    UNPACKL1_N(s, k, 5); /* 31 levels */
                }
                5 => {
                    UNPACKL1_N(s, k, 6); /* 63 levels */
                }
                6 => {
                    UNPACKL1_N(s, k, 7); /* 127 levels */
                }
                7 => {
                    UNPACKL1_N(s, k, 8); /* 255 levels */
                }
                8 => {
                    UNPACKL1_N(s, k, 9); /* 511 levels */
                }
                9 => {
                    UNPACKL1_N(s, k, 10); /* 1023 levels */
                }
                10 => {
                    UNPACKL1_N(s, k, 11); /* 2047 levels */
                }
                11 => {
                    UNPACKL1_N(s, k, 12); /* 4095 levels */
                }
                12 => {
                    UNPACKL1_N(s, k, 13); /* 8191 levels */
                }
                13 => {
                    UNPACKL1_N(s, k, 14); /* 16383 levels */
                }
                14 => {
                    UNPACKL1_N(s, k, 15); /* 32767 levels */
                }
                /* -- joint ---- */
                15 => {
                    *s.add(k as usize) = 0.0f32;
                    *s.add((k + 1) as usize) = *s.add(k as usize);
                    k += 1; /* skip right chan dispatch */
                }
                /* -- joint ---- */
                16 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 2); /*  3 levels */
                }
                17 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 3); /*  7 levels */
                }
                18 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 4); /* 15 levels */
                }
                19 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 5); /* 31 levels */
                }
                20 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 6); /* 63 levels */
                }
                21 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 7); /* 127 levels */
                }
                22 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 8); /* 255 levels */
                }
                23 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 9); /* 511 levels */
                }
                24 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 10); /* 1023 levels */
                }
                25 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 11); /* 2047 levels */
                }
                26 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 12); /* 4095 levels */
                }
                27 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 13); /* 8191 levels */
                }
                28 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 14); /* 16383 levels */
                }
                29 => {
                    UNPACKL1J_N(s, addr_of_mut!(k), 15); /* 32767 levels */
                }

                /* -- end of dispatch -- */
                31 => {
                    skip((*stream_l1_2()).bit_skip);
                    s = s.add(64);
                    break;
                }
                30 => {
                    s = s.add(64);
                    break;
                }
                _ => {
                    s = s.add(64);
                    break;
                }
            }
        } /* end switch */
        j += 1;
    } /* end j loop */

    /*-- done --*/
}

/*-------------------------------------------------------------------*/
pub unsafe extern "C" fn L1audio_decode(bs: *mut c_uchar, pcm: *mut c_short) -> IN_OUT {
    let sync: c_int;
    let prot: c_int;
    let mut in_out: IN_OUT = IN_OUT {
        in_bytes: 0,
        out_bytes: 0,
    };
    let stream = stream_l1_2();

    load_init(bs); /* initialize bit getter */
    /* test sync */
    in_out.in_bytes = 0; /* assume fail */
    in_out.out_bytes = 0;
    sync = load(12) as c_int;
    if sync != 0xFFF {
        return in_out; /* sync fail */
    }

    load(3); /* skip id and option (checked by init) */
    prot = load(1) as c_int; /* load prot bit */
    load(6); /* skip to pad */
    (*pMP3Stream).pad = (load(1) as c_int) << 2;
    load(1); /* skip to mode */
    (*stream).stereo_sb = look_joint[load(4) as usize];
    if prot != 0 {
        load(4); /* skip to data */
    } else {
        load(20); /* skip crc */
    }

    unpack_baL1(); /* unpack bit allocation */
    unpack_sfL1(); /* unpack scale factor */
    unpack_sampL1(); /* unpack samples */

    (*stream)
        .sbt
        .unwrap_unchecked()(addr_of_mut!(sample) as *mut f32, pcm, 12);
    /*-----------*/
    in_out.in_bytes = (*pMP3Stream).framebytes + (*pMP3Stream).pad;
    in_out.out_bytes = (*pMP3Stream).outbytes;

    return in_out;
}

/*-------------------------------------------------------------------------*/
pub unsafe extern "C" fn L1audio_decode_init(
    h: *mut MPEG_HEAD,
    framebytes_arg: c_int,
    mut reduction_code: c_int,
    mut transform_code: c_int,
    mut convert_code: c_int,
    mut freq_limit: c_int,
) -> c_int {
    let mut i: c_int;
    let k: c_int;
    static mut first_pass: c_int = 1;
    let samprate: c_long;
    let mut limit: c_int;
    let mut step: c_long;
    let mut bit_code: c_int;
    let stream: LP_MP3STREAM;
    let stream_l1: *mut super::mp3struct_h::MP3STREAM_L1_2;

    /*--- sf init done by layer II init ---*/
    if first_pass != 0 {
        step = 4;
        i = 1;
        while i < 16 {
            *addr_of_mut!(look_c_valueL1).cast::<f32>().add(i as usize) =
                (2.0f64 / (step - 1) as f64) as f32;
            i += 1;
            step <<= 1;
        }
        first_pass = 0;
    }
    stream = pMP3Stream;
    stream_l1 = addr_of_mut!((*stream).u.L1_2);
    (*stream_l1).cs_factorL1 = addr_of_mut!((*stream_l1).cs_factor).cast::<f32>();

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

    (*stream).framebytes = framebytes_arg;
    /* check if code handles */
    if (*h).option != 3 {
        return 0; /* layer I only */
    }

    (*stream_l1).nbatL1 = 32;
    (*stream_l1).max_sb = (*stream_l1).nbatL1;
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
    if (*stream).nsb_limit > (*stream_l1).max_sb {
        (*stream).nsb_limit = (*stream_l1).max_sb;
    }

    (*stream).outvalues = 384 >> reduction_code;
    if (*h).mode != 3 {
        /* adjust for 2 channel modes */
        (*stream_l1).nbatL1 *= 2;
        (*stream_l1).max_sb *= 2;
        (*stream).nsb_limit *= 2;
    }

    /* set sbt function */
    k = if (*h).mode == 3 { 0 } else { 1 + convert_code };
    (*stream_l1).sbt = sbt_table[bit_code as usize][reduction_code as usize][k as usize];
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
    while i < 768 {
        *addr_of_mut!(sample).cast::<f32>().add(i as usize) = 0.0f32;
        i += 1;
    }

    /* init sub-band transform */
    sbt_init();

    return 1;
}

/*---------------------------------------------------------*/
