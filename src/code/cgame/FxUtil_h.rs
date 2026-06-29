// Faithful port of oracle/code/cgame/FxUtil.h.
// Include-guard directives (#ifndef FX_UTIL_H_INC / #endif) omitted: Rust modules have no
// double-include problem.  The conditional #if !defined(FX_PRIMITIVES_H_INC) around the
// FxPrimitives.h include is likewise omitted for the same reason.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

// #include "FxPrimitives.h"
// Trusted import — do NOT redefine CParticle, CLine, CElectricity, CTail, CCylinder,
// CEmitter, CLight, COrientedParticle, CPoly, CFlash, CBezier here.  They (and vec3_t,
// vec2_t, qhandle_t, etc.) are expected to be re-exported from this module.
use crate::code::cgame::FxPrimitives_h::*;

extern "C" {
    pub fn FX_Free() -> bool; // ditches all active effects;
    pub fn FX_Init() -> c_int; // called in CG_Init to purge the fx system.
    pub fn FX_Add(portal: bool); // called every cgame frame to add all fx into the scene.
    pub fn FX_Stop(); // ditches all active effects without touching the templates.

    pub fn FX_ActiveFx() -> bool; // returns whether there are any active or scheduled effects

    pub fn FX_AddParticle(
        clientID: c_int,
        org: *const vec3_t,
        vel: *const vec3_t,
        accel: *const vec3_t,
        gravity: f32,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: *const vec3_t,
        rgb2: *const vec3_t,
        rgbParm: f32,
        rotation: f32,
        rotationDelta: f32,
        min: *const vec3_t,
        max: *const vec3_t,
        elasticity: f32,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int, // C++ default: -1
        boltNum: c_int,  // C++ default: -1
    ) -> *mut CParticle;

    // Porting note: C++ has three overloads of FX_AddLine.  The one returning CLine* (below)
    // keeps its original name.  The two void convenience overloads are renamed FX_AddLine_1 /
    // FX_AddLine_2 at the bottom of this block, since Rust extern "C" cannot have duplicate
    // function names.
    pub fn FX_AddLine(
        clientID: c_int,
        start: *mut vec3_t,
        end: *mut vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: f32,
        killTime: c_int,
        shader: qhandle_t,
        impactFX_id: c_int,
        flags: c_int,
        modelNum: c_int, // C++ default: -1
        boltNum: c_int,  // C++ default: -1
    ) -> *mut CLine;

    pub fn FX_AddElectricity(
        clientID: c_int,
        start: *mut vec3_t,
        end: *mut vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        rgbParm: f32,
        chaos: f32,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int, // C++ default: -1
        boltNum: c_int,  // C++ default: -1
    ) -> *mut CElectricity;

    pub fn FX_AddTail(
        clientID: c_int,
        org: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        length1: f32,
        length2: f32,
        lengthParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: f32,
        min: *mut vec3_t,
        max: *mut vec3_t,
        elasticity: f32,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int, // C++ default: -1
        boltNum: c_int,  // C++ default: -1
    ) -> *mut CTail;

    pub fn FX_AddCylinder(
        clientID: c_int,
        start: *mut vec3_t,
        normal: *mut vec3_t,
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
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: f32,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int, // C++ default: -1
        boltNum: c_int,  // C++ default: -1
    ) -> *mut CCylinder;

    pub fn FX_AddEmitter(
        org: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: f32,
        angs: *mut vec3_t,
        deltaAngs: *mut vec3_t,
        min: *mut vec3_t,
        max: *mut vec3_t,
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
        org: *mut vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: f32,
        killTime: c_int,
        flags: c_int,
    ) -> *mut CLight;

    pub fn FX_AddOrientedParticle(
        clientID: c_int,
        org: *mut vec3_t,
        norm: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: f32,
        rotation: f32,
        rotationDelta: f32,
        min: *mut vec3_t,
        max: *mut vec3_t,
        bounce: f32,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        modelNum: c_int, // C++ default: -1
        boltNum: c_int,  // C++ default: -1
    ) -> *mut COrientedParticle;

    pub fn FX_AddPoly(
        verts: *mut vec3_t,
        st: *mut vec2_t,
        numVerts: c_int,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: f32,
        rotationDelta: *mut vec3_t,
        bounce: f32,
        motionDelay: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
    ) -> *mut CPoly;

    pub fn FX_AddFlash(
        origin: *mut vec3_t,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        rgbParm: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int,
    ) -> *mut CFlash;

    // Included for backwards compatibility with CHC and for doing quick programmatic effects.
    // Porting note: C++ declares two overloads of FX_AddSprite here.  Rust cannot have
    // two extern "C" items with the same name, so the first overload (without sRGB/eRGB)
    // is kept as FX_AddSprite, and the second overload (with sRGB/eRGB) is renamed
    // FX_AddSprite_2.  Same for FX_AddLine: the two void overloads become FX_AddLine_1
    // and FX_AddLine_2.  C++ default argument `flags = 0` is noted in comments.
    pub fn FX_AddSprite(
        origin: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        scale: f32,
        dscale: f32,
        sAlpha: f32,
        eAlpha: f32,
        rotation: f32,
        bounce: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int, // C++ default: 0
    );

    pub fn FX_AddSprite_2(
        origin: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        scale: f32,
        dscale: f32,
        sAlpha: f32,
        eAlpha: f32,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        rotation: f32,
        bounce: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int, // C++ default: 0
    );

    pub fn FX_AddLine_1(
        start: *mut vec3_t,
        end: *mut vec3_t,
        stScale: f32,
        width: f32,
        dwidth: f32,
        sAlpha: f32,
        eAlpha: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int, // C++ default: 0
    );

    pub fn FX_AddLine_2(
        start: *mut vec3_t,
        end: *mut vec3_t,
        stScale: f32,
        width: f32,
        dwidth: f32,
        sAlpha: f32,
        eAlpha: f32,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        life: c_int,
        shader: qhandle_t,
        flags: c_int, // C++ default: 0
    );

    pub fn FX_AddQuad(
        origin: *mut vec3_t,
        normal: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        sradius: f32,
        eradius: f32,
        salpha: f32,
        ealpha: f32,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        rotation: f32,
        life: c_int,
        shader: qhandle_t,
        flags: c_int, // C++ default: 0
    );

    pub fn FX_AddBezier(
        start: *const vec3_t,
        end: *const vec3_t,
        control1: *const vec3_t,
        control1Vel: *const vec3_t,
        control2: *const vec3_t,
        control2Vel: *const vec3_t,
        size1: f32,
        size2: f32,
        sizeParm: f32,
        alpha1: f32,
        alpha2: f32,
        alphaParm: f32,
        sRGB: *const vec3_t,
        eRGB: *const vec3_t,
        rgbParm: f32,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int, // C++ default: 0
    ) -> *mut CBezier;
}
