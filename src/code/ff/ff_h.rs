#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

use core::ffi::{c_char, c_int};

// #include "../ff/ff_public.h"

// LOCAL STUB: qboolean type from engine
pub type qboolean = c_int;

// From ff_public.h
pub const FF_HANDLE_NULL: c_int = 0;
pub const FF_CLIENT_LOCAL: c_int = -2;
pub type ffHandle_t = c_int;

// Placeholder for QTRUE from q_shared.h
pub const QTRUE: c_int = 1;

#[cfg(feature = "_FF")]
extern "C" {
    //
    //	Externally visible functions
    //

    pub fn FF_Init() -> qboolean;
    pub fn FF_Shutdown();
    pub fn FF_IsAvailable() -> qboolean;
    pub fn FF_IsInitialized() -> qboolean;
    // Note: In C, this function has a default parameter: qboolean notfound = qtrue
    // Rust extern "C" functions don't support default parameters, so the caller must provide all arguments
    pub fn FF_Register(ff: *const c_char, channel: c_int, notfound: qboolean) -> ffHandle_t;
    pub fn FF_Play(ff: ffHandle_t) -> qboolean;
    pub fn FF_EnsurePlaying(ff: ffHandle_t) -> qboolean;
    pub fn FF_Stop(ff: ffHandle_t) -> qboolean;
    pub fn FF_StopAll() -> qboolean;
    pub fn FF_Shake(intensity: c_int, duration: c_int) -> qboolean;
}

#[cfg(all(feature = "_FF", feature = "FF_CONSOLECOMMAND"))]
pub type xcommand_t = extern "C" fn();

#[cfg(all(feature = "_FF", feature = "FF_CONSOLECOMMAND"))]
extern "C" {
    pub fn CMD_FF_StopAll();
    pub fn CMD_FF_Info();
}

// //ffExport_t* GetFFAPI ( int apiVersion, ffImport_t *ffimp );
