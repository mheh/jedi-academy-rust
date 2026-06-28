//! Mechanical port of `codemp/client/snd_local_console.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_uint, c_void};

use crate::codemp::game::q_shared_h::{byte, vec3_t};
use crate::ffi::types::qboolean;

// Following #define is ONLY for MP JKA code.
// They want to keep qboolean pure enum in that code, so all
// sound code uses sboolean.
pub type sboolean = c_int;

// Includes
// #include "../game/q_shared.h"
// #include "../qcommon/qcommon.h"
// #include "snd_public.h"
// #include "../mp3code/mp3struct.h"
//
// #include "openal/al.h"
// #include "openal/alc.h"

pub type streamHandle_t = c_int;

// Unported OpenAL typedef dependency from `openal/al.h`.
pub type ALuint = c_uint;

// Header-local stub for qcommon.h's cvar_t dependency.
// nothing outside the Cvar_*() functions should modify these fields!
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

//from SND_AMBIENT
unsafe extern "C" {
    pub fn AS_Init();
    pub fn AS_Free();
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct wavinfo_t {
    pub format: c_int,
    pub size: c_int,
    pub width: c_int,
    pub rate: c_int,
}

const _: () = assert!(core::mem::size_of::<wavinfo_t>() == 16);
const _: () = assert!(core::mem::align_of::<wavinfo_t>() == 4);

unsafe extern "C" {
    pub fn GetWavInfo(data: *mut byte) -> wavinfo_t;
}

pub const SFX_FLAG_UNLOADED: c_int = 1 << 0;
pub const SFX_FLAG_LOADING: c_int = 1 << 1;
pub const SFX_FLAG_RESIDENT: c_int = 1 << 2;
pub const SFX_FLAG_DEFAULT: c_int = 1 << 3;
pub const SFX_FLAG_DEMAND: c_int = 1 << 4;
pub const SFX_FLAG_VOICE: c_int = 1 << 5;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sfx_s {
    pub iFlags: c_int,
    pub iSoundLength: c_int,  // length in bytes
    pub iLastTimeUsed: c_int, // last time sound was played in ms
    pub iFileCode: c_int,     // CRC of the file name
    pub iStreamHandle: streamHandle_t, // handle to the sound file when reading
    pub pSoundData: *mut c_void, // buffer to hold sound as we are loading it
    pub Buffer: ALuint,
}

pub type sfx_t = sfx_s;

const _: () = assert!(core::mem::offset_of!(sfx_s, iFlags) == 0);
const _: () = assert!(core::mem::offset_of!(sfx_s, iStreamHandle) == 16);
const _: () = assert!(core::mem::offset_of!(sfx_s, pSoundData) == 24);
const _: () = assert!(core::mem::offset_of!(sfx_s, Buffer) == 32);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct channel_t {
    pub entnum: c_int,     // to allow overriding a specific sound
    pub entchannel: c_int, // to allow overriding a specific sound
    pub master_vol: c_int, // 0-255 volume before spatialization
    pub fLastVolume: c_float, // 0-1 last volume sent to AL

    pub origin: vec3_t,       // sound location
    pub bOriginDirty: bool,   // does the AL position need to be updated

    pub thesfx: *mut sfx_t, // sfx structure

    pub loopChannel: c_int, // index into loopSounds (if appropriate)

    pub bPlaying: bool, // Set to true when a sound is playing on this channel / source
    pub b2D: bool,      // Signifies a 2d sound
    pub bLooping: bool, // Signifies if this channel / source is playing a looping sound
    pub alSource: ALuint, // Open AL Source

    pub iLastPlayTime: c_uint, // Last time a sound was played on this channel
}

const _: () = assert!(core::mem::offset_of!(channel_t, entnum) == 0);
const _: () = assert!(core::mem::offset_of!(channel_t, fLastVolume) == 12);
const _: () = assert!(core::mem::offset_of!(channel_t, origin) == 16);
const _: () = assert!(core::mem::offset_of!(channel_t, bOriginDirty) == 28);
const _: () = assert!(core::mem::offset_of!(channel_t, thesfx) == 32);
const _: () = assert!(core::mem::offset_of!(channel_t, loopChannel) == 40);
const _: () = assert!(core::mem::offset_of!(channel_t, bPlaying) == 44);
const _: () = assert!(core::mem::offset_of!(channel_t, alSource) == 48);
const _: () = assert!(core::mem::offset_of!(channel_t, iLastPlayTime) == 52);

unsafe extern "C" {
    pub static mut s_nosound: *mut cvar_t;
    pub static mut s_allowDynamicMusic: *mut cvar_t;
    pub static mut s_show: *mut cvar_t;

    pub static mut s_testsound: *mut cvar_t;
    pub static mut s_separation: *mut cvar_t;

    pub static mut s_entityWavVol: *mut c_int;
}

unsafe extern "C" {
    pub fn Sys_GetFileCode(sSoundName: *const c_char) -> c_int;
    pub fn S_GetFileCode(sSoundName: *const c_char) -> c_int;

    pub fn S_StartLoadSound(sfx: *mut sfx_t) -> qboolean;
    pub fn S_EndLoadSound(sfx: *mut sfx_t) -> qboolean;

    pub fn S_InitLoad();
    pub fn S_CloseLoad();
    pub fn S_UpdateLoading();

    // New stuff from VV
    pub fn S_LoadSound(sfxHandle: crate::codemp::client::snd_public_h::sfxHandle_t);

    // picks a channel based on priorities, empty slots, number of channels
    pub fn S_PickChannel(entnum: c_int, entchannel: c_int) -> *mut channel_t;

    //////////////////////////////////
    //
    // new stuff from TA codebase

    pub fn SND_update(sfx: *mut sfx_t);
    pub fn SND_setup();
    // C++ default parameter omitted: pButNotThisOne = NULL.
    pub fn SND_FreeOldestSound(pButNotThisOne: *mut sfx_t) -> c_int;
    pub fn SND_TouchSFX(sfx: *mut sfx_t);

    pub fn S_DisplayFreeMemory();
    pub fn S_memoryLoad(sfx: *mut sfx_t);

    pub fn Sys_StreamIsReading(handle: streamHandle_t) -> bool;
    pub fn Sys_StreamOpen(code: c_int, handle: *mut streamHandle_t) -> c_int;
    pub fn Sys_StreamRead(
        buffer: *mut c_void,
        size: c_int,
        pos: c_int,
        handle: streamHandle_t,
    ) -> bool;
    pub fn Sys_StreamClose(handle: streamHandle_t);
    pub fn Sys_StreamIsError(handle: streamHandle_t) -> bool;

    //
    //////////////////////////////////
}
