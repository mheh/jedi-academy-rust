//! Mechanical port of `codemp/cgame/cg_light.c`.
//!
//! Light style management: initializes, updates, and applies light style colors based on
//! time and configuration strings.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_int;
use core::ptr::addr_of_mut;
use crate::codemp::cgame::cg_lights_h::clightstyle_t;
use crate::codemp::game::q_shared_h::{byte, MAX_QPATH, MAX_LIGHT_STYLES, ERR_DROP};
use crate::codemp::game::bg_public::CS_LIGHT_STYLES;

// ============================================================================
// External functions and globals
// ============================================================================

extern "C" {
    /// Declared in cg_local.h: retrieves a configuration string by index.
    pub fn CG_ConfigString(index: c_int) -> *const u8;

    /// Declared in cg_local.h: sets the renderer's light style.
    pub fn trap_R_SetLightStyle(style: c_int, color: c_int);

    /// Declared in game/g_main.h or similar: error reporting with formatted message.
    pub fn Com_Error(error_level: c_int, fmt: *const u8, ...);

    /// Standard C library: memory fill operation.
    pub fn memset(dest: *mut core::ffi::c_void, c: c_int, count: usize) -> *mut core::ffi::c_void;
}

/// Safe wrapper for strlen that operates on raw C strings.
///
/// # Safety
/// `s` must point to a NUL-terminated buffer.
unsafe fn strlen(s: *const u8) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

// ============================================================================
// Stub for cg global access
// ============================================================================

/// Stub for cg_t struct to allow access to cg.time.
/// The full struct is not ported yet; only the time field is defined.
#[repr(C)]
pub struct cg_t {
    // Placeholder fields to reach the time field offset
    // Based on cg_local.h: clientFrame, clientNum, demoPlayback, levelShot, etc.
    // This is a simplified stub; the actual layout should be verified against cg_local.h
    pub _pad: [u8; 48], // Offset to the time field
    pub time: c_int,
}

extern "C" {
    /// Global client game state, declared in cg_main.c.
    pub static cg: cg_t;
}

// ============================================================================
// Static globals for light styles
// ============================================================================

/// Static array of light style entries, one per MAX_LIGHT_STYLES.
static mut cl_lightstyle: [clightstyle_t; MAX_LIGHT_STYLES] = [clightstyle_t {
    length: 0,
    value: [0; 4],
    map: [[0; 4]; MAX_QPATH],
}; MAX_LIGHT_STYLES];

/// Last computed offset for light style animation.
static mut lastofs: c_int = -1;

// ============================================================================
// Public functions
// ============================================================================

/*
================
FX_ClearLightStyles
================
*/
/// Clears all light styles and initializes them to defaults.
///
/// # Safety
/// This function modifies the static `cl_lightstyle` array and calls external functions.
pub unsafe extern "C" fn CG_ClearLightStyles() {
    memset(
        addr_of_mut!(cl_lightstyle) as *mut core::ffi::c_void,
        0,
        core::mem::size_of_val(&cl_lightstyle),
    );
    lastofs = -1;

    let mut i = 0;
    while i < MAX_LIGHT_STYLES as c_int * 3 {
        CG_SetLightstyle(i);
        i += 1;
    }
}

/*
================
FX_RunLightStyles
================
*/
/// Updates light styles based on the current time.
///
/// Cycles through light styles and sets their current color values based on
/// the time offset and the light style map. Sets the renderer's light style colors.
///
/// # Safety
/// This function accesses the static `cl_lightstyle` array and the global `cg` state,
/// and calls external functions.
pub unsafe extern "C" fn CG_RunLightStyles() {
    let ofs = cg.time / 50;
    //	if (ofs == lastofs)
    //		return;
    lastofs = ofs;

    let mut i = 0;
    let mut ls = addr_of_mut!(cl_lightstyle[0]);
    while i < MAX_LIGHT_STYLES as c_int {
        if (*ls).length == 0 {
            (*ls).value[0] = 255;
            (*ls).value[1] = 255;
            (*ls).value[2] = 255;
            (*ls).value[3] = 255;
        } else if (*ls).length == 1 {
            (*ls).value[0] = (*ls).map[0][0];
            (*ls).value[1] = (*ls).map[0][1];
            (*ls).value[2] = (*ls).map[0][2];
            (*ls).value[3] = 255; //(*ls).map[0][3];
        } else {
            (*ls).value[0] = (*ls).map[(ofs % (*ls).length) as usize][0];
            (*ls).value[1] = (*ls).map[(ofs % (*ls).length) as usize][1];
            (*ls).value[2] = (*ls).map[(ofs % (*ls).length) as usize][2];
            (*ls).value[3] = 255; //(*ls).map[(ofs % (*ls).length) as usize][3];
        }
        trap_R_SetLightStyle(i, *((*ls).value.as_ptr() as *const c_int));
        i += 1;
        ls = ls.add(1);
    }
}

/// Parses a light style configuration string and updates the corresponding light style entry.
///
/// Light style strings use characters 'a' through 'z' to represent intensity levels.
///
/// # Safety
/// This function calls external functions and accesses the static `cl_lightstyle` array.
pub unsafe extern "C" fn CG_SetLightstyle(i: c_int) {
    let s = CG_ConfigString(i + CS_LIGHT_STYLES);
    let j = strlen(s);
    if j >= MAX_QPATH {
        Com_Error(ERR_DROP, b"svc_lightstyle length=%i\0".as_ptr(), j as c_int);
    }

    cl_lightstyle[(i / 3) as usize].length = j as c_int;
    let mut k = 0;
    while k < j {
        let intensity = (*s.add(k) as c_int - b'a' as c_int) as f32
            / (b'z' as c_int - b'a' as c_int) as f32
            * 255.0;
        cl_lightstyle[(i / 3) as usize].map[k][(i % 3) as usize] = intensity as byte;
        k += 1;
    }
}
