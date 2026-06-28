// snd_mem.c: sound caching

// leave this as first line for PCH reasons...
//

// Open AL
extern "C" {
    pub fn S_PreProcessLipSync(sfx: *mut sfx_t);
    pub static mut s_UseOpenAL: c_int;
}

use core::ffi::{c_int, c_char, c_void};
use std::mem;
use std::ptr;

/*
===============================================================================

WAV loading

===============================================================================
*/

static mut data_p: *mut u8 = ptr::null_mut();
static mut iff_end: *mut u8 = ptr::null_mut();
static mut last_chunk: *mut u8 = ptr::null_mut();
static mut iff_data: *mut u8 = ptr::null_mut();
static mut iff_chunk_len: c_int = 0;

extern "C" {
    pub static mut s_knownSfx: [sfx_t; 0];
    pub static mut s_numSfx: c_int;
}

extern "C" {
    pub static mut s_lip_threshold_1: *mut cvar_t;
    pub static mut s_lip_threshold_2: *mut cvar_t;
    pub static mut s_lip_threshold_3: *mut cvar_t;
    pub static mut s_lip_threshold_4: *mut cvar_t;
}

#[repr(C)]
pub struct wavinfo_t {
    pub format: i16,
    pub channels: i16,
    pub rate: c_int,
    pub width: c_int,
    pub samples: c_int,
    pub dataofs: c_int,
}

#[repr(C)]
pub struct sfx_t {
    // placeholder - actual definition in snd_local.h
}

#[repr(C)]
pub struct cvar_t {
    // placeholder - actual definition elsewhere
}

#[repr(C)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    pub title: [c_char; 30],
    pub artist: [c_char; 30],
    pub album: [c_char; 30],
    pub year: [c_char; 4],
    pub comment: [c_char; 30],
}

extern "C" {
    pub fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    pub fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn strlwr(s: *mut c_char) -> *mut c_char;
    pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;

    pub fn Com_Printf(format: *const c_char, ...);
    pub fn Com_DPrintf(format: *const c_char, ...);
    pub fn Com_Error(code: c_int, format: *const c_char, ...);

    pub fn FS_ListFiles(path: *const c_char, ext: *const c_char, numfiles: *mut c_int) -> *mut *mut c_char;
    pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    pub fn FS_FOpenFileRead(filename: *const c_char, f: *mut c_int, flush: c_int) -> c_int;
    pub fn FS_FOpenFileWrite(filename: *const c_char) -> c_int;
    pub fn FS_Write(buffer: *const c_void, len: c_int, f: c_int) -> c_int;
    pub fn FS_FCloseFile(f: c_int);
    pub fn FS_FreeFile(buffer: *mut c_void);
    pub fn FS_FreeFileList(list: *mut *mut c_char);

    pub fn SND_malloc(size: usize, sfx: *mut sfx_t) -> *mut c_void;
    pub fn SND_TouchSFX(sfx: *mut sfx_t);
    pub fn SND_FreeOldestSound();

    pub fn Z_Malloc(size: usize, tag: c_int, clear: bool) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);

    pub fn LittleShort(l: i16) -> i16;

    pub fn Cmd_Argc() -> c_int;
    pub fn Cmd_Argv(arg: c_int) -> *mut c_char;

    pub fn S_StopAllSounds();
    pub fn S_FindName(name: *const c_char) -> *mut sfx_t;

    pub fn MP3_ReadSpecialTagInfo(data: *mut u8, size: c_int, tag: *mut *mut id3v1_1) -> bool;
    pub fn MP3_IsValid(filename: *const c_char, data: *mut u8, size: c_int, stereo: bool) -> bool;
    pub fn MP3_GetUnpackedSize(filename: *const c_char, data: *mut u8, size: c_int, calc: bool, stereo: bool) -> c_int;
    pub fn MP3_UnpackRawPCM(filename: *const c_char, data: *mut u8, size: c_int, buffer: *mut u8) -> c_int;
    pub fn MP3_UnpackRawPCM_with_flag(filename: *const c_char, data: *mut u8, size: c_int, buffer: *mut u8, flag: bool) -> c_int;
    pub fn MP3_FakeUpWAVInfo(filename: *const c_char, data: *mut u8, size: c_int, samples: c_int,
                              format: *mut i16, rate: *mut c_int, width: *mut c_int, channels: *mut i16,
                              num_samples: *mut c_int, dataofs: *mut c_int);
    pub fn MP3_FakeUpWAVInfo_with_flag(filename: *const c_char, data: *mut u8, size: c_int, samples: c_int,
                                         format: *mut i16, rate: *mut c_int, width: *mut c_int, channels: *mut i16,
                                         num_samples: *mut c_int, dataofs: *mut c_int, flag: bool);
    pub fn MP3Stream_InitFromFile(sfx: *mut sfx_t, data: *mut u8, size: c_int, filename: *const c_char,
                                   bufsize: c_int, flag: bool) -> bool;

    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn COM_DefaultExtension(path: *mut c_char, pathsize: c_int, extension: *const c_char);

    pub static mut com_buildScript: *mut cvar_t;
    pub static mut s_language: *mut cvar_t;

    pub fn alGetError() -> c_int;
    pub fn alGenBuffers(n: c_int, buffers: *mut c_int);
    pub fn alBufferData(buffer: c_int, format: c_int, data: *const c_void, size: c_int, freq: c_int);

    pub fn va(format: *const c_char, ...) -> *mut c_char;
}

const ERR_DROP: c_int = 0;
const ERR_FATAL: c_int = 1;
const MAX_QPATH: usize = 256;
const TAG_SND_RAWDATA: c_int = 0;
const TAG_TEMP_WORKSPACE: c_int = 0;
const AL_NO_ERROR: c_int = 0;
const AL_FORMAT_MONO16: c_int = 0x1101;

// Stub definitions for types that might be needed
type fileHandle_t = c_int;

extern "C" {
    pub static mut dma: dma_t;
}

#[repr(C)]
pub struct dma_t {
    pub speed: c_int,
    // ... other fields not shown
}

const S_COLOR_YELLOW: &str = "^3";

fn GetLittleShort() -> i16 {
    unsafe {
        let mut val: i16 = 0;
        val = *data_p as i16;
        val = (val as c_int + ((*data_p.offset(1) as c_int) << 8)) as i16;
        data_p = data_p.offset(2);
        val
    }
}

fn GetLittleLong() -> c_int {
    unsafe {
        let mut val: c_int = 0;
        val = *data_p as c_int;
        val = val + ((*data_p.offset(1) as c_int) << 8);
        val = val + ((*data_p.offset(2) as c_int) << 16);
        val = val + ((*data_p.offset(3) as c_int) << 24);
        data_p = data_p.offset(4);
        val
    }
}

fn FindNextChunk(name: *const c_char) {
    unsafe {
        loop {
            data_p = last_chunk;

            if data_p >= iff_end {
                // didn't find the chunk
                data_p = ptr::null_mut();
                return;
            }

            data_p = data_p.offset(4);
            iff_chunk_len = GetLittleLong();
            if iff_chunk_len < 0 {
                data_p = ptr::null_mut();
                return;
            }
            data_p = data_p.offset(-8);
            last_chunk = data_p.offset(8 + (((iff_chunk_len + 1) & !1) as isize));
            if strncmp(data_p as *const c_char, name, 4) == 0 {
                return;
            }
        }
    }
}

fn FindChunk(name: *const c_char) {
    unsafe {
        last_chunk = iff_data;
        FindNextChunk(name);
    }
}

fn DumpChunks() {
    unsafe {
        let mut str: [c_char; 5] = [0; 5];
        str[4] = 0;
        data_p = iff_data;
        loop {
            memcpy(str.as_mut_ptr() as *mut c_void, data_p as *const c_void, 4);
            data_p = data_p.offset(4);
            iff_chunk_len = GetLittleLong();
            Com_Printf(b"0x%x : %s (%d)\n\0".as_ptr() as *const c_char,
                       (data_p as usize - 4), str.as_ptr(), iff_chunk_len);
            data_p = data_p.offset((((iff_chunk_len + 1) & !1) as isize));
            if !(data_p < iff_end) {
                break;
            }
        }
    }
}

/*
============
GetWavinfo
============
*/
fn GetWavinfo(name: *const c_char, wav: *mut u8, wavlength: c_int) -> wavinfo_t {
    unsafe {
        let mut info: wavinfo_t = mem::zeroed();
        let mut samples: c_int;

        memset(&mut info as *mut wavinfo_t as *mut c_void, 0, mem::size_of::<wavinfo_t>());

        if wav.is_null() {
            return info;
        }

        iff_data = wav;
        iff_end = wav.offset(wavlength as isize);

        // find "RIFF" chunk
        FindChunk(b"RIFF\0".as_ptr() as *const c_char);
        if !((!data_p.is_null()) && (strncmp(data_p.offset(8) as *const c_char, b"WAVE\0".as_ptr() as *const c_char, 4) == 0)) {
            Com_Printf(b"Missing RIFF/WAVE chunks\n\0".as_ptr() as *const c_char);
            return info;
        }

        // get "fmt " chunk
        iff_data = data_p.offset(12);
        // DumpChunks ();

        FindChunk(b"fmt \0".as_ptr() as *const c_char);
        if data_p.is_null() {
            Com_Printf(b"Missing fmt chunk\n\0".as_ptr() as *const c_char);
            return info;
        }
        data_p = data_p.offset(8);
        info.format = GetLittleShort();
        info.channels = GetLittleShort();
        info.rate = GetLittleLong();
        data_p = data_p.offset(4 + 2);
        info.width = GetLittleShort() as c_int / 8;

        if info.format != 1 {
            Com_Printf(b"Microsoft PCM format only\n\0".as_ptr() as *const c_char);
            return info;
        }

        // find data chunk
        FindChunk(b"data\0".as_ptr() as *const c_char);
        if data_p.is_null() {
            Com_Printf(b"Missing data chunk\n\0".as_ptr() as *const c_char);
            return info;
        }

        data_p = data_p.offset(4);
        samples = GetLittleLong() / info.width;

        if info.samples != 0 {
            if samples < info.samples {
                Com_Error(ERR_DROP, b"Sound %s has a bad loop length\0".as_ptr() as *const c_char, name);
            }
        } else {
            info.samples = samples;
        }

        info.dataofs = (data_p as usize - wav as usize) as c_int;

        info
    }
}

/*
================
ResampleSfx

resample / decimate to the current source rate
================
*/
fn ResampleSfx(sfx: *mut sfx_t, iInRate: c_int, iInWidth: c_int, pData: *mut u8) {
    unsafe {
        let mut iOutCount: c_int;
        let mut iSrcSample: c_int;
        let mut fStepScale: f32;
        let mut i: c_int;
        let mut iSample: c_int;
        let mut uiSampleFrac: u32 = 0; // uiSampleFrac MUST be unsigned, or large samples (eg music tracks) crash
        let mut uiFracStep: u32;

        fStepScale = (iInRate as f32) / dma.speed as f32; // this is usually 0.5, 1, or 2

        // When stepscale is > 1 (we're downsampling), we really ought to run a low pass filter on the samples

        iOutCount = ((*sfx).iSoundLengthInSamples as f32 / fStepScale) as c_int;
        (*sfx).iSoundLengthInSamples = iOutCount;

        (*sfx).pSoundData = SND_malloc(((*sfx).iSoundLengthInSamples as usize * 2) as usize, sfx) as *mut i16;

        (*sfx).fVolRange = 0.0;
        uiSampleFrac = 0;
        uiFracStep = (fStepScale * 256.0) as u32;

        i = 0;
        while i < (*sfx).iSoundLengthInSamples {
            iSrcSample = (uiSampleFrac >> 8) as c_int;
            uiSampleFrac = uiSampleFrac.wrapping_add(uiFracStep);
            if iInWidth == 2 {
                iSample = LittleShort((*(pData.offset(iSrcSample as isize * 2) as *mut i16)).offset(iSrcSample as isize));
            } else {
                iSample = ((*pData.offset(iSrcSample as isize) as c_int) - 128) << 8;
            }

            *(((*sfx).pSoundData).offset(i as isize)) = iSample as i16;

            // work out max vol for this sample...
            //
            let mut vol_sample = iSample;
            if vol_sample < 0 {
                vol_sample = -vol_sample;
            }
            if (*sfx).fVolRange < ((vol_sample >> 8) as f32) {
                (*sfx).fVolRange = (vol_sample >> 8) as f32;
            }

            i += 1;
        }
    }
}

//=============================================================================

fn S_LoadSound_Finalize(info: *mut wavinfo_t, sfx: *mut sfx_t, data: *mut u8) {
    unsafe {
        let mut stepscale: f32 = ((*info).rate as f32) / dma.speed as f32;
        let mut len: c_int = ((*info).samples as f32 / stepscale) as c_int;

        len = len * (*info).width;

        (*sfx).eSoundCompressionMethod = 16; // ct_16
        (*sfx).iSoundLengthInSamples = (*info).samples;
        ResampleSfx(sfx, (*info).rate, (*info).width, data.offset((*info).dataofs as isize));
    }
}

// maybe I'm re-inventing the wheel, here, but I can't see any functions that already do this, so...
//
fn Filename_WithoutPath(psFilename: *const c_char) -> *mut c_char {
    unsafe {
        static mut sString: [c_char; MAX_QPATH] = [0; MAX_QPATH]; // !!
        let mut p: *mut c_char = strrchr(psFilename, b'\\' as c_int);

        if p.is_null() {
            p = psFilename as *mut c_char;
        } else {
            p = p.offset(1);
        }

        strcpy(sString.as_mut_ptr(), p);

        sString.as_mut_ptr()
    }
}

// returns (eg) "\dir\name" for "\dir\name.bmp"
//
fn Filename_WithoutExt(psFilename: *const c_char) -> *mut c_char {
    unsafe {
        static mut sString: [c_char; MAX_QPATH] = [0; MAX_QPATH]; // !

        strcpy(sString.as_mut_ptr(), psFilename);

        let mut p: *mut c_char = strrchr(sString.as_mut_ptr(), b'.' as c_int);
        let mut p2: *mut c_char = strrchr(sString.as_mut_ptr(), b'\\' as c_int);

        // special check, make sure the first suffix we found from the end wasn't just a directory suffix (eg on a path'd filename with no extension anyway)
        //
        if !p.is_null() && (p2.is_null() || (!p2.is_null() && p > p2)) {
            *p = 0;
        }

        sString.as_mut_ptr()
    }
}

static mut iFilesFound: c_int = 0;
static mut iFilesUpdated: c_int = 0;
static mut iErrors: c_int = 0;
static mut qbForceRescan: bool = false;
static mut qbForceStereo: bool = false;
static mut strErrors: String = String::new();

fn R_CheckMP3s(psDir: *const c_char) {
    unsafe {
        //  Com_Printf(va("Scanning Dir: %s\n",psDir));
        Com_Printf(b".\0".as_ptr() as *const c_char); // stops useful info scrolling off screen

        let mut sysFiles: *mut *mut c_char;
        let mut dirFiles: *mut *mut c_char;
        let mut numSysFiles: c_int = 0;
        let mut i: c_int;
        let mut numdirs: c_int = 0;

        dirFiles = FS_ListFiles(psDir, b"/\0".as_ptr() as *const c_char, &mut numdirs);
        if numdirs > 2 {
            i = 2;
            while i < numdirs {
                let mut sDirName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                sprintf(sDirName.as_mut_ptr(), b"%s\\%s\0".as_ptr() as *const c_char, psDir, *dirFiles.offset(i as isize));
                R_CheckMP3s(sDirName.as_ptr());
                i += 1;
            }
        }

        sysFiles = FS_ListFiles(psDir, b".mp3\0".as_ptr() as *const c_char, &mut numSysFiles);
        i = 0;
        while i < numSysFiles {
            let mut sFilename: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            sprintf(sFilename.as_mut_ptr(), b"%s\\%s\0".as_ptr() as *const c_char, psDir, *sysFiles.offset(i as isize));

            Com_Printf(b"%sFound file: %s\0".as_ptr() as *const c_char, if i == 0 { b"\n\0".as_ptr() as *const c_char } else { b"\0".as_ptr() as *const c_char }, sFilename.as_ptr());

            iFilesFound += 1;

            // read it in...
            //
            let mut pbData: *mut u8 = ptr::null_mut();
            let mut iSize: c_int = FS_ReadFile(sFilename.as_ptr(), &mut (pbData as *mut c_void));

            if !pbData.is_null() {
                let mut pTAG: *mut id3v1_1;

                // do NOT check 'qbForceRescan' here as an opt, because we need to actually fill in 'pTAG' if there is one...
                //
                let qbTagNeedsUpdating: bool = if !MP3_ReadSpecialTagInfo(pbData, iSize, &mut pTAG) { true } else { false };

                if pTAG.is_null() || qbTagNeedsUpdating || qbForceRescan {
                    Com_Printf(b" ( Updating )\n\0".as_ptr() as *const c_char);

                    // I need to scan this file to get the volume...
                    //
                    // For EF1 I used a temp sfx_t struct, but I can't do that now with this new alloc scheme,
                    //  I have to ask for it legally, so I'll keep re-using one, and restoring it's name after use.
                    //  (slightly dodgy, but works ok if no-one else changes stuff)
                    //
                    //sfx_t SFX = {0};
                    //
                    static mut pSFX: *mut sfx_t = ptr::null_mut();
                    let sReservedSFXEntrynameForMP3: &[u8] = b"reserved_for_mp3\0"; // ( strlen() < MAX_QPATH )

                    if pSFX.is_null() { // once only
                        pSFX = S_FindName(sReservedSFXEntrynameForMP3.as_ptr() as *const c_char); // always returns, else ERR_FATAL
                    }

                    if MP3_IsValid(sFilename.as_ptr(), pbData, iSize, qbForceStereo) {
                        let mut info: wavinfo_t = mem::zeroed();

                        let iRawPCMDataSize: c_int = MP3_GetUnpackedSize(sFilename.as_ptr(), pbData, iSize, true, qbForceStereo);

                        if iRawPCMDataSize != 0 { // should always be true, unless file is fucked, in which case, stop this conversion process
                            let mut fMaxVol: f32 = 128.0; // any old default
                            let mut iActualUnpackedSize: c_int = iRawPCMDataSize; // default, override later if not doing music

                            if !qbForceStereo { // no point for stereo files, which are for music and therefore no lip-sync
                                let pbUnpackBuffer: *mut u8 = Z_Malloc((iRawPCMDataSize + 10) as usize, TAG_TEMP_WORKSPACE, false) as *mut u8; // won't return if fails

                                iActualUnpackedSize = MP3_UnpackRawPCM(sFilename.as_ptr(), pbData, iSize, pbUnpackBuffer);
                                if iActualUnpackedSize != iRawPCMDataSize {
                                    Com_Error(ERR_DROP, b"******* Whoah! MP3 %s unpacked to %d bytes, but size calc said %d!\n\0".as_ptr() as *const c_char, sFilename.as_ptr(), iActualUnpackedSize, iRawPCMDataSize);
                                }

                                // fake up a WAV structure so I can use the other post-load sound code such as volume calc for lip-synching
                                //
                                MP3_FakeUpWAVInfo(sFilename.as_ptr(), pbData, iSize, iActualUnpackedSize,
                                    // these params are all references...
                                    &mut info.format, &mut info.rate, &mut info.width, &mut info.channels, &mut info.samples, &mut info.dataofs
                                );

                                S_LoadSound_Finalize(&mut info, pSFX, pbUnpackBuffer); // all this just for lipsynch. Oh well.

                                fMaxVol = (*pSFX).fVolRange;

                                // free sfx->data...
                                //
                                {
                                    // #ifndef INT_MIN
                                    // #define INT_MIN     (-2147483647 - 1) /* minimum (signed) int value */
                                    // #endif
                                    const INT_MIN: c_int = -2147483647 - 1;
                                    //
                                    (*pSFX).iLastTimeUsed = INT_MIN; // force this to be oldest sound file, therefore disposable...
                                    (*pSFX).bInMemory = true;
                                    SND_FreeOldestSound(); // ... and do the disposal

                                    // now set our temp SFX struct back to default name so nothing else accidentally uses it...
                                    //
                                    strcpy((*pSFX).sSoundName.as_mut_ptr(), sReservedSFXEntrynameForMP3.as_ptr() as *const c_char);
                                    (*pSFX).bDefaultSound = false;
                                }

                                // OutputDebugString(va("File: \"%s\"   MaxVol %f\n",sFilename,pSFX->fVolRange));

                                // other stuff...
                                //
                                Z_Free(pbUnpackBuffer as *mut c_void);
                            }

                            // well, time to update the file now...
                            //
                            let f: fileHandle_t = FS_FOpenFileWrite(sFilename.as_ptr());
                            if f != 0 {
                                // write the file back out, but omitting the tag if there was one...
                                //
                                let iWritten: c_int = FS_Write(pbData as *const c_void, iSize - (if !pTAG.is_null() { mem::size_of::<id3v1_1>() as c_int } else { 0 }), f);

                                if iWritten != 0 {
                                    // make up a new tag if we didn't find one in the original file...
                                    //
                                    let mut TAG: id3v1_1 = mem::zeroed();
                                    if pTAG.is_null() {
                                        pTAG = &mut TAG;
                                        memset(&mut TAG as *mut id3v1_1 as *mut c_void, 0, mem::size_of::<id3v1_1>());
                                        strncpy(pTAG.as_mut().unwrap().id.as_mut_ptr(), b"TAG\0".as_ptr() as *const c_char, 3);
                                    }

                                    strncpy((*pTAG).title.as_mut_ptr(), Filename_WithoutPath(Filename_WithoutExt(sFilename.as_ptr())), mem::size_of_val(&(*pTAG).title));
                                    strncpy((*pTAG).artist.as_mut_ptr(), b"Raven Software\0".as_ptr() as *const c_char, mem::size_of_val(&(*pTAG).artist));
                                    strncpy((*pTAG).year.as_mut_ptr(), b"2002\0".as_ptr() as *const c_char, mem::size_of_val(&(*pTAG).year));
                                    strncpy((*pTAG).comment.as_mut_ptr(), va(b"%s %g\0".as_ptr() as *const c_char, b"mp3_vol\0".as_ptr() as *const c_char, fMaxVol), mem::size_of_val(&(*pTAG).comment));
                                    strncpy((*pTAG).album.as_mut_ptr(), va(b"%s %d\0".as_ptr() as *const c_char, b"uncomp\0".as_ptr() as *const c_char, iActualUnpackedSize), mem::size_of_val(&(*pTAG).album));

                                    if FS_Write(pTAG as *const c_void, mem::size_of::<id3v1_1>() as c_int, f) != 0 { // NZ = success
                                        iFilesUpdated += 1;
                                    } else {
                                        Com_Printf(b"*********** Failed write to file \"%s\"!\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                                        iErrors += 1;
                                        strErrors.push_str(&format!("Failed to write: \"{}\"\n", std::ffi::CStr::from_ptr(sFilename.as_ptr()).to_string_lossy()));
                                    }
                                } else {
                                    Com_Printf(b"*********** Failed write to file \"%s\"!\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                                    iErrors += 1;
                                    strErrors.push_str(&format!("Failed to write: \"{}\"\n", std::ffi::CStr::from_ptr(sFilename.as_ptr()).to_string_lossy()));
                                }
                                FS_FCloseFile(f);
                            } else {
                                Com_Printf(b"*********** Failed to re-open for write \"%s\"!\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                                iErrors += 1;
                                strErrors.push_str(&format!("Failed to re-open for write: \"{}\"\n", std::ffi::CStr::from_ptr(sFilename.as_ptr()).to_string_lossy()));
                            }
                        } else {
                            Com_Error(ERR_DROP, b"******* This MP3 should be deleted: \"%s\"\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                        }
                    } else {
                        Com_Printf(b"*********** File was not a valid MP3!: \"%s\"\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                        iErrors += 1;
                        strErrors.push_str(&format!("Not game-legal MP3 format: \"{}\"\n", std::ffi::CStr::from_ptr(sFilename.as_ptr()).to_string_lossy()));
                    }
                } else {
                    Com_Printf(b" ( OK )\n\0".as_ptr() as *const c_char);
                }

                FS_FreeFile(pbData as *mut c_void);
            }

            i += 1;
        }
        FS_FreeFileList(sysFiles);
        FS_FreeFileList(dirFiles);
    }
}

// this console-function is for development purposes, and makes sure that sound/*.mp3 /s have tags in them
//  specifying stuff like their max volume (and uncompressed size) etc...
//
pub extern "C" fn S_MP3_CalcVols_f() {
    unsafe {
        let mut sStartDir: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        sStartDir[0] = b's' as c_char;
        sStartDir[1] = b'o' as c_char;
        sStartDir[2] = b'u' as c_char;
        sStartDir[3] = b'n' as c_char;
        sStartDir[4] = b'd' as c_char;
        sStartDir[5] = 0;

        let sUsage: &[u8] = b"Usage: mp3_calcvols [-rescan] <startdir>\ne.g. mp3_calcvols sound/chars\0";

        if Cmd_Argc() == 1 || Cmd_Argc() > 4 { // 3 optional arguments
            Com_Printf(sUsage.as_ptr() as *const c_char);
            return;
        }

        S_StopAllSounds();

        qbForceRescan = false;
        qbForceStereo = false;
        iFilesFound = 0;
        iFilesUpdated = 0;
        iErrors = 0;
        strErrors.clear();

        let mut i: c_int = 1;
        while i < Cmd_Argc() {
            if *Cmd_Argv(i) as u8 == b'-' {
                if Q_stricmp(Cmd_Argv(i), b"-rescan\0".as_ptr() as *const c_char) == 0 {
                    qbForceRescan = true;
                } else if Q_stricmp(Cmd_Argv(i), b"-stereo\0".as_ptr() as *const c_char) == 0 {
                    qbForceStereo = true;
                } else {
                    // unknown switch...
                    //
                    Com_Printf(sUsage.as_ptr() as *const c_char);
                    return;
                }
                i += 1;
                continue;
            }
            strcpy(sStartDir.as_mut_ptr(), Cmd_Argv(i));
            i += 1;
        }

        Com_Printf(va(b"Starting Scan for Updates in Dir: %s\n\0".as_ptr() as *const c_char, sStartDir.as_ptr()));
        R_CheckMP3s(sStartDir.as_ptr());

        Com_Printf(b"\n%d files found/scanned, %d files updated      ( %d errors total)\n\0".as_ptr() as *const c_char, iFilesFound, iFilesUpdated, iErrors);

        if iErrors != 0 {
            Com_Printf(b"\nBad Files:\n%s\n\0".as_ptr() as *const c_char, strErrors.as_ptr() as *const c_char);
        }
    }
}

// adjust filename for foreign languages and WAV/MP3 issues.
//
// returns qfalse if failed to load, else fills in *pData
//
extern "C" {
    pub static mut com_buildScript: *mut cvar_t;
}

fn S_LoadSound_FileLoadAndNameAdjuster(psFilename: *mut c_char, pData: *mut *mut u8, piSize: *mut c_int, iNameStrlen: c_int) -> bool {
    unsafe {
        let mut psVoice: *mut c_char = strstr(psFilename, b"chars\0".as_ptr() as *const c_char);
        if !psVoice.is_null() {
            // cache foreign voices...
            //
            if (*com_buildScript).integer != 0 {
                let mut hFile: fileHandle_t;
                //German
                strncpy(psVoice, b"chr_d\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
                FS_FOpenFileRead(psFilename, &mut hFile, false as c_int); //cache the wav
                if hFile == 0 {
                    *psFilename.offset((iNameStrlen - 3) as isize) = b'm' as c_char;
                    *psFilename.offset((iNameStrlen - 2) as isize) = b'p' as c_char;
                    *psFilename.offset((iNameStrlen - 1) as isize) = b'3' as c_char;
                    FS_FOpenFileRead(psFilename, &mut hFile, false as c_int); //cache the mp3
                }
                if hFile != 0 {
                    FS_FCloseFile(hFile);
                }
                *psFilename.offset((iNameStrlen - 3) as isize) = b'w' as c_char;
                *psFilename.offset((iNameStrlen - 2) as isize) = b'a' as c_char;
                *psFilename.offset((iNameStrlen - 1) as isize) = b'v' as c_char;

                //French
                strncpy(psVoice, b"chr_f\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
                FS_FOpenFileRead(psFilename, &mut hFile, false as c_int); //cache the wav
                if hFile == 0 {
                    *psFilename.offset((iNameStrlen - 3) as isize) = b'm' as c_char;
                    *psFilename.offset((iNameStrlen - 2) as isize) = b'p' as c_char;
                    *psFilename.offset((iNameStrlen - 1) as isize) = b'3' as c_char;
                    FS_FOpenFileRead(psFilename, &mut hFile, false as c_int); //cache the mp3
                }
                if hFile != 0 {
                    FS_FCloseFile(hFile);
                }
                *psFilename.offset((iNameStrlen - 3) as isize) = b'w' as c_char;
                *psFilename.offset((iNameStrlen - 2) as isize) = b'a' as c_char;
                *psFilename.offset((iNameStrlen - 1) as isize) = b'v' as c_char;

                //Spanish
                strncpy(psVoice, b"chr_e\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
                FS_FOpenFileRead(psFilename, &mut hFile, false as c_int); //cache the wav
                if hFile == 0 {
                    *psFilename.offset((iNameStrlen - 3) as isize) = b'm' as c_char;
                    *psFilename.offset((iNameStrlen - 2) as isize) = b'p' as c_char;
                    *psFilename.offset((iNameStrlen - 1) as isize) = b'3' as c_char;
                    FS_FOpenFileRead(psFilename, &mut hFile, false as c_int); //cache the mp3
                }
                if hFile != 0 {
                    FS_FCloseFile(hFile);
                }
                *psFilename.offset((iNameStrlen - 3) as isize) = b'w' as c_char;
                *psFilename.offset((iNameStrlen - 2) as isize) = b'a' as c_char;
                *psFilename.offset((iNameStrlen - 1) as isize) = b'v' as c_char;

                strncpy(psVoice, b"chars\0".as_ptr() as *const c_char, 5); //put it back to chars
            }

            // account for foreign voices...
            //
            let s_language_ptr: *mut cvar_t = s_language;
            if !s_language_ptr.is_null() && stricmp(b"DEUTSCH\0".as_ptr() as *const c_char, (*s_language_ptr).string) == 0 {
                strncpy(psVoice, b"chr_d\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            } else if !s_language_ptr.is_null() && stricmp(b"FRANCAIS\0".as_ptr() as *const c_char, (*s_language_ptr).string) == 0 {
                strncpy(psVoice, b"chr_f\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            } else if !s_language_ptr.is_null() && stricmp(b"ESPANOL\0".as_ptr() as *const c_char, (*s_language_ptr).string) == 0 {
                strncpy(psVoice, b"chr_e\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            } else {
                psVoice = ptr::null_mut(); // use this ptr as a flag as to whether or not we substituted with a foreign version
            }
        }

        *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try WAV
        if (*pData).is_null() {
            *psFilename.offset((iNameStrlen - 3) as isize) = b'm' as c_char;
            *psFilename.offset((iNameStrlen - 2) as isize) = b'p' as c_char;
            *psFilename.offset((iNameStrlen - 1) as isize) = b'3' as c_char;
            *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try MP3

            if (*pData).is_null() {
                //hmmm, not found, ok, maybe we were trying a foreign noise ("arghhhhh.mp3" that doesn't matter?) but it
                // was missing?   Can't tell really, since both types are now in sound/chars. Oh well, fall back to English for now...

                if !psVoice.is_null() { // were we trying to load foreign?
                    // yep, so fallback to re-try the english...
                    //
                    #[cfg(not(feature = "FINAL_BUILD"))]
                    Com_Printf(b"^3Foreign file missing: \"%s\"! (using English...)\n\0".as_ptr() as *const c_char, psFilename);

                    strncpy(psVoice, b"chars\0".as_ptr() as *const c_char, 5);

                    *psFilename.offset((iNameStrlen - 3) as isize) = b'w' as c_char;
                    *psFilename.offset((iNameStrlen - 2) as isize) = b'a' as c_char;
                    *psFilename.offset((iNameStrlen - 1) as isize) = b'v' as c_char;
                    *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try English WAV
                    if (*pData).is_null() {
                        *psFilename.offset((iNameStrlen - 3) as isize) = b'm' as c_char;
                        *psFilename.offset((iNameStrlen - 2) as isize) = b'p' as c_char;
                        *psFilename.offset((iNameStrlen - 1) as isize) = b'3' as c_char;
                        *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try English MP3
                    }
                }

                if (*pData).is_null() {
                    return false; // sod it, give up...
                }
            }
        }

        true
    }
}

// returns qtrue if this dir is allowed to keep loaded MP3s, else qfalse if they should be WAV'd instead...
//
// note that this is passed the original, un-language'd name
//
// (I was going to remove this, but on kejim_post I hit an assert because someone had got an ambient sound when the
//  perimter fence goes online that was an MP3, then it tried to get added as looping. Presumably it sounded ok or
//  they'd have noticed, but we therefore need to stop other levels using those. "sound/ambience" I can check for,
//  but doors etc could be anything. Sigh...)
//
fn S_LoadSound_DirIsAllowedToKeepMP3s(psFilename: *const c_char) -> bool {
    unsafe {
        let psAllowedDirs: [&[u8]; 1] = [
            b"sound/chars/\0",
            //  b"sound/chr_d/"  // no need for this now, or any other language, since we'll always compare against english
        ];

        let mut i: usize = 0;
        while i < psAllowedDirs.len() {
            if strnicmp(psFilename, psAllowedDirs[i].as_ptr() as *const c_char, strlen(psAllowedDirs[i].as_ptr() as *const c_char)) == 0 {
                return true; // found a dir that's allowed to keep MP3s
            }
            i += 1;
        }

        false
    }
}

/*
==============
S_LoadSound

The filename may be different than sfx->name in the case
of a forced fallback of a player specific sound  (or of a wav/mp3 substitution now -Ste)
==============
*/
static mut gbInsideLoadSound: bool = false;

fn S_LoadSound_Actual(sfx: *mut sfx_t) -> bool {
    unsafe {
        let mut data: *mut u8;
        let mut samples: *mut i16;
        let mut info: wavinfo_t;
        let mut size: c_int;
        let mut psExt: *mut c_char;
        let mut Buffer: c_int;

        let len: c_int = strlen((*sfx).sSoundName.as_ptr() as *const c_char) as c_int;
        if len < 5 {
            return false;
        }

        // player specific sounds are never directly loaded...
        //
        if *(*sfx).sSoundName.as_ptr() as u8 == b'*' {
            return false;
        }
        // make up a local filename to try wav/mp3 substitutes...
        //
        let mut sLoadName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        Q_strncpyz(sLoadName.as_mut_ptr(), (*sfx).sSoundName.as_ptr(), MAX_QPATH as c_int);
        strlwr(sLoadName.as_mut_ptr());
        //
        // Ensure name has an extension (which it must have, but you never know), and get ptr to it...
        //
        psExt = &mut sLoadName[(strlen(sLoadName.as_ptr() as *const c_char) - 4) as usize] as *mut c_char;
        if *psExt as u8 != b'.' {
            //Com_Printf( b"WARNING: soundname '%s' does not have 3-letter extension\n", sLoadName.as_ptr());
            COM_DefaultExtension(sLoadName.as_mut_ptr(), MAX_QPATH as c_int, b".wav\0".as_ptr() as *const c_char); // so psExt below is always valid
            psExt = &mut sLoadName[(strlen(sLoadName.as_ptr() as *const c_char) - 4) as usize] as *mut c_char;
            //len = strlen(sLoadName.as_ptr() as *const c_char) as c_int;
        }

        if !S_LoadSound_FileLoadAndNameAdjuster(sLoadName.as_mut_ptr(), &mut data, &mut size, len) {
            return false;
        }

        SND_TouchSFX(sfx);
        //=========
        if strnicmp(psExt, b".mp3\0".as_ptr() as *const c_char, 4) == 0 {
            // load MP3 file instead...
            //
            if MP3_IsValid(sLoadName.as_ptr(), data, size, false) {
                let iRawPCMDataSize: c_int = MP3_GetUnpackedSize(sLoadName.as_ptr(), data, size, false, false);

                if S_LoadSound_DirIsAllowedToKeepMP3s((*sfx).sSoundName.as_ptr())    // NOT sLoadName, this uses original un-languaged name
                    &&
                    MP3Stream_InitFromFile(sfx, data, size, sLoadName.as_ptr(), iRawPCMDataSize + 2304 /* + 1 MP3 frame size, jic */, false)
                {
                    //              Com_DPrintf(b"(Keeping file \"%s\" as MP3)\n", sLoadName.as_ptr());

                    if s_UseOpenAL != 0 {
                        // Create space for lipsync data (4 lip sync values per streaming AL buffer)
                        if !strstr((*sfx).sSoundName.as_ptr(), b"chars\0".as_ptr() as *const c_char).is_null() {
                            (*sfx).lipSyncData = Z_Malloc(16, TAG_SND_RAWDATA, false) as *mut c_char;
                        } else {
                            (*sfx).lipSyncData = ptr::null_mut();
                        }
                    }
                } else {
                    // small file, not worth keeping as MP3 since it would increase in size (with MP3 header etc)...
                    //
                    Com_DPrintf(b"S_LoadSound: Unpacking MP3 file \"%s\" to wav.\n\0".as_ptr() as *const c_char, sLoadName.as_ptr());
                    //
                    // unpack and convert into WAV...
                    //
                    {
                        let pbUnpackBuffer: *mut u8 = Z_Malloc((iRawPCMDataSize + 10 + 2304) as usize, TAG_TEMP_WORKSPACE, false) as *mut u8; // won't return if fails

                        {
                            let mut iResultBytes: c_int = MP3_UnpackRawPCM(sLoadName.as_ptr(), data, size, pbUnpackBuffer);

                            if iResultBytes != iRawPCMDataSize {
                                Com_Printf(b"^3**** MP3 %s final unpack size %d different to previous value %d\n\0".as_ptr() as *const c_char, sLoadName.as_ptr(), iResultBytes, iRawPCMDataSize);
                                //assert (iResultBytes == iRawPCMDataSize);
                            }

                            // fake up a WAV structure so I can use the other post-load sound code such as volume calc for lip-synching
                            //
                            // (this is a bit crap really, but it lets me drop through into existing code)...
                            //
                            MP3_FakeUpWAVInfo_with_flag(sLoadName.as_ptr(), data, size, iResultBytes,
                                // these params are all references...
                                &mut info.format, &mut info.rate, &mut info.width, &mut info.channels, &mut info.samples, &mut info.dataofs,
                                false
                            );

                            S_LoadSound_Finalize(&mut info, sfx, pbUnpackBuffer);

                            // Open AL
                            if s_UseOpenAL != 0 {
                                if !strstr((*sfx).sSoundName.as_ptr(), b"chars\0".as_ptr() as *const c_char).is_null() {
                                    (*sfx).lipSyncData = Z_Malloc((((*sfx).iSoundLengthInSamples / 1000) + 1) as usize, TAG_SND_RAWDATA, false) as *mut c_char;
                                    S_PreProcessLipSync(sfx);
                                } else {
                                    (*sfx).lipSyncData = ptr::null_mut();
                                }

                                // Clear Open AL Error state
                                alGetError();

                                // Generate AL Buffer
                                alGenBuffers(1, &mut Buffer);
                                if alGetError() == AL_NO_ERROR {
                                    // Copy audio data to AL Buffer
                                    alBufferData(Buffer, AL_FORMAT_MONO16, (*sfx).pSoundData as *const c_void, (*sfx).iSoundLengthInSamples * 2, 22050);
                                    if alGetError() == AL_NO_ERROR {
                                        (*sfx).Buffer = Buffer;
                                        Z_Free((*sfx).pSoundData as *mut c_void);
                                        (*sfx).pSoundData = ptr::null_mut();
                                    }
                                }
                            }

                            Z_Free(pbUnpackBuffer as *mut c_void);
                        }
                    }
                }
            } else {
                // MP3_IsValid() will already have printed any errors via Com_Printf at this point...
                //
                FS_FreeFile(data as *mut c_void);
                return false;
            }
        } else {
            // loading a WAV, presumably...

            //=========

            info = GetWavinfo(sLoadName.as_ptr(), data, size);
            if info.channels != 1 {
                Com_Printf(b"%s is a stereo wav file\n\0".as_ptr() as *const c_char, sLoadName.as_ptr());
                FS_FreeFile(data as *mut c_void);
                return false;
            }

            /*      if ( info.width == 1 ) {
                        Com_Printf(b"^3WARNING: %s is a 8 bit wav file\n", sLoadName.as_ptr());
                    }

                    if ( info.rate != 22050 ) {
                        Com_Printf(b"^3WARNING: %s is not a 22kHz wav file\n", sLoadName.as_ptr());
                    }
            */
            samples = Z_Malloc((info.samples as usize) * mem::size_of::<i16>() * 2, TAG_TEMP_WORKSPACE, false) as *mut i16;

            (*sfx).eSoundCompressionMethod = 16; // ct_16
            (*sfx).iSoundLengthInSamples = info.samples;
            (*sfx).pSoundData = ptr::null_mut();
            ResampleSfx(sfx, info.rate, info.width, data.offset(info.dataofs as isize));

            // Open AL
            if s_UseOpenAL != 0 {
                if !strstr((*sfx).sSoundName.as_ptr(), b"chars\0".as_ptr() as *const c_char).is_null() || !strstr((*sfx).sSoundName.as_ptr(), b"CHARS\0".as_ptr() as *const c_char).is_null() {
                    (*sfx).lipSyncData = Z_Malloc((((*sfx).iSoundLengthInSamples / 1000) + 1) as usize, TAG_SND_RAWDATA, false) as *mut c_char;
                    S_PreProcessLipSync(sfx);
                } else {
                    (*sfx).lipSyncData = ptr::null_mut();
                }

                // Clear Open AL Error State
                alGetError();

                // Generate AL Buffer
                alGenBuffers(1, &mut Buffer);
                if alGetError() == AL_NO_ERROR {
                    // Copy audio data to AL Buffer
                    alBufferData(Buffer, AL_FORMAT_MONO16, (*sfx).pSoundData as *const c_void, (*sfx).iSoundLengthInSamples * 2, 22050);
                    if alGetError() == AL_NO_ERROR {
                        // Store AL Buffer in sfx struct, and release sample data
                        (*sfx).Buffer = Buffer;
                        Z_Free((*sfx).pSoundData as *mut c_void);
                        (*sfx).pSoundData = ptr::null_mut();
                    }
                }
            }

            Z_Free(samples as *mut c_void);
        }

        FS_FreeFile(data as *mut c_void);

        true
    }
}

// wrapper function for above so I can guarantee that we don't attempt any audio-dumping during this call because
//  of a z_malloc() fail recovery...
//
pub extern "C" fn S_LoadSound(sfx: *mut sfx_t) -> bool {
    unsafe {
        gbInsideLoadSound = true; // !!!!!!!!!!!!!

        let bReturn: bool = S_LoadSound_Actual(sfx);

        gbInsideLoadSound = false; // !!!!!!!!!!!!!

        bReturn
    }
}

/*
    Precalculate the lipsync values for the whole sample
*/
fn S_PreProcessLipSync_internal(sfx: *mut sfx_t) {
    unsafe {
        let mut i: c_int;
        let mut j: c_int;
        let mut sample: c_int;
        let mut sampleTotal: c_int = 0;

        j = 0;
        i = 0;
        while i < (*sfx).iSoundLengthInSamples {
            sample = LittleShort(*(((*sfx).pSoundData).offset(i as isize)));

            sample = sample >> 8;
            sampleTotal += sample * sample;
            if ((i + 100) % 1000) == 0 {
                sampleTotal /= 10;

                if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_1).value {
                    // tell the scripts that are relying on this that we are still going, but actually silent right now.
                    sample = -1;
                } else if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_2).value {
                    sample = 1;
                } else if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_3).value {
                    sample = 2;
                } else if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_4).value {
                    sample = 3;
                } else {
                    sample = 4;
                }

                *(((*sfx).lipSyncData).offset(j as isize)) = sample as c_char;
                j += 1;

                sampleTotal = 0;
            }

            i += 100;
        }

        if (i % 1000) == 0 {
            return;
        }

        i -= 100;
        i = i % 1000;
        i = i / 100;
        // Process last < 1000 samples
        if i != 0 {
            sampleTotal /= i;
        } else {
            sampleTotal = 0;
        }

        if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_1).value {
            // tell the scripts that are relying on this that we are still going, but actually silent right now.
            sample = -1;
        } else if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_2).value {
            sample = 1;
        } else if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_3).value {
            sample = 2;
        } else if (sampleTotal as f32) < (*sfx).fVolRange * (*s_lip_threshold_4).value {
            sample = 3;
        } else {
            sample = 4;
        }

        *(((*sfx).lipSyncData).offset(j as isize)) = sample as c_char;
    }
}

// strncpy helper function
fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char {
    unsafe {
        let mut i: usize = 0;
        while i < n {
            *dest.offset(i as isize) = *src.offset(i as isize);
            if *src.offset(i as isize) == 0 {
                break;
            }
            i += 1;
        }
        dest
    }
}
