//! `g_saga.c` — server-side Siege (SAGA) game logic.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_saga.c`, incrementally as consumers
//! need it (the same lazy strategy used for the `trap_*` surface, `g_utils.c`, and
//! `g_session.c`). This first slice is **`G_ValidateSiegeClassForTeam`**
//! (g_saga.c:758), pulled forward because it is a direct dependency of
//! `ClientUserinfoChanged` (g_client.c) — the keystone of the client-infra cluster.
//!
//! The class-table machinery it leans on (`bgSiegeClasses`, `BG_SiegeFindThemeForTeam`,
//! `BG_SiegeFindClassIndexByName`) already lives in `bg_saga.rs`.

use crate::codemp::game::bg_misc::{BG_FindItemForHoldable, BG_FindItemForWeapon};
use crate::codemp::game::bg_public::{
    CS_SIEGE_OBJECTIVES, CS_SIEGE_WINTEAM, EF_CLIENTSMOOTH, EF_NODRAW, EF_RADAROBJECT, ET_GENERAL,
    ET_NPC, ET_PLAYER, EV_SIEGE_OBJECTIVECOMPLETE, EV_SIEGE_ROUNDOVER, GT_SIEGE, HI_NUM_HOLDABLE,
    MASK_PLAYERSOLID, PM_SPECTATOR, STAT_HEALTH, STAT_MAX_HEALTH,
};
use crate::codemp::game::bg_saga::{
    bgSiegeClasses, siege_info, siege_valid, BG_SiegeFindClassIndexByName, BG_SiegeFindThemeForTeam,
    BG_SiegeGetPairedValue, BG_SiegeGetValueGroup,
};
use crate::codemp::game::bg_saga_h::{
    siegeClass_t, MAX_EXDATA_ENTS_TO_SEND, MAX_SIEGE_INFO_SIZE, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2,
    SIEGE_POINTS_FINALOBJECTIVECOMPLETED, SIEGE_POINTS_OBJECTIVECOMPLETED,
    SIEGE_POINTS_TEAMWONROUND, SIEGE_ROUND_BEGIN_TIME,
};
use crate::ffi::types::{fileHandle_t, vmCvar_t};
use crate::codemp::game::g_combat::AddScore;
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::bg_weapons_h::WP_NUM_WEAPONS;
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{gentity_s, gentity_t, FRAMETIME};
use crate::codemp::game::g_main::{
    g_entities, g_gametype, g_siegeTeam1, g_siegeTeam2, g_siegeTeamSwitch, level, Com_Error,
    Com_Printf, G_Error, G_Printf, LogExit,
};
use crate::codemp::game::g_client::{ClientBegin, ClientSpawn, ClientUserinfoChanged};
use crate::codemp::game::bg_saga::{
    bgNumSiegeClasses, bgNumSiegeTeams, BG_PrecacheSabersForSiegeTeam, BG_SiegeLoadClasses,
    BG_SiegeLoadTeams, BG_SiegeSetTeamTheme,
};
use crate::codemp::game::g_public_h::{SVF_BROADCAST, Q3_INFINITE};
use crate::codemp::game::bg_public::{
    CS_SIEGE_STATE, CS_SIEGE_TIMEOVERRIDE, EV_PLAYER_TELEPORT_IN, TEAM_BLUE, TEAM_RED,
    TEAM_SPECTATOR,
};
use crate::codemp::game::g_local::{
    CON_CONNECTED, SPECTATOR_FREE, SPECTATOR_NOT, TEAM_BEGIN,
};
use crate::codemp::game::q_shared::Info_SetValueForKey;
use crate::codemp::game::q_shared_h::{siegePers_t, FS_READ, MAX_INFO_STRING, CVAR_ROM, CVAR_SERVERINFO};
use crate::codemp::game::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString, G_SpawnVector};
use crate::codemp::game::g_utils::{
    G_EffectIndex, G_Find, G_FreeEntity, G_IconIndex, G_ModelIndex, G_PlayEffectID, G_ScaleNetHealth,
    G_SetOrigin, G_Sound, G_SoundIndex, G_TempEntity, G_UseTargets2, GlobalUse,
};
use crate::codemp::game::q_math::{vec3_origin, VectorCopy, VectorSet};
use crate::codemp::game::q_shared_h::{trace_t, vec3_t};
use crate::codemp::game::q_shared::{va, Com_sprintf, Q_strcat, Q_stricmp};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{
    qboolean, CHAN_AUTO, ENTITYNUM_NONE, ERR_DROP, MAX_CLIENTS, MAX_STRING_CHARS, QFALSE, QTRUE,
};
use crate::codemp::game::bg_public::PMF_FOLLOW;
use crate::codemp::game::g_exphysics::G_RunExPhys;
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_NODROP, CONTENTS_SOLID, CONTENTS_TERRAIN, CONTENTS_TRIGGER,
};
use crate::trap;
use core::ffi::{c_char, c_int, CStr};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut};

/// `static char gObjectiveCfgStr[1024]` (g_saga.c:46) — file-scope scratch buffer holding the
/// `CS_SIEGE_OBJECTIVES` configstring (e.g. `"t1-0-0|t2-0-0"`). Built by
/// `G_SiegeSetCampaignData` (not yet ported) and mutated in place by
/// [`G_SiegeSetObjectiveComplete`].
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut gObjectiveCfgStr: [c_char; 1024] = [0; 1024];

/// `qboolean gSiegeRoundBegun = qfalse;` (g_saga.c:36) — file-scope flag set once the Siege
/// round's start countdown has elapsed. Objective items refuse pickup until it is true
/// (gated in [`SiegeItemTouch`]). Cleared on round/level reset by code not yet ported.
#[allow(non_upper_case_globals)] // C global name kept verbatim
pub(crate) static mut gSiegeRoundBegun: qboolean = QFALSE;

/// `qboolean gSiegeRoundEnded = qfalse;` (g_saga.c:37) — file-scope flag set once the Siege
/// round has ended (e.g. `LogExit`). Objective-(de)complete triggers short-circuit while it
/// is true. Cleared on round/level reset by code not yet ported.
#[allow(non_upper_case_globals)] // C global name kept verbatim
pub(crate) static mut gSiegeRoundEnded: qboolean = QFALSE;

/// `qboolean gSiegeRoundWinningTeam = 0;` (g_saga.c:38) — declared `qboolean` in C but assigned
/// the winning team id (an int) in [`SiegeRoundComplete`]; `qboolean` is a `c_int` alias here so it
/// stores the team id verbatim, matching JKA.
#[allow(non_upper_case_globals)] // C global name kept verbatim
pub(crate) static mut gSiegeRoundWinningTeam: qboolean = 0;

/// `static char team1[512];` (g_saga.c:17) — file-scope scratch holding the team1 group name
/// used to index `siege_info`. Populated by Siege campaign-setup code not yet ported.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut team1: [c_char; 512] = [0; 512];

/// `static char team2[512];` (g_saga.c:18) — file-scope scratch holding the team2 group name
/// used to index `siege_info`. Populated by Siege campaign-setup code not yet ported.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut team2: [c_char; 512] = [0; 512];

/// `static char gParseObjectives[MAX_SIEGE_INFO_SIZE];` (g_saga.c:45) — file-scope scratch
/// buffer that [`BG_SiegeGetValueGroup`] fills with a team's objective group before it is
/// re-parsed for individual objective entries.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut gParseObjectives: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];

/// `int imperial_goals_completed = 0;` (g_saga.c:23) — running count of completed imperial
/// (team1) goals; decremented here when an objective is de-completed. Maintained by Siege
/// objective-(de)complete code.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut imperial_goals_completed: c_int = 0;

/// `int rebel_goals_completed = 0;` (g_saga.c:25) — running count of completed rebel (team2)
/// goals; decremented here when an objective is de-completed. Maintained by Siege
/// objective-(de)complete code.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut rebel_goals_completed: c_int = 0;

/// `siegePers_t g_siegePersistant = {qfalse, 0, 0};` (g_saga.c:20) — Siege data carried across
/// level/round changes by the engine via `trap_SiegePersGet`/`trap_SiegePersSet` (used for the
/// "beat their time" team-switch mechanic).
#[allow(non_upper_case_globals)] // C global name kept verbatim
pub static mut g_siegePersistant: siegePers_t = siegePers_t {
    beatingTime: QFALSE,
    lastTeam: 0,
    lastTime: 0,
};

/// `int imperial_goals_required = 0;` (g_saga.c:22) — number of imperial (team1) objectives that
/// must be completed to win the round. Parsed from the `.siege` level file in [`InitSiegeMode`].
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut imperial_goals_required: c_int = 0;

/// `int rebel_goals_required = 0;` (g_saga.c:24) — number of rebel (team2) objectives that must be
/// completed to win the round.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut rebel_goals_required: c_int = 0;

/// `int imperial_time_limit = 0;` (g_saga.c:27) — imperial (team1) round time limit in ms; 0 means
/// no time limit.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut imperial_time_limit: c_int = 0;

/// `int rebel_time_limit = 0;` (g_saga.c:28) — rebel (team2) round time limit in ms; 0 means no
/// time limit. Only one team may have a time limit.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut rebel_time_limit: c_int = 0;

/// `int gImperialCountdown = 0;` (g_saga.c:30) — absolute `level.time` at which the imperial
/// (team1) countdown expires.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut gImperialCountdown: c_int = 0;

/// `int gRebelCountdown = 0;` (g_saga.c:31) — absolute `level.time` at which the rebel (team2)
/// countdown expires.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut gRebelCountdown: c_int = 0;

/// `int rebel_attackers = 0;` (g_saga.c:33) — set to non-0 if the rebel team is the attacking side
/// (parsed from the level's team objective group).
#[allow(non_upper_case_globals)] // C global name kept verbatim
pub(crate) static mut rebel_attackers: c_int = 0;

/// `int imperial_attackers = 0;` (g_saga.c:34) — set to non-0 if the imperial team is the attacking
/// side.
#[allow(non_upper_case_globals)] // C global name kept verbatim
pub(crate) static mut imperial_attackers: c_int = 0;

/// `int gSiegeBeginTime = Q3_INFINITE;` (g_saga.c:38) — absolute `level.time` at which the round is
/// allowed to begin once both teams have players; reset to `Q3_INFINITE` while waiting.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut gSiegeBeginTime: c_int = Q3_INFINITE;

/// `int g_preroundState = 0;` (g_saga.c:40) — default to starting as spec (1 is starting ingame).
#[allow(non_upper_case_globals)] // C global name kept verbatim
static mut g_preroundState: c_int = 0;

// libc string helpers (the `bg_lib` copies are `Q3_VM`-only — same precedent as
// `bg_panimate.rs` / `bg_misc.rs`).
extern "C" {
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn atoi(s: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
}

/// `void G_SiegeRegisterWeaponsAndHoldables( int team )` (g_saga.c:50).
///
/// Go through all classes on a team and register their weapons and items for
/// precaching. Finds the team's theme (`BG_SiegeFindThemeForTeam`), then for each class
/// walks the `weapons`/`invenItems` bitmasks and [`RegisterItem`]s the matching
/// [`BG_FindItemForWeapon`] / [`BG_FindItemForHoldable`] entry. No oracle (precache
/// side-effects via `RegisterItem`, which fans out into configstring/model traps; reads
/// the global Siege class tables).
///
/// # Safety
/// The global `bgSiegeClasses` / Siege team tables and the item system must be
/// initialised; `RegisterItem` issues engine syscalls.
pub unsafe fn G_SiegeRegisterWeaponsAndHoldables(team: c_int) {
    let stm = BG_SiegeFindThemeForTeam(team);

    if !stm.is_null() {
        let mut i: c_int = 0;
        while i < (*stm).numClasses {
            let scl: *mut siegeClass_t = (*stm).classes[i as usize];

            if !scl.is_null() {
                let mut j: c_int = 0;
                while j < WP_NUM_WEAPONS {
                    if (*scl).weapons & (1 << j) != 0 {
                        //we use this weapon so register it.
                        RegisterItem(BG_FindItemForWeapon(j));
                    }
                    j += 1;
                }
                j = 0;
                while j < HI_NUM_HOLDABLE {
                    if (*scl).invenItems & (1 << j) != 0 {
                        //we use this item so register it.
                        RegisterItem(BG_FindItemForHoldable(j));
                    }
                    j += 1;
                }
            }
            i += 1;
        }
    }
}

/// `void G_ValidateSiegeClassForTeam( gentity_t *ent, int team )` (g_saga.c:758).
///
/// If the client's current Siege class is not legal for `team`, swap it for the first
/// legal class on that team — preferring one whose `playerClass` matches the current
/// class. Walks the team's theme (`BG_SiegeFindThemeForTeam`) class list; when the
/// current class name already appears it returns unchanged. No oracle (mutates the
/// global Siege class tables + the client session through pointer fields).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`; the global
/// `bgSiegeClasses` / Siege team tables must be initialised.
pub unsafe fn G_ValidateSiegeClassForTeam(ent: *mut gentity_t, team: c_int) {
    let scl: *mut siegeClass_t;
    let stm;
    let mut new_class_index: c_int = -1;

    if (*(*ent).client).siegeClass == -1 {
        // uh.. sure.
        return;
    }

    let base = addr_of_mut!(bgSiegeClasses) as *mut siegeClass_t;
    scl = base.add((*(*ent).client).siegeClass as usize);

    stm = BG_SiegeFindThemeForTeam(team);
    if !stm.is_null() {
        let mut i: c_int = 0;

        while i < (*stm).numClasses {
            // go through the team and see its valid classes, can we find one that
            // matches our current player class?
            if !(*stm).classes[i as usize].is_null() {
                if Q_stricmp((*scl).name.as_ptr(), (*(*stm).classes[i as usize]).name.as_ptr()) == 0
                {
                    // the class we're using is already ok for this team.
                    return;
                }
                if (*(*stm).classes[i as usize]).playerClass == (*scl).playerClass
                    || new_class_index == -1
                {
                    new_class_index = i;
                }
            }
            i += 1;
        }

        if new_class_index != -1 {
            // ok, let's find it in the global class array
            (*(*ent).client).siegeClass = BG_SiegeFindClassIndexByName(
                (*(*stm).classes[new_class_index as usize]).name.as_ptr(),
            );
            strcpy(
                (*(*ent).client).sess.siegeClass.as_mut_ptr(),
                (*(*stm).classes[new_class_index as usize]).name.as_ptr(),
            );
        }
    }
}

/// `void SiegeItemRemoveOwner( gentity_t *ent, gentity_t *carrier )` (g_saga.c:1352).
///
/// Detach a Siege objective item from its carrier: clear the picked-up flag
/// (`genericValue2`), mark the carrying entity as none (`genericValue8`), and, when a
/// carrier is supplied, drop its `holdingObjectiveItem` and the `SVF_BROADCAST` flag.
/// No oracle (pure entity-state field writes to the ent/client).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; if `carrier` is non-null it must point to a
/// valid `gentity_t` with a non-null `client`.
pub unsafe fn SiegeItemRemoveOwner(ent: *mut gentity_t, carrier: *mut gentity_t) {
    (*ent).genericValue2 = 0; // Remove picked-up flag

    (*ent).genericValue8 = ENTITYNUM_NONE; // Mark entity carrying us as none

    if !carrier.is_null() {
        (*(*carrier).client).holdingObjectiveItem = 0; // The carrier is no longer carrying us
        (*carrier).r.svFlags &= !SVF_BROADCAST;
    }
}

/// `static void SiegeItemRespawnEffect( gentity_t *ent, vec3_t newOrg )` (g_saga.c:1365).
///
/// Fire a Siege objective item's respawn-side effects: trigger its `target5` chain
/// (`G_UseTargets2`), then — if it has a respawn effect id (`genericValue10`) — play that
/// effect (pointing straight up) once at the item's current origin and once at the
/// destination origin `newOrg`. No oracle (effect/trap side-effects on entity origins).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `newOrg` must be a valid origin vector.
pub unsafe fn SiegeItemRespawnEffect(ent: *mut gentity_t, newOrg: &vec3_t) {
    let mut upAng: vec3_t = [0.0; 3];

    if !(*ent).target5.is_null() && *(*ent).target5 != 0 {
        G_UseTargets2(ent, ent, (*ent).target5);
    }

    if (*ent).genericValue10 == 0 {
        // no respawn effect
        return;
    }

    VectorSet(&mut upAng, 0.0, 0.0, 1.0);

    // Play it once on the current origin, and once on the origin we're respawning to.
    G_PlayEffectID((*ent).genericValue10, &(*ent).r.currentOrigin, &upAng);
    G_PlayEffectID((*ent).genericValue10, newOrg, &upAng);
}

/// `void G_SiegeSetObjectiveComplete( int team, int objective, qboolean failIt )` (g_saga.c:390).
///
/// Flip the completion flag of a single Siege objective inside the
/// [`gObjectiveCfgStr`] configstring and re-publish it via `CS_SIEGE_OBJECTIVES`. Finds
/// the team's segment (`"t1"`/`"t2"`), walks `-`-delimited objective entries up to
/// `objective`, then writes the status char to `'1'` (completed) — or `'0'` when
/// `failIt` is set (objective taken away). No oracle (mutates the file-global config
/// buffer and ends in a `trap_SetConfigstring` engine syscall).
///
/// # Safety
/// Reads and mutates the process-global [`gObjectiveCfgStr`]; not re-entrant.
pub unsafe fn G_SiegeSetObjectiveComplete(team: c_int, objective: c_int, failIt: qboolean) {
    let mut p: *mut c_char = core::ptr::null_mut();
    let mut onObjective: c_int = 0;

    let cfg = addr_of_mut!(gObjectiveCfgStr) as *mut c_char;

    if team == SIEGETEAM_TEAM1 {
        p = strstr(cfg, c"t1".as_ptr());
    } else if team == SIEGETEAM_TEAM2 {
        p = strstr(cfg, c"t2".as_ptr());
    }

    if p.is_null() {
        debug_assert!(false);
        return;
    }

    //Parse from the beginning of this team's objectives until we get to the desired objective
    //number.
    while !p.is_null() && *p != 0 && *p != b'|' as c_char {
        if *p == b'-' as c_char {
            onObjective += 1;
        }

        if onObjective == objective {
            //this is the one we want
            //Move to the next char, the status of this objective
            p = p.add(1);

            //Now change it from '0' to '1' if we are completeing the objective
            //or vice versa if the objective has been taken away
            if failIt != QFALSE {
                *p = b'0' as c_char;
            } else {
                *p = b'1' as c_char;
            }
            break;
        }

        p = p.add(1);
    }

    //Now re-update the configstring.
    let s = CStr::from_ptr(cfg).to_str().unwrap_or("");
    trap::SetConfigstring(CS_SIEGE_OBJECTIVES, s);
}

/// `void SiegeBroadcast_ROUNDOVER( int winningteam, int winningclient )` (g_saga.c:557).
///
/// Fire a broadcast temp-entity carrying the [`EV_SIEGE_ROUNDOVER`] event so every
/// client learns who won the Siege round. The origin is irrelevant (the C clears a
/// throwaway `nomatter` vector — identical to [`vec3_origin`]). `winningteam` rides in
/// `eventParm`, `winningclient` in `weapon`. No oracle: spawns into the live entity pool
/// via [`G_TempEntity`] (entity/trap control-flow, same as the surrounding broadcast
/// leaves).
///
/// # Safety
/// The entity system must be initialised; [`G_TempEntity`] allocates from `g_entities`.
pub unsafe fn SiegeBroadcast_ROUNDOVER(winningteam: c_int, winningclient: c_int) {
    // vec3_t nomatter; VectorClear(nomatter);
    let te = G_TempEntity(&vec3_origin, EV_SIEGE_ROUNDOVER);
    (*te).r.svFlags |= SVF_BROADCAST;
    (*te).s.eventParm = winningteam;
    (*te).s.weapon = winningclient;
}

/// `void SiegeSetCompleteData( int team )` (g_saga.c:90).
///
/// Tell clients that `team` won and have it printed on their scoreboard for
/// intermission (or whatever): publish `team` into the `CS_SIEGE_WINTEAM`
/// configstring. No oracle (a single `trap_SetConfigstring` engine syscall, routed
/// through [`va`] verbatim as in the C).
///
/// # Safety
/// [`va`] aliases shared static storage; the engine syscall surface must be wired.
pub unsafe fn SiegeSetCompleteData(team: c_int) {
    trap::SetConfigstring(
        CS_SIEGE_WINTEAM,
        &CStr::from_ptr(va(format_args!("{}", team))).to_string_lossy(),
    );
}

/// `void SiegeItemTouch( gentity_t *self, gentity_t *other, trace_t *trace )` (g_saga.c:1491).
///
/// Touch handler for a Siege objective item. Rejects non-player / dead / spectator /
/// already-carrying touchers, items that are already picked up, players on the item's
/// "no-touch" team (`genericValue6`), and pickups before the round has begun
/// (`gSiegeRoundBegun`). On a valid pickup it plays the pickup sound, marks itself
/// carried (`genericValue2`), records the carrier (`genericValue8` / the carrier's
/// `holdingObjectiveItem`), broadcasts the carrier (`SVF_BROADCAST`), fires the
/// `target2` pickup chain (respecting the fire-once `genericValue4`/`genericValue5`
/// flags), and flags itself to blink on radar indefinitely (`s.time2 = 0xFFFFFFFF`).
/// If the toucher is invalid and the item is wedged in solid, it nudges itself up one
/// unit to escape. No oracle (a void touch handler that mutates ent/client state and
/// fans out into `G_Sound`/`G_SetOrigin`/`G_UseTargets2` traps).
///
/// # Safety
/// `self` must point to a valid `gentity_t`; `other` and `trace` may be null. When
/// `other` is a live client its `client` pointer must be valid.
pub unsafe extern "C" fn SiegeItemTouch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    trace: *mut trace_t,
) {
    if other.is_null()
        || (*other).inuse == QFALSE
        || (*other).client.is_null()
        || (*other).s.eType == ET_NPC
    {
        if !trace.is_null() && (*trace).startsolid != 0 {
            //let me out! (ideally this should not happen, but such is life)
            let mut escapePos: vec3_t = [0.0; 3];
            VectorCopy(&(*self_).r.currentOrigin, &mut escapePos);
            escapePos[2] += 1.0;

            //I hope you weren't stuck in the ceiling.
            G_SetOrigin(self_, &escapePos);
        }
        return;
    }

    if (*other).health < 1 {
        //dead people can't pick us up.
        return;
    }

    if (*(*other).client).holdingObjectiveItem != 0 {
        //this guy's already carrying a siege item
        return;
    }

    if (*(*other).client).ps.pm_type == PM_SPECTATOR {
        //spectators don't pick stuff up
        return;
    }

    if (*self_).genericValue2 != 0 {
        //Am I already picked up?
        return;
    }

    if (*self_).genericValue6 == (*(*other).client).sess.sessionTeam {
        //Set to not be touchable by players on this team.
        return;
    }

    if gSiegeRoundBegun == QFALSE {
        //can't pick it up if round hasn't started yet
        return;
    }

    if (*self_).noise_index != 0 {
        //play the pickup noise.
        G_Sound(other, CHAN_AUTO, (*self_).noise_index);
    }

    (*self_).genericValue2 = 1; //Mark it as picked up.

    (*(*other).client).holdingObjectiveItem = (*other).s.number;
    (*other).r.svFlags |= SVF_BROADCAST; //broadcast player while he carries this
    (*self_).genericValue8 = (*other).s.number; //Keep the index so we know who is "carrying" us

    (*self_).genericValue9 = 0; //So it doesn't think it has to respawn.

    if !(*self_).target2.is_null()
        && *(*self_).target2 != 0
        && ((*self_).genericValue4 == 0 || (*self_).genericValue5 == 0)
    {
        //fire the target for pickup, if it's set to fire every time, or set to only fire the first time and the first time has not yet occured.
        G_UseTargets2(self_, self_, (*self_).target2);
        (*self_).genericValue5 = 1; //mark it as having been picked up
    }

    // time2 set to -1 will blink the item on the radar indefinately
    (*self_).s.time2 = 0xFFFFFFFFu32 as c_int;
}

/// `void SiegeItemPain( gentity_t *self, gentity_t *attacker, int damage )` (g_saga.c:1561).
///
/// Pain callback for a Siege objective item. Stamps `s.time2` with the current
/// `level.time` — Time 2 is used to pulse the radar icon to show it's under attack. No
/// oracle (a void pain callback that writes one entity-state field from the global level
/// clock; `attacker`/`damage` are unused but kept to match the C pain-callback ABI).
///
/// # Safety
/// `self` must point to a valid `gentity_t`. `attacker`/`damage` are unused.
#[allow(unused_variables)] // attacker/damage unused, kept to match the C pain-callback ABI
pub unsafe extern "C" fn SiegeItemPain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    // Time 2 is used to pulse the radar icon to show its under attack
    (*self_).s.time2 = (*addr_of_mut!(level)).time;
}

/// `void SiegeItemDie( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int meansOfDeath )` (g_saga.c:1567).
///
/// Die callback for a Siege objective item. Disables further damage (so it cannot die
/// more than once), plays its indexed death effect (`genericValue3`) straight up if it
/// has one, then schedules itself to free next frame (`neverFree = qfalse`,
/// `think = G_FreeEntity`, `nextthink = level.time`). Finally fires its death target
/// chain (`target4`) if set. No oracle (a void die handler that mutates entity state and
/// fans out into `G_PlayEffectID`/`G_UseTargets2` FX traps plus a `G_FreeEntity` think,
/// none bit-exact-returnable — matches the cycle-6 siege-leaf no-oracle precedent).
///
/// # Safety
/// `self` must point to a valid `gentity_t`. `inflictor`/`attacker` are unused.
#[allow(unused_variables)] // inflictor/attacker/damage/meansOfDeath unused, kept to match the C die-callback ABI
pub unsafe extern "C" fn SiegeItemDie(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    meansOfDeath: c_int,
) {
    (*self_).takedamage = QFALSE; //don't die more than once

    if (*self_).genericValue3 != 0 {
        //An indexed effect to play on death
        let mut upAng: vec3_t = [0.0; 3];

        VectorSet(&mut upAng, 0.0, 0.0, 1.0);
        G_PlayEffectID((*self_).genericValue3, &(*self_).r.currentOrigin, &upAng);
    }

    (*self_).neverFree = QFALSE;
    (*self_).think = Some(G_FreeEntity);
    (*self_).nextthink = (*addr_of_mut!(level)).time;

    //Fire off the death target if we've got one.
    if !(*self_).target4.is_null() && *(*self_).target4 != 0 {
        G_UseTargets2(self_, self_, (*self_).target4);
    }
}

/// `void SiegeBroadcast_OBJECTIVECOMPLETE( int team, int client, int objective )` (g_saga.c:543).
///
/// Fire a broadcast temp-entity carrying the [`EV_SIEGE_OBJECTIVECOMPLETE`] event so
/// every client learns an objective was completed. The origin is irrelevant (the C
/// clears a throwaway `nomatter` vector — identical to [`vec3_origin`]). `team` rides in
/// `eventParm`, `client` in `weapon`, `objective` in `trickedentindex`. No oracle: spawns
/// into the live entity pool via [`G_TempEntity`] (entity/trap control-flow, same as
/// [`SiegeBroadcast_ROUNDOVER`]).
///
/// # Safety
/// The entity system must be initialised; [`G_TempEntity`] allocates from `g_entities`.
pub unsafe fn SiegeBroadcast_OBJECTIVECOMPLETE(team: c_int, client: c_int, objective: c_int) {
    // vec3_t nomatter; VectorClear(nomatter);
    let te = G_TempEntity(&vec3_origin, EV_SIEGE_OBJECTIVECOMPLETE);
    (*te).r.svFlags |= SVF_BROADCAST;
    (*te).s.eventParm = team;
    (*te).s.weapon = client;
    (*te).s.trickedentindex = objective;
}

/// `void BroadcastObjectiveCompletion( int team, int objective, int final, int client )`
/// (g_saga.c:570).
///
/// Announce an objective completion to all clients. If `client` names a valid player on the
/// `team` that just lost the objective (i.e. the completer is on the opposing side), he is
/// awarded [`SIEGE_POINTS_OBJECTIVECOMPLETED`] via [`AddScore`]. Then fires the broadcast
/// temp-entity through [`SiegeBroadcast_OBJECTIVECOMPLETE`]. The `final` flag is unused here
/// (the C only references it in a commented-out `G_Printf`, carried over below). No oracle:
/// mutates live client score state via [`AddScore`] over the global `g_entities` pool.
///
/// # Safety
/// `client` indexes the global `g_entities`; the entity system must be initialised.
pub unsafe fn BroadcastObjectiveCompletion(team: c_int, objective: c_int, _final: c_int, client: c_int) {
    let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(client as usize);
    if client != ENTITYNUM_NONE
        && !(*ent).client.is_null()
        && (*(*ent).client).sess.sessionTeam == team
    { //guy who completed this objective gets points, providing he's on the opposing team
        AddScore(
            ent,
            &(*(*ent).client).ps.origin,
            SIEGE_POINTS_OBJECTIVECOMPLETED,
        );
    }

    SiegeBroadcast_OBJECTIVECOMPLETE(team, client, objective);
    //G_Printf("Broadcast goal completion team %i objective %i final %i\n", team, objective, final);
}

/// `void AddSiegeWinningTeamPoints( int team, int winner )` (g_saga.c:581).
///
/// Award round-win points to every player on the winning `team`: the `winner` client (the one
/// who completed the final objective) gets [`SIEGE_POINTS_TEAMWONROUND`] +
/// [`SIEGE_POINTS_FINALOBJECTIVECOMPLETED`], everyone else on the team gets just
/// [`SIEGE_POINTS_TEAMWONROUND`]. Walks all [`MAX_CLIENTS`] slots. No oracle: mutates live
/// client score state via [`AddScore`] over the global `g_entities` pool.
///
/// # Safety
/// Walks the global `g_entities`; the entity system must be initialised.
pub unsafe fn AddSiegeWinningTeamPoints(team: c_int, winner: c_int) {
    let mut i: c_int = 0;

    while i < MAX_CLIENTS as c_int {
        let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if !ent.is_null() && !(*ent).client.is_null() && (*(*ent).client).sess.sessionTeam == team {
            if i == winner {
                AddScore(
                    ent,
                    &(*(*ent).client).ps.origin,
                    SIEGE_POINTS_TEAMWONROUND + SIEGE_POINTS_FINALOBJECTIVECOMPLETED,
                );
            } else {
                AddScore(
                    ent,
                    &(*(*ent).client).ps.origin,
                    SIEGE_POINTS_TEAMWONROUND,
                );
            }
        }

        i += 1;
    }
}

/// `qboolean G_SiegeGetCompletionStatus( int team, int objective )` (g_saga.c:445).
///
/// Returns `qtrue` if the given Siege objective is currently complete, otherwise
/// `qfalse`. Finds the team's segment (`"t1"`/`"t2"`) inside the [`gObjectiveCfgStr`]
/// configstring, walks `-`-delimited objective entries up to `objective`, then reads the
/// following status char (`'1'` = complete). No oracle (reads the file-global config
/// buffer — same global-state precedent as [`G_SiegeSetObjectiveComplete`]).
///
/// # Safety
/// Reads the process-global [`gObjectiveCfgStr`]; not re-entrant.
pub unsafe fn G_SiegeGetCompletionStatus(team: c_int, objective: c_int) -> qboolean {
    let mut p: *mut c_char = core::ptr::null_mut();
    let mut onObjective: c_int = 0;

    let cfg = addr_of_mut!(gObjectiveCfgStr) as *mut c_char;

    if team == SIEGETEAM_TEAM1 {
        p = strstr(cfg, c"t1".as_ptr());
    } else if team == SIEGETEAM_TEAM2 {
        p = strstr(cfg, c"t2".as_ptr());
    }

    if p.is_null() {
        debug_assert!(false);
        return QFALSE;
    }

    //Parse from the beginning of this team's objectives until we get to the desired objective
    //number.
    while !p.is_null() && *p != 0 && *p != b'|' as c_char {
        if *p == b'-' as c_char {
            onObjective += 1;
        }

        if onObjective == objective {
            //this is the one we want
            //Move to the next char, the status of this objective
            p = p.add(1);

            //return qtrue if it's '1', qfalse if it's anything else
            if *p == b'1' as c_char {
                return QTRUE;
            } else {
                return QFALSE;
            }
        }

        p = p.add(1);
    }

    QFALSE
}

/// `void UseSiegeTarget( gentity_t *other, gentity_t *en, char *target )` (g_saga.c:497).
///
/// Actually use the player which triggered the object which triggered the siege
/// objective to trigger the target. If no player (`en`) is available the activating
/// entity (`other`) is used as the user; if there is no player entity at all, or no
/// `target`, it bails. Otherwise it fires the `use` of every entity whose `targetname`
/// matches `target` (warning on self-use, bailing if the user is freed mid-loop). No
/// oracle (entity-list walk that fans out into `G_Find`/`GlobalUse` plus `G_Printf`
/// traps — mirrors the [`G_UseTargets2`] loop precedent).
///
/// # Safety
/// `other` may be null; when `en` is non-null it must point to a valid `gentity_t`.
/// `target` may be null.
pub unsafe fn UseSiegeTarget(other: *mut gentity_t, en: *mut gentity_t, target: *mut c_char) {
    let mut t: *mut gentity_t;
    let ent: *mut gentity_t;

    if en.is_null() || (*en).client.is_null() {
        //looks like we don't have access to a player, so just use the activating entity
        ent = other;
    } else {
        ent = en;
    }

    if en.is_null() {
        return;
    }

    if target.is_null() {
        return;
    }

    t = core::ptr::null_mut();
    loop {
        t = G_Find(t, offset_of!(gentity_s, targetname), target);
        if t.is_null() {
            break;
        }
        if t == ent {
            G_Printf("WARNING: Entity used itself.\n");
        } else if (*t).r#use.is_some() {
            GlobalUse(t, ent, ent);
        }
        if (*ent).inuse == QFALSE {
            G_Printf("entity was removed while using targets\n");
            return;
        }
    }
}

/// `void SiegeIconUse( gentity_t *ent, gentity_t *other, gentity_t *activator )` (g_saga.c:1196).
///
/// `use` callback for an `info_siege_radaricon` entity — toggle its radar icon on and
/// off. When the icon is on (`EF_RADAROBJECT`) it clears that flag plus `SVF_BROADCAST`;
/// otherwise it sets both. No oracle (pure entity-state field flips; `other`/`activator`
/// are unused but kept to match the C `use`-callback ABI).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
#[allow(unused_variables)] // other/activator unused, kept to match the C use-callback ABI
pub unsafe extern "C" fn SiegeIconUse(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    //toggle it on and off
    if ((*ent).s.eFlags & EF_RADAROBJECT) != 0 {
        (*ent).s.eFlags &= !EF_RADAROBJECT;
        (*ent).r.svFlags &= !SVF_BROADCAST;
    } else {
        (*ent).s.eFlags |= EF_RADAROBJECT;
        (*ent).r.svFlags |= SVF_BROADCAST;
    }
}

//sends extra data about other client's in this client's PVS
//used for support guy etc.
//with this formatting:
//sxd 16,999,999,999|17,999,999,999
//assumed max 2 chars for cl num, 3 chars per ammo/health/maxhealth, even a single string full of
//info for all 32 clients should not get much past 450 bytes, which is well within a
//reasonable range. We don't need to send anything about the max ammo or current weapon, because
//currentState.weapon can be checked for the ent in question on the client. -rww
/// `void G_SiegeClientExData( gentity_t *msgTarg )` (g_saga.c:1859).
///
/// Build and send the `sxd` extra-data string to `msgTarg`: walk the live entity list and,
/// for every other in-use player client on the same team that is in `msgTarg`'s PVS
/// (`trap_InPVS`), append `"<num>|<health>|<maxhealth>|<ammo-for-current-weapon>"` (space
/// separated, prefixed once with `"sxd "`). Sends nothing if no clients qualify. No oracle
/// (walks the global `g_entities`/`level` state, calls `trap_InPVS`, and ends in a
/// `trap_SendServerCommand` engine syscall).
///
/// # Safety
/// `msgTarg` must point to a valid `gentity_t` with a non-null `client`; the entity system
/// and global Siege/weapon tables must be initialised.
pub unsafe fn G_SiegeClientExData(msgTarg: *mut gentity_t) {
    let mut count: c_int = 0;
    let mut i: c_int = 0;
    let mut str: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
    let mut scratch: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

    while i < (*addr_of!(level)).num_entities && (count as usize) < MAX_EXDATA_ENTS_TO_SEND {
        let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*ent).inuse != QFALSE
            && !(*ent).client.is_null()
            && (*msgTarg).s.number != (*ent).s.number
            && (*ent).s.eType == ET_PLAYER
            && (*(*msgTarg).client).sess.sessionTeam == (*(*ent).client).sess.sessionTeam
            && trap::InPVS(
                &(*(*msgTarg).client).ps.origin,
                &(*(*ent).client).ps.origin,
            ) != QFALSE
        {
            //another client in the same pvs, send his jive
            if count != 0 {
                //append a seperating space if we are not the first in the list
                Q_strcat(str.as_mut_ptr(), str.len() as c_int, c" ".as_ptr());
            } else {
                //otherwise create the prepended chunk
                strcpy(str.as_mut_ptr(), c"sxd ".as_ptr());
            }

            //append the stats
            Com_sprintf(
                scratch.as_mut_ptr(),
                scratch.len() as c_int,
                format_args!(
                    "{}|{}|{}|{}",
                    (*ent).s.number,
                    (*(*ent).client).ps.stats[STAT_HEALTH as usize],
                    (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize],
                    (*(*ent).client).ps.ammo
                        [weaponData[(*(*ent).client).ps.weapon as usize].ammoIndex as usize]
                ),
            );
            Q_strcat(str.as_mut_ptr(), str.len() as c_int, scratch.as_ptr());
            count += 1;
        }
        i += 1;
    }

    if count == 0 {
        //nothing to send
        return;
    }

    //send the string to him
    let client_num = msgTarg.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as i32;
    let s = CStr::from_ptr(str.as_ptr()).to_string_lossy();
    trap::SendServerCommand(client_num, &s);
}

/// `#define SIEGE_ITEM_RESPAWN_TIME 20000` (g_saga.c:1351) — grace period (ms) before a
/// Siege objective item left in a nonstandard place gives up and respawns on its original
/// spot.
const SIEGE_ITEM_RESPAWN_TIME: c_int = 20000;

/// `static void SiegeItemRespawnOnOriginalSpot( gentity_t *ent, gentity_t *carrier )` (g_saga.c:1386).
///
/// Send a Siege objective item back to its spawn point (`pos1`): play the respawn effects
/// ([`SiegeItemRespawnEffect`]), move it there ([`G_SetOrigin`]), detach its carrier
/// ([`SiegeItemRemoveOwner`]), and stop it flashing on the radar (`s.time2 = 0`). No oracle
/// (entity-state writes plus FX/trap fan-out — matches the surrounding siege-leaf
/// no-oracle precedent).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `carrier` may be null (forwarded to
/// [`SiegeItemRemoveOwner`]).
unsafe fn SiegeItemRespawnOnOriginalSpot(ent: *mut gentity_t, carrier: *mut gentity_t) {
    SiegeItemRespawnEffect(ent, &(*ent).pos1);
    G_SetOrigin(ent, &(*ent).pos1);
    SiegeItemRemoveOwner(ent, carrier);

    // Stop the item from flashing on the radar
    (*ent).s.time2 = 0;
}

/// `void SiegeItemThink( gentity_t *ent )` (g_saga.c:1396).
///
/// Per-frame think for a Siege objective item. Optionally recharges health
/// (`genericValue12`/`13`/`14`). If carried (`genericValue8 != ENTITYNUM_NONE`) it sticks
/// the item onto the carrier's origin (keeping it in the same PVS so it renders bolted
/// on); otherwise, if physics are enabled (`genericValue1`), it runs them via
/// [`G_RunExPhys`]. It sets `s.boltToPlayer` when carried by a real client. When carried,
/// it handles the carrier going invalid / switching off a siege team / following
/// (respawn on the original spot), or dying (drop where he is — or, in nodrop, back to the
/// original spot — and give it a random pop velocity plus a [`SIEGE_ITEM_RESPAWN_TIME`]
/// fallback timer). A pending fallback timer (`genericValue9`) that has expired respawns
/// it on the original spot. Always reschedules itself half a frame out. No oracle (entity-
/// state + trap/physics control flow).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity system and global `level`/`g_entities`
/// state must be initialised.
pub unsafe extern "C" fn SiegeItemThink(ent: *mut gentity_t) {
    let mut carrier: *mut gentity_t = core::ptr::null_mut();

    if (*ent).genericValue12 != 0 {
        //recharge health
        if (*ent).health > 0
            && (*ent).health < (*ent).maxHealth
            && (*ent).genericValue14 < (*addr_of!(level)).time
        {
            (*ent).genericValue14 = (*addr_of!(level)).time + (*ent).genericValue13;
            (*ent).health += (*ent).genericValue12;
            if (*ent).health > (*ent).maxHealth {
                (*ent).health = (*ent).maxHealth;
            }
        }
    }

    if (*ent).genericValue8 != ENTITYNUM_NONE {
        //Just keep sticking it on top of the owner. We need it in the same PVS as him so it will render bolted onto him properly.
        carrier = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).genericValue8 as usize);

        if (*carrier).inuse != QFALSE && !(*carrier).client.is_null() {
            VectorCopy(
                &(*(*carrier).client).ps.origin,
                &mut (*ent).r.currentOrigin,
            );
            trap::LinkEntity(ent);
        }
    } else if (*ent).genericValue1 != 0 {
        //this means we want to run physics on the object
        G_RunExPhys(
            ent,
            (*ent).radius,
            (*ent).mass,
            (*ent).random,
            QFALSE as i32,
            core::ptr::null_mut(),
            0,
        );
    }

    //Bolt us to whoever is carrying us if a client
    if (*ent).genericValue8 < MAX_CLIENTS as c_int {
        (*ent).s.boltToPlayer = (*ent).genericValue8 + 1;
    } else {
        (*ent).s.boltToPlayer = 0;
    }

    if !carrier.is_null() {
        let carrier: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*ent).genericValue8 as usize);

        //This checking can be a bit iffy on the death stuff, but in theory we should always
        //get a think in before the default minimum respawn time is exceeded.
        if (*carrier).inuse == QFALSE
            || (*carrier).client.is_null()
            || ((*(*carrier).client).sess.sessionTeam != SIEGETEAM_TEAM1
                && (*(*carrier).client).sess.sessionTeam != SIEGETEAM_TEAM2)
            || ((*(*carrier).client).ps.pm_flags & PMF_FOLLOW) != 0
        {
            //respawn on the original spot
            SiegeItemRespawnOnOriginalSpot(ent, core::ptr::null_mut());
        } else if (*carrier).health < 1 {
            //The carrier died so pop out where he is (unless in nodrop).
            if !(*ent).target6.is_null() && *(*ent).target6 != 0 {
                G_UseTargets2(ent, ent, (*ent).target6);
            }

            if trap::PointContents(&(*(*carrier).client).ps.origin, (*carrier).s.number)
                & CONTENTS_NODROP
                != 0
            {
                //In nodrop land, go back to the original spot.
                SiegeItemRespawnOnOriginalSpot(ent, carrier);
            } else {
                G_SetOrigin(ent, &(*(*carrier).client).ps.origin);
                (*ent).epVelocity[0] = Q_irand(-80, 80) as f32;
                (*ent).epVelocity[1] = Q_irand(-80, 80) as f32;
                (*ent).epVelocity[2] = Q_irand(40, 80) as f32;

                //We're in a nonstandard place, so if we go this long without being touched,
                //assume we may not be reachable and respawn on the original spot.
                (*ent).genericValue9 = (*addr_of!(level)).time + SIEGE_ITEM_RESPAWN_TIME;

                SiegeItemRemoveOwner(ent, carrier);
            }
        }
    }

    if (*ent).genericValue9 != 0 && (*ent).genericValue9 < (*addr_of!(level)).time {
        //time to respawn on the original spot then
        SiegeItemRespawnEffect(ent, &(*ent).pos1);
        G_SetOrigin(ent, &(*ent).pos1);
        (*ent).genericValue9 = 0;

        // stop flashing on radar
        (*ent).s.time2 = 0;
    }

    (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME / 2;
}

/// `#define SIEGEITEM_STARTOFFRADAR 8` (g_saga.c:15) — spawnflag bit: a Siege item that
/// starts hidden from the radar until used.
const SIEGEITEM_STARTOFFRADAR: c_int = 8;

/// `void SiegeItemUse( gentity_t *ent, gentity_t *other, gentity_t *activator )` (g_saga.c:1590).
///
/// `use` callback that activates a Siege objective item. Sets it to show on the radar
/// (`EF_RADAROBJECT`); for a `STARTOFFRADAR` item that is still drawn, that is all there is
/// to do. Makes it pickup-able-by-walking-in (a `CONTENTS_TRIGGER` with [`SiegeItemTouch`]
/// when `genericValue11`) or solid (`MASK_PLAYERSOLID`) depending on its flags/takedamage.
/// Starts its [`SiegeItemThink`] (half a frame out), clears `EF_NODRAW`, and — if it has a
/// `paintarget` — relocates onto that target's origin. No oracle (entity-state writes plus
/// `G_Find`/`G_SetOrigin`/trap fan-out; `other`/`activator` unused, kept to match the C
/// `use`-callback ABI).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity system must be initialised.
#[allow(unused_variables)] // other/activator unused, kept to match the C use-callback ABI
pub unsafe extern "C" fn SiegeItemUse(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    //once used, become active
    if ((*ent).spawnflags & SIEGEITEM_STARTOFFRADAR) != 0 {
        //start showing on radar
        (*ent).s.eFlags |= EF_RADAROBJECT;

        if ((*ent).s.eFlags & EF_NODRAW) == 0 {
            //we've nothing else to do here
            return;
        }
    } else {
        //make sure it's showing up
        (*ent).s.eFlags |= EF_RADAROBJECT;
    }

    if (*ent).genericValue11 != 0 || (*ent).takedamage == QFALSE {
        //We want to be able to walk into it to pick it up then.
        (*ent).r.contents = CONTENTS_TRIGGER;
        (*ent).clipmask = CONTENTS_SOLID | CONTENTS_TERRAIN;
        if (*ent).genericValue11 != 0 {
            (*ent).touch = Some(SiegeItemTouch);
        }
    } else {
        //Make it solid.
        (*ent).r.contents = MASK_PLAYERSOLID;
        (*ent).clipmask = MASK_PLAYERSOLID;
    }

    (*ent).think = Some(SiegeItemThink);
    (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME / 2;

    //take off nodraw
    (*ent).s.eFlags &= !EF_NODRAW;

    if !(*ent).paintarget.is_null() && *(*ent).paintarget != 0 {
        //want to be on this guy's origin now then
        let targ: *mut gentity_t = G_Find(
            core::ptr::null_mut(),
            offset_of!(gentity_s, targetname),
            (*ent).paintarget,
        );

        if !targ.is_null() && (*targ).inuse != QFALSE {
            G_SetOrigin(ent, &(*targ).r.currentOrigin);
            trap::LinkEntity(ent);
        }
    }
}

/// `void decompTriggerUse( gentity_t *ent, gentity_t *other, gentity_t *activator )` (g_saga.c:1250).
///
/// `use` callback for an `info_siege_decomplete`: take a previously-completed Siege
/// objective back away. Does nothing if the round has ended ([`gSiegeRoundEnded`]) or the
/// objective is not currently complete ([`G_SiegeGetCompletionStatus`]). Otherwise it marks
/// the objective complete-flag back off ([`G_SiegeSetObjectiveComplete`] with `failIt`),
/// looks up whether the objective counts toward the team's final goal count (parsing the
/// team's group out of `siege_info` via [`BG_SiegeGetValueGroup`] /
/// [`BG_SiegeGetPairedValue`]), and, unless `final == -1`, decrements the matching team's
/// completed-goal counter ([`imperial_goals_completed`] / [`rebel_goals_completed`]). No
/// oracle (reads/writes process-global Siege state and fans out into Siege config parsing;
/// `other`/`activator` unused, kept to match the C `use`-callback ABI).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; reads/mutates the process-global Siege state
/// (`siege_info`, `gParseObjectives`, `team1`/`team2`, goal counters) — not re-entrant.
#[allow(unused_variables)] // other/activator unused, kept to match the C use-callback ABI
pub unsafe extern "C" fn decompTriggerUse(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let mut final_: c_int = 0;
    let mut teamstr: [c_char; 1024] = [0; 1024];
    let mut objectivestr: [c_char; 64] = [0; 64];
    let mut desiredobjective: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];

    if gSiegeRoundEnded != QFALSE {
        return;
    }

    if G_SiegeGetCompletionStatus((*ent).side, (*ent).objective) == QFALSE {
        //if it's not complete then there's nothing to do here
        return;
    }

    //Update the configstring status
    G_SiegeSetObjectiveComplete((*ent).side, (*ent).objective, QTRUE);

    //Find out if this objective counts toward the final objective count
    if (*ent).side == SIEGETEAM_TEAM1 {
        Com_sprintf(
            teamstr.as_mut_ptr(),
            teamstr.len() as c_int,
            format_args!(
                "{}",
                CStr::from_ptr(addr_of!(team1) as *const c_char).to_string_lossy()
            ),
        );
    } else {
        Com_sprintf(
            teamstr.as_mut_ptr(),
            teamstr.len() as c_int,
            format_args!(
                "{}",
                CStr::from_ptr(addr_of!(team2) as *const c_char).to_string_lossy()
            ),
        );
    }

    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        teamstr.as_mut_ptr(),
        addr_of_mut!(gParseObjectives) as *mut c_char,
    ) != 0
    {
        Com_sprintf(
            objectivestr.as_mut_ptr(),
            objectivestr.len() as c_int,
            format_args!("Objective{}", (*ent).objective),
        );

        if BG_SiegeGetValueGroup(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            objectivestr.as_mut_ptr(),
            desiredobjective.as_mut_ptr(),
        ) != 0
        {
            if BG_SiegeGetPairedValue(
                desiredobjective.as_mut_ptr(),
                c"final".as_ptr() as *mut c_char,
                teamstr.as_mut_ptr(),
            ) != 0
            {
                final_ = atoi(teamstr.as_ptr());
            }
        }
    }

    //Subtract the goal num if applicable
    if final_ != -1 {
        if (*ent).side == SIEGETEAM_TEAM1 {
            imperial_goals_completed -= 1;
        } else {
            rebel_goals_completed -= 1;
        }
    }
}

/*QUAKED info_siege_radaricon (1 0 1) (-16 -16 -24) (16 16 32) ?
Used to arbitrarily display radar icons at placed location. Can be used
to toggle on and off.

"icon" - icon that represents the objective on the radar
"startoff" - if 1 start off
*/
/// `void SP_info_siege_radaricon( gentity_t *ent )` (g_saga.c:1217).
///
/// Spawn-initializer for an `info_siege_radaricon` entity. Frees itself outside of a valid
/// Siege game. Reads `startoff`; unless set it starts on (`EF_RADAROBJECT` + `SVF_BROADCAST`).
/// Requires an `icon` key — bails with [`Com_Error`]`(ERR_DROP, ...)` if missing (that is the
/// whole point of the entity). Installs [`SiegeIconUse`] as its `use` toggle, indexes the icon
/// into `genericenemyindex`, and links into the world. No oracle (entity-state writes plus
/// `G_FreeEntity`/`G_IconIndex`/`trap_LinkEntity` syscalls and the global Siege gate).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity system and global Siege state must be
/// initialised.
pub unsafe fn SP_info_siege_radaricon(ent: *mut gentity_t) {
    let mut s: *mut c_char = core::ptr::null_mut();
    let mut i: c_int = 0;

    if siege_valid == 0 || g_gametype.integer != GT_SIEGE {
        G_FreeEntity(ent);
        return;
    }

    G_SpawnInt(c"startoff".as_ptr(), c"0".as_ptr(), &mut i);

    if i == 0 {
        //start on then
        (*ent).s.eFlags |= EF_RADAROBJECT;
        (*ent).r.svFlags |= SVF_BROADCAST;
    }

    G_SpawnString(c"icon".as_ptr(), c"".as_ptr(), &mut s);
    if s.is_null() || *s == 0 {
        //that's the whole point of the entity
        Com_Error(ERR_DROP, "misc_siege_radaricon without an icon");
    }

    (*ent).r#use = Some(SiegeIconUse);

    (*ent).s.genericenemyindex = G_IconIndex(&CStr::from_ptr(s).to_string_lossy());

    trap::LinkEntity(ent);
}


/*QUAKED info_siege_decomplete (1 0 1) (-16 -16 -24) (16 16 32)
"objective" - specifies the objective to decomplete upon activation
"side" - set to 1 to specify an imperial (team1) goal, 2 to specify rebels (team2)
*/
/// `void SP_info_siege_decomplete( gentity_t *ent )` (g_saga.c:1311).
///
/// Spawn-initializer for an `info_siege_decomplete` entity. Frees itself outside of a valid
/// Siege game. Installs [`decompTriggerUse`] as its `use` callback and reads `objective`/`side`;
/// if either is zero (the mapper fux0red something up) it frees itself and prints an error. No
/// oracle (entity-state writes plus `G_FreeEntity`/`G_Printf` syscalls and the global Siege
/// gate).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity system and global Siege state must be
/// initialised.
pub unsafe fn SP_info_siege_decomplete(ent: *mut gentity_t) {
    if siege_valid == 0 || g_gametype.integer != GT_SIEGE {
        G_FreeEntity(ent);
        return;
    }

    (*ent).r#use = Some(decompTriggerUse);
    G_SpawnInt(c"objective".as_ptr(), c"0".as_ptr(), &mut (*ent).objective);
    G_SpawnInt(c"side".as_ptr(), c"0".as_ptr(), &mut (*ent).side);

    if (*ent).objective == 0 || (*ent).side == 0 {
        //j00 fux0red something up
        G_FreeEntity(ent);
        G_Printf("ERROR: info_siege_objective_decomplete without an objective or side value\n");
    }
}


/// `void siegeEndUse( gentity_t *ent, gentity_t *other, gentity_t *activator )` (g_saga.c:1331).
///
/// `use` callback for a `target_siege_end` entity — do a [`LogExit`]`("Round ended")` to end
/// the Siege round when triggered. No oracle (a void `use` callback fanning out into the
/// `LogExit` round-end path; `ent`/`other`/`activator` unused, kept to match the C
/// `use`-callback ABI).
///
/// # Safety
/// The level/game state must be initialised (consumed by [`LogExit`]). The parameters are
/// unused.
#[allow(unused_variables)] // ent/other/activator unused, kept to match the C use-callback ABI
pub unsafe extern "C" fn siegeEndUse(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    LogExit("Round ended");
}


/*QUAKED misc_siege_item (1 0 1) (-16 -16 -24) (16 16 32) ? x x STARTOFFRADAR
STARTOFFRADAR - start not displaying on radar, don't display until used.

"model"				Name of model to use for the object
"mins"				Actual mins of the object. Careful not to place it into a solid,
					as these new mins will not be reflected visually in the editor.
					Default value is "-16 -16 -24".
"maxs"				Same as above for maxs. Default value is "16 16 32".
"targetname"		If it has a targetname, it will only spawn upon being used.
"target2"			Target to fire upon pickup. If none, nothing will happen.
"pickuponlyonce"	If non-0, target2 will only be fired on the first pickup. If the item is
					dropped and picked up again later, the target will not be fired off on
					the sequential pickup. Default value is 1.
"target3"			Target to fire upon delivery of the item to the goal point.
					If none, nothing will happen. (but you should always want something to happen)
"health"			If > 0, object can be damaged and will die once health reaches 0. Default is 0.
"showhealth"		if health > 0, will show a health meter for this item
"teamowner"			Which team owns this item, used only for deciding what color to make health meter
"target4"			Target to fire upon death, if damageable. Default is none.
"deathfx"			Effect to play on death, if damageable. Default is none.
"canpickup"			If non-0, item can be picked up. Otherwise it will just be solid and sit on the
					ground. Default is 1.
"pickupsound"		Sound to play on pickup, if any.
"goaltarget"		Must be the targetname of a trigger_multi/trigger_once. Once a player carrying
					this object is brought inside the specified trigger, then that trigger will be
					allowed to fire. Ideally it will target a siege objective or something like that.
"usephysics"		If non-0, run standard physics on the object. Default is 1.
"mass"				If usephysics, this will be the factored object mass. Default is 0.09.
"gravity"			If usephysics, this will be the factored gravitational pull. Default is 3.0.
"bounce"			If usephysics, this will be the factored bounce amount. Default is 1.3.
"teamnotouch"		If 1 don't let team 1 pickup, if 2 don't let team 2. By default both teams
					can pick this object up and carry it around until death.
"teamnocomplete"	Same values as above, but controls if this object can be taken into the objective
					area by said team.
"respawnfx"			Plays this effect when respawning (e.g. it is left in an unknown area too long
					and goes back to the original spot). If this is not specified there will be
					no effect. (expected format is .efx file)
"paintarget"		plop self on top of this guy's origin when we are used (only applies if the siege
					item has a targetname)
"noradar"			if non-0 this thing will not show up on radar

"forcelimit"		if non-0, while carrying this item, the carrier's force powers will be crippled.
"target5"			target to fire when respawning.
"target6"			target to fire when dropped by someone carrying this item.

"icon"				icon that represents the gametype item on the radar

health charge things only work with showhealth 1 on siege items that take damage.
"health_chargeamt"	if non-0 will recharge this much health every...
"health_chargerate"	...this many milliseconds
*/
/// `void SP_misc_siege_item( gentity_t *ent )` (g_saga.c:1690).
///
/// Spawn-initializer for a `misc_siege_item` objective entity. Frees itself outside a valid
/// Siege game and [`G_Error`]s if no model is specified. Reads its many tuning keys
/// (`canpickup`, `usephysics`, `noradar`, `pickuponlyonce`, `teamnotouch`, `teamnocomplete`,
/// physics mass/gravity/bounce, pickup/death/respawn FX + icon, mins/maxs, `forcelimit`),
/// indexes its model (flagging Ghoul2 for a `.glm`), stores the origin for respawning, and —
/// if it has health — wires up [`SiegeItemPain`]/[`SiegeItemDie`] and optional health-meter /
/// recharge. Installs [`SiegeItemUse`] for `STARTOFFRADAR`/targeted items (else starts its
/// contents/touch/[`SiegeItemThink`] immediately), initializes the carrier to none, marks
/// `neverFree`, and links into the world. No oracle (entity-state writes plus the full Siege
/// spawn syscall fan-out and the global Siege gate).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity system and global Siege state must be
/// initialised.
pub unsafe fn SP_misc_siege_item(ent: *mut gentity_t) {
    let mut canpickup: c_int = 0;
    let mut noradar: c_int = 0;
    let mut s: *mut c_char = core::ptr::null_mut();

    if siege_valid == 0 || g_gametype.integer != GT_SIEGE {
        G_FreeEntity(ent);
        return;
    }

    if (*ent).model.is_null() || *(*ent).model == 0 {
        G_Error("You must specify a model for misc_siege_item types.");
    }

    G_SpawnInt(c"canpickup".as_ptr(), c"1".as_ptr(), &mut canpickup);
    G_SpawnInt(
        c"usephysics".as_ptr(),
        c"1".as_ptr(),
        &mut (*ent).genericValue1,
    );

    if (*ent).genericValue1 != 0 {
        //if we're using physics we want lerporigin smoothing
        (*ent).s.eFlags |= EF_CLIENTSMOOTH;
    }

    G_SpawnInt(c"noradar".as_ptr(), c"0".as_ptr(), &mut noradar);
    //Want it to always show up as a goal object on radar
    if noradar == 0 && ((*ent).spawnflags & SIEGEITEM_STARTOFFRADAR) == 0 {
        (*ent).s.eFlags |= EF_RADAROBJECT;
    }

    //All clients want to know where it is at all times for radar
    (*ent).r.svFlags |= SVF_BROADCAST;

    G_SpawnInt(
        c"pickuponlyonce".as_ptr(),
        c"1".as_ptr(),
        &mut (*ent).genericValue4,
    );

    G_SpawnInt(
        c"teamnotouch".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue6,
    );
    G_SpawnInt(
        c"teamnocomplete".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue7,
    );

    //Get default physics values.
    G_SpawnFloat(c"mass".as_ptr(), c"0.09".as_ptr(), &mut (*ent).mass);
    G_SpawnFloat(c"gravity".as_ptr(), c"3.0".as_ptr(), &mut (*ent).radius);
    G_SpawnFloat(c"bounce".as_ptr(), c"1.3".as_ptr(), &mut (*ent).random);

    G_SpawnString(c"pickupsound".as_ptr(), c"".as_ptr(), &mut s);

    if !s.is_null() && *s != 0 {
        //We have a pickup sound, so index it now.
        (*ent).noise_index = G_SoundIndex(&CStr::from_ptr(s).to_string_lossy());
    }

    G_SpawnString(c"deathfx".as_ptr(), c"".as_ptr(), &mut s);

    if !s.is_null() && *s != 0 {
        //We have a death effect, so index it now.
        (*ent).genericValue3 = G_EffectIndex(&CStr::from_ptr(s).to_string_lossy());
    }

    G_SpawnString(c"respawnfx".as_ptr(), c"".as_ptr(), &mut s);

    if !s.is_null() && *s != 0 {
        //We have a respawn effect, so index it now.
        (*ent).genericValue10 = G_EffectIndex(&CStr::from_ptr(s).to_string_lossy());
    }

    G_SpawnString(c"icon".as_ptr(), c"".as_ptr(), &mut s);

    if !s.is_null() && *s != 0 {
        // We have an icon, so index it now.  We are reusing the genericenemyindex
        // variable rather than adding a new one to the entity state.
        (*ent).s.genericenemyindex = G_IconIndex(&CStr::from_ptr(s).to_string_lossy());
    }

    (*ent).s.modelindex = G_ModelIndex(&CStr::from_ptr((*ent).model).to_string_lossy());

    //Is the model a ghoul2 model?
    if Q_stricmp(
        (*ent).model.add(strlen((*ent).model) - 4),
        c".glm".as_ptr(),
    ) == 0
    {
        //apparently so.
        (*ent).s.modelGhoul2 = 1;
    }

    (*ent).s.eType = ET_GENERAL;

    //Set the mins/maxs with default values.
    G_SpawnVector(
        c"mins".as_ptr(),
        c"-16 -16 -24".as_ptr(),
        (*ent).r.mins.as_mut_ptr(),
    );
    G_SpawnVector(
        c"maxs".as_ptr(),
        c"16 16 32".as_ptr(),
        (*ent).r.maxs.as_mut_ptr(),
    );

    VectorCopy(&(*ent).s.origin, &mut (*ent).pos1); //store off the initial origin for respawning
    G_SetOrigin(ent, &(*ent).s.origin);

    VectorCopy(&(*ent).s.angles, &mut (*ent).r.currentAngles);
    VectorCopy(&(*ent).s.angles, &mut (*ent).s.apos.trBase);

    G_SpawnInt(
        c"forcelimit".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue15,
    );

    if (*ent).health > 0 {
        //If it has health, it can be killed.
        let mut t: c_int = 0;

        (*ent).pain = Some(SiegeItemPain);
        (*ent).die = Some(SiegeItemDie);
        (*ent).takedamage = QTRUE;

        G_SpawnInt(c"showhealth".as_ptr(), c"0".as_ptr(), &mut t);
        if t != 0 {
            //a non-0 maxhealth value will mean we want to show the health on the hud
            (*ent).maxHealth = (*ent).health;
            G_ScaleNetHealth(ent);

            G_SpawnInt(
                c"health_chargeamt".as_ptr(),
                c"0".as_ptr(),
                &mut (*ent).genericValue12,
            );
            G_SpawnInt(
                c"health_chargerate".as_ptr(),
                c"0".as_ptr(),
                &mut (*ent).genericValue13,
            );
        }
    } else {
        //Otherwise no.
        (*ent).takedamage = QFALSE;
    }

    if ((*ent).spawnflags & SIEGEITEM_STARTOFFRADAR) != 0 {
        (*ent).r#use = Some(SiegeItemUse);
    } else if !(*ent).targetname.is_null() && *(*ent).targetname != 0 {
        (*ent).s.eFlags |= EF_NODRAW; //kind of hacky, but whatever
        (*ent).genericValue11 = canpickup;
        (*ent).r#use = Some(SiegeItemUse);
        (*ent).s.eFlags &= !EF_RADAROBJECT;
    }

    if ((*ent).targetname.is_null() || *(*ent).targetname == 0)
        || ((*ent).spawnflags & SIEGEITEM_STARTOFFRADAR) != 0
    {
        if canpickup != 0 || (*ent).takedamage == QFALSE {
            //We want to be able to walk into it to pick it up then.
            (*ent).r.contents = CONTENTS_TRIGGER;
            (*ent).clipmask = CONTENTS_SOLID | CONTENTS_TERRAIN;
            if canpickup != 0 {
                (*ent).touch = Some(SiegeItemTouch);
            }
        } else {
            //Make it solid.
            (*ent).r.contents = MASK_PLAYERSOLID;
            (*ent).clipmask = MASK_PLAYERSOLID;
        }

        (*ent).think = Some(SiegeItemThink);
        (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME / 2;
    }

    (*ent).genericValue8 = ENTITYNUM_NONE; //initialize the carrier to none

    (*ent).neverFree = QTRUE; //never free us unless we specifically request it.

    trap::LinkEntity(ent);
}

/// `void SiegeClearSwitchData( void )` (g_saga.c:606).
///
/// Reset the cross-round Siege persistent data to all-zero and push it back to the engine
/// (`trap_SiegePersSet`), clearing any "beat their time" team-switch state. No oracle (a
/// memset of the file-global [`g_siegePersistant`] followed by a `trap_SiegePersSet` engine
/// syscall).
///
/// # Safety
/// Mutates the process-global [`g_siegePersistant`]; the engine syscall surface must be wired.
pub unsafe fn SiegeClearSwitchData() {
    *addr_of_mut!(g_siegePersistant) = siegePers_t {
        beatingTime: QFALSE,
        lastTeam: 0,
        lastTime: 0,
    };
    trap::SiegePersSet(&*addr_of!(g_siegePersistant));
}

/// `void SiegeSetCompleteData( int team )` — see [`SiegeSetCompleteData`] above; round-end
/// data publisher used by the round-complete / team-switch chain.

/// `void SiegeTeamSwitch( int winTeam, int winTime )` (g_saga.c:647).
///
/// Drive the optional Siege "beat their time" team-switch mechanic. Reads the persistent
/// data (`trap_SiegePersGet`); if already in switched mode it announces the overall
/// `winTeam` via [`SiegeSetCompleteData`] and clears the data ([`SiegeClearSwitchData`]).
/// Otherwise it records this round's winner/time and pushes it back
/// (`trap_SiegePersSet`) so the other team must beat it next round. No oracle (reads/mutates
/// the process-global [`g_siegePersistant`] and ends in `trap_SiegePers*` syscalls).
///
/// # Safety
/// Mutates the process-global [`g_siegePersistant`]; the engine syscall surface must be wired.
pub unsafe fn SiegeTeamSwitch(winTeam: c_int, winTime: c_int) {
    trap::SiegePersGet(&mut *addr_of_mut!(g_siegePersistant));
    if (*addr_of!(g_siegePersistant)).beatingTime != QFALSE {
        //was already in "switched" mode, change back
        //announce the winning team.
        //either the first team won again, or the second
        //team beat the time set by the initial team. In any
        //case the winTeam here is the overall winning team.
        SiegeSetCompleteData(winTeam);
        SiegeClearSwitchData();
    } else {
        //go into "beat their time" mode
        (*addr_of_mut!(g_siegePersistant)).beatingTime = QTRUE;
        (*addr_of_mut!(g_siegePersistant)).lastTeam = winTeam;
        (*addr_of_mut!(g_siegePersistant)).lastTime = winTime;

        trap::SiegePersSet(&*addr_of!(g_siegePersistant));
    }
}

/// `void SiegeRoundComplete( int winningteam, int winningclient )` (g_saga.c:669).
///
/// End the current Siege round in favour of `winningteam`. If `winningclient` actually
/// belongs to the losing side, it is forgotten (`ENTITYNUM_NONE`). Broadcasts the round-over
/// event ([`SiegeBroadcast_ROUNDOVER`]), awards team points ([`AddSiegeWinningTeamPoints`]),
/// sets the `CS_SIEGE_STATE` configstring to "ended", flips the round flags, and — instead of
/// exiting directly — fires the team's `roundover_target` (parsed from `siege_info`) so a
/// script can handle the transition (falling back to [`LogExit`] when none is set, or to the
/// first in-use entity when no winning client is available). Finally, for time-limited
/// switch games, computes the elapsed time and calls [`SiegeTeamSwitch`]; otherwise clears the
/// switch data. No oracle (process-global Siege state + the round-end trap/`LogExit` fan-out).
///
/// # Safety
/// `winningclient` indexes the global `g_entities`; the entity system and process-global Siege
/// state must be initialised.
pub unsafe fn SiegeRoundComplete(winningteam: c_int, mut winningclient: c_int) {
    let mut teamstr: [c_char; 1024] = [0; 1024];
    let originalWinningClient_init = winningclient;
    let mut originalWinningClient = originalWinningClient_init;

    //G_Printf("Team %i won\n", winningteam);

    if winningclient != ENTITYNUM_NONE
        && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(winningclient as usize)).client.is_null()
        && (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(winningclient as usize)).client)
            .sess
            .sessionTeam
            != winningteam
    {
        //this person just won the round for the other team..
        winningclient = ENTITYNUM_NONE;
    }

    // vec3_t nomatter; VectorClear(nomatter); -- unused beyond declaration in C

    SiegeBroadcast_ROUNDOVER(winningteam, winningclient);

    AddSiegeWinningTeamPoints(winningteam, winningclient);

    //Instead of exiting like this, fire off a target, and let it handle things.
    //Can be a script or whatever the designer wants.
    if winningteam == SIEGETEAM_TEAM1 {
        Com_sprintf(
            teamstr.as_mut_ptr(),
            teamstr.len() as c_int,
            format_args!(
                "{}",
                CStr::from_ptr(addr_of!(team1) as *const c_char).to_string_lossy()
            ),
        );
    } else {
        Com_sprintf(
            teamstr.as_mut_ptr(),
            teamstr.len() as c_int,
            format_args!(
                "{}",
                CStr::from_ptr(addr_of!(team2) as *const c_char).to_string_lossy()
            ),
        );
    }

    trap::SetConfigstring(
        CS_SIEGE_STATE,
        &CStr::from_ptr(va(format_args!("3|{}", (*addr_of!(level)).time))).to_string_lossy(),
    ); //ended
    gSiegeRoundBegun = QFALSE;
    gSiegeRoundEnded = QTRUE;
    gSiegeRoundWinningTeam = winningteam;

    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        teamstr.as_mut_ptr(),
        addr_of_mut!(gParseObjectives) as *mut c_char,
    ) != 0
    {
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"roundover_target".as_ptr() as *mut c_char,
            teamstr.as_mut_ptr(),
        ) == 0
        {
            //didn't find the name of the thing to target upon win, just logexit now then.
            LogExit("Objectives completed");
            return;
        }

        if originalWinningClient == ENTITYNUM_NONE {
            //oh well, just find something active and use it then.
            let mut i: c_int = 0;

            while i < MAX_CLIENTS as c_int {
                let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

                if (*ent).inuse != QFALSE {
                    //sure, you'll do.
                    originalWinningClient = (*ent).s.number;
                    break;
                }

                i += 1;
            }
        }
        G_UseTargets2(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(originalWinningClient as usize),
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(originalWinningClient as usize),
            teamstr.as_mut_ptr(),
        );
    }

    if g_siegeTeamSwitch.integer != 0 && (imperial_time_limit != 0 || rebel_time_limit != 0) {
        //handle stupid team switching crap
        let mut time: c_int = 0;
        if imperial_time_limit != 0 {
            time = imperial_time_limit - (gImperialCountdown - (*addr_of!(level)).time);
        } else if rebel_time_limit != 0 {
            time = rebel_time_limit - (gRebelCountdown - (*addr_of!(level)).time);
        }

        if time < 1 {
            time = 1;
        }
        SiegeTeamSwitch(winningteam, time);
    } else {
        //assure it's clear for next round
        SiegeClearSwitchData();
    }
}

/// `void SetTeamQuick( gentity_t *ent, int team, qboolean doBegin )` (g_saga.c:801).
///
/// Bypass most of the normal checks in `SetTeam`: directly move `ent` to `team`. Validates
/// the player's Siege class for the team ([`G_ValidateSiegeClassForTeam`]) when in Siege,
/// updates the session team/spectator state, rewrites the `"team"` userinfo key
/// (`s`/`r`/`b`/`?`), republishes it (`trap_SetUserinfo`), resets the spectator/team-state
/// bookkeeping, runs [`ClientUserinfoChanged`], and — when `doBegin` — calls [`ClientBegin`].
/// No oracle (userinfo trap plumbing plus client-spawn fan-out, same precedent as
/// `Cmd_TeamTask_f`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`; the level/client system
/// must be initialised.
pub unsafe fn SetTeamQuick(ent: *mut gentity_t, team: c_int, doBegin: qboolean) {
    let mut userinfo = [0 as c_char; MAX_INFO_STRING];

    // trap_GetUserinfo( ent->s.number, userinfo, sizeof( userinfo ) );
    {
        let info = trap::GetUserinfo((*ent).s.number);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_STRING - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }

    if g_gametype.integer == GT_SIEGE {
        G_ValidateSiegeClassForTeam(ent, team);
    }

    (*(*ent).client).sess.sessionTeam = team;

    if team == TEAM_SPECTATOR {
        (*(*ent).client).sess.spectatorState = SPECTATOR_FREE;
        Info_SetValueForKey(userinfo.as_mut_ptr(), c"team".as_ptr(), c"s".as_ptr());
    } else {
        (*(*ent).client).sess.spectatorState = SPECTATOR_NOT;
        if team == TEAM_RED {
            Info_SetValueForKey(userinfo.as_mut_ptr(), c"team".as_ptr(), c"r".as_ptr());
        } else if team == TEAM_BLUE {
            Info_SetValueForKey(userinfo.as_mut_ptr(), c"team".as_ptr(), c"b".as_ptr());
        } else {
            Info_SetValueForKey(userinfo.as_mut_ptr(), c"team".as_ptr(), c"?".as_ptr());
        }
    }

    trap::SetUserinfo(
        (*ent).s.number,
        &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
    );

    (*(*ent).client).sess.spectatorClient = 0;

    (*(*ent).client).pers.teamState.state = TEAM_BEGIN;

    ClientUserinfoChanged((*ent).s.number);

    if doBegin != QFALSE {
        ClientBegin((*ent).s.number, QFALSE);
    }
}

/// `void SiegeDoTeamAssign( void )` (g_saga.c:612).
///
/// Swap every connected client's Siege team allegiance (the "switch teams around" pass after a
/// switch-game round): for each [`MAX_CLIENTS`] slot that is a connected client, flip its
/// `siegeDesiredTeam` between team1/team2, and move its current `sessionTeam` to the opposite
/// team via [`SetTeamQuick`] (without re-begin). No oracle (walks the global `g_entities` and
/// fans out into the userinfo/client-spawn trap surface through [`SetTeamQuick`]).
///
/// # Safety
/// Walks the global `g_entities`; the entity/client system must be initialised.
pub unsafe fn SiegeDoTeamAssign() {
    let mut i: c_int = 0;

    //yeah, this is great...
    while i < MAX_CLIENTS as c_int {
        let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*ent).inuse != QFALSE
            && !(*ent).client.is_null()
            && (*(*ent).client).pers.connected == CON_CONNECTED
        {
            //a connected client, switch his frickin teams around
            if (*(*ent).client).sess.siegeDesiredTeam == SIEGETEAM_TEAM1 {
                (*(*ent).client).sess.siegeDesiredTeam = SIEGETEAM_TEAM2;
            } else if (*(*ent).client).sess.siegeDesiredTeam == SIEGETEAM_TEAM2 {
                (*(*ent).client).sess.siegeDesiredTeam = SIEGETEAM_TEAM1;
            }

            if (*(*ent).client).sess.sessionTeam == SIEGETEAM_TEAM1 {
                SetTeamQuick(ent, SIEGETEAM_TEAM2, QFALSE);
            } else if (*(*ent).client).sess.sessionTeam == SIEGETEAM_TEAM2 {
                SetTeamQuick(ent, SIEGETEAM_TEAM1, QFALSE);
            }
        }
        i += 1;
    }
}

/// `void SiegeRespawn( gentity_t *ent )` (g_saga.c:850).
///
/// Respawn a Siege player. If his current `sessionTeam` differs from his `siegeDesiredTeam`,
/// move him there with a re-begin ([`SetTeamQuick`] with `doBegin`). Otherwise spawn him in
/// place ([`ClientSpawn`]) and add a teleport-in temp-entity effect ([`G_TempEntity`] with
/// [`EV_PLAYER_TELEPORT_IN`]). No oracle (client-spawn + temp-entity trap fan-out).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`; the client/entity system
/// must be initialised.
pub unsafe fn SiegeRespawn(ent: *mut gentity_t) {
    let tent: *mut gentity_t;

    if (*(*ent).client).sess.sessionTeam != (*(*ent).client).sess.siegeDesiredTeam {
        SetTeamQuick(ent, (*(*ent).client).sess.siegeDesiredTeam, QTRUE);
    } else {
        ClientSpawn(ent);
        // add a teleportation effect
        tent = G_TempEntity(&(*(*ent).client).ps.origin, EV_PLAYER_TELEPORT_IN);
        (*tent).s.clientNum = (*ent).s.clientNum;
    }
}

/// `void SiegeBeginRound( int entNum )` (g_saga.c:867).
///
/// Perform the round-start tasks. `entNum` is just used as something to fire targets from.
/// When players are not ingame on round start (`!g_preroundState`), respawn everyone now
/// ([`SiegeRespawn`]) — non-spectators, and spectators that have a desired red/blue team. Then,
/// if the level defines a `roundbegin_target`, fire it ([`G_UseTargets2`]). Finally set the
/// `CS_SIEGE_STATE` configstring to "0" (ready to go). No oracle (process-global Siege state +
/// the respawn/target trap fan-out).
///
/// # Safety
/// `entNum` indexes the global `g_entities`; the entity/client system and process-global Siege
/// state must be initialised.
pub unsafe fn SiegeBeginRound(entNum: c_int) {
    let mut targname: [c_char; 1024] = [0; 1024];

    if g_preroundState == 0 {
        //if players are not ingame on round start then respawn them now
        let mut i: c_int = 0;
        let mut spawnEnt: qboolean = QFALSE;

        //respawn everyone now
        while i < MAX_CLIENTS as c_int {
            let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if (*ent).inuse != QFALSE && !(*ent).client.is_null() {
                if (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
                    && ((*(*ent).client).ps.pm_flags & PMF_FOLLOW) == 0
                {
                    //not a spec, just respawn them
                    spawnEnt = QTRUE;
                } else if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR
                    && ((*(*ent).client).sess.siegeDesiredTeam == TEAM_RED
                        || (*(*ent).client).sess.siegeDesiredTeam == TEAM_BLUE)
                {
                    //spectator but has a desired team
                    spawnEnt = QTRUE;
                }
            }

            if spawnEnt != QFALSE {
                SiegeRespawn(ent);

                spawnEnt = QFALSE;
            }
            i += 1;
        }
    }

    //Now check if there's something to fire off at the round start, if so do it.
    if BG_SiegeGetPairedValue(
        addr_of_mut!(siege_info) as *mut c_char,
        c"roundbegin_target".as_ptr() as *mut c_char,
        targname.as_mut_ptr(),
    ) != 0
    {
        if targname[0] != 0 {
            G_UseTargets2(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entNum as usize),
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(entNum as usize),
                targname.as_mut_ptr(),
            );
        }
    }

    trap::SetConfigstring(
        CS_SIEGE_STATE,
        &CStr::from_ptr(va(format_args!("0|{}", (*addr_of!(level)).time))).to_string_lossy(),
    ); //we're ready to g0g0g0
}

/// `void SiegeCheckTimers( void )` (g_saga.c:919).
///
/// Per-frame Siege round timing/state machine. No-ops outside Siege, during intermission, or
/// once the round has ended. While the round has not begun it tallies connected players whose
/// `siegeDesiredTeam` is team1/team2 and keeps each team's countdown reset to the configured
/// time limit (or the persistent "beat their time" value). It then expires the imperial/rebel
/// time limit, completing the round for the other team ([`SiegeRoundComplete`]). Finally, while
/// still pre-round, it manages the begin countdown: waiting for both teams, beginning the round
/// ([`SiegeBeginRound`]) once `gSiegeBeginTime` elapses, and republishing the `CS_SIEGE_STATE`
/// configstring. No oracle (process-global Siege state machine + configstring/round traps).
///
/// # Safety
/// Walks the global `g_entities`/`level`; the entity system and process-global Siege state must
/// be initialised.
pub unsafe fn SiegeCheckTimers() {
    let mut i: c_int = 0;
    let mut numTeam1: c_int = 0;
    let mut numTeam2: c_int = 0;

    if g_gametype.integer != GT_SIEGE {
        return;
    }

    if (*addr_of!(level)).intermissiontime != 0 {
        return;
    }

    if gSiegeRoundEnded != QFALSE {
        return;
    }

    if gSiegeRoundBegun == QFALSE {
        //check if anyone is active on this team - if not, keep the timer set up.
        i = 0;

        while i < MAX_CLIENTS as c_int {
            let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if !ent.is_null()
                && (*ent).inuse != QFALSE
                && !(*ent).client.is_null()
                && (*(*ent).client).pers.connected == CON_CONNECTED
                && (*(*ent).client).sess.siegeDesiredTeam == SIEGETEAM_TEAM1
            {
                numTeam1 += 1;
            }
            i += 1;
        }

        i = 0;

        while i < MAX_CLIENTS as c_int {
            let ent: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if !ent.is_null()
                && (*ent).inuse != QFALSE
                && !(*ent).client.is_null()
                && (*(*ent).client).pers.connected == CON_CONNECTED
                && (*(*ent).client).sess.siegeDesiredTeam == SIEGETEAM_TEAM2
            {
                numTeam2 += 1;
            }
            i += 1;
        }

        if g_siegeTeamSwitch.integer != 0 && (*addr_of!(g_siegePersistant)).beatingTime != QFALSE {
            gImperialCountdown = (*addr_of!(level)).time + (*addr_of!(g_siegePersistant)).lastTime;
            gRebelCountdown = (*addr_of!(level)).time + (*addr_of!(g_siegePersistant)).lastTime;
        } else {
            gImperialCountdown = (*addr_of!(level)).time + imperial_time_limit;
            gRebelCountdown = (*addr_of!(level)).time + rebel_time_limit;
        }
    }

    if imperial_time_limit != 0 {
        //team1
        if gImperialCountdown < (*addr_of!(level)).time {
            SiegeRoundComplete(SIEGETEAM_TEAM2, ENTITYNUM_NONE);
            imperial_time_limit = 0;
            return;
        }
    }

    if rebel_time_limit != 0 {
        //team2
        if gRebelCountdown < (*addr_of!(level)).time {
            SiegeRoundComplete(SIEGETEAM_TEAM1, ENTITYNUM_NONE);
            rebel_time_limit = 0;
            return;
        }
    }

    if gSiegeRoundBegun == QFALSE {
        if numTeam1 == 0 || numTeam2 == 0 {
            //don't have people on both teams yet.
            gSiegeBeginTime = (*addr_of!(level)).time + SIEGE_ROUND_BEGIN_TIME;
            trap::SetConfigstring(CS_SIEGE_STATE, "1"); //"waiting for players on both teams"
        } else if gSiegeBeginTime < (*addr_of!(level)).time {
            //mark the round as having begun
            gSiegeRoundBegun = QTRUE;
            SiegeBeginRound(i); //perform any round start tasks
        } else if gSiegeBeginTime > ((*addr_of!(level)).time + SIEGE_ROUND_BEGIN_TIME) {
            gSiegeBeginTime = (*addr_of!(level)).time + SIEGE_ROUND_BEGIN_TIME;
        } else {
            trap::SetConfigstring(
                CS_SIEGE_STATE,
                &CStr::from_ptr(va(format_args!(
                    "2|{}",
                    gSiegeBeginTime - SIEGE_ROUND_BEGIN_TIME
                )))
                .to_string_lossy(),
            ); //getting ready to begin
        }
    }
}

/// `void SiegeObjectiveCompleted( int team, int objective, int final, int client )` (g_saga.c:1029).
///
/// Mark a Siege objective complete and check for round end. No-ops if the round has ended.
/// Updates the objective configstring ([`G_SiegeSetObjectiveComplete`]); when `final != -1` it
/// bumps the team's completed-goal counter. If this was the final objective (`final == 1`) or the
/// team has met its required goal count, the round is complete ([`SiegeRoundComplete`]);
/// otherwise it just broadcasts the completion ([`BroadcastObjectiveCompletion`]). No oracle
/// (process-global Siege goal state + the broadcast/round-end trap fan-out).
///
/// # Safety
/// `client` indexes the global `g_entities`; the entity system and process-global Siege state
/// must be initialised.
pub unsafe fn SiegeObjectiveCompleted(team: c_int, objective: c_int, final_: c_int, client: c_int) {
    let goals_completed: c_int;
    let goals_required: c_int;

    if gSiegeRoundEnded != QFALSE {
        return;
    }

    //Update the configstring status
    G_SiegeSetObjectiveComplete(team, objective, QFALSE);

    if final_ != -1 {
        if team == SIEGETEAM_TEAM1 {
            imperial_goals_completed += 1;
        } else {
            rebel_goals_completed += 1;
        }
    }

    if team == SIEGETEAM_TEAM1 {
        goals_completed = imperial_goals_completed;
        goals_required = imperial_goals_required;
    } else {
        goals_completed = rebel_goals_completed;
        goals_required = rebel_goals_required;
    }

    if final_ == 1 || goals_completed >= goals_required {
        SiegeRoundComplete(team, client);
    } else {
        BroadcastObjectiveCompletion(team, objective, final_, client);
    }
}

/// `void siegeTriggerUse( gentity_t *ent, gentity_t *other, gentity_t *activator )` (g_saga.c:1074).
///
/// `use` callback for an `info_siege_objective`. No-ops outside a valid Siege game. If the
/// objective is not yet showing on radar it just toggles `EF_RADAROBJECT` on and exits. Otherwise
/// it records the activating client, resolves the team group, looks up `Objective<n>` in
/// `siege_info`, fires the objective's `target` (stripping CR/LF) and the entity's own `target`
/// through [`UseSiegeTarget`], and reports the completion ([`SiegeObjectiveCompleted`]). No oracle
/// (process-global Siege state + the target/objective trap fan-out; `other`/`activator` retained
/// to match the C `use`-callback ABI).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other`/`activator` may be null. Reads/mutates the
/// process-global Siege state.
pub unsafe extern "C" fn siegeTriggerUse(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let mut teamstr: [c_char; 64] = [0; 64];
    let mut objectivestr: [c_char; 64] = [0; 64];
    let mut desiredobjective: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];
    let mut clUser: c_int = ENTITYNUM_NONE;
    let mut final_: c_int = 0;
    let mut i: c_int = 0;

    if siege_valid == 0 {
        return;
    }

    if ((*ent).s.eFlags & EF_RADAROBJECT) == 0 {
        //toggle radar on and exit if it is not showing up already
        (*ent).s.eFlags |= EF_RADAROBJECT;
        return;
    }

    if !activator.is_null() && !(*activator).client.is_null() {
        //activator will hopefully be the person who triggered this event
        clUser = (*activator).s.number;
    }

    if (*ent).side == SIEGETEAM_TEAM1 {
        Com_sprintf(
            teamstr.as_mut_ptr(),
            teamstr.len() as c_int,
            format_args!(
                "{}",
                CStr::from_ptr(addr_of!(team1) as *const c_char).to_string_lossy()
            ),
        );
    } else {
        Com_sprintf(
            teamstr.as_mut_ptr(),
            teamstr.len() as c_int,
            format_args!(
                "{}",
                CStr::from_ptr(addr_of!(team2) as *const c_char).to_string_lossy()
            ),
        );
    }

    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        teamstr.as_mut_ptr(),
        addr_of_mut!(gParseObjectives) as *mut c_char,
    ) != 0
    {
        Com_sprintf(
            objectivestr.as_mut_ptr(),
            objectivestr.len() as c_int,
            format_args!("Objective{}", (*ent).objective),
        );

        if BG_SiegeGetValueGroup(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            objectivestr.as_mut_ptr(),
            desiredobjective.as_mut_ptr(),
        ) != 0
        {
            if BG_SiegeGetPairedValue(
                desiredobjective.as_mut_ptr(),
                c"final".as_ptr() as *mut c_char,
                teamstr.as_mut_ptr(),
            ) != 0
            {
                final_ = atoi(teamstr.as_ptr());
            }

            if BG_SiegeGetPairedValue(
                desiredobjective.as_mut_ptr(),
                c"target".as_ptr() as *mut c_char,
                teamstr.as_mut_ptr(),
            ) != 0
            {
                while teamstr[i as usize] != 0 {
                    if teamstr[i as usize] == b'\r' as c_char
                        || teamstr[i as usize] == b'\n' as c_char
                    {
                        teamstr[i as usize] = b'\0' as c_char;
                    }

                    i += 1;
                }
                UseSiegeTarget(other, activator, teamstr.as_mut_ptr());
            }

            if !(*ent).target.is_null() && *(*ent).target != 0 {
                //use this too
                UseSiegeTarget(other, activator, (*ent).target);
            }

            SiegeObjectiveCompleted((*ent).side, (*ent).objective, final_, clUser);
        }
    }
}

/*QUAKED info_siege_objective (1 0 1) (-16 -16 -24) (16 16 32) ? x x STARTOFFRADAR
STARTOFFRADAR - start not displaying on radar, don't display until used.

"objective" - specifies the objective to complete upon activation
"side" - set to 1 to specify an imperial goal, 2 to specify rebels
"icon" - icon that represents the objective on the radar
*/
/// `void SP_info_siege_objective( gentity_t *ent )` (g_saga.c:1151).
///
/// Spawn-initializer for an `info_siege_objective` entity. Frees itself outside a valid Siege
/// game. Installs [`siegeTriggerUse`] and reads `objective`/`side`; if either is zero (the mapper
/// fux0red something up) it frees itself and prints an error. Unless `STARTOFFRADAR` it shows on
/// radar (`EF_RADAROBJECT`), broadcasts to all clients (so radar tracks it), indexes an optional
/// `icon` into `genericenemyindex`, stashes side/objective into `brokenLimbs`/`frame`, and links
/// into the world. No oracle (entity-state writes plus the Siege spawn syscall fan-out and the
/// global Siege gate).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity system and global Siege state must be
/// initialised.
pub unsafe fn SP_info_siege_objective(ent: *mut gentity_t) {
    let mut s: *mut c_char = core::ptr::null_mut();

    if siege_valid == 0 || g_gametype.integer != GT_SIEGE {
        G_FreeEntity(ent);
        return;
    }

    (*ent).r#use = Some(siegeTriggerUse);
    G_SpawnInt(c"objective".as_ptr(), c"0".as_ptr(), &mut (*ent).objective);
    G_SpawnInt(c"side".as_ptr(), c"0".as_ptr(), &mut (*ent).side);

    if (*ent).objective == 0 || (*ent).side == 0 {
        //j00 fux0red something up
        G_FreeEntity(ent);
        G_Printf("ERROR: info_siege_objective without an objective or side value\n");
        return;
    }

    //Set it up to be drawn on radar
    if ((*ent).spawnflags & SIEGEITEM_STARTOFFRADAR) == 0 {
        (*ent).s.eFlags |= EF_RADAROBJECT;
    }

    //All clients want to know where it is at all times for radar
    (*ent).r.svFlags |= SVF_BROADCAST;

    G_SpawnString(c"icon".as_ptr(), c"".as_ptr(), &mut s);

    if !s.is_null() && *s != 0 {
        // We have an icon, so index it now.  We are reusing the genericenemyindex
        // variable rather than adding a new one to the entity state.
        (*ent).s.genericenemyindex = G_IconIndex(&CStr::from_ptr(s).to_string_lossy());
    }

    (*ent).s.brokenLimbs = (*ent).side;
    (*ent).s.frame = (*ent).objective;
    trap::LinkEntity(ent);
}

/*QUAKED target_siege_end (1 0 1) (-16 -16 -24) (16 16 32)
Do a logexit for siege when used.
*/
/// `void SP_target_siege_end( gentity_t *ent )` (g_saga.c:1339).
///
/// Spawn-initializer for a `target_siege_end` entity. Frees itself outside a valid Siege game,
/// otherwise installs [`siegeEndUse`] as its `use` callback (which does a `LogExit` when
/// triggered). No oracle (entity-state write plus the `G_FreeEntity` syscall and the global Siege
/// gate).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; the entity system and global Siege state must be
/// initialised.
pub unsafe fn SP_target_siege_end(ent: *mut gentity_t) {
    if siege_valid == 0 || g_gametype.integer != GT_SIEGE {
        G_FreeEntity(ent);
        return;
    }

    (*ent).r#use = Some(siegeEndUse);
}

/// `void InitSiegeMode( void )` (g_saga.c:95).
///
/// Initialise everything for a Siege game. Bails (`siege_valid = 0`) outside `GT_SIEGE`. Clears the
/// win-team configstring, restores cross-level persistent timing data when team-switching is on,
/// then opens `maps/<mapname>.siege` and reads it into `siege_info`. From the parsed file it pulls
/// the preround state, the two team group names (honouring the `g_siegeTeam1`/`g_siegeTeam2`
/// overrides), each team's icon / required-objective count / time limit / attacker flag, loads the
/// player classes ([`BG_SiegeLoadClasses`]) and teams ([`BG_SiegeLoadTeams`]), assigns each team's
/// theme, counts objectives to build the initial `CS_SIEGE_OBJECTIVES` status string, precaches
/// sabers ([`BG_PrecacheSabersForSiegeTeam`]) and weapons/holdables
/// ([`G_SiegeRegisterWeaponsAndHoldables`]) for both teams. On any failure it marks `siege_valid = 0`.
/// No oracle (the full Siege bootstrap: file IO, cvar, configstring and precache syscalls over
/// process-global Siege state).
///
/// # Safety
/// Mutates the process-global Siege state (`siege_info`, `siege_valid`, the goal/timer statics,
/// `team1`/`team2`, `gObjectiveCfgStr`); the engine syscall surface and global Siege class/team
/// tables must be initialised. Not re-entrant.
pub unsafe fn InitSiegeMode() {
    let mut mapname: vmCvar_t = vmCvar_t::zeroed();
    let mut levelname: [c_char; 512] = [0; 512];
    let mut teamIcon: [c_char; 128] = [0; 128];
    let mut goalreq: [c_char; 64] = [0; 64];
    let mut teams: [c_char; 2048] = [0; 2048];
    let mut objective: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];
    let mut objecStr: [c_char; 8192] = [0; 8192];
    let len: c_int;
    // C: `int i = 0;` — the 0 initializer is dead (every use first assigns `i = 1`).
    let mut i: c_int;
    //	int				j = 0;
    let mut objectiveNumTeam1: c_int = 0;
    let mut objectiveNumTeam2: c_int = 0;
    let f: fileHandle_t;

    if g_gametype.integer != GT_SIEGE {
        // goto failure;
        siege_valid = 0;
        return;
    }

    //reset
    SiegeSetCompleteData(0);

    //get pers data in case it existed from last level
    if g_siegeTeamSwitch.integer != 0 {
        trap::SiegePersGet(&mut *addr_of_mut!(g_siegePersistant));
        if (*addr_of!(g_siegePersistant)).beatingTime != QFALSE {
            trap::SetConfigstring(
                CS_SIEGE_TIMEOVERRIDE,
                &CStr::from_ptr(va(format_args!(
                    "{}",
                    (*addr_of!(g_siegePersistant)).lastTime
                )))
                .to_string_lossy(),
            );
        } else {
            trap::SetConfigstring(CS_SIEGE_TIMEOVERRIDE, "0");
        }
    } else {
        //hmm, ok, nothing.
        trap::SetConfigstring(CS_SIEGE_TIMEOVERRIDE, "0");
    }

    imperial_goals_completed = 0;
    rebel_goals_completed = 0;

    trap::Cvar_Register(
        Some(&mut mapname),
        "mapname",
        "",
        CVAR_SERVERINFO | CVAR_ROM,
    );

    Com_sprintf(
        levelname.as_mut_ptr(),
        levelname.len() as c_int,
        format_args!(
            "maps/{}.siege\0",
            CStr::from_ptr(mapname.string.as_ptr()).to_string_lossy()
        ),
    );

    if levelname[0] == 0 {
        // goto failure;
        siege_valid = 0;
        return;
    }

    let (l, fh) = trap::FS_FOpenFile(
        &CStr::from_ptr(levelname.as_ptr()).to_string_lossy(),
        FS_READ,
    );
    len = l;
    f = fh;

    if f == 0 || len >= MAX_SIEGE_INFO_SIZE as c_int {
        // goto failure;
        siege_valid = 0;
        return;
    }

    {
        let buf = core::slice::from_raw_parts_mut(
            addr_of_mut!(siege_info) as *mut u8,
            len as usize,
        );
        trap::FS_Read(buf, f);
    }

    trap::FS_FCloseFile(f);

    siege_valid = 1;

    //See if players should be specs or ingame preround
    if BG_SiegeGetPairedValue(
        addr_of_mut!(siege_info) as *mut c_char,
        c"preround_state".as_ptr() as *mut c_char,
        teams.as_mut_ptr(),
    ) != 0
    {
        if teams[0] != 0 {
            g_preroundState = atoi(teams.as_ptr());
        }
    }

    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        c"Teams".as_ptr() as *mut c_char,
        teams.as_mut_ptr(),
    ) != 0
    {
        if (*addr_of!(g_siegeTeam1)).string[0] != 0
            && Q_stricmp((*addr_of!(g_siegeTeam1)).string.as_ptr(), c"none".as_ptr()) != 0
        {
            //check for override
            strcpy(
                addr_of_mut!(team1) as *mut c_char,
                (*addr_of!(g_siegeTeam1)).string.as_ptr(),
            );
        } else {
            //otherwise use level default
            BG_SiegeGetPairedValue(
                teams.as_mut_ptr(),
                c"team1".as_ptr() as *mut c_char,
                addr_of_mut!(team1) as *mut c_char,
            );
        }

        if (*addr_of!(g_siegeTeam2)).string[0] != 0
            && Q_stricmp((*addr_of!(g_siegeTeam2)).string.as_ptr(), c"none".as_ptr()) != 0
        {
            //check for override
            strcpy(
                addr_of_mut!(team2) as *mut c_char,
                (*addr_of!(g_siegeTeam2)).string.as_ptr(),
            );
        } else {
            //otherwise use level default
            BG_SiegeGetPairedValue(
                teams.as_mut_ptr(),
                c"team2".as_ptr() as *mut c_char,
                addr_of_mut!(team2) as *mut c_char,
            );
        }
    } else {
        G_Error("Siege teams not defined");
    }

    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        addr_of_mut!(team2) as *mut c_char,
        addr_of_mut!(gParseObjectives) as *mut c_char,
    ) != 0
    {
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"TeamIcon".as_ptr() as *mut c_char,
            teamIcon.as_mut_ptr(),
        ) != 0
        {
            trap::Cvar_Set(
                "team2_icon",
                &CStr::from_ptr(teamIcon.as_ptr()).to_string_lossy(),
            );
        }

        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"RequiredObjectives".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            rebel_goals_required = atoi(goalreq.as_ptr());
        }
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"Timed".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            rebel_time_limit = atoi(goalreq.as_ptr()) * 1000;
            if g_siegeTeamSwitch.integer != 0 && (*addr_of!(g_siegePersistant)).beatingTime != QFALSE
            {
                gRebelCountdown = (*addr_of!(level)).time + (*addr_of!(g_siegePersistant)).lastTime;
            } else {
                gRebelCountdown = (*addr_of!(level)).time + rebel_time_limit;
            }
        }
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"attackers".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            rebel_attackers = atoi(goalreq.as_ptr());
        }
    }

    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        addr_of_mut!(team1) as *mut c_char,
        addr_of_mut!(gParseObjectives) as *mut c_char,
    ) != 0
    {
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"TeamIcon".as_ptr() as *mut c_char,
            teamIcon.as_mut_ptr(),
        ) != 0
        {
            trap::Cvar_Set(
                "team1_icon",
                &CStr::from_ptr(teamIcon.as_ptr()).to_string_lossy(),
            );
        }

        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"RequiredObjectives".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            imperial_goals_required = atoi(goalreq.as_ptr());
        }
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"Timed".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            if rebel_time_limit != 0 {
                Com_Printf("Tried to set imperial time limit, but there's already a rebel time limit!\nOnly one team can have a time limit.\n");
            } else {
                imperial_time_limit = atoi(goalreq.as_ptr()) * 1000;
                if g_siegeTeamSwitch.integer != 0
                    && (*addr_of!(g_siegePersistant)).beatingTime != QFALSE
                {
                    gImperialCountdown =
                        (*addr_of!(level)).time + (*addr_of!(g_siegePersistant)).lastTime;
                } else {
                    gImperialCountdown = (*addr_of!(level)).time + imperial_time_limit;
                }
            }
        }
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"attackers".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            imperial_attackers = atoi(goalreq.as_ptr());
        }
    }

    //Load the player class types
    BG_SiegeLoadClasses(core::ptr::null_mut());

    if bgNumSiegeClasses == 0 {
        //We didn't find any?!
        G_Error("Couldn't find any player classes for Siege");
    }

    /*
    //We could probably just see what teams are used on this level,
    //then see what classes are used by those teams, and then precache
    //all weapons for said classes. However, I'm just going to do them
    //all for now.
    while (i < bgNumSiegeClasses)
    {
        cl = &bgSiegeClasses[i];
        j = 0;

        while (j < WP_NUM_WEAPONS)
        {
            if (cl->weapons & (1 << j))
            { //we use this weapon so register it.
                RegisterItem(BG_FindItemForWeapon(j));
            }

            j++;
        }

        i++;
    }
    */
    //Ok, I'm adding inventory item precaching now, so I'm finally going to optimize this
    //to only do weapons/items for the current teams used on the level.

    //Now load the teams since we have class data.
    BG_SiegeLoadTeams();

    if bgNumSiegeTeams == 0 {
        //React same as with classes.
        G_Error("Couldn't find any player teams for Siege");
    }

    //Get and set the team themes for each team. This will control which classes can be
    //used on each team.
    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        addr_of_mut!(team1) as *mut c_char,
        addr_of_mut!(gParseObjectives) as *mut c_char,
    ) != 0
    {
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"UseTeam".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            BG_SiegeSetTeamTheme(SIEGETEAM_TEAM1, goalreq.as_mut_ptr());
        }

        //Now count up the objectives for this team.
        i = 1;
        strcpy(
            objecStr.as_mut_ptr(),
            va(format_args!("Objective{}", i)),
        );
        while BG_SiegeGetValueGroup(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            objecStr.as_mut_ptr(),
            objective.as_mut_ptr(),
        ) != 0
        {
            objectiveNumTeam1 += 1;
            i += 1;
            strcpy(
                objecStr.as_mut_ptr(),
                va(format_args!("Objective{}", i)),
            );
        }
    }
    if BG_SiegeGetValueGroup(
        addr_of_mut!(siege_info) as *mut c_char,
        addr_of_mut!(team2) as *mut c_char,
        addr_of_mut!(gParseObjectives) as *mut c_char,
    ) != 0
    {
        if BG_SiegeGetPairedValue(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            c"UseTeam".as_ptr() as *mut c_char,
            goalreq.as_mut_ptr(),
        ) != 0
        {
            BG_SiegeSetTeamTheme(SIEGETEAM_TEAM2, goalreq.as_mut_ptr());
        }

        //Now count up the objectives for this team.
        i = 1;
        strcpy(
            objecStr.as_mut_ptr(),
            va(format_args!("Objective{}", i)),
        );
        while BG_SiegeGetValueGroup(
            addr_of_mut!(gParseObjectives) as *mut c_char,
            objecStr.as_mut_ptr(),
            objective.as_mut_ptr(),
        ) != 0
        {
            objectiveNumTeam2 += 1;
            i += 1;
            strcpy(
                objecStr.as_mut_ptr(),
                va(format_args!("Objective{}", i)),
            );
        }
    }

    //Set the configstring to show status of all current objectives
    strcpy(addr_of_mut!(gObjectiveCfgStr) as *mut c_char, c"t1".as_ptr());
    while objectiveNumTeam1 > 0 {
        //mark them all as not completed since we just initialized
        Q_strcat(
            addr_of_mut!(gObjectiveCfgStr) as *mut c_char,
            1024,
            c"-0".as_ptr(),
        );
        objectiveNumTeam1 -= 1;
    }
    //Finished doing team 1's objectives, now do team 2's
    Q_strcat(
        addr_of_mut!(gObjectiveCfgStr) as *mut c_char,
        1024,
        c"|t2".as_ptr(),
    );
    while objectiveNumTeam2 > 0 {
        Q_strcat(
            addr_of_mut!(gObjectiveCfgStr) as *mut c_char,
            1024,
            c"-0".as_ptr(),
        );
        objectiveNumTeam2 -= 1;
    }

    //And finally set the actual config string
    trap::SetConfigstring(
        CS_SIEGE_OBJECTIVES,
        &CStr::from_ptr(addr_of!(gObjectiveCfgStr) as *const c_char).to_string_lossy(),
    );

    //precache saber data for classes that use sabers on both teams
    BG_PrecacheSabersForSiegeTeam(SIEGETEAM_TEAM1);
    BG_PrecacheSabersForSiegeTeam(SIEGETEAM_TEAM2);

    G_SiegeRegisterWeaponsAndHoldables(SIEGETEAM_TEAM1);
    G_SiegeRegisterWeaponsAndHoldables(SIEGETEAM_TEAM2);
}
