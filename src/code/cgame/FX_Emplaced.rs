//! Mechanical port of `code/cgame/FX_Emplaced.cpp`.

// Emplaced Weapon

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
    /// Normalize the input vector and stores result; returns magnitude of original vector.
    pub fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> f32;

    /// Scale input vector by scalar and stores result.
    pub fn VectorScale(i: *const vec3_t, scale: f32, o: *mut vec3_t);

    /// Global effect scheduler instance (theFxScheduler C++ object).
    /// This is a wrapper to CFxScheduler::PlayEffect for string-based effect names.
    pub fn FX_PlayEffect(name: *const u8, origin: *const vec3_t, normal: *const vec3_t);
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
    pub _pad1: [u8; 16], // Padding to weapon
    pub weapon: c_int,
    pub _pad2: [u8; 64], // Padding to rest
}

/// Stub for gentity_t: game entity.
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub _pad0: [u8; 512], // Padding to client
    pub client: *mut u8,
    pub _pad1: [u8; 16], // Padding to owner
    pub owner: *mut gentity_t,
    pub _pad2: [u8; 8],  // Padding to activator
    pub activator: *mut gentity_t,
    pub _pad3: [u8; 4],  // Padding to alt_fire
    pub alt_fire: c_int,
    pub _pad4: [u8; 48], // Padding to rest
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
    // Fields not needed for FX_Emplaced
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
// Weapon type constants
// ============================================================================

const WP_TIE_FIGHTER: c_int = 5; // Approximate weapon type for tie-fighter

/*
---------------------------
FX_EmplacedProjectileThink
---------------------------
*/

/// Project the Emplaced weapon projectile effect along its trajectory.
///
/// # Safety
/// `cent` and `weapon` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_EmplacedProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(
        core::ptr::addr_of!((*(*cent).gent).s.pos.trDelta),
        core::ptr::addr_of_mut!(forward),
    ) == 0.0f32
    {
        if VectorNormalize2(
            core::ptr::addr_of!((*cent).currentState.pos.trDelta),
            core::ptr::addr_of_mut!(forward),
        ) == 0.0f32
        {
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

        VectorScale(
            core::ptr::addr_of!(forward),
            scale,
            core::ptr::addr_of_mut!(forward),
        );
    }

    // If tie-fighter missle use green shot.
    if (*cent).currentState.weapon == WP_TIE_FIGHTER {
        FX_PlayEffect(
            b"ships/imp_blastershot\0".as_ptr(),
            core::ptr::addr_of!((*cent).lerpOrigin),
            core::ptr::addr_of!(forward),
        );
    } else {
        if !(*cent).gent.is_null()
            && !(*(*cent).gent).owner.is_null()
            && !(*(*(*cent).gent).owner).activator.is_null()
            && (*(*(*(*cent).gent).owner).activator).s.number > 0
        {
            // NPC's do short shot
            if (*(*cent).gent).alt_fire != 0 {
                FX_PlayEffect(
                    b"eweb/shotNPC\0".as_ptr(),
                    core::ptr::addr_of!((*cent).lerpOrigin),
                    core::ptr::addr_of!(forward),
                );
            } else {
                FX_PlayEffect(
                    b"emplaced/shotNPC\0".as_ptr(),
                    core::ptr::addr_of!((*cent).lerpOrigin),
                    core::ptr::addr_of!(forward),
                );
            }
        } else {
            // players do long shot
            if !(*cent).gent.is_null() && (*(*cent).gent).alt_fire != 0 {
                FX_PlayEffect(
                    b"eweb/shotNPC\0".as_ptr(),
                    core::ptr::addr_of!((*cent).lerpOrigin),
                    core::ptr::addr_of!(forward),
                );
            } else {
                FX_PlayEffect(
                    b"emplaced/shot\0".as_ptr(),
                    core::ptr::addr_of!((*cent).lerpOrigin),
                    core::ptr::addr_of!(forward),
                );
            }
        }
    }
}

/*
---------------------------
FX_EmplacedHitWall
---------------------------
*/

/// Play the Emplaced weapon wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_EmplacedHitWall(
    origin: *const vec3_t,
    normal: *const vec3_t,
    eweb: qboolean,
) {
    if eweb != 0 {
        FX_PlayEffect(b"eweb/wall_impact\0".as_ptr(), origin, normal);
    } else {
        FX_PlayEffect(b"emplaced/wall_impact\0".as_ptr(), origin, normal);
    }
}

/*
---------------------------
FX_EmplacedHitPlayer
---------------------------
*/

/// Play the Emplaced weapon player impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_EmplacedHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    eweb: qboolean,
) {
    if eweb != 0 {
        FX_PlayEffect(b"eweb/flesh_impact\0".as_ptr(), origin, normal);
    } else {
        FX_PlayEffect(b"emplaced/wall_impact\0".as_ptr(), origin, normal);
    }
}

/*
---------------------------
FX_TurretProjectileThink
---------------------------
*/

/// Project the Turret projectile effect along its trajectory.
///
/// # Safety
/// `cent` and `weapon` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_TurretProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(
        core::ptr::addr_of!((*(*cent).gent).s.pos.trDelta),
        core::ptr::addr_of_mut!(forward),
    ) == 0.0f32
    {
        if VectorNormalize2(
            core::ptr::addr_of!((*cent).currentState.pos.trDelta),
            core::ptr::addr_of_mut!(forward),
        ) == 0.0f32
        {
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

        VectorScale(
            core::ptr::addr_of!(forward),
            scale,
            core::ptr::addr_of_mut!(forward),
        );
    }

    FX_PlayEffect(
        b"turret/shot\0".as_ptr(),
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
    );
}
