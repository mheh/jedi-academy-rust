#![allow(non_snake_case, non_camel_case_types, unused_variables)]

use core::ffi::{c_char, c_float, c_int, c_uint};

use crate::ffi::types::{qboolean, QTRUE};

pub type GLenum = c_uint;
#[cfg(windows)]
pub type BOOL = c_int;

#[cfg(windows)]
#[no_mangle]
pub static mut qwglSwapIntervalEXT: Option<unsafe extern "system" fn(interval: c_int) -> BOOL> =
    None;

#[cfg(not(windows))]
#[no_mangle]
pub static mut qwglSwapIntervalEXT: Option<unsafe extern "C" fn(interval: c_int) -> qboolean> =
    None;

#[cfg(not(windows))]
#[no_mangle]
pub static mut qglMultiTexCoord2fARB: Option<
    unsafe extern "C" fn(texture: GLenum, s: c_float, t: c_float),
> = None;

#[cfg(not(windows))]
#[no_mangle]
pub static mut qglActiveTextureARB: Option<unsafe extern "C" fn(texture: GLenum)> = None;

#[cfg(not(windows))]
#[no_mangle]
pub static mut qglClientActiveTextureARB: Option<unsafe extern "C" fn(texture: GLenum)> = None;

#[cfg(windows)]
#[no_mangle]
pub static mut qglLockArraysEXT: Option<unsafe extern "system" fn(arg0: c_int, arg1: c_int)> =
    None;

#[cfg(not(windows))]
#[no_mangle]
pub static mut qglLockArraysEXT: Option<unsafe extern "C" fn(arg0: c_int, arg1: c_int)> = None;

#[cfg(windows)]
#[no_mangle]
pub static mut qglUnlockArraysEXT: Option<unsafe extern "system" fn()> = None;

#[cfg(not(windows))]
#[no_mangle]
pub static mut qglUnlockArraysEXT: Option<unsafe extern "C" fn()> = None;

#[no_mangle]
pub extern "C" fn GLimp_EndFrame() {}

#[no_mangle]
pub extern "C" fn GLimp_Init() {}

#[no_mangle]
pub extern "C" fn GLimp_Shutdown() {}

#[no_mangle]
pub extern "C" fn GLimp_EnableLogging(enable: qboolean) {}

#[no_mangle]
pub extern "C" fn GLimp_LogComment(comment: *mut c_char) {}

#[no_mangle]
pub extern "C" fn QGL_Init(dllname: *const c_char) -> qboolean {
    QTRUE
}

#[no_mangle]
pub extern "C" fn QGL_Shutdown() {}
