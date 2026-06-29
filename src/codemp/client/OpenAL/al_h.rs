#![allow(non_snake_case)]

use core::ffi::{c_char, c_double, c_float, c_int, c_short, c_uint, c_uchar, c_ushort, c_void};

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

// Type definitions from altypes.h

/// OpenAL boolean type
pub type ALboolean = c_char;

/// OpenAL 8bit signed byte
pub type ALbyte = c_char;

/// OpenAL 8bit unsigned byte
pub type ALubyte = c_uchar;

/// OpenAL 16bit signed short integer type
pub type ALshort = c_short;

/// OpenAL 16bit unsigned short integer type
pub type ALushort = c_ushort;

/// OpenAL 32bit unsigned integer type
pub type ALuint = c_uint;

/// OpenAL 32bit signed integer type
pub type ALint = c_int;

/// OpenAL 32bit floating point type
pub type ALfloat = c_float;

/// OpenAL 64bit double point type
pub type ALdouble = c_double;

/// OpenAL 32bit type
pub type ALsizei = c_uint;

/// OpenAL void type
pub type ALvoid = c_void;

/// OpenAL enumerations
pub type ALenum = c_int;

// OpenAL Maintenance Functions
// Initialization and exiting.
// State Management and Query.
// Error Handling.
// Extension Support.

extern "C" {
    /// State management.
    pub fn alEnable(capability: ALenum);
    pub fn alDisable(capability: ALenum);
    pub fn alIsEnabled(capability: ALenum) -> ALboolean;

    /// Application preferences for driver performance choices.
    pub fn alHint(target: ALenum, mode: ALenum);

    /// State retrieval.
    pub fn alGetBoolean(param: ALenum) -> ALboolean;
    pub fn alGetInteger(param: ALenum) -> ALint;
    pub fn alGetFloat(param: ALenum) -> ALfloat;
    pub fn alGetDouble(param: ALenum) -> ALdouble;
    pub fn alGetBooleanv(param: ALenum, data: *mut ALboolean);
    pub fn alGetIntegerv(param: ALenum, data: *mut ALint);
    pub fn alGetFloatv(param: ALenum, data: *mut ALfloat);
    pub fn alGetDoublev(param: ALenum, data: *mut ALdouble);
    pub fn alGetString(param: ALenum) -> *mut ALubyte;

    // Error support.
    // Obtain the most recent error generated in the AL state machine.
    pub fn alGetError() -> ALenum;

    // Extension support.
    // Obtain the address of a function (usually an extension)
    //  with the name fname. All addresses are context-independent.
    pub fn alIsExtensionPresent(fname: *mut ALubyte) -> ALboolean;

    // Extension support.
    // Obtain the address of a function (usually an extension)
    //  with the name fname. All addresses are context-independent.
    pub fn alGetProcAddress(fname: *mut ALubyte) -> *mut ALvoid;

    // Extension support.
    // Obtain the integer value of an enumeration (usually an extension) with the name ename.
    pub fn alGetEnumValue(ename: *mut ALubyte) -> ALenum;
}

#[cfg(target_os = "xbox")]
extern "C" {
    // Update cycle.
    pub fn alUpdate();

    // Returns a global state parameter.
    pub fn alGeti(param: ALenum, value: *mut ALint);

    // Adjust the size of the sound buffer pool.
    pub fn alResizePool(size: ALuint);

    // Listener create and delete
    pub fn alGenListeners(n: ALsizei, listeners: *mut ALuint);
    pub fn alDeleteListeners(n: ALsizei, listeners: *mut ALuint);
}

// LISTENER
// Listener is the sample position for a given context.
// The multi-channel (usually stereo) output stream generated
//  by the mixer is parametrized by this Listener object:
//  its position and velocity relative to Sources, within
//  occluder and reflector geometry.

// VV's Console version of openAL includes multiple listener support,
// as an extra first arg to the alListener functions. We wrap that up
// in a macro here, to make the following more concise.

#[cfg(target_os = "xbox")]
extern "C" {
    // Listener Environment:  default 0.
    pub fn alListeneri(listener: ALuint, param: ALenum, value: ALint);

    // Listener Gain:  default 1.0f.
    pub fn alListenerf(listener: ALuint, param: ALenum, value: ALfloat);

    // Listener Position.
    // Listener Velocity.
    pub fn alListener3f(listener: ALuint, param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);

    // Listener Position:        ALfloat[3]
    // Listener Velocity:        ALfloat[3]
    // Listener Orientation:     ALfloat[6]  (forward and up vector).
    pub fn alListenerfv(listener: ALuint, param: ALenum, values: *mut ALfloat);

    pub fn alGetListeneri(listener: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetListenerf(listener: ALuint, param: ALenum, value: *mut ALfloat);
    pub fn alGetListener3f(listener: ALuint, param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
    pub fn alGetListenerfv(listener: ALuint, param: ALenum, values: *mut ALfloat);
}

#[cfg(not(target_os = "xbox"))]
extern "C" {
    // Listener Environment:  default 0.
    pub fn alListeneri(param: ALenum, value: ALint);

    // Listener Gain:  default 1.0f.
    pub fn alListenerf(param: ALenum, value: ALfloat);

    // Listener Position.
    // Listener Velocity.
    pub fn alListener3f(param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);

    // Listener Position:        ALfloat[3]
    // Listener Velocity:        ALfloat[3]
    // Listener Orientation:     ALfloat[6]  (forward and up vector).
    pub fn alListenerfv(param: ALenum, values: *mut ALfloat);

    pub fn alGetListeneri(param: ALenum, value: *mut ALint);
    pub fn alGetListenerf(param: ALenum, value: *mut ALfloat);
    pub fn alGetListener3f(param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
    pub fn alGetListenerfv(param: ALenum, values: *mut ALfloat);
}

// SOURCE
// Source objects are by default localized. Sources
//  take the PCM data provided in the specified Buffer,
//  apply Source-specific modifications, and then
//  submit them to be mixed according to spatial
//  arrangement etc.

extern "C" {
    /// Create Source objects.
    #[cfg(target_os = "xbox")]
    pub fn alGenSources2D(n: ALsizei, sources: *mut ALuint);
    #[cfg(target_os = "xbox")]
    pub fn alGenSources3D(n: ALsizei, sources: *mut ALuint);
    #[cfg(not(target_os = "xbox"))]
    pub fn alGenSources(n: ALsizei, sources: *mut ALuint);

    /// Delete Source objects.
    pub fn alDeleteSources(n: ALsizei, sources: *mut ALuint);

    /// Verify a handle is a valid Source.
    pub fn alIsSource(id: ALuint) -> ALboolean;

    /// Set an integer parameter for a Source object.
    pub fn alSourcei(source: ALuint, param: ALenum, value: ALint);
    pub fn alSourcef(source: ALuint, param: ALenum, value: ALfloat);
    pub fn alSource3f(source: ALuint, param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);
    pub fn alSourcefv(source: ALuint, param: ALenum, values: *mut ALfloat);

    /// Get an integer parameter for a Source object.
    pub fn alGetSourcei(source: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetSourcef(source: ALuint, param: ALenum, value: *mut ALfloat);
    pub fn alGetSource3f(source: ALuint, param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
    pub fn alGetSourcefv(source: ALuint, param: ALenum, values: *mut ALfloat);

    pub fn alSourcePlayv(n: ALsizei, sources: *mut ALuint);
    pub fn alSourcePausev(n: ALsizei, sources: *mut ALuint);
    pub fn alSourceStopv(n: ALsizei, sources: *mut ALuint);
    pub fn alSourceRewindv(n: ALsizei, sources: *mut ALuint);

    /// Activate a source, start replay.
    pub fn alSourcePlay(source: ALuint);

    // Pause a source,
    //  temporarily remove it from the mixer list.
    pub fn alSourcePause(source: ALuint);

    // Stop a source,
    //  temporarily remove it from the mixer list,
    //  and reset its internal state to pre-Play.
    // To remove a Source completely, it has to be
    //  deleted following Stop, or before Play.
    pub fn alSourceStop(source: ALuint);

    // Rewinds a source,
    //  temporarily remove it from the mixer list,
    //  and reset its internal state to pre-Play.
    pub fn alSourceRewind(source: ALuint);
}

// BUFFER
// Buffer objects are storage space for sample data.
// Buffers are referred to by Sources. There can be more than
//  one Source using the same Buffer data. If Buffers have
//  to be duplicated on a per-Source basis, the driver has to
//  take care of allocation, copying, and deallocation as well
//  as propagating buffer data changes.

extern "C" {
    /// Buffer object generation.
    pub fn alGenBuffers(n: ALsizei, buffers: *mut ALuint);
    pub fn alDeleteBuffers(n: ALsizei, buffers: *mut ALuint);
    pub fn alIsBuffer(buffer: ALuint) -> ALboolean;

    /// Specify the data to be filled into a buffer.
    pub fn alBufferData(
        buffer: ALuint,
        format: ALenum,
        data: *mut ALvoid,
        size: ALsizei,
        freq: ALsizei,
    );

    pub fn alGetBufferi(buffer: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetBufferf(buffer: ALuint, param: ALenum, value: *mut ALfloat);
}

// Queue stuff

extern "C" {
    pub fn alSourceQueueBuffers(source: ALuint, n: ALsizei, buffers: *mut ALuint);
    pub fn alSourceUnqueueBuffers(source: ALuint, n: ALsizei, buffers: *mut ALuint);
}

#[cfg(target_os = "xbox")]
extern "C" {
    // STREAM
    // Stream objects encapsulate traditional sound stream and
    //  act as both sources and buffers.  They emit sound and
    //  manage sound data.

    pub fn alGenStream();
    pub fn alDeleteStream();

    pub fn alStreamStop();
    pub fn alStreamPlay(offset: ALsizei, file: ALint, al_loop: ALint);

    pub fn alStreamf(param: ALenum, value: ALfloat);

    pub fn alGetStreamf(param: ALenum, value: *mut ALfloat);
    pub fn alGetStreami(param: ALenum, value: *mut ALint);
}

// Knobs and dials

extern "C" {
    pub fn alDistanceModel(value: ALenum);
    pub fn alDopplerFactor(value: ALfloat);
    pub fn alDopplerVelocity(value: ALfloat);
}

#[cfg(target_os = "xbox")]
extern "C" {
    pub fn alGain(value: ALfloat);
}
