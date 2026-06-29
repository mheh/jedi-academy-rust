//! Mechanical port of `oracle/codemp/cgame/fx_local.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::codemp::cgame::fx_bowcaster::centity_t;
use crate::codemp::cgame::fx_flechette::weaponInfo_s;
use core::ffi::c_int;

//
// fx_*.c
//

// NOTENOTE This is not the best, DO NOT CHANGE THESE!
pub const FX_ALPHA_LINEAR: c_int = 0x00000001;
pub const FX_SIZE_LINEAR: c_int = 0x00000100;

// Bryar
unsafe extern "C" {
    pub fn FX_BryarProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BryarAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BryarHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_BryarAltHitWall(origin: [f32; 3], normal: [f32; 3], power: c_int);
    pub fn FX_BryarHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
    pub fn FX_BryarAltHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
}

// Blaster
unsafe extern "C" {
    pub fn FX_BlasterProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BlasterAltFireThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BlasterWeaponHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_BlasterWeaponHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
}

// Disruptor
unsafe extern "C" {
    pub fn FX_DisruptorMainShot(start: [f32; 3], end: [f32; 3]);
    pub fn FX_DisruptorAltShot(start: [f32; 3], end: [f32; 3], fullCharge: c_int);
    pub fn FX_DisruptorAltMiss(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_DisruptorAltHit(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_DisruptorHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_DisruptorHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
}

// Bowcaster
unsafe extern "C" {
    pub fn FX_BowcasterProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BowcasterAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BowcasterHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_BowcasterHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
}

// Heavy Repeater
unsafe extern "C" {
    pub fn FX_RepeaterProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_RepeaterAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_RepeaterHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_RepeaterAltHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_RepeaterHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
    pub fn FX_RepeaterAltHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
}

// DEMP2
unsafe extern "C" {
    pub fn FX_DEMP2_ProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_DEMP2_HitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_DEMP2_HitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
    pub fn FX_DEMP2_AltDetonate(org: [f32; 3], size: f32);
}

// Golan Arms Flechette
unsafe extern "C" {
    pub fn FX_FlechetteProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_FlechetteWeaponHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_FlechetteWeaponHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
    pub fn FX_FlechetteAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
}

// Personal Rocket Launcher
unsafe extern "C" {
    pub fn FX_RocketProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_RocketAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_RocketHitWall(origin: [f32; 3], normal: [f32; 3]);
    pub fn FX_RocketHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: c_int);
}
