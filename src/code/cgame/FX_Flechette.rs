//! Mechanical port of `code/cgame/FX_Flechette.cpp`.

// Golan Arms Flechette Weapon

// this line must stay at top so the whole PCH thing works...

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

use crate::code::game::q_shared_h::vec3_t;
use crate::codemp::game::q_shared_h::qboolean;

// ============================================================================
// External functions
// ============================================================================

extern "C" {
    /// Evaluate trajectory delta vector at a given time; stores result.
    pub fn EvaluateTrajectoryDelta(
        tr: *const trajectory_t,
        atTime: c_int,
        result: *mut vec3_t,
    );

    /// Normalize the input vector in-place and return magnitude of original vector.
    pub fn VectorNormalize(vec: *mut vec3_t) -> f32;

    /// Normalize the input vector and stores result; returns magnitude of original vector.
    pub fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> f32;
}

// ============================================================================
// Stubs for cgame types
// ============================================================================

/// Stub for trajectory_t: a trajectory with delta vector.
#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

/// Stub for entityState_t: minimal definition for necessary fields.
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub pos: trajectory_t,
    pub apos: trajectory_t,
    pub _pad0: [u8; 32], // Padding to angles
    pub angles: vec3_t,
    pub _pad1: [u8; 80], // Padding to modelScale
    pub modelScale: vec3_t,
    // ... rest of fields omitted
}

/// Stub for gentity_t: game entity.
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub _pad0: [u8; 512], // Padding to client
    pub client: *mut u8,
    pub _pad1: [u8; 16], // Padding to owner
    pub owner: *mut gentity_t,
    pub _pad2: [u8; 64], // Padding to rest
    // ... rest of fields omitted
}

/// Stub for centity_t: client entity with position and rendering state.
#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub _pad0: [u8; 8],
    pub lerpOrigin: vec3_t,
    pub _pad1: [u8; 44],
    pub gent: *mut gentity_t,
}

/// Stub for weaponInfo_s: weapon metadata.
#[repr(C)]
pub struct weaponInfo_s {
    // Fields not needed for FX_Flechette
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
// Global cgs state (cgame static)
// ============================================================================

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 344], // Padding to bowcasterShotEffect
    pub _bowcasterShotEffect: c_int,
    pub _bowcasterBounceEffect: c_int,
    pub _bowcasterImpactEffect: c_int,
    pub _pad1: [u8; 12], // Padding to flechetteShotEffect
    pub flechetteShotEffect: c_int,
    pub flechetteAltShotEffect: c_int,
    pub flechetteShotDeathEffect: c_int,
    pub flechetteFleshImpactEffect: c_int,
    pub flechetteRicochetEffect: c_int,
    // ... rest of fields omitted
}

/// Stub for cgMedia_t: media resources.
#[repr(C)]
pub struct cgMedia_t {
    pub _pad0: [u8; 5344],
    // ... rest of fields omitted
}

/// Stub for cgs_t struct (client game static state).
#[repr(C)]
pub struct cgs_t {
    pub _pad0: [u8; 9424], // Padding to media
    pub media: cgMedia_t,
    pub effects: cgEffects_t,
}

extern "C" {
    /// Global client game static state, declared in cg_main.cpp.
    pub static cgs: cgs_t;
}

// ============================================================================
// External C++ object and member function for theFxScheduler
// ============================================================================

/// Opaque struct for CFxScheduler C++ class.
#[repr(C)]
pub struct CFxScheduler {
    _unused: [u8; 0],
}

extern "C" {
    /// Global instance of the effect scheduler (defined in FxScheduler.cpp).
    /// This is a C++ global object; accessing its methods requires proper C++ name mangling
    /// or linking against a C wrapper function.
    pub static theFxScheduler: CFxScheduler;
}

extern "C" {
    /// Play an effect by ID at the given origin with a forward direction vector.
    /// This corresponds to CFxScheduler::PlayEffect(int id, vec3_t org, vec3_t fwd, bool isPortal=false).
    /// Uses the GCC/Clang C++ name mangling for the method signature.
    /// On MSVC, the link_name would differ and may need adjustment.
    #[link_name = "_ZN11CFxScheduler9PlayEffectEiA3_fS1_b"]
    pub fn CFxScheduler_PlayEffect_v1(
        this: *const CFxScheduler,
        id: c_int,
        origin: *const vec3_t,
        forward: *const vec3_t,
        isPortal: c_int,
    );
}

// ============================================================================
// Public functions
// ============================================================================

/*
-------------------------
FX_FlechetteProjectileThink
-------------------------
*/

/// Project the Flechette projectile effect along its trajectory.
///
/// # Safety
/// `cent` and `weapon` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_FlechetteProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    EvaluateTrajectoryDelta(
        core::ptr::addr_of!((*(*cent).gent).s.pos),
        cg.time,
        core::ptr::addr_of_mut!(forward),
    );

    if VectorNormalize(core::ptr::addr_of_mut!(forward)) == 0.0f32 {
        forward[2] = 1.0f32;
    }

    CFxScheduler_PlayEffect_v1(
        core::ptr::addr_of!(theFxScheduler),
        cgs.effects.flechetteShotEffect,
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
        0, // isPortal = false
    );
}

/*
-------------------------
FX_FlechetteWeaponHitWall
-------------------------
*/

/// Play the Flechette wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_FlechetteWeaponHitWall(
    origin: *const vec3_t,
    normal: *const vec3_t,
) {
    CFxScheduler_PlayEffect_v1(
        core::ptr::addr_of!(theFxScheduler),
        cgs.effects.flechetteShotDeathEffect,
        origin,
        normal,
        0, // isPortal = false
    );
}

/*
-------------------------
FX_BlasterWeaponHitPlayer
-------------------------
*/

/// Play the Flechette player impact effect.
///
/// # Safety
/// `origin`, `normal` must be valid vec3_t pointers, and `humanoid` is a qboolean flag.
#[no_mangle]
pub unsafe extern "C" fn FX_FlechetteWeaponHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: qboolean,
) {
    // if ( humanoid )
    // {
    CFxScheduler_PlayEffect_v1(
        core::ptr::addr_of!(theFxScheduler),
        cgs.effects.flechetteFleshImpactEffect,
        origin,
        normal,
        0, // isPortal = false
    );
    // }
    // else
    // {
    //     CFxScheduler_PlayEffect(..., "blaster/droid_impact", ...);
    // }
}

/*
-------------------------
FX_FlechetteProjectileThink
-------------------------
*/

/// Play the Flechette alt fire projectile effect.
///
/// # Safety
/// `cent` and `weapon` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_FlechetteAltProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(
        core::ptr::addr_of!((*cent).currentState.pos.trDelta),
        core::ptr::addr_of_mut!(forward),
    ) == 0.0f32
    {
        forward[2] = 1.0f32;
    }

    CFxScheduler_PlayEffect_v1(
        core::ptr::addr_of!(theFxScheduler),
        cgs.effects.flechetteAltShotEffect,
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
        0, // isPortal = false
    );
}
