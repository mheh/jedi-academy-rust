//! Mechanical port of `code/cgame/FX_ATSTMain.cpp`.

// Bowcaster Weapon

// this line must stay at top so the whole PCH thing works...

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

    /// Scales input vector by scalar and stores result.
    pub fn VectorScale(i: *const vec3_t, scale: f32, o: *mut vec3_t);

    /// Global effect scheduler instance (theFxScheduler C++ object).
    /// This is a wrapper to CFxScheduler::PlayEffect for string-based effect names.
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
    // Fields not needed for FX_ATSTMain
}

// ============================================================================
// Global client game state
// ============================================================================

/// Stub for cg_t struct to allow access to cg.time.
#[repr(C)]
pub struct cg_t {
    pub _pad: [u8; 316],
    pub time: c_int,
}

extern "C" {
    /// Global client game state, declared in cg_main.cpp.
    pub static cg: cg_t;
}

// ============================================================================
// Public functions
// ============================================================================

/*
---------------------------
FX_ATSTMainProjectileThink
---------------------------
*/

/// Plays the ATST main projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
#[no_mangle]
pub unsafe extern "C" fn FX_ATSTMainProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*(*cent).gent).s.pos.trDelta, &mut forward) == 0.0f32 {
        if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
            forward[2] = 1.0f32;
        }
    }

    // hack the scale of the forward vector if we were just fired or bounced...this will shorten up the tail for a split second so tails don't clip so harshly
    let mut dif = cg.time - (*(*cent).gent).s.pos.trTime;

    if dif < 30 {
        if dif < 0 {
            dif = 0;
        }

        let scale = (dif as f32 / 30.0f32) * 0.95f32 + 0.05f32;

        VectorScale(&forward, scale, &mut forward);
    }

    FX_PlayEffect(
        b"atst/shot\0".as_ptr(),
        &(*cent).lerpOrigin,
        &forward,
    );
}

/*
---------------------------
FX_ATSTMainHitWall
---------------------------
*/

/// Plays the ATST main wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_ATSTMainHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    FX_PlayEffect(b"atst/wall_impact\0".as_ptr(), origin, normal);
}

/*
---------------------------
FX_ATSTMainHitPlayer
---------------------------
*/

/// Plays the ATST main player impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_ATSTMainHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    if humanoid != 0 {
        FX_PlayEffect(b"atst/flesh_impact\0".as_ptr(), origin, normal);
    } else {
        FX_PlayEffect(b"atst/droid_impact\0".as_ptr(), origin, normal);
    }
}

/*
---------------------------
FX_ATSTSideAltProjectileThink
---------------------------
*/

/// Plays the ATST side alt projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
#[no_mangle]
pub unsafe extern "C" fn FX_ATSTSideAltProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
        forward[2] = 1.0f32;
    }

    FX_PlayEffect(
        b"atst/side_alt_shot\0".as_ptr(),
        &(*cent).lerpOrigin,
        &forward,
    );
}

/*
---------------------------
FX_ATSTSideMainProjectileThink
---------------------------
*/

/// Plays the ATST side main projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
#[no_mangle]
pub unsafe extern "C" fn FX_ATSTSideMainProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
        forward[2] = 1.0f32;
    }

    FX_PlayEffect(
        b"atst/side_main_shot\0".as_ptr(),
        &(*cent).lerpOrigin,
        &forward,
    );
}
