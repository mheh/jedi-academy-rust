//! Mechanical port of `code/cgame/FX_HeavyRepeater.cpp`.

// Heavy Repeater Weapon

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

    /// Global effect scheduler instance (theFxScheduler C++ object).
    /// This is a wrapper to CFxScheduler::PlayEffect for string-based effect names.
    pub fn FX_PlayEffect(name: *const u8, origin: *const vec3_t, normal: *const vec3_t);

    /// Global effect scheduler instance (theFxScheduler C++ object).
    /// This is a wrapper to CFxScheduler::PlayEffect for string-based effect names with origin only.
    pub fn FX_PlayEffect_Origin(name: *const u8, origin: *const vec3_t);
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
    pub gent: *mut u8,
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
    // Fields not needed for FX_HeavyRepeater
}

/*
---------------------------
FX_RepeaterProjectileThink
---------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_RepeaterProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(
        core::ptr::addr_of!((*cent).currentState.pos.trDelta),
        core::ptr::addr_of_mut!(forward),
    ) == 0.0
    {
        forward[2] = 1.0;
    }

    FX_PlayEffect(
        b"repeater/projectile\0".as_ptr(),
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
    );
}

/*
------------------------
FX_RepeaterHitWall
------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_RepeaterHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    FX_PlayEffect(b"repeater/wall_impact\0".as_ptr(), origin, normal);
}

/*
------------------------
FX_RepeaterHitPlayer
------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_RepeaterHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    FX_PlayEffect(b"repeater/wall_impact\0".as_ptr(), origin, normal);
    //	theFxScheduler.PlayEffect( "repeater/flesh_impact", origin, normal );
}

/*
------------------------------
FX_RepeaterAltProjectileThink
-----------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_RepeaterAltProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(
        core::ptr::addr_of!((*cent).currentState.pos.trDelta),
        core::ptr::addr_of_mut!(forward),
    ) == 0.0
    {
        forward[2] = 1.0;
    }

    FX_PlayEffect(
        b"repeater/alt_projectile\0".as_ptr(),
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
    );
    //	theFxScheduler.PlayEffect( "repeater/alt_projectile", cent->lerpOrigin, forward );
}

/*
------------------------
FX_RepeaterAltHitWall
------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_RepeaterAltHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    FX_PlayEffect(b"repeater/concussion\0".as_ptr(), origin, normal);
    //	theFxScheduler.PlayEffect( "repeater/alt_wall_impact2", origin, normal );
}

/*
------------------------
FX_RepeaterAltHitPlayer
------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_RepeaterAltHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    FX_PlayEffect_Origin(b"repeater/concussion\0".as_ptr(), origin);
    //	theFxScheduler.PlayEffect( "repeater/alt_wall_impact2", origin, normal );
}
