//! Mechanical port of `code/cgame/FxUtil.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

use crate::code::qcommon::q_shared_h::{qhandle_t, vec2_t, vec3_t};

// Opaque C/C++ dependencies from `FxPrimitives.h`. This header
// only passes pointers to these types.
#[repr(C)]
pub struct CParticle {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CLine {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CElectricity {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CTail {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CCylinder {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CEmitter {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CLight {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct COrientedParticle {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CPoly {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CFlash {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct CBezier {
    _unused: [u8; 0],
}

extern "C" {
    // ditches all active effects;
    pub fn FX_Free() -> bool;
    // called in CG_Init to purge the fx system.
    pub fn FX_Init() -> c_int;
    // called every cgame frame to add all fx into the scene.
    pub fn FX_Add(portal: bool);
    // ditches all active effects without touching the templates.
    pub fn FX_Stop();

    // returns whether there are any active or scheduled effects
    pub fn FX_ActiveFx() -> bool;

    pub fn FX_AddParticle(
        clientID: c_int,
        org: vec3_t,
        vel: vec3_t,
        accel: vec3_t,
        gravity: f32,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        rotation: f32,
        rotationDelta: f32,
        min: vec3_t,
        max: vec3_t,
        elasticity: f32,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CParticle;

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
        shader: qhandle_t,
        impactFX_id: c_int,
        flags: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CLine;

    pub fn FX_AddElectricity(
        clientID: c_int,
        start: vec3_t,
        end: vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        sRGB: vec3_t,
        eRGB: vec3_t,
        rgbParm: f32,
        chaos: f32,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CElectricity;

    pub fn FX_AddTail(
        clientID: c_int,
        org: vec3_t,
        vel: vec3_t,
        accel: vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        length1: f32,
        length2: f32,
        lengthParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        min: vec3_t,
        max: vec3_t,
        elasticity: f32,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CTail;

    pub fn FX_AddCylinder(
        clientID: c_int,
        start: vec3_t,
        normal: vec3_t,
        size1s: f32,
        size1e: f32,
        size1Parm: f32,
        size2s: f32,
        size2e: f32,
        size2Parm: f32,
        length1: f32,
        length2: f32,
        lengthParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CCylinder;

    pub fn FX_AddEmitter(
        org: vec3_t,
        vel: vec3_t,
        accel: vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        angs: vec3_t,
        deltaAngs: vec3_t,
        min: vec3_t,
        max: vec3_t,
        elasticity: f32,
        deathID: c_int,
        impactID: c_int,
        emitterID: c_int,
        density: f32,
        variance: f32,
        killTime: c_int,
        model: qhandle_t,
        flags: c_int,
    ) -> *mut CEmitter;

    pub fn FX_AddLight(
        org: vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        killTime: c_int,
        flags: c_int,
    ) -> *mut CLight;

    pub fn FX_AddOrientedParticle(
        clientID: c_int,
        org: vec3_t,
        norm: vec3_t,
        vel: vec3_t,
        accel: vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        rotation: f32,
        rotationDelta: f32,
        min: vec3_t,
        max: vec3_t,
        bounce: f32,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut COrientedParticle;

    pub fn FX_AddPoly(
        verts: *mut vec3_t,
        st: *mut vec2_t,
        numVerts: c_int,
        vel: vec3_t,
        accel: vec3_t,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: vec3_t,
        rgb2: vec3_t,
        rgbParm: f32,
        rotationDelta: vec3_t,
        bounce: f32,
        motionDelay: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
    ) -> *mut CPoly;

    pub fn FX_AddFlash(
        origin: vec3_t,
        sRGB: vec3_t,
        eRGB: vec3_t,
        rgbParm: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int,
    ) -> *mut CFlash;

    // Included for backwards compatibility with CHC and for doing quick programmatic effects.
    pub fn FX_AddSprite_1(
        origin: vec3_t,
        vel: vec3_t,
        accel: vec3_t,
        scale: f32,
        dscale: f32,
        sAlpha: f32,
        eAlpha: f32,
        rotation: f32,
        bounce: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int,
    );

    pub fn FX_AddSprite_2(
        origin: vec3_t,
        vel: vec3_t,
        accel: vec3_t,
        scale: f32,
        dscale: f32,
        sAlpha: f32,
        eAlpha: f32,
        sRGB: vec3_t,
        eRGB: vec3_t,
        rotation: f32,
        bounce: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int,
    );

    pub fn FX_AddLine_1(
        start: vec3_t,
        end: vec3_t,
        stScale: f32,
        width: f32,
        dwidth: f32,
        sAlpha: f32,
        eAlpha: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int,
    );

    pub fn FX_AddLine_2(
        start: vec3_t,
        end: vec3_t,
        stScale: f32,
        width: f32,
        dwidth: f32,
        sAlpha: f32,
        eAlpha: f32,
        sRGB: vec3_t,
        eRGB: vec3_t,
        life: c_int,
        shader: qhandle_t,
        flags: c_int,
    );

    pub fn FX_AddQuad(
        origin: vec3_t,
        normal: vec3_t,
        vel: vec3_t,
        accel: vec3_t,
        sradius: f32,
        eradius: f32,
        salpha: f32,
        ealpha: f32,
        sRGB: vec3_t,
        eRGB: vec3_t,
        rotation: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int,
    );

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
        shader: qhandle_t,
        flags: c_int,
    ) -> *mut CBezier;
}

// Porting note: The original C++ header file used function overloading for convenience
// wrapper functions (FX_AddSprite, FX_AddLine). Since Rust doesn't support C++ style
// function overloading, these have been renamed with numeric suffixes (_1, _2) to
// distinguish between the different signatures. The first overload typically has more
// parameters, and the second is a convenience variant with fewer parameters.
