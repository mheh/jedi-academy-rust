#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};

// From original alut.h:
// #define ALUTAPI
// #define ALUTAPIENTRY __cdecl
// These map directly to the extern "C" calling convention.

// From altypes.h (via al.h and alu.h includes):
// typedef char ALboolean;
// typedef char ALbyte;
// typedef int ALint;
// typedef unsigned int ALsizei;
// typedef void ALvoid;
// typedef int ALenum;

extern "C" {
    /// Initializes the audio library (alutInit in original)
    pub fn alutInit(argc: *mut c_int, argv: *mut *mut c_char);

    /// Closes the audio library (alutExit in original)
    pub fn alutExit();

    /// Loads a WAV file from disk (alutLoadWAVFile in original)
    pub fn alutLoadWAVFile(
        file: *mut c_char,
        format: *mut c_int,
        data: *mut *mut c_void,
        size: *mut c_uint,
        freq: *mut c_uint,
        loop_: *mut c_char,
    );

    /// Loads a WAV file from memory (alutLoadWAVMemory in original)
    pub fn alutLoadWAVMemory(
        memory: *mut c_char,
        format: *mut c_int,
        data: *mut *mut c_void,
        size: *mut c_uint,
        freq: *mut c_uint,
        loop_: *mut c_char,
    );

    /// Unloads WAV data (alutUnloadWAV in original)
    pub fn alutUnloadWAV(format: c_int, data: *mut c_void, size: c_uint, freq: c_uint);
}
