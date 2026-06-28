//! Mechanical port of `code/ff/cl_ff.cpp`.
//!
//! Porting note: The original file was wrapped in `#ifdef _IMMERSION`, but this
//! code is translated unconditionally since _IMMERSION is not yet a Cargo feature.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int};

use crate::ffi::types::qboolean;

// Local stub types and constants for force feedback
pub const FF_HANDLE_NULL: c_int = 0;
pub const FF_CLIENT_LOCAL: c_int = -2;

pub type ffHandle_t = c_int;

// Stub struct for cvar_t (full definition is in codemp/unix/linux_snd.rs)
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

const CVAR_ARCHIVE: c_int = 0x00000001;

// Stub type for clientActive_t (full definition elsewhere)
#[repr(C)]
pub struct clientActive_t {
    // Members omitted - not needed for this translation
}

// External references and function declarations
extern "C" {
    // Global: clientActive_t cl (defined in client module)
    pub static mut cl: clientActive_t;

    // Cvar functions
    pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;

    // Force feedback functions
    pub fn FF_Init() -> qboolean;
    pub fn FF_Shutdown();
    pub fn FF_AddForce(ff: ffHandle_t);
    pub fn FF_Stop(ff: ffHandle_t);
    pub fn FF_AddLoopingForce(ff: ffHandle_t);
}

pub fn CL_InitFF() {
    unsafe {
        let use_ff = Cvar_Get(c"use_ff".as_ptr(), c"0".as_ptr(), CVAR_ARCHIVE);

        if use_ff.is_null() || (*use_ff).integer == 0 || FF_Init() == 0 {
            FF_Shutdown();
        }
    }
}

pub fn CL_ShutdownFF() {
    unsafe {
        FF_Shutdown();
    }
}

fn IsLocalClient(clientNum: c_int) -> qboolean {
    if clientNum == 0 // clientNum == cl.snap.ps.clientNum
        || clientNum == FF_CLIENT_LOCAL // assumed local
    {
        1 // qtrue
    } else {
        0 // qfalse
    }
}

pub fn CL_FF_Start(ff: ffHandle_t, clientNum: c_int) {
    if IsLocalClient(clientNum) != 0 {
        // FF_Play( ff );	// plays instantly
        unsafe {
            FF_AddForce(ff); // plays at end of frame
        }
    }
}

pub fn CL_FF_Stop(ff: ffHandle_t, clientNum: c_int) {
    if IsLocalClient(clientNum) != 0 {
        unsafe {
            FF_Stop(ff);
        }
    }
}

/*
pub fn CL_FF_EnsurePlaying(ff: ffHandle_t, clientNum: c_int) {
    if IsLocalClient(clientNum) != 0 {
        unsafe {
            FF_EnsurePlaying(ff);
        }
    }
}
*/

pub fn CL_FF_AddLoopingForce(ff: ffHandle_t, clientNum: c_int) {
    if IsLocalClient(clientNum) != 0 {
        unsafe {
            FF_AddLoopingForce(ff);
        }
    }
}
