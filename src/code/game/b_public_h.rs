#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// Type imports from bstate.h and AI.h modules
use crate::code::game::bstate_h::{bState_t, qboolean, rank_t, usercmd_t, vec2_t, vec3_t};
use crate::code::game::ai_h::AIGroupInfo_t;

// Forward declaration of gentity_t - defined in g_public_h or similar
// This is a stub for the generic entity type
pub struct gentity_t;

// NPCAI flags
pub const NPCAI_CHECK_WEAPON: c_int = 0x00000001;
pub const NPCAI_BURST_WEAPON: c_int = 0x00000002;
pub const NPCAI_MOVING: c_int = 0x00000004;
pub const NPCAI_TOUCHED_GOAL: c_int = 0x00000008;
pub const NPCAI_PUSHED: c_int = 0x00000010;
pub const NPCAI_NO_COLL_AVOID: c_int = 0x00000020;
pub const NPCAI_BLOCKED: c_int = 0x00000040;
pub const NPCAI_SUBBOSS_CHARACTER: c_int = 0x00000080; // Alora, tough reborn
pub const NPCAI_OFF_PATH: c_int = 0x00000100;
pub const NPCAI_IN_SQUADPOINT: c_int = 0x00000200;
pub const NPCAI_STRAIGHT_TO_DESTPOS: c_int = 0x00000400;
pub const NPCAI_HEAVY_MELEE: c_int = 0x00000800; // 4x melee damage, dismemberment
pub const NPCAI_NO_SLOWDOWN: c_int = 0x00001000;
pub const NPCAI_LOST: c_int = 0x00002000; // Can't nav to his goal
pub const NPCAI_SHIELDS: c_int = 0x00004000; // Has shields, borg can adapt
pub const NPCAI_GREET_ALLIES: c_int = 0x00008000; // Say hi to nearby allies
pub const NPCAI_FORM_TELE_NAV: c_int = 0x00010000; // Tells formation people to use nav info to get to
pub const NPCAI_ENROUTE_TO_HOMEWP: c_int = 0x00020000; // Lets us know to run our lostenemyscript when we get to homeWp
pub const NPCAI_MATCHPLAYERWEAPON: c_int = 0x00040000; // Match the player's weapon except when it changes during cinematics
pub const NPCAI_DIE_ON_IMPACT: c_int = 0x00100000; // Next time you crashland, die!
pub const NPCAI_WALKING: c_int = 0x00200000;
pub const NPCAI_STOP_AT_LOS: c_int = 0x00400000; // Stop Running When We Hit LOS
pub const NPCAI_NAV_THROUGH_BREAKABLES: c_int = 0x00800000; // Navigation allows connections through breakable (func_glass, func_breakable or misc_model_breakable)
pub const NPCAI_KNEEL: c_int = 0x01000000; // Kneel befor Zod
pub const NPCAI_FLY: c_int = 0x02000000; // Fly, My Pretty!
pub const NPCAI_FLAMETHROW: c_int = 0x04000000;
pub const NPCAI_ROSH: c_int = 0x08000000; // I am Rosh, when I'm hurt, drop to one knee and wait for Vil or Dasariah to heal me
pub const NPCAI_HEAL_ROSH: c_int = 0x10000000; // Constantly look for NPC with NPC_type of rosh_dark, follow him, heal him if needbe
pub const NPCAI_JUMP: c_int = 0x20000000; // Jump Now
pub const NPCAI_BOSS_CHARACTER: c_int = 0x40000000; // Boss NPC flag for certain immunities/defenses
pub const NPCAI_NO_JEDI_DELAY: c_int = 0x80000000; // Reborn/Jedi don't taunt enemy before attacking

// Script flags
pub const SCF_CROUCHED: c_int = 0x00000001; // Force ucmd.upmove to be -127
pub const SCF_WALKING: c_int = 0x00000002; // Force BUTTON_WALKING to be pressed
pub const SCF_MORELIGHT: c_int = 0x00000004; // NPC will have a minlight of 96
pub const SCF_LEAN_RIGHT: c_int = 0x00000008; // Force rightmove+BUTTON_USE
pub const SCF_LEAN_LEFT: c_int = 0x00000010; // Force leftmove+BUTTON_USE
pub const SCF_RUNNING: c_int = 0x00000020; // Takes off walking button, overrides SCF_WALKING
pub const SCF_ALT_FIRE: c_int = 0x00000040; // Force to use alt-fire when firing
pub const SCF_NO_RESPONSE: c_int = 0x00000080; // NPC will not do generic responses to being used
pub const SCF_FFDEATH: c_int = 0x00000100; // Just tells player_die to run the friendly fire deathscript
pub const SCF_NO_COMBAT_TALK: c_int = 0x00000200; // NPC will not use their generic combat chatter stuff
pub const SCF_CHASE_ENEMIES: c_int = 0x00000400; // NPC chase enemies - FIXME: right now this is synonymous with using combat points... should it be?
pub const SCF_LOOK_FOR_ENEMIES: c_int = 0x00000800; // NPC be on the lookout for enemies
pub const SCF_FACE_MOVE_DIR: c_int = 0x00001000; // NPC face direction it's moving - FIXME: not really implemented right now
pub const SCF_IGNORE_ALERTS: c_int = 0x00002000; // NPC ignore alert events
pub const SCF_DONT_FIRE: c_int = 0x00004000; // NPC won't shoot
pub const SCF_DONT_FLEE: c_int = 0x00008000; // NPC never flees
pub const SCF_FORCED_MARCH: c_int = 0x00010000; // NPC that the player must aim at to make him walk
pub const SCF_NO_GROUPS: c_int = 0x00020000; // NPC cannot alert groups or be part of a group
pub const SCF_FIRE_WEAPON: c_int = 0x00040000; // NPC will fire his (her) weapon
pub const SCF_NO_MIND_TRICK: c_int = 0x00080000; // Not succeptible to mind tricks
pub const SCF_USE_CP_NEAREST: c_int = 0x00100000; // Will use combat point close to it, not next to player or try and flank player
pub const SCF_NO_FORCE: c_int = 0x00200000; // Not succeptible to force powers
pub const SCF_NO_FALLTODEATH: c_int = 0x00400000; // NPC will not scream and tumble and fall to hit death over large drops
pub const SCF_NO_ACROBATICS: c_int = 0x00800000; // Jedi won't jump, roll or cartwheel
pub const SCF_USE_SUBTITLES: c_int = 0x01000000; // Regardless of subtitle setting, this NPC will display subtitles when it speaks lines
pub const SCF_NO_ALERT_TALK: c_int = 0x02000000; // Will not say alert sounds, but still can be woken up by alerts
pub const SCF_NAV_CAN_FLY: c_int = 0x04000000; // Navigation allows connections through air
pub const SCF_FLY_WITH_JET: c_int = 0x08000000; // Must Fly With A Jet
pub const SCF_PILOT: c_int = 0x10000000; // Can pilot a vehicle
pub const SCF_NAV_CAN_JUMP: c_int = 0x20000000; // Can attempt to jump when blocked
pub const SCF_FIRE_WEAPON_NO_ANIM: c_int = 0x40000000; // Fire weapon but don't play weapon firing anim
pub const SCF_SAFE_REMOVE: c_int = 0x80000000; // Remove NPC when it's safe (when player isn't looking)

// #ifdef __DEBUG

// Debug flag definitions

pub const AID_IDLE: c_int = 0x00000000; // Nothing is happening
pub const AID_ACQUIRED: c_int = 0x00000001; // A target has been found
pub const AID_LOST: c_int = 0x00000002; // Alert, but no target is in sight
pub const AID_CONFUSED: c_int = 0x00000004; // Is unable to come up with a course of action
pub const AID_LOSTPATH: c_int = 0x00000008; // Cannot make a valid movement due to lack of connections

// #endif //__DEBUG

// extern qboolean showWaypoints;

#[repr(C)]
pub enum visibility_t {
    VIS_UNKNOWN = 0,
    VIS_NOT = 1,
    VIS_PVS = 2,
    VIS_360 = 3,
    VIS_FOV = 4,
    VIS_SHOOT = 5,
}

#[repr(C)]
pub enum spot_t {
    SPOT_ORIGIN = 0,
    SPOT_CHEST = 1,
    SPOT_HEAD = 2,
    SPOT_HEAD_LEAN = 3,
    SPOT_WEAPON = 4,
    SPOT_LEGS = 5,
    SPOT_GROUND = 6,
}

#[repr(C)]
pub enum lookMode_t {
    LM_ENT = 0,
    LM_INTEREST = 1,
}

#[repr(C)]
pub enum jumpState_t {
    JS_WAITING = 0,
    JS_FACING = 1,
    JS_CROUCHING = 2,
    JS_JUMPING = 3,
    JS_LANDING = 4,
}

#[repr(C)]
pub enum sexType_t {
    SEX_NEUTRAL = 0,
    SEX_MALE = 1,
    SEX_FEMALE = 2,
    SEX_SHEMALE = 3, // what the Hell, ya never know...
}

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct gNPCstats_t {
    // Stats, loaded in, and can be set by scripts
    // AI
    pub aggression: c_int,
    pub aim: c_int,
    pub earshot: f32,
    pub evasion: c_int,
    pub hfov: c_int, // horizontal field of view
    pub intelligence: c_int,
    pub move: c_int,
    pub reactions: c_int, // 1-5, higher is better
    pub shootDistance: f32, // Maximum range- overrides range set for weapon if nonzero
    pub vfov: c_int, // vertical field of view
    pub vigilance: f32,
    pub visrange: f32,
    // Movement
    pub runSpeed: c_int,
    pub walkSpeed: c_int,
    pub yawSpeed: f32, // 1 - whatever, default is 50
    pub health: c_int,
    pub acceleration: c_int,
    // sex
    pub sex: sexType_t, // male, female, etc.
}

pub const MAX_ENEMY_POS_LAG: c_int = 2400;
pub const ENEMY_POS_LAG_INTERVAL: c_int = 100;
pub const ENEMY_POS_LAG_STEPS: c_int = 24; // MAX_ENEMY_POS_LAG / ENEMY_POS_LAG_INTERVAL = 2400 / 100

// !!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!
#[repr(C)]
pub struct gNPC_t {
    // FIXME: Put in playerInfo or something
    pub timeOfDeath: c_int, // FIXME do we really need both of these
    pub touchedByPlayer: *mut gentity_t,

    pub enemyLastVisibility: visibility_t,

    pub aimTime: c_int,
    pub desiredYaw: f32,
    pub desiredPitch: f32,
    pub lockedDesiredYaw: f32,
    pub lockedDesiredPitch: f32,
    pub aimingBeam: *mut gentity_t, // debugging aid

    pub enemyLastSeenLocation: vec3_t,
    pub enemyLastSeenTime: c_int,
    pub enemyLastHeardLocation: vec3_t,
    pub enemyLastHeardTime: c_int,
    pub lastAlertID: c_int, // unique ID

    pub eFlags: c_int,
    pub aiFlags: c_int,

    pub currentAmmo: c_int, // this sucks, need to find a better way
    pub shotTime: c_int,
    pub burstCount: c_int,
    pub burstMin: c_int,
    pub burstMean: c_int,
    pub burstMax: c_int,
    pub burstSpacing: c_int,
    pub attackHold: c_int,
    pub attackHoldTime: c_int,
    pub shootAngles: vec3_t, // Angles to where bot is shooting - fixme: make he torso turn to reflect these

    // extra character info
    pub rank: rank_t, // for pips

    // Behavior state info
    pub behaviorState: bState_t, // determines what actions he should be doing
    pub defaultBehavior: bState_t, // State bot will default to if none other set
    pub tempBehavior: bState_t, // While valid, overrides other behavior

    pub ignorePain: qboolean, // only play pain scripts when take pain

    pub duckDebounceTime: c_int, // Keeps them ducked for a certain time
    pub walkDebounceTime: c_int,
    pub enemyCheckDebounceTime: c_int,
    pub investigateDebounceTime: c_int,
    pub investigateCount: c_int,
    pub investigateGoal: vec3_t,
    pub investigateSoundDebounceTime: c_int,
    pub greetingDebounceTime: c_int, // when we can greet someone next
    pub eventOwner: *mut gentity_t,

    // bState-specific fields
    pub coverTarg: *mut gentity_t,
    pub jumpState: jumpState_t,
    pub followDist: f32,

    // goal, navigation & pathfinding
    pub tempGoal: *mut gentity_t, // used for locational goals (player's last seen/heard position)
    pub goalEntity: *mut gentity_t,
    pub lastGoalEntity: *mut gentity_t,
    pub eventualGoal: *mut gentity_t,
    pub captureGoal: *mut gentity_t, // Where we should try to capture
    pub defendEnt: *mut gentity_t, // Who we're trying to protect
    pub greetEnt: *mut gentity_t, // Who we're greeting
    pub goalTime: c_int, // FIXME: This is never actually used
    pub straightToGoal: qboolean, // move straight at navgoals
    pub distToGoal: f32,
    pub navTime: c_int,
    pub blockingEntNum: c_int,
    pub blockedSpeechDebounceTime: c_int,

    pub homeWp: c_int,
    pub avoidSide: c_int,
    pub leaderAvoidSide: c_int,
    pub lastAvoidSteerSide: c_int,
    pub lastAvoidSteerSideDebouncer: c_int,
    pub group: *mut AIGroupInfo_t,
    pub troop: c_int,

    pub lastPathAngles: vec3_t, // So we know which way to face generally when we stop

    // stats
    pub stats: gNPCstats_t,
    pub aimErrorDebounceTime: c_int,
    pub lastAimErrorYaw: f32,
    pub lastAimErrorPitch: f32,
    pub aimOfs: vec3_t,
    pub currentAim: c_int,
    pub currentAggression: c_int,

    // scriptflags
    pub scriptFlags: c_int, // in b_local.h

    // moveInfo
    pub desiredSpeed: c_int,
    pub currentSpeed: c_int,
    pub last_forwardmove: c_char,
    pub last_rightmove: c_char,
    pub lastClearOrigin: vec3_t,
    pub shoveCount: c_int,

    pub blockedDebounceTime: c_int,
    pub blockedEntity: *mut gentity_t, // The entity That Causes The Current Blockage

    pub blockedTargetPosition: vec3_t, // Where the actor was trying to get TO before blocked
    pub blockedTargetEntity: *mut gentity_t, // Where the actor was trying to get TO before blocked

    // jump info
    pub jumpDest: vec3_t, // Where The Actor Is Trying To Jump TO
    pub jumpTarget: *mut gentity_t, // What Entity The Actor Is Trying To Jump TO
    pub jumpMaxXYDist: f32, // The Minimal Delta On The XY Plane Allowed To Jump To The Dest
    pub jumpMazZDist: f32,
    pub jumpSide: c_int, // Which Side The Last Jump Occured On
    pub jumpTime: c_int, // When The Last Jump Started
    pub jumpBackupTime: c_int, // If Active, Then The Guy Should Backup Before Jumping
    pub jumpNextCheckTime: c_int, // The Minimal Next Time To Check For A Jump

    //
    pub combatPoint: c_int, // NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
    pub lastFailedCombatPoint: c_int, // NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
    pub movementSpeech: c_int, // what to say when you first successfully move
    pub movementSpeechChance: f32, // how likely you are to say it

    // Testing physics at 20fps
    pub nextBStateThink: c_int,
    pub last_ucmd: usercmd_t,

    //
    // JWEIER ADDITIONS START

    pub combatMove: qboolean,
    pub goalRadius: c_int,

    // FIXME: These may be redundant

    /*
    int			weaponTime;		//Time until refire is valid
    int			jumpTime;
    */
    pub pauseTime: c_int, // Time to stand still
    pub standTime: c_int,

    pub localState: c_int, // Tracking information local to entity
    pub squadState: c_int, // Tracking information for team level interaction

    // JWEIER ADDITIONS END
    //

    pub confusionTime: c_int, // Doesn't respond to alerts or pick up enemies (unless shot) until this time is up
    pub charmedTime: c_int, // charmed to enemy team
    pub controlledTime: c_int, // controlled by player
    pub surrenderTime: c_int, // Hands up
    pub kneelTime: c_int, // kneeling (for troopers)

    // Lagging enemy position - FIXME: seems awful wasteful...
    pub enemyLaggedPos: [vec3_t; 24], // ENEMY_POS_LAG_STEPS

    pub watchTarget: *mut gentity_t, // for BS_CINEMATIC, keeps facing this ent

    pub ffireCount: c_int, // sigh... you'd think I'd be able to find a way to do this without having to use 3 int fields, but...
    pub ffireDebounce: c_int,
    pub ffireFadeDebounce: c_int,
}

extern "C" {
    pub fn G_SquadPathsInit();
    pub fn NPC_InitGame();
    pub fn G_LoadBoltOns();
    pub fn Svcmd_NPC_f();
}

/*
extern "C" {
    pub fn Bot_InitGame();
    pub fn Bot_InitPreSpawn();
    pub fn Bot_InitPostSpawn();
    pub fn Bot_Shutdown();
    pub fn Bot_Think(ent: *mut gentity_t, msec: c_int);
    pub fn Bot_Connect(bot: *mut gentity_t, botName: *mut c_char);
    pub fn Bot_Begin(bot: *mut gentity_t);
    pub fn Bot_Disconnect(bot: *mut gentity_t);
    pub fn Svcmd_Bot_f();
    pub fn Nav_ItemSpawn(ent: *mut gentity_t, remaining: c_int);
}
*/

//
// This section should be moved to QFILES.H
//
/*
pub const NAVFILE_ID: c_int = (('I' as c_int) + ('N' as c_int) << 8) + ('A' as c_int) << 16) + ('V' as c_int) << 24);
pub const NAVFILE_VERSION: c_int = 6;

#[repr(C)]
pub struct navheader_t {
    pub id: c_uint,
    pub version: c_uint,
    pub checksum: c_uint,
    pub surfaceCount: c_uint,
    pub edgeCount: c_uint,
}

pub const MAX_SURFACES: c_int = 4096;

pub const NSF_PUSH: c_int = 0x00000001;
pub const NSF_WATERLEVEL1: c_int = 0x00000002;
pub const NSF_WATERLEVEL2: c_int = 0x00000004;
pub const NSF_WATER_NOAIR: c_int = 0x00000008;
pub const NSF_DUCK: c_int = 0x00000010;
pub const NSF_PAIN: c_int = 0x00000020;
pub const NSF_TELEPORTER: c_int = 0x00000040;
pub const NSF_PLATHIGH: c_int = 0x00000080;
pub const NSF_PLATLOW: c_int = 0x00000100;
pub const NSF_DOOR_FLOOR: c_int = 0x00000200;
pub const NSF_DOOR_SHOOT: c_int = 0x00000400;
pub const NSF_DOOR_BUTTON: c_int = 0x00000800;
pub const NSF_BUTTON: c_int = 0x00001000;

#[repr(C)]
pub struct nsurface_t {
    pub origin: vec3_t,
    pub absmin: vec2_t,
    pub absmax: vec2_t,
    pub parm: c_int,
    pub flags: c_uint,
    pub edgeCount: c_uint,
    pub edgeIndex: c_uint,
}

pub const NEF_DUCK: c_int = 0x00000001;
pub const NEF_JUMP: c_int = 0x00000002;
pub const NEF_HOLD: c_int = 0x00000004;
pub const NEF_WALK: c_int = 0x00000008;
pub const NEF_RUN: c_int = 0x00000010;
pub const NEF_NOAIRMOVE: c_int = 0x00000020;
pub const NEF_LEFTGROUND: c_int = 0x00000040;
pub const NEF_PLAT: c_int = 0x00000080;
pub const NEF_FALL1: c_int = 0x00000100;
pub const NEF_FALL2: c_int = 0x00000200;
pub const NEF_DOOR_SHOOT: c_int = 0x00000400;
pub const NEF_DOOR_BUTTON: c_int = 0x00000800;
pub const NEF_BUTTON: c_int = 0x00001000;

#[repr(C)]
pub struct nedge_t {
    pub origin: vec3_t,
    pub absmin: vec2_t,  // region within this surface that is the portal to the other surface
    pub absmax: vec2_t,
    pub surfaceNum: c_int,
    pub flags: c_uint,  // jump, prerequisite button, will take falling damage, etc...
    pub cost: f32,
    pub dirIndex: c_int,
    pub endSpot: vec3_t,
    pub parm: c_int,
}
*/
