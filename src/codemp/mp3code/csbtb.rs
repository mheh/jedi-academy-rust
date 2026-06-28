// Filename: csbtb.c
//
// include to csbt.c
//
// MPEG audio decoder, dct and window - byte (8 pcm bit output)
// portable C

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uchar};

use super::mp3struct_h::pMP3Stream;

unsafe extern "C" {
    fn fdct32(sample: *mut f32, vbuf: *mut f32);
    fn fdct32_dual(sample: *mut f32, vbuf: *mut f32);
    fn fdct32_dual_mono(sample: *mut f32, vbuf: *mut f32);
    fn fdct16(sample: *mut f32, vbuf: *mut f32);
    fn fdct16_dual(sample: *mut f32, vbuf: *mut f32);
    fn fdct16_dual_mono(sample: *mut f32, vbuf: *mut f32);
    fn fdct8(sample: *mut f32, vbuf: *mut f32);
    fn fdct8_dual(sample: *mut f32, vbuf: *mut f32);
    fn fdct8_dual_mono(sample: *mut f32, vbuf: *mut f32);

    fn windowB(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB16(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB16_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB8(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
    fn windowB8_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut c_uchar);
}

#[no_mangle]
pub unsafe extern "C" fn sbtB_mono(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct32(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB_dual(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct32_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            fdct32_dual(sample.offset(1), (*pMP3Stream).vbuf2.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB_dual((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            windowB_dual((*pMP3Stream).vbuf2.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(64);
        }
        i += 1;
    }
}

/* convert dual to mono */
#[no_mangle]
pub unsafe extern "C" fn sbtB_dual_mono(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct32_dual_mono(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}

/* convert dual to left */
#[no_mangle]
pub unsafe extern "C" fn sbtB_dual_left(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct32_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}

/* convert dual to right */
#[no_mangle]
pub unsafe extern "C" fn sbtB_dual_right(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    unsafe {
        sample = sample.offset(1); /* point to right chan */
    }
    i = 0;
    while i < n {
        unsafe {
            fdct32_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}

/*---------------- 16 pt sbt's  -------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbtB16_mono(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB16_dual(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            fdct16_dual(sample.offset(1), (*pMP3Stream).vbuf2.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB16_dual((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            windowB16_dual((*pMP3Stream).vbuf2.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB16_dual_mono(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16_dual_mono(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB16_dual_left(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB16_dual_right(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    unsafe {
        sample = sample.offset(1);
    }
    i = 0;
    while i < n {
        unsafe {
            fdct16_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}

/*---------------- 8 pt sbt's  -------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbtB8_mono(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB8_dual(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            fdct8_dual(sample.offset(1), (*pMP3Stream).vbuf2.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB8_dual((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            windowB8_dual((*pMP3Stream).vbuf2.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB8_dual_mono(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8_dual_mono(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB8_dual_left(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn sbtB8_dual_right(mut sample: *mut f32, mut pcm: *mut c_uchar, n: c_int) {
    let mut i: c_int;

    unsafe {
        sample = sample.offset(1);
    }
    i = 0;
    while i < n {
        unsafe {
            fdct8_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            windowB8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}
