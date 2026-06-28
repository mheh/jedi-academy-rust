//! Port of `ai_main.c` — the server-side bot intelligence (`bot_state_t` think loop,
//! waypoint nav, combat decisions). It funnels into the NAV/bot trap layer + `bot_state_t`;
//! the trap wrappers are in place, so the bot AI is **in scope** to port. The
//! self-contained leaf helpers that already-ported callers reach in for — pure visibility
//! and field-of-vision checks — landed first; the `bot_state_t` brain lands across
//! subsequent passes. Port functions whose callees are all already ported.

use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut, write_bytes};

use crate::codemp::game::ai_main_h::{
    bot_state_t, boteventtracker_t, BASE_FLAGWAIT_DISTANCE, BASE_GETENEMYFLAG_DISTANCE,
    BASE_GUARD_DISTANCE, BOT_FLAG_GET_DISTANCE, BOT_MAX_WEAPON_CHASE_CTF, BOT_MAX_WEAPON_GATHER_TIME,
    BOT_MIN_SIEGE_GOAL_SHOOT, BOT_MIN_SIEGE_GOAL_TRAVEL,
    BOT_PLANT_BLOW_DISTANCE, BOT_RUN_HEALTH,
    BWEAPONRANGE_LONG, BWEAPONRANGE_MELEE,
    BWEAPONRANGE_MID, BWEAPONRANGE_SABER, CTFSTATE_ATTACKER, CTFSTATE_DEFENDER,
    CTFSTATE_GETFLAGHOME, CTFSTATE_GUARDCARRIER, CTFSTATE_MAXCTFSTATES, CTFSTATE_NONE,
    CTFSTATE_RETRIEVAL, BOT_MAX_WEAPON_CHASE_TIME, SIEGESTATE_ATTACKER, SIEGESTATE_DEFENDER,
    SIEGESTATE_MAXSIEGESTATES, SIEGESTATE_NONE, TEAMPLAYSTATE_MAXTPSTATES,
    WPFLAG_SIEGE_IMPERIALOBJ, WPFLAG_SIEGE_REBELOBJ,
    TEAMPLAYSTATE_NONE, ENEMY_FORGET_MS,
    LEVELFLAG_IGNOREINFALLBACK, LEVELFLAG_IMUSTNTRUNAWAY, MAX_CHICKENWUSS_TIME, MELEE_ATTACK_RANGE,
    SABER_ATTACK_RANGE, TEAMPLAYSTATE_ASSISTING, TEAMPLAYSTATE_FOLLOWING, TEAMPLAYSTATE_REGROUP,
    WPFLAG_BLUE_FLAG, WPFLAG_DUCK, WPFLAG_GOALPOINT, WPFLAG_JUMP, WPFLAG_NOMOVEFUNC,
    WPFLAG_ONEWAY_BACK,
    WPFLAG_ONEWAY_FWD, WPFLAG_RED_FLAG, WPFLAG_SNIPEORCAMP, WPFLAG_SNIPEORCAMPSTAND, WP_KEEP_FLAG_DIST,
    BOT_PLANT_DISTANCE, BOT_PLANT_INTERVAL, BOT_SABER_THROW_RANGE, BOT_WPTOUCH_DISTANCE,
    LEVELFLAG_NOPOINTPREDICTION, WPFLAG_NOVIS, WPFLAG_WAITFORFUNC,
};
use crate::codemp::game::ai_wpnav::{gLevelFlags, gWPArray, gWPNum};
use crate::codemp::game::ai_util::{B_Alloc, BotDoChat, BotUtilizePersonality};
use crate::codemp::game::botlib_h::{
    bot_input_t, ACTION_ALT_ATTACK, ACTION_ATTACK, ACTION_CROUCH, ACTION_DELAYEDJUMP,
    ACTION_FORCEPOWER, ACTION_GESTURE, ACTION_JUMP, ACTION_MOVEBACK, ACTION_MOVEFORWARD,
    ACTION_MOVELEFT, ACTION_MOVERIGHT, ACTION_RESPAWN, ACTION_USE, ACTION_WALK, PRT_FATAL,
};
use crate::codemp::game::bg_panimate::{BG_SaberInKata, BG_SaberInSpecial};
use crate::codemp::game::bg_public::{LS_SPINATTACK, LS_SPINATTACK_DUAL};
use crate::codemp::game::bg_pmove::forceJumpStrength;
use crate::codemp::game::be_aas_h::aas_entityinfo_t;
use crate::codemp::game::bg_public::{
    ET_SPECIAL, EV_ALT_FIRE, EV_FIRE_WEAPON, EV_FOOTSTEP, EV_FOOTSTEP_METAL, EV_FOOTWADE,
    EV_GLOBAL_SOUND, EV_JUMP, EV_ROLL, EV_SABER_ATTACK, EV_STEP_12, EV_STEP_16, EV_STEP_4,
    EV_STEP_8, GT_CTF, GT_CTY, GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_SINGLE_PLAYER,
    GT_TEAM,
    HI_MEDPAC, HI_MEDPAC_BIG, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD, IT_AMMO, IT_HOLDABLE, IT_POWERUP,
    IT_WEAPON, MASK_PLAYERSOLID, MASK_SOLID, PM_INTERMISSION, PM_SPECTATOR, PW_BLUEFLAG, PW_REDFLAG,
    STAT_HOLDABLE_ITEM, STAT_HOLDABLE_ITEMS, STAT_WEAPONS, TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR,
    WEAPON_CHARGING, WEAPON_CHARGING_ALT, DEFAULT_MAXS_2, ET_NPC, PMF_JUMP_HELD, WEAPON_READY,
};
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DEMP2, WP_DET_PACK, WP_DISRUPTOR, WP_FLECHETTE,
    WP_MELEE, WP_NONE, WP_NUM_WEAPONS, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON,
    WP_THERMAL, WP_TRIP_MINE,
};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::g_local::{
    bot_settings_t, gclient_t, gentity_t, CON_CONNECTED, CON_DISCONNECTED, FL_DROPPED_ITEM,
};
use crate::codemp::game::g_main::{
    g_entities, g_forcePowerDisable, g_friendlyFire, g_gametype, g_privateDuel, g_RMG, level,
};
use crate::codemp::game::g_cmds::{Cmd_EngageDuel_f, Cmd_SaberAttackCycle_f, Cmd_ToggleSaber_f};
use crate::codemp::game::w_force::ForcePowerUsableOn;
use crate::codemp::game::bg_pmove::forcePowerNeeded;
use crate::codemp::game::w_saber_h::{
    FORCE_LIGHTNING_RADIUS, MAX_DRAIN_DISTANCE, MAX_GRIP_DISTANCE, MAX_TRICK_DISTANCE,
};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::codemp::game::ai_wpnav::gDeactivated;
use crate::codemp::game::ai_wpnav::{gBotEdit, BotWaypointRender};
use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::g_bot::G_CheckBotSpawn;
use crate::codemp::game::g_combat::G_ThereIsAMaster;
use crate::codemp::game::g_public_h::SVF_NOCLIENT;
use crate::codemp::game::bg_misc::BG_GetItemIndexByTag;
use crate::codemp::game::bg_saga_h::SIEGETEAM_TEAM1;
use crate::codemp::game::g_client::gJMSaberEnt;
use crate::codemp::game::g_saga::{imperial_attackers, rebel_attackers};
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::g_utils::G_Find;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleMod, AngleVectors, DotProduct, VectorCopy, VectorLength,
    VectorNormalize, VectorSubtract,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    entityState_t, forcedata_t, playerState_t, usercmd_t, vec3_t, wpobject_t, Q_IsColorString,
    ANGLE2SHORT, BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_FORCEPOWER, BUTTON_GESTURE, BUTTON_USE,
    CA_ACTIVE, CA_AUTHORIZING,
    BUTTON_USE_HOLDABLE, BUTTON_WALKING, CVAR_CHEAT, ENTITYNUM_NONE, FP_LEVITATION, FP_RAGE,
    MAX_CLIENTS, MAX_PS_EVENTS, PITCH, ROLL, SHORT2ANGLE, YAW,
    FP_ABSORB, FP_DRAIN, FP_GRIP, FP_HEAL, FP_LIGHTNING, FP_PROTECT, FP_PULL, FP_PUSH, FP_SABERTHROW,
    FP_SABER_OFFENSE, FP_SEE, FP_SPEED, FP_TEAM_FORCE, FP_TEAM_HEAL, FP_TELEPATHY,
    FORCE_DARKSIDE, FORCE_LEVEL_1, FORCE_LIGHTSIDE, MAX_GENTITIES, SS_DUAL, SS_FAST, SS_MEDIUM,
    SS_STAFF, SS_STRONG,
};
use crate::ffi::types::{qboolean, vmCvar_t, QFALSE, QTRUE};
use crate::trap;

extern "C" {
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
}

// ===========================================================================
// File-scope globals defined in ai_main.c. The waypoint-nav globals (gWPArray,
// gWPNum, nodetable, nodenum, gWPRenderTime, gDeactivated, gBotEdit,
// gWPRenderedFrame, gLastPrintedIndex, gLevelFlags) are declared `extern` in
// ai_main.h but *defined* in ai_wpnav.c, so they land with that file — not here.
// ===========================================================================

/// `bot_state_t *botstates[MAX_CLIENTS]` (ai_main.c:46) — per-client bot state slots.
#[allow(non_upper_case_globals)]
pub static mut botstates: [*mut bot_state_t; MAX_CLIENTS] = [null_mut(); MAX_CLIENTS];

/// `int numbots` (ai_main.c:48) — number of bots currently in the game.
#[allow(non_upper_case_globals)]
pub static mut numbots: c_int = 0;

/// `float floattime` (ai_main.c:50) — the current bot-library floating-point time.
#[allow(non_upper_case_globals)]
pub static mut floattime: f32 = 0.0;

/// `float regularupdate_time` (ai_main.c:52) — time to do a regular update.
#[allow(non_upper_case_globals)]
pub static mut regularupdate_time: f32 = 0.0;

/// `int gUpdateVars = 0;` (ai_main.c:7478) — debounce gate for the per-frame bot-cvar
/// `trap_Cvar_Update` batch in [`BotAIStartFrame`] (refreshed at most once per second).
#[allow(non_upper_case_globals)]
static mut gUpdateVars: c_int = 0;

/// `boteventtracker_t gBotEventTracker[MAX_CLIENTS]` (ai_main.c:60) — per-client
/// event sequence tracking for the bot AI.
#[allow(non_upper_case_globals)]
pub static mut gBotEventTracker: [boteventtracker_t; MAX_CLIENTS] = [boteventtracker_t {
    eventSequence: 0,
    events: [0; crate::codemp::game::q_shared_h::MAX_PS_EVENTS],
    eventTime: 0.0,
}; MAX_CLIENTS];

// CTF flag waypoint objects (ai_main.c:86-89).
/// `wpobject_t *flagRed` (ai_main.c:86)
#[allow(non_upper_case_globals)]
pub static mut flagRed: *mut wpobject_t = null_mut();
/// `wpobject_t *oFlagRed` (ai_main.c:87)
#[allow(non_upper_case_globals)]
pub static mut oFlagRed: *mut wpobject_t = null_mut();
/// `wpobject_t *flagBlue` (ai_main.c:88)
#[allow(non_upper_case_globals)]
pub static mut flagBlue: *mut wpobject_t = null_mut();
/// `wpobject_t *oFlagBlue` (ai_main.c:89)
#[allow(non_upper_case_globals)]
pub static mut oFlagBlue: *mut wpobject_t = null_mut();

// CTF flag entities (ai_main.c:91-94).
/// `gentity_t *eFlagRed` (ai_main.c:91)
#[allow(non_upper_case_globals)]
pub static mut eFlagRed: *mut gentity_t = null_mut();
/// `gentity_t *droppedRedFlag` (ai_main.c:92)
#[allow(non_upper_case_globals)]
pub static mut droppedRedFlag: *mut gentity_t = null_mut();
/// `gentity_t *eFlagBlue` (ai_main.c:93)
#[allow(non_upper_case_globals)]
pub static mut eFlagBlue: *mut gentity_t = null_mut();
/// `gentity_t *droppedBlueFlag` (ai_main.c:94)
#[allow(non_upper_case_globals)]
pub static mut droppedBlueFlag: *mut gentity_t = null_mut();

// Team-order status descriptions (ai_main.c:106-126), indexed by the bot's ctf/siege/teamplay
// state and spoken to the team by `BotReportStatus`. Modelled as `&str` arrays because the only
// consumer (`EA_SayTeam`) takes `&str` and re-`cstr`s it; the C `ctfStateNames[]` (ai_main.c:96)
// is referenced nowhere in the module and is omitted.
/// `char *ctfStateDescriptions[]` (ai_main.c:106)
#[allow(non_upper_case_globals)]
pub static ctfStateDescriptions: [&str; 6] = [
    "I'm not occupied",
    "I'm attacking the enemy's base",
    "I'm defending our base",
    "I'm getting our flag back",
    "I'm escorting our flag carrier",
    "I've got the enemy's flag",
];
/// `char *siegeStateDescriptions[]` (ai_main.c:115)
#[allow(non_upper_case_globals)]
pub static siegeStateDescriptions: [&str; 3] = [
    "I'm not occupied",
    "I'm attemtping to complete the current objective",
    "I'm preventing the enemy from completing their objective",
];
/// `char *teamplayStateDescriptions[]` (ai_main.c:121)
#[allow(non_upper_case_globals)]
pub static teamplayStateDescriptions: [&str; 4] = [
    "I'm not occupied",
    "I'm following my squad commander",
    "I'm assisting my commanding",
    "I'm attempting to regroup and form a new squad",
];

// rww - new bot cvars (ai_main.c:62-83). The `#ifdef _DEBUG` ones (`bot_nogoals`,
// `bot_debugmessages`) are omitted: `_DEBUG` is not defined in the release server build.
// `bot_getinthecarrr` is under `#ifndef FINAL_BUILD`, which IS compiled in the dev build,
// so it is kept.
/// `vmCvar_t bot_forcepowers` (ai_main.c:62)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_forcepowers: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_forgimmick` (ai_main.c:63)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_forgimmick: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_honorableduelacceptance` (ai_main.c:64)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_honorableduelacceptance: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_pvstype` (ai_main.c:65)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_pvstype: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_normgpath` (ai_main.c:66)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_normgpath: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_getinthecarrr` (ai_main.c:68, `#ifndef FINAL_BUILD`)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_getinthecarrr: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_attachments` (ai_main.c:76)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_attachments: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_camp` (ai_main.c:77)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_camp: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_wp_info` (ai_main.c:79)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_wp_info: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_wp_edit` (ai_main.c:80)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_wp_edit: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_wp_clearweight` (ai_main.c:81)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_wp_clearweight: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_wp_distconnect` (ai_main.c:82)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_wp_distconnect: vmCvar_t = vmCvar_t::zeroed();
/// `vmCvar_t bot_wp_visconnect` (ai_main.c:83)
#[allow(non_upper_case_globals, dead_code)]
pub(crate) static mut bot_wp_visconnect: vmCvar_t = vmCvar_t::zeroed();
// end rww

/// `#define BOT_THINK_TIME 0` (ai_main.c:43) — bot think interval (disabled at 0 in JKA).
pub const BOT_THINK_TIME: c_int = 0;

/// `#define FloatTime() floattime` (ai_main.h:411) — the bot-library current time.
///
/// # Safety
/// Reads the `floattime` mutable global.
#[inline]
#[allow(non_snake_case)]
pub unsafe fn FloatTime() -> f32 {
    floattime
}

//rww - bot ai

/// `void QDECL BotAI_Print(int type, char *fmt, ...)` (ai_main.c:336) — the C body is the
/// single statement `{ return; }`: a faithful no-op debug printer that computes nothing and
/// ignores all of its arguments (the original BotLib trace/print hook left disabled in JKA).
///
/// Ported as a fixed-arg no-op: stable Rust cannot express the C variadic `...` tail, and the
/// body discards it anyway (see the `bg_saberLoad` SyscallFn note). `type`/`fmt` are taken to
/// preserve the named-parameter ABI; the variadic tail is dropped because the function never
/// reads it.
#[allow(unused_variables)]
pub fn BotAI_Print(r#type: c_int, fmt: *mut c_char) {
    // { return; }
}

/// `int BotMindTricked(int botClient, int enemyClient)` (ai_main.c:283) — has `enemyClient`
/// mind-tricked `botClient` out of view? Reads the enemy's force data and tests the
/// 64-bit-wide mind-trick target bitmask (split across four `forceMindtrickTargetIndex*`
/// ints, 16 clients each) for `botClient`'s bit. Returns `0` if the enemy slot is not a
/// client.
///
/// # Safety
/// `enemyClient` must index a valid [`g_entities`] slot.
pub unsafe fn BotMindTricked(botClient: c_int, enemyClient: c_int) -> c_int {
    let fd: *mut forcedata_t;

    if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(enemyClient as usize))
        .client
        .is_null()
    {
        return 0;
    }

    fd = &mut (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(enemyClient as usize)).client)
        .ps
        .fd;

    if fd.is_null() {
        return 0;
    }

    if botClient > 47 {
        if (*fd).forceMindtrickTargetIndex4 & (1 << (botClient - 48)) != 0 {
            return 1;
        }
    } else if botClient > 31 {
        if (*fd).forceMindtrickTargetIndex3 & (1 << (botClient - 32)) != 0 {
            return 1;
        }
    } else if botClient > 15 {
        if (*fd).forceMindtrickTargetIndex2 & (1 << (botClient - 16)) != 0 {
            return 1;
        }
    } else if (*fd).forceMindtrickTargetIndex & (1 << botClient) != 0 {
        return 1;
    }

    0
}

/// `int IsTeamplay(void)` (ai_main.c:340) — `1` once the gametype reaches [`GT_TEAM`] (team
/// deathmatch and above: CTF, CTY, siege), `0` for the free-for-all gametypes below it.
pub fn IsTeamplay() -> c_int {
    unsafe {
        if (*addr_of!(g_gametype)).integer < GT_TEAM {
            return 0;
        }
    }

    1
}

/*
==================
BotAI_GetClientState
==================
*/
/// `int BotAI_GetClientState(int clientNum, playerState_t *state)` (ai_main.c:355) — copy
/// client `clientNum`'s `playerState_t` out into `*state`, returning `qfalse` (`0`) if the
/// entity slot is unused or not a client.
///
/// # Safety
/// `clientNum` must index a valid [`g_entities`] slot and `state` must point to a writable
/// `playerState_t`.
pub unsafe fn BotAI_GetClientState(clientNum: c_int, state: *mut playerState_t) -> c_int {
    let ent: *mut gentity_t;

    ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(clientNum as usize);
    if (*ent).inuse == QFALSE {
        return QFALSE;
    }
    if (*ent).client.is_null() {
        return QFALSE;
    }

    *state = (*(*ent).client).ps;
    QTRUE
}

/*
==================
BotAI_GetEntityState
==================
*/
/// `int BotAI_GetEntityState(int entityNum, entityState_t *state)` (ai_main.c:375) — zero
/// `*state`, then copy entity `entityNum`'s `entityState_t` into it, returning `qfalse` (`0`)
/// for an unused, unlinked, or `SVF_NOCLIENT` entity.
///
/// # Safety
/// `entityNum` must index a valid [`g_entities`] slot and `state` must point to a writable
/// `entityState_t`.
pub unsafe fn BotAI_GetEntityState(entityNum: c_int, state: *mut entityState_t) -> c_int {
    let ent: *mut gentity_t;

    ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entityNum as usize);
    write_bytes(state, 0, 1);
    if (*ent).inuse == QFALSE {
        return QFALSE;
    }
    if (*ent).r.linked == QFALSE {
        return QFALSE;
    }
    if (*ent).r.svFlags & SVF_NOCLIENT != 0 {
        return QFALSE;
    }
    *state = (*ent).s;
    QTRUE
}

/// `void BotEntityInfo(int entnum, aas_entityinfo_t *info)` (ai_main.c:411) — fetch entity
/// `entnum`'s AAS snapshot via the bot-library trap.
///
/// # Safety
/// `info` must point to a writable [`aas_entityinfo_t`].
pub unsafe fn BotEntityInfo(entnum: c_int, info: *mut aas_entityinfo_t) {
    trap::AAS_EntityInfo(entnum, &mut *info);
}

/*
==============
NumBots
==============
*/
/// `int NumBots(void)` (ai_main.c:420) — the number of bots currently in the game.
///
/// # Safety
/// Reads the [`numbots`] mutable global.
pub unsafe fn NumBots() -> c_int {
    numbots
}

//==============
//AngleDifference
//==============
/// `float AngleDifference(float ang1, float ang2)` (ai_main.c:421) — pure float math, no
/// callees: the signed difference `ang1 - ang2`, wrap-corrected into the `(-180, 180]`
/// range by ±360 depending on which angle is larger.
pub fn AngleDifference(ang1: f32, ang2: f32) -> f32 {
    let mut diff: f32;

    diff = ang1 - ang2;
    if ang1 > ang2 {
        if diff > 180.0 {
            diff -= 360.0;
        }
    } else if diff < -180.0 {
        diff += 360.0;
    }
    diff
}

//==============
//BotChangeViewAngle
//==============
/// `float BotChangeViewAngle(float angle, float ideal_angle, float speed)` (ai_main.c:439) —
/// pure: normalize both angles via [`AngleMod`], compute the wrap-corrected move toward
/// `ideal_angle`, clamp it to `±speed`, and return the new normalized angle.
pub fn BotChangeViewAngle(mut angle: f32, mut ideal_angle: f32, speed: f32) -> f32 {
    let mut move_: f32;

    angle = AngleMod(angle);
    ideal_angle = AngleMod(ideal_angle);
    if angle == ideal_angle {
        return angle;
    }
    move_ = ideal_angle - angle;
    if ideal_angle > angle {
        if move_ > 180.0 {
            move_ -= 360.0;
        }
    } else if move_ < -180.0 {
        move_ += 360.0;
    }
    if move_ > 0.0 {
        if move_ > speed {
            move_ = speed;
        }
    } else if move_ < -speed {
        move_ = -speed;
    }
    AngleMod(angle + move_)
}

/// `void BotAIRegularUpdate(void)` (ai_main.c:662) — throttled (every 0.3s) refresh of the
/// botlib's view of entity items: once [`regularupdate_time`] falls behind [`FloatTime`], poke
/// the engine via [`trap::BotUpdateEntityItems`] and push the next update 0.3s out.
///
/// No oracle: bot/engine glue (mutates a process-global + calls a trap; no pure return value).
///
/// # Safety
/// Reads/writes the process-global [`regularupdate_time`].
pub unsafe fn BotAIRegularUpdate() {
    if regularupdate_time < FloatTime() {
        trap::BotUpdateEntityItems();
        regularupdate_time = (FloatTime() as f64 + 0.3) as f32;
    }
}

/// `void RemoveColorEscapeSequences(char *text)` (ai_main.c:666) — strip Quake color codes
/// (`^x`) from `text` in place: walk the string, skip the two-byte `^x` pairs (via
/// [`Q_IsColorString`]) and any byte above `0x7E`, compacting the survivors down and
/// re-terminating with `'\0'`.
///
/// # Safety
/// `text` must point to a writable, NUL-terminated C string.
pub unsafe fn RemoveColorEscapeSequences(text: *mut c_char) {
    let mut i: isize;
    let mut l: isize;

    l = 0;
    i = 0;
    while *text.offset(i) != 0 {
        if Q_IsColorString(text.offset(i)) {
            i += 1;
            i += 1;
            continue;
        }
        if *text.offset(i) > 0x7E {
            i += 1;
            continue;
        }
        *text.offset(l) = *text.offset(i);
        l += 1;
        i += 1;
    }
    *text.offset(l) = b'\0' as c_char;
}

/// `int PlayersInGame(void)` (ai_main.c:785) — count the clients that are fully connected
/// (`pers.connected == CON_CONNECTED`) across the first [`MAX_CLIENTS`] entity slots.
pub fn PlayersInGame() -> c_int {
    let mut i: usize = 0;
    let mut pl: c_int = 0;

    while i < MAX_CLIENTS {
        unsafe {
            let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);

            if !ent.is_null()
                && !(*ent).client.is_null()
                && (*(*ent).client).pers.connected == CON_CONNECTED
            {
                pl += 1;
            }
        }

        i += 1;
    }

    pl
}

//standard visibility check
/// `int OrgVisible(vec3_t org1, vec3_t org2, int ignore)` (ai_main.c:957) — a straight
/// line-of-sight test: trace a ray (NULL bbox) from `org1` to `org2` against `MASK_SOLID`,
/// skipping entity `ignore`; visible (`1`) iff nothing blocked (`tr.fraction == 1`).
/// The C passes `NULL, NULL` for the trace mins/maxs (a zero-extent ray); the engine reads a
/// NULL bbox as the origin point, so `&vec3_origin` reproduces it exactly.
pub fn OrgVisible(org1: &vec3_t, org2: &vec3_t, ignore: c_int) -> c_int {
    let tr = trap::Trace(org1, &vec3_origin, &vec3_origin, org2, ignore, MASK_SOLID);

    if tr.fraction == 1.0 {
        return 1;
    }

    0
}

//special waypoint visibility check
/// `int WPOrgVisible(gentity_t *bot, vec3_t org1, vec3_t org2, int ignore)` (ai_main.c:972) —
/// a waypoint-aware line-of-sight test. First trace against `MASK_SOLID`: if anything blocks,
/// return `0`. Otherwise re-trace against `MASK_PLAYERSOLID`; if that newly hits an
/// [`ET_SPECIAL`] entity (a force field) owned by a client, the field is "visible-through"
/// (`1`) for the field's owner or teammates and "blocking" (`2`) for everyone else. With no
/// special entity in the way, it's plainly visible (`1`).
///
/// # Safety
/// `bot` must be a valid `gentity_t` pointer; the contacted-entity lookups index
/// [`g_entities`] by the engine-returned `entityNum`.
pub unsafe fn WPOrgVisible(
    bot: *mut gentity_t,
    org1: &vec3_t,
    org2: &vec3_t,
    ignore: c_int,
) -> c_int {
    let ownent: *mut gentity_t;

    let tr = trap::Trace(org1, &vec3_origin, &vec3_origin, org2, ignore, MASK_SOLID);

    if tr.fraction == 1.0 {
        let tr = trap::Trace(org1, &vec3_origin, &vec3_origin, org2, ignore, MASK_PLAYERSOLID);

        if tr.fraction != 1.0
            && tr.entityNum as c_int != ENTITYNUM_NONE
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize)).s.eType == ET_SPECIAL
        {
            let hitent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);
            if !(*hitent).parent.is_null() && !(*(*hitent).parent).client.is_null() {
                ownent = (*hitent).parent;

                if OnSameTeam(bot, ownent) != QFALSE || (*bot).s.number == (*ownent).s.number {
                    return 1;
                }
            }
            return 2;
        }

        return 1;
    }

    0
}

/// `qboolean BotPVSCheck( const vec3_t p1, const vec3_t p2 )` (ai_main.c:1061) — on RMG maps
/// with `bot_pvstype` set, this skips the engine PVS test and instead approximates visibility
/// by a 5000-unit distance cutoff; otherwise it defers to [`trap::InPVS`].
///
/// # Safety
/// Reads the `g_RMG`/`bot_pvstype` mutable cvars.
pub unsafe fn BotPVSCheck(p1: &vec3_t, p2: &vec3_t) -> qboolean {
    if (*addr_of!(g_RMG)).integer != 0 && bot_pvstype.integer != 0 {
        let mut sub_point: vec3_t = vec3_origin;
        VectorSubtract(p1, p2, &mut sub_point);

        if VectorLength(&sub_point) > 5000.0 {
            return QFALSE;
        }
        return QTRUE;
    }

    trap::InPVS(p1, p2)
}

//visibility check with hull trace
/// `int OrgVisibleBox(vec3_t org1, vec3_t mins, vec3_t maxs, vec3_t org2, int ignore)`
/// (ai_main.c:1001) — like [`OrgVisible`] but sweeps the caller's `mins`/`maxs` hull. On RMG
/// (randomly-generated) maps the box is dropped to a zero-extent ray (the C passes `NULL` for
/// the bbox there); the reproduction is `&vec3_origin` exactly as in `OrgVisible`. Visible
/// (`1`) iff the trace completed clean (`fraction == 1`) and started outside solid
/// (`!startsolid && !allsolid`).
pub fn OrgVisibleBox(
    org1: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    org2: &vec3_t,
    ignore: c_int,
) -> c_int {
    let tr = unsafe {
        if (*addr_of!(g_RMG)).integer != 0 {
            trap::Trace(org1, &vec3_origin, &vec3_origin, org2, ignore, MASK_SOLID)
        } else {
            trap::Trace(org1, mins, maxs, org2, ignore, MASK_SOLID)
        }
    };

    if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
        return 1;
    }

    0
}

//get the index to the nearest visible waypoint in the global trail
/// `int GetNearestVisibleWP(vec3_t org, int ignore)` (ai_main.c:1079) — scan the global
/// waypoint trail for the nearest in-use point that is both within `bestdist` (300 on RMG,
/// else 800 to avoid giant trace speed hits) and visible from `org` via [`OrgVisibleBox`]
/// (plus [`BotPVSCheck`] off RMG). Returns the trail index of the best, or `-1` if none.
///
/// # Safety
/// Reads the `gWPArray`/`gWPNum`/`g_RMG` mutable globals.
pub unsafe fn GetNearestVisibleWP(org: &vec3_t, ignore: c_int) -> c_int {
    let mut i: c_int;
    let mut bestdist: f32;
    let mut fl_len: f32;
    let mut bestindex: c_int;
    let mut a: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;

    i = 0;
    if (*addr_of!(g_RMG)).integer != 0 {
        bestdist = 300.0;
    } else {
        bestdist = 800.0; //99999;
                          //don't trace over 800 units away to avoid GIANT HORRIBLE SPEED HITS ^_^
    }
    bestindex = -1;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -1.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 1.0;

    while i < gWPNum {
        if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
            VectorSubtract(org, &(*gWPArray[i as usize]).origin, &mut a);
            fl_len = VectorLength(&a);

            if fl_len < bestdist
                && ((*addr_of!(g_RMG)).integer != 0
                    || BotPVSCheck(org, &(*gWPArray[i as usize]).origin) != QFALSE)
                && OrgVisibleBox(org, &mins, &maxs, &(*gWPArray[i as usize]).origin, ignore) != 0
            {
                bestdist = fl_len;
                bestindex = i;
            }
        }

        i += 1;
    }

    bestindex
}

//see if there's a func_* ent under the given pos.
//kind of badly done, but this shouldn't happen
//often.
/// `int CheckForFunc(vec3_t org, int ignore)` (ai_main.c:1028) — trace 64 units straight down
/// from `org`; if something is hit, look up the contacted entity in [`g_entities`] and report
/// `1` when its `classname` contains `"func_"` (a brush mover under your feet), else `0`.
pub fn CheckForFunc(org: &vec3_t, ignore: c_int) -> c_int {
    let fent: *mut gentity_t;
    let mut under: vec3_t = vec3_origin;

    VectorCopy(org, &mut under);

    under[2] -= 64.0;

    let tr = trap::Trace(org, &vec3_origin, &vec3_origin, &under, ignore, MASK_SOLID);

    if tr.fraction == 1.0 {
        return 0;
    }

    unsafe {
        fent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

        if fent.is_null() {
            return 0;
        }

        if !strstr((*fent).classname, c"func_".as_ptr()).is_null() {
            return 1; //there's a func brush here
        }
    }

    0
}

//check if said angles are within our fov
/// `int InFieldOfVision(vec3_t viewangles, float fov, vec3_t angles)` (ai_main.c:2009) — pure
/// FOV math: for the pitch and yaw axes (`i = 0, 1`), normalize both `viewangles[i]` and
/// `angles[i]` via [`AngleMod`], take their wrap-corrected difference, and return `0` the moment
/// either axis falls outside the half-FOV cone; `1` if both pass. Note the C mutates the caller's
/// `angles[]` in place (each element is replaced by its `AngleMod`), so `angles` is `&mut`.
pub fn InFieldOfVision(viewangles: &vec3_t, fov: f32, angles: &mut vec3_t) -> c_int {
    let mut diff: f32;
    let mut angle: f32;

    for i in 0..2 {
        angle = AngleMod(viewangles[i]);
        angles[i] = AngleMod(angles[i]);
        diff = angles[i] - angle;
        if angles[i] > angle {
            if diff > 180.0 {
                diff -= 360.0;
            }
        } else if diff < -180.0 {
            diff += 360.0;
        }
        if diff > 0.0 {
            if diff > fov * 0.5 {
                return 0;
            }
        } else if diff < -fov * 0.5 {
            return 0;
        }
    }
    1
}

/// `int EntityVisibleBox(vec3_t org1, vec3_t mins, vec3_t maxs, vec3_t org2, int ignore,
/// int ignore2)` (ai_main.c:3011) — a hull trace against `MASK_SOLID` skipping `ignore`.
/// Visible (`1`) iff the sweep completed clean and started outside solid; also counts as
/// visible (`1`) when the only thing hit *is* the second ignore entity `ignore2` (so a target
/// you're tracing toward doesn't read as occluding itself).
pub fn EntityVisibleBox(
    org1: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    org2: &vec3_t,
    ignore: c_int,
    ignore2: c_int,
) -> c_int {
    let tr = trap::Trace(org1, mins, maxs, org2, ignore, MASK_SOLID);

    if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
        return 1;
    } else if tr.entityNum as c_int != ENTITYNUM_NONE && tr.entityNum as c_int == ignore2 {
        return 1;
    }

    0
}

//could we block projectiles from the weapon potentially with a light saber?
/// `int BotWeaponBlockable(int weapon)` (ai_main.c:5839) — pure `int`→`int` classification of
/// `WP_*` weapon ids: melee/stun, the disruptor, demp2, rocket launcher, thermal, trip mine and
/// det pack all fire unblockable (or non-projectile) damage and return `0`; everything else
/// (the blaster-family projectiles) returns `1` (a saber could deflect it).
pub fn BotWeaponBlockable(weapon: c_int) -> c_int {
    match weapon {
        WP_STUN_BATON | WP_MELEE => 0,
        WP_DISRUPTOR => 0,
        WP_DEMP2 => 0,
        WP_ROCKET_LAUNCHER => 0,
        WP_THERMAL => 0,
        WP_TRIP_MINE => 0,
        WP_DET_PACK => 0,
        _ => 1,
    }
}

/// `void MoveTowardIdealAngles(bot_state_t *bs)` (ai_main.c:1518) — set the bot's
/// `ideal_viewangles` to its current `goalAngles` (look straight at the goal).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn MoveTowardIdealAngles(bs: *mut bot_state_t) {
    VectorCopy(&(*bs).goalAngles, &mut (*bs).ideal_viewangles);
}

/// `int BotGetWeaponRange(bot_state_t *bs)` (ai_main.c:2239) — classify the bot's current
/// weapon into a [`BWEAPONRANGE_MELEE`]/`SABER`/`MID`/`LONG` bucket.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotGetWeaponRange(bs: *mut bot_state_t) -> c_int {
    match (*bs).cur_ps.weapon {
        WP_STUN_BATON | WP_MELEE => BWEAPONRANGE_MELEE,
        WP_SABER => BWEAPONRANGE_SABER,
        WP_BRYAR_PISTOL => BWEAPONRANGE_MID,
        WP_BLASTER => BWEAPONRANGE_MID,
        WP_DISRUPTOR => BWEAPONRANGE_MID,
        WP_BOWCASTER => BWEAPONRANGE_LONG,
        WP_REPEATER => BWEAPONRANGE_MID,
        WP_DEMP2 => BWEAPONRANGE_LONG,
        WP_FLECHETTE => BWEAPONRANGE_LONG,
        WP_ROCKET_LAUNCHER => BWEAPONRANGE_LONG,
        WP_THERMAL => BWEAPONRANGE_LONG,
        WP_TRIP_MINE => BWEAPONRANGE_LONG,
        WP_DET_PACK => BWEAPONRANGE_LONG,
        _ => BWEAPONRANGE_MID,
    }
}

//see if we want to run away from the opponent for whatever reason
/// `int BotIsAChickenWuss(bot_state_t *bs)` (ai_main.c:2276) — decide whether the bot should
/// flee its current enemy. Returns 0 (stand) when the level forbids running, in single-player,
/// while raging, or once all flee-checks pass; 1 (flee) on low health / weak-weapon / saber-vs-
/// saber / lightning danger; 2 (don't keep flip-flopping) while the recalculation timer is hot.
/// In GT_JEDIMASTER a non-Master knows no fear except of a strong Master (then may pursue with
/// explosives) or, in the later check, is simply frightened of the Master.
///
/// # Safety
/// `bs` must be valid; reads `gLevelFlags`/`level`/`g_entities`/`g_gametype`.
pub unsafe fn BotIsAChickenWuss(bs: *mut bot_state_t) -> c_int {
    let b_w_range: c_int;

    if (gLevelFlags & LEVELFLAG_IMUSTNTRUNAWAY) != 0 {
        //The level says we mustn't run away!
        return 0;
    }

    if (*addr_of!(g_gametype)).integer == GT_SINGLE_PLAYER {
        //"coop" (not really)
        return 0;
    }

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER && (*bs).cur_ps.isJediMaster == QFALSE {
        //Then you may know no fear.
        //Well, unless he's strong.
        if !(!(*bs).currentEnemy.is_null()
            && !(*(*bs).currentEnemy).client.is_null()
            && (*(*(*bs).currentEnemy).client).ps.isJediMaster != QFALSE
            && (*(*bs).currentEnemy).health > 40
            && (*bs).cur_ps.weapon < WP_ROCKET_LAUNCHER)
        {
            //explosive weapons are most effective against the Jedi Master
            //(otherwise the Master isn't a threat we'll chase) -> fall through to jmPass
            return 0;
        }
    //goto jmPass; (skip the CTF block below)
    } else if (*addr_of!(g_gametype)).integer == GT_CTF
        && !(*bs).currentEnemy.is_null()
        && !(*(*bs).currentEnemy).client.is_null()
    {
        if (*(*(*bs).currentEnemy).client).ps.powerups[PW_REDFLAG as usize] != 0
            || (*(*(*bs).currentEnemy).client).ps.powerups[PW_BLUEFLAG as usize] != 0
        {
            //don't be afraid of flag carriers, they must die!
            return 0;
        }
    }

    //jmPass:
    if (*bs).chickenWussCalculationTime > (*addr_of!(level)).time as f32 {
        return 2; //don't want to keep going between two points...
    }

    if ((*bs).cur_ps.fd.forcePowersActive & (1 << FP_RAGE)) != 0 {
        //don't run while raging
        return 0;
    }

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER && (*bs).cur_ps.isJediMaster == QFALSE {
        //be frightened of the jedi master? I guess in this case.
        return 1;
    }

    (*bs).chickenWussCalculationTime = ((*addr_of!(level)).time + MAX_CHICKENWUSS_TIME) as f32;

    if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < BOT_RUN_HEALTH {
        //we're low on health, let's get away
        return 1;
    }

    b_w_range = BotGetWeaponRange(bs);

    if b_w_range == BWEAPONRANGE_MELEE || b_w_range == BWEAPONRANGE_SABER {
        if b_w_range != BWEAPONRANGE_SABER || (*bs).saberSpecialist == 0 {
            //run away if we're using melee, or if we're using a saber and not a "saber specialist"
            return 1;
        }
    }

    if (*bs).cur_ps.weapon == WP_BRYAR_PISTOL {
        //the bryar is a weak weapon, so just try to find a new one if it's what you're having to use
        return 1;
    }

    if !(*bs).currentEnemy.is_null()
        && !(*(*bs).currentEnemy).client.is_null()
        && (*(*(*bs).currentEnemy).client).ps.weapon == WP_SABER
        && (*bs).frame_Enemy_Len < 512.0
        && (*bs).cur_ps.weapon != WP_SABER
    {
        //if close to an enemy with a saber and not using a saber, then try to back off
        return 1;
    }

    if ((*addr_of!(level)).time - (*bs).cur_ps.electrifyTime) < 16000 {
        //lightning is dangerous.
        return 1;
    }

    //didn't run, reset the timer
    (*bs).chickenWussCalculationTime = 0.0;

    0
}

/// `int PassStandardEnemyChecks(bot_state_t *bs, gentity_t *en)` (ai_main.c:1728) — the common
/// validity gate for treating `en` as an enemy: must be a live, damageable, connected,
/// non-spectator, solid client that is not us, not a teammate, and not a duelist we're not
/// dueling. Mind-trick at >64 units hides the enemy. Returns 1 if `en` is a valid target, else 0.
/// (The commented-out Jedi-Master block is inert in source and omitted.)
///
/// # Safety
/// `bs`/`en` must be valid; reads `level`/`g_entities`/`gLevelFlags`.
pub unsafe fn PassStandardEnemyChecks(bs: *mut bot_state_t, en: *mut gentity_t) -> c_int {
    if bs.is_null() || en.is_null() {
        //shouldn't happen
        return 0;
    }

    if (*en).client.is_null() {
        //not a client, don't care about him
        return 0;
    }

    if (*en).health < 1 {
        //he's already dead
        return 0;
    }

    if (*en).takedamage == QFALSE {
        //a client that can't take damage?
        return 0;
    }

    if (*bs).doingFallback != QFALSE && (gLevelFlags & LEVELFLAG_IGNOREINFALLBACK) != 0 {
        //we screwed up in our nav routines somewhere and we've reverted to a fallback state to
        //try to get back on the trail. If the level specifies to ignore enemies in this state,
        //then ignore them.
        return 0;
    }

    if (*(*en).client).ps.pm_type == PM_INTERMISSION
        || (*(*en).client).ps.pm_type == PM_SPECTATOR
        || (*(*en).client).sess.sessionTeam == TEAM_SPECTATOR
    {
        //don't attack spectators
        return 0;
    }

    if (*(*en).client).pers.connected == CON_DISCONNECTED {
        //a "zombie" client?
        return 0;
    }

    if (*en).s.solid == 0 {
        //shouldn't happen
        return 0;
    }

    if (*bs).client == (*en).s.number {
        //don't attack yourself
        return 0;
    }

    if OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), en) != QFALSE {
        //don't attack teammates
        return 0;
    }

    if BotMindTricked((*bs).client, (*en).s.number) != 0 {
        if !(*bs).currentEnemy.is_null() && (*(*bs).currentEnemy).s.number == (*en).s.number {
            //if mindtricked by this enemy, then be less "aware" of them, even though
            //we know they're there.
            let mut vs: vec3_t = vec3_origin;
            let v_len: f32;

            VectorSubtract(&(*bs).origin, &(*(*en).client).ps.origin, &mut vs);
            v_len = VectorLength(&vs);

            if v_len > 64.0
            /*&& (level.time - en->client->dangerTime) > 150*/
            {
                return 0;
            }
        }
    }

    if (*(*en).client).ps.duelInProgress != QFALSE && (*(*en).client).ps.duelIndex != (*bs).client {
        //don't attack duelists unless you're dueling them
        return 0;
    }

    if (*bs).cur_ps.duelInProgress != QFALSE && (*en).s.number != (*bs).cur_ps.duelIndex {
        //ditto, the other way around
        return 0;
    }

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER
        && (*(*en).client).ps.isJediMaster == QFALSE
        && (*bs).cur_ps.isJediMaster == QFALSE
    {
        //rules for attacking non-JM in JM mode
        let mut vs: vec3_t = vec3_origin;
        let v_len: f32;

        if (*addr_of!(g_friendlyFire)).integer == 0 {
            //can't harm non-JM in JM mode if FF is off
            return 0;
        }

        VectorSubtract(&(*bs).origin, &(*(*en).client).ps.origin, &mut vs);
        v_len = VectorLength(&vs);

        if v_len > 350.0 {
            return 0;
        }
    }

    1
}


//We cannot hurt the ones we love. Unless of course this
//function says we can.
/// `int PassLovedOneCheck(bot_state_t *bs, gentity_t *ent)` (ai_main.c:2053) — return 0 only
/// when `ent` is a "loved" bot we should refuse to attack (per-name attachment list, gated by
/// teamplay + love level). Returns 1 (attackable) for non-loved targets, non-bots, 1-on-1
/// modes, or when `bot_attachments` is off.
///
/// # Safety
/// `bs`/`ent` must be valid; reads `level`/`g_entities`/`bot_attachments`.
pub unsafe fn PassLovedOneCheck(bs: *mut bot_state_t, ent: *mut gentity_t) -> c_int {
    let mut i: c_int;
    let loved: *mut bot_state_t;

    if (*bs).lovednum == 0 {
        return 1;
    }

    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        //There is no love in 1-on-1
        return 1;
    }

    i = 0;

    if (*addr_of!(botstates))[(*ent).s.number as usize].is_null() {
        //not a bot
        return 1;
    }

    if bot_attachments.integer == 0 {
        return 1;
    }

    loved = (*addr_of!(botstates))[(*ent).s.number as usize];

    while i < (*bs).lovednum {
        if strcmp(
            (*(*addr_of!(level)).clients.add((*loved).client as usize))
                .pers
                .netname
                .as_ptr(),
            (*bs).loved[i as usize].name.as_ptr(),
        ) == 0
        {
            if IsTeamplay() == 0 && (*bs).loved[i as usize].level < 2 {
                //if FFA and level of love is not greater than 1, just don't care
                return 1;
            } else if IsTeamplay() != 0
                && OnSameTeam(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize),
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*loved).client as usize),
                ) == QFALSE
                && (*bs).loved[i as usize].level < 2
            {
                //is teamplay, but not on same team and level < 2
                return 1;
            } else {
                return 0;
            }
        }

        i += 1;
    }

    1
}


/// `int GetLoveLevel(bot_state_t *bs, bot_state_t *love)` (ai_main.c:5245) — look up how much
/// `bs` "loves" the bot `love` by matching `love`'s netname against `bs`'s attachment list;
/// returns that attachment's level. 0 in 1-on-1 / on missing data / no attachments; 1 when
/// `bot_attachments` is off.
///
/// # Safety
/// `bs`/`love` must be valid; reads `g_entities`/`bot_attachments`.
pub unsafe fn GetLoveLevel(bs: *mut bot_state_t, love: *mut bot_state_t) -> c_int {
    let mut i: c_int = 0;
    let lname: *const c_char;

    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        //There is no love in 1-on-1
        return 0;
    }

    if bs.is_null()
        || love.is_null()
        || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*love).client as usize))
            .client
            .is_null()
    {
        return 0;
    }

    if (*bs).lovednum == 0 {
        return 0;
    }

    if bot_attachments.integer == 0 {
        return 1;
    }

    lname = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*love).client as usize)).client)
        .pers
        .netname
        .as_ptr();

    if lname.is_null() {
        return 0;
    }

    while i < (*bs).lovednum {
        if strcmp((*bs).loved[i as usize].name.as_ptr(), lname) == 0 {
            return (*bs).loved[i as usize].level;
        }

        i += 1;
    }

    0
}


//standard check to find a new enemy.
/// `int ScanForEnemies(bot_state_t *bs)` (ai_main.c:2109) — sweep every client for the closest
/// valid, visible (or audible) non-teammate enemy that passes [`PassStandardEnemyChecks`],
/// [`BotPVSCheck`] and [`PassLovedOneCheck`]; only switches to a meaningfully closer target
/// (128+) than the current one. Returns the chosen client index, or -1. In GT_JEDIMASTER the
/// Jedi Master is forced to top priority (and is attackable even when non-JM targeting is
/// suppressed because friendly fire is off).
///
/// # Safety
/// `bs` must be valid; reads `g_entities`/`level`.
pub unsafe fn ScanForEnemies(bs: *mut bot_state_t) -> c_int {
    let mut a: vec3_t = vec3_origin;
    let mut distcheck: f32;
    let mut closest: f32;
    let mut bestindex: c_int;
    let mut i: c_int;
    let mut has_enemy_dist: f32 = 0.0;
    let mut no_attack_non_jm: qboolean = QFALSE;

    closest = 999999.0;
    i = 0;
    bestindex = -1;

    if !(*bs).currentEnemy.is_null() {
        //only switch to a new enemy if he's significantly closer
        has_enemy_dist = (*bs).frame_Enemy_Len;
    }

    if !(*bs).currentEnemy.is_null()
        && !(*(*bs).currentEnemy).client.is_null()
        && (*(*(*bs).currentEnemy).client).ps.isJediMaster != QFALSE
    {
        //The Jedi Master must die.
        return -1;
    }

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER
        && G_ThereIsAMaster() != QFALSE
        && (*bs).cur_ps.isJediMaster == QFALSE
    {
        //if friendly fire is on in jedi master we can attack people that bug us
        if (*addr_of!(g_friendlyFire)).integer == 0 {
            no_attack_non_jm = QTRUE;
        } else {
            closest = 128.0; //only get mad at people if they get close enough to you to anger you, or hurt you
        }
    }

    while i <= MAX_CLIENTS as c_int {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if i != (*bs).client
            && !(*ent).client.is_null()
            && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) == QFALSE
            && PassStandardEnemyChecks(bs, ent) != 0
            && BotPVSCheck(&(*(*ent).client).ps.origin, &(*bs).eye) != QFALSE
            && PassLovedOneCheck(bs, ent) != 0
        {
            VectorSubtract(&(*(*ent).client).ps.origin, &(*bs).eye, &mut a);
            distcheck = VectorLength(&a);
            let a_copy = a;
            vectoangles(&a_copy, &mut a);

            if (*(*ent).client).ps.isJediMaster != QFALSE {
                //make us think the Jedi Master is close so we'll attack him above all
                distcheck = 1.0;
            }

            if distcheck < closest
                && ((InFieldOfVision(&(*bs).viewangles, 90.0, &mut a) != 0
                    && BotMindTricked((*bs).client, i) == 0)
                    || BotCanHear(bs, ent, distcheck) != 0)
                && OrgVisible(&(*bs).eye, &(*(*ent).client).ps.origin, -1) != 0
            {
                if BotMindTricked((*bs).client, i) != 0 {
                    if distcheck < 256.0
                        || ((*addr_of!(level)).time - (*(*ent).client).dangerTime) < 100
                    {
                        if has_enemy_dist == 0.0 || distcheck < (has_enemy_dist - 128.0) {
                            //if we have an enemy, only switch to closer if he is 128+ closer to avoid flipping out
                            if no_attack_non_jm == QFALSE
                                || (*(*ent).client).ps.isJediMaster != QFALSE
                            {
                                closest = distcheck;
                                bestindex = i;
                            }
                        }
                    }
                } else if has_enemy_dist == 0.0 || distcheck < (has_enemy_dist - 128.0) {
                    //if we have an enemy, only switch to closer if he is 128+ closer to avoid flipping out
                    if no_attack_non_jm == QFALSE || (*(*ent).client).ps.isJediMaster != QFALSE {
                        closest = distcheck;
                        bestindex = i;
                    }
                }
            }
        }
        i += 1;
    }

    bestindex
}


//Notifies the bot that he has taken damage from "attacker".
/// `void BotDamageNotification(gclient_t *bot, gentity_t *attacker)` (ai_main.c:1836) — record
/// that `attacker` hurt the bot. Bot-vs-bot transfers "lastAttacked" exclusivity; a real-client
/// attacker clears everyone's claim. If the bot has no current enemy and `attacker` passes the
/// standard + loved-one checks, it becomes the bot's current enemy.
///
/// # Safety
/// `bot`/`attacker` must be valid; reads/writes `botstates`/`level`/`g_entities`.
pub unsafe fn BotDamageNotification(bot: *mut gclient_t, attacker: *mut gentity_t) {
    let bs: *mut bot_state_t;
    let bs_a: *mut bot_state_t;
    let mut i: c_int;

    if bot.is_null() || attacker.is_null() || (*attacker).client.is_null() {
        return;
    }

    if (*bot).ps.clientNum >= MAX_CLIENTS as c_int {
        //an NPC.. do nothing for them.
        return;
    }

    if (*attacker).s.number >= MAX_CLIENTS as c_int {
        //if attacker is an npc also don't care I suppose.
        return;
    }

    bs_a = (*addr_of!(botstates))[(*attacker).s.number as usize];

    if !bs_a.is_null() {
        //if the client attacking us is a bot as well
        (*bs_a).lastAttacked = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bot).ps.clientNum as usize);
        i = 0;

        while i < MAX_CLIENTS as c_int {
            if !(*addr_of!(botstates))[i as usize].is_null()
                && i != (*bs_a).client
                && (*(*addr_of!(botstates))[i as usize]).lastAttacked
                    == (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bot).ps.clientNum as usize)
            {
                (*(*addr_of!(botstates))[i as usize]).lastAttacked = null_mut();
            }

            i += 1;
        }
    } else {
        //got attacked by a real client, so no one gets rights to lastAttacked
        i = 0;

        while i < MAX_CLIENTS as c_int {
            if !(*addr_of!(botstates))[i as usize].is_null()
                && (*(*addr_of!(botstates))[i as usize]).lastAttacked
                    == (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bot).ps.clientNum as usize)
            {
                (*(*addr_of!(botstates))[i as usize]).lastAttacked = null_mut();
            }

            i += 1;
        }
    }

    bs = (*addr_of!(botstates))[(*bot).ps.clientNum as usize];

    if bs.is_null() {
        return;
    }

    (*bs).lastHurt = attacker;

    if !(*bs).currentEnemy.is_null() {
        //we don't care about the guy attacking us if we have an enemy already
        return;
    }

    if PassStandardEnemyChecks(bs, attacker) == 0 {
        //the person that hurt us is not a valid enemy
        return;
    }

    if PassLovedOneCheck(bs, attacker) != 0 {
        //the person that hurt us is the one we love!
        (*bs).currentEnemy = attacker;
        (*bs).enemySeenTime = ((*addr_of!(level)).time + ENEMY_FORGET_MS) as f32;
    }
}


/// `gentity_t *CheckForFriendInLOF(bot_state_t *bs)` (ai_main.c:5520) — trace 2048 units along
/// the bot's view; if the first client hit is a teammate (teamplay) or a sufficiently-loved bot
/// (love level > 1), return it so the caller can hold fire. Otherwise NULL.
///
/// # Safety
/// `bs` must be valid; reads `g_entities`/`botstates`.
pub unsafe fn CheckForFriendInLOF(bs: *mut bot_state_t) -> *mut gentity_t {
    let mut fwd: vec3_t = vec3_origin;
    let mut trfrom: vec3_t = vec3_origin;
    let mut trto: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;
    let trent: *mut gentity_t;

    mins[0] = -3.0;
    mins[1] = -3.0;
    mins[2] = -3.0;

    maxs[0] = 3.0;
    maxs[1] = 3.0;
    maxs[2] = 3.0;

    AngleVectors(&(*bs).viewangles, Some(&mut fwd), None, None);

    VectorCopy(&(*bs).eye, &mut trfrom);

    trto[0] = trfrom[0] + fwd[0] * 2048.0;
    trto[1] = trfrom[1] + fwd[1] * 2048.0;
    trto[2] = trfrom[2] + fwd[2] * 2048.0;

    let tr = trap::Trace(&trfrom, &mins, &maxs, &trto, (*bs).client, MASK_PLAYERSOLID);

    if tr.fraction != 1.0 && (tr.entityNum as c_int) <= MAX_CLIENTS as c_int {
        trent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);

        if !trent.is_null() && !(*trent).client.is_null() {
            if IsTeamplay() != 0
                && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), trent) != QFALSE
            {
                return trent;
            }

            if !(*addr_of!(botstates))[(*trent).s.number as usize].is_null()
                && GetLoveLevel(bs, (*addr_of!(botstates))[(*trent).s.number as usize]) > 1
            {
                return trent;
            }
        }
    }

    null_mut()
}


//get the nearest possible waypoint to the flag since it's not in its original position
/// `void GetNewFlagPoint(wpobject_t *wp, gentity_t *flagEnt, int team)` (ai_main.c:2714) — when a
/// CTF flag has moved off its original waypoint, find the nearest waypoint that has a clear
/// `MASK_SOLID` line to the flag entity and record it as the team's flag point. If the current
/// `wp` is already close (`WP_KEEP_FLAG_DIST`) and clear, keep it.
///
/// # Safety
/// `wp`/`flagEnt` must be valid; reads/writes `gWPArray`/`gWPNum`/`flagRed`/`flagBlue`.
pub unsafe fn GetNewFlagPoint(wp: *mut wpobject_t, flag_ent: *mut gentity_t, team: c_int) {
    let mut i: c_int = 0;
    let mut a: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;
    let mut bestdist: f32;
    let mut testdist: f32;
    let mut bestindex: c_int = 0;
    let mut foundindex: c_int = 0;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -5.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 5.0;

    VectorSubtract(&(*wp).origin, &(*flag_ent).s.pos.trBase, &mut a);

    bestdist = VectorLength(&a);

    if bestdist <= WP_KEEP_FLAG_DIST as f32 {
        let tr = trap::Trace(
            &(*wp).origin,
            &mins,
            &maxs,
            &(*flag_ent).s.pos.trBase,
            (*flag_ent).s.number,
            MASK_SOLID,
        );

        if tr.fraction == 1.0 {
            //this point is good
            return;
        }
    }

    while i < gWPNum {
        VectorSubtract(
            &(*gWPArray[i as usize]).origin,
            &(*flag_ent).s.pos.trBase,
            &mut a,
        );
        testdist = VectorLength(&a);

        if testdist < bestdist {
            let tr = trap::Trace(
                &(*gWPArray[i as usize]).origin,
                &mins,
                &maxs,
                &(*flag_ent).s.pos.trBase,
                (*flag_ent).s.number,
                MASK_SOLID,
            );

            if tr.fraction == 1.0 {
                foundindex = 1;
                bestindex = i;
                bestdist = testdist;
            }
        }

        i += 1;
    }

    if foundindex != 0 {
        if team == TEAM_RED {
            flagRed = gWPArray[bestindex as usize];
        } else {
            flagBlue = gWPArray[bestindex as usize];
        }
    }
}


/// `float BotWeaponCanLead(bot_state_t *bs)` (ai_main.c:4546) — per-weapon aim-lead factor for
/// the bot's current weapon; `0` for weapons we don't lead with.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotWeaponCanLead(bs: *mut bot_state_t) -> f32 {
    let weap = (*bs).cur_ps.weapon;

    if weap == WP_BRYAR_PISTOL {
        return 0.5;
    }
    if weap == WP_BLASTER {
        return 0.35;
    }
    if weap == WP_BOWCASTER {
        return 0.5;
    }
    if weap == WP_REPEATER {
        return 0.45;
    }
    if weap == WP_THERMAL {
        return 0.5;
    }
    if weap == WP_DEMP2 {
        return 0.35;
    }
    if weap == WP_ROCKET_LAUNCHER {
        return 0.7;
    }

    0.0
}

/// `qboolean BotWeaponSelectable(bot_state_t *bs, int weapon)` (ai_main.c:5082) — can the bot
/// switch to `weapon`? `qfalse` for [`WP_NONE`]; otherwise `qtrue` iff it holds the weapon
/// (the [`STAT_WEAPONS`] bit) and has enough ammo for one shot.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer and `weapon` a valid `WP_*` index into
/// [`weaponData`].
pub unsafe fn BotWeaponSelectable(bs: *mut bot_state_t, weapon: c_int) -> qboolean {
    if weapon == WP_NONE {
        return QFALSE;
    }

    if (*bs).cur_ps.ammo[(*addr_of!(weaponData))[weapon as usize].ammoIndex as usize]
        >= (*addr_of!(weaponData))[weapon as usize].energyPerShot
        && (*bs).cur_ps.stats[STAT_WEAPONS as usize] & (1 << weapon) != 0
    {
        return QTRUE;
    }

    QFALSE
}

/// `int PrimFiring(bot_state_t *bs)` (ai_main.c:5448) — is the bot's primary fire effectively
/// active? `1` when it wants to attack and isn't charging, or when it's charging and has
/// released (charging weapons fire on release).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn PrimFiring(bs: *mut bot_state_t) -> c_int {
    if (*bs).cur_ps.weaponstate != WEAPON_CHARGING && (*bs).doAttack != 0 {
        return 1;
    }

    if (*bs).cur_ps.weaponstate == WEAPON_CHARGING && (*bs).doAttack == 0 {
        return 1;
    }

    0
}

/// `int KeepPrimFromFiring(bot_state_t *bs)` (ai_main.c:5466) — force the bot's primary fire
/// off by inverting `doAttack` to whatever state suppresses firing for the current weaponstate
/// (the mirror of [`PrimFiring`]). Always returns `0`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn KeepPrimFromFiring(bs: *mut bot_state_t) -> c_int {
    if (*bs).cur_ps.weaponstate != WEAPON_CHARGING && (*bs).doAttack != 0 {
        (*bs).doAttack = 0;
    }

    if (*bs).cur_ps.weaponstate == WEAPON_CHARGING && (*bs).doAttack == 0 {
        (*bs).doAttack = 1;
    }

    0
}

/// `int AltFiring(bot_state_t *bs)` (ai_main.c:5484) — the alt-fire mirror of [`PrimFiring`]:
/// `1` when the bot wants to alt-attack and isn't alt-charging, or is alt-charging and has
/// released.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn AltFiring(bs: *mut bot_state_t) -> c_int {
    if (*bs).cur_ps.weaponstate != WEAPON_CHARGING_ALT && (*bs).doAltAttack != 0 {
        return 1;
    }

    if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT && (*bs).doAltAttack == 0 {
        return 1;
    }

    0
}

/// `int KeepAltFromFiring(bot_state_t *bs)` (ai_main.c:5502) — force alt-fire off by inverting
/// `doAltAttack` (the mirror of [`KeepPrimFromFiring`]). Always returns `0`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn KeepAltFromFiring(bs: *mut bot_state_t) -> c_int {
    if (*bs).cur_ps.weaponstate != WEAPON_CHARGING_ALT && (*bs).doAltAttack != 0 {
        (*bs).doAltAttack = 0;
    }

    if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT && (*bs).doAltAttack == 0 {
        (*bs).doAltAttack = 1;
    }

    0
}

/// `void UpdateEventTracker(void)` (ai_main.c:1988) — for every client, if its
/// `ps.eventSequence` has advanced, snapshot its latest two events into [`gBotEventTracker`]
/// and stamp a 0.5s expiry (`level.time + 0.5`).
///
/// # Safety
/// Reads/writes the [`gBotEventTracker`] global and the [`level`] clients array; valid only
/// once `level.clients` points at the allocated `MAX_CLIENTS` client array.
pub unsafe fn UpdateEventTracker() {
    let mut i: usize;

    i = 0;

    while i < MAX_CLIENTS {
        let cl = (*addr_of!(level)).clients.add(i);
        if (*addr_of!(gBotEventTracker))[i].eventSequence != (*cl).ps.eventSequence {
            //updated event
            let tracker = &mut (*addr_of_mut!(gBotEventTracker))[i];
            tracker.eventSequence = (*cl).ps.eventSequence;
            tracker.events[0] = (*cl).ps.events[0];
            tracker.events[1] = (*cl).ps.events[1];
            tracker.eventTime = (*addr_of!(level)).time as f32 + 0.5;
        }

        i += 1;
    }
}

/// `int BotSurfaceNear(bot_state_t *bs)` (ai_main.c:5817) — is there solid within 64 units
/// straight ahead of the bot? Trace a zero-extent ray from the bot's origin along its forward
/// view vector and return `1` if anything was hit. The C passes `NULL, NULL` mins/maxs (a
/// point ray); `&vec3_origin` reproduces it as in [`OrgVisible`].
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotSurfaceNear(bs: *mut bot_state_t) -> c_int {
    let mut fwd: vec3_t = vec3_origin;

    AngleVectors(&(*bs).viewangles, Some(&mut fwd), None, None);

    fwd[0] = (*bs).origin[0] + (fwd[0] * 64.0);
    fwd[1] = (*bs).origin[1] + (fwd[1] * 64.0);
    fwd[2] = (*bs).origin[2] + (fwd[2] * 64.0);

    let tr = trap::Trace(&(*bs).origin, &vec3_origin, &vec3_origin, &fwd, (*bs).client, MASK_SOLID);

    if tr.fraction != 1.0 {
        return 1;
    }

    0
}

/// `int WaitingForNow(bot_state_t *bs, vec3_t goalpos)` (ai_main.c:2197) — is the bot riding an
/// elevator toward `goalpos`? Only when `goalpos` matches the current waypoint origin (integer
/// compare). If the bot is near the waypoint in XY and its waypoint distance is large, a
/// [`CheckForFunc`] under its feet means it's probably on a riding mover (`1`); a looser
/// near-check instead just defers its use-trigger (`noUseTime`).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads its `wpCurrent` waypoint object.
pub unsafe fn WaitingForNow(bs: *mut bot_state_t, goalpos: &vec3_t) -> c_int {
    //checks if the bot is doing something along the lines of waiting for an elevator to raise up
    let mut xybot: vec3_t = vec3_origin;
    let mut xywp: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;

    if (*bs).wpCurrent.is_null() {
        return 0;
    }

    if goalpos[0] as c_int != (*(*bs).wpCurrent).origin[0] as c_int
        || goalpos[1] as c_int != (*(*bs).wpCurrent).origin[1] as c_int
        || goalpos[2] as c_int != (*(*bs).wpCurrent).origin[2] as c_int
    {
        return 0;
    }

    VectorCopy(&(*bs).origin, &mut xybot);
    VectorCopy(&(*(*bs).wpCurrent).origin, &mut xywp);

    xybot[2] = 0.0;
    xywp[2] = 0.0;

    VectorSubtract(&xybot, &xywp, &mut a);

    if VectorLength(&a) < 16.0 && (*bs).frame_Waypoint_Len > 100.0 {
        if CheckForFunc(&(*bs).origin, (*bs).client) != 0 {
            return 1; //we're probably standing on an elevator and riding up/down. Or at least we hope so.
        }
    } else if VectorLength(&a) < 64.0
        && (*bs).frame_Waypoint_Len > 64.0
        && CheckForFunc(&(*bs).origin, (*bs).client) != 0
    {
        (*bs).noUseTime = (*addr_of!(level)).time + 2000;
    }

    0
}

/// `int BotCanHear(bot_state_t *bs, gentity_t *en, float endist)` (ai_main.c:1920) — could the
/// bot have heard `en` at distance `endist`? Establishes a `minlen` audible radius from the
/// enemy's most recent noise — a recent other-sound, a footstep, or the kind of the last tracked
/// event (weapon fire 512, footsteps/jumps/global-sound 256, else effectively inaudible) — then
/// halves it if `en` has mind-tricked the bot, and returns `1` iff `endist <= minlen`.
///
/// # Safety
/// `bs`/`en` must be valid pointers; reads [`gBotEventTracker`]/[`level`] and indexes
/// [`g_entities`] via `en->s.number`.
pub unsafe fn BotCanHear(bs: *mut bot_state_t, en: *mut gentity_t, endist: f32) -> c_int {
    let mut minlen: f32;

    if en.is_null() || (*en).client.is_null() {
        return 0;
    }

    // The C reaches `checkStep` either by an early `goto` (recent sound/footstep) or by
    // falling through the event switch; this block reproduces that fall-through.
    'compute: {
        if !en.is_null()
            && !(*en).client.is_null()
            && (*(*en).client).ps.otherSoundTime > (*addr_of!(level)).time
        {
            //they made a noise in recent time
            minlen = (*(*en).client).ps.otherSoundLen;
            break 'compute;
        }

        if !en.is_null()
            && !(*en).client.is_null()
            && (*(*en).client).ps.footstepTime > (*addr_of!(level)).time
        {
            //they made a footstep
            minlen = 256.0;
            break 'compute;
        }

        if (*addr_of!(gBotEventTracker))[(*en).s.number as usize].eventTime
            < (*addr_of!(level)).time as f32
        {
            //no recent events to check
            return 0;
        }

        let tracker = &(*addr_of!(gBotEventTracker))[(*en).s.number as usize];
        match tracker.events[(tracker.eventSequence & (MAX_PS_EVENTS as c_int - 1)) as usize] {
            //did the last event contain a sound?
            EV_GLOBAL_SOUND => {
                minlen = 256.0;
            }
            EV_FIRE_WEAPON | EV_ALT_FIRE | EV_SABER_ATTACK => {
                minlen = 512.0;
            }
            EV_STEP_4 | EV_STEP_8 | EV_STEP_12 | EV_STEP_16 | EV_FOOTSTEP | EV_FOOTSTEP_METAL
            | EV_FOOTWADE => {
                minlen = 256.0;
            }
            EV_JUMP | EV_ROLL => {
                minlen = 256.0;
            }
            _ => {
                minlen = 999999.0;
            }
        }
    }

    // checkStep:
    if BotMindTricked((*bs).client, (*en).s.number) != 0 {
        //if mindtricked by this person, cut down on the minlen so they can't "hear" as well
        minlen /= 4.0;
    }

    if endist <= minlen {
        //we heard it
        return 1;
    }

    0
}

/// `int BotHasAssociated(bot_state_t *bs, wpobject_t *wp)` (ai_main.c:3476) — does the bot
/// already own the item the waypoint `wp` is associated with (so it needn't bother going for
/// it)? A waypoint with no association ([`ENTITYNUM_NONE`]) reads as "already have it" (`1`);
/// otherwise look up the associated entity's item and test the matching inventory bit/count
/// per item type (weapon/holdable/powerup, or >10 ammo by the C's hack).
///
/// # Safety
/// `bs`/`wp` must be valid pointers; `wp->associated_entity` indexes [`g_entities`].
pub unsafe fn BotHasAssociated(bs: *mut bot_state_t, wp: *mut wpobject_t) -> c_int {
    let as_: *mut gentity_t;

    if (*wp).associated_entity == ENTITYNUM_NONE {
        //make it think this is an item we have so we don't go after nothing
        return 1;
    }

    as_ = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*wp).associated_entity as usize);

    if as_.is_null() || (*as_).item.is_null() {
        return 0;
    }

    if (*(*as_).item).giType == IT_WEAPON {
        if (*bs).cur_ps.stats[STAT_WEAPONS as usize] & (1 << (*(*as_).item).giTag) != 0 {
            return 1;
        }

        return 0;
    } else if (*(*as_).item).giType == IT_HOLDABLE {
        if (*bs).cur_ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << (*(*as_).item).giTag) != 0 {
            return 1;
        }

        return 0;
    } else if (*(*as_).item).giType == IT_POWERUP {
        if (*bs).cur_ps.powerups[(*(*as_).item).giTag as usize] != 0 {
            return 1;
        }

        return 0;
    } else if (*(*as_).item).giType == IT_AMMO {
        if (*bs).cur_ps.ammo[(*(*as_).item).giTag as usize] > 10 {
            //hack
            return 1;
        }

        return 0;
    }

    0
}

// #define BOT_STRAFE_AVOIDANCE (ai_main.c) — the strafe-around trace block below.
const STRAFEAROUND_RIGHT: c_int = 1;
const STRAFEAROUND_LEFT: c_int = 2;

/// `int BotTrace_Strafe(bot_state_t *bs, vec3_t traceto)` (ai_main.c:1531) — should the bot
/// strafe around an obstacle on the way to `traceto`? Only while grounded and roughly facing
/// the travel direction. Forward-trace the player hull 32u: if clear, no strafe (`0`). Else
/// try the same trace shifted 32u right ([`STRAFEAROUND_RIGHT`]) then 32u left of that
/// ([`STRAFEAROUND_LEFT`]); `0` if neither side is open.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotTrace_Strafe(bs: *mut bot_state_t, traceto: &vec3_t) -> c_int {
    let player_mins: vec3_t = [-15.0, -15.0, /*DEFAULT_MINS_2*/ -8.0];
    let player_maxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as f32];
    let mut from: vec3_t = vec3_origin;
    let mut to: vec3_t = vec3_origin;
    let mut dir_ang: vec3_t = vec3_origin;
    let mut dir_dif: vec3_t = vec3_origin;
    let mut forward: vec3_t = vec3_origin;
    let mut right: vec3_t = vec3_origin;

    if (*bs).cur_ps.groundEntityNum == ENTITYNUM_NONE {
        //don't do this in the air, it can be.. dangerous.
        return 0;
    }

    VectorSubtract(traceto, &(*bs).origin, &mut dir_ang);
    VectorNormalize(&mut dir_ang);
    let dir_ang_copy = dir_ang;
    vectoangles(&dir_ang_copy, &mut dir_ang);

    if AngleDifference((*bs).viewangles[YAW], dir_ang[YAW]) > 60.0
        || AngleDifference((*bs).viewangles[YAW], dir_ang[YAW]) < -60.0
    {
        //If we aren't facing the direction we're going here, then we've got enough excuse to be too stupid to strafe around anyway
        return 0;
    }

    VectorCopy(&(*bs).origin, &mut from);
    VectorCopy(traceto, &mut to);

    VectorSubtract(&to, &from, &mut dir_dif);
    VectorNormalize(&mut dir_dif);
    let dir_dif_copy = dir_dif;
    vectoangles(&dir_dif_copy, &mut dir_dif);

    AngleVectors(&dir_dif, Some(&mut forward), None, None);

    to[0] = from[0] + forward[0] * 32.0;
    to[1] = from[1] + forward[1] * 32.0;
    to[2] = from[2] + forward[2] * 32.0;

    let tr = trap::Trace(&from, &player_mins, &player_maxs, &to, (*bs).client, MASK_PLAYERSOLID);

    if tr.fraction == 1.0 {
        return 0;
    }

    AngleVectors(&dir_ang, None, Some(&mut right), None);

    from[0] += right[0] * 32.0;
    from[1] += right[1] * 32.0;
    from[2] += right[2] * 16.0;

    to[0] += right[0] * 32.0;
    to[1] += right[1] * 32.0;
    to[2] += right[2] * 32.0;

    let tr = trap::Trace(&from, &player_mins, &player_maxs, &to, (*bs).client, MASK_PLAYERSOLID);

    if tr.fraction == 1.0 {
        return STRAFEAROUND_RIGHT;
    }

    from[0] -= right[0] * 64.0;
    from[1] -= right[1] * 64.0;
    from[2] -= right[2] * 64.0;

    to[0] -= right[0] * 64.0;
    to[1] -= right[1] * 64.0;
    to[2] -= right[2] * 64.0;

    let tr = trap::Trace(&from, &player_mins, &player_maxs, &to, (*bs).client, MASK_PLAYERSOLID);

    if tr.fraction == 1.0 {
        return STRAFEAROUND_LEFT;
    }

    0
}

/// `int BotTrace_Jump(bot_state_t *bs, vec3_t traceto)` (ai_main.c:1613) — is there a low
/// obstacle toward `traceto` the bot should jump over? Trace a tall-ish hull 4u ahead: if clear,
/// nothing to jump (`0`). Otherwise re-trace a thin slab raised 41u; if *that* is clear there's
/// a jumpable ledge — but suppress (`0`) when the blocker is another bot already mid-jump, or is
/// the bot's current saber/melee-range enemy.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; the blocker index `orTr` indexes [`botstates`].
pub unsafe fn BotTrace_Jump(bs: *mut bot_state_t, traceto: &vec3_t) -> c_int {
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;
    let mut fwd: vec3_t = vec3_origin;
    let mut traceto_mod: vec3_t = vec3_origin;
    let mut tracefrom_mod: vec3_t = vec3_origin;
    let or_tr: c_int;

    VectorSubtract(traceto, &(*bs).origin, &mut a);
    let a_copy = a;
    vectoangles(&a_copy, &mut a);

    AngleVectors(&a, Some(&mut fwd), None, None);

    traceto_mod[0] = (*bs).origin[0] + fwd[0] * 4.0;
    traceto_mod[1] = (*bs).origin[1] + fwd[1] * 4.0;
    traceto_mod[2] = (*bs).origin[2] + fwd[2] * 4.0;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -18.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 32.0;

    let tr = trap::Trace(
        &(*bs).origin,
        &mins,
        &maxs,
        &traceto_mod,
        (*bs).client,
        MASK_PLAYERSOLID,
    );

    if tr.fraction == 1.0 {
        return 0;
    }

    or_tr = tr.entityNum as c_int;

    VectorCopy(&(*bs).origin, &mut tracefrom_mod);

    tracefrom_mod[2] += 41.0;
    traceto_mod[2] += 41.0;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = 0.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 8.0;

    let tr = trap::Trace(
        &tracefrom_mod,
        &mins,
        &maxs,
        &traceto_mod,
        (*bs).client,
        MASK_PLAYERSOLID,
    );

    if tr.fraction == 1.0 {
        if or_tr >= 0
            && or_tr < MAX_CLIENTS as c_int
            && !(*addr_of!(botstates))[or_tr as usize].is_null()
            && (*(*addr_of!(botstates))[or_tr as usize]).jumpTime > (*addr_of!(level)).time as f32
        {
            return 0; //so bots don't try to jump over each other at the same time
        }

        if !(*bs).currentEnemy.is_null()
            && (*(*bs).currentEnemy).s.number == or_tr
            && (BotGetWeaponRange(bs) == BWEAPONRANGE_SABER
                || BotGetWeaponRange(bs) == BWEAPONRANGE_MELEE)
        {
            return 0;
        }

        return 1;
    }

    0
}

/// `int BotTrace_Duck(bot_state_t *bs, vec3_t traceto)` (ai_main.c:1677) — is there an overhead
/// obstacle toward `traceto` the bot could duck under? Trace a low standing hull 4u ahead: if
/// blocked, can't simply duck (`0`). Otherwise re-trace a raised crouch-height hull; if *that*
/// is blocked, there's something overhead to duck beneath (`1`).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotTrace_Duck(bs: *mut bot_state_t, traceto: &vec3_t) -> c_int {
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;
    let mut fwd: vec3_t = vec3_origin;
    let mut traceto_mod: vec3_t = vec3_origin;
    let mut tracefrom_mod: vec3_t = vec3_origin;

    VectorSubtract(traceto, &(*bs).origin, &mut a);
    let a_copy = a;
    vectoangles(&a_copy, &mut a);

    AngleVectors(&a, Some(&mut fwd), None, None);

    traceto_mod[0] = (*bs).origin[0] + fwd[0] * 4.0;
    traceto_mod[1] = (*bs).origin[1] + fwd[1] * 4.0;
    traceto_mod[2] = (*bs).origin[2] + fwd[2] * 4.0;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -23.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 8.0;

    let tr = trap::Trace(
        &(*bs).origin,
        &mins,
        &maxs,
        &traceto_mod,
        (*bs).client,
        MASK_PLAYERSOLID,
    );

    if tr.fraction != 1.0 {
        return 0;
    }

    VectorCopy(&(*bs).origin, &mut tracefrom_mod);

    tracefrom_mod[2] += 31.0; //33;
    traceto_mod[2] += 31.0; //33;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = 0.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 32.0;

    let tr = trap::Trace(
        &tracefrom_mod,
        &mins,
        &maxs,
        &traceto_mod,
        (*bs).client,
        MASK_PLAYERSOLID,
    );

    if tr.fraction != 1.0 {
        return 1;
    }

    0
}

/// `void BotAimLeading(bot_state_t *bs, vec3_t headlevel, float leadAmount)` (ai_main.c:4583) —
/// lead the bot's aim ahead of a moving enemy. Sum the enemy's absolute velocity (clamped at
/// 400), project a point `x` units along its normalized movement vector from `headlevel`
/// (`x` scaled by enemy distance, `leadAmount`, and speed), and set `goalAngles` to look at that
/// predicted spot. No-op without a client enemy or a known enemy distance.
///
/// Note `x` is a C `int`: each assignment truncates toward zero, so the projection distance is
/// integral — reproduced here with `c_int`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
// The first `x = frame_Enemy_Len*leadAmount` is overwritten unconditionally below; kept to
// mirror the C exactly.
#[allow(unused_assignments)]
pub unsafe fn BotAimLeading(bs: *mut bot_state_t, headlevel: &vec3_t, lead_amount: f32) {
    let mut x: c_int;
    let mut predicted_spot: vec3_t = vec3_origin;
    let mut movement_vector: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;
    let mut ang: vec3_t = vec3_origin;
    let mut vtotal: f32;

    if (*bs).currentEnemy.is_null() || (*(*bs).currentEnemy).client.is_null() {
        return;
    }

    if (*bs).frame_Enemy_Len == 0.0 {
        return;
    }

    vtotal = 0.0;

    if (*(*(*bs).currentEnemy).client).ps.velocity[0] < 0.0 {
        vtotal += -(*(*(*bs).currentEnemy).client).ps.velocity[0];
    } else {
        vtotal += (*(*(*bs).currentEnemy).client).ps.velocity[0];
    }

    if (*(*(*bs).currentEnemy).client).ps.velocity[1] < 0.0 {
        vtotal += -(*(*(*bs).currentEnemy).client).ps.velocity[1];
    } else {
        vtotal += (*(*(*bs).currentEnemy).client).ps.velocity[1];
    }

    if (*(*(*bs).currentEnemy).client).ps.velocity[2] < 0.0 {
        vtotal += -(*(*(*bs).currentEnemy).client).ps.velocity[2];
    } else {
        vtotal += (*(*(*bs).currentEnemy).client).ps.velocity[2];
    }

    //G_Printf("Leadin target with a velocity total of %f\n", vtotal);

    VectorCopy(&(*(*(*bs).currentEnemy).client).ps.velocity, &mut movement_vector);

    VectorNormalize(&mut movement_vector);

    x = ((*bs).frame_Enemy_Len * lead_amount) as c_int; //hardly calculated with an exact science, but it works

    if vtotal > 400.0 {
        vtotal = 400.0;
    }

    if vtotal != 0.0 {
        x = (((*bs).frame_Enemy_Len * 0.9) * lead_amount * (vtotal * 0.0012)) as c_int; //hardly calculated with an exact science, but it works
    } else {
        x = (((*bs).frame_Enemy_Len * 0.9) * lead_amount) as c_int; //hardly calculated with an exact science, but it works
    }

    predicted_spot[0] = headlevel[0] + (movement_vector[0] * x as f32);
    predicted_spot[1] = headlevel[1] + (movement_vector[1] * x as f32);
    predicted_spot[2] = headlevel[2] + (movement_vector[2] * x as f32);

    VectorSubtract(&predicted_spot, &(*bs).eye, &mut a);
    vectoangles(&a, &mut ang);
    VectorCopy(&ang, &mut (*bs).goalAngles);
}

/// `void BotAimOffsetGoalAngles(bot_state_t *bs)` (ai_main.c:4663) — perturb the bot's
/// `goalAngles` so its aim isn't perfect. While a previously-rolled offset is still live
/// (`aimOffsetTime > level.time`) just re-apply it (wrapping each axis into `[0,360)`).
/// Otherwise compute an accuracy value from skill (worse when mind-tricked, better when enraged
/// at a revenge target, scaled by enemy/self motion), clamp it, and roll fresh random yaw/pitch
/// offsets with a new expiry. `perfectaim` bots are left untouched.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotAimOffsetGoalAngles(bs: *mut bot_state_t) {
    let mut i: c_int;
    let mut acc_val: f32;
    i = 0;

    if (*bs).skills.perfectaim != 0 {
        return;
    }

    if (*bs).aimOffsetTime > (*addr_of!(level)).time as f32 {
        if (*bs).aimOffsetAmtYaw != 0.0 {
            (*bs).goalAngles[YAW] += (*bs).aimOffsetAmtYaw;
        }

        if (*bs).aimOffsetAmtPitch != 0.0 {
            (*bs).goalAngles[PITCH] += (*bs).aimOffsetAmtPitch;
        }

        while i <= 2 {
            if (*bs).goalAngles[i as usize] > 360.0 {
                (*bs).goalAngles[i as usize] -= 360.0;
            }

            if (*bs).goalAngles[i as usize] < 0.0 {
                (*bs).goalAngles[i as usize] += 360.0;
            }

            i += 1;
        }
        return;
    }

    acc_val = (*bs).skills.accuracy / (*bs).settings.skill;

    if !(*bs).currentEnemy.is_null() && BotMindTricked((*bs).client, (*(*bs).currentEnemy).s.number) != 0
    {
        //having to judge where they are by hearing them, so we should be quite inaccurate here
        acc_val *= 7.0;

        if acc_val < 30.0 {
            acc_val = 30.0;
        }
    }

    if !(*bs).revengeEnemy.is_null()
        && (*bs).revengeHateLevel != 0
        && (*bs).currentEnemy == (*bs).revengeEnemy
    {
        //bot becomes more skilled as anger level raises
        acc_val /= (*bs).revengeHateLevel as f32;
    }

    if !(*bs).currentEnemy.is_null() && (*bs).frame_Enemy_Vis != 0 {
        //assume our goal is aiming at the enemy, seeing as he's visible and all
        if (*(*bs).currentEnemy).s.pos.trDelta[0] == 0.0
            && (*(*bs).currentEnemy).s.pos.trDelta[1] == 0.0
            && (*(*bs).currentEnemy).s.pos.trDelta[2] == 0.0
        {
            acc_val = 0.0; //he's not even moving, so he shouldn't really be hard to hit.
        } else {
            acc_val += acc_val * 0.25; //if he's moving he's this much harder to hit
        }

        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).s.pos.trDelta[0] != 0.0
            || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).s.pos.trDelta[1] != 0.0
            || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).s.pos.trDelta[2] != 0.0
        {
            acc_val += acc_val * 0.15; //make it somewhat harder to aim if we're moving also
        }
    }

    if acc_val > 90.0 {
        acc_val = 90.0;
    }
    if acc_val < 1.0 {
        acc_val = 0.0;
    }

    if acc_val == 0.0 {
        (*bs).aimOffsetAmtYaw = 0.0;
        (*bs).aimOffsetAmtPitch = 0.0;
        return;
    }

    if rand() % 10 <= 5 {
        (*bs).aimOffsetAmtYaw = (rand() % acc_val as c_int) as f32;
    } else {
        (*bs).aimOffsetAmtYaw = -(rand() % acc_val as c_int) as f32;
    }

    if rand() % 10 <= 5 {
        (*bs).aimOffsetAmtPitch = (rand() % acc_val as c_int) as f32;
    } else {
        (*bs).aimOffsetAmtPitch = -(rand() % acc_val as c_int) as f32;
    }

    (*bs).aimOffsetTime = ((*addr_of!(level)).time + rand() % 500 + 200) as f32;
}

/// `int ShouldSecondaryFire(bot_state_t *bs)` (ai_main.c:4780) — decide the bot's alt-fire
/// intent for its current weapon: `0` = no, `1` = alt-fire/hold charge, `2` = release a held
/// charge. Bails to `0` without alt ammo. Rocket-launcher alt charging waits for a lock (release
/// after a good lock or a 5s timeout); other charging weapons release once held past
/// `altChargeTime`. Otherwise a per-weapon range table picks alt-fire.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; `weap` indexes [`weaponData`].
pub unsafe fn ShouldSecondaryFire(bs: *mut bot_state_t) -> c_int {
    let weap: c_int;
    let dif: c_int;
    let mut r_time: f32;

    weap = (*bs).cur_ps.weapon;

    if (*bs).cur_ps.ammo[(*addr_of!(weaponData))[weap as usize].ammoIndex as usize]
        < (*addr_of!(weaponData))[weap as usize].altEnergyPerShot
    {
        return 0;
    }

    if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT
        && (*bs).cur_ps.weapon == WP_ROCKET_LAUNCHER
    {
        let held_time: f32 = ((*addr_of!(level)).time - (*bs).cur_ps.weaponChargeTime) as f32;

        r_time = (*bs).cur_ps.rocketLockTime;

        if r_time < 1.0 {
            r_time = (*bs).cur_ps.rocketLastValidTime;
        }

        if held_time > 5000.0 {
            //just give up and release it if we can't manage a lock in 5 seconds
            return 2;
        }

        if r_time > 0.0 {
            dif = (((*addr_of!(level)).time as f32 - r_time) / (1200.0f32 / 16.0f32)) as c_int;

            if dif >= 10 {
                return 2;
            } else if (*bs).frame_Enemy_Len > 250.0 {
                return 1;
            }
        } else if (*bs).frame_Enemy_Len > 250.0 {
            return 1;
        }
    } else if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT
        && ((*addr_of!(level)).time - (*bs).cur_ps.weaponChargeTime) > (*bs).altChargeTime
    {
        return 2;
    } else if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT {
        return 1;
    }

    if weap == WP_BRYAR_PISTOL && (*bs).frame_Enemy_Len < 300.0 {
        return 1;
    } else if weap == WP_BOWCASTER && (*bs).frame_Enemy_Len > 300.0 {
        return 1;
    } else if weap == WP_REPEATER && (*bs).frame_Enemy_Len < 600.0 && (*bs).frame_Enemy_Len > 250.0
    {
        return 1;
    } else if weap == WP_BLASTER && (*bs).frame_Enemy_Len < 300.0 {
        return 1;
    } else if weap == WP_ROCKET_LAUNCHER && (*bs).frame_Enemy_Len > 250.0 {
        return 1;
    }

    0
}

//wpDirection
//0 == FORWARD
//1 == BACKWARD

//see if this is a valid waypoint to pick up in our
//current state (whatever that may be)
/// `int PassWayCheck(bot_state_t *bs, int windex)` (ai_main.c:1132) — gate-check whether the
/// bot may pick up waypoint `windex` given its current travel direction (`wpDirection`),
/// one-way flags, and force-jump capability. RMG flag points are always passable. Returns 1
/// if passable, 0 otherwise.
///
/// # Safety
/// `bs` must be valid; reads the `gWPArray`/`g_RMG` mutable globals.
pub unsafe fn PassWayCheck(bs: *mut bot_state_t, windex: c_int) -> c_int {
    if gWPArray[windex as usize].is_null() || (*gWPArray[windex as usize]).inuse == 0 {
        //bad point index
        return 0;
    }

    if (*addr_of!(g_RMG)).integer != 0 {
        if ((*gWPArray[windex as usize]).flags & WPFLAG_RED_FLAG) != 0
            || ((*gWPArray[windex as usize]).flags & WPFLAG_BLUE_FLAG) != 0
        {
            //red or blue flag, we'd like to get here
            return 1;
        }
    }

    if (*bs).wpDirection != 0 && ((*gWPArray[windex as usize]).flags & WPFLAG_ONEWAY_FWD) != 0 {
        //we're not travelling in a direction on the trail that will allow us to pass this point
        return 0;
    } else if (*bs).wpDirection == 0 && ((*gWPArray[windex as usize]).flags & WPFLAG_ONEWAY_BACK) != 0
    {
        //we're not travelling in a direction on the trail that will allow us to pass this point
        return 0;
    }

    if !(*bs).wpCurrent.is_null()
        && (*gWPArray[windex as usize]).forceJumpTo != 0
        && (*gWPArray[windex as usize]).origin[2] > ((*(*bs).wpCurrent).origin[2] + 64.0)
        && (*bs).cur_ps.fd.forcePowerLevel[FP_LEVITATION as usize]
            < (*gWPArray[windex as usize]).forceJumpTo
    {
        //waypoint requires force jump level greater than our current one to pass
        return 0;
    }

    1
}

//tally up the distance between two waypoints
/// `float TotalTrailDistance(int start, int end, bot_state_t *bs)` (ai_main.c:1168) — sum the
/// `disttonext` of every trail point from `min(start,end)` up to (excluding) `max(start,end)`.
/// Returns -1 if any index is invalid or a one-way point blocks travel in the requested
/// direction (off RMG). The `#if 0` force-jump block in the C is disabled and omitted.
///
/// # Safety
/// Reads the `gWPArray`/`gWPNum`/`g_RMG` mutable globals.
pub unsafe fn TotalTrailDistance(start: c_int, end: c_int, _bs: *mut bot_state_t) -> f32 {
    let beginat: c_int;
    let endat: c_int;
    let mut distancetotal: f32;

    distancetotal = 0.0;

    if start > end {
        beginat = end;
        endat = start;
    } else {
        beginat = start;
        endat = end;
    }

    let mut beginat = beginat;
    while beginat < endat {
        if beginat >= gWPNum
            || gWPArray[beginat as usize].is_null()
            || (*gWPArray[beginat as usize]).inuse == 0
        {
            //invalid waypoint index
            return -1.0;
        }

        if (*addr_of!(g_RMG)).integer == 0 {
            if (end > start && ((*gWPArray[beginat as usize]).flags & WPFLAG_ONEWAY_BACK) != 0)
                || (start > end && ((*gWPArray[beginat as usize]).flags & WPFLAG_ONEWAY_FWD) != 0)
            {
                //a one-way point, this means this path cannot be travelled to the final point
                return -1.0;
            }
        }

        //#if 0 force-jump checks disabled in source, omitted

        distancetotal += (*gWPArray[beginat as usize]).disttonext;

        beginat += 1;
    }

    distancetotal
}

//see if there's a route shorter than our current one to get
//to the final destination we currently desire
/// `void CheckForShorterRoutes(bot_state_t *bs, int newwpindex)` (ai_main.c:1238) — among the
/// neighbors of `newwpindex`, find the one whose [`TotalTrailDistance`] to the current
/// destination is shortest (and 64+ shorter than the current path, or the current path is
/// blocked), respecting the bot's force-jump level. If a better neighbor is found, switch the
/// current waypoint to it (3s switch cooldown) and set up a force-jump if required.
/// `FORCEJUMP_INSTANTMETHOD` is undefined, so the `#ifndef` charge-up branch is taken.
///
/// # Safety
/// `bs` must be valid; reads the `gWPArray` mutable globals + `level`.
pub unsafe fn CheckForShorterRoutes(bs: *mut bot_state_t, newwpindex: c_int) {
    let mut bestlen: f32;
    let mut checklen: f32;
    let mut bestindex: c_int;
    let mut i: c_int;
    let mut fj: c_int;

    i = 0;
    fj = 0;

    if (*bs).wpDestination.is_null() {
        return;
    }

    //set our traversal direction based on the index of the point
    if newwpindex < (*(*bs).wpDestination).index {
        (*bs).wpDirection = 0;
    } else if newwpindex > (*(*bs).wpDestination).index {
        (*bs).wpDirection = 1;
    }

    //can't switch again yet
    if (*bs).wpSwitchTime > (*addr_of!(level)).time as f32 {
        return;
    }

    //no neighboring points to check off of
    if (*gWPArray[newwpindex as usize]).neighbornum == 0 {
        return;
    }

    //get the trail distance for our wp
    bestindex = newwpindex;
    bestlen = TotalTrailDistance(newwpindex, (*(*bs).wpDestination).index, bs);

    while i < (*gWPArray[newwpindex as usize]).neighbornum {
        //now go through the neighbors and check the distance to the desired point from each neighbor
        checklen = TotalTrailDistance(
            (*gWPArray[newwpindex as usize]).neighbors[i as usize].num,
            (*(*bs).wpDestination).index,
            bs,
        );

        if checklen < bestlen - 64.0 || bestlen == -1.0 {
            //this path covers less distance, let's take it instead
            if (*bs).cur_ps.fd.forcePowerLevel[FP_LEVITATION as usize]
                >= (*gWPArray[newwpindex as usize]).neighbors[i as usize].forceJumpTo
            {
                bestlen = checklen;
                bestindex = (*gWPArray[newwpindex as usize]).neighbors[i as usize].num;

                if (*gWPArray[newwpindex as usize]).neighbors[i as usize].forceJumpTo != 0 {
                    fj = (*gWPArray[newwpindex as usize]).neighbors[i as usize].forceJumpTo;
                } else {
                    fj = 0;
                }
            }
        }

        i += 1;
    }

    if bestindex != newwpindex && bestindex != -1 {
        //we found a path we want to switch to, let's do it
        (*bs).wpCurrent = gWPArray[bestindex as usize];
        (*bs).wpSwitchTime = ((*addr_of!(level)).time + 3000) as f32;

        if fj != 0 {
            //do we have to force jump to get to this neighbor?
            //#ifndef FORCEJUMP_INSTANTMETHOD (undefined -> this branch)
            (*bs).forceJumpChargeTime = (*addr_of!(level)).time + 1000;
            (*bs).beStill = ((*addr_of!(level)).time + 1000) as f32;
            (*bs).forceJumping = (*bs).forceJumpChargeTime as f32;
        }
    }
}

//check for flags on the waypoint we're currently travelling to
//and perform the desired behavior based on the flag
/// `void WPConstantRoutine(bot_state_t *bs)` (ai_main.c:1328) — apply the per-frame behaviour
/// implied by the current waypoint's flags: duck on `WPFLAG_DUCK`; on `WPFLAG_JUMP`, charge a
/// force jump if high enough above (and capable), or abandon+reverse the point if it needs
/// force-jump we don't have; charge for `forceJumpTo` points. `FORCEJUMP_INSTANTMETHOD` is
/// undefined, so the `#ifndef` charge-up branches are taken.
///
/// # Safety
/// `bs` must be valid; reads `level`.
pub unsafe fn WPConstantRoutine(bs: *mut bot_state_t) {
    if (*bs).wpCurrent.is_null() {
        return;
    }

    if ((*(*bs).wpCurrent).flags & WPFLAG_DUCK) != 0 {
        //duck while travelling to this point
        (*bs).duckTime = ((*addr_of!(level)).time + 100) as f32;
    }

    //#ifndef FORCEJUMP_INSTANTMETHOD (undefined -> this branch)
    if ((*(*bs).wpCurrent).flags & WPFLAG_JUMP) != 0 {
        //jump while travelling to this point
        let mut height_dif: f32 = (*(*bs).wpCurrent).origin[2] - (*bs).origin[2] + 16.0;

        if (*bs).origin[2] + 16.0 >= (*(*bs).wpCurrent).origin[2] {
            //don't need to jump, we're already higher than this point
            height_dif = 0.0;
        }

        if height_dif > 40.0
            && ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_LEVITATION)) != 0
            && ((*bs).cur_ps.fd.forceJumpCharge
                < (forceJumpStrength
                    [(*bs).cur_ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
                    - 100.0)
                || (*bs).cur_ps.groundEntityNum == ENTITYNUM_NONE)
        {
            //alright, let's jump
            (*bs).forceJumpChargeTime = (*addr_of!(level)).time + 1000;
            if (*bs).cur_ps.groundEntityNum != ENTITYNUM_NONE
                && (*bs).jumpPrep < ((*addr_of!(level)).time - 300) as f32
            {
                (*bs).jumpPrep = ((*addr_of!(level)).time + 700) as f32;
            }
            (*bs).beStill = ((*addr_of!(level)).time + 300) as f32;
            (*bs).jumpTime = 0.0;

            if (*bs).wpSeenTime < ((*addr_of!(level)).time + 600) as f32 {
                (*bs).wpSeenTime = ((*addr_of!(level)).time + 600) as f32;
            }
        } else if height_dif > 64.0
            && ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_LEVITATION)) == 0
        {
            //this point needs force jump to reach and we don't have it
            //Kill the current point and turn around
            (*bs).wpCurrent = null_mut();
            if (*bs).wpDirection != 0 {
                (*bs).wpDirection = 0;
            } else {
                (*bs).wpDirection = 1;
            }

            return;
        }
    }

    if (*(*bs).wpCurrent).forceJumpTo != 0 {
        //#ifndef FORCEJUMP_INSTANTMETHOD (undefined -> this branch)
        let mut height_dif: f32 = (*(*bs).wpCurrent).origin[2] - (*bs).origin[2] + 16.0;

        if (*bs).origin[2] + 16.0 >= (*(*bs).wpCurrent).origin[2] {
            //then why exactly would we be force jumping?
            height_dif = 0.0;
        }
        let _ = height_dif;

        if (*bs).cur_ps.fd.forceJumpCharge
            < (forceJumpStrength
                [(*bs).cur_ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
                - 100.0)
        {
            (*bs).forceJumpChargeTime = (*addr_of!(level)).time + 200;
        }
    }
}

/// `qboolean BotCTFGuardDuty(bot_state_t *bs)` (ai_main.c:1407) — `qtrue` only in CTF/CTY when
/// the bot's CTF role is [`CTFSTATE_DEFENDER`]; `qfalse` otherwise.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotCTFGuardDuty(bs: *mut bot_state_t) -> qboolean {
    if (*addr_of!(g_gametype)).integer != GT_CTF && (*addr_of!(g_gametype)).integer != GT_CTY {
        return QFALSE;
    }

    if (*bs).ctfState == CTFSTATE_DEFENDER {
        return QTRUE;
    }

    QFALSE
}

//when we reach the waypoint we are travelling to,
//this function will be called. We will perform any
//checks for flags on the current wp and activate
//any "touch" events based on that.
/// `void WPTouchRoutine(bot_state_t *bs)` (ai_main.c:1427) — fired when the bot reaches its
/// current waypoint: refresh the travel timer, suppress map-object use on `WPFLAG_NOMOVEFUNC`,
/// jump on `WPFLAG_JUMP` (non-force-jump points), enter camp/snipe behaviour when a chicken/
/// guard/forced-camper hits a camp point, drop camping for melee weapons, and either retire the
/// destination (on arrival) or look for a shorter route. `FORCEJUMP_INSTANTMETHOD` is undefined,
/// so the `#else` (non-force-jump) jump branch is taken.
///
/// # Safety
/// `bs` must be valid; reads `level`/`gWPArray`/`bot_camp`.
pub unsafe fn WPTouchRoutine(bs: *mut bot_state_t) {
    let last_num: c_int;

    if (*bs).wpCurrent.is_null() {
        return;
    }

    (*bs).wpTravelTime = ((*addr_of!(level)).time + 10000) as f32;

    if ((*(*bs).wpCurrent).flags & WPFLAG_NOMOVEFUNC) != 0 {
        //don't try to use any nearby map objects for a little while
        (*bs).noUseTime = (*addr_of!(level)).time + 4000;
    }

    //#ifndef FORCEJUMP_INSTANTMETHOD (undefined -> this branch)
    if ((*(*bs).wpCurrent).flags & WPFLAG_JUMP) != 0 && (*(*bs).wpCurrent).forceJumpTo == 0 {
        //jump if we're flagged to but not if this indicates a force jump point. Force jumping is
        //handled elsewhere.
        (*bs).jumpTime = ((*addr_of!(level)).time + 100) as f32;
    }

    if (*bs).isCamper != 0
        && bot_camp.integer != 0
        && (BotIsAChickenWuss(bs) != 0 || BotCTFGuardDuty(bs) != QFALSE || (*bs).isCamper == 2)
        && (((*(*bs).wpCurrent).flags & WPFLAG_SNIPEORCAMP) != 0
            || ((*(*bs).wpCurrent).flags & WPFLAG_SNIPEORCAMPSTAND) != 0)
        && (*bs).cur_ps.weapon != WP_SABER
        && (*bs).cur_ps.weapon != WP_MELEE
        && (*bs).cur_ps.weapon != WP_STUN_BATON
    {
        //if we're a camper and a chicken then camp
        if (*bs).wpDirection != 0 {
            last_num = (*(*bs).wpCurrent).index + 1;
        } else {
            last_num = (*(*bs).wpCurrent).index - 1;
        }

        if !gWPArray[last_num as usize].is_null()
            && (*gWPArray[last_num as usize]).inuse != 0
            && (*gWPArray[last_num as usize]).index != 0
            && (*bs).isCamping < (*addr_of!(level)).time as f32
        {
            (*bs).isCamping = ((*addr_of!(level)).time + rand() % 15000 + 30000) as f32;
            (*bs).wpCamping = (*bs).wpCurrent;
            (*bs).wpCampingTo = gWPArray[last_num as usize];

            if ((*(*bs).wpCurrent).flags & WPFLAG_SNIPEORCAMPSTAND) != 0 {
                (*bs).campStanding = QTRUE;
            } else {
                (*bs).campStanding = QFALSE;
            }
        }
    } else if ((*bs).cur_ps.weapon == WP_SABER
        || (*bs).cur_ps.weapon == WP_STUN_BATON
        || (*bs).cur_ps.weapon == WP_MELEE)
        && (*bs).isCamping > (*addr_of!(level)).time as f32
    {
        //don't snipe/camp with a melee weapon, that would be silly
        (*bs).isCamping = 0.0;
        (*bs).wpCampingTo = null_mut();
        (*bs).wpCamping = null_mut();
    }

    if !(*bs).wpDestination.is_null() {
        if (*(*bs).wpCurrent).index == (*(*bs).wpDestination).index {
            (*bs).wpDestination = null_mut();

            if (*bs).runningLikeASissy != 0 {
                //this obviously means we're scared and running, so we'll want to keep our navigational priorities less delayed
                (*bs).destinationGrabTime = ((*addr_of!(level)).time + 500) as f32;
            } else {
                (*bs).destinationGrabTime = ((*addr_of!(level)).time + 3500) as f32;
            }
        } else {
            CheckForShorterRoutes(bs, (*(*bs).wpCurrent).index);
        }
    }
}

/// `void BotScheduleBotThink(void)` (ai_main.c:770) — stagger every active bot's
/// `botthink_residual` across the [`BOT_THINK_TIME`] window so they don't all think on the same
/// frame (with `BOT_THINK_TIME == 0` the residuals all come out 0, but the structure is kept).
///
/// # Safety
/// Reads/writes the [`botstates`] slots and the [`numbots`] global.
pub unsafe fn BotScheduleBotThink() {
    let mut botnum: c_int;

    botnum = 0;

    for i in 0..MAX_CLIENTS {
        if (*addr_of!(botstates))[i].is_null() || (*(*addr_of!(botstates))[i]).inuse == 0 {
            continue;
        }
        //initialize the bot think residual time
        (*(*addr_of!(botstates))[i]).botthink_residual = BOT_THINK_TIME * botnum / numbots;
        botnum += 1;
    }
}

/// `void BotResetState(bot_state_t *bs)` (ai_main.c:905) — wipe a bot's state back to zero
/// while preserving the identity/handle fields that must survive a reset: its move/goal/weapon
/// state handles, current player state, settings, in-use flag, client/entity numbers, and
/// enter-game time.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotResetState(bs: *mut bot_state_t) {
    //save some things that should not be reset here
    let settings: bot_settings_t = (*bs).settings;
    let ps: playerState_t = (*bs).cur_ps;
    let inuse: c_int = (*bs).inuse;
    let client: c_int = (*bs).client;
    let entitynum: c_int = (*bs).entitynum;
    let movestate: c_int = (*bs).ms;
    let goalstate: c_int = (*bs).gs;
    let weaponstate: c_int = (*bs).ws;
    let entergame_time: f32 = (*bs).entergame_time;
    //reset the whole state
    write_bytes(bs, 0, 1);
    //copy back some state stuff that should not be reset
    (*bs).ms = movestate;
    (*bs).gs = goalstate;
    (*bs).ws = weaponstate;
    (*bs).cur_ps = ps;
    (*bs).settings = settings;
    (*bs).inuse = inuse;
    (*bs).client = client;
    (*bs).entitynum = entitynum;
    (*bs).entergame_time = entergame_time;
    //reset several states
    if (*bs).ms != 0 {
        trap::BotResetMoveState((*bs).ms);
    }
    if (*bs).gs != 0 {
        trap::BotResetGoalState((*bs).gs);
    }
    if (*bs).ws != 0 {
        trap::BotResetWeaponState((*bs).ws);
    }
    if (*bs).gs != 0 {
        trap::BotResetAvoidGoals((*bs).gs);
    }
    if (*bs).ms != 0 {
        trap::BotResetAvoidReach((*bs).ms);
    }
}

/*
==============
BotAILoadMap
==============
*/
/// `int BotAILoadMap(int restart)` (ai_main.c:941) — on a (re)load, reset every active bot's
/// state via [`BotResetState`] and re-flag it for setup (`setupcount = 4`). Always `qtrue`.
///
/// # Safety
/// Touches the [`botstates`] slots.
#[allow(unused_variables)]
pub unsafe fn BotAILoadMap(restart: c_int) -> c_int {
    for i in 0..MAX_CLIENTS {
        if !(*addr_of!(botstates))[i].is_null() && (*(*addr_of!(botstates))[i]).inuse != 0 {
            BotResetState((*addr_of!(botstates))[i]);
            (*(*addr_of!(botstates))[i]).setupcount = 4;
        }
    }

    QTRUE
}

/// `int BotAISetup( int restart )` (ai_main.c:7569) — register all the `bot_*` cvars and (on a
/// non-restart) zero the bot states and start the bot library.
/// The `#ifndef FINAL_BUILD` `bot_getinthecarrr` register is kept (compiled in the dev build);
/// the `#ifdef _DEBUG` `bot_nogoals`/`bot_debugmessages` registers are omitted (`_DEBUG` off in
/// the release server build). Returns `qtrue` on success / a tournament restart, `qfalse` if the
/// bot library fails to start.
///
/// # Safety
/// Registers into / writes the module-global bot cvars and [`botstates`].
pub unsafe fn BotAISetup(restart: c_int) -> c_int {
    //rww - new bot cvars..
    trap::Cvar_Register(
        addr_of_mut!(bot_forcepowers).as_mut(),
        "bot_forcepowers",
        "1",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        addr_of_mut!(bot_forgimmick).as_mut(),
        "bot_forgimmick",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        addr_of_mut!(bot_honorableduelacceptance).as_mut(),
        "bot_honorableduelacceptance",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        addr_of_mut!(bot_pvstype).as_mut(),
        "bot_pvstype",
        "1",
        CVAR_CHEAT,
    );
    //#ifndef FINAL_BUILD (compiled in dev build)
    trap::Cvar_Register(
        addr_of_mut!(bot_getinthecarrr).as_mut(),
        "bot_getinthecarrr",
        "0",
        0,
    );

    //#ifdef _DEBUG bot_nogoals/bot_debugmessages omitted (_DEBUG off in release server build)

    trap::Cvar_Register(
        addr_of_mut!(bot_attachments).as_mut(),
        "bot_attachments",
        "1",
        0,
    );
    trap::Cvar_Register(addr_of_mut!(bot_camp).as_mut(), "bot_camp", "1", 0);

    trap::Cvar_Register(addr_of_mut!(bot_wp_info).as_mut(), "bot_wp_info", "1", 0);
    trap::Cvar_Register(
        addr_of_mut!(bot_wp_edit).as_mut(),
        "bot_wp_edit",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        addr_of_mut!(bot_wp_clearweight).as_mut(),
        "bot_wp_clearweight",
        "1",
        0,
    );
    trap::Cvar_Register(
        addr_of_mut!(bot_wp_distconnect).as_mut(),
        "bot_wp_distconnect",
        "1",
        0,
    );
    trap::Cvar_Register(
        addr_of_mut!(bot_wp_visconnect).as_mut(),
        "bot_wp_visconnect",
        "1",
        0,
    );

    trap::Cvar_Update(&mut *addr_of_mut!(bot_forcepowers));
    //end rww

    //if the game is restarted for a tournament
    if restart != 0 {
        return QTRUE;
    }

    //initialize the bot states
    write_bytes(addr_of_mut!(botstates), 0, 1);

    if trap::BotLibSetup() == 0 {
        return QFALSE; //wts?!
    }

    QTRUE
}

/// `void MeleeCombatHandling(bot_state_t *bs)` (ai_main.c:4245) — close-range (fists/stun-baton)
/// combat movement. Drops vertical traces under the enemy, the bot, and the midpoint between
/// them; if all three rest on the same floor height (i.e. flat ground, no ledge between us),
/// it sets the goal position onto the enemy so the bot closes in. Also flips a strafe direction
/// on a randomized timer.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn MeleeCombatHandling(bs: *mut bot_state_t) {
    let mut usethisvec: vec3_t = vec3_origin;
    let mut downvec: vec3_t = vec3_origin;
    let mut midorg: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;
    let mut fwd: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;
    let en_down: c_int;
    let me_down: c_int;
    let mid_down: c_int;

    if (*bs).currentEnemy.is_null() {
        return;
    }

    if !(*(*bs).currentEnemy).client.is_null() {
        VectorCopy(
            &(*(*(*bs).currentEnemy).client).ps.origin,
            &mut usethisvec,
        );
    } else {
        VectorCopy(&(*(*bs).currentEnemy).s.origin, &mut usethisvec);
    }

    if (*bs).meleeStrafeTime < (*addr_of!(level)).time as f32 {
        if (*bs).meleeStrafeDir != 0 {
            (*bs).meleeStrafeDir = 0;
        } else {
            (*bs).meleeStrafeDir = 1;
        }

        (*bs).meleeStrafeTime = ((*addr_of!(level)).time + Q_irand(500, 1800)) as f32;
    }

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -24.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 32.0;

    VectorCopy(&usethisvec, &mut downvec);
    downvec[2] -= 4096.0;

    let tr = trap::Trace(&usethisvec, &mins, &maxs, &downvec, -1, MASK_SOLID);

    en_down = tr.endpos[2] as c_int;

    VectorCopy(&(*bs).origin, &mut downvec);
    downvec[2] -= 4096.0;

    let tr = trap::Trace(&(*bs).origin, &mins, &maxs, &downvec, -1, MASK_SOLID);

    me_down = tr.endpos[2] as c_int;

    VectorSubtract(&usethisvec, &(*bs).origin, &mut a);
    let a_copy = a;
    vectoangles(&a_copy, &mut a);
    AngleVectors(&a, Some(&mut fwd), None, None);

    midorg[0] = (*bs).origin[0] + fwd[0] * (*bs).frame_Enemy_Len / 2.0;
    midorg[1] = (*bs).origin[1] + fwd[1] * (*bs).frame_Enemy_Len / 2.0;
    midorg[2] = (*bs).origin[2] + fwd[2] * (*bs).frame_Enemy_Len / 2.0;

    VectorCopy(&midorg, &mut downvec);
    downvec[2] -= 4096.0;

    let tr = trap::Trace(&midorg, &mins, &maxs, &downvec, -1, MASK_SOLID);

    mid_down = tr.endpos[2] as c_int;

    if me_down == en_down && en_down == mid_down {
        VectorCopy(&usethisvec, &mut (*bs).goalPosition);
    }
}

/// `void SaberCombatHandling(bot_state_t *bs)` (ai_main.c:4330) — saber-combat movement and
/// defend/attack decision logic. Like [`MeleeCombatHandling`] it floor-traces the enemy, self,
/// and midpoint to confirm flat ground, then tunes the bot's goal position, jump timing, saber
/// defense toggling, and back-off behavior based on engagement distance and the enemy's saber
/// move. Simple, but it works.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn SaberCombatHandling(bs: *mut bot_state_t) {
    let mut usethisvec: vec3_t = vec3_origin;
    let mut downvec: vec3_t = vec3_origin;
    let mut midorg: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;
    let mut fwd: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;
    let mut en_down: c_int;
    let mut me_down: c_int;
    let mid_down: c_int;

    if (*bs).currentEnemy.is_null() {
        return;
    }

    if !(*(*bs).currentEnemy).client.is_null() {
        VectorCopy(
            &(*(*(*bs).currentEnemy).client).ps.origin,
            &mut usethisvec,
        );
    } else {
        VectorCopy(&(*(*bs).currentEnemy).s.origin, &mut usethisvec);
    }

    if (*bs).meleeStrafeTime < (*addr_of!(level)).time as f32 {
        if (*bs).meleeStrafeDir != 0 {
            (*bs).meleeStrafeDir = 0;
        } else {
            (*bs).meleeStrafeDir = 1;
        }

        (*bs).meleeStrafeTime = ((*addr_of!(level)).time + Q_irand(500, 1800)) as f32;
    }

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -24.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 32.0;

    VectorCopy(&usethisvec, &mut downvec);
    downvec[2] -= 4096.0;

    let tr = trap::Trace(&usethisvec, &mins, &maxs, &downvec, -1, MASK_SOLID);

    en_down = tr.endpos[2] as c_int;

    if tr.startsolid != 0 || tr.allsolid != 0 {
        en_down = 1;
        me_down = 2;
    } else {
        VectorCopy(&(*bs).origin, &mut downvec);
        downvec[2] -= 4096.0;

        let tr = trap::Trace(&(*bs).origin, &mins, &maxs, &downvec, -1, MASK_SOLID);

        me_down = tr.endpos[2] as c_int;

        if tr.startsolid != 0 || tr.allsolid != 0 {
            en_down = 1;
            me_down = 2;
        }
    }

    VectorSubtract(&usethisvec, &(*bs).origin, &mut a);
    let a_copy = a;
    vectoangles(&a_copy, &mut a);
    AngleVectors(&a, Some(&mut fwd), None, None);

    midorg[0] = (*bs).origin[0] + fwd[0] * (*bs).frame_Enemy_Len / 2.0;
    midorg[1] = (*bs).origin[1] + fwd[1] * (*bs).frame_Enemy_Len / 2.0;
    midorg[2] = (*bs).origin[2] + fwd[2] * (*bs).frame_Enemy_Len / 2.0;

    VectorCopy(&midorg, &mut downvec);
    downvec[2] -= 4096.0;

    let tr = trap::Trace(&midorg, &mins, &maxs, &downvec, -1, MASK_SOLID);

    mid_down = tr.endpos[2] as c_int;

    if me_down == en_down && en_down == mid_down {
        if usethisvec[2] > ((*bs).origin[2] + 32.0)
            && !(*(*bs).currentEnemy).client.is_null()
            && (*(*(*bs).currentEnemy).client).ps.groundEntityNum == ENTITYNUM_NONE
        {
            (*bs).jumpTime = ((*addr_of!(level)).time + 100) as f32;
        }

        if (*bs).frame_Enemy_Len > 128.0 {
            //be ready to attack
            (*bs).saberDefending = 0;
            (*bs).saberDefendDecideTime = (*addr_of!(level)).time + Q_irand(1000, 2000);
        } else if (*bs).saberDefendDecideTime < (*addr_of!(level)).time {
            if (*bs).saberDefending != 0 {
                (*bs).saberDefending = 0;
            } else {
                (*bs).saberDefending = 1;
            }

            (*bs).saberDefendDecideTime = (*addr_of!(level)).time + Q_irand(500, 2000);
        }

        if (*bs).frame_Enemy_Len < 54.0 {
            VectorCopy(&(*bs).origin, &mut (*bs).goalPosition);
            (*bs).saberBFTime = 0;
        } else {
            VectorCopy(&usethisvec, &mut (*bs).goalPosition);
        }

        if !(*bs).currentEnemy.is_null() && !(*(*bs).currentEnemy).client.is_null() {
            let ecl = (*(*bs).currentEnemy).client;
            if BG_SaberInSpecial((*ecl).ps.saberMove) == QFALSE
                && (*bs).frame_Enemy_Len > 90.0
                && (*bs).saberBFTime > (*addr_of!(level)).time
                && (*bs).saberBTime > (*addr_of!(level)).time
                && (*bs).beStill < (*addr_of!(level)).time as f32
                && (*bs).saberSTime < (*addr_of!(level)).time
            {
                (*bs).beStill = ((*addr_of!(level)).time + Q_irand(500, 1000)) as f32;
                (*bs).saberSTime = (*addr_of!(level)).time + Q_irand(1200, 1800);
            } else if (*ecl).ps.weapon == WP_SABER
                && (*bs).frame_Enemy_Len < 80.0
                && (Q_irand(1, 10) < 8 && (*bs).saberBFTime < (*addr_of!(level)).time)
                || (*bs).saberBTime > (*addr_of!(level)).time
                || BG_SaberInKata((*ecl).ps.saberMove) != QFALSE
                || (*ecl).ps.saberMove == LS_SPINATTACK
                || (*ecl).ps.saberMove == LS_SPINATTACK_DUAL
            {
                let mut vs: vec3_t = vec3_origin;
                let mut groundcheck: vec3_t = vec3_origin;
                let idealDist: c_int;
                let mut checkIncr: c_int = 0;

                VectorSubtract(&(*bs).origin, &usethisvec, &mut vs);
                VectorNormalize(&mut vs);

                if BG_SaberInKata((*ecl).ps.saberMove) != QFALSE
                    || (*ecl).ps.saberMove == LS_SPINATTACK
                    || (*ecl).ps.saberMove == LS_SPINATTACK_DUAL
                {
                    idealDist = 256;
                } else {
                    idealDist = 64;
                }

                while checkIncr < idealDist {
                    (*bs).goalPosition[0] = (*bs).origin[0] + vs[0] * checkIncr as f32;
                    (*bs).goalPosition[1] = (*bs).origin[1] + vs[1] * checkIncr as f32;
                    (*bs).goalPosition[2] = (*bs).origin[2] + vs[2] * checkIncr as f32;

                    if (*bs).saberBTime < (*addr_of!(level)).time {
                        (*bs).saberBFTime = (*addr_of!(level)).time + Q_irand(900, 1300);
                        (*bs).saberBTime = (*addr_of!(level)).time + Q_irand(300, 700);
                    }

                    VectorCopy(&(*bs).goalPosition, &mut groundcheck);

                    groundcheck[2] -= 64.0;

                    let tr = trap::Trace(
                        &(*bs).goalPosition,
                        &vec3_origin,
                        &vec3_origin,
                        &groundcheck,
                        (*bs).client,
                        MASK_SOLID,
                    );

                    if tr.fraction == 1.0 {
                        //don't back off of a ledge
                        VectorCopy(&usethisvec, &mut (*bs).goalPosition);
                        break;
                    }
                    checkIncr += 64;
                }
            } else if (*ecl).ps.weapon == WP_SABER && (*bs).frame_Enemy_Len >= 75.0 {
                (*bs).saberBFTime = (*addr_of!(level)).time + Q_irand(700, 1300);
                (*bs).saberBTime = 0;
            }
        }

        /*AngleVectors(bs->viewangles, NULL, fwd, NULL);

        if (bs->meleeStrafeDir)
        {
            bs->goalPosition[0] += fwd[0]*16;
            bs->goalPosition[1] += fwd[1]*16;
            bs->goalPosition[2] += fwd[2]*16;
        }
        else
        {
            bs->goalPosition[0] -= fwd[0]*16;
            bs->goalPosition[1] -= fwd[1]*16;
            bs->goalPosition[2] -= fwd[2]*16;
        }*/
    } else if (*bs).frame_Enemy_Len <= 56.0 {
        (*bs).doAttack = 1;
        (*bs).saberDefending = 0;
    }
}

/// `void BotDoTeamplayAI(bot_state_t *bs)` (ai_main.c:4119) — teamplay-FFA squad follower logic
/// for a non-leader bot. Adopts any player-forced teamplay state, and on a regroup order drops
/// its current squad leader so it scans for a new one next frame.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotDoTeamplayAI(bs: *mut bot_state_t) {
    if (*bs).state_Forced != 0 {
        (*bs).teamplayState = (*bs).state_Forced;
    }

    if (*bs).teamplayState == TEAMPLAYSTATE_REGROUP {
        //force to find a new leader
        (*bs).squadLeader = null_mut();
        (*bs).isSquadLeader = 0;
    }
}

/// `void CommanderBotCTFAI(bot_state_t *bs)` (ai_main.c:3887) — squad-leader command logic for
/// CTF. Surveys the field (who holds which flag, team sizes, current attacker/defender split),
/// gathers this leader's squad, then assigns each squad member a CTF role
/// (attack/retrieve/defend/guard) by alternating priorities, never overriding a member already
/// bringing the flag home.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
// faithful: C computes `numOnEnemyTeam` and `numDefenders` but never reads them (dead stores).
#[allow(unused_assignments, unused_variables)]
pub unsafe fn CommanderBotCTFAI(bs: *mut bot_state_t) {
    let mut i: c_int = 0;
    let mut ent: *mut gentity_t;
    let mut squadmates: c_int = 0;
    let mut squad: [*mut gentity_t; MAX_CLIENTS] = [null_mut(); MAX_CLIENTS];
    let mut defendAttackPriority: c_int = 0; //0 == attack, 1 == defend
    let mut guardDefendPriority: c_int = 0; //0 == defend, 1 == guard
    let mut attackRetrievePriority: c_int = 0; //0 == retrieve, 1 == attack
    let myFlag: c_int;
    let enemyFlag: c_int;
    let mut enemyHasOurFlag: c_int = 0;
    let mut weHaveEnemyFlag: c_int = 0;
    let mut numOnMyTeam: c_int = 0;
    let mut numOnEnemyTeam: c_int = 0;
    let mut numAttackers: c_int = 0;
    let mut numDefenders: c_int = 0;

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        myFlag = PW_REDFLAG;
    } else {
        myFlag = PW_BLUEFLAG;
    }

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        enemyFlag = PW_BLUEFLAG;
    } else {
        enemyFlag = PW_REDFLAG;
    }

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null() && !(*ent).client.is_null() {
            if (*(*ent).client).ps.powerups[enemyFlag as usize] != 0
                && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE
            {
                weHaveEnemyFlag = 1;
            } else if (*(*ent).client).ps.powerups[myFlag as usize] != 0
                && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) == QFALSE
            {
                enemyHasOurFlag = 1;
            }

            if OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE {
                numOnMyTeam += 1;
            } else {
                numOnEnemyTeam += 1;
            }

            if !(*addr_of!(botstates))[(*ent).s.number as usize].is_null() {
                if (*(*addr_of!(botstates))[(*ent).s.number as usize]).ctfState == CTFSTATE_ATTACKER
                    || (*(*addr_of!(botstates))[(*ent).s.number as usize]).ctfState
                        == CTFSTATE_RETRIEVAL
                {
                    numAttackers += 1;
                } else {
                    numDefenders += 1;
                }
            } else {
                //assume real players to be attackers in our logic
                numAttackers += 1;
            }
        }
        i += 1;
    }

    i = 0;

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && !(*addr_of!(botstates))[i as usize].is_null()
            && !(*(*addr_of!(botstates))[i as usize]).squadLeader.is_null()
            && (*(*(*addr_of!(botstates))[i as usize]).squadLeader).s.number == (*bs).client
            && i != (*bs).client
        {
            squad[squadmates as usize] = ent;
            squadmates += 1;
        }

        i += 1;
    }

    squad[squadmates as usize] = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize);
    squadmates += 1;

    i = 0;

    if enemyHasOurFlag != 0 && weHaveEnemyFlag == 0 {
        //start off with an attacker instead of a retriever if we don't have the enemy flag yet so that they can't capture it first.
        //after that we focus on getting our flag back.
        attackRetrievePriority = 1;
    }

    while i < squadmates {
        if !squad[i as usize].is_null()
            && !(*squad[i as usize]).client.is_null()
            && !(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize].is_null()
        {
            if (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize]).ctfState
                != CTFSTATE_GETFLAGHOME
            {
                //never tell a bot to stop trying to bring the flag to the base
                if defendAttackPriority != 0 {
                    if weHaveEnemyFlag != 0 {
                        if guardDefendPriority != 0 {
                            (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize])
                                .ctfState = CTFSTATE_GUARDCARRIER;
                            guardDefendPriority = 0;
                        } else {
                            (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize])
                                .ctfState = CTFSTATE_DEFENDER;
                            guardDefendPriority = 1;
                        }
                    } else {
                        (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize]).ctfState =
                            CTFSTATE_DEFENDER;
                    }
                    defendAttackPriority = 0;
                } else {
                    if enemyHasOurFlag != 0 {
                        if attackRetrievePriority != 0 {
                            (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize])
                                .ctfState = CTFSTATE_ATTACKER;
                            attackRetrievePriority = 0;
                        } else {
                            (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize])
                                .ctfState = CTFSTATE_RETRIEVAL;
                            attackRetrievePriority = 1;
                        }
                    } else {
                        (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize]).ctfState =
                            CTFSTATE_ATTACKER;
                    }
                    defendAttackPriority = 1;
                }
            } else if (numOnMyTeam < 2 || numAttackers == 0) && enemyHasOurFlag != 0 {
                //I'm the only one on my team who will attack and the enemy has my flag, I have to go after him
                (*(*addr_of!(botstates))[(*squad[i as usize]).s.number as usize]).ctfState =
                    CTFSTATE_RETRIEVAL;
            }
        }

        i += 1;
    }
}

/// `void CommanderBotSiegeAI(bot_state_t *bs)` (ai_main.c:4053) — squad-leader command logic for
/// Siege (similar to CTF). Gathers same-team bots that are not squad leaders and not already
/// player-forced into a squad (and counts the already-commanded ones), then orders up to half
/// the team to mirror this leader's `siegeState`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn CommanderBotSiegeAI(bs: *mut bot_state_t) {
    let mut i: c_int = 0;
    let mut squadmates: c_int = 0;
    let mut commanded: c_int = 0;
    let mut teammates: c_int = 0;
    let mut squad: [*mut gentity_t; MAX_CLIENTS] = [null_mut(); MAX_CLIENTS];
    let mut ent: *mut gentity_t;
    let mut bst: *mut bot_state_t;

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE
            && !(*addr_of!(botstates))[(*ent).s.number as usize].is_null()
        {
            bst = (*addr_of!(botstates))[(*ent).s.number as usize];

            if !bst.is_null() && (*bst).isSquadLeader == 0 && (*bst).state_Forced == 0 {
                squad[squadmates as usize] = ent;
                squadmates += 1;
            } else if !bst.is_null() && (*bst).isSquadLeader == 0 && (*bst).state_Forced != 0 {
                //count them as commanded
                commanded += 1;
            }
        }

        if !ent.is_null()
            && !(*ent).client.is_null()
            && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE
        {
            teammates += 1;
        }

        i += 1;
    }

    if squadmates == 0 {
        return;
    }

    //tell squad mates to do what I'm doing, up to half of team, let the other half make their own decisions
    i = 0;

    while i < squadmates && !squad[i as usize].is_null() {
        bst = (*addr_of!(botstates))[(*squad[i as usize]).s.number as usize];

        if commanded > teammates / 2 {
            break;
        }

        if !bst.is_null() {
            (*bst).state_Forced = (*bs).siegeState;
            (*bst).siegeState = (*bs).siegeState;
            commanded += 1;
        }

        i += 1;
    }
}

/// `void CommanderBotTeamplayAI(bot_state_t *bs)` (ai_main.c:4134) — squad-leader command logic
/// for team FFA. Like the CTF and Siege commanders, it instructs the squad: dispatch a member to
/// assist whichever teammate is hurt worst, recall assisters once nobody needs help, and
/// periodically order a regroup (relinquishing leadership) for variation.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
// faithful: C counts `teammates` here but never reads it (dead store).
#[allow(unused_assignments, unused_variables)]
pub unsafe fn CommanderBotTeamplayAI(bs: *mut bot_state_t) {
    let mut i: c_int = 0;
    let mut squadmates: c_int = 0;
    let mut teammates: c_int = 0;
    let mut teammate_indanger: c_int = -1;
    let mut teammate_helped: c_int = 0;
    let mut foundsquadleader: c_int = 0;
    let mut worsthealth: c_int = 50;
    let mut squad: [*mut gentity_t; MAX_CLIENTS] = [null_mut(); MAX_CLIENTS];
    let mut ent: *mut gentity_t;
    let mut bst: *mut bot_state_t;

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE
            && !(*addr_of!(botstates))[(*ent).s.number as usize].is_null()
        {
            bst = (*addr_of!(botstates))[(*ent).s.number as usize];

            if foundsquadleader != 0 && !bst.is_null() && (*bst).isSquadLeader != 0 {
                //never more than one squad leader
                (*bst).isSquadLeader = 0;
            }

            if !bst.is_null() && (*bst).isSquadLeader == 0 {
                squad[squadmates as usize] = ent;
                squadmates += 1;
            } else if !bst.is_null() {
                foundsquadleader = 1;
            }
        }

        if !ent.is_null()
            && !(*ent).client.is_null()
            && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE
        {
            teammates += 1;

            if (*ent).health < worsthealth {
                teammate_indanger = (*ent).s.number;
                worsthealth = (*ent).health;
            }
        }

        i += 1;
    }

    if squadmates == 0 {
        return;
    }

    i = 0;

    while i < squadmates && !squad[i as usize].is_null() {
        bst = (*addr_of!(botstates))[(*squad[i as usize]).s.number as usize];

        if !bst.is_null() && (*bst).state_Forced == 0 {
            //only order if this guy is not being ordered directly by the real player team leader
            if teammate_indanger >= 0 && teammate_helped == 0 {
                //send someone out to help whoever needs help most at the moment
                (*bst).teamplayState = TEAMPLAYSTATE_ASSISTING;
                (*bst).squadLeader = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(teammate_indanger as usize);
                teammate_helped = 1;
            } else if (teammate_indanger == -1 || teammate_helped != 0)
                && (*bst).teamplayState == TEAMPLAYSTATE_ASSISTING
            {
                //no teammates need help badly, but this guy is trying to help them anyway, so stop
                (*bst).teamplayState = TEAMPLAYSTATE_FOLLOWING;
                (*bst).squadLeader = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize);
            }

            if (*bs).squadRegroupInterval < (*addr_of!(level)).time && Q_irand(1, 10) < 5 {
                //every so often tell the squad to regroup for the sake of variation
                if (*bst).teamplayState == TEAMPLAYSTATE_FOLLOWING {
                    (*bst).teamplayState = TEAMPLAYSTATE_REGROUP;
                }

                (*bs).isSquadLeader = 0;
                (*bs).squadCannotLead = (*addr_of!(level)).time + 500;
                (*bs).squadRegroupInterval = (*addr_of!(level)).time + Q_irand(45000, 65000);
            }
        }

        i += 1;
    }
}

/// `void CommanderBotAI(bot_state_t *bs)` (ai_main.c:4228) — pick which commander AI to run for a
/// squad leader based on the current gametype (CTF/CTY, Siege, or Team FFA).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn CommanderBotAI(bs: *mut bot_state_t) {
    if (*addr_of!(g_gametype)).integer == GT_CTF || (*addr_of!(g_gametype)).integer == GT_CTY {
        CommanderBotCTFAI(bs);
    } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        CommanderBotSiegeAI(bs);
    } else if (*addr_of!(g_gametype)).integer == GT_TEAM {
        CommanderBotTeamplayAI(bs);
    }
}

/// `int CombatBotAI(bot_state_t *bs, float thinktime)` (ai_main.c:4861) — the combat firing
/// decision: given the current enemy and weapon, decide whether to attack (and whether primary
/// or alt fire), accounting for saber/melee attack ranges, the field-of-vision cone (widened for
/// charging weapons and close range), thermal/rocket charge handling, and the secondary-fire
/// preference from [`ShouldSecondaryFire`]. Returns `1` if a charge was released this frame.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn CombatBotAI(bs: *mut bot_state_t, _thinktime: f32) -> c_int {
    let mut eorg: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;
    let secFire: c_int;
    let mut fovcheck: f32;

    if (*bs).currentEnemy.is_null() {
        return 0;
    }

    if !(*(*bs).currentEnemy).client.is_null() {
        VectorCopy(&(*(*(*bs).currentEnemy).client).ps.origin, &mut eorg);
    } else {
        VectorCopy(&(*(*bs).currentEnemy).s.origin, &mut eorg);
    }

    VectorSubtract(&eorg, &(*bs).eye, &mut a);
    let a_copy = a;
    vectoangles(&a_copy, &mut a);

    if BotGetWeaponRange(bs) == BWEAPONRANGE_SABER {
        if (*bs).frame_Enemy_Len <= SABER_ATTACK_RANGE as f32 {
            (*bs).doAttack = 1;
        }
    } else if BotGetWeaponRange(bs) == BWEAPONRANGE_MELEE {
        if (*bs).frame_Enemy_Len <= MELEE_ATTACK_RANGE as f32 {
            (*bs).doAttack = 1;
        }
    } else {
        if (*bs).cur_ps.weapon == WP_THERMAL || (*bs).cur_ps.weapon == WP_ROCKET_LAUNCHER {
            //be careful with the hurty weapons
            fovcheck = 40.0;

            if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT
                && (*bs).cur_ps.weapon == WP_ROCKET_LAUNCHER
            {
                //if we're charging the weapon up then we can hold fire down within a normal fov
                fovcheck = 60.0;
            }
        } else {
            fovcheck = 60.0;
        }

        if (*bs).cur_ps.weaponstate == WEAPON_CHARGING
            || (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT
        {
            fovcheck = 160.0;
        }

        if (*bs).frame_Enemy_Len < 128.0 {
            fovcheck *= 2.0;
        }

        if InFieldOfVision(&(*bs).viewangles, fovcheck, &mut a) != 0 {
            if (*bs).cur_ps.weapon == WP_THERMAL {
                if ((((*addr_of!(level)).time - (*bs).cur_ps.weaponChargeTime) as f32)
                    < ((*bs).frame_Enemy_Len * 2.0)
                    && ((*addr_of!(level)).time - (*bs).cur_ps.weaponChargeTime) < 4000
                    && (*bs).frame_Enemy_Len > 64.0)
                    || ((*bs).cur_ps.weaponstate != WEAPON_CHARGING
                        && (*bs).cur_ps.weaponstate != WEAPON_CHARGING_ALT)
                {
                    if (*bs).cur_ps.weaponstate != WEAPON_CHARGING
                        && (*bs).cur_ps.weaponstate != WEAPON_CHARGING_ALT
                    {
                        if (*bs).frame_Enemy_Len > 512.0 && (*bs).frame_Enemy_Len < 800.0 {
                            (*bs).doAltAttack = 1;
                            //bs->doAttack = 1;
                        } else {
                            (*bs).doAttack = 1;
                            //bs->doAltAttack = 1;
                        }
                    }

                    if (*bs).cur_ps.weaponstate == WEAPON_CHARGING {
                        (*bs).doAttack = 1;
                    } else if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT {
                        (*bs).doAltAttack = 1;
                    }
                }
            } else {
                secFire = ShouldSecondaryFire(bs);

                if (*bs).cur_ps.weaponstate != WEAPON_CHARGING_ALT
                    && (*bs).cur_ps.weaponstate != WEAPON_CHARGING
                {
                    (*bs).altChargeTime = Q_irand(500, 1000);
                }

                if secFire == 1 {
                    (*bs).doAltAttack = 1;
                } else if secFire == 0 {
                    if (*bs).cur_ps.weapon != WP_THERMAL {
                        if (*bs).cur_ps.weaponstate != WEAPON_CHARGING
                            || (*bs).altChargeTime
                                > ((*addr_of!(level)).time - (*bs).cur_ps.weaponChargeTime)
                        {
                            (*bs).doAttack = 1;
                        }
                    } else {
                        (*bs).doAttack = 1;
                    }
                }

                if secFire == 2 {
                    //released a charge
                    return 1;
                }
            }
        }
    }

    0
}

/// `int BotAI_GetSnapshotEntity(int clientNum, int sequence, entityState_t *state)`
/// (ai_main.c:392) — fetch the entity at snapshot slot `sequence` for `clientNum`. If the
/// engine reports no entity (`-1`), zero `state` and return `-1`; otherwise fill it via
/// [`BotAI_GetEntityState`] and return the next sequence index.
///
/// # Safety
/// `state` must point at a valid [`entityState_t`].
pub unsafe fn BotAI_GetSnapshotEntity(
    clientNum: c_int,
    sequence: c_int,
    state: *mut entityState_t,
) -> c_int {
    let entNum = trap::BotGetSnapshotEntity(clientNum, sequence);
    if entNum == -1 {
        write_bytes(state, 0, 1);
        return -1;
    }

    BotAI_GetEntityState(entNum, state);

    sequence + 1
}

/// `void BotChangeViewAngles(bot_state_t *bs, float thinktime)` (ai_main.c:474) — slew the bot's
/// `viewangles` toward `ideal_viewangles` at a skill/turnspeed-scaled rate, clamped to
/// `skills.maxturn * thinktime`, then push the result to the engine via `trap_EA_View`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotChangeViewAngles(bs: *mut bot_state_t, thinktime: f32) {
    let mut diff: f32;
    let mut factor: f32;
    let mut maxchange: f32;
    let mut anglespeed: f32;
    let mut disired_speed: f32;
    let mut i: usize;

    if (*bs).ideal_viewangles[PITCH] > 180.0 {
        (*bs).ideal_viewangles[PITCH] -= 360.0;
    }

    if !(*bs).currentEnemy.is_null() && (*bs).frame_Enemy_Vis != 0 {
        if (*bs).settings.skill <= 1.0 {
            factor = ((*bs).skills.turnspeed_combat * 0.4) * (*bs).settings.skill;
        } else if (*bs).settings.skill <= 2.0 {
            factor = ((*bs).skills.turnspeed_combat * 0.6) * (*bs).settings.skill;
        } else if (*bs).settings.skill <= 3.0 {
            factor = ((*bs).skills.turnspeed_combat * 0.8) * (*bs).settings.skill;
        } else {
            factor = (*bs).skills.turnspeed_combat * (*bs).settings.skill;
        }
    } else {
        factor = (*bs).skills.turnspeed;
    }

    if factor > 1.0 {
        factor = 1.0;
    }
    if factor < 0.001 {
        factor = 0.001;
    }

    maxchange = (*bs).skills.maxturn;

    //if (maxchange < 240) maxchange = 240;
    maxchange *= thinktime;
    i = 0;
    while i < 2 {
        (*bs).viewangles[i] = AngleMod((*bs).viewangles[i]);
        (*bs).ideal_viewangles[i] = AngleMod((*bs).ideal_viewangles[i]);
        diff = AngleDifference((*bs).viewangles[i], (*bs).ideal_viewangles[i]);
        disired_speed = diff * factor;
        (*bs).viewanglespeed[i] += (*bs).viewanglespeed[i] - disired_speed;
        if (*bs).viewanglespeed[i] > 180.0 {
            (*bs).viewanglespeed[i] = maxchange;
        }
        if (*bs).viewanglespeed[i] < -180.0 {
            (*bs).viewanglespeed[i] = -maxchange;
        }
        anglespeed = (*bs).viewanglespeed[i];
        if anglespeed > maxchange {
            anglespeed = maxchange;
        }
        if anglespeed < -maxchange {
            anglespeed = -maxchange;
        }
        (*bs).viewangles[i] += anglespeed;
        (*bs).viewangles[i] = AngleMod((*bs).viewangles[i]);
        (*bs).viewanglespeed[i] *= (0.45 * (1.0 - factor) as f64) as f32;

        i += 1;
    }
    if (*bs).viewangles[PITCH] > 180.0 {
        (*bs).viewangles[PITCH] -= 360.0;
    }
    trap::ea::EA_View((*bs).client, &(*bs).viewangles);
}

/// `void BotInputToUserCommand(bot_input_t *bi, usercmd_t *ucmd, int delta_angles[3], int time,
/// int useTime)` (ai_main.c:541) — translate the bot's `bot_input_t` into a `usercmd_t`: map the
/// `ACTION_*` flags to button bits, fold view angles through `ANGLE2SHORT` minus the delta
/// angles, and project the movement direction onto the (pitch-flattened) forward/right vectors.
///
/// The `#if 0` TA-button block (ai_main.c:572-582 — `ACTION_AFFIRMATIVE`/`GETFLAG`/etc.) is
/// omitted, matching the disabled C; those `ACTION_*` flags are intentionally undefined.
///
/// # Safety
/// `bi`/`ucmd` must be valid pointers; `delta_angles` must point at 3 ints.
pub unsafe fn BotInputToUserCommand(
    bi: *mut bot_input_t,
    ucmd: *mut usercmd_t,
    delta_angles: *const c_int,
    time: c_int,
    useTime: c_int,
) {
    let mut angles: vec3_t = vec3_origin;
    let mut forward: vec3_t = vec3_origin;
    let mut right: vec3_t = vec3_origin;
    let mut temp: i16;
    let mut j: usize;

    //clear the whole structure
    write_bytes(ucmd, 0, 1);
    //
    //the duration for the user command in milli seconds
    (*ucmd).serverTime = time;
    //
    if (*bi).actionflags & ACTION_DELAYEDJUMP != 0 {
        (*bi).actionflags |= ACTION_JUMP;
        (*bi).actionflags &= !ACTION_DELAYEDJUMP;
    }
    //set the buttons
    if (*bi).actionflags & ACTION_RESPAWN != 0 {
        (*ucmd).buttons = BUTTON_ATTACK;
    }
    if (*bi).actionflags & ACTION_ATTACK != 0 {
        (*ucmd).buttons |= BUTTON_ATTACK;
    }
    if (*bi).actionflags & ACTION_ALT_ATTACK != 0 {
        (*ucmd).buttons |= BUTTON_ALT_ATTACK;
    }
    //	if (bi->actionflags & ACTION_TALK) ucmd->buttons |= BUTTON_TALK;
    if (*bi).actionflags & ACTION_GESTURE != 0 {
        (*ucmd).buttons |= BUTTON_GESTURE;
    }
    if (*bi).actionflags & ACTION_USE != 0 {
        (*ucmd).buttons |= BUTTON_USE_HOLDABLE;
    }
    if (*bi).actionflags & ACTION_WALK != 0 {
        (*ucmd).buttons |= BUTTON_WALKING;
    }

    if (*bi).actionflags & ACTION_FORCEPOWER != 0 {
        (*ucmd).buttons |= BUTTON_FORCEPOWER;
    }

    if useTime < (*addr_of!(level)).time && Q_irand(1, 10) < 5 {
        //for now just hit use randomly in case there's something useable around
        (*ucmd).buttons |= BUTTON_USE;
    }

    if (*bi).weapon == WP_NONE {
        (*bi).weapon = WP_BRYAR_PISTOL;
    }

    //
    (*ucmd).weapon = (*bi).weapon as u8;
    //set the view angles
    //NOTE: the ucmd->angles are the angles WITHOUT the delta angles
    (*ucmd).angles[PITCH] = ANGLE2SHORT((*bi).viewangles[PITCH]);
    (*ucmd).angles[YAW] = ANGLE2SHORT((*bi).viewangles[YAW]);
    (*ucmd).angles[ROLL] = ANGLE2SHORT((*bi).viewangles[ROLL]);
    //subtract the delta angles
    j = 0;
    while j < 3 {
        temp = ((*ucmd).angles[j] - *delta_angles.add(j)) as i16;
        (*ucmd).angles[j] = temp as c_int;
        j += 1;
    }
    //NOTE: movement is relative to the REAL view angles
    //get the horizontal forward and right vector
    //get the pitch in the range [-180, 180]
    if (*bi).dir[2] != 0.0 {
        angles[PITCH] = (*bi).viewangles[PITCH];
    } else {
        angles[PITCH] = 0.0;
    }
    angles[YAW] = (*bi).viewangles[YAW];
    angles[ROLL] = 0.0;
    AngleVectors(&angles, Some(&mut forward), Some(&mut right), None);
    //bot input speed is in the range [0, 400]
    (*bi).speed = (*bi).speed * 127.0 / 400.0;
    //set the view independent movement
    (*ucmd).forwardmove = (DotProduct(&forward, &(*bi).dir) * (*bi).speed) as i8;
    (*ucmd).rightmove = (DotProduct(&right, &(*bi).dir) * (*bi).speed) as i8;
    (*ucmd).upmove = (((forward[2] as c_int).abs() as f32) * (*bi).dir[2] * (*bi).speed) as i8;
    //normal keyboard movement
    if (*bi).actionflags & ACTION_MOVEFORWARD != 0 {
        (*ucmd).forwardmove = (*ucmd).forwardmove.wrapping_add(127);
    }
    if (*bi).actionflags & ACTION_MOVEBACK != 0 {
        (*ucmd).forwardmove = (*ucmd).forwardmove.wrapping_sub(127);
    }
    if (*bi).actionflags & ACTION_MOVELEFT != 0 {
        (*ucmd).rightmove = (*ucmd).rightmove.wrapping_sub(127);
    }
    if (*bi).actionflags & ACTION_MOVERIGHT != 0 {
        (*ucmd).rightmove = (*ucmd).rightmove.wrapping_add(127);
    }
    //jump/moveup
    if (*bi).actionflags & ACTION_JUMP != 0 {
        (*ucmd).upmove = (*ucmd).upmove.wrapping_add(127);
    }
    //crouch/movedown
    if (*bi).actionflags & ACTION_CROUCH != 0 {
        (*ucmd).upmove = (*ucmd).upmove.wrapping_sub(127);
    }
}

/// `void BotUpdateInput(bot_state_t *bs, int time, int elapsed_time)` (ai_main.c:637) — drive one
/// frame of bot input: add the delta angles, slew view angles ([`BotChangeViewAngles`]), pull the
/// elementary-action input (`trap_EA_GetInput`), apply the respawn hack, convert to the bot's
/// `lastucmd` ([`BotInputToUserCommand`]), then subtract the delta angles back out.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotUpdateInput(bs: *mut bot_state_t, time: c_int, elapsed_time: c_int) {
    let mut bi: bot_input_t = bot_input_t::default();
    let mut j: usize;

    //add the delta angles to the bot's current view angles
    j = 0;
    while j < 3 {
        (*bs).viewangles[j] =
            AngleMod((*bs).viewangles[j] + SHORT2ANGLE((*bs).cur_ps.delta_angles[j]));
        j += 1;
    }
    //change the bot view angles
    BotChangeViewAngles(bs, elapsed_time as f32 / 1000.0);
    //retrieve the bot input
    trap::ea::EA_GetInput(
        (*bs).client,
        time as f32 / 1000.0,
        addr_of_mut!(bi) as *mut core::ffi::c_void,
    );
    //respawn hack
    if bi.actionflags & ACTION_RESPAWN != 0 {
        if (*bs).lastucmd.buttons & BUTTON_ATTACK != 0 {
            bi.actionflags &= !(ACTION_RESPAWN | ACTION_ATTACK);
        }
    }
    //convert the bot input to a usercmd
    BotInputToUserCommand(
        addr_of_mut!(bi),
        addr_of_mut!((*bs).lastucmd),
        (*bs).cur_ps.delta_angles.as_ptr(),
        time,
        (*bs).noUseTime,
    );
    //subtract the delta angles
    j = 0;
    while j < 3 {
        (*bs).viewangles[j] =
            AngleMod((*bs).viewangles[j] - SHORT2ANGLE((*bs).cur_ps.delta_angles[j]));
        j += 1;
    }
}

/// `void BotSelectWeapon(int client, int weapon)` (ai_main.c:157) — ask the engine to switch
/// `client` to `weapon` via `trap_EA_SelectWeapon`. `WP_NONE` (or lower) is a no-op.
pub fn BotSelectWeapon(client: c_int, weapon: c_int) {
    if weapon <= WP_NONE {
        //		assert(0);
        return;
    }
    trap::ea::EA_SelectWeapon(client, weapon);
}

/// `void BotReportStatus(bot_state_t *bs)` (ai_main.c:167) — bot says its current
/// team/siege/ctf state description to its team (`trap_EA_SayTeam`).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotReportStatus(bs: *mut bot_state_t) {
    if (*addr_of!(g_gametype)).integer == GT_TEAM {
        trap::ea::EA_SayTeam((*bs).client, teamplayStateDescriptions[(*bs).teamplayState as usize]);
    } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        trap::ea::EA_SayTeam((*bs).client, siegeStateDescriptions[(*bs).siegeState as usize]);
    } else if (*addr_of!(g_gametype)).integer == GT_CTF || (*addr_of!(g_gametype)).integer == GT_CTY
    {
        trap::ea::EA_SayTeam((*bs).client, ctfStateDescriptions[(*bs).ctfState as usize]);
    }
}

/// `void BotStraightTPOrderCheck(gentity_t *ent, int ordernum, bot_state_t *bs)` (ai_main.c:128) —
/// apply a teamplay order: clear/follow/assist set the squad leader and teamplay state; any other
/// order is stored verbatim as the teamplay state. The commented-out `BotDoChat` blocks are not
/// ported (chat is omitted from the module).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; `ent` a valid (or null) entity.
pub unsafe fn BotStraightTPOrderCheck(ent: *mut gentity_t, ordernum: c_int, bs: *mut bot_state_t) {
    match ordernum {
        0 => {
            if (*bs).squadLeader == ent {
                (*bs).teamplayState = 0;
                (*bs).squadLeader = null_mut();
            }
        }
        TEAMPLAYSTATE_FOLLOWING => {
            (*bs).teamplayState = ordernum;
            (*bs).isSquadLeader = 0;
            (*bs).squadLeader = ent;
            (*bs).wpDestSwitchTime = 0.0;
        }
        TEAMPLAYSTATE_ASSISTING => {
            (*bs).teamplayState = ordernum;
            (*bs).isSquadLeader = 0;
            (*bs).squadLeader = ent;
            (*bs).wpDestSwitchTime = 0.0;
        }
        _ => {
            (*bs).teamplayState = ordernum;
        }
    }
}

/// `void BotOrder(gentity_t *ent, int clientnum, int ordernum)` (ai_main.c:184) — a team-leader
/// player issues `ordernum` to bot `clientnum` (or every same-team bot when `clientnum == -1`),
/// validated against the per-gametype state range. `-1` ordernum requests a status report; any
/// other accepted order runs [`BotStraightTPOrderCheck`], stamps `state_Forced`, and issues an
/// `OrderAccepted` chat.
///
/// # Safety
/// `ent` must be valid (or null); reads the [`botstates`]/[`g_entities`] globals.
pub unsafe fn BotOrder(ent: *mut gentity_t, clientnum: c_int, ordernum: c_int) {
    let mut stateMin: c_int = 0;
    let mut stateMax: c_int = 0;
    let mut i: c_int = 0;

    if ent.is_null() || (*ent).client.is_null() || (*(*ent).client).sess.teamLeader == QFALSE {
        return;
    }

    if clientnum != -1 && botstates[clientnum as usize].is_null() {
        return;
    }

    if clientnum != -1
        && OnSameTeam(ent, (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(clientnum as usize)) == QFALSE
    {
        return;
    }

    if (*addr_of!(g_gametype)).integer != GT_CTF
        && (*addr_of!(g_gametype)).integer != GT_CTY
        && (*addr_of!(g_gametype)).integer != GT_SIEGE
        && (*addr_of!(g_gametype)).integer != GT_TEAM
    {
        return;
    }

    if (*addr_of!(g_gametype)).integer == GT_CTF || (*addr_of!(g_gametype)).integer == GT_CTY {
        stateMin = CTFSTATE_NONE;
        stateMax = CTFSTATE_MAXCTFSTATES;
    } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        stateMin = SIEGESTATE_NONE;
        stateMax = SIEGESTATE_MAXSIEGESTATES;
    } else if (*addr_of!(g_gametype)).integer == GT_TEAM {
        stateMin = TEAMPLAYSTATE_NONE;
        stateMax = TEAMPLAYSTATE_MAXTPSTATES;
    }

    if (ordernum < stateMin && ordernum != -1) || ordernum >= stateMax {
        return;
    }

    if clientnum != -1 {
        if ordernum == -1 {
            BotReportStatus(botstates[clientnum as usize]);
        } else {
            BotStraightTPOrderCheck(ent, ordernum, botstates[clientnum as usize]);
            (*botstates[clientnum as usize]).state_Forced = ordernum;
            (*botstates[clientnum as usize]).chatObject = ent;
            (*botstates[clientnum as usize]).chatAltObject = null_mut();
            if BotDoChat(
                botstates[clientnum as usize],
                c"OrderAccepted".as_ptr() as *mut c_char,
                1,
            ) != 0
            {
                (*botstates[clientnum as usize]).chatTeam = 1;
            }
        }
    } else {
        while i < MAX_CLIENTS as c_int {
            if !botstates[i as usize].is_null()
                && OnSameTeam(ent, (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize)) != QFALSE
            {
                if ordernum == -1 {
                    BotReportStatus(botstates[i as usize]);
                } else {
                    BotStraightTPOrderCheck(ent, ordernum, botstates[i as usize]);
                    (*botstates[i as usize]).state_Forced = ordernum;
                    (*botstates[i as usize]).chatObject = ent;
                    (*botstates[i as usize]).chatAltObject = null_mut();
                    if BotDoChat(
                        botstates[i as usize],
                        c"OrderAccepted".as_ptr() as *mut c_char,
                        0,
                    ) != 0
                    {
                        (*botstates[i as usize]).chatTeam = 1;
                    }
                }
            }

            i += 1;
        }
    }
}

/// `int BotFallbackNavigation(bot_state_t *bs)` (ai_main.c:5005) — when the waypoint nav has no
/// path, nudge the bot forward: trace a 16-unit step along its goal yaw; if clear, set that as the
/// goal position (`1`), otherwise pick a random new yaw (`0`). Returns `2` (busy) if fighting a
/// visible enemy.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotFallbackNavigation(bs: *mut bot_state_t) -> c_int {
    let mut b_angle: vec3_t = vec3_origin;
    let mut fwd: vec3_t = vec3_origin;
    let mut trto: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;

    if !(*bs).currentEnemy.is_null() && (*bs).frame_Enemy_Vis != 0 {
        return 2; //we're busy
    }

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = 0.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 32.0;

    (*bs).goalAngles[PITCH] = 0.0;
    (*bs).goalAngles[ROLL] = 0.0;

    VectorCopy(&(*bs).goalAngles, &mut b_angle);

    AngleVectors(&b_angle, Some(&mut fwd), None, None);

    trto[0] = (*bs).origin[0] + fwd[0] * 16.0;
    trto[1] = (*bs).origin[1] + fwd[1] * 16.0;
    trto[2] = (*bs).origin[2] + fwd[2] * 16.0;

    let tr = trap::Trace(&(*bs).origin, &mins, &maxs, &trto, ENTITYNUM_NONE, MASK_SOLID);

    if tr.fraction == 1.0 {
        VectorCopy(&trto, &mut (*bs).goalPosition);
        return 1; //success!
    } else {
        (*bs).goalAngles[YAW] = (rand() % 360) as f32;
    }

    0
}

/// `int BotTryAnotherWeapon(bot_state_t *bs)` (ai_main.c:5048) — out of ammo on the current
/// weapon: select the first weapon we own that has ammo; failing that, fall back to weapon 1
/// (which we should always have). Returns `1` if a switch was made.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotTryAnotherWeapon(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int;

    i = 1;

    while i < WP_NUM_WEAPONS {
        if (*bs).cur_ps.ammo[(*addr_of!(weaponData))[i as usize].ammoIndex as usize]
            >= (*addr_of!(weaponData))[i as usize].energyPerShot
            && (*bs).cur_ps.stats[STAT_WEAPONS as usize] & (1 << i) != 0
        {
            (*bs).virtualWeapon = i;
            BotSelectWeapon((*bs).client, i);
            return 1;
        }

        i += 1;
    }

    if (*bs).cur_ps.weapon != 1 && (*bs).virtualWeapon != 1 {
        //should always have this.. shouldn't we?
        (*bs).virtualWeapon = 1;
        BotSelectWeapon((*bs).client, 1);
        return 1;
    }

    0
}

/// `int BotSelectIdealWeapon(bot_state_t *bs)` (ai_main.c:5099) — pick the highest-weight weapon
/// we own that has ammo (thermals only when an enemy is within 700u), then override toward the
/// saber at close range or toward a ranged weapon if the saber is selected but the enemy is far
/// and not also using a saber. Returns `1` if a switch was made.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotSelectIdealWeapon(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int;
    let mut bestweight: c_int = -1;
    let mut bestweapon: c_int = 0;

    i = 0;

    while i < WP_NUM_WEAPONS {
        if (*bs).cur_ps.ammo[(*addr_of!(weaponData))[i as usize].ammoIndex as usize]
            >= (*addr_of!(weaponData))[i as usize].energyPerShot
            && (*bs).botWeaponWeights[i as usize] > bestweight as f32
            && (*bs).cur_ps.stats[STAT_WEAPONS as usize] & (1 << i) != 0
        {
            if i == WP_THERMAL {
                //special case..
                if !(*bs).currentEnemy.is_null() && (*bs).frame_Enemy_Len < 700.0 {
                    bestweight = (*bs).botWeaponWeights[i as usize] as c_int;
                    bestweapon = i;
                }
            } else {
                bestweight = (*bs).botWeaponWeights[i as usize] as c_int;
                bestweapon = i;
            }
        }

        i += 1;
    }

    if !(*bs).currentEnemy.is_null()
        && (*bs).frame_Enemy_Len < 300.0
        && (bestweapon == WP_BRYAR_PISTOL || bestweapon == WP_BLASTER || bestweapon == WP_BOWCASTER)
        && (*bs).cur_ps.stats[STAT_WEAPONS as usize] & (1 << WP_SABER) != 0
    {
        bestweapon = WP_SABER;
        bestweight = 1;
    }

    if !(*bs).currentEnemy.is_null()
        && (*bs).frame_Enemy_Len > 300.0
        && !(*(*bs).currentEnemy).client.is_null()
        && (*(*(*bs).currentEnemy).client).ps.weapon != WP_SABER
        && bestweapon == WP_SABER
    {
        //if the enemy is far away, and we have our saber selected, see if we have any good distance weapons instead
        if BotWeaponSelectable(bs, WP_DISRUPTOR) != QFALSE {
            bestweapon = WP_DISRUPTOR;
            bestweight = 1;
        } else if BotWeaponSelectable(bs, WP_ROCKET_LAUNCHER) != QFALSE {
            bestweapon = WP_ROCKET_LAUNCHER;
            bestweight = 1;
        } else if BotWeaponSelectable(bs, WP_BOWCASTER) != QFALSE {
            bestweapon = WP_BOWCASTER;
            bestweight = 1;
        } else if BotWeaponSelectable(bs, WP_BLASTER) != QFALSE {
            bestweapon = WP_BLASTER;
            bestweight = 1;
        } else if BotWeaponSelectable(bs, WP_REPEATER) != QFALSE {
            bestweapon = WP_REPEATER;
            bestweight = 1;
        } else if BotWeaponSelectable(bs, WP_DEMP2) != QFALSE {
            bestweapon = WP_DEMP2;
            bestweight = 1;
        }
    }

    //assert(bs->cur_ps.weapon > 0 && bestweapon > 0);

    if bestweight != -1 && (*bs).cur_ps.weapon != bestweapon && (*bs).virtualWeapon != bestweapon {
        (*bs).virtualWeapon = bestweapon;
        BotSelectWeapon((*bs).client, bestweapon);
        return 1;
    }

    //assert(bs->cur_ps.weapon > 0);

    0
}

/// `int BotSelectChoiceWeapon(bot_state_t *bs, int weapon, int doselection)` (ai_main.c:5192) —
/// check whether the bot owns `weapon` with ammo; if so and `doselection` is set, switch to it
/// (returns `2`), else just report ownership (`1`/`0`).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotSelectChoiceWeapon(bs: *mut bot_state_t, weapon: c_int, doselection: c_int) -> c_int {
    let mut i: c_int;
    let mut hasit: c_int = 0;

    i = 0;

    while i < WP_NUM_WEAPONS {
        if (*bs).cur_ps.ammo[(*addr_of!(weaponData))[i as usize].ammoIndex as usize]
            > (*addr_of!(weaponData))[i as usize].energyPerShot
            && i == weapon
            && (*bs).cur_ps.stats[STAT_WEAPONS as usize] & (1 << i) != 0
        {
            hasit = 1;
            break;
        }

        i += 1;
    }

    if hasit != 0
        && (*bs).cur_ps.weapon != weapon
        && doselection != 0
        && (*bs).virtualWeapon != weapon
    {
        (*bs).virtualWeapon = weapon;
        BotSelectWeapon((*bs).client, weapon);
        return 2;
    }

    if hasit != 0 {
        return 1;
    }

    0
}

/// `int BotSelectMelee(bot_state_t *bs)` (ai_main.c:5230) — force-select weapon 1 (melee/stun)
/// unless already selected. Returns `1` if a switch was made.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotSelectMelee(bs: *mut bot_state_t) -> c_int {
    if (*bs).cur_ps.weapon != 1 && (*bs).virtualWeapon != 1 {
        (*bs).virtualWeapon = 1;
        BotSelectWeapon((*bs).client, 1);
        return 1;
    }

    0
}

/// `void BotLovedOneDied(bot_state_t *bs, bot_state_t *loved, int lovelevel)` (ai_main.c:5291) — a
/// bot the player `bs` is attached to was killed: subject to teamplay/duel and not-a-teammate
/// checks, escalate `bs`'s revenge hatred toward the killer (`loved->lastHurt`), or switch revenge
/// targets if the current grudge is weak enough, issuing the appropriate `BotDoChat`.
///
/// # Safety
/// `bs`/`loved` must be valid [`bot_state_t`] pointers.
pub unsafe fn BotLovedOneDied(bs: *mut bot_state_t, loved: *mut bot_state_t, lovelevel: c_int) {
    if (*loved).lastHurt.is_null()
        || (*(*loved).lastHurt).client.is_null()
        || (*(*loved).lastHurt).s.number == (*loved).client
    {
        return;
    }

    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        //There is no love in 1-on-1
        return;
    }

    if IsTeamplay() == 0 {
        if lovelevel < 2 {
            return;
        }
    } else if OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), (*loved).lastHurt)
        != QFALSE
    {
        //don't hate teammates no matter what
        return;
    }

    if (*loved).client == (*(*loved).lastHurt).s.number {
        return;
    }

    if (*bs).client == (*(*loved).lastHurt).s.number {
        //oops!
        return;
    }

    if (*addr_of!(bot_attachments)).integer == 0 {
        return;
    }

    if PassLovedOneCheck(bs, (*loved).lastHurt) == 0 {
        //a loved one killed a loved one.. you cannot hate them
        (*bs).chatObject = (*loved).lastHurt;
        (*bs).chatAltObject =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*loved).client as usize);
        BotDoChat(bs, c"LovedOneKilledLovedOne".as_ptr() as *mut c_char, 0);
        return;
    }

    if (*bs).revengeEnemy == (*loved).lastHurt {
        if (*bs).revengeHateLevel < (*bs).loved_death_thresh {
            (*bs).revengeHateLevel += 1;

            if (*bs).revengeHateLevel == (*bs).loved_death_thresh {
                //broke into the highest anger level
                //CHAT: Hatred section
                (*bs).chatObject = (*loved).lastHurt;
                (*bs).chatAltObject = null_mut();
                BotDoChat(bs, c"Hatred".as_ptr() as *mut c_char, 1);
            }
        }
    } else if (*bs).revengeHateLevel < (*bs).loved_death_thresh - 1 {
        //only switch hatred if we don't hate the existing revenge-enemy too much
        //CHAT: BelovedKilled section
        (*bs).chatObject =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*loved).client as usize);
        (*bs).chatAltObject = (*loved).lastHurt;
        BotDoChat(bs, c"BelovedKilled".as_ptr() as *mut c_char, 0);
        (*bs).revengeHateLevel = 0;
        (*bs).revengeEnemy = (*loved).lastHurt;
    }
}

/// `void BotDeathNotify(bot_state_t *bs)` (ai_main.c:5372) — `bs` just died: notify every bot that
/// has `bs` in its loved list ([`BotLovedOneDied`]), matching by netname.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`botstates`]/[`level`].
pub unsafe fn BotDeathNotify(bs: *mut bot_state_t) {
    //in case someone has an emotional attachment to us, we'll notify them
    let mut i: c_int = 0;
    let mut ltest: c_int;

    while i < MAX_CLIENTS as c_int {
        if !botstates[i as usize].is_null() && (*botstates[i as usize]).lovednum != 0 {
            ltest = 0;
            while ltest < (*botstates[i as usize]).lovednum {
                if strcmp(
                    (*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .pers
                        .netname
                        .as_ptr(),
                    (*botstates[i as usize]).loved[ltest as usize].name.as_ptr(),
                ) == 0
                {
                    BotLovedOneDied(
                        botstates[i as usize],
                        bs,
                        (*botstates[i as usize]).loved[ltest as usize].level,
                    );
                    break;
                }

                ltest += 1;
            }
        }

        i += 1;
    }
}

/// `void StrafeTracing(bot_state_t *bs)` (ai_main.c:5399) — sanity-check the bot's melee strafe
/// direction: trace 32u to the side, and trace 32u down from there; if either hits a wall or
/// finds a ledge (no floor), disable strafing for a randomized 0.5-1.5s.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn StrafeTracing(bs: *mut bot_state_t) {
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;
    let mut right: vec3_t = vec3_origin;
    let mut rorg: vec3_t = vec3_origin;
    let mut drorg: vec3_t = vec3_origin;

    mins[0] = -15.0;
    mins[1] = -15.0;
    //mins[2] = -24;
    mins[2] = -22.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 32.0;

    AngleVectors(&(*bs).viewangles, None, Some(&mut right), None);

    if (*bs).meleeStrafeDir != 0 {
        rorg[0] = (*bs).origin[0] - right[0] * 32.0;
        rorg[1] = (*bs).origin[1] - right[1] * 32.0;
        rorg[2] = (*bs).origin[2] - right[2] * 32.0;
    } else {
        rorg[0] = (*bs).origin[0] + right[0] * 32.0;
        rorg[1] = (*bs).origin[1] + right[1] * 32.0;
        rorg[2] = (*bs).origin[2] + right[2] * 32.0;
    }

    let mut tr = trap::Trace(&(*bs).origin, &mins, &maxs, &rorg, (*bs).client, MASK_SOLID);

    if tr.fraction != 1.0 {
        (*bs).meleeStrafeDisable = ((*addr_of!(level)).time + Q_irand(500, 1500)) as f32;
    }

    VectorCopy(&rorg, &mut drorg);

    drorg[2] -= 32.0;

    tr = trap::Trace(&rorg, &vec3_origin, &vec3_origin, &drorg, (*bs).client, MASK_SOLID);

    if tr.fraction == 1.0 {
        //this may be a dangerous ledge, so don't strafe over it just in case
        (*bs).meleeStrafeDisable = ((*addr_of!(level)).time + Q_irand(500, 1500)) as f32;
    }
}

/// `void BotScanForLeader(bot_state_t *bs)` (ai_main.c:5567) — auto-pick a squad leader: the first
/// other bot that is a squad leader and either on our team or (outside teamplay) a loved one.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`g_entities`]/[`botstates`].
pub unsafe fn BotScanForLeader(bs: *mut bot_state_t) {
    //bots will only automatically obtain a leader if it's another bot using this method.
    let mut i: c_int = 0;
    let mut ent: *mut gentity_t;

    if (*bs).isSquadLeader != 0 {
        return;
    }

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && !botstates[i as usize].is_null()
            && (*botstates[i as usize]).isSquadLeader != 0
            && (*bs).client != i
        {
            if OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE {
                (*bs).squadLeader = ent;
                break;
            }
            if GetLoveLevel(bs, botstates[i as usize]) > 1 && IsTeamplay() == 0 {
                //ignore love status regarding squad leaders if we're in teamplay
                (*bs).squadLeader = ent;
                break;
            }
        }

        i += 1;
    }
}

/// `void BotReplyGreetings(bot_state_t *bs)` (ai_main.c:5618) — when `bs` greets, have every other
/// chat-capable bot (skipping `bs` itself) fire a `"ResponseGreetings"` line aimed at `bs`'s client
/// entity, capping it at 4 replies so a crowd of bots doesn't all answer at once.
///
/// No oracle: bot/engine glue (mutates per-client [`botstates`] + calls [`BotDoChat`]; no pure
/// return value).
///
/// # Safety
/// `bs` must be valid; reads/writes the process-global [`botstates`] and [`g_entities`].
pub unsafe fn BotReplyGreetings(bs: *mut bot_state_t) {
    let mut i: c_int = 0;
    let mut numhello: c_int = 0;

    while i < MAX_CLIENTS as c_int {
        if !(*addr_of!(botstates))[i as usize].is_null()
            && (*(*addr_of!(botstates))[i as usize]).canChat != 0
            && i != (*bs).client
        {
            (*(*addr_of_mut!(botstates))[i as usize]).chatObject =
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize);
            (*(*addr_of_mut!(botstates))[i as usize]).chatAltObject = null_mut();
            if BotDoChat(
                (*addr_of!(botstates))[i as usize],
                c"ResponseGreetings".as_ptr() as *mut c_char,
                0,
            ) != 0
            {
                numhello += 1;
            }
        }

        if numhello > 3 {
            //don't let more than 4 bots say hello at once
            return;
        }

        i += 1;
    }
}

/// `void CTFFlagMovement(bot_state_t *bs)` (ai_main.c:5631) — when heading for a flag waypoint and
/// a matching dropped flag is within reach (and traceable), redirect the bot's goal position
/// straight to the dropped flag; otherwise drop a stale `wantFlag`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads the CTF flag globals.
pub unsafe fn CTFFlagMovement(bs: *mut bot_state_t) {
    //try to move in to grab a nearby flag
    let mut diddrop: c_int = 0;
    let mut desiredDrop: *mut gentity_t = null_mut();
    let mut a: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;

    mins[0] = -15.0;
    mins[1] = -15.0;
    mins[2] = -7.0;
    maxs[0] = 15.0;
    maxs[1] = 15.0;
    maxs[2] = 7.0;

    if !(*bs).wantFlag.is_null() && (*(*bs).wantFlag).flags & FL_DROPPED_ITEM != 0 {
        if (*bs).staticFlagSpot[0] == (*(*bs).wantFlag).s.pos.trBase[0]
            && (*bs).staticFlagSpot[1] == (*(*bs).wantFlag).s.pos.trBase[1]
            && (*bs).staticFlagSpot[2] == (*(*bs).wantFlag).s.pos.trBase[2]
        {
            VectorSubtract(&(*bs).origin, &(*(*bs).wantFlag).s.pos.trBase, &mut a);

            if VectorLength(&a) <= BOT_FLAG_GET_DISTANCE as f32 {
                VectorCopy(&(*(*bs).wantFlag).s.pos.trBase, &mut (*bs).goalPosition);
                return;
            } else {
                (*bs).wantFlag = null_mut();
            }
        } else {
            (*bs).wantFlag = null_mut();
        }
    } else if !(*bs).wantFlag.is_null() {
        (*bs).wantFlag = null_mut();
    }

    if !(*addr_of!(flagRed)).is_null() && !(*addr_of!(flagBlue)).is_null() {
        if (*bs).wpDestination == *addr_of!(flagRed)
            || (*bs).wpDestination == *addr_of!(flagBlue)
        {
            if (*bs).wpDestination == *addr_of!(flagRed)
                && !(*addr_of!(droppedRedFlag)).is_null()
                && (*(*addr_of!(droppedRedFlag))).flags & FL_DROPPED_ITEM != 0
                && !(*(*addr_of!(droppedRedFlag))).classname.is_null()
                && strcmp((*(*addr_of!(droppedRedFlag))).classname, c"freed".as_ptr()) != 0
            {
                desiredDrop = *addr_of!(droppedRedFlag);
                diddrop = 1;
            }
            if (*bs).wpDestination == *addr_of!(flagBlue)
                && !(*addr_of!(droppedBlueFlag)).is_null()
                && (*(*addr_of!(droppedBlueFlag))).flags & FL_DROPPED_ITEM != 0
                && !(*(*addr_of!(droppedBlueFlag))).classname.is_null()
                && strcmp((*(*addr_of!(droppedBlueFlag))).classname, c"freed".as_ptr()) != 0
            {
                desiredDrop = *addr_of!(droppedBlueFlag);
                diddrop = 1;
            }

            if diddrop != 0 && !desiredDrop.is_null() {
                VectorSubtract(&(*bs).origin, &(*desiredDrop).s.pos.trBase, &mut a);

                if VectorLength(&a) <= BOT_FLAG_GET_DISTANCE as f32 {
                    let tr = trap::Trace(
                        &(*bs).origin,
                        &mins,
                        &maxs,
                        &(*desiredDrop).s.pos.trBase,
                        (*bs).client,
                        MASK_SOLID,
                    );

                    if tr.fraction == 1.0 || tr.entityNum as c_int == (*desiredDrop).s.number {
                        VectorCopy(&(*desiredDrop).s.pos.trBase, &mut (*bs).goalPosition);
                        VectorCopy(&(*desiredDrop).s.pos.trBase, &mut (*bs).staticFlagSpot);
                        return;
                    }
                }
            }
        }
    }
}

/// `void BotCheckDetPacks(bot_state_t *bs)` (ai_main.c:5710) — if the bot has a planted detpack
/// and the enemy is closer to it than the bot is (and is visible, or the plant is very fresh),
/// arm the "blow it" timer.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotCheckDetPacks(bs: *mut bot_state_t) {
    //see if we want to make our detpacks blow up
    let mut dp: *mut gentity_t = null_mut();
    let mut myDet: *mut gentity_t = null_mut();
    let mut a: vec3_t = vec3_origin;
    let enLen: f32;
    let myLen: f32;

    loop {
        dp = G_Find(dp, offset_of!(gentity_t, classname), c"detpack".as_ptr());
        if dp.is_null() {
            break;
        }
        if !dp.is_null() && !(*dp).parent.is_null() && (*(*dp).parent).s.number == (*bs).client {
            myDet = dp;
            break;
        }
    }

    if myDet.is_null() {
        return;
    }

    if (*bs).currentEnemy.is_null()
        || (*(*bs).currentEnemy).client.is_null()
        || (*bs).frame_Enemy_Vis == 0
    {
        //require the enemy to be visilbe just to be fair..

        //unless..
        if !(*bs).currentEnemy.is_null()
            && !(*(*bs).currentEnemy).client.is_null()
            && ((*addr_of!(level)).time - (*bs).plantContinue) < 5000
        {
            //it's a fresh plant (within 5 seconds) so we should be able to guess
            // goto stillmadeit;
        } else {
            return;
        }
    }

    // stillmadeit:

    VectorSubtract(
        &(*(*(*bs).currentEnemy).client).ps.origin,
        &(*myDet).s.pos.trBase,
        &mut a,
    );
    enLen = VectorLength(&a);

    VectorSubtract(&(*bs).origin, &(*myDet).s.pos.trBase, &mut a);
    myLen = VectorLength(&a);

    if enLen > myLen {
        return;
    }

    if enLen < BOT_PLANT_BLOW_DISTANCE as f32
        && OrgVisible(
            &(*(*(*bs).currentEnemy).client).ps.origin,
            &(*myDet).s.pos.trBase,
            (*(*bs).currentEnemy).s.number,
        ) != 0
    {
        //we could just call the "blow all my detpacks" function here, but I guess that's cheating.
        (*bs).plantKillEmAll = (*addr_of!(level)).time + 500;
    }
}

/// `int BotUseInventoryItem(bot_state_t *bs)` (ai_main.c:5764) — pick the most useful held item to
/// deploy this frame (medpacs by health, seeker/sentry when fighting, shield when fleeing) and
/// stage it into the client's `STAT_HOLDABLE_ITEM`. Returns `1` if an item was chosen.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotUseInventoryItem(bs: *mut bot_state_t) -> c_int {
    //see if it would be beneficial at this time to use one of our inv items
    let mut wantuseitem = false;

    if (*bs).cur_ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_MEDPAC) != 0 {
        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health <= 75 {
            (*bs).cur_ps.stats[STAT_HOLDABLE_ITEM as usize] =
                BG_GetItemIndexByTag(HI_MEDPAC, IT_HOLDABLE);
            wantuseitem = true;
        }
    }
    if !wantuseitem && (*bs).cur_ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_MEDPAC_BIG) != 0 {
        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health <= 50 {
            (*bs).cur_ps.stats[STAT_HOLDABLE_ITEM as usize] =
                BG_GetItemIndexByTag(HI_MEDPAC_BIG, IT_HOLDABLE);
            wantuseitem = true;
        }
    }
    if !wantuseitem && (*bs).cur_ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SEEKER) != 0 {
        if !(*bs).currentEnemy.is_null() && (*bs).frame_Enemy_Vis != 0 {
            (*bs).cur_ps.stats[STAT_HOLDABLE_ITEM as usize] =
                BG_GetItemIndexByTag(HI_SEEKER, IT_HOLDABLE);
            wantuseitem = true;
        }
    }
    if !wantuseitem && (*bs).cur_ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SENTRY_GUN) != 0 {
        if !(*bs).currentEnemy.is_null() && (*bs).frame_Enemy_Vis != 0 {
            (*bs).cur_ps.stats[STAT_HOLDABLE_ITEM as usize] =
                BG_GetItemIndexByTag(HI_SENTRY_GUN, IT_HOLDABLE);
            wantuseitem = true;
        }
    }
    if !wantuseitem && (*bs).cur_ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SHIELD) != 0 {
        if !(*bs).currentEnemy.is_null()
            && (*bs).frame_Enemy_Vis != 0
            && (*bs).runningToEscapeThreat != 0
        {
            //this will (hopefully) result in the bot placing the shield down while facing
            //the enemy and running away
            (*bs).cur_ps.stats[STAT_HOLDABLE_ITEM as usize] =
                BG_GetItemIndexByTag(HI_SHIELD, IT_HOLDABLE);
            wantuseitem = true;
        }
    }

    if !wantuseitem {
        return 0;
    }

    // wantuseitem:
    (*(*addr_of!(level)).clients.add((*bs).client as usize)).ps.stats
        [STAT_HOLDABLE_ITEM as usize] = (*bs).cur_ps.stats[STAT_HOLDABLE_ITEM as usize];

    1
}

/// `void Bot_SetForcedMovement(int bot, int forward, int right, int up)` (ai_main.c:5867) — toggle
/// the bot's forced movement overrides: each non-`-1` axis flips between the requested value and 0.
///
/// # Safety
/// Reads [`botstates`].
pub unsafe fn Bot_SetForcedMovement(bot: c_int, forward: c_int, right: c_int, up: c_int) {
    //movement overrides
    let bs: *mut bot_state_t = botstates[bot as usize];

    if bs.is_null() {
        //not a bot
        return;
    }

    if forward != -1 {
        if (*bs).forceMove_Forward != 0 {
            (*bs).forceMove_Forward = 0;
        } else {
            (*bs).forceMove_Forward = forward;
        }
    }
    if right != -1 {
        if (*bs).forceMove_Right != 0 {
            (*bs).forceMove_Right = 0;
        } else {
            (*bs).forceMove_Right = right;
        }
    }
    if up != -1 {
        if (*bs).forceMove_Up != 0 {
            (*bs).forceMove_Up = 0;
        } else {
            (*bs).forceMove_Up = up;
        }
    }
}

/// `int BotAISetupClient(int client, struct bot_settings_s *settings, qboolean restart)`
/// (ai_main.c:811) — allocate (engine-zone `B_Alloc`) and zero a fresh [`bot_state_t`] for
/// `client`, copy in its settings, seed the default per-weapon weights (saber boosted in duel),
/// run its personality, mark it in use, bump [`numbots`], and reschedule bot thinking.
///
/// # Safety
/// `settings` must point at a valid [`bot_settings_t`]; writes the [`botstates`] global.
pub unsafe fn BotAISetupClient(
    client: c_int,
    settings: *mut bot_settings_t,
    _restart: qboolean,
) -> c_int {
    let bs: *mut bot_state_t;

    if botstates[client as usize].is_null() {
        botstates[client as usize] = B_Alloc(core::mem::size_of::<bot_state_t>() as c_int)
            as *mut bot_state_t; //G_Alloc(sizeof(bot_state_t));
                                 //rww - G_Alloc bad! B_Alloc good.
    }

    write_bytes(botstates[client as usize], 0, 1);

    bs = botstates[client as usize];

    if !bs.is_null() && (*bs).inuse != 0 {
        BotAI_Print(PRT_FATAL, c"BotAISetupClient: client %d already setup\n".as_ptr() as *mut c_char);
        return QFALSE as c_int;
    }

    core::ptr::copy_nonoverlapping(settings, &mut (*bs).settings, 1);

    (*bs).client = client; //need to know the client number before doing personality stuff

    //initialize weapon weight defaults..
    (*bs).botWeaponWeights[WP_NONE as usize] = 0.0;
    (*bs).botWeaponWeights[WP_STUN_BATON as usize] = 1.0;
    (*bs).botWeaponWeights[WP_SABER as usize] = 10.0;
    (*bs).botWeaponWeights[WP_BRYAR_PISTOL as usize] = 11.0;
    (*bs).botWeaponWeights[WP_BLASTER as usize] = 12.0;
    (*bs).botWeaponWeights[WP_DISRUPTOR as usize] = 13.0;
    (*bs).botWeaponWeights[WP_BOWCASTER as usize] = 14.0;
    (*bs).botWeaponWeights[WP_REPEATER as usize] = 15.0;
    (*bs).botWeaponWeights[WP_DEMP2 as usize] = 16.0;
    (*bs).botWeaponWeights[WP_FLECHETTE as usize] = 17.0;
    (*bs).botWeaponWeights[WP_ROCKET_LAUNCHER as usize] = 18.0;
    (*bs).botWeaponWeights[WP_THERMAL as usize] = 14.0;
    (*bs).botWeaponWeights[WP_TRIP_MINE as usize] = 0.0;
    (*bs).botWeaponWeights[WP_DET_PACK as usize] = 0.0;
    (*bs).botWeaponWeights[WP_MELEE as usize] = 1.0;

    BotUtilizePersonality(bs);

    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        (*bs).botWeaponWeights[WP_SABER as usize] = 13.0;
    }

    //allocate a goal state
    (*bs).gs = trap::BotAllocGoalState(client);

    //allocate a weapon state
    (*bs).ws = trap::BotAllocWeaponState();

    (*bs).inuse = QTRUE as c_int;
    (*bs).entitynum = client;
    (*bs).setupcount = 4;
    (*bs).entergame_time = FloatTime();
    (*bs).ms = trap::BotAllocMoveState();
    numbots += 1;

    //NOTE: reschedule the bot thinking
    BotScheduleBotThink();

    if PlayersInGame() != 0 {
        //don't talk to yourself
        BotDoChat(bs, c"GeneralGreetings".as_ptr() as *mut c_char, 0);
    }

    QTRUE as c_int
}

/// `int BotAIShutdownClient(int client, qboolean restart)` (ai_main.c:878) — tear down a bot:
/// zero its state, clear `inuse`, decrement [`numbots`]. No-op (returns false) if not in use.
///
/// # Safety
/// Writes the [`botstates`] global.
pub unsafe fn BotAIShutdownClient(client: c_int, _restart: qboolean) -> c_int {
    let bs: *mut bot_state_t;

    bs = botstates[client as usize];
    if bs.is_null() || (*bs).inuse == 0 {
        //BotAI_Print(PRT_ERROR, "BotAIShutdownClient: client %d already shutdown\n", client);
        return QFALSE as c_int;
    }

    trap::BotFreeMoveState((*bs).ms);
    //free the goal state`
    trap::BotFreeGoalState((*bs).gs);
    //free the weapon weights
    trap::BotFreeWeaponState((*bs).ws);
    //
    //clear the bot state
    write_bytes(bs, 0, 1);
    //set the inuse flag to qfalse
    (*bs).inuse = QFALSE as c_int;
    //there's one bot less
    numbots -= 1;
    //everything went ok
    QTRUE as c_int
}

/// `int BotAIShutdown(int restart)` (ai_main.c:7623) — shut down all bots: per-client on a
/// tournament restart (keeping the bot library alive), else shut down the whole bot library.
///
/// # Safety
/// Reads/writes the [`botstates`] global.
pub unsafe fn BotAIShutdown(restart: c_int) -> c_int {
    let mut i: c_int;

    //if the game is restarted for a tournament
    if restart != 0 {
        //shutdown all the bots in the botlib
        i = 0;
        while i < MAX_CLIENTS as c_int {
            if !botstates[i as usize].is_null() && (*botstates[i as usize]).inuse != 0 {
                BotAIShutdownClient((*botstates[i as usize]).client, restart as qboolean);
            }
            i += 1;
        }
        //don't shutdown the bot library
    } else {
        trap::BotLibShutdown();
    }

    QTRUE as c_int
}

/// `gentity_t *GetNearestBadThing(bot_state_t *bs)` (ai_main.c:2370) — scan for the nearest
/// dangerous non-client entity (damaging projectile/mine within 800u, or an enemy sentry gun),
/// weighting mines/detpacks heavier and ignoring friendly rockets; force-pushes near projectiles
/// at high skill, and adopts a projectile's owner as the enemy when we have none. Returns the
/// nearest threat (and stamps `dontGoBack`) or null.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`g_entities`]/[`level`].
// `factor`'s `= 0` initializer is a faithful C dead store (always reassigned before use).
#[allow(unused_assignments)]
pub unsafe fn GetNearestBadThing(bs: *mut bot_state_t) -> *mut gentity_t {
    let mut i: c_int = 0;
    let mut glen: f32;
    let mut hold: vec3_t = vec3_origin;
    let mut bestindex: c_int = 0;
    let mut bestdist: f32 = 800.0; //if not within a radius of 800, it's no threat anyway
    let mut foundindex: c_int = 0;
    let mut factor: f32 = 0.0;
    let mut ent: *mut gentity_t;

    while i < (*addr_of!(level)).num_entities {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (!ent.is_null()
            && (*ent).client.is_null()
            && (*ent).inuse != 0
            && (*ent).damage != 0
            && (*ent).s.weapon != 0
            && (*ent).splashDamage != 0)
            || (!ent.is_null()
                && (*ent).genericValue5 == 1000
                && (*ent).inuse != 0
                && (*ent).health > 0
                && (*ent).genericValue3 != (*bs).client
                && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).genericValue3 as usize))
                    .client
                    .is_null()
                && OnSameTeam(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize),
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).genericValue3 as usize),
                ) == QFALSE)
        {
            //try to escape from anything with a non-0 s.weapon and non-0 damage. This hopefully only means dangerous projectiles.
            //Or a sentry gun if bolt_Head == 1000. This is a terrible hack, yes.
            VectorSubtract(&(*bs).origin, &(*ent).r.currentOrigin, &mut hold);
            glen = VectorLength(&hold);

            if (*ent).s.weapon != WP_THERMAL
                && (*ent).s.weapon != WP_FLECHETTE
                && (*ent).s.weapon != WP_DET_PACK
                && (*ent).s.weapon != WP_TRIP_MINE
            {
                factor = 0.5;

                if (*ent).s.weapon != 0 && glen <= 256.0 && (*bs).settings.skill > 2.0 {
                    //it's a projectile so push it away
                    (*bs).doForcePush = (*addr_of!(level)).time + 700;
                }
            } else {
                factor = 1.0;
            }

            if (*ent).s.weapon == WP_ROCKET_LAUNCHER
                && ((*ent).r.ownerNum == (*bs).client
                    || ((*ent).r.ownerNum > 0
                        && (*ent).r.ownerNum < MAX_CLIENTS as c_int
                        && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize))
                            .client
                            .is_null()
                        && OnSameTeam(
                            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize),
                            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize),
                        ) != QFALSE))
            {
                //don't be afraid of your own rockets or your teammates' rockets
                factor = 0.0;
            }

            if glen < bestdist * factor && BotPVSCheck(&(*bs).origin, &(*ent).s.pos.trBase) != QFALSE
            {
                let tr = trap::Trace(
                    &(*bs).origin,
                    &vec3_origin,
                    &vec3_origin,
                    &(*ent).s.pos.trBase,
                    (*bs).client,
                    MASK_SOLID,
                );

                if tr.fraction == 1.0 || tr.entityNum as c_int == (*ent).s.number {
                    bestindex = i;
                    bestdist = glen;
                    foundindex = 1;
                }
            }
        }

        if !ent.is_null()
            && (*ent).client.is_null()
            && (*ent).inuse != 0
            && (*ent).damage != 0
            && (*ent).s.weapon != 0
            && (*ent).r.ownerNum < MAX_CLIENTS as c_int
            && (*ent).r.ownerNum >= 0
        {
            //if we're in danger of a projectile belonging to someone and don't have an enemy, set the enemy to them
            let projOwner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).r.ownerNum as usize);

            if !projOwner.is_null() && (*projOwner).inuse != 0 && !(*projOwner).client.is_null() {
                if (*bs).currentEnemy.is_null() {
                    if PassStandardEnemyChecks(bs, projOwner) != 0 {
                        if PassLovedOneCheck(bs, projOwner) != 0 {
                            VectorSubtract(&(*bs).origin, &(*ent).r.currentOrigin, &mut hold);
                            glen = VectorLength(&hold);

                            if glen < 512.0 {
                                (*bs).currentEnemy = projOwner;
                                (*bs).enemySeenTime =
                                    ((*addr_of!(level)).time + ENEMY_FORGET_MS) as f32;
                            }
                        }
                    }
                }
            }
        }

        i += 1;
    }

    if foundindex != 0 {
        (*bs).dontGoBack = ((*addr_of!(level)).time + 1500) as f32;
        return (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(bestindex as usize);
    } else {
        null_mut()
    }
}

/// `int BotDefendFlag(bot_state_t *bs)` (ai_main.c:2482) — keep the CTF nav goal on our own flag
/// point when we've strayed past `BASE_GUARD_DISTANCE`. Returns `1` if our team has a flag point.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotDefendFlag(bs: *mut bot_state_t) -> c_int {
    let flagPoint: *mut wpobject_t;
    let mut a: vec3_t = vec3_origin;

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        flagPoint = *addr_of!(flagRed);
    } else if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_BLUE
    {
        flagPoint = *addr_of!(flagBlue);
    } else {
        return 0;
    }

    if flagPoint.is_null() {
        return 0;
    }

    VectorSubtract(&(*bs).origin, &(*flagPoint).origin, &mut a);

    if VectorLength(&a) > BASE_GUARD_DISTANCE as f32 {
        (*bs).wpDestination = flagPoint;
    }

    1
}

/// `int BotGetEnemyFlag(bot_state_t *bs)` (ai_main.c:2516) — keep the CTF nav goal on the enemy
/// flag point when past `BASE_GETENEMYFLAG_DISTANCE`. Returns `1` if there is an enemy flag point.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotGetEnemyFlag(bs: *mut bot_state_t) -> c_int {
    let flagPoint: *mut wpobject_t;
    let mut a: vec3_t = vec3_origin;

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        flagPoint = *addr_of!(flagBlue);
    } else if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_BLUE
    {
        flagPoint = *addr_of!(flagRed);
    } else {
        return 0;
    }

    if flagPoint.is_null() {
        return 0;
    }

    VectorSubtract(&(*bs).origin, &(*flagPoint).origin, &mut a);

    if VectorLength(&a) > BASE_GETENEMYFLAG_DISTANCE as f32 {
        (*bs).wpDestination = flagPoint;
    }

    1
}

/// `int BotGetFlagBack(bot_state_t *bs)` (ai_main.c:2550) — our flag was taken: find the enemy
/// carrier and route toward the nearest waypoint to them (re-evaluated on a 1-5s timer). Returns
/// `1` if a carrier was found.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotGetFlagBack(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int = 0;
    let myFlag: c_int;
    let mut foundCarrier: c_int = 0;
    let tempInt: c_int;
    let mut ent: *mut gentity_t = null_mut();
    let mut usethisvec: vec3_t = vec3_origin;

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        myFlag = PW_REDFLAG;
    } else {
        myFlag = PW_BLUEFLAG;
    }

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && (*(*ent).client).ps.powerups[myFlag as usize] != 0
            && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) == QFALSE
        {
            foundCarrier = 1;
            break;
        }

        i += 1;
    }

    if foundCarrier == 0 {
        return 0;
    }

    if ent.is_null() {
        return 0;
    }

    if ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
        if !(*ent).client.is_null() {
            VectorCopy(&(*(*ent).client).ps.origin, &mut usethisvec);
        } else {
            VectorCopy(&(*ent).s.origin, &mut usethisvec);
        }

        tempInt = GetNearestVisibleWP(&usethisvec, 0);

        if tempInt != -1 && TotalTrailDistance((*(*bs).wpCurrent).index, tempInt, bs) != -1.0 {
            (*bs).wpDestination = (*addr_of!(gWPArray))[tempInt as usize];
            (*bs).wpDestSwitchTime = ((*addr_of!(level)).time + Q_irand(1000, 5000)) as f32;
        }
    }

    1
}

/// `int BotGuardFlagCarrier(bot_state_t *bs)` (ai_main.c:2616) — a teammate has the enemy flag:
/// route toward them (re-evaluated on a 1-5s timer). Returns `1` if a friendly carrier was found.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotGuardFlagCarrier(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int = 0;
    let enemyFlag: c_int;
    let mut foundCarrier: c_int = 0;
    let tempInt: c_int;
    let mut ent: *mut gentity_t = null_mut();
    let mut usethisvec: vec3_t = vec3_origin;

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        enemyFlag = PW_BLUEFLAG;
    } else {
        enemyFlag = PW_REDFLAG;
    }

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && (*(*ent).client).ps.powerups[enemyFlag as usize] != 0
            && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE
        {
            foundCarrier = 1;
            break;
        }

        i += 1;
    }

    if foundCarrier == 0 {
        return 0;
    }

    if ent.is_null() {
        return 0;
    }

    if ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
        if !(*ent).client.is_null() {
            VectorCopy(&(*(*ent).client).ps.origin, &mut usethisvec);
        } else {
            VectorCopy(&(*ent).s.origin, &mut usethisvec);
        }

        tempInt = GetNearestVisibleWP(&usethisvec, 0);

        if tempInt != -1 && TotalTrailDistance((*(*bs).wpCurrent).index, tempInt, bs) != -1.0 {
            (*bs).wpDestination = (*addr_of!(gWPArray))[tempInt as usize];
            (*bs).wpDestSwitchTime = ((*addr_of!(level)).time + Q_irand(1000, 5000)) as f32;
        }
    }

    1
}

/// `int BotGetFlagHome(bot_state_t *bs)` (ai_main.c:2681) — we carry the enemy flag: head for our
/// own flag point until within `BASE_FLAGWAIT_DISTANCE`. Returns `1` if our flag point exists.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
pub unsafe fn BotGetFlagHome(bs: *mut bot_state_t) -> c_int {
    let flagPoint: *mut wpobject_t;
    let mut a: vec3_t = vec3_origin;

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        flagPoint = *addr_of!(flagRed);
    } else if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_BLUE
    {
        flagPoint = *addr_of!(flagBlue);
    } else {
        return 0;
    }

    if flagPoint.is_null() {
        return 0;
    }

    VectorSubtract(&(*bs).origin, &(*flagPoint).origin, &mut a);

    if VectorLength(&a) > BASE_FLAGWAIT_DISTANCE as f32 {
        (*bs).wpDestination = flagPoint;
    }

    1
}

/// `int Siege_TargetClosestObjective(bot_state_t *bs, int flag)` (ai_main.c:3030) — pick the
/// nearest usable siege-objective waypoint flagged `flag`, route to it, and decide whether to
/// shoot it (a `takedamage` brush within range, never with a melee/saber weapon) or to touch it.
/// Returns `1` if an objective was found. The hacky brush-center via `(absmax+absmin)/2` is kept.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`gWPArray`]/[`g_entities`].
pub unsafe fn Siege_TargetClosestObjective(bs: *mut bot_state_t, flag: c_int) -> c_int {
    let mut i: c_int = 0;
    let mut bestindex: c_int = -1;
    let mut testdistance: f32;
    let mut bestdistance: f32 = 999999999.0;
    let goalent: *mut gentity_t;
    let mut a: vec3_t = vec3_origin;
    let mut dif: vec3_t = vec3_origin;
    let mut mins: vec3_t = vec3_origin;
    let mut maxs: vec3_t = vec3_origin;

    mins[0] = -1.0;
    mins[1] = -1.0;
    mins[2] = -1.0;

    maxs[0] = 1.0;
    maxs[1] = 1.0;
    maxs[2] = 1.0;

    'hasPoint: {
        if !(*bs).wpDestination.is_null()
            && (*(*bs).wpDestination).flags & flag != 0
            && (*(*bs).wpDestination).associated_entity != ENTITYNUM_NONE
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*bs).wpDestination).associated_entity as usize))
                .r#use
                .is_some()
        {
            break 'hasPoint;
        }

        while i < (*addr_of!(gWPNum)) {
            if !(*addr_of!(gWPArray))[i as usize].is_null()
                && (*(*addr_of!(gWPArray))[i as usize]).inuse != 0
                && (*(*addr_of!(gWPArray))[i as usize]).flags & flag != 0
                && (*(*addr_of!(gWPArray))[i as usize]).associated_entity != ENTITYNUM_NONE
                && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*addr_of!(gWPArray))[i as usize]).associated_entity as usize))
                .r#use
                .is_some()
            {
                VectorSubtract(&(*(*addr_of!(gWPArray))[i as usize]).origin, &(*bs).origin, &mut a);
                testdistance = VectorLength(&a);

                if testdistance < bestdistance {
                    bestdistance = testdistance;
                    bestindex = i;
                }
            }

            i += 1;
        }

        if bestindex != -1 {
            (*bs).wpDestination = (*addr_of!(gWPArray))[bestindex as usize];
        } else {
            return 0;
        }
    }
    // hasPoint:
    goalent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*(*bs).wpDestination).associated_entity as usize);

    if goalent.is_null() {
        return 0;
    }

    VectorSubtract(&(*bs).origin, &(*(*bs).wpDestination).origin, &mut a);

    testdistance = VectorLength(&a);

    dif[0] = ((*goalent).r.absmax[0] + (*goalent).r.absmin[0]) / 2.0;
    dif[1] = ((*goalent).r.absmax[1] + (*goalent).r.absmin[1]) / 2.0;
    dif[2] = ((*goalent).r.absmax[2] + (*goalent).r.absmin[2]) / 2.0;
    //brush models can have tricky origins, so this is our hacky method of getting the center point

    if (*goalent).takedamage != QFALSE
        && testdistance < BOT_MIN_SIEGE_GOAL_SHOOT as f32
        && EntityVisibleBox(&(*bs).origin, &mins, &maxs, &dif, (*bs).client, (*goalent).s.number)
            != 0
    {
        (*bs).shootGoal = goalent;
        (*bs).touchGoal = null_mut();
    } else if (*goalent).r#use.is_some() && testdistance < BOT_MIN_SIEGE_GOAL_TRAVEL as f32 {
        (*bs).shootGoal = null_mut();
        (*bs).touchGoal = goalent;
    } else {
        //don't know how to handle this goal object!
        (*bs).shootGoal = null_mut();
        (*bs).touchGoal = null_mut();
    }

    if BotGetWeaponRange(bs) == BWEAPONRANGE_MELEE || BotGetWeaponRange(bs) == BWEAPONRANGE_SABER {
        (*bs).shootGoal = null_mut(); //too risky
    }

    if !(*bs).touchGoal.is_null() {
        VectorCopy(&dif, &mut (*bs).goalPosition);
    }

    1
}

/// `void Siege_DefendFromAttackers(bot_state_t *bs)` (ai_main.c:3129) — pick a defensive nav goal
/// by routing to the nearest waypoint to the closest living enemy (the likely attacker).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`g_entities`]/[`gWPArray`].
pub unsafe fn Siege_DefendFromAttackers(bs: *mut bot_state_t) {
    //this may be a little cheap, but the best way to find our defending point is probably
    //to just find the nearest person on the opposing team since they'll most likely
    //be on offense in this situation
    let wpClose: c_int;
    let mut i: c_int = 0;
    let mut testdist: f32;
    let mut bestindex: c_int = -1;
    let mut bestdist: f32 = 999999.0;
    let mut ent: *mut gentity_t;
    let mut a: vec3_t = vec3_origin;

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null()
            && !(*ent).client.is_null()
            && (*(*ent).client).sess.sessionTeam
                != (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                    .sess
                    .sessionTeam
            && (*ent).health > 0
            && (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
        {
            VectorSubtract(&(*(*ent).client).ps.origin, &(*bs).origin, &mut a);

            testdist = VectorLength(&a);

            if testdist < bestdist {
                bestindex = i;
                bestdist = testdist;
            }
        }

        i += 1;
    }

    if bestindex == -1 {
        return;
    }

    wpClose = GetNearestVisibleWP(
        &(*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(bestindex as usize)).client).ps.origin,
        -1,
    );

    if wpClose != -1
        && !(*addr_of!(gWPArray))[wpClose as usize].is_null()
        && (*(*addr_of!(gWPArray))[wpClose as usize]).inuse != 0
    {
        (*bs).wpDestination = (*addr_of!(gWPArray))[wpClose as usize];
        (*bs).destinationGrabTime = ((*addr_of!(level)).time + 10000) as f32;
    }
}

/// `int Siege_CountDefenders(bot_state_t *bs)` (ai_main.c:3177) — count bots on our team in
/// `SIEGESTATE_DEFENDER`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`g_entities`]/[`botstates`].
pub unsafe fn Siege_CountDefenders(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int = 0;
    let mut num: c_int = 0;
    let mut ent: *mut gentity_t;
    let mut bot: *mut bot_state_t;

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        bot = botstates[i as usize];

        if !ent.is_null() && !(*ent).client.is_null() && !bot.is_null() {
            if (*bot).siegeState == SIEGESTATE_DEFENDER
                && (*(*ent).client).sess.sessionTeam
                    == (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                        .sess
                        .sessionTeam
            {
                num += 1;
            }
        }

        i += 1;
    }

    num
}

/// `int Siege_CountTeammates(bot_state_t *bs)` (ai_main.c:3205) — count all clients on our team.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`g_entities`].
pub unsafe fn Siege_CountTeammates(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int = 0;
    let mut num: c_int = 0;
    let mut ent: *mut gentity_t;

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null() && !(*ent).client.is_null() {
            if (*(*ent).client).sess.sessionTeam
                == (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                    .sess
                    .sessionTeam
            {
                num += 1;
            }
        }

        i += 1;
    }

    num
}

/// `int GetBestIdleGoal(bot_state_t *bs)` (ai_main.c:3535) — when idle, choose the best
/// goal-point waypoint we don't already have the item for, weighted by waypoint weight minus
/// trail distance. Randomly suppresses item-seeking (`randomNav`) on a 5-15s timer for
/// non-campers. Returns the waypoint index, or `-1`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer; reads [`gWPArray`]/[`gWPNum`].
pub unsafe fn GetBestIdleGoal(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int = 0;
    let mut highestweight: c_int = 0;
    let mut desiredindex: c_int = -1;
    let mut dist_to_weight: c_int;
    let mut traildist: f32;

    if (*bs).wpCurrent.is_null() {
        return -1;
    }

    if (*bs).isCamper != 2 {
        if ((*bs).randomNavTime as c_int) < (*addr_of!(level)).time {
            if Q_irand(1, 10) < 5 {
                (*bs).randomNav = 1;
            } else {
                (*bs).randomNav = 0;
            }

            (*bs).randomNavTime = (*addr_of!(level)).time + Q_irand(5000, 15000);
        }
    }

    if (*bs).randomNav != 0 {
        //stop looking for items and/or camping on them
        return -1;
    }

    while i < (*addr_of!(gWPNum)) {
        if !(*addr_of!(gWPArray))[i as usize].is_null()
            && (*(*addr_of!(gWPArray))[i as usize]).inuse != 0
            && (*(*addr_of!(gWPArray))[i as usize]).flags & WPFLAG_GOALPOINT != 0
            && (*(*addr_of!(gWPArray))[i as usize]).weight > highestweight as f32
            && BotHasAssociated(bs, (*addr_of!(gWPArray))[i as usize]) == 0
        {
            traildist = TotalTrailDistance((*(*bs).wpCurrent).index, i, bs);

            if traildist != -1.0 {
                dist_to_weight = traildist as c_int / 10000;
                dist_to_weight =
                    ((*(*addr_of!(gWPArray))[i as usize]).weight - dist_to_weight as f32) as c_int;

                if dist_to_weight > highestweight {
                    highestweight = dist_to_weight;
                    desiredindex = i;
                }
            }
        }

        i += 1;
    }

    desiredindex
}

/// `int CTFTakesPriority(bot_state_t *bs)` (ai_main.c:2779) — drive the bot's nav by its CTF role
/// (attacker/defender/retrieval/guard/get-home), re-deriving the role from who holds which flag
/// and the attacker/defender balance, refreshing the dropped-flag waypoints, and honoring a
/// player-forced order. Returns `1` if CTF took the nav goal. Early-spawn weapon gathering can
/// short-circuit or be carried through (`dosw`). The `BOT_CTF_DEBUG` block is omitted.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
// `numOnEnemyTeam`/`numDefenders`/`weHaveEnemyFlag` are tallied but never read (faithful to the
// C); the others feed the role-balance decision.
#[allow(unused_assignments, unused_variables)]
pub unsafe fn CTFTakesPriority(bs: *mut bot_state_t) -> c_int {
    let mut ent: *mut gentity_t;
    let mut enemyFlag: c_int = 0;
    let mut myFlag: c_int = 0;
    let mut enemyHasOurFlag: c_int = 0;
    let mut weHaveEnemyFlag: c_int = 0;
    let mut numOnMyTeam: c_int = 0;
    let mut numOnEnemyTeam: c_int = 0;
    let mut numAttackers: c_int = 0;
    let mut numDefenders: c_int = 0;
    let mut i: c_int;
    let idleWP: c_int;
    let mut dosw: c_int = 0;
    let mut dest_sw: *mut wpobject_t = null_mut();

    if (*addr_of!(g_gametype)).integer != GT_CTF && (*addr_of!(g_gametype)).integer != GT_CTY {
        return 0;
    }

    if (*bs).cur_ps.weapon == WP_BRYAR_PISTOL
        && ((*addr_of!(level)).time - (*bs).lastDeadTime) < BOT_MAX_WEAPON_GATHER_TIME
    {
        //get the nearest weapon laying around base before heading off for battle
        idleWP = GetBestIdleGoal(bs);

        if idleWP != -1
            && !(*addr_of!(gWPArray))[idleWP as usize].is_null()
            && (*(*addr_of!(gWPArray))[idleWP as usize]).inuse != 0
        {
            if ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
                (*bs).wpDestination = (*addr_of!(gWPArray))[idleWP as usize];
            }
            return 1;
        }
    } else if (*bs).cur_ps.weapon == WP_BRYAR_PISTOL
        && ((*addr_of!(level)).time - (*bs).lastDeadTime) < BOT_MAX_WEAPON_CHASE_CTF
        && !(*bs).wpDestination.is_null()
        && (*(*bs).wpDestination).weight != 0.0
    {
        dest_sw = (*bs).wpDestination;
        dosw = 1;
    }

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        myFlag = PW_REDFLAG;
    } else {
        myFlag = PW_BLUEFLAG;
    }

    if (*(*addr_of!(level)).clients.add((*bs).client as usize))
        .sess
        .sessionTeam
        == TEAM_RED
    {
        enemyFlag = PW_BLUEFLAG;
    } else {
        enemyFlag = PW_REDFLAG;
    }

    if (*addr_of!(flagRed)).is_null()
        || (*addr_of!(flagBlue)).is_null()
        || (*(*addr_of!(flagRed))).inuse == 0
        || (*(*addr_of!(flagBlue))).inuse == 0
        || (*addr_of!(eFlagRed)).is_null()
        || (*addr_of!(eFlagBlue)).is_null()
    {
        return 0;
    }

    if !(*addr_of!(droppedRedFlag)).is_null()
        && (*(*addr_of!(droppedRedFlag))).flags & FL_DROPPED_ITEM != 0
    {
        GetNewFlagPoint(*addr_of!(flagRed), *addr_of!(droppedRedFlag), TEAM_RED);
    } else {
        *addr_of_mut!(flagRed) = *addr_of!(oFlagRed);
    }

    if !(*addr_of!(droppedBlueFlag)).is_null()
        && (*(*addr_of!(droppedBlueFlag))).flags & FL_DROPPED_ITEM != 0
    {
        GetNewFlagPoint(*addr_of!(flagBlue), *addr_of!(droppedBlueFlag), TEAM_BLUE);
    } else {
        *addr_of_mut!(flagBlue) = *addr_of!(oFlagBlue);
    }

    if (*bs).ctfState == 0 {
        return 0;
    }

    i = 0;

    while i < MAX_CLIENTS as c_int {
        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null() && !(*ent).client.is_null() {
            if (*(*ent).client).ps.powerups[enemyFlag as usize] != 0
                && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE
            {
                weHaveEnemyFlag = 1;
            } else if (*(*ent).client).ps.powerups[myFlag as usize] != 0
                && OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) == QFALSE
            {
                enemyHasOurFlag = 1;
            }

            if OnSameTeam((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize), ent) != QFALSE {
                numOnMyTeam += 1;
            } else {
                numOnEnemyTeam += 1;
            }

            if !botstates[(*ent).s.number as usize].is_null() {
                if (*botstates[(*ent).s.number as usize]).ctfState == CTFSTATE_ATTACKER
                    || (*botstates[(*ent).s.number as usize]).ctfState == CTFSTATE_RETRIEVAL
                {
                    numAttackers += 1;
                } else {
                    numDefenders += 1;
                }
            } else {
                //assume real players to be attackers in our logic
                numAttackers += 1;
            }
        }
        i += 1;
    }

    if (*bs).cur_ps.powerups[enemyFlag as usize] != 0 {
        if (numOnMyTeam < 2 || numAttackers == 0) && enemyHasOurFlag != 0 {
            (*bs).ctfState = CTFSTATE_RETRIEVAL;
        } else {
            (*bs).ctfState = CTFSTATE_GETFLAGHOME;
        }
    } else if (*bs).ctfState == CTFSTATE_GETFLAGHOME {
        (*bs).ctfState = 0;
    }

    if (*bs).state_Forced != 0 {
        (*bs).ctfState = (*bs).state_Forced;
    }

    let mut success = false;

    'success: {
        if (*bs).ctfState == CTFSTATE_DEFENDER {
            if BotDefendFlag(bs) != 0 {
                success = true;
                break 'success;
            }
        }

        if (*bs).ctfState == CTFSTATE_ATTACKER {
            if BotGetEnemyFlag(bs) != 0 {
                success = true;
                break 'success;
            }
        }

        if (*bs).ctfState == CTFSTATE_RETRIEVAL {
            if BotGetFlagBack(bs) != 0 {
                success = true;
                break 'success;
            } else {
                //can't find anyone on another team being a carrier, so ignore this priority
                (*bs).ctfState = 0;
            }
        }

        if (*bs).ctfState == CTFSTATE_GUARDCARRIER {
            if BotGuardFlagCarrier(bs) != 0 {
                success = true;
                break 'success;
            } else {
                //can't find anyone on our team being a carrier, so ignore this priority
                (*bs).ctfState = 0;
            }
        }

        if (*bs).ctfState == CTFSTATE_GETFLAGHOME {
            if BotGetFlagHome(bs) != 0 {
                success = true;
                break 'success;
            }
        }
    }

    if !success {
        return 0;
    }

    // success:
    if dosw != 0 {
        //allow ctf code to run, but if after a particular item then keep going after it
        (*bs).wpDestination = dest_sw;
    }

    1
}

/// `int SiegeTakesPriority(bot_state_t *bs)` (ai_main.c:3231) — drive siege nav: attackers chase
/// the closest attackable objective (or fall back to defending against attackers if none),
/// defenders defend, and a portion of a defending team is reassigned to attack. Validates a
/// chosen shoot-goal by PVS + trace. Returns `1` in siege. Early-spawn weapon gathering as in
/// [`CTFTakesPriority`].
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
// `flagForDefendableObjective` is computed but never read (faithful to the C).
#[allow(unused_assignments)]
pub unsafe fn SiegeTakesPriority(bs: *mut bot_state_t) -> c_int {
    let attacker: c_int;
    let flagForDefendableObjective: c_int;
    let flagForAttackableObjective: c_int;
    let defenders: c_int;
    let teammates: c_int;
    let idleWP: c_int;
    let mut dest_sw: *mut wpobject_t = null_mut();
    let mut dosw: c_int = 0;
    let bcl: *mut gclient_t;
    let mut dif: vec3_t = vec3_origin;

    if (*addr_of!(g_gametype)).integer != GT_SIEGE {
        return 0;
    }

    bcl = (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client;

    if bcl.is_null() {
        return 0;
    }

    if (*bs).cur_ps.weapon == WP_BRYAR_PISTOL
        && ((*addr_of!(level)).time - (*bs).lastDeadTime) < BOT_MAX_WEAPON_GATHER_TIME
    {
        //get the nearest weapon laying around base before heading off for battle
        idleWP = GetBestIdleGoal(bs);

        if idleWP != -1
            && !(*addr_of!(gWPArray))[idleWP as usize].is_null()
            && (*(*addr_of!(gWPArray))[idleWP as usize]).inuse != 0
        {
            if ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
                (*bs).wpDestination = (*addr_of!(gWPArray))[idleWP as usize];
            }
            return 1;
        }
    } else if (*bs).cur_ps.weapon == WP_BRYAR_PISTOL
        && ((*addr_of!(level)).time - (*bs).lastDeadTime) < BOT_MAX_WEAPON_CHASE_TIME
        && !(*bs).wpDestination.is_null()
        && (*(*bs).wpDestination).weight != 0.0
    {
        dest_sw = (*bs).wpDestination;
        dosw = 1;
    }

    if (*bcl).sess.sessionTeam == SIEGETEAM_TEAM1 {
        attacker = *addr_of!(imperial_attackers);
        flagForDefendableObjective = WPFLAG_SIEGE_REBELOBJ;
        flagForAttackableObjective = WPFLAG_SIEGE_IMPERIALOBJ;
    } else {
        attacker = *addr_of!(rebel_attackers);
        flagForDefendableObjective = WPFLAG_SIEGE_IMPERIALOBJ;
        flagForAttackableObjective = WPFLAG_SIEGE_REBELOBJ;
    }

    if attacker != 0 {
        (*bs).siegeState = SIEGESTATE_ATTACKER;
    } else {
        (*bs).siegeState = SIEGESTATE_DEFENDER;
        defenders = Siege_CountDefenders(bs);
        teammates = Siege_CountTeammates(bs);

        if defenders > teammates / 3 && teammates > 1 {
            //devote around 1/4 of our team to completing our own side goals even if we're a defender.
            //If we have no side goals we will realize that later on and join the defenders
            (*bs).siegeState = SIEGESTATE_ATTACKER;
        }
    }

    if (*bs).state_Forced != 0 {
        (*bs).siegeState = (*bs).state_Forced;
    }

    if (*bs).siegeState == SIEGESTATE_ATTACKER {
        if Siege_TargetClosestObjective(bs, flagForAttackableObjective) == 0 {
            //looks like we have no goals other than to keep the other team from completing objectives
            Siege_DefendFromAttackers(bs);
            if !(*bs).shootGoal.is_null() {
                dif[0] = ((*(*bs).shootGoal).r.absmax[0] + (*(*bs).shootGoal).r.absmin[0]) / 2.0;
                dif[1] = ((*(*bs).shootGoal).r.absmax[1] + (*(*bs).shootGoal).r.absmin[1]) / 2.0;
                dif[2] = ((*(*bs).shootGoal).r.absmax[2] + (*(*bs).shootGoal).r.absmin[2]) / 2.0;

                if BotPVSCheck(&(*bs).origin, &dif) == QFALSE {
                    (*bs).shootGoal = null_mut();
                } else {
                    let tr = trap::Trace(
                        &(*bs).origin,
                        &vec3_origin,
                        &vec3_origin,
                        &dif,
                        (*bs).client,
                        MASK_SOLID,
                    );

                    if tr.fraction != 1.0 && tr.entityNum as c_int != (*(*bs).shootGoal).s.number {
                        (*bs).shootGoal = null_mut();
                    }
                }
            }
        }
    } else if (*bs).siegeState == SIEGESTATE_DEFENDER {
        Siege_DefendFromAttackers(bs);
        if !(*bs).shootGoal.is_null() {
            dif[0] = ((*(*bs).shootGoal).r.absmax[0] + (*(*bs).shootGoal).r.absmin[0]) / 2.0;
            dif[1] = ((*(*bs).shootGoal).r.absmax[1] + (*(*bs).shootGoal).r.absmin[1]) / 2.0;
            dif[2] = ((*(*bs).shootGoal).r.absmax[2] + (*(*bs).shootGoal).r.absmin[2]) / 2.0;

            if BotPVSCheck(&(*bs).origin, &dif) == QFALSE {
                (*bs).shootGoal = null_mut();
            } else {
                let tr = trap::Trace(
                    &(*bs).origin,
                    &vec3_origin,
                    &vec3_origin,
                    &dif,
                    (*bs).client,
                    MASK_SOLID,
                );

                if tr.fraction != 1.0 && tr.entityNum as c_int != (*(*bs).shootGoal).s.number {
                    (*bs).shootGoal = null_mut();
                }
            }
        }
    } else {
        //get busy!
        Siege_TargetClosestObjective(bs, flagForAttackableObjective);
        if !(*bs).shootGoal.is_null() {
            dif[0] = ((*(*bs).shootGoal).r.absmax[0] + (*(*bs).shootGoal).r.absmin[0]) / 2.0;
            dif[1] = ((*(*bs).shootGoal).r.absmax[1] + (*(*bs).shootGoal).r.absmin[1]) / 2.0;
            dif[2] = ((*(*bs).shootGoal).r.absmax[2] + (*(*bs).shootGoal).r.absmin[2]) / 2.0;

            if BotPVSCheck(&(*bs).origin, &dif) == QFALSE {
                (*bs).shootGoal = null_mut();
            } else {
                let tr = trap::Trace(
                    &(*bs).origin,
                    &vec3_origin,
                    &vec3_origin,
                    &dif,
                    (*bs).client,
                    MASK_SOLID,
                );

                if tr.fraction != 1.0 && tr.entityNum as c_int != (*(*bs).shootGoal).s.number {
                    (*bs).shootGoal = null_mut();
                }
            }
        }
    }

    if dosw != 0 {
        //allow siege objective code to run, but if after a particular item then keep going after it
        (*bs).wpDestination = dest_sw;
    }

    let _ = flagForDefendableObjective;

    1
}

/// `int JMTakesPriority(bot_state_t *bs)` (ai_main.c:3399) — in Jedi Master, route a non-Master
/// bot toward the saber's location (the carrier client found by scanning for `isJediMaster`, or
/// the free saber entity [`gJMSaberEnt`] when nobody holds it) on a refresh timer. Returns `0`
/// outside JM mode or when the bot itself is the Jedi Master, else `1`.
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
// `wpClose`'s `= -1` initializer is a faithful C dead store (reassigned before use).
#[allow(unused_assignments)]
pub unsafe fn JMTakesPriority(bs: *mut bot_state_t) -> c_int {
    let mut i: c_int = 0;
    let mut wpClose: c_int = -1;
    let theImportantEntity: *mut gentity_t;

    if (*addr_of!(g_gametype)).integer != GT_JEDIMASTER {
        return 0;
    }

    if (*bs).cur_ps.isJediMaster != QFALSE {
        return 0;
    }

    //jmState becomes the index for the one who carries the saber. If jmState is -1 then the saber is currently
    //without an owner
    (*bs).jmState = -1;

    while i < MAX_CLIENTS as c_int {
        if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize)).client.is_null()
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize)).inuse != 0
            && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize)).client).ps.isJediMaster != QFALSE
        {
            (*bs).jmState = i;
            break;
        }

        i += 1;
    }

    if (*bs).jmState != -1 {
        theImportantEntity = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).jmState as usize);
    } else {
        theImportantEntity = *addr_of!(gJMSaberEnt);
    }

    if !theImportantEntity.is_null()
        && (*theImportantEntity).inuse != 0
        && ((*bs).destinationGrabTime as c_int) < (*addr_of!(level)).time
    {
        if !(*theImportantEntity).client.is_null() {
            wpClose = GetNearestVisibleWP(
                &(*(*theImportantEntity).client).ps.origin,
                (*theImportantEntity).s.number,
            );
        } else {
            wpClose = GetNearestVisibleWP(
                &(*theImportantEntity).r.currentOrigin,
                (*theImportantEntity).s.number,
            );
        }

        if wpClose != -1
            && !(*addr_of!(gWPArray))[wpClose as usize].is_null()
            && (*(*addr_of!(gWPArray))[wpClose as usize]).inuse != 0
        {
            (*bs).wpDestination = (*addr_of!(gWPArray))[wpClose as usize];
            (*bs).destinationGrabTime = ((*addr_of!(level)).time + 4000) as f32;
        }
    }

    1
}

/// `void GetIdealDestination(bot_state_t *bs)` (ai_main.c:3601) — the top-level nav-goal selector:
/// flee a nearby `GetNearestBadThing` (reversing waypoint direction), else honor camping / CTF /
/// siege / JM priorities, else chase a revenge target, squad leader, or current enemy (closing or
/// fleeing per [`BotIsAChickenWuss`] and weapon range), else fall back to an idle goal. The
/// `_DEBUG` `bot_nogoals` early-out is omitted (release build).
///
/// Note: the `revengeEnemy`/`squadLeader` connection check compares `pers.connected` against
/// `CA_ACTIVE`/`CA_AUTHORIZING` (`connstate_t`), not `CON_CONNECTED` — a faithful reproduction of
/// the original JKA cross-enum comparison (OpenJK fixed this to `CON_CONNECTED`).
///
/// # Safety
/// `bs` must be a valid [`bot_state_t`] pointer.
#[allow(unused_assignments)]
pub unsafe fn GetIdealDestination(bs: *mut bot_state_t) {
    let mut tempInt: c_int;
    let cWPIndex: c_int;
    let bChicken: c_int;
    let mut idleWP: c_int;
    let mut distChange: f32;
    let plusLen: f32;
    let minusLen: f32;
    let mut usethisvec: vec3_t = vec3_origin;
    let mut a: vec3_t = vec3_origin;
    let badthing: *mut gentity_t;

    if (*bs).wpCurrent.is_null() {
        return;
    }

    if ((*addr_of!(level)).time - (*bs).escapeDirTime as c_int) > 4000 {
        badthing = GetNearestBadThing(bs);
    } else {
        badthing = null_mut();
    }

    if !badthing.is_null()
        && (*badthing).inuse != 0
        && (*badthing).health > 0
        && (*badthing).takedamage != QFALSE
    {
        (*bs).dangerousObject = badthing;
    } else {
        (*bs).dangerousObject = null_mut();
    }

    if badthing.is_null() && (*bs).wpDestIgnoreTime as c_int > (*addr_of!(level)).time {
        return;
    }

    if badthing.is_null() && (*bs).dontGoBack as c_int > (*addr_of!(level)).time {
        if !(*bs).wpDestination.is_null() {
            (*bs).wpStoreDest = (*bs).wpDestination;
        }
        (*bs).wpDestination = null_mut();
        return;
    } else if badthing.is_null() && !(*bs).wpStoreDest.is_null() {
        //after we finish running away, switch back to our original destination
        (*bs).wpDestination = (*bs).wpStoreDest;
        (*bs).wpStoreDest = null_mut();
    }

    if !badthing.is_null() && !(*bs).wpCamping.is_null() {
        (*bs).wpCamping = null_mut();
    }

    if !(*bs).wpCamping.is_null() {
        (*bs).wpDestination = (*bs).wpCamping;
        return;
    }

    if badthing.is_null() && CTFTakesPriority(bs) != 0 {
        if (*bs).ctfState != 0 {
            (*bs).runningToEscapeThreat = 1;
        }
        return;
    } else if badthing.is_null() && SiegeTakesPriority(bs) != 0 {
        if (*bs).siegeState != 0 {
            (*bs).runningToEscapeThreat = 1;
        }
        return;
    } else if badthing.is_null() && JMTakesPriority(bs) != 0 {
        (*bs).runningToEscapeThreat = 1;
    }

    if !badthing.is_null() {
        (*bs).runningLikeASissy = (*addr_of!(level)).time + 100;

        if !(*bs).wpDestination.is_null() {
            (*bs).wpStoreDest = (*bs).wpDestination;
        }
        (*bs).wpDestination = null_mut();

        if (*bs).wpDirection != 0 {
            tempInt = (*(*bs).wpCurrent).index + 1;
        } else {
            tempInt = (*(*bs).wpCurrent).index - 1;
        }

        if !(*addr_of!(gWPArray))[tempInt as usize].is_null()
            && (*(*addr_of!(gWPArray))[tempInt as usize]).inuse != 0
            && ((*bs).escapeDirTime as c_int) < (*addr_of!(level)).time
        {
            VectorSubtract(&(*badthing).s.pos.trBase, &(*(*bs).wpCurrent).origin, &mut a);
            plusLen = VectorLength(&a);
            VectorSubtract(
                &(*badthing).s.pos.trBase,
                &(*(*addr_of!(gWPArray))[tempInt as usize]).origin,
                &mut a,
            );
            minusLen = VectorLength(&a);

            if plusLen < minusLen {
                if (*bs).wpDirection != 0 {
                    (*bs).wpDirection = 0;
                } else {
                    (*bs).wpDirection = 1;
                }

                (*bs).wpCurrent = (*addr_of!(gWPArray))[tempInt as usize];

                (*bs).escapeDirTime = ((*addr_of!(level)).time + Q_irand(500, 1000)) as f32;
            }
        }
        return;
    }

    distChange = 0.0; //keep the compiler from complaining

    tempInt = BotGetWeaponRange(bs);

    if tempInt == BWEAPONRANGE_MELEE {
        distChange = 1.0;
    } else if tempInt == BWEAPONRANGE_SABER {
        distChange = 1.0;
    } else if tempInt == BWEAPONRANGE_MID {
        distChange = 128.0;
    } else if tempInt == BWEAPONRANGE_LONG {
        distChange = 300.0;
    }

    if !(*bs).revengeEnemy.is_null()
        && (*(*bs).revengeEnemy).health > 0
        && !(*(*bs).revengeEnemy).client.is_null()
        && ((*(*(*bs).revengeEnemy).client).pers.connected == CA_ACTIVE
            || (*(*(*bs).revengeEnemy).client).pers.connected == CA_AUTHORIZING)
    {
        //if we hate someone, always try to get to them
        if ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
            if !(*(*bs).revengeEnemy).client.is_null() {
                VectorCopy(&(*(*(*bs).revengeEnemy).client).ps.origin, &mut usethisvec);
            } else {
                VectorCopy(&(*(*bs).revengeEnemy).s.origin, &mut usethisvec);
            }

            tempInt = GetNearestVisibleWP(&usethisvec, 0);

            if tempInt != -1 && TotalTrailDistance((*(*bs).wpCurrent).index, tempInt, bs) != -1.0 {
                (*bs).wpDestination = (*addr_of!(gWPArray))[tempInt as usize];
                (*bs).wpDestSwitchTime = ((*addr_of!(level)).time + Q_irand(5000, 10000)) as f32;
            }
        }
    } else if !(*bs).squadLeader.is_null()
        && (*(*bs).squadLeader).health > 0
        && !(*(*bs).squadLeader).client.is_null()
        && ((*(*(*bs).squadLeader).client).pers.connected == CA_ACTIVE
            || (*(*(*bs).squadLeader).client).pers.connected == CA_AUTHORIZING)
    {
        if ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
            if !(*(*bs).squadLeader).client.is_null() {
                VectorCopy(&(*(*(*bs).squadLeader).client).ps.origin, &mut usethisvec);
            } else {
                VectorCopy(&(*(*bs).squadLeader).s.origin, &mut usethisvec);
            }

            tempInt = GetNearestVisibleWP(&usethisvec, 0);

            if tempInt != -1 && TotalTrailDistance((*(*bs).wpCurrent).index, tempInt, bs) != -1.0 {
                (*bs).wpDestination = (*addr_of!(gWPArray))[tempInt as usize];
                (*bs).wpDestSwitchTime = ((*addr_of!(level)).time + Q_irand(5000, 10000)) as f32;
            }
        }
    } else if !(*bs).currentEnemy.is_null() {
        if !(*(*bs).currentEnemy).client.is_null() {
            VectorCopy(&(*(*(*bs).currentEnemy).client).ps.origin, &mut usethisvec);
        } else {
            VectorCopy(&(*(*bs).currentEnemy).s.origin, &mut usethisvec);
        }

        bChicken = BotIsAChickenWuss(bs);
        (*bs).runningToEscapeThreat = bChicken;

        if (*bs).frame_Enemy_Len < distChange || (bChicken != 0 && bChicken != 2) {
            cWPIndex = (*(*bs).wpCurrent).index;

            if (*bs).frame_Enemy_Len > 400.0 {
                //good distance away, start running toward a good place for an item or powerup or whatever
                idleWP = GetBestIdleGoal(bs);

                if idleWP != -1
                    && !(*addr_of!(gWPArray))[idleWP as usize].is_null()
                    && (*(*addr_of!(gWPArray))[idleWP as usize]).inuse != 0
                {
                    (*bs).wpDestination = (*addr_of!(gWPArray))[idleWP as usize];
                }
            } else if !(*addr_of!(gWPArray))[(cWPIndex - 1) as usize].is_null()
                && (*(*addr_of!(gWPArray))[(cWPIndex - 1) as usize]).inuse != 0
                && !(*addr_of!(gWPArray))[(cWPIndex + 1) as usize].is_null()
                && (*(*addr_of!(gWPArray))[(cWPIndex + 1) as usize]).inuse != 0
            {
                VectorSubtract(
                    &(*(*addr_of!(gWPArray))[(cWPIndex + 1) as usize]).origin,
                    &usethisvec,
                    &mut a,
                );
                plusLen = VectorLength(&a);
                VectorSubtract(
                    &(*(*addr_of!(gWPArray))[(cWPIndex - 1) as usize]).origin,
                    &usethisvec,
                    &mut a,
                );
                minusLen = VectorLength(&a);

                if minusLen > plusLen {
                    (*bs).wpDestination = (*addr_of!(gWPArray))[(cWPIndex - 1) as usize];
                } else {
                    (*bs).wpDestination = (*addr_of!(gWPArray))[(cWPIndex + 1) as usize];
                }
            }
        } else if bChicken != 2 && ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
            tempInt = GetNearestVisibleWP(&usethisvec, 0);

            if tempInt != -1 && TotalTrailDistance((*(*bs).wpCurrent).index, tempInt, bs) != -1.0 {
                (*bs).wpDestination = (*addr_of!(gWPArray))[tempInt as usize];

                if (*addr_of!(g_gametype)).integer == GT_SINGLE_PLAYER {
                    //be more aggressive
                    (*bs).wpDestSwitchTime = ((*addr_of!(level)).time + Q_irand(300, 1000)) as f32;
                } else {
                    (*bs).wpDestSwitchTime = ((*addr_of!(level)).time + Q_irand(1000, 5000)) as f32;
                }
            }
        }
    }

    if (*bs).wpDestination.is_null() && ((*bs).wpDestSwitchTime as c_int) < (*addr_of!(level)).time {
        idleWP = GetBestIdleGoal(bs);

        if idleWP != -1
            && !(*addr_of!(gWPArray))[idleWP as usize].is_null()
            && (*(*addr_of!(gWPArray))[idleWP as usize]).inuse != 0
        {
            (*bs).wpDestination = (*addr_of!(gWPArray))[idleWP as usize];
        }
    }
}

/// `void StandardBotAI(bot_state_t *bs, float thinktime)` (ai_main.c:5915) — the master
/// per-frame bot think routine. Faithful line-by-line port of the JKA original.
pub unsafe fn StandardBotAI(bs: *mut bot_state_t, thinktime: f32) {
    let wp: c_int;
    let enemy: c_int;
    let desiredIndex: c_int;
    let goalWPIndex: c_int;
    let mut doingFallback: c_int;
    let mut fjHalt: c_int;
    let mut a: vec3_t = vec3_origin;
    let mut ang: vec3_t = vec3_origin;
    let mut headlevel: vec3_t = vec3_origin;
    let mut eorg: vec3_t = vec3_origin;
    let mut noz_x: vec3_t = vec3_origin;
    let mut noz_y: vec3_t = vec3_origin;
    let mut dif: vec3_t = vec3_origin;
    let mut a_fo: vec3_t = vec3_origin;
    let reaction: f32;
    let bLeadAmount: f32;
    let mut meleestrafe: c_int = 0;
    let mut useTheForce: c_int = 0;
    let mut forceHostile: c_int = 0;
    // `cBAI` captures `CombatBotAI`'s return value but is never read (mirrors the C local).
    #[allow(unused_variables, unused_assignments)]
    let mut cBAI: c_int = 0;
    let friendInLOF: *mut gentity_t;
    let mLen: f32;
    let visResult: c_int;
    let mut selResult: c_int = 0;
    let mut mineSelect: c_int;
    let mut detSelect: c_int;
    let mut preFrameGAngles: vec3_t = vec3_origin;

    if gDeactivated != 0.0 {
        (*bs).wpCurrent = null_mut();
        (*bs).currentEnemy = null_mut();
        (*bs).wpDestination = null_mut();
        (*bs).wpDirection = 0;
        return;
    }

    if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).inuse != QFALSE
        && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client.is_null()
        && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
            .sess
            .sessionTeam
            == TEAM_SPECTATOR
    {
        (*bs).wpCurrent = null_mut();
        (*bs).currentEnemy = null_mut();
        (*bs).wpDestination = null_mut();
        (*bs).wpDirection = 0;
        return;
    }

    // #ifndef FINAL_BUILD
    if (*addr_of!(bot_getinthecarrr)).integer != 0 {
        //stupid vehicle debug, I tire of having to connect another client to test passengers.
        let botEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize);

        if (*botEnt).inuse != QFALSE
            && !(*botEnt).client.is_null()
            && (*(*botEnt).client).ps.m_iVehicleNum != 0
        {
            //in a vehicle, so...
            (*bs).noUseTime = (*addr_of!(level)).time + 5000;

            if (*addr_of!(bot_getinthecarrr)).integer != 2 {
                trap::ea::EA_MoveForward((*bs).client);

                if (*addr_of!(bot_getinthecarrr)).integer == 3 {
                    //use alt fire
                    trap::ea::EA_Alt_Attack((*bs).client);
                }
            }
        } else {
            //find one, get in
            let mut i: c_int = 0;
            let mut vehicle: *mut gentity_t;
            //find the nearest, manned vehicle
            loop {
                if !((i as usize) < MAX_GENTITIES) {
                    break;
                }
                vehicle = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

                if (*vehicle).inuse != QFALSE
                    && !(*vehicle).client.is_null()
                    && (*vehicle).s.eType == ET_NPC
                    && (*vehicle).s.NPC_class == CLASS_VEHICLE
                    && !(*vehicle).m_pVehicle.is_null()
                    && ((*(*vehicle).client).ps.m_iVehicleNum != 0
                        || (*addr_of!(bot_getinthecarrr)).integer == 2)
                {
                    //ok, this is a vehicle, and it has a pilot/passengers
                    break;
                }
                i += 1;
            }
            if i as usize != MAX_GENTITIES {
                //broke before end so we must've found something
                let vehicle: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
                let mut v: vec3_t = vec3_origin;

                VectorSubtract(&(*(*vehicle).client).ps.origin, &(*bs).origin, &mut v);
                VectorNormalize(&mut v);
                let v_copy = v;
                vectoangles(&v_copy, &mut (*bs).goalAngles);
                MoveTowardIdealAngles(bs);
                trap::ea::EA_Move((*bs).client, &v, 5000.0);

                if ((*bs).noUseTime) < ((*addr_of!(level)).time - 400) {
                    (*bs).noUseTime = (*addr_of!(level)).time + 500;
                }
            }
        }

        return;
    }
    // #endif

    if (*addr_of!(bot_forgimmick)).integer != 0 {
        (*bs).wpCurrent = null_mut();
        (*bs).currentEnemy = null_mut();
        (*bs).wpDestination = null_mut();
        (*bs).wpDirection = 0;

        if (*addr_of!(bot_forgimmick)).integer == 2 {
            //for debugging saber stuff, this is handy
            trap::ea::EA_Attack((*bs).client);
        }

        if (*addr_of!(bot_forgimmick)).integer == 3 {
            //for testing cpu usage moving around rmg terrain without AI
            let mut mdir: vec3_t = vec3_origin;

            VectorSubtract(&(*bs).origin, &vec3_origin, &mut mdir);
            VectorNormalize(&mut mdir);
            trap::ea::EA_Attack((*bs).client);
            trap::ea::EA_Move((*bs).client, &mdir, 5000.0);
        }

        if (*addr_of!(bot_forgimmick)).integer == 4 {
            //constantly move toward client 0
            if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0)).client.is_null()
                && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0)).inuse != QFALSE
            {
                let mut mdir: vec3_t = vec3_origin;

                VectorSubtract(
                    &(*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0)).client).ps.origin,
                    &(*bs).origin,
                    &mut mdir,
                );
                VectorNormalize(&mut mdir);
                trap::ea::EA_Move((*bs).client, &mdir, 5000.0);
            }
        }

        if (*bs).forceMove_Forward != 0 {
            if (*bs).forceMove_Forward > 0 {
                trap::ea::EA_MoveForward((*bs).client);
            } else {
                trap::ea::EA_MoveBack((*bs).client);
            }
        }
        if (*bs).forceMove_Right != 0 {
            if (*bs).forceMove_Right > 0 {
                trap::ea::EA_MoveRight((*bs).client);
            } else {
                trap::ea::EA_MoveLeft((*bs).client);
            }
        }
        if (*bs).forceMove_Up != 0 {
            trap::ea::EA_Jump((*bs).client);
        }
        return;
    }

    if (*bs).lastDeadTime == 0 {
        //just spawned in?
        (*bs).lastDeadTime = (*addr_of!(level)).time;
    }

    if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < 1 {
        (*bs).lastDeadTime = (*addr_of!(level)).time;

        if (*bs).deathActivitiesDone == 0
            && !(*bs).lastHurt.is_null()
            && !(*(*bs).lastHurt).client.is_null()
            && (*(*bs).lastHurt).s.number != (*bs).client
        {
            BotDeathNotify(bs);
            if PassLovedOneCheck(bs, (*bs).lastHurt) != 0 {
                //CHAT: Died
                (*bs).chatObject = (*bs).lastHurt;
                (*bs).chatAltObject = null_mut();
                BotDoChat(bs, c"Died".as_ptr() as *mut c_char, 0);
            } else if PassLovedOneCheck(bs, (*bs).lastHurt) == 0
                && !(*addr_of!(botstates))[(*(*bs).lastHurt).s.number as usize].is_null()
                && PassLovedOneCheck(
                    (*addr_of!(botstates))[(*(*bs).lastHurt).s.number as usize],
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize),
                ) != 0
            {
                //killed by a bot that I love, but that does not love me
                (*bs).chatObject = (*bs).lastHurt;
                (*bs).chatAltObject = null_mut();
                BotDoChat(bs, c"KilledOnPurposeByLove".as_ptr() as *mut c_char, 0);
            }

            (*bs).deathActivitiesDone = 1;
        }

        (*bs).wpCurrent = null_mut();
        (*bs).currentEnemy = null_mut();
        (*bs).wpDestination = null_mut();
        (*bs).wpCamping = null_mut();
        (*bs).wpCampingTo = null_mut();
        (*bs).wpStoreDest = null_mut();
        (*bs).wpDestIgnoreTime = 0.0;
        (*bs).wpDestSwitchTime = 0.0;
        (*bs).wpSeenTime = 0.0;
        (*bs).wpDirection = 0;

        if rand() % 10 < 5 && ((*bs).doChat == 0 || (*bs).chatTime < (*addr_of!(level)).time as f32)
        {
            trap::ea::EA_Attack((*bs).client);
        }

        return;
    }

    VectorCopy(&(*bs).goalAngles, &mut preFrameGAngles);

    (*bs).doAttack = 0;
    (*bs).doAltAttack = 0;
    //reset the attack states

    if (*bs).isSquadLeader != 0 {
        CommanderBotAI(bs);
    } else {
        BotDoTeamplayAI(bs);
    }

    if (*bs).currentEnemy.is_null() {
        (*bs).frame_Enemy_Vis = 0;
    }

    if !(*bs).revengeEnemy.is_null()
        && !(*(*bs).revengeEnemy).client.is_null()
        && (*(*(*bs).revengeEnemy).client).pers.connected != CA_ACTIVE
        && (*(*(*bs).revengeEnemy).client).pers.connected != CA_AUTHORIZING
    {
        (*bs).revengeEnemy = null_mut();
        (*bs).revengeHateLevel = 0;
    }

    if !(*bs).currentEnemy.is_null()
        && !(*(*bs).currentEnemy).client.is_null()
        && (*(*(*bs).currentEnemy).client).pers.connected != CA_ACTIVE
        && (*(*(*bs).currentEnemy).client).pers.connected != CA_AUTHORIZING
    {
        (*bs).currentEnemy = null_mut();
    }

    fjHalt = 0;

    // #ifndef FORCEJUMP_INSTANTMETHOD (undefined -> this branch)
    if (*bs).forceJumpChargeTime > (*addr_of!(level)).time {
        useTheForce = 1;
        forceHostile = 0;
    }

    if !(*bs).currentEnemy.is_null()
        && !(*(*bs).currentEnemy).client.is_null()
        && (*bs).frame_Enemy_Vis != 0
        && (*bs).forceJumpChargeTime < (*addr_of!(level)).time
    {
        VectorSubtract(&(*(*(*bs).currentEnemy).client).ps.origin, &(*bs).eye, &mut a_fo);
        let a_fo_copy = a_fo;
        vectoangles(&a_fo_copy, &mut a_fo);

        //do this above all things
        if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_PUSH)) != 0
            && ((*bs).doForcePush > (*addr_of!(level)).time
                || (*bs).cur_ps.fd.forceGripBeingGripped > (*addr_of!(level)).time as f32)
            && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .ps
                .fd
                .forcePower
                > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerLevel[FP_PUSH as usize] as usize][FP_PUSH as usize]
        /*&& InFieldOfVision(bs->viewangles, 50, a_fo)*/
        {
            (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .ps
                .fd
                .forcePowerSelected = FP_PUSH;
            useTheForce = 1;
            forceHostile = 1;
        } else if (*bs).cur_ps.fd.forceSide == FORCE_DARKSIDE {
            //try dark side powers
            //in order of priority top to bottom
            if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_GRIP)) != 0
                && ((*bs).cur_ps.fd.forcePowersActive & (1 << FP_GRIP)) != 0
                && InFieldOfVision(&(*bs).viewangles, 50.0, &mut a_fo) != 0
            {
                //already gripping someone, so hold it
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_GRIP;
                useTheForce = 1;
                forceHostile = 1;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_LIGHTNING)) != 0
                && (*bs).frame_Enemy_Len < FORCE_LIGHTNING_RADIUS as f32
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > 50
                && InFieldOfVision(&(*bs).viewangles, 50.0, &mut a_fo) != 0
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_LIGHTNING;
                useTheForce = 1;
                forceHostile = 1;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_GRIP)) != 0
                && (*bs).frame_Enemy_Len < MAX_GRIP_DISTANCE as f32
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_GRIP as usize]
                        as usize][FP_GRIP as usize]
                && InFieldOfVision(&(*bs).viewangles, 50.0, &mut a_fo) != 0
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_GRIP;
                useTheForce = 1;
                forceHostile = 1;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_RAGE)) != 0
                && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < 25
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_RAGE as usize]
                        as usize][FP_RAGE as usize]
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_RAGE;
                useTheForce = 1;
                forceHostile = 0;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_DRAIN)) != 0
                && (*bs).frame_Enemy_Len < MAX_DRAIN_DISTANCE as f32
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > 50
                && InFieldOfVision(&(*bs).viewangles, 50.0, &mut a_fo) != 0
                && (*(*(*bs).currentEnemy).client).ps.fd.forcePower > 10
                && (*(*(*bs).currentEnemy).client).ps.fd.forceSide == FORCE_LIGHTSIDE
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_DRAIN;
                useTheForce = 1;
                forceHostile = 1;
            }
        } else if (*bs).cur_ps.fd.forceSide == FORCE_LIGHTSIDE {
            //try light side powers
            if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_ABSORB)) != 0
                && (*bs).cur_ps.fd.forceGripCripple != 0
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_ABSORB as usize]
                        as usize][FP_ABSORB as usize]
            {
                //absorb to get out
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_ABSORB;
                useTheForce = 1;
                forceHostile = 0;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_ABSORB)) != 0
                && (*bs).cur_ps.electrifyTime >= (*addr_of!(level)).time
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_ABSORB as usize]
                        as usize][FP_ABSORB as usize]
            {
                //absorb lightning
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_ABSORB;
                useTheForce = 1;
                forceHostile = 0;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_TELEPATHY)) != 0
                && (*bs).frame_Enemy_Len < MAX_TRICK_DISTANCE as f32
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_TELEPATHY as usize]
                        as usize][FP_TELEPATHY as usize]
                && InFieldOfVision(&(*bs).viewangles, 50.0, &mut a_fo) != 0
                && ((*(*(*bs).currentEnemy).client).ps.fd.forcePowersActive & (1 << FP_SEE)) == 0
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_TELEPATHY;
                useTheForce = 1;
                forceHostile = 1;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_ABSORB)) != 0
                && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < 75
                && (*(*(*bs).currentEnemy).client).ps.fd.forceSide == FORCE_DARKSIDE
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_ABSORB as usize]
                        as usize][FP_ABSORB as usize]
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_ABSORB;
                useTheForce = 1;
                forceHostile = 0;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_PROTECT)) != 0
                && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < 35
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_PROTECT as usize]
                        as usize][FP_PROTECT as usize]
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_PROTECT;
                useTheForce = 1;
                forceHostile = 0;
            }
        }

        if useTheForce == 0 {
            //try neutral powers
            if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_PUSH)) != 0
                && (*bs).cur_ps.fd.forceGripBeingGripped > (*addr_of!(level)).time as f32
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_PUSH as usize]
                        as usize][FP_PUSH as usize]
                && InFieldOfVision(&(*bs).viewangles, 50.0, &mut a_fo) != 0
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_PUSH;
                useTheForce = 1;
                forceHostile = 1;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_SPEED)) != 0
                && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < 25
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_SPEED as usize]
                        as usize][FP_SPEED as usize]
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_SPEED;
                useTheForce = 1;
                forceHostile = 0;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_SEE)) != 0
                && BotMindTricked((*bs).client, (*(*bs).currentEnemy).s.number) != 0
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_SEE as usize]
                        as usize][FP_SEE as usize]
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_SEE;
                useTheForce = 1;
                forceHostile = 0;
            } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_PULL)) != 0
                && (*bs).frame_Enemy_Len < 256.0
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > 75
                && InFieldOfVision(&(*bs).viewangles, 50.0, &mut a_fo) != 0
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_PULL;
                useTheForce = 1;
                forceHostile = 1;
            }
        }
    }

    if useTheForce == 0 {
        //try powers that we don't care if we have an enemy for
        if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_HEAL)) != 0
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < 50
            && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .ps
                .fd
                .forcePower
                > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerLevel[FP_HEAL as usize]
                    as usize][FP_HEAL as usize]
            && (*bs).cur_ps.fd.forcePowerLevel[FP_HEAL as usize] > FORCE_LEVEL_1
        {
            (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .ps
                .fd
                .forcePowerSelected = FP_HEAL;
            useTheForce = 1;
            forceHostile = 0;
        } else if ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_HEAL)) != 0
            && (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).health < 50
            && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .ps
                .fd
                .forcePower
                > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerLevel[FP_HEAL as usize]
                    as usize][FP_HEAL as usize]
            && (*bs).currentEnemy.is_null()
            && (*bs).isCamping > (*addr_of!(level)).time as f32
        {
            //only meditate and heal if we're camping
            (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .ps
                .fd
                .forcePowerSelected = FP_HEAL;
            useTheForce = 1;
            forceHostile = 0;
        }
    }

    if useTheForce != 0 && forceHostile != 0 {
        if !(*bs).currentEnemy.is_null()
            && !(*(*bs).currentEnemy).client.is_null()
            && ForcePowerUsableOn(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize),
                (*bs).currentEnemy,
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected,
            ) == 0
        {
            useTheForce = 0;
            forceHostile = 0;
        }
    }

    doingFallback = 0;

    (*bs).deathActivitiesDone = 0;

    if BotUseInventoryItem(bs) != 0 {
        if rand() % 10 < 5 {
            trap::ea::EA_Use((*bs).client);
        }
    }

    if (*bs).cur_ps.ammo
        [weaponData[(*bs).cur_ps.weapon as usize].ammoIndex as usize]
        < weaponData[(*bs).cur_ps.weapon as usize].energyPerShot
    {
        if BotTryAnotherWeapon(bs) != 0 {
            return;
        }
    } else {
        if !(*bs).currentEnemy.is_null()
            && (*bs).lastVisibleEnemyIndex == (*(*bs).currentEnemy).s.number
            && (*bs).frame_Enemy_Vis != 0
            && (*bs).forceWeaponSelect != 0
        /*&& bs->plantContinue < level.time*/
        {
            (*bs).forceWeaponSelect = 0;
        }

        if (*bs).plantContinue > (*addr_of!(level)).time {
            (*bs).doAttack = 1;
            (*bs).destinationGrabTime = 0.0;
        }

        if (*bs).forceWeaponSelect == 0
            && (*bs).cur_ps.hasDetPackPlanted != QFALSE
            && (*bs).plantKillEmAll > (*addr_of!(level)).time
        {
            (*bs).forceWeaponSelect = WP_DET_PACK;
        }

        if (*bs).forceWeaponSelect != 0 {
            selResult = BotSelectChoiceWeapon(bs, (*bs).forceWeaponSelect, 1);
        }

        if selResult != 0 {
            if selResult == 2 {
                //newly selected
                return;
            }
        } else if BotSelectIdealWeapon(bs) != 0 {
            return;
        }
    }
    /*if (BotSelectMelee(bs))
    {
        return;
    }*/

    reaction = (*bs).skills.reflex as f32 / (*bs).settings.skill;

    let mut reaction = reaction;
    if reaction < 0.0 {
        reaction = 0.0;
    }
    if reaction > 2000.0 {
        reaction = 2000.0;
    }

    if (*bs).currentEnemy.is_null() {
        (*bs).timeToReact = (*addr_of!(level)).time as f32 + reaction;
    }

    if (*bs).cur_ps.weapon == WP_DET_PACK
        && (*bs).cur_ps.hasDetPackPlanted != QFALSE
        && (*bs).plantKillEmAll > (*addr_of!(level)).time
    {
        (*bs).doAltAttack = 1;
    }

    if !(*bs).wpCamping.is_null() {
        if (*bs).isCamping < (*addr_of!(level)).time as f32 {
            (*bs).wpCamping = null_mut();
            (*bs).isCamping = 0.0;
        }

        if !(*bs).currentEnemy.is_null() && (*bs).frame_Enemy_Vis != 0 {
            (*bs).wpCamping = null_mut();
            (*bs).isCamping = 0.0;
        }
    }

    if !(*bs).wpCurrent.is_null()
        && ((*bs).wpSeenTime < (*addr_of!(level)).time as f32
            || (*bs).wpTravelTime < (*addr_of!(level)).time as f32)
    {
        (*bs).wpCurrent = null_mut();
    }

    if !(*bs).currentEnemy.is_null() {
        if (*bs).enemySeenTime < (*addr_of!(level)).time as f32
            || PassStandardEnemyChecks(bs, (*bs).currentEnemy) == 0
        {
            if (*bs).revengeEnemy == (*bs).currentEnemy
                && (*(*bs).currentEnemy).health < 1
                && !(*bs).lastAttacked.is_null()
                && (*bs).lastAttacked == (*bs).currentEnemy
            {
                //CHAT: Destroyed hated one [KilledHatedOne section]
                (*bs).chatObject = (*bs).revengeEnemy;
                (*bs).chatAltObject = null_mut();
                BotDoChat(bs, c"KilledHatedOne".as_ptr() as *mut c_char, 1);
                (*bs).revengeEnemy = null_mut();
                (*bs).revengeHateLevel = 0;
            } else if (*(*bs).currentEnemy).health < 1
                && PassLovedOneCheck(bs, (*bs).currentEnemy) != 0
                && !(*bs).lastAttacked.is_null()
                && (*bs).lastAttacked == (*bs).currentEnemy
            {
                //CHAT: Killed
                (*bs).chatObject = (*bs).currentEnemy;
                (*bs).chatAltObject = null_mut();
                BotDoChat(bs, c"Killed".as_ptr() as *mut c_char, 0);
            }

            (*bs).currentEnemy = null_mut();
        }
    }

    if (*addr_of!(bot_honorableduelacceptance)).integer != 0 {
        if !(*bs).currentEnemy.is_null()
            && !(*(*bs).currentEnemy).client.is_null()
            && (*bs).cur_ps.weapon == WP_SABER
            && (*addr_of!(g_privateDuel)).integer != 0
            && (*bs).frame_Enemy_Vis != 0
            && (*bs).frame_Enemy_Len < 400.0
            && (*(*(*bs).currentEnemy).client).ps.weapon == WP_SABER
            && (*(*(*bs).currentEnemy).client).ps.saberHolstered != 0
        {
            let mut e_ang_vec: vec3_t = vec3_origin;

            VectorSubtract(
                &(*(*(*bs).currentEnemy).client).ps.origin,
                &(*bs).eye,
                &mut e_ang_vec,
            );

            if InFieldOfVision(&(*bs).viewangles, 100.0, &mut e_ang_vec) != 0 {
                //Our enemy has his saber holstered and has challenged us to a duel, so challenge him back
                if (*bs).cur_ps.saberHolstered == 0 {
                    Cmd_ToggleSaber_f((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize));
                } else {
                    if (*(*(*bs).currentEnemy).client).ps.duelIndex == (*bs).client
                        && (*(*(*bs).currentEnemy).client).ps.duelTime > (*addr_of!(level)).time
                        && (*bs).cur_ps.duelInProgress == QFALSE
                    {
                        Cmd_EngageDuel_f((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize));
                    }
                }

                (*bs).doAttack = 0;
                (*bs).doAltAttack = 0;
                (*bs).botChallengingTime = (*addr_of!(level)).time + 100;
                (*bs).beStill = ((*addr_of!(level)).time + 100) as f32;
            }
        }
    }
    //Apparently this "allows you to cheese" when fighting against bots. I'm not sure why you'd want to con bots
    //into an easy kill, since they're bots and all. But whatever.

    if (*bs).wpCurrent.is_null() {
        wp = GetNearestVisibleWP(&(*bs).origin, (*bs).client);

        if wp != -1 {
            (*bs).wpCurrent = (*addr_of!(gWPArray))[wp as usize];
            (*bs).wpSeenTime = ((*addr_of!(level)).time + 1500) as f32;
            (*bs).wpTravelTime = ((*addr_of!(level)).time + 10000) as f32; //never take more than 10 seconds to travel to a waypoint
        }
    }

    if (*bs).enemySeenTime < (*addr_of!(level)).time as f32
        || (*bs).frame_Enemy_Vis == 0
        || (*bs).currentEnemy.is_null()
        || (!(*bs).currentEnemy.is_null()/*&& bs->cur_ps.weapon == WP_SABER && bs->frame_Enemy_Len > 300*/)
    {
        enemy = ScanForEnemies(bs);

        if enemy != -1 {
            (*bs).currentEnemy = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(enemy as usize);
            (*bs).enemySeenTime = ((*addr_of!(level)).time + ENEMY_FORGET_MS) as f32;
        }
    }

    if (*bs).squadLeader.is_null() && (*bs).isSquadLeader == 0 {
        BotScanForLeader(bs);
    }

    if (*bs).squadLeader.is_null() && (*bs).squadCannotLead < (*addr_of!(level)).time {
        //if still no leader after scanning, then become a squad leader
        (*bs).isSquadLeader = 1;
    }

    if (*bs).isSquadLeader != 0 && !(*bs).squadLeader.is_null() {
        //we don't follow anyone if we are a leader
        (*bs).squadLeader = null_mut();
    }

    //ESTABLISH VISIBILITIES AND DISTANCES FOR THE WHOLE FRAME HERE
    if !(*bs).wpCurrent.is_null() {
        if (*addr_of!(g_RMG)).integer != 0 {
            //this is somewhat hacky, but in RMG we don't really care about vertical placement because points are scattered across only the terrain.
            let mut vecB: vec3_t = vec3_origin;
            let mut vecC: vec3_t = vec3_origin;

            vecB[0] = (*bs).origin[0];
            vecB[1] = (*bs).origin[1];
            vecB[2] = (*bs).origin[2];

            vecC[0] = (*(*bs).wpCurrent).origin[0];
            vecC[1] = (*(*bs).wpCurrent).origin[1];
            vecC[2] = vecB[2];

            VectorSubtract(&vecC, &vecB, &mut a);
        } else {
            VectorSubtract(&(*(*bs).wpCurrent).origin, &(*bs).origin, &mut a);
        }
        (*bs).frame_Waypoint_Len = VectorLength(&a);

        visResult = WPOrgVisible(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize),
            &(*bs).origin,
            &(*(*bs).wpCurrent).origin,
            (*bs).client,
        );

        if visResult == 2 {
            (*bs).frame_Waypoint_Vis = 0;
            (*bs).wpSeenTime = 0.0;
            (*bs).wpDestination = null_mut();
            (*bs).wpDestIgnoreTime = ((*addr_of!(level)).time + 5000) as f32;

            if (*bs).wpDirection != 0 {
                (*bs).wpDirection = 0;
            } else {
                (*bs).wpDirection = 1;
            }
        } else if visResult != 0 {
            (*bs).frame_Waypoint_Vis = 1;
        } else {
            (*bs).frame_Waypoint_Vis = 0;
        }
    }

    if !(*bs).currentEnemy.is_null() {
        if !(*(*bs).currentEnemy).client.is_null() {
            VectorCopy(&(*(*(*bs).currentEnemy).client).ps.origin, &mut eorg);
            eorg[2] += (*(*(*bs).currentEnemy).client).ps.viewheight as f32;
        } else {
            VectorCopy(&(*(*bs).currentEnemy).s.origin, &mut eorg);
        }

        VectorSubtract(&eorg, &(*bs).eye, &mut a);
        (*bs).frame_Enemy_Len = VectorLength(&a);

        if OrgVisible(&(*bs).eye, &eorg, (*bs).client) != 0 {
            (*bs).frame_Enemy_Vis = 1;
            VectorCopy(&eorg, &mut (*bs).lastEnemySpotted);
            VectorCopy(&(*bs).origin, &mut (*bs).hereWhenSpotted);
            (*bs).lastVisibleEnemyIndex = (*(*bs).currentEnemy).s.number;
            //VectorCopy(bs->eye, bs->lastEnemySpotted);
            (*bs).hitSpotted = 0;
        } else {
            (*bs).frame_Enemy_Vis = 0;
        }
    } else {
        (*bs).lastVisibleEnemyIndex = ENTITYNUM_NONE;
    }
    //END

    if (*bs).frame_Enemy_Vis != 0 {
        (*bs).enemySeenTime = ((*addr_of!(level)).time + ENEMY_FORGET_MS) as f32;
    }

    if !(*bs).wpCurrent.is_null() {
        let mut wpTouchDist: c_int = BOT_WPTOUCH_DISTANCE;
        WPConstantRoutine(bs);

        if (*bs).wpCurrent.is_null() {
            //WPConstantRoutine has the ability to nullify the waypoint if it fails certain checks, so..
            return;
        }

        if (*(*bs).wpCurrent).flags & WPFLAG_WAITFORFUNC != 0 {
            if CheckForFunc(&(*(*bs).wpCurrent).origin, -1) == 0 {
                (*bs).beStill = ((*addr_of!(level)).time + 500) as f32; //no func brush under.. wait
            }
        }
        if (*(*bs).wpCurrent).flags & WPFLAG_NOMOVEFUNC != 0 {
            if CheckForFunc(&(*(*bs).wpCurrent).origin, -1) != 0 {
                (*bs).beStill = ((*addr_of!(level)).time + 500) as f32; //func brush under.. wait
            }
        }

        if (*bs).frame_Waypoint_Vis != 0 || ((*(*bs).wpCurrent).flags & WPFLAG_NOVIS) != 0 {
            if (*addr_of!(g_RMG)).integer != 0 {
                (*bs).wpSeenTime = ((*addr_of!(level)).time + 5000) as f32; //if we lose sight of the point, we have 1.5 seconds to regain it before we drop it
            } else {
                (*bs).wpSeenTime = ((*addr_of!(level)).time + 1500) as f32; //if we lose sight of the point, we have 1.5 seconds to regain it before we drop it
            }
        }
        VectorCopy(&(*(*bs).wpCurrent).origin, &mut (*bs).goalPosition);
        if (*bs).wpDirection != 0 {
            goalWPIndex = (*(*bs).wpCurrent).index - 1;
        } else {
            goalWPIndex = (*(*bs).wpCurrent).index + 1;
        }

        if !(*bs).wpCamping.is_null() {
            VectorSubtract(&(*(*bs).wpCampingTo).origin, &(*bs).origin, &mut a);
            vectoangles(&a, &mut ang);
            VectorCopy(&ang, &mut (*bs).goalAngles);

            VectorSubtract(&(*bs).origin, &(*(*bs).wpCamping).origin, &mut a);
            if VectorLength(&a) < 64.0 {
                VectorCopy(&(*(*bs).wpCamping).origin, &mut (*bs).goalPosition);
                (*bs).beStill = ((*addr_of!(level)).time + 1000) as f32;

                if (*bs).campStanding == QFALSE {
                    (*bs).duckTime = ((*addr_of!(level)).time + 1000) as f32;
                }
            }
        } else if !(*addr_of!(gWPArray))[goalWPIndex as usize].is_null()
            && (*(*addr_of!(gWPArray))[goalWPIndex as usize]).inuse != 0
            && (gLevelFlags & LEVELFLAG_NOPOINTPREDICTION) == 0
        {
            VectorSubtract(
                &(*(*addr_of!(gWPArray))[goalWPIndex as usize]).origin,
                &(*bs).origin,
                &mut a,
            );
            vectoangles(&a, &mut ang);
            VectorCopy(&ang, &mut (*bs).goalAngles);
        } else {
            VectorSubtract(&(*(*bs).wpCurrent).origin, &(*bs).origin, &mut a);
            vectoangles(&a, &mut ang);
            VectorCopy(&ang, &mut (*bs).goalAngles);
        }

        if (*bs).destinationGrabTime < (*addr_of!(level)).time as f32
        /*&& (!bs->wpDestination || (bs->currentEnemy && bs->frame_Enemy_Vis))*/
        {
            GetIdealDestination(bs);
        }

        if !(*bs).wpCurrent.is_null() && !(*bs).wpDestination.is_null() {
            if TotalTrailDistance((*(*bs).wpCurrent).index, (*(*bs).wpDestination).index, bs)
                == -1.0
            {
                (*bs).wpDestination = null_mut();
                (*bs).destinationGrabTime = ((*addr_of!(level)).time + 10000) as f32;
            }
        }

        if (*addr_of!(g_RMG)).integer != 0 {
            if (*bs).frame_Waypoint_Vis != 0 {
                if !(*bs).wpCurrent.is_null() && (*(*bs).wpCurrent).flags == 0 {
                    wpTouchDist *= 3;
                }
            }
        }

        if (*bs).frame_Waypoint_Len < wpTouchDist as f32
            || ((*addr_of!(g_RMG)).integer != 0
                && (*bs).frame_Waypoint_Len < (wpTouchDist * 2) as f32)
        {
            WPTouchRoutine(bs);

            if (*bs).wpDirection == 0 {
                desiredIndex = (*(*bs).wpCurrent).index + 1;
            } else {
                desiredIndex = (*(*bs).wpCurrent).index - 1;
            }

            if !(*addr_of!(gWPArray))[desiredIndex as usize].is_null()
                && (*(*addr_of!(gWPArray))[desiredIndex as usize]).inuse != 0
                && desiredIndex < gWPNum
                && desiredIndex >= 0
                && PassWayCheck(bs, desiredIndex) != 0
            {
                (*bs).wpCurrent = (*addr_of!(gWPArray))[desiredIndex as usize];
            } else {
                if !(*bs).wpDestination.is_null() {
                    (*bs).wpDestination = null_mut();
                    (*bs).destinationGrabTime = ((*addr_of!(level)).time + 10000) as f32;
                }

                if (*bs).wpDirection != 0 {
                    (*bs).wpDirection = 0;
                } else {
                    (*bs).wpDirection = 1;
                }
            }
        }
    } else
    //We can't find a waypoint, going to need a fallback routine.
    {
        /*if (g_gametype.integer == GT_DUEL)*/
        {
            //helps them get out of messy situations
            /*if ((level.time - bs->forceJumpChargeTime) > 3500)
            {
                bs->forceJumpChargeTime = level.time + 2000;
                trap_EA_MoveForward(bs->client);
            }
            */
            (*bs).jumpTime = ((*addr_of!(level)).time + 1500) as f32;
            (*bs).jumpHoldTime = ((*addr_of!(level)).time + 1500) as f32;
            (*bs).jDelay = 0.0;
        }
        doingFallback = BotFallbackNavigation(bs);
    }

    if (*addr_of!(g_RMG)).integer != 0 {
        //for RMG if the bot sticks around an area too long, jump around randomly some to spread to a new area (horrible hacky method)
        let mut vSubDif: vec3_t = vec3_origin;

        VectorSubtract(&(*bs).origin, &(*bs).lastSignificantAreaChange, &mut vSubDif);
        if VectorLength(&vSubDif) > 1500.0 {
            VectorCopy(&(*bs).origin, &mut (*bs).lastSignificantAreaChange);
            (*bs).lastSignificantChangeTime = (*addr_of!(level)).time + 20000;
        }

        if (*bs).lastSignificantChangeTime < (*addr_of!(level)).time {
            (*bs).iHaveNoIdeaWhereIAmGoing = (*addr_of!(level)).time + 17000;
        }
    }

    if (*bs).iHaveNoIdeaWhereIAmGoing > (*addr_of!(level)).time && (*bs).currentEnemy.is_null() {
        VectorCopy(&preFrameGAngles, &mut (*bs).goalAngles);
        (*bs).wpCurrent = null_mut();
        (*bs).wpSwitchTime = ((*addr_of!(level)).time + 150) as f32;
        doingFallback = BotFallbackNavigation(bs);
        (*bs).jumpTime = ((*addr_of!(level)).time + 150) as f32;
        (*bs).jumpHoldTime = ((*addr_of!(level)).time + 150) as f32;
        (*bs).jDelay = 0.0;
        (*bs).lastSignificantChangeTime = (*addr_of!(level)).time + 25000;
    }

    if !(*bs).wpCurrent.is_null() && (*addr_of!(g_RMG)).integer != 0 {
        let mut doJ: qboolean = QFALSE;

        if (*(*bs).wpCurrent).origin[2] - 192.0 > (*bs).origin[2] {
            doJ = QTRUE;
        } else if ((*bs).wpTravelTime - (*addr_of!(level)).time as f32) < 5000.0
            && (*(*bs).wpCurrent).origin[2] - 64.0 > (*bs).origin[2]
        {
            doJ = QTRUE;
        } else if ((*bs).wpTravelTime - (*addr_of!(level)).time as f32) < 7000.0
            && ((*(*bs).wpCurrent).flags & WPFLAG_RED_FLAG) != 0
        {
            if ((*addr_of!(level)).time as f32 - (*bs).jumpTime) > 200.0 {
                (*bs).jumpTime = ((*addr_of!(level)).time + 100) as f32;
                (*bs).jumpHoldTime = ((*addr_of!(level)).time + 100) as f32;
                (*bs).jDelay = 0.0;
            }
        } else if ((*bs).wpTravelTime - (*addr_of!(level)).time as f32) < 7000.0
            && ((*(*bs).wpCurrent).flags & WPFLAG_BLUE_FLAG) != 0
        {
            if ((*addr_of!(level)).time as f32 - (*bs).jumpTime) > 200.0 {
                (*bs).jumpTime = ((*addr_of!(level)).time + 100) as f32;
                (*bs).jumpHoldTime = ((*addr_of!(level)).time + 100) as f32;
                (*bs).jDelay = 0.0;
            }
        } else if (*(*bs).wpCurrent).index > 0 {
            if ((*bs).wpTravelTime - (*addr_of!(level)).time as f32) < 7000.0 {
                if ((*(*addr_of!(gWPArray))[((*(*bs).wpCurrent).index - 1) as usize]).flags
                    & WPFLAG_RED_FLAG)
                    != 0
                    || ((*(*addr_of!(gWPArray))[((*(*bs).wpCurrent).index - 1) as usize]).flags
                        & WPFLAG_BLUE_FLAG)
                        != 0
                {
                    if ((*addr_of!(level)).time as f32 - (*bs).jumpTime) > 200.0 {
                        (*bs).jumpTime = ((*addr_of!(level)).time + 100) as f32;
                        (*bs).jumpHoldTime = ((*addr_of!(level)).time + 100) as f32;
                        (*bs).jDelay = 0.0;
                    }
                }
            }
        }

        if doJ != QFALSE {
            (*bs).jumpTime = ((*addr_of!(level)).time + 1500) as f32;
            (*bs).jumpHoldTime = ((*addr_of!(level)).time + 1500) as f32;
            (*bs).jDelay = 0.0;
        }
    }

    if doingFallback != 0 {
        (*bs).doingFallback = QTRUE;
    } else {
        (*bs).doingFallback = QFALSE;
    }

    if (*bs).timeToReact < (*addr_of!(level)).time as f32
        && !(*bs).currentEnemy.is_null()
        && (*bs).enemySeenTime as f64
            > (*addr_of!(level)).time as f64
                + (ENEMY_FORGET_MS as f64 - (ENEMY_FORGET_MS as f64 * 0.2))
    {
        if (*bs).frame_Enemy_Vis != 0 {
            #[allow(unused_assignments)]
            {
                cBAI = CombatBotAI(bs, thinktime);
            }
        } else if (*bs).cur_ps.weaponstate == WEAPON_CHARGING_ALT {
            //keep charging in case we see him again before we lose track of him
            (*bs).doAltAttack = 1;
        } else if (*bs).cur_ps.weaponstate == WEAPON_CHARGING {
            //keep charging in case we see him again before we lose track of him
            (*bs).doAttack = 1;
        }

        if (*bs).destinationGrabTime > ((*addr_of!(level)).time + 100) as f32 {
            (*bs).destinationGrabTime = ((*addr_of!(level)).time + 100) as f32; //assures that we will continue staying within a general area of where we want to be in a combat situation
        }

        if !(*(*bs).currentEnemy).client.is_null() {
            VectorCopy(&(*(*(*bs).currentEnemy).client).ps.origin, &mut headlevel);
            headlevel[2] += (*(*(*bs).currentEnemy).client).ps.viewheight as f32;
        } else {
            VectorCopy(&(*(*(*bs).currentEnemy).client).ps.origin, &mut headlevel);
        }

        if (*bs).frame_Enemy_Vis == 0 {
            //if (!bs->hitSpotted && VectorLength(a) > 256)
            if OrgVisible(&(*bs).eye, &(*bs).lastEnemySpotted, -1) != 0 {
                VectorCopy(&(*bs).lastEnemySpotted, &mut headlevel);
                VectorSubtract(&headlevel, &(*bs).eye, &mut a);
                vectoangles(&a, &mut ang);
                VectorCopy(&ang, &mut (*bs).goalAngles);

                if (*bs).cur_ps.weapon == WP_FLECHETTE
                    && (*bs).cur_ps.weaponstate == WEAPON_READY
                    && !(*bs).currentEnemy.is_null()
                    && !(*(*bs).currentEnemy).client.is_null()
                {
                    mLen = (VectorLength(&a) > 128.0) as c_int as f32;
                    if mLen > 128.0 && mLen < 1024.0 {
                        VectorSubtract(
                            &(*(*(*bs).currentEnemy).client).ps.origin,
                            &(*bs).lastEnemySpotted,
                            &mut a,
                        );

                        if VectorLength(&a) < 300.0 {
                            (*bs).doAltAttack = 1;
                        }
                    }
                }
            }
        } else {
            bLeadAmount = BotWeaponCanLead(bs);
            if ((*bs).skills.accuracy / (*bs).settings.skill) <= 8.0 && bLeadAmount != 0.0 {
                BotAimLeading(bs, &headlevel, bLeadAmount);
            } else {
                VectorSubtract(&headlevel, &(*bs).eye, &mut a);
                vectoangles(&a, &mut ang);
                VectorCopy(&ang, &mut (*bs).goalAngles);
            }

            BotAimOffsetGoalAngles(bs);
        }
    }

    if (*bs).cur_ps.saberInFlight != QFALSE {
        (*bs).saberThrowTime = (*addr_of!(level)).time + Q_irand(4000, 10000);
    }

    if !(*bs).currentEnemy.is_null() {
        if BotGetWeaponRange(bs) == BWEAPONRANGE_SABER {
            let mut saberRange: c_int = SABER_ATTACK_RANGE;

            VectorSubtract(&(*(*(*bs).currentEnemy).client).ps.origin, &(*bs).eye, &mut a_fo);
            let a_fo_copy = a_fo;
            vectoangles(&a_fo_copy, &mut a_fo);

            if (*bs).saberPowerTime < (*addr_of!(level)).time {
                //Don't just use strong attacks constantly, switch around a bit
                if Q_irand(1, 10) <= 5 {
                    (*bs).saberPower = QTRUE;
                } else {
                    (*bs).saberPower = QFALSE;
                }

                (*bs).saberPowerTime = (*addr_of!(level)).time + Q_irand(3000, 15000);
            }

            if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                .ps
                .fd
                .saberAnimLevel
                != SS_STAFF
                && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                    .ps
                    .fd
                    .saberAnimLevel
                    != SS_DUAL
            {
                if (*(*bs).currentEnemy).health > 75
                    && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                        .ps
                        .fd
                        .forcePowerLevel[FP_SABER_OFFENSE as usize]
                        > 2
                {
                    if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                        .ps
                        .fd
                        .saberAnimLevel
                        != SS_STRONG
                        && (*bs).saberPower != QFALSE
                    {
                        //if we are up against someone with a lot of health and we have a strong attack available, then h4q them
                        Cmd_SaberAttackCycle_f((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize));
                    }
                } else if (*(*bs).currentEnemy).health > 40
                    && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                        .ps
                        .fd
                        .forcePowerLevel[FP_SABER_OFFENSE as usize]
                        > 1
                {
                    if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                        .ps
                        .fd
                        .saberAnimLevel
                        != SS_MEDIUM
                    {
                        //they're down on health a little, use level 2 if we can
                        Cmd_SaberAttackCycle_f((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize));
                    }
                } else {
                    if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                        .ps
                        .fd
                        .saberAnimLevel
                        != SS_FAST
                    {
                        //they've gone below 40 health, go at them with quick attacks
                        Cmd_SaberAttackCycle_f((core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize));
                    }
                }
            }

            if (*addr_of!(g_gametype)).integer == GT_SINGLE_PLAYER {
                saberRange *= 3;
            }

            if (*bs).frame_Enemy_Len <= saberRange as f32 {
                SaberCombatHandling(bs);

                if (*bs).frame_Enemy_Len < 80.0 {
                    meleestrafe = 1;
                }
            } else if (*bs).saberThrowTime < (*addr_of!(level)).time
                && (*bs).cur_ps.saberInFlight == QFALSE
                && ((*bs).cur_ps.fd.forcePowersKnown & (1 << FP_SABERTHROW)) != 0
                && InFieldOfVision(&(*bs).viewangles, 30.0, &mut a_fo) != 0
                && (*bs).frame_Enemy_Len < BOT_SABER_THROW_RANGE as f32
                && (*bs).cur_ps.fd.saberAnimLevel != SS_STAFF
            {
                (*bs).doAltAttack = 1;
                (*bs).doAttack = 0;
            } else if (*bs).cur_ps.saberInFlight != QFALSE
                && (*bs).frame_Enemy_Len > 300.0
                && (*bs).frame_Enemy_Len < BOT_SABER_THROW_RANGE as f32
            {
                (*bs).doAltAttack = 1;
                (*bs).doAttack = 0;
            }
        } else if BotGetWeaponRange(bs) == BWEAPONRANGE_MELEE {
            if (*bs).frame_Enemy_Len <= MELEE_ATTACK_RANGE as f32 {
                MeleeCombatHandling(bs);
                meleestrafe = 1;
            }
        }
    }

    if doingFallback != 0 && !(*bs).currentEnemy.is_null() {
        //just stand and fire if we have no idea where we are
        VectorCopy(&(*bs).origin, &mut (*bs).goalPosition);
    }

    if (*bs).forceJumping > (*addr_of!(level)).time as f32 {
        VectorCopy(&(*bs).origin, &mut noz_x);
        VectorCopy(&(*bs).goalPosition, &mut noz_y);

        noz_x[2] = noz_y[2];

        let noz_x_copy = noz_x;
        VectorSubtract(&noz_x_copy, &noz_y, &mut noz_x);

        if VectorLength(&noz_x) < 32.0 {
            fjHalt = 1;
        }
    }

    /*
        if (bs->doChat && bs->chatTime > level.time && (!bs->currentEnemy || !bs->frame_Enemy_Vis))
        {
            return;
        }
        else if (bs->doChat && bs->currentEnemy && bs->frame_Enemy_Vis)
        {
            //bs->chatTime = level.time + bs->chatTime_stored;
            bs->doChat = 0; //do we want to keep the bot waiting to chat until after the enemy is gone?
            bs->chatTeam = 0;
        }
        else if (bs->doChat && bs->chatTime <= level.time)
        {
            if (bs->chatTeam)
            {
                trap_EA_SayTeam(bs->client, bs->currentChat);
                bs->chatTeam = 0;
            }
            else
            {
                trap_EA_Say(bs->client, bs->currentChat);
            }
            if (bs->doChat == 2)
            {
                BotReplyGreetings(bs);
            }
            bs->doChat = 0;
        }
    */

    CTFFlagMovement(bs);

    if /*bs->wpDestination &&*/ !(*bs).shootGoal.is_null()
        /*bs->wpDestination->associated_entity == bs->shootGoal->s.number &&*/
        && (*(*bs).shootGoal).health > 0
        && (*(*bs).shootGoal).takedamage != QFALSE
    {
        dif[0] = ((*(*bs).shootGoal).r.absmax[0] + (*(*bs).shootGoal).r.absmin[0]) / 2.0;
        dif[1] = ((*(*bs).shootGoal).r.absmax[1] + (*(*bs).shootGoal).r.absmin[1]) / 2.0;
        dif[2] = ((*(*bs).shootGoal).r.absmax[2] + (*(*bs).shootGoal).r.absmin[2]) / 2.0;

        if (*bs).currentEnemy.is_null() || (*bs).frame_Enemy_Len > 256.0 {
            //if someone is close then don't stop shooting them for this
            VectorSubtract(&dif, &(*bs).eye, &mut a);
            let a_copy = a;
            vectoangles(&a_copy, &mut a);
            VectorCopy(&a, &mut (*bs).goalAngles);

            if InFieldOfVision(&(*bs).viewangles, 30.0, &mut a) != 0
                && EntityVisibleBox(
                    &(*bs).origin,
                    &vec3_origin,
                    &vec3_origin,
                    &dif,
                    (*bs).client,
                    (*(*bs).shootGoal).s.number,
                ) != 0
            {
                (*bs).doAttack = 1;
            }
        }
    }

    if (*bs).cur_ps.hasDetPackPlanted != QFALSE {
        //check if our enemy gets near it and detonate if he does
        BotCheckDetPacks(bs);
    } else if !(*bs).currentEnemy.is_null()
        && (*bs).lastVisibleEnemyIndex == (*(*bs).currentEnemy).s.number
        && (*bs).frame_Enemy_Vis == 0
        && (*bs).plantTime < (*addr_of!(level)).time
        && (*bs).doAttack == 0
        && (*bs).doAltAttack == 0
    {
        VectorSubtract(&(*bs).origin, &(*bs).hereWhenSpotted, &mut a);

        if (*bs).plantDecided > (*addr_of!(level)).time
            || ((*bs).frame_Enemy_Len < (BOT_PLANT_DISTANCE * 2) as f32
                && VectorLength(&a) < BOT_PLANT_DISTANCE as f32)
        {
            mineSelect = BotSelectChoiceWeapon(bs, WP_TRIP_MINE, 0);
            detSelect = BotSelectChoiceWeapon(bs, WP_DET_PACK, 0);
            if (*bs).cur_ps.hasDetPackPlanted != QFALSE {
                detSelect = 0;
            }

            if (*bs).plantDecided > (*addr_of!(level)).time
                && (*bs).forceWeaponSelect != 0
                && (*bs).cur_ps.weapon == (*bs).forceWeaponSelect
            {
                (*bs).doAttack = 1;
                (*bs).plantDecided = 0;
                (*bs).plantTime = (*addr_of!(level)).time + BOT_PLANT_INTERVAL;
                (*bs).plantContinue = (*addr_of!(level)).time + 500;
                (*bs).beStill = ((*addr_of!(level)).time + 500) as f32;
            } else if mineSelect != 0 || detSelect != 0 {
                if BotSurfaceNear(bs) != 0 {
                    if mineSelect == 0 {
                        //if no mines use detpacks, otherwise use mines
                        mineSelect = WP_DET_PACK;
                    } else {
                        mineSelect = WP_TRIP_MINE;
                    }

                    detSelect = BotSelectChoiceWeapon(bs, mineSelect, 1);

                    if detSelect != 0 && detSelect != 2 {
                        //We have it and it is now our weapon
                        (*bs).plantDecided = (*addr_of!(level)).time + 1000;
                        (*bs).forceWeaponSelect = mineSelect;
                        return;
                    } else if detSelect == 2 {
                        (*bs).forceWeaponSelect = mineSelect;
                        return;
                    }
                }
            }
        }
    } else if (*bs).plantContinue < (*addr_of!(level)).time {
        (*bs).forceWeaponSelect = 0;
    }

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER
        && (*bs).cur_ps.isJediMaster == QFALSE
        && (*bs).jmState == -1
        && !(*addr_of!(gJMSaberEnt)).is_null()
        && (*(*addr_of!(gJMSaberEnt))).inuse != 0
    {
        let mut saberLen: vec3_t = vec3_origin;
        let f_saber_len: f32;

        VectorSubtract(
            &(*bs).origin,
            &(*(*addr_of!(gJMSaberEnt))).r.currentOrigin,
            &mut saberLen,
        );
        f_saber_len = VectorLength(&saberLen);

        if f_saber_len < 256.0 {
            if OrgVisible(
                &(*bs).origin,
                &(*(*addr_of!(gJMSaberEnt))).r.currentOrigin,
                (*bs).client,
            ) != 0
            {
                VectorCopy(
                    &(*(*addr_of!(gJMSaberEnt))).r.currentOrigin,
                    &mut (*bs).goalPosition,
                );
            }
        }
    }

    if (*bs).beStill < (*addr_of!(level)).time as f32
        && WaitingForNow(bs, &(*bs).goalPosition) == 0
        && fjHalt == 0
    {
        VectorSubtract(&(*bs).goalPosition, &(*bs).origin, &mut (*bs).goalMovedir);
        VectorNormalize(&mut (*bs).goalMovedir);

        if (*bs).jumpTime > (*addr_of!(level)).time as f32
            && (*bs).jDelay < (*addr_of!(level)).time as f32
            && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .pers
                .cmd
                .upmove
                > 0
        {
            //	trap_EA_Move(bs->client, bs->origin, 5000);
            (*bs).beStill = ((*addr_of!(level)).time + 200) as f32;
        } else {
            let goalMovedir = (*bs).goalMovedir;
            trap::ea::EA_Move((*bs).client, &goalMovedir, 5000.0);
        }

        if meleestrafe != 0 {
            StrafeTracing(bs);
        }

        if (*bs).meleeStrafeDir != 0
            && meleestrafe != 0
            && (*bs).meleeStrafeDisable < (*addr_of!(level)).time as f32
        {
            trap::ea::EA_MoveRight((*bs).client);
        } else if meleestrafe != 0 && (*bs).meleeStrafeDisable < (*addr_of!(level)).time as f32 {
            trap::ea::EA_MoveLeft((*bs).client);
        }

        if BotTrace_Jump(bs, &(*bs).goalPosition) != 0 {
            (*bs).jumpTime = ((*addr_of!(level)).time + 100) as f32;
        } else if BotTrace_Duck(bs, &(*bs).goalPosition) != 0 {
            (*bs).duckTime = ((*addr_of!(level)).time + 100) as f32;
        }
        // #ifdef BOT_STRAFE_AVOIDANCE (defined -> this branch)
        else {
            let strafeAround: c_int = BotTrace_Strafe(bs, &(*bs).goalPosition);

            if strafeAround == STRAFEAROUND_RIGHT {
                trap::ea::EA_MoveRight((*bs).client);
            } else if strafeAround == STRAFEAROUND_LEFT {
                trap::ea::EA_MoveLeft((*bs).client);
            }
        }
        // #endif
    }

    // #ifndef FORCEJUMP_INSTANTMETHOD (undefined -> this branch)
    if (*bs).forceJumpChargeTime > (*addr_of!(level)).time {
        (*bs).jumpTime = 0.0;
    }
    // #endif

    if (*bs).jumpPrep > (*addr_of!(level)).time as f32 {
        (*bs).forceJumpChargeTime = 0;
    }

    if (*bs).forceJumpChargeTime > (*addr_of!(level)).time {
        (*bs).jumpHoldTime =
            (((*bs).forceJumpChargeTime - (*addr_of!(level)).time) / 2 + (*addr_of!(level)).time)
                as f32;
        (*bs).forceJumpChargeTime = 0;
    }

    if (*bs).jumpHoldTime > (*addr_of!(level)).time as f32 {
        (*bs).jumpTime = (*bs).jumpHoldTime;
    }

    if (*bs).jumpTime > (*addr_of!(level)).time as f32
        && (*bs).jDelay < (*addr_of!(level)).time as f32
    {
        if (*bs).jumpHoldTime > (*addr_of!(level)).time as f32 {
            trap::ea::EA_Jump((*bs).client);
            if !(*bs).wpCurrent.is_null() {
                if ((*(*bs).wpCurrent).origin[2] - (*bs).origin[2]) < 64.0 {
                    trap::ea::EA_MoveForward((*bs).client);
                }
            } else {
                trap::ea::EA_MoveForward((*bs).client);
            }
            if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                .ps
                .groundEntityNum
                == ENTITYNUM_NONE
            {
                (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*bs).client as usize)).client)
                    .ps
                    .pm_flags |= PMF_JUMP_HELD;
            }
        } else if ((*bs).cur_ps.pm_flags & PMF_JUMP_HELD) == 0 {
            trap::ea::EA_Jump((*bs).client);
        }
    }

    if (*bs).duckTime > (*addr_of!(level)).time as f32 {
        trap::ea::EA_Crouch((*bs).client);
    }

    if !(*bs).dangerousObject.is_null()
        && (*(*bs).dangerousObject).inuse != QFALSE
        && (*(*bs).dangerousObject).health > 0
        && (*(*bs).dangerousObject).takedamage != QFALSE
        && ((*bs).frame_Enemy_Vis == 0 || (*bs).currentEnemy.is_null())
        && (BotGetWeaponRange(bs) == BWEAPONRANGE_MID
            || BotGetWeaponRange(bs) == BWEAPONRANGE_LONG)
        && (*bs).cur_ps.weapon != WP_DET_PACK
        && (*bs).cur_ps.weapon != WP_TRIP_MINE
        && (*bs).shootGoal.is_null()
    {
        let danLen: f32;

        VectorSubtract(&(*(*bs).dangerousObject).r.currentOrigin, &(*bs).eye, &mut a);

        danLen = VectorLength(&a);

        if danLen > 256.0 {
            let a_copy = a;
            vectoangles(&a_copy, &mut a);
            VectorCopy(&a, &mut (*bs).goalAngles);

            if Q_irand(1, 10) < 5 {
                (*bs).goalAngles[YAW] += Q_irand(0, 3) as f32;
                (*bs).goalAngles[PITCH] += Q_irand(0, 3) as f32;
            } else {
                (*bs).goalAngles[YAW] -= Q_irand(0, 3) as f32;
                (*bs).goalAngles[PITCH] -= Q_irand(0, 3) as f32;
            }

            if InFieldOfVision(&(*bs).viewangles, 30.0, &mut a) != 0
                && EntityVisibleBox(
                    &(*bs).origin,
                    &vec3_origin,
                    &vec3_origin,
                    &(*(*bs).dangerousObject).r.currentOrigin,
                    (*bs).client,
                    (*(*bs).dangerousObject).s.number,
                ) != 0
            {
                (*bs).doAttack = 1;
            }
        }
    }

    if PrimFiring(bs) != 0 || AltFiring(bs) != 0 {
        friendInLOF = CheckForFriendInLOF(bs);

        if !friendInLOF.is_null() {
            if PrimFiring(bs) != 0 {
                KeepPrimFromFiring(bs);
            }
            if AltFiring(bs) != 0 {
                KeepAltFromFiring(bs);
            }
            if useTheForce != 0 && forceHostile != 0 {
                useTheForce = 0;
            }

            if useTheForce == 0 && !(*friendInLOF).client.is_null() {
                //we have a friend here and are not currently using force powers, see if we can help them out
                if (*friendInLOF).health <= 50
                    && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePower
                        > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                            .ps
                            .fd
                            .forcePowerLevel[FP_TEAM_HEAL as usize]
                            as usize][FP_TEAM_HEAL as usize]
                {
                    (*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerSelected = FP_TEAM_HEAL;
                    useTheForce = 1;
                    forceHostile = 0;
                } else if (*(*friendInLOF).client).ps.fd.forcePower <= 50
                    && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePower
                        > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                            .ps
                            .fd
                            .forcePowerLevel[FP_TEAM_FORCE as usize]
                            as usize][FP_TEAM_FORCE as usize]
                {
                    (*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerSelected = FP_TEAM_FORCE;
                    useTheForce = 1;
                    forceHostile = 0;
                }
            }
        }
    } else if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        //still check for anyone to help..
        friendInLOF = CheckForFriendInLOF(bs);

        if useTheForce == 0 && !friendInLOF.is_null() {
            if (*friendInLOF).health <= 50
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_TEAM_HEAL as usize]
                        as usize][FP_TEAM_HEAL as usize]
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_TEAM_HEAL;
                useTheForce = 1;
                forceHostile = 0;
            } else if (*(*friendInLOF).client).ps.fd.forcePower <= 50
                && (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePower
                    > forcePowerNeeded[(*(*addr_of!(level)).clients.add((*bs).client as usize))
                        .ps
                        .fd
                        .forcePowerLevel[FP_TEAM_FORCE as usize]
                        as usize][FP_TEAM_FORCE as usize]
            {
                (*(*addr_of!(level)).clients.add((*bs).client as usize))
                    .ps
                    .fd
                    .forcePowerSelected = FP_TEAM_FORCE;
                useTheForce = 1;
                forceHostile = 0;
            }
        }
    }

    if (*bs).doAttack != 0
        && (*bs).cur_ps.weapon == WP_DET_PACK
        && (*bs).cur_ps.hasDetPackPlanted != QFALSE
    {
        //maybe a bit hackish, but bots only want to plant one of these at any given time to avoid complications
        (*bs).doAttack = 0;
    }

    if (*bs).doAttack != 0
        && (*bs).cur_ps.weapon == WP_SABER
        && (*bs).saberDefending != 0
        && !(*bs).currentEnemy.is_null()
        && !(*(*bs).currentEnemy).client.is_null()
        && BotWeaponBlockable((*(*(*bs).currentEnemy).client).ps.weapon) != 0
    {
        (*bs).doAttack = 0;
    }

    if (*bs).cur_ps.saberLockTime > (*addr_of!(level)).time {
        if rand() % 10 < 5 {
            (*bs).doAttack = 1;
        } else {
            (*bs).doAttack = 0;
        }
    }

    if (*bs).botChallengingTime > (*addr_of!(level)).time {
        (*bs).doAttack = 0;
        (*bs).doAltAttack = 0;
    }

    if (*bs).cur_ps.weapon == WP_SABER
        && (*bs).cur_ps.saberInFlight != QFALSE
        && (*bs).cur_ps.saberEntityNum == 0
    {
        //saber knocked away, keep trying to get it back
        (*bs).doAttack = 1;
        (*bs).doAltAttack = 0;
    }

    if (*bs).doAttack != 0 {
        trap::ea::EA_Attack((*bs).client);
    } else if (*bs).doAltAttack != 0 {
        trap::ea::EA_Alt_Attack((*bs).client);
    }

    if useTheForce != 0 && forceHostile != 0 && (*bs).botChallengingTime > (*addr_of!(level)).time {
        useTheForce = QFALSE as c_int;
    }

    if useTheForce != 0 {
        // #ifndef FORCEJUMP_INSTANTMETHOD (undefined -> this branch)
        if (*bs).forceJumpChargeTime > (*addr_of!(level)).time {
            (*(*addr_of!(level)).clients.add((*bs).client as usize))
                .ps
                .fd
                .forcePowerSelected = FP_LEVITATION;
            trap::ea::EA_ForcePower((*bs).client);
        } else {
            // #endif
            if (*addr_of!(bot_forcepowers)).integer != 0
                && (*addr_of!(g_forcePowerDisable)).integer == 0
            {
                trap::ea::EA_ForcePower((*bs).client);
            }
            // #ifndef FORCEJUMP_INSTANTMETHOD
        }
        // #endif
    }

    MoveTowardIdealAngles(bs);
}

/// `int BotAI(int client, float thinktime)` (ai_main.c:688) — run one think frame for a single
/// bot: reset its elementary input, fetch its current player state, drain (and discard) any
/// waiting server commands, fold the delta angles into the view angles, advance the bot's local
/// time, snapshot its origin/eye, run [`StandardBotAI`], then subtract the delta angles back out.
/// The `#ifdef _DEBUG` frame-timing instrumentation is omitted (`_DEBUG` off in the release
/// server build — the same reason the `bot_debugmessages` cvar is omitted).
///
/// # Safety
/// `client` must index a valid bot slot in [`botstates`]/[`g_entities`].
pub unsafe fn BotAI(client: c_int, thinktime: f32) -> c_int {
    let mut buf: [c_char; 1024] = [0; 1024];

    trap::ea::EA_ResetInput(client);
    //
    let bs: *mut bot_state_t = (*addr_of!(botstates))[client as usize];
    if bs.is_null() || (*bs).inuse == 0 {
        BotAI_Print(
            PRT_FATAL,
            c"BotAI: client %d is not setup\n".as_ptr() as *mut c_char,
        );
        return QFALSE as c_int;
    }

    //retrieve the current client state
    BotAI_GetClientState(client, &mut (*bs).cur_ps);

    //retrieve any waiting server commands
    while trap::BotGetServerCommand(client, &mut buf) != 0 {
        //have buf point to the command and args to the command arguments
        // args = strchr( buf, ' ');
        let mut a: usize = 0;
        while buf[a] != 0 && buf[a] != b' ' as c_char {
            a += 1;
        }
        if buf[a] == 0 {
            continue; // strchr returned NULL — no space in the command
        }
        // *args++ = '\0';
        buf[a] = 0;
        let args = addr_of_mut!(buf[a + 1]);

        //remove color espace sequences from the arguments
        RemoveColorEscapeSequences(args);

        let cmd = buf.as_ptr();
        if Q_stricmp(cmd, c"cp ".as_ptr()) == 0 {
            /*CenterPrintf*/
        } else if Q_stricmp(cmd, c"cs".as_ptr()) == 0 {
            /*ConfigStringModified*/
        } else if Q_stricmp(cmd, c"scores".as_ptr()) == 0 {
            /*FIXME: parse scores?*/
        } else if Q_stricmp(cmd, c"clientLevelShot".as_ptr()) == 0 {
            /*ignore*/
        }
    }
    //add the delta angles to the bot's current view angles
    for j in 0..3 {
        (*bs).viewangles[j] =
            AngleMod((*bs).viewangles[j] + SHORT2ANGLE((*bs).cur_ps.delta_angles[j]));
    }
    //increase the local time of the bot
    (*bs).ltime += thinktime;
    //
    (*bs).thinktime = thinktime;
    //origin of the bot
    VectorCopy(&(*bs).cur_ps.origin, &mut (*bs).origin);
    //eye coordinates of the bot
    VectorCopy(&(*bs).cur_ps.origin, &mut (*bs).eye);
    (*bs).eye[2] += (*bs).cur_ps.viewheight as f32;
    //get the area the bot is in

    StandardBotAI(bs, thinktime);

    //subtract the delta angles
    for j in 0..3 {
        (*bs).viewangles[j] =
            AngleMod((*bs).viewangles[j] - SHORT2ANGLE((*bs).cur_ps.delta_angles[j]));
    }
    //everything was ok
    QTRUE as c_int
}

/// `int BotAIStartFrame(int time)` (ai_main.c:7488) — the engine's per-frame bot entry point
/// (`BOTAI_START_FRAME`): refresh the bot cvars at most once a second ([`gUpdateVars`]),
/// spawn pending bots, render waypoint debug (`gBotEdit`), reschedule on a think-time change,
/// then run the scheduled bot AI ([`BotAI`]) and flush each bot's user command
/// ([`BotUpdateInput`] + `trap_BotUserCommand`). The function-local `static int botlib_residual`
/// in the C is declared but never referenced — omitted (no behavior).
pub unsafe fn BotAIStartFrame(time: c_int) -> c_int {
    let elapsed_time: c_int;
    let thinktime: c_int;
    #[allow(non_upper_case_globals)]
    static mut local_time: c_int = 0;
    #[allow(non_upper_case_globals)]
    static mut lastbotthink_time: c_int = 0;

    if (*addr_of!(gUpdateVars)) < (*addr_of!(level)).time {
        trap::Cvar_Update(&mut *addr_of_mut!(bot_pvstype));
        trap::Cvar_Update(&mut *addr_of_mut!(bot_camp));
        trap::Cvar_Update(&mut *addr_of_mut!(bot_attachments));
        trap::Cvar_Update(&mut *addr_of_mut!(bot_forgimmick));
        trap::Cvar_Update(&mut *addr_of_mut!(bot_honorableduelacceptance));
        // #ifndef FINAL_BUILD
        trap::Cvar_Update(&mut *addr_of_mut!(bot_getinthecarrr));
        // #endif
        *addr_of_mut!(gUpdateVars) = (*addr_of!(level)).time + 1000;
    }

    G_CheckBotSpawn();

    //rww - addl bot frame functions
    if (*addr_of!(gBotEdit)) != 0.0 {
        trap::Cvar_Update(&mut *addr_of_mut!(bot_wp_info));
        BotWaypointRender();
    }

    UpdateEventTracker();
    //end rww

    //cap the bot think time
    //if the bot think time changed we should reschedule the bots
    if BOT_THINK_TIME != (*addr_of!(lastbotthink_time)) {
        *addr_of_mut!(lastbotthink_time) = BOT_THINK_TIME;
        BotScheduleBotThink();
    }

    elapsed_time = time - (*addr_of!(local_time));
    *addr_of_mut!(local_time) = time;

    if elapsed_time > BOT_THINK_TIME {
        thinktime = elapsed_time;
    } else {
        thinktime = BOT_THINK_TIME;
    }

    // execute scheduled bot AI
    let mut i: usize = 0;
    while i < MAX_CLIENTS {
        if (*addr_of!(botstates))[i].is_null() || (*(*addr_of!(botstates))[i]).inuse == 0 {
            i += 1;
            continue;
        }
        //
        (*(*addr_of!(botstates))[i]).botthink_residual += elapsed_time;
        //
        if (*(*addr_of!(botstates))[i]).botthink_residual >= thinktime {
            (*(*addr_of!(botstates))[i]).botthink_residual -= thinktime;

            if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i)).client).pers.connected == CON_CONNECTED {
                BotAI(i as c_int, thinktime as f32 / 1000.0);
            }
        }
        i += 1;
    }

    // execute bot user commands every frame
    let mut i: usize = 0;
    while i < MAX_CLIENTS {
        if (*addr_of!(botstates))[i].is_null() || (*(*addr_of!(botstates))[i]).inuse == 0 {
            i += 1;
            continue;
        }
        if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i)).client).pers.connected != CON_CONNECTED {
            i += 1;
            continue;
        }

        BotUpdateInput((*addr_of!(botstates))[i], time, elapsed_time);
        trap::BotUserCommand(
            (*(*addr_of!(botstates))[i]).client,
            addr_of_mut!((*(*addr_of!(botstates))[i]).lastucmd),
        );
        i += 1;
    }

    QTRUE as c_int
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    /// A spread of angle/FOV inputs: in-cone, edge, out-of-cone, and wrap-around cases on
    /// both axes, including negative and >360 values that exercise the `AngleMod` normalize
    /// and the ±180 wrap correction.
    #[test]
    fn InFieldOfVision_matches_oracle_bit_exact() {
        let view_samples: [vec3_t; 6] = [
            [0.0, 0.0, 0.0],
            [0.0, 90.0, 0.0],
            [0.0, 350.0, 0.0],
            [10.0, 170.0, 0.0],
            [-45.0, -170.0, 0.0],
            [30.0, 720.0, 0.0],
        ];
        let angle_samples: [vec3_t; 7] = [
            [0.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [0.0, 100.0, 0.0],
            [0.0, 359.0, 0.0],
            [20.0, 190.0, 0.0],
            [-90.0, -185.0, 0.0],
            [5.0, 45.0, 0.0],
        ];
        let fovs: [f32; 5] = [0.0, 45.0, 90.0, 180.0, 360.0];

        for &view in &view_samples {
            for &ang in &angle_samples {
                for &fov in &fovs {
                    // angles is mutated in place by both impls — give each its own copy.
                    let mut rust_ang = ang;
                    let mut c_ang = ang;
                    let rust = InFieldOfVision(&view, fov, &mut rust_ang);
                    let c = unsafe {
                        oracle::jka_InFieldOfVision(view.as_ptr(), fov, c_ang.as_mut_ptr())
                    };
                    assert_eq!(
                        rust, c,
                        "InFieldOfVision(view={view:?}, fov={fov}, angles={ang:?}): rust={rust} c={c}"
                    );
                    // The in-place mutation of angles[] must match bit-for-bit too.
                    for k in 0..3 {
                        assert_eq!(
                            rust_ang[k].to_bits(),
                            c_ang[k].to_bits(),
                            "angles[{k}] after InFieldOfVision(view={view:?}, fov={fov}, angles={ang:?})"
                        );
                    }
                }
            }
        }
    }

    /// A spread of angle pairs exercising both wrap branches (ang1>ang2 with diff>180, and
    /// ang1<=ang2 with diff<-180), plus negative and out-of-range inputs.
    #[test]
    fn AngleDifference_matches_oracle_bit_exact() {
        use crate::codemp::game::ai_main::AngleDifference;
        let samples: [f32; 9] =
            [0.0, 10.0, 90.0, 179.0, 181.0, 359.0, -45.0, -200.0, 720.0];
        for &a in &samples {
            for &b in &samples {
                let rust = AngleDifference(a, b);
                let c = unsafe { oracle::jka_AngleDifference(a, b) };
                assert_eq!(
                    rust.to_bits(),
                    c.to_bits(),
                    "AngleDifference({a}, {b}): rust={rust} c={c}"
                );
            }
        }
    }

    /// Inputs covering equal angles (early return), both move-clamp branches, and the ±180
    /// wrap corrections, across several speeds.
    #[test]
    fn BotChangeViewAngle_matches_oracle_bit_exact() {
        use crate::codemp::game::ai_main::BotChangeViewAngle;
        let angles: [f32; 7] = [0.0, 45.0, 90.0, 179.0, 270.0, -30.0, 400.0];
        let speeds: [f32; 4] = [0.0, 5.0, 45.0, 1000.0];
        for &angle in &angles {
            for &ideal in &angles {
                for &speed in &speeds {
                    let rust = BotChangeViewAngle(angle, ideal, speed);
                    let c = unsafe { oracle::jka_BotChangeViewAngle(angle, ideal, speed) };
                    assert_eq!(
                        rust.to_bits(),
                        c.to_bits(),
                        "BotChangeViewAngle({angle}, {ideal}, {speed}): rust={rust} c={c}"
                    );
                }
            }
        }
    }

    /// Every weapon id from `WP_NONE` through `WP_TURRET` (and a couple out-of-range ids that
    /// hit the `default` arm), confirming the blockable classification matches the C switch.
    #[test]
    fn BotWeaponBlockable_matches_oracle_bit_exact() {
        use crate::codemp::game::ai_main::BotWeaponBlockable;
        for weapon in -2..=20 {
            let rust = BotWeaponBlockable(weapon);
            let c = unsafe { oracle::jka_BotWeaponBlockable(weapon) };
            assert_eq!(rust, c, "BotWeaponBlockable({weapon}): rust={rust} c={c}");
        }
    }
}
