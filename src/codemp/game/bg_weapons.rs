//! `bg_weapons.c` — "part of bg_pmove functionality": the shared weapon/ammo data
//! tables (`WP_MuzzlePoint`, `weaponData[]`, `ammoData[]`). The types live in
//! `bg_weapons.h` → [`bg_weapons_h`](crate::codemp::game::bg_weapons_h).
//!
//! These are read-only data tables in practice, so they are `pub static` (immutable),
//! matching their use as constant lookup tables. `weaponData[]` rows are built through
//! a positional `const fn wd()` that mirrors the C's aggregate initializer (the
//! `g_main.c` `cv()` convention), with the C's commented-out alternative values carried
//! as notes. (The `bg_local.h` the C includes is not needed for these tables.)

#![allow(non_upper_case_globals, non_snake_case)]

use crate::codemp::game::bg_weapons_h::*;
use crate::codemp::game::q_shared_h::vec3_t;
use core::ffi::c_int;

// Muzzle point table...
//	Fwd,	right,	up.
/// `WP_MuzzlePoint[WP_NUM_WEAPONS]` (bg_weapons.c) — per-weapon muzzle offset.
pub static WP_MuzzlePoint: [vec3_t; WP_NUM_WEAPONS as usize] = [
    [0.0, 0.0, 0.0],    // WP_NONE,
    [0.0, 8.0, 0.0],    // WP_STUN_BATON,
    [0.0, 8.0, 0.0],    // WP_MELEE,
    [8.0, 16.0, 0.0],   // WP_SABER,
    [12.0, 6.0, -6.0],  // WP_BRYAR_PISTOL,
    [12.0, 6.0, -6.0],  // WP_BLASTER,
    [12.0, 6.0, -6.0],  // WP_DISRUPTOR,
    [12.0, 2.0, -6.0],  // WP_BOWCASTER,
    [12.0, 4.5, -6.0],  // WP_REPEATER,
    [12.0, 6.0, -6.0],  // WP_DEMP2,
    [12.0, 6.0, -6.0],  // WP_FLECHETTE,
    [12.0, 8.0, -4.0],  // WP_ROCKET_LAUNCHER,
    [12.0, 0.0, -4.0],  // WP_THERMAL,
    [12.0, 0.0, -10.0], // WP_TRIP_MINE,
    [12.0, 0.0, -4.0],  // WP_DET_PACK,
    [12.0, 6.0, -6.0],  // WP_CONCUSSION
    [12.0, 6.0, -6.0],  // WP_BRYAR_OLD,
    // The C initializer lists only 17 rows for a [WP_NUM_WEAPONS] (19) array, so C
    // zero-fills the last two -- carried explicitly here:
    [0.0, 0.0, 0.0], // WP_EMPLACED_GUN (implicit {0,0,0})
    [0.0, 0.0, 0.0], // WP_TURRET (implicit {0,0,0})
];

/// Builds one `weaponData_t` row positionally, mirroring the C aggregate initializer
/// (field order: ammoIndex, ammoLow, energyPerShot, fireTime, range, altEnergyPerShot,
/// altFireTime, altRange, chargeSubTime, altChargeSubTime, chargeSub, altChargeSub,
/// maxCharge, altMaxCharge).
#[allow(clippy::too_many_arguments)]
const fn wd(
    ammoIndex: c_int,
    ammoLow: c_int,
    energyPerShot: c_int,
    fireTime: c_int,
    range: c_int,
    altEnergyPerShot: c_int,
    altFireTime: c_int,
    altRange: c_int,
    chargeSubTime: c_int,
    altChargeSubTime: c_int,
    chargeSub: c_int,
    altChargeSub: c_int,
    maxCharge: c_int,
    altMaxCharge: c_int,
) -> weaponData_t {
    weaponData_t {
        ammoIndex,
        ammoLow,
        energyPerShot,
        fireTime,
        range,
        altEnergyPerShot,
        altFireTime,
        altRange,
        chargeSubTime,
        altChargeSubTime,
        chargeSub,
        altChargeSub,
        maxCharge,
        altMaxCharge,
    }
}

/// `weaponData[WP_NUM_WEAPONS]` (bg_weapons.c) — per-weapon firing stats.
#[rustfmt::skip]
pub static weaponData: [weaponData_t; WP_NUM_WEAPONS as usize] = [
    wd(AMMO_NONE,        0,  0,  0,    0,    0,  0,    0,    0,   0,   0, 0,    0, 0),    // WP_NONE
    wd(AMMO_NONE,        5,  0,  400,  8192, 0,  400,  8192, 0,   0,   0, 0,    0, 0),    // WP_STUN_BATON
    wd(AMMO_NONE,        5,  0,  400,  8192, 0,  400,  8192, 0,   0,   0, 0,    0, 0),    // WP_MELEE
    wd(AMMO_NONE,        5,  0,  100,  8192, 0,  100,  8192, 0,   0,   0, 0,    0, 0),    // WP_SABER
    // WP_BRYAR_PISTOL -- C commented-out alts: ammoLow 15, energyPerShot 2, fireTime 400,
    // altEnergyPerShot 2, altFireTime 400, altChargeSubTime 200, altChargeSub 1, altMaxCharge 1500.
    wd(AMMO_BLASTER,     0,  0,  800,  8192, 0,  800,  8192, 0,   0,   0, 0,    0, 0),    // WP_BRYAR_PISTOL
    wd(AMMO_BLASTER,     5,  2,  350,  8192, 3,  150,  8192, 0,   0,   0, 0,    0, 0),    // WP_BLASTER
    wd(AMMO_POWERCELL,   5,  5,  600,  8192, 6,  1300, 8192, 0,   200, 0, 3,    0, 1700), // WP_DISRUPTOR
    wd(AMMO_POWERCELL,   5,  5,  1000, 8192, 5,  750,  8192, 400, 0,   5, 0, 1700, 0),    // WP_BOWCASTER
    wd(AMMO_METAL_BOLTS, 5,  1,  100,  8192, 15, 800,  8192, 0,   0,   0, 0,    0, 0),    // WP_REPEATER
    wd(AMMO_POWERCELL,   5,  8,  500,  8192, 6,  900,  8192, 0,   250, 0, 3,    0, 2100), // WP_DEMP2
    wd(AMMO_METAL_BOLTS, 5,  10, 700,  8192, 15, 800,  8192, 0,   0,   0, 0,    0, 0),    // WP_FLECHETTE
    wd(AMMO_ROCKETS,     5,  1,  900,  8192, 2,  1200, 8192, 0,   0,   0, 0,    0, 0),    // WP_ROCKET_LAUNCHER
    wd(AMMO_THERMAL,     0,  1,  800,  8192, 1,  400,  8192, 0,   0,   0, 0,    0, 0),    // WP_THERMAL
    wd(AMMO_TRIPMINE,    0,  1,  800,  8192, 1,  400,  8192, 0,   0,   0, 0,    0, 0),    // WP_TRIP_MINE
    wd(AMMO_DETPACK,     0,  1,  800,  8192, 0,  400,  8192, 0,   0,   0, 0,    0, 0),    // WP_DET_PACK
    wd(AMMO_METAL_BOLTS, 40, 40, 800,  8192, 50, 1200, 8192, 0,   0,   0, 0,    0, 0),    // WP_CONCUSSION
    wd(AMMO_BLASTER,     15, 2,  400,  8192, 2,  400,  8192, 0,   200, 0, 1,    0, 1500), // WP_BRYAR_OLD
    // WP_EMPLCACED_GUN -- C commented-out alts: ammoIndex AMMO_BLASTER, ammoLow 5, energyPerShot 2, altEnergyPerShot 3.
    wd(0,                0,  0,  100,  8192, 0,  100,  8192, 0,   0,   0, 0,    0, 0),    // WP_EMPLACED_GUN
    // WP_TURRET (NOT ACTUALLY USEABLE BY PLAYER!) -- same commented-out alts as above.
    wd(0,                0,  0,  0,    0,    0,  0,    0,    0,   0,   0, 0,    0, 0),    // WP_TURRET
];

/// `ammoData[AMMO_MAX]` (bg_weapons.c) — max-carry per ammo type.
pub static ammoData: [ammoData_t; AMMO_MAX as usize] = [
    ammoData_t { max: 0 },   // AMMO_NONE
    ammoData_t { max: 100 }, // AMMO_FORCE
    ammoData_t { max: 300 }, // AMMO_BLASTER
    ammoData_t { max: 300 }, // AMMO_POWERCELL
    ammoData_t { max: 300 }, // AMMO_METAL_BOLTS
    ammoData_t { max: 25 },  // AMMO_ROCKETS
    ammoData_t { max: 800 }, // AMMO_EMPLACED
    ammoData_t { max: 10 },  // AMMO_THERMAL
    ammoData_t { max: 10 },  // AMMO_TRIPMINE
    ammoData_t { max: 10 },  // AMMO_DETPACK
];

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::codemp::game::bg_weapons_h::{AMMO_MAX, WP_NUM_WEAPONS};
    use crate::oracle::*;

    /// Value parity: every element of the three weapon/ammo tables matches the
    /// authentic C data (independently transcribed in the oracle TU). `weaponData_t`/
    /// `ammoData_t` derive `PartialEq`; the muzzle `vec3` values are all exactly
    /// representable in `f32`, so `==` is exact.
    #[test]
    fn bg_weapons_tables_match_c() {
        unsafe {
            let n = WP_NUM_WEAPONS as usize;
            let c_muzzle = core::slice::from_raw_parts(jka_bw_muzzle_ptr(), n);
            assert_eq!(&WP_MuzzlePoint[..], c_muzzle);
            let c_wd = core::slice::from_raw_parts(jka_bw_weaponData_ptr(), n);
            assert_eq!(&weaponData[..], c_wd);
            let c_ammo = core::slice::from_raw_parts(jka_bw_ammoData_ptr(), AMMO_MAX as usize);
            assert_eq!(&ammoData[..], c_ammo);
        }
    }
}
