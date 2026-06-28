#![allow(non_snake_case)]

use core::ffi::{c_int, c_void, c_char};
use core::ptr;

// Anything above this include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #include <float.h>

// #include "../client/snd_local.h"
// #include "win_local.h"

// Windows type definitions
type HRESULT = i32;
type DWORD = u32;
type HINSTANCE = *mut c_void;
type LPDIRECTSOUND = *mut c_void;
type LPDIRECTSOUNDBUFFER = *mut c_void;
type UINT = u32;
type WORD = u16;
type BYTE = u8;
type LONG = i32;

// DirectSound constants
const DS_OK: i32 = 0;
const DSERR_BUFFERLOST: i32 = -2147024891i32; // 0x88780096
const DSERR_INVALIDCALL: i32 = -2147024809i32; // 0x88780078
const DSERR_INVALIDPARAM: i32 = -2147024809i32; // 0x88780078
const DSERR_PRIOLEVELNEEDED: i32 = -2147024821i32; // 0x88780084
const DSERR_ALLOCATED: i32 = -2147024894i32; // 0x88780099
const DSERR_UNINITIALIZED: i32 = -2147024833i32; // 0x8878005F
const DSERR_UNSUPPORTED: i32 = -2147024837i32; // 0x8878005B

const DSBCAPS_CTRLFREQUENCY: u32 = 0x00000001;
const DSBCAPS_LOCHARDWARE: u32 = 0x00000004;
const idDSBCAPS_GETCURRENTPOSITION2: u32 = 0x00010000;
const DSBPLAY_LOOPING: u32 = 0x00000004;
const DSBSTATUS_PLAYING: u32 = 0x00000001;
const DSBSTATUS_BUFFERLOST: u32 = 0x00000002;
const WAVE_FORMAT_PCM: u16 = 1;
const TIME_SAMPLES: u32 = 2;
const DSSCL_NORMAL: u32 = 1;
const DSSCL_PRIORITY: u32 = 2;

// WAVEFORMATEX structure
#[repr(C)]
pub struct WAVEFORMATEX {
    pub wFormatTag: WORD,
    pub nChannels: WORD,
    pub nSamplesPerSec: DWORD,
    pub nAvgBytesPerSec: DWORD,
    pub nBlockAlign: WORD,
    pub wBitsPerSample: WORD,
    pub cbSize: WORD,
}

// DSBUFFERDESC structure
#[repr(C)]
pub struct DSBUFFERDESC {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwBufferBytes: DWORD,
    pub dwReserved: DWORD,
    pub lpwfxFormat: *mut WAVEFORMATEX,
    pub guid3DAlgorithm: [u8; 16],
}

// DSBCAPS structure
#[repr(C)]
pub struct DSBCAPS {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwBufferBytes: DWORD,
    pub dwUnlockTransferRate: DWORD,
    pub dwPlayCpuOverhead: DWORD,
}

// MMTIME structure - union type
#[repr(C)]
pub struct MMTIME {
    pub wType: UINT,
    pub u: MMTIME_U,
}

#[repr(C)]
pub union MMTIME_U {
    pub ms: DWORD,
    pub sample: DWORD,
    pub cb: DWORD,
    pub ticks: DWORD,
}

// External types and functions we depend on
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...) -> c_void;
    fn Com_DPrintf(fmt: *const c_char, ...) -> c_void;
    fn S_Shutdown() -> c_void;
    fn S_AL_MuteAllSounds(mute: c_int) -> c_void;

    fn LoadLibrary(lpLibFileName: *const c_char) -> HINSTANCE;
    fn FreeLibrary(hLibModule: HINSTANCE) -> i32;
    fn GetProcAddress(hModule: HINSTANCE, lpProcName: *const c_char) -> *mut c_void;
    fn Sleep(dwMilliseconds: DWORD) -> c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// External global variables we depend on
extern "C" {
    static s_UseOpenAL: c_int;

    // dma structure - external definition
    static dma: DMABuffer;

    // g_wv structure - external definition
    static g_wv: WinVars;

    // s_khz - cvar structure
    static s_khz: CVar;
}

// Forward declarations of external types
#[repr(C)]
pub struct DMABuffer {
    pub channels: c_int,
    pub samplebits: c_int,
    pub speed: c_int,
    pub samples: c_int,
    pub submission_chunk: c_int,
    pub buffer: *mut c_void,
}

#[repr(C)]
pub struct WinVars {
    pub hWnd: *mut c_void,
    // ... other fields
}

#[repr(C)]
pub struct CVar {
    pub integer: c_int,
    // ... other fields
}

// Function pointer type for DirectSoundCreate
type PDirectSoundCreate = unsafe extern "stdcall" fn(
    *mut c_void,
    *mut LPDIRECTSOUND,
    *mut c_void,
) -> HRESULT;

// Macro equivalents
// #define iDirectSoundCreate(a,b,c)	pDirectSoundCreate(a,b,c)
#[inline]
fn iDirectSoundCreate(
    a: *mut c_void,
    b: *mut LPDIRECTSOUND,
    c: *mut c_void,
) -> HRESULT {
    unsafe {
        if !pDirectSoundCreate.is_null() {
            pDirectSoundCreate(a, b, c)
        } else {
            -1
        }
    }
}

// #define SECONDARY_BUFFER_SIZE	0x10000
const SECONDARY_BUFFER_SIZE: usize = 0x10000;

// Static variables
static mut pDirectSoundCreate: Option<PDirectSoundCreate> = None;
static mut dsound_init: c_int = 0; // qboolean (false)
static mut sample16: c_int = 0;
static mut gSndBufSize: DWORD = 0;
static mut locksize: DWORD = 0;
static mut pDS: LPDIRECTSOUND = ptr::null_mut();
static mut pDSBuf: LPDIRECTSOUNDBUFFER = ptr::null_mut();
static mut pDSPBuf: LPDIRECTSOUNDBUFFER = ptr::null_mut();
static mut hInstDS: HINSTANCE = ptr::null_mut();

static DSoundError_BUFFERLOST: &[u8] = b"DSERR_BUFFERLOST\0";
static DSoundError_INVALIDCALL: &[u8] = b"DSERR_INVALIDCALLS\0";
static DSoundError_INVALIDPARAM: &[u8] = b"DSERR_INVALIDPARAM\0";
static DSoundError_PRIOLEVELNEEDED: &[u8] = b"DSERR_PRIOLEVELNEEDED\0";
static DSoundError_ALLOCATED: &[u8] = b"DSERR_ALLOCATED\0";
static DSoundError_UNINITIALIZED: &[u8] = b"DSERR_UNINITIALIZED\0";
static DSoundError_UNSUPPORTED: &[u8] = b"DSERR_UNSUPPORTED \0";
static DSoundError_unknown: &[u8] = b"unknown\0";

unsafe fn DSoundError(error: i32) -> *const c_char {
    match error {
        DSERR_BUFFERLOST => DSoundError_BUFFERLOST.as_ptr() as *const c_char,
        DSERR_INVALIDCALL => DSoundError_INVALIDCALL.as_ptr() as *const c_char,
        DSERR_INVALIDPARAM => DSoundError_INVALIDPARAM.as_ptr() as *const c_char,
        DSERR_PRIOLEVELNEEDED => DSoundError_PRIOLEVELNEEDED.as_ptr() as *const c_char,
        DSERR_ALLOCATED => DSoundError_ALLOCATED.as_ptr() as *const c_char,
        DSERR_UNINITIALIZED => DSoundError_UNINITIALIZED.as_ptr() as *const c_char,
        DSERR_UNSUPPORTED => DSoundError_UNSUPPORTED.as_ptr() as *const c_char,
        _ => DSoundError_unknown.as_ptr() as *const c_char,
    }
}

/*
==================
SNDDMA_Shutdown
==================
*/
pub unsafe fn SNDDMA_Shutdown() {
    Com_DPrintf(b"Shutting down sound system\n\0".as_ptr() as *const c_char);

    if !pDS.is_null() {
        Com_DPrintf(b"Destroying DS buffers\n\0".as_ptr() as *const c_char);
        if !pDS.is_null() {
            Com_DPrintf(b"...setting NORMAL coop level\n\0".as_ptr() as *const c_char);
            // pDS->SetCooperativeLevel( g_wv.hWnd, DSSCL_NORMAL );
            // COM method call - need to dereference and call vtable
            let pds_vt = *(pDS as *const *const DSVtable);
            ((*pds_vt).SetCooperativeLevel)(pDS, g_wv.hWnd, DSSCL_NORMAL);
        }

        if !pDSBuf.is_null() {
            Com_DPrintf(b"...stopping and releasing sound buffer\n\0".as_ptr() as *const c_char);
            // pDSBuf->Stop( );
            let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
            ((*pdsbuf_vt).Stop)(pDSBuf);
            // pDSBuf->Release( );
            ((*pdsbuf_vt).Release)(pDSBuf);
        }

        // only release primary buffer if it's not also the mixing buffer we just released
        if !pDSPBuf.is_null() && pDSBuf != pDSPBuf {
            Com_DPrintf(b"...releasing primary buffer\n\0".as_ptr() as *const c_char);
            // pDSPBuf->Release( );
            let pdspbuf_vt = *(pDSPBuf as *const *const DSBufferVtable);
            ((*pdspbuf_vt).Release)(pDSPBuf);
        }
        pDSBuf = ptr::null_mut();
        pDSPBuf = ptr::null_mut();

        // dma.buffer = NULL;
        let dma_ptr = &mut dma as *mut _;
        (*dma_ptr).buffer = ptr::null_mut();

        Com_DPrintf(b"...releasing DS object\n\0".as_ptr() as *const c_char);
        // pDS->Release( );
        let pds_vt = *(pDS as *const *const DSVtable);
        ((*pds_vt).Release)(pDS);
    }

    if !hInstDS.is_null() {
        Com_DPrintf(b"...freeing DSOUND.DLL\n\0".as_ptr() as *const c_char);
        FreeLibrary(hInstDS);
        hInstDS = ptr::null_mut();
    }

    pDS = ptr::null_mut();
    pDSBuf = ptr::null_mut();
    pDSPBuf = ptr::null_mut();
    dsound_init = 0; // qfalse
    memset(&mut dma as *mut _ as *mut c_void, 0, core::mem::size_of::<DMABuffer>());
}

/*
==================
SNDDMA_Init

Initialize direct sound
Returns false if failed
==================
*/
pub unsafe fn SNDDMA_Init() -> c_int {
    memset(&mut dma as *mut _ as *mut c_void, 0, core::mem::size_of::<DMABuffer>());
    dsound_init = 0; // qfalse

    if SNDDMA_InitDS() == 0 {
        return 0; // qfalse
    }

    dsound_init = 1; // qtrue

    Com_DPrintf(b"Completed successfully\n\0".as_ptr() as *const c_char);

    return 1; // qtrue
}

pub unsafe fn SNDDMA_InitDS() -> c_int {
    let mut hresult: HRESULT;
    let mut pauseTried: c_int = 0; // qboolean
    let mut dsbuf: DSBUFFERDESC;
    let mut dsbcaps: DSBCAPS;
    let mut format: WAVEFORMATEX;

    Com_Printf(b"Initializing DirectSound\n\0".as_ptr() as *const c_char);

    if hInstDS.is_null() {
        Com_DPrintf(b"...loading dsound.dll: \0".as_ptr() as *const c_char);

        hInstDS = LoadLibrary(b"dsound.dll\0".as_ptr() as *const c_char);

        if hInstDS.is_null() {
            Com_Printf(b"failed\n\0".as_ptr() as *const c_char);
            return 0;
        }

        Com_DPrintf(b"ok\n\0".as_ptr() as *const c_char);
        let proc_addr = GetProcAddress(hInstDS, b"DirectSoundCreate\0".as_ptr() as *const c_char);
        pDirectSoundCreate = core::mem::transmute(proc_addr);

        if pDirectSoundCreate.is_none() {
            Com_Printf(b"*** couldn't get DS proc addr ***\n\0".as_ptr() as *const c_char);
            return 0;
        }
    }

    Com_DPrintf(b"...creating DS object: \0".as_ptr() as *const c_char);
    pauseTried = 0; // qfalse
    loop {
        hresult = iDirectSoundCreate(ptr::null_mut(), &mut pDS as *mut LPDIRECTSOUND, ptr::null_mut());
        if hresult == DS_OK {
            break;
        }

        if hresult != DSERR_ALLOCATED {
            Com_Printf(b"failed\n\0".as_ptr() as *const c_char);
            return 0;
        }

        if pauseTried != 0 {
            Com_Printf(b"failed, hardware already in use\n\0".as_ptr() as *const c_char);
            return 0;
        }
        // first try just waiting five seconds and trying again
        // this will handle the case of a sysyem beep playing when the
        // game starts
        Com_DPrintf(b"retrying...\n\0".as_ptr() as *const c_char);
        Sleep(3000);
        pauseTried = 1; // qtrue
    }
    Com_DPrintf(b"ok\n\0".as_ptr() as *const c_char);

    Com_DPrintf(b"...setting DSSCL_PRIORITY coop level: \0".as_ptr() as *const c_char);

    {
        let pds_vt = *(pDS as *const *const DSVtable);
        if ((*pds_vt).SetCooperativeLevel)(pDS, g_wv.hWnd, DSSCL_PRIORITY) != DS_OK as i32 {
            Com_Printf(b"failed\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
            return 0; // qfalse
        }
    }
    Com_DPrintf(b"ok\n\0".as_ptr() as *const c_char);

    // create the secondary buffer we'll actually work with
    let dma_ptr = &mut dma as *mut _;
    (*dma_ptr).channels = 2;
    (*dma_ptr).samplebits = 16;

    if s_khz.integer == 44 {
        (*dma_ptr).speed = 44100;
    } else if s_khz.integer == 22 {
        (*dma_ptr).speed = 22050;
    } else {
        (*dma_ptr).speed = 11025;
    }

    memset(&mut format as *mut _ as *mut c_void, 0, core::mem::size_of::<WAVEFORMATEX>());
    format.wFormatTag = WAVE_FORMAT_PCM;
    format.nChannels = (*dma_ptr).channels as u16;
    format.wBitsPerSample = (*dma_ptr).samplebits as u16;
    format.nSamplesPerSec = (*dma_ptr).speed as u32;
    format.nBlockAlign = ((format.nChannels as u32 * format.wBitsPerSample as u32) / 8) as u16;
    format.cbSize = 0;
    format.nAvgBytesPerSec = format.nSamplesPerSec * format.nBlockAlign as u32;

    memset(&mut dsbuf as *mut _ as *mut c_void, 0, core::mem::size_of::<DSBUFFERDESC>());
    dsbuf.dwSize = core::mem::size_of::<DSBUFFERDESC>() as u32;

    dsbuf.dwFlags = DSBCAPS_CTRLFREQUENCY | DSBCAPS_LOCHARDWARE | idDSBCAPS_GETCURRENTPOSITION2;
    dsbuf.dwBufferBytes = SECONDARY_BUFFER_SIZE as u32;
    dsbuf.lpwfxFormat = &mut format as *mut WAVEFORMATEX;

    Com_DPrintf(b"...creating secondary buffer: \0".as_ptr() as *const c_char);
    {
        let pds_vt = *(pDS as *const *const DSVtable);
        if ((*pds_vt).CreateSoundBuffer)(pDS, &mut dsbuf as *mut _, &mut pDSBuf as *mut LPDIRECTSOUNDBUFFER, ptr::null_mut()) != DS_OK as i32 {
            Com_Printf(b" - using ancient version of DirectX -- this will slow FPS\n\0".as_ptr() as *const c_char);
            dsbuf.dwFlags = DSBCAPS_CTRLFREQUENCY;
            hresult = ((*pds_vt).CreateSoundBuffer)(pDS, &mut dsbuf as *mut _, &mut pDSBuf as *mut LPDIRECTSOUNDBUFFER, ptr::null_mut());
            if hresult != DS_OK as i32 {
                Com_Printf(b"failed to create secondary buffer - %s\n\0".as_ptr() as *const c_char, DSoundError(hresult));
                SNDDMA_Shutdown();
                return 0; // qfalse
            }
        }
    }
    Com_Printf(b"locked hardware.  ok\n\0".as_ptr() as *const c_char);

    // Make sure mixer is active
    {
        let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
        if ((*pdsbuf_vt).Play)(pDSBuf, 0, 0, DSBPLAY_LOOPING) != DS_OK as i32 {
            Com_Printf(b"*** Looped sound play failed ***\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
            return 0; // qfalse
        }
    }

    memset(&mut dsbcaps as *mut _ as *mut c_void, 0, core::mem::size_of::<DSBCAPS>());
    dsbcaps.dwSize = core::mem::size_of::<DSBCAPS>() as u32;
    // get the returned buffer size
    {
        let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
        if ((*pdsbuf_vt).GetCaps)(pDSBuf, &mut dsbcaps as *mut _) != DS_OK as i32 {
            Com_Printf(b"*** GetCaps failed ***\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
            return 0; // qfalse
        }
    }

    gSndBufSize = dsbcaps.dwBufferBytes;

    let dma_ptr = &mut dma as *mut _;
    (*dma_ptr).channels = format.nChannels as i32;
    (*dma_ptr).samplebits = format.wBitsPerSample as i32;
    (*dma_ptr).speed = format.nSamplesPerSec as i32;
    (*dma_ptr).samples = (gSndBufSize / (((*dma_ptr).samplebits / 8) as u32)) as i32;
    (*dma_ptr).submission_chunk = 1;
    (*dma_ptr).buffer = ptr::null_mut(); // must be locked first

    sample16 = ((*dma_ptr).samplebits / 8) - 1;

    SNDDMA_BeginPainting();
    if !dma.buffer.is_null() {
        memset(dma.buffer, 0, (dma.samples * dma.samplebits / 8) as usize);
    }
    SNDDMA_Submit();
    return 1;
}

/*
==============
SNDDMA_GetDMAPos

return the current sample position (in mono samples read)
inside the recirculating dma buffer, so the mixing code will know
how many sample are required to fill it up.
===============
*/
pub unsafe fn SNDDMA_GetDMAPos() -> c_int {
    let mut mmtime: MMTIME;
    let mut s: c_int;
    let mut dwWrite: DWORD;

    if dsound_init == 0 {
        return 0;
    }

    mmtime.wType = TIME_SAMPLES;
    {
        let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
        ((*pdsbuf_vt).GetCurrentPosition)(pDSBuf, &mut mmtime.u.sample as *mut u32, &mut dwWrite as *mut u32);
    }

    s = mmtime.u.sample as i32;

    s >>= sample16;

    s &= (dma.samples - 1);

    return s;
}

/*
==============
SNDDMA_BeginPainting

Makes sure dma.buffer is valid
===============
*/
pub unsafe fn SNDDMA_BeginPainting() {
    let mut reps: c_int;
    let mut dwSize2: DWORD;
    let mut pbuf: *mut DWORD;
    let mut pbuf2: *mut DWORD;
    let mut hresult: HRESULT;
    let mut dwStatus: DWORD;

    if pDSBuf.is_null() {
        return;
    }

    // if the buffer was lost or stopped, restore it and/or restart it
    {
        let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
        if ((*pdsbuf_vt).GetStatus)(pDSBuf, &mut dwStatus as *mut u32) != DS_OK as i32 {
            Com_Printf(b"Couldn't get sound buffer status\n\0".as_ptr() as *const c_char);
        }
    }

    if (dwStatus & DSBSTATUS_BUFFERLOST) != 0 {
        let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
        ((*pdsbuf_vt).Restore)(pDSBuf);
    }

    if (dwStatus & DSBSTATUS_PLAYING) == 0 {
        let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
        ((*pdsbuf_vt).Play)(pDSBuf, 0, 0, DSBPLAY_LOOPING);
    }

    // lock the dsound buffer

    reps = 0;
    let dma_ptr = &mut dma as *mut _;
    (*dma_ptr).buffer = ptr::null_mut();

    loop {
        {
            let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
            hresult = ((*pdsbuf_vt).Lock)(pDSBuf, 0, gSndBufSize, &mut pbuf as *mut *mut DWORD as *mut *mut c_void, &mut locksize as *mut u32, &mut pbuf2 as *mut *mut DWORD as *mut *mut c_void, &mut dwSize2 as *mut u32, 0);
        }

        if hresult == DS_OK {
            break;
        }

        if hresult != DSERR_BUFFERLOST {
            Com_Printf(b"SNDDMA_BeginPainting: Lock failed with error '%s'\n\0".as_ptr() as *const c_char, DSoundError(hresult));
            S_Shutdown();
            return;
        } else {
            let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
            ((*pdsbuf_vt).Restore)(pDSBuf);
        }

        reps += 1;
        if reps > 2 {
            return;
        }
    }
    (*dma_ptr).buffer = pbuf as *mut c_void;
}

/*
==============
SNDDMA_Submit

Send sound to device if buffer isn't really the dma buffer
Also unlocks the dsound buffer
===============
*/
pub unsafe fn SNDDMA_Submit() {
    // unlock the dsound buffer
    if !pDSBuf.is_null() {
        let pdsbuf_vt = *(pDSBuf as *const *const DSBufferVtable);
        ((*pdsbuf_vt).Unlock)(pDSBuf, dma.buffer, locksize, ptr::null_mut(), 0);
    }
}

/*
=================
SNDDMA_Activate

When we change windows we need to do this
=================
*/
pub unsafe fn SNDDMA_Activate(bAppActive: c_int) {
    if s_UseOpenAL != 0 {
        S_AL_MuteAllSounds(if bAppActive == 0 { 1 } else { 0 });
    }

    if pDS.is_null() {
        return;
    }

    {
        let pds_vt = *(pDS as *const *const DSVtable);
        if ((*pds_vt).SetCooperativeLevel)(pDS, g_wv.hWnd, DSSCL_PRIORITY) != DS_OK as i32 {
            Com_Printf(b"sound SetCooperativeLevel failed\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
        }
    }
}

// COM interface vtables (simplified for this port)
#[repr(C)]
pub struct DSVtable {
    pub QueryInterface: unsafe extern "stdcall" fn(*mut c_void, *const [u8; 16], *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "stdcall" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "stdcall" fn(*mut c_void) -> u32,
    pub CreateSoundBuffer: unsafe extern "stdcall" fn(*mut c_void, *mut DSBUFFERDESC, *mut LPDIRECTSOUNDBUFFER, *mut c_void) -> HRESULT,
    pub GetCaps: unsafe extern "stdcall" fn(*mut c_void, *mut DSBCAPS) -> HRESULT,
    pub DuplicateSoundBuffer: unsafe extern "stdcall" fn(*mut c_void, *mut c_void, *mut LPDIRECTSOUNDBUFFER) -> HRESULT,
    pub SetCooperativeLevel: unsafe extern "stdcall" fn(*mut c_void, *mut c_void, u32) -> HRESULT,
    pub Compact: unsafe extern "stdcall" fn(*mut c_void) -> HRESULT,
    pub GetSpeakerConfig: unsafe extern "stdcall" fn(*mut c_void, *mut u32) -> HRESULT,
    pub SetSpeakerConfig: unsafe extern "stdcall" fn(*mut c_void, u32) -> HRESULT,
    pub Initialize: unsafe extern "stdcall" fn(*mut c_void, *mut c_void, *mut c_void) -> HRESULT,
}

#[repr(C)]
pub struct DSBufferVtable {
    pub QueryInterface: unsafe extern "stdcall" fn(*mut c_void, *const [u8; 16], *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "stdcall" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "stdcall" fn(*mut c_void) -> u32,
    pub GetCaps: unsafe extern "stdcall" fn(*mut c_void, *mut DSBCAPS) -> HRESULT,
    pub GetCurrentPosition: unsafe extern "stdcall" fn(*mut c_void, *mut u32, *mut u32) -> HRESULT,
    pub GetFormat: unsafe extern "stdcall" fn(*mut c_void, *mut WAVEFORMATEX, u32, *mut u32) -> HRESULT,
    pub GetVolume: unsafe extern "stdcall" fn(*mut c_void, *mut i32) -> HRESULT,
    pub GetPan: unsafe extern "stdcall" fn(*mut c_void, *mut i32) -> HRESULT,
    pub GetFrequency: unsafe extern "stdcall" fn(*mut c_void, *mut u32) -> HRESULT,
    pub GetStatus: unsafe extern "stdcall" fn(*mut c_void, *mut u32) -> HRESULT,
    pub Initialize: unsafe extern "stdcall" fn(*mut c_void, *mut c_void, *mut WAVEFORMATEX) -> HRESULT,
    pub Lock: unsafe extern "stdcall" fn(*mut c_void, u32, u32, *mut *mut c_void, *mut u32, *mut *mut c_void, *mut u32, u32) -> HRESULT,
    pub Play: unsafe extern "stdcall" fn(*mut c_void, u32, u32, u32) -> HRESULT,
    pub SetCurrentPosition: unsafe extern "stdcall" fn(*mut c_void, u32) -> HRESULT,
    pub SetFormat: unsafe extern "stdcall" fn(*mut c_void, *mut WAVEFORMATEX) -> HRESULT,
    pub SetVolume: unsafe extern "stdcall" fn(*mut c_void, i32) -> HRESULT,
    pub SetPan: unsafe extern "stdcall" fn(*mut c_void, i32) -> HRESULT,
    pub SetFrequency: unsafe extern "stdcall" fn(*mut c_void, u32) -> HRESULT,
    pub Stop: unsafe extern "stdcall" fn(*mut c_void) -> HRESULT,
    pub Unlock: unsafe extern "stdcall" fn(*mut c_void, *mut c_void, u32, *mut c_void, u32) -> HRESULT,
    pub Restore: unsafe extern "stdcall" fn(*mut c_void) -> HRESULT,
}
