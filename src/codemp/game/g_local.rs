//! Local definitions for the game module — from `g_local.h`.
//!
//! Home of the two engine-visible master structs `gentity_t` (`gentity_s`) and
//! `gclient_t` (`gclient_s`), plus the game-local enums/structs they embed. Unlike
//! the q_shared.h networked structs, **these carry pointers** (raw `*mut`, function
//! pointers, `gentity_t*` links), so their layout is **arch-dependent** — the
//! `entityState_t s; playerState_t *playerState; …` prefix shifts every following
//! offset by the pointer-width delta on 64- vs 32-bit. The literal `size_of`/
//! `offset_of` asserts are therefore gated `#[cfg(target_pointer_width = "64")]`
//! and validated against the host-64-bit oracle (`oracle/g_local_oracle.c`); only
//! `offset_of(s/ps) == 0` is arch-independent (see `DEVIATIONS.md`, the
//! "pointer-prefix hazard"). Mirrors upstream `codemp/game/g_local.h`.
//!
//! This is the gentity_t/gclient_t slice; `level_locals_t` (game-internal) lands
//! in a later slice with the ai.h `AIGroupInfo_t` it embeds.

#![allow(non_camel_case_types)]

use crate::codemp::game::ai_h::{AIGroupInfo_t, MAX_FRAME_GROUPS};
use crate::codemp::game::b_public_h::{gNPC_t, lookMode_t};
use crate::codemp::game::bg_public::{
    gitem_t, MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS, TEAM_NUM_TEAMS,
};
use crate::codemp::game::g_public_h::{
    entityShared_t, parms_t, Vehicle_t, MAX_FAILED_NODES, NUM_BSETS, NUM_TIDS,
};
use crate::codemp::game::q_shared_h::{
    entityState_t, material_t, playerState_t, qboolean, saberInfo_t, trace_t, usercmd_t, vec3_t,
    MAX_CLIENTS, MAX_QPATH, MAX_SABERS, MAX_STRING_CHARS,
};
use crate::codemp::game::teams_h::{class_t, npcteam_t};
use crate::ffi::types::fileHandle_t;
use core::ffi::{c_char, c_int, c_uint, c_void};

/// `GAMEVERSION` — the "gameversion" client command prints this plus compile date.
pub const GAMEVERSION: &str = "basejka";

pub const BODY_QUEUE_SIZE: usize = 8;

/// `INFINITE` (g_local.h `#ifndef INFINITE`).
pub const INFINITE: c_int = 1000000;

pub const FRAMETIME: c_int = 100; // msec
pub const CARNAGE_REWARD_TIME: c_int = 3000;
pub const REWARD_SPRITE_TIME: c_int = 2000;

pub const INTERMISSION_DELAY_TIME: c_int = 1000;
pub const SP_INTERMISSION_DELAY_TIME: c_int = 5000;

//primarily used by NPCs
pub const START_TIME_LINK_ENTS: c_int = FRAMETIME * 1; // time-delay after map start at which all ents have been spawned, so can link them
pub const START_TIME_FIND_LINKS: c_int = FRAMETIME * 2; // time-delay after map start at which you can find linked entities
pub const START_TIME_MOVERS_SPAWNED: c_int = FRAMETIME * 2; // time-delay after map start at which all movers should be spawned
pub const START_TIME_REMOVE_ENTS: c_int = FRAMETIME * 3; // time-delay after map start to remove temporary ents
pub const START_TIME_NAV_CALC: c_int = FRAMETIME * 4; // time-delay after map start to connect waypoints and calc routes
pub const START_TIME_FIND_WAYPOINT: c_int = FRAMETIME * 5; // time-delay after map start after which it's okay to try to find your best waypoint

// gentity->flags
pub const FL_GODMODE: c_int = 0x00000010;
pub const FL_NOTARGET: c_int = 0x00000020;
pub const FL_TEAMSLAVE: c_int = 0x00000400; // not the first on the team
pub const FL_NO_KNOCKBACK: c_int = 0x00000800;
pub const FL_DROPPED_ITEM: c_int = 0x00001000;
pub const FL_NO_BOTS: c_int = 0x00002000; // spawn point not for bot use
pub const FL_NO_HUMANS: c_int = 0x00004000; // spawn point just for bots
pub const FL_FORCE_GESTURE: c_int = 0x00008000; // force gesture on client
pub const FL_INACTIVE: c_int = 0x00010000; // inactive
pub const FL_NAVGOAL: c_int = 0x00020000; // for npc nav stuff
pub const FL_DONT_SHOOT: c_int = 0x00040000;
pub const FL_SHIELDED: c_int = 0x00080000;
pub const FL_UNDYING: c_int = 0x00100000; // takes damage down to 1, but never dies

//ex-eFlags -rww (note: FL_BOUNCE intentionally shares FL_UNDYING's value in the original)
pub const FL_BOUNCE: c_int = 0x00100000; // for missiles
pub const FL_BOUNCE_HALF: c_int = 0x00200000; // for missiles
pub const FL_BOUNCE_SHRAPNEL: c_int = 0x00400000; // special shrapnel flag

//vehicle game-local stuff -rww
pub const FL_VEH_BOARDING: c_int = 0x00800000; // special shrapnel flag

//breakable flags -rww
pub const FL_DMG_BY_SABER_ONLY: c_int = 0x01000000; //only take dmg from saber
pub const FL_DMG_BY_HEAVY_WEAP_ONLY: c_int = 0x02000000; //only take dmg from explosives

pub const FL_BBRUSH: c_int = 0x04000000; //I am a breakable brush

pub const MAX_G_SHARED_BUFFER_SIZE: usize = 8192;

/// `moverState_t` (g_local.h) — movers are things like doors, plats, buttons, etc.
pub type moverState_t = c_int;
pub const MOVER_POS1: moverState_t = 0;
pub const MOVER_POS2: moverState_t = 1;
pub const MOVER_1TO2: moverState_t = 2;
pub const MOVER_2TO1: moverState_t = 3;

/// `SP_PODIUM_MODEL` (g_local.h).
pub const SP_PODIUM_MODEL: &str = "models/mapobjects/podium/podium4.md3";

// Hit-location enum (anonymous in g_local.h, so plain `c_int` consts). `HL_MAX`
// sizes `gentity_t::locationDamage`. PC activates HL_GENERIC1..6 (the Xbox tree
// commented them out), so HL_MAX = 23 (Xbox: 17).
pub const HL_NONE: c_int = 0;
pub const HL_FOOT_RT: c_int = 1;
pub const HL_FOOT_LT: c_int = 2;
pub const HL_LEG_RT: c_int = 3;
pub const HL_LEG_LT: c_int = 4;
pub const HL_WAIST: c_int = 5;
pub const HL_BACK_RT: c_int = 6;
pub const HL_BACK_LT: c_int = 7;
pub const HL_BACK: c_int = 8;
pub const HL_CHEST_RT: c_int = 9;
pub const HL_CHEST_LT: c_int = 10;
pub const HL_CHEST: c_int = 11;
pub const HL_ARM_RT: c_int = 12;
pub const HL_ARM_LT: c_int = 13;
pub const HL_HAND_RT: c_int = 14;
pub const HL_HAND_LT: c_int = 15;
pub const HL_HEAD: c_int = 16;
pub const HL_GENERIC1: c_int = 17;
pub const HL_GENERIC2: c_int = 18;
pub const HL_GENERIC3: c_int = 19;
pub const HL_GENERIC4: c_int = 20;
pub const HL_GENERIC5: c_int = 21;
pub const HL_GENERIC6: c_int = 22;
pub const HL_MAX: c_int = 23;

/// `gentity_s` / `gentity_t` (g_local.h) — the server-side game entity. The first
/// members up to `next_roff_time` are the engine-visible prefix shared with
/// `centity_t`/`bgEntity_t` ("DO NOT MODIFY ANYTHING ABOVE THIS, THE SERVER EXPECTS
/// THE FIELDS IN THAT ORDER!"). Pointer-bearing => arch-dependent layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gentity_s {
    //rww - entstate must be first, to correspond with the bg shared entity structure
    pub s: entityState_t,                // communicated by server to clients
    pub playerState: *mut playerState_t, //ptr to playerstate if applicable (for bg ents)
    pub m_pVehicle: *mut Vehicle_t,      //vehicle data
    pub ghoul2: *mut c_void,             //g2 instance
    pub localAnimIndex: c_int,           //index locally (game/cgame) to anim data for this skel
    pub modelScale: vec3_t,              //needed for g2 collision

    //From here up must be the same as centity_t/bgEntity_t
    pub r: entityShared_t, // shared by both the server system and game

    //rww - these are shared icarus things. They must be in this order as well in relation to the entityshared structure.
    pub taskID: [c_int; NUM_TIDS],
    pub parms: *mut parms_t,
    pub behaviorSet: [*mut c_char; NUM_BSETS],
    pub script_targetname: *mut c_char,
    pub delayScriptTime: c_int,
    pub fullName: *mut c_char,

    //rww - targetname and classname are now shared as well. ICARUS needs access to them.
    pub targetname: *mut c_char,
    pub classname: *mut c_char, // set in QuakeEd

    //rww - and yet more things to share. This is because the nav code is in the exe because it's all C++.
    pub waypoint: c_int, //Set once per frame, if you've moved, and if someone asks
    pub lastWaypoint: c_int, //To make sure you don't double-back
    pub lastValidWaypoint: c_int, //ALWAYS valid -used for tracking someone you lost
    pub noWaypointTime: c_int, //Debouncer - so don't keep checking every waypoint in existance every frame that you can't find one
    pub combatPoint: c_int,
    pub failedWaypoints: [c_int; MAX_FAILED_NODES],
    pub failedWaypointCheckTime: c_int,

    pub next_roff_time: c_int, //rww - npc's need to know when they're getting roff'd

    // DO NOT MODIFY ANYTHING ABOVE THIS, THE SERVER
    // EXPECTS THE FIELDS IN THAT ORDER!
    //================================
    pub client: *mut gclient_s, // NULL if not a client

    pub NPC: *mut gNPC_t,           //Only allocated if the entity becomes an NPC
    pub cantHitEnemyCounter: c_int, //HACK - Makes them look for another enemy on the same team if the one they're after can't be hit

    pub noLumbar: qboolean, //see note in cg_local.h

    pub inuse: qboolean,

    pub lockCount: c_int, //used by NPCs

    pub spawnflags: c_int, // set in QuakeEd

    pub teamnodmg: c_int, // damage will be ignored if it comes from this team

    pub roffname: *mut c_char,   // set in QuakeEd
    pub rofftarget: *mut c_char, // set in QuakeEd

    pub healingclass: *mut c_char, //set in quakeed
    pub healingsound: *mut c_char, //set in quakeed
    pub healingrate: c_int,        //set in quakeed
    pub healingDebounce: c_int,    //debounce for generic object healing shiz

    pub ownername: *mut c_char,

    pub objective: c_int,
    pub side: c_int,

    pub passThroughNum: c_int, // set to index to pass through (+1) for missiles

    pub aimDebounceTime: c_int,
    pub painDebounceTime: c_int,
    pub attackDebounceTime: c_int,
    pub alliedTeam: c_int, // only useable by this team, never target this team

    pub roffid: c_int, // if roffname != NULL then set on spawn

    pub neverFree: qboolean, // if true, FreeEntity will only unlink
    // bodyque uses this
    pub flags: c_int, // FL_* variables

    pub model: *mut c_char,
    pub model2: *mut c_char,
    pub freetime: c_int, // level.time when the object was freed

    pub eventTime: c_int, // events will be cleared EVENT_VALID_MSEC after set
    pub freeAfterEvent: qboolean,
    pub unlinkAfterEvent: qboolean,

    pub physicsObject: qboolean, // if true, it can be pushed by movers and fall off edges
    // all game items are physicsObjects,
    pub physicsBounce: f32, // 1.0 = continuous bounce, 0.0 = no bounce
    pub clipmask: c_int,    // brushes with this content value will be collided against
    // when moving.  items and corpses do not collide against
    // players, for instance

    //Only used by NPC_spawners
    pub NPC_type: *mut c_char,
    pub NPC_targetname: *mut c_char,
    pub NPC_target: *mut c_char,

    // movers
    pub moverState: moverState_t,
    pub soundPos1: c_int,
    pub sound1to2: c_int,
    pub sound2to1: c_int,
    pub soundPos2: c_int,
    pub soundLoop: c_int,
    pub parent: *mut gentity_s,
    pub nextTrain: *mut gentity_s,
    pub prevTrain: *mut gentity_s,
    pub pos1: vec3_t,
    pub pos2: vec3_t,

    //for npc's
    pub pos3: vec3_t,

    pub message: *mut c_char,

    pub timestamp: c_int, // body queue sinking, etc

    pub angle: f32, // set in editor, -1 = up, -2 = down
    pub target: *mut c_char,
    pub target2: *mut c_char,
    pub target3: *mut c_char, //For multiple targets, not used for firing/triggering/using, though, only for path branches
    pub target4: *mut c_char, //For multiple targets, not used for firing/triggering/using, though, only for path branches
    pub target5: *mut c_char, //mainly added for siege items
    pub target6: *mut c_char, //mainly added for siege items

    pub team: *mut c_char,
    pub targetShaderName: *mut c_char,
    pub targetShaderNewName: *mut c_char,
    pub target_ent: *mut gentity_s,

    pub closetarget: *mut c_char,
    pub opentarget: *mut c_char,
    pub paintarget: *mut c_char,

    pub goaltarget: *mut c_char,
    pub idealclass: *mut c_char,

    pub radius: f32,

    pub maxHealth: c_int, //used as a base for crosshair health display

    pub speed: f32,
    pub movedir: vec3_t,
    pub mass: f32,
    pub setTime: c_int,

    //Think Functions
    pub nextthink: c_int,
    pub think: Option<unsafe extern "C" fn(*mut gentity_s)>,
    pub reached: Option<unsafe extern "C" fn(*mut gentity_s)>, // movers call this when hitting endpoint
    pub blocked: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s)>,
    pub touch: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, *mut trace_t)>,
    pub r#use: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, *mut gentity_s)>,
    pub pain: Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, c_int)>,
    pub die:
        Option<unsafe extern "C" fn(*mut gentity_s, *mut gentity_s, *mut gentity_s, c_int, c_int)>,

    pub pain_debounce_time: c_int,
    pub fly_sound_debounce_time: c_int, // wind tunnel
    pub last_move_time: c_int,

    //Health and damage fields
    pub health: c_int,
    pub takedamage: qboolean,
    pub material: material_t,

    pub damage: c_int,
    pub dflags: c_int,
    pub splashDamage: c_int, // quad will increase this without increasing radius
    pub splashRadius: c_int,
    pub methodOfDeath: c_int,
    pub splashMethodOfDeath: c_int,

    pub locationDamage: [c_int; HL_MAX as usize], // Damage accumulated on different body locations

    pub count: c_int,
    pub bounceCount: c_int,
    pub alt_fire: qboolean,

    pub chain: *mut gentity_s,
    pub enemy: *mut gentity_s,
    pub lastEnemy: *mut gentity_s,
    pub activator: *mut gentity_s,
    pub teamchain: *mut gentity_s,  // next entity in team
    pub teammaster: *mut gentity_s, // master of the team

    pub watertype: c_int,
    pub waterlevel: c_int,

    pub noise_index: c_int,

    // timing variables
    pub wait: f32,
    pub random: f32,
    pub delay: c_int,

    //generic values used by various entities for different purposes.
    pub genericValue1: c_int,
    pub genericValue2: c_int,
    pub genericValue3: c_int,
    pub genericValue4: c_int,
    pub genericValue5: c_int,
    pub genericValue6: c_int,
    pub genericValue7: c_int,
    pub genericValue8: c_int,
    pub genericValue9: c_int,
    pub genericValue10: c_int,
    pub genericValue11: c_int,
    pub genericValue12: c_int,
    pub genericValue13: c_int,
    pub genericValue14: c_int,
    pub genericValue15: c_int,

    pub soundSet: *mut c_char,

    pub isSaberEntity: qboolean,

    pub damageRedirect: c_int,   //if entity takes damage, redirect to..
    pub damageRedirectTo: c_int, //this entity number

    pub epVelocity: vec3_t,
    pub epGravFactor: f32,

    pub item: *mut gitem_t, // for bonus items
}

/// `gentity_t` (g_local.h `typedef struct gentity_s gentity_t`).
pub type gentity_t = gentity_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<gentity_t>() == 1832);
const _: () = assert!(core::mem::offset_of!(gentity_t, s) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, r) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, taskID) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, client) == 976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, moverState) == 1176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, think) == 1440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, material) == 1516);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, locationDamage) == 1544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gentity_t, item) == 1824);

pub const DAMAGEREDIRECT_HEAD: c_int = 1;
pub const DAMAGEREDIRECT_RLEG: c_int = 2;
pub const DAMAGEREDIRECT_LLEG: c_int = 3;

/// `clientConnected_t` (g_local.h) — anonymous enum + `typedef int`.
pub type clientConnected_t = c_int;
pub const CON_DISCONNECTED: clientConnected_t = 0;
pub const CON_CONNECTING: clientConnected_t = 1;
pub const CON_CONNECTED: clientConnected_t = 2;

/// `spectatorState_t` (g_local.h).
pub type spectatorState_t = c_int;
pub const SPECTATOR_NOT: spectatorState_t = 0;
pub const SPECTATOR_FREE: spectatorState_t = 1;
pub const SPECTATOR_FOLLOW: spectatorState_t = 2;
pub const SPECTATOR_SCOREBOARD: spectatorState_t = 3;

/// `playerTeamStateState_t` (g_local.h).
pub type playerTeamStateState_t = c_int;
pub const TEAM_BEGIN: playerTeamStateState_t = 0; // Beginning a team game, spawn at base
pub const TEAM_ACTIVE: playerTeamStateState_t = 1; // Now actively playing

/// `playerTeamState_t` (g_local.h) — status in teamplay games. Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct playerTeamState_t {
    pub state: playerTeamStateState_t,

    pub location: c_int,

    pub captures: c_int,
    pub basedefense: c_int,
    pub carrierdefense: c_int,
    pub flagrecovery: c_int,
    pub fragcarrier: c_int,
    pub assists: c_int,

    pub lasthurtcarrier: f32,
    pub lastreturnedflag: f32,
    pub flagsince: f32,
    pub lastfraggedcarrier: f32,
}
const _: () = assert!(core::mem::size_of::<playerTeamState_t>() == 48);

// the auto following clients don't follow a specific client
// number, but instead follow the first two active players
pub const FOLLOW_ACTIVE1: c_int = -1;
pub const FOLLOW_ACTIVE2: c_int = -2;

/// `clientSession_t` (g_local.h) — client data that stays across multiple levels
/// or tournament restarts (written to cvar strings at game shutdown, read back at
/// connection time). "Anything added here MUST be dealt with in G_InitSessionData()
/// / G_ReadSessionData() / G_WriteSessionData()." Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct clientSession_t {
    pub sessionTeam: c_int,   // team_t
    pub spectatorTime: c_int, // for determining next-in-line to play
    pub spectatorState: spectatorState_t,
    pub spectatorClient: c_int, // for chasecam and follow mode
    pub wins: c_int,            // tournament stats
    pub losses: c_int,
    pub selectedFP: c_int, // check against this, if doesn't match value in playerstate then update userinfo
    pub saberLevel: c_int, // similar to above method, but for current saber attack level
    pub setForce: qboolean, // set to true once player is given the chance to set force powers
    pub updateUITime: c_int, // only update userinfo for FP/SL if < level.time
    pub teamLeader: qboolean, // true when this client is a team leader
    pub siegeClass: [c_char; 64],
    pub saberType: [c_char; 64],
    pub saber2Type: [c_char; 64],
    pub duelTeam: c_int,
    pub siegeDesiredTeam: c_int,
    pub killCount: c_int,
    pub TKCount: c_int,
    pub IPstring: [c_char; 32], // yeah, I know, could be 16, but, just in case...
}
const _: () = assert!(core::mem::size_of::<clientSession_t>() == 284);

// playerstate mGameFlags
pub const PSG_VOTED: c_int = 1 << 0; // already cast a vote
pub const PSG_TEAMVOTED: c_int = 1 << 1; // already cast a team vote

pub const MAX_NETNAME: usize = 36;
pub const MAX_VOTE_COUNT: c_int = 3;

/// `clientPersistant_t` (g_local.h) — client data that stays across multiple
/// respawns, but is cleared on each level change or team change at ClientBegin().
/// Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct clientPersistant_t {
    pub connected: clientConnected_t,
    pub cmd: usercmd_t,              // we would lose angles if not persistant
    pub localClient: qboolean,       // true if "ip" info key is "localhost"
    pub initialSpawn: qboolean,      // the first spawn should be at a cool location
    pub predictItemPickup: qboolean, // based on cg_predictItems userinfo
    pub pmoveFixed: qboolean,        //
    pub netname: [c_char; MAX_NETNAME],
    pub netnameTime: c_int,           // Last time the name was changed
    pub maxHealth: c_int,             // for handicapping
    pub enterTime: c_int,             // level.time the client entered the game
    pub teamState: playerTeamState_t, // status in teamplay games
    pub voteCount: c_int,             // to prevent people from constantly calling votes
    pub teamVoteCount: c_int,         // to prevent people from constantly calling votes
    pub teamInfo: qboolean,           // send team overlay updates?
}
const _: () = assert!(core::mem::size_of::<clientPersistant_t>() == 156);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, netname) == 48);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, teamState) == 96);

/// `renderInfo_t` (g_local.h) — per-client model-rendering state (head/torso look
/// ranges, muzzle points, bolt indices, …). Carries a `void *lastG2`, so it is
/// arch-dependent (64-bit layout asserted).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct renderInfo_t {
    //In whole degrees, How far to let the different model parts yaw and pitch
    pub headYawRangeLeft: c_int,
    pub headYawRangeRight: c_int,
    pub headPitchRangeUp: c_int,
    pub headPitchRangeDown: c_int,

    pub torsoYawRangeLeft: c_int,
    pub torsoYawRangeRight: c_int,
    pub torsoPitchRangeUp: c_int,
    pub torsoPitchRangeDown: c_int,

    pub legsFrame: c_int,
    pub torsoFrame: c_int,

    pub legsFpsMod: f32,
    pub torsoFpsMod: f32,

    //Fields to apply to entire model set, individual model's equivalents will modify this value
    pub customRGB: vec3_t,  //Red Green Blue, 0 = don't apply
    pub customAlpha: c_int, //Alpha to apply, 0 = none?

    //RF?
    pub renderFlags: c_int,

    //
    pub muzzlePoint: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDirOld: vec3_t,
    //vec3_t		muzzlePointNext;	// Muzzle point one server frame in the future!
    //vec3_t		muzzleDirNext;
    pub mPCalcTime: c_int, //Last time muzzle point was calced

    //
    pub lockYaw: f32, //

    //
    pub headPoint: vec3_t,   //Where your tag_head is
    pub headAngles: vec3_t,  //where the tag_head in the torso is pointing
    pub handRPoint: vec3_t,  //where your right hand is
    pub handLPoint: vec3_t,  //where your left hand is
    pub crotchPoint: vec3_t, //Where your crotch is
    pub footRPoint: vec3_t,  //where your right hand is
    pub footLPoint: vec3_t,  //where your left hand is
    pub torsoPoint: vec3_t,  //Where your chest is
    pub torsoAngles: vec3_t, //Where the chest is pointing
    pub eyePoint: vec3_t,    //Where your eyes are
    pub eyeAngles: vec3_t,   //Where your eyes face
    pub lookTarget: c_int,   //Which ent to look at with lookAngles
    pub lookMode: lookMode_t,
    pub lookTargetClearTime: c_int,  //Time to clear the lookTarget
    pub lastVoiceVolume: c_int,      //Last frame's voice volume
    pub lastHeadAngles: vec3_t,      //Last headAngles, NOT actual facing of head model
    pub headBobAngles: vec3_t,       //headAngle offsets
    pub targetHeadBobAngles: vec3_t, //head bob angles will try to get to targetHeadBobAngles
    pub lookingDebounceTime: c_int,  //When we can stop using head looking angle behavior
    pub legsYaw: f32,                //yaw angle your legs are actually rendering at

    //for tracking legitimate bolt indecies
    pub lastG2: *mut c_void, //if it doesn't match ent->ghoul2, the bolts are considered invalid.
    pub headBolt: c_int,
    pub handRBolt: c_int,
    pub handLBolt: c_int,
    pub torsoBolt: c_int,
    pub crotchBolt: c_int,
    pub footRBolt: c_int,
    pub footLBolt: c_int,
    pub motionBolt: c_int,

    pub boltValidityTime: c_int,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<renderInfo_t>() == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lookMode) == 260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(renderInfo_t, lastG2) == 320);

/// `gclient_s` / `gclient_t` (g_local.h) — the game-side client. `ps` MUST be the
/// first element (the server expects it); the rest is private to the game.
/// "Cleared on each ClientSpawn(), except for `client->pers` and `client->sess`."
/// Pointer-bearing => arch-dependent layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gclient_s {
    // ps MUST be the first element, because the server expects it
    pub ps: playerState_t, // communicated by server to clients

    // the rest of the structure is private to game
    pub pers: clientPersistant_t,
    pub sess: clientSession_t,

    pub saber: [saberInfo_t; MAX_SABERS],
    pub weaponGhoul2: [*mut c_void; MAX_SABERS],

    pub tossableItemDebounce: c_int,

    pub bodyGrabTime: c_int,
    pub bodyGrabIndex: c_int,

    pub pushEffectTime: c_int,

    pub invulnerableTimer: c_int,

    pub saberCycleQueue: c_int,

    pub legsAnimExecute: c_int,
    pub torsoAnimExecute: c_int,
    pub legsLastFlip: qboolean,
    pub torsoLastFlip: qboolean,

    pub readyToExit: qboolean, // wishes to leave the intermission

    pub noclip: qboolean,

    pub lastCmdTime: c_int, // level.time of last usercmd_t, for EF_CONNECTION
    // we can't just use pers.lastCommand.time, because
    // of the g_sycronousclients case
    pub buttons: c_int,
    pub oldbuttons: c_int,
    pub latched_buttons: c_int,

    pub oldOrigin: vec3_t,

    // sum up damage over an entire frame, so
    // shotgun blasts give a single big kick
    pub damage_armor: c_int,        // damage absorbed by armor
    pub damage_blood: c_int,        // damage taken out of health
    pub damage_knockback: c_int,    // impact damage
    pub damage_from: vec3_t,        // origin for vector calculation
    pub damage_fromWorld: qboolean, // if true, don't use the damage_from vector

    pub damageBoxHandle_Head: c_int, //entity number of head damage box
    pub damageBoxHandle_RLeg: c_int, //entity number of right leg damage box
    pub damageBoxHandle_LLeg: c_int, //entity number of left leg damage box

    pub accurateCount: c_int, // for "impressive" reward sound

    pub accuracy_shots: c_int, // total number of shots
    pub accuracy_hits: c_int,  // total number of hits

    //
    pub lastkilled_client: c_int, // last client that this client killed
    pub lasthurt_client: c_int,   // last client that damaged this client
    pub lasthurt_mod: c_int,      // type of damage the client did

    // timers
    pub respawnTime: c_int, // can respawn when time > this, force after g_forcerespwan
    pub inactivityTime: c_int, // kick players when time > this
    pub inactivityWarning: qboolean, // qtrue if the five seoond warning has been given
    pub rewardTime: c_int,  // clear the EF_AWARD_IMPRESSIVE, etc when time > this

    pub airOutTime: c_int,

    pub lastKillTime: c_int, // for multiple kill rewards

    pub fireHeld: qboolean,   // used for hook
    pub hook: *mut gentity_s, // grapple hook if out

    pub switchTeamTime: c_int, // time the player switched teams

    pub switchDuelTeamTime: c_int, // time the player switched duel teams

    pub switchClassTime: c_int, // class changed debounce timer

    // timeResidual is used to handle events that happen every second
    // like health / armor countdowns and regeneration
    pub timeResidual: c_int,

    pub areabits: *mut c_char,

    pub g2LastSurfaceHit: c_int, //index of surface hit during the most recent ghoul2 collision performed on this client.
    pub g2LastSurfaceTime: c_int, //time when the surface index was set (to make sure it's up to date)

    pub corrTime: c_int,

    pub lastHeadAngles: vec3_t,
    pub lookTime: c_int,

    pub brokenLimbs: c_int,

    pub noCorpse: qboolean, //don't leave a corpse on respawn this time.

    pub jetPackTime: c_int,

    pub jetPackOn: qboolean,
    pub jetPackToggleTime: c_int,
    pub jetPackDebRecharge: c_int,
    pub jetPackDebReduce: c_int,

    pub cloakToggleTime: c_int,
    pub cloakDebRecharge: c_int,
    pub cloakDebReduce: c_int,

    pub saberStoredIndex: c_int, //stores saberEntityNum from playerstate for when it's set to 0 (indicating saber was knocked out of the air)

    pub saberKnockedTime: c_int, //if saber gets knocked away, can't pull it back until this value is < level.time

    pub olderSaberBase: vec3_t, //Set before lastSaberBase_Always, to whatever lastSaberBase_Always was previously
    pub olderIsValid: qboolean, //is it valid?

    pub lastSaberDir_Always: vec3_t, //every getboltmatrix, set to saber dir
    pub lastSaberBase_Always: vec3_t, //every getboltmatrix, set to saber base
    pub lastSaberStorageTime: c_int, //server time that the above two values were updated (for making sure they aren't out of date)

    pub hasCurrentPosition: qboolean, //are lastSaberTip and lastSaberBase valid?

    pub dangerTime: c_int, // level.time when last attack occured

    pub idleTime: c_int, //keep track of when to play an idle anim on the client.

    pub idleHealth: c_int,      //stop idling if health decreases
    pub idleViewAngles: vec3_t, //stop idling if viewangles change

    pub forcePowerSoundDebounce: c_int, //if > level.time, don't do certain sound events again (drain sound, absorb sound, etc)

    pub modelname: [c_char; MAX_QPATH],

    pub fjDidJump: qboolean,

    pub ikStatus: qboolean,

    pub throwingIndex: c_int,
    pub beingThrown: c_int,
    pub doingThrow: c_int,

    pub hiddenDist: f32,   //How close ents have to be to pick you up as an enemy
    pub hiddenDir: vec3_t, //Normalized direction in which NPCs can't see you (you are hidden)

    pub renderInfo: renderInfo_t,

    //mostly NPC stuff:
    pub playerTeam: npcteam_t,
    pub enemyTeam: npcteam_t,
    pub squadname: *mut c_char,
    pub team_leader: *mut gentity_s,
    pub leader: *mut gentity_s,
    pub follower: *mut gentity_s,
    pub numFollowers: c_int,
    pub formationGoal: *mut gentity_s,
    pub nextFormGoal: c_int,
    pub NPC_class: class_t,

    pub pushVec: vec3_t,
    pub pushVecTime: c_int,

    pub siegeClass: c_int,
    pub holdingObjectiveItem: c_int,

    //time values for when being healed/supplied by supplier class
    pub isMedHealed: c_int,
    pub isMedSupplied: c_int,

    //seperate debounce time for refilling someone's ammo as a supplier
    pub medSupplyDebounce: c_int,

    //used in conjunction with ps.hackingTime
    pub isHacking: c_int,
    pub hackingAngles: vec3_t,

    //debounce time for sending extended siege data to certain classes
    pub siegeEDataSend: c_int,

    pub ewebIndex: c_int,  //index of e-web gun if spawned
    pub ewebTime: c_int,   //e-web use debounce
    pub ewebHealth: c_int, //health of e-web (to keep track between deployments)

    pub inSpaceIndex: c_int,       //ent index of space trigger if inside one
    pub inSpaceSuffocation: c_int, //suffocation timer

    pub tempSpectate: c_int, //time to force spectator mode

    //keep track of last person kicked and the time so we don't hit multiple times per kick
    pub jediKickIndex: c_int,
    pub jediKickTime: c_int,

    //special moves (designed for kyle boss npc, but useable by players in mp)
    pub grappleIndex: c_int,
    pub grappleState: c_int,

    pub solidHack: c_int,

    pub noLightningTime: c_int,

    pub mGameFlags: c_uint,

    //fallen duelist
    pub iAmALoser: qboolean,

    pub lastGenCmd: c_int,
    pub lastGenCmdTime: c_int,

    //can't put these in playerstate, crashes game (need to change exe?)
    pub otherKillerMOD: c_int,
    pub otherKillerVehWeapon: c_int,
    pub otherKillerWeaponType: c_int,
}

/// `gclient_t` (g_local.h `typedef struct gclient_s gclient_t`).
pub type gclient_t = gclient_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<gclient_t>() == 7344);
const _: () = assert!(core::mem::offset_of!(gclient_t, ps) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, pers) == 1552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, sess) == 1708);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, saber) == 1992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, renderInfo) == 6776);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, NPC_class) == 7204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(gclient_t, lastGenCmdTime) == 7324);

// ===========================================================================
// level_locals_t and its support types (game-internal; the engine never sees
// these). `level_locals_t` embeds the ai.h `AIGroupInfo_t` by value, so this slice
// lands after ai_h.rs.
// ===========================================================================

//Interest points
pub const MAX_INTEREST_POINTS: usize = 64;

/// `interestPoint_t` (g_local.h) — squadmates look at these when idle and close.
/// Carries a `char *target` => arch-dependent.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct interestPoint_t {
    pub origin: vec3_t,
    pub target: *mut c_char,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<interestPoint_t>() == 24);

//Combat points
pub const MAX_COMBAT_POINTS: usize = 512;

/// `combatPoint_t` (g_local.h) — NPCs in bState BS_COMBAT_POINT find their closest
/// empty combat_point. Pointer-free (the `NPC_targetname`/`team` members are
/// commented out in the original).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct combatPoint_t {
    pub origin: vec3_t,
    pub flags: c_int,
    //	char		*NPC_targetname;
    //	team_t		team;
    pub occupied: qboolean,
    pub waypoint: c_int,
    pub dangerTime: c_int,
}
const _: () = assert!(core::mem::size_of::<combatPoint_t>() == 28);

// Alert events
pub const MAX_ALERT_EVENTS: usize = 32;

/// `alertEventType_e` (g_local.h).
pub type alertEventType_e = c_int;
pub const AET_SIGHT: alertEventType_e = 0;
pub const AET_SOUND: alertEventType_e = 1;

/// `alertEventLevel_e` (g_local.h).
pub type alertEventLevel_e = c_int;
pub const AEL_MINOR: alertEventLevel_e = 0; //Enemy responds to the sound, but only by looking
pub const AEL_SUSPICIOUS: alertEventLevel_e = 1; //Enemy looks at the sound, and will also investigate it
pub const AEL_DISCOVERED: alertEventLevel_e = 2; //Enemy knows the player is around, and will actively hunt
pub const AEL_DANGER: alertEventLevel_e = 3; //Enemy should try to find cover
pub const AEL_DANGER_GREAT: alertEventLevel_e = 4; //Enemy should run like hell!

/// `alertEvent_t` (g_local.h). Carries a `gentity_t *owner` => arch-dependent.
/// `type` is a Rust keyword, hence `r#type` (the C field is `type`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct alertEvent_t {
    pub position: vec3_t,         //Where the event is located
    pub radius: f32,              //Consideration radius
    pub level: alertEventLevel_e, //Priority level of the event
    pub r#type: alertEventType_e, //Event type (sound,sight)
    pub owner: *mut gentity_s,    //Who made the sound
    pub light: f32,               //ambient light level at point
    pub addLight: f32,            //additional light- makes it more noticable, even in darkness
    pub ID: c_int, //unique... if get a ridiculous number, this will repeat, but should not be a problem as it's just comparing it to your lastAlertID
    pub timestamp: c_int, //when it was created
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<alertEvent_t>() == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, owner) == 24);

/// `waypointData_t` (g_local.h) — "this structure is cleared as each map is
/// entered". Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct waypointData_t {
    pub targetname: [c_char; MAX_QPATH],
    pub target: [c_char; MAX_QPATH],
    pub target2: [c_char; MAX_QPATH],
    pub target3: [c_char; MAX_QPATH],
    pub target4: [c_char; MAX_QPATH],
    pub nodeID: c_int,
}
const _: () = assert!(core::mem::size_of::<waypointData_t>() == 324);

/// `level_locals_t` (g_local.h) — "this structure is cleared as each map is
/// entered". Game-internal (not engine-visible). Pointer-bearing => arch-dependent;
/// embeds `AIGroupInfo_t groups[MAX_FRAME_GROUPS]` and the alert/interest/combat
/// arrays by value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct level_locals_t {
    pub clients: *mut gclient_s, // [maxclients]

    pub gentities: *mut gentity_s,
    pub gentitySize: c_int,
    pub num_entities: c_int, // current number, <= MAX_GENTITIES

    pub warmupTime: c_int, // restart match at this time

    pub logFile: fileHandle_t,

    // store latched cvars here that we want to get at often
    pub maxclients: c_int,

    pub framenum: c_int,
    pub time: c_int,         // in msec
    pub previousTime: c_int, // so movers can back up when blocked

    pub startTime: c_int, // level.time the map was started

    pub teamScores: [c_int; TEAM_NUM_TEAMS as usize],
    pub lastTeamLocationTime: c_int, // last time of client team location update

    pub newSession: qboolean, // don't use any old session data, because
    // we changed gametype
    pub restarted: qboolean, // waiting for a map_restart to fire

    pub numConnectedClients: c_int,
    pub numNonSpectatorClients: c_int, // includes connecting clients
    pub numPlayingClients: c_int,      // connected, non-spectators
    pub sortedClients: [c_int; MAX_CLIENTS], // sorted by score
    pub follow1: c_int,                // clientNums for auto-follow spectators
    pub follow2: c_int,

    pub snd_fry: c_int, // sound index for standing in lava

    pub snd_hack: c_int,        //hacking loop sound
    pub snd_medHealed: c_int,   //being healed by supply class
    pub snd_medSupplied: c_int, //being supplied by supply class

    pub warmupModificationCount: c_int, // for detecting if g_warmup is changed

    // voting state
    pub voteString: [c_char; MAX_STRING_CHARS],
    pub voteDisplayString: [c_char; MAX_STRING_CHARS],
    pub voteTime: c_int,        // level.time vote was called
    pub voteExecuteTime: c_int, // time the vote is executed
    pub voteYes: c_int,
    pub voteNo: c_int,
    pub numVotingClients: c_int, // set by CalculateRanks

    pub votingGametype: qboolean,
    pub votingGametypeTo: c_int,

    // team voting state
    pub teamVoteString: [[c_char; MAX_STRING_CHARS]; 2],
    pub teamVoteTime: [c_int; 2], // level.time vote was called
    pub teamVoteYes: [c_int; 2],
    pub teamVoteNo: [c_int; 2],
    pub numteamVotingClients: [c_int; 2], // set by CalculateRanks

    // spawn variables
    pub spawning: qboolean, // the G_Spawn*() functions are valid
    pub numSpawnVars: c_int,
    pub spawnVars: [[*mut c_char; 2]; MAX_SPAWN_VARS as usize], // key / value pairs
    pub numSpawnVarChars: c_int,
    pub spawnVarChars: [c_char; MAX_SPAWN_VARS_CHARS as usize],

    // intermission state
    pub intermissionQueued: c_int, // intermission was qualified, but
    // wait INTERMISSION_DELAY_TIME before
    // actually going there so the last
    // frag can be watched.  Disable future
    // kills during this delay
    pub intermissiontime: c_int, // time the intermission was started
    pub changemap: *mut c_char,
    pub readyToExit: qboolean, // at least one client wants to exit
    pub exitTime: c_int,
    pub intermission_origin: vec3_t, // also used for spectator spawns
    pub intermission_angle: vec3_t,

    pub locationLinked: qboolean,     // target_locations get linked
    pub locationHead: *mut gentity_s, // head of the location list
    pub bodyQueIndex: c_int,          // dead bodies
    pub bodyQue: [*mut gentity_s; BODY_QUEUE_SIZE],
    pub portalSequence: c_int,

    pub alertEvents: [alertEvent_t; MAX_ALERT_EVENTS],
    pub numAlertEvents: c_int,
    pub curAlertID: c_int,

    pub groups: [AIGroupInfo_t; MAX_FRAME_GROUPS],

    //Interest points- squadmates automatically look at these if standing around and close to them
    pub interestPoints: [interestPoint_t; MAX_INTEREST_POINTS],
    pub numInterestPoints: c_int,

    //Combat points- NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
    pub combatPoints: [combatPoint_t; MAX_COMBAT_POINTS],
    pub numCombatPoints: c_int,

    //rwwRMG - added:
    pub mNumBSPInstances: c_int,
    pub mBSPInstanceDepth: c_int,
    pub mOriginAdjust: vec3_t,
    pub mRotationAdjust: f32,
    pub mTargetAdjust: *mut c_char,

    pub mTeamFilter: [c_char; MAX_QPATH],
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<level_locals_t>() == 47176);
const _: () = assert!(core::mem::offset_of!(level_locals_t, clients) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, groups) == 11232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, combatPoints) == 32740);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mTeamFilter) == 47112);

// ===========================================================================
// Remaining g_local.h data: damage flags, mover spawnflags, and the two later
// pointer-free structs (reference_tag_t in g_misc.c, bot_settings_t in ai_util.c).
// ===========================================================================

// damage flags (used by G_Damage's `dflags`)
pub const DAMAGE_NORMAL: c_int = 0x00000000; // No flags set.
pub const DAMAGE_RADIUS: c_int = 0x00000001; // damage was indirect
pub const DAMAGE_NO_ARMOR: c_int = 0x00000002; // armour does not protect from this damage
pub const DAMAGE_NO_KNOCKBACK: c_int = 0x00000004; // do not affect velocity, just view angles
pub const DAMAGE_NO_PROTECTION: c_int = 0x00000008; // armor, shields, invulnerability, and godmode have no effect
pub const DAMAGE_NO_TEAM_PROTECTION: c_int = 0x00000010; // armor, shields, invulnerability, and godmode have no effect
                                                         //JK2 flags
pub const DAMAGE_EXTRA_KNOCKBACK: c_int = 0x00000040; // add extra knockback to this damage
pub const DAMAGE_DEATH_KNOCKBACK: c_int = 0x00000080; // only does knockback on death of target
pub const DAMAGE_IGNORE_TEAM: c_int = 0x00000100; // damage is always done, regardless of teams
pub const DAMAGE_NO_DAMAGE: c_int = 0x00000200; // do no actual damage but react as if damage was taken
pub const DAMAGE_HALF_ABSORB: c_int = 0x00000400; // half shields, half health
pub const DAMAGE_HALF_ARMOR_REDUCTION: c_int = 0x00000800; // This damage doesn't whittle down armor as efficiently.
pub const DAMAGE_HEAVY_WEAP_CLASS: c_int = 0x00001000; // Heavy damage
pub const DAMAGE_NO_HIT_LOC: c_int = 0x00002000; // No hit location
pub const DAMAGE_NO_SELF_PROTECTION: c_int = 0x00004000; // Dont apply half damage to self attacks
pub const DAMAGE_NO_DISMEMBER: c_int = 0x00008000; // Dont do dismemberment
pub const DAMAGE_SABER_KNOCKBACK1: c_int = 0x00010000; // Check the attacker's first saber for a knockbackScale
pub const DAMAGE_SABER_KNOCKBACK2: c_int = 0x00020000; // Check the attacker's second saber for a knockbackScale
pub const DAMAGE_SABER_KNOCKBACK1_B2: c_int = 0x00040000; // Check the attacker's first saber for a knockbackScale2
pub const DAMAGE_SABER_KNOCKBACK2_B2: c_int = 0x00080000; // Check the attacker's second saber for a knockbackScale2

// g_mover.c button spawnflags
pub const SPF_BUTTON_USABLE: c_int = 1;
pub const SPF_BUTTON_FPUSHABLE: c_int = 2;

// g_misc.c reference tags
pub const MAX_REFNAME: usize = 32;
pub const RTF_NONE: c_int = 0;
pub const RTF_NAVGOAL: c_int = 0x00000001;

/// `reference_tag_t` (g_local.h, g_misc.c). Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct reference_tag_t {
    pub name: [c_char; MAX_REFNAME],
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub flags: c_int,  //Just in case
    pub radius: c_int, //For nav goals
    pub inuse: qboolean,
}
const _: () = assert!(core::mem::size_of::<reference_tag_t>() == 68);

/// `MAX_FILEPATH` (g_local.h, ai_main.c).
pub const MAX_FILEPATH: usize = 144;

/// `bot_settings_t` (g_local.h, ai_util.c). Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct bot_settings_t {
    pub personalityfile: [c_char; MAX_FILEPATH],
    pub skill: f32,
    pub team: [c_char; MAX_FILEPATH],
}
const _: () = assert!(core::mem::size_of::<bot_settings_t>() == 292);

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::{offset_of, size_of};

    /// Parity: the HL_* terminal that sizes `gentity_t::locationDamage`.
    #[test]
    fn hl_max_matches_c() {
        unsafe {
            assert_eq!(HL_MAX, jka_gl_HL_MAX());
        }
    }

    /// Parity: the pointer-free local structs match the authentic C `sizeof` /
    /// `offsetof` (arch-independent).
    #[test]
    fn local_struct_layout_matches_c() {
        unsafe {
            assert_eq!(
                size_of::<playerTeamState_t>(),
                jka_gl_sizeof_playerTeamState_t()
            );
            assert_eq!(
                size_of::<clientSession_t>(),
                jka_gl_sizeof_clientSession_t()
            );
            assert_eq!(
                size_of::<clientPersistant_t>(),
                jka_gl_sizeof_clientPersistant_t()
            );
            assert_eq!(
                offset_of!(clientPersistant_t, netname),
                jka_gl_off_cp_netname()
            );
            assert_eq!(
                offset_of!(clientPersistant_t, teamState),
                jka_gl_off_cp_teamState()
            );
        }
    }

    /// Parity: the pointer-bearing master structs (`gentity_t`/`gclient_t`) and
    /// `renderInfo_t` match the authentic C `sizeof`/`offsetof` at the host 64-bit
    /// word size (the `#[cfg(target_pointer_width = "64")]` asserts above pin the
    /// literals; this ties them to the real C compiler).
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn master_struct_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<renderInfo_t>(), jka_gl_sizeof_renderInfo_t());
            assert_eq!(offset_of!(renderInfo_t, lookMode), jka_gl_off_ri_lookMode());
            assert_eq!(offset_of!(renderInfo_t, lastG2), jka_gl_off_ri_lastG2());

            assert_eq!(size_of::<gentity_t>(), jka_gl_sizeof_gentity_t());
            assert_eq!(offset_of!(gentity_t, r), jka_gl_off_ent_r());
            assert_eq!(offset_of!(gentity_t, taskID), jka_gl_off_ent_taskID());
            assert_eq!(offset_of!(gentity_t, client), jka_gl_off_ent_client());
            assert_eq!(
                offset_of!(gentity_t, moverState),
                jka_gl_off_ent_moverState()
            );
            assert_eq!(offset_of!(gentity_t, think), jka_gl_off_ent_think());
            assert_eq!(offset_of!(gentity_t, material), jka_gl_off_ent_material());
            assert_eq!(
                offset_of!(gentity_t, locationDamage),
                jka_gl_off_ent_locationDamage()
            );
            assert_eq!(offset_of!(gentity_t, item), jka_gl_off_ent_item());

            assert_eq!(size_of::<gclient_t>(), jka_gl_sizeof_gclient_t());
            assert_eq!(offset_of!(gclient_t, pers), jka_gl_off_cl_pers());
            assert_eq!(offset_of!(gclient_t, sess), jka_gl_off_cl_sess());
            assert_eq!(offset_of!(gclient_t, saber), jka_gl_off_cl_saber());
            assert_eq!(
                offset_of!(gclient_t, renderInfo),
                jka_gl_off_cl_renderInfo()
            );
            assert_eq!(offset_of!(gclient_t, NPC_class), jka_gl_off_cl_NPC_class());
            assert_eq!(
                offset_of!(gclient_t, lastGenCmdTime),
                jka_gl_off_cl_lastGenCmdTime()
            );
        }
    }

    /// Parity: the pointer-free Part-B local structs (combatPoint_t, waypointData_t,
    /// reference_tag_t, bot_settings_t) match the authentic C (arch-independent).
    #[test]
    fn levellocals_pointerfree_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<combatPoint_t>(), jka_gl_sizeof_combatPoint_t());
            assert_eq!(size_of::<waypointData_t>(), jka_gl_sizeof_waypointData_t());
            assert_eq!(
                size_of::<reference_tag_t>(),
                jka_gl_sizeof_reference_tag_t()
            );
            assert_eq!(size_of::<bot_settings_t>(), jka_gl_sizeof_bot_settings_t());
        }
    }

    /// Parity: `level_locals_t` (game-internal) and its pointer-bearing members
    /// (`interestPoint_t`, `alertEvent_t`) at the host 64-bit layout.
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn level_locals_layout_matches_c() {
        unsafe {
            assert_eq!(
                size_of::<interestPoint_t>(),
                jka_gl_sizeof_interestPoint_t()
            );
            assert_eq!(size_of::<alertEvent_t>(), jka_gl_sizeof_alertEvent_t());
            assert_eq!(offset_of!(alertEvent_t, owner), jka_gl_off_ae_owner());

            assert_eq!(size_of::<level_locals_t>(), jka_gl_sizeof_level_locals_t());
            assert_eq!(offset_of!(level_locals_t, groups), jka_gl_off_ll_groups());
            assert_eq!(
                offset_of!(level_locals_t, combatPoints),
                jka_gl_off_ll_combatPoints()
            );
            assert_eq!(
                offset_of!(level_locals_t, mTeamFilter),
                jka_gl_off_ll_mTeamFilter()
            );
        }
    }
}
