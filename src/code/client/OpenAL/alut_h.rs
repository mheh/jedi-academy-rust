#![allow(non_snake_case)]

// Original header guard: #ifndef _ALUT_H_ / #define _ALUT_H_

// #define ALUTAPI
// #define ALUTAPIENTRY __cdecl

// #include "al.h"
// #include "alu.h"

use core::ffi::{c_char, c_int, c_uint, c_void};

extern "C" {
    pub fn alutInit(argc: *mut c_int, argv: *mut *mut c_char);
    pub fn alutExit();
    pub fn alutLoadWAVFile(
        file: *mut c_char,
        format: *mut c_int,
        data: *mut *mut c_void,
        size: *mut c_uint,
        freq: *mut c_uint,
        r#loop: *mut c_char,
    );
    pub fn alutLoadWAVMemory(
        memory: *mut c_char,
        format: *mut c_int,
        data: *mut *mut c_void,
        size: *mut c_uint,
        freq: *mut c_uint,
        r#loop: *mut c_char,
    );
    pub fn alutUnloadWAV(format: c_int, data: *mut c_void, size: c_uint, freq: c_uint);
}
