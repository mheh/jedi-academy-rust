//! `bg_public.h` — definitions shared by both the server game and client game
//! modules (the "BG" = both-games layer).
//!
//! This is a large header (≈1670 lines): game-tuning constants, the config-string
//! ranges, ~35 shared enums, the pmove flag sets, and a handful of structs. It is
//! ported **incrementally in logical groups** (one commit per group), mirroring the
//! faithful-first convention used for the rest of the foundation.
//!
//! Header organization notes carried into the port:
//! - The config-string limits (`MAX_MODELS`, `MAX_CONFIGSTRINGS`, …) that the `CS_*`
//!   chain is computed from live in `q_shared.h`, so they are in [`q_shared_h`] and
//!   referenced here.
//! - Forward declarations of `BG_*` functions and `extern` data tables (`bg_itemlist`,
//!   `saberMoveData`, `bgHumanoidAnimations`, …) are not header *data*; they land when
//!   their defining `.c` file is ported, not here.
//! - **Not yet ported** (depend on types/consts from not-yet-ported headers; see
//!   `DEVIATIONS.md`): `bgEntity_t`/`pmove_t` (need `Vehicle_t` from `bg_vehicles.h`
//!   + function pointers), and the `MASK_*` content masks (need the `CONTENTS_*`
//!   surface flags). The inline `BG_GiveMeVectorFromMatrix` has landed (its
//!   `mdxaBone_t` dep is now in [`q_shared_h`]).
//!
//! [`q_shared_h`]: crate::codemp::game::q_shared_h

// Several saber-move enumerators carry double / trailing underscores verbatim from
// the C header (e.g. `LS_A_JUMP_T__B_`, `LS_T1_T___L`), which trips the
// upper-case-globals style lint in rust-analyzer; the names are load-bearing and
// cannot change, so allow it module-wide (per the keep-C-names convention).
#![allow(non_camel_case_types, non_upper_case_globals)]

use crate::codemp::game::bg_vehicles_h::Vehicle_t;
use crate::codemp::game::q_shared_h::{
    entityState_t, mdxaBone_t, playerState_t, qboolean, trace_t, usercmd_t, vec3_t, vec_t,
    MAX_AMBIENT_SETS, MAX_CLIENTS, MAX_CONFIGSTRINGS, MAX_FX, MAX_G2BONES, MAX_ICONS,
    MAX_LIGHT_STYLES, MAX_LOCATIONS, MAX_MODELS, MAX_QPATH, MAX_SOUNDS, MAX_TERRAINS, NEGATIVE_X,
    NEGATIVE_Y, NEGATIVE_Z, ORIGIN, POSITIVE_X, POSITIVE_Y, POSITIVE_Z,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_CORPSE, CONTENTS_LAVA, CONTENTS_MONSTERCLIP, CONTENTS_PLAYERCLIP,
    CONTENTS_SLIME, CONTENTS_SOLID, CONTENTS_TERRAIN, CONTENTS_WATER,
};
use core::ffi::{c_char, c_int, c_uint, c_void};

// ===========================================================================
// Game-tuning / dimension constants (bg_public.h top block).
// ===========================================================================

// these two defs are shared now because we do clientside ent parsing
pub const MAX_SPAWN_VARS: c_int = 64;
pub const MAX_SPAWN_VARS_CHARS: c_int = 4096;

/// `GAME_VERSION` — the BG version string (must match between game and cgame).
/// Retail PC (raven-jediacademy) value is `"basejka-1"`; the Xbox (grayj) tree
/// used `"basejk-1"`. We follow retail PC, which also matches the OpenJK/TaystJK
/// runtime client cgame (it compares its `"basejka-1"` against the server's
/// `CS_GAME_VERSION` and drops on mismatch). See crate/DEVIATIONS.md.
pub const GAME_VERSION: &str = "basejka-1";

pub const STEPSIZE: c_int = 18;

pub const DEFAULT_GRAVITY: c_int = 800;
pub const GIB_HEALTH: c_int = -40;
/// Shields only stop 50% of armor-piercing dmg.
pub const ARMOR_PROTECTION: f32 = 0.50;
/// Certain damage doesn't take off armor as efficiently.
pub const ARMOR_REDUCTION_FACTOR: f32 = 0.50;

pub const JUMP_VELOCITY: c_int = 225; // 270

// ---------------------------------------------------------------------------
// content masks (bg_public.h, "//rwwRMG - added in CONTENTS_TERRAIN"). The
// CONTENTS_* bits come from surfaceflags.h; these composites are bg_public.h's.
// ---------------------------------------------------------------------------
pub const MASK_ALL: c_int = -1;
pub const MASK_SOLID: c_int = CONTENTS_SOLID | CONTENTS_TERRAIN;
pub const MASK_PLAYERSOLID: c_int =
    CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;
pub const MASK_NPCSOLID: c_int =
    CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;
pub const MASK_DEADSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_TERRAIN;
pub const MASK_WATER: c_int = CONTENTS_WATER | CONTENTS_LAVA | CONTENTS_SLIME;
pub const MASK_OPAQUE: c_int = CONTENTS_SOLID | CONTENTS_SLIME | CONTENTS_LAVA | CONTENTS_TERRAIN;
pub const MASK_SHOT: c_int = CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE | CONTENTS_TERRAIN;

pub const MAX_ITEMS: c_int = 256;

pub const RANK_TIED_FLAG: c_int = 0x4000;

/// item sizes are needed for client side pickup detection
pub const ITEM_RADIUS: c_int = 15;

/// for the CS_SCORES[12] when only one player is present
pub const SCORE_NOT_PRESENT: c_int = -9999;

/// 30 seconds before vote times out
pub const VOTE_TIME: c_int = 30000;

pub const DEFAULT_MINS_2: c_int = -24;
pub const DEFAULT_MAXS_2: c_int = 40;
pub const CROUCH_MAXS_2: c_int = 16;
pub const STANDARD_VIEWHEIGHT_OFFSET: c_int = -4;

pub const MINS_Z: c_int = -24;
pub const DEFAULT_VIEWHEIGHT: c_int = DEFAULT_MAXS_2 + STANDARD_VIEWHEIGHT_OFFSET; // 26
pub const CROUCH_VIEWHEIGHT: c_int = CROUCH_MAXS_2 + STANDARD_VIEWHEIGHT_OFFSET; // 12
pub const DEAD_VIEWHEIGHT: c_int = -16;

pub const MAX_CLIENT_SCORE_SEND: c_int = 20;

// ===========================================================================
// Config strings (bg_public.h). A general means of communicating variable
// length strings from the server to all connected clients. CS_SERVERINFO (0)
// and CS_SYSTEMINFO (1) are reserved in q_shared.h (see `q_shared_h`). The tail
// of the range is computed from the q_shared.h config-string limits; the final
// `CS_MAX` must stay <= MAX_CONFIGSTRINGS, which the C enforces with a `#error`
// and we reproduce as a compile-time assert.
//
// This block follows the retail PC (raven-jediacademy) numbering, NOT the Xbox
// (grayj) tree the port originally read. The configstring index is a live
// engine↔client contract: the module writes index N via trap_SetConfigstring and
// the loaded client cgame reads CS_X at N. Retail PC drops the Xbox's
// CS_VOTE_CALLER (12), so everything from CS_TEAMVOTE_TIME up is one lower than
// Xbox, and lands CS_AMBIENT_SET at 37 (Xbox 38). This also matches the OpenJK /
// TaystJK runtime client, which shares the retail-PC numbering; the one OpenJK-
// only extension we keep is CS_LEGACY_FIXES (36), in the unused gap between
// CS_GLOBAL_AMBIENT_SET (32) and CS_AMBIENT_SET (37) — it does not perturb the
// computed CS_MODELS..CS_MAX tail. Using the Xbox numbers makes the client
// misread configstrings (the duel-health string landing in CS_GLOBAL_AMBIENT_SET
// -> "AS_ParseSets: Unable to find ambient soundset", and an empty
// CS_GAME_VERSION -> "Client/Server game mismatch"). See crate/DEVIATIONS.md.
// The bg_public oracle below is pinned to these retail-PC values to match.
// ===========================================================================

pub const CS_MUSIC: c_int = 2;
pub const CS_MESSAGE: c_int = 3; // from the map worldspawn's message field
pub const CS_MOTD: c_int = 4; // g_motd string for server message of the day
pub const CS_WARMUP: c_int = 5; // server time when the match will be restarted
pub const CS_SCORES1: c_int = 6;
pub const CS_SCORES2: c_int = 7;
pub const CS_VOTE_TIME: c_int = 8;
pub const CS_VOTE_STRING: c_int = 9;
pub const CS_VOTE_YES: c_int = 10;
pub const CS_VOTE_NO: c_int = 11;
// NOTE: the Xbox (grayj) tree has CS_VOTE_CALLER = 12 here; retail PC dropped it
// (constant + its g_cmds.c write). We follow retail PC, so this slot is
// CS_TEAMVOTE_TIME and the whole region from here up is one lower than Xbox.

pub const CS_TEAMVOTE_TIME: c_int = 12;
pub const CS_TEAMVOTE_STRING: c_int = 14;
pub const CS_TEAMVOTE_YES: c_int = 16;
pub const CS_TEAMVOTE_NO: c_int = 18;

pub const CS_GAME_VERSION: c_int = 20;
pub const CS_LEVEL_START_TIME: c_int = 21; // so the timer only shows the current level
pub const CS_INTERMISSION: c_int = 22; // when 1, fraglimit/timelimit has been hit and intermission will start in a second or two
pub const CS_FLAGSTATUS: c_int = 23; // string indicating flag status in CTF
pub const CS_SHADERSTATE: c_int = 24;
pub const CS_BOTINFO: c_int = 25;

pub const CS_ITEMS: c_int = 27; // string of 0's and 1's that tell which items are present

pub const CS_CLIENT_JEDIMASTER: c_int = 28; // current jedi master
pub const CS_CLIENT_DUELWINNER: c_int = 29; // current duel round winner - needed for printing at top of scoreboard
pub const CS_CLIENT_DUELISTS: c_int = 30; // client numbers for both current duelists. Needed for a number of client-side things.
pub const CS_CLIENT_DUELHEALTHS: c_int = 31; // nmckenzie: DUEL_HEALTH.  Hopefully adding this cs is safe and good?
pub const CS_GLOBAL_AMBIENT_SET: c_int = 32;

// OpenJK-only extension (absent from both Xbox and retail PC), kept for the
// OpenJK/TaystJK runtime client; sits in the unused gap below CS_AMBIENT_SET so
// it does not shift the computed tail.
pub const CS_LEGACY_FIXES: c_int = 36; // serverside legacy-fix flags bitmask

pub const CS_AMBIENT_SET: c_int = 37;

pub const CS_SIEGE_STATE: c_int = CS_AMBIENT_SET + MAX_AMBIENT_SETS as c_int;
pub const CS_SIEGE_OBJECTIVES: c_int = CS_SIEGE_STATE + 1;
pub const CS_SIEGE_TIMEOVERRIDE: c_int = CS_SIEGE_OBJECTIVES + 1;
pub const CS_SIEGE_WINTEAM: c_int = CS_SIEGE_TIMEOVERRIDE + 1;
pub const CS_SIEGE_ICONS: c_int = CS_SIEGE_WINTEAM + 1;

pub const CS_MODELS: c_int = CS_SIEGE_ICONS + 1;
pub const CS_SKYBOXORG: c_int = CS_MODELS + MAX_MODELS as c_int; // rww - skybox info
pub const CS_SOUNDS: c_int = CS_SKYBOXORG + 1;
pub const CS_ICONS: c_int = CS_SOUNDS + MAX_SOUNDS as c_int;
pub const CS_PLAYERS: c_int = CS_ICONS + MAX_ICONS as c_int;
// Ghoul2 Insert Start / End: CS_G2BONES used to be CS_CHARSKINS.
pub const CS_G2BONES: c_int = CS_PLAYERS + MAX_CLIENTS as c_int;
pub const CS_LOCATIONS: c_int = CS_G2BONES + MAX_G2BONES as c_int;
pub const CS_PARTICLES: c_int = CS_LOCATIONS + MAX_LOCATIONS as c_int;
pub const CS_EFFECTS: c_int = CS_PARTICLES + MAX_LOCATIONS as c_int;
pub const CS_LIGHT_STYLES: c_int = CS_EFFECTS + MAX_FX as c_int;

// rwwRMG - added:
pub const CS_TERRAINS: c_int = CS_LIGHT_STYLES + (MAX_LIGHT_STYLES as c_int * 3);
pub const CS_BSP_MODELS: c_int = CS_TERRAINS + MAX_TERRAINS as c_int;
// `#define MAX_SUB_BSP 32 //rwwRMG - added` (q_shared.h:1931) — commented out in JKA
// alongside the `CS_MAX` form above; reconstructed here for the sub-BSP `G_BSPIndex`
// configstring range (the `misc_bsp` instancing path).
pub const MAX_SUB_BSP: c_int = 32;

// Retail PC (raven-jediacademy): `CS_MAX = (CS_BSP_MODELS + MAX_SUB_BSP)` (no +1).
// The Xbox (grayj) tree commented out the sub-BSP term and used `(CS_BSP_MODELS)+1`.
pub const CS_MAX: c_int = CS_BSP_MODELS + MAX_SUB_BSP;

// C: `#if (CS_MAX) > MAX_CONFIGSTRINGS #error overflow`.
const _: () = assert!(CS_MAX <= MAX_CONFIGSTRINGS as c_int);

// ===========================================================================
// Shared enums (bg_public.h). C enums are int-width and used as plain ints
// throughout the game code, so each becomes a `c_int` alias (named / `typedef
// int` forms) or bare consts (anonymous, un-typedef'd), one `pub const` per
// enumerator -- the same faithful recipe as `anims.rs`. Numbering (incl. the
// explicit `= N` jumps) is carried verbatim and C-oracle-verified below.
// ===========================================================================

// ---------------------------------------------------------------------------
// `g2ModelParts_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `g2ModelParts_t` (bg_public.h).
pub type g2ModelParts_t = c_int;

pub const G2_MODELPART_HEAD: g2ModelParts_t = 10;
pub const G2_MODELPART_WAIST: g2ModelParts_t = 11;
pub const G2_MODELPART_LARM: g2ModelParts_t = 12;
pub const G2_MODELPART_RARM: g2ModelParts_t = 13;
pub const G2_MODELPART_RHAND: g2ModelParts_t = 14;
pub const G2_MODELPART_LLEG: g2ModelParts_t = 15;
pub const G2_MODELPART_RLEG: g2ModelParts_t = 16;

// ---------------------------------------------------------------------------
// `forceHandAnims_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `forceHandAnims_t` (bg_public.h).
pub type forceHandAnims_t = c_int;

pub const HANDEXTEND_NONE: forceHandAnims_t = 0;
pub const HANDEXTEND_FORCEPUSH: forceHandAnims_t = 1;
pub const HANDEXTEND_FORCEPULL: forceHandAnims_t = 2;
pub const HANDEXTEND_FORCE_HOLD: forceHandAnims_t = 3;
pub const HANDEXTEND_SABERPULL: forceHandAnims_t = 4;
pub const HANDEXTEND_CHOKE: forceHandAnims_t = 5; //use handextend priorities to choke someone being gripped
pub const HANDEXTEND_WEAPONREADY: forceHandAnims_t = 6;
pub const HANDEXTEND_DODGE: forceHandAnims_t = 7;
pub const HANDEXTEND_KNOCKDOWN: forceHandAnims_t = 8;
pub const HANDEXTEND_DUELCHALLENGE: forceHandAnims_t = 9;
pub const HANDEXTEND_TAUNT: forceHandAnims_t = 10;

pub const HANDEXTEND_PRETHROW: forceHandAnims_t = 11;
pub const HANDEXTEND_POSTTHROW: forceHandAnims_t = 12;
pub const HANDEXTEND_PRETHROWN: forceHandAnims_t = 13;
pub const HANDEXTEND_POSTTHROWN: forceHandAnims_t = 14;

pub const HANDEXTEND_DRAGGING: forceHandAnims_t = 15;

pub const HANDEXTEND_JEDITAUNT: forceHandAnims_t = 16;

// ---------------------------------------------------------------------------
// `brokenLimb_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `brokenLimb_t` (bg_public.h).
pub type brokenLimb_t = c_int;

pub const BROKENLIMB_NONE: brokenLimb_t = 0;
pub const BROKENLIMB_LARM: brokenLimb_t = 1;
pub const BROKENLIMB_RARM: brokenLimb_t = 2;
pub const NUM_BROKENLIMBS: brokenLimb_t = 3;

// ---------------------------------------------------------------------------
// `gametype_t` (bg_public.h) -- anonymous enum + `typedef int`.
// ---------------------------------------------------------------------------
/// `gametype_t` (bg_public.h).
pub type gametype_t = c_int;

pub const GT_FFA: gametype_t = 0; // free for all
pub const GT_HOLOCRON: gametype_t = 1; // holocron ffa
pub const GT_JEDIMASTER: gametype_t = 2; // jedi master
pub const GT_DUEL: gametype_t = 3; // one on one tournament
pub const GT_POWERDUEL: gametype_t = 4;
pub const GT_SINGLE_PLAYER: gametype_t = 5; // single player ffa

//-- team games go after this --

pub const GT_TEAM: gametype_t = 6; // team deathmatch
pub const GT_SIEGE: gametype_t = 7; // siege
pub const GT_CTF: gametype_t = 8; // capture the flag
pub const GT_CTY: gametype_t = 9;
pub const GT_MAX_GAME_TYPE: gametype_t = 10;

// ---------------------------------------------------------------------------
// `gender_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `gender_t` (bg_public.h).
pub type gender_t = c_int;

pub const GENDER_MALE: gender_t = 0;
pub const GENDER_FEMALE: gender_t = 1;
pub const GENDER_NEUTER: gender_t = 2;

// ---------------------------------------------------------------------------
// `(anonymous enum)` (bg_public.h) -- anonymous enum (no typedef).
// ---------------------------------------------------------------------------

pub const SABERLOCK_TOP: c_int = 0;
pub const SABERLOCK_SIDE: c_int = 1;
pub const SABERLOCK_LOCK: c_int = 2;
pub const SABERLOCK_BREAK: c_int = 3;
pub const SABERLOCK_SUPERBREAK: c_int = 4;
pub const SABERLOCK_WIN: c_int = 5;
pub const SABERLOCK_LOSE: c_int = 6;

// ---------------------------------------------------------------------------
// `(anonymous enum)` (bg_public.h) -- anonymous enum (no typedef).
// ---------------------------------------------------------------------------

pub const DIR_RIGHT: c_int = 0;
pub const DIR_LEFT: c_int = 1;
pub const DIR_FRONT: c_int = 2;
pub const DIR_BACK: c_int = 3;

// ---------------------------------------------------------------------------
// `footstepType_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `footstepType_t` (bg_public.h).
pub type footstepType_t = c_int;

pub const FOOTSTEP_R: footstepType_t = 0;
pub const FOOTSTEP_L: footstepType_t = 1;
pub const FOOTSTEP_HEAVY_R: footstepType_t = 2;
pub const FOOTSTEP_HEAVY_L: footstepType_t = 3;
pub const NUM_FOOTSTEP_TYPES: footstepType_t = 4;

// ---------------------------------------------------------------------------
// `animEventType_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `animEventType_t` (bg_public.h).
pub type animEventType_t = c_int;

//NOTENOTE:  Be sure to update animEventTypeTable and ParseAnimationEvtBlock(...) if you change this enum list!
pub const AEV_NONE: animEventType_t = 0;
pub const AEV_SOUND: animEventType_t = 1; //# animID AEV_SOUND framenum soundpath randomlow randomhi chancetoplay
pub const AEV_FOOTSTEP: animEventType_t = 2; //# animID AEV_FOOTSTEP framenum footstepType chancetoplay
pub const AEV_EFFECT: animEventType_t = 3; //# animID AEV_EFFECT framenum effectpath boltName chancetoplay
pub const AEV_FIRE: animEventType_t = 4; //# animID AEV_FIRE framenum altfire chancetofire
pub const AEV_MOVE: animEventType_t = 5; //# animID AEV_MOVE framenum forwardpush rightpush uppush
pub const AEV_SOUNDCHAN: animEventType_t = 6; //# animID AEV_SOUNDCHAN framenum CHANNEL soundpath randomlow randomhi chancetoplay
                                              // Retail PC (raven-jediacademy) adds these two saber anim events (absent in Xbox),
                                              // shifting AEV_NUM_AEV 7 -> 9.
pub const AEV_SABER_SWING: animEventType_t = 7; //# animID AEV_SABER_SWING framenum CHANNEL randomlow randomhi chancetoplay
pub const AEV_SABER_SPIN: animEventType_t = 8; //# animID AEV_SABER_SPIN framenum CHANNEL chancetoplay
pub const AEV_NUM_AEV: animEventType_t = 9;

// ---------------------------------------------------------------------------
// `pmtype_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `pmtype_t` (bg_public.h).
pub type pmtype_t = c_int;

pub const PM_NORMAL: pmtype_t = 0; // can accelerate and turn
pub const PM_JETPACK: pmtype_t = 1; // special jetpack movement
pub const PM_FLOAT: pmtype_t = 2; // float with no gravity in general direction of velocity (intended for gripping)
pub const PM_NOCLIP: pmtype_t = 3; // noclip movement
pub const PM_SPECTATOR: pmtype_t = 4; // still run into walls
pub const PM_DEAD: pmtype_t = 5; // no acceleration or turning, but free falling
pub const PM_FREEZE: pmtype_t = 6; // stuck in place with no control
pub const PM_INTERMISSION: pmtype_t = 7; // no movement or status bar
pub const PM_SPINTERMISSION: pmtype_t = 8; // no movement or status bar

// ---------------------------------------------------------------------------
// `weaponstate_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `weaponstate_t` (bg_public.h).
pub type weaponstate_t = c_int;

pub const WEAPON_READY: weaponstate_t = 0;
pub const WEAPON_RAISING: weaponstate_t = 1;
pub const WEAPON_DROPPING: weaponstate_t = 2;
pub const WEAPON_FIRING: weaponstate_t = 3;
pub const WEAPON_CHARGING: weaponstate_t = 4;
pub const WEAPON_CHARGING_ALT: weaponstate_t = 5;
pub const WEAPON_IDLE: weaponstate_t = 6; //lowered		// NOTENOTE Added with saber

// ---------------------------------------------------------------------------
// `(anonymous enum)` (bg_public.h) -- anonymous enum (no typedef).
// ---------------------------------------------------------------------------

pub const FORCE_MASTERY_UNINITIATED: c_int = 0;
pub const FORCE_MASTERY_INITIATE: c_int = 1;
pub const FORCE_MASTERY_PADAWAN: c_int = 2;
pub const FORCE_MASTERY_JEDI: c_int = 3;
pub const FORCE_MASTERY_JEDI_GUARDIAN: c_int = 4;
pub const FORCE_MASTERY_JEDI_ADEPT: c_int = 5;
pub const FORCE_MASTERY_JEDI_KNIGHT: c_int = 6;
pub const FORCE_MASTERY_JEDI_MASTER: c_int = 7;
pub const NUM_FORCE_MASTERY_LEVELS: c_int = 8;

// ---------------------------------------------------------------------------
// `statIndex_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `statIndex_t` (bg_public.h).
pub type statIndex_t = c_int;

pub const STAT_HEALTH: statIndex_t = 0;
pub const STAT_HOLDABLE_ITEM: statIndex_t = 1;
pub const STAT_HOLDABLE_ITEMS: statIndex_t = 2;
pub const STAT_PERSISTANT_POWERUP: statIndex_t = 3;
//MAKE SURE STAT_WEAPONS REMAINS 4!!!!
//There is a hardcoded reference in msg.cpp to send it in 32 bits -rww
pub const STAT_WEAPONS: statIndex_t = 4; // 16 bit fields
pub const STAT_ARMOR: statIndex_t = 5;
pub const STAT_DEAD_YAW: statIndex_t = 6; // look this direction when dead (FIXME: get rid of?)
pub const STAT_CLIENTS_READY: statIndex_t = 7; // bit mask of clients wishing to exit the intermission (FIXME: configstring?)
pub const STAT_MAX_HEALTH: statIndex_t = 8; // health / armor limit, changable by handicap

// ---------------------------------------------------------------------------
// `persEnum_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `persEnum_t` (bg_public.h).
pub type persEnum_t = c_int;

pub const PERS_SCORE: persEnum_t = 0; // !!! MUST NOT CHANGE, SERVER AND GAME BOTH REFERENCE !!!
pub const PERS_HITS: persEnum_t = 1; // total points damage inflicted so damage beeps can sound on change
pub const PERS_RANK: persEnum_t = 2; // player rank or team rank
pub const PERS_TEAM: persEnum_t = 3; // player team
pub const PERS_SPAWN_COUNT: persEnum_t = 4; // incremented every respawn
pub const PERS_PLAYEREVENTS: persEnum_t = 5; // 16 bits that can be flipped for events
pub const PERS_ATTACKER: persEnum_t = 6; // clientnum of last damage inflicter
pub const PERS_ATTACKEE_ARMOR: persEnum_t = 7; // health/armor of last person we attacked
pub const PERS_KILLED: persEnum_t = 8; // count of the number of times you died
                                       // player awards tracking
pub const PERS_IMPRESSIVE_COUNT: persEnum_t = 9; // two railgun hits in a row
pub const PERS_EXCELLENT_COUNT: persEnum_t = 10; // two successive kills in a short amount of time
pub const PERS_DEFEND_COUNT: persEnum_t = 11; // defend awards
pub const PERS_ASSIST_COUNT: persEnum_t = 12; // assist awards
pub const PERS_GAUNTLET_FRAG_COUNT: persEnum_t = 13; // kills with the guantlet
pub const PERS_CAPTURES: persEnum_t = 14; // captures

// ---------------------------------------------------------------------------
// `effectTypes_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `effectTypes_t` (bg_public.h).
pub type effectTypes_t = c_int;

pub const EFFECT_NONE: effectTypes_t = 0;
pub const EFFECT_SMOKE: effectTypes_t = 1;
pub const EFFECT_EXPLOSION: effectTypes_t = 2;
pub const EFFECT_EXPLOSION_PAS: effectTypes_t = 3;
pub const EFFECT_SPARK_EXPLOSION: effectTypes_t = 4;
pub const EFFECT_EXPLOSION_TRIPMINE: effectTypes_t = 5;
pub const EFFECT_EXPLOSION_DETPACK: effectTypes_t = 6;
pub const EFFECT_EXPLOSION_FLECHETTE: effectTypes_t = 7;
pub const EFFECT_STUNHIT: effectTypes_t = 8;
pub const EFFECT_EXPLOSION_DEMP2ALT: effectTypes_t = 9;
pub const EFFECT_EXPLOSION_TURRET: effectTypes_t = 10;
pub const EFFECT_SPARKS: effectTypes_t = 11;
pub const EFFECT_WATER_SPLASH: effectTypes_t = 12;
pub const EFFECT_ACID_SPLASH: effectTypes_t = 13;
pub const EFFECT_LAVA_SPLASH: effectTypes_t = 14;
pub const EFFECT_LANDING_MUD: effectTypes_t = 15;
pub const EFFECT_LANDING_SAND: effectTypes_t = 16;
pub const EFFECT_LANDING_DIRT: effectTypes_t = 17;
pub const EFFECT_LANDING_SNOW: effectTypes_t = 18;
pub const EFFECT_LANDING_GRAVEL: effectTypes_t = 19;
pub const EFFECT_MAX: effectTypes_t = 20;

// ---------------------------------------------------------------------------
// `powerup_t` (bg_public.h) -- anonymous enum + `typedef int`.
// ---------------------------------------------------------------------------
/// `powerup_t` (bg_public.h).
pub type powerup_t = c_int;

pub const PW_NONE: powerup_t = 0;

pub const PW_QUAD: powerup_t = 1;
pub const PW_BATTLESUIT: powerup_t = 2;
pub const PW_PULL: powerup_t = 3;
//PW_INVIS, //rww - removed
//PW_REGEN, //rww - removed
//PW_FLIGHT, //rww - removed

pub const PW_REDFLAG: powerup_t = 4;
pub const PW_BLUEFLAG: powerup_t = 5;
pub const PW_NEUTRALFLAG: powerup_t = 6;

pub const PW_SHIELDHIT: powerup_t = 7;

//PW_SCOUT, //rww - removed
//PW_GUARD, //rww - removed
//PW_DOUBLER, //rww - removed
//PW_AMMOREGEN, //rww - removed
pub const PW_SPEEDBURST: powerup_t = 8;
pub const PW_DISINT_4: powerup_t = 9;
pub const PW_SPEED: powerup_t = 10;
pub const PW_CLOAKED: powerup_t = 11;
pub const PW_FORCE_ENLIGHTENED_LIGHT: powerup_t = 12;
pub const PW_FORCE_ENLIGHTENED_DARK: powerup_t = 13;
pub const PW_FORCE_BOON: powerup_t = 14;
pub const PW_YSALAMIRI: powerup_t = 15;

pub const PW_NUM_POWERUPS: powerup_t = 16;

// ---------------------------------------------------------------------------
// `holdable_t` (bg_public.h) -- anonymous enum + `typedef int`.
// ---------------------------------------------------------------------------
/// `holdable_t` (bg_public.h).
pub type holdable_t = c_int;

pub const HI_NONE: holdable_t = 0;

pub const HI_SEEKER: holdable_t = 1;
pub const HI_SHIELD: holdable_t = 2;
pub const HI_MEDPAC: holdable_t = 3;
pub const HI_MEDPAC_BIG: holdable_t = 4;
pub const HI_BINOCULARS: holdable_t = 5;
pub const HI_SENTRY_GUN: holdable_t = 6;
pub const HI_JETPACK: holdable_t = 7;

pub const HI_HEALTHDISP: holdable_t = 8;
pub const HI_AMMODISP: holdable_t = 9;
pub const HI_EWEB: holdable_t = 10;
pub const HI_CLOAK: holdable_t = 11;

pub const HI_NUM_HOLDABLE: holdable_t = 12;

// ---------------------------------------------------------------------------
// `ctfMsg_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `ctfMsg_t` (bg_public.h).
pub type ctfMsg_t = c_int;

pub const CTFMESSAGE_FRAGGED_FLAG_CARRIER: ctfMsg_t = 0;
pub const CTFMESSAGE_FLAG_RETURNED: ctfMsg_t = 1;
pub const CTFMESSAGE_PLAYER_RETURNED_FLAG: ctfMsg_t = 2;
pub const CTFMESSAGE_PLAYER_CAPTURED_FLAG: ctfMsg_t = 3;
pub const CTFMESSAGE_PLAYER_GOT_FLAG: ctfMsg_t = 4;

// ---------------------------------------------------------------------------
// `pdSounds_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `pdSounds_t` (bg_public.h).
pub type pdSounds_t = c_int;

pub const PDSOUND_NONE: pdSounds_t = 0;
pub const PDSOUND_PROTECTHIT: pdSounds_t = 1;
pub const PDSOUND_PROTECT: pdSounds_t = 2;
pub const PDSOUND_ABSORBHIT: pdSounds_t = 3;
pub const PDSOUND_ABSORB: pdSounds_t = 4;
pub const PDSOUND_FORCEJUMP: pdSounds_t = 5;
pub const PDSOUND_FORCEGRIP: pdSounds_t = 6;

// ---------------------------------------------------------------------------
// `entity_event_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `entity_event_t` (bg_public.h). There is a maximum of 256 events (8 bits transmission, 2 high bits for uniqueness)
pub type entity_event_t = c_int;

pub const EV_NONE: entity_event_t = 0;

pub const EV_CLIENTJOIN: entity_event_t = 1;

pub const EV_FOOTSTEP: entity_event_t = 2;
pub const EV_FOOTSTEP_METAL: entity_event_t = 3;
pub const EV_FOOTSPLASH: entity_event_t = 4;
pub const EV_FOOTWADE: entity_event_t = 5;
pub const EV_SWIM: entity_event_t = 6;

pub const EV_STEP_4: entity_event_t = 7;
pub const EV_STEP_8: entity_event_t = 8;
pub const EV_STEP_12: entity_event_t = 9;
pub const EV_STEP_16: entity_event_t = 10;

pub const EV_FALL: entity_event_t = 11;

pub const EV_JUMP_PAD: entity_event_t = 12; // boing sound at origin, jump sound on player

pub const EV_GHOUL2_MARK: entity_event_t = 13; //create a projectile impact mark on something with a client-side g2 instance.

pub const EV_GLOBAL_DUEL: entity_event_t = 14;
pub const EV_PRIVATE_DUEL: entity_event_t = 15;

pub const EV_JUMP: entity_event_t = 16;
pub const EV_ROLL: entity_event_t = 17;
pub const EV_WATER_TOUCH: entity_event_t = 18; // foot touches
pub const EV_WATER_LEAVE: entity_event_t = 19; // foot leaves
pub const EV_WATER_UNDER: entity_event_t = 20; // head touches
pub const EV_WATER_CLEAR: entity_event_t = 21; // head leaves

pub const EV_ITEM_PICKUP: entity_event_t = 22; // normal item pickups are predictable
pub const EV_GLOBAL_ITEM_PICKUP: entity_event_t = 23; // powerup / team sounds are broadcast to everyone

pub const EV_VEH_FIRE: entity_event_t = 24;

pub const EV_NOAMMO: entity_event_t = 25;
pub const EV_CHANGE_WEAPON: entity_event_t = 26;
pub const EV_FIRE_WEAPON: entity_event_t = 27;
pub const EV_ALT_FIRE: entity_event_t = 28;
pub const EV_SABER_ATTACK: entity_event_t = 29;
pub const EV_SABER_HIT: entity_event_t = 30;
pub const EV_SABER_BLOCK: entity_event_t = 31;
pub const EV_SABER_CLASHFLARE: entity_event_t = 32;
pub const EV_SABER_UNHOLSTER: entity_event_t = 33;
pub const EV_BECOME_JEDIMASTER: entity_event_t = 34;
pub const EV_DISRUPTOR_MAIN_SHOT: entity_event_t = 35;
pub const EV_DISRUPTOR_SNIPER_SHOT: entity_event_t = 36;
pub const EV_DISRUPTOR_SNIPER_MISS: entity_event_t = 37;
pub const EV_DISRUPTOR_HIT: entity_event_t = 38;
pub const EV_DISRUPTOR_ZOOMSOUND: entity_event_t = 39;

pub const EV_PREDEFSOUND: entity_event_t = 40;

pub const EV_TEAM_POWER: entity_event_t = 41;

pub const EV_SCREENSHAKE: entity_event_t = 42;

pub const EV_LOCALTIMER: entity_event_t = 43;

pub const EV_USE: entity_event_t = 44; // +Use key

pub const EV_USE_ITEM0: entity_event_t = 45;
pub const EV_USE_ITEM1: entity_event_t = 46;
pub const EV_USE_ITEM2: entity_event_t = 47;
pub const EV_USE_ITEM3: entity_event_t = 48;
pub const EV_USE_ITEM4: entity_event_t = 49;
pub const EV_USE_ITEM5: entity_event_t = 50;
pub const EV_USE_ITEM6: entity_event_t = 51;
pub const EV_USE_ITEM7: entity_event_t = 52;
pub const EV_USE_ITEM8: entity_event_t = 53;
pub const EV_USE_ITEM9: entity_event_t = 54;
pub const EV_USE_ITEM10: entity_event_t = 55;
pub const EV_USE_ITEM11: entity_event_t = 56;
pub const EV_USE_ITEM12: entity_event_t = 57;
pub const EV_USE_ITEM13: entity_event_t = 58;
pub const EV_USE_ITEM14: entity_event_t = 59;
pub const EV_USE_ITEM15: entity_event_t = 60;

pub const EV_ITEMUSEFAIL: entity_event_t = 61;

pub const EV_ITEM_RESPAWN: entity_event_t = 62;
pub const EV_ITEM_POP: entity_event_t = 63;
pub const EV_PLAYER_TELEPORT_IN: entity_event_t = 64;
pub const EV_PLAYER_TELEPORT_OUT: entity_event_t = 65;

pub const EV_GRENADE_BOUNCE: entity_event_t = 66; // eventParm will be the soundindex
pub const EV_MISSILE_STICK: entity_event_t = 67; // eventParm will be the soundindex

pub const EV_PLAY_EFFECT: entity_event_t = 68;
pub const EV_PLAY_EFFECT_ID: entity_event_t = 69;
pub const EV_PLAY_PORTAL_EFFECT_ID: entity_event_t = 70;

pub const EV_PLAYDOORSOUND: entity_event_t = 71;
pub const EV_PLAYDOORLOOPSOUND: entity_event_t = 72;
pub const EV_BMODEL_SOUND: entity_event_t = 73;

pub const EV_MUTE_SOUND: entity_event_t = 74;
pub const EV_VOICECMD_SOUND: entity_event_t = 75;
pub const EV_GENERAL_SOUND: entity_event_t = 76;
pub const EV_GLOBAL_SOUND: entity_event_t = 77; // no attenuation
pub const EV_GLOBAL_TEAM_SOUND: entity_event_t = 78;
pub const EV_ENTITY_SOUND: entity_event_t = 79;

pub const EV_PLAY_ROFF: entity_event_t = 80;

pub const EV_GLASS_SHATTER: entity_event_t = 81;
pub const EV_DEBRIS: entity_event_t = 82;
pub const EV_MISC_MODEL_EXP: entity_event_t = 83;

pub const EV_CONC_ALT_IMPACT: entity_event_t = 84;

pub const EV_MISSILE_HIT: entity_event_t = 85;
pub const EV_MISSILE_MISS: entity_event_t = 86;
pub const EV_MISSILE_MISS_METAL: entity_event_t = 87;
pub const EV_BULLET: entity_event_t = 88; // otherEntity is the shooter

pub const EV_PAIN: entity_event_t = 89;
pub const EV_DEATH1: entity_event_t = 90;
pub const EV_DEATH2: entity_event_t = 91;
pub const EV_DEATH3: entity_event_t = 92;
pub const EV_OBITUARY: entity_event_t = 93;

pub const EV_POWERUP_QUAD: entity_event_t = 94;
pub const EV_POWERUP_BATTLESUIT: entity_event_t = 95;
//EV_POWERUP_REGEN,

pub const EV_FORCE_DRAINED: entity_event_t = 96;

pub const EV_GIB_PLAYER: entity_event_t = 97; // gib a previously living player
pub const EV_SCOREPLUM: entity_event_t = 98; // score plum

pub const EV_CTFMESSAGE: entity_event_t = 99;

pub const EV_BODYFADE: entity_event_t = 100;

pub const EV_SIEGE_ROUNDOVER: entity_event_t = 101;
pub const EV_SIEGE_OBJECTIVECOMPLETE: entity_event_t = 102;

pub const EV_DESTROY_GHOUL2_INSTANCE: entity_event_t = 103;

pub const EV_DESTROY_WEAPON_MODEL: entity_event_t = 104;

pub const EV_GIVE_NEW_RANK: entity_event_t = 105;
pub const EV_SET_FREE_SABER: entity_event_t = 106;
pub const EV_SET_FORCE_DISABLE: entity_event_t = 107;

pub const EV_WEAPON_CHARGE: entity_event_t = 108;
pub const EV_WEAPON_CHARGE_ALT: entity_event_t = 109;

pub const EV_SHIELD_HIT: entity_event_t = 110;

pub const EV_DEBUG_LINE: entity_event_t = 111;
pub const EV_TESTLINE: entity_event_t = 112;
pub const EV_STOPLOOPINGSOUND: entity_event_t = 113;
pub const EV_STARTLOOPINGSOUND: entity_event_t = 114;
pub const EV_TAUNT: entity_event_t = 115;

//rww - Begin NPC sound events
pub const EV_ANGER1: entity_event_t = 116; //Say when acquire an enemy when didn't have one before
pub const EV_ANGER2: entity_event_t = 117;
pub const EV_ANGER3: entity_event_t = 118;

pub const EV_VICTORY1: entity_event_t = 119; //Say when killed an enemy
pub const EV_VICTORY2: entity_event_t = 120;
pub const EV_VICTORY3: entity_event_t = 121;

pub const EV_CONFUSE1: entity_event_t = 122; //Say when confused
pub const EV_CONFUSE2: entity_event_t = 123;
pub const EV_CONFUSE3: entity_event_t = 124;

pub const EV_PUSHED1: entity_event_t = 125; //Say when pushed
pub const EV_PUSHED2: entity_event_t = 126;
pub const EV_PUSHED3: entity_event_t = 127;

pub const EV_CHOKE1: entity_event_t = 128; //Say when choking
pub const EV_CHOKE2: entity_event_t = 129;
pub const EV_CHOKE3: entity_event_t = 130;

pub const EV_FFWARN: entity_event_t = 131; //ffire founds
pub const EV_FFTURN: entity_event_t = 132;
//extra sounds for ST
pub const EV_CHASE1: entity_event_t = 133;
pub const EV_CHASE2: entity_event_t = 134;
pub const EV_CHASE3: entity_event_t = 135;
pub const EV_COVER1: entity_event_t = 136;
pub const EV_COVER2: entity_event_t = 137;
pub const EV_COVER3: entity_event_t = 138;
pub const EV_COVER4: entity_event_t = 139;
pub const EV_COVER5: entity_event_t = 140;
pub const EV_DETECTED1: entity_event_t = 141;
pub const EV_DETECTED2: entity_event_t = 142;
pub const EV_DETECTED3: entity_event_t = 143;
pub const EV_DETECTED4: entity_event_t = 144;
pub const EV_DETECTED5: entity_event_t = 145;
pub const EV_LOST1: entity_event_t = 146;
pub const EV_OUTFLANK1: entity_event_t = 147;
pub const EV_OUTFLANK2: entity_event_t = 148;
pub const EV_ESCAPING1: entity_event_t = 149;
pub const EV_ESCAPING2: entity_event_t = 150;
pub const EV_ESCAPING3: entity_event_t = 151;
pub const EV_GIVEUP1: entity_event_t = 152;
pub const EV_GIVEUP2: entity_event_t = 153;
pub const EV_GIVEUP3: entity_event_t = 154;
pub const EV_GIVEUP4: entity_event_t = 155;
pub const EV_LOOK1: entity_event_t = 156;
pub const EV_LOOK2: entity_event_t = 157;
pub const EV_SIGHT1: entity_event_t = 158;
pub const EV_SIGHT2: entity_event_t = 159;
pub const EV_SIGHT3: entity_event_t = 160;
pub const EV_SOUND1: entity_event_t = 161;
pub const EV_SOUND2: entity_event_t = 162;
pub const EV_SOUND3: entity_event_t = 163;
pub const EV_SUSPICIOUS1: entity_event_t = 164;
pub const EV_SUSPICIOUS2: entity_event_t = 165;
pub const EV_SUSPICIOUS3: entity_event_t = 166;
pub const EV_SUSPICIOUS4: entity_event_t = 167;
pub const EV_SUSPICIOUS5: entity_event_t = 168;
//extra sounds for Jedi
pub const EV_COMBAT1: entity_event_t = 169;
pub const EV_COMBAT2: entity_event_t = 170;
pub const EV_COMBAT3: entity_event_t = 171;
pub const EV_JDETECTED1: entity_event_t = 172;
pub const EV_JDETECTED2: entity_event_t = 173;
pub const EV_JDETECTED3: entity_event_t = 174;
pub const EV_TAUNT1: entity_event_t = 175;
pub const EV_TAUNT2: entity_event_t = 176;
pub const EV_TAUNT3: entity_event_t = 177;
pub const EV_JCHASE1: entity_event_t = 178;
pub const EV_JCHASE2: entity_event_t = 179;
pub const EV_JCHASE3: entity_event_t = 180;
pub const EV_JLOST1: entity_event_t = 181;
pub const EV_JLOST2: entity_event_t = 182;
pub const EV_JLOST3: entity_event_t = 183;
pub const EV_DEFLECT1: entity_event_t = 184;
pub const EV_DEFLECT2: entity_event_t = 185;
pub const EV_DEFLECT3: entity_event_t = 186;
pub const EV_GLOAT1: entity_event_t = 187;
pub const EV_GLOAT2: entity_event_t = 188;
pub const EV_GLOAT3: entity_event_t = 189;
pub const EV_PUSHFAIL: entity_event_t = 190;

pub const EV_SIEGESPEC: entity_event_t = 191;

// ---------------------------------------------------------------------------
// `global_team_sound_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `global_team_sound_t` (bg_public.h).
pub type global_team_sound_t = c_int;

pub const GTS_RED_CAPTURE: global_team_sound_t = 0;
pub const GTS_BLUE_CAPTURE: global_team_sound_t = 1;
pub const GTS_RED_RETURN: global_team_sound_t = 2;
pub const GTS_BLUE_RETURN: global_team_sound_t = 3;
pub const GTS_RED_TAKEN: global_team_sound_t = 4;
pub const GTS_BLUE_TAKEN: global_team_sound_t = 5;
pub const GTS_REDTEAM_SCORED: global_team_sound_t = 6;
pub const GTS_BLUETEAM_SCORED: global_team_sound_t = 7;
pub const GTS_REDTEAM_TOOK_LEAD: global_team_sound_t = 8;
pub const GTS_BLUETEAM_TOOK_LEAD: global_team_sound_t = 9;
pub const GTS_TEAMS_ARE_TIED: global_team_sound_t = 10;

// ---------------------------------------------------------------------------
// `team_t` (bg_public.h) -- anonymous enum + `typedef int`.
// ---------------------------------------------------------------------------
/// `team_t` (bg_public.h).
pub type team_t = c_int;

pub const TEAM_FREE: team_t = 0;
pub const TEAM_RED: team_t = 1;
pub const TEAM_BLUE: team_t = 2;
pub const TEAM_SPECTATOR: team_t = 3;

pub const TEAM_NUM_TEAMS: team_t = 4;

// ---------------------------------------------------------------------------
// `duelTeam_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `duelTeam_t` (bg_public.h).
pub type duelTeam_t = c_int;

pub const DUELTEAM_FREE: duelTeam_t = 0;
pub const DUELTEAM_LONE: duelTeam_t = 1;
pub const DUELTEAM_DOUBLE: duelTeam_t = 2;

pub const DUELTEAM_SINGLE: duelTeam_t = 3; // for regular duel matches (not power duel)

// ---------------------------------------------------------------------------
// `teamtask_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `teamtask_t` (bg_public.h).
pub type teamtask_t = c_int;

pub const TEAMTASK_NONE: teamtask_t = 0;
pub const TEAMTASK_OFFENSE: teamtask_t = 1;
pub const TEAMTASK_DEFENSE: teamtask_t = 2;
pub const TEAMTASK_PATROL: teamtask_t = 3;
pub const TEAMTASK_FOLLOW: teamtask_t = 4;
pub const TEAMTASK_RETRIEVE: teamtask_t = 5;
pub const TEAMTASK_ESCORT: teamtask_t = 6;
pub const TEAMTASK_CAMP: teamtask_t = 7;

// ---------------------------------------------------------------------------
// `meansOfDeath_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `meansOfDeath_t` (bg_public.h).
pub type meansOfDeath_t = c_int;

pub const MOD_UNKNOWN: meansOfDeath_t = 0;
pub const MOD_STUN_BATON: meansOfDeath_t = 1;
pub const MOD_MELEE: meansOfDeath_t = 2;
pub const MOD_SABER: meansOfDeath_t = 3;
pub const MOD_BRYAR_PISTOL: meansOfDeath_t = 4;
pub const MOD_BRYAR_PISTOL_ALT: meansOfDeath_t = 5;
pub const MOD_BLASTER: meansOfDeath_t = 6;
pub const MOD_TURBLAST: meansOfDeath_t = 7;
pub const MOD_DISRUPTOR: meansOfDeath_t = 8;
pub const MOD_DISRUPTOR_SPLASH: meansOfDeath_t = 9;
pub const MOD_DISRUPTOR_SNIPER: meansOfDeath_t = 10;
pub const MOD_BOWCASTER: meansOfDeath_t = 11;
pub const MOD_REPEATER: meansOfDeath_t = 12;
pub const MOD_REPEATER_ALT: meansOfDeath_t = 13;
pub const MOD_REPEATER_ALT_SPLASH: meansOfDeath_t = 14;
pub const MOD_DEMP2: meansOfDeath_t = 15;
pub const MOD_DEMP2_ALT: meansOfDeath_t = 16;
pub const MOD_FLECHETTE: meansOfDeath_t = 17;
pub const MOD_FLECHETTE_ALT_SPLASH: meansOfDeath_t = 18;
pub const MOD_ROCKET: meansOfDeath_t = 19;
pub const MOD_ROCKET_SPLASH: meansOfDeath_t = 20;
pub const MOD_ROCKET_HOMING: meansOfDeath_t = 21;
pub const MOD_ROCKET_HOMING_SPLASH: meansOfDeath_t = 22;
pub const MOD_THERMAL: meansOfDeath_t = 23;
pub const MOD_THERMAL_SPLASH: meansOfDeath_t = 24;
pub const MOD_TRIP_MINE_SPLASH: meansOfDeath_t = 25;
pub const MOD_TIMED_MINE_SPLASH: meansOfDeath_t = 26;
pub const MOD_DET_PACK_SPLASH: meansOfDeath_t = 27;
pub const MOD_VEHICLE: meansOfDeath_t = 28;
pub const MOD_CONC: meansOfDeath_t = 29;
pub const MOD_CONC_ALT: meansOfDeath_t = 30;
pub const MOD_FORCE_DARK: meansOfDeath_t = 31;
pub const MOD_SENTRY: meansOfDeath_t = 32;
pub const MOD_WATER: meansOfDeath_t = 33;
pub const MOD_SLIME: meansOfDeath_t = 34;
pub const MOD_LAVA: meansOfDeath_t = 35;
pub const MOD_CRUSH: meansOfDeath_t = 36;
pub const MOD_TELEFRAG: meansOfDeath_t = 37;
pub const MOD_FALLING: meansOfDeath_t = 38;
// Retail PC (raven-jediacademy) inserts these two after MOD_FALLING (absent in
// Xbox), shifting MOD_SUICIDE..MOD_MAX up by 2.
pub const MOD_COLLISION: meansOfDeath_t = 39;
pub const MOD_VEH_EXPLOSION: meansOfDeath_t = 40;
pub const MOD_SUICIDE: meansOfDeath_t = 41;
pub const MOD_TARGET_LASER: meansOfDeath_t = 42;
pub const MOD_TRIGGER_HURT: meansOfDeath_t = 43;
pub const MOD_TEAM_CHANGE: meansOfDeath_t = 44;
//AURELIO: when/if you put this back in, remember to make a case for it in all the other places where
//mod's are checked. Also, it probably isn't the most elegant solution for what you want - just add
//a frag back to the player after you call the player_die (and keep a local of his pre-death score to
//make sure he actually lost points, there may be cases where you don't lose points on changing teams
//or suiciding, and so you would actually be giving him a point) -Rich
// I put it back in for now, if it becomes a problem we'll work around it later (it shouldn't though)...
pub const MOD_MAX: meansOfDeath_t = 45;

// ---------------------------------------------------------------------------
// `itemType_t` (bg_public.h) -- anonymous enum + `typedef int`.
// ---------------------------------------------------------------------------
/// `itemType_t` (bg_public.h).
pub type itemType_t = c_int;

pub const IT_BAD: itemType_t = 0;
pub const IT_WEAPON: itemType_t = 1; // EFX: rotate + upscale + minlight
pub const IT_AMMO: itemType_t = 2; // EFX: rotate
pub const IT_ARMOR: itemType_t = 3; // EFX: rotate + minlight
pub const IT_HEALTH: itemType_t = 4; // EFX: static external sphere + rotating internal
pub const IT_POWERUP: itemType_t = 5; // instant on, timer based
                                      // EFX: rotate + external ring that rotates
pub const IT_HOLDABLE: itemType_t = 6; // single use, holdable item
                                       // EFX: rotate + bob
pub const IT_PERSISTANT_POWERUP: itemType_t = 7;
pub const IT_TEAM: itemType_t = 8;

// ---------------------------------------------------------------------------
// `entityType_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `entityType_t` (bg_public.h).
pub type entityType_t = c_int;

pub const ET_GENERAL: entityType_t = 0;
pub const ET_PLAYER: entityType_t = 1;
pub const ET_ITEM: entityType_t = 2;
pub const ET_MISSILE: entityType_t = 3;
pub const ET_SPECIAL: entityType_t = 4; // rww - force fields
pub const ET_HOLOCRON: entityType_t = 5; // rww - holocron icon displays
pub const ET_MOVER: entityType_t = 6;
pub const ET_BEAM: entityType_t = 7;
pub const ET_PORTAL: entityType_t = 8;
pub const ET_SPEAKER: entityType_t = 9;
pub const ET_PUSH_TRIGGER: entityType_t = 10;
pub const ET_TELEPORT_TRIGGER: entityType_t = 11;
pub const ET_INVISIBLE: entityType_t = 12;
pub const ET_NPC: entityType_t = 13; // ghoul2 player-like entity
pub const ET_TEAM: entityType_t = 14;
pub const ET_BODY: entityType_t = 15;
pub const ET_TERRAIN: entityType_t = 16;
pub const ET_FX: entityType_t = 17;

pub const ET_EVENTS: entityType_t = 18; // any of the EV_* events can be added freestanding
                                        // by setting eType to ET_EVENTS + eventNum
                                        // this avoids having to set eFlags and eventNum

// ---------------------------------------------------------------------------
// `fieldtype_t` (bg_public.h) -- typedef enum.
// In C this is under `#ifdef _GAME_SIDE` (defined when QAGAME or CGAME is) -- it
// is the spawn-field type tag used by g_spawn. The Rust port is the server game
// module (QAGAME), so it is always present here; kept unconditional.
// ---------------------------------------------------------------------------
/// `fieldtype_t` (bg_public.h) — under `#ifdef _GAME_SIDE` in C (always on for the
/// game module). Spawn-field type tags (`F_INT`/`F_FLOAT`/… consumed by g_spawn).
pub type fieldtype_t = c_int;

pub const F_INT: fieldtype_t = 0;
pub const F_FLOAT: fieldtype_t = 1;
pub const F_LSTRING: fieldtype_t = 2; // string on disk, pointer in memory, TAG_LEVEL
pub const F_GSTRING: fieldtype_t = 3; // string on disk, pointer in memory, TAG_GAME
pub const F_VECTOR: fieldtype_t = 4;
pub const F_ANGLEHACK: fieldtype_t = 5;
pub const F_ENTITY: fieldtype_t = 6; // index on disk, pointer in memory
pub const F_ITEM: fieldtype_t = 7; // index on disk, pointer in memory
pub const F_CLIENT: fieldtype_t = 8; // index on disk, pointer in memory
pub const F_PARM1: fieldtype_t = 9; // Special case for parms
pub const F_PARM2: fieldtype_t = 10; // Special case for parms
pub const F_PARM3: fieldtype_t = 11; // Special case for parms
pub const F_PARM4: fieldtype_t = 12; // Special case for parms
pub const F_PARM5: fieldtype_t = 13; // Special case for parms
pub const F_PARM6: fieldtype_t = 14; // Special case for parms
pub const F_PARM7: fieldtype_t = 15; // Special case for parms
pub const F_PARM8: fieldtype_t = 16; // Special case for parms
pub const F_PARM9: fieldtype_t = 17; // Special case for parms
pub const F_PARM10: fieldtype_t = 18; // Special case for parms
pub const F_PARM11: fieldtype_t = 19; // Special case for parms
pub const F_PARM12: fieldtype_t = 20; // Special case for parms
pub const F_PARM13: fieldtype_t = 21; // Special case for parms
pub const F_PARM14: fieldtype_t = 22; // Special case for parms
pub const F_PARM15: fieldtype_t = 23; // Special case for parms
pub const F_PARM16: fieldtype_t = 24; // Special case for parms
pub const F_IGNORE: fieldtype_t = 25;

// ---------------------------------------------------------------------------
// `saberMoveName_t` (bg_public.h) -- anonymous enum + `typedef int`.
// ---------------------------------------------------------------------------
/// `saberMoveName_t` (bg_public.h).
pub type saberMoveName_t = c_int;

// totally invalid (retail PC / raven-jediacademy only; absent in Xbox)
pub const LS_INVALID: saberMoveName_t = -1;
// Invalid, or saber not armed
pub const LS_NONE: saberMoveName_t = 0;

// General movements with saber
pub const LS_READY: saberMoveName_t = 1;
pub const LS_DRAW: saberMoveName_t = 2;
pub const LS_PUTAWAY: saberMoveName_t = 3;

// Attacks
pub const LS_A_TL2BR: saberMoveName_t = 4; //4
pub const LS_A_L2R: saberMoveName_t = 5;
pub const LS_A_BL2TR: saberMoveName_t = 6;
pub const LS_A_BR2TL: saberMoveName_t = 7;
pub const LS_A_R2L: saberMoveName_t = 8;
pub const LS_A_TR2BL: saberMoveName_t = 9;
pub const LS_A_T2B: saberMoveName_t = 10;
pub const LS_A_BACKSTAB: saberMoveName_t = 11;
pub const LS_A_BACK: saberMoveName_t = 12;
pub const LS_A_BACK_CR: saberMoveName_t = 13;
pub const LS_ROLL_STAB: saberMoveName_t = 14;
pub const LS_A_LUNGE: saberMoveName_t = 15;
pub const LS_A_JUMP_T__B_: saberMoveName_t = 16;
pub const LS_A_FLIP_STAB: saberMoveName_t = 17;
pub const LS_A_FLIP_SLASH: saberMoveName_t = 18;
pub const LS_JUMPATTACK_DUAL: saberMoveName_t = 19;
pub const LS_JUMPATTACK_ARIAL_LEFT: saberMoveName_t = 20;
pub const LS_JUMPATTACK_ARIAL_RIGHT: saberMoveName_t = 21;
pub const LS_JUMPATTACK_CART_LEFT: saberMoveName_t = 22;
pub const LS_JUMPATTACK_CART_RIGHT: saberMoveName_t = 23;
pub const LS_JUMPATTACK_STAFF_LEFT: saberMoveName_t = 24;
pub const LS_JUMPATTACK_STAFF_RIGHT: saberMoveName_t = 25;
pub const LS_BUTTERFLY_LEFT: saberMoveName_t = 26;
pub const LS_BUTTERFLY_RIGHT: saberMoveName_t = 27;
pub const LS_A_BACKFLIP_ATK: saberMoveName_t = 28;
pub const LS_SPINATTACK_DUAL: saberMoveName_t = 29;
pub const LS_SPINATTACK: saberMoveName_t = 30;
pub const LS_LEAP_ATTACK: saberMoveName_t = 31;
pub const LS_SWOOP_ATTACK_RIGHT: saberMoveName_t = 32;
pub const LS_SWOOP_ATTACK_LEFT: saberMoveName_t = 33;
pub const LS_TAUNTAUN_ATTACK_RIGHT: saberMoveName_t = 34;
pub const LS_TAUNTAUN_ATTACK_LEFT: saberMoveName_t = 35;
pub const LS_KICK_F: saberMoveName_t = 36;
pub const LS_KICK_B: saberMoveName_t = 37;
pub const LS_KICK_R: saberMoveName_t = 38;
pub const LS_KICK_L: saberMoveName_t = 39;
pub const LS_KICK_S: saberMoveName_t = 40;
pub const LS_KICK_BF: saberMoveName_t = 41;
pub const LS_KICK_RL: saberMoveName_t = 42;
pub const LS_KICK_F_AIR: saberMoveName_t = 43;
pub const LS_KICK_B_AIR: saberMoveName_t = 44;
pub const LS_KICK_R_AIR: saberMoveName_t = 45;
pub const LS_KICK_L_AIR: saberMoveName_t = 46;
pub const LS_STABDOWN: saberMoveName_t = 47;
pub const LS_STABDOWN_STAFF: saberMoveName_t = 48;
pub const LS_STABDOWN_DUAL: saberMoveName_t = 49;
pub const LS_DUAL_SPIN_PROTECT: saberMoveName_t = 50;
pub const LS_STAFF_SOULCAL: saberMoveName_t = 51;
pub const LS_A1_SPECIAL: saberMoveName_t = 52;
pub const LS_A2_SPECIAL: saberMoveName_t = 53;
pub const LS_A3_SPECIAL: saberMoveName_t = 54;
pub const LS_UPSIDE_DOWN_ATTACK: saberMoveName_t = 55;
pub const LS_PULL_ATTACK_STAB: saberMoveName_t = 56;
pub const LS_PULL_ATTACK_SWING: saberMoveName_t = 57;
pub const LS_SPINATTACK_ALORA: saberMoveName_t = 58;
pub const LS_DUAL_FB: saberMoveName_t = 59;
pub const LS_DUAL_LR: saberMoveName_t = 60;
pub const LS_HILT_BASH: saberMoveName_t = 61;

//starts
pub const LS_S_TL2BR: saberMoveName_t = 62; //26
pub const LS_S_L2R: saberMoveName_t = 63;
pub const LS_S_BL2TR: saberMoveName_t = 64; //# Start of attack chaining to SLASH LR2UL
pub const LS_S_BR2TL: saberMoveName_t = 65; //# Start of attack chaining to SLASH LR2UL
pub const LS_S_R2L: saberMoveName_t = 66;
pub const LS_S_TR2BL: saberMoveName_t = 67;
pub const LS_S_T2B: saberMoveName_t = 68;

//returns
pub const LS_R_TL2BR: saberMoveName_t = 69; //33
pub const LS_R_L2R: saberMoveName_t = 70;
pub const LS_R_BL2TR: saberMoveName_t = 71;
pub const LS_R_BR2TL: saberMoveName_t = 72;
pub const LS_R_R2L: saberMoveName_t = 73;
pub const LS_R_TR2BL: saberMoveName_t = 74;
pub const LS_R_T2B: saberMoveName_t = 75;

//transitions
pub const LS_T1_BR__R: saberMoveName_t = 76; //40
pub const LS_T1_BR_TR: saberMoveName_t = 77;
pub const LS_T1_BR_T_: saberMoveName_t = 78;
pub const LS_T1_BR_TL: saberMoveName_t = 79;
pub const LS_T1_BR__L: saberMoveName_t = 80;
pub const LS_T1_BR_BL: saberMoveName_t = 81;
pub const LS_T1__R_BR: saberMoveName_t = 82; //46
pub const LS_T1__R_TR: saberMoveName_t = 83;
pub const LS_T1__R_T_: saberMoveName_t = 84;
pub const LS_T1__R_TL: saberMoveName_t = 85;
pub const LS_T1__R__L: saberMoveName_t = 86;
pub const LS_T1__R_BL: saberMoveName_t = 87;
pub const LS_T1_TR_BR: saberMoveName_t = 88; //52
pub const LS_T1_TR__R: saberMoveName_t = 89;
pub const LS_T1_TR_T_: saberMoveName_t = 90;
pub const LS_T1_TR_TL: saberMoveName_t = 91;
pub const LS_T1_TR__L: saberMoveName_t = 92;
pub const LS_T1_TR_BL: saberMoveName_t = 93;
pub const LS_T1_T__BR: saberMoveName_t = 94; //58
pub const LS_T1_T___R: saberMoveName_t = 95;
pub const LS_T1_T__TR: saberMoveName_t = 96;
pub const LS_T1_T__TL: saberMoveName_t = 97;
pub const LS_T1_T___L: saberMoveName_t = 98;
pub const LS_T1_T__BL: saberMoveName_t = 99;
pub const LS_T1_TL_BR: saberMoveName_t = 100; //64
pub const LS_T1_TL__R: saberMoveName_t = 101;
pub const LS_T1_TL_TR: saberMoveName_t = 102;
pub const LS_T1_TL_T_: saberMoveName_t = 103;
pub const LS_T1_TL__L: saberMoveName_t = 104;
pub const LS_T1_TL_BL: saberMoveName_t = 105;
pub const LS_T1__L_BR: saberMoveName_t = 106; //70
pub const LS_T1__L__R: saberMoveName_t = 107;
pub const LS_T1__L_TR: saberMoveName_t = 108;
pub const LS_T1__L_T_: saberMoveName_t = 109;
pub const LS_T1__L_TL: saberMoveName_t = 110;
pub const LS_T1__L_BL: saberMoveName_t = 111;
pub const LS_T1_BL_BR: saberMoveName_t = 112; //76
pub const LS_T1_BL__R: saberMoveName_t = 113;
pub const LS_T1_BL_TR: saberMoveName_t = 114;
pub const LS_T1_BL_T_: saberMoveName_t = 115;
pub const LS_T1_BL_TL: saberMoveName_t = 116;
pub const LS_T1_BL__L: saberMoveName_t = 117;

//Bounces
pub const LS_B1_BR: saberMoveName_t = 118;
pub const LS_B1__R: saberMoveName_t = 119;
pub const LS_B1_TR: saberMoveName_t = 120;
pub const LS_B1_T_: saberMoveName_t = 121;
pub const LS_B1_TL: saberMoveName_t = 122;
pub const LS_B1__L: saberMoveName_t = 123;
pub const LS_B1_BL: saberMoveName_t = 124;

//Deflected attacks
pub const LS_D1_BR: saberMoveName_t = 125;
pub const LS_D1__R: saberMoveName_t = 126;
pub const LS_D1_TR: saberMoveName_t = 127;
pub const LS_D1_T_: saberMoveName_t = 128;
pub const LS_D1_TL: saberMoveName_t = 129;
pub const LS_D1__L: saberMoveName_t = 130;
pub const LS_D1_BL: saberMoveName_t = 131;
pub const LS_D1_B_: saberMoveName_t = 132;

//Reflected attacks
pub const LS_V1_BR: saberMoveName_t = 133;
pub const LS_V1__R: saberMoveName_t = 134;
pub const LS_V1_TR: saberMoveName_t = 135;
pub const LS_V1_T_: saberMoveName_t = 136;
pub const LS_V1_TL: saberMoveName_t = 137;
pub const LS_V1__L: saberMoveName_t = 138;
pub const LS_V1_BL: saberMoveName_t = 139;
pub const LS_V1_B_: saberMoveName_t = 140;

// Broken parries
pub const LS_H1_T_: saberMoveName_t = 141; //
pub const LS_H1_TR: saberMoveName_t = 142;
pub const LS_H1_TL: saberMoveName_t = 143;
pub const LS_H1_BR: saberMoveName_t = 144;
pub const LS_H1_B_: saberMoveName_t = 145;
pub const LS_H1_BL: saberMoveName_t = 146;

// Knockaways
pub const LS_K1_T_: saberMoveName_t = 147; //
pub const LS_K1_TR: saberMoveName_t = 148;
pub const LS_K1_TL: saberMoveName_t = 149;
pub const LS_K1_BR: saberMoveName_t = 150;
pub const LS_K1_BL: saberMoveName_t = 151;

// Parries
pub const LS_PARRY_UP: saberMoveName_t = 152; //
pub const LS_PARRY_UR: saberMoveName_t = 153;
pub const LS_PARRY_UL: saberMoveName_t = 154;
pub const LS_PARRY_LR: saberMoveName_t = 155;
pub const LS_PARRY_LL: saberMoveName_t = 156;

// Projectile Reflections
pub const LS_REFLECT_UP: saberMoveName_t = 157; //
pub const LS_REFLECT_UR: saberMoveName_t = 158;
pub const LS_REFLECT_UL: saberMoveName_t = 159;
pub const LS_REFLECT_LR: saberMoveName_t = 160;
pub const LS_REFLECT_LL: saberMoveName_t = 161;

pub const LS_MOVE_MAX: saberMoveName_t = 162; //

// ---------------------------------------------------------------------------
// `saberQuadrant_t` (bg_public.h) -- typedef enum.
// ---------------------------------------------------------------------------
/// `saberQuadrant_t` (bg_public.h).
pub type saberQuadrant_t = c_int;

pub const Q_BR: saberQuadrant_t = 0;
pub const Q_R: saberQuadrant_t = 1;
pub const Q_TR: saberQuadrant_t = 2;
pub const Q_T: saberQuadrant_t = 3;
pub const Q_TL: saberQuadrant_t = 4;
pub const Q_L: saberQuadrant_t = 5;
pub const Q_BL: saberQuadrant_t = 6;
pub const Q_B: saberQuadrant_t = 7;
pub const Q_NUM_QUADS: saberQuadrant_t = 8;

// ===========================================================================
// Flag sets and remaining constant blocks (bg_public.h). Bit flags OR'd into
// `int` fields are `c_int`; array-sizing limits are `usize`; the HYPERSPACE_*
// fractions/speeds are `f32`. Composites are kept as the original expressions.
// ===========================================================================

/// `G2_MODEL_PART` — base offset for the `g2ModelParts_t` model-part ids.
pub const G2_MODEL_PART: c_int = 50;

/// `BG_NUM_TOGGLEABLE_SURFACES` — sizes the `bgToggleableSurfaces` tables.
pub const BG_NUM_TOGGLEABLE_SURFACES: usize = 31;

/// `MAX_CUSTOM_SIEGE_SOUNDS` — sizes `bg_customSiegeSoundNames`.
pub const MAX_CUSTOM_SIEGE_SOUNDS: usize = 30;

/// `TOSS_DEBOUNCE_TIME` — for supplier class items.
pub const TOSS_DEBOUNCE_TIME: c_int = 5000;

// Retail PC (raven-jediacademy): MAX_ANIM_FILES = 64. The Xbox (grayj) tree used
// 6 (with a commented `#define MAX_ANIM_FILES 16`, "I know that I had this number
// smaller once!"). Sizes the bgAllAnims/bgAllEvents statics (bg_panimate.rs),
// both zero-initialized, so the bump is layout-only; no oracle covers it.
pub const MAX_ANIM_FILES: usize = 64;
pub const MAX_ANIM_EVENTS: usize = 300;

// --- Anim eventData array size + index layout (bg_public.h) ---
/// size of the per-AEV random-sound set
pub const MAX_RANDOM_ANIM_SOUNDS: usize = 4;
pub const AED_ARRAY_SIZE: usize = MAX_RANDOM_ANIM_SOUNDS + 3;
// indices for AEV_SOUND data
pub const AED_SOUNDINDEX_START: usize = 0;
pub const AED_SOUNDINDEX_END: usize = MAX_RANDOM_ANIM_SOUNDS - 1;
pub const AED_SOUND_NUMRANDOMSNDS: usize = MAX_RANDOM_ANIM_SOUNDS;
pub const AED_SOUND_PROBABILITY: usize = MAX_RANDOM_ANIM_SOUNDS + 1;
// indices for AEV_SOUNDCHAN data
pub const AED_SOUNDCHANNEL: usize = MAX_RANDOM_ANIM_SOUNDS + 2;
// indices for AEV_FOOTSTEP data
pub const AED_FOOTSTEP_TYPE: usize = 0;
pub const AED_FOOTSTEP_PROBABILITY: usize = 1;
// indices for AEV_EFFECT data
pub const AED_EFFECTINDEX: usize = 0;
pub const AED_BOLTINDEX: usize = 1;
pub const AED_EFFECT_PROBABILITY: usize = 2;
pub const AED_MODELINDEX: usize = 3;
// indices for AEV_FIRE data
pub const AED_FIRE_ALT: usize = 0;
pub const AED_FIRE_PROBABILITY: usize = 1;
// indices for AEV_MOVE data
pub const AED_MOVE_FWD: usize = 0;
pub const AED_MOVE_RT: usize = 1;
pub const AED_MOVE_UP: usize = 2;
// indices for AEV_SABER_SWING data (retail PC only)
pub const AED_SABER_SWING_SABERNUM: usize = 0;
pub const AED_SABER_SWING_TYPE: usize = 1;
pub const AED_SABER_SWING_PROBABILITY: usize = 2;
// indices for AEV_SABER_SPIN data (retail PC only)
pub const AED_SABER_SPIN_SABERNUM: usize = 0;
pub const AED_SABER_SPIN_TYPE: usize = 1; // 0 = saberspinoff, 1 = saberspin, 2-4 = saberspin1-saberspin3
pub const AED_SABER_SPIN_PROBABILITY: usize = 2;

// --- pmove->pm_flags (bg_public.h) ---
pub const PMF_DUCKED: c_int = 1;
pub const PMF_JUMP_HELD: c_int = 2;
pub const PMF_ROLLING: c_int = 4;
pub const PMF_BACKWARDS_JUMP: c_int = 8; // go into backwards land
pub const PMF_BACKWARDS_RUN: c_int = 16; // coast down to backwards run
pub const PMF_TIME_LAND: c_int = 32; // pm_time is time before rejump
pub const PMF_TIME_KNOCKBACK: c_int = 64; // pm_time is an air-accelerate only time
pub const PMF_FIX_MINS: c_int = 128; // mins have been brought up, keep tracing down to fix them
pub const PMF_TIME_WATERJUMP: c_int = 256; // pm_time is waterjump
pub const PMF_RESPAWNED: c_int = 512; // clear after attack and jump buttons come up
pub const PMF_USE_ITEM_HELD: c_int = 1024;
pub const PMF_UPDATE_ANIM: c_int = 2048; // The server updated the animation, the pmove should set the ghoul2 anim to match.
pub const PMF_FOLLOW: c_int = 4096; // spectate following another player
pub const PMF_SCOREBOARD: c_int = 8192; // spectate as a scoreboard
pub const PMF_STUCK_TO_WALL: c_int = 16384; // grabbing a wall

pub const PMF_ALL_TIMES: c_int = PMF_TIME_WATERJUMP | PMF_TIME_LAND | PMF_TIME_KNOCKBACK;

/// `MAXTOUCH` — sizes `pmove_t::touchents`.
pub const MAXTOUCH: usize = 32;

// --- SetAnim flags (bg_public.h) ---
pub const SETANIM_TORSO: c_int = 1;
pub const SETANIM_LEGS: c_int = 2;
pub const SETANIM_BOTH: c_int = SETANIM_TORSO | SETANIM_LEGS; // 3

pub const SETANIM_FLAG_NORMAL: c_int = 0; // Only set if timer is 0
pub const SETANIM_FLAG_OVERRIDE: c_int = 1; // Override previous
pub const SETANIM_FLAG_HOLD: c_int = 2; // Set the new timer
pub const SETANIM_FLAG_RESTART: c_int = 4; // Allow restarting the anim if playing the same one (weapon fires)
pub const SETANIM_FLAG_HOLDLESS: c_int = 8; // Set the new timer

// --- entityState_t->eFlags (bg_public.h) ---
pub const EF_G2ANIMATING: c_int = 1 << 0; // perform g2 bone anims based on torsoAnim and legsAnim, works for ET_GENERAL -rww
pub const EF_DEAD: c_int = 1 << 1; // don't draw a foe marker over players with EF_DEAD
pub const EF_RADAROBJECT: c_int = 1 << 2; // display on team radar
pub const EF_TELEPORT_BIT: c_int = 1 << 3; // toggled every time the origin abruptly changes
pub const EF_SHADER_ANIM: c_int = 1 << 4; // Animating shader (by s.frame)
pub const EF_PLAYER_EVENT: c_int = 1 << 5;
pub const EF_RAG: c_int = 1 << 6; // ragdoll him even if he's alive
pub const EF_PERMANENT: c_int = 1 << 7; // rww - I am claiming this. (for permanent entities)
pub const EF_NODRAW: c_int = 1 << 8; // may have an event, but no model (unspawned items)
pub const EF_FIRING: c_int = 1 << 9; // for lightning gun
pub const EF_ALT_FIRING: c_int = 1 << 10; // for alt-fires, mostly for lightning guns though
pub const EF_JETPACK_ACTIVE: c_int = 1 << 11; // jetpack is activated
pub const EF_NOT_USED_1: c_int = 1 << 12; // not used
pub const EF_TALK: c_int = 1 << 13; // draw a talk balloon
pub const EF_CONNECTION: c_int = 1 << 14; // draw a connection trouble sprite
pub const EF_NOT_USED_6: c_int = 1 << 15; // not used
pub const EF_NOT_USED_2: c_int = 1 << 16; // not used
pub const EF_NOT_USED_3: c_int = 1 << 17; // not used
pub const EF_NOT_USED_4: c_int = 1 << 18; // not used
pub const EF_BODYPUSH: c_int = 1 << 19; // rww - claiming this for fullbody push effect
pub const EF_DOUBLE_AMMO: c_int = 1 << 20; // Hacky way to get around ammo max
pub const EF_SEEKERDRONE: c_int = 1 << 21; // show seeker drone floating around head
pub const EF_MISSILE_STICK: c_int = 1 << 22; // missiles that stick to the wall.
pub const EF_ITEMPLACEHOLDER: c_int = 1 << 23; // item effect
pub const EF_SOUNDTRACKER: c_int = 1 << 24; // sound position needs to be updated in relation to another entity
pub const EF_DROPPEDWEAPON: c_int = 1 << 25; // it's a dropped weapon
pub const EF_DISINTEGRATION: c_int = 1 << 26; // being disintegrated by the disruptor
pub const EF_INVULNERABLE: c_int = 1 << 27; // just spawned in or whatever, so is protected
pub const EF_CLIENTSMOOTH: c_int = 1 << 28; // standard lerporigin smooth override on client
pub const EF_JETPACK: c_int = 1 << 29; // rww - wearing a jetpack
pub const EF_JETPACK_FLAMING: c_int = 1 << 30; // rww - jetpack fire effect
pub const EF_NOT_USED_5: c_int = 1 << 31; // not used

// --- entityState_t->eFlags2 (bg_public.h) -- NPC flags; only 10 allowed ---
pub const EF2_HELD_BY_MONSTER: c_int = 1 << 0; // Being held by something, like a Rancor or a Wampa
pub const EF2_USE_ALT_ANIM: c_int = 1 << 1; // For certain special runs/stands for creatures like the Rancor and Wampa whose runs/stands are conditional
pub const EF2_ALERTED: c_int = 1 << 2; // For certain special anims, for Rancor: means you've had an enemy, so use the more alert stand
pub const EF2_GENERIC_NPC_FLAG: c_int = 1 << 3; // So far, used for Rancor...
pub const EF2_FLYING: c_int = 1 << 4; // Flying FIXME: only used on NPCs doesn't *really* have to be passed over, does it?
pub const EF2_HYPERSPACE: c_int = 1 << 5; // Used to both start the hyperspace effect on the predicted client and to let the vehicle know it can now jump into hyperspace (after turning to face the proper angle)
pub const EF2_BRACKET_ENTITY: c_int = 1 << 6; // Draw as bracketed
pub const EF2_SHIP_DEATH: c_int = 1 << 7; // "died in ship" mode
pub const EF2_NOT_USED_1: c_int = 1 << 8; // not used

// --- reward sounds (stored in ps->persistant[PERS_PLAYEREVENTS]) ---
pub const PLAYEREVENT_DENIEDREWARD: c_int = 0x0001;
pub const PLAYEREVENT_GAUNTLETREWARD: c_int = 0x0002;

// --- entityState_t->event uniqueness bits (bg_public.h) ---
// two bits at the top of the entityState->event field, incremented with each
// change so an identical event started twice in a row can be distinguished.
pub const EV_EVENT_BIT1: c_int = 0x00000100;
pub const EV_EVENT_BIT2: c_int = 0x00000200;
pub const EV_EVENT_BITS: c_int = EV_EVENT_BIT1 | EV_EVENT_BIT2;

pub const EVENT_VALID_MSEC: c_int = 300;

/// Time between location updates.
pub const TEAM_LOCATION_UPDATE_TIME: c_int = 1000;
/// How many players on the overlay.
pub const TEAM_MAXOVERLAY: c_int = 32;

/// `MAX_ITEM_MODELS` — sizes `gitem_t::world_model`.
pub const MAX_ITEM_MODELS: usize = 4;

/// number of milliseconds a block animation should take.
pub const SABER_BLOCK_DUR: c_int = 150;

// --- g_dmflags->integer flags (bg_public.h) ---
pub const DF_NO_FALLING: c_int = 8;
pub const DF_FIXED_FOV: c_int = 16;
pub const DF_NO_FOOTSTEPS: c_int = 32;

// (The `MASK_*` content masks are not yet ported -- they OR together the `CONTENTS_*`
// surface flags, which live in a not-yet-ported header. See `DEVIATIONS.md`.)

// --- ET_FX states (stored in modelindex2) (bg_public.h) ---
pub const FX_STATE_OFF: c_int = 0;
pub const FX_STATE_ONE_SHOT: c_int = 1;
pub const FX_STATE_ONE_SHOT_LIMIT: c_int = 10;
pub const FX_STATE_CONTINUOUS: c_int = 20;

// --- arenas / bots (bg_public.h) ---
pub const ARENAS_PER_TIER: c_int = 4;
pub const MAX_ARENAS: c_int = 1024;
pub const MAX_ARENAS_TEXT: c_int = 8192;

pub const MAX_BOTS: c_int = 1024;
pub const MAX_BOTS_TEXT: c_int = 8192;

// --- hyperspace (bg_public.h) ---
pub const HYPERSPACE_TIME: c_int = 4000; // For hyperspace triggers
pub const HYPERSPACE_TELEPORT_FRAC: f32 = 0.75;
pub const HYPERSPACE_SPEED: f32 = 10000.0; // was 30000
pub const HYPERSPACE_TURN_RATE: f32 = 45.0;

// Compile-time pins for the composite / computed values.
const _: () = assert!(AED_ARRAY_SIZE == 7);
const _: () = assert!(AED_SOUNDINDEX_END == 3);
const _: () = assert!(PMF_ALL_TIMES == 256 | 32 | 64);
const _: () = assert!(SETANIM_BOTH == 3);
const _: () = assert!(EV_EVENT_BITS == 0x300);
const _: () = assert!(EF_NOT_USED_5 == i32::MIN); // 1 << 31

// ===========================================================================
// Structs (bg_public.h). `animation_t` is `#pragma pack(1)` and pointer-free, so
// its layout is arch-independent (size 7). The rest carry raw pointers, so their
// size/alignment is arch-dependent -- ported faithfully with raw `*mut`, the
// 64-bit layout pinned by `#[cfg(target_pointer_width = "64")]` size asserts plus
// arch-independent interior `offset_of` asserts, and the host-64-bit C oracle
// (per the Stage-1 pointer-struct convention; see `DEVIATIONS.md`).
//
// `bgEntity_t` and `pmove_t` are now ported (below) -- `Vehicle_t` (bgEntity_t's
// `m_pVehicle` target) landed in `bg_vehicles_h`. The previously-unported items have
// since landed too: the inline `BG_GiveMeVectorFromMatrix` (once `mdxaBone_t` landed)
// and the `MASK_*` content masks (once `surfaceflags.h`'s `CONTENTS_*` landed).
// ===========================================================================

use core::mem::{offset_of, size_of};

/// `animation_t` (bg_public.h) — one row of the animation table. `#pragma pack(1)`
/// in C: no padding, size 7, align 1 (pointer-free, identical on 32/64-bit).
/// "initialLerp is abs(frameLerp)" — there is no separate initialLerp field.
/// (Packed, so no `Debug`/`PartialEq` derive — those would take references to
/// possibly-unaligned fields.)
#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct animation_t {
    pub firstFrame: u16,
    pub numFrames: u16,
    pub frameLerp: i16, // msec between frames
    pub loopFrames: i8, // 0 to numFrames
}
const _: () = assert!(size_of::<animation_t>() == 7);
const _: () = assert!(core::mem::align_of::<animation_t>() == 1);

/// `animevent_t` (bg_public.h) — a parsed animation event. Carries a raw `char *`
/// (`stringData`), so it is arch-dependent (64-bit layout asserted below).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct animevent_t {
    pub eventType: animEventType_t,
    pub keyFrame: u16,                    // Frame to play event on
    pub eventData: [i16; AED_ARRAY_SIZE], // Unique IDs: soundIndex / effect index / footstep type, etc.
    pub stringData: *mut c_char,          // one temporarily-stored string (NULLed after look-up)
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<animevent_t>() == 32);
const _: () = assert!(offset_of!(animevent_t, eventData) == 6);

/// `bgLoadedAnim_t` (bg_public.h) — a loaded animation set. Raw `animation_t *`
/// (arch-dependent). The commented-out `animsounds_t`/`soundsCached` members in
/// the C are omitted here too.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bgLoadedAnim_t {
    pub filename: [c_char; MAX_QPATH],
    pub anims: *mut animation_t,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<bgLoadedAnim_t>() == 72);
const _: () = assert!(offset_of!(bgLoadedAnim_t, anims) == MAX_QPATH); // 64

/// `bgLoadedEvents_t` (bg_public.h) — torso/legs anim events for a loaded set.
/// Embeds `animevent_t` (which carries a pointer), so arch-dependent. The C type
/// is defined unconditionally (only the `bgAllEvents` table extern is
/// `#ifndef QAGAME`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bgLoadedEvents_t {
    pub filename: [c_char; MAX_QPATH],
    pub torsoAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub legsAnimEvents: [animevent_t; MAX_ANIM_EVENTS],
    pub eventsParsed: qboolean,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<bgLoadedEvents_t>() == 19272);

/// `gitem_t` (bg_public.h) — an item definition (the `bg_itemlist` row type). Many
/// raw `char *` => arch-dependent. (The commented-out `pickup_name` member is
/// omitted as in C.)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gitem_t {
    pub classname: *mut c_char, // spawning name
    pub pickup_sound: *mut c_char,
    pub world_model: [*mut c_char; MAX_ITEM_MODELS],
    pub view_model: *mut c_char,
    pub icon: *mut c_char,
    pub quantity: c_int,    // for ammo how much, or duration of powerup
    pub giType: itemType_t, // IT_* flags
    pub giTag: c_int,
    pub precaches: *mut c_char, // string of all models and images this item will use
    pub sounds: *mut c_char,    // string of all sounds this item will use
    pub description: *mut c_char,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<gitem_t>() == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(gitem_t, quantity) == 64);

/// `saberMoveData_t` (bg_public.h) — one row of the `saberMoveData` FSM table.
/// Raw `char *name` => arch-dependent.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct saberMoveData_t {
    pub name: *mut c_char,
    pub animToUse: c_int,
    pub startQuad: c_int,
    pub endQuad: c_int,
    pub animSetFlags: c_uint,
    pub blendTime: c_int,
    pub blocking: c_int,
    pub chain_idle: saberMoveName_t, // move if attack not pressed at end of this anim
    pub chain_attack: saberMoveName_t, // move if attack (and nothing else) pressed
    pub trailLength: qboolean,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<saberMoveData_t>() == 48);

/// `BG_field_t` (bg_public.h) — a spawn-field descriptor (C: under `#ifdef
/// _GAME_SIDE`, always on for the game module; pairs with `fieldtype_t`). Raw
/// `char *name`. `type` is a Rust keyword, hence `r#type` (the C field is `type`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BG_field_t {
    pub name: *mut c_char,
    pub ofs: c_int,
    pub r#type: fieldtype_t,
    pub flags: c_int,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<BG_field_t>() == 24);

/// `bgEntity_s` / `bgEntity_t` (bg_public.h) — the both-games shared entity, whose
/// fields "must directly correspond to the head of the gentity and centity
/// structures". Pointer-bearing => arch-dependent layout. Now unblocked: its
/// `m_pVehicle` points at the real [`Vehicle_t`] from [`bg_vehicles_h`].
///
/// Cross-check: `size_of::<bgEntity_t>()` (568, the full shared prefix) equals
/// `offset_of!(gentity_t, r)` in [`g_local`] — `bgEntity_t` *is* that prefix.
///
/// [`bg_vehicles_h`]: crate::codemp::game::bg_vehicles_h
/// [`g_local`]: crate::codemp::game::g_local
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bgEntity_t {
    pub s: entityState_t,
    pub playerState: *mut playerState_t,
    pub m_pVehicle: *mut Vehicle_t, //vehicle data
    pub ghoul2: *mut c_void,        //g2 instance
    pub localAnimIndex: c_int,      //index locally (game/cgame) to anim data for this skel
    pub modelScale: vec3_t,         //needed for g2 collision

                                    //Data type(s) must directly correspond to the head of the gentity and centity structures
}
const _: () = assert!(offset_of!(bgEntity_t, s) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<bgEntity_t>() == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(bgEntity_t, playerState) == 536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(bgEntity_t, m_pVehicle) == 544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(bgEntity_t, modelScale) == 564);

/// `pmove_t` (bg_public.h) — the player-movement I/O struct passed to `Pmove`.
/// Pointer-bearing (and carries the `trace`/`pointcontents` C callbacks "different
/// functions during game and cgame"), so the layout is arch-dependent. Anonymous
/// `typedef struct { ... } pmove_t;` in C (no tag).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct pmove_t {
    // state (in / out)
    pub ps: *mut playerState_t,

    //rww - shared ghoul2 stuff (not actually the same data, but hey)
    pub ghoul2: *mut c_void,
    pub g2Bolts_LFoot: c_int,
    pub g2Bolts_RFoot: c_int,
    pub modelScale: vec3_t,

    //hacky bool so we know if we're dealing with a nonhumanoid (which is probably a rockettrooper)
    pub nonHumanoid: qboolean,

    // command (in)
    pub cmd: usercmd_t,
    pub tracemask: c_int,      // collide against these types of surfaces
    pub debugLevel: c_int,     // if set, diagnostic output will be printed
    pub noFootsteps: qboolean, // if the game is setup for no footsteps by the server
    pub gauntletHit: qboolean, // true if a gauntlet attack would actually hit something

    pub framecount: c_int,

    // results (out)
    pub numtouch: c_int,
    pub touchents: [c_int; MAXTOUCH],

    pub useEvent: c_int,

    pub mins: vec3_t, // bounding box size
    pub maxs: vec3_t,

    pub watertype: c_int,
    pub waterlevel: c_int,

    pub gametype: c_int,

    pub debugMelee: c_int,
    pub stepSlideFix: c_int,
    pub noSpecMove: c_int,

    pub animations: *mut animation_t,

    pub xyspeed: f32,

    // for fixed msec Pmove
    pub pmove_fixed: c_int,
    pub pmove_msec: c_int,

    // callbacks to test the world
    // these will be different functions during game and cgame
    pub trace: Option<
        unsafe extern "C" fn(
            results: *mut trace_t,
            start: *const vec_t,
            mins: *const vec_t,
            maxs: *const vec_t,
            end: *const vec_t,
            passEntityNum: c_int,
            contentMask: c_int,
        ),
    >,
    pub pointcontents:
        Option<unsafe extern "C" fn(point: *const vec_t, passEntityNum: c_int) -> c_int>,

    pub checkDuelLoss: c_int,

    //rww - bg entitystate access method
    pub baseEnt: *mut bgEntity_t, //base address of the entity array (g_entities or cg_entities)
    pub entSize: c_int, //size of the struct (gentity_t or centity_t) so things can be dynamic
}
const _: () = assert!(offset_of!(pmove_t, ps) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<pmove_t>() == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(pmove_t, cmd) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(pmove_t, mins) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(pmove_t, animations) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(pmove_t, trace) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(pmove_t, pointcontents) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(pmove_t, baseEnt) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(pmove_t, entSize) == 328);

// extern pmove_t *pm; -- extern global, lands with bg_pmove.c.

/// `BG_GiveMeVectorFromMatrix` (bg_public.h) — given a ghoul2 bolt matrix, return
/// in `vec` the normalised axis vector (or origin column) selected by `flags` (an
/// `Eorientations` value). The positive cases copy a column of the 3×4 transform;
/// the negative cases copy the negated column; `ORIGIN` returns the translation
/// (column 3). An unrecognised `flags` leaves `vec` untouched (C falls through the
/// `switch` with no `default`).
///
/// In the native build C declares this `static ID_INLINE` in the header; only the
/// `#ifdef __LCC__` VM build emits an out-of-line copy (an identical body) in
/// bg_misc.c (bg_misc.c:736). We port the one canonical body here.
#[inline]
pub fn BG_GiveMeVectorFromMatrix(boltMatrix: &mdxaBone_t, flags: c_int, vec: &mut vec3_t) {
    match flags {
        ORIGIN => {
            vec[0] = boltMatrix.matrix[0][3];
            vec[1] = boltMatrix.matrix[1][3];
            vec[2] = boltMatrix.matrix[2][3];
        }
        POSITIVE_Y => {
            vec[0] = boltMatrix.matrix[0][1];
            vec[1] = boltMatrix.matrix[1][1];
            vec[2] = boltMatrix.matrix[2][1];
        }
        POSITIVE_X => {
            vec[0] = boltMatrix.matrix[0][0];
            vec[1] = boltMatrix.matrix[1][0];
            vec[2] = boltMatrix.matrix[2][0];
        }
        POSITIVE_Z => {
            vec[0] = boltMatrix.matrix[0][2];
            vec[1] = boltMatrix.matrix[1][2];
            vec[2] = boltMatrix.matrix[2][2];
        }
        NEGATIVE_Y => {
            vec[0] = -boltMatrix.matrix[0][1];
            vec[1] = -boltMatrix.matrix[1][1];
            vec[2] = -boltMatrix.matrix[2][1];
        }
        NEGATIVE_X => {
            vec[0] = -boltMatrix.matrix[0][0];
            vec[1] = -boltMatrix.matrix[1][0];
            vec[2] = -boltMatrix.matrix[2][0];
        }
        NEGATIVE_Z => {
            vec[0] = -boltMatrix.matrix[0][2];
            vec[1] = -boltMatrix.matrix[1][2];
            vec[2] = -boltMatrix.matrix[2][2];
        }
        _ => {}
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::{offset_of, size_of};

    /// Parity: the computed `CS_*` config-string chain (which threads through the
    /// q_shared.h config-string limits) matches the authentic C arithmetic. The
    /// oracle reproduces the verbatim chain over the real limit values.
    #[test]
    fn cs_chain_matches_c() {
        unsafe {
            assert_eq!(CS_SIEGE_STATE, jka_bgp_CS_SIEGE_STATE());
            assert_eq!(CS_MODELS, jka_bgp_CS_MODELS());
            assert_eq!(CS_ICONS, jka_bgp_CS_ICONS());
            assert_eq!(CS_LIGHT_STYLES, jka_bgp_CS_LIGHT_STYLES());
            assert_eq!(CS_TERRAINS, jka_bgp_CS_TERRAINS());
            assert_eq!(CS_BSP_MODELS, jka_bgp_CS_BSP_MODELS());
            assert_eq!(CS_MAX, jka_bgp_CS_MAX());
        }
    }

    /// Parity: `BG_GiveMeVectorFromMatrix` extracts the same (signed) column of a
    /// bolt matrix as the authentic C inline, bit-for-bit, for every
    /// `Eorientations` selector plus an out-of-range value (no-op fall-through).
    #[test]
    fn give_me_vector_from_matrix_matches_c() {
        // A bolt matrix with distinct, sign-varied entries so a wrong column or a
        // missed negation can't alias to the right answer.
        let bone = mdxaBone_t {
            matrix: [
                [1.5, -2.25, 3.75, -4.0],
                [5.5, 6.0, -7.5, 8.25],
                [-9.0, 10.5, 11.25, -12.0],
            ],
        };
        let m12: [f32; 12] = [
            1.5, -2.25, 3.75, -4.0, 5.5, 6.0, -7.5, 8.25, -9.0, 10.5, 11.25, -12.0,
        ];
        for flags in [
            ORIGIN, POSITIVE_X, POSITIVE_Y, POSITIVE_Z, NEGATIVE_X, NEGATIVE_Y, NEGATIVE_Z, 99,
        ] {
            // Pre-seed with a sentinel so the out-of-range no-op is observable.
            let mut rust: vec3_t = [-111.0, -222.0, -333.0];
            let mut c: vec3_t = [-111.0, -222.0, -333.0];
            BG_GiveMeVectorFromMatrix(&bone, flags, &mut rust);
            unsafe { jka_bgp_GiveMeVectorFromMatrix(m12.as_ptr(), flags, c.as_mut_ptr()) };
            assert_eq!(rust.map(f32::to_bits), c.map(f32::to_bits), "flags={flags}");
        }
    }

    /// Parity: every shared enum's checkpoint values (first / terminal /
    /// each explicit `= N` / interior spread) match the authentic C enums,
    /// whose bodies the oracle copies verbatim and lets the C compiler number.
    #[test]
    fn shared_enum_values_match_c() {
        unsafe {
            assert_eq!(
                G2_MODELPART_HEAD,
                jka_bge_G2_MODELPART_HEAD(),
                "G2_MODELPART_HEAD"
            );
            assert_eq!(
                G2_MODELPART_RLEG,
                jka_bge_G2_MODELPART_RLEG(),
                "G2_MODELPART_RLEG"
            );
            assert_eq!(
                HANDEXTEND_NONE,
                jka_bge_HANDEXTEND_NONE(),
                "HANDEXTEND_NONE"
            );
            assert_eq!(
                HANDEXTEND_JEDITAUNT,
                jka_bge_HANDEXTEND_JEDITAUNT(),
                "HANDEXTEND_JEDITAUNT"
            );
            assert_eq!(
                BROKENLIMB_NONE,
                jka_bge_BROKENLIMB_NONE(),
                "BROKENLIMB_NONE"
            );
            assert_eq!(
                NUM_BROKENLIMBS,
                jka_bge_NUM_BROKENLIMBS(),
                "NUM_BROKENLIMBS"
            );
            assert_eq!(GT_FFA, jka_bge_GT_FFA(), "GT_FFA");
            assert_eq!(
                GT_MAX_GAME_TYPE,
                jka_bge_GT_MAX_GAME_TYPE(),
                "GT_MAX_GAME_TYPE"
            );
            assert_eq!(GENDER_MALE, jka_bge_GENDER_MALE(), "GENDER_MALE");
            assert_eq!(GENDER_NEUTER, jka_bge_GENDER_NEUTER(), "GENDER_NEUTER");
            assert_eq!(SABERLOCK_TOP, jka_bge_SABERLOCK_TOP(), "SABERLOCK_TOP");
            assert_eq!(SABERLOCK_LOSE, jka_bge_SABERLOCK_LOSE(), "SABERLOCK_LOSE");
            assert_eq!(DIR_RIGHT, jka_bge_DIR_RIGHT(), "DIR_RIGHT");
            assert_eq!(DIR_BACK, jka_bge_DIR_BACK(), "DIR_BACK");
            assert_eq!(FOOTSTEP_R, jka_bge_FOOTSTEP_R(), "FOOTSTEP_R");
            assert_eq!(
                NUM_FOOTSTEP_TYPES,
                jka_bge_NUM_FOOTSTEP_TYPES(),
                "NUM_FOOTSTEP_TYPES"
            );
            assert_eq!(AEV_NONE, jka_bge_AEV_NONE(), "AEV_NONE");
            assert_eq!(AEV_NUM_AEV, jka_bge_AEV_NUM_AEV(), "AEV_NUM_AEV");
            assert_eq!(PM_NORMAL, jka_bge_PM_NORMAL(), "PM_NORMAL");
            assert_eq!(
                PM_SPINTERMISSION,
                jka_bge_PM_SPINTERMISSION(),
                "PM_SPINTERMISSION"
            );
            assert_eq!(WEAPON_READY, jka_bge_WEAPON_READY(), "WEAPON_READY");
            assert_eq!(WEAPON_IDLE, jka_bge_WEAPON_IDLE(), "WEAPON_IDLE");
            assert_eq!(
                FORCE_MASTERY_UNINITIATED,
                jka_bge_FORCE_MASTERY_UNINITIATED(),
                "FORCE_MASTERY_UNINITIATED"
            );
            assert_eq!(
                NUM_FORCE_MASTERY_LEVELS,
                jka_bge_NUM_FORCE_MASTERY_LEVELS(),
                "NUM_FORCE_MASTERY_LEVELS"
            );
            assert_eq!(STAT_HEALTH, jka_bge_STAT_HEALTH(), "STAT_HEALTH");
            assert_eq!(STAT_WEAPONS, jka_bge_STAT_WEAPONS(), "STAT_WEAPONS");
            assert_eq!(
                STAT_MAX_HEALTH,
                jka_bge_STAT_MAX_HEALTH(),
                "STAT_MAX_HEALTH"
            );
            assert_eq!(PERS_SCORE, jka_bge_PERS_SCORE(), "PERS_SCORE");
            assert_eq!(PERS_CAPTURES, jka_bge_PERS_CAPTURES(), "PERS_CAPTURES");
            assert_eq!(EFFECT_NONE, jka_bge_EFFECT_NONE(), "EFFECT_NONE");
            assert_eq!(EFFECT_MAX, jka_bge_EFFECT_MAX(), "EFFECT_MAX");
            assert_eq!(PW_NONE, jka_bge_PW_NONE(), "PW_NONE");
            assert_eq!(
                PW_NUM_POWERUPS,
                jka_bge_PW_NUM_POWERUPS(),
                "PW_NUM_POWERUPS"
            );
            assert_eq!(HI_NONE, jka_bge_HI_NONE(), "HI_NONE");
            assert_eq!(
                HI_NUM_HOLDABLE,
                jka_bge_HI_NUM_HOLDABLE(),
                "HI_NUM_HOLDABLE"
            );
            assert_eq!(
                CTFMESSAGE_FRAGGED_FLAG_CARRIER,
                jka_bge_CTFMESSAGE_FRAGGED_FLAG_CARRIER(),
                "CTFMESSAGE_FRAGGED_FLAG_CARRIER"
            );
            assert_eq!(
                CTFMESSAGE_PLAYER_GOT_FLAG,
                jka_bge_CTFMESSAGE_PLAYER_GOT_FLAG(),
                "CTFMESSAGE_PLAYER_GOT_FLAG"
            );
            assert_eq!(PDSOUND_NONE, jka_bge_PDSOUND_NONE(), "PDSOUND_NONE");
            assert_eq!(
                PDSOUND_FORCEGRIP,
                jka_bge_PDSOUND_FORCEGRIP(),
                "PDSOUND_FORCEGRIP"
            );
            assert_eq!(EV_NONE, jka_bge_EV_NONE(), "EV_NONE");
            assert_eq!(EV_USE_ITEM5, jka_bge_EV_USE_ITEM5(), "EV_USE_ITEM5");
            assert_eq!(EV_BODYFADE, jka_bge_EV_BODYFADE(), "EV_BODYFADE");
            assert_eq!(EV_ESCAPING2, jka_bge_EV_ESCAPING2(), "EV_ESCAPING2");
            assert_eq!(EV_SIEGESPEC, jka_bge_EV_SIEGESPEC(), "EV_SIEGESPEC");
            assert_eq!(
                GTS_RED_CAPTURE,
                jka_bge_GTS_RED_CAPTURE(),
                "GTS_RED_CAPTURE"
            );
            assert_eq!(
                GTS_TEAMS_ARE_TIED,
                jka_bge_GTS_TEAMS_ARE_TIED(),
                "GTS_TEAMS_ARE_TIED"
            );
            assert_eq!(TEAM_FREE, jka_bge_TEAM_FREE(), "TEAM_FREE");
            assert_eq!(TEAM_NUM_TEAMS, jka_bge_TEAM_NUM_TEAMS(), "TEAM_NUM_TEAMS");
            assert_eq!(DUELTEAM_FREE, jka_bge_DUELTEAM_FREE(), "DUELTEAM_FREE");
            assert_eq!(
                DUELTEAM_SINGLE,
                jka_bge_DUELTEAM_SINGLE(),
                "DUELTEAM_SINGLE"
            );
            assert_eq!(TEAMTASK_NONE, jka_bge_TEAMTASK_NONE(), "TEAMTASK_NONE");
            assert_eq!(TEAMTASK_CAMP, jka_bge_TEAMTASK_CAMP(), "TEAMTASK_CAMP");
            assert_eq!(MOD_UNKNOWN, jka_bge_MOD_UNKNOWN(), "MOD_UNKNOWN");
            assert_eq!(MOD_MAX, jka_bge_MOD_MAX(), "MOD_MAX");
            assert_eq!(IT_BAD, jka_bge_IT_BAD(), "IT_BAD");
            assert_eq!(IT_TEAM, jka_bge_IT_TEAM(), "IT_TEAM");
            assert_eq!(ET_GENERAL, jka_bge_ET_GENERAL(), "ET_GENERAL");
            assert_eq!(ET_EVENTS, jka_bge_ET_EVENTS(), "ET_EVENTS");
            assert_eq!(F_INT, jka_bge_F_INT(), "F_INT");
            assert_eq!(F_IGNORE, jka_bge_F_IGNORE(), "F_IGNORE");
            assert_eq!(LS_NONE, jka_bge_LS_NONE(), "LS_NONE");
            assert_eq!(
                LS_DUAL_SPIN_PROTECT,
                jka_bge_LS_DUAL_SPIN_PROTECT(),
                "LS_DUAL_SPIN_PROTECT"
            );
            assert_eq!(LS_T1_TL_BR, jka_bge_LS_T1_TL_BR(), "LS_T1_TL_BR");
            assert_eq!(LS_K1_BR, jka_bge_LS_K1_BR(), "LS_K1_BR");
            assert_eq!(LS_MOVE_MAX, jka_bge_LS_MOVE_MAX(), "LS_MOVE_MAX");
            assert_eq!(Q_BR, jka_bge_Q_BR(), "Q_BR");
            assert_eq!(Q_NUM_QUADS, jka_bge_Q_NUM_QUADS(), "Q_NUM_QUADS");
        }
    }

    /// Parity: the bg_public.h struct layouts match the authentic C `sizeof` /
    /// `offsetof`. `animation_t` is packed + pointer-free (arch-independent); the
    /// rest carry pointers, so the oracle (built host-64-bit) validates the 64-bit
    /// layout (matching the `#[cfg(target_pointer_width = "64")]` asserts above).
    #[test]
    fn struct_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<animation_t>(), jka_bgs_sizeof_animation_t());
            assert_eq!(
                offset_of!(animation_t, loopFrames),
                jka_bgs_off_animation_loopFrames()
            );

            assert_eq!(size_of::<animevent_t>(), jka_bgs_sizeof_animevent_t());
            assert_eq!(
                offset_of!(animevent_t, eventData),
                jka_bgs_off_animevent_eventData()
            );
            assert_eq!(
                offset_of!(animevent_t, stringData),
                jka_bgs_off_animevent_stringData()
            );

            assert_eq!(size_of::<bgLoadedAnim_t>(), jka_bgs_sizeof_bgLoadedAnim_t());
            assert_eq!(
                offset_of!(bgLoadedAnim_t, anims),
                jka_bgs_off_bgLoadedAnim_anims()
            );

            assert_eq!(
                size_of::<bgLoadedEvents_t>(),
                jka_bgs_sizeof_bgLoadedEvents_t()
            );
            assert_eq!(
                offset_of!(bgLoadedEvents_t, legsAnimEvents),
                jka_bgs_off_bgLoadedEvents_legsAnimEvents()
            );
            assert_eq!(
                offset_of!(bgLoadedEvents_t, eventsParsed),
                jka_bgs_off_bgLoadedEvents_eventsParsed()
            );

            assert_eq!(size_of::<gitem_t>(), jka_bgs_sizeof_gitem_t());
            assert_eq!(
                offset_of!(gitem_t, world_model),
                jka_bgs_off_gitem_world_model()
            );
            assert_eq!(offset_of!(gitem_t, quantity), jka_bgs_off_gitem_quantity());
            assert_eq!(
                offset_of!(gitem_t, precaches),
                jka_bgs_off_gitem_precaches()
            );

            assert_eq!(
                size_of::<saberMoveData_t>(),
                jka_bgs_sizeof_saberMoveData_t()
            );
            assert_eq!(
                offset_of!(saberMoveData_t, chain_idle),
                jka_bgs_off_saberMoveData_chain_idle()
            );
            assert_eq!(
                offset_of!(saberMoveData_t, trailLength),
                jka_bgs_off_saberMoveData_trailLength()
            );

            assert_eq!(size_of::<BG_field_t>(), jka_bgs_sizeof_BG_field_t());
            assert_eq!(offset_of!(BG_field_t, r#type), jka_bgs_off_BG_field_type());

            // bgEntity_t (the both-games shared entity head). Its oracle lives in the
            // bg_vehicles TU (`jka_bv_*`) because it shares the Vehicle_t cluster's
            // by-value deps. Cross-check: size 568 == offset_of!(gentity_t, r).
            assert_eq!(size_of::<bgEntity_t>(), jka_bv_sizeof_bgEntity_t());
            assert_eq!(
                offset_of!(bgEntity_t, playerState),
                jka_bv_off_be_playerState()
            );
            assert_eq!(
                offset_of!(bgEntity_t, m_pVehicle),
                jka_bv_off_be_m_pVehicle()
            );
            assert_eq!(
                offset_of!(bgEntity_t, modelScale),
                jka_bv_off_be_modelScale()
            );

            // pmove_t (the Pmove I/O struct). Same TU rationale as bgEntity_t.
            assert_eq!(size_of::<pmove_t>(), jka_bv_sizeof_pmove_t());
            assert_eq!(offset_of!(pmove_t, cmd), jka_bv_off_pm_cmd());
            assert_eq!(offset_of!(pmove_t, mins), jka_bv_off_pm_mins());
            assert_eq!(offset_of!(pmove_t, animations), jka_bv_off_pm_animations());
            assert_eq!(offset_of!(pmove_t, trace), jka_bv_off_pm_trace());
            assert_eq!(
                offset_of!(pmove_t, pointcontents),
                jka_bv_off_pm_pointcontents()
            );
            assert_eq!(offset_of!(pmove_t, baseEnt), jka_bv_off_pm_baseEnt());
            assert_eq!(offset_of!(pmove_t, entSize), jka_bv_off_pm_entSize());
        }
    }
}
