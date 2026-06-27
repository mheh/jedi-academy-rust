//! `bg_weapons.h` — weapon and ammo data types shared by both the client and server.
//!
//! Per the C header: "This crosses both client and server. It could all be crammed
//! into bg_public, but isolation of this type of data is best." Defines the `weapon_t`
//! (`WP_*`) and `ammo_t` (`AMMO_*`) enums and the pointer-free `weaponData_t` /
//! `ammoData_t` stat structs. The `weaponData[]` / `ammoData[]` tables themselves are
//! defined in `bg_weapons.c` → [`bg_weapons`](crate::codemp::game::bg_weapons).
//!
//! All structs here are pointer-free (plain `int`s), so identical layout on 32- and
//! 64-bit; oracle-verified.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// `weapon_t` (bg_weapons.h) — the weapon index. Anonymous C `enum` + `typedef int
/// weapon_t;`, so ported as a `c_int` alias + enumerator consts.
pub type weapon_t = c_int;
pub const WP_NONE: weapon_t = 0;

pub const WP_STUN_BATON: weapon_t = 1;
pub const WP_MELEE: weapon_t = 2;
pub const WP_SABER: weapon_t = 3;
pub const WP_BRYAR_PISTOL: weapon_t = 4;
pub const WP_BLASTER: weapon_t = 5;
pub const WP_DISRUPTOR: weapon_t = 6;
pub const WP_BOWCASTER: weapon_t = 7;
pub const WP_REPEATER: weapon_t = 8;
pub const WP_DEMP2: weapon_t = 9;
pub const WP_FLECHETTE: weapon_t = 10;
pub const WP_ROCKET_LAUNCHER: weapon_t = 11;
pub const WP_THERMAL: weapon_t = 12;
pub const WP_TRIP_MINE: weapon_t = 13;
pub const WP_DET_PACK: weapon_t = 14;
pub const WP_CONCUSSION: weapon_t = 15;
pub const WP_BRYAR_OLD: weapon_t = 16;
pub const WP_EMPLACED_GUN: weapon_t = 17;
pub const WP_TURRET: weapon_t = 18;

//	WP_GAUNTLET,
//	WP_MACHINEGUN,			// Bryar
//	WP_SHOTGUN,				// Blaster
//	WP_GRENADE_LAUNCHER,	// Thermal
//	WP_LIGHTNING,			//
//	WP_RAILGUN,				//
//	WP_GRAPPLING_HOOK,

pub const WP_NUM_WEAPONS: weapon_t = 19;

//anything > this will be considered not player useable
/// `LAST_USEABLE_WEAPON` (bg_weapons.h).
pub const LAST_USEABLE_WEAPON: weapon_t = WP_BRYAR_OLD;

/// `ammo_t` (bg_weapons.h `//# ammo_e`).
pub type ammo_t = c_int;
pub const AMMO_NONE: ammo_t = 0;
pub const AMMO_FORCE: ammo_t = 1; // AMMO_PHASER
pub const AMMO_BLASTER: ammo_t = 2; // AMMO_STARFLEET,
pub const AMMO_POWERCELL: ammo_t = 3; // AMMO_ALIEN,
pub const AMMO_METAL_BOLTS: ammo_t = 4;
pub const AMMO_ROCKETS: ammo_t = 5;
pub const AMMO_EMPLACED: ammo_t = 6;
pub const AMMO_THERMAL: ammo_t = 7;
pub const AMMO_TRIPMINE: ammo_t = 8;
pub const AMMO_DETPACK: ammo_t = 9;
pub const AMMO_MAX: ammo_t = 10;

/// `weaponData_s` / `weaponData_t` (bg_weapons.h) — per-weapon firing stats.
/// Pointer-free (all `int`); identical on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct weaponData_t {
    //	char	classname[32];		// Spawning name
    pub ammoIndex: c_int, // Index to proper ammo slot
    pub ammoLow: c_int,   // Count when ammo is low

    pub energyPerShot: c_int, // Amount of energy used per shot
    pub fireTime: c_int,      // Amount of time between firings
    pub range: c_int,         // Range of weapon

    pub altEnergyPerShot: c_int, // Amount of energy used for alt-fire
    pub altFireTime: c_int,      // Amount of time between alt-firings
    pub altRange: c_int,         // Range of alt-fire

    pub chargeSubTime: c_int,    // ms interval for subtracting ammo during charge
    pub altChargeSubTime: c_int, // above for secondary

    pub chargeSub: c_int,    // amount to subtract during charge on each interval
    pub altChargeSub: c_int, // above for secondary

    pub maxCharge: c_int,    // stop subtracting once charged for this many ms
    pub altMaxCharge: c_int, // above for secondary
}
const _: () = assert!(core::mem::size_of::<weaponData_t>() == 56);
const _: () = assert!(core::mem::align_of::<weaponData_t>() == 4);

/// `ammoData_s` / `ammoData_t` (bg_weapons.h) — per-ammo-type stats. Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ammoData_t {
    //	char	icon[32];	// Name of ammo icon file
    pub max: c_int, // Max amount player can hold of ammo
}
const _: () = assert!(core::mem::size_of::<ammoData_t>() == 4);
const _: () = assert!(core::mem::align_of::<ammoData_t>() == 4);

// extern weaponData_t weaponData[WP_NUM_WEAPONS]; extern ammoData_t ammoData[AMMO_MAX];
// -- the data tables, defined in bg_weapons.c -> bg_weapons.rs.

// Specific weapon information

/// `FIRST_WEAPON` (bg_weapons.h) — this is the first weapon for next and prev weapon switching.
pub const FIRST_WEAPON: weapon_t = WP_BRYAR_PISTOL;
/// `MAX_PLAYER_WEAPONS` (bg_weapons.h) — the max you can switch to and get with the give all.
pub const MAX_PLAYER_WEAPONS: weapon_t = WP_NUM_WEAPONS - 1;

pub const DEFAULT_SHOTGUN_SPREAD: c_int = 700;
pub const DEFAULT_SHOTGUN_COUNT: c_int = 11;

pub const LIGHTNING_RANGE: c_int = 768;

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::size_of;

    /// Parity: the `weaponData_t`/`ammoData_t` `sizeof` and the `weapon_t`/`ammo_t`
    /// enum terminals match the authentic C (read from the real header). All
    /// pointer-free => arch-independent.
    #[test]
    fn bg_weapons_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<weaponData_t>(), jka_bw_sizeof_weaponData_t());
            assert_eq!(size_of::<ammoData_t>(), jka_bw_sizeof_ammoData_t());
            assert_eq!(WP_NONE, jka_bw_WP_NONE());
            assert_eq!(WP_NUM_WEAPONS, jka_bw_WP_NUM_WEAPONS());
            assert_eq!(LAST_USEABLE_WEAPON, jka_bw_LAST_USEABLE_WEAPON());
            assert_eq!(AMMO_NONE, jka_bw_AMMO_NONE());
            assert_eq!(AMMO_MAX, jka_bw_AMMO_MAX());
        }
    }
}
