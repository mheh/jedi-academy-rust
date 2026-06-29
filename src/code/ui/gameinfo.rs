//
// gameinfo.c
//

// *** This file is used by both the game and the user interface ***

// ... and for that reason is excluded from PCH usage for the moment =Ste.

#![allow(non_snake_case)]
#![allow(static_mut_refs)]

use core::ffi::{c_char, c_int, c_void};
use crate::code::ui::gameinfo_h::gameinfo_import_t;

// Local type stubs for weapon/ammo data structures
// (matching oracle/code/game/weapons.h structures)

pub type vec_t = f32;
pub type vec3_t = [f32; 3];

#[repr(C)]
#[derive(Clone, Copy)]
pub struct weaponData_t {
    pub classname: [c_char; 32],    // Spawning name
    pub weaponMdl: [c_char; 64],    // Weapon Model
    pub firingSnd: [c_char; 64],    // Sound made when fired
    pub altFiringSnd: [c_char; 64], // Sound made when alt-fired
    //	pub flashSnd: [c_char; 64],		// Sound made by flash
    //	pub altFlashSnd: [c_char; 64],	// Sound made by an alt-flash
    pub stopSnd: [c_char; 64],      // Sound made when weapon stops firing
    pub chargeSnd: [c_char; 64],    // sound to start when the weapon initiates the charging sequence
    pub altChargeSnd: [c_char; 64], // alt sound to start when the weapon initiates the charging sequence
    pub selectSnd: [c_char; 64],    // the sound to play when this weapon gets selected

    // #ifdef _IMMERSION
    // pub firingFrc: [c_char; 64],
    // pub altFiringFrc: [c_char; 64],
    // pub stopFrc: [c_char; 64],
    // pub chargeFrc: [c_char; 64],
    // pub altChargeFrc: [c_char; 64],
    // pub selectFrc: [c_char; 64],
    // #endif // _IMMERSION

    pub ammoIndex: c_int,     // Index to proper ammo slot
    pub ammoLow: c_int,       // Count when ammo is low

    pub energyPerShot: c_int,     // Amount of energy used per shot
    pub fireTime: c_int,          // Amount of time between firings
    pub range: c_int,             // Range of weapon

    pub altEnergyPerShot: c_int, // Amount of energy used for alt-fire
    pub altFireTime: c_int,      // Amount of time between alt-firings
    pub altRange: c_int,         // Range of alt-fire

    pub weaponIcon: [c_char; 64], // Name of weapon icon file
    pub numBarrels: c_int,        // how many barrels should we expect for this weapon?

    pub missileMdl: [c_char; 64],      // Missile Model
    pub missileSound: [c_char; 64],    // Missile flight sound
    pub missileDlight: f32,            // what is says
    pub missileDlightColor: vec3_t,    // ditto

    pub alt_missileMdl: [c_char; 64],      // Missile Model
    pub alt_missileSound: [c_char; 64],    // Missile sound
    pub alt_missileDlight: f32,            // what is says
    pub alt_missileDlightColor: vec3_t,    // ditto

    pub missileHitSound: [c_char; 64],    // Missile impact sound
    pub altmissileHitSound: [c_char; 64], // alt Missile impact sound
    // #ifndef _USRDLL
    pub func: *mut c_void,
    pub altfunc: *mut c_void,

    pub mMuzzleEffect: [c_char; 64],
    pub mMuzzleEffectID: c_int,
    pub mAltMuzzleEffect: [c_char; 64],
    pub mAltMuzzleEffectID: c_int,
    // #endif
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ammoData_t {
    pub icon: [c_char; 32], // Name of ammo icon file
    pub max: c_int,         // Max amount player can hold of ammo
}

// Weapon enum constants
pub const WP_NONE: c_int = 0;

// Player weapons
pub const WP_SABER: c_int = 1;             // player and NPC weapon
pub const WP_BLASTER_PISTOL: c_int = 2;   // player and NPC weapon
pub const WP_BLASTER: c_int = 3;          // player and NPC weapon
pub const WP_DISRUPTOR: c_int = 4;        // player and NPC weapon
pub const WP_BOWCASTER: c_int = 5;        // NPC weapon - player can pick this up, but never starts with them
pub const WP_REPEATER: c_int = 6;         // NPC weapon - player can pick this up, but never starts with them
pub const WP_DEMP2: c_int = 7;            // NPC weapon - player can pick this up, but never starts with them
pub const WP_FLECHETTE: c_int = 8;        // NPC weapon - player can pick this up, but never starts with them
pub const WP_ROCKET_LAUNCHER: c_int = 9;  // NPC weapon - player can pick this up, but never starts with them
pub const WP_THERMAL: c_int = 10;         // player and NPC weapon
pub const WP_TRIP_MINE: c_int = 11;       // NPC weapon - player can pick this up, but never starts with them
pub const WP_DET_PACK: c_int = 12;        // NPC weapon - player can pick this up, but never starts with them
pub const WP_CONCUSSION: c_int = 13;      // NPC weapon - player can pick this up, but never starts with them

// extras
pub const WP_MELEE: c_int = 14; // player and NPC weapon - Any ol' melee attack

// when in atst
pub const WP_ATST_MAIN: c_int = 15;
pub const WP_ATST_SIDE: c_int = 16;

// These can never be gotten directly by the player
pub const WP_STUN_BATON: c_int = 17; // stupid weapon, should remove

// NPC weapons
pub const WP_BRYAR_PISTOL: c_int = 18; // NPC weapon - player can pick this up, but never starts with them

pub const WP_EMPLACED_GUN: c_int = 19;

pub const WP_BOT_LASER: c_int = 20; // Probe droid	- Laser blast

pub const WP_TURRET: c_int = 21; // turret guns

pub const WP_TIE_FIGHTER: c_int = 22;

pub const WP_RAPID_FIRE_CONC: c_int = 23;

pub const WP_JAWA: c_int = 24;
pub const WP_TUSKEN_RIFLE: c_int = 25;
pub const WP_TUSKEN_STAFF: c_int = 26;
pub const WP_SCEPTER: c_int = 27;
pub const WP_NOGHRI_STICK: c_int = 28;

pub const WP_NUM_WEAPONS: usize = 29;

#[allow(non_upper_case_globals)]
pub const FIRST_WEAPON: c_int = WP_SABER; // this is the first weapon for next and prev weapon switching
#[allow(non_upper_case_globals)]
pub const MAX_PLAYER_WEAPONS: c_int = WP_STUN_BATON; // this is the max you can switch to and get with the give all. - FIXME: it's actually this one *minus* one... why?

// Ammo enum constants
// AMMO_NONE must be first and AMMO_MAX must be last, cause weapon load validates based off of these vals
pub const AMMO_NONE: c_int = 0;
pub const AMMO_FORCE: c_int = 1;        // AMMO_PHASER
pub const AMMO_BLASTER: c_int = 2;      // AMMO_STARFLEET,
pub const AMMO_POWERCELL: c_int = 3;    // AMMO_ALIEN,
pub const AMMO_METAL_BOLTS: c_int = 4;
pub const AMMO_ROCKETS: c_int = 5;
pub const AMMO_EMPLACED: c_int = 6;
pub const AMMO_THERMAL: c_int = 7;
pub const AMMO_TRIPMINE: c_int = 8;
pub const AMMO_DETPACK: c_int = 9;
pub const AMMO_MAX: usize = 10;

//
// Initialization - Read in files and parse into infos
//

extern "C" {
    fn WP_LoadWeaponParms();
}

pub static mut gi: gameinfo_import_t = unsafe { core::mem::zeroed() };

// Intentionally unsafe: raw pointer arithmetic, BSS-like zero-initialization of static arrays
pub static mut weaponData: [weaponData_t; WP_NUM_WEAPONS] = unsafe { core::mem::zeroed() };
pub static mut ammoData: [ammoData_t; AMMO_MAX] = unsafe { core::mem::zeroed() };

/*
===============
GI_Init
===============
*/
pub fn GI_Init(import: *mut gameinfo_import_t) {
    unsafe {
        gi = *import;

        WP_LoadWeaponParms();
    }
}
