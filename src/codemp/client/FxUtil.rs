//! Mechanical port of `codemp/client/FxUtil.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(improper_ctypes)]
#![allow(clippy::too_many_arguments)]

use core::ffi::{c_float, c_int};
use core::ptr;
use std::ptr::addr_of_mut;

use crate::codemp::client::FxPrimitives_h::{
    CEffect, CParticle, CLine, CElectricity, CTail, CCylinder, CEmitter, CLight,
    COrientedParticle, CPoly, CFlash, CBezier, MAX_EFFECTS, EMatImpactEffect,
    FX_RGB_PARM_MASK, FX_ALPHA_PARM_MASK, FX_SIZE_PARM_MASK, FX_SIZE2_PARM_MASK,
    FX_LENGTH_PARM_MASK, FX_RGB_WAVE, FX_ALPHA_WAVE, FX_SIZE_WAVE, FX_SIZE2_WAVE,
    FX_LENGTH_WAVE, FX_KILL_ON_IMPACT, FX_RELATIVE,
};
use crate::codemp::client::FxSystem_h::{SFxHelper, theFxHelper, refdef_t};
use crate::codemp::client::FxScheduler_h::SFxScheduler;
use crate::codemp::game::q_math::{VectorCopy, Vector2Copy};
use crate::codemp::game::q_shared_h::{qhandle_t, qboolean, vec2_t, vec3_t};
use crate::codemp::qcommon::files_h::cvar_t;

// Unported dependency stub from main client/server engine
extern "C" {
    pub fn Cvar_Get(var_name: *const core::ffi::c_char, value: *const core::ffi::c_char, flags: c_int) -> *mut cvar_t;
}

pub const PI: c_float = 3.14159f32;

// Struct definition matching C layout
#[repr(C)]
pub struct SEffectList {
    pub mEffect: *mut CEffect,
    pub mKillTime: c_int,
    pub mPortal: bool,
}

// Globals
pub static WHITE: vec3_t = [1.0f32, 1.0f32, 1.0f32];

pub static mut effectList: [SEffectList; MAX_EFFECTS as usize] = unsafe {
    [SEffectList {
        mEffect: ptr::null_mut(),
        mKillTime: 0,
        mPortal: false,
    }; MAX_EFFECTS as usize]
};

pub static mut nextValidEffect: *mut SEffectList = ptr::null_mut();

// Declared external from FxSystem
pub static mut theFxScheduler: SFxScheduler = unsafe {
    core::mem::zeroed()
};

pub static mut activeFx: c_int = 0;
pub static mut drawnFx: c_int = 0;
pub static mut fxInitialized: qboolean = 0 as qboolean;

// Cvars
pub static mut fx_debug: *mut cvar_t = ptr::null_mut();
#[cfg(feature = "sof2dev")]
pub static mut fx_freeze: *mut cvar_t = ptr::null_mut();
pub static mut fx_countScale: *mut cvar_t = ptr::null_mut();
pub static mut fx_nearCull: *mut cvar_t = ptr::null_mut();

//-------------------------
// FX_Free
//
// Frees all FX
//-------------------------
pub fn FX_Free(templates: bool) -> bool {
    for i in 0..MAX_EFFECTS as usize {
        if unsafe { !(*addr_of_mut!(effectList[i])).mEffect.is_null() } {
            unsafe {
                let effect = (*addr_of_mut!(effectList[i])).mEffect;
                let _ = Box::from_raw(effect);
            }
        }

        unsafe {
            (*addr_of_mut!(effectList[i])).mEffect = ptr::null_mut();
        }
    }

    unsafe {
        activeFx = 0;
        theFxScheduler.Clean(templates);
    }
    true
}

//-------------------------
// FX_Stop
//
// Frees all active FX but leaves the templates
//-------------------------
pub fn FX_Stop() {
    for i in 0..MAX_EFFECTS as usize {
        if unsafe { !(*addr_of_mut!(effectList[i])).mEffect.is_null() } {
            unsafe {
                let effect = (*addr_of_mut!(effectList[i])).mEffect;
                let _ = Box::from_raw(effect);
            }
        }

        unsafe {
            (*addr_of_mut!(effectList[i])).mEffect = ptr::null_mut();
        }
    }

    unsafe {
        activeFx = 0;
        theFxScheduler.Clean(false);
    }
}

//-------------------------
// FX_Init
//
// Preps system for use
//-------------------------
pub fn FX_Init(refdef: *mut refdef_t) -> c_int {
    // FX_Free( true );
    unsafe {
        if fxInitialized == 0 as qboolean {
            fxInitialized = 1 as qboolean;

            for i in 0..MAX_EFFECTS as usize {
                (*addr_of_mut!(effectList[i])).mEffect = ptr::null_mut();
            }
        }
        nextValidEffect = addr_of_mut!(effectList[0]);

        #[cfg(feature = "sof2dev")]
        {
            fx_freeze = Cvar_Get(b"fx_freeze\0".as_ptr() as *const _, b"0\0".as_ptr() as *const _, 0x00000004); // CVAR_CHEAT
        }

        fx_debug = Cvar_Get(b"fx_debug\0".as_ptr() as *const _, b"0\0".as_ptr() as *const _, 0x00000010); // CVAR_TEMP
        fx_countScale = Cvar_Get(b"fx_countScale\0".as_ptr() as *const _, b"1\0".as_ptr() as *const _, 0x00000001); // CVAR_ARCHIVE
        fx_nearCull = Cvar_Get(b"fx_nearCull\0".as_ptr() as *const _, b"16\0".as_ptr() as *const _, 0x00000001); // CVAR_ARCHIVE

        theFxHelper.ReInit(refdef);
    }

    1 as c_int
}

pub fn FX_SetRefDef(refdef: *mut refdef_t) {
    unsafe {
        theFxHelper.refdef = refdef;
    }
}

//-------------------------
// FX_FreeMember
//-------------------------
unsafe fn FX_FreeMember(obj: *mut SEffectList) {
    (*(*obj).mEffect).Die();
    let effect = (*obj).mEffect;
    let _ = Box::from_raw(effect);
    (*obj).mEffect = ptr::null_mut();

    // May as well mark this to be used next
    nextValidEffect = obj;

    activeFx -= 1;
}

//-------------------------
// FX_GetValidEffect
//
// Finds an unused effect slot
//
// Note - in the editor, this function may return NULL, indicating that all
// effects are being stopped.
//-------------------------
unsafe fn FX_GetValidEffect() -> *mut SEffectList {
    if (*nextValidEffect).mEffect.is_null() {
        return nextValidEffect;
    }

    // Blah..plow through the list till we find something that is currently untainted
    let mut i = 0;
    let mut ef = addr_of_mut!(effectList[0]);
    while i < MAX_EFFECTS as usize {
        if (*ef).mEffect.is_null() {
            return ef;
        }
        i += 1;
        ef = addr_of_mut!(effectList[i]);
    }

    // report the error.
    #[cfg(not(feature = "final_build"))]
    {
        theFxHelper.Print(b"FX system out of effects\n\0".as_ptr() as *const _);
    }

    // Hmmm.. just trashing the first effect in the list is a poor approach
    FX_FreeMember(addr_of_mut!(effectList[0]));

    // Recursive call
    nextValidEffect
}

//-------------------------
// FX_Add
//
// Adds all fx to the view
//-------------------------
pub fn FX_Add(portal: bool) {
    unsafe {
        drawnFx = 0;

        let numFx = activeFx; //but stop when there can't be any more left!
        let mut i = 0;
        let mut ef = addr_of_mut!(effectList[0]);
        let mut remaining = numFx;

        while i < MAX_EFFECTS as usize && remaining > 0 {
            if !(*ef).mEffect.is_null() {
                remaining -= 1;
                if portal != (*ef).mPortal {
                    i += 1;
                    ef = addr_of_mut!(effectList[i]);
                    continue; //this one does not render in this scene
                }
                // Effect is active
                if theFxHelper.mTime > (*ef).mKillTime {
                    // Clean up old effects, calling any death effects as needed
                    // this flag just has to be cleared otherwise death effects might not happen correctly
                    (*(*ef).mEffect).ClearFlags(FX_KILL_ON_IMPACT);
                    FX_FreeMember(ef);
                } else {
                    if (*(*ef).mEffect).Update() == false {
                        // We've been marked for death
                        FX_FreeMember(ef);
                        i += 1;
                        ef = addr_of_mut!(effectList[i]);
                        continue;
                    }
                }
            }
            i += 1;
            ef = addr_of_mut!(effectList[i]);
        }

        if !fx_debug.is_null() && (*fx_debug).integer != 0 && !portal {
            theFxHelper.Print(
                b"Active    FX: %i\n\0".as_ptr() as *const _,
                activeFx,
            );
            theFxHelper.Print(
                b"Drawn     FX: %i\n\0".as_ptr() as *const _,
                drawnFx,
            );
            theFxHelper.Print(
                b"Scheduled FX: %i\n\0".as_ptr() as *const _,
                theFxScheduler.NumScheduledFx(),
            );
        }
    }
}

//-------------------------
// FX_AddPrimitive
//
// Note - in the editor, this function may change *pEffect to NULL, indicating that
// all effects are being stopped.
//-------------------------
extern "C" {
    pub static gEffectsInPortal: bool; //from FXScheduler.cpp so i don't have to pass it in on EVERY FX_ADD*
}

pub unsafe fn FX_AddPrimitive(pEffect: *mut *mut CEffect, killTime: c_int) {
    let item = FX_GetValidEffect();

    (*item).mEffect = *pEffect;
    (*item).mKillTime = theFxHelper.mTime + killTime;
    (*item).mPortal = gEffectsInPortal; //global set in AddScheduledEffects

    activeFx += 1;

    // Stash these in the primitive so it has easy access to the vals
    (*(*pEffect)).SetTimeStart(theFxHelper.mTime);
    (*(*pEffect)).SetTimeEnd(theFxHelper.mTime + killTime);
}

//-------------------------
//  FX_AddParticle
//-------------------------
pub unsafe fn FX_AddParticle(
    org: *mut vec3_t,
    vel: *mut vec3_t,
    accel: *mut vec3_t,
    size1: c_float,
    size2: c_float,
    sizeParm: c_float,
    alpha1: c_float,
    alpha2: c_float,
    alphaParm: c_float,
    sRGB: *mut vec3_t,
    eRGB: *mut vec3_t,
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
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CParticle {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CParticle::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            (*fx).SetOrigin1(ptr::null_mut());
            (*fx).SetOrgOffset(org);
            (*fx).SetBoltinfo(iGhoul2, entNum, modelNum, boltNum);
        } else {
            (*fx).SetOrigin1(org);
        }
        (*fx).SetOrigin1(org);
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);
        (*fx).SetVel(vel);
        (*fx).SetAccel(accel);

        // RGB----------------
        (*fx).SetRGBStart(sRGB);
        (*fx).SetRGBEnd(eRGB);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetFlags(flags);
        (*fx).SetShader(shader);
        (*fx).SetRotation(rotation);
        (*fx).SetRotationDelta(rotationDelta);
        (*fx).SetElasticity(elasticity);
        (*fx).SetMin(min);
        (*fx).SetMax(max);
        (*fx).SetDeathFxID(deathID);
        (*fx).SetImpactFxID(impactID);

        (*fx).Init();

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CParticle
}

//-------------------------
//  FX_AddLine
//-------------------------
pub unsafe fn FX_AddLine(
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
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CLine {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding new effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CLine::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            (*fx).SetOrigin1(ptr::null_mut());
            (*fx).SetOrgOffset(start); //offset from bolt pos
            (*fx).SetVel(end); //vel is the vector offset from bolt+orgOffset
            (*fx).SetBoltinfo(iGhoul2, entNum, modelNum, boltNum);
        } else {
            (*fx).SetOrigin1(start);
            (*fx).SetOrigin2(end);
        }
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);

        // RGB----------------
        (*fx).SetRGBStart(sRGB);
        (*fx).SetRGBEnd(eRGB);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetShader(shader);
        (*fx).SetFlags(flags);

        (*fx).SetSTScale(1.0f32, 1.0f32);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CLine
}

//-------------------------
//  FX_AddElectricity
//-------------------------
pub unsafe fn FX_AddElectricity(
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
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CElectricity {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding new effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CElectricity::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            (*fx).SetOrigin1(ptr::null_mut());
            (*fx).SetOrgOffset(start); //offset
            (*fx).SetVel(end); //vel is the vector offset from bolt+orgOffset
            (*fx).SetBoltinfo(iGhoul2, entNum, modelNum, boltNum);
        } else {
            (*fx).SetOrigin1(start);
            (*fx).SetOrigin2(end);
        }
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);

        // RGB----------------
        (*fx).SetRGBStart(sRGB);
        (*fx).SetRGBEnd(eRGB);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetShader(shader);
        (*fx).SetFlags(flags);
        (*fx).SetChaos(chaos);

        (*fx).SetSTScale(1.0f32, 1.0f32);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
        // in the editor, fx may now be NULL?
        if !fx.is_null() {
            (*fx).Initialize();
        }
    }

    fx as *mut CElectricity
}

//-------------------------
//  FX_AddTail
//-------------------------
pub unsafe fn FX_AddTail(
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
    sRGB: *mut vec3_t,
    eRGB: *mut vec3_t,
    rgbParm: c_float,
    min: *mut vec3_t,
    max: *mut vec3_t,
    elasticity: c_float,
    deathID: c_int,
    impactID: c_int,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CTail {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CTail::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            (*fx).SetOrigin1(ptr::null_mut());
            (*fx).SetOrgOffset(org);
            (*fx).SetBoltinfo(iGhoul2, entNum, modelNum, boltNum);
        } else {
            (*fx).SetOrigin1(org);
        }
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);
        (*fx).SetVel(vel);
        (*fx).SetAccel(accel);

        // RGB----------------
        (*fx).SetRGBStart(sRGB);
        (*fx).SetRGBEnd(eRGB);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Length----------------
        (*fx).SetLengthStart(length1);
        (*fx).SetLengthEnd(length2);

        if (flags & FX_LENGTH_PARM_MASK as c_int) == FX_LENGTH_WAVE as c_int {
            (*fx).SetLengthParm(lengthParm * PI * 0.001f32);
        } else if (flags & FX_LENGTH_PARM_MASK as c_int) != 0 {
            (*fx).SetLengthParm(lengthParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetFlags(flags);
        (*fx).SetShader(shader);
        (*fx).SetElasticity(elasticity);
        (*fx).SetMin(min);
        (*fx).SetMax(max);
        (*fx).SetSTScale(1.0f32, 1.0f32);
        (*fx).SetDeathFxID(deathID);
        (*fx).SetImpactFxID(impactID);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CTail
}

//-------------------------
//  FX_AddCylinder
//-------------------------
pub unsafe fn FX_AddCylinder(
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
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
    traceEnd: qboolean,
) -> *mut CCylinder {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding new effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CCylinder::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            (*fx).SetOrigin1(ptr::null_mut());
            (*fx).SetOrgOffset(start); //offset
            (*fx).SetBoltinfo(iGhoul2, entNum, modelNum, boltNum);
        } else {
            (*fx).SetOrigin1(start);
        }
        (*fx).SetTraceEnd(traceEnd);

        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);
        (*fx).SetOrigin1(start);
        (*fx).SetNormal(normal);

        // RGB----------------
        (*fx).SetRGBStart(rgb1);
        (*fx).SetRGBEnd(rgb2);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size1----------------
        (*fx).SetSizeStart(size1s);
        (*fx).SetSizeEnd(size1e);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(size1Parm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(size1Parm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size2----------------
        (*fx).SetSize2Start(size2s);
        (*fx).SetSize2End(size2e);

        if (flags & FX_SIZE2_PARM_MASK as c_int) == FX_SIZE2_WAVE as c_int {
            (*fx).SetSize2Parm(size2Parm * PI * 0.001f32);
        } else if (flags & FX_SIZE2_PARM_MASK as c_int) != 0 {
            (*fx).SetSize2Parm(size2Parm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Length1---------------
        (*fx).SetLengthStart(length1);
        (*fx).SetLengthEnd(length2);

        if (flags & FX_LENGTH_PARM_MASK as c_int) == FX_LENGTH_WAVE as c_int {
            (*fx).SetLengthParm(lengthParm * PI * 0.001f32);
        } else if (flags & FX_LENGTH_PARM_MASK as c_int) != 0 {
            (*fx).SetLengthParm(lengthParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetShader(shader);
        (*fx).SetFlags(flags);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CCylinder
}

//-------------------------
//  FX_AddEmitter
//-------------------------
pub unsafe fn FX_AddEmitter(
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
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CEmitter {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CEmitter::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            panic!("not done"); //assert(0);//not done
                                 //			fx->SetBoltinfo( iGhoul2, entNum, modelNum, boltNum );
        }
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);
        (*fx).SetOrigin1(org);
        (*fx).SetVel(vel);
        (*fx).SetAccel(accel);

        // RGB----------------
        (*fx).SetRGBStart(rgb1);
        (*fx).SetRGBEnd(rgb2);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetAngles(angs);
        (*fx).SetAngleDelta(deltaAngs);
        (*fx).SetFlags(flags);
        (*fx).SetModel(model);
        (*fx).SetElasticity(elasticity);
        (*fx).SetMin(min);
        (*fx).SetMax(max);
        (*fx).SetDeathFxID(deathID);
        (*fx).SetImpactFxID(impactID);
        (*fx).SetEmitterFxID(emitterID);
        (*fx).SetDensity(density);
        (*fx).SetVariance(variance);
        (*fx).SetOldTime(theFxHelper.mTime);

        (*fx).SetLastOrg(org);
        (*fx).SetLastVel(vel);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CEmitter
}

//-------------------------
//  FX_AddLight
//-------------------------
pub unsafe fn FX_AddLight(
    org: *mut vec3_t,
    size1: c_float,
    size2: c_float,
    sizeParm: c_float,
    rgb1: *mut vec3_t,
    rgb2: *mut vec3_t,
    rgbParm: c_float,
    killTime: c_int,
    flags: c_int,
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CLight {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CLight::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            (*fx).SetOrigin1(ptr::null_mut());
            (*fx).SetOrgOffset(org); //offset
            (*fx).SetBoltinfo(iGhoul2, entNum, modelNum, boltNum);
        } else {
            (*fx).SetOrigin1(org);
        }
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);

        // RGB----------------
        (*fx).SetRGBStart(rgb1);
        (*fx).SetRGBEnd(rgb2);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetFlags(flags);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CLight
}

//-------------------------
//  FX_AddOrientedParticle
//-------------------------
pub unsafe fn FX_AddOrientedParticle(
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
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
    iGhoul2: c_int,
    entNum: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut COrientedParticle {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(COrientedParticle::default()));

    if !fx.is_null() {
        if (flags & FX_RELATIVE as c_int) != 0 && iGhoul2 > 0 {
            (*fx).SetOrigin1(ptr::null_mut());
            (*fx).SetOrgOffset(org); //offset
            (*fx).SetBoltinfo(iGhoul2, entNum, modelNum, boltNum);
        } else {
            (*fx).SetOrigin1(org);
        }
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);
        (*fx).SetOrigin1(org);
        (*fx).SetNormal(norm);
        (*fx).SetVel(vel);
        (*fx).SetAccel(accel);

        // RGB----------------
        (*fx).SetRGBStart(rgb1);
        (*fx).SetRGBEnd(rgb2);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetFlags(flags);
        (*fx).SetShader(shader);
        (*fx).SetRotation(rotation);
        (*fx).SetRotationDelta(rotationDelta);
        (*fx).SetElasticity(bounce);
        (*fx).SetMin(min);
        (*fx).SetMax(max);
        (*fx).SetDeathFxID(deathID);
        (*fx).SetImpactFxID(impactID);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut COrientedParticle
}

//-------------------------
//  FX_AddPoly
//-------------------------
pub unsafe fn FX_AddPoly(
    verts: *mut *mut vec3_t,
    st: *mut *mut vec2_t,
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
) -> *mut CPoly {
    if theFxHelper.mFrameTime < 1 || verts.is_null() {
        // disallow adding effects when the system is paused or the user doesn't pass in a vert array
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CPoly::default()));

    if !fx.is_null() {
        // Do a cheesy copy of the verts and texture coords into our own structure
        for i in 0..numVerts as usize {
            VectorCopy(&*(*verts.add(i)), &mut (*fx).mOrg[i]);
            Vector2Copy(&*(*st.add(i)), &mut (*fx).mST[i]);
        }

        (*fx).SetVel(vel);
        (*fx).SetAccel(accel);

        // RGB----------------
        (*fx).SetRGBStart(rgb1);
        (*fx).SetRGBEnd(rgb2);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetFlags(flags);
        (*fx).SetShader(shader);
        (*fx).SetRot(rotationDelta);
        (*fx).SetElasticity(bounce);
        (*fx).SetMotionTimeStamp(motionDelay);
        (*fx).SetNumVerts(numVerts);

        // Now that we've set our data up, let's process it into a useful format
        (*fx).PolyInit();

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CPoly
}

//-------------------------
//  FX_AddFlash
//-------------------------
pub unsafe fn FX_AddFlash(
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
    flags: c_int,
    matImpactFX: EMatImpactEffect,
    fxParm: c_int,
) -> *mut CFlash {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding new effects when the system is paused
        return ptr::null_mut();
    }

    if shader == 0 {
        //yeah..this is bad, I guess, but SP seems to handle it by not drawing the flash, so I will too.
        panic!("shader is 0");
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CFlash::default()));

    if !fx.is_null() {
        (*fx).SetMatImpactFX(matImpactFX);
        (*fx).SetMatImpactParm(fxParm);
        (*fx).SetOrigin1(origin);

        // RGB----------------
        (*fx).SetRGBStart(sRGB);
        (*fx).SetRGBEnd(eRGB);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetShader(shader);
        (*fx).SetFlags(flags);

        //		fx->SetSTScale( 1.0f, 1.0f );

        (*fx).Init();

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CFlash
}

//-------------------------
//  FX_AddBezier
//-------------------------
pub unsafe fn FX_AddBezier(
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
    flags: c_int,
) -> *mut CBezier {
    if theFxHelper.mFrameTime < 1 {
        // disallow adding new effects when the system is paused
        return ptr::null_mut();
    }

    let fx = Box::into_raw(Box::new(CBezier::default()));

    if !fx.is_null() {
        (*fx).SetOrigin1(start);
        (*fx).SetOrigin2(end);

        (*fx).SetControlPoints(control1, control2);
        (*fx).SetControlVel(control1Vel, control2Vel);

        // RGB----------------
        (*fx).SetRGBStart(sRGB);
        (*fx).SetRGBEnd(eRGB);

        if (flags & FX_RGB_PARM_MASK as c_int) == FX_RGB_WAVE as c_int {
            (*fx).SetRGBParm(rgbParm * PI * 0.001f32);
        } else if (flags & FX_RGB_PARM_MASK as c_int) != 0 {
            // rgbParm should be a value from 0-100..
            (*fx).SetRGBParm(rgbParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Alpha----------------
        (*fx).SetAlphaStart(alpha1);
        (*fx).SetAlphaEnd(alpha2);

        if (flags & FX_ALPHA_PARM_MASK as c_int) == FX_ALPHA_WAVE as c_int {
            (*fx).SetAlphaParm(alphaParm * PI * 0.001f32);
        } else if (flags & FX_ALPHA_PARM_MASK as c_int) != 0 {
            (*fx).SetAlphaParm(alphaParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        // Size----------------
        (*fx).SetSizeStart(size1);
        (*fx).SetSizeEnd(size2);

        if (flags & FX_SIZE_PARM_MASK as c_int) == FX_SIZE_WAVE as c_int {
            (*fx).SetSizeParm(sizeParm * PI * 0.001f32);
        } else if (flags & FX_SIZE_PARM_MASK as c_int) != 0 {
            (*fx).SetSizeParm(sizeParm * 0.01f32 * killTime as c_float + theFxHelper.mTime as c_float);
        }

        (*fx).SetShader(shader);
        (*fx).SetFlags(flags);

        (*fx).SetSTScale(1.0f32, 1.0f32);

        FX_AddPrimitive(&mut (fx as *mut CEffect), killTime);
    }

    fx as *mut CBezier
}
