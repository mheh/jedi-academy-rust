//! Mechanical port of `code/client/snd_mem_console.cpp`.
//!
//! Sound caching.

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::code::client::snd_local_console_h::*;
use crate::codemp::game::q_shared_h::byte;

// #include "snd_local_console.h"
//
// #ifdef _XBOX
// #include <xtl.h>
// #endif

// ============================================================================
// Constants
// ============================================================================

const SND_MAX_LOADS: usize = 48;

// ============================================================================
// Global variables
// ============================================================================

static mut s_LoadList: *mut *mut sfx_t = ptr::null_mut();
static mut s_LoadListSize: c_int = 0;
pub static mut gbInsideLoadSound: c_int = 0;	// Needed to link VVFIXME

// ============================================================================
// External declarations
// ============================================================================

unsafe extern "C" {
	pub fn Z_Malloc(size: c_int, tag: c_int, clear: bool, alignment: c_int) -> *mut c_void;
	pub fn Z_Free(ptr: *mut c_void);
	pub fn Com_Milliseconds() -> c_int;
	pub fn alGetError() -> c_int;
	pub fn alGenBuffers(n: c_int, buffers: *mut c_int);
	pub fn alBufferData(buffer: c_int, format: c_int, data: *const c_void, size: c_int, freq: c_int);
	pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
	pub fn Q_strlwr(s: *mut c_char) -> *mut c_char;
	pub fn strlen(s: *const c_char) -> usize;
	pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
	pub fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
	pub fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
	pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;

	pub static s_soundStarted: c_int;
}

// AL constants (from OpenAL/al.h)
const AL_FORMAT_MONO4: c_int = 4112;
const AL_FORMAT_STEREO4: c_int = 4113;
const AL_NO_ERROR: c_int = 0;

// Memory tag
const TAG_SND_RAWDATA: c_int = 18;

// ============================================================================
// WAVEFORMATEX structure (Windows-specific)
// ============================================================================

#[cfg(not(feature = "gamecube"))]
#[repr(C)]
pub struct WAVEFORMATEX {
	pub wFormatTag: u16,
	pub nChannels: i16,
	pub nSamplesPerSec: c_int,
	pub nAvgBytesPerSec: c_int,
	pub nBlockAlign: u16,
	pub wBitsPerSample: i16,
	pub cbSize: c_int,
}

// ============================================================================
// Drain sound main memory into ARAM.
// ============================================================================

pub fn S_DrainRawSoundData() {
	unsafe {
		if s_soundStarted == 0 {
			return;
		}

		loop {
			S_UpdateLoading();

			#[cfg(feature = "gamecube")]
			{
				extern "C" {
					pub fn ERR_DiscFail(b: bool);
				}
				ERR_DiscFail(true);
			}

			if s_LoadListSize == 0 {
				break;
			}
		}
	}
}

// ============================================================================
// GetWavInfo
// ============================================================================

pub fn GetWavInfoLocal(data: *mut byte) -> wavinfo_t {
	let mut info: wavinfo_t = unsafe { std::mem::zeroed() };

	if data.is_null() {
		return info;
	}

	unsafe {
		#[cfg(feature = "gamecube")]
		{
			if *(data.add(14) as *mut i16) != 0 {
				// invalid type, abort
				return info;
			}

			info.format = AL_FORMAT_MONO4;
			info.width = 4;
			info.size = ((*(data.add(20) as *mut c_int)) >> 1) + 96;
			info.rate = *(data.add(8) as *mut c_int);
		}

		#[cfg(not(feature = "gamecube"))]
		{
			let mut dataofs: c_int = 0;
			if strncmp(data.add(dataofs as usize) as *const c_char, b"RIFF\0".as_ptr() as *const c_char, 4) != 0
				|| strncmp(data.add((dataofs + 8) as usize) as *const c_char, b"WAVE\0".as_ptr() as *const c_char, 4) != 0
			{
				// invalid type, abort
				return info;
			}
			dataofs += 12; // done with riff chunk

			let wav = (data.add(dataofs as usize + 8) as *const WAVEFORMATEX);
			info.format = if (*wav).nChannels == 1 { AL_FORMAT_MONO4 } else { AL_FORMAT_STEREO4 };
			info.rate = (*wav).nSamplesPerSec;
			info.width = (*wav).wBitsPerSample;
			dataofs += (std::mem::size_of::<WAVEFORMATEX>() as c_int) + (*wav).cbSize + 8; // done with fmt chunk

			info.size = *(data.add(dataofs as usize + 4) as *mut c_int);
			info.samples = info.size * 2;

			dataofs += 8; // done with data chunk
		}
	}

	info
}

// ============================================================================
// adjust filename for foreign languages and WAV/MP3 issues.
// ============================================================================

fn S_LoadSound_FileNameAdjuster(psFilename: *mut c_char) -> c_int {
	#[cfg(feature = "xbox")]
	let ext = b"wxb".as_ptr() as *const u8;
	#[cfg(all(not(feature = "xbox"), feature = "windows"))]
	let ext = b"wav".as_ptr() as *const u8;
	#[cfg(feature = "gamecube")]
	let ext = b"wgc".as_ptr() as *const u8;
	#[cfg(all(not(feature = "xbox"), not(feature = "windows"), not(feature = "gamecube")))]
	let ext = b"wav".as_ptr() as *const u8;  // default to wav

	unsafe {
		let len = strlen(psFilename);

		#[cfg(not(feature = "disable_voice_handling"))]
		let psVoice: *mut c_char = {
			let search = b"chars\0".as_ptr() as *const c_char;
			let result = strstr(psFilename, search);
			if !result.is_null() {
				// account for foreign voices...
				//
				extern "C" {
					pub static sp_language: *mut cvar_t;
				}

				const SP_LANGUAGE_GERMAN: c_int = 1;
				const SP_LANGUAGE_FRENCH: c_int = 2;

				if !sp_language.is_null() && (*sp_language).integer == SP_LANGUAGE_GERMAN {
					strncpy(result, b"chr_d\0".as_ptr() as *const c_char, 5);	// same number of letters as "chars"
					result
				} else if !sp_language.is_null() && (*sp_language).integer == SP_LANGUAGE_FRENCH {
					strncpy(result, b"chr_f\0".as_ptr() as *const c_char, 5);	// same number of letters as "chars"
					result
				} else {
					ptr::null_mut()	// use this ptr as a flag as to whether or not we substituted with a foreign version
				}
			} else {
				ptr::null_mut()
			}
		};

		#[cfg(feature = "disable_voice_handling")]
		let psVoice: *mut c_char = ptr::null_mut();

		*psFilename.add(len - 3) = *ext as c_char;
		*psFilename.add(len - 2) = *(ext.add(1)) as c_char;
		*psFilename.add(len - 1) = *(ext.add(2)) as c_char;
		let mut code = Sys_GetFileCode(psFilename);

		if code == -1 {
			//hmmm, not found, ok, maybe we were trying a foreign noise ("arghhhhh.mp3" that doesn't matter?) but it
			// was missing?   Can't tell really, since both types are now in sound/chars. Oh well, fall back to English for now...

			if !psVoice.is_null() {	// were we trying to load foreign?
				// yep, so fallback to re-try the english...
				//
				strncpy(psVoice, b"chars\0".as_ptr() as *const c_char, 5);

				*psFilename.add(len - 3) = *ext as c_char;
				*psFilename.add(len - 2) = *(ext.add(1)) as c_char;
				*psFilename.add(len - 1) = *(ext.add(2)) as c_char;
				code = Sys_GetFileCode(psFilename);
			}
		}

		code
	}
}

// ============================================================================
// S_GetFileCode
// ============================================================================

pub fn S_GetFileCode(sSoundName: *const c_char) -> c_int {
	const MAX_QPATH: usize = 64;
	let mut sLoadName: [c_char; MAX_QPATH] = [0; MAX_QPATH];

	unsafe {
		// make up a local filename to try wav/mp3 substitutes...
		//
		Q_strncpyz(sLoadName.as_mut_ptr(), sSoundName, MAX_QPATH as c_int);
		Q_strlwr(sLoadName.as_mut_ptr());

		// make sure we have an extension...
		//
		let slen = strlen(sLoadName.as_ptr());
		if sLoadName[slen - 4] as u8 as char != '.' {
			strcat(sLoadName.as_mut_ptr(), b".xxx\0".as_ptr() as *const c_char);
		}

		S_LoadSound_FileNameAdjuster(sLoadName.as_mut_ptr())
	}
}

// ============================================================================
// S_UpdateLoading
// ============================================================================

pub fn S_UpdateLoading() {
	unsafe {
		let mut i: c_int = 0;
		while i < SND_MAX_LOADS as c_int {
			if !(*s_LoadList.add(i as usize)).is_null() {
				let sfx = *s_LoadList.add(i as usize);
				if ((*sfx).iFlags & SFX_FLAG_LOADING) != 0
					&& !Sys_StreamIsReading((*sfx).iStreamHandle)
				{
					S_EndLoadSound(sfx);
					*s_LoadList.add(i as usize) = ptr::null_mut();
					s_LoadListSize -= 1;
				}
			}
			i += 1;
		}
	}
}

// ============================================================================
// S_BeginLoadSound
// ============================================================================

pub fn S_StartLoadSound(sfx: *mut sfx_t) -> c_int {
	unsafe {
		assert!((*sfx).iFlags & SFX_FLAG_UNLOADED != 0);
		(*sfx).iFlags &= !SFX_FLAG_UNLOADED;

		// Valid file?
		if (*sfx).iFileCode == -1 {
			(*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
			return 0;
		}

		// Finish up any pending loads
		loop {
			S_UpdateLoading();
			if s_LoadListSize < SND_MAX_LOADS as c_int {
				break;
			}
		}

		// Open the file
		(*sfx).iSoundLength = Sys_StreamOpen((*sfx).iFileCode, &mut (*sfx).iStreamHandle);
		if (*sfx).iSoundLength <= 0 {
			(*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
			return 0;
		}

		#[cfg(feature = "gamecube")]
		{
			// Allocate a buffer to read into...
			(*sfx).pSoundData = Z_Malloc((*sfx).iSoundLength + 64, TAG_SND_RAWDATA, true, 32);
		}

		#[cfg(not(feature = "gamecube"))]
		{
			// Allocate a buffer to read into...
			(*sfx).pSoundData = Z_Malloc((*sfx).iSoundLength, TAG_SND_RAWDATA, true, 32);
		}

		// Setup the background read
		if (*sfx).pSoundData.is_null()
			|| !Sys_StreamRead((*sfx).pSoundData, (*sfx).iSoundLength, 0, (*sfx).iStreamHandle)
		{
			if !(*sfx).pSoundData.is_null() {
				Z_Free((*sfx).pSoundData);
			}
			Sys_StreamClose((*sfx).iStreamHandle);
			(*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
			return 0;
		}
		(*sfx).iFlags |= SFX_FLAG_LOADING;

		// add sound to load list
		let mut i: c_int = 0;
		while i < SND_MAX_LOADS as c_int {
			if (*s_LoadList.add(i as usize)).is_null() {
				*s_LoadList.add(i as usize) = sfx;
				s_LoadListSize += 1;
				break;
			}
			i += 1;
		}

		1
	}
}

// ============================================================================
// S_EndLoadSound
// ============================================================================

pub fn S_EndLoadSound(sfx: *mut sfx_t) -> c_int {
	unsafe {
		let info: wavinfo_t;
		let data: *mut byte;

		assert!((*sfx).iFlags & SFX_FLAG_LOADING != 0);
		(*sfx).iFlags &= !SFX_FLAG_LOADING;

		// was the read successful?
		if Sys_StreamIsError((*sfx).iStreamHandle) {
			#[cfg(feature = "final_build")]
			{
				/*
				extern "C" {
					pub fn ERR_DiscFail(b: bool);
				}
				ERR_DiscFail(false);
				*/
			}
			Sys_StreamClose((*sfx).iStreamHandle);
			Z_Free((*sfx).pSoundData);
			(*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
			return 0;
		}

		Sys_StreamClose((*sfx).iStreamHandle);
		SND_TouchSFX(sfx);

		(*sfx).iLastTimeUsed = Com_Milliseconds() + 1;	// why +1? Hmmm, leave it for now I guess

		// loading a WAV, presumably...
		data = (*sfx).pSoundData as *mut byte;
		info = GetWavInfoLocal(data);

		if info.size == 0 {
			Z_Free((*sfx).pSoundData);
			(*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
			return 0;
		}

		(*sfx).iSoundLength = info.size;

		// make sure we have enough space for the sound
		SND_update(sfx);

		// Clear Open AL Error State
		alGetError();

		// Generate AL Buffer
		let mut buf: c_int = 0;
		alGenBuffers(1, &mut buf);

		// Copy audio data to AL Buffer
		alBufferData(buf, info.format, data as *const c_void,
			(*sfx).iSoundLength, info.rate);
		if alGetError() != AL_NO_ERROR {
			Z_Free((*sfx).pSoundData);
			(*sfx).iFlags |= SFX_FLAG_UNLOADED;
			return 0;
		}

		(*sfx).Buffer = buf as u32;

		#[cfg(feature = "gamecube")]
		{
			Z_Free((*sfx).pSoundData);
		}
		(*sfx).iFlags |= SFX_FLAG_RESIDENT;

		1
	}
}

// ============================================================================
// S_InitLoad
// ============================================================================

pub fn S_InitLoad() {
	unsafe {
		s_LoadList = Z_Malloc((SND_MAX_LOADS * std::mem::size_of::<*mut sfx_t>()) as c_int, TAG_SND_RAWDATA, true, std::mem::align_of::<*mut sfx_t>() as c_int) as *mut *mut sfx_t;

		// Initialize all entries to NULL
		let mut i = 0;
		while i < SND_MAX_LOADS {
			*s_LoadList.add(i) = ptr::null_mut();
			i += 1;
		}

		s_LoadListSize = 0;
	}
}

// ============================================================================
// S_CloseLoad
// ============================================================================

pub fn S_CloseLoad() {
	unsafe {
		Z_Free(s_LoadList as *mut c_void);
	}
}
