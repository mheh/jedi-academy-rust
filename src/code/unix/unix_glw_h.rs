// You shouldnt be including this file on non-Linux platforms
#![cfg(target_os = "linux")]
#![allow(non_snake_case)]

use core::ffi::c_void;

#[repr(C)]
pub struct glwstate_t {
    pub OpenGLLib: *mut c_void, // instance of OpenGL library
    pub log_fp: *mut c_void,
}

pub static mut glw_state: glwstate_t = glwstate_t {
    OpenGLLib: 0 as *mut c_void,
    log_fp: 0 as *mut c_void,
};
