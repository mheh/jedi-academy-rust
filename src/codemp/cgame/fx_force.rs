//! Mechanical port of `codemp/cgame/fx_force.c`.
//!
//! Any dedicated force oriented effects

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_int;
use crate::codemp::game::q_shared_h::vec3_t;

// ============================================================================
// External functions and globals
// ============================================================================

extern "C" {
    /// Declared in cg_local.h: plays an effect by ID.
    pub fn trap_FX_PlayEffectID(
        fxHandle: c_int,
        origin: *const vec3_t,
        forward: *const vec3_t,
        dontKill: c_int,
        unk: c_int,
    );
}

// ============================================================================
// Stubs for cgame types
// ============================================================================

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 5772], // Padding to reach forceDrained effect
    pub forceDrained: c_int,
    // ... rest omitted
}

/// Stub for cgs_t struct (client game static state).
#[repr(C)]
pub struct cgs_t {
    pub _pad0: [u8; 9632], // Padding to reach effects
    pub effects: cgEffects_t,
}

extern "C" {
    /// Global client game static state, declared in cg_main.c.
    pub static cgs: cgs_t;
}

// ============================================================================
// Public functions
// ============================================================================

/*
-------------------------
FX_ForceDrained
-------------------------
*/
// This effect is not generic because of possible enhancements
/// Plays the force drained effect.
///
/// # Safety
/// `origin` and `dir` must be valid vec3_t pointers. `dir` may be modified in-place.
pub unsafe extern "C" fn FX_ForceDrained(origin: *const vec3_t, dir: *mut vec3_t) {
    // VectorScale(dir, -1.0, dir)
    (*dir)[0] *= -1.0;
    (*dir)[1] *= -1.0;
    (*dir)[2] *= -1.0;

    trap_FX_PlayEffectID((*cgs).effects.forceDrained, origin, dir, -1, -1);
}
