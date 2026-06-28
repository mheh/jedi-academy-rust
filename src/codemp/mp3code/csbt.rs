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

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_short};
use core::ptr::addr_of_mut;

use super::cdct::{
    dct_coef_addr, fdct16, fdct16_dual, fdct16_dual_mono, fdct32, fdct32_dual,
    fdct32_dual_mono, fdct8, fdct8_dual, fdct8_dual_mono,
};
use super::cwin::{
    window, window_dual, window16, window16_dual, window8, window8_dual,
};
use super::mp3struct_h::pMP3Stream;

/*-------------------------------------------------------------------------*/
/* circular window buffers */

/*======================================================================*/
unsafe fn gencoef() {
    static mut iOnceOnly: c_int = 0;
    let mut p: c_int;
    let mut n: c_int;
    let mut i: c_int;
    let mut k: c_int;
    let mut t: f64;
    let mut pi: f64;
    let mut coef32: *mut f32;

    if iOnceOnly == 0 {
        iOnceOnly += 1;
        coef32 = dct_coef_addr();

        pi = 4.0f64 * 1.0f64.atan();
        n = 16;
        k = 0;
        i = 0;
        while i < 5 {
            p = 0;
            while p < n {
                t = (pi / (4 * n) as f64) * (2 * p + 1) as f64;
                *coef32.offset(k as isize) = (0.50f64 / t.cos()) as f32;
                p += 1;
                k += 1;
            }
            i += 1;
            n = n / 2;
        }
    } else {
        iOnceOnly += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt_init() {
    let mut i: c_int;
    static mut first_pass: c_int = 1;

    if first_pass != 0 {
        gencoef();
        first_pass = 0;
    }

    /* clear window pMP3Stream->vbuf */
    i = 0;
    while i < 512 {
        *(addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset(i as isize) = 0.0f32;
        *(addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32).offset(i as isize) = 0.0f32;
        i += 1;
    }
    (*pMP3Stream).vb_ptr = 0;
    (*pMP3Stream).vb2_ptr = (*pMP3Stream).vb_ptr;
}

/*============================================================*/
/*============================================================*/
/*============================================================*/
#[no_mangle]
pub unsafe extern "C" fn sbt_mono(mut sample: *mut f32, mut pcm: *mut c_short, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct32(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
        pcm = pcm.offset(32);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt_dual(mut sample: *mut f32, mut pcm: *mut c_short, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct32_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        fdct32_dual(
            sample.offset(1),
            (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window_dual(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        window_dual(
            addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm.offset(1),
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
        pcm = pcm.offset(64);
        i += 1;
    }
}

/*------------------------------------------------------------*/
/* convert dual to mono */
#[no_mangle]
pub unsafe extern "C" fn sbt_dual_mono(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct32_dual_mono(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
        pcm = pcm.offset(32);
        i += 1;
    }
}

/*------------------------------------------------------------*/
/* convert dual to left */
#[no_mangle]
pub unsafe extern "C" fn sbt_dual_left(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct32_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
        pcm = pcm.offset(32);
        i += 1;
    }
}

/*------------------------------------------------------------*/
/* convert dual to right */
#[no_mangle]
pub unsafe extern "C" fn sbt_dual_right(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    sample = sample.offset(1); /* point to right chan */
    i = 0;
    while i < n {
        fdct32_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
        pcm = pcm.offset(32);
        i += 1;
    }
}

/*------------------------------------------------------------*/
/*---------------- 16 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt16_mono(mut sample: *mut f32, mut pcm: *mut c_short, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct16(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window16(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
        pcm = pcm.offset(16);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt16_dual(mut sample: *mut f32, mut pcm: *mut c_short, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct16_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        fdct16_dual(
            sample.offset(1),
            (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window16_dual(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        window16_dual(
            addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm.offset(1),
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
        pcm = pcm.offset(32);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt16_dual_mono(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct16_dual_mono(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window16(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
        pcm = pcm.offset(16);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt16_dual_left(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct16_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window16(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
        pcm = pcm.offset(16);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt16_dual_right(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    sample = sample.offset(1);
    i = 0;
    while i < n {
        fdct16_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window16(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
        pcm = pcm.offset(16);
        i += 1;
    }
}

/*------------------------------------------------------------*/
/*---------------- 8 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt8_mono(mut sample: *mut f32, mut pcm: *mut c_short, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct8(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window8(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
        pcm = pcm.offset(8);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt8_dual(mut sample: *mut f32, mut pcm: *mut c_short, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct8_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        fdct8_dual(
            sample.offset(1),
            (addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window8_dual(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        window8_dual(
            addr_of_mut!((*pMP3Stream).vbuf2) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm.offset(1),
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
        pcm = pcm.offset(16);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt8_dual_mono(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct8_dual_mono(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window8(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
        pcm = pcm.offset(8);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt8_dual_left(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    i = 0;
    while i < n {
        fdct8_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window8(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
        pcm = pcm.offset(8);
        i += 1;
    }
}

/*------------------------------------------------------------*/
#[no_mangle]
pub unsafe extern "C" fn sbt8_dual_right(
    mut sample: *mut f32,
    mut pcm: *mut c_short,
    n: c_int,
) {
    let mut i: c_int;

    sample = sample.offset(1);
    i = 0;
    while i < n {
        fdct8_dual(
            sample,
            (addr_of_mut!((*pMP3Stream).vbuf) as *mut f32).offset((*pMP3Stream).vb_ptr as isize),
        );
        window8(
            addr_of_mut!((*pMP3Stream).vbuf) as *mut f32,
            (*pMP3Stream).vb_ptr,
            pcm,
        );
        sample = sample.offset(64);
        (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
        pcm = pcm.offset(8);
        i += 1;
    }
}
