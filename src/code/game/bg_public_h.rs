// bg_public.h -- definitions shared by both the server game and client game modules
// [from oracle/code/game/bg_public.h]

use core::ffi::{c_char, c_int, c_void};

// Dependency modules referenced from original includes
// #include "weapons.h"
// #include "g_items.h"
// #include "teams.h"
// #include "statindex.h"

pub const DEFAULT_GRAVITY: c_int = 800;
pub const GIB_HEALTH: c_int = -40;
pub const ARMOR_PROTECTION: f32 = 0.40;

pub const MAX_ITEMS: c_int = 128;

pub const RANK_TIED_FLAG: c_int = 0x4000;

pub const DEFAULT_SHOTGUN_SPREAD: c_int = 700;
pub const DEFAULT_SHOTGUN_COUNT: c_int = 11;

pub const ITEM_RADIUS: c_int = 15; // item sizes are needed for client side pickup detection

// Player sizes
#[link(name = "game", kind = "static")]
extern "C" {
    pub static mut DEFAULT_MINS_0: f32;
    pub static mut DEFAULT_MINS_1: f32;
    pub static mut DEFAULT_MAXS_0: f32;
    pub static mut DEFAULT_MAXS_1: f32;
    pub static mut DEFAULT_PLAYER_RADIUS: f32;
}

pub const DEFAULT_MINS_2: c_int = -24;
pub const DEFAULT_MAXS_2: c_int = 40; // was 32, but too short for player
pub const CROUCH_MAXS_2: c_int = 16;

pub const ATST_MINS0: c_int = -40;
pub const ATST_MINS1: c_int = -40;
pub const ATST_MINS2: c_int = -24;
pub const ATST_MAXS0: c_int = 40;
pub const ATST_MAXS1: c_int = 40;
pub const ATST_MAXS2: c_int = 248;

// Player viewheights
pub const STANDARD_VIEWHEIGHT_OFFSET: c_int = -4;
//#define	RAVEN_VIEWHEIGHT_ADJ 2
//#define	DEFAULT_VIEWHEIGHT	(26+RAVEN_VIEWHEIGHT_ADJ)
//#define CROUCH_VIEWHEIGHT	12
pub const DEAD_VIEWHEIGHT: c_int = -16;

// Player movement values
pub const MIN_WALK_NORMAL: f32 = 0.7; // can't walk on very steep slopes
pub const JUMP_VELOCITY: c_int = 225; // 270
pub const STEPSIZE: c_int = 18;

/*
===================================================================================

PMOVE MODULE

The pmove code takes a player_state_t and a usercmd_t and generates a new player_state_t
and some other output data.  Used for local prediction on the client game and true
movement on the server game.
===================================================================================
*/

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum pmtype_t {
    PM_NORMAL = 0,   // can accelerate and turn
    PM_NOCLIP = 1,   // noclip movement
    PM_SPECTATOR = 2, // still run into walls
    PM_DEAD = 3,     // no acceleration or turning, but free falling
    PM_FREEZE = 4,   // stuck in place with no control
    PM_INTERMISSION = 5, // no movement or status bar
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum weaponstate_t {
    WEAPON_READY = 0,
    WEAPON_RAISING = 1,
    WEAPON_DROPPING = 2,
    WEAPON_FIRING = 3,
    WEAPON_CHARGING = 4,
    WEAPON_CHARGING_ALT = 5,
    WEAPON_IDLE = 6, // lowered
}

// pmove->pm_flags
pub const PMF_DUCKED: c_int = 1 << 0; // 1
pub const PMF_JUMP_HELD: c_int = 1 << 1; // 2
pub const PMF_JUMPING: c_int = 1 << 2; // 4 - yes, I really am in a jump -- Mike, you may want to come up with something better here since this is really a temp fix.
pub const PMF_BACKWARDS_JUMP: c_int = 1 << 3; // 8 - go into backwards land
pub const PMF_BACKWARDS_RUN: c_int = 1 << 4; // 16 - coast down to backwards run
pub const PMF_TIME_LAND: c_int = 1 << 5; // 32 - pm_time is time before rejump
pub const PMF_TIME_KNOCKBACK: c_int = 1 << 6; // 64 - pm_time is an air-accelerate only time
pub const PMF_TIME_NOFRICTION: c_int = 1 << 7; // 128 - pm_time is a no-friction time
pub const PMF_TIME_WATERJUMP: c_int = 1 << 8; // 256 - pm_time is waterjump
pub const PMF_RESPAWNED: c_int = 1 << 9; // 512 - clear after attack and jump buttons come up
pub const PMF_USEFORCE_HELD: c_int = 1 << 10; // 1024 - for debouncing the button
pub const PMF_JUMP_DUCKED: c_int = 1 << 11; // 2048 - viewheight changes in mid-air
pub const PMF_TRIGGER_PUSHED: c_int = 1 << 12; // 4096 - pushed by a trigger_push or other such thing - cannot force jump and will not take impact damage
pub const PMF_STUCK_TO_WALL: c_int = 1 << 13; // 8192 - grabbing a wall
pub const PMF_SLOW_MO_FALL: c_int = 1 << 14; // 16384 - Fall slower until hit ground
pub const PMF_ATTACK_HELD: c_int = 1 << 15; // 32768 - Holding down the attack button
pub const PMF_ALT_ATTACK_HELD: c_int = 1 << 16; // 65536 - Holding down the alt-attack button
pub const PMF_BUMPED: c_int = 1 << 17; // 131072 - Bumped into something
pub const PMF_FORCE_FOCUS_HELD: c_int = 1 << 18; // 262144 - Holding down the saberthrow/kick button
pub const PMF_FIX_MINS: c_int = 1 << 19; // 524288 - Mins raised for dual forward jump, fix them
pub const PMF_ALL_TIMES: c_int =
    PMF_TIME_WATERJUMP | PMF_TIME_LAND | PMF_TIME_KNOCKBACK | PMF_TIME_NOFRICTION;

#[cfg(target_os = "windows")]
#[cfg(not(feature = "trace_functor_t_defined"))]
mod trace_functor {
    use super::*;
    use core::ffi::c_int;

    // Function objects to replace the function pointers used for trace in pmove_t
    // We can't have default arguments on function pointers, but this allows us to
    // do the same thing with minimal impact elsewhere.
    #[repr(C)]
    pub struct Trace_Functor_t {
        pub trace_func: *const c_void,
        // Placeholder for operator() and operator= implementations
        // Rust doesn't support C++ function call operators in the same way
    }
}

pub const MAXTOUCH: c_int = 32;

// Forward declarations
// typedef struct gentity_s gentity_t;

#[repr(C)]
pub struct pmove_t {
    // state (in / out)
    pub ps: *mut playerState_t,

    // command (in)
    pub cmd: usercmd_t,
    pub tracemask: c_int, // collide against these types of surfaces
    pub debugLevel: c_int, // if set, diagnostic output will be printed
    pub noFootsteps: qboolean, // if the game is setup for no footsteps by the server

    // results (out)
    pub numtouch: c_int,
    pub touchents: [c_int; MAXTOUCH as usize],

    pub useEvent: c_int,

    pub mins: vec3_t,
    pub maxs: vec3_t, // bounding box size

    pub watertype: c_int,
    pub waterlevel: c_int,

    pub xyspeed: f32,
    pub gent: *mut gentity_t, // Pointer to entity in g_entities[]

    // callbacks to test the world
    // these will be different functions during game and cgame
    #[cfg(target_os = "windows")]
    pub trace: Trace_Functor_t,
    #[cfg(not(target_os = "windows"))]
    pub trace: unsafe extern "C" fn(
        *mut trace_t,
        *const vec3_t,
        *const vec3_t,
        *const vec3_t,
        *const vec3_t,
        c_int,
        c_int,
        EG2_Collision,
        c_int,
    ),
    pub pointcontents: unsafe extern "C" fn(*const vec3_t, c_int) -> c_int,
}

// if a full pmove isn't done on the client, you can just update the angles
extern "C" {
    pub fn PM_UpdateViewAngles(ps: *mut playerState_t, cmd: *mut usercmd_t, gent: *mut gentity_t);
    pub fn Pmove(pmove: *mut pmove_t);
}

pub const SETANIM_TORSO: c_int = 1;
pub const SETANIM_LEGS: c_int = 2;
pub const SETANIM_BOTH: c_int = SETANIM_TORSO | SETANIM_LEGS; // 3

pub const SETANIM_FLAG_NORMAL: c_int = 0; // Only set if timer is 0
pub const SETANIM_FLAG_OVERRIDE: c_int = 1; // Override previous
pub const SETANIM_FLAG_HOLD: c_int = 2; // Set the new timer
pub const SETANIM_FLAG_RESTART: c_int = 4; // Allow restarting the anim if playing the same one (weapon fires)
pub const SETANIM_FLAG_HOLDLESS: c_int = 8; // Set the new timer

pub const SETANIM_BLEND_DEFAULT: c_int = 100;

extern "C" {
    pub fn PM_SetAnim(
        pm: *mut pmove_t,
        setAnimParts: c_int,
        anim: c_int,
        setAnimFlags: c_int,
        blendTime: c_int,
    );
    pub fn PM_SetAnimFinal(
        torsoAnim: *mut c_int,
        legsAnim: *mut c_int,
        type_: c_int,
        anim: c_int,
        priority: c_int,
        torsoAnimTimer: *mut c_int,
        legsAnimTimer: *mut c_int,
        gent: *mut gentity_t,
        blendTime: c_int,
    );
}

//===================================================================================

// player_state->persistant[] indexes
// these fields are the only part of player_state that isn't
// cleared on respawn
//
//  NOTE!!! Even though this is an enum, the array that contains these uses #define MAX_PERSISTANT 16 in q_shared.h,
//		so be careful how many you add since it'll just overflow without telling you -slc
//
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum persEnum_t {
    PERS_SCORE = 0,                  // !!! MUST NOT CHANGE, SERVER AND GAME BOTH REFERENCE !!!
    PERS_HITS = 1,                   // total points damage inflicted so damage beeps can sound on change
    PERS_TEAM = 2,
    PERS_SPAWN_COUNT = 3,            // incremented every respawn
    //	PERS_REWARD_COUNT,				// incremented for each reward sound
    PERS_ATTACKER = 4,               // clientnum of last damage inflicter
    PERS_KILLED = 5,                 // count of the number of times you died

    PERS_ACCURACY_SHOTS = 6,         // scoreboard - number of player shots
    PERS_ACCURACY_HITS = 7,          // scoreboard - number of player shots that hit an enemy
    PERS_ENEMIES_KILLED = 8,         // scoreboard - number of enemies player killed
    PERS_TEAMMATES_KILLED = 9, // scoreboard - number of teammates killed
}

// entityState_t->eFlags
pub const EF_HELD_BY_SAND_CREATURE: u32 = 0x00000001; // In a sand creature's mouth
pub const EF_HELD_BY_RANCOR: u32 = 0x00000002; // Being held by Rancor
pub const EF_TELEPORT_BIT: u32 = 0x00000004; // toggled every time the origin abruptly changes
pub const EF_SHADER_ANIM: u32 = 0x00000008; // Animating shader (by s.frame)
pub const EF_BOUNCE: u32 = 0x00000010; // for missiles
pub const EF_BOUNCE_HALF: u32 = 0x00000020; // for missiles
pub const EF_MISSILE_STICK: u32 = 0x00000040; // missiles that stick to the wall.
pub const EF_NODRAW: u32 = 0x00000080; // may have an event, but no model (unspawned items)
pub const EF_FIRING: u32 = 0x00000100; // for lightning gun
pub const EF_ALT_FIRING: u32 = 0x00000200; // for alt-fires, mostly for lightning guns though
pub const EF_VEH_BOARDING: u32 = 0x00000400; // Whether a vehicle is being boarded or not.
pub const EF_AUTO_SIZE: u32 = 0x00000800; // CG_Ents will create the mins & max itself based on model bounds
pub const EF_BOUNCE_SHRAPNEL: u32 = 0x00001000; // special shrapnel flag
pub const EF_USE_ANGLEDELTA: u32 = 0x00002000; // Not used.
pub const EF_ANIM_ALLFAST: u32 = 0x00004000; // automatically cycle through all frames at 10hz
pub const EF_ANIM_ONCE: u32 = 0x00008000; // cycle through all frames just once then stop
pub const EF_HELD_BY_WAMPA: u32 = 0x00010000; // being held by the Wampa
pub const EF_PROX_TRIP: u32 = 0x00020000; // Proximity trip mine has been activated
pub const EF_LOCKED_TO_WEAPON: u32 = 0x00040000; // When we use an emplaced weapon, we turn this on to lock us to that weapon

// rest not sent over net?

pub const EF_PERMANENT: u32 = 0x00080000; // this entity is permanent and is never updated (sent only in the game state)
pub const EF_SPOTLIGHT: u32 = 0x00100000; // Your lights are on...
pub const EF_PLANTED_CHARGE: u32 = 0x00200000; // For detpack charge
pub const EF_POWERING_ROSH: u32 = 0x00400000; // Only for Twins powering up Rosh
pub const EF_FORCE_VISIBLE: u32 = 0x00800000; // Always visible with force sight
pub const EF_IN_ATST: u32 = 0x01000000; // Driving an ATST
pub const EF_DISINTEGRATION: u32 = 0x02000000; // Disruptor effect
pub const EF_LESS_ATTEN: u32 = 0x04000000; // Use less sound attenuation (louder even when farther).
pub const EF_JETPACK_ACTIVE: u32 = 0x08000000; // Not used
pub const EF_DISABLE_SHADER_ANIM: u32 = 0x10000000; // Normally shader animation chugs along, but movers can force shader animation to be on frame 1
pub const EF_FORCE_GRIPPED: u32 = 0x20000000; // Force gripped effect
pub const EF_FORCE_DRAINED: u32 = 0x40000000; // Force drained effect
pub const EF_BLOCKED_MOVER: u32 = 0x80000000; // for movers that are blocked - shared with previous

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum powerup_t {
    PW_NONE = 0,
    PW_QUAD = 1,              // This can go away
    PW_BATTLESUIT = 2,
    PW_HASTE = 3,             // This can go away
    PW_CLOAKED = 4,
    PW_UNCLOAKING = 5,
    PW_DISRUPTION = 6,
    PW_GALAK_SHIELD = 7,
    //	PW_WEAPON_OVERCHARGE,
    PW_SEEKER = 8,
    PW_SHOCKED = 9,           // electricity effect
    PW_DRAINED = 10,          // drain effect
    PW_DISINT_2 = 11,         // ghost
    PW_INVINCIBLE = 12,
    PW_FORCE_PUSH = 13,
    PW_FORCE_PUSH_RHAND = 14,

    PW_NUM_POWERUPS = 15,
}

pub const PW_REMOVE_AT_DEATH: c_int = (1 << 1)
    | (1 << 2)
    | (1 << 3)
    | (1 << 4)
    | (1 << 5)
    | (1 << 5)
    | (1 << 7)
    | (1 << 11)
    | (1 << 12)
    | (1 << 8);

// entityState_t->event values
// entity events are for effects that take place relative
// to an existing entities origin.  Very network efficient.

// two bits at the top of the entityState->event field
// will be incremented with each change in the event so
// that an identical event started twice in a row can
// be distinguished.  And off the value with ~EV_EVENT_BITS
// to retrieve the actual event number
pub const EV_EVENT_BIT1: c_int = 0x00000100;
pub const EV_EVENT_BIT2: c_int = 0x00000200;
pub const EV_EVENT_BITS: c_int = EV_EVENT_BIT1 | EV_EVENT_BIT2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum entity_event_t {
    EV_NONE = 0,

    EV_FOOTSTEP = 1,
    EV_FOOTSTEP_METAL = 2,
    EV_FOOTSPLASH = 3,
    EV_FOOTWADE = 4,
    EV_SWIM = 5,

    EV_STEP_4 = 6,
    EV_STEP_8 = 7,
    EV_STEP_12 = 8,
    EV_STEP_16 = 9,

    EV_FALL_SHORT = 10,
    EV_FALL_MEDIUM = 11,
    EV_FALL_FAR = 12,

    EV_JUMP = 13,
    EV_ROLL = 14,
    EV_WATER_TOUCH = 15,  // foot touches
    EV_WATER_LEAVE = 16,  // foot leaves
    EV_WATER_UNDER = 17,  // head touches
    EV_WATER_CLEAR = 18,  // head leaves
    EV_WATER_GURP1 = 19,  // need air 1
    EV_WATER_GURP2 = 20,  // need air 2
    EV_WATER_DROWN = 21,  // drowned
    EV_LAVA_TOUCH = 22,   // foot touches
    EV_LAVA_LEAVE = 23,   // foot leaves
    EV_LAVA_UNDER = 24,   // head touches

    EV_ITEM_PICKUP = 25,

    EV_NOAMMO = 26,
    EV_CHANGE_WEAPON = 27,
    EV_FIRE_WEAPON = 28,
    EV_ALT_FIRE = 29,
    EV_POWERUP_SEEKER_FIRE = 30,
    EV_POWERUP_BATTLESUIT = 31,
    EV_USE = 32,

    EV_REPLICATOR = 33,

    EV_BATTERIES_CHARGED = 34,

    EV_GRENADE_BOUNCE = 35,        // eventParm will be the soundindex
    EV_MISSILE_STICK = 36,         // eventParm will be the soundindex

    EV_BMODEL_SOUND = 37,
    EV_GENERAL_SOUND = 38,
    EV_GLOBAL_SOUND = 39,          // no attenuation

    #[cfg(feature = "immersion")]
    EV_ENTITY_FORCE = 40,
    #[cfg(feature = "immersion")]
    EV_AREA_FORCE = 41,
    #[cfg(feature = "immersion")]
    EV_GLOBAL_FORCE = 42,
    #[cfg(feature = "immersion")]
    EV_FORCE_STOP = 43,

    #[cfg(feature = "immersion")]
    EV_PLAY_EFFECT = 44,
    #[cfg(feature = "immersion")]
    EV_PLAY_MUZZLE_EFFECT = 45,
    #[cfg(feature = "immersion")]
    EV_STOP_EFFECT = 46,

    #[cfg(not(feature = "immersion"))]
    EV_PLAY_EFFECT = 40,
    #[cfg(not(feature = "immersion"))]
    EV_PLAY_MUZZLE_EFFECT = 41,
    #[cfg(not(feature = "immersion"))]
    EV_STOP_EFFECT = 42,

    #[cfg(not(feature = "immersion"))]
    EV_TARGET_BEAM_DRAW = 43,

    #[cfg(not(feature = "immersion"))]
    EV_DISRUPTOR_MAIN_SHOT = 44,
    #[cfg(not(feature = "immersion"))]
    EV_DISRUPTOR_SNIPER_SHOT = 45,
    #[cfg(not(feature = "immersion"))]
    EV_DISRUPTOR_SNIPER_MISS = 46,

    #[cfg(not(feature = "immersion"))]
    EV_DEMP2_ALT_IMPACT = 47,
    //NEW for JKA weapons:
    #[cfg(not(feature = "immersion"))]
    EV_CONC_ALT_SHOT = 48,
    #[cfg(not(feature = "immersion"))]
    EV_CONC_ALT_MISS = 49,
    //END JKA weapons

    #[cfg(feature = "immersion")]
    EV_TARGET_BEAM_DRAW = 47,

    #[cfg(feature = "immersion")]
    EV_DISRUPTOR_MAIN_SHOT = 48,
    #[cfg(feature = "immersion")]
    EV_DISRUPTOR_SNIPER_SHOT = 49,
    #[cfg(feature = "immersion")]
    EV_DISRUPTOR_SNIPER_MISS = 50,

    #[cfg(feature = "immersion")]
    EV_DEMP2_ALT_IMPACT = 51,
    //NEW for JKA weapons:
    #[cfg(feature = "immersion")]
    EV_CONC_ALT_SHOT = 52,
    #[cfg(feature = "immersion")]
    EV_CONC_ALT_MISS = 53,
    //END JKA weapons

    EV_PAIN = 50,
    EV_DEATH1 = 51,
    EV_DEATH2 = 52,
    EV_DEATH3 = 53,

    EV_MISSILE_HIT = 54,
    EV_MISSILE_MISS = 55,

    EV_DISINTEGRATION = 56,

    EV_ANGER1 = 57,          //Say when acquire an enemy when didn't have one before
    EV_ANGER2 = 58,
    EV_ANGER3 = 59,

    EV_VICTORY1 = 60,        //Say when killed an enemy
    EV_VICTORY2 = 61,
    EV_VICTORY3 = 62,

    EV_CONFUSE1 = 63,        //Say when confused
    EV_CONFUSE2 = 64,
    EV_CONFUSE3 = 65,

    EV_PUSHED1 = 66,         //Say when pushed
    EV_PUSHED2 = 67,
    EV_PUSHED3 = 68,

    EV_CHOKE1 = 69,          //Say when choking
    EV_CHOKE2 = 70,
    EV_CHOKE3 = 71,

    EV_FFWARN = 72,          //ffire founds
    EV_FFTURN = 73,
    //extra sounds for ST
    EV_CHASE1 = 74,
    EV_CHASE2 = 75,
    EV_CHASE3 = 76,
    EV_COVER1 = 77,
    EV_COVER2 = 78,
    EV_COVER3 = 79,
    EV_COVER4 = 80,
    EV_COVER5 = 81,
    EV_DETECTED1 = 82,
    EV_DETECTED2 = 83,
    EV_DETECTED3 = 84,
    EV_DETECTED4 = 85,
    EV_DETECTED5 = 86,
    EV_LOST1 = 87,
    EV_OUTFLANK1 = 88,
    EV_OUTFLANK2 = 89,
    EV_ESCAPING1 = 90,
    EV_ESCAPING2 = 91,
    EV_ESCAPING3 = 92,
    EV_GIVEUP1 = 93,
    EV_GIVEUP2 = 94,
    EV_GIVEUP3 = 95,
    EV_GIVEUP4 = 96,
    EV_LOOK1 = 97,
    EV_LOOK2 = 98,
    EV_SIGHT1 = 99,
    EV_SIGHT2 = 100,
    EV_SIGHT3 = 101,
    EV_SOUND1 = 102,
    EV_SOUND2 = 103,
    EV_SOUND3 = 104,
    EV_SUSPICIOUS1 = 105,
    EV_SUSPICIOUS2 = 106,
    EV_SUSPICIOUS3 = 107,
    EV_SUSPICIOUS4 = 108,
    EV_SUSPICIOUS5 = 109,
    //extra sounds for Jedi
    EV_COMBAT1 = 110,
    EV_COMBAT2 = 111,
    EV_COMBAT3 = 112,
    EV_JDETECTED1 = 113,
    EV_JDETECTED2 = 114,
    EV_JDETECTED3 = 115,
    EV_TAUNT1 = 116,
    EV_TAUNT2 = 117,
    EV_TAUNT3 = 118,
    EV_JCHASE1 = 119,
    EV_JCHASE2 = 120,
    EV_JCHASE3 = 121,
    EV_JLOST1 = 122,
    EV_JLOST2 = 123,
    EV_JLOST3 = 124,
    EV_DEFLECT1 = 125,
    EV_DEFLECT2 = 126,
    EV_DEFLECT3 = 127,
    EV_GLOAT1 = 128,
    EV_GLOAT2 = 129,
    EV_GLOAT3 = 130,
    EV_PUSHFAIL = 131,

    EV_USE_ITEM = 132,

    EV_USE_INV_BINOCULARS = 133,
    EV_USE_INV_BACTA = 134,
    EV_USE_INV_SEEKER = 135,
    EV_USE_INV_LIGHTAMP_GOGGLES = 136,
    EV_USE_INV_SENTRY = 137,

    EV_USE_FORCE = 138,

    EV_DRUGGED = 139,        // hit by an interrogator

    EV_DEBUG_LINE = 140,
    EV_KOTHOS_BEAM = 141,
}

#[repr(C, packed(1))]
pub struct animation_t {
    pub firstFrame: u16,
    pub numFrames: u16,
    pub frameLerp: i16, // msec between frames
    // initial lerp is abs(frameLerp)
    pub loopFrames: i8,   // 0 to numFrames, -1 = no loop
    pub glaIndex: u8,
}

#[cfg(target_os = "windows")]
pub const MAX_ANIM_FILES: c_int = 10; // Feel free to re-increase this if necessary, worst case right now is vjun3 -> 9
#[cfg(not(target_os = "windows"))]
pub const MAX_ANIM_FILES: c_int = 16;

pub const MAX_ANIM_EVENTS: c_int = 300;

// size of Anim eventData array...
pub const MAX_RANDOM_ANIM_SOUNDS: c_int = 8;
pub const AED_ARRAY_SIZE: c_int = MAX_RANDOM_ANIM_SOUNDS + 3;

// indices for AEV_SOUND data
pub const AED_SOUNDINDEX_START: c_int = 0;
pub const AED_SOUNDINDEX_END: c_int = MAX_RANDOM_ANIM_SOUNDS - 1;
pub const AED_SOUND_NUMRANDOMSNDS: c_int = MAX_RANDOM_ANIM_SOUNDS;
pub const AED_SOUND_PROBABILITY: c_int = MAX_RANDOM_ANIM_SOUNDS + 1;

// indices for AEV_SOUNDCHAN data
pub const AED_SOUNDCHANNEL: c_int = MAX_RANDOM_ANIM_SOUNDS + 2;

// indices for AEV_FOOTSTEP data
pub const AED_FOOTSTEP_TYPE: c_int = 0;
pub const AED_FOOTSTEP_PROBABILITY: c_int = 1;

// indices for AEV_EFFECT data
pub const AED_EFFECTINDEX: c_int = 0;
pub const AED_BOLTINDEX: c_int = 1;
pub const AED_EFFECT_PROBABILITY: c_int = 2;
pub const AED_MODELINDEX: c_int = 3;

// indices for AEV_FIRE data
pub const AED_FIRE_ALT: c_int = 0;
pub const AED_FIRE_PROBABILITY: c_int = 1;

// indices for AEV_MOVE data
pub const AED_MOVE_FWD: c_int = 0;
pub const AED_MOVE_RT: c_int = 1;
pub const AED_MOVE_UP: c_int = 2;

// indices for AEV_SABER_SWING data
pub const AED_SABER_SWING_SABERNUM: c_int = 0;
pub const AED_SABER_SWING_TYPE: c_int = 1;
pub const AED_SABER_SWING_PROBABILITY: c_int = 2;

// indices for AEV_SABER_SPIN data
pub const AED_SABER_SPIN_SABERNUM: c_int = 0;
pub const AED_SABER_SPIN_TYPE: c_int = 1; // 0 = saberspinoff, 1 = saberspin, 2-4 = saberspin1-saberspin3
pub const AED_SABER_SPIN_PROBABILITY: c_int = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum animEventType_t {
    // NOTENOTE:  Be sure to update animEventTypeTable and ParseAnimationEvtBlock(...) if you change this enum list!
    AEV_NONE = 0,
    AEV_SOUND = 1,              // # animID AEV_SOUND framenum soundpath randomlow randomhi chancetoplay
    AEV_FOOTSTEP = 2,           // # animID AEV_FOOTSTEP framenum footstepType chancetoplay
    AEV_EFFECT = 3,             // # animID AEV_EFFECT framenum effectpath boltName chancetoplay
    AEV_FIRE = 4,               // # animID AEV_FIRE framenum altfire chancetofire
    AEV_MOVE = 5,               // # animID AEV_MOVE framenum forwardpush rightpush uppush
    AEV_SOUNDCHAN = 6,          // # animID AEV_SOUNDCHAN framenum CHANNEL soundpath randomlow randomhi chancetoplay
    AEV_SABER_SWING = 7,        // # animID AEV_SABER_SWING framenum CHANNEL randomlow randomhi chancetoplay
    AEV_SABER_SPIN = 8,         // # animID AEV_SABER_SPIN framenum CHANNEL chancetoplay
    AEV_NUM_AEV = 9,
}

#[cfg(target_os = "windows")]
#[repr(C, packed(1))]
pub struct animevent_t {
    pub eventType: animEventType_t,
    pub modelOnly: i16,      // event is specific to a modelname to skeleton
    pub glaIndex: u16,
    pub keyFrame: u16,       // Frame to play event on
    pub eventData: [i16; AED_ARRAY_SIZE as usize], // Unique IDs, can be soundIndex of sound file to play OR effect index or footstep type, etc.
    pub stringData: *mut c_char, // we allow storage of one string, temporarily (in case we have to look up an index later, then make sure to set stringData to NULL so we only do the look-up once)
}

#[cfg(not(target_os = "windows"))]
#[repr(C)]
pub struct animevent_t {
    pub eventType: animEventType_t,
    pub modelOnly: i16,      // event is specific to a modelname to skeleton
    pub glaIndex: u16,
    pub keyFrame: u16,       // Frame to play event on
    pub eventData: [i16; AED_ARRAY_SIZE as usize], // Unique IDs, can be soundIndex of sound file to play OR effect index or footstep type, etc.
    pub stringData: *mut c_char, // we allow storage of one string, temporarily (in case we have to look up an index later, then make sure to set stringData to NULL so we only do the look-up once)
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum footstepType_t {
    FOOTSTEP_R = 0,
    FOOTSTEP_L = 1,
    FOOTSTEP_HEAVY_R = 2,
    FOOTSTEP_HEAVY_L = 3,
    NUM_FOOTSTEP_TYPES = 4,
}

// means of death
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum meansOfDeath_t {
    MOD_UNKNOWN = 0,

    // weapons
    MOD_SABER = 1,
    MOD_BRYAR = 2,
    MOD_BRYAR_ALT = 3,
    MOD_BLASTER = 4,
    MOD_BLASTER_ALT = 5,
    MOD_DISRUPTOR = 6,
    MOD_SNIPER = 7,
    MOD_BOWCASTER = 8,
    MOD_BOWCASTER_ALT = 9,
    MOD_REPEATER = 10,
    MOD_REPEATER_ALT = 11,
    MOD_DEMP2 = 12,
    MOD_DEMP2_ALT = 13,
    MOD_FLECHETTE = 14,
    MOD_FLECHETTE_ALT = 15,
    MOD_ROCKET = 16,
    MOD_ROCKET_ALT = 17,
    // NEW for JKA weapons:
    MOD_CONC = 18,
    MOD_CONC_ALT = 19,
    // END JKA weapons.
    MOD_THERMAL = 20,
    MOD_THERMAL_ALT = 21,
    MOD_DETPACK = 22,
    MOD_LASERTRIP = 23,
    MOD_LASERTRIP_ALT = 24,
    MOD_MELEE = 25,
    MOD_SEEKER = 26,
    MOD_FORCE_GRIP = 27,
    MOD_FORCE_LIGHTNING = 28,
    MOD_FORCE_DRAIN = 29,
    MOD_EMPLACED = 30,

    // world / generic
    MOD_ELECTROCUTE = 31,
    MOD_EXPLOSIVE = 32,
    MOD_EXPLOSIVE_SPLASH = 33,
    MOD_KNOCKOUT = 34,
    MOD_ENERGY = 35,
    MOD_ENERGY_SPLASH = 36,
    MOD_WATER = 37,
    MOD_SLIME = 38,
    MOD_LAVA = 39,
    MOD_CRUSH = 40,
    MOD_IMPACT = 41,
    MOD_FALLING = 42,
    MOD_SUICIDE = 43,
    MOD_TRIGGER_HURT = 44,
    MOD_GAS = 45,

    NUM_MODS = 46,
}

//---------------------------------------------------------

// gitem_t->type
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum itemType_t {
    IT_BAD = 0,
    IT_WEAPON = 1,
    IT_AMMO = 2,
    IT_ARMOR = 3,
    IT_HEALTH = 4,
    IT_HOLDABLE = 5,
    IT_BATTERY = 6,
    IT_HOLOCRON = 7,
}

#[repr(C)]
pub struct gitem_t {
    pub classname: *mut c_char,    // spawning name
    pub pickup_sound: *mut c_char,
    pub world_model: *mut c_char,

    pub icon: *mut c_char,

    pub quantity: c_int,           // for ammo how much, or duration of powerup
    pub giType: itemType_t,        // IT_* flags

    pub giTag: c_int,

    pub precaches: *mut c_char,    // string of all models and images this item will use
    pub sounds: *mut c_char,       // string of all sounds this item will use
    pub mins: vec3_t,              // Bbox
    pub maxs: vec3_t,              // Bbox
    #[cfg(feature = "immersion")]
    pub pickup_force: *mut c_char,
    #[cfg(feature = "immersion")]
    pub forces: *mut c_char,
}

// included in both the game dll and the client
extern "C" {
    pub static bg_itemlist: [gitem_t; 0];
    pub static bg_numItems: c_int;
}

//==============================================================================

/*
typedef struct ginfoitem_s
{
    char				*infoString;// Text message
    vec3_t				color;		// Text color

} ginfoitem_t;
*/

//==============================================================================

extern "C" {
    pub static weaponData: [weaponData_t; 0];
}

//==============================================================================

extern "C" {
    pub static ammoData: [ammoData_t; 0];
}

//==============================================================================

extern "C" {
    pub fn FindItem(className: *const c_char) -> *mut gitem_t;
    pub fn FindItemForWeapon(weapon: weapon_t) -> *mut gitem_t;
    pub fn FindItemForInventory(inv: c_int) -> *mut gitem_t;
}

#[inline]
pub fn ITEM_INDEX(x: *const gitem_t) -> isize {
    unsafe {
        (x as *const u8).offset_from(core::ptr::addr_of!(bg_itemlist) as *const u8) / core::mem::size_of::<gitem_t>() as isize
    }
}

extern "C" {
    pub fn BG_CanItemBeGrabbed(ent: *const entityState_t, ps: *const playerState_t) -> qboolean;
}

// content masks
pub const MASK_ALL: c_int = -1;
pub const MASK_SOLID: c_int = CONTENTS_SOLID | CONTENTS_TERRAIN;
pub const MASK_PLAYERSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;
pub const MASK_NPCSOLID: c_int = CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;
pub const MASK_DEADSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_TERRAIN;
pub const MASK_WATER: c_int = CONTENTS_WATER | CONTENTS_LAVA | CONTENTS_SLIME;
pub const MASK_OPAQUE: c_int = CONTENTS_OPAQUE | CONTENTS_SLIME | CONTENTS_LAVA; // was CONTENTS_SOLID, not CONTENTS_OPAQUE...?
/*
Ghoul2 Insert Start
*/
pub const MASK_SHOT: c_int =
    CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE | CONTENTS_SHOTCLIP | CONTENTS_TERRAIN;
/*
Ghoul2 Insert End
*/

//
// entityState_t->eType
//
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum entityType_t {
    ET_GENERAL = 0,
    ET_PLAYER = 1,
    ET_ITEM = 2,
    ET_MISSILE = 3,
    ET_MOVER = 4,
    ET_BEAM = 5,
    ET_PORTAL = 6,
    ET_SPEAKER = 7,
    ET_PUSH_TRIGGER = 8,
    ET_TELEPORT_TRIGGER = 9,
    ET_INVISIBLE = 10,
    ET_THINKER = 11,
    ET_CLOUD = 12, // dumb
    ET_TERRAIN = 13,

    ET_EVENTS = 14, // any of the EV_* events can be added freestanding
                    // by setting eType to ET_EVENTS + eventNum
                    // this avoids having to set eFlags and eventNum
}

extern "C" {
    pub fn EvaluateTrajectory(
        tr: *const trajectory_t,
        atTime: c_int,
        result: *mut vec3_t,
    );
    pub fn EvaluateTrajectoryDelta(
        tr: *const trajectory_t,
        atTime: c_int,
        result: *mut vec3_t,
    );

    pub fn AddEventToPlayerstate(newEvent: c_int, eventParm: c_int, ps: *mut playerState_t);
    pub fn CurrentPlayerstateEvent(ps: *mut playerState_t) -> c_int;

    pub fn PlayerStateToEntityState(ps: *mut playerState_t, s: *mut entityState_t);

    pub fn BG_PlayerTouchesItem(
        ps: *mut playerState_t,
        item: *mut entityState_t,
        atTime: c_int,
    ) -> qboolean;
}

// ===== Forward declarations for dependencies =====
// These are placeholder types for external dependencies that are defined elsewhere

#[repr(C)]
pub struct playerState_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct usercmd_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct gentity_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct EG2_Collision {
    _private: [u8; 0],
}

#[repr(C)]
pub struct trajectory_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct entityState_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct weaponData_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ammoData_t {
    _private: [u8; 0],
}

pub type weapon_t = c_int;
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// Content flags (from shared headers, referenced by MASK macros)
pub const CONTENTS_SOLID: c_int = 1;
pub const CONTENTS_TERRAIN: c_int = 2;
pub const CONTENTS_PLAYERCLIP: c_int = 4;
pub const CONTENTS_BODY: c_int = 8;
pub const CONTENTS_MONSTERCLIP: c_int = 16;
pub const CONTENTS_WATER: c_int = 32;
pub const CONTENTS_LAVA: c_int = 64;
pub const CONTENTS_SLIME: c_int = 128;
pub const CONTENTS_OPAQUE: c_int = 256;
pub const CONTENTS_CORPSE: c_int = 512;
pub const CONTENTS_SHOTCLIP: c_int = 1024;
