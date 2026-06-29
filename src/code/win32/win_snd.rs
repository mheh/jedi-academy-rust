// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::addr_of_mut;

// Extern declarations
extern "C" {
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn S_Shutdown();
    fn S_AL_MuteAllSounds(mute: c_int);

    // Windows API functions
    fn LoadLibrary(lpLibFileName: *const c_char) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const c_char) -> *mut c_void;
    fn FreeLibrary(hLibModule: *mut c_void) -> c_int;
    fn Sleep(dwMilliseconds: u32);

    // From C runtime
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    // Global variables from other modules
    static mut dma: dma_t;
    static mut g_wv: g_wv_t;
    static s_khz: *mut cvar_s;
    static mut s_UseOpenAL: c_int;
}

// Type aliases for Windows COM types
type HRESULT = c_int;
type DWORD = u32;
type LPDIRECTSOUND = *mut c_void;
type LPDIRECTSOUNDBUFFER = *mut c_void;
type HINSTANCE = *mut c_void;

// Constants for DirectSound error codes and constants
const DS_OK: HRESULT = 0;
const DSERR_BUFFERLOST: HRESULT = -2147483648i32;
const DSERR_INVALIDCALL: HRESULT = -2147483647i32;
const DSERR_INVALIDPARAM: HRESULT = -2147483646i32;
const DSERR_PRIOLEVELNEEDED: HRESULT = -2147483645i32;
const DSERR_ALLOCATED: HRESULT = -2147483644i32;
const DSERR_UNINITIALIZED: HRESULT = -2147483643i32;
const DSERR_UNSUPPORTED: HRESULT = -2147483642i32;
const DSERR_CONTROLUNAVAIL: HRESULT = -2147483641i32;

// Constants for DirectSound buffer flags
const DSBCAPS_CTRLFREQUENCY: DWORD = 0x00000001;
const DSBCAPS_LOCHARDWARE: DWORD = 0x00000004;
const DSBCAPS_LOCSOFTWARE: DWORD = 0x00000008;

// Constants for cooperative levels
const DSSCL_NORMAL: DWORD = 1;
const DSSCL_PRIORITY: DWORD = 2;

// Constants for play flags
const DSBPLAY_LOOPING: DWORD = 0x00000001;

// Constants for status flags
const DSBSTATUS_PLAYING: DWORD = 0x00000001;
const DSBSTATUS_BUFFERLOST: DWORD = 0x00000002;

// Constants for time format
const TIME_SAMPLES: DWORD = 1;

// Wave format constants
const WAVE_FORMAT_PCM: u16 = 1;

// Constants
const SECONDARY_BUFFER_SIZE: DWORD = 0x10000;

const qfalse: c_int = 0;
const qtrue: c_int = 1;

#[repr(C)]
pub struct WAVEFORMATEX {
    pub wFormatTag: u16,
    pub nChannels: u16,
    pub nSamplesPerSec: u32,
    pub nAvgBytesPerSec: u32,
    pub nBlockAlign: u16,
    pub wBitsPerSample: u16,
    pub cbSize: u16,
}

#[repr(C)]
pub struct DSBUFFERDESC {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwBufferBytes: DWORD,
    pub dwReserved: DWORD,
    pub lpwfxFormat: *mut WAVEFORMATEX,
    pub guid3DAlgorithm: [u8; 16],
}

#[repr(C)]
pub struct DSBCAPS {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub dwBufferBytes: DWORD,
    pub dwUnlockTransferRate: DWORD,
    pub dwPlayCpuOverhead: DWORD,
}

#[repr(C)]
pub union MMTIME_u {
    pub sample: u32,
    pub ms: u32,
}

#[repr(C)]
pub struct MMTIME {
    pub wType: DWORD,
    pub u: MMTIME_u,
}

// Opaque types for structures from other modules
#[repr(C)]
pub struct dma_t {
    pub channels: c_int,
    pub samplebits: c_int,
    pub speed: c_int,
    pub samples: c_int,
    pub submission_chunk: c_int,
    pub buffer: *mut c_void,
}

#[repr(C)]
pub struct g_wv_t {
    pub hWnd: *mut c_void,
}

#[repr(C)]
pub struct cvar_s {
    pub integer: c_int,
}

// Function pointer type for DirectSoundCreate
type DirectSoundCreateFn = unsafe extern "C" fn(
    lpGUID: *mut c_void,
    lplpDS: *mut LPDIRECTSOUND,
    pUnkOuter: *mut c_void,
) -> HRESULT;

// Global function pointer for DirectSoundCreate
static mut pDirectSoundCreate: Option<DirectSoundCreateFn> = None;

// Macro equivalent: iDirectSoundCreate(a,b,c) -> pDirectSoundCreate(a,b,c)
#[inline]
unsafe fn iDirectSoundCreate(
    a: *mut c_void,
    b: *mut LPDIRECTSOUND,
    c: *mut c_void,
) -> HRESULT {
    if let Some(fn_ptr) = pDirectSoundCreate {
        fn_ptr(a, b, c)
    } else {
        -1
    }
}

static mut dsound_init: c_int = 0; // qboolean
static mut sample16: c_int = 0;
static mut gSndBufSize: DWORD = 0;
static mut locksize: DWORD = 0;
static mut pDS: LPDIRECTSOUND = std::ptr::null_mut();
static mut pDSBuf: LPDIRECTSOUNDBUFFER = std::ptr::null_mut();
static mut pDSPBuf: LPDIRECTSOUNDBUFFER = std::ptr::null_mut();
static mut hInstDS: HINSTANCE = std::ptr::null_mut();

fn DSoundError(error: HRESULT) -> &'static str {
    match error {
        DSERR_BUFFERLOST => "DSERR_BUFFERLOST",
        DSERR_INVALIDCALL => "DSERR_INVALIDCALLS",
        DSERR_INVALIDPARAM => "DSERR_INVALIDPARAM",
        DSERR_PRIOLEVELNEEDED => "DSERR_PRIOLEVELNEEDED",
        DSERR_ALLOCATED => "DSERR_ALLOCATED",
        DSERR_UNINITIALIZED => "DSERR_UNINITIALIZED",
        DSERR_UNSUPPORTED => "DSERR_UNSUPPORTED ",
        _ => "unknown",
    }
}

/*
==================
SNDDMA_Shutdown
==================
*/
#[no_mangle]
pub extern "C" fn SNDDMA_Shutdown() {
    unsafe {
        Com_DPrintf(b"Shutting down sound system\n\0".as_ptr() as *const c_char);

        if !pDS.is_null() {
            Com_DPrintf(b"Destroying DS buffers\n\0".as_ptr() as *const c_char);
            if !pDS.is_null() {
                Com_DPrintf(
                    b"...setting NORMAL coop level\n\0".as_ptr() as *const c_char,
                );
                // pDS->SetCooperativeLevel( g_wv.hWnd, DSSCL_NORMAL );
            }

            if !pDSBuf.is_null() {
                Com_DPrintf(
                    b"...stopping and releasing sound buffer\n\0".as_ptr() as *const c_char,
                );
                // pDSBuf->Stop( );
                // pDSBuf->Release( );
            }

            // only release primary buffer if it's not also the mixing buffer we just released
            if !pDSPBuf.is_null() && pDSBuf != pDSPBuf {
                Com_DPrintf(b"...releasing primary buffer\n\0".as_ptr() as *const c_char);
                // pDSPBuf->Release( );
            }
            pDSBuf = std::ptr::null_mut();
            pDSPBuf = std::ptr::null_mut();

            dma.buffer = std::ptr::null_mut();

            Com_DPrintf(b"...releasing DS object\n\0".as_ptr() as *const c_char);
            // pDS->Release( );
        }

        if !hInstDS.is_null() {
            Com_DPrintf(b"...freeing DSOUND.DLL\n\0".as_ptr() as *const c_char);
            FreeLibrary(hInstDS);
            hInstDS = std::ptr::null_mut();
        }

        pDS = std::ptr::null_mut();
        pDSBuf = std::ptr::null_mut();
        pDSPBuf = std::ptr::null_mut();
        dsound_init = qfalse;
        memset(
            addr_of_mut!(dma) as *mut c_void,
            0,
            std::mem::size_of::<dma_t>(),
        );
    }
}

/*
==================
SNDDMA_Init

Initialize direct sound
Returns false if failed
==================
*/
#[no_mangle]
pub extern "C" fn SNDDMA_Init() -> c_int {
    unsafe {
        memset(
            addr_of_mut!(dma) as *mut c_void,
            0,
            std::mem::size_of::<dma_t>(),
        );
        dsound_init = qfalse;

        if SNDDMA_InitDS() == 0 {
            return qfalse;
        }

        dsound_init = qtrue;

        Com_DPrintf(b"Completed successfully\n\0".as_ptr() as *const c_char);

        return qtrue;
    }
}

fn SNDDMA_InitDS() -> c_int {
    unsafe {
        let mut hresult: HRESULT;
        let mut pauseTried: c_int;
        let mut dsbuf: DSBUFFERDESC = std::mem::zeroed();
        let mut dsbcaps: DSBCAPS = std::mem::zeroed();
        let mut format: WAVEFORMATEX = std::mem::zeroed();

        Com_Printf(b"Initializing DirectSound\n\0".as_ptr() as *const c_char);

        if hInstDS.is_null() {
            Com_DPrintf(b"...loading dsound.dll: \0".as_ptr() as *const c_char);

            hInstDS = LoadLibrary(b"dsound.dll\0".as_ptr() as *const c_char);

            if hInstDS.is_null() {
                Com_Printf(b"failed\n\0".as_ptr() as *const c_char);
                return 0;
            }

            Com_DPrintf(b"ok\n\0".as_ptr() as *const c_char);
            pDirectSoundCreate = std::mem::transmute(GetProcAddress(
                hInstDS,
                b"DirectSoundCreate\0".as_ptr() as *const c_char,
            ));

            if pDirectSoundCreate.is_none() {
                Com_Printf(b"*** couldn't get DS proc addr ***\n\0".as_ptr() as *const c_char);
                return 0;
            }
        }

        Com_DPrintf(b"...creating DS object: \0".as_ptr() as *const c_char);
        pauseTried = qfalse;
        loop {
            hresult = iDirectSoundCreate(std::ptr::null_mut(), addr_of_mut!(pDS), std::ptr::null_mut());
            if hresult == DS_OK {
                break;
            }
            if hresult != DSERR_ALLOCATED {
                Com_Printf(b"failed\n\0".as_ptr() as *const c_char);
                return 0;
            }

            if pauseTried != 0 {
                Com_Printf(
                    b"failed, hardware already in use\n\0".as_ptr() as *const c_char,
                );
                return 0;
            }
            // first try just waiting five seconds and trying again
            // this will handle the case of a sysyem beep playing when the
            // game starts
            Com_DPrintf(b"retrying...\n\0".as_ptr() as *const c_char);
            Sleep(3000);
            pauseTried = qtrue;
        }
        Com_DPrintf(b"ok\n\0".as_ptr() as *const c_char);

        Com_DPrintf(b"...setting DSSCL_PRIORITY coop level: \0".as_ptr() as *const c_char);

        if DS_OK != DS_OK {
            Com_Printf(b"failed\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
            return qfalse;
        }
        Com_DPrintf(b"ok\n\0".as_ptr() as *const c_char);

        // create the secondary buffer we'll actually work with
        dma.channels = 2;
        dma.samplebits = 16;

        if (*s_khz).integer == 44 {
            dma.speed = 44100;
        } else if (*s_khz).integer == 22 {
            dma.speed = 22050;
        } else {
            dma.speed = 11025;
        }

        memset(addr_of_mut!(format) as *mut c_void, 0, std::mem::size_of::<WAVEFORMATEX>());
        format.wFormatTag = WAVE_FORMAT_PCM;
        format.nChannels = dma.channels as u16;
        format.wBitsPerSample = dma.samplebits as u16;
        format.nSamplesPerSec = dma.speed as u32;
        format.nBlockAlign = (format.nChannels as u32 * format.wBitsPerSample as u32 / 8) as u16;
        format.cbSize = 0;
        format.nAvgBytesPerSec = format.nSamplesPerSec * format.nBlockAlign as u32;

        memset(
            addr_of_mut!(dsbuf) as *mut c_void,
            0,
            std::mem::size_of::<DSBUFFERDESC>(),
        );
        dsbuf.dwSize = std::mem::size_of::<DSBUFFERDESC>() as DWORD;

        #[allow(non_upper_case_globals)]
        const idDSBCAPS_GETCURRENTPOSITION2: DWORD = 0x00010000;

        dsbuf.dwFlags = DSBCAPS_CTRLFREQUENCY | DSBCAPS_LOCHARDWARE | idDSBCAPS_GETCURRENTPOSITION2;
        dsbuf.dwBufferBytes = SECONDARY_BUFFER_SIZE;
        dsbuf.lpwfxFormat = addr_of_mut!(format);

        Com_DPrintf(b"...creating secondary buffer: \0".as_ptr() as *const c_char);
        // hresult = pDS->CreateSoundBuffer(&dsbuf, &pDSBuf, NULL);
        hresult = DS_OK - 1;

        if hresult != DS_OK {
            if hresult == DSERR_CONTROLUNAVAIL {
                Com_Printf(
                    b" - Ancient version of DirectX - this will slow FPS\n\0".as_ptr()
                        as *const c_char,
                );
                dsbuf.dwFlags &= !idDSBCAPS_GETCURRENTPOSITION2; // lose this DX8 cursor-position feature, and try again
                // hresult = pDS->CreateSoundBuffer(&dsbuf, &pDSBuf, NULL);
                hresult = DS_OK - 1;
            }

            if hresult != DS_OK {
                // we can't even specify sounds should be in hardware?...
                //
                //  ( this seems to happen on integrated sound devices (eg SoundMax), regardless of DX version )
                //
                dsbuf.dwFlags = DSBCAPS_CTRLFREQUENCY; // note that DX docs say that this can still use hardware if it wants to, since neither DSBCAPS_LOCHARDWARE nor DSBCAPS_LOCSOFTWARE were specified
                // hresult = pDS->CreateSoundBuffer(&dsbuf, &pDSBuf, NULL);
                hresult = DS_OK - 1;
                if hresult != DS_OK {
                    Com_Printf(
                        b"failed to create secondary buffer - %s\n\0".as_ptr() as *const c_char,
                        DSoundError(hresult).as_ptr() as *const c_char,
                    );
                    SNDDMA_Shutdown();
                    return qfalse;
                }
            }
        }
        Com_Printf(b"locked hardware.  ok\n\0".as_ptr() as *const c_char);

        // Make sure mixer is active
        // if ( DS_OK != pDSBuf->Play(0, 0, DSBPLAY_LOOPING) )
        if DS_OK != DS_OK {
            Com_Printf(b"*** Looped sound play failed ***\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
            return qfalse;
        }

        memset(addr_of_mut!(dsbcaps) as *mut c_void, 0, std::mem::size_of::<DSBCAPS>());
        dsbcaps.dwSize = std::mem::size_of::<DSBCAPS>() as DWORD;
        // get the returned buffer size
        // if ( DS_OK != pDSBuf->GetCaps (&dsbcaps) )
        if DS_OK != DS_OK {
            Com_Printf(b"*** GetCaps failed ***\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
            return qfalse;
        }

        gSndBufSize = dsbcaps.dwBufferBytes;

        dma.channels = format.nChannels as c_int;
        dma.samplebits = format.wBitsPerSample as c_int;
        dma.speed = format.nSamplesPerSec as c_int;
        dma.samples = (gSndBufSize / ((dma.samplebits) / 8)) as c_int;
        dma.submission_chunk = 1;
        dma.buffer = std::ptr::null_mut(); // must be locked first

        sample16 = ((dma.samplebits) / 8) - 1;

        SNDDMA_BeginPainting();
        if !dma.buffer.is_null() {
            memset(dma.buffer, 0, (dma.samples * dma.samplebits / 8) as usize);
        }
        SNDDMA_Submit();
        return 1;
    }
}

/*
==============
SNDDMA_GetDMAPos

return the current sample position (in mono samples read)
inside the recirculating dma buffer, so the mixing code will know
how many sample are required to fill it up.
===============
*/
#[no_mangle]
pub extern "C" fn SNDDMA_GetDMAPos() -> c_int {
    unsafe {
        let mut mmtime: MMTIME = std::mem::zeroed();
        let mut s: c_int;
        let mut dwWrite: DWORD = 0;

        if dsound_init == 0 {
            return 0;
        }

        mmtime.wType = TIME_SAMPLES;
        // pDSBuf->GetCurrentPosition(&mmtime.u.sample, &dwWrite);

        s = mmtime.u.sample as c_int;

        s >>= sample16;

        s &= (dma.samples - 1);

        return s;
    }
}

/*
==============
SNDDMA_BeginPainting

Makes sure dma.buffer is valid
===============
*/
#[no_mangle]
pub extern "C" fn SNDDMA_BeginPainting() {
    unsafe {
        let mut reps: c_int;
        let mut dwSize2: DWORD = 0;
        let mut pbuf: *mut c_void = std::ptr::null_mut();
        let mut pbuf2: *mut c_void = std::ptr::null_mut();
        let mut hresult: HRESULT;
        let mut dwStatus: DWORD = 0;

        if pDSBuf.is_null() {
            return;
        }

        // if the buffer was lost or stopped, restore it and/or restart it
        // if ( pDSBuf->GetStatus (&dwStatus) != DS_OK ) {
        // 	Com_Printf ("Couldn't get sound buffer status\n");
        // }

        if (dwStatus & DSBSTATUS_BUFFERLOST) != 0 {
            // pDSBuf->Restore ();
        }

        if (dwStatus & DSBSTATUS_PLAYING) == 0 {
            // pDSBuf->Play(0, 0, DSBPLAY_LOOPING);
        }

        // lock the dsound buffer

        reps = 0;
        dma.buffer = std::ptr::null_mut();

        loop {
            // hresult = pDSBuf->Lock(0, gSndBufSize, (void **)&pbuf, &locksize,
            //                        (void **)&pbuf2, &dwSize2, 0)
            hresult = DS_OK - 1;

            if hresult == DS_OK {
                break;
            }

            if hresult != DSERR_BUFFERLOST {
                Com_Printf(
                    b"SNDDMA_BeginPainting: Lock failed with error '%s'\n\0".as_ptr()
                        as *const c_char,
                    DSoundError(hresult).as_ptr() as *const c_char,
                );
                S_Shutdown();
                return;
            } else {
                // pDSBuf->Restore( );
            }

            reps += 1;
            if reps > 2 {
                return;
            }
        }
        dma.buffer = pbuf;
    }
}

/*
==============
SNDDMA_Submit

Send sound to device if buffer isn't really the dma buffer
Also unlocks the dsound buffer
===============
*/
#[no_mangle]
pub extern "C" fn SNDDMA_Submit() {
    unsafe {
        // unlock the dsound buffer
        if !pDSBuf.is_null() {
            // pDSBuf->Unlock(dma.buffer, locksize, NULL, 0);
        }
    }
}

/*
=================
SNDDMA_Activate

When we change windows we need to do this
=================
*/
#[no_mangle]
pub extern "C" fn SNDDMA_Activate(bAppActive: c_int) {
    unsafe {
        if s_UseOpenAL != 0 {
            S_AL_MuteAllSounds(if bAppActive != 0 { 0 } else { 1 });
        }

        if pDS.is_null() {
            return;
        }

        if DS_OK != DS_OK {
            Com_Printf(b"sound SetCooperativeLevel failed\n\0".as_ptr() as *const c_char);
            SNDDMA_Shutdown();
        }
    }
}

// I know this is a bit horrible, but I need to pass our LPDIRECTSOUND ptr to Bink for video playback,
//	and I don't want other modules to have to know about LPDIRECTSOUND handles, hence the int casting
//
// (I'd prefer to use DWORD, but not all modules understand those)
//
#[no_mangle]
pub extern "C" fn SNDDMA_GetDSHandle() -> u32 {
    unsafe { pDS as u32 }
}
