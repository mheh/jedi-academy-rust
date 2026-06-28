//! Mechanical port of `codemp/client/snd_local.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_short, c_uint};

use crate::codemp::game::q_shared_h::{byte, qboolean, vec3_t, MAX_QPATH};
use crate::codemp::mp3code::mp3struct_h::MP3STREAM;

// Following #define is ONLY for MP JKA code.
// They want to keep qboolean pure enum in that code, so all
// sound code uses sboolean.
pub type sboolean = c_int;

// Includes
// #include "snd_public.h"
// #include "../mp3code/mp3struct.h"
//
// #include "openal/al.h"
// #include "openal/alc.h"
// #include "eax/eax.h"
// #include "eax/eaxman.h"

// Unported OpenAL typedef dependency from `openal/al.h`.
pub type ALuint = c_uint;

// Header-local stub for qcommon.h's cvar_t dependency.
// nothing outside the Cvar_*() functions should modify these fields!
#[repr(C)]
#[derive(Clone, Copy)]
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

// Added for Open AL to know when to mute all sounds (e.g when app. loses focus)
unsafe extern "C" {
    pub fn S_AL_MuteAllSounds(bMute: sboolean);
}

//from SND_AMBIENT
unsafe extern "C" {
    pub fn AS_Init();
    pub fn AS_Free();
}

pub const PAINTBUFFER_SIZE: c_int = 1024;

// !!! if this is changed, the asm code must change !!!
#[repr(C)]
#[derive(Clone, Copy)]
pub struct portable_samplepair_t {
    // the final values will be clamped to +/- 0x00ffff00 and shifted down
    pub left: c_int,
    pub right: c_int,
}

const _: () = assert!(core::mem::size_of::<portable_samplepair_t>() == 8);
const _: () = assert!(core::mem::align_of::<portable_samplepair_t>() == 4);

// keep this enum in sync with the table "sSoundCompressionMethodStrings" -ste
//
pub type SoundCompressionMethod_t = c_int;
pub const ct_16: SoundCompressionMethod_t = 0; // formerly ct_NONE in EF1, now indicates 16-bit samples (the default)
pub const ct_MP3: SoundCompressionMethod_t = 1;
//
pub const ct_NUMBEROF: SoundCompressionMethod_t = 2; // used only for array sizing

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sfx_s {
    pub pSoundData: *mut c_short,
    pub bDefaultSound: sboolean, // couldn't be loaded, so use buzz
    pub bInMemory: sboolean, // not in Memory, set qtrue when loaded, and qfalse when its buffers are freed up because of being old, so can be reloaded
    pub eSoundCompressionMethod: SoundCompressionMethod_t,
    pub pMP3StreamHeader: *mut MP3STREAM, // NULL ptr unless this sfx_t is an MP3. Use Z_Malloc and Z_Free
    pub iSoundLengthInSamples: c_int, // length in samples, always kept as 16bit now so this is #shorts (watch for stereo later for music?)
    pub sSoundName: [c_char; MAX_QPATH],
    pub iLastTimeUsed: c_int,
    pub fVolRange: c_float, // used to set the highest volume this sample has at load time - used for lipsynching
    pub iLastLevelUsedOn: c_int, // used for cacheing purposes

    // Open AL
    pub Buffer: ALuint,
    pub lipSyncData: *mut c_char,

    pub next: *mut sfx_s, // only used because of hash table when registering
}

pub type sfx_t = sfx_s;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct dma_t {
    pub channels: c_int,
    pub samples: c_int, // mono samples in buffer
    pub submission_chunk: c_int, // don't mix less than this #
    pub samplebits: c_int,
    pub speed: c_int,
    pub buffer: *mut byte,
}

pub const START_SAMPLE_IMMEDIATE: c_uint = 0x7fffffff;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct STREAMINGBUFFER {
    pub BufferID: ALuint,
    pub Status: ALuint,
    pub Data: *mut c_char,
}

const _: () = assert!(core::mem::size_of::<STREAMINGBUFFER>() == 16);
const _: () = assert!(core::mem::align_of::<STREAMINGBUFFER>() == 8);

pub const NUM_STREAMING_BUFFERS: usize = 4;
pub const STREAMING_BUFFER_SIZE: usize = 4608; // 4 decoded MP3 frames

pub const QUEUED: c_int = 1;
pub const UNQUEUED: c_int = 2;

pub const WAV_FORMAT_PCM: c_int = 1;
pub const WAV_FORMAT_ADPCM: c_int = 2; // not actually implemented, but is the value that you get in a header
pub const WAV_FORMAT_MP3: c_int = 3; // not actually used this way, but just ensures we don't match one of the legit formats

#[repr(C)]
#[derive(Clone, Copy)]
pub struct channel_t {
    // back-indented fields new in TA codebase, will re-format when MP3 code finished -ste
    // note: field missing in TA: sboolean	loopSound;		// from an S_AddLoopSound call, cleared each frame
    //
    pub startSample: c_uint, // START_SAMPLE_IMMEDIATE = set immediately on next mix
    pub entnum: c_int, // to allow overriding a specific sound
    pub entchannel: c_int, // to allow overriding a specific sound
    pub leftvol: c_int, // 0-255 volume after spatialization
    pub rightvol: c_int, // 0-255 volume after spatialization
    pub master_vol: c_int, // 0-255 volume before spatialization

    pub origin: vec3_t, // only use if fixed_origin is set

    pub fixed_origin: sboolean, // use origin instead of fetching entnum's origin
    pub thesfx: *mut sfx_t, // sfx structure
    pub loopSound: sboolean, // from an S_AddLoopSound call, cleared each frame
    //
    pub MP3StreamHeader: MP3STREAM,
    pub MP3SlidingDecodeBuffer: [byte; 50000 /*12000*/], // typical back-request = -3072, so roughly double is 6000 (safety), then doubled again so the 6K pos is in the middle of the buffer)
    pub iMP3SlidingDecodeWritePos: c_int,
    pub iMP3SlidingDecodeWindowPos: c_int,

    // Open AL specific
    pub bLooping: bool, // Signifies if this channel / source is playing a looping sound
    //	pub bAmbient;	// Signifies if this channel / source is playing a looping ambient sound
    pub bProcessed: bool, // Signifies if this channel / source has been processed
    pub bStreaming: bool, // Set to true if the data needs to be streamed (MP3 or dialogue)
    pub buffers: [STREAMINGBUFFER; NUM_STREAMING_BUFFERS], // AL Buffers for streaming
    pub alSource: ALuint, // Open AL Source
    pub bPlaying: bool, // Set to true when a sound is playing on this channel / source
    pub iStartTime: c_int, // Time playback of Source begins
    pub lSlotID: c_int, // ID of Slot rendering Source's environment (enables a send to this FXSlot)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct wavinfo_t {
    pub format: c_int,
    pub rate: c_int,
    pub width: c_int,
    pub channels: c_int,
    pub samples: c_int,
    pub dataofs: c_int, // chunk starts this many bytes from file start
}

const _: () = assert!(core::mem::size_of::<wavinfo_t>() == 24);
const _: () = assert!(core::mem::align_of::<wavinfo_t>() == 4);

/*
====================================================================

  SYSTEM SPECIFIC FUNCTIONS

====================================================================
*/

// initializes cycling through a DMA buffer and returns information on it
unsafe extern "C" {
    pub fn SNDDMA_Init() -> qboolean;

    // gets the current DMA position
    pub fn SNDDMA_GetDMAPos() -> c_int;

    // shutdown the DMA xfer.
    pub fn SNDDMA_Shutdown();

    pub fn SNDDMA_BeginPainting();

    pub fn SNDDMA_Submit();
}

//====================================================================

pub const MAX_CHANNELS: usize = 32;

unsafe extern "C" {
    pub static mut s_channels: [channel_t; MAX_CHANNELS];

    pub static mut s_paintedtime: c_int;
    pub static mut s_rawend: c_int;
    pub static mut listener_origin: vec3_t;
    pub static mut listener_forward: vec3_t;
    pub static mut listener_right: vec3_t;
    pub static mut listener_up: vec3_t;
    pub static mut dma: dma_t;
}

pub const MAX_RAW_SAMPLES: usize = 16384;

unsafe extern "C" {
    pub static mut s_rawsamples: [portable_samplepair_t; MAX_RAW_SAMPLES];
    pub fn S_GetRawSamplePointer() -> *mut portable_samplepair_t;
}

unsafe extern "C" {
    pub static mut s_volume: *mut cvar_t;
    pub static mut s_volumeVoice: *mut cvar_t;
    pub static mut s_nosound: *mut cvar_t;
    pub static mut s_khz: *mut cvar_t;
    pub static mut s_allowDynamicMusic: *mut cvar_t;
    pub static mut s_show: *mut cvar_t;
    pub static mut s_mixahead: *mut cvar_t;

    pub static mut s_testsound: *mut cvar_t;
    pub static mut s_separation: *mut cvar_t;
}

unsafe extern "C" {
    pub fn GetWavinfo(name: *const c_char, wav: *mut byte, wavlength: c_int) -> wavinfo_t;

    pub fn S_LoadSound(sfx: *mut sfx_t) -> sboolean;

    pub fn S_PaintChannels(endtime: c_int);

    // picks a channel based on priorities, empty slots, number of channels
    pub fn S_PickChannel(entnum: c_int, entchannel: c_int) -> *mut channel_t;

    // spatializes a channel
    pub fn S_Spatialize(ch: *mut channel_t);

    //////////////////////////////////
    //
    // new stuff from TA codebase

    pub fn SND_malloc(iSize: c_int, sfx: *mut sfx_t) -> *mut byte;
    pub fn SND_setup();
    pub fn SND_FreeOldestSound(pButNotThisOne: *mut sfx_t) -> c_int;
    pub fn SND_TouchSFX(sfx: *mut sfx_t);

    pub fn S_DisplayFreeMemory();
    pub fn S_memoryLoad(sfx: *mut sfx_t);
    //
    //////////////////////////////////
}
