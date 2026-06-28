// snd_mem.c: sound caching

use core::ffi::{c_int, c_char, c_void};
use std::ffi::{CStr, CString};
use libc;

// Porting note: extern "C" declarations for FFI functions
extern "C" {
    // From snd_local.h and related
    static mut s_knownSfx: *mut sfx_t;
    static mut s_numSfx: c_int;

    // Console variables
    static mut s_lip_threshold_1: *mut cvar_t;
    static mut s_lip_threshold_2: *mut cvar_t;
    static mut s_lip_threshold_3: *mut cvar_t;
    static mut s_lip_threshold_4: *mut cvar_t;
    static mut com_buildScript: *mut cvar_t;
    static mut s_language: *mut cvar_t;

    // Global audio device state
    static mut dma: dma_t;

    // OpenAL flag
    static s_UseOpenAL: c_int;

    // Memory management
    fn SND_malloc(size: c_int, sfx: *mut sfx_t) -> *mut c_void;
    fn SND_TouchSFX(sfx: *mut sfx_t);
    fn SND_FreeOldestSound();

    // Byte swapping
    fn LittleShort(x: c_int) -> c_int;

    // File system
    fn FS_ListFiles(
        path: *const c_char,
        ext: *const c_char,
        numfiles: *mut c_int,
    ) -> *mut *mut c_char;
    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn FS_FOpenFileWrite(name: *const c_char) -> fileHandle_t;
    fn FS_FOpenFileRead(name: *const c_char, f: *mut fileHandle_t, unique_only: c_int);
    fn FS_Write(buf: *const c_void, len: c_int, f: fileHandle_t) -> c_int;
    fn FS_FCloseFile(f: fileHandle_t);
    fn FS_FreeFile(buf: *mut c_void);
    fn FS_FreeFileList(list: *mut *mut c_char);

    // Memory allocation
    fn Z_Malloc(size: c_int, tag: c_int, zero: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Z_Size(ptr: *mut c_void) -> c_int;

    // String functions
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_strncpyz(dst: *mut c_char, src: *const c_char, dstsize: c_int);
    fn COM_DefaultExtension(path: *mut c_char, pathlen: c_int, ext: *const c_char);

    // Logging
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn va(fmt: *const c_char, ...) -> *const c_char;

    // Command console
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn S_StopAllSounds();

    // Sound functions
    fn S_FindName(name: *const c_char) -> *mut sfx_t;

    // MP3 functions
    fn MP3_IsValid(filename: *const c_char, data: *const c_void, size: c_int, stereo: c_int) -> c_int;
    fn MP3_GetUnpackedSize(
        filename: *const c_char,
        data: *const c_void,
        size: c_int,
        ignore_id3: c_int,
        stereo: c_int,
    ) -> c_int;
    fn MP3_UnpackRawPCM(
        filename: *const c_char,
        data: *const c_void,
        size: c_int,
        buf: *mut u8,
        stereo: c_int,
    ) -> c_int;
    fn MP3_FakeUpWAVInfo(
        filename: *const c_char,
        data: *const c_void,
        size: c_int,
        unpacked_size: c_int,
        format: *mut c_int,
        rate: *mut c_int,
        width: *mut c_int,
        channels: *mut c_int,
        samples: *mut c_int,
        dataofs: *mut c_int,
        stereo: c_int,
    ) -> c_int;
    fn MP3_ReadSpecialTagInfo(
        data: *const u8,
        size: c_int,
        tag: *mut *mut id3v1_1,
    ) -> c_int;
    fn MP3Stream_InitFromFile(
        sfx: *mut sfx_t,
        data: *const u8,
        size: c_int,
        filename: *const c_char,
        unpacked_size: c_int,
        stereo: c_int,
    ) -> c_int;

    // OpenAL functions
    fn alGetError() -> c_int;
    fn alGenBuffers(n: c_int, buffers: *mut u32);
    fn alBufferData(buffer: u32, format: c_int, data: *const c_void, size: c_int, freq: c_int);
}

// Stub types for unported dependencies
// Porting note: struct definitions based on oracle headers; some fields may be unused in this file
#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub resetString: *const c_char,
    pub latchedString: *const c_char,
    pub flags: c_int,
    pub modified: c_int, // qboolean
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

#[repr(C)]
pub struct sfx_t {
    pub pSoundData: *mut i16,
    pub bDefaultSound: c_int,
    pub bInMemory: c_int,
    pub iLastLevelUsedOn: c_int,
    pub eSoundCompressionMethod: c_int,
    pub pMP3StreamHeader: *mut MP3STREAM,
    pub iSoundLengthInSamples: c_int,
    pub sSoundName: [c_char; 256],
    pub iLastTimeUsed: c_int,
    pub fVolRange: f32,
    pub Buffer: u32,
    pub lipSyncData: *mut c_char,
    pub next: *mut sfx_t,
}

#[repr(C)]
pub struct dma_t {
    pub channels: c_int,
    pub samples: c_int,
    pub submission_chunk: c_int,
    pub samplebits: c_int,
    pub speed: c_int,
    pub buffer: *mut u8,
}

#[repr(C)]
pub struct MP3STREAM {
    // Placeholder
    _opaque: [u8; 0],
}

pub type fileHandle_t = c_int;

#[repr(C)]
pub struct id3v1_1 {
    pub id: [c_char; 3],
    pub title: [c_char; 30],
    pub artist: [c_char; 30],
    pub album: [c_char; 30],
    pub year: [c_char; 4],
    pub comment: [c_char; 28],
    pub zero: c_char,
    pub track: c_char,
    pub genre: c_char,
}

#[repr(C)]
pub struct wavinfo_t {
    pub format: c_int,
    pub rate: c_int,
    pub width: c_int,
    pub channels: c_int,
    pub samples: c_int,
    pub dataofs: c_int,
}

// Constants
const qtrue: c_int = 1;
const qfalse: c_int = 0;
const TAG_TEMP_WORKSPACE: c_int = 5;
const TAG_SND_RAWDATA: c_int = 6;
const MAX_QPATH: usize = 256;
const AL_FORMAT_MONO16: c_int = 0x1101;
const AL_NO_ERROR: c_int = 0;
const ERR_DROP: c_int = 0;

// Porting note: String constants used in the original code
const S_COLOR_YELLOW: &str = "^3";

// WAV loading

// Porting note: These are module-level statics representing the state of WAV file parsing
static mut data_p: *mut u8 = std::ptr::null_mut();
static mut iff_end: *mut u8 = std::ptr::null_mut();
static mut last_chunk: *mut u8 = std::ptr::null_mut();
static mut iff_data: *mut u8 = std::ptr::null_mut();
static mut iff_chunk_len: c_int = 0;

// ===============================================================================
// WAV loading
// ===============================================================================

unsafe fn GetLittleShort() -> c_int {
    let mut val: c_int = 0;
    val = *data_p as c_int;
    val = (val as c_int) + ((*data_p.offset(1) as c_int) << 8);
    data_p = data_p.offset(2);
    val
}

unsafe fn GetLittleLong() -> c_int {
    let mut val: c_int = 0;
    val = *data_p as c_int;
    val = val + ((*data_p.offset(1) as c_int) << 8);
    val = val + ((*data_p.offset(2) as c_int) << 16);
    val = val + ((*data_p.offset(3) as c_int) << 24);
    data_p = data_p.offset(4);
    val
}

unsafe fn FindNextChunk(name: *const c_char) {
    loop {
        data_p = last_chunk;

        if data_p >= iff_end {
            // didn't find the chunk
            data_p = std::ptr::null_mut();
            return;
        }

        data_p = data_p.offset(4);
        iff_chunk_len = GetLittleLong();
        if iff_chunk_len < 0 {
            data_p = std::ptr::null_mut();
            return;
        }
        data_p = data_p.offset(-8);
        last_chunk = data_p.offset(8 + ((iff_chunk_len + 1) & !1) as isize);
        if libc::strncmp(data_p as *const c_char, name, 4) == 0 {
            return;
        }
    }
}

unsafe fn FindChunk(name: *const c_char) {
    last_chunk = iff_data;
    FindNextChunk(name);
}

unsafe fn DumpChunks() {
    let mut str: [c_char; 5] = [0; 5];

    str[4] = 0;
    data_p = iff_data;
    loop {
        libc::memcpy(str.as_mut_ptr() as *mut c_void, data_p as *const c_void, 4);
        data_p = data_p.offset(4);
        iff_chunk_len = GetLittleLong();
        Com_Printf(
            b"0x%x : %s (%d)\n\0".as_ptr() as *const c_char,
            data_p.offset(-4) as c_int,
            str.as_ptr(),
            iff_chunk_len,
        );
        data_p = data_p.offset(((iff_chunk_len + 1) & !1) as isize);
        if !(data_p < iff_end) {
            break;
        }
    }
}

// ============
// GetWavinfo
// ============
pub unsafe fn GetWavinfo(name: *const c_char, wav: *mut u8, wavlength: c_int) -> wavinfo_t {
    let mut info: wavinfo_t = wavinfo_t {
        format: 0,
        rate: 0,
        width: 0,
        channels: 0,
        samples: 0,
        dataofs: 0,
    };
    let mut samples: c_int;

    libc::memset(&mut info as *mut wavinfo_t as *mut c_void, 0, std::mem::size_of::<wavinfo_t>());

    if wav.is_null() {
        return info;
    }

    iff_data = wav;
    iff_end = wav.offset(wavlength as isize);

    // find "RIFF" chunk
    FindChunk(b"RIFF\0".as_ptr() as *const c_char);
    if data_p.is_null() || libc::strncmp(data_p.offset(8) as *const c_char, b"WAVE\0".as_ptr() as *const c_char, 4) != 0 {
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
    data_p = data_p.offset(6);
    info.width = GetLittleShort() / 8;

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
            Com_Error(
                ERR_DROP,
                b"Sound %s has a bad loop length\0".as_ptr() as *const c_char,
                name,
            );
        }
    } else {
        info.samples = samples;
    }

    info.dataofs = (data_p as isize - wav as isize) as c_int;

    info
}

// ================
// ResampleSfx
//
// resample / decimate to the current source rate
// ================
pub unsafe fn ResampleSfx(sfx: *mut sfx_t, iInRate: c_int, iInWidth: c_int, pData: *mut u8) {
    let mut iOutCount: c_int;
    let mut iSrcSample: c_int;
    let mut fStepScale: f32;
    let mut i: c_int;
    let mut iSample: c_int;
    let mut uiSampleFrac: c_int;
    let mut uiFracStep: c_int;

    fStepScale = iInRate as f32 / dma.speed as f32; // this is usually 0.5, 1, or 2

    // When stepscale is > 1 (we're downsampling), we really ought to run a low pass filter on the samples

    iOutCount = ((*sfx).iSoundLengthInSamples as f32 / fStepScale) as c_int;
    (*sfx).iSoundLengthInSamples = iOutCount;

    (*sfx).pSoundData = SND_malloc((*sfx).iSoundLengthInSamples * 2, sfx) as *mut i16;

    (*sfx).fVolRange = 0.0;
    uiSampleFrac = 0;
    uiFracStep = (fStepScale * 256.0) as c_int;

    i = 0;
    while i < (*sfx).iSoundLengthInSamples {
        iSrcSample = uiSampleFrac >> 8;
        uiSampleFrac += uiFracStep;
        if iInWidth == 2 {
            iSample = LittleShort((*(pData as *mut i16).offset(iSrcSample as isize) as c_int));
        } else {
            iSample = (((*pData.offset(iSrcSample as isize) as c_int) - 128) as c_int) << 8;
        }

        *(*sfx).pSoundData.offset(i as isize) = iSample as i16;

        // work out max vol for this sample...
        //
        let mut abs_sample = iSample;
        if abs_sample < 0 {
            abs_sample = -abs_sample;
        }
        if (*sfx).fVolRange < (abs_sample >> 8) as f32 {
            (*sfx).fVolRange = (abs_sample >> 8) as f32;
        }

        i += 1;
    }
}

// =============================================================================

pub unsafe fn S_LoadSound_Finalize(info: *mut wavinfo_t, sfx: *mut sfx_t, data: *mut u8) {
    let stepscale: f32 = (*info).rate as f32 / dma.speed as f32;
    let mut len: c_int = ((*info).samples as f32 / stepscale) as c_int;

    len *= (*info).width;

    (*sfx).eSoundCompressionMethod = 0; // ct_16
    (*sfx).iSoundLengthInSamples = (*info).samples;
    ResampleSfx(sfx, (*info).rate, (*info).width, data.offset((*info).dataofs as isize));
}

// maybe I'm re-inventing the wheel, here, but I can't see any functions that already do this, so...
//
pub unsafe fn Filename_WithoutPath(psFilename: *const c_char) -> *mut c_char {
    static mut sString: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    let p = libc::strrchr(psFilename, b'\\' as c_int);

    let start = if p.is_null() {
        psFilename
    } else {
        p.offset(1)
    };

    libc::strcpy(sString.as_mut_ptr(), start);

    sString.as_mut_ptr()
}

// returns (eg) "\dir\name" for "\dir\name.bmp"
//
pub unsafe fn Filename_WithoutExt(psFilename: *const c_char) -> *mut c_char {
    static mut sString: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    libc::strcpy(sString.as_mut_ptr(), psFilename);

    let p = libc::strrchr(sString.as_mut_ptr(), b'.' as c_int);
    let p2 = libc::strrchr(sString.as_mut_ptr(), b'\\' as c_int);

    // special check, make sure the first suffix we found from the end wasn't just a directory suffix (eg on a path'd filename with no extension anyway)
    //
    if !p.is_null() && (p2.is_null() || (!p2.is_null() && p > p2)) {
        *p = 0;
    }

    sString.as_mut_ptr()
}

static mut iFilesFound: c_int = 0;
static mut iFilesUpdated: c_int = 0;
static mut iErrors: c_int = 0;
static mut qbForceRescan: c_int = 0;
static mut qbForceStereo: c_int = 0;
static mut strErrors: String = String::new();

unsafe fn R_CheckMP3s(psDir: *const c_char) {
    // Com_Printf(va("Scanning Dir: %s\n",psDir));
    Com_Printf(b".\0".as_ptr() as *const c_char); // stops useful info scrolling off screen

    let mut sysFiles: *mut *mut c_char;
    let mut dirFiles: *mut *mut c_char;
    let mut numSysFiles: c_int = 0;
    let mut numdirs: c_int = 0;
    let mut i: c_int;

    dirFiles = FS_ListFiles(psDir, b"/\0".as_ptr() as *const c_char, &mut numdirs);
    if numdirs > 2 {
        i = 2;
        while i < numdirs {
            let mut sDirName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
            libc::sprintf(
                sDirName.as_mut_ptr(),
                b"%s\\%s\0".as_ptr() as *const c_char,
                psDir,
                *dirFiles.offset(i as isize),
            );
            R_CheckMP3s(sDirName.as_ptr());
            i += 1;
        }
    }

    sysFiles = FS_ListFiles(psDir, b".mp3\0".as_ptr() as *const c_char, &mut numSysFiles);
    i = 0;
    while i < numSysFiles {
        let mut sFilename: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        libc::sprintf(
            sFilename.as_mut_ptr(),
            b"%s\\%s\0".as_ptr() as *const c_char,
            psDir,
            *sysFiles.offset(i as isize),
        );

        Com_Printf(
            b"%sFound file: %s\0".as_ptr() as *const c_char,
            if i == 0 { b"\n\0".as_ptr() as *const c_char } else { b"\0".as_ptr() as *const c_char },
            sFilename.as_ptr(),
        );

        iFilesFound += 1;

        // read it in...
        //
        let mut pbData: *mut u8 = std::ptr::null_mut();
        let iSize: c_int = FS_ReadFile(sFilename.as_ptr(), &mut (pbData as *mut c_void));

        if !pbData.is_null() {
            let mut pTAG: *mut id3v1_1;

            // do NOT check 'qbForceRescan' here as an opt, because we need to actually fill in 'pTAG' if there is one...
            //
            let qbTagNeedsUpdating: c_int =
                if MP3_ReadSpecialTagInfo(pbData, iSize, &mut pTAG) != 0 { qfalse } else { qtrue };

            if pTAG.is_null() || qbTagNeedsUpdating != qfalse || qbForceRescan != qfalse {
                Com_Printf(b" ( Updating )\n\0".as_ptr() as *const c_char);

                // I need to scan this file to get the volume...
                //
                // For EF1 I used a temp sfx_t struct, but I can't do that now with this new alloc scheme,
                //	I have to ask for it legally, so I'll keep re-using one, and restoring it's name after use.
                //	(slightly dodgy, but works ok if no-one else changes stuff)
                //
                // sfx_t SFX = {0};
                // extern sfx_t *S_FindName( const char *name );
                //
                static mut pSFX: *mut sfx_t = std::ptr::null_mut();
                const sReservedSFXEntrynameForMP3: &[u8] = b"reserved_for_mp3";

                if pSFX.is_null() {
                    // once only
                    pSFX = S_FindName(b"reserved_for_mp3\0".as_ptr() as *const c_char); // always returns, else ERR_FATAL
                }

                if MP3_IsValid(sFilename.as_ptr(), pbData as *const c_void, iSize, qfalse) != 0 {
                    let mut info: wavinfo_t = wavinfo_t {
                        format: 0,
                        rate: 0,
                        width: 0,
                        channels: 0,
                        samples: 0,
                        dataofs: 0,
                    };

                    let iRawPCMDataSize: c_int =
                        MP3_GetUnpackedSize(sFilename.as_ptr(), pbData as *const c_void, iSize, qtrue, qbForceStereo);

                    if iRawPCMDataSize != 0 {
                        // should always be true, unless file is fucked, in which case, stop this conversion process
                        let mut fMaxVol: f32 = 128.0; // any old default
                        let mut iActualUnpackedSize: c_int = iRawPCMDataSize; // default, override later if not doing music

                        if qbForceStereo == qfalse {
                            // no point for stereo files, which are for music and therefore no lip-sync
                            let pbUnpackBuffer: *mut u8 =
                                Z_Malloc(iRawPCMDataSize + 10, TAG_TEMP_WORKSPACE, qfalse) as *mut u8; // won't return if fails

                            iActualUnpackedSize =
                                MP3_UnpackRawPCM(sFilename.as_ptr(), pbData as *const c_void, iSize, pbUnpackBuffer, qfalse);
                            if iActualUnpackedSize != iRawPCMDataSize {
                                Com_Error(
                                    ERR_DROP,
                                    b"******* Whoah! MP3 %s unpacked to %d bytes, but size calc said %d!\n\0".as_ptr()
                                        as *const c_char,
                                    sFilename.as_ptr(),
                                    iActualUnpackedSize,
                                    iRawPCMDataSize,
                                );
                            }

                            // fake up a WAV structure so I can use the other post-load sound code such as volume calc for lip-synching
                            //
                            MP3_FakeUpWAVInfo(
                                sFilename.as_ptr(),
                                pbData as *const c_void,
                                iSize,
                                iActualUnpackedSize,
                                // these params are all references...
                                &mut info.format,
                                &mut info.rate,
                                &mut info.width,
                                &mut info.channels,
                                &mut info.samples,
                                &mut info.dataofs,
                                qfalse,
                            );

                            S_LoadSound_Finalize(&mut info, pSFX, pbUnpackBuffer); // all this just for lipsynch. Oh well.

                            fMaxVol = (*pSFX).fVolRange;

                            // free sfx->data...
                            //
                            {
                                const INT_MIN: c_int = -2147483647 - 1; /* minimum (signed) int value */

                                (*pSFX).iLastTimeUsed = INT_MIN; // force this to be oldest sound file, therefore disposable...
                                (*pSFX).bInMemory = qtrue;
                                SND_FreeOldestSound(); // ... and do the disposal

                                // now set our temp SFX struct back to default name so nothing else accidentally uses it...
                                //
                                libc::strcpy(
                                    (*pSFX).sSoundName.as_mut_ptr() as *mut c_char,
                                    b"reserved_for_mp3\0".as_ptr() as *const c_char,
                                );
                                (*pSFX).bDefaultSound = qfalse;
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
                            let iWritten: c_int = FS_Write(
                                pbData as *const c_void,
                                iSize - (if !pTAG.is_null() { std::mem::size_of::<id3v1_1>() } else { 0 }) as c_int,
                                f,
                            );

                            if iWritten != 0 {
                                // make up a new tag if we didn't find one in the original file...
                                //
                                let mut TAG: id3v1_1 = id3v1_1 {
                                    id: [0; 3],
                                    title: [0; 30],
                                    artist: [0; 30],
                                    album: [0; 30],
                                    year: [0; 4],
                                    comment: [0; 28],
                                    zero: 0,
                                    track: 0,
                                    genre: 0,
                                };

                                if pTAG.is_null() {
                                    pTAG = &mut TAG;
                                    libc::memset(&mut TAG as *mut id3v1_1 as *mut c_void, 0, std::mem::size_of::<id3v1_1>());
                                    libc::strncpy((*pTAG).id.as_mut_ptr(), b"TAG\0".as_ptr() as *const c_char, 3);
                                }

                                libc::strncpy(
                                    (*pTAG).title.as_mut_ptr(),
                                    Filename_WithoutPath(Filename_WithoutExt(sFilename.as_ptr())),
                                    std::mem::size_of_val(&(*pTAG).title),
                                );
                                libc::strncpy(
                                    (*pTAG).artist.as_mut_ptr(),
                                    b"Raven Software\0".as_ptr() as *const c_char,
                                    std::mem::size_of_val(&(*pTAG).artist),
                                );
                                libc::strncpy(
                                    (*pTAG).year.as_mut_ptr(),
                                    b"2002\0".as_ptr() as *const c_char,
                                    std::mem::size_of_val(&(*pTAG).year),
                                );
                                // Note: using va() function to format strings like original code
                                let comment_str_ptr = va(b"#MAXVOL %g\0".as_ptr() as *const c_char, fMaxVol as c_int);
                                libc::strncpy(
                                    (*pTAG).comment.as_mut_ptr(),
                                    comment_str_ptr,
                                    std::mem::size_of_val(&(*pTAG).comment),
                                );
                                let album_str_ptr = va(b"#UNCOMP %d\0".as_ptr() as *const c_char, iActualUnpackedSize);
                                libc::strncpy(
                                    (*pTAG).album.as_mut_ptr(),
                                    album_str_ptr,
                                    std::mem::size_of_val(&(*pTAG).album),
                                );

                                if FS_Write(pTAG as *const c_void, std::mem::size_of::<id3v1_1>() as c_int, f) != 0 {
                                    // NZ = success
                                    iFilesUpdated += 1;
                                } else {
                                    Com_Printf(
                                        b"*********** Failed write to file \"%s\"!\n\0".as_ptr() as *const c_char,
                                        sFilename.as_ptr(),
                                    );
                                    iErrors += 1;
                                    // Porting note: construct error message using va()
                                    let err_msg = va(b"Failed to write: \"%s\"\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                                    strErrors.push_str(CStr::from_ptr(err_msg).to_string_lossy().as_ref());
                                }
                            } else {
                                Com_Printf(
                                    b"*********** Failed write to file \"%s\"!\n\0".as_ptr() as *const c_char,
                                    sFilename.as_ptr(),
                                );
                                iErrors += 1;
                                let err_msg = va(b"Failed to write: \"%s\"\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                                strErrors.push_str(CStr::from_ptr(err_msg).to_string_lossy().as_ref());
                            }
                            FS_FCloseFile(f);
                        } else {
                            Com_Printf(
                                b"*********** Failed to re-open for write \"%s\"!\n\0".as_ptr() as *const c_char,
                                sFilename.as_ptr(),
                            );
                            iErrors += 1;
                            let err_msg = va(b"Failed to re-open for write: \"%s\"\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                            strErrors.push_str(CStr::from_ptr(err_msg).to_string_lossy().as_ref());
                        }
                    } else {
                        Com_Error(
                            ERR_DROP,
                            b"******* This MP3 should be deleted: \"%s\"\n\0".as_ptr() as *const c_char,
                            sFilename.as_ptr(),
                        );
                    }
                } else {
                    // MP3_IsValid() will already have printed any errors via Com_Printf at this point...
                    Com_Printf(
                        b"*********** File was not a valid MP3!: \"%s\"\n\0".as_ptr() as *const c_char,
                        sFilename.as_ptr(),
                    );
                    iErrors += 1;
                    let err_msg = va(b"Not game-legal MP3 format: \"%s\"\n\0".as_ptr() as *const c_char, sFilename.as_ptr());
                    strErrors.push_str(CStr::from_ptr(err_msg).to_string_lossy().as_ref());
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

// this console-function is for development purposes, and makes sure that sound/*.mp3 /s have tags in them
//	specifying stuff like their max volume (and uncompressed size) etc...
//
pub unsafe fn S_MP3_CalcVols_f() {
    let mut sStartDir: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    sStartDir[0] = b's' as c_char;
    sStartDir[1] = b'o' as c_char;
    sStartDir[2] = b'u' as c_char;
    sStartDir[3] = b'n' as c_char;
    sStartDir[4] = b'd' as c_char;
    sStartDir[5] = 0;

    let sUsage: &[u8] = b"Usage: mp3_calcvols [-rescan] <startdir>\ne.g. mp3_calcvols sound/chars\0";

    if Cmd_Argc() == 1 || Cmd_Argc() > 4 {
        // 3 optional arguments
        Com_Printf(sUsage.as_ptr() as *const c_char);
        return;
    }

    S_StopAllSounds();

    qbForceRescan = qfalse;
    qbForceStereo = qfalse;
    iFilesFound = 0;
    iFilesUpdated = 0;
    iErrors = 0;
    strErrors.clear();

    let mut i: c_int = 1;
    while i < Cmd_Argc() {
        if *Cmd_Argv(i) == b'-' as c_char {
            if Q_stricmp(Cmd_Argv(i), b"-rescan\0".as_ptr() as *const c_char) == 0 {
                qbForceRescan = qtrue;
            } else if Q_stricmp(Cmd_Argv(i), b"-stereo\0".as_ptr() as *const c_char) == 0 {
                qbForceStereo = qtrue;
            } else {
                // unknown switch...
                //
                Com_Printf(sUsage.as_ptr() as *const c_char);
                return;
            }
            i += 1;
            continue;
        }
        libc::strcpy(sStartDir.as_mut_ptr(), Cmd_Argv(i));
        i += 1;
    }

    Com_Printf(
        b"Starting Scan for Updates in Dir: %s\n\0".as_ptr() as *const c_char,
        sStartDir.as_ptr(),
    );
    R_CheckMP3s(sStartDir.as_ptr());

    Com_Printf(
        b"\n%d files found/scanned, %d files updated      ( %d errors total)\n\0".as_ptr() as *const c_char,
        iFilesFound,
        iFilesUpdated,
        iErrors,
    );

    if iErrors != 0 {
        if let Ok(errors_cstring) = CString::new(strErrors.clone()) {
            Com_Printf(
                b"\nBad Files:\n%s\n\0".as_ptr() as *const c_char,
                errors_cstring.as_ptr(),
            );
        }
    }
}

// adjust filename for foreign languages and WAV/MP3 issues.
//
// returns qfalse if failed to load, else fills in *pData
//
unsafe fn S_LoadSound_FileLoadAndNameAdjuster(
    psFilename: *mut c_char,
    pData: *mut *mut u8,
    piSize: *mut c_int,
    iNameStrlen: c_int,
) -> c_int {
    let psVoice: *mut c_char = libc::strstr(psFilename, b"chars\0".as_ptr() as *const c_char) as *mut c_char;
    if !psVoice.is_null() {
        // cache foreign voices...
        //
        if !com_buildScript.is_null() && (*com_buildScript).integer != 0 {
            let mut hFile: fileHandle_t = 0;
            // German
            libc::strncpy(psVoice, b"chr_d\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            FS_FOpenFileRead(psFilename, &mut hFile, qfalse); // cache the wav
            if hFile == 0 {
                libc::strcpy(
                    &mut *psFilename.offset((iNameStrlen - 3) as isize),
                    b"mp3\0".as_ptr() as *const c_char,
                );
                FS_FOpenFileRead(psFilename, &mut hFile, qfalse); // cache the mp3
            }
            if hFile != 0 {
                FS_FCloseFile(hFile);
            }
            libc::strcpy(
                &mut *psFilename.offset((iNameStrlen - 3) as isize),
                b"wav\0".as_ptr() as *const c_char,
            ); // put it back to wav

            // French
            libc::strncpy(psVoice, b"chr_f\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            FS_FOpenFileRead(psFilename, &mut hFile, qfalse); // cache the wav
            if hFile == 0 {
                libc::strcpy(
                    &mut *psFilename.offset((iNameStrlen - 3) as isize),
                    b"mp3\0".as_ptr() as *const c_char,
                );
                FS_FOpenFileRead(psFilename, &mut hFile, qfalse); // cache the mp3
            }
            if hFile != 0 {
                FS_FCloseFile(hFile);
            }
            libc::strcpy(
                &mut *psFilename.offset((iNameStrlen - 3) as isize),
                b"wav\0".as_ptr() as *const c_char,
            ); // put it back to wav

            // Spanish
            libc::strncpy(psVoice, b"chr_e\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
            FS_FOpenFileRead(psFilename, &mut hFile, qfalse); // cache the wav
            if hFile == 0 {
                libc::strcpy(
                    &mut *psFilename.offset((iNameStrlen - 3) as isize),
                    b"mp3\0".as_ptr() as *const c_char,
                );
                FS_FOpenFileRead(psFilename, &mut hFile, qfalse); // cache the mp3
            }
            if hFile != 0 {
                FS_FCloseFile(hFile);
            }
            libc::strcpy(
                &mut *psFilename.offset((iNameStrlen - 3) as isize),
                b"wav\0".as_ptr() as *const c_char,
            ); // put it back to wav

            libc::strncpy(psVoice, b"chars\0".as_ptr() as *const c_char, 5); // put it back to chars
        }

        // account for foreign voices...
        //
        if !s_language.is_null() && libc::stricmp(b"DEUTSCH\0".as_ptr() as *const c_char, (*s_language).string as *const c_char) == 0 {
            libc::strncpy(psVoice, b"chr_d\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
        } else if !s_language.is_null()
            && libc::stricmp(b"FRANCAIS\0".as_ptr() as *const c_char, (*s_language).string as *const c_char) == 0
        {
            libc::strncpy(psVoice, b"chr_f\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
        } else if !s_language.is_null()
            && libc::stricmp(b"ESPANOL\0".as_ptr() as *const c_char, (*s_language).string as *const c_char) == 0
        {
            libc::strncpy(psVoice, b"chr_e\0".as_ptr() as *const c_char, 5); // same number of letters as "chars"
        } else {
            // use this ptr as a flag as to whether or not we substituted with a foreign version
        }
    }

    *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try WAV
    if pData.is_null() || (*pData).is_null() {
        *psFilename.offset((iNameStrlen - 3) as isize) = b'm' as c_char;
        *psFilename.offset((iNameStrlen - 2) as isize) = b'p' as c_char;
        *psFilename.offset((iNameStrlen - 1) as isize) = b'3' as c_char;
        *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try MP3

        if pData.is_null() || (*pData).is_null() {
            // hmmm, not found, ok, maybe we were trying a foreign noise ("arghhhhh.mp3" that doesn't matter?) but it
            // was missing?   Can't tell really, since both types are now in sound/chars. Oh well, fall back to English for now...

            if !psVoice.is_null() {
                // yep, so fallback to re-try the english...
                Com_Printf(
                    b"^3Foreign file missing: \"%s\"! (using English...)\n\0".as_ptr() as *const c_char,
                    psFilename,
                );

                libc::strncpy(psVoice, b"chars\0".as_ptr() as *const c_char, 5);

                *psFilename.offset((iNameStrlen - 3) as isize) = b'w' as c_char;
                *psFilename.offset((iNameStrlen - 2) as isize) = b'a' as c_char;
                *psFilename.offset((iNameStrlen - 1) as isize) = b'v' as c_char;
                *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try English WAV
                if pData.is_null() || (*pData).is_null() {
                    *psFilename.offset((iNameStrlen - 3) as isize) = b'm' as c_char;
                    *psFilename.offset((iNameStrlen - 2) as isize) = b'p' as c_char;
                    *psFilename.offset((iNameStrlen - 1) as isize) = b'3' as c_char;
                    *piSize = FS_ReadFile(psFilename, pData as *mut *mut c_void); // try English MP3
                }
            }

            if pData.is_null() || (*pData).is_null() {
                return qfalse; // sod it, give up...
            }
        }
    }

    qtrue
}

// returns qtrue if this dir is allowed to keep loaded MP3s, else qfalse if they should be WAV'd instead...
//
// note that this is passed the original, un-language'd name
//
// (I was going to remove this, but on kejim_post I hit an assert because someone had got an ambient sound when the
//	perimter fence goes online that was an MP3, then it tried to get added as looping. Presumably it sounded ok or
//	they'd have noticed, but we therefore need to stop other levels using those. "sound/ambience" I can check for,
//	but doors etc could be anything. Sigh...)
//
unsafe fn S_LoadSound_DirIsAllowedToKeepMP3s(psFilename: *const c_char) -> c_int {
    let psAllowedDirs: &[&[u8]] = &[
        b"sound/chars/", // b"sound/chr_d/"	// no need for this now, or any other language, since we'll always compare against english
    ];

    let mut i: usize = 0;
    while i < psAllowedDirs.len() {
        if libc::strnicmp(
            psFilename,
            psAllowedDirs[i].as_ptr() as *const c_char,
            libc::strlen(psAllowedDirs[i].as_ptr() as *const c_char),
        ) == 0
        {
            return qtrue; // found a dir that's allowed to keep MP3s
        }
        i += 1;
    }

    qfalse
}

// ==============
// S_LoadSound
//
// The filename may be different than sfx->name in the case
// of a forced fallback of a player specific sound	(or of a wav/mp3 substitution now -Ste)
// ==============
pub static mut gbInsideLoadSound: c_int = qfalse;

unsafe fn S_LoadSound_Actual(sfx: *mut sfx_t) -> c_int {
    let mut data: *mut u8 = std::ptr::null_mut();
    let mut samples: *mut i16;
    let mut info: wavinfo_t = wavinfo_t {
        format: 0,
        rate: 0,
        width: 0,
        channels: 0,
        samples: 0,
        dataofs: 0,
    };
    let mut size: c_int = 0;
    let mut psExt: *mut c_char;
    let mut sLoadName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut Buffer: u32 = 0;

    let len: c_int = libc::strlen((*sfx).sSoundName.as_ptr() as *const c_char) as c_int;
    if len < 5 {
        return qfalse;
    }

    // player specific sounds are never directly loaded...
    //
    if (*sfx).sSoundName[0] as c_int == b'*' as c_int {
        return qfalse;
    }
    // make up a local filename to try wav/mp3 substitutes...
    //
    Q_strncpyz(sLoadName.as_mut_ptr(), (*sfx).sSoundName.as_ptr(), MAX_QPATH as c_int);
    // strlwr( sLoadName );  // Porting note: convert to lowercase - need libc::strlwr or manual implementation
    //
    // Ensure name has an extension (which it must have, but you never know), and get ptr to it...
    //
    psExt = &mut sLoadName[(libc::strlen(sLoadName.as_ptr()) - 4) as usize];
    if *psExt != b'.' as c_char {
        // Com_Printf( "WARNING: soundname '%s' does not have 3-letter extension\n",sLoadName);
        COM_DefaultExtension(sLoadName.as_mut_ptr(), MAX_QPATH as c_int, b".wav\0".as_ptr() as *const c_char); // so psExt below is always valid
        psExt = &mut sLoadName[(libc::strlen(sLoadName.as_ptr()) - 4) as usize];
    }

    if S_LoadSound_FileLoadAndNameAdjuster(sLoadName.as_mut_ptr(), &mut data, &mut size, len) == qfalse {
        return qfalse;
    }

    SND_TouchSFX(sfx);

    // =========
    if libc::strnicmp(psExt, b".mp3\0".as_ptr() as *const c_char, 4) == 0 {
        // load MP3 file instead...
        //
        if MP3_IsValid(sLoadName.as_ptr(), data as *const c_void, size, qfalse) != 0 {
            let iRawPCMDataSize: c_int = MP3_GetUnpackedSize(sLoadName.as_ptr(), data as *const c_void, size, qfalse, qfalse);

            if S_LoadSound_DirIsAllowedToKeepMP3s((*sfx).sSoundName.as_ptr()) != qfalse
                // NOT sLoadName, this uses original un-languaged name
                && MP3Stream_InitFromFile(sfx, data, size, sLoadName.as_ptr(), iRawPCMDataSize + 2304, /* + 1 MP3 frame size, jic */ qfalse) != qfalse
            {
                // Com_DPrintf("(Keeping file \"%s\" as MP3)\n",sLoadName);

                if s_UseOpenAL != 0 {
                    // Create space for lipsync data (4 lip sync values per streaming AL buffer)
                    if (libc::strstr((*sfx).sSoundName.as_ptr(), b"chars\0".as_ptr() as *const c_char) as *const c_void) != std::ptr::null()
                        || (libc::strstr((*sfx).sSoundName.as_ptr(), b"CHARS\0".as_ptr() as *const c_char) as *const c_void) != std::ptr::null()
                    {
                        (*sfx).lipSyncData = Z_Malloc(16, TAG_SND_RAWDATA, qfalse) as *mut c_char;
                    } else {
                        (*sfx).lipSyncData = std::ptr::null_mut();
                    }
                }
            } else {
                // small file, not worth keeping as MP3 since it would increase in size (with MP3 header etc)...
                //
                Com_DPrintf(
                    b"S_LoadSound: Unpacking MP3 file(%i) \"%s\" to wav(%i).\n\0".as_ptr() as *const c_char,
                    size,
                    sLoadName.as_ptr(),
                    iRawPCMDataSize,
                );
                //
                // unpack and convert into WAV...
                //
                {
                    let pbUnpackBuffer: *mut u8 = Z_Malloc(iRawPCMDataSize + 10 + 2304, TAG_TEMP_WORKSPACE, qfalse) as *mut u8; // <g>

                    {
                        let iResultBytes: c_int = MP3_UnpackRawPCM(sLoadName.as_ptr(), data as *const c_void, size, pbUnpackBuffer, qfalse);

                        if iResultBytes != iRawPCMDataSize {
                            Com_Printf(
                                b"^3**** MP3 %s final unpack size %d different to previous value %d\n\0".as_ptr() as *const c_char,
                                sLoadName.as_ptr(),
                                iResultBytes,
                                iRawPCMDataSize,
                            );
                            // assert (iResultBytes == iRawPCMDataSize);
                        }

                        // fake up a WAV structure so I can use the other post-load sound code such as volume calc for lip-synching
                        //
                        // (this is a bit crap really, but it lets me drop through into existing code)...
                        //
                        MP3_FakeUpWAVInfo(
                            sLoadName.as_ptr(),
                            data as *const c_void,
                            size,
                            iResultBytes,
                            // these params are all references...
                            &mut info.format,
                            &mut info.rate,
                            &mut info.width,
                            &mut info.channels,
                            &mut info.samples,
                            &mut info.dataofs,
                            qfalse,
                        );

                        S_LoadSound_Finalize(&mut info, sfx, pbUnpackBuffer);

                        // Open AL
                        if s_UseOpenAL != 0 {
                            if (libc::strstr((*sfx).sSoundName.as_ptr(), b"chars\0".as_ptr() as *const c_char) as *const c_void)
                                != std::ptr::null()
                                || (libc::strstr((*sfx).sSoundName.as_ptr(), b"CHARS\0".as_ptr() as *const c_char) as *const c_void)
                                    != std::ptr::null()
                            {
                                (*sfx).lipSyncData = Z_Malloc(
                                    ((*sfx).iSoundLengthInSamples / 1000) + 1,
                                    TAG_SND_RAWDATA,
                                    qfalse,
                                ) as *mut c_char;
                                S_PreProcessLipSync(sfx);
                            } else {
                                (*sfx).lipSyncData = std::ptr::null_mut();
                            }

                            // Clear Open AL Error state
                            alGetError();

                            // Generate AL Buffer
                            alGenBuffers(1, &mut Buffer);
                            if alGetError() == AL_NO_ERROR {
                                // Copy audio data to AL Buffer
                                alBufferData(
                                    Buffer,
                                    AL_FORMAT_MONO16,
                                    (*sfx).pSoundData as *const c_void,
                                    (*sfx).iSoundLengthInSamples * 2,
                                    22050,
                                );
                                if alGetError() == AL_NO_ERROR {
                                    (*sfx).Buffer = Buffer;
                                    Z_Free((*sfx).pSoundData as *mut c_void);
                                    (*sfx).pSoundData = std::ptr::null_mut();
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
            return qfalse;
        }
    } else {
        // loading a WAV, presumably...

        // =========

        info = GetWavinfo(sLoadName.as_ptr(), data, size);
        if info.channels != 1 {
            Com_Printf(b"%s is a stereo wav file\n\0".as_ptr() as *const c_char, sLoadName.as_ptr());
            FS_FreeFile(data as *mut c_void);
            return qfalse;
        }

        /*		if ( info.width == 1 ) {
            Com_Printf(S_COLOR_YELLOW "WARNING: %s is a 8 bit wav file\n", sLoadName);
        }

        if ( info.rate != 22050 ) {
            Com_Printf(S_COLOR_YELLOW "WARNING: %s is not a 22kHz wav file\n", sLoadName);
        }
        */

        samples = Z_Malloc(
            (info.samples * std::mem::size_of::<i16>() as c_int * 2),
            TAG_TEMP_WORKSPACE,
            qfalse,
        ) as *mut i16;

        (*sfx).eSoundCompressionMethod = 0; // ct_16
        (*sfx).iSoundLengthInSamples = info.samples;
        (*sfx).pSoundData = std::ptr::null_mut();
        ResampleSfx(sfx, info.rate, info.width, data.offset(info.dataofs as isize));

        // Open AL
        if s_UseOpenAL != 0 {
            if (libc::strstr((*sfx).sSoundName.as_ptr(), b"chars\0".as_ptr() as *const c_char) as *const c_void) != std::ptr::null()
                || (libc::strstr((*sfx).sSoundName.as_ptr(), b"CHARS\0".as_ptr() as *const c_char) as *const c_void) != std::ptr::null()
            {
                (*sfx).lipSyncData =
                    Z_Malloc(((*sfx).iSoundLengthInSamples / 1000) + 1, TAG_SND_RAWDATA, qfalse) as *mut c_char;
                S_PreProcessLipSync(sfx);
            } else {
                (*sfx).lipSyncData = std::ptr::null_mut();
            }

            // Clear Open AL Error State
            alGetError();

            // Generate AL Buffer
            alGenBuffers(1, &mut Buffer);
            if alGetError() == AL_NO_ERROR {
                // Copy audio data to AL Buffer
                alBufferData(
                    Buffer,
                    AL_FORMAT_MONO16,
                    (*sfx).pSoundData as *const c_void,
                    (*sfx).iSoundLengthInSamples * 2,
                    22050,
                );
                if alGetError() == AL_NO_ERROR {
                    // Store AL Buffer in sfx struct, and release sample data
                    (*sfx).Buffer = Buffer;
                    Z_Free((*sfx).pSoundData as *mut c_void);
                    (*sfx).pSoundData = std::ptr::null_mut();
                }
            }
        }

        Z_Free(samples as *mut c_void);
    }

    FS_FreeFile(data as *mut c_void);

    qtrue
}

// wrapper function for above so I can guarantee that we don't attempt any audio-dumping during this call because
//	of a z_malloc() fail recovery...
//
pub unsafe fn S_LoadSound(sfx: *mut sfx_t) -> c_int {
    gbInsideLoadSound = qtrue; // !!!!!!!!!!!!!

    let bReturn: c_int = S_LoadSound_Actual(sfx);

    gbInsideLoadSound = qfalse; // !!!!!!!!!!!!!

    bReturn
}

// Precalculate the lipsync values for the whole sample
pub unsafe fn S_PreProcessLipSync(sfx: *mut sfx_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut sample: c_int;
    let mut sampleTotal: c_int = 0;

    j = 0;
    i = 0;
    while i < (*sfx).iSoundLengthInSamples {
        sample = LittleShort((*(*sfx).pSoundData.offset(i as isize) as c_int));

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

            *(*sfx).lipSyncData.offset(j as isize) = sample as c_char;
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

    *(*sfx).lipSyncData.offset(j as isize) = sample as c_char;
}
