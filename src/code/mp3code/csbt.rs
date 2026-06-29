/*____________________________________________________________________________

	FreeAmp - The Free MP3 Player

        MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
        Corp.  http://www.xingtech.com

	Portions Copyright (C) 1998 EMusic.com

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

	$Id: csbt.c,v 1.2 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  csbt.c  ***************************************************

MPEG audio decoder, dct and window
portable C

1/7/96 mod for Layer III

******************************************************************/

use core::ffi::c_int;
use crate::code::mp3code::mp3struct_h::pMP3Stream;

extern "C" {
    fn fdct32(arg1: *mut f32, arg2: *mut f32);
    fn fdct32_dual(arg1: *mut f32, arg2: *mut f32);
    fn fdct32_dual_mono(arg1: *mut f32, arg2: *mut f32);
    fn fdct16(arg1: *mut f32, arg2: *mut f32);
    fn fdct16_dual(arg1: *mut f32, arg2: *mut f32);
    fn fdct16_dual_mono(arg1: *mut f32, arg2: *mut f32);
    fn fdct8(arg1: *mut f32, arg2: *mut f32);
    fn fdct8_dual(arg1: *mut f32, arg2: *mut f32);
    fn fdct8_dual_mono(arg1: *mut f32, arg2: *mut f32);

    fn window(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
    fn window_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
    fn window16(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
    fn window16_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
    fn window8(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
    fn window8_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);

    fn dct_coef_addr() -> *mut f32;
}

/*-------------------------------------------------------------------------*/
/* circular window buffers */
////static signed int vb_ptr;	// !!!!!!!!!!!!!
////static signed int vb2_ptr;	// !!!!!!!!!!!!!
////static float pMP3Stream->vbuf[512];		// !!!!!!!!!!!!!
////static float vbuf2[512];	// !!!!!!!!!!!!!

/*======================================================================*/
/* gen coef for N=32 (31 coefs) */
static mut iOnceOnly: c_int = 0;

fn gencoef() {
    let mut p: c_int;
    let mut n: c_int;
    let mut i: c_int;
    let mut k: c_int;
    let mut t: f64;
    let mut pi: f64;
    let mut coef32: *mut f32;

    unsafe {
        if iOnceOnly == 0 {
            iOnceOnly = 1;
            coef32 = dct_coef_addr();

            pi = 4.0 * (1.0_f64).atan();
            n = 16;
            k = 0;
            i = 0;
            while i < 5 {
                p = 0;
                while p < n {
                    t = (pi / (4.0 * n as f64)) * (2.0 * p as f64 + 1.0);
                    *coef32.offset(k as isize) = (0.50 / t.cos()) as f32;
                    p += 1;
                    k += 1;
                }
                n = n / 2;
                i += 1;
            }
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt_init() {
    let mut i: c_int;
    static mut first_pass: c_int = 1;

    unsafe {
        if first_pass != 0 {
            gencoef();
            first_pass = 0;
        }

        /* clear window pMP3Stream->vbuf */
        i = 0;
        while i < 512 {
            (*pMP3Stream).vbuf[i as usize] = 0.0_f32;
            (*pMP3Stream).vbuf2[i as usize] = 0.0_f32;
            i += 1;
        }
        (*pMP3Stream).vb2_ptr = 0;
        (*pMP3Stream).vb_ptr = 0;
    }
}
/*============================================================*/
/*============================================================*/
/*============================================================*/
pub fn sbt_mono(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct32(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt_dual(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct32_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            fdct32_dual(sample.offset(1), (*pMP3Stream).vbuf2.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window_dual((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            window_dual((*pMP3Stream).vbuf2.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(64);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
/* convert dual to mono */
pub fn sbt_dual_mono(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct32_dual_mono(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
/* convert dual to left */
pub fn sbt_dual_left(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct32_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
/* convert dual to right */
pub fn sbt_dual_right(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        sample = sample.offset(1);			/* point to right chan */
        i = 0;
        while i < n {
            fdct32_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
/*---------------- 16 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
pub fn sbt16_mono(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct16(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt16_dual(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct16_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            fdct16_dual(sample.offset(1), (*pMP3Stream).vbuf2.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window16_dual((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            window16_dual((*pMP3Stream).vbuf2.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(32);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt16_dual_mono(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct16_dual_mono(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt16_dual_left(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct16_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt16_dual_right(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        sample = sample.offset(1);
        i = 0;
        while i < n {
            fdct16_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window16((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
/*---------------- 8 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
pub fn sbt8_mono(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct8(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt8_dual(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct8_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            fdct8_dual(sample.offset(1), (*pMP3Stream).vbuf2.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window8_dual((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            window8_dual((*pMP3Stream).vbuf2.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(16);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt8_dual_mono(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct8_dual_mono(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt8_dual_left(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < n {
            fdct8_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
pub fn sbt8_dual_right(sample: *mut f32, pcm: *mut i16, n: c_int) {
    let mut i: c_int;

    unsafe {
        sample = sample.offset(1);
        i = 0;
        while i < n {
            fdct8_dual(sample, (*pMP3Stream).vbuf.as_mut_ptr().offset((*pMP3Stream).vb_ptr as isize));
            window8((*pMP3Stream).vbuf.as_mut_ptr(), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
            i += 1;
        }
    }
}
/*------------------------------------------------------------*/
/*------------------------------------------------------------*/
#[cfg(feature = "compile_me")]
mod csbtb {
    include!("csbtb.rs");		/* 8 bit output */
}

#[cfg(feature = "compile_me")]
mod csbtL3 {
    include!("csbtl3.rs");		/* Layer III */
}
/*------------------------------------------------------------*/
