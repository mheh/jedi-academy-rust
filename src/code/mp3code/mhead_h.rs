#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_long};

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

	$Id: mhead.h,v 1.3 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/* portable copy of eco\mhead.h */
/* mpeg audio header   */
#[repr(C)]
pub struct MPEG_HEAD {
	pub sync: c_int,			// 1 if valid sync
	pub id: c_int,
	pub option: c_int,
	pub prot: c_int,
	pub br_index: c_int,
	pub sr_index: c_int,
	pub pad: c_int,
	pub private_bit: c_int,
	pub mode: c_int,
	pub mode_ext: c_int,
	pub cr: c_int,
	pub original: c_int,
	pub emphasis: c_int,
}

/* portable mpeg audio decoder, decoder functions */

// IN_OUT type from small_header.h (stub)
pub type IN_OUT = c_int;

#[repr(C)]
pub struct DEC_INFO {
	pub channels: c_int,
	pub outvalues: c_int,
	pub samprate: c_long,
	pub bits: c_int,
	pub framebytes: c_int,
	pub type_: c_int,
}

extern "C" {
	pub fn head_info(buf: *mut u8, n: c_uint, h: *mut MPEG_HEAD) -> c_int;
	pub fn head_info2(
		buf: *mut u8,
		n: c_uint,
		h: *mut MPEG_HEAD,
		br: *mut c_int,
	) -> c_int;
	pub fn head_info3(
		buf: *mut u8,
		n: c_uint,
		h: *mut MPEG_HEAD,
		br: *mut c_int,
		searchForward: *mut c_uint,
	) -> c_int;
	// head_info returns framebytes > 0 for success
	// audio_decode_init returns 1 for success, 0 for fail
	// audio_decode returns in_bytes = 0 on sync loss

	pub fn audio_decode_init(
		h: *mut MPEG_HEAD,
		framebytes_arg: c_int,
		reduction_code: c_int,
		transform_code: c_int,
		convert_code: c_int,
		freq_limit: c_int,
	) -> c_int;
	pub fn audio_decode_info(info: *mut DEC_INFO);
	pub fn audio_decode(
		bs: *mut u8,
		pcm: *mut i16,
		pNextByteAfterData: *mut u8,
	) -> IN_OUT;

	pub fn audio_decode8_init(
		h: *mut MPEG_HEAD,
		framebytes_arg: c_int,
		reduction_code: c_int,
		transform_code: c_int,
		convert_code: c_int,
		freq_limit: c_int,
	) -> c_int;
	pub fn audio_decode8_info(info: *mut DEC_INFO);
	pub fn audio_decode8(bs: *mut u8, pcmbuf: *mut i16) -> IN_OUT;
}
