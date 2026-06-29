//! Mechanical port of `code/null/null_glimp.c`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// ============================================================================
// Type definitions (from renderer/tr_local.h stubs)
// ============================================================================

/// `qboolean` — engine boolean, passed as a C `int`.
pub type qboolean = c_int;

/// GL texture unit enum stub (from OpenGL extension interface).
pub type texture = c_int;

/// Renderer error return type (from renderer/tr_local.h).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum rserr_t {
    RSERR_OK = 0,
}

// ============================================================================
// Global function pointers
// ============================================================================

pub static mut qwglSwapIntervalEXT: Option<extern "C" fn(c_int) -> qboolean> = None;
pub static mut qglMultiTexCoord2fARB: Option<extern "C" fn(texture, f32, f32)> = None;
pub static mut qglActiveTextureARB: Option<extern "C" fn(texture)> = None;
pub static mut qglClientActiveTextureARB: Option<extern "C" fn(texture)> = None;

pub static mut qglLockArraysEXT: Option<extern "C" fn(c_int, c_int)> = None;
pub static mut qglUnlockArraysEXT: Option<extern "C" fn()> = None;

// ============================================================================
// Functions
// ============================================================================

pub fn GLimp_EndFrame() {
}

pub fn GLimp_Init() -> c_int {
}

pub fn GLimp_Shutdown() {
}

pub fn GLimp_SetMode(
    drivername: *const c_char,
    pWidth: *mut c_int,
    pHeight: *mut c_int,
    mode: c_int,
    fullscreen: qboolean,
) -> rserr_t {
}

pub fn GLimp_EnableLogging(enable: qboolean) {
}

pub fn GLimp_LogComment(comment: *mut c_char) {
}

pub fn QGL_Init(dllname: *const c_char) -> qboolean {
    true as c_int
}

pub fn QGL_Shutdown() {
}
