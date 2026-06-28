//! Mechanical port of `code/cgame/FX_Bowcaster.cpp`.

// Bowcaster Weapon

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

use crate::code::game::q_shared_h::vec3_t;

// ============================================================================
// External functions and types
// ============================================================================

extern "C" {
    /// Normalized the input vector and stores result; returns magnitude of original vector.
    pub fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> f32;

    /// Scales input vector by scalar and stores result.
    pub fn VectorScale(i: *const vec3_t, scale: f32, o: *mut vec3_t);
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
    // Fields not needed for FX_Bowcaster
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
    pub bowcasterShotEffect: c_int,
    pub bowcasterBounceEffect: c_int,
    pub bowcasterImpactEffect: c_int,
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
---------------------------
FX_BowcasterProjectileThink
---------------------------
*/

/// Projects the Bowcaster projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
#[no_mangle]
pub unsafe extern "C" fn FX_BowcasterProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*(*cent).gent).s.pos.trDelta, &mut forward) == 0.0f32
    {
        if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0f32 {
            forward[2] = 1.0f32;
        }
    }

    // hack the scale of the forward vector if we were just fired or bounced...this will shorten up the tail for a split second so tails don't clip so harshly
    let mut dif = cg.time - (*(*cent).gent).s.pos.trTime;

    if dif < 75 {
        if dif < 0 {
            dif = 0;
        }

        let scale = (dif as f32 / 75.0f32) * 0.95f32 + 0.05f32;

        VectorScale(&forward, scale, &mut forward);
    }

    CFxScheduler_PlayEffect_v1(
        &theFxScheduler,
        cgs.effects.bowcasterShotEffect,
        &(*cent).lerpOrigin,
        &forward,
        0, // isPortal = false
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
#[no_mangle]
pub unsafe extern "C" fn FX_BowcasterHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    CFxScheduler_PlayEffect_v1(
        &theFxScheduler,
        cgs.effects.bowcasterImpactEffect,
        origin,
        normal,
        0, // isPortal = false
    );
}

/*
---------------------------
FX_BowcasterHitPlayer
---------------------------
*/

/// Plays the Bowcaster player impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_BowcasterHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    CFxScheduler_PlayEffect_v1(
        &theFxScheduler,
        cgs.effects.bowcasterImpactEffect,
        origin,
        normal,
        0, // isPortal = false
    );
}
