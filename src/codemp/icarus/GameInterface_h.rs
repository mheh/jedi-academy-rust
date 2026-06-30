#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_long};
use std::collections::BTreeMap;
use crate::ffi::types::qboolean;
use crate::codemp::icarus::instance_h::ICARUS_Instance;
use crate::codemp::icarus::interface_h::interface_export_t;
use crate::codemp::game::g_public_h::sharedEntity_t;

#[repr(C)]
pub struct pscript_s {
    pub buffer: *mut c_char,
    pub length: c_long,
}

pub type pscript_t = pscript_s;

pub type entlist_t = BTreeMap<String, c_int>;

pub type bufferlist_t = BTreeMap<String, *mut pscript_t>;

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
