//! Mechanical port of `codemp/client/FXExport.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int};

use crate::codemp::game::q_shared_h::{qboolean, vec3_t};

#[repr(C)]
pub struct refdef_t {
    _unused: [u8; 0],
}

unsafe extern "C" {
    pub fn FX_RegisterEffect(file: *const c_char) -> c_int;

    pub fn FX_PlayEffect(file: *const c_char, org: *mut vec3_t, fwd: *mut vec3_t, vol: c_int, rad: c_int);

    // C++ default argument `isPortal = qfalse` is not representable in Rust FFI.
    pub fn FX_PlayEffectID(
        id: c_int,
        org: *mut vec3_t,
        fwd: *mut vec3_t,
        vol: c_int,
        rad: c_int,
        isPortal: qboolean,
    );
    pub fn FX_PlayEntityEffectID(
        id: c_int,
        org: *mut vec3_t,
        axis: *mut vec3_t,
        boltInfo: c_int,
        entNum: c_int,
        vol: c_int,
        rad: c_int,
    );
    pub fn FX_PlayBoltedEffectID(
        id: c_int,
        org: *mut vec3_t,
        boltInfo: c_int,
        iGhoul2: c_int,
        iLooptime: c_int,
        isRelative: qboolean,
    );

    pub fn FX_AddScheduledEffects(portal: qboolean);
    pub fn FX_Draw2DEffects(screenXScale: c_float, screenYScale: c_float);

    pub fn FX_InitSystem(refdef: *mut refdef_t) -> c_int;
    pub fn FX_SetRefDefFromCGame(refdef: *mut refdef_t);
    pub fn FX_FreeSystem() -> qboolean;
    pub fn FX_AdjustTime(time: c_int);
}
