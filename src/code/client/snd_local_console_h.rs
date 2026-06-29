// snd_local.h -- private sound definations

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint};

// Following #define is ONLY for MP JKA code.
// They want to keep qboolean pure enum in that code, so all
// sound code uses sboolean.
pub type sboolean = c_int;

// STUB: External module includes
// #include "../game/q_shared.h"
// #include "../qcommon/qcommon.h"
// #include "snd_public.h"
// #include "../mp3code/mp3struct.h"
// #include "openal/al.h"
// #include "openal/alc.h"

// STUB: External types from includes
pub type qboolean = c_int;
pub type sfxHandle_t = c_int;
pub type cvar_t = c_void;
pub type ALuint = c_uint;

pub type streamHandle_t = c_int;

// from SND_AMBIENT
extern "C" {
    pub fn AS_Init();
    pub fn AS_Free();
}

#[repr(C)]
pub struct wavinfo_t {
    pub format: c_int,
    pub size: c_int,
    pub width: c_int,
    pub rate: c_int,
    pub samples: c_int,
}

extern "C" {
    pub fn GetWavInfo(data: *mut u8) -> wavinfo_t;
}

pub const SFX_FLAG_UNLOADED: c_int = 1 << 0;
pub const SFX_FLAG_LOADING: c_int = 1 << 1;
pub const SFX_FLAG_RESIDENT: c_int = 1 << 2;
pub const SFX_FLAG_DEFAULT: c_int = 1 << 3;
pub const SFX_FLAG_DEMAND: c_int = 1 << 4;
pub const SFX_FLAG_VOICE: c_int = 1 << 5;

#[repr(C)]
pub struct sfx_s {
    pub iFlags: c_int,
    pub iSoundLength: c_int,            // length in bytes
    pub iLastTimeUsed: c_int,           // last time sound was played in ms
    pub iFileCode: c_int,               // CRC of the file name
    pub iStreamHandle: streamHandle_t,  // handle to the sound file when reading
    pub pSoundData: *mut c_void,        // buffer to hold sound as we are loading it
    pub pLipSyncData: *mut c_char,      // buffer to hold lip sync information on characters
                                        // store the total number of samples in the first 4 bytes
                                        // followed by the actual lipsync data in the remaining bytes
    pub Buffer: ALuint,
    // char*			sSoundName;				// added for debugging
}

#[repr(C)]
pub struct channel_t {
    pub entnum: c_int,          // to allow overriding a specific sound
    pub entchannel: c_int,      // to allow overriding a specific sound
    pub master_vol: c_int,      // 0-255 volume before spatialization
    pub fLastVolume: f32,       // 0-1 last volume sent to AL

    pub origin: [f32; 3],       // sound location
    pub bOriginDirty: bool,     // does the AL position need to be updated

    pub thesfx: *mut sfx_s,     // sfx structure

    pub loopChannel: c_int,     // index into loopSounds (if appropriate)

    pub bPlaying: bool,         // Set to true when a sound is playing on this channel / source
    pub b2D: bool,              // Signifies a 2d sound
    pub bLooping: bool,         // Signifies if this channel / source is playing a looping sound
    pub alSource: ALuint,       // Open AL Source

    pub iLastPlayTime: c_uint,  // Last time a sound was played on this channel
}

extern "C" {
    pub static s_nosound: *mut cvar_t;
    pub static s_allowDynamicMusic: *mut cvar_t;
    pub static s_show: *mut cvar_t;

    pub static s_testsound: *mut cvar_t;
    pub static s_separation: *mut cvar_t;

    pub static s_entityWavVol: *mut c_int;
}

extern "C" {
    pub fn Sys_GetFileCode(sSoundName: *const c_char) -> c_int;
    pub fn S_GetFileCode(sSoundName: *const c_char) -> c_int;

    pub fn S_StartLoadSound(sfx: *mut sfx_s) -> qboolean;
    pub fn S_EndLoadSound(sfx: *mut sfx_s) -> qboolean;

    pub fn S_InitLoad();
    pub fn S_CloseLoad();
    pub fn S_UpdateLoading();

    // New stuff from VV
    pub fn S_LoadSound(sfxHandle: sfxHandle_t);

    // picks a channel based on priorities, empty slots, number of channels
    pub fn S_PickChannel(entnum: c_int, entchannel: c_int) -> *mut channel_t;

    // new stuff from TA codebase
    pub fn SND_update(sfx: *mut sfx_s);
    pub fn SND_setup();
    pub fn SND_FreeOldestSound(pButNotThisOne: *mut sfx_s) -> c_int;
    pub fn SND_TouchSFX(sfx: *mut sfx_s);

    pub fn S_DisplayFreeMemory();
    pub fn S_memoryLoad(sfx: *mut sfx_s);
    pub fn S_PreProcessLipSync(sfx: *mut sfx_s);

    pub fn Sys_StreamIsReading(handle: streamHandle_t) -> bool;
    pub fn Sys_StreamOpen(code: c_int, handle: *mut streamHandle_t) -> c_int;
    pub fn Sys_StreamRead(buffer: *mut c_void, size: c_int, pos: c_int, handle: streamHandle_t) -> bool;
    pub fn Sys_StreamClose(handle: streamHandle_t);
    pub fn Sys_StreamIsError(handle: streamHandle_t) -> bool;
}
