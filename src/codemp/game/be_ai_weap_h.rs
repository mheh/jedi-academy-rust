//! Bot weapon-AI types from `be_ai_weap.h`.
//!
//! `weaponinfo_t` crosses the VM ABI by pointer in `trap_BotGetWeaponInfo`, and embeds
//! `projectileinfo_t` by value, so both layouts are load-bearing. Faithful 1:1 with the
//! original-JKA `refs/raven-jediacademy/codemp/game/be_ai_weap.h`. `MAX_STRINGFIELD` is the
//! 80-byte string field width defined in `be_aas.h`.

#![allow(non_camel_case_types)]

use super::q_shared_h::vec3_t;
use core::ffi::{c_char, c_int};

/// `MAX_STRINGFIELD` (be_aas.h) — fixed string field width in bot AI structs.
pub const MAX_STRINGFIELD: usize = 80;

/// `projectileinfo_t` (be_ai_weap.h) — projectile description embedded by value in
/// [`weaponinfo_t`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct projectileinfo_t {
    pub name: [c_char; MAX_STRINGFIELD],
    pub model: [c_char; MAX_STRINGFIELD],
    pub flags: c_int,
    pub gravity: f32,
    pub damage: c_int,
    pub radius: f32,
    pub visdamage: c_int,
    pub damagetype: c_int,
    pub healthinc: c_int,
    pub push: f32,
    pub detonation: f32,
    pub bounce: f32,
    pub bouncefric: f32,
    pub bouncestop: f32,
}

impl Default for projectileinfo_t {
    fn default() -> Self {
        // SAFETY: all fields are plain `int`/`float`/`char` arrays — an all-zero bit
        // pattern is a valid value, matching C's zero-initialized projectileinfo_t.
        unsafe { core::mem::zeroed() }
    }
}

/// `weaponinfo_t` (be_ai_weap.h) — current-weapon info filled by `trap_BotGetWeaponInfo`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct weaponinfo_t {
    /// true if the weapon info is valid
    pub valid: c_int,
    /// number of the weapon
    pub number: c_int,
    pub name: [c_char; MAX_STRINGFIELD],
    pub model: [c_char; MAX_STRINGFIELD],
    pub level: c_int,
    pub weaponindex: c_int,
    pub flags: c_int,
    pub projectile: [c_char; MAX_STRINGFIELD],
    pub numprojectiles: c_int,
    pub hspread: f32,
    pub vspread: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub recoil: vec3_t,
    pub offset: vec3_t,
    pub angleoffset: vec3_t,
    pub extrazvelocity: f32,
    pub ammoamount: c_int,
    pub ammoindex: c_int,
    pub activate: f32,
    pub reload: f32,
    pub spinup: f32,
    pub spindown: f32,
    /// the used projectile
    pub proj: projectileinfo_t,
}

impl Default for weaponinfo_t {
    fn default() -> Self {
        // SAFETY: all fields are plain scalars / char arrays / a zeroable embedded
        // struct — an all-zero bit pattern is valid, matching C zero-init.
        unsafe { core::mem::zeroed() }
    }
}
