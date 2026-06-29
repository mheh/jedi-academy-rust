// Faithful port of oracle/codemp/client/FxUtil.h.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_float, c_int};

// #include "FxPrimitives.h" (guarded: #if !defined(FX_PRIMITIVES_H_INC))
use crate::codemp::client::FxPrimitives_h::*;
// NOTE (port): oracle FxUtil.h includes only FxPrimitives.h; refdef_t and other
// external types arrive transitively through that chain (FxPrimitives.h -> FxSystem.h).

unsafe extern "C" {
    pub fn FX_Free(templates: bool) -> bool; // ditches all active effects;
    pub fn FX_Init(refdef: *mut refdef_t) -> c_int; // called in CG_Init to purge the fx system.
    pub fn FX_SetRefDef(refdef: *mut refdef_t);
    pub fn FX_Add(portal: bool); // called every cgame frame to add all fx into the scene.
    pub fn FX_Stop(); // ditches all active effects without touching the templates.

    pub fn FX_AddParticle(
        org: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        rotation: c_float,
        rotationDelta: c_float,
        min: *mut vec3_t,
        max: *mut vec3_t,
        elasticity: c_float,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CParticle;

    pub fn FX_AddLine(
        start: *mut vec3_t,
        end: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CLine;

    pub fn FX_AddElectricity(
        start: *mut vec3_t,
        end: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        rgbParm: c_float,
        chaos: c_float,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CElectricity;

    pub fn FX_AddTail(
        org: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        length1: c_float,
        length2: c_float,
        lengthParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        min: *mut vec3_t,
        max: *mut vec3_t,
        elasticity: c_float,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CTail;

    pub fn FX_AddCylinder(
        start: *mut vec3_t,
        normal: *mut vec3_t,
        size1s: c_float,
        size1e: c_float,
        size1Parm: c_float,
        size2s: c_float,
        size2e: c_float,
        size2Parm: c_float,
        length1: c_float,
        length2: c_float,
        lengthParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1, traceEnd = qfalse
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
        traceEnd: qboolean,
    ) -> *mut CCylinder;

    pub fn FX_AddEmitter(
        org: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        angs: *mut vec3_t,
        deltaAngs: *mut vec3_t,
        min: *mut vec3_t,
        max: *mut vec3_t,
        elasticity: c_float,
        deathID: c_int,
        impactID: c_int,
        emitterID: c_int,
        density: c_float,
        variance: c_float,
        killTime: c_int,
        model: qhandle_t,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CEmitter;

    pub fn FX_AddLight(
        org: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        killTime: c_int,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut CLight;

    pub fn FX_AddOrientedParticle(
        org: *mut vec3_t,
        norm: *mut vec3_t,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        rotation: c_float,
        rotationDelta: c_float,
        min: *mut vec3_t,
        max: *mut vec3_t,
        bounce: c_float,
        deathID: c_int,
        impactID: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1,
        // iGhoul2=0, entNum=-1, modelNum=-1, boltNum=-1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) -> *mut COrientedParticle;

    pub fn FX_AddPoly(
        verts: *mut vec3_t,
        st: *mut vec2_t,
        numVerts: c_int,
        vel: *mut vec3_t,
        accel: *mut vec3_t,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        rgb1: *mut vec3_t,
        rgb2: *mut vec3_t,
        rgbParm: c_float,
        rotationDelta: *mut vec3_t,
        bounce: c_float,
        motionDelay: c_int,
        killTime: c_int,
        shader: qhandle_t,
        flags: c_int,
    ) -> *mut CPoly;

    pub fn FX_AddFlash(
        origin: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        rgbParm: c_float,
        killTime: c_int,
        shader: qhandle_t,
        // C++ default: flags = 0
        flags: c_int,
        // C++ defaults: matImpactFX = MATIMPACTFX_NONE, fxParm = -1
        matImpactFX: EMatImpactEffect,
        fxParm: c_int,
    ) -> *mut CFlash;

    pub fn FX_AddBezier(
        start: *mut vec3_t,
        end: *mut vec3_t,
        control1: *mut vec3_t,
        control1Vel: *mut vec3_t,
        control2: *mut vec3_t,
        control2Vel: *mut vec3_t,
        size1: c_float,
        size2: c_float,
        sizeParm: c_float,
        alpha1: c_float,
        alpha2: c_float,
        alphaParm: c_float,
        sRGB: *mut vec3_t,
        eRGB: *mut vec3_t,
        rgbParm: c_float,
        killTime: c_int,
        shader: qhandle_t,
        // C++ default: flags = 0
        flags: c_int,
    ) -> *mut CBezier;
}
