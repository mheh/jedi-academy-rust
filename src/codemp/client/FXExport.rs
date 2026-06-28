//! Mechanical port of `codemp/client/FXExport.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int};

use crate::codemp::game::q_shared_h::qboolean;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::codemp::client::FxScheduler_h::theFxScheduler;
use crate::codemp::client::FxSystem_h::theFxHelper;
use crate::codemp::client::FxUtil_h::{FX_Init, FX_SetRefDef, FX_Free};

// Local refdef_t stub for this module (already defined in FxScheduler_h or FxUtil_h headers).
#[repr(C)]
pub struct refdef_t {
    _unused: [u8; 0],
}

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "client.h"
// #include "FXScheduler.h"

//#define __FXCHECKER

#[no_mangle]
pub extern "C" fn FX_RegisterEffect(file: *const c_char) -> c_int {
    unsafe { (*core::ptr::addr_of_mut!(theFxScheduler)).RegisterEffect(file, true) }
}

#[no_mangle]
pub extern "C" fn FX_PlayEffect(file: *const c_char, org: *mut vec3_t, fwd: *mut vec3_t, vol: c_int, rad: c_int) {
    #[cfg(debug_assertions)]
    unsafe {
        if (*org)[0].is_nan() || (*org)[1].is_nan() || (*org)[2].is_nan() {
            assert!(false);
        }
        if (*fwd)[0].is_nan() || (*fwd)[1].is_nan() || (*fwd)[2].is_nan() {
            assert!(false);
        }
        if (*fwd)[0].abs() < 0.1 && (*fwd)[1].abs() < 0.1 && (*fwd)[2].abs() < 0.1 {
            assert!(false);
        }
    }

    unsafe {
        let org_val = *org;
        let fwd_val = *fwd;
        (*core::ptr::addr_of_mut!(theFxScheduler)).PlayEffect_file_org_fwd(file, org_val, fwd_val, vol, rad);
    }
}

#[no_mangle]
pub extern "C" fn FX_PlayEffectID(id: c_int, org: *mut vec3_t, fwd: *mut vec3_t, vol: c_int, rad: c_int, isPortal: qboolean) {
    #[cfg(debug_assertions)]
    unsafe {
        if (*org)[0].is_nan() || (*org)[1].is_nan() || (*org)[2].is_nan() {
            assert!(false);
        }
        if (*fwd)[0].is_nan() || (*fwd)[1].is_nan() || (*fwd)[2].is_nan() {
            assert!(false);
        }
        if (*fwd)[0].abs() < 0.1 && (*fwd)[1].abs() < 0.1 && (*fwd)[2].abs() < 0.1 {
            assert!(false);
        }
    }

    unsafe {
        let org_val = *org;
        let fwd_val = *fwd;
        (*core::ptr::addr_of_mut!(theFxScheduler)).PlayEffect_id_org_fwd(id, org_val, fwd_val, vol, rad, isPortal != 0);
    }
}

#[no_mangle]
pub extern "C" fn FX_PlayBoltedEffectID(id: c_int, org: *mut vec3_t, boltInfo: c_int, iGhoul2: c_int, iLooptime: c_int, isRelative: qboolean) {
    unsafe {
        let org_val = *org;
        let axis: [vec3_t; 3] = Default::default();
        let mut axis_mut = axis;
        (*core::ptr::addr_of_mut!(theFxScheduler)).PlayEffect_id_origin_axis(
            id,
            org_val,
            &mut axis_mut,
            boltInfo,
            iGhoul2,
            -1,
            -1,
            -1,
            false,
            iLooptime,
            isRelative != 0,
        );
    }
}

#[no_mangle]
pub extern "C" fn FX_PlayEntityEffectID(id: c_int, org: *mut vec3_t, axis: *mut [vec3_t; 3], boltInfo: c_int, entNum: c_int, vol: c_int, rad: c_int) {
    #[cfg(debug_assertions)]
    unsafe {
        if (*org)[0].is_nan() || (*org)[1].is_nan() || (*org)[2].is_nan() {
            assert!(false);
        }
    }

    unsafe {
        let org_val = *org;
        let axis_ref = &mut *axis;
        (*core::ptr::addr_of_mut!(theFxScheduler)).PlayEffect_id_origin_axis(
            id,
            org_val,
            axis_ref,
            boltInfo,
            0,
            -1,
            vol,
            rad,
            false,
            -1,
            false,
        );
    }
}

#[no_mangle]
pub extern "C" fn FX_AddScheduledEffects(portal: qboolean) {
    unsafe {
        (*core::ptr::addr_of_mut!(theFxScheduler)).AddScheduledEffects(portal != 0);
    }
}

#[no_mangle]
pub extern "C" fn FX_Draw2DEffects(screenXScale: c_float, screenYScale: c_float) {
    unsafe {
        (*core::ptr::addr_of_mut!(theFxScheduler)).Draw2DEffects(screenXScale, screenYScale);
    }
}

#[no_mangle]
pub extern "C" fn FX_InitSystem(refdef: *mut refdef_t) -> c_int {
    unsafe { FX_Init(refdef) }
}

#[no_mangle]
pub extern "C" fn FX_SetRefDefFromCGame(refdef: *mut refdef_t) {
    unsafe { FX_SetRefDef(refdef); }
}

#[no_mangle]
pub extern "C" fn FX_FreeSystem() -> qboolean {
    unsafe {
        if FX_Free(true) {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn FX_AdjustTime(time: c_int) {
    unsafe {
        (*core::ptr::addr_of_mut!(theFxHelper)).AdjustTime(time);
    }
}
