#![allow(non_snake_case)]

use core::ffi::{c_int, c_short, c_uint, c_void};
use super::altypes_h::{ALint, ALshort, ALfloat, ALvoid, ALuint, ALsizei, ALenum};

// ALUAPI - empty macro for export annotation
// ALUAPIENTRY __cdecl - calling convention macro for function entries

pub const BUFFERSIZE: c_int = 48000;
pub const FRACTIONBITS: c_int = 14;
pub const FRACTIONMASK: c_int = ((1 << FRACTIONBITS) - 1);
pub const OUTPUTCHANNELS: c_int = 2;

extern "C" {
    pub fn aluF2L(value: ALfloat) -> ALint;
    pub fn aluF2S(value: ALfloat) -> ALshort;
    pub fn aluCrossproduct(inVector1: *mut ALfloat, inVector2: *mut ALfloat, outVector: *mut ALfloat);
    pub fn aluDotproduct(inVector1: *mut ALfloat, inVector2: *mut ALfloat) -> ALfloat;
    pub fn aluNormalize(inVector: *mut ALfloat);
    pub fn aluMatrixVector(matrix: *mut [[ALfloat; 3]; 3], vector: *mut ALfloat);
    pub fn aluCalculateSourceParameters(source: ALuint, channels: ALuint, drysend: *mut ALfloat, wetsend: *mut ALfloat, pitch: *mut ALfloat);
    pub fn aluMixData(context: *mut ALvoid, buffer: *mut ALvoid, size: ALsizei, format: ALenum);
    pub fn aluSetReverb(Reverb: *mut ALvoid, Environment: ALuint);
    pub fn aluReverb(Reverb: *mut ALvoid, Buffer: *mut [[ALfloat; 2]], BufferSize: ALsizei);
}
