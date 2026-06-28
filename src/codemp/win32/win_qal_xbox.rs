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

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint, c_ulong};
use std::collections::{BTreeMap, VecDeque};
use std::ptr;

// Stub imports for DirectSound and Xbox types (would come from winapi crate in real port)
type IDirectSound8 = c_void;
type IDirectSoundBuffer = c_void;
type IDirectSoundStream = c_void;
type XFileMediaObject = c_void;
type LPDSEFFECTIMAGEDESC = *mut c_void;
type HANDLE = *mut c_void;

type DWORD = c_ulong;
type FLOAT = f32;
type LPVOID = *mut c_void;
type DWORD_PTR = usize;
type MAXULONG_PTR = usize;

// OpenAL types (stubs)
type ALuint = c_uint;
type ALenum = c_uint;
type ALsizei = c_int;
type ALfloat = f32;
type ALint = c_int;
type ALvoid = c_void;
type ALCdevice = c_void;
type ALCcontext = c_void;
type ALCubyte = u8;
type ALCint = c_int;
type ALCvoid = c_void;
type ALCenum = c_uint;
type ALCboolean = c_int;
type ALboolean = c_int;

// OpenAL constants
const AL_NO_ERROR: ALenum = 0;
const AL_OUT_OF_MEMORY: ALenum = 0xA005;
const AL_FORMAT_MONO4: ALenum = 0x1300;
const AL_FORMAT_STEREO4: ALenum = 0x1301;
const AL_FORMAT_MONO8: ALenum = 0x1100;
const AL_FORMAT_STEREO8: ALenum = 0x1101;
const AL_FORMAT_MONO16: ALenum = 0x1101;
const AL_FORMAT_STEREO16: ALenum = 0x1102;
const AL_POSITION: ALenum = 0x1004;
const AL_ORIENTATION: ALenum = 0x100F;
const AL_LOOPING: ALenum = 0x1007;
const AL_BUFFER: ALenum = 0x1009;
const AL_REFERENCE_DISTANCE: ALenum = 0x1020;
const AL_GAIN: ALenum = 0x100A;
const AL_SOURCE_STATE: ALenum = 0x1010;
const AL_PLAYING: ALenum = 0x1012;
const AL_STOPPED: ALenum = 0x1014;
const AL_MEMORY_USED: ALenum = 0x10001;
const AL_TIME: ALenum = 0x10001;
const ALC_NO_ERROR: ALCenum = 0;

// DirectSound constants
const DS_OK: DWORD = 0;
const DSBCAPS_CTRL3D: DWORD = 0x00000001;
const DSBCAPS_MUTE3DATMAXDISTANCE: DWORD = 0x00000020;
const DSBSTATUS_PLAYING: DWORD = 0x00000001;
const DS3D_DEFERRED: DWORD = 1;
const DSBVOLUME_MIN: i32 = -10000;
const DSBPLAY_LOOPING: DWORD = 0x00000001;
const DSSTREAMPAUSE_RESUME: DWORD = 0;
const PAGE_READWRITE: DWORD = 4;
const PAGE_NOCACHE: DWORD = 0x200;
const INFINITE: DWORD = 0xFFFFFFFF;
const FALSE: c_int = 0;
const FILE_BEGIN: DWORD = 0;
const WAVE_FORMAT_XBOX_ADPCM: u16 = 0x0069;

// DirectSound structures (stubs)
#[repr(C)]
pub struct WAVEFORMATEX {
    wFormatTag: u16,
    nChannels: u16,
    nSamplesPerSec: DWORD,
    nAvgBytesPerSec: DWORD,
    nBlockAlign: u16,
    wBitsPerSample: u16,
    cbSize: u16,
}

#[repr(C)]
pub struct XBOXADPCMWAVEFORMAT {
    wfx: WAVEFORMATEX,
    wSamplesPerBlock: u16,
}

#[repr(C)]
pub struct D3DXVECTOR3 {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(C)]
pub struct D3DXVECTOR4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[repr(C)]
pub struct D3DXMATRIX {
    m: [[f32; 4]; 4],
}

#[repr(C)]
pub struct DSI3DL2LISTENER {
    dwSize: DWORD,
    lRoom: i32,
    lRoomHF: i32,
    flRoomRolloffFactor: f32,
    flDecayTime: f32,
    flDecayHFRatio: f32,
    lReflections: i32,
    flReflectionsDelay: f32,
    lReverb: i32,
    flReverbDelay: f32,
    flDiffusion: f32,
    flDensity: f32,
    flHFReference: f32,
}

#[repr(C)]
pub struct DSBUFFERDESC {
    dwSize: DWORD,
    dwFlags: DWORD,
    dwBufferBytes: DWORD,
    dwReserved: DWORD,
    lpwfxFormat: *mut WAVEFORMATEX,
    lpMixBins: *mut c_void,
    dwInputMixBin: DWORD,
}

#[repr(C)]
pub struct DSSTREAMDESC {
    dwSize: DWORD,
    dwFlags: DWORD,
    dwMaxAttachedPackets: DWORD,
    lpwfxFormat: *mut WAVEFORMATEX,
    lpMixBins: *mut c_void,
}

#[repr(C)]
pub struct XMEDIAPACKET {
    pvBuffer: *mut u8,
    dwMaxSize: DWORD,
    pdwCompletedSize: *mut DWORD,
    pdwStatus: *mut DWORD,
    pContext: *mut c_void,
}

#[repr(C)]
pub struct DSEFFECTIMAGELOC {
    dwI3DL2ReverbIndex: DWORD,
    dwCrosstalkIndex: DWORD,
}

const XMEDIAPACKET_STATUS_SUCCESS: DWORD = 0;
const XMEDIAPACKET_STATUS_PENDING: DWORD = 1;

#[derive(Copy, Clone)]
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

const DSMIXBINVOLUMEPAIRS_DEFAULT_5CHANNEL_3D: [DSMIXBINVOLUMEPAIR; 6] = [
    DSMIXBINVOLUMEPAIR { dwMixBin: 0, lVolume: 0 },
    DSMIXBINVOLUMEPAIR { dwMixBin: 1, lVolume: 0 },
    DSMIXBINVOLUMEPAIR { dwMixBin: 2, lVolume: 0 },
    DSMIXBINVOLUMEPAIR { dwMixBin: 3, lVolume: 0 },
    DSMIXBINVOLUMEPAIR { dwMixBin: 4, lVolume: 0 },
    DSMIXBINVOLUMEPAIR { dwMixBin: 5, lVolume: 0 },
];

const GraphI3DL2_I3DL2Reverb: DWORD = 0;
const GraphXTalk_XTalk: DWORD = 1;
const DSI3DL2_ENVIRONMENT_PRESET_NOREVERB: DWORD = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ZeroMemory;

const QAL_STREAM_WAIT_TIME: DWORD = 500;
const QAL_MAX_STREAM_PACKETS: usize = 2;

// About 1 second of audio at 44100, stereo, ADPCM
const QAL_STREAM_PACKET_SIZE: usize = 44136;

// Un-comment to enable 5-channel 3-d sound mixing
// #define _FIVE_CHANNEL

extern "C" {
    static Sys_FileStreamMutex: HANDLE;
    fn Sys_GetFileCodeName(code: c_int) -> *const c_char;
    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn Z_Free(ptr: *mut c_void);
    fn Sys_Milliseconds() -> c_int;
    fn DirectSoundCreate(
        pcGuidDevice: *const c_void,
        ppDS: *mut *mut IDirectSound8,
        pUnkOuter: *mut c_void,
    ) -> DWORD;
    fn DirectSoundUseFullHRTF();
    fn XWaveFileCreateMediaObject(
        pszFileName: *const c_char,
        ppwfxFormat: *mut *const WAVEFORMATEX,
        ppMediaObject: *mut *mut XFileMediaObject,
    ) -> DWORD;
    fn D3DXMatrixIdentity(pOut: *mut D3DXMATRIX);
    fn D3DXMatrixTranslation(
        pOut: *mut D3DXMATRIX,
        x: f32,
        y: f32,
        z: f32,
    );
    fn D3DXMatrixMultiply(
        pOut: *mut D3DXMATRIX,
        pM1: *const D3DXMATRIX,
        pM2: *const D3DXMATRIX,
    );
    fn D3DXVec3Cross(
        pOut: *mut D3DXVECTOR3,
        pLeft: *const D3DXVECTOR3,
        pRight: *const D3DXVECTOR3,
    );
    fn D3DXMatrixInverse(
        pOut: *mut D3DXMATRIX,
        pDeterminant: *mut FLOAT,
        pM: *const D3DXMATRIX,
    );
    fn XPhysicalAlloc(
        Size: DWORD,
        HighAddress: DWORD_PTR,
        Alignment: DWORD,
        Protect: DWORD,
    ) -> *mut c_void;
    fn XPhysicalFree(Address: *mut c_void);
    fn CreateSemaphore(
        lpSemaphoreAttributes: *mut c_void,
        lInitialCount: c_int,
        lMaximumCount: c_int,
        lpName: *const c_char,
    ) -> HANDLE;
    fn CreateMutex(
        lpMutexAttributes: *mut c_void,
        bInitialOwner: c_int,
        lpName: *const c_char,
    ) -> HANDLE;
    fn CreateThread(
        lpThreadAttributes: *mut c_void,
        dwStackSize: DWORD,
        lpStartAddress: extern "C" fn(*mut c_void) -> DWORD,
        lpParameter: *mut c_void,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    fn ReleaseMutex(hMutex: HANDLE) -> c_int;
    fn ReleaseSemaphore(hSemaphore: HANDLE, lReleaseCount: c_int, lpPreviousCount: *mut c_int) -> c_int;
    fn CloseHandle(hObject: HANDLE) -> c_int;
    fn Sleep(dwMilliseconds: DWORD);
    fn ExitThread(dwExitCode: DWORD) -> !;
}

// Methods that would be on COM objects (stubs for now)
impl_com_methods!();

/***********************************************
*
* OpenAL STATE - Main container for all AL objects
*
************************************************/

struct ListenerInfo {
    m_Position: D3DXVECTOR3,
    m_LTM: D3DXMATRIX,
}

struct SourceInfo {
    m_Voices: BTreeMap<ALuint, *mut IDirectSoundBuffer>,

    m_Buffer: ALuint,

    m_Gain: FLOAT,
    m_GainDirty: bool,

    m_Loop: bool,

    m_Is3d: bool,
    m_Position: D3DXVECTOR3,
}

struct BufferInfo {
    m_Data: *mut c_void,
    m_DataOffset: DWORD,
    m_WAVFormat: XBOXADPCMWAVEFORMAT,

    m_Freq: DWORD,
    m_Size: DWORD,

    m_Valid: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
enum RequestType {
    REQ_NOP = 0,
    REQ_PLAY = 1,
    REQ_STOP = 2,
    REQ_SHUTDOWN = 3,
}

struct Request {
    m_Type: RequestType,
    m_Data: [DWORD; 3],
}

struct StreamInfo {
    m_pVoice: *mut IDirectSoundStream,
    m_pFile: *mut XFileMediaObject,

    m_StartTime: c_uint,

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

    m_Queue: *mut VecDeque<Request>,
}

struct QALState {
    m_SoundObject: *mut IDirectSound8,
    m_ImageDesc: LPDSEFFECTIMAGEDESC,

    m_MemoryUsed: ALuint,
    m_Error: ALenum,
    m_Gain: FLOAT,

    m_Listeners: BTreeMap<ALuint, *mut ListenerInfo>,
    m_NextListener: ALuint,

    m_Sources: BTreeMap<ALuint, *mut SourceInfo>,
    m_NextSource: ALuint,

    m_Buffers: BTreeMap<ALuint, *mut BufferInfo>,
    m_NextBuffer: ALuint,

    m_Stream: StreamInfo,
}

static mut s_pState: *mut QALState = ptr::null_mut();

/***********************************************
*
* HACK - Voice initialization needs this
*
************************************************/

fn getEffectsImageDesc() -> LPDSEFFECTIMAGEDESC {
    unsafe {
        if s_pState.is_null() {
            ptr::null_mut()
        } else {
            (*s_pState).m_ImageDesc
        }
    }
}


/***********************************************
*
* DEVICES AND CONTEXTS
*
************************************************/

pub extern "C" fn alcOpenDevice(deviceName: *mut ALCubyte) -> *mut ALCdevice {
    unsafe {
        if !s_pState.is_null() {
            return ptr::null_mut();
        }

        let state = Box::new(QALState {
            m_SoundObject: ptr::null_mut(),
            m_ImageDesc: ptr::null_mut(),
            m_MemoryUsed: 0,
            m_Error: AL_NO_ERROR,
            m_Gain: 1.0f32,
            m_Listeners: BTreeMap::new(),
            m_NextListener: 1,
            m_Sources: BTreeMap::new(),
            m_NextSource: 1,
            m_Buffers: BTreeMap::new(),
            m_NextBuffer: 1,
            m_Stream: StreamInfo {
                m_pVoice: ptr::null_mut(),
                m_pFile: ptr::null_mut(),
                m_StartTime: 0,
                m_Open: false,
                m_Playing: false,
                m_Valid: false,
                m_Gain: 1.0f32,
                m_GainDirty: false,
                m_Looping: false,
                m_pPacketBuffer: ptr::null_mut(),
                m_PacketStatus: [0; QAL_MAX_STREAM_PACKETS],
                m_CurrentPacket: 0,
                m_Thread: ptr::null_mut(),
                m_Mutex: ptr::null_mut(),
                m_QueueLen: ptr::null_mut(),
                m_Queue: ptr::null_mut(),
            },
        });

        s_pState = Box::into_raw(state);

        let queue = Box::new(VecDeque::<Request>::new());
        (*s_pState).m_Stream.m_Queue = Box::into_raw(queue);

        (*s_pState).m_Gain = 1.0f32;
        (*s_pState).m_Error = AL_NO_ERROR;
        (*s_pState).m_MemoryUsed = 0;
        (*s_pState).m_NextBuffer = 1;
        (*s_pState).m_NextListener = 1;
        (*s_pState).m_NextSource = 1;
        (*s_pState).m_Stream.m_Valid = false;

        // init the sound hardware
        if DirectSoundCreate(ptr::null(), &mut (*s_pState).m_SoundObject, ptr::null_mut()) != DS_OK {
            Box::from_raw(s_pState);
            s_pState = ptr::null_mut();
            return ptr::null_mut();
        }

        DirectSoundUseFullHRTF();

        // download effects image to hardware
        let mut image: *mut c_void = ptr::null_mut();
        let len = FS_ReadFile(b"sound/dsstdfx.bin\0".as_ptr() as *const c_char, &mut image);
        if len <= 0 {
            Box::from_raw(s_pState);
            s_pState = ptr::null_mut();
            return ptr::null_mut();
        }

        let mut effect = DSEFFECTIMAGELOC {
            dwI3DL2ReverbIndex: GraphI3DL2_I3DL2Reverb,
            dwCrosstalkIndex: GraphXTalk_XTalk,
        };

        // TODO: Call DownloadEffectsImage on SoundObject
        // s_pState->m_SoundObject->DownloadEffectsImage(image, len, &effect, &s_pState->m_ImageDesc);

        Z_Free(image);

        // setup default reverb
        let mut reverb = DSI3DL2LISTENER {
            dwSize: std::mem::size_of::<DSI3DL2LISTENER>() as DWORD,
            lRoom: 0,
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
        // TODO: Call SetI3DL2Listener
        // s_pState->m_SoundObject->SetI3DL2Listener(&reverb, DS3D_DEFERRED);

        (*s_pState).m_SoundObject as *mut ALCdevice
    }
}

pub extern "C" fn alcCloseDevice(device: *mut ALCdevice) {
    unsafe {
        // shutdown the sound hardware
        if !s_pState.is_null() && !(*s_pState).m_SoundObject.is_null() {
            // TODO: Call Release on SoundObject
            // (*s_pState).m_SoundObject->Release();
        }

        if !s_pState.is_null() {
            Box::from_raw(s_pState);
            s_pState = ptr::null_mut();
        }
    }
}

pub extern "C" fn alcCreateContext(device: *mut ALCdevice, attrList: *mut ALCint) -> *mut ALCcontext {
    1 as *mut ALCcontext
}

pub extern "C" fn alcMakeContextCurrent(context: *mut ALCcontext) -> ALCboolean {
    1
}

pub extern "C" fn alcGetCurrentContext() -> *mut ALCcontext {
    1 as *mut ALCcontext
}

pub extern "C" fn alcGetContextsDevice(context: *mut ALCcontext) -> *mut ALCdevice {
    unsafe {
        if s_pState.is_null() {
            return ptr::null_mut();
        }
        (*s_pState).m_SoundObject as *mut ALCdevice
    }
}

pub extern "C" fn alcDestroyContext(context: *mut ALCcontext) {
}

pub extern "C" fn alcGetError(device: *mut ALCdevice) -> ALCenum {
    ALC_NO_ERROR
}




/***********************************************
*
* LISTENERS
*
************************************************/

pub extern "C" fn alGenListeners(mut n: ALsizei, listeners: *mut ALuint) {
    unsafe {
        let mut i = 0;
        while n > 0 {
            let info = Box::new(ListenerInfo {
                m_Position: D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 },
                m_LTM: D3DXMATRIX { m: [[0.0; 4]; 4] },
            });

            D3DXMatrixIdentity(&mut (*Box::as_mut(&mut Box::from_raw(&mut *(Box::into_raw(info) as *mut ListenerInfo)))).m_LTM);

            let info_ptr = Box::into_raw(info);
            let listener_id = (*s_pState).m_NextListener;
            (*s_pState).m_Listeners.insert(listener_id, info_ptr);
            *listeners.add(i) = listener_id;
            (*s_pState).m_NextListener += 1;

            n -= 1;
            i += 1;
        }
    }
}

pub extern "C" fn alDeleteListeners(mut n: ALsizei, listeners: *mut ALuint) {
    unsafe {
        while n > 0 {
            let listener_id = *listeners.add((n - 1) as usize);
            if let Some((_k, info)) = (*s_pState).m_Listeners.remove_entry(&listener_id) {
                Box::from_raw(info);
            }
            n -= 1;
        }
    }
}

pub extern "C" fn alListenerfv(listener: ALuint, param: ALenum, values: *mut ALfloat) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Listeners.get(&listener) {
            let info = *info_ptr;

            match param {
                AL_POSITION => {
                    (*info).m_Position.x = *values;
                    (*info).m_Position.y = *values.add(1);
                    (*info).m_Position.z = *values.add(2);

                    // translation
                    let mut trans = D3DXMATRIX { m: [[0.0; 4]; 4] };
                    D3DXMatrixTranslation(&mut trans, -*values, -*values.add(1), -*values.add(2));
                    let mut result = D3DXMATRIX { m: [[0.0; 4]; 4] };
                    D3DXMatrixMultiply(&mut result, &trans, &(*info).m_LTM);
                    (*info).m_LTM = result;
                }

                AL_ORIENTATION => {
                    D3DXMatrixIdentity(&mut (*info).m_LTM);

                    // at vector
                    (*info).m_LTM.m[2][0] = *values;
                    (*info).m_LTM.m[2][1] = *values.add(1);
                    (*info).m_LTM.m[2][2] = *values.add(2);

                    // up vector
                    (*info).m_LTM.m[1][0] = *values.add(3);
                    (*info).m_LTM.m[1][1] = *values.add(4);
                    (*info).m_LTM.m[1][2] = *values.add(5);

                    // Hack. We switched the sign on values[2] up above, need to do that here
                    let mut right = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
                    D3DXVec3Cross(&mut right, values as *const D3DXVECTOR3, values.add(3) as *const D3DXVECTOR3);

                    // right vector
                    (*info).m_LTM.m[0][0] = right.x;
                    (*info).m_LTM.m[0][1] = right.y;
                    (*info).m_LTM.m[0][2] = right.z;

                    // convert to local space transform
                    let mut det = 0.0f32;
                    let mut inv = D3DXMATRIX { m: [[0.0; 4]; 4] };
                    D3DXMatrixInverse(&mut inv, &mut det, &(*info).m_LTM);
                    (*info).m_LTM = inv;

                    // translation
                    let mut trans = D3DXMATRIX { m: [[0.0; 4]; 4] };
                    D3DXMatrixTranslation(&mut trans,
                        -(*info).m_Position.x, -(*info).m_Position.y, -(*info).m_Position.z);
                    let mut result = D3DXMATRIX { m: [[0.0; 4]; 4] };
                    D3DXMatrixMultiply(&mut result, &trans, &(*info).m_LTM);
                    (*info).m_LTM = result;
                }
                _ => {}
            }
        }
    }
}




/***********************************************
*
* SOURCES
*
************************************************/

fn _wavSetFormat(wav: *mut XBOXADPCMWAVEFORMAT, format: ALenum, freq: ALsizei) {
    unsafe {
        match format {
            AL_FORMAT_MONO4 => {
                (*wav).wfx.wFormatTag = WAVE_FORMAT_XBOX_ADPCM;
                (*wav).wfx.nChannels = 1;
                (*wav).wfx.nSamplesPerSec = freq as DWORD;
                (*wav).wfx.nBlockAlign = (36 * (*wav).wfx.nChannels) as u16;
                (*wav).wfx.nAvgBytesPerSec = ((*wav).wfx.nSamplesPerSec * (*wav).wfx.nBlockAlign as DWORD / 64);
                (*wav).wfx.wBitsPerSample = 4;
                (*wav).wfx.cbSize = (std::mem::size_of::<XBOXADPCMWAVEFORMAT>() - std::mem::size_of::<WAVEFORMATEX>()) as u16;
                (*wav).wSamplesPerBlock = 64;
            }

            AL_FORMAT_STEREO4 => {
                (*wav).wfx.wFormatTag = WAVE_FORMAT_XBOX_ADPCM;
                (*wav).wfx.nChannels = 2;
                (*wav).wfx.nSamplesPerSec = freq as DWORD;
                (*wav).wfx.nBlockAlign = (36 * (*wav).wfx.nChannels) as u16;
                (*wav).wfx.nAvgBytesPerSec = ((*wav).wfx.nSamplesPerSec * (*wav).wfx.nBlockAlign as DWORD / 64);
                (*wav).wfx.wBitsPerSample = 4;
                (*wav).wfx.cbSize = (std::mem::size_of::<XBOXADPCMWAVEFORMAT>() - std::mem::size_of::<WAVEFORMATEX>()) as u16;
                (*wav).wSamplesPerBlock = 64;
            }

            _ => {
                // assert(0);
            }
        }
    }
}

fn _genSource(is3d: bool) -> bool {
    unsafe {
        // alloc a new source
        let sinfo = Box::new(SourceInfo {
            m_Voices: BTreeMap::new(),
            m_Buffer: 0,
            m_Gain: 1.0f32,
            m_GainDirty: true,
            m_Loop: false,
            m_Is3d: is3d,
            m_Position: D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 },
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
            dwSize: std::mem::size_of::<DSBUFFERDESC>() as DWORD,
            dwFlags: if is3d { DSBCAPS_CTRL3D | DSBCAPS_MUTE3DATMAXDISTANCE } else { 0 },
            dwBufferBytes: 0,
            dwReserved: 0,
            lpwfxFormat: &mut wav as *mut _ as *mut WAVEFORMATEX,
            lpMixBins: ptr::null_mut(),
            dwInputMixBin: 0,
        };

        // create voice for all listeners
        let mut sinfo = sinfo;
        for (listener_id, _listener) in (*s_pState).m_Listeners.iter() {
            // create the voice
            // TODO: Call CreateSoundBuffer
            // if (s_pState->m_SoundObject->CreateSoundBuffer(&desc, &voice, NULL) != DS_OK)
            // {
            //     s_pState->m_Error = AL_OUT_OF_MEMORY;
            //     return false;
            // }

            // For now, just insert null pointer
            (*sinfo).m_Voices.insert(*listener_id, ptr::null_mut());

            // only create a single voice for 2d sounds
            if !is3d { break; }
        }

        // setup some defaults
        (*sinfo).m_Buffer = 0;
        (*sinfo).m_Gain = 1.0f32;
        (*sinfo).m_GainDirty = true;
        (*sinfo).m_Loop = false;
        (*sinfo).m_Is3d = is3d;
        (*sinfo).m_Position = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };

        let source_id = (*s_pState).m_NextSource;
        let sinfo_ptr = Box::into_raw(Box::new(*sinfo));
        (*s_pState).m_Sources.insert(source_id, sinfo_ptr);

        true
    }
}

fn _attachBuffer(source: ALuint, buffer: ALuint) {
    unsafe {
        if let Some(sinfo_ptr) = (*s_pState).m_Sources.get(&source) {
            if let Some(binfo_ptr) = (*s_pState).m_Buffers.get(&buffer) {
                let sinfo = *sinfo_ptr;
                let binfo = *binfo_ptr;

                // setup voices for all listeners
                for (_listener_id, voice) in (*sinfo).m_Voices.iter() {
                    // TODO: SetFormat
                    // (*voice)->SetFormat((WAVEFORMATEX*)&(*binfo).m_WAVFormat);

                    // #ifdef _FIVE_CHANNEL
                    //     DSMIXBINVOLUMEPAIR dsmbvp[6] = {
                    //         DSMIXBINVOLUMEPAIRS_DEFAULT_5CHANNEL_3D,
                    //     };
                    //     DSMIXBINS dsmb;
                    //     dsmb.dwMixBinCount = 6;
                    //     dsmb.lpMixBinVolumePairs = dsmbvp;
                    //
                    //     (*voice)->SetMixBins(&dsmb);
                    // #endif

                    // TODO: SetBufferData
                    // (*voice)->SetBufferData((char*)(*binfo).m_Data + (*binfo).m_DataOffset, (*binfo).m_Size);
                }

                (*sinfo).m_Buffer = buffer;
            }
        }
    }
}

fn _dettachBuffer(source: ALuint) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Sources.get(&source) {
            let info = *info_ptr;

            // clear buffer on voices
            for (_listener_id, voice) in (*info).m_Voices.iter() {
                if !voice.is_null() {
                    // TODO: Stop
                    // (*voice)->Stop();
                    // TODO: SetBufferData
                    // (*voice)->SetBufferData(NULL, 0);
                }
            }

            (*info).m_Buffer = 0;
        }
    }
}

fn _sourceSetRefDist(info: *mut SourceInfo, value: FLOAT) {
    unsafe {
        for (_listener_id, voice) in (*info).m_Voices.iter() {
            if !voice.is_null() {
                // In order to prevent debug DX from complaining that
                // the max dist is greater than the min dist, I clear
                // the min dist _before_ setting the max.  Ug.
                // TODO: SetMinDistance
                // (*voice)->SetMinDistance(1, DS3D_DEFERRED);

                // New algorithm - ref dist is supposed to be dist at which sound is 1/2 volume,
                // which happens at double min distance in DS, thus: (reverted)
                // TODO: SetMaxDistance, SetMinDistance
                // (*voice)->SetMaxDistance(value * 10.f, DS3D_DEFERRED);
                // (*voice)->SetMinDistance(value, DS3D_DEFERRED);
                // (*voice)->SetMinDistance(value / 2.f, DS3D_DEFERRED);
            }
        }
    }
}

pub extern "C" fn alGenSources2D(mut n: ALsizei, sources: *mut ALuint) {
    unsafe {
        while n > 0 {
            if !_genSource(false) { break; }
            *sources.add((n - 1) as usize) = (*s_pState).m_NextSource;
            (*s_pState).m_NextSource += 1;
            n -= 1;
        }
    }
}

pub extern "C" fn alGenSources3D(mut n: ALsizei, sources: *mut ALuint) {
    unsafe {
        while n > 0 {
            if !_genSource(true) { break; }
            *sources.add((n - 1) as usize) = (*s_pState).m_NextSource;
            (*s_pState).m_NextSource += 1;
            n -= 1;
        }
    }
}

pub extern "C" fn alDeleteSources(mut n: ALsizei, sources: *mut ALuint) {
    unsafe {
        while n > 0 {
            let source_id = *sources.add((n - 1) as usize);

            if let Some((_k, info)) = (*s_pState).m_Sources.remove_entry(&source_id) {
                // stop using any buffers
                _dettachBuffer(source_id);

                // free associated voices
                for (_listener_id, _voice) in (*info).m_Voices.iter() {
                    // TODO: Release
                    // (*voice)->Release();
                }

                Box::from_raw(info);
            }
            n -= 1;
        }
    }
}

pub extern "C" fn alSourcei(source: ALuint, param: ALenum, value: ALint) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Sources.get(&source) {
            let info = *info_ptr;

            match param {
                AL_LOOPING => {
                    (*info).m_Loop = value != 0;
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
}

pub extern "C" fn alSourcef(source: ALuint, param: ALenum, value: ALfloat) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Sources.get(&source) {
            let info = *info_ptr;

            match param {
                AL_REFERENCE_DISTANCE => {
                    _sourceSetRefDist(info, value);
                }
                AL_GAIN => {
                    (*info).m_Gain = value;
                    (*info).m_GainDirty = true;
                }
                _ => {}
            }
        }
    }
}

pub extern "C" fn alSourcefv(source: ALuint, param: ALenum, values: *mut ALfloat) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Sources.get(&source) {
            let info = *info_ptr;

            match param {
                AL_POSITION => {
                    (*info).m_Position.x = *values;
                    (*info).m_Position.y = *values.add(1);
                    (*info).m_Position.z = *values.add(2);
                }
                _ => {}
            }
        }
    }
}

pub extern "C" fn alSourceStop(source: ALuint) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Sources.get(&source) {
            let info = *info_ptr;

            // stop playing for all listeners
            for (_listener_id, voice) in (*info).m_Voices.iter() {
                if !voice.is_null() {
                    // TODO: Stop
                    // (*voice)->Stop();

                    let mut status = 1;
                    loop {
                        // TODO: GetStatus
                        // (*voice)->GetStatus(&status);
                        if status == 0 { break; }
                    }
                }
            }
        }
    }
}

pub extern "C" fn alSourcePlay(source: ALuint) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Sources.get(&source) {
            let info = *info_ptr;

            if (*info).m_Buffer == 0 {
                return;
            }

            // start playing for all listeners
            for (_listener_id, voice) in (*info).m_Voices.iter() {
                if !voice.is_null() {
                    // TODO: SetCurrentPosition
                    // (*voice)->SetCurrentPosition(0);
                    let flags = if (*info).m_Loop { DSBPLAY_LOOPING } else { 0 };
                    // TODO: Play
                    // (*voice)->Play(0, 0, flags);
                }
            }
        }
    }
}

pub extern "C" fn alGetSourcei(source: ALuint, param: ALenum, value: *mut ALint) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Sources.get(&source) {
            let info = *info_ptr;

            match param {
                AL_SOURCE_STATE => {
                    let mut status = 0u32;
                    if let Some((_k, voice)) = (*info).m_Voices.iter().next() {
                        // TODO: GetStatus
                        // (*voice)->GetStatus(&status);
                    }
                    *value = if (status & DSBSTATUS_PLAYING) != 0 { AL_PLAYING as ALint } else { AL_STOPPED as ALint };
                }
                _ => {}
            }
        }
    }
}




/***********************************************
*
* BUFFERS
*
************************************************/

pub extern "C" fn alGenBuffers(mut n: ALsizei, buffers: *mut ALuint) {
    unsafe {
        while n > 0 {
            let info = Box::new(BufferInfo {
                m_Data: ptr::null_mut(),
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

            let buffer_id = (*s_pState).m_NextBuffer;
            let info_ptr = Box::into_raw(info);
            (*s_pState).m_Buffers.insert(buffer_id, info_ptr);
            *buffers.add((n - 1) as usize) = buffer_id;
            (*s_pState).m_NextBuffer += 1;

            n -= 1;
        }
    }
}

pub extern "C" fn alDeleteBuffers(mut n: ALsizei, buffers: *mut ALuint) {
    unsafe {
        while n > 0 {
            let buffer_id = *buffers.add((n - 1) as usize);

            if let Some((_k, binfo)) = (*s_pState).m_Buffers.remove_entry(&buffer_id) {

                if (*binfo).m_Valid {
                    // dettach buffer from any sources using it (may block)
                    let sources_to_detach: Vec<ALuint> = (*s_pState).m_Sources.iter()
                        .filter(|(_k, sinfo)| (*sinfo).m_Buffer == buffer_id)
                        .map(|(&k, _)| k)
                        .collect();

                    for source_id in sources_to_detach {
                        _dettachBuffer(source_id);
                    }

                    // free the memory
                    Z_Free((*binfo).m_Data);
                    (*s_pState).m_MemoryUsed -= (*binfo).m_Size;
                }

                Box::from_raw(binfo);
            }
            n -= 1;
        }
    }
}

pub extern "C" fn alBufferData(buffer: ALuint, format: ALenum, data: *mut ALvoid, size: ALsizei, freq: ALsizei) {
    unsafe {
        if let Some(info_ptr) = (*s_pState).m_Buffers.get(&buffer) {
            let info = *info_ptr;

            // if this buffer has been used before, clear the old data
            if (*info).m_Valid {
                Z_Free((*info).m_Data);
                (*s_pState).m_MemoryUsed -= (*info).m_Size;
                (*info).m_Valid = false;
            }

            (*info).m_Data = data;

            // assume we have a wave file...
            let wav = (data as *mut u8).add(20) as *mut WAVEFORMATEX;
            (*info).m_DataOffset = (20 + std::mem::size_of::<WAVEFORMATEX>() + (*wav).cbSize as usize + 8) as DWORD;

            (*info).m_Size = size as DWORD;
            (*s_pState).m_MemoryUsed += (*info).m_Size;

            _wavSetFormat(&mut (*info).m_WAVFormat, format, freq);

            (*info).m_Valid = true;
        }
    }
}


/***********************************************
*
* STREAMS
*
************************************************/

fn _streamFromFile() -> i32 {
    unsafe {
        let mut total = 0u32;
        let mut used = 0u32;

        // setup a media packet for reading from the file
        let mut xmp = XMEDIAPACKET {
            pvBuffer: ((*s_pState).m_Stream.m_pPacketBuffer as *mut u8)
                .add(QAL_STREAM_PACKET_SIZE * (*s_pState).m_Stream.m_CurrentPacket as usize),
            dwMaxSize: QAL_STREAM_PACKET_SIZE as DWORD,
            pdwCompletedSize: &mut used,
            pdwStatus: ptr::null_mut(),
            pContext: ptr::null_mut(),
        };

        WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

        // loop until we have a full packet of data
        while total < QAL_STREAM_PACKET_SIZE as u32 {
            // TODO: Call Process on m_pFile
            // if (DS_OK != s_pState->m_Stream.m_pFile->Process(NULL, &xmp))
            if false {
                ReleaseMutex(Sys_FileStreamMutex);
                return -1;
            }

            total += used;

            // did we get enough data?
            if used < xmp.dwMaxSize {
                if (*s_pState).m_Stream.m_Looping {
                    // must have reached the end of the file, loop back
                    // around to the beginning and get more data
                    xmp.pvBuffer = xmp.pvBuffer.add(used as usize);
                    xmp.dwMaxSize = xmp.dwMaxSize - used;

                    // TODO: Call Seek on m_pFile
                    // if (DS_OK != s_pState->m_Stream.m_pFile->Seek(0, FILE_BEGIN, NULL))
                    if false {
                        ReleaseMutex(Sys_FileStreamMutex);
                        return -1;
                    }
                } else {
                    // reached end, finish up
                    (*s_pState).m_Stream.m_Playing = false;
                    ReleaseMutex(Sys_FileStreamMutex);
                    return used as i32;
                }
            }
        }

        ReleaseMutex(Sys_FileStreamMutex);

        QAL_STREAM_PACKET_SIZE as i32
    }
}

fn _streamToVoice(size: i32) {
    unsafe {
        // setup a packet with the current data
        let mut xmp = XMEDIAPACKET {
            pvBuffer: ((*s_pState).m_Stream.m_pPacketBuffer as *mut u8)
                .add(QAL_STREAM_PACKET_SIZE * (*s_pState).m_Stream.m_CurrentPacket as usize),
            dwMaxSize: size as DWORD,
            pdwCompletedSize: ptr::null_mut(),
            pdwStatus: &mut (*s_pState).m_Stream.m_PacketStatus[(*s_pState).m_Stream.m_CurrentPacket as usize],
            pContext: ptr::null_mut(),
        };

        // sent to the voice
        // TODO: Call Process on m_pVoice
        // s_pState->m_Stream.m_pVoice->Process(&xmp, NULL);

        // make sure we're playing
        // TODO: Call Pause on m_pVoice
        // s_pState->m_Stream.m_pVoice->Pause(DSSTREAMPAUSE_RESUME);
        if (*s_pState).m_Stream.m_StartTime == 0 {
            (*s_pState).m_Stream.m_StartTime = Sys_Milliseconds() as c_uint;
        }
    }
}

fn _streamFill() {
    unsafe {
        // do we have any free packets?
        if XMEDIAPACKET_STATUS_PENDING !=
            (*s_pState).m_Stream.m_PacketStatus[(*s_pState).m_Stream.m_CurrentPacket as usize] {
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
                // TODO: Call Discontinuity on m_pVoice
                // s_pState->m_Stream.m_pVoice->Discontinuity();
            }
        }
    }
}

fn _streamOpen(file: DWORD, offset: DWORD, looping: bool) {
    unsafe {
        if (*s_pState).m_Stream.m_Open {
            // if a stream is current playing, interrupt it
            // TODO: Call Flush, Release
            // s_pState->m_Stream.m_pVoice->Flush();
            // s_pState->m_Stream.m_pFile->Release();
            (*s_pState).m_Stream.m_Playing = false;
            (*s_pState).m_Stream.m_Open = false;
        }

        let name = Sys_GetFileCodeName(file as c_int);

        WaitForSingleObject(Sys_FileStreamMutex, INFINITE);

        // open the file for streaming
        let mut fmt: *const WAVEFORMATEX = ptr::null();
        if DS_OK == XWaveFileCreateMediaObject(
            name, &mut fmt, &mut (*s_pState).m_Stream.m_pFile) {
            // set the voice based on the file format
            // TODO: Call SetFormat
            // s_pState->m_Stream.m_pVoice->SetFormat(fmt);

            // #ifdef _FIVE_CHANNEL
            //     DSMIXBINVOLUMEPAIR dsmbvp[6] = {
            //         DSMIXBINVOLUMEPAIRS_DEFAULT_5CHANNEL_3D,
            //     };
            //     DSMIXBINS dsmb;
            //     dsmb.dwMixBinCount = 6;
            //     dsmb.lpMixBinVolumePairs = dsmbvp;
            //
            //     s_pState->m_Stream.m_pVoice->SetMixBins(&dsmb);
            // #endif

            // seek the requested start position
            // TODO: Call Seek - RoundDown is macro that we need to handle
            let rounded = (offset / 72) * 72; // RoundDown(offset, 72)
            // s_pState->m_Stream.m_pFile->Seek(rounded, FILE_BEGIN, NULL);

            (*s_pState).m_Stream.m_StartTime = 0;
            (*s_pState).m_Stream.m_Looping = looping;
            (*s_pState).m_Stream.m_Playing = true;
            (*s_pState).m_Stream.m_Open = true;
        }

        ReleaseMutex(Sys_FileStreamMutex);
    }
}

fn _streamClose() {
    unsafe {
        if (*s_pState).m_Stream.m_Open {
            // stop the stream
            // TODO: Call Flush, Release
            // s_pState->m_Stream.m_pVoice->Flush();
            // s_pState->m_Stream.m_pFile->Release();
            (*s_pState).m_Stream.m_Playing = false;
            (*s_pState).m_Stream.m_Open = false;
        }
    }
}

extern "C" fn _streamThread(_lpParameter: *mut c_void) -> DWORD {
    unsafe {
        loop {
            let strm = &mut (*s_pState).m_Stream;
            let mut req = Request {
                m_Type: RequestType::REQ_NOP,
                m_Data: [0; 3],
            };

            // Wait for the queue to fill
            WaitForSingleObject(strm.m_QueueLen, QAL_STREAM_WAIT_TIME);

            // Grab the next request
            WaitForSingleObject(strm.m_Mutex, INFINITE);
            if !(*strm.m_Queue).is_empty() {
                req = (*strm.m_Queue).pop_front().unwrap();
            } else {
                req.m_Type = RequestType::REQ_NOP;
            }
            ReleaseMutex(strm.m_Mutex);

            // Process request
            match req.m_Type {
                RequestType::REQ_PLAY => {
                    _streamOpen(req.m_Data[0], req.m_Data[1], req.m_Data[2] != 0);
                }

                RequestType::REQ_STOP => {
                    _streamClose();
                }

                RequestType::REQ_SHUTDOWN => {
                    ExitThread(0);
                }

                RequestType::REQ_NOP => {
                }
            }

            // fill the stream with data
            if strm.m_Open && strm.m_Playing {
                _streamFill();
            }
        }
    }
}

fn _postStreamRequest(req: Request) {
    unsafe {
        // Add request to queue
        WaitForSingleObject((*s_pState).m_Stream.m_Mutex, INFINITE);
        (*(*s_pState).m_Stream.m_Queue).push_back(req);
        ReleaseMutex((*s_pState).m_Stream.m_Mutex);

        // Let thread know it has one more pending request
        ReleaseSemaphore((*s_pState).m_Stream.m_QueueLen, 1, ptr::null_mut());

        // Give the stream thread some CPU
        Sleep(0);
    }
}

pub extern "C" fn Sys_StreamRequestQueueClear() {
    unsafe {
        WaitForSingleObject((*s_pState).m_Stream.m_Mutex, INFINITE);
        Box::from_raw((*s_pState).m_Stream.m_Queue);
        (*s_pState).m_Stream.m_Queue = Box::into_raw(Box::new(VecDeque::<Request>::new()));
        ReleaseMutex((*s_pState).m_Stream.m_Mutex);
    }
}

pub extern "C" fn alGenStream() {
    unsafe {
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
            dwSize: std::mem::size_of::<DSSTREAMDESC>() as DWORD,
            dwFlags: 0,
            dwMaxAttachedPackets: QAL_MAX_STREAM_PACKETS as DWORD,
            lpwfxFormat: &mut wav as *mut _ as *mut WAVEFORMATEX,
            lpMixBins: ptr::null_mut(),
        };

        // create a voice for the stream
        // TODO: Call CreateSoundStream
        // if (s_pState->m_SoundObject->CreateSoundStream(&desc,
        //     &s_pState->m_Stream.m_pVoice, NULL) != DS_OK)
        if false {
            (*s_pState).m_Error = AL_OUT_OF_MEMORY;
            return;
        }

        // get some memory to hold the stream data
        (*s_pState).m_Stream.m_pPacketBuffer =
            XPhysicalAlloc((QAL_MAX_STREAM_PACKETS * QAL_STREAM_PACKET_SIZE) as DWORD,
            MAXULONG_PTR, 0, PAGE_READWRITE | PAGE_NOCACHE);

        // setup some defaults
        (*s_pState).m_Stream.m_Gain = 1.0f32;
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
        (*s_pState).m_Stream.m_QueueLen = CreateSemaphore(ptr::null_mut(), 0, 256, ptr::null());
        (*s_pState).m_Stream.m_Mutex = CreateMutex(ptr::null_mut(), FALSE, ptr::null());
        (*s_pState).m_Stream.m_Thread = CreateThread(ptr::null_mut(), 0,
            _streamThread, ptr::null_mut(), 0, ptr::null_mut());
    }
}

pub extern "C" fn alDeleteStream() {
    unsafe {
        // stop the audio
        alStreamStop();

        // kill the thread
        let req = Request {
            m_Type: RequestType::REQ_SHUTDOWN,
            m_Data: [0; 3],
        };
        _postStreamRequest(req);

        // Wait for thread to close
        WaitForSingleObject((*s_pState).m_Stream.m_Thread, INFINITE);

        // thread handles
        CloseHandle((*s_pState).m_Stream.m_Thread);
        CloseHandle((*s_pState).m_Stream.m_Mutex);
        CloseHandle((*s_pState).m_Stream.m_QueueLen);

        // release the stream
        // TODO: Call Release on m_pVoice
        // s_pState->m_Stream.m_pVoice->Release();
        XPhysicalFree((*s_pState).m_Stream.m_pPacketBuffer);

        (*s_pState).m_Stream.m_Valid = false;
    }
}

pub extern "C" fn alStreamStop() {
    unsafe {
        let req = Request {
            m_Type: RequestType::REQ_STOP,
            m_Data: [0; 3],
        };
        _postStreamRequest(req);
    }
}

pub extern "C" fn alStreamPlay(offset: ALsizei, file: ALint, looping: ALint) {
    unsafe {
        let req = Request {
            m_Type: RequestType::REQ_PLAY,
            m_Data: [file as DWORD, offset as DWORD, looping as DWORD],
        };
        _postStreamRequest(req);

        (*s_pState).m_Stream.m_Playing = true;
    }
}

pub extern "C" fn alStreamf(param: ALenum, value: ALfloat) {
    unsafe {
        match param {
            AL_GAIN => {
                (*s_pState).m_Stream.m_Gain = value;
                (*s_pState).m_Stream.m_GainDirty = true;
            }
            _ => {}
        }
    }
}

pub extern "C" fn alGetStreamf(param: ALenum, value: *mut ALfloat) {
    unsafe {
        match param {
            AL_TIME => {
                if (*s_pState).m_Stream.m_Open && (*s_pState).m_Stream.m_StartTime != 0 {
                    *value = (Sys_Milliseconds() as c_uint - (*s_pState).m_Stream.m_StartTime) as f32 / 1000.0f32;
                } else {
                    *value = 0.0f32;
                }
            }
            _ => {}
        }
    }
}

pub extern "C" fn alGetStreami(param: ALenum, value: *mut ALint) {
    unsafe {
        match param {
            AL_SOURCE_STATE => {
                *value = if (*s_pState).m_Stream.m_Playing { AL_PLAYING as ALint } else { AL_STOPPED as ALint };
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

fn _updateVoiceGain(voice: *mut IDirectSoundBuffer, gain: FLOAT) {
    unsafe {
        // compute aggregate gain
        let mut g = (*s_pState).m_Gain * gain;

        if g <= 0.0f32 {
            // mute the sound
            // TODO: Call SetVolume
            // voice->SetVolume(DSBVOLUME_MIN);
        } else {
            // convert to dB
            g = 20.0f32 * g.log10();

            if g < -100.0f32 {
                g = -100.0f32;
            }

            // set the volume
            // TODO: Call SetVolume
            // voice->SetVolume((g * 100.0f32) as i32);
        }
    }
}

fn _updateVoicePos(voice: *mut IDirectSoundBuffer, pos: *const D3DXVECTOR3,
    listener: *mut ListenerInfo) {
    unsafe {
        // get source pos in listener space
        let mut lpos = D3DXVECTOR4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
        // TODO: Call D3DXVec3Transform
        // D3DXVec3Transform(&lpos, pos, &(*listener).m_LTM);

        // TODO: Call SetPosition
        // voice->SetPosition(lpos.x, lpos.y, lpos.z, DS3D_DEFERRED);
    }
}

fn _updateSource(source: *mut SourceInfo) {
    unsafe {
        // loop through all the voices at this source
        for (_listener_id, voice) in (*source).m_Voices.iter() {
            // update the gain
            if (*source).m_GainDirty {
                _updateVoiceGain(*voice, (*source).m_Gain);
            }

            // update position
            if (*source).m_Is3d {
                // get the listener for this voice
                if let Some(listener_ptr) = (*s_pState).m_Listeners.get(&_listener_id) {
                    _updateVoicePos(
                        *voice,
                        &(*source).m_Position,
                        *listener_ptr);
                }
            }
        }

        (*source).m_GainDirty = false;
    }
}

fn _updateStream() {
    unsafe {
        if (*s_pState).m_Stream.m_Open && (*s_pState).m_Stream.m_GainDirty {
            // compute aggregate gain
            let mut g = (*s_pState).m_Gain * (*s_pState).m_Stream.m_Gain;
            if g <= 0.0f32 {
                // mute the sound
                // TODO: Call SetVolume
                // s_pState->m_Stream.m_pVoice->SetVolume(DSBVOLUME_MIN);
            } else {
                // convert to dB
                g = 20.0f32 * g.log10();

                if g < -100.0f32 {
                    g = -100.0f32;
                }

                // set the volume
                // TODO: Call SetVolume
                // s_pState->m_Stream.m_pVoice->SetVolume((g * 100.0f32) as i32);
            }

            (*s_pState).m_Stream.m_GainDirty = false;
        }
    }
}

pub extern "C" fn alGetError() -> ALenum {
    unsafe {
        let error = (*s_pState).m_Error;
        (*s_pState).m_Error = AL_NO_ERROR;
        error
    }
}

pub extern "C" fn alUpdate() {
    unsafe {
        // TODO: Call DirectSoundDoWork
        // DirectSoundDoWork();

        // update sources
        let source_ids: Vec<ALuint> = (*s_pState).m_Sources.keys().copied().collect();
        for source_id in source_ids {
            if let Some(info_ptr) = (*s_pState).m_Sources.get(&source_id) {
                let info = *info_ptr;

                // 3d sounds and dirty sources must be updated
                if (*info).m_Is3d || (*info).m_GainDirty {
                    // only playing sources should be updated
                    let mut status = 0u32;
                    if let Some((_k, voice)) = (*info).m_Voices.iter().next() {
                        // TODO: Call GetStatus
                        // (*voice)->GetStatus(&status);
                    }

                    if (status & DSBSTATUS_PLAYING) != 0 {
                        _updateSource(info);
                    }
                }
            }
        }

        // update stream
        _updateStream();

        // TODO: Call CommitDeferredSettings
        // s_pState->m_SoundObject->CommitDeferredSettings();
    }
}

pub extern "C" fn alGeti(param: ALenum, value: *mut ALint) {
    unsafe {
        match param {
            AL_MEMORY_USED => {
                *value = (*s_pState).m_MemoryUsed as ALint;
            }
            _ => {}
        }
    }
}

pub extern "C" fn alGain(value: ALfloat) {
    unsafe {
        (*s_pState).m_Gain = value;

        // set gain dirty for all sources
        let source_ids: Vec<ALuint> = (*s_pState).m_Sources.keys().copied().collect();
        for source_id in source_ids {
            if let Some(info_ptr) = (*s_pState).m_Sources.get(&source_id) {
                (*info_ptr).m_GainDirty = true;
            }
        }

        // set gain dirty for stream
        (*s_pState).m_Stream.m_GainDirty = true;
    }
}

// Placeholder macro implementation for compile-time
#[allow(non_snake_case)]
mod impl_com_methods {
    // Placeholder for COM method implementations
    // These would normally come from DirectX SDK
}
