//! Mechanical port of `code/cgame/FX_DEMP2.cpp`.

// DEMP2 Weapon

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

    /// Copies vector from src to dst.
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);

    /// Allocates a new local entity from the pool.
    pub fn CG_AllocLocalEntity() -> *mut localEntity_t;

    /// Registers a shader by name.
    pub fn cgi_R_RegisterShader(name: *const u8) -> c_int;

    /// Registers a model by name.
    pub fn cgi_R_RegisterModel(name: *const u8) -> c_int;

    /// Memory set function.
    pub fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8;
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
    // Fields not needed for FX_DEMP2
}

/// Stub for refEntity_t: rendering entity.
#[repr(C)]
pub struct refEntity_t {
    pub renderfx: c_int,
    pub _pad0: [u8; 52], // Padding to customShader
    pub customShader: c_int,
    pub hModel: c_int,
    pub _pad1: [u8; 32], // Padding to origin
    pub origin: vec3_t,
    // ... rest of fields omitted
}

/// Stub for localEntity_t: local client entity.
#[repr(C)]
pub struct localEntity_t {
    pub leType: c_int,
    pub _pad0: [u8; 8],
    pub startTime: c_int,
    pub endTime: c_int,
    pub _pad1: [u8; 16],
    pub radius: f32,
    pub _pad2: [u8; 64], // Padding to refEntity
    pub refEntity: refEntity_t,
    pub _pad3: [u8; 40], // Padding to color
    pub color: [f32; 3],
    // ... rest of fields omitted
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
// FX Scheduler interface
// ============================================================================

extern "C" {
    /// Global effect scheduler instance (theFxScheduler C++ object).
    pub fn FX_PlayEffect(name: *const u8, origin: *const vec3_t, normal: *const vec3_t);
}

// Constants for local entity types
const LE_FADE_SCALE_MODEL: c_int = 2;

// Constants for render effects
const RF_VOLUMETRIC: c_int = 0x00020;

/*
---------------------------
FX_DEMP2_ProjectileThink
---------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_DEMP2_ProjectileThink(
    cent: *mut centity_t,
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

    //	theFxScheduler.PlayEffect( "demp2/shot", cent->lerpOrigin, forward );
    //	theFxScheduler.PlayEffect( "demp2/shot2", cent->lerpOrigin, forward );
    FX_PlayEffect(
        b"demp2/projectile\0".as_ptr(),
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
    );
}

/*
---------------------------
FX_DEMP2_HitWall
---------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_DEMP2_HitWall(origin: *const vec3_t, normal: *const vec3_t) {
    FX_PlayEffect(b"demp2/wall_impact\0".as_ptr(), origin, normal);
}

/*
---------------------------
FX_DEMP2_HitPlayer
---------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_DEMP2_HitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    FX_PlayEffect(b"demp2/flesh_impact\0".as_ptr(), origin, normal);
}

/*
---------------------------
FX_DEMP2_AltProjectileThink
---------------------------
*/

#[no_mangle]
pub unsafe extern "C" fn FX_DEMP2_AltProjectileThink(
    cent: *mut centity_t,
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
        b"demp2/projectile\0".as_ptr(),
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
    );
}

//---------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn FX_DEMP2_AltDetonate(org: *const vec3_t, size: f32) {
    let ex: *mut localEntity_t = CG_AllocLocalEntity();

    (*ex).leType = LE_FADE_SCALE_MODEL;
    memset(
        core::ptr::addr_of_mut!((*ex).refEntity) as *mut u8,
        0,
        core::mem::size_of::<refEntity_t>(),
    );

    (*ex).refEntity.renderfx |= RF_VOLUMETRIC;

    (*ex).startTime = cg.time;
    (*ex).endTime = (*ex).startTime + 1300;

    (*ex).radius = size;
    (*ex).refEntity.customShader = cgi_R_RegisterShader(b"gfx/effects/demp2shell\0".as_ptr());

    (*ex).refEntity.hModel = cgi_R_RegisterModel(b"models/items/sphere.md3\0".as_ptr());
    VectorCopy(org, core::ptr::addr_of_mut!((*ex).refEntity.origin));

    (*ex).color[0] = 255.0;
    (*ex).color[1] = 255.0;
    (*ex).color[2] = 255.0;
}
