// Filename:-	cl_mp3.h
//
// (Interface to the rest of the game for the MP3 functions)
//

use core::ffi::{c_int, c_char, c_void, c_uint};

// Opaque type placeholders for types from other modules
// sfx_t is conditionally included from snd_local.h
pub type sfx_t = c_void;
pub type LP_MP3STREAM = *mut c_void;
pub type channel_t = c_void;
pub type qboolean = c_int;
pub type byte = u8;

#[repr(C)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    pub title: [c_char; 30],		// <file basename>
    pub artist: [c_char; 30],	// "Raven Software"
    pub album: [c_char; 30],		// "#UNCOMP %d"		// needed
    pub year: [c_char; 4],		// "2000"
    pub comment: [c_char; 28],	// "#MAXVOL %g"		// needed
    pub zero: c_char,
    pub track: c_char,
    pub genre: c_char,
} // 128 bytes in size

extern "C" {
    pub static sKEY_MAXVOL: c_char;
    pub static sKEY_UNCOMP: c_char;
}

// (so far, all these functions are only called from one place in snd_mem.cpp)
//
// (filenames are used purely for error reporting, all files should already be loaded before you get here)
//
extern "C" {
    pub fn MP3_InitCvars() -> c_void;
    pub fn MP3_IsValid(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, bStereoDesired: qboolean) -> qboolean;
    pub fn MP3_GetUnpackedSize(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, qbIgnoreID3Tag: qboolean, bStereoDesired: qboolean) -> c_int;
    pub fn MP3_UnpackRawPCM(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, pbUnpackBuffer: *mut byte, bStereoDesired: qboolean) -> qboolean;
    pub fn MP3Stream_InitPlayingTimeFields(lpMP3Stream: LP_MP3STREAM, psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, bStereoDesired: qboolean) -> qboolean;
    pub fn MP3Stream_GetPlayingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> f32;
    pub fn MP3Stream_GetRemainingTimeInSeconds(lpMP3Stream: LP_MP3STREAM) -> f32;
    pub fn MP3_FakeUpWAVInfo(psLocalFilename: *const c_char, pvData: *mut c_void, iDataLen: c_int, iUnpackedDataLength: c_int, format: *mut c_int, rate: *mut c_int, width: *mut c_int, channels: *mut c_int, samples: *mut c_int, dataofs: *mut c_int, bStereoDesired: qboolean) -> qboolean;
    pub fn MP3_ReadSpecialTagInfo(pbLoadedFile: *mut byte, iLoadedFileLen: c_int, ppTAG: *mut *mut id3v1_1, piUncompressedSize: *mut c_int, pfMaxVol: *mut f32) -> qboolean;
    pub fn MP3Stream_InitFromFile(sfx: *mut sfx_t, pbSrcData: *mut byte, iSrcDatalen: c_int, psSrcDataFilename: *const c_char, iMP3UnPackedSize: c_int, bStereoDesired: qboolean) -> qboolean;
    pub fn MP3Stream_Decode(lpMP3Stream: LP_MP3STREAM, bDoingMusic: qboolean) -> c_int;
    pub fn MP3Stream_SeekTo(ch: *mut channel_t, fTimeToSeekTo: f32) -> qboolean;
    pub fn MP3Stream_Rewind(ch: *mut channel_t) -> qboolean;
    pub fn MP3Stream_GetSamples(ch: *mut channel_t, startingSampleNum: c_int, count: c_int, buf: *mut i16, bStereo: qboolean) -> qboolean;
}

///////////////////////////////////////
//
// the real worker code deep down in the MP3 C code...  (now externalised here so the music streamer can access one)
//
extern "C" {
    pub fn C_MP3_IsValid(pvData: *mut c_void, iDataLen: c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3_GetUnpackedSize(pvData: *mut c_void, iDataLen: c_int, piUnpackedSize: *mut c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3_UnpackRawPCM(pvData: *mut c_void, iDataLen: c_int, piUnpackedSize: *mut c_int, pbUnpackBuffer: *mut c_void, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3_GetHeaderData(pvData: *mut c_void, iDataLen: c_int, piRate: *mut c_int, piWidth: *mut c_int, piChannels: *mut c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3Stream_DecodeInit(pSFX_MP3Stream: LP_MP3STREAM, pvSourceData: *mut c_void, iSourceBytesRemaining: c_int, iGameAudioSampleRate: c_int, iGameAudioSampleBits: c_int, bStereoDesired: c_int) -> *mut c_char;
    pub fn C_MP3Stream_Decode(pSFX_MP3Stream: LP_MP3STREAM, bFastForwarding: c_int) -> c_uint;
    pub fn C_MP3Stream_Rewind(pSFX_MP3Stream: LP_MP3STREAM) -> *mut c_char;
}
//
///////////////////////////////////////

///////////////// eof /////////////////////
