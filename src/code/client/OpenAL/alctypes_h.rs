// OpenAL cross platform audio library
// Copyright (C) 1999-2000 by authors.
// This library is free software; you can redistribute it and/or
//  modify it under the terms of the GNU Library General Public
//  License as published by the Free Software Foundation; either
//  version 2 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  Library General Public License for more details.
//
// You should have received a copy of the GNU Library General Public
//  License along with this library; if not, write to the
//  Free Software Foundation, Inc., 59 Temple Place - Suite 330,
//  Boston, MA  02111-1307, USA.
// Or go to http://www.gnu.org/copyleft/lgpl.html

use core::ffi::{c_char, c_int, c_uint, c_short, c_ushort, c_double, c_void};

#[cfg(xbox)]
pub type ALCdevice = c_int;

#[cfg(xbox)]
pub type ALCcontext = c_int;

// ALC boolean type.
pub type ALCboolean = c_char;

// ALC 8bit signed byte.
pub type ALCbyte = c_char;

// ALC 8bit unsigned byte.
pub type ALCubyte = u8;

// ALC 16bit signed short integer type.
pub type ALCshort = c_short;

// ALC 16bit unsigned short integer type.
pub type ALCushort = c_ushort;

// ALC 32bit unsigned integer type.
pub type ALCuint = c_uint;

// ALC 32bit signed integer type.
pub type ALCint = c_int;

// ALC 32bit floating point type.
pub type ALCfloat = f32;

// ALC 64bit double point type.
pub type ALCdouble = c_double;

// ALC 32bit type.
pub type ALCsizei = c_uint;

// ALC void type
pub type ALCvoid = c_void;

// ALC enumerations.
pub type ALCenum = c_int;

// Bad value.
pub const ALC_INVALID: i32 = -1;

// Boolean False.
pub const ALC_FALSE: i32 = 0;

// Boolean True.
pub const ALC_TRUE: i32 = 1;

// Errors: No Error.
pub const ALC_NO_ERROR: i32 = ALC_FALSE;

pub const ALC_MAJOR_VERSION: i32 = 0x1000;
pub const ALC_MINOR_VERSION: i32 = 0x1001;
pub const ALC_ATTRIBUTES_SIZE: i32 = 0x1002;
pub const ALC_ALL_ATTRIBUTES: i32 = 0x1003;

pub const ALC_DEFAULT_DEVICE_SPECIFIER: i32 = 0x1004;
pub const ALC_DEVICE_SPECIFIER: i32 = 0x1005;
pub const ALC_EXTENSIONS: i32 = 0x1006;

pub const ALC_FREQUENCY: i32 = 0x1007;
pub const ALC_REFRESH: i32 = 0x1008;
pub const ALC_SYNC: i32 = 0x1009;

// The device argument does not name a valid dvice.
pub const ALC_INVALID_DEVICE: i32 = 0xA001;

// The context argument does not name a valid context.
pub const ALC_INVALID_CONTEXT: i32 = 0xA002;

// A function was called at inappropriate time,
//  or in an inappropriate way, causing an illegal state.
// This can be an incompatible ALenum, object ID,
//  and/or function.
pub const ALC_INVALID_ENUM: i32 = 0xA003;

// Illegal value passed as an argument to an AL call.
// Applies to parameter values, but not to enumerations.
pub const ALC_INVALID_VALUE: i32 = 0xA004;

// A function could not be completed,
// because there is not enough memory available.
pub const ALC_OUT_OF_MEMORY: i32 = 0xA005;
