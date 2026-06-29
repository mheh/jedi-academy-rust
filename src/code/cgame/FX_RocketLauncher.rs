//! Mechanical port of `code/cgame/FX_RocketLauncher.cpp`.

// Rocket Launcher Weapon

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

use crate::code::game::q_shared_h::vec3_t;

// ============================================================================
// External functions
// ============================================================================

extern "C" {
    /// Normalized the input vector and stores result; returns magnitude of original vector.
    pub fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> f32;

    /// Global effect scheduler wrapper for string-based effect names.
    pub fn FX_PlayEffect(name: *const u8, origin: *const vec3_t, normal: *const vec3_t);
}

// ============================================================================
// Stubs for cgame types
// ============================================================================

/// Stub for centity_t: client entity with position and rendering state.
#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    // Padding to lerpOrigin
    pub _pad0: [u8; 8],
    pub lerpOrigin: vec3_t,
    pub _pad1: [u8; 44],
    pub gent: *mut gentity_t,
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

/// Stub for gentity_t: game entity.
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    // ... rest of fields omitted
}

/// Stub for weaponInfo_s: weapon metadata.
#[repr(C)]
pub struct weaponInfo_s {
    // Fields not needed for FX_RocketLauncher
}

// ============================================================================
// Public functions
// ============================================================================

/*
---------------------------
FX_RocketProjectileThink
---------------------------
*/

/// Plays the rocket projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
#[no_mangle]
pub unsafe extern "C" fn FX_RocketProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
        forward[2] = 1.0f32;
    }

    FX_PlayEffect(
        b"rocket/shot\0".as_ptr(),
        &(*cent).lerpOrigin,
        &forward,
    );
}

/*
---------------------------
FX_RocketHitWall
---------------------------
*/

/// Plays the rocket wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_RocketHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    FX_PlayEffect(b"rocket/explosion\0".as_ptr(), origin, normal);
}

/*
---------------------------
FX_RocketHitPlayer
---------------------------
*/

/// Plays the rocket player impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_RocketHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    FX_PlayEffect(b"rocket/explosion\0".as_ptr(), origin, normal);
}

/*
---------------------------
FX_RocketAltProjectileThink
---------------------------
*/

/// Plays the rocket alternate projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
#[no_mangle]
pub unsafe extern "C" fn FX_RocketAltProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
        forward[2] = 1.0f32;
    }

    FX_PlayEffect(
        b"rocket/shot\0".as_ptr(),
        &(*cent).lerpOrigin,
        &forward,
    );
}
