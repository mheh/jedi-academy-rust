//! Mechanical port of `codemp/cgame/fx_heavyrepeater.c`.
//!
//! Heavy Repeater Weapon

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_int;
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;
use crate::codemp::game::q_math::{VectorNormalize2, VectorCopy, VectorSubtract, VectorLength, VectorNormalize, vectoangles, AnglesToAxis, VectorScale};
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

    /// Declared in cg_local.h: adds a ref entity to the scene for rendering.
    pub fn trap_R_AddRefEntityToScene(ent: *const refEntity_t);

    /// Standard C library: memory fill operation.
    pub fn memset(dest: *mut core::ffi::c_void, c: c_int, count: usize) -> *mut core::ffi::c_void;

    /// Global client game state, declared in cg_main.c.
    pub static mut cg: cg_t;

    /// Global client game static state, declared in cg_main.c.
    pub static mut cgs: cgs_t;

    /// Render to texture effects enabled.
    pub static cg_renderToTextureFX: c_int;

    /// Enable distortion orb for repeater alt fire.
    pub static cg_repeaterOrb: c_int;
}

// ============================================================================
// Stubs for cgame types
// ============================================================================

/// Stub for centity_t: client entity with position and rendering state.
/// Full struct not needed for fx_heavyrepeater.c; only lerpOrigin, currentState.pos.trDelta, and trickAlpha required.
#[repr(C)]
pub struct centity_t {
    pub lerpOrigin: vec3_t,
    pub currentState: entityState_t,
    pub _pad0: [u8; 272], // Padding to reach trickAlpha
    pub trickAlpha: f32,
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
    // Fields not needed for fx_heavyrepeater.c
}

/// Stub for refEntity_t: renderer entity with origin and shader state.
#[repr(C)]
pub struct refEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
    pub hModel: c_int,
    pub axis: [[f32; 3]; 3],
    pub nonNormalizedAxes: qboolean,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub customShader: c_int,
    pub shaderRGBA: [u8; 4],
    pub shaderTexCoord: [f32; 2],
    pub radius: f32,
    pub rotation: f32,
    pub shaderTime: f32,
    pub frame: c_int,
    // ... rest of fields omitted
}

/// Stub for refdef_t: defines the rendering view.
#[repr(C)]
pub struct refdef_t {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub fov_x: f32,
    pub fov_y: f32,
    pub vieworg: vec3_t,
    // ... rest of fields omitted
}

/// Stub for cg_t struct to allow access to refdef.
#[repr(C)]
pub struct cg_t {
    pub _pad: [u8; 48],
    pub refdef: refdef_t,
    // ... rest of fields omitted
}

/// Stub for cgMedia_t: media resources.
#[repr(C)]
pub struct cgMedia_t {
    pub _pad0: [u8; 5344], // Padding to reach halfShieldModel
    pub halfShieldModel: c_int,
    // ... rest omitted
}

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 5704], // Padding to reach repeater effects
    pub repeaterProjectileEffect: c_int,
    pub repeaterWallImpactEffect: c_int,
    pub repeaterFleshImpactEffect: c_int,
    pub repeaterAltProjectileEffect: c_int,
    pub repeaterAltWallImpactEffect: c_int,
    // ... rest omitted
}

/// Stub for cgs_t struct (client game static state).
#[repr(C)]
pub struct cgs_t {
    pub _pad0: [u8; 9424], // Padding to reach media
    pub media: cgMedia_t,
    pub effects: cgEffects_t,
}

// ============================================================================
// Constants
// ============================================================================

const ROLL: usize = 2;
const RF_DISTORTION: c_int = 0x00000040;
const RF_RGB_TINT: c_int = 0x00000200;

// ============================================================================
// Private functions
// ============================================================================

/*
------------------------------
CG_DistortionOrb
------------------------------
*/
unsafe fn CG_DistortionOrb(cent: *mut centity_t) {
    let mut ent = MaybeUninit::<refEntity_t>::zeroed().assume_init();
    let mut ang: vec3_t = [0.0; 3];
    let scale: f32 = 0.5f32;
    let mut vLen: f32;

    if cg_renderToTextureFX == 0 {
        return;
    }

    VectorCopy(&(*cent).lerpOrigin, &mut ent.origin);

    VectorSubtract(&ent.origin, &(*addr_of_mut!(cg)).refdef.vieworg, &mut ent.axis[0]);
    vLen = VectorLength(&ent.axis[0]);
    if VectorNormalize(&mut ent.axis[0]) <= 0.1f32 {
        // Entity is right on vieworg.  quit.
        return;
    }

    // VectorCopy(cg.refdef.viewaxis[2], ent.axis[2]);
    // CrossProduct(ent.axis[0], ent.axis[2], ent.axis[1]);
    vectoangles(&ent.axis[0], &mut ang);
    ang[ROLL] = (*cent).trickAlpha;
    (*cent).trickAlpha += 16.0; //spin the half-sphere to give a "screwdriver" effect
    AnglesToAxis(&ang, &mut ent.axis);

    //radius must be a power of 2, and is the actual captured texture size
    if vLen < 128.0f32 {
        ent.radius = 256.0;
    } else if vLen < 256.0f32 {
        ent.radius = 128.0;
    } else if vLen < 512.0f32 {
        ent.radius = 64.0;
    } else {
        ent.radius = 32.0;
    }

    VectorScale(&ent.axis[0], scale, &mut ent.axis[0]);
    VectorScale(&ent.axis[1], scale, &mut ent.axis[1]);
    VectorScale(&ent.axis[2], -scale, &mut ent.axis[2]);

    ent.hModel = (*addr_of_mut!(cgs)).media.halfShieldModel;
    ent.customShader = 0; //cgs.media.halfShieldShader;

    ent.renderfx = (RF_DISTORTION | RF_RGB_TINT);

    //tint the whole thing a shade of blue
    ent.shaderRGBA[0] = 200;
    ent.shaderRGBA[1] = 200;
    ent.shaderRGBA[2] = 255;

    trap_R_AddRefEntityToScene(&ent);
}

// ============================================================================
// Public functions
// ============================================================================

/*
---------------------------
FX_RepeaterProjectileThink
---------------------------
*/
/// Projects the repeater projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
pub unsafe extern "C" fn FX_RepeaterProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap_FX_PlayEffectID(
        (*addr_of_mut!(cgs)).effects.repeaterProjectileEffect,
        &(*cent).lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/*
------------------------
FX_RepeaterHitWall
------------------------
*/
/// Plays the repeater wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
pub unsafe extern "C" fn FX_RepeaterHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    trap_FX_PlayEffectID(
        (*addr_of_mut!(cgs)).effects.repeaterWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
------------------------
FX_RepeaterHitPlayer
------------------------
*/
/// Plays the repeater flesh impact effect.
///
/// # Safety
/// `origin`, `normal` must be valid vec3_t pointers. `humanoid` parameter is unused.
pub unsafe extern "C" fn FX_RepeaterHitPlayer(origin: *const vec3_t, normal: *const vec3_t, humanoid: qboolean) {
    trap_FX_PlayEffectID(
        (*addr_of_mut!(cgs)).effects.repeaterFleshImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
------------------------------
FX_RepeaterAltProjectileThink
-----------------------------
*/
/// Projects the repeater alt-fire projectile effect along its trajectory, optionally rendering a distortion orb.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
pub unsafe extern "C" fn FX_RepeaterAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    if cg_repeaterOrb != 0 {
        CG_DistortionOrb(cent);
    }
    trap_FX_PlayEffectID(
        (*addr_of_mut!(cgs)).effects.repeaterAltProjectileEffect,
        &(*cent).lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/*
------------------------
FX_RepeaterAltHitWall
------------------------
*/
/// Plays the repeater alt-fire wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
pub unsafe extern "C" fn FX_RepeaterAltHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    trap_FX_PlayEffectID(
        (*addr_of_mut!(cgs)).effects.repeaterAltWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
------------------------
FX_RepeaterAltHitPlayer
------------------------
*/
/// Plays the repeater alt-fire flesh impact effect.
///
/// # Safety
/// `origin`, `normal` must be valid vec3_t pointers. `humanoid` parameter is unused.
pub unsafe extern "C" fn FX_RepeaterAltHitPlayer(origin: *const vec3_t, normal: *const vec3_t, humanoid: qboolean) {
    trap_FX_PlayEffectID(
        (*addr_of_mut!(cgs)).effects.repeaterAltWallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}
