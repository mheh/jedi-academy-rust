// snd_mem.c: sound caching

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"

use crate::snd_local_console_h::*;
use core::ffi::{c_int, c_char, c_void};
use core::mem;

#[cfg(target_os = "xbox")]
use core::ffi::c_uint;

const SND_MAX_LOADS: c_int = 48;
static mut s_LoadList: *mut *mut sfx_t = core::ptr::null_mut();
static mut s_LoadListSize: c_int = 0;
pub static mut gbInsideLoadSound: qboolean = qfalse; // Needed to link VVFIXME

extern "C" {
    fn Sys_GetFileCode(name: *const c_char) -> c_int;
}

// Drain sound main memory into ARAM.
pub fn S_DrainRawSoundData() {
    extern "C" {
        static mut s_soundStarted: c_int;
    }
    unsafe {
        if s_soundStarted == 0 {
            return;
        }

        loop {
            S_UpdateLoading();

            #[cfg(target_os = "gamecube")]
            {
                extern "C" {
                    fn ERR_DiscFail(arg: bool);
                }
                ERR_DiscFail(true);
            }

            if s_LoadListSize == 0 {
                break;
            }
        }
    }
}

/*
============
GetWavInfo
============
*/
pub fn GetWavInfo(data: *mut u8) -> wavinfo_t {
    let mut info: wavinfo_t;
    unsafe {
        info = core::mem::zeroed();
    }

    if data.is_null() {
        return info;
    }

    unsafe {
        #[cfg(target_os = "gamecube")]
        {
            if *(data.add(14) as *const i16) != 0 {
                // invalid type, abort
                return info;
            }

            info.format = AL_FORMAT_MONO4;
            info.width = 4;
            info.size = ((*(data.add(20) as *const c_int) >> 1) + 96);
            info.rate = *(data.add(8) as *const c_int);
        }

        #[cfg(not(target_os = "gamecube"))]
        {
            let mut dataofs: c_int = 0;
            if libc::strncmp(data.add(dataofs as usize) as *const c_char, b"RIFF\0".as_ptr() as *const c_char, 4) != 0 ||
                libc::strncmp(data.add((dataofs + 8) as usize) as *const c_char, b"WAVE\0".as_ptr() as *const c_char, 4) != 0
            {
                // invalid type, abort
                return info;
            }
            dataofs += 12; // done with riff chunk

            let wav = data.add((dataofs + 8) as usize) as *const WAVEFORMATEX;
            info.format = if (*wav).nChannels == 1 { AL_FORMAT_MONO4 } else { AL_FORMAT_STEREO4 };
            info.rate = (*wav).nSamplesPerSec;
            info.width = (*wav).wBitsPerSample;
            dataofs += mem::size_of::<WAVEFORMATEX>() as c_int + (*wav).cbSize + 8; // done with fmt chunk

            info.size = *(data.add((dataofs + 4) as usize) as *const c_int);
            dataofs += 8; // done with data chunk
        }
    }

    info
}

// adjust filename for foreign languages and WAV/MP3 issues.
//
static fn S_LoadSound_FileNameAdjuster(psFilename: *mut c_char) -> c_int {
    #[cfg(target_os = "xbox")]
    let ext = b"wxb";
    #[cfg(target_os = "windows")]
    let ext = b"wav";
    #[cfg(target_os = "gamecube")]
    let ext = b"wgc";

    let len = unsafe { libc::strlen(psFilename) as c_int };

    #[allow(unused_assignments)]
    let mut psVoice: *mut c_char = core::ptr::null_mut();

    // #if 0 commented out foreign language logic
    #[cfg(feature = "foreign_voice_support")]
    unsafe {
        let chars_str = b"chars\0".as_ptr() as *const c_char;
        psVoice = libc::strstr(psFilename, chars_str);
        if !psVoice.is_null() {
            // account for foreign voices...
            //
            extern "C" {
                static mut sp_language: *mut cvar_t;
            }
            if !sp_language.is_null() && (*sp_language).integer == SP_LANGUAGE_GERMAN {
                libc::strncpy(psVoice, b"chr_d\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            } else if !sp_language.is_null() && (*sp_language).integer == SP_LANGUAGE_FRENCH {
                libc::strncpy(psVoice, b"chr_f\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            } else {
                psVoice = core::ptr::null_mut(); // use this ptr as a flag as to whether or not we substituted with a foreign version
            }
        }
    }

    unsafe {
        *psFilename.add((len - 3) as usize) = ext[0] as c_char;
        *psFilename.add((len - 2) as usize) = ext[1] as c_char;
        *psFilename.add((len - 1) as usize) = ext[2] as c_char;
    }
    let code = unsafe { Sys_GetFileCode(psFilename) };

    if code == -1 {
        // hmmm, not found, ok, maybe we were trying a foreign noise ("arghhhhh.mp3" that doesn't matter?) but it
        // was missing?   Can't tell really, since both types are now in sound/chars. Oh well, fall back to English for now...

        if !psVoice.is_null() {
            // were we trying to load foreign?
            // yep, so fallback to re-try the english...
            //
            unsafe {
                libc::strncpy(psVoice, b"chars\0".as_ptr() as *const c_char, 5);

                *psFilename.add((len - 3) as usize) = ext[0] as c_char;
                *psFilename.add((len - 2) as usize) = ext[1] as c_char;
                *psFilename.add((len - 1) as usize) = ext[2] as c_char;
            }
            let code = unsafe { Sys_GetFileCode(psFilename) };
            return code;
        }
    }

    code
}

/*
==============
S_GetFileCode
==============
*/
pub fn S_GetFileCode(sSoundName: *const c_char) -> c_int {
    let mut sLoadName: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    // make up a local filename to try wav/mp3 substitutes...
    //
    unsafe {
        Q_strncpyz(sLoadName.as_mut_ptr(), sSoundName, mem::size_of_val(&sLoadName));
        Q_strlwr(sLoadName.as_mut_ptr());
    }

    // make sure we have an extension...
    //
    unsafe {
        if sLoadName[libc::strlen(sLoadName.as_ptr()) - 4] != b'.' as c_char {
            libc::strcat(sLoadName.as_mut_ptr(), b".xxx\0".as_ptr() as *const c_char);
        }
    }

    S_LoadSound_FileNameAdjuster(sLoadName.as_mut_ptr())
}

/*
============
S_UpdateLoading
============
*/
pub fn S_UpdateLoading() {
    unsafe {
        for i in 0..SND_MAX_LOADS as usize {
            if !(*s_LoadList.add(i)).is_null() &&
                ((*(*s_LoadList.add(i))).iFlags & SFX_FLAG_LOADING) != 0 &&
                !Sys_StreamIsReading((*(*s_LoadList.add(i))).iStreamHandle)
            {
                S_EndLoadSound(*s_LoadList.add(i));
                *s_LoadList.add(i) = core::ptr::null_mut();
                s_LoadListSize -= 1;
            }
        }
    }
}

/*
==============
S_BeginLoadSound
==============
*/
pub fn S_StartLoadSound(sfx: *mut sfx_t) -> qboolean {
    unsafe {
        debug_assert!((*sfx).iFlags & SFX_FLAG_UNLOADED != 0);
        (*sfx).iFlags &= !SFX_FLAG_UNLOADED;

        // Valid file?
        if (*sfx).iFileCode == -1 {
            (*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
            return qfalse;
        }

        // Finish up any pending loads
        loop {
            S_UpdateLoading();
            if s_LoadListSize < SND_MAX_LOADS {
                break;
            }
        }

        // Open the file
        (*sfx).iSoundLength = Sys_StreamOpen((*sfx).iFileCode, &mut (*sfx).iStreamHandle);
        if (*sfx).iSoundLength <= 0 {
            (*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
            return qfalse;
        }

        #[cfg(target_os = "gamecube")]
        {
            // Allocate a buffer to read into...
            (*sfx).pSoundData = Z_Malloc(
                ((*sfx).iSoundLength + 64) as usize,
                TAG_SND_RAWDATA,
                qtrue,
                32,
            );
        }

        #[cfg(not(target_os = "gamecube"))]
        {
            // Allocate a buffer to read into...
            (*sfx).pSoundData = Z_Malloc((*sfx).iSoundLength as usize, TAG_SND_RAWDATA, qtrue, 32);
        }

        // Setup the background read
        if (*sfx).pSoundData.is_null() ||
            !Sys_StreamRead(
                (*sfx).pSoundData,
                (*sfx).iSoundLength,
                0,
                (*sfx).iStreamHandle,
            )
        {
            if !(*sfx).pSoundData.is_null() {
                Z_Free((*sfx).pSoundData);
            }
            Sys_StreamClose((*sfx).iStreamHandle);
            (*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
            return qfalse;
        }
        (*sfx).iFlags |= SFX_FLAG_LOADING;

        // add sound to load list
        for i in 0..SND_MAX_LOADS as usize {
            if (*s_LoadList.add(i)).is_null() {
                *s_LoadList.add(i) = sfx;
                s_LoadListSize += 1;
                break;
            }
        }
    }

    qtrue
}

/*
==============
S_EndLoadSound
==============
*/
pub fn S_EndLoadSound(sfx: *mut sfx_t) -> qboolean {
    let mut info: wavinfo_t;
    let data: *mut u8;
    let mut Buffer: ALuint;

    unsafe {
        debug_assert!((*sfx).iFlags & SFX_FLAG_LOADING != 0);
        (*sfx).iFlags &= !SFX_FLAG_LOADING;

        // was the read successful?
        if Sys_StreamIsError((*sfx).iStreamHandle) {
            // #if 0 // defined(FINAL_BUILD) //PORT
            // extern void ERR_DiscFail(bool);
            // ERR_DiscFail(false);
            // #endif
            Sys_StreamClose((*sfx).iStreamHandle);
            Z_Free((*sfx).pSoundData);
            (*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
            return qfalse;
        }

        Sys_StreamClose((*sfx).iStreamHandle);
        SND_TouchSFX(sfx);

        (*sfx).iLastTimeUsed = Com_Milliseconds() + 1; // why +1? Hmmm, leave it for now I guess

        // loading a WAV, presumably...
        data = (*sfx).pSoundData as *mut u8;
        info = GetWavInfo(data);

        if info.size == 0 {
            Z_Free((*sfx).pSoundData);
            (*sfx).iFlags |= SFX_FLAG_RESIDENT | SFX_FLAG_DEFAULT;
            return qfalse;
        }

        (*sfx).iSoundLength = info.size;

        // make sure we have enough space for the sound
        SND_update(sfx);

        // Clear Open AL Error State
        alGetError();

        // Generate AL Buffer
        alGenBuffers(1, &mut Buffer);

        // Copy audio data to AL Buffer
        alBufferData(Buffer, info.format, data, (*sfx).iSoundLength, info.rate);
        if alGetError() != AL_NO_ERROR {
            Z_Free((*sfx).pSoundData);
            (*sfx).iFlags |= SFX_FLAG_UNLOADED;
            return qfalse;
        }

        (*sfx).Buffer = Buffer;
        #[cfg(target_os = "gamecube")]
        {
            Z_Free((*sfx).pSoundData);
        }
        (*sfx).iFlags |= SFX_FLAG_RESIDENT;
    }

    qtrue
}

/*
============
S_InitLoad
============
*/
pub fn S_InitLoad() {
    unsafe {
        s_LoadList = libc::malloc((SND_MAX_LOADS as usize) * mem::size_of::<*mut sfx_t>()) as *mut *mut sfx_t;
        libc::memset(s_LoadList as *mut c_void, 0, (SND_MAX_LOADS as usize) * mem::size_of::<*mut sfx_t>());
        s_LoadListSize = 0;
    }
}

/*
============
S_CloseLoad
============
*/
pub fn S_CloseLoad() {
    unsafe {
        libc::free(s_LoadList as *mut c_void);
    }
}
