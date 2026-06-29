// Filename:-	weapons.h
//
// Note that this is now included from both server and game modules, so don't include any other header files
//	within this one that might break stuff...

use core::ffi::{c_int, c_char, c_void};

// weapon_e
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum weapon_t {
    WP_NONE = 0,

    // Player weapons
    WP_SABER = 1,                // player and NPC weapon
    WP_BLASTER_PISTOL = 2,       // player and NPC weapon
    WP_BLASTER = 3,              // player and NPC weapon
    WP_DISRUPTOR = 4,            // player and NPC weapon
    WP_BOWCASTER = 5,            // NPC weapon - player can pick this up, but never starts with them
    WP_REPEATER = 6,             // NPC weapon - player can pick this up, but never starts with them
    WP_DEMP2 = 7,                // NPC weapon - player can pick this up, but never starts with them
    WP_FLECHETTE = 8,            // NPC weapon - player can pick this up, but never starts with them
    WP_ROCKET_LAUNCHER = 9,      // NPC weapon - player can pick this up, but never starts with them
    WP_THERMAL = 10,             // player and NPC weapon
    WP_TRIP_MINE = 11,           // NPC weapon - player can pick this up, but never starts with them
    WP_DET_PACK = 12,            // NPC weapon - player can pick this up, but never starts with them
    WP_CONCUSSION = 13,          // NPC weapon - player can pick this up, but never starts with them

    //extras
    WP_MELEE = 14,               // player and NPC weapon - Any ol' melee attack

    //when in atst
    WP_ATST_MAIN = 15,
    WP_ATST_SIDE = 16,

    // These can never be gotten directly by the player
    WP_STUN_BATON = 17,          // stupid weapon, should remove

    //NPC weapons
    WP_BRYAR_PISTOL = 18,        // NPC weapon - player can pick this up, but never starts with them

    WP_EMPLACED_GUN = 19,

    WP_BOT_LASER = 20,           // Probe droid	- Laser blast

    WP_TURRET = 21,              // turret guns

    WP_TIE_FIGHTER = 22,

    WP_RAPID_FIRE_CONC = 23,

    WP_JAWA = 24,
    WP_TUSKEN_RIFLE = 25,
    WP_TUSKEN_STAFF = 26,
    WP_SCEPTER = 27,
    WP_NOGHRI_STICK = 28,

    //# #eol
    WP_NUM_WEAPONS = 29,
}

pub const FIRST_WEAPON: c_int = 1;              // this is the first weapon for next and prev weapon switching (WP_SABER)
pub const MAX_PLAYER_WEAPONS: c_int = 17;      // this is the max you can switch to and get with the give all. - FIXME: it's actually this one *minus* one... why? (WP_STUN_BATON)

// AMMO_NONE must be first and AMMO_MAX must be last, cause weapon load validates based off of these vals
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ammo_t {
    AMMO_NONE = 0,
    AMMO_FORCE = 1,              // AMMO_PHASER
    AMMO_BLASTER = 2,            // AMMO_STARFLEET,
    AMMO_POWERCELL = 3,          // AMMO_ALIEN,
    AMMO_METAL_BOLTS = 4,
    AMMO_ROCKETS = 5,
    AMMO_EMPLACED = 6,
    AMMO_THERMAL = 7,
    AMMO_TRIPMINE = 8,
    AMMO_DETPACK = 9,
    AMMO_MAX = 10,
}

#[repr(C)]
pub struct weaponData_t {
    pub classname: [c_char; 32],                // Spawning name
    pub weaponMdl: [c_char; 64],                // Weapon Model
    pub firingSnd: [c_char; 64],                // Sound made when fired
    pub altFiringSnd: [c_char; 64],             // Sound made when alt-fired
    //	pub flashSnd: [c_char; 64],                // Sound made by flash
    //	pub altFlashSnd: [c_char; 64],             // Sound made by an alt-flash
    pub stopSnd: [c_char; 64],                  // Sound made when weapon stops firing
    pub chargeSnd: [c_char; 64],                // sound to start when the weapon initiates the charging sequence
    pub altChargeSnd: [c_char; 64],             // alt sound to start when the weapon initiates the charging sequence
    pub selectSnd: [c_char; 64],                // the sound to play when this weapon gets selected

    // #ifdef _IMMERSION
    pub firingFrc: [c_char; 64],
    pub altFiringFrc: [c_char; 64],
    pub stopFrc: [c_char; 64],
    pub chargeFrc: [c_char; 64],
    pub altChargeFrc: [c_char; 64],
    pub selectFrc: [c_char; 64],
    // #endif // _IMMERSION

    pub ammoIndex: c_int,                       // Index to proper ammo slot
    pub ammoLow: c_int,                         // Count when ammo is low

    pub energyPerShot: c_int,                   // Amount of energy used per shot
    pub fireTime: c_int,                        // Amount of time between firings
    pub range: c_int,                           // Range of weapon

    pub altEnergyPerShot: c_int,                // Amount of energy used for alt-fire
    pub altFireTime: c_int,                     // Amount of time between alt-firings
    pub altRange: c_int,                        // Range of alt-fire

    pub weaponIcon: [c_char; 64],               // Name of weapon icon file
    pub numBarrels: c_int,                      // how many barrels should we expect for this weapon?

    pub missileMdl: [c_char; 64],               // Missile Model
    pub missileSound: [c_char; 64],             // Missile flight sound
    pub missileDlight: f32,                     // what is says
    pub missileDlightColor: [f32; 3],           // ditto

    pub alt_missileMdl: [c_char; 64],           // Missile Model
    pub alt_missileSound: [c_char; 64],         // Missile sound
    pub alt_missileDlight: f32,                 // what is says
    pub alt_missileDlightColor: [f32; 3],       // ditto

    pub missileHitSound: [c_char; 64],          // Missile impact sound
    pub altmissileHitSound: [c_char; 64],       // alt Missile impact sound

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
pub struct ammoData_t {
    pub icon: [c_char; 32],                     // Name of ammo icon file
    pub max: c_int,                             // Max amount player can hold of ammo
}
