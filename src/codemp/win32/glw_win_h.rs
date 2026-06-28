#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::ffi::types::qboolean;
use core::ffi::{c_char, c_int, c_uint, c_void};

pub type HWND = *mut c_void;
pub type HDC = *mut c_void;
pub type HGLRC = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type UINT = c_uint;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;
pub type FILE = c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct glwstate_t {
    pub wndproc: WNDPROC,

    pub hDC: HDC,     // handle to device context
    pub hGLRC: HGLRC, // handle to GL rendering context

    pub hinstOpenGL: HINSTANCE, // HINSTANCE for the OpenGL library

    pub allowdisplaydepthchange: qboolean,
    pub pixelFormatSet: qboolean,

    pub desktopBitsPixel: c_int,
    pub desktopWidth: c_int,
    pub desktopHeight: c_int,

    pub cdsFullscreen: qboolean,

    pub log_fp: *mut FILE,
}

extern "C" {
    pub static mut glw_state: glwstate_t;

    pub fn GL_CheckForExtension(ext: *const c_char) -> bool;
}
