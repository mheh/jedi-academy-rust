#![allow(non_snake_case)]

use super::altypes_h::*;
use core::ffi::c_void;

/**
 * OpenAL cross platform audio library
 * Copyright (C) 1999-2000 by authors.
 * This library is free software; you can redistribute it and/or
 *  modify it under the terms of the GNU Library General Public
 *  License as published by the Free Software Foundation; either
 *  version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 *  Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public
 *  License along with this library; if not, write to the
 *  Free Software Foundation, Inc., 59 Temple Place - Suite 330,
 *  Boston, MA  02111-1307, USA.
 * Or go to http://www.gnu.org/copyleft/lgpl.html
 */

// Platform-specific calling convention definitions.
// In Rust, extern "C" handles the calling convention,
// so ALAPI/ALAPIENTRY/AL_CALLBACK macros are not needed.

#[cfg(not(target_os = "xbox360"))]
extern "C" {
    /**
     * OpenAL Maintenance Functions
     * Initialization and exiting.
     * State Management and Query.
     * Error Handling.
     * Extension Support.
     */

    /** State management. */
    pub fn alEnable(capability: ALenum);
    pub fn alDisable(capability: ALenum);
    pub fn alIsEnabled(capability: ALenum) -> ALboolean;

    /** Application preferences for driver performance choices. */
    pub fn alHint(target: ALenum, mode: ALenum);

    /** State retrieval. */
    pub fn alGetBoolean(param: ALenum) -> ALboolean;
    pub fn alGetInteger(param: ALenum) -> ALint;
    pub fn alGetFloat(param: ALenum) -> ALfloat;
    pub fn alGetDouble(param: ALenum) -> ALdouble;
    pub fn alGetBooleanv(param: ALenum, data: *mut ALboolean);
    pub fn alGetIntegerv(param: ALenum, data: *mut ALint);
    pub fn alGetFloatv(param: ALenum, data: *mut ALfloat);
    pub fn alGetDoublev(param: ALenum, data: *mut ALdouble);
    pub fn alGetString(param: ALenum) -> *mut ALubyte;

    /**
     * Error support.
     * Obtain the most recent error generated in the AL state machine.
     */
    pub fn alGetError() -> ALenum;

    /**
     * Extension support.
     * Obtain the address of a function (usually an extension)
     *  with the name fname. All addresses are context-independent.
     */
    pub fn alIsExtensionPresent(fname: *mut ALubyte) -> ALboolean;

    /**
     * Extension support.
     * Obtain the address of a function (usually an extension)
     *  with the name fname. All addresses are context-independent.
     */
    pub fn alGetProcAddress(fname: *mut ALubyte) -> *mut c_void;

    /**
     * Extension support.
     * Obtain the integer value of an enumeration (usually an extension) with the name ename.
     */
    pub fn alGetEnumValue(ename: *mut ALubyte) -> ALenum;

    /**
     * LISTENER
     * Listener is the sample position for a given context.
     * The multi-channel (usually stereo) output stream generated
     *  by the mixer is parametrized by this Listener object:
     *  its position and velocity relative to Sources, within
     *  occluder and reflector geometry.
     */

    /**
     *
     * Listener Environment:  default 0.
     */
    pub fn alListeneri(param: ALenum, value: ALint);

    /**
     *
     * Listener Gain:  default 1.0f.
     */
    pub fn alListenerf(param: ALenum, value: ALfloat);

    /**
     *
     * Listener Position.
     * Listener Velocity.
     */
    pub fn alListener3f(param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);

    /**
     *
     * Listener Position:        ALfloat[3]
     * Listener Velocity:        ALfloat[3]
     * Listener Orientation:     ALfloat[6]  (forward and up vector).
     */
    pub fn alListenerfv(param: ALenum, values: *mut ALfloat);

    pub fn alGetListeneri(param: ALenum, value: *mut ALint);
    pub fn alGetListenerf(param: ALenum, value: *mut ALfloat);
    pub fn alGetListener3f(param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
    pub fn alGetListenerfv(param: ALenum, values: *mut ALfloat);

    /**
     * SOURCE
     * Source objects are by default localized. Sources
     *  take the PCM data provided in the specified Buffer,
     *  apply Source-specific modifications, and then
     *  submit them to be mixed according to spatial
     *  arrangement etc.
     */

    /** Create Source objects. */
    pub fn alGenSources(n: ALsizei, sources: *mut ALuint);

    /** Delete Source objects. */
    pub fn alDeleteSources(n: ALsizei, sources: *mut ALuint);

    /** Verify a handle is a valid Source. */
    pub fn alIsSource(id: ALuint) -> ALboolean;

    /** Set an integer parameter for a Source object. */
    pub fn alSourcei(source: ALuint, param: ALenum, value: ALint);
    pub fn alSourcef(source: ALuint, param: ALenum, value: ALfloat);
    pub fn alSource3f(source: ALuint, param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);
    pub fn alSourcefv(source: ALuint, param: ALenum, values: *mut ALfloat);

    /** Get an integer parameter for a Source object. */
    pub fn alGetSourcei(source: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetSourcef(source: ALuint, param: ALenum, value: *mut ALfloat);
    pub fn alGetSource3f(source: ALuint, param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
    pub fn alGetSourcefv(source: ALuint, param: ALenum, values: *mut ALfloat);

    pub fn alSourcePlayv(n: ALsizei, sources: *mut ALuint);
    pub fn alSourcePausev(n: ALsizei, sources: *mut ALuint);
    pub fn alSourceStopv(n: ALsizei, sources: *mut ALuint);
    pub fn alSourceRewindv(n: ALsizei, sources: *mut ALuint);

    /** Activate a source, start replay. */
    pub fn alSourcePlay(source: ALuint);

    /**
     * Pause a source,
     *  temporarily remove it from the mixer list.
     */
    pub fn alSourcePause(source: ALuint);

    /**
     * Stop a source,
     *  temporarily remove it from the mixer list,
     *  and reset its internal state to pre-Play.
     * To remove a Source completely, it has to be
     *  deleted following Stop, or before Play.
     */
    pub fn alSourceStop(source: ALuint);

    /**
     * Rewinds a source,
     *  temporarily remove it from the mixer list,
     *  and reset its internal state to pre-Play.
     */
    pub fn alSourceRewind(source: ALuint);

    /**
     * BUFFER
     * Buffer objects are storage space for sample data.
     * Buffers are referred to by Sources. There can be more than
     *  one Source using the same Buffer data. If Buffers have
     *  to be duplicated on a per-Source basis, the driver has to
     *  take care of allocation, copying, and deallocation as well
     *  as propagating buffer data changes.
     */

    /** Buffer object generation. */
    pub fn alGenBuffers(n: ALsizei, buffers: *mut ALuint);
    pub fn alDeleteBuffers(n: ALsizei, buffers: *mut ALuint);
    pub fn alIsBuffer(buffer: ALuint) -> ALboolean;

    /**
     * Specify the data to be filled into a buffer.
     */
    pub fn alBufferData(buffer: ALuint, format: ALenum, data: *mut c_void, size: ALsizei, freq: ALsizei);

    pub fn alGetBufferi(buffer: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetBufferf(buffer: ALuint, param: ALenum, value: *mut ALfloat);

    /**
     * Queue stuff
     */
    pub fn alSourceQueueBuffers(source: ALuint, n: ALsizei, buffers: *mut ALuint);
    pub fn alSourceUnqueueBuffers(source: ALuint, n: ALsizei, buffers: *mut ALuint);

    /**
     * Knobs and dials
     */
    pub fn alDistanceModel(value: ALenum);
    pub fn alDopplerFactor(value: ALfloat);
    pub fn alDopplerVelocity(value: ALfloat);
}

#[cfg(target_os = "xbox360")]
extern "C" {
    /**
     * OpenAL Maintenance Functions
     * Initialization and exiting.
     * State Management and Query.
     * Error Handling.
     * Extension Support.
     */

    /** State management. */
    pub fn alEnable(capability: ALenum);
    pub fn alDisable(capability: ALenum);
    pub fn alIsEnabled(capability: ALenum) -> ALboolean;

    /** Application preferences for driver performance choices. */
    pub fn alHint(target: ALenum, mode: ALenum);

    /** State retrieval. */
    pub fn alGetBoolean(param: ALenum) -> ALboolean;
    pub fn alGetInteger(param: ALenum) -> ALint;
    pub fn alGetFloat(param: ALenum) -> ALfloat;
    pub fn alGetDouble(param: ALenum) -> ALdouble;
    pub fn alGetBooleanv(param: ALenum, data: *mut ALboolean);
    pub fn alGetIntegerv(param: ALenum, data: *mut ALint);
    pub fn alGetFloatv(param: ALenum, data: *mut ALfloat);
    pub fn alGetDoublev(param: ALenum, data: *mut ALdouble);
    pub fn alGetString(param: ALenum) -> *mut ALubyte;

    /**
     * Error support.
     * Obtain the most recent error generated in the AL state machine.
     */
    pub fn alGetError() -> ALenum;

    /**
     * Update cycle.
     */
    pub fn alUpdate();

    /**
     * Returns a global state parameter.
     */
    pub fn alGeti(param: ALenum, value: *mut ALint);

    /**
     * Adjust the size of the sound buffer pool.
     */
    pub fn alResizePool(size: ALuint);

    /**
     * Extension support.
     * Obtain the address of a function (usually an extension)
     *  with the name fname. All addresses are context-independent.
     */
    pub fn alIsExtensionPresent(fname: *mut ALubyte) -> ALboolean;

    /**
     * Extension support.
     * Obtain the address of a function (usually an extension)
     *  with the name fname. All addresses are context-independent.
     */
    pub fn alGetProcAddress(fname: *mut ALubyte) -> *mut c_void;

    /**
     * Extension support.
     * Obtain the integer value of an enumeration (usually an extension) with the name ename.
     */
    pub fn alGetEnumValue(ename: *mut ALubyte) -> ALenum;

    /**
     * LISTENER
     * Listener is the sample position for a given context.
     * The multi-channel (usually stereo) output stream generated
     *  by the mixer is parametrized by this Listener object:
     *  its position and velocity relative to Sources, within
     *  occluder and reflector geometry.
     */

    /**
     * Listener create and delete
     */
    pub fn alGenListeners(n: ALsizei, listeners: *mut ALuint);
    pub fn alDeleteListeners(n: ALsizei, listeners: *mut ALuint);

    // VV's Console version of openAL includes multiple listener support,
    // as an extra first arg to the alListener functions.

    /**
     *
     * Listener Environment:  default 0.
     */
    pub fn alListeneri(listener: ALuint, param: ALenum, value: ALint);

    /**
     *
     * Listener Gain:  default 1.0f.
     */
    pub fn alListenerf(listener: ALuint, param: ALenum, value: ALfloat);

    /**
     *
     * Listener Position.
     * Listener Velocity.
     */
    pub fn alListener3f(listener: ALuint, param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);

    /**
     *
     * Listener Position:        ALfloat[3]
     * Listener Velocity:        ALfloat[3]
     * Listener Orientation:     ALfloat[6]  (forward and up vector).
     */
    pub fn alListenerfv(listener: ALuint, param: ALenum, values: *mut ALfloat);

    pub fn alGetListeneri(listener: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetListenerf(listener: ALuint, param: ALenum, value: *mut ALfloat);
    pub fn alGetListener3f(listener: ALuint, param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
    pub fn alGetListenerfv(listener: ALuint, param: ALenum, values: *mut ALfloat);

    /**
     * SOURCE
     * Source objects are by default localized. Sources
     *  take the PCM data provided in the specified Buffer,
     *  apply Source-specific modifications, and then
     *  submit them to be mixed according to spatial
     *  arrangement etc.
     */

    /** Create Source objects. */
    pub fn alGenSources2D(n: ALsizei, sources: *mut ALuint);
    pub fn alGenSources3D(n: ALsizei, sources: *mut ALuint);

    /** Delete Source objects. */
    pub fn alDeleteSources(n: ALsizei, sources: *mut ALuint);

    /** Verify a handle is a valid Source. */
    pub fn alIsSource(id: ALuint) -> ALboolean;

    /** Set an integer parameter for a Source object. */
    pub fn alSourcei(source: ALuint, param: ALenum, value: ALint);
    pub fn alSourcef(source: ALuint, param: ALenum, value: ALfloat);
    pub fn alSource3f(source: ALuint, param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);
    pub fn alSourcefv(source: ALuint, param: ALenum, values: *mut ALfloat);

    /** Get an integer parameter for a Source object. */
    pub fn alGetSourcei(source: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetSourcef(source: ALuint, param: ALenum, value: *mut ALfloat);
    pub fn alGetSource3f(source: ALuint, param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
    pub fn alGetSourcefv(source: ALuint, param: ALenum, values: *mut ALfloat);

    pub fn alSourcePlayv(n: ALsizei, sources: *mut ALuint);
    pub fn alSourcePausev(n: ALsizei, sources: *mut ALuint);
    pub fn alSourceStopv(n: ALsizei, sources: *mut ALuint);
    pub fn alSourceRewindv(n: ALsizei, sources: *mut ALuint);

    /** Activate a source, start replay. */
    pub fn alSourcePlay(source: ALuint);

    /**
     * Pause a source,
     *  temporarily remove it from the mixer list.
     */
    pub fn alSourcePause(source: ALuint);

    /**
     * Stop a source,
     *  temporarily remove it from the mixer list,
     *  and reset its internal state to pre-Play.
     * To remove a Source completely, it has to be
     *  deleted following Stop, or before Play.
     */
    pub fn alSourceStop(source: ALuint);

    /**
     * Rewinds a source,
     *  temporarily remove it from the mixer list,
     *  and reset its internal state to pre-Play.
     */
    pub fn alSourceRewind(source: ALuint);

    /**
     * BUFFER
     * Buffer objects are storage space for sample data.
     * Buffers are referred to by Sources. There can be more than
     *  one Source using the same Buffer data. If Buffers have
     *  to be duplicated on a per-Source basis, the driver has to
     *  take care of allocation, copying, and deallocation as well
     *  as propagating buffer data changes.
     */

    /** Buffer object generation. */
    pub fn alGenBuffers(n: ALsizei, buffers: *mut ALuint);
    pub fn alDeleteBuffers(n: ALsizei, buffers: *mut ALuint);
    pub fn alIsBuffer(buffer: ALuint) -> ALboolean;

    /**
     * Specify the data to be filled into a buffer.
     */
    pub fn alBufferData(buffer: ALuint, format: ALenum, data: *mut c_void, size: ALsizei, freq: ALsizei);

    pub fn alGetBufferi(buffer: ALuint, param: ALenum, value: *mut ALint);
    pub fn alGetBufferf(buffer: ALuint, param: ALenum, value: *mut ALfloat);

    /**
     * Queue stuff
     */
    pub fn alSourceQueueBuffers(source: ALuint, n: ALsizei, buffers: *mut ALuint);
    pub fn alSourceUnqueueBuffers(source: ALuint, n: ALsizei, buffers: *mut ALuint);

    /**
     * STREAM
     * Stream objects encapsulate traditional sound stream and
     *  act as both sources and buffers.  They emit sound and
     *  manage sound data.
     */
    pub fn alGenStream();
    pub fn alDeleteStream();

    pub fn alStreamStop();
    pub fn alStreamPlay(offset: ALsizei, file: ALint, r#loop: ALint);

    pub fn alStreamf(param: ALenum, value: ALfloat);

    pub fn alGetStreamf(param: ALenum, value: *mut ALfloat);
    pub fn alGetStreami(param: ALenum, value: *mut ALint);

    /**
     * Knobs and dials
     */
    pub fn alDistanceModel(value: ALenum);
    pub fn alDopplerFactor(value: ALfloat);
    pub fn alDopplerVelocity(value: ALfloat);
    pub fn alGain(value: ALfloat);
}

// Function pointer types and global variables for dynamic loading (AL_NO_PROTOTYPES)
pub type alEnable_fn = unsafe extern "C" fn(capability: ALenum);
pub type alDisable_fn = unsafe extern "C" fn(capability: ALenum);
pub type alIsEnabled_fn = unsafe extern "C" fn(capability: ALenum) -> ALboolean;
pub type alHint_fn = unsafe extern "C" fn(target: ALenum, mode: ALenum);
pub type alGetBoolean_fn = unsafe extern "C" fn(param: ALenum) -> ALboolean;
pub type alGetInteger_fn = unsafe extern "C" fn(param: ALenum) -> ALint;
pub type alGetFloat_fn = unsafe extern "C" fn(param: ALenum) -> ALfloat;
pub type alGetDouble_fn = unsafe extern "C" fn(param: ALenum) -> ALdouble;
pub type alGetBooleanv_fn = unsafe extern "C" fn(param: ALenum, data: *mut ALboolean);
pub type alGetIntegerv_fn = unsafe extern "C" fn(param: ALenum, data: *mut ALint);
pub type alGetFloatv_fn = unsafe extern "C" fn(param: ALenum, data: *mut ALfloat);
pub type alGetDoublev_fn = unsafe extern "C" fn(param: ALenum, data: *mut ALdouble);
pub type alGetString_fn = unsafe extern "C" fn(param: ALenum) -> *mut ALubyte;
pub type alGetError_fn = unsafe extern "C" fn() -> ALenum;
pub type alIsExtensionPresent_fn = unsafe extern "C" fn(fname: *mut ALubyte) -> ALboolean;
pub type alGetProcAddress_fn = unsafe extern "C" fn(fname: *mut ALubyte) -> *mut c_void;
pub type alGetEnumValue_fn = unsafe extern "C" fn(ename: *mut ALubyte) -> ALenum;
pub type alListeneri_fn = unsafe extern "C" fn(param: ALenum, value: ALint);
pub type alListenerf_fn = unsafe extern "C" fn(param: ALenum, value: ALfloat);
pub type alListener3f_fn = unsafe extern "C" fn(param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);
pub type alListenerfv_fn = unsafe extern "C" fn(param: ALenum, values: *mut ALfloat);
pub type alGetListeneri_fn = unsafe extern "C" fn(param: ALenum, value: *mut ALint);
pub type alGetListenerf_fn = unsafe extern "C" fn(param: ALenum, value: *mut ALfloat);
pub type alGetListener3f_fn = unsafe extern "C" fn(param: ALenum, v1: *mut ALfloat, v2: *mut ALfloat, v3: *mut ALfloat);
pub type alGetListenerfv_fn = unsafe extern "C" fn(param: ALenum, values: *mut ALfloat);
pub type alGenSources_fn = unsafe extern "C" fn(n: ALsizei, sources: *mut ALuint);
pub type alDeleteSources_fn = unsafe extern "C" fn(n: ALsizei, sources: *mut ALuint);
pub type alIsSource_fn = unsafe extern "C" fn(id: ALuint) -> ALboolean;
pub type alSourcei_fn = unsafe extern "C" fn(source: ALuint, param: ALenum, value: ALint);
pub type alSourcef_fn = unsafe extern "C" fn(source: ALuint, param: ALenum, value: ALfloat);
pub type alSource3f_fn = unsafe extern "C" fn(source: ALuint, param: ALenum, v1: ALfloat, v2: ALfloat, v3: ALfloat);
pub type alSourcefv_fn = unsafe extern "C" fn(source: ALuint, param: ALenum, values: *mut ALfloat);
pub type alGetSourcei_fn = unsafe extern "C" fn(source: ALuint, param: ALenum, value: *mut ALint);
pub type alGetSourcef_fn = unsafe extern "C" fn(source: ALuint, param: ALenum, value: *mut ALfloat);
pub type alGetSourcefv_fn = unsafe extern "C" fn(source: ALuint, param: ALenum, values: *mut ALfloat);
pub type alSourcePlayv_fn = unsafe extern "C" fn(n: ALsizei, sources: *mut ALuint);
pub type alSourceStopv_fn = unsafe extern "C" fn(n: ALsizei, sources: *mut ALuint);
pub type alSourcePlay_fn = unsafe extern "C" fn(source: ALuint);
pub type alSourcePause_fn = unsafe extern "C" fn(source: ALuint);
pub type alSourceStop_fn = unsafe extern "C" fn(source: ALuint);
pub type alGenBuffers_fn = unsafe extern "C" fn(n: ALsizei, buffers: *mut ALuint);
pub type alDeleteBuffers_fn = unsafe extern "C" fn(n: ALsizei, buffers: *mut ALuint);
pub type alIsBuffer_fn = unsafe extern "C" fn(buffer: ALuint) -> ALboolean;
pub type alBufferData_fn = unsafe extern "C" fn(buffer: ALuint, format: ALenum, data: *mut c_void, size: ALsizei, freq: ALsizei);
pub type alGetBufferi_fn = unsafe extern "C" fn(buffer: ALuint, param: ALenum, value: *mut ALint);
pub type alGetBufferf_fn = unsafe extern "C" fn(buffer: ALuint, param: ALenum, value: *mut ALfloat);
pub type alSourceQueueBuffers_fn = unsafe extern "C" fn(source: ALuint, n: ALsizei, buffers: *mut ALuint);
pub type alSourceUnqueueBuffers_fn = unsafe extern "C" fn(source: ALuint, n: ALsizei, buffers: *mut ALuint);
pub type alDistanceModel_fn = unsafe extern "C" fn(value: ALenum);
pub type alDopplerFactor_fn = unsafe extern "C" fn(value: ALfloat);
pub type alDopplerVelocity_fn = unsafe extern "C" fn(value: ALfloat);

// Global function pointer variables for dynamic loading
pub static mut alEnable: Option<alEnable_fn> = None;
pub static mut alDisable: Option<alDisable_fn> = None;
pub static mut alIsEnabled: Option<alIsEnabled_fn> = None;
pub static mut alHint: Option<alHint_fn> = None;
pub static mut alGetBoolean: Option<alGetBoolean_fn> = None;
pub static mut alGetInteger: Option<alGetInteger_fn> = None;
pub static mut alGetFloat: Option<alGetFloat_fn> = None;
pub static mut alGetDouble: Option<alGetDouble_fn> = None;
pub static mut alGetBooleanv: Option<alGetBooleanv_fn> = None;
pub static mut alGetIntegerv: Option<alGetIntegerv_fn> = None;
pub static mut alGetFloatv: Option<alGetFloatv_fn> = None;
pub static mut alGetDoublev: Option<alGetDoublev_fn> = None;
pub static mut alGetString: Option<alGetString_fn> = None;
pub static mut alGetError: Option<alGetError_fn> = None;
pub static mut alIsExtensionPresent: Option<alIsExtensionPresent_fn> = None;
pub static mut alGetProcAddress: Option<alGetProcAddress_fn> = None;
pub static mut alGetEnumValue: Option<alGetEnumValue_fn> = None;
pub static mut alListeneri: Option<alListeneri_fn> = None;
pub static mut alListenerf: Option<alListenerf_fn> = None;
pub static mut alListener3f: Option<alListener3f_fn> = None;
pub static mut alListenerfv: Option<alListenerfv_fn> = None;
pub static mut alGetListeneri: Option<alGetListeneri_fn> = None;
pub static mut alGetListenerf: Option<alGetListenerf_fn> = None;
pub static mut alGetListener3f: Option<alGetListener3f_fn> = None;
pub static mut alGetListenerfv: Option<alGetListenerfv_fn> = None;
pub static mut alGenSources: Option<alGenSources_fn> = None;
pub static mut alDeleteSources: Option<alDeleteSources_fn> = None;
pub static mut alIsSource: Option<alIsSource_fn> = None;
pub static mut alSourcei: Option<alSourcei_fn> = None;
pub static mut alSourcef: Option<alSourcef_fn> = None;
pub static mut alSource3f: Option<alSource3f_fn> = None;
pub static mut alSourcefv: Option<alSourcefv_fn> = None;
pub static mut alGetSourcei: Option<alGetSourcei_fn> = None;
pub static mut alGetSourcef: Option<alGetSourcef_fn> = None;
pub static mut alGetSourcefv: Option<alGetSourcefv_fn> = None;
pub static mut alSourcePlayv: Option<alSourcePlayv_fn> = None;
pub static mut alSourceStopv: Option<alSourceStopv_fn> = None;
pub static mut alSourcePlay: Option<alSourcePlay_fn> = None;
pub static mut alSourcePause: Option<alSourcePause_fn> = None;
pub static mut alSourceStop: Option<alSourceStop_fn> = None;
pub static mut alGenBuffers: Option<alGenBuffers_fn> = None;
pub static mut alDeleteBuffers: Option<alDeleteBuffers_fn> = None;
pub static mut alIsBuffer: Option<alIsBuffer_fn> = None;
pub static mut alBufferData: Option<alBufferData_fn> = None;
pub static mut alGetBufferi: Option<alGetBufferi_fn> = None;
pub static mut alGetBufferf: Option<alGetBufferf_fn> = None;
pub static mut alSourceQueueBuffers: Option<alSourceQueueBuffers_fn> = None;
pub static mut alSourceUnqueueBuffers: Option<alSourceUnqueueBuffers_fn> = None;
pub static mut alDistanceModel: Option<alDistanceModel_fn> = None;
pub static mut alDopplerFactor: Option<alDopplerFactor_fn> = None;
pub static mut alDopplerVelocity: Option<alDopplerVelocity_fn> = None;
