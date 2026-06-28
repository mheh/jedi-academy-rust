//! Mechanical port of `codemp/cgame/fx_demp2.c`.
//!
//! DEMP2 weapon effects: projectile, impact, and detonation visual effects.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::c_int;
use core::ptr::addr_of_mut;
use crate::codemp::game::q_math::{VectorNormalize2};
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

    /// Declared in cg_local.h: allocates a local entity.
    pub fn CG_AllocLocalEntity() -> *mut localEntity_t;

    /// Standard C library: memory fill operation.
    pub fn memset(dest: *mut core::ffi::c_void, c: c_int, count: usize) -> *mut core::ffi::c_void;
}

// ============================================================================
// Stubs for cgame types
// ============================================================================

/// Stub for centity_t: client entity with position and rendering state.
/// Full struct not needed for fx_demp2.c; only lerpOrigin and currentState.pos.trDelta required.
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
    // Fields not needed for fx_demp2.c
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
    // Fields below are full refEntity_t extensions
    pub lightingOrigin: vec3_t,
    pub shadowPlane: f32,
    pub oldframe: c_int,
    pub backlerp: f32,
    pub skinNum: c_int,
    pub customSkin: c_int,
    pub uRefEnt: refEntity_uRefEnt_u,
    pub data: refEntity_data_u,
    pub endTime: f32,
    pub saberLength: f32,
    pub angles: vec3_t,
    pub modelScale: vec3_t,
    pub ghoul2: *mut core::ffi::c_void,
}

#[repr(C)]
pub union refEntity_uRefEnt_u {
    pub uMini: refEntity_uMini,
}

#[repr(C)]
pub struct refEntity_uMini {
    pub miniStart: c_int,
    pub miniCount: c_int,
}

#[repr(C)]
pub union refEntity_data_u {
    pub sprite: refEntity_sprite,
    pub line: refEntity_line,
    pub bezier: refEntity_bezier,
    pub cylinder: refEntity_cylinder,
    pub electricity: refEntity_electricity,
}

#[repr(C)]
pub struct refEntity_sprite {
    pub rotation: f32,
    pub radius: f32,
    pub vertRGBA: [[u8; 4]; 4],
}

#[repr(C)]
pub struct refEntity_line {
    pub width: f32,
    pub width2: f32,
    pub stscale: f32,
}

#[repr(C)]
pub struct refEntity_bezier {
    pub width: f32,
    pub control1: vec3_t,
    pub control2: vec3_t,
}

#[repr(C)]
pub struct refEntity_cylinder {
    pub width: f32,
    pub width2: f32,
    pub stscale: f32,
    pub height: f32,
    pub bias: f32,
    pub wrap: qboolean,
}

#[repr(C)]
pub struct refEntity_electricity {
    pub width: f32,
    pub deviation: f32,
    pub stscale: f32,
    pub wrap: qboolean,
    pub taper: qboolean,
}

/// Stub for localEntity_t: client-side effects entity.
#[repr(C)]
pub struct localEntity_t {
    pub prev: *mut localEntity_t,
    pub next: *mut localEntity_t,
    pub leType: c_int,
    pub leFlags: c_int,
    pub startTime: c_int,
    pub endTime: c_int,
    pub fadeInTime: c_int,
    pub lifeRate: f32,
    pub pos: trajectory_t,
    pub angles: trajectory_t,
    pub bounceFactor: f32,
    pub bounceSound: c_int,
    pub alpha: f32,
    pub dalpha: f32,
    pub forceAlpha: c_int,
    pub color: [f32; 4],
    pub radius: f32,
    pub light: f32,
    pub lightColor: vec3_t,
    pub leMarkType: c_int,
    pub leBounceSoundType: c_int,
    pub data: localEntity_data_u,
    pub refEntity: refEntity_t,
}

#[repr(C)]
pub union localEntity_data_u {
    pub sprite: localEntity_sprite,
    pub trail: localEntity_trail,
    pub line: localEntity_line,
    pub line2: localEntity_line2,
    pub cylinder: localEntity_cylinder,
    pub electricity: localEntity_electricity,
    pub particle: localEntity_particle,
    pub spawner: localEntity_spawner,
    pub fragment: localEntity_fragment,
}

#[repr(C)]
pub struct localEntity_sprite {
    pub radius: f32,
    pub dradius: f32,
    pub startRGB: vec3_t,
    pub dRGB: vec3_t,
}

#[repr(C)]
pub struct localEntity_trail {
    pub width: f32,
    pub dwidth: f32,
    pub length: f32,
    pub dlength: f32,
    pub startRGB: vec3_t,
    pub dRGB: vec3_t,
}

#[repr(C)]
pub struct localEntity_line {
    pub width: f32,
    pub dwidth: f32,
    pub control1: vec3_t,
    pub control2: vec3_t,
    pub control1_velocity: vec3_t,
    pub control2_velocity: vec3_t,
    pub control1_acceleration: vec3_t,
    pub control2_acceleration: vec3_t,
}

#[repr(C)]
pub struct localEntity_line2 {
    pub width: f32,
    pub dwidth: f32,
    pub width2: f32,
    pub dwidth2: f32,
    pub startRGB: vec3_t,
    pub dRGB: vec3_t,
}

#[repr(C)]
pub struct localEntity_cylinder {
    pub width: f32,
    pub dwidth: f32,
    pub width2: f32,
    pub dwidth2: f32,
    pub height: f32,
    pub dheight: f32,
}

#[repr(C)]
pub struct localEntity_electricity {
    pub width: f32,
    pub dwidth: f32,
}

#[repr(C)]
pub struct localEntity_particle {
    pub radius: f32,
    pub dradius: f32,
    pub thinkFn: Option<unsafe extern "C" fn(*mut localEntity_t) -> qboolean>,
    pub dir: vec3_t,
}

#[repr(C)]
pub struct localEntity_spawner {
    pub dontDie: qboolean,
    pub dir: vec3_t,
    pub variance: f32,
    pub delay: c_int,
    pub nextthink: c_int,
    pub thinkFn: Option<unsafe extern "C" fn(*mut localEntity_t) -> qboolean>,
    pub data1: c_int,
    pub data2: c_int,
}

#[repr(C)]
pub struct localEntity_fragment {
    pub radius: f32,
}

// ============================================================================
// Global cg state
// ============================================================================

/// Stub for cg_t struct to allow access to cg.time.
#[repr(C)]
pub struct cg_t {
    pub _pad: [u8; 48],
    pub time: c_int,
}

extern "C" {
    /// Global client game state, declared in cg_main.c.
    pub static cg: cg_t;
}

// ============================================================================
// Global cgs state (cgame static)
// ============================================================================

/// Stub for cgMedia_t: media resources.
#[repr(C)]
pub struct cgMedia_t {
    pub _pad0: [u8; 5344], // Padding to reach demp2Shell
    pub demp2Shell: c_int,
    pub demp2ShellShader: c_int,
    // ... rest omitted
}

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 5704], // Padding to reach demp2 effects
    pub demp2ProjectileEffect: c_int,
    pub demp2WallImpactEffect: c_int,
    pub demp2FleshImpactEffect: c_int,
    // ... rest omitted
}

/// Stub for cgs_t struct (client game static state).
#[repr(C)]
pub struct cgs_t {
    pub _pad0: [u8; 9424], // Padding to reach media
    pub media: cgMedia_t,
    pub effects: cgEffects_t,
}

extern "C" {
    /// Global client game static state, declared in cg_main.c.
    pub static cgs: cgs_t;
}

// ============================================================================
// Constants
// ============================================================================

pub const LE_FADE_SCALE_MODEL: c_int = 3; // currently only for Demp2 shock sphere
pub const RF_VOLUMETRIC: c_int = 0x00020; // fake volumetric shading

// ============================================================================
// Public functions
// ============================================================================

/*
---------------------------
FX_DEMP2_ProjectileThink
---------------------------
*/
/// Projects the DEMP2 projectile effect along its trajectory.
///
/// # Safety
/// `cent` must be valid; `weapon` may be unused.
pub unsafe extern "C" fn FX_DEMP2_ProjectileThink(cent: *const centity_t, weapon: *const weaponInfo_s) {
    let mut forward: vec3_t = [0.0; 3];

    if VectorNormalize2(&(*cent).currentState.pos.trDelta, &mut forward) == 0.0 {
        forward[2] = 1.0;
    }

    trap_FX_PlayEffectID(
        (*cgs).effects.demp2ProjectileEffect,
        &(*cent).lerpOrigin,
        &forward,
        -1,
        -1,
    );
}

/*
---------------------------
FX_DEMP2_HitWall
---------------------------
*/
/// Plays the DEMP2 wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
pub unsafe extern "C" fn FX_DEMP2_HitWall(origin: *const vec3_t, normal: *const vec3_t) {
    trap_FX_PlayEffectID(
        (*cgs).effects.demp2WallImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
---------------------------
FX_DEMP2_HitPlayer
---------------------------
*/
/// Plays the DEMP2 flesh impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers. `humanoid` is unused in the current
/// implementation (handled by effect choice on the C side).
pub unsafe extern "C" fn FX_DEMP2_HitPlayer(
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: qboolean,
) {
    trap_FX_PlayEffectID(
        (*cgs).effects.demp2FleshImpactEffect,
        origin,
        normal,
        -1,
        -1,
    );
}

/*
---------------------------
FX_DEMP2_AltBeam
---------------------------
*/
/// Renders the DEMP2 alt-fire beam effect (currently unimplemented stub).
///
/// The original C code has this function extensively commented out (NOTENOTE: "Fix this after
/// trap calls for all primitives are created"). The body is a placeholder that does nothing.
///
/// # Safety
/// `start`, `end`, `normal`, `targ1`, `targ2` must be valid vec3_t pointers.
pub unsafe extern "C" fn FX_DEMP2_AltBeam(
    start: *const vec3_t,
    end: *const vec3_t,
    normal: *const vec3_t,
    targ1: *const vec3_t,
    targ2: *const vec3_t,
) {
    // NOTENOTE Fix this after trap calls for all primitives are created.
    // The original C implementation of this function is entirely commented out:
    // it would compute Bezier curves and lightning-like visual effects between
    // start, targ1, and targ2 points. This is a stub until the underlying
    // FX system primitives (FX_AddBezier, FX_AddSprite, trap_R_RegisterShader)
    // are ported.
}

//---------------------------------------------

/*
---------------------------
FX_DEMP2_AltDetonate
---------------------------
*/
/// Creates a fading shock sphere at the detonation point.
///
/// Allocates a local entity of type LE_FADE_SCALE_MODEL with the DEMP2 shell model
/// and shader, fading over 800ms from full opacity.
///
/// # Safety
/// `org` must be a valid vec3_t pointer.
pub unsafe extern "C" fn FX_DEMP2_AltDetonate(org: *const vec3_t, size: f32) {
    let ex: *mut localEntity_t;

    ex = CG_AllocLocalEntity();
    (*ex).leType = LE_FADE_SCALE_MODEL;
    memset(
        addr_of_mut!((*ex).refEntity) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<refEntity_t>(),
    );

    (*ex).refEntity.renderfx |= RF_VOLUMETRIC;

    (*ex).startTime = (*cg).time;
    (*ex).endTime = (*ex).startTime + 800; //1600;

    (*ex).radius = size;
    (*ex).refEntity.customShader = (*cgs).media.demp2ShellShader;
    (*ex).refEntity.hModel = (*cgs).media.demp2Shell;
    (*ex).refEntity.origin = *org;

    (*ex).color[0] = 255.0;
    (*ex).color[1] = 255.0;
    (*ex).color[2] = 255.0;
}
