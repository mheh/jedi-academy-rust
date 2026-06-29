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

use core::ffi::{c_int, c_uint, c_char, c_void};

// OpenAL boolean type.
pub type ALboolean = c_char;

// OpenAL 8bit signed byte.
pub type ALbyte = c_char;

// OpenAL 8bit unsigned byte.
pub type ALubyte = u8;

// OpenAL 16bit signed short integer type.
pub type ALshort = i16;

// OpenAL 16bit unsigned short integer type.
pub type ALushort = u16;

// OpenAL 32bit unsigned integer type.
pub type ALuint = c_uint;

// OpenAL 32bit signed integer type.
pub type ALint = c_int;

// OpenAL 32bit floating point type.
pub type ALfloat = f32;

// OpenAL 64bit double point type.
pub type ALdouble = f64;

// OpenAL 32bit type.
pub type ALsizei = c_uint;

// OpenAL void type
pub type ALvoid = c_void;

// OpenAL enumerations.
pub type ALenum = c_int;

// Bad value.
pub const AL_INVALID: i32 = -1;

// Disable value.
pub const AL_NONE: i32 = 0;

// Boolean False.
pub const AL_FALSE: i32 = 0;

// Boolean True.
pub const AL_TRUE: i32 = 1;

// Indicate the type of AL_SOURCE.
// Sources can be spatialized
pub const AL_SOURCE_TYPE: i32 = 0x200;

// Indicate source has absolute coordinates.
pub const AL_SOURCE_ABSOLUTE: i32 = 0x201;

// Indicate Source has listener relative coordinates.
pub const AL_SOURCE_RELATIVE: i32 = 0x202;

// Directional source, inner cone angle, in degrees.
// Range:    [0-360]
// Default:  360
pub const AL_CONE_INNER_ANGLE: i32 = 0x1001;

// Directional source, outer cone angle, in degrees.
// Range:    [0-360]
// Default:  360
pub const AL_CONE_OUTER_ANGLE: i32 = 0x1002;

// Specify the pitch to be applied, either at source,
//  or on mixer results, at listener.
// Range:	 [0.5-2.0]
// Default:  1.0
pub const AL_PITCH: i32 = 0x1003;

// Specify the current location in three dimensional space.
// OpenAL, like OpenGL, uses a right handed coordinate system,
//  where in a frontal default view X (thumb) points right,
//  Y points up (index finger), and Z points towards the
//  viewer/camera (middle finger).
// To switch from a left handed coordinate system, flip the
//  sign on the Z coordinate.
// Listener position is always in the world coordinate system.
pub const AL_POSITION: i32 = 0x1004;

// Specify the current direction as forward vector.
pub const AL_DIRECTION: i32 = 0x1005;

// Specify the current velocity in three dimensional space.
pub const AL_VELOCITY: i32 = 0x1006;

// Indicate whether source has to loop infinite.
// Type: ALboolean
// Range:    [AL_TRUE, AL_FALSE]
// Default:  AL_FALSE
pub const AL_LOOPING: i32 = 0x1007;

// Indicate the buffer to provide sound samples.
// Type: ALuint.
// Range: any valid Buffer id.
pub const AL_BUFFER: i32 = 0x1009;

// Indicate the gain (volume amplification) applied.
// Type:     ALfloat.
// Range:    ]0.0-  ]
// A value of 1.0 means un-attenuated/unchanged.
// Each division by 2 equals an attenuation of -6dB.
// Each multiplicaton with 2 equals an amplification of +6dB.
// A value of 0.0 is meaningless with respect to a logarithmic
//  scale; it is interpreted as zero volume - the channel
//  is effectively disabled.
pub const AL_GAIN: i32 = 0x100A;

// Indicate minimum source attenuation.
// Type:     ALfloat
// Range:	 [0.0 - 1.0]
pub const AL_MIN_GAIN: i32 = 0x100D;

// Indicate maximum source attenuation.
// Type:	 ALfloat
// Range:	 [0.0 - 1.0]
pub const AL_MAX_GAIN: i32 = 0x100E;

// Specify the current orientation.
// Type:	 ALfv6 (at/up)
// Range:	 N/A
pub const AL_ORIENTATION: i32 = 0x100F;

// byte offset into source (in canon format).  -1 if source
// is not playing.  Don't set this, get this.
//
// Type:     ALfloat
// Range:    [0.0 - ]
// Default:  1.0
pub const AL_REFERENCE_DISTANCE: i32 = 0x1020;

// Indicate the rolloff factor for the source.
// Type: ALfloat
// Range:    [0.0 - ]
// Default:  1.0
pub const AL_ROLLOFF_FACTOR: i32 = 0x1021;

// Indicate the gain (volume amplification) applied.
// Type:     ALfloat.
// Range:    ]0.0-  ]
// A value of 1.0 means un-attenuated/unchanged.
// Each division by 2 equals an attenuation of -6dB.
// Each multiplicaton with 2 equals an amplification of +6dB.
// A value of 0.0 is meaningless with respect to a logarithmic
//  scale; it is interpreted as zero volume - the channel
//  is effectively disabled.
pub const AL_CONE_OUTER_GAIN: i32 = 0x1022;

// Specify the maximum distance.
// Type:	 ALfloat
// Range:	 [0.0 - ]
pub const AL_MAX_DISTANCE: i32 = 0x1023;

// Specify the panning to be applied (2D only.)
// Range:	 [-1.0 - 1.0]
// Default:  0.0
pub const AL_PAN: i32 = 0x1024;

// Get the playing time of a stream.
// Range:	 [0.0 - infinity]
pub const AL_TIME: i32 = 0x1025;

// Specify the channel mask. (Creative)
// Type:	 ALuint
// Range:	 [0 - 255]
pub const AL_CHANNEL_MASK: i32 = 0x3000;

// Source state information
pub const AL_SOURCE_STATE: i32 = 0x1010;
pub const AL_INITIAL: i32 = 0x1011;
pub const AL_PLAYING: i32 = 0x1012;
pub const AL_PAUSED: i32 = 0x1013;
pub const AL_STOPPED: i32 = 0x1014;

// Buffer Queue params
pub const AL_BUFFERS_QUEUED: i32 = 0x1015;
pub const AL_BUFFERS_PROCESSED: i32 = 0x1016;

// Sound buffers: format specifier.
pub const AL_FORMAT_MONO8: i32 = 0x1100;
pub const AL_FORMAT_MONO16: i32 = 0x1101;
pub const AL_FORMAT_STEREO8: i32 = 0x1102;
pub const AL_FORMAT_STEREO16: i32 = 0x1103;
pub const AL_FORMAT_MONO4: i32 = 0x1104;
pub const AL_FORMAT_STEREO4: i32 = 0x1105;

// Sound buffers: frequency, in units of Hertz [Hz].
// This is the number of samples per second. Half of the
//  sample frequency marks the maximum significant
//  frequency component.
pub const AL_FREQUENCY: i32 = 0x2001;
pub const AL_BITS: i32 = 0x2002;
pub const AL_CHANNELS: i32 = 0x2003;
pub const AL_SIZE: i32 = 0x2004;
pub const AL_DATA: i32 = 0x2005;

// Buffer state.
//
// Not supported for public use (yet).
pub const AL_UNUSED: i32 = 0x2010;
pub const AL_PENDING: i32 = 0x2011;
pub const AL_PROCESSED: i32 = 0x2012;

// Errors: No Error.
pub const AL_NO_ERROR: i32 = AL_FALSE;

// Illegal name passed as an argument to an AL call.
pub const AL_INVALID_NAME: i32 = 0xA001;

// Illegal enum passed as an argument to an AL call.
pub const AL_INVALID_ENUM: i32 = 0xA002;

// Illegal value passed as an argument to an AL call.
// Applies to parameter values, but not to enumerations.
pub const AL_INVALID_VALUE: i32 = 0xA003;

// A function was called at inappropriate time,
//  or in an inappropriate way, causing an illegal state.
// This can be an incompatible ALenum, object ID,
//  and/or function.
pub const AL_INVALID_OPERATION: i32 = 0xA004;

// A function could not be completed,
// because there is not enough memory available.
pub const AL_OUT_OF_MEMORY: i32 = 0xA005;

// Context strings: Vendor Name.
pub const AL_VENDOR: i32 = 0xB001;
pub const AL_VERSION: i32 = 0xB002;
pub const AL_RENDERER: i32 = 0xB003;
pub const AL_EXTENSIONS: i32 = 0xB004;
pub const AL_MEMORY_USED: i32 = 0xB005;
pub const AL_MEMORY_ALLOCATOR: i32 = 0xB006;
pub const AL_MEMORY_DEALLOCATOR: i32 = 0xB007;
pub const AL_STEREO: i32 = 0xB008;

// Global tweakage.

// Doppler scale.  Default 1.0
pub const AL_DOPPLER_FACTOR: i32 = 0xC000;

// Doppler velocity.  Default 1.0
pub const AL_DOPPLER_VELOCITY: i32 = 0xC001;

// Distance model.  Default AL_INVERSE_DISTANCE_CLAMPED
pub const AL_DISTANCE_MODEL: i32 = 0xD000;

// Distance models.

pub const AL_INVERSE_DISTANCE: i32 = 0xD001;
pub const AL_INVERSE_DISTANCE_CLAMPED: i32 = 0xD002;
