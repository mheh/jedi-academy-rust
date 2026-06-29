#![allow(non_snake_case)]

use core::ffi::c_int;

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

	$Id: csbtL3.c,v 1.2 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  csbtL3.c  ***************************************************

layer III

  include to  csbt.c

******************************************************************/

// Stub declarations for external functions and globals.
// These would be defined elsewhere in the mp3code module.
extern "C" {
	fn fdct32(sample: *mut f32, vbuf: *mut f32);
	fn fdct16(sample: *mut f32, vbuf: *mut f32);
	fn fdct8(sample: *mut f32, vbuf: *mut f32);
	fn window(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
	fn window_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
	fn window16(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
	fn window16_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
	fn window8(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
	fn window8_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut i16);
	fn windowB(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
	fn windowB_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
	fn windowB16(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
	fn windowB16_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
	fn windowB8(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
	fn windowB8_dual(vbuf: *mut f32, vb_ptr: c_int, pcm: *mut u8);
}

// Stub for pMP3Stream global structure.
// In the actual codebase, this would be a pointer to the MP3 stream context.
extern "C" {
	static mut pMP3Stream: *mut MP3StreamContext;
}

#[repr(C)]
pub struct MP3StreamContext {
	pub vbuf: *mut f32,
	pub vb_ptr: c_int,
	pub vbuf2: *mut f32,
	pub vb2_ptr: c_int,
	// ... other fields as needed
}

/*============================================================*/
/*============ Layer III =====================================*/
/*============================================================*/
pub unsafe extern "C" fn sbt_mono_L3(sample: *mut f32, pcm: *mut i16, mut ch: c_int) {
	let mut i: c_int;

	ch = 0;
	i = 0;
	while i < 18 {
		fdct32(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
		window((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
		sample = sample.add(32);
		(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
		pcm = pcm.add(32);
		i += 1;
	}
}
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt_dual_L3(sample: *mut f32, pcm: *mut i16, ch: c_int) {
	let mut i: c_int;

	if ch == 0 {
		i = 0;
		while i < 18 {
			fdct32(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
			window_dual((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
			sample = sample.add(32);
			(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
			pcm = pcm.add(64);
			i += 1;
		}
	} else {
		i = 0;
		while i < 18 {
			fdct32(sample, (*pMP3Stream).vbuf2.add((*pMP3Stream).vb2_ptr as usize) as *mut f32);
			window_dual((*pMP3Stream).vbuf2, (*pMP3Stream).vb2_ptr, pcm.add(1));
			sample = sample.add(32);
			(*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 32) & 511;
			pcm = pcm.add(64);
			i += 1;
		}
	}
}
/*------------------------------------------------------------*/
/*------------------------------------------------------------*/
/*---------------- 16 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt16_mono_L3(sample: *mut f32, pcm: *mut i16, mut ch: c_int) {
	let mut i: c_int;

	ch = 0;
	i = 0;
	while i < 18 {
		fdct16(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
		window16((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
		sample = sample.add(32);
		(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
		pcm = pcm.add(16);
		i += 1;
	}
}
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt16_dual_L3(sample: *mut f32, pcm: *mut i16, ch: c_int) {
	let mut i: c_int;

	if ch == 0 {
		i = 0;
		while i < 18 {
			fdct16(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
			window16_dual((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
			sample = sample.add(32);
			(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
			pcm = pcm.add(32);
			i += 1;
		}
	} else {
		i = 0;
		while i < 18 {
			fdct16(sample, (*pMP3Stream).vbuf2.add((*pMP3Stream).vb2_ptr as usize) as *mut f32);
			window16_dual((*pMP3Stream).vbuf2, (*pMP3Stream).vb2_ptr, pcm.add(1));
			sample = sample.add(32);
			(*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 16) & 255;
			pcm = pcm.add(32);
			i += 1;
		}
	}
}
/*------------------------------------------------------------*/
/*---------------- 8 pt sbt's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt8_mono_L3(sample: *mut f32, pcm: *mut i16, mut ch: c_int) {
	let mut i: c_int;

	ch = 0;
	i = 0;
	while i < 18 {
		fdct8(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
		window8((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
		sample = sample.add(32);
		(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
		pcm = pcm.add(8);
		i += 1;
	}
}
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbt8_dual_L3(sample: *mut f32, pcm: *mut i16, ch: c_int) {
	let mut i: c_int;

	if ch == 0 {
		i = 0;
		while i < 18 {
			fdct8(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
			window8_dual((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
			sample = sample.add(32);
			(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
			pcm = pcm.add(16);
			i += 1;
		}
	} else {
		i = 0;
		while i < 18 {
			fdct8(sample, (*pMP3Stream).vbuf2.add((*pMP3Stream).vb2_ptr as usize) as *mut f32);
			window8_dual((*pMP3Stream).vbuf2, (*pMP3Stream).vb2_ptr, pcm.add(1));
			sample = sample.add(32);
			(*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 8) & 127;
			pcm = pcm.add(16);
			i += 1;
		}
	}
}
/*------------------------------------------------------------*/
/*------- 8 bit output ---------------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB_mono_L3(sample: *mut f32, pcm: *mut u8, mut ch: c_int) {
	let mut i: c_int;

	ch = 0;
	i = 0;
	while i < 18 {
		fdct32(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
		windowB((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
		sample = sample.add(32);
		(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
		pcm = pcm.add(32);
		i += 1;
	}
}
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB_dual_L3(sample: *mut f32, pcm: *mut u8, ch: c_int) {
	let mut i: c_int;

	if ch == 0 {
		i = 0;
		while i < 18 {
			fdct32(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
			windowB_dual((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
			sample = sample.add(32);
			(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 32) & 511;
			pcm = pcm.add(64);
			i += 1;
		}
	} else {
		i = 0;
		while i < 18 {
			fdct32(sample, (*pMP3Stream).vbuf2.add((*pMP3Stream).vb2_ptr as usize) as *mut f32);
			windowB_dual((*pMP3Stream).vbuf2, (*pMP3Stream).vb2_ptr, pcm.add(1));
			sample = sample.add(32);
			(*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 32) & 511;
			pcm = pcm.add(64);
			i += 1;
		}
	}
}
/*------------------------------------------------------------*/
/*------------------------------------------------------------*/
/*---------------- 16 pt sbtB's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB16_mono_L3(sample: *mut f32, pcm: *mut u8, mut ch: c_int) {
	let mut i: c_int;

	ch = 0;
	i = 0;
	while i < 18 {
		fdct16(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
		windowB16((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
		sample = sample.add(32);
		(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
		pcm = pcm.add(16);
		i += 1;
	}
}
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB16_dual_L3(sample: *mut f32, pcm: *mut u8, ch: c_int) {
	let mut i: c_int;

	if ch == 0 {
		i = 0;
		while i < 18 {
			fdct16(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
			windowB16_dual((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
			sample = sample.add(32);
			(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 16) & 255;
			pcm = pcm.add(32);
			i += 1;
		}
	} else {
		i = 0;
		while i < 18 {
			fdct16(sample, (*pMP3Stream).vbuf2.add((*pMP3Stream).vb2_ptr as usize) as *mut f32);
			windowB16_dual((*pMP3Stream).vbuf2, (*pMP3Stream).vb2_ptr, pcm.add(1));
			sample = sample.add(32);
			(*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 16) & 255;
			pcm = pcm.add(32);
			i += 1;
		}
	}
}
/*------------------------------------------------------------*/
/*---------------- 8 pt sbtB's  -------------------------------*/
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB8_mono_L3(sample: *mut f32, pcm: *mut u8, mut ch: c_int) {
	let mut i: c_int;

	ch = 0;
	i = 0;
	while i < 18 {
		fdct8(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
		windowB8((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
		sample = sample.add(32);
		(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
		pcm = pcm.add(8);
		i += 1;
	}
}
/*------------------------------------------------------------*/
pub unsafe extern "C" fn sbtB8_dual_L3(sample: *mut f32, pcm: *mut u8, ch: c_int) {
	let mut i: c_int;

	if ch == 0 {
		i = 0;
		while i < 18 {
			fdct8(sample, (*pMP3Stream).vbuf.add((*pMP3Stream).vb_ptr as usize) as *mut f32);
			windowB8_dual((*pMP3Stream).vbuf, (*pMP3Stream).vb_ptr, pcm);
			sample = sample.add(32);
			(*pMP3Stream).vb_ptr = ((*pMP3Stream).vb_ptr - 8) & 127;
			pcm = pcm.add(16);
			i += 1;
		}
	} else {
		i = 0;
		while i < 18 {
			fdct8(sample, (*pMP3Stream).vbuf2.add((*pMP3Stream).vb2_ptr as usize) as *mut f32);
			windowB8_dual((*pMP3Stream).vbuf2, (*pMP3Stream).vb2_ptr, pcm.add(1));
			sample = sample.add(32);
			(*pMP3Stream).vb2_ptr = ((*pMP3Stream).vb2_ptr - 8) & 127;
			pcm = pcm.add(16);
			i += 1;
		}
	}
}
/*------------------------------------------------------------*/
