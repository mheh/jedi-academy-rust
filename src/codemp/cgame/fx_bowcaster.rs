//! Mechanical port of `codemp/cgame/fx_bowcaster.c`.
//!
//! Bowcaster Weapon

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_int;
use crate::codemp::game::q_math::VectorNormalize2;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;

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

/// Stub for centity_t: client entity with position and rendering state.
/// Full struct not needed for fx_bowcaster.c; only lerpOrigin and currentState.pos.trDelta required.
#[repr(C)]
pub struct centity_t {
    pub lerpOrigin: vec3_t,
    pub currentState: entityState_t,
    // ... rest of fields omitted
}

/// Stub for entityState_t: minimal definition for pos field.
#[repr(C)]
pub struct entityState_t {
    pub pos: trajectory_t,
    // ... rest of fields omitted
}

/// Stub for trajectory_t: a trajectory with delta vector.
#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

/// Stub for weaponInfo_s: weapon metadata.
#[repr(C)]
pub struct weaponInfo_s {
    // Fields not needed for fx_bowcaster.c
}

// ============================================================================
// Global cgs state (cgame static)
// ============================================================================

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 76], // Padding to reach bowcasterShotEffect
    pub bowcasterShotEffect: c_int,
    pub bowcasterImpactEffect: c_int,
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
---------------------------
FX_BowcasterProjectileThink
---------------------------
*/
/// Projects the Bowcaster projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
pub unsafe extern "C" fn FX_BowcasterProjectileThink(cent: *const centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap_FX_PlayEffectID(
        (*cgs).effects.bowcasterShotEffect,
        &(*cent).lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/*
---------------------------
FX_BowcasterHitWall
---------------------------
*/
/// Plays the Bowcaster wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
pub unsafe extern "C" fn FX_BowcasterHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    trap_FX_PlayEffectID(
        (*cgs).effects.bowcasterImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
---------------------------
FX_BowcasterHitPlayer
---------------------------
*/
/// Plays the Bowcaster flesh impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers. `humanoid` is unused in the current
/// implementation (handled by effect choice on the C side).
pub unsafe extern "C" fn FX_BowcasterHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: qboolean,
) {
    trap_FX_PlayEffectID(
        (*cgs).effects.bowcasterImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
------------------------------
FX_BowcasterAltProjectileThink
------------------------------
*/
/// Projects the Bowcaster alt-fire projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
pub unsafe extern "C" fn FX_BowcasterAltProjectileThink(cent: *const centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap_FX_PlayEffectID(
        (*cgs).effects.bowcasterShotEffect,
        &(*cent).lerpOrigin,
        &forward,
        -1,
        -1,
    );
}
