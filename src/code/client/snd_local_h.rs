//! Mechanical port of `code/client/snd_local.h`.
//!
//! Private sound definitions.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_uint, c_uchar};

// Imported types from shared headers.
use crate::codemp::game::q_shared_h::{byte, qboolean, vec3_t};
use crate::codemp::mp3code::mp3struct_h::MP3STREAM;

// Opaque C/C++ dependencies that are only ever passed by reference.
/// Forward reference to `cvar_t` from qcommon.h (only used as pointer).
#[repr(C)]
pub struct cvar_t {
    _unused: [u8; 0],
}

// OpenAL types (opaque, only used as handles/identifiers).
pub type ALuint = c_uint;

// soundChannel_t is actually just an int enum.
pub type soundChannel_t = c_int;

// ============================================================================
// Constants
// ============================================================================

#[allow(non_upper_case_globals)]
pub const PAINTBUFFER_SIZE: c_int = 1024;

#[allow(non_upper_case_globals)]
pub const MAX_CHANNELS: c_int = 32;

#[allow(non_upper_case_globals)]
pub const MAX_RAW_SAMPLES: c_int = 16384;

#[allow(non_upper_case_globals)]
pub const START_SAMPLE_IMMEDIATE: c_int = 0x7fffffff;

#[allow(non_upper_case_globals)]
pub const NUM_STREAMING_BUFFERS: c_int = 4;

#[allow(non_upper_case_globals)]
pub const STREAMING_BUFFER_SIZE: c_int = 4608; // 4 decoded MP3 frames

#[allow(non_upper_case_globals)]
pub const QUEUED: c_int = 1;

#[allow(non_upper_case_globals)]
pub const UNQUEUED: c_int = 2;

#[allow(non_upper_case_globals)]
pub const WAV_FORMAT_PCM: c_int = 1;

#[allow(non_upper_case_globals)]
pub const WAV_FORMAT_ADPCM: c_int = 2; // not actually implemented, but is the value that you get in a header

#[allow(non_upper_case_globals)]
pub const WAV_FORMAT_MP3: c_int = 3; // not actually used this way, but just ensures we don't match one of the legit formats

// ============================================================================
// Structs and Types
// ============================================================================

// !!! if this is changed, the asm code must change !!!
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct portable_samplepair_t {
    pub left: c_int,  // the final values will be clamped to +/- 0x00ffff00 and shifted down
    pub right: c_int,
}

// keep this enum in sync with the table "sSoundCompressionMethodStrings" -ste
//
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundCompressionMethod_t {
    ct_16 = 0, // formerly ct_NONE in EF1, now indicates 16-bit samples (the default)
    ct_MP3 = 1,
    //
    ct_NUMBEROF = 2, // used only for array sizing
}

#[repr(C)]
pub struct sfx_s {
    pub pSoundData: *mut i16,
    pub bDefaultSound: bool,                   // couldn't be loaded, so use buzz
    pub bInMemory: bool,                       // not in Memory, set qtrue when loaded, and qfalse when its buffers are freed up because of being old, so can be reloaded
    pub iLastLevelUsedOn: i16,                 // used for cacheing purposes
    pub eSoundCompressionMethod: SoundCompressionMethod_t,
    pub pMP3StreamHeader: *mut MP3STREAM,      // NULL ptr unless this sfx_t is an MP3. Use Z_Malloc and Z_Free
    pub iSoundLengthInSamples: c_int,          // length in samples, always kept as 16bit now so this is #shorts (watch for stereo later for music?)
    pub sSoundName: [c_char; 64],              // MAX_QPATH = 64
    pub iLastTimeUsed: c_int,
    pub fVolRange: f32, // used to set the highest volume this sample has at load time - used for lipsynching

    // Open AL
    pub Buffer: ALuint,
    pub lipSyncData: *mut c_char,

    pub next: *mut sfx_s, // only used because of hash table when registering
}

pub type sfx_t = sfx_s;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct dma_t {
    pub channels: c_int,
    pub samples: c_int,          // mono samples in buffer
    pub submission_chunk: c_int,  // don't mix less than this #
    pub samplebits: c_int,
    pub speed: c_int,
    pub buffer: *mut byte,
}

// Open AL specific
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct STREAMINGBUFFER {
    pub BufferID: ALuint,
    pub Status: ALuint,
    pub Data: *mut c_char,
}

#[repr(C)]
pub struct channel_t {
    // back-indented fields new in TA codebase, will re-format when MP3 code finished -ste
    // note: field missing in TA: qboolean	loopSound;		// from an S_AddLoopSound call, cleared each frame
    //
    pub startSample: c_int, // START_SAMPLE_IMMEDIATE = set immediately on next mix
    pub entnum: c_int,      // to allow overriding a specific sound
    pub entchannel: soundChannel_t, // to allow overriding a specific sound
    pub leftvol: c_int,     // 0-255 volume after spatialization
    pub rightvol: c_int,    // 0-255 volume after spatialization
    pub master_vol: c_int,  // 0-255 volume before spatialization

    pub origin: vec3_t, // only use if fixed_origin is set

    pub fixed_origin: qboolean, // use origin instead of fetching entnum's origin
    pub thesfx: *mut sfx_t, // sfx structure
    pub loopSound: qboolean, // from an S_AddLoopSound call, cleared each frame
    //
    pub MP3StreamHeader: MP3STREAM,
    pub MP3SlidingDecodeBuffer: [byte; 50000], // 12000 - typical back-request = -3072, so roughly double is 6000 (safety), then doubled again so the 6K pos is in the middle of the buffer)
    pub iMP3SlidingDecodeWritePos: c_int,
    pub iMP3SlidingDecodeWindowPos: c_int,

    // Open AL specific
    pub bLooping: bool, // Signifies if this channel / source is playing a looping sound
    //	pub bAmbient: bool,	// Signifies if this channel / source is playing a looping ambient sound
    pub bProcessed: bool, // Signifies if this channel / source has been processed
    pub bStreaming: bool, // Set to true if the data needs to be streamed (MP3 or dialogue)
    pub buffers: [STREAMINGBUFFER; 4], // AL Buffers for streaming (NUM_STREAMING_BUFFERS)
    pub alSource: ALuint, // Open AL Source
    pub bPlaying: bool,   // Set to true when a sound is playing on this channel / source
    pub iStartTime: c_int, // Time playback of Source begins
    pub lSlotID: c_int,    // ID of Slot rendering Source's environment (enables a send to this FXSlot)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct wavinfo_t {
    pub format: c_int,
    pub rate: c_int,
    pub width: c_int,
    pub channels: c_int,
    pub samples: c_int,
    pub dataofs: c_int, // chunk starts this many bytes from file start
}

// ============================================================================
// SYSTEM SPECIFIC FUNCTIONS
// ============================================================================

// initializes cycling through a DMA buffer and returns information on it
extern "C" {
    pub fn SNDDMA_Init() -> qboolean;

    // gets the current DMA position
    pub fn SNDDMA_GetDMAPos() -> c_int;

    // shutdown the DMA xfer.
    pub fn SNDDMA_Shutdown();

    pub fn SNDDMA_BeginPainting();

    pub fn SNDDMA_Submit();
}

// ============================================================================
// Global declarations
// ============================================================================

extern "C" {
    #[link_name = "s_channels"]
    pub static s_channels: [channel_t; MAX_CHANNELS as usize];

    pub static s_paintedtime: c_int;
    pub static s_rawend: c_int;
    pub static listener_origin: vec3_t;
    pub static listener_forward: vec3_t;
    pub static listener_right: vec3_t;
    pub static listener_up: vec3_t;
    pub static dma: dma_t;

    pub static s_rawsamples: [portable_samplepair_t; MAX_RAW_SAMPLES as usize];
}

/// TA added this, but it just returns the s_rawsamples[] array above. Oh well...
extern "C" {
    pub fn S_GetRawSamplePointer() -> *mut portable_samplepair_t;
}

extern "C" {
    pub static s_volume: *mut cvar_t;
    pub static s_volumeVoice: *mut cvar_t;
    pub static s_nosound: *mut cvar_t;
    pub static s_khz: *mut cvar_t;
    pub static s_allowDynamicMusic: *mut cvar_t;
    pub static s_show: *mut cvar_t;
    pub static s_mixahead: *mut cvar_t;

    pub static s_testsound: *mut cvar_t;
    pub static s_separation: *mut cvar_t;
}

// ============================================================================
// Function declarations
// ============================================================================

extern "C" {
    pub fn GetWavinfo(name: *const c_char, wav: *mut byte, wavlength: c_int) -> wavinfo_t;

    pub fn S_LoadSound(sfx: *mut sfx_t) -> qboolean;

    pub fn S_PaintChannels(endtime: c_int);

    // picks a channel based on priorities, empty slots, number of channels
    pub fn S_PickChannel(entnum: c_int, entchannel: c_int) -> *mut channel_t;

    // spatializes a channel
    pub fn S_Spatialize(ch: *mut channel_t);
}

// //////////////////////////////////
// //
// // new stuff from TA codebase

extern "C" {
    pub fn SND_malloc(iSize: c_int, sfx: *mut sfx_t) -> *mut byte;
    pub fn SND_setup();
    pub fn SND_FreeOldestSound(pButNotThisOne: *mut sfx_t) -> c_int;
    pub fn SND_TouchSFX(sfx: *mut sfx_t);

    pub fn S_DisplayFreeMemory();
    pub fn S_memoryLoad(sfx: *mut sfx_t);
}
//
// //////////////////////////////////

// ============================================================================
// OpenAL-specific functions
// ============================================================================

extern "C" {
    // Added for Open AL to know when to mute all sounds (e.g when app. loses focus)
    pub fn S_AL_MuteAllSounds(bMute: qboolean);
}

// //from SND_AMBIENT
extern "C" {
    pub fn AS_Init();
    pub fn AS_Free();
}

// Note: This file includes "cl_mp3.h" at the end in the original, but that is
// an implementation detail that typically doesn't need to be mirrored in the Rust module.
// If needed, define it as a separate module and include it here.
