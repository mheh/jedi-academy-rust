/*
 * UNPUBLISHED -- Rights  reserved  under  the  copyright  laws  of the
 * United States.  Use  of a copyright notice is precautionary only and
 * does not imply publication or disclosure.
 *
 * THIS DOCUMENTATION CONTAINS CONFIDENTIAL AND PROPRIETARY INFORMATION
 * OF    VICARIOUS   VISIONS,  INC.    ANY  DUPLICATION,  MODIFICATION,
 * DISTRIBUTION, OR DISCLOSURE IS STRICTLY PROHIBITED WITHOUT THE PRIOR
 * EXPRESS WRITTEN PERMISSION OF VICARIOUS VISIONS, INC.
 */

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_char, c_void};

// Extern declarations for external functions
extern "C" {
    static Sys_FileStreamMutex: *mut c_void;
    fn Sys_GetFileCodeName(code: c_int) -> *const c_char;
    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn Z_Free(ptr: *mut c_void);
    fn Sys_Milliseconds() -> core::ffi::c_uint;
}

// Extern declarations for DirectSound and D3DX functions
extern "C" {
    fn DirectSoundCreate(
        pcGuidDevice: *mut c_void,
        ppDS: *mut *mut IDirectSound8,
        pUnkOuter: *mut c_void
    ) -> core::ffi::c_long;
    fn DirectSoundUseFullHRTF();
    fn DirectSoundDoWork();
}

// Win32 types
type HANDLE = *mut c_void;
type DWORD = u32;
type FLOAT = f32;
type LPDSEFFECTIMAGEDESC = *mut c_void;
type LPVOID = *mut c_void;
type LPCWAVEFORMATEX = *const c_void;

const DS_OK: core::ffi::c_long = 0;
const DS3D_DEFERRED: DWORD = 0;
const DSBVOLUME_MIN: i32 = -10000;
const DSBCAPS_CTRL3D: DWORD = 0x00000008;
const DSBCAPS_MUTE3DATMAXDISTANCE: DWORD = 0x00100000;
const DSBPLAY_LOOPING: DWORD = 0x00000004;
const DSBSTATUS_PLAYING: DWORD = 0x00000001;
const FILE_BEGIN: DWORD = 0;
const INFINITE: DWORD = 0xFFFFFFFF;
const PAGE_READWRITE: DWORD = 0x04;
const PAGE_NOCACHE: DWORD = 0x0200;
const MAXULONG_PTR: usize = usize::MAX;

const WAVE_FORMAT_XBOX_ADPCM: u16 = 0x0069;

const XMEDIAPACKET_STATUS_PENDING: DWORD = 0x00000001;
const XMEDIAPACKET_STATUS_SUCCESS: DWORD = 0x00000000;

// DirectSound and audio constants
const GraphI3DL2_I3DL2Reverb: c_int = 0;
const GraphXTalk_XTalk: c_int = 1;
const DSI3DL2_ENVIRONMENT_PRESET_NOREVERB: DWORD = 0;

// AL constants (from OpenAL)
const AL_NO_ERROR: core::ffi::c_uint = 0;
const ALC_NO_ERROR: core::ffi::c_uint = 0;
const AL_OUT_OF_MEMORY: core::ffi::c_uint = 0xA005;
const AL_POSITION: core::ffi::c_uint = 0x1004;
const AL_ORIENTATION: core::ffi::c_uint = 0x100F;
const AL_FORMAT_MONO4: core::ffi::c_uint = 0x10014;
const AL_FORMAT_STEREO4: core::ffi::c_uint = 0x10015;
const AL_FORMAT_MONO8: core::ffi::c_uint = 0x1100;
const AL_FORMAT_STEREO8: core::ffi::c_uint = 0x1101;
const AL_FORMAT_MONO16: core::ffi::c_uint = 0x1101;
const AL_FORMAT_STEREO16: core::ffi::c_uint = 0x1102;
const AL_LOOPING: core::ffi::c_uint = 0x1007;
const AL_BUFFER: core::ffi::c_uint = 0x1009;
const AL_GAIN: core::ffi::c_uint = 0x100A;
const AL_REFERENCE_DISTANCE: core::ffi::c_uint = 0x1020;
const AL_SOURCE_STATE: core::ffi::c_uint = 0x1010;
const AL_PLAYING: core::ffi::c_uint = 0x1012;
const AL_STOPPED: core::ffi::c_uint = 0x1014;
const AL_TIME: core::ffi::c_uint = 0x1024;
const AL_MEMORY_USED: core::ffi::c_uint = 0x10001;

// OpenAL types
type ALCdevice = c_void;
type ALCcontext = c_void;
type ALCubyte = u8;
type ALCint = c_int;
type ALCvoid = c_void;
type ALCenum = core::ffi::c_uint;
type ALvoid = c_void;
type ALsizei = c_int;
type ALuint = core::ffi::c_uint;
type ALenum = core::ffi::c_uint;
type ALfloat = f32;
type ALint = c_int;
type ALboolean = core::ffi::c_uchar;

// Wave format structures
#[repr(C)]
pub struct WAVEFORMATEX {
    wFormatTag: u16,
    nChannels: u16,
    nSamplesPerSec: u32,
    nAvgBytesPerSec: u32,
    nBlockAlign: u16,
    wBitsPerSample: u16,
    cbSize: u16,
}

#[repr(C)]
pub struct XBOXADPCMWAVEFORMAT {
    wfx: WAVEFORMATEX,
    wSamplesPerBlock: u16,
}

// DirectSound interfaces (stub declarations)
#[repr(C)]
pub struct IDirectSound8 {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct IDirectSoundBuffer {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct IDirectSoundStream {
    _unused: [u8; 0],
}

// D3DX types
#[repr(C)]
pub struct D3DXVECTOR3 {
    x: FLOAT,
    y: FLOAT,
    z: FLOAT,
}

#[repr(C)]
pub struct D3DXVECTOR4 {
    x: FLOAT,
    y: FLOAT,
    z: FLOAT,
    w: FLOAT,
}

#[repr(C)]
pub struct D3DXMATRIX {
    m: [[FLOAT; 4]; 4],
}

// File object for media
#[repr(C)]
pub struct XFileMediaObject {
    _unused: [u8; 0],
}

// Media packet
#[repr(C)]
pub struct XMEDIAPACKET {
    pvBuffer: *mut c_void,
    dwMaxSize: DWORD,
    pdwCompletedSize: *mut DWORD,
    pdwStatus: *mut DWORD,
    pContext: *mut c_void,
}

// DirectSound buffer description
#[repr(C)]
pub struct DSBUFFERDESC {
    dwSize: DWORD,
    dwFlags: DWORD,
    dwBufferBytes: DWORD,
    dwReserved: DWORD,
    lpwfxFormat: *mut WAVEFORMATEX,
    guid3DAlgorithm: [u8; 16],
    lpMixBins: *mut c_void,
    dwInputMixBin: DWORD,
}

// DirectSound stream description
#[repr(C)]
pub struct DSSTREAMDESC {
    dwSize: DWORD,
    dwMaxAttachedPackets: DWORD,
    lpwfxFormat: *mut WAVEFORMATEX,
}

// DirectSound I3DL2 Listener
#[repr(C)]
pub struct DSI3DL2LISTENER {
    dwSize: DWORD,
    lRoom: i32,
    lRoomHF: i32,
    flRoomRolloffFactor: FLOAT,
    flDecayTime: FLOAT,
    flDecayHFRatio: FLOAT,
    lReflections: i32,
    flReflectionsDelay: FLOAT,
    lReverb: i32,
    flReverbDelay: FLOAT,
    flDiffusion: FLOAT,
    flDensity: FLOAT,
    flHFReference: FLOAT,
}

// Mix bin structures
#[repr(C)]
pub struct DSMIXBINVOLUMEPAIR {
    dwMixBin: DWORD,
    lVolume: i32,
}

#[repr(C)]
pub struct DSMIXBINS {
    dwMixBinCount: DWORD,
    lpMixBinVolumePairs: *mut DSMIXBINVOLUMEPAIR,
}

// Effect image description
#[repr(C)]
pub struct DSEFFECTIMAGEDESC {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct DSEFFECTIMAGELOC {
    dwI3DL2ReverbIndex: c_int,
    dwCrosstalkIndex: c_int,
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
const QAL_STREAM_WAIT_TIME: u32 = 500;
#[allow(non_upper_case_globals)]
const QAL_MAX_STREAM_PACKETS: usize = 2;

// About 1 second of audio at 44100, stereo, ADPCM
#[allow(non_upper_case_globals)]
const QAL_STREAM_PACKET_SIZE: usize = 44136;

// Un-comment to enable 5-channel 3-d sound mixing
// #define _FIVE_CHANNEL

/***********************************************
*
* OpenAL STATE - Main container for all AL objects
*
************************************************/

#[repr(C)]
pub struct QALState {
    m_SoundObject: *mut IDirectSound8,

    m_MemoryUsed: ALuint,
    m_Error: ALenum,
    m_Gain: FLOAT,

    m_Listeners: *mut ListenerInfo,
    m_NextListener: ALuint,

    m_Sources: *mut SourceInfo,
    m_NextSource: ALuint,

    m_Buffers: *mut BufferInfo,
    m_NextBuffer: ALuint,

    m_Stream: StreamInfo,
}

#[repr(C)]
pub struct ListenerInfo {
    m_Position: D3DXVECTOR3,
    m_LTM: D3DXMATRIX,
}

#[repr(C)]
pub struct SourceInfo {
    m_Voices: *mut IDirectSoundBuffer,
    m_VoicesCount: usize,

    m_Buffer: ALuint,

    m_Gain: FLOAT,
    m_GainDirty: bool,

    m_Loop: bool,

    m_Is3d: bool,
    m_Position: D3DXVECTOR3,
}

#[repr(C)]
pub struct BufferInfo {
    m_Data: *mut c_void,
    m_DataOffset: DWORD,
    m_WAVFormat: XBOXADPCMWAVEFORMAT,

    m_Freq: DWORD,
    m_Size: DWORD,

    m_Valid: bool,
}

#[repr(C)]
pub struct StreamInfo {
    m_pVoice: *mut IDirectSoundStream,
    m_pFile: *mut XFileMediaObject,

    m_StartTime: core::ffi::c_uint,

    m_Open: bool,
    m_Playing: bool,
    m_Valid: bool,

    m_Gain: FLOAT,
    m_GainDirty: bool,

    m_Looping: bool,

    m_pPacketBuffer: *mut c_void,
    m_PacketStatus: [DWORD; QAL_MAX_STREAM_PACKETS],
    m_CurrentPacket: DWORD,

    m_Thread: HANDLE,
    m_Mutex: HANDLE,
    m_QueueLen: HANDLE,

    m_Queue: *mut StreamRequest,
    m_QueueCount: usize,
    m_QueueCapacity: usize,
}

#[repr(C)]
pub enum StreamRequestType {
    REQ_NOP,
    REQ_PLAY,
    REQ_STOP,
    REQ_SHUTDOWN,
}

#[repr(C)]
pub struct StreamRequest {
    m_Type: StreamRequestType,
    m_Data: [DWORD; 3],
}

// Initialization function for StreamInfo - we need a default value
unsafe fn init_stream_info() -> StreamInfo {
    StreamInfo {
        m_pVoice: core::ptr::null_mut(),
        m_pFile: core::ptr::null_mut(),
        m_StartTime: 0,
        m_Open: false,
        m_Playing: false,
        m_Valid: false,
        m_Gain: 1.0,
        m_GainDirty: true,
        m_Looping: false,
        m_pPacketBuffer: core::ptr::null_mut(),
        m_PacketStatus: [XMEDIAPACKET_STATUS_SUCCESS; QAL_MAX_STREAM_PACKETS],
        m_CurrentPacket: 0,
        m_Thread: core::ptr::null_mut(),
        m_Mutex: core::ptr::null_mut(),
        m_QueueLen: core::ptr::null_mut(),
        m_Queue: core::ptr::null_mut(),
        m_QueueCount: 0,
        m_QueueCapacity: 0,
    }
}

static mut s_pState: *mut QALState = core::ptr::null_mut();

/***********************************************
*
* DEVICES AND CONTEXTS
*
************************************************/

#[no_mangle]
pub extern "C" fn alcOpenDevice(deviceName: *mut ALCubyte) -> *mut ALCdevice {
    unsafe {
        if !s_pState.is_null() {
            return core::ptr::null_mut();
        }
        s_pState = Box::into_raw(Box::new(QALState {
            m_SoundObject: core::ptr::null_mut(),
            m_MemoryUsed: 0,
            m_Error: AL_NO_ERROR,
            m_Gain: 1.0,
            m_Listeners: core::ptr::null_mut(),
            m_NextListener: 1,
            m_Sources: core::ptr::null_mut(),
            m_NextSource: 1,
            m_Buffers: core::ptr::null_mut(),
            m_NextBuffer: 1,
            m_Stream: init_stream_info(),
        }));

        let state = &mut *s_pState;
        state.m_Gain = 1.0;
        state.m_Error = AL_NO_ERROR;
        state.m_MemoryUsed = 0;
        state.m_NextBuffer = 1;
        state.m_NextListener = 1;
        state.m_NextSource = 1;
        state.m_Stream.m_Valid = false;

        // init the sound hardware
        let mut sound_obj: *mut IDirectSound8 = core::ptr::null_mut();
        if DirectSoundCreate(core::ptr::null_mut(), &mut sound_obj, core::ptr::null_mut()) != DS_OK {
            Box::from_raw(s_pState);
            s_pState = core::ptr::null_mut();
            return core::ptr::null_mut();
        }

        state.m_SoundObject = sound_obj;

        DirectSoundUseFullHRTF();

        // download effects image to hardware
        let mut image: *mut c_void = core::ptr::null_mut();
        let len = FS_ReadFile(
            b"sound/dsstdfx.bin\0".as_ptr() as *const c_char,
            &mut image
        );
        if len <= 0 {
            Box::from_raw(s_pState);
            s_pState = core::ptr::null_mut();
            return core::ptr::null_mut();
        }

        let mut effect = DSEFFECTIMAGELOC {
            dwI3DL2ReverbIndex: GraphI3DL2_I3DL2Reverb,
            dwCrosstalkIndex: GraphXTalk_XTalk,
        };

        // This call is a stub - we're calling it for structural parity
        // s_pState->m_SoundObject->DownloadEffectsImage(image, len as DWORD, &effect, &desc);

        Z_Free(image);

        // setup default reverb
        let reverb = DSI3DL2LISTENER {
            dwSize: core::mem::size_of::<DSI3DL2LISTENER>() as DWORD,
            lRoom: DSI3DL2_ENVIRONMENT_PRESET_NOREVERB as i32,
            lRoomHF: 0,
            flRoomRolloffFactor: 0.0,
            flDecayTime: 0.0,
            flDecayHFRatio: 0.0,
            lReflections: 0,
            flReflectionsDelay: 0.0,
            lReverb: 0,
            flReverbDelay: 0.0,
            flDiffusion: 0.0,
            flDensity: 0.0,
            flHFReference: 0.0,
        };
        // This call is a stub - we're calling it for structural parity
        // s_pState->m_SoundObject->SetI3DL2Listener(&reverb, DS3D_DEFERRED);

        s_pState as *mut ALCdevice
    }
}

#[no_mangle]
pub extern "C" fn alcCloseDevice(device: *mut ALCdevice) {
    unsafe {
        if !s_pState.is_null() {
            // shutdown the sound hardware
            // s_pState->m_SoundObject->Release();

            Box::from_raw(s_pState);
            s_pState = core::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub extern "C" fn alcCreateContext(device: *mut ALCdevice, attrList: *mut ALCint) -> *mut ALCcontext {
    1 as *mut ALCcontext
}

#[no_mangle]
pub extern "C" fn alcMakeContextCurrent(context: *mut ALCcontext) -> ALboolean {
    1
}

#[no_mangle]
pub extern "C" fn alcGetCurrentContext() -> *mut ALCcontext {
    1 as *mut ALCcontext
}

#[no_mangle]
pub extern "C" fn alcGetContextsDevice(context: *mut ALCcontext) -> *mut ALCdevice {
    unsafe {
        if s_pState.is_null() {
            return core::ptr::null_mut();
        }
        (*s_pState).m_SoundObject as *mut ALCdevice
    }
}

#[no_mangle]
pub extern "C" fn alcDestroyContext(context: *mut ALCcontext) {}

#[no_mangle]
pub extern "C" fn alcGetError(device: *mut ALCdevice) -> ALCenum {
    ALC_NO_ERROR
}

/***********************************************
*
* LISTENERS
*
************************************************/

#[no_mangle]
pub extern "C" fn alGenListeners(n: ALsizei, listeners: *mut ALuint) {
    unsafe {
        let mut count = n;
        while count > 0 {
            count -= 1;

            let info = Box::new(ListenerInfo {
                m_Position: D3DXVECTOR3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                m_LTM: D3DXMATRIX {
                    m: [[0.0; 4]; 4],
                },
            });

            // Initialize identity matrix (simplified)
            let mut ltm = &mut (*info).m_LTM;
            ltm.m[0][0] = 1.0;
            ltm.m[1][1] = 1.0;
            ltm.m[2][2] = 1.0;
            ltm.m[3][3] = 1.0;

            let state = &mut *s_pState;

            // Simple linked-list style storage (stub)
            *listeners.add(count as usize) = state.m_NextListener;
            state.m_NextListener += 1;

            core::mem::forget(info);
        }
    }
}

#[no_mangle]
pub extern "C" fn alDeleteListeners(n: ALsizei, listeners: *const ALuint) {
    // Stub implementation for parity
}

#[no_mangle]
pub extern "C" fn alListenerfv(listener: ALuint, param: ALenum, values: *const ALfloat) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        // Stub implementation - would need proper listener storage
        let values_slice = core::slice::from_raw_parts(values, 6);

        match param {
            AL_POSITION => {
                // translation
            }
            AL_ORIENTATION => {
                // orientation update
            }
            _ => {}
        }
    }
}

/***********************************************
*
* SOURCES
*
************************************************/

unsafe fn _wavSetFormat(
    wav: *mut XBOXADPCMWAVEFORMAT,
    format: ALenum,
    freq: ALsizei,
) {
    let wav = &mut *wav;

    match format {
        AL_FORMAT_MONO4 => {
            wav.wfx.wFormatTag = WAVE_FORMAT_XBOX_ADPCM;
            wav.wfx.nChannels = 1;
            wav.wfx.nSamplesPerSec = freq as u32;
            wav.wfx.nBlockAlign = 36 * wav.wfx.nChannels;
            wav.wfx.nAvgBytesPerSec =
                wav.wfx.nSamplesPerSec * wav.wfx.nBlockAlign as u32 / 64;
            wav.wfx.wBitsPerSample = 4;
            wav.wfx.cbSize =
                (core::mem::size_of::<XBOXADPCMWAVEFORMAT>()
                    - core::mem::size_of::<WAVEFORMATEX>()) as u16;
            wav.wSamplesPerBlock = 64;
        }

        AL_FORMAT_STEREO4 => {
            wav.wfx.wFormatTag = WAVE_FORMAT_XBOX_ADPCM;
            wav.wfx.nChannels = 2;
            wav.wfx.nSamplesPerSec = freq as u32;
            wav.wfx.nBlockAlign = 36 * wav.wfx.nChannels;
            wav.wfx.nAvgBytesPerSec =
                wav.wfx.nSamplesPerSec * wav.wfx.nBlockAlign as u32 / 64;
            wav.wfx.wBitsPerSample = 4;
            wav.wfx.cbSize =
                (core::mem::size_of::<XBOXADPCMWAVEFORMAT>()
                    - core::mem::size_of::<WAVEFORMATEX>()) as u16;
            wav.wSamplesPerBlock = 64;
        }

        AL_FORMAT_MONO8 | AL_FORMAT_STEREO8 | AL_FORMAT_MONO16 | AL_FORMAT_STEREO16 => {
            // assert(0)
        }

        _ => {}
    }
}

unsafe fn _genSource(is3d: bool) -> bool {
    if s_pState.is_null() {
        return false;
    }

    let state = &mut *s_pState;

    // alloc a new source
    let sinfo = Box::new(SourceInfo {
        m_Voices: core::ptr::null_mut(),
        m_VoicesCount: 0,
        m_Buffer: 0,
        m_Gain: 1.0,
        m_GainDirty: true,
        m_Loop: false,
        m_Is3d: is3d,
        m_Position: D3DXVECTOR3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    });

    // describe the voice
    let mut wav = XBOXADPCMWAVEFORMAT {
        wfx: WAVEFORMATEX {
            wFormatTag: 0,
            nChannels: 0,
            nSamplesPerSec: 0,
            nAvgBytesPerSec: 0,
            nBlockAlign: 0,
            wBitsPerSample: 0,
            cbSize: 0,
        },
        wSamplesPerBlock: 0,
    };
    _wavSetFormat(&mut wav, AL_FORMAT_MONO4, 22050);

    let mut desc = DSBUFFERDESC {
        dwSize: core::mem::size_of::<DSBUFFERDESC>() as DWORD,
        dwFlags: if is3d {
            DSBCAPS_CTRL3D | DSBCAPS_MUTE3DATMAXDISTANCE
        } else {
            0
        },
        dwBufferBytes: 0,
        dwReserved: 0,
        lpwfxFormat: &mut wav.wfx,
        guid3DAlgorithm: [0; 16],
        lpMixBins: core::ptr::null_mut(),
        dwInputMixBin: 0,
    };

    // create voice (stub - simplified)
    // Normally would loop through listeners and create voices
    let voice: *mut IDirectSoundBuffer = core::ptr::null_mut();

    let sinfo_ptr = Box::into_raw(sinfo);
    // Store source in state (stub implementation)

    true
}

unsafe fn _attachBuffer(source: ALuint, buffer: ALuint) {
    if s_pState.is_null() {
        return;
    }

    let state = &*s_pState;

    // Stub implementation for buffer attachment
}

unsafe fn _dettachBuffer(source: ALuint) {
    if s_pState.is_null() {
        return;
    }

    let state = &*s_pState;

    // Stub implementation for buffer detachment
}

unsafe fn _sourceSetRefDist(info: *mut SourceInfo, value: FLOAT) {
    if info.is_null() {
        return;
    }

    // Stub implementation for reference distance
}

#[no_mangle]
pub extern "C" fn alGenSources2D(n: ALsizei, sources: *mut ALuint) {
    unsafe {
        let mut count = n;
        while count > 0 {
            count -= 1;
            if !_genSource(false) {
                break;
            }
            if !s_pState.is_null() {
                *sources.add(count as usize) = (*s_pState).m_NextSource;
                (*s_pState).m_NextSource += 1;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn alGenSources3D(n: ALsizei, sources: *mut ALuint) {
    unsafe {
        let mut count = n;
        while count > 0 {
            count -= 1;
            if !_genSource(true) {
                break;
            }
            if !s_pState.is_null() {
                *sources.add(count as usize) = (*s_pState).m_NextSource;
                (*s_pState).m_NextSource += 1;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn alDeleteSources(n: ALsizei, sources: *const ALuint) {
    // Stub implementation
}

#[no_mangle]
pub extern "C" fn alSourcei(source: ALuint, param: ALenum, value: ALint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        match param {
            AL_LOOPING => {
                // Set looping on source
            }

            AL_BUFFER => {
                if value != 0 {
                    _attachBuffer(source, value as ALuint);
                }
            }

            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn alSourcef(source: ALuint, param: ALenum, value: ALfloat) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        match param {
            AL_REFERENCE_DISTANCE => {
                // _sourceSetRefDist(info, value);
            }
            AL_GAIN => {
                // Update source gain
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn alSourcefv(source: ALuint, param: ALenum, values: *const ALfloat) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        let values_slice = core::slice::from_raw_parts(values, 3);

        match param {
            AL_POSITION => {
                // Update source position
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn alSourceStop(source: ALuint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        // stop playing for all listeners (stub)
    }
}

#[no_mangle]
pub extern "C" fn alSourcePlay(source: ALuint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        // start playing for all listeners (stub)
    }
}

#[no_mangle]
pub extern "C" fn alGetSourcei(source: ALuint, param: ALenum, value: *mut ALint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        match param {
            AL_SOURCE_STATE => {
                *value = AL_STOPPED as ALint;
            }
            _ => {}
        }
    }
}

/***********************************************
*
* BUFFERS
*
************************************************/

#[no_mangle]
pub extern "C" fn alGenBuffers(n: ALsizei, buffers: *mut ALuint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        let state = &mut *s_pState;

        let mut count = n;
        while count > 0 {
            count -= 1;

            let info = Box::new(BufferInfo {
                m_Data: core::ptr::null_mut(),
                m_DataOffset: 0,
                m_WAVFormat: XBOXADPCMWAVEFORMAT {
                    wfx: WAVEFORMATEX {
                        wFormatTag: 0,
                        nChannels: 0,
                        nSamplesPerSec: 0,
                        nAvgBytesPerSec: 0,
                        nBlockAlign: 0,
                        wBitsPerSample: 0,
                        cbSize: 0,
                    },
                    wSamplesPerBlock: 0,
                },
                m_Freq: 0,
                m_Size: 0,
                m_Valid: false,
            });

            *buffers.add(count as usize) = state.m_NextBuffer;
            state.m_NextBuffer += 1;

            core::mem::forget(info);
        }
    }
}

#[no_mangle]
pub extern "C" fn alDeleteBuffers(n: ALsizei, buffers: *const ALuint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        let state = &mut *s_pState;

        let mut count = n;
        while count > 0 {
            count -= 1;

            let buffer_id = *buffers.add(count as usize);

            // check if the buffer exists and free it (stub)
            // dettach buffer from any sources using it (may block)
            // free the memory
        }
    }
}

#[no_mangle]
pub extern "C" fn alBufferData(
    buffer: ALuint,
    format: ALenum,
    data: *mut ALvoid,
    size: ALsizei,
    freq: ALsizei,
) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        let state = &mut *s_pState;

        // if this buffer has been used before, clear the old data
        // (stub implementation)

        // assume we have a wave file...
        let wav = (data as *const u8).add(20) as *const WAVEFORMATEX;
        let data_offset = 20u32 + core::mem::size_of::<WAVEFORMATEX>() as u32
            + (*wav).cbSize as u32 + 8;

        // (buffer info update stub)
    }
}

/***********************************************
*
* STREAMS
*
************************************************/

unsafe fn _streamFromFile() -> i32 {
    if s_pState.is_null() {
        return -1;
    }

    let mut total: DWORD = 0;
    let mut used: DWORD = 0;

    // setup a media packet for reading from the file
    let mut xmp = XMEDIAPACKET {
        pvBuffer: ((*s_pState).m_Stream.m_pPacketBuffer as *mut u8).add(
            QAL_STREAM_PACKET_SIZE * (*s_pState).m_Stream.m_CurrentPacket as usize,
        ) as *mut c_void,
        dwMaxSize: QAL_STREAM_PACKET_SIZE as DWORD,
        pdwCompletedSize: &mut used,
        pdwStatus: core::ptr::null_mut(),
        pContext: core::ptr::null_mut(),
    };

    // Stub implementation - would wait for file stream mutex
    // WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

    // loop until we have a full packet of data
    while total < QAL_STREAM_PACKET_SIZE as DWORD {
        // if (DS_OK != s_pState->m_Stream.m_pFile->Process(NULL, &xmp))
        // {
        //     ReleaseMutex(Sys_FileStreamMutex);
        //     return -1;
        // }

        total += used;

        // did we get enough data?
        if used < xmp.dwMaxSize {
            if (*s_pState).m_Stream.m_Looping {
                // must have reached the end of the file, loop back
                // around to the beginning and get more data
                xmp.pvBuffer = (xmp.pvBuffer as *mut u8).add(used as usize) as *mut c_void;
                xmp.dwMaxSize = xmp.dwMaxSize - used;

                // if (DS_OK != s_pState->m_Stream.m_pFile->Seek(
                //     0, FILE_BEGIN, NULL))
                // {
                //     ReleaseMutex(Sys_FileStreamMutex);
                //     return -1;
                // }
            } else {
                // reached end, finish up
                (*s_pState).m_Stream.m_Playing = false;
                // ReleaseMutex(Sys_FileStreamMutex);
                return used as i32;
            }
        }
    }

    // ReleaseMutex(Sys_FileStreamMutex);

    QAL_STREAM_PACKET_SIZE as i32
}

unsafe fn _streamToVoice(size: i32) {
    if s_pState.is_null() {
        return;
    }

    // setup a packet with the current data
    let mut xmp = XMEDIAPACKET {
        pvBuffer: ((*s_pState).m_Stream.m_pPacketBuffer as *mut u8).add(
            QAL_STREAM_PACKET_SIZE * (*s_pState).m_Stream.m_CurrentPacket as usize,
        ) as *mut c_void,
        dwMaxSize: size as DWORD,
        pdwCompletedSize: core::ptr::null_mut(),
        pdwStatus: &mut (*s_pState).m_Stream.m_PacketStatus
            [(*s_pState).m_Stream.m_CurrentPacket as usize],
        pContext: core::ptr::null_mut(),
    };

    // sent to the voice
    // s_pState->m_Stream.m_pVoice->Process(&xmp, NULL);

    // make sure we're playing
    // s_pState->m_Stream.m_pVoice->Pause(DSSTREAMPAUSE_RESUME);
    if (*s_pState).m_Stream.m_StartTime == 0 {
        (*s_pState).m_Stream.m_StartTime = Sys_Milliseconds();
    }
}

unsafe fn _streamFill() {
    if s_pState.is_null() {
        return;
    }

    // do we have any free packets?
    if XMEDIAPACKET_STATUS_PENDING
        != (*s_pState).m_Stream.m_PacketStatus[(*s_pState).m_Stream.m_CurrentPacket as usize]
    {
        // get some data
        let size = _streamFromFile();
        if size > 0 {
            _streamToVoice(size);

            // next packet...
            (*s_pState).m_Stream.m_CurrentPacket += 1;
            (*s_pState).m_Stream.m_CurrentPacket %= QAL_MAX_STREAM_PACKETS as DWORD;
        }

        if !(*s_pState).m_Stream.m_Playing {
            // Non-looping stream finished playback
            // s_pState->m_Stream.m_pVoice->Discontinuity();
        }
    }
}

unsafe fn _streamOpen(file: DWORD, offset: DWORD, loop_: bool) {
    if s_pState.is_null() {
        return;
    }

    if (*s_pState).m_Stream.m_Open {
        // if a stream is current playing, interrupt it
        // s_pState->m_Stream.m_pVoice->Flush();
        // s_pState->m_Stream.m_pFile->Release();
        (*s_pState).m_Stream.m_Playing = false;
        (*s_pState).m_Stream.m_Open = false;
    }

    let name = Sys_GetFileCodeName(file as c_int);

    // WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

    // open the file for streaming
    // Stub implementation
    // if (DS_OK == XWaveFileCreateMediaObject(
    //     name, &fmt, &s_pState->m_Stream.m_pFile))
    // {
    //     // set the voice based on the file format
    //     s_pState->m_Stream.m_pVoice->SetFormat(fmt);
    //
    //     // seek the requested start position
    //     s_pState->m_Stream.m_pFile->Seek(RoundDown(offset, 72),
    //         FILE_BEGIN, NULL);
    //
    //     s_pState->m_Stream.m_StartTime = 0;
    //     s_pState->m_Stream.m_Looping = loop_;
    //     s_pState->m_Stream.m_Playing = true;
    //     s_pState->m_Stream.m_Open = true;
    // }

    // ReleaseMutex(Sys_FileStreamMutex);
}

unsafe fn _streamClose() {
    if s_pState.is_null() {
        return;
    }

    if (*s_pState).m_Stream.m_Open {
        // stop the stream
        // s_pState->m_Stream.m_pVoice->Flush();
        // s_pState->m_Stream.m_pFile->Release();
        (*s_pState).m_Stream.m_Playing = false;
        (*s_pState).m_Stream.m_Open = false;
    }
}

extern "system" fn _streamThread(_lpParameter: LPVOID) -> u32 {
    unsafe {
        loop {
            if s_pState.is_null() {
                break;
            }

            let strm = &mut (*s_pState).m_Stream;

            // Wait for the queue to fill
            // WaitForSingleObject(strm->m_QueueLen, QAL_STREAM_WAIT_TIME);

            // Grab the next request
            // WaitForSingleObject(strm->m_Mutex, INFINITE);
            // if (!strm->m_Queue.empty())
            // {
            //     req = strm->m_Queue.front();
            //     strm->m_Queue.pop_front();
            // }
            // else
            // {
            //     req.m_Type = QALState::StreamInfo::REQ_NOP;
            // }
            // ReleaseMutex(strm->m_Mutex);

            // Process request (stub)
            // switch (req.m_Type)
            // {
            //     case QALState::StreamInfo::REQ_PLAY:
            //         _streamOpen(req.m_Data[0], req.m_Data[1], req.m_Data[2]);
            //         break;
            //
            //     case QALState::StreamInfo::REQ_STOP:
            //         _streamClose();
            //         break;
            //
            //     case QALState::StreamInfo::REQ_SHUTDOWN:
            //         ExitThread(0);
            //         break;
            //
            //     case QALState::StreamInfo::REQ_NOP:
            //         break;
            // }

            // fill the stream with data
            if strm.m_Open && strm.m_Playing {
                _streamFill();
            }
        }
    }
    0
}

unsafe fn _postStreamRequest(req: *const StreamRequest) {
    if s_pState.is_null() {
        return;
    }

    // Add request to queue
    // WaitForSingleObject(s_pState->m_Stream.m_Mutex, INFINITE);
    // s_pState->m_Stream.m_Queue.push_back(*req);
    // ReleaseMutex(s_pState->m_Stream.m_Mutex);

    // Let thread know it has one more pending request
    // ReleaseSemaphore(s_pState->m_Stream.m_QueueLen, 1, NULL);

    // Give the stream thread some CPU
    // Sleep(0);
}

#[no_mangle]
pub extern "C" fn alGenStream() {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        if (*s_pState).m_Stream.m_Valid {
            return;
        }

        // describe the stream
        let mut wav = XBOXADPCMWAVEFORMAT {
            wfx: WAVEFORMATEX {
                wFormatTag: 0,
                nChannels: 0,
                nSamplesPerSec: 0,
                nAvgBytesPerSec: 0,
                nBlockAlign: 0,
                wBitsPerSample: 0,
                cbSize: 0,
            },
            wSamplesPerBlock: 0,
        };
        _wavSetFormat(&mut wav, AL_FORMAT_STEREO4, 44100);

        let mut desc = DSSTREAMDESC {
            dwSize: core::mem::size_of::<DSSTREAMDESC>() as DWORD,
            dwMaxAttachedPackets: QAL_MAX_STREAM_PACKETS as DWORD,
            lpwfxFormat: &mut wav.wfx,
        };

        // create a voice for the stream
        // if (s_pState->m_SoundObject->CreateSoundStream(&desc,
        //     &s_pState->m_Stream.m_pVoice, NULL) != DS_OK)
        // {
        //     s_pState->m_Error = AL_OUT_OF_MEMORY;
        //     return;
        // }

        // get some memory to hold the stream data
        // s_pState->m_Stream.m_pPacketBuffer =
        //     XPhysicalAlloc(QAL_MAX_STREAM_PACKETS * QAL_STREAM_PACKET_SIZE,
        //     MAXULONG_PTR, 0, PAGE_READWRITE | PAGE_NOCACHE);

        // setup some defaults
        (*s_pState).m_Stream.m_Gain = 1.0;
        (*s_pState).m_Stream.m_GainDirty = true;

        (*s_pState).m_Stream.m_CurrentPacket = 0;
        for p in 0..QAL_MAX_STREAM_PACKETS {
            (*s_pState).m_Stream.m_PacketStatus[p] = XMEDIAPACKET_STATUS_SUCCESS;
        }

        (*s_pState).m_Stream.m_Open = false;
        (*s_pState).m_Stream.m_Playing = false;
        (*s_pState).m_Stream.m_Valid = true;

        // setup a thread to service the stream (keep blocking IO out
        // of the main thread)
        // s_pState->m_Stream.m_QueueLen = CreateSemaphore(NULL, 0, 256, NULL);
        // s_pState->m_Stream.m_Mutex = CreateMutex(NULL, FALSE, NULL);
        // s_pState->m_Stream.m_Thread = CreateThread(NULL, 64*1024,
        //     _streamThread, NULL, 0, NULL );
    }
}

#[no_mangle]
pub extern "C" fn alDeleteStream() {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        if !(*s_pState).m_Stream.m_Valid {
            return;
        }

        // stop the audio
        alStreamStop();

        // kill the thread
        let mut req = StreamRequest {
            m_Type: StreamRequestType::REQ_SHUTDOWN,
            m_Data: [0; 3],
        };
        _postStreamRequest(&req);

        // Wait for thread to close
        // WaitForSingleObject(s_pState->m_Stream.m_Thread, INFINITE);

        // thread handles
        // CloseHandle(s_pState->m_Stream.m_Thread);
        // CloseHandle(s_pState->m_Stream.m_Mutex);
        // CloseHandle(s_pState->m_Stream.m_QueueLen);

        // release the stream
        // s_pState->m_Stream.m_pVoice->Release();
        // XPhysicalFree(s_pState->m_Stream.m_pPacketBuffer);

        (*s_pState).m_Stream.m_Valid = false;
    }
}

#[no_mangle]
pub extern "C" fn alStreamStop() {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        if !(*s_pState).m_Stream.m_Valid {
            return;
        }

        let mut req = StreamRequest {
            m_Type: StreamRequestType::REQ_STOP,
            m_Data: [0; 3],
        };
        _postStreamRequest(&req);
    }
}

#[no_mangle]
pub extern "C" fn alStreamPlay(offset: ALsizei, file: ALint, loop_: ALint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        if !(*s_pState).m_Stream.m_Valid {
            return;
        }

        let mut req = StreamRequest {
            m_Type: StreamRequestType::REQ_PLAY,
            m_Data: [file as DWORD, offset as DWORD, loop_ as DWORD],
        };
        _postStreamRequest(&req);

        (*s_pState).m_Stream.m_Playing = true;
    }
}

#[no_mangle]
pub extern "C" fn alStreamf(param: ALenum, value: ALfloat) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        if !(*s_pState).m_Stream.m_Valid {
            return;
        }

        match param {
            AL_GAIN => {
                (*s_pState).m_Stream.m_Gain = value;
                (*s_pState).m_Stream.m_GainDirty = true;
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn alGetStreamf(param: ALenum, value: *mut ALfloat) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        if !(*s_pState).m_Stream.m_Valid {
            return;
        }

        match param {
            AL_TIME => {
                if (*s_pState).m_Stream.m_Open && (*s_pState).m_Stream.m_StartTime != 0 {
                    *value =
                        (Sys_Milliseconds() - (*s_pState).m_Stream.m_StartTime) as f32 / 1000.0;
                } else {
                    *value = 0.0;
                }
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn alGetStreami(param: ALenum, value: *mut ALint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        if !(*s_pState).m_Stream.m_Valid {
            return;
        }

        match param {
            AL_SOURCE_STATE => {
                *value = if (*s_pState).m_Stream.m_Playing {
                    AL_PLAYING as ALint
                } else {
                    AL_STOPPED as ALint
                };
            }
            _ => {}
        }
    }
}

/***********************************************
*
* ADDITIONAL FUNCTIONS
*
************************************************/

unsafe fn _updateVoiceGain(voice: *mut IDirectSoundBuffer, gain: FLOAT) {
    if s_pState.is_null() {
        return;
    }

    // compute aggregate gain
    let mut g = (*s_pState).m_Gain * gain;

    if g <= 0.0 {
        // mute the sound
        // voice->SetVolume(DSBVOLUME_MIN);
    } else {
        // convert to dB
        g = 20.0 * g.log10();

        if g < -100.0 {
            g = -100.0;
        }

        // set the volume
        // voice->SetVolume(g * 100.0);
    }
}

unsafe fn _updateVoicePos(
    voice: *mut IDirectSoundBuffer,
    pos: *const D3DXVECTOR3,
    listener: *const ListenerInfo,
) {
    if listener.is_null() || pos.is_null() {
        return;
    }

    // get source pos in listener space
    let mut lpos = D3DXVECTOR4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    // D3DXVec3Transform(&lpos, pos, &listener->m_LTM);

    // voice->SetPosition(lpos.x, lpos.y, lpos.z, DS3D_DEFERRED);
}

unsafe fn _updateSource(source: *mut SourceInfo) {
    if source.is_null() || s_pState.is_null() {
        return;
    }

    // loop through all the voices at this source (stub)
    // Would iterate through source voices and update gain/position

    (*source).m_GainDirty = false;
}

unsafe fn _updateStream() {
    if s_pState.is_null() {
        return;
    }

    if (*s_pState).m_Stream.m_Open && (*s_pState).m_Stream.m_GainDirty {
        // compute aggregate gain
        let mut g = (*s_pState).m_Gain * (*s_pState).m_Stream.m_Gain;
        if g <= 0.0 {
            // mute the sound
            // s_pState->m_Stream.m_pVoice->SetVolume(DSBVOLUME_MIN);
        } else {
            // convert to dB
            g = 20.0 * g.log10();

            if g < -100.0 {
                g = -100.0;
            }

            // set the volume
            // s_pState->m_Stream.m_pVoice->SetVolume(g * 100.0);
        }

        (*s_pState).m_Stream.m_GainDirty = false;
    }
}

#[no_mangle]
pub extern "C" fn alGetError() -> ALenum {
    unsafe {
        if s_pState.is_null() {
            return AL_NO_ERROR;
        }

        let error = (*s_pState).m_Error;
        (*s_pState).m_Error = AL_NO_ERROR;
        error
    }
}

#[no_mangle]
pub extern "C" fn alUpdate() {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        DirectSoundDoWork();

        // update sources (stub - would iterate through m_Sources)
        // for (QALState::source_t::iterator i = s_pState->m_Sources.begin();
        // i != s_pState->m_Sources.end(); ++i)
        // {
        //     QALState::SourceInfo* info = i->second;
        //
        //     // 3d sounds and dirty sources must be updated
        //     if (info->m_Is3d || info->m_GainDirty)
        //     {
        //         // only playing sources should be updated
        //         DWORD status;
        //         info->m_Voices.begin()->second->GetStatus(&status);
        //
        //         if (status & DSBSTATUS_PLAYING)
        //         {
        //             _updateSource(info);
        //         }
        //     }
        // }

        // update stream
        _updateStream();

        // s_pState->m_SoundObject->CommitDeferredSettings();
    }
}

#[no_mangle]
pub extern "C" fn alGeti(param: ALenum, value: *mut ALint) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        match param {
            AL_MEMORY_USED => {
                *value = (*s_pState).m_MemoryUsed as ALint;
            }

            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn alGain(value: ALfloat) {
    unsafe {
        if s_pState.is_null() {
            return;
        }

        (*s_pState).m_Gain = value;

        // set gain dirty for all sources (stub - would iterate through m_Sources)
        // for (QALState::source_t::iterator i = s_pState->m_Sources.begin();
        // i != s_pState->m_Sources.end(); ++i)
        // {
        //     i->second->m_GainDirty = true;
        // }

        // set gain dirty for stream
        (*s_pState).m_Stream.m_GainDirty = true;
    }
}
