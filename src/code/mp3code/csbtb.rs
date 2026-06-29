// Porting note: This file was originally wrapped in #ifdef COMPILE_ME / #endif in C
// #pragma warning(disable:4206)	// nonstandard extension used : translation unit is empty

//____________________________________________________________________________
//
//	FreeAmp - The Free MP3 Player
//
//    MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
//    Corp.  http://www.xingtech.com
//
//	Portions Copyright (C) 1998 EMusic.com
//
//	This program is free software; you can redistribute it and/or modify
//	it under the terms of the GNU General Public License as published by
//	the Free Software Foundation; either version 2 of the License, or
//	(at your option) any later version.
//
//	This program is distributed in the hope that it will be useful,
//	but WITHOUT ANY WARRANTY; without even the implied warranty of
//	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//	GNU General Public License for more details.
//
//	You should have received a copy of the GNU General Public License
//	along with this program; if not, write to the Free Software
//	Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
//
//	$Id: csbtb.c,v 1.2 1999/10/19 07:13:08 elrod Exp $
//____________________________________________________________________________

//****  csbtb.c  ***************************************************
//include to csbt.c
//
//MPEG audio decoder, dct and window - byte (8 pcm bit output)
//portable C
//
//******************************************************************

#![allow(non_snake_case)]

use core::ffi::c_int;
use crate::code::mp3code::mp3struct_h::{MP3STREAM, LP_MP3STREAM};

//============================================================
//============================================================
// Forward declarations
extern "C" {
    fn windowB(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
    fn windowB_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
    fn windowB16(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
    fn windowB16_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
    fn windowB8(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
    fn windowB8_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);

    fn fdct32(sample: *mut f32, vbuf: *mut f32);
    fn fdct32_dual(sample: *mut f32, vbuf: *mut f32);
    fn fdct32_dual_mono(sample: *mut f32, vbuf: *mut f32);
    fn fdct16(sample: *mut f32, vbuf: *mut f32);
    fn fdct16_dual(sample: *mut f32, vbuf: *mut f32);
    fn fdct16_dual_mono(sample: *mut f32, vbuf: *mut f32);
    fn fdct8(sample: *mut f32, vbuf: *mut f32);
    fn fdct8_dual(sample: *mut f32, vbuf: *mut f32);
    fn fdct8_dual_mono(sample: *mut f32, vbuf: *mut f32);

    static mut pMP3Stream: LP_MP3STREAM;
}

//============================================================
pub fn sbtB_mono(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            let mp3_stream = *pMP3Stream;
            fdct32(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB_dual(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct32_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            fdct32_dual(sample.offset(1), (&mut (*pMP3Stream).vbuf2[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB_dual((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            windowB_dual((&mut (*pMP3Stream).vbuf2[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(64);
        }
        i += 1;
    }
}
//------------------------------------------------------------
// convert dual to mono
pub fn sbtB_dual_mono(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct32_dual_mono(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}
//------------------------------------------------------------
// convert dual to left
pub fn sbtB_dual_left(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct32_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}
//------------------------------------------------------------
// convert dual to right
pub fn sbtB_dual_right(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    sample = unsafe { sample.offset(1) };			// point to right chan
    i = 0;
    while i < n {
        unsafe {
            fdct32_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}
//------------------------------------------------------------
//---------------- 16 pt sbt's  -------------------------------
//------------------------------------------------------------
pub fn sbtB16_mono(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB16((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB16_dual(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            fdct16_dual(sample.offset(1), (&mut (*pMP3Stream).vbuf2[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB16_dual((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            windowB16_dual((&mut (*pMP3Stream).vbuf2[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(32);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB16_dual_mono(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16_dual_mono(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB16((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB16_dual_left(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct16_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB16((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB16_dual_right(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    sample = unsafe { sample.offset(1) };
    i = 0;
    while i < n {
        unsafe {
            fdct16_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB16((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}
//------------------------------------------------------------
//---------------- 8 pt sbt's  -------------------------------
//------------------------------------------------------------
pub fn sbtB8_mono(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB8((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB8_dual(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            fdct8_dual(sample.offset(1), (&mut (*pMP3Stream).vbuf2[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB8_dual((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            windowB8_dual((&mut (*pMP3Stream).vbuf2[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm.offset(1));
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(16);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB8_dual_mono(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8_dual_mono(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB8((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB8_dual_left(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    i = 0;
    while i < n {
        unsafe {
            fdct8_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB8((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}
//------------------------------------------------------------
pub fn sbtB8_dual_right(mut sample: *mut f32, mut pcm: *mut u8, n: c_int) {
    let mut i: c_int;

    sample = unsafe { sample.offset(1) };
    i = 0;
    while i < n {
        unsafe {
            fdct8_dual(sample, (&mut (*pMP3Stream).vbuf[0] as *mut f32).offset((*pMP3Stream).vb_ptr as isize));
            windowB8((&mut (*pMP3Stream).vbuf[0] as *mut f32), (*pMP3Stream).vb_ptr, pcm);
            sample = sample.offset(64);
            (*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
            pcm = pcm.offset(8);
        }
        i += 1;
    }
}
//------------------------------------------------------------
