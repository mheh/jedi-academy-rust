// mac_snddma.rs
// all other sound mixing is portable

use core::ffi::{c_int, c_void};
use std::ptr::{addr_of_mut, write_bytes};
use std::mem;

// Opaque types from <sound.h> (Mac Sound Manager)
#[repr(C)]
pub struct SndChannel {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct SndCommand {
    pub cmd: c_int,
    pub param1: c_int,
    pub param2: c_int,
}

#[repr(C)]
pub struct ExtSoundHeader {
    pub samplePtr: *mut c_void,
    pub numChannels: c_int,
    pub sampleRate: c_int,
    pub loopStart: c_int,
    pub loopEnd: c_int,
    pub encode: c_int,
    pub baseFrequency: c_int,
    pub numFrames: c_int,
    pub markerChunk: *mut c_void,
    pub instrumentChunks: *mut c_void,
    pub AESRecording: *mut c_void,
    pub sampleSize: c_int,
}

// Mac Sound Manager functions
extern "C" {
    fn SndNewChannel(channel: *mut *mut SndChannel, synth: c_int, init: c_int, callback: *mut c_void) -> c_int;
    fn SndDoCommand(channel: *mut SndChannel, cmd: *mut SndCommand, noWait: bool) -> c_int;
    fn SndDisposeChannel(channel: *mut SndChannel, quietNow: bool) -> c_int;
    fn NewSndCallBackProc(callback: *const c_void) -> *mut c_void;
}

// Mac Sound Manager constants
extern "C" {
    static rate22khz: c_int;
    static extSH: c_int;
    static sampledSynth: c_int;
    static initStereo: c_int;
    static bufferCmd: c_int;
    static callBackCmd: c_int;
}

// Game engine DMA struct (from ../client/snd_local.h)
#[repr(C)]
pub struct dma_t {
    pub channels: c_int,
    pub samples: c_int,
    pub submission_chunk: c_int,
    pub samplebits: c_int,
    pub speed: c_int,
    pub buffer: *mut u8,
}

extern "C" {
    pub static mut dma: dma_t;
}

const MAX_MIXED_SAMPLES: usize = 0x8000;
const SUBMISSION_CHUNK: usize = 0x100;

static mut s_mixedSamples: [i16; MAX_MIXED_SAMPLES] = [0; MAX_MIXED_SAMPLES];
static mut s_chunkCount: c_int = 0;			// number of chunks submitted
static mut s_sndChan: *mut SndChannel = 0 as *mut SndChannel;
static mut s_sndHeader: ExtSoundHeader = ExtSoundHeader {
    samplePtr: 0 as *mut c_void,
    numChannels: 0,
    sampleRate: 0,
    loopStart: 0,
    loopEnd: 0,
    encode: 0,
    baseFrequency: 0,
    numFrames: 0,
    markerChunk: 0 as *mut c_void,
    instrumentChunks: 0 as *mut c_void,
    AESRecording: 0 as *mut c_void,
    sampleSize: 0,
};

/*
===============
S_Callback
===============
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn S_Callback(sc: *mut SndChannel, cmd: *mut SndCommand) {
    let mut mySndCmd: SndCommand = mem::zeroed();
    let mut mySndCmd2: SndCommand = mem::zeroed();
    let offset: c_int;

    offset = (s_chunkCount * (SUBMISSION_CHUNK as c_int)) & ((MAX_MIXED_SAMPLES as c_int) - 1);

    // queue up another sound buffer
    write_bytes(addr_of_mut!(s_sndHeader) as *mut u8, 0, mem::size_of::<ExtSoundHeader>());
    s_sndHeader.samplePtr = (addr_of_mut!(s_mixedSamples) as *mut i16).add(offset as usize) as *mut c_void;
    s_sndHeader.numChannels = 2;
    s_sndHeader.sampleRate = rate22khz;
    s_sndHeader.loopStart = 0;
    s_sndHeader.loopEnd = 0;
    s_sndHeader.encode = extSH;
    s_sndHeader.baseFrequency = 1;
    s_sndHeader.numFrames = (SUBMISSION_CHUNK / 2) as c_int;
    s_sndHeader.markerChunk = 0 as *mut c_void;
    s_sndHeader.instrumentChunks = 0 as *mut c_void;
    s_sndHeader.AESRecording = 0 as *mut c_void;
    s_sndHeader.sampleSize = 16;

    mySndCmd.cmd = bufferCmd;
    mySndCmd.param1 = 0;
    mySndCmd.param2 = addr_of_mut!(s_sndHeader) as c_int;
    SndDoCommand(sc, addr_of_mut!(mySndCmd), true);

    // and another callback
    mySndCmd2.cmd = callBackCmd;
    mySndCmd2.param1 = 0;
    mySndCmd2.param2 = 0;
    SndDoCommand(sc, addr_of_mut!(mySndCmd2), true);

    s_chunkCount += 1;		// this is the next buffer we will submit
}

/*
===============
S_MakeTestPattern
===============
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn S_MakeTestPattern() {
    let mut i: c_int;
    let mut v: f32;
    let mut sample: c_int;

    i = 0;
    while i < (dma.samples / 2) {
        v = (std::f32::consts::PI * 2.0 * (i as f32) / 64.0).sin();
        sample = (v * 0x4000 as f32) as c_int;
        let buffer_ptr = dma.buffer as *mut i16;
        *buffer_ptr.add((i * 2) as usize) = sample as i16;
        *buffer_ptr.add(((i * 2) + 1) as usize) = sample as i16;
        i += 1;
    }
}

/*
===============
SNDDMA_Init
===============
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn SNDDMA_Init() -> bool {
    let err: c_int;

    // create a sound channel
    s_sndChan = 0 as *mut SndChannel;
    err = SndNewChannel(addr_of_mut!(s_sndChan), sampledSynth, initStereo, NewSndCallBackProc(S_Callback as *const c_void));
    if err != 0 {
        return false;
    }

    dma.channels = 2;
    dma.samples = MAX_MIXED_SAMPLES as c_int;
    dma.submission_chunk = SUBMISSION_CHUNK as c_int;
    dma.samplebits = 16;
    dma.speed = 22050;
    dma.buffer = s_mixedSamples.as_mut_ptr() as *mut u8;

    // que up the first submission-chunk sized buffer
    s_chunkCount = 0;

    S_Callback(s_sndChan, 0 as *mut SndCommand);

    return true;
}

/*
===============
SNDDMA_GetDMAPos
===============
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn SNDDMA_GetDMAPos() -> c_int {
    return s_chunkCount * (SUBMISSION_CHUNK as c_int);
}

/*
===============
SNDDMA_Shutdown
===============
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn SNDDMA_Shutdown() {
    if !s_sndChan.is_null() {
        SndDisposeChannel(s_sndChan, true);
        s_sndChan = 0 as *mut SndChannel;
    }
}

/*
===============
SNDDMA_BeginPainting
===============
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn SNDDMA_BeginPainting() {
}

/*
===============
SNDDMA_Submit
===============
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn SNDDMA_Submit() {
}
