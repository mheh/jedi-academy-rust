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

#![allow(non_snake_case)]

/* Original C configuration for reference:
 *
 * #include <limits.h>
 *
 * #define HAVE_IO_H 1
 * #define HAVE_ERRNO_H 1
 *
 * #if HAVE_UNISTD_H
 * #define RD_BNRY_FLAGS O_RDONLY
 * #elif HAVE_IO_H
 * #define RD_BNRY_FLAGS O_RDONLY | O_BINARY
 * #endif
 */

/* Endian Issues */
#[cfg(target_os = "linux")]
// #include <endian.h>
pub const _ENDIAN_AWARE: bool = true;

#[cfg(target_os = "windows")]
// Platform-specific endian definitions for Windows
pub const __LITTLE_ENDIAN: u32 = 1234;
pub const __BIG_ENDIAN: u32 = 4321;
pub const __PDP_ENDIAN: u32 = 3412;
pub const __BYTE_ORDER: u32 = __LITTLE_ENDIAN;

// usleep(x) ::Sleep(x/1000)
// strcasecmp(a,b) stricmp(a,b)
// strncasecmp(a,b,c) strnicmp(a,b,c)
// typedef int socklen_t;

#[cfg(not(target_os = "windows"))]
pub const __LITTLE_ENDIAN: u32 = 1234;
#[cfg(not(target_os = "windows"))]
pub const __BIG_ENDIAN: u32 = 4321;
#[cfg(not(target_os = "windows"))]
pub const __PDP_ENDIAN: u32 = 3412;
#[cfg(not(target_os = "windows"))]
pub const __BYTE_ORDER: u32 = __LITTLE_ENDIAN;

/* _MAX_PATH */
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const _MAX_PATH: usize = 260;

/* define our datatypes */
// real number
pub type real = f64;

/* Check for 8-bit types */
// #if UCHAR_MAX == 0xff
// typedef unsigned char	uint8;
// typedef signed char		int8;
// #else
// #error This machine has no 8-bit type
// #endif
pub type uint8 = u8;
pub type int8 = i8;

/* Check for 16-bit types */
// #if UINT_MAX == 0xffff
// typedef unsigned int	uint16;
// typedef int				int16;
// #elif USHRT_MAX == 0xffff
// typedef unsigned short	uint16;
// typedef short			int16;
// #else
// #error This machine has no 16-bit type
// #endif
pub type uint16 = u16;
pub type int16 = i16;

/* Check for 32-bit types */
// #if UINT_MAX == 0xfffffffful
// typedef unsigned int	uint32;
// typedef int				int32;
// #elif ULONG_MAX == 0xfffffffful
// typedef unsigned long	uint32;
// typedef long			int32;
// #elif USHRT_MAX == 0xfffffffful
// typedef unsigned short	uint32;
// typedef short			int32;
// #else
// #error This machine has no 32-bit type
// #endif
pub type uint32 = u32;
pub type int32 = i32;

// What character marks the end of a directory entry? For DOS and
// Windows, it is "\"; in UNIX it is "/".
#[cfg(any(target_os = "windows"))]
pub const DIR_MARKER: u8 = b'\\';
#[cfg(any(target_os = "windows"))]
pub const DIR_MARKER_STR: &str = "\\";
#[cfg(not(any(target_os = "windows")))]
pub const DIR_MARKER: u8 = b'/';
#[cfg(not(any(target_os = "windows")))]
pub const DIR_MARKER_STR: &str = "/";

// What character(s) marks the end of a line in a text file?
// For DOS and Windows, it is "\r\n"; in UNIX it is "\r".
#[cfg(any(target_os = "windows"))]
pub const LINE_END_MARKER_STR: &str = "\r\n";
#[cfg(not(any(target_os = "windows")))]
pub const LINE_END_MARKER_STR: &str = "\n";
