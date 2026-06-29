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

	$Id: port.h,v 1.2 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint};

#[cfg(not(any(
    target_os = "dos",
    target_os = "windows",
    target_arch = "x86"
)))]
pub const O_BINARY: c_int = 0;

// no kb function unless DOS

#[cfg(any(
    target_os = "dos",
    all(target_os = "windows", target_env = "msvc")
))]
pub const KB_OK: bool = true;

// -- no pcm conversion to wave required
// if short = 16 bits and little endian --

// mods 1/9/97 LITTLE_SHORT16 detect

// JDW //
// #ifdef LITTLE_SHORT16
// #define cvt_to_wave_init(a)
// #define cvt_to_wave(a, b)  b
// #else
// void cvt_to_wave_init(int bits);
// unsigned int cvt_to_wave(void *a, unsigned int b);
// #endif

#[cfg(any(
    target_os = "dos",
    target_os = "windows",
    target_arch = "x86"
))]
#[inline]
pub fn cvt_to_wave_init(_a: c_int) {}

#[cfg(any(
    target_os = "dos",
    target_os = "windows",
    target_arch = "x86"
))]
#[inline]
pub fn cvt_to_wave(_a: *const u8, b: c_uint) -> c_uint {
    b
}

#[cfg(not(any(
    target_os = "dos",
    target_os = "windows",
    target_arch = "x86"
)))]
extern "C" {
    pub fn cvt_to_wave_init(arg: c_int);
    pub fn cvt_to_wave(a: *mut u8, b: c_uint) -> c_uint;
}
