//! Mechanical port of `codemp/client/snd_mp3.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_short, c_uint, c_void};

use crate::codemp::game::q_shared_h::byte;
use crate::codemp::mp3code::mp3struct_h::LP_MP3STREAM;

// Unported sound-local typedef dependency: `snd_local.h` has `#define sboolean int`.
pub type sboolean = c_int;

// Unported `snd_local.h` struct dependencies. `snd_mp3.h` only passes these by pointer.
#[repr(C)]
pub struct sfx_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct channel_t {
    _opaque: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    pub title: [c_char; 30],   // <file basename>
    pub artist: [c_char; 30],  // "Raven Software"
    pub album: [c_char; 30],   // "#UNCOMP %d"		// needed
    pub year: [c_char; 4],     // "2000"
    pub comment: [c_char; 28], // "#MAXVOL %g"		// needed
    pub zero: c_char,
    pub track: c_char,
    pub genre: c_char,
} // 128 bytes in size

const _: () = assert!(core::mem::size_of::<id3v1_1>() == 128);
const _: () = assert!(core::mem::align_of::<id3v1_1>() == 1);
const _: () = assert!(core::mem::offset_of!(id3v1_1, title) == 3);
const _: () = assert!(core::mem::offset_of!(id3v1_1, artist) == 33);
const _: () = assert!(core::mem::offset_of!(id3v1_1, album) == 63);
const _: () = assert!(core::mem::offset_of!(id3v1_1, year) == 93);
const _: () = assert!(core::mem::offset_of!(id3v1_1, comment) == 97);
const _: () = assert!(core::mem::offset_of!(id3v1_1, zero) == 125);
const _: () = assert!(core::mem::offset_of!(id3v1_1, track) == 126);
const _: () = assert!(core::mem::offset_of!(id3v1_1, genre) == 127);

unsafe extern "C" {
    // C `extern const char name[]`; Rust FFI represents this as the first element.
    pub static sKEY_MAXVOL: c_char;
    pub static sKEY_UNCOMP: c_char;
}

// (so far, all these functions are only called from one place in snd_mem.cpp)
//
// (filenames are used purely for error reporting, all files should already be loaded before you get here)
//
unsafe extern "C" {
    pub fn MP3_InitCvars();

    // C++ default argument `bStereoDesired = qfalse` is not represented in Rust FFI.
    pub fn MP3_IsValid(
        psLocalFilename: *const c_char,
        pvData: *mut c_void,
        iDataLen: c_int,
        bStereoDesired: sboolean,
    ) -> sboolean;

    // C++ default arguments (`qbIgnoreID3Tag = qfalse`, `bStereoDesired = qfalse`)
    // are not represented in Rust FFI.
    pub fn MP3_GetUnpackedSize(
        psLocalFilename: *const c_char,
        pvData: *mut c_void,
        iDataLen: c_int,
        qbIgnoreID3Tag: sboolean,
        bStereoDesired: sboolean,
    ) -> c_int;

    // C++ default argument `bStereoDesired = qfalse` is not represented in Rust FFI.
    pub fn MP3_UnpackRawPCM(
        psLocalFilename: *const c_char,
        pvData: *mut c_void,
        iDataLen: c_int,
        pbUnpackBuffer: *mut byte,
        bStereoDesired: sboolean,
    ) -> sboolean;

    // C++ default argument `bStereoDesired = qfalse` is not represented in Rust FFI.
    pub fn MP3Stream_InitPlayingTimeFields(
        lpMP3Stream: LP_MP3STREAM,
        psLocalFilename: *const c_char,
        pvData: *mut c_void,
        iDataLen: c_int,
        bStereoDesired: sboolean,
    ) -> sboolean;

    pub fn MP3Stream_GetPlayingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> c_float;
    pub fn MP3Stream_GetRemainingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> c_float;

    // C++ reference parameters (`int &...`) are represented as mutable pointers.
    // C++ default argument `bStereoDesired = qfalse` is not represented in Rust FFI.
    pub fn MP3_FakeUpWAVInfo(
        psLocalFilename: *const c_char,
        pvData: *mut c_void,
        iDataLen: c_int,
        iUnpackedDataLength: c_int,
        format: *mut c_int,
        rate: *mut c_int,
        width: *mut c_int,
        channels: *mut c_int,
        samples: *mut c_int,
        dataofs: *mut c_int,
        bStereoDesired: sboolean,
    ) -> sboolean;

    // C++ default arguments (`ppTAG = NULL`, `piUncompressedSize = NULL`,
    // `pfMaxVol = NULL`) are not represented in Rust FFI.
    pub fn MP3_ReadSpecialTagInfo(
        pbLoadedFile: *mut byte,
        iLoadedFileLen: c_int,
        ppTAG: *mut *mut id3v1_1,
        piUncompressedSize: *mut c_int,
        pfMaxVol: *mut c_float,
    ) -> sboolean;

    // C++ default argument `bStereoDesired = qfalse` is not represented in Rust FFI.
    pub fn MP3Stream_InitFromFile(
        sfx: *mut sfx_t,
        pbSrcData: *mut byte,
        iSrcDatalen: c_int,
        psSrcDataFilename: *const c_char,
        iMP3UnPackedSize: c_int,
        bStereoDesired: sboolean,
    ) -> sboolean;

    pub fn MP3Stream_Decode(lpMP3Stream: LP_MP3STREAM, bDoingMusic: sboolean) -> c_int;
    pub fn MP3Stream_SeekTo(ch: *mut channel_t, fTimeToSeekTo: c_float) -> sboolean;
    pub fn MP3Stream_Rewind(ch: *mut channel_t) -> sboolean;
    pub fn MP3Stream_GetSamples(
        ch: *mut channel_t,
        startingSampleNum: c_int,
        count: c_int,
        buf: *mut c_short,
        bStereo: sboolean,
    ) -> sboolean;
}

///////////////////////////////////////
//
// the real worker code deep down in the MP3 C code...  (now externalised here so the music streamer can access one)
//
unsafe extern "C" {
    pub fn C_MP3_IsValid(pvData: *mut c_void, iDataLen: c_int, bStereoDesired: c_int)
        -> *mut c_char;
    pub fn C_MP3_GetUnpackedSize(
        pvData: *mut c_void,
        iDataLen: c_int,
        piUnpackedSize: *mut c_int,
        bStereoDesired: c_int,
    ) -> *mut c_char;
    pub fn C_MP3_UnpackRawPCM(
        pvData: *mut c_void,
        iDataLen: c_int,
        piUnpackedSize: *mut c_int,
        pbUnpackBuffer: *mut c_void,
        bStereoDesired: c_int,
    ) -> *mut c_char;
    pub fn C_MP3_GetHeaderData(
        pvData: *mut c_void,
        iDataLen: c_int,
        piRate: *mut c_int,
        piWidth: *mut c_int,
        piChannels: *mut c_int,
        bStereoDesired: c_int,
    ) -> *mut c_char;
    pub fn C_MP3Stream_DecodeInit(
        pSFX_MP3Stream: LP_MP3STREAM,
        pvSourceData: *mut c_void,
        iSourceBytesRemaining: c_int,
        iGameAudioSampleRate: c_int,
        iGameAudioSampleBits: c_int,
        bStereoDesired: c_int,
    ) -> *mut c_char;
    pub fn C_MP3Stream_Decode(pSFX_MP3Stream: LP_MP3STREAM, bFastForwarding: c_int) -> c_uint;
    pub fn C_MP3Stream_Rewind(pSFX_MP3Stream: LP_MP3STREAM) -> *mut c_char;
}
//
///////////////////////////////////////

///////////////// eof /////////////////////
