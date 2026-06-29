//! Mechanical port of `code/cgame/FX_Concussion.cpp`.

// Concussion Rifle Weapon

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
    /// Normalize the input vector and stores result; returns magnitude of original vector.
    pub fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> f32;

    /// Scale vector by scalar and add to destination: dst = base + scale * dir.
    pub fn VectorMA(base: *const vec3_t, scale: f32, dir: *const vec3_t, dst: *mut vec3_t);

    /// Copy vector from src to dst.
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);

    /// Add two vectors: dst = a + b.
    pub fn VectorAdd(a: *const vec3_t, b: *const vec3_t, dst: *mut vec3_t);

    /// Registers a shader by name.
    pub fn cgi_R_RegisterShader(name: *const u8) -> c_int;

    /// Global effect scheduler instance (theFxScheduler C++ object).
    /// This is a wrapper to CFxScheduler::PlayEffect for string-based effect names.
    pub fn FX_PlayEffect(name: *const u8, origin: *const vec3_t, normal: *const vec3_t);

    /// Add a line effect to the scene.
    pub fn FX_AddLine(
        clientID: c_int,
        start: vec3_t,
        end: vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        killTime: c_int,
        shader: c_int,
        impactFX_id: c_int,
        flags: c_int,
        modelNum: c_int,
        boltNum: c_int,
    );

    /// Add a bezier curve effect to the scene.
    pub fn FX_AddBezier(
        start: vec3_t,
        end: vec3_t,
        control1: vec3_t,
        control1Vel: vec3_t,
        control2: vec3_t,
        control2Vel: vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        sRGB: vec3_t,
        eRGB: vec3_t,
        rgbParm: f32,
        killTime: c_int,
        shader: c_int,
        flags: c_int,
    );
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
    // ... rest of fields omitted
}

/// Stub for centity_t: client entity with position and rendering state.
#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub _pad0: [u8; 8],
    pub lerpOrigin: vec3_t,
    pub _pad1: [u8; 44],
    // ... rest of fields omitted
}

/// Stub for weaponInfo_s: weapon metadata.
#[repr(C)]
pub struct weaponInfo_s {
    // Fields not needed for FX_Concussion
}

// ============================================================================
// Constants
// ============================================================================

const FX_SIZE_LINEAR: c_int = 0x00000100;
const FX_ALPHA_LINEAR: c_int = 0x00000001;
const FX_ALPHA_WAVE: c_int = 0x00000008;

// Global constant for white color
pub static WHITE: vec3_t = [1.0f32, 1.0f32, 1.0f32];

// Global constant for origin (zero vector)
pub const vec3_origin: vec3_t = [0.0f32, 0.0f32, 0.0f32];

/*
---------------------------
FX_ConcProjectileThink
---------------------------
*/

/// Project the Concussion projectile effect along its trajectory.
///
/// # Safety
/// `cent` and `weapon` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_ConcProjectileThink(
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

    FX_PlayEffect(
        b"concussion/shot\0".as_ptr(),
        core::ptr::addr_of!((*cent).lerpOrigin),
        core::ptr::addr_of!(forward),
    );
}

/*
---------------------------
FX_ConcHitWall
---------------------------
*/

/// Play the Concussion wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_ConcHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    FX_PlayEffect(b"concussion/explosion\0".as_ptr(), origin, normal);
}

/*
---------------------------
FX_ConcHitPlayer
---------------------------
*/

/// Play the Concussion player impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_ConcHitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    FX_PlayEffect(b"concussion/explosion\0".as_ptr(), origin, normal);
}

/*
---------------------------
FX_ConcAltShot
---------------------------
*/

/// Add visual effects for the Concussion alt fire shot.
///
/// # Safety
/// `start` and `end` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_ConcAltShot(start: *const vec3_t, end: *const vec3_t) {
    // //"concussion/beam"
    FX_AddLine(
        -1,
        *start,
        *end,
        0.1f32,
        10.0f32,
        0.0f32,
        1.0f32,
        0.0f32,
        0.0f32,
        WHITE,
        WHITE,
        0.0f32,
        175,
        cgi_R_RegisterShader(b"gfx/effects/blueLine\0".as_ptr()),
        0,
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        -1,
        -1,
    );

    let BRIGHT: vec3_t = [0.75f32, 0.5f32, 1.0f32];

    // add some beef
    FX_AddLine(
        -1,
        *start,
        *end,
        0.1f32,
        7.0f32,
        0.0f32,
        1.0f32,
        0.0f32,
        0.0f32,
        BRIGHT,
        BRIGHT,
        0.0f32,
        150,
        cgi_R_RegisterShader(b"gfx/misc/whiteline2\0".as_ptr()),
        0,
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        -1,
        -1,
    );
}

/*
---------------------------
FX_ConcAltMiss
---------------------------
*/

/// Add visual effects for the Concussion alt fire miss.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_ConcAltMiss(origin: *const vec3_t, normal: *const vec3_t) {
    let mut pos: vec3_t = [0.0; 3];
    let mut c1: vec3_t = [0.0; 3];
    let mut c2: vec3_t = [0.0; 3];

    VectorMA(*origin, 4.0f32, *normal, core::ptr::addr_of_mut!(c1));
    VectorCopy(core::ptr::addr_of!(c1), core::ptr::addr_of_mut!(c2));
    c1[2] += 4.0f32;
    c2[2] += 12.0f32;

    VectorAdd(*origin, *normal, core::ptr::addr_of_mut!(pos));
    pos[2] += 28.0f32;

    FX_AddBezier(
        *origin,
        pos,
        c1,
        vec3_origin,
        c2,
        vec3_origin,
        6.0f32,
        6.0f32,
        0.0f32,
        0.0f32,
        0.2f32,
        0.5f32,
        WHITE,
        WHITE,
        0.0f32,
        4000,
        cgi_R_RegisterShader(b"gfx/effects/smokeTrail\0".as_ptr()),
        FX_ALPHA_WAVE,
    );

    FX_PlayEffect(b"concussion/alt_miss\0".as_ptr(), origin, normal);
}
