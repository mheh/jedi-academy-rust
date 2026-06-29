////////////////////////////////////////////////////////////////////////////////////////
// RAVEN SOFTWARE - STAR WARS: JK II
//  (c) 2002 Activision
//
// World Effects
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

use core::ffi::c_char;

// From "../game/q_shared.h"		// For Vec3_t
pub type vec3_t = [f32; 3];


////////////////////////////////////////////////////////////////////////////////////////
// Supported Commands
////////////////////////////////////////////////////////////////////////////////////////

extern "C" {
    pub fn R_AddWeatherZone(mins: vec3_t, maxs: vec3_t);

    pub fn R_InitWorldEffects();
    pub fn R_ShutdownWorldEffects();
    pub fn RB_RenderWorldEffects();

    pub fn R_WorldEffectCommand(command: *const c_char);
    pub fn R_WorldEffect_f();
}

////////////////////////////////////////////////////////////////////////////////////////
// Exported Functionality
////////////////////////////////////////////////////////////////////////////////////////

extern "C" {
    pub fn R_GetWindVector(windVector: vec3_t, atpoint: vec3_t) -> bool;
    pub fn R_GetWindSpeed(windSpeed: *mut f32, atpoint: vec3_t) -> bool;
    pub fn R_GetWindGusting(atpoint: vec3_t) -> bool;
    pub fn R_IsOutside(pos: vec3_t) -> bool;
    pub fn R_IsOutsideCausingPain(pos: vec3_t) -> f32;
    pub fn R_GetChanceOfSaberFizz() -> f32;
    pub fn R_IsShaking(pos: vec3_t) -> bool;
    pub fn R_SetTempGlobalFogColor(color: vec3_t) -> bool;

    pub fn R_IsRaining() -> bool;
    pub fn R_IsPuffing() -> bool;
}
