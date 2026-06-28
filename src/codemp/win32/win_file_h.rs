#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

pub type wfhandle_t = c_int;

extern "C" {
    pub fn WF_Init();
    pub fn WF_Shutdown();
    pub fn WF_Open(name: *const c_char, read: bool, aligned: bool) -> wfhandle_t;
    pub fn WF_Close(handle: wfhandle_t);
    pub fn WF_Read(buffer: *mut c_void, len: c_int, handle: wfhandle_t) -> c_int;
    pub fn WF_Write(buffer: *const c_void, len: c_int, handle: wfhandle_t) -> c_int;
    pub fn WF_Seek(offset: c_int, origin: c_int, handle: wfhandle_t) -> c_int;
    pub fn WF_Tell(handle: wfhandle_t) -> c_int;
    pub fn WF_Resize(size: c_int, handle: wfhandle_t) -> c_int;

    pub fn Sys_GetFileCode(name: *const c_char) -> c_int;
    pub fn Sys_InitFileCodes();
    pub fn Sys_ShutdownFileCodes();
}
