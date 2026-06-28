//! Header types from `ai_main.h` — the bot-AI state backbone.
//!
//! The big `bot_state_t` and its sub-structs (`botattachment_t`, `nodeobject_t`,
//! `boteventtracker_t`, `botskills_t`), plus the bot-AI `#define` consts and the
//! waypoint/level flag bitmasks. Faithful 1:1 port; `#[repr(C)]` throughout so the
//! layout matches the engine ABI. Reused types (`playerState_t`, `usercmd_t`,
//! `bot_settings_t`, `gentity_t`, `wpobject_t`, ...) are imported, never redefined.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_char;

use crate::codemp::game::bg_weapons_h::WP_NUM_WEAPONS;
use crate::codemp::game::g_local::{bot_settings_t, gentity_t};
use crate::codemp::game::q_shared_h::{
    playerState_t, qboolean, usercmd_t, vec3_t, wpobject_t, MAX_PS_EVENTS,
};

pub const DEFAULT_FORCEPOWERS: &str = "5-1-000000000000000000";

// #define FORCEJUMP_INSTANTMETHOD 1   // (commented out in C — left disabled)

// MAX_CHAT_BUFFER_SIZE is 256 under _XBOX, else 8192. We build the non-XBOX value.
pub const MAX_CHAT_BUFFER_SIZE: i32 = 8192; // (_XBOX: 256)
pub const MAX_CHAT_LINE_SIZE: i32 = 128;

pub const TABLE_BRANCH_DISTANCE: i32 = 32;
pub const MAX_NODETABLE_SIZE: usize = 16384;

pub const MAX_LOVED_ONES: usize = 4;
pub const MAX_ATTACHMENT_NAME: usize = 64;

pub const MAX_FORCE_INFO_SIZE: usize = 2048;

pub const WPFLAG_JUMP: i32 = 0x00000010; //jump when we hit this
pub const WPFLAG_DUCK: i32 = 0x00000020; //duck while moving around here
pub const WPFLAG_NOVIS: i32 = 0x00000400; //go here for a bit even with no visibility
pub const WPFLAG_SNIPEORCAMPSTAND: i32 = 0x00000800; //a good position to snipe or camp - stand
pub const WPFLAG_WAITFORFUNC: i32 = 0x00001000; //wait for a func brushent under this point before moving here
pub const WPFLAG_SNIPEORCAMP: i32 = 0x00002000; //a good position to snipe or camp - crouch
pub const WPFLAG_ONEWAY_FWD: i32 = 0x00004000; //can only go forward on the trial from here (e.g. went over a ledge)
pub const WPFLAG_ONEWAY_BACK: i32 = 0x00008000; //can only go backward on the trail from here
pub const WPFLAG_GOALPOINT: i32 = 0x00010000; //make it a goal to get here.. goal points will be decided by setting "weight" values
pub const WPFLAG_RED_FLAG: i32 = 0x00020000; //red flag
pub const WPFLAG_BLUE_FLAG: i32 = 0x00040000; //blue flag
pub const WPFLAG_SIEGE_REBELOBJ: i32 = 0x00080000; //rebel siege objective
pub const WPFLAG_SIEGE_IMPERIALOBJ: i32 = 0x00100000; //imperial siege objective
pub const WPFLAG_NOMOVEFUNC: i32 = 0x00200000; //don't move over if a func is under

pub const WPFLAG_CALCULATED: i32 = 0x00400000; //don't calculate it again
pub const WPFLAG_NEVERONEWAY: i32 = 0x00800000; //never flag it as one-way

pub const LEVELFLAG_NOPOINTPREDICTION: i32 = 1; //don't take waypoint beyond current into account when adjusting path view angles
pub const LEVELFLAG_IGNOREINFALLBACK: i32 = 2; //ignore enemies when in a fallback navigation routine
pub const LEVELFLAG_IMUSTNTRUNAWAY: i32 = 4; //don't be scared

pub const WP_KEEP_FLAG_DIST: i32 = 128;

pub const BWEAPONRANGE_MELEE: i32 = 1;
pub const BWEAPONRANGE_MID: i32 = 2;
pub const BWEAPONRANGE_LONG: i32 = 3;
pub const BWEAPONRANGE_SABER: i32 = 4;

pub const MELEE_ATTACK_RANGE: i32 = 256;
pub const SABER_ATTACK_RANGE: i32 = 128;
pub const MAX_CHICKENWUSS_TIME: i32 = 10000; //wait 10 secs between checking which run-away path to take

pub const BOT_RUN_HEALTH: i32 = 40;
pub const BOT_WPTOUCH_DISTANCE: i32 = 32;
pub const ENEMY_FORGET_MS: i32 = 10000;

pub const BOT_PLANT_DISTANCE: i32 = 256; //plant if within this radius from the last spotted enemy position
pub const BOT_PLANT_INTERVAL: i32 = 15000; //only plant once per 15 seconds at max
pub const BOT_PLANT_BLOW_DISTANCE: i32 = 256; //blow det packs if enemy is within this radius and I am further away than the enemy

pub const BOT_MAX_WEAPON_GATHER_TIME: i32 = 1000; //spend a max of 1 second after spawn issuing orders to gather weapons before attacking enemy base
pub const BOT_MAX_WEAPON_CHASE_TIME: i32 = 15000; //time to spend gathering the weapon before persuing the enemy base (in case it takes longer than expected)

pub const BOT_MAX_WEAPON_CHASE_CTF: i32 = 5000; //time to spend gathering the weapon before persuing the enemy base [ctf-only]

pub const BOT_MIN_SIEGE_GOAL_SHOOT: i32 = 1024;
pub const BOT_MIN_SIEGE_GOAL_TRAVEL: i32 = 128;

pub const BASE_GUARD_DISTANCE: i32 = 256; //guarding the flag
pub const BASE_FLAGWAIT_DISTANCE: i32 = 256; //has the enemy flag and is waiting in his own base for his flag to be returned
pub const BASE_GETENEMYFLAG_DISTANCE: i32 = 256; //waiting around to get the enemy's flag

pub const BOT_FLAG_GET_DISTANCE: i32 = 256;

pub const BOT_SABER_THROW_RANGE: i32 = 800;

// bot_ctf_state_t (ai_main.h:81)
pub type bot_ctf_state_t = i32;
pub const CTFSTATE_NONE: bot_ctf_state_t = 0;
pub const CTFSTATE_ATTACKER: bot_ctf_state_t = 1;
pub const CTFSTATE_DEFENDER: bot_ctf_state_t = 2;
pub const CTFSTATE_RETRIEVAL: bot_ctf_state_t = 3;
pub const CTFSTATE_GUARDCARRIER: bot_ctf_state_t = 4;
pub const CTFSTATE_GETFLAGHOME: bot_ctf_state_t = 5;
pub const CTFSTATE_MAXCTFSTATES: bot_ctf_state_t = 6;

// bot_siege_state_t (ai_main.h:92)
pub type bot_siege_state_t = i32;
pub const SIEGESTATE_NONE: bot_siege_state_t = 0;
pub const SIEGESTATE_ATTACKER: bot_siege_state_t = 1;
pub const SIEGESTATE_DEFENDER: bot_siege_state_t = 2;
pub const SIEGESTATE_MAXSIEGESTATES: bot_siege_state_t = 3;

// bot_teamplay_state_t (ai_main.h:100)
pub type bot_teamplay_state_t = i32;
pub const TEAMPLAYSTATE_NONE: bot_teamplay_state_t = 0;
pub const TEAMPLAYSTATE_FOLLOWING: bot_teamplay_state_t = 1;
pub const TEAMPLAYSTATE_ASSISTING: bot_teamplay_state_t = 2;
pub const TEAMPLAYSTATE_REGROUP: bot_teamplay_state_t = 3;
pub const TEAMPLAYSTATE_MAXTPSTATES: bot_teamplay_state_t = 4;

/// `botattachment_t` (ai_main.h:109)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct botattachment_t {
    pub level: i32,
    pub name: [c_char; MAX_ATTACHMENT_NAME],
}

/// `nodeobject_t` (ai_main.h:115) — non-`_XBOX` variant (`int` neighbornum/inuse).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct nodeobject_t {
    pub origin: vec3_t,
    //	int index;
    pub weight: f32,
    pub flags: i32,
    pub neighbornum: i32,
    pub inuse: i32,
}

/// `boteventtracker_t` (ai_main.h:130)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct boteventtracker_t {
    pub eventSequence: i32,
    pub events: [i32; MAX_PS_EVENTS],
    pub eventTime: f32,
}

/// `botskills_t` (ai_main.h:137)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct botskills_t {
    pub reflex: i32,
    pub accuracy: f32,
    pub turnspeed: f32,
    pub turnspeed_combat: f32,
    pub maxturn: f32,
    pub perfectaim: i32,
}

/// `bot_state_t` (ai_main.h:148) — the per-bot AI state. Faithful field order/types.
#[repr(C)]
pub struct bot_state_t {
    pub inuse: i32,               //true if this state is used by a bot client
    pub botthink_residual: i32,   //residual for the bot thinks
    pub client: i32,              //client number of the bot
    pub entitynum: i32,           //entity number of the bot
    pub cur_ps: playerState_t,    //current player state
    pub lastucmd: usercmd_t,      //usercmd from last frame
    pub settings: bot_settings_t, //several bot settings
    pub thinktime: f32,           //time the bot thinks this frame
    pub origin: vec3_t,           //origin of the bot
    pub velocity: vec3_t,         //velocity of the bot
    pub eye: vec3_t,              //eye coordinates of the bot
    pub setupcount: i32,          //true when the bot has just been setup
    pub ltime: f32,               //local bot time
    pub entergame_time: f32,      //time the bot entered the game
    pub ms: i32,                  //move state of the bot
    pub gs: i32,                  //goal state of the bot
    pub ws: i32,                  //weapon state of the bot
    pub viewangles: vec3_t,       //current view angles
    pub ideal_viewangles: vec3_t, //ideal view angles
    pub viewanglespeed: vec3_t,

    //rww - new AI values
    pub currentEnemy: *mut gentity_t,
    pub revengeEnemy: *mut gentity_t,

    pub squadLeader: *mut gentity_t,

    pub lastHurt: *mut gentity_t,
    pub lastAttacked: *mut gentity_t,

    pub wantFlag: *mut gentity_t,

    pub touchGoal: *mut gentity_t,
    pub shootGoal: *mut gentity_t,

    pub dangerousObject: *mut gentity_t,

    pub staticFlagSpot: vec3_t,

    pub revengeHateLevel: i32,
    pub isSquadLeader: i32,

    pub squadRegroupInterval: i32,
    pub squadCannotLead: i32,

    pub lastDeadTime: i32,

    pub wpCurrent: *mut wpobject_t,
    pub wpDestination: *mut wpobject_t,
    pub wpStoreDest: *mut wpobject_t,
    pub goalAngles: vec3_t,
    pub goalMovedir: vec3_t,
    pub goalPosition: vec3_t,

    pub lastEnemySpotted: vec3_t,
    pub hereWhenSpotted: vec3_t,
    pub lastVisibleEnemyIndex: i32,
    pub hitSpotted: i32,

    pub wpDirection: i32,

    pub destinationGrabTime: f32,
    pub wpSeenTime: f32,
    pub wpTravelTime: f32,
    pub wpDestSwitchTime: f32,
    pub wpSwitchTime: f32,
    pub wpDestIgnoreTime: f32,

    pub timeToReact: f32,

    pub enemySeenTime: f32,

    pub chickenWussCalculationTime: f32,

    pub beStill: f32,
    pub duckTime: f32,
    pub jumpTime: f32,
    pub jumpHoldTime: f32,
    pub jumpPrep: f32,
    pub forceJumping: f32,
    pub jDelay: f32,

    pub aimOffsetTime: f32,
    pub aimOffsetAmtYaw: f32,
    pub aimOffsetAmtPitch: f32,

    pub frame_Waypoint_Len: f32,
    pub frame_Waypoint_Vis: i32,
    pub frame_Enemy_Len: f32,
    pub frame_Enemy_Vis: i32,

    pub isCamper: i32,
    pub isCamping: f32,
    pub wpCamping: *mut wpobject_t,
    pub wpCampingTo: *mut wpobject_t,
    pub campStanding: qboolean,

    pub randomNavTime: i32,
    pub randomNav: i32,

    pub saberSpecialist: i32,

    // PC `ai_main.h` activates these 9 bot-chat fields (the Xbox tree wrapped them in
    // `/* */`). Faithful field order/types from the PC header.
    pub canChat: i32,
    pub chatFrequency: i32,
    pub currentChat: [c_char; MAX_CHAT_LINE_SIZE as usize],
    pub chatTime: f32,
    pub chatTime_stored: f32,
    pub doChat: i32,
    pub chatTeam: i32,
    pub chatObject: *mut gentity_t,
    pub chatAltObject: *mut gentity_t,

    pub meleeStrafeTime: f32,
    pub meleeStrafeDir: i32,
    pub meleeStrafeDisable: f32,

    pub altChargeTime: i32,

    pub escapeDirTime: f32,

    pub dontGoBack: f32,

    pub doAttack: i32,
    pub doAltAttack: i32,

    pub forceWeaponSelect: i32,
    pub virtualWeapon: i32,

    pub plantTime: i32,
    pub plantDecided: i32,
    pub plantContinue: i32,
    pub plantKillEmAll: i32,

    pub runningLikeASissy: i32,
    pub runningToEscapeThreat: i32,

    //char				chatBuffer[MAX_CHAT_BUFFER_SIZE];
    //Since we're once again not allocating bot structs dynamically,
    //shoving a 64k chat buffer into one is a bad thing.
    pub skills: botskills_t,

    pub loved: [botattachment_t; MAX_LOVED_ONES],
    pub lovednum: i32,

    pub loved_death_thresh: i32,

    pub deathActivitiesDone: i32,

    pub botWeaponWeights: [f32; WP_NUM_WEAPONS as usize],

    pub ctfState: i32,

    pub siegeState: i32,

    pub teamplayState: i32,

    pub jmState: i32,

    pub state_Forced: i32, //set by player ordering menu

    pub saberDefending: i32,
    pub saberDefendDecideTime: i32,
    pub saberBFTime: i32,
    pub saberBTime: i32,
    pub saberSTime: i32,
    pub saberThrowTime: i32,

    pub saberPower: qboolean,
    pub saberPowerTime: i32,

    pub botChallengingTime: i32,

    pub forceinfo: [c_char; MAX_FORCE_INFO_SIZE],

    // #ifndef FORCEJUMP_INSTANTMETHOD (active — FORCEJUMP_INSTANTMETHOD undefined)
    pub forceJumpChargeTime: i32,

    pub doForcePush: i32,

    pub noUseTime: i32,
    pub doingFallback: qboolean,

    pub iHaveNoIdeaWhereIAmGoing: i32,
    pub lastSignificantAreaChange: vec3_t,
    pub lastSignificantChangeTime: i32,

    pub forceMove_Forward: i32,
    pub forceMove_Right: i32,
    pub forceMove_Up: i32,
    //end rww
}
