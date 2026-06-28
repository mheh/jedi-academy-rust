#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_long};
use crate::ffi::types::qboolean;

// Stub for C++ class ICARUS_Instance (opaque in Rust)
pub struct ICARUS_Instance {
    _private: [u8; 0],
}

#[repr(C)]
pub struct pscript_s {
    pub buffer: *mut c_char,
    pub length: c_long,
}

pub type pscript_t = pscript_s;

// Stub for C++ std::map<string, int, ...>
pub struct entlist_t {
    _private: [u8; 0],
}

// Stub for C++ std::map<string, pscript_t*, ...>
pub struct bufferlist_t {
    _private: [u8; 0],
}

// Forward declaration stub for interface_export_t (defined elsewhere)
pub struct interface_export_t {
    _private: [u8; 0],
}

// Stub for sharedEntity_t
pub struct sharedEntity_t {
    _private: [u8; 0],
}

// ICARUS includes
unsafe extern "C" {
    pub static interface_export: interface_export_t;

    pub fn Interface_Init(pe: *mut interface_export_t);
    pub fn ICARUS_RunScript(ent: *mut sharedEntity_t, name: *const c_char) -> c_int;
    pub fn ICARUS_RegisterScript(name: *const c_char, bCalledDuringInterrogate: qboolean) -> qboolean;

    pub static iICARUS: *mut ICARUS_Instance;
    pub static ICARUS_BufferList: bufferlist_t;
    pub static ICARUS_EntList: entlist_t;
}

// g_ICARUS.cpp
unsafe extern "C" {
    pub fn ICARUS_Init();
    pub fn ICARUS_ValidEnt(ent: *mut sharedEntity_t) -> qboolean;
    pub fn ICARUS_InitEnt(ent: *mut sharedEntity_t);
    pub fn ICARUS_FreeEnt(ent: *mut sharedEntity_t);
    pub fn ICARUS_AssociateEnt(ent: *mut sharedEntity_t);
    pub fn ICARUS_Shutdown();
    pub fn Svcmd_ICARUS_f();

    pub static ICARUS_entFilter: c_int;
}
