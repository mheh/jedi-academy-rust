#![allow(non_snake_case)]

use core::ffi::{c_int, c_short, c_uint, c_void};

// Note: altypes.h types are defined as:
// ALint = c_int
// ALshort = c_short
// ALfloat = f32
// ALuint = c_uint
// ALsizei = c_uint
// ALvoid = c_void
// ALenum = c_int

pub const ALUAPI: &str = "";
pub const ALUAPIENTRY: &str = "__cdecl";

pub const BUFFERSIZE: c_int = 48000;
pub const FRACTIONBITS: c_int = 14;
pub const FRACTIONMASK: c_uint = ((1u32 << 14) - 1) as c_uint;
pub const OUTPUTCHANNELS: c_int = 2;

extern "C" {
    pub fn aluF2L(value: f32) -> c_int;
    pub fn aluF2S(value: f32) -> c_short;
    pub fn aluCrossproduct(inVector1: *mut f32, inVector2: *mut f32, outVector: *mut f32);
    pub fn aluDotproduct(inVector1: *mut f32, inVector2: *mut f32) -> f32;
    pub fn aluNormalize(inVector: *mut f32);
    pub fn aluMatrixVector(matrix: *mut [[f32; 3]; 3], vector: *mut f32);
    pub fn aluCalculateSourceParameters(
        source: c_uint,
        channels: c_uint,
        drysend: *mut f32,
        wetsend: *mut f32,
        pitch: *mut f32,
    );
    pub fn aluMixData(context: *mut c_void, buffer: *mut c_void, size: c_uint, format: c_int);
    pub fn aluSetReverb(Reverb: *mut c_void, Environment: c_uint);
    pub fn aluReverb(Reverb: *mut c_void, Buffer: *mut [[f32; 2]], BufferSize: c_uint);
}
