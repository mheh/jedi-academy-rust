// Port of codemp/mp3code/csbtl3.c.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_short, c_uchar};
use core::ptr::addr_of_mut;

use super::mp3struct_h::pMP3Stream;

unsafe extern "C" {
    fn fdct32(sample: *mut f32, p: *mut f32);
    fn fdct16(sample: *mut f32, p: *mut f32);
    fn fdct8(sample: *mut f32, p: *mut f32);
    fn window(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_short);
    fn window_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_short);
    fn window16(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_short);
    fn window16_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_short);
    fn window8(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_short);
    fn window8_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_short);
    fn windowB(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB16(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB16_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB8(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB8_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
}

/*============================================================*/
/*============ Layer III =====================================*/
/*============================================================*/
pub unsafe extern "C" fn sbt_mono_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    mut ch: c_int,
) {
    let mut i: c_int;

    ch = 0;
    i = 0;
    while i < 18 {
        fdct32(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(32);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
        pcm = pcm.offset(32);
        i += 1;
    }
}

/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt_dual_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    ch: c_int,
) {
    let mut i: c_int;

    if ch == 0 {
        i = 0;
        while i < 18 {
            fdct32(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32)
                    .offset((*pMP3Stream).vb_ptr as isize),
            );
            window_dual(
                addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
                (*pMP3Stream).vb_ptr,
                pcm,
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(64);
            i += 1;
        }
    } else {
        i = 0;
        while i < 18 {
            fdct32(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32)
                    .offset((*pMP3Stream).vb2_ptr as isize),
            );
            window_dual(
                addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
                (*pMP3Stream).vb2_ptr,
                pcm.offset(1),
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 32) & 511;
            pcm = pcm.offset(64);
            i += 1;
        }
    }
}

/*------------------------------------------------------------*/
/*------------------------------------------------------------*/
/*---------------- 16 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt16_mono_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    mut ch: c_int,
) {
    let mut i: c_int;

    ch = 0;
    i = 0;
    while i < 18 {
        fdct16(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window16(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(32);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
        pcm = pcm.offset(16);
        i += 1;
    }
}

/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt16_dual_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    ch: c_int,
) {
    let mut i: c_int;

    if ch == 0 {
        i = 0;
        while i < 18 {
            fdct16(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32)
                    .offset((*pMP3Stream).vb_ptr as isize),
            );
            window16_dual(
                addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
                (*pMP3Stream).vb_ptr,
                pcm,
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(32);
            i += 1;
        }
    } else {
        i = 0;
        while i < 18 {
            fdct16(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32)
                    .offset((*pMP3Stream).vb2_ptr as isize),
            );
            window16_dual(
                addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
                (*pMP3Stream).vb2_ptr,
                pcm.offset(1),
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 16) & 255;
            pcm = pcm.offset(32);
            i += 1;
        }
    }
}

/*------------------------------------------------------------*/
/*---------------- 8 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt8_mono_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    mut ch: c_int,
) {
    let mut i: c_int;

    ch = 0;
    i = 0;
    while i < 18 {
        fdct8(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window8(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(32);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
        pcm = pcm.offset(8);
        i += 1;
    }
}

/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt8_dual_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    ch: c_int,
) {
    let mut i: c_int;

    if ch == 0 {
        i = 0;
        while i < 18 {
            fdct8(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32)
                    .offset((*pMP3Stream).vb_ptr as isize),
            );
            window8_dual(
                addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
                (*pMP3Stream).vb_ptr,
                pcm,
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(16);
            i += 1;
        }
    } else {
        i = 0;
        while i < 18 {
            fdct8(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32)
                    .offset((*pMP3Stream).vb2_ptr as isize),
            );
            window8_dual(
                addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
                (*pMP3Stream).vb2_ptr,
                pcm.offset(1),
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 8) & 127;
            pcm = pcm.offset(16);
            i += 1;
        }
    }
}

/*------------------------------------------------------------*/
/*------- 8 bit output ---------------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB_mono_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_uchar,
    mut ch: c_int,
) {
    let mut i: c_int;

    ch = 0;
    i = 0;
    while i < 18 {
        fdct32(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        windowB(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(32);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
        pcm = pcm.offset(32);
        i += 1;
    }
}

/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB_dual_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_uchar,
    ch: c_int,
) {
    let mut i: c_int;

    if ch == 0 {
        i = 0;
        while i < 18 {
            fdct32(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32)
                    .offset((*pMP3Stream).vb_ptr as isize),
            );
            windowB_dual(
                addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
                (*pMP3Stream).vb_ptr,
                pcm,
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(64);
            i += 1;
        }
    } else {
        i = 0;
        while i < 18 {
            fdct32(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32)
                    .offset((*pMP3Stream).vb2_ptr as isize),
            );
            windowB_dual(
                addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
                (*pMP3Stream).vb2_ptr,
                pcm.offset(1),
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 32) & 511;
            pcm = pcm.offset(64);
            i += 1;
        }
    }
}

/*------------------------------------------------------------*/
/*------------------------------------------------------------*/
/*---------------- 16 pt sbtB's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB16_mono_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_uchar,
    mut ch: c_int,
) {
    let mut i: c_int;

    ch = 0;
    i = 0;
    while i < 18 {
        fdct16(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        windowB16(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(32);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
        pcm = pcm.offset(16);
        i += 1;
    }
}

/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB16_dual_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_uchar,
    ch: c_int,
) {
    let mut i: c_int;

    if ch == 0 {
        i = 0;
        while i < 18 {
            fdct16(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32)
                    .offset((*pMP3Stream).vb_ptr as isize),
            );
            windowB16_dual(
                addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
                (*pMP3Stream).vb_ptr,
                pcm,
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(32);
            i += 1;
        }
    } else {
        i = 0;
        while i < 18 {
            fdct16(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32)
                    .offset((*pMP3Stream).vb2_ptr as isize),
            );
            windowB16_dual(
                addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
                (*pMP3Stream).vb2_ptr,
                pcm.offset(1),
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 16) & 255;
            pcm = pcm.offset(32);
            i += 1;
        }
    }
}

/*------------------------------------------------------------*/
/*---------------- 8 pt sbtB's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB8_mono_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_uchar,
    mut ch: c_int,
) {
    let mut i: c_int;

    ch = 0;
    i = 0;
    while i < 18 {
        fdct8(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        windowB8(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(32);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
        pcm = pcm.offset(8);
        i += 1;
    }
}

/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB8_dual_L3(
    mut sample: *mut f32,
    mut pcm: *mut c_uchar,
    ch: c_int,
) {
    let mut i: c_int;

    if ch == 0 {
        i = 0;
        while i < 18 {
            fdct8(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32)
                    .offset((*pMP3Stream).vb_ptr as isize),
            );
            windowB8_dual(
                addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
                (*pMP3Stream).vb_ptr,
                pcm,
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(16);
            i += 1;
        }
    } else {
        i = 0;
        while i < 18 {
            fdct8(
                sample,
                (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32)
                    .offset((*pMP3Stream).vb2_ptr as isize),
            );
            windowB8_dual(
                addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
                (*pMP3Stream).vb2_ptr,
                pcm.offset(1),
            );
            sample = sample.offset(32);
            (*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 8) & 127;
            pcm = pcm.offset(16);
            i += 1;
        }
    }
}
