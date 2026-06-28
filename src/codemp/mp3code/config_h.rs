/*____________________________________________________________________________

    FreeAmp - The Free MP3 Player

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

    $Id: config.win32,v 1.16 1999/12/09 08:44:07 elrod Exp $
____________________________________________________________________________*/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_uint, c_void};

pub const HAVE_IO_H: c_int = 1;
pub const HAVE_ERRNO_H: c_int = 1;

pub const O_RDONLY: c_int = 0;
pub const O_BINARY: c_int = 0;
pub const RD_BNRY_FLAGS: c_int = O_RDONLY | O_BINARY;

/* Endian Issues */
pub const __LITTLE_ENDIAN: c_int = 1234;
pub const __BIG_ENDIAN: c_int = 4321;
pub const __PDP_ENDIAN: c_int = 3412;

#[cfg(target_endian = "little")]
pub const __BYTE_ORDER: c_int = __LITTLE_ENDIAN;
#[cfg(target_endian = "big")]
pub const __BYTE_ORDER: c_int = __BIG_ENDIAN;

#[cfg(windows)]
pub type socklen_t = c_int;

#[cfg(windows)]
extern "C" {
    pub fn Sleep(dwMilliseconds: c_uint);
    pub fn stricmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn strnicmp(a: *const c_char, b: *const c_char, c: usize) -> c_int;
}

#[cfg(windows)]
#[inline]
pub unsafe fn usleep(x: c_uint) {
    // SAFETY: Mirrors the C `#define usleep(x) ::Sleep(x/1000)` Windows shim.
    unsafe { Sleep(x / 1000) };
}

#[cfg(windows)]
#[inline]
pub unsafe fn strcasecmp(a: *const c_char, b: *const c_char) -> c_int {
    // SAFETY: Raw C string pointers are forwarded exactly as the original macro did.
    unsafe { stricmp(a, b) }
}

#[cfg(windows)]
#[inline]
pub unsafe fn strncasecmp(a: *const c_char, b: *const c_char, c: usize) -> c_int {
    // SAFETY: Raw C string pointers are forwarded exactly as the original macro did.
    unsafe { strnicmp(a, b, c) }
}

pub const _MAX_PATH: usize = 260;

/* define our datatypes */
// real number
pub type real = f64;

pub type uint8 = u8;
pub type int8 = i8;
pub type uint16 = u16;
pub type int16 = i16;
pub type uint32 = u32;
pub type int32 = i32;

const _: () = assert!(core::mem::size_of::<uint8>() == 1);
const _: () = assert!(core::mem::size_of::<int8>() == 1);
const _: () = assert!(core::mem::size_of::<uint16>() == 2);
const _: () = assert!(core::mem::size_of::<int16>() == 2);
const _: () = assert!(core::mem::size_of::<uint32>() == 4);
const _: () = assert!(core::mem::size_of::<int32>() == 4);

// What character marks the end of a directory entry? For DOS and
// Windows, it is "\"; in UNIX it is "/".
#[cfg(windows)]
pub const DIR_MARKER: c_char = b'\\' as c_char;
#[cfg(windows)]
pub const DIR_MARKER_STR: &str = "\\";

#[cfg(not(windows))]
pub const DIR_MARKER: c_char = b'/' as c_char;
#[cfg(not(windows))]
pub const DIR_MARKER_STR: &str = "/";

// What character(s) marks the end of a line in a text file?
// For DOS and Windows, it is "\r\n"; in UNIX it is "\r".
#[cfg(windows)]
pub const LINE_END_MARKER_STR: &str = "\r\n";

#[cfg(not(windows))]
pub const LINE_END_MARKER_STR: &str = "\n";

pub const NULL: *mut c_void = core::ptr::null_mut();
