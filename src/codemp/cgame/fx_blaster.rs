//! Mechanical port of `codemp/cgame/fx_blaster.c`.
//!
//! Blaster Weapon

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
/// Full struct not needed for fx_blaster.c; only lerpOrigin and currentState.pos.trDelta required.
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
    // Fields not needed for fx_blaster.c
}

// ============================================================================
// Global cgs state (cgame static)
// ============================================================================

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 36], // Padding to reach blasterShotEffect
    pub blasterShotEffect: c_int,
    pub blasterWallImpactEffect: c_int,
    pub blasterFleshImpactEffect: c_int,
    pub blasterDroidImpactEffect: c_int,
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
FX_BlasterProjectileThink
-------------------------
*/
/// Projects the blaster projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
pub unsafe extern "C" fn FX_BlasterProjectileThink(cent: *const centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap_FX_PlayEffectID(
        (*cgs).effects.blasterShotEffect,
        &(*cent).lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/*
-------------------------
FX_BlasterAltFireThink
-------------------------
*/
/// Projects the blaster alt-fire projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
pub unsafe extern "C" fn FX_BlasterAltFireThink(cent: *const centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap_FX_PlayEffectID(
        (*cgs).effects.blasterShotEffect,
        &(*cent).lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/*
-------------------------
FX_BlasterWeaponHitWall
-------------------------
*/
/// Plays the blaster wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
pub unsafe extern "C" fn FX_BlasterWeaponHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    trap_FX_PlayEffectID(
        (*cgs).effects.blasterWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
-------------------------
FX_BlasterWeaponHitPlayer
-------------------------
*/
/// Plays the blaster flesh impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers. `humanoid` determines which effect is played.
pub unsafe extern "C" fn FX_BlasterWeaponHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: qboolean,
) {
    if humanoid != 0 {
        trap_FX_PlayEffectID(
            (*cgs).effects.blasterFleshImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    } else {
        trap_FX_PlayEffectID(
            (*cgs).effects.blasterDroidImpactEffect,
            origin,
            normal,
            -1,
            -1,
        );
    }
}
