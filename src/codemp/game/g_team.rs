//! Port of `g_team.c` — team gameplay logic (CTF flags, team scoring, team queries).
//! Landed incrementally: `OnSameTeam` lands first as a dependency of `g_combat.c`'s
//! `G_Damage` (friendly-fire checks). The pure team-name/query helpers
//! (`OtherTeam`/`TeamName`/`OtherTeamName`/`TeamColorString`) follow as the most
//! self-contained leaves of the team-flag subsystem (oracle-tested bit-exact), the first
//! step toward unblocking g_items's `Touch_Item`/`Drop_Item`/`LaunchItem`/`G_RunItem`.

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // C global name `teamgame` kept verbatim
#![allow(non_camel_case_types)] // C type name `teamgame_t` kept verbatim

use core::ffi::{c_char, c_int, c_void};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::bg_lib;
use crate::codemp::game::bg_public::{
    team_t, CS_FLAGSTATUS, CTFMESSAGE_FLAG_RETURNED, CTFMESSAGE_FRAGGED_FLAG_CARRIER,
    CTFMESSAGE_PLAYER_CAPTURED_FLAG, CTFMESSAGE_PLAYER_GOT_FLAG, CTFMESSAGE_PLAYER_RETURNED_FLAG,
    ET_NPC, ET_PLAYER, EV_CTFMESSAGE, EV_GLOBAL_TEAM_SOUND, GTS_BLUETEAM_SCORED,
    GTS_BLUETEAM_TOOK_LEAD, GTS_BLUE_CAPTURE, GTS_BLUE_RETURN, GTS_BLUE_TAKEN, GTS_REDTEAM_SCORED,
    GTS_REDTEAM_TOOK_LEAD, GTS_RED_CAPTURE, GTS_RED_RETURN, GTS_RED_TAKEN, GTS_TEAMS_ARE_TIED,
    GT_CTF, GT_CTY, GT_POWERDUEL, GT_SIEGE, GT_SINGLE_PLAYER, GT_TEAM, PERS_ASSIST_COUNT,
    PERS_CAPTURES, PERS_DEFEND_COUNT, PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG, STAT_ARMOR,
    STAT_HEALTH, TEAM_BLUE, TEAM_FREE, TEAM_LOCATION_UPDATE_TIME, TEAM_MAXOVERLAY, TEAM_RED,
    TEAM_SPECTATOR,
};
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::bg_saga_h::SIEGETEAM_TEAM1;
use crate::codemp::game::g_client::{SelectSpawnPoint, SpotWouldTelefrag};
use crate::codemp::game::g_combat::{AddScore, G_CheckVehicleNPCTeamDamage};
use crate::codemp::game::g_items::RespawnItem;
use crate::codemp::game::g_local::{
    gentity_t, playerTeamStateState_t, CON_CONNECTED, FL_DROPPED_ITEM, FL_FORCE_GESTURE,
    REWARD_SPRITE_TIME, TEAM_BEGIN,
};
use crate::codemp::game::g_main::{
    g_entities, g_gametype, g_maxclients, level, CalculateRanks, G_Printf,
};
use crate::codemp::game::g_public_h::{SVF_BOT, SVF_BROADCAST};
use crate::codemp::game::g_utils::{G_Find, G_FreeEntity, G_TempEntity};
use crate::codemp::game::q_math::{vec3_origin, VectorCopy, VectorLength, VectorSubtract};
use crate::codemp::game::q_shared::{
    flagStatus_t, Com_sprintf, Q_stricmp, Sz, FLAG_ATBASE, FLAG_DROPPED, FLAG_TAKEN,
};
use crate::codemp::game::q_shared_h::{vec3_t, MAX_CLIENTS, Q_COLOR_ESCAPE};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `teamgame_t` (g_team.c:8) — the file-scope team-flag game state for CTF / One-Flag CTF.
/// In C this is a bare file global (`teamgame_t teamgame;`); here it is a zero-initialised
/// `static mut` accessed only through raw pointers (`addr_of!`/`addr_of_mut!`), per the
/// `static_mut_refs` discipline.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct teamgame_t {
    pub last_flag_capture: f32,
    pub last_capture_team: c_int,
    pub redStatus: flagStatus_t,  // CTF
    pub blueStatus: flagStatus_t, // CTF
    pub flagStatus: flagStatus_t, // One Flag CTF
    pub redTakenTime: c_int,
    pub blueTakenTime: c_int,
}

/// `teamgame_t teamgame;` (g_team.c:18) — the CTF flag-state file global. Zero-initialised
/// (matching the BSS-zeroed C global before `Team_InitGame` memsets it). Touch only via
/// `addr_of!`/`addr_of_mut!`.
pub static mut teamgame: teamgame_t = teamgame_t {
    last_flag_capture: 0.0,
    last_capture_team: 0,
    redStatus: 0,
    blueStatus: 0,
    flagStatus: 0,
    redTakenTime: 0,
    blueTakenTime: 0,
};

/// `void Team_InitGame( void )` (g_team.c:22) — reset the CTF/CTY team-flag game state at the
/// start of a round: zero the `teamgame` file global (the C `memset(&teamgame, 0, sizeof
/// teamgame)`), then for the flag gametypes force both flag statuses to "at base" by first
/// poisoning `redStatus`/`blueStatus` to `-1` (so the `Team_SetFlagStatus` change-guard always
/// fires and broadcasts the configstring) and calling `Team_SetFlagStatus` for each team.
///
/// No oracle — mutates the `teamgame` file global and (via `Team_SetFlagStatus`) calls
/// `trap_SetConfigstring`.
///
/// # Safety
/// Touches the `teamgame` / `g_gametype` file globals; call only on the game thread.
pub unsafe fn Team_InitGame() {
    *addr_of_mut!(teamgame) = core::mem::zeroed();

    match (*addr_of!(g_gametype)).integer {
        x if x == GT_CTF || x == GT_CTY => {
            (*addr_of_mut!(teamgame)).redStatus = -1; // Invalid to force update
            (*addr_of_mut!(teamgame)).blueStatus = -1;
            Team_SetFlagStatus(TEAM_RED, FLAG_ATBASE);
            Team_SetFlagStatus(TEAM_BLUE, FLAG_ATBASE);
        }
        _ => {}
    }
}

// CTF scoring bonuses and protection radii / timeouts (g_team.h:4-22). Ported here as the
// first consumer (g_team.c's flag-scoring fns) lands; there is no separate `g_team_h.rs`.
pub const CTF_CAPTURE_BONUS: c_int = 100; // what you get for capture
pub const CTF_TEAM_BONUS: c_int = 25; // what your team gets for capture
pub const CTF_RECOVERY_BONUS: c_int = 10; // what you get for recovery
pub const CTF_FLAG_BONUS: c_int = 10; // what you get for picking up enemy flag
pub const CTF_FRAG_CARRIER_BONUS: c_int = 20; // what you get for fragging enemy flag carrier

pub const CTF_CARRIER_DANGER_PROTECT_BONUS: c_int = 5; // bonus for fraggin someone who has recently hurt your flag carrier
pub const CTF_CARRIER_PROTECT_BONUS: c_int = 2; // bonus for fraggin someone while either you or your target are near your flag carrier
pub const CTF_FLAG_DEFENSE_BONUS: c_int = 10; // bonus for fraggin someone while either you or your target are near your flag
pub const CTF_RETURN_FLAG_ASSIST_BONUS: c_int = 10; // awarded for returning a flag that causes a capture to happen almost immediately
pub const CTF_FRAG_CARRIER_ASSIST_BONUS: c_int = 10; // award for fragging a flag carrier if a capture happens almost immediately

pub const CTF_TARGET_PROTECT_RADIUS: f32 = 1000.0; // the radius around an object being defended where a target will be worth extra frags
pub const CTF_ATTACKER_PROTECT_RADIUS: f32 = 1000.0; // the radius around an object being defended where an attacker will get extra frags when making kills

pub const CTF_CARRIER_DANGER_PROTECT_TIMEOUT: c_int = 8000;
pub const CTF_FRAG_CARRIER_ASSIST_TIMEOUT: c_int = 10000;
pub const CTF_RETURN_FLAG_ASSIST_TIMEOUT: c_int = 10000;

/// `static char ctfFlagStatusRemap[]` (g_team.c:270) — maps a `flagStatus_t` to the single
/// configstring character broadcast to clients: `FLAG_ATBASE`→`'0'`, `FLAG_TAKEN`→`'1'`,
/// the two One-Flag-CTF "taken" states→`'*'`, `FLAG_DROPPED`→`'2'`.
static CTF_FLAG_STATUS_REMAP: [u8; 5] = [b'0', b'1', b'*', b'*', b'2'];

/// `int OtherTeam(int team)` (g_team.c:37) — the opposing CTF team. `TEAM_RED`↔`TEAM_BLUE`;
/// anything else (e.g. `TEAM_FREE`/`TEAM_SPECTATOR`) is returned unchanged.
///
/// Oracle-checked against the real Raven C (`jka_OtherTeam`).
pub fn OtherTeam(team: c_int) -> c_int {
    if team == TEAM_RED {
        TEAM_BLUE
    } else if team == TEAM_BLUE {
        TEAM_RED
    } else {
        team
    }
}

/// `const char *TeamName(int team)` (g_team.c:45) — the uppercase name of a team
/// (`"RED"`/`"BLUE"`/`"SPECTATOR"`), defaulting to `"FREE"`.
///
/// Returns a pointer to a `'static` NUL-terminated literal. Oracle-checked against the real
/// Raven C (`jka_TeamName`).
pub fn TeamName(team: c_int) -> *const c_char {
    if team == TEAM_RED {
        c"RED".as_ptr()
    } else if team == TEAM_BLUE {
        c"BLUE".as_ptr()
    } else if team == TEAM_SPECTATOR {
        c"SPECTATOR".as_ptr()
    } else {
        c"FREE".as_ptr()
    }
}

/// `const char *OtherTeamName(int team)` (g_team.c:55) — the name of the *opposing* team
/// (`TEAM_RED`→`"BLUE"`, `TEAM_BLUE`→`"RED"`), with `TEAM_SPECTATOR`→`"SPECTATOR"` and a
/// `"FREE"` default.
///
/// Returns a pointer to a `'static` NUL-terminated literal. Oracle-checked against the real
/// Raven C (`jka_OtherTeamName`).
pub fn OtherTeamName(team: c_int) -> *const c_char {
    if team == TEAM_RED {
        c"BLUE".as_ptr()
    } else if team == TEAM_BLUE {
        c"RED".as_ptr()
    } else if team == TEAM_SPECTATOR {
        c"SPECTATOR".as_ptr()
    } else {
        c"FREE".as_ptr()
    }
}

/// `const char *TeamColorString(int team)` (g_team.c:65) — the Quake color-code string for a
/// team: `^1` (red), `^4` (blue), `^3` (yellow, spectator), `^7` (white) default.
///
/// Returns a pointer to a `'static` NUL-terminated literal. Oracle-checked against the real
/// Raven C (`jka_TeamColorString`).
pub fn TeamColorString(team: c_int) -> *const c_char {
    if team == TEAM_RED {
        c"^1".as_ptr()
    } else if team == TEAM_BLUE {
        c"^4".as_ptr()
    } else if team == TEAM_SPECTATOR {
        c"^3".as_ptr()
    } else {
        c"^7".as_ptr()
    }
}

/// `qboolean OnSameTeam( gentity_t *ent1, gentity_t *ent2 )` (g_team.c:186) — are two
/// entities friendly? Non-clients are never on a team. Power-duel uses `duelTeam`;
/// single-player groups by bot-vs-bot / human-vs-human; below `GT_TEAM` nothing is
/// teamed. Otherwise team vehicles share their pilot's team, NPC-vs-NPC never teams up,
/// an NPC-in-a-vehicle (droid) vs a player teams when their `sessionTeam`/`teamnodmg`
/// matches (via [`G_CheckVehicleNPCTeamDamage`]), and the rest compare `sessionTeam`.
///
/// No oracle — reads the global `g_gametype` cvar and entity/client game state.
///
/// # Safety
/// `ent1`/`ent2` must be valid entities; their `client` pointers are NULL-checked.
pub unsafe fn OnSameTeam(ent1: *mut gentity_t, ent2: *mut gentity_t) -> qboolean {
    if (*ent1).client.is_null() || (*ent2).client.is_null() {
        return QFALSE;
    }

    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
        if (*(*ent1).client).sess.duelTeam == (*(*ent2).client).sess.duelTeam {
            return QTRUE;
        }
        return QFALSE;
    }

    if (*addr_of!(g_gametype)).integer == GT_SINGLE_PLAYER {
        let mut ent1_is_bot: qboolean = QFALSE;
        let mut ent2_is_bot: qboolean = QFALSE;

        if (*ent1).r.svFlags & SVF_BOT != 0 {
            ent1_is_bot = QTRUE;
        }
        if (*ent2).r.svFlags & SVF_BOT != 0 {
            ent2_is_bot = QTRUE;
        }

        if (ent1_is_bot != QFALSE && ent2_is_bot != QFALSE)
            || (ent1_is_bot == QFALSE && ent2_is_bot == QFALSE)
        {
            return QTRUE;
        }
        return QFALSE;
    }

    if (*addr_of!(g_gametype)).integer < GT_TEAM {
        return QFALSE;
    }

    if (*ent1).s.eType == ET_NPC
        && (*ent1).s.NPC_class == CLASS_VEHICLE
        && !(*ent1).client.is_null()
        && (*(*ent1).client).sess.sessionTeam != TEAM_FREE
        && !(*ent2).client.is_null()
        && (*(*ent1).client).sess.sessionTeam == (*(*ent2).client).sess.sessionTeam
    {
        return QTRUE;
    }
    if (*ent2).s.eType == ET_NPC
        && (*ent2).s.NPC_class == CLASS_VEHICLE
        && !(*ent2).client.is_null()
        && (*(*ent2).client).sess.sessionTeam != TEAM_FREE
        && !(*ent1).client.is_null()
        && (*(*ent2).client).sess.sessionTeam == (*(*ent1).client).sess.sessionTeam
    {
        return QTRUE;
    }

    if (*(*ent1).client).sess.sessionTeam == TEAM_FREE
        && (*(*ent2).client).sess.sessionTeam == TEAM_FREE
        && (*ent1).s.eType == ET_NPC
        && (*ent2).s.eType == ET_NPC
    {
        // NPCs don't do normal team rules
        return QFALSE;
    }

    if (*ent1).s.eType == ET_NPC && (*ent2).s.eType == ET_PLAYER {
        if G_CheckVehicleNPCTeamDamage(ent1) != QFALSE {
            // hit an NPC that is in a vehicle - a droid?
            if (*(*ent1).client).sess.sessionTeam == (*(*ent2).client).sess.sessionTeam
                || (*ent1).teamnodmg == (*(*ent2).client).sess.sessionTeam
            {
                return QTRUE;
            }
        }
        return QFALSE;
    } else if (*ent1).s.eType == ET_PLAYER && (*ent2).s.eType == ET_NPC {
        return QFALSE;
    }

    if (*(*ent1).client).sess.sessionTeam == (*(*ent2).client).sess.sessionTeam {
        return QTRUE;
    }

    QFALSE
}

/// `void Team_SetFlagStatus( int team, flagStatus_t status )` (g_team.c:272) — update the
/// stored CTF / One-Flag-CTF flag state for `team` and, if it changed, rebroadcast the
/// `CS_FLAGSTATUS` configstring (a 2-char red/blue status string for CTF/CTY).
///
/// No oracle — mutates the `teamgame` file global and calls `trap_SetConfigstring`.
///
/// Deviation from C: the original builds an uninitialised `char st[4]` and only fills
/// `st[0..2]` (NUL at `st[2]`) inside the CTF/CTY branch, then passes `st` unconditionally —
/// so in non-CTF/CTY modes it sends an uninitialised buffer. That path is effectively dead
/// (this fn only runs in CTF/CTY via `Team_InitGame`); we send an empty string there rather
/// than read uninitialised memory. See `crate/DEVIATIONS.md`.
pub fn Team_SetFlagStatus(team: c_int, status: flagStatus_t) {
    let mut modified = false;

    match team {
        TEAM_RED => unsafe {
            // CTF
            if (*addr_of!(teamgame)).redStatus != status {
                (*addr_of_mut!(teamgame)).redStatus = status;
                modified = true;
            }
        },
        TEAM_BLUE => unsafe {
            // CTF
            if (*addr_of!(teamgame)).blueStatus != status {
                (*addr_of_mut!(teamgame)).blueStatus = status;
                modified = true;
            }
        },
        TEAM_FREE => unsafe {
            // One Flag CTF
            if (*addr_of!(teamgame)).flagStatus != status {
                (*addr_of_mut!(teamgame)).flagStatus = status;
                modified = true;
            }
        },
        _ => {}
    }

    if modified {
        let gametype = unsafe { (*addr_of!(g_gametype)).integer };
        let st = if gametype == GT_CTF || gametype == GT_CTY {
            let (red, blue) = unsafe {
                (
                    (*addr_of!(teamgame)).redStatus,
                    (*addr_of!(teamgame)).blueStatus,
                )
            };
            let st = [
                CTF_FLAG_STATUS_REMAP[red as usize],
                CTF_FLAG_STATUS_REMAP[blue as usize],
            ];
            // Both remap entries are ASCII, so this is always valid UTF-8.
            String::from_utf8_lossy(&st).into_owned()
        } else {
            String::new()
        };

        trap::SetConfigstring(CS_FLAGSTATUS, &st);
    }
}

/// `void Team_CheckDroppedItem( gentity_t *dropped )` (g_team.c:311) — when a flag item is
/// dropped, mark the owning team's flag `FLAG_DROPPED` (dispatched on the item's `giTag`:
/// `PW_REDFLAG`→red, `PW_BLUEFLAG`→blue, `PW_NEUTRALFLAG`→the One-Flag `TEAM_FREE` slot).
///
/// No oracle — dereferences the entity's item and mutates the `teamgame` file global via
/// `Team_SetFlagStatus`.
///
/// # Safety
/// `dropped` must be a valid entity whose `item` pointer is non-NULL (it always is for a
/// dropped item).
pub unsafe fn Team_CheckDroppedItem(dropped: *mut gentity_t) {
    let tag = (*(*dropped).item).giTag;
    if tag == PW_REDFLAG {
        Team_SetFlagStatus(TEAM_RED, FLAG_DROPPED);
    } else if tag == PW_BLUEFLAG {
        Team_SetFlagStatus(TEAM_BLUE, FLAG_DROPPED);
    } else if tag == PW_NEUTRALFLAG {
        Team_SetFlagStatus(TEAM_FREE, FLAG_DROPPED);
    }
}

/// `void PrintCTFMessage(int plIndex, int teamIndex, int ctfMessage)` (g_team.c:100) — fire a
/// broadcast `EV_CTFMESSAGE` temp-entity carrying a CTF status message (flag taken/returned/
/// captured, carrier fragged). `plIndex == -1` is remapped to `MAX_CLIENTS+1` (no player) and
/// `teamIndex == -1` to `50` (no team). For a flag-capture message the stored team is the
/// *losing* team (the opposite of `teamIndex`); otherwise it is `teamIndex` verbatim.
///
/// No oracle — spawns a temp entity and mutates engine-visible entity-state fields.
pub unsafe fn PrintCTFMessage(mut plIndex: c_int, mut teamIndex: c_int, ctfMessage: c_int) {
    if plIndex == -1 {
        plIndex = MAX_CLIENTS as c_int + 1;
    }
    if teamIndex == -1 {
        teamIndex = 50;
    }

    let te = G_TempEntity(&vec3_origin, EV_CTFMESSAGE);
    (*te).r.svFlags |= SVF_BROADCAST;
    (*te).s.eventParm = ctfMessage;
    (*te).s.trickedentindex = plIndex;
    if ctfMessage == CTFMESSAGE_PLAYER_CAPTURED_FLAG {
        if teamIndex == TEAM_RED {
            (*te).s.trickedentindex2 = TEAM_BLUE;
        } else {
            (*te).s.trickedentindex2 = TEAM_RED;
        }
    } else {
        (*te).s.trickedentindex2 = teamIndex;
    }
}

/// `void Team_FragBonuses(gentity_t *targ, gentity_t *inflictor, gentity_t *attacker)`
/// (g_team.c:354) — Calculate the bonuses for flag defense, flag carrier defense, etc. Note
/// that bonuses are not cumulative. You get one, they are in importance order.
///
/// No oracle — CTF scoring side-effects: mutates client `teamState` / `persistant` / `rewardTime`,
/// calls `AddScore`, walks `g_entities`, and reads the `level` / `g_maxclients` globals.
///
/// Faithful 1:1, including the original's dead `tokens` "skull carrier" branch (`tokens` is
/// declared, set to 0 and never reassigned in the original) and its reuse of `v1`/`v2` in the
/// flag-carrier-area-defense block (the C overwrites `v1` twice and tests the stale `v2` from the
/// base-flag-defense block above).
///
/// # Safety
/// `targ`/`attacker` must be valid entities; their `client` pointers are NULL-checked.
pub unsafe fn Team_FragBonuses(
    targ: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
) {
    let mut i: c_int;
    let mut ent: *mut gentity_t;
    let flag_pw: c_int;
    let enemy_flag_pw: c_int;
    let otherteam: c_int;
    let tokens: c_int;
    let mut flag: *mut gentity_t;
    let mut carrier: *mut gentity_t = null_mut();
    let c: *const c_char;
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];
    let team: c_int;

    // no bonus for fragging yourself or team mates
    if (*targ).client.is_null()
        || (*attacker).client.is_null()
        || targ == attacker
        || OnSameTeam(targ, attacker) != QFALSE
    {
        return;
    }

    team = (*(*targ).client).sess.sessionTeam;
    otherteam = OtherTeam((*(*targ).client).sess.sessionTeam);
    if otherteam < 0 {
        return; // whoever died isn't on a team
    }

    // same team, if the flag at base, check to he has the enemy flag
    if team == TEAM_RED {
        flag_pw = PW_REDFLAG;
        enemy_flag_pw = PW_BLUEFLAG;
    } else {
        flag_pw = PW_BLUEFLAG;
        enemy_flag_pw = PW_REDFLAG;
    }

    // did the attacker frag the flag carrier?
    tokens = 0;
    if (*(*targ).client).ps.powerups[enemy_flag_pw as usize] != 0 {
        (*(*attacker).client).pers.teamState.lastfraggedcarrier = (*addr_of!(level)).time as f32;
        AddScore(attacker, &(*targ).r.currentOrigin, CTF_FRAG_CARRIER_BONUS);
        (*(*attacker).client).pers.teamState.fragcarrier += 1;
        //PrintMsg(NULL, "%s" S_COLOR_WHITE " fragged %s's flag carrier!\n",
        //	attacker->client->pers.netname, TeamName(team));
        PrintCTFMessage((*attacker).s.number, team, CTFMESSAGE_FRAGGED_FLAG_CARRIER);

        // the target had the flag, clear the hurt carrier
        // field on the other team
        i = 0;
        while i < (*addr_of!(g_maxclients)).integer {
            ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
            if (*ent).inuse != QFALSE && (*(*ent).client).sess.sessionTeam == otherteam {
                (*(*ent).client).pers.teamState.lasthurtcarrier = 0.0;
            }
            i += 1;
        }
        return;
    }

    // did the attacker frag a head carrier? other->client->ps.generic1
    if tokens != 0 {
        (*(*attacker).client).pers.teamState.lastfraggedcarrier = (*addr_of!(level)).time as f32;
        AddScore(
            attacker,
            &(*targ).r.currentOrigin,
            CTF_FRAG_CARRIER_BONUS * tokens * tokens,
        );
        (*(*attacker).client).pers.teamState.fragcarrier += 1;
        //PrintMsg(NULL, "%s" S_COLOR_WHITE " fragged %s's skull carrier!\n",
        //	attacker->client->pers.netname, TeamName(team));

        // the target had the flag, clear the hurt carrier
        // field on the other team
        i = 0;
        while i < (*addr_of!(g_maxclients)).integer {
            ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
            if (*ent).inuse != QFALSE && (*(*ent).client).sess.sessionTeam == otherteam {
                (*(*ent).client).pers.teamState.lasthurtcarrier = 0.0;
            }
            i += 1;
        }
        return;
    }

    if (*(*targ).client).pers.teamState.lasthurtcarrier != 0.0
        && (*addr_of!(level)).time as f32 - (*(*targ).client).pers.teamState.lasthurtcarrier
            < CTF_CARRIER_DANGER_PROTECT_TIMEOUT as f32
        && (*(*attacker).client).ps.powerups[flag_pw as usize] == 0
    {
        // attacker is on the same team as the flag carrier and
        // fragged a guy who hurt our flag carrier
        AddScore(
            attacker,
            &(*targ).r.currentOrigin,
            CTF_CARRIER_DANGER_PROTECT_BONUS,
        );

        (*(*attacker).client).pers.teamState.carrierdefense += 1;
        (*(*targ).client).pers.teamState.lasthurtcarrier = 0.0;

        (*(*attacker).client).ps.persistant[PERS_DEFEND_COUNT as usize] += 1;
        // C: team = attacker->client->sess.sessionTeam; (dead store — `team` is never read
        // again on this return path; omitted to keep 0 warnings).
        (*(*attacker).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;

        return;
    }

    if (*(*targ).client).pers.teamState.lasthurtcarrier != 0.0
        && (*addr_of!(level)).time as f32 - (*(*targ).client).pers.teamState.lasthurtcarrier
            < CTF_CARRIER_DANGER_PROTECT_TIMEOUT as f32
    {
        // attacker is on the same team as the skull carrier and
        AddScore(
            attacker,
            &(*targ).r.currentOrigin,
            CTF_CARRIER_DANGER_PROTECT_BONUS,
        );

        (*(*attacker).client).pers.teamState.carrierdefense += 1;
        (*(*targ).client).pers.teamState.lasthurtcarrier = 0.0;

        (*(*attacker).client).ps.persistant[PERS_DEFEND_COUNT as usize] += 1;
        // C: team = attacker->client->sess.sessionTeam; (dead store — same as above).
        (*(*attacker).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;

        return;
    }

    // flag and flag carrier area defense bonuses

    // we have to find the flag and carrier entities

    // find the flag
    match (*(*attacker).client).sess.sessionTeam {
        TEAM_RED => {
            c = c"team_CTF_redflag".as_ptr();
        }
        TEAM_BLUE => {
            c = c"team_CTF_blueflag".as_ptr();
        }
        _ => {
            return;
        }
    }
    // find attacker's team's flag carrier
    i = 0;
    while i < (*addr_of!(g_maxclients)).integer {
        carrier = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*carrier).inuse != QFALSE && (*(*carrier).client).ps.powerups[flag_pw as usize] != 0 {
            break;
        }
        carrier = null_mut();
        i += 1;
    }
    flag = null_mut();
    loop {
        flag = G_Find(flag, offset_of!(gentity_t, classname), c);
        if flag.is_null() {
            break;
        }
        if (*flag).flags & FL_DROPPED_ITEM == 0 {
            break;
        }
    }

    if flag.is_null() {
        return; // can't find attacker's flag
    }

    // ok we have the attackers flag and a pointer to the carrier

    // check to see if we are defending the base's flag
    VectorSubtract(&(*targ).r.currentOrigin, &(*flag).r.currentOrigin, &mut v1);
    VectorSubtract(
        &(*attacker).r.currentOrigin,
        &(*flag).r.currentOrigin,
        &mut v2,
    );

    if ((VectorLength(&v1) < CTF_TARGET_PROTECT_RADIUS
        && trap::InPVS(&(*flag).r.currentOrigin, &(*targ).r.currentOrigin) != QFALSE)
        || (VectorLength(&v2) < CTF_TARGET_PROTECT_RADIUS
            && trap::InPVS(&(*flag).r.currentOrigin, &(*attacker).r.currentOrigin) != QFALSE))
        && (*(*attacker).client).sess.sessionTeam != (*(*targ).client).sess.sessionTeam
    {
        // we defended the base flag
        AddScore(attacker, &(*targ).r.currentOrigin, CTF_FLAG_DEFENSE_BONUS);
        (*(*attacker).client).pers.teamState.basedefense += 1;

        (*(*attacker).client).ps.persistant[PERS_DEFEND_COUNT as usize] += 1;
        (*(*attacker).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;

        return;
    }

    if !carrier.is_null() && carrier != attacker {
        VectorSubtract(
            &(*targ).r.currentOrigin,
            &(*carrier).r.currentOrigin,
            &mut v1,
        );
        VectorSubtract(
            &(*attacker).r.currentOrigin,
            &(*carrier).r.currentOrigin,
            &mut v1,
        );

        if ((VectorLength(&v1) < CTF_ATTACKER_PROTECT_RADIUS
            && trap::InPVS(&(*carrier).r.currentOrigin, &(*targ).r.currentOrigin) != QFALSE)
            || (VectorLength(&v2) < CTF_ATTACKER_PROTECT_RADIUS
                && trap::InPVS(&(*carrier).r.currentOrigin, &(*attacker).r.currentOrigin)
                    != QFALSE))
            && (*(*attacker).client).sess.sessionTeam != (*(*targ).client).sess.sessionTeam
        {
            AddScore(
                attacker,
                &(*targ).r.currentOrigin,
                CTF_CARRIER_PROTECT_BONUS,
            );
            (*(*attacker).client).pers.teamState.carrierdefense += 1;

            (*(*attacker).client).ps.persistant[PERS_DEFEND_COUNT as usize] += 1;
            (*(*attacker).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;

            return;
        }
    }
}

/// `void Team_CheckHurtCarrier( gentity_t *targ, gentity_t *attacker )` (g_team.c:535) — record
/// that `attacker` hurt `targ` while `targ` (an enemy) was carrying the flag or a skull, by
/// stamping `attacker`'s `teamState.lasthurtcarrier` with `level.time` (used later when handing
/// out flag-carrier-defense assist bonuses). No-op if either side has no client.
///
/// `lasthurtcarrier` is a C `float`, so the int `level.time` is stored via `as f32` (matching the
/// implicit C int→float conversion). No oracle (reads/mutates entity state + the `level` global).
pub unsafe fn Team_CheckHurtCarrier(targ: *mut gentity_t, attacker: *mut gentity_t) {
    if (*targ).client.is_null() || (*attacker).client.is_null() {
        return;
    }

    let flag_pw = if (*(*targ).client).sess.sessionTeam == TEAM_RED {
        PW_BLUEFLAG
    } else {
        PW_REDFLAG
    };

    let now = (*addr_of!(level)).time as f32;

    // flags
    if (*(*targ).client).ps.powerups[flag_pw as usize] != 0
        && (*(*targ).client).sess.sessionTeam != (*(*attacker).client).sess.sessionTeam
    {
        (*(*attacker).client).pers.teamState.lasthurtcarrier = now;
    }

    // skulls
    if (*(*targ).client).ps.generic1 != 0
        && (*(*targ).client).sess.sessionTeam != (*(*attacker).client).sess.sessionTeam
    {
        (*(*attacker).client).pers.teamState.lasthurtcarrier = now;
    }
}

/// `gentity_t *Team_ResetFlag( int team )` (g_team.c:559) — respawn a team's flag at its base.
/// Walks every entity whose `classname` is the team's flag (`team_CTF_redflag`/`blueflag`/
/// `neutralflag`): dropped flag instances are freed, the base flag is respawned (and returned).
/// Finally sets the team's flag status to `FLAG_ATBASE`. Returns the base flag entity (or NULL
/// for an unknown team).
///
/// No oracle — scans the entity array, frees/respawns entities and mutates the `teamgame`
/// global.
pub unsafe fn Team_ResetFlag(team: c_int) -> *mut gentity_t {
    let c: *const c_char = match team {
        TEAM_RED => c"team_CTF_redflag".as_ptr(),
        TEAM_BLUE => c"team_CTF_blueflag".as_ptr(),
        TEAM_FREE => c"team_CTF_neutralflag".as_ptr(),
        _ => return null_mut(),
    };

    let mut rent: *mut gentity_t = null_mut();
    let mut ent: *mut gentity_t = null_mut();
    loop {
        ent = G_Find(ent, offset_of!(gentity_t, classname), c);
        if ent.is_null() {
            break;
        }
        if (*ent).flags & FL_DROPPED_ITEM != 0 {
            G_FreeEntity(ent);
        } else {
            rent = ent;
            RespawnItem(ent);
        }
    }

    Team_SetFlagStatus(team, FLAG_ATBASE);

    rent
}

/// `void Team_ResetFlags( void )` (g_team.c:592) — in CTF / CTY, reset both the red and blue
/// flags to their bases.
///
/// No oracle — calls `Team_ResetFlag` (entity scan + global mutation).
pub unsafe fn Team_ResetFlags() {
    let gametype = (*addr_of!(g_gametype)).integer;
    if gametype == GT_CTF || gametype == GT_CTY {
        Team_ResetFlag(TEAM_RED);
        Team_ResetFlag(TEAM_BLUE);
    }
}

/// `void AddTeamScore( vec3_t origin, int team, int score )` (g_team.c:142) — apply `score` to
/// `team`'s `level.teamScores` total and broadcast the matching `GTS_*` global team sound at
/// `origin`: tied, took-the-lead, or plain-scored, decided by comparing the post-score totals.
/// `TEAM_RED` is the explicit branch; any other `team` takes the blue branch.
///
/// No oracle — spawns a temp entity, mutates the `level` global, and emits a broadcast sound.
pub unsafe fn AddTeamScore(origin: &vec3_t, team: c_int, score: c_int) {
    let te = G_TempEntity(origin, EV_GLOBAL_TEAM_SOUND);
    (*te).r.svFlags |= SVF_BROADCAST;

    let lvl = addr_of_mut!(level);
    let red = (*lvl).teamScores[TEAM_RED as usize];
    let blue = (*lvl).teamScores[TEAM_BLUE as usize];

    if team == TEAM_RED {
        if red + score == blue {
            // teams are tied sound
            (*te).s.eventParm = GTS_TEAMS_ARE_TIED;
        } else if red <= blue && red + score > blue {
            // red took the lead sound
            (*te).s.eventParm = GTS_REDTEAM_TOOK_LEAD;
        } else {
            // red scored sound
            (*te).s.eventParm = GTS_REDTEAM_SCORED;
        }
    } else if blue + score == red {
        // teams are tied sound
        (*te).s.eventParm = GTS_TEAMS_ARE_TIED;
    } else if blue <= red && blue + score > red {
        // blue took the lead sound
        (*te).s.eventParm = GTS_BLUETEAM_TOOK_LEAD;
    } else {
        // blue scored sound
        (*te).s.eventParm = GTS_BLUETEAM_SCORED;
    }
    (*lvl).teamScores[team as usize] += score;
}

/// `void Team_TakeFlagSound( gentity_t *ent, int team )` (g_team.c:617) — broadcast the
/// enemy-flag-taken global team sound at `ent`'s position. A NULL `ent` only warns. The sound is
/// suppressed if the relevant flag is not at base and was already taken within the last 10s
/// (debounce); the take time is stamped either way. `TEAM_BLUE` plays the red-taken sound,
/// anything else the blue-taken sound.
///
/// No oracle — spawns a temp entity, reads/mutates `teamgame` + `level.time`, broadcasts a sound.
pub unsafe fn Team_TakeFlagSound(ent: *mut gentity_t, team: c_int) {
    if ent.is_null() {
        G_Printf("Warning:  NULL passed to Team_TakeFlagSound\n");
        return;
    }

    let tg = addr_of_mut!(teamgame);
    let now = (*addr_of!(level)).time;

    // only play sound when the flag was at the base
    // or not picked up the last 10 seconds
    match team {
        TEAM_RED => {
            if (*tg).blueStatus != FLAG_ATBASE && (*tg).blueTakenTime > now - 10000 {
                return;
            }
            (*tg).blueTakenTime = now;
        }
        TEAM_BLUE => {
            // CTF
            if (*tg).redStatus != FLAG_ATBASE && (*tg).redTakenTime > now - 10000 {
                return;
            }
            (*tg).redTakenTime = now;
        }
        _ => {}
    }

    let te = G_TempEntity(&(*ent).s.pos.trBase, EV_GLOBAL_TEAM_SOUND);
    if team == TEAM_BLUE {
        (*te).s.eventParm = GTS_RED_TAKEN;
    } else {
        (*te).s.eventParm = GTS_BLUE_TAKEN;
    }
    (*te).r.svFlags |= SVF_BROADCAST;
}

/// `void Team_CaptureFlagSound( gentity_t *ent, int team )` (g_team.c:655) — broadcast the
/// flag-capture global team sound at `ent`'s position. A NULL `ent` only warns. `TEAM_BLUE` plays
/// the blue-capture sound, anything else the red-capture sound.
///
/// No oracle — spawns a temp entity and emits a broadcast sound event.
pub unsafe fn Team_CaptureFlagSound(ent: *mut gentity_t, team: c_int) {
    if ent.is_null() {
        G_Printf("Warning:  NULL passed to Team_CaptureFlagSound\n");
        return;
    }

    let te = G_TempEntity(&(*ent).s.pos.trBase, EV_GLOBAL_TEAM_SOUND);
    if team == TEAM_BLUE {
        (*te).s.eventParm = GTS_BLUE_CAPTURE;
    } else {
        (*te).s.eventParm = GTS_RED_CAPTURE;
    }
    (*te).r.svFlags |= SVF_BROADCAST;
}

/// `void Team_ReturnFlagSound( gentity_t *ent, int team )` (g_team.c:599) — broadcast the
/// "flag returned" global team sound at `ent`'s position. A NULL `ent` only warns (the C
/// passes the result of `Team_ResetFlag`, which can be NULL). `TEAM_BLUE` plays the red-return
/// sound, anything else the blue-return sound.
///
/// No oracle — spawns a temp entity and emits a broadcast sound event.
pub unsafe fn Team_ReturnFlagSound(ent: *mut gentity_t, team: c_int) {
    if ent.is_null() {
        G_Printf("Warning:  NULL passed to Team_ReturnFlagSound\n");
        return;
    }

    let te = G_TempEntity(&(*ent).s.pos.trBase, EV_GLOBAL_TEAM_SOUND);
    if team == TEAM_BLUE {
        (*te).s.eventParm = GTS_RED_RETURN;
    } else {
        (*te).s.eventParm = GTS_BLUE_RETURN;
    }
    (*te).r.svFlags |= SVF_BROADCAST;
}

/// `void Team_ReturnFlag( int team )` (g_team.c:673) — reset a team's flag to base and play
/// the return sound, then (for a real team, not the One-Flag `TEAM_FREE`) broadcast the
/// `CTFMESSAGE_FLAG_RETURNED` CTF message.
///
/// No oracle — composes `Team_ResetFlag` / `Team_ReturnFlagSound` / `PrintCTFMessage`.
pub unsafe fn Team_ReturnFlag(team: c_int) {
    Team_ReturnFlagSound(Team_ResetFlag(team), team);
    if team == TEAM_FREE {
        // PrintMsg(NULL, "The flag has returned!\n" );
    } else {
        // flag should always have team in normal CTF
        // PrintMsg(NULL, "The %s flag has returned!\n", TeamName(team));
        PrintCTFMessage(-1, team, CTFMESSAGE_FLAG_RETURNED);
    }
}

/// `void Team_FreeEntity( gentity_t *ent )` (g_team.c:684) — when a dropped flag item is freed
/// by the engine, return the owning team's flag to its base (dispatched on the item's `giTag`).
/// Assigned to flag items so a flag returns when its dropped instance times out / is collected.
///
/// No oracle — dereferences the item and calls `Team_ReturnFlag`.
///
/// # Safety
/// `ent` must be a valid entity whose `item` pointer is non-NULL (it always is for a flag item).
pub unsafe fn Team_FreeEntity(ent: *mut gentity_t) {
    let tag = (*(*ent).item).giTag;
    if tag == PW_REDFLAG {
        Team_ReturnFlag(TEAM_RED);
    } else if tag == PW_BLUEFLAG {
        Team_ReturnFlag(TEAM_BLUE);
    } else if tag == PW_NEUTRALFLAG {
        Team_ReturnFlag(TEAM_FREE);
    }
}

/// `void Team_DroppedFlagThink(gentity_t *ent)` (g_team.c:705) — think function set in
/// `LaunchItem` for dropped flag items: when the drop times out, return the flag to its base
/// (team dispatched on the item's `giTag`). `Team_ResetFlag` deletes this entity.
///
/// No oracle — dereferences the item and calls `Team_ResetFlag` / `Team_ReturnFlagSound`.
///
/// # Safety
/// `ent` must be a valid entity whose `item` pointer is non-NULL (it always is for a flag item).
pub unsafe fn Team_DroppedFlagThink(ent: *mut gentity_t) {
    let mut team = TEAM_FREE;

    let tag = (*(*ent).item).giTag;
    if tag == PW_REDFLAG {
        team = TEAM_RED;
    } else if tag == PW_BLUEFLAG {
        team = TEAM_BLUE;
    } else if tag == PW_NEUTRALFLAG {
        team = TEAM_FREE;
    }

    Team_ReturnFlagSound(Team_ResetFlag(team), team);
    // Reset Flag will delete this entity
}

/// `int Team_TouchOurFlag( gentity_t *ent, gentity_t *other, int team )` (g_team.c:728) —
/// `other` touched its *own* team's flag entity `ent`. If `ent` is a dropped instance, `other`
/// recovers it (recovery bonus + return sound, flag reset). Otherwise the flag is at base: if
/// `other` is carrying the enemy flag this is a capture — clear the carried flag, bump the team
/// score, hand out the capture/team/assist bonuses across the team, reset the flags and
/// recalculate ranks. Always returns 0 (do not auto-respawn).
///
/// No oracle — flag-capture scoring side-effects: mutates client `ps`/`teamState`/`persistant`/
/// `rewardTime`, the `teamgame`/`level` globals, walks `g_entities` and calls `AddScore`/
/// `AddTeamScore`/`CalculateRanks`/flag-reset helpers.
///
/// # Safety
/// `ent`/`other` must be valid entities; `other->client` must be non-NULL (it is, for a touch).
pub unsafe fn Team_TouchOurFlag(ent: *mut gentity_t, other: *mut gentity_t, team: c_int) -> c_int {
    let mut i: c_int;
    let mut player: *mut gentity_t;
    let cl = (*other).client;
    let enemy_flag: c_int;

    if (*cl).sess.sessionTeam == TEAM_RED {
        enemy_flag = PW_BLUEFLAG;
    } else {
        enemy_flag = PW_REDFLAG;
    }

    if (*ent).flags & FL_DROPPED_ITEM != 0 {
        // hey, its not home.  return it by teleporting it back
        //PrintMsg( NULL, "%s" S_COLOR_WHITE " returned the %s flag!\n",
        //	cl->pers.netname, TeamName(team));
        PrintCTFMessage((*other).s.number, team, CTFMESSAGE_PLAYER_RETURNED_FLAG);

        AddScore(other, &(*ent).r.currentOrigin, CTF_RECOVERY_BONUS);
        (*(*other).client).pers.teamState.flagrecovery += 1;
        (*(*other).client).pers.teamState.lastreturnedflag = (*addr_of!(level)).time as f32;
        //ResetFlag will remove this entity!  We must return zero
        Team_ReturnFlagSound(Team_ResetFlag(team), team);
        return 0;
    }

    // the flag is at home base.  if the player has the enemy
    // flag, he's just won!
    if (*cl).ps.powerups[enemy_flag as usize] == 0 {
        return 0; // We don't have the flag
    }
    //PrintMsg( NULL, "%s" S_COLOR_WHITE " captured the %s flag!\n", cl->pers.netname, TeamName(OtherTeam(team)));
    PrintCTFMessage((*other).s.number, team, CTFMESSAGE_PLAYER_CAPTURED_FLAG);

    (*cl).ps.powerups[enemy_flag as usize] = 0;

    (*addr_of_mut!(teamgame)).last_flag_capture = (*addr_of!(level)).time as f32;
    (*addr_of_mut!(teamgame)).last_capture_team = team;

    // Increase the team's score
    AddTeamScore(&(*ent).s.pos.trBase, (*(*other).client).sess.sessionTeam, 1);
    //	Team_ForceGesture(other->client->sess.sessionTeam);
    //rww - don't really want to do this now. Mainly because performing a gesture disables your upper torso animations until it's done and you can't fire

    (*(*other).client).pers.teamState.captures += 1;
    (*(*other).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;
    (*(*other).client).ps.persistant[PERS_CAPTURES as usize] += 1;

    // other gets another 10 frag bonus
    AddScore(other, &(*ent).r.currentOrigin, CTF_CAPTURE_BONUS);

    Team_CaptureFlagSound(ent, team);

    // Ok, let's do the player loop, hand out the bonuses
    i = 0;
    while i < (*addr_of!(g_maxclients)).integer {
        player = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*player).inuse == QFALSE {
            i += 1;
            continue;
        }

        if (*(*player).client).sess.sessionTeam != (*cl).sess.sessionTeam {
            (*(*player).client).pers.teamState.lasthurtcarrier = -5.0;
        } else if (*(*player).client).sess.sessionTeam == (*cl).sess.sessionTeam {
            if player != other {
                AddScore(player, &(*ent).r.currentOrigin, CTF_TEAM_BONUS);
            }
            // award extra points for capture assists
            if (*(*player).client).pers.teamState.lastreturnedflag
                + CTF_RETURN_FLAG_ASSIST_TIMEOUT as f32
                > (*addr_of!(level)).time as f32
            {
                AddScore(
                    player,
                    &(*ent).r.currentOrigin,
                    CTF_RETURN_FLAG_ASSIST_BONUS,
                );
                (*(*other).client).pers.teamState.assists += 1;

                (*(*player).client).ps.persistant[PERS_ASSIST_COUNT as usize] += 1;
                (*(*player).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;
            } else if (*(*player).client).pers.teamState.lastfraggedcarrier
                + CTF_FRAG_CARRIER_ASSIST_TIMEOUT as f32
                > (*addr_of!(level)).time as f32
            {
                AddScore(
                    player,
                    &(*ent).r.currentOrigin,
                    CTF_FRAG_CARRIER_ASSIST_BONUS,
                );
                (*(*other).client).pers.teamState.assists += 1;
                (*(*player).client).ps.persistant[PERS_ASSIST_COUNT as usize] += 1;
                (*(*player).client).rewardTime = (*addr_of!(level)).time + REWARD_SPRITE_TIME;
            }
        }
        i += 1;
    }
    Team_ResetFlags();

    CalculateRanks();

    0 // Do not respawn this automatically
}

/// `int Team_TouchEnemyFlag( gentity_t *ent, gentity_t *other, int team )` (g_team.c:818) —
/// `other` picked up the enemy `team`'s flag entity `ent`: broadcast the got-flag message, give
/// `other` the never-expiring flag powerup, mark the team's flag `FLAG_TAKEN`, award the pickup
/// bonus, stamp `flagsince` and play the take sound. Returns -1 (do not auto-respawn, but delete
/// it if it was a dropped instance).
///
/// No oracle — flag-pickup side-effects: mutates client `ps`/`teamState`, the `teamgame` global
/// (via `Team_SetFlagStatus`), and calls `AddScore`/`Team_TakeFlagSound`.
///
/// `INT_MAX` (flags never expire) is `c_int::MAX`.
///
/// # Safety
/// `ent`/`other` must be valid entities; `other->client` must be non-NULL (it is, for a touch).
pub unsafe fn Team_TouchEnemyFlag(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    team: c_int,
) -> c_int {
    let cl = (*other).client;

    //PrintMsg (NULL, "%s" S_COLOR_WHITE " got the %s flag!\n",
    //	other->client->pers.netname, TeamName(team));
    PrintCTFMessage((*other).s.number, team, CTFMESSAGE_PLAYER_GOT_FLAG);

    if team == TEAM_RED {
        (*cl).ps.powerups[PW_REDFLAG as usize] = c_int::MAX; // flags never expire
    } else {
        (*cl).ps.powerups[PW_BLUEFLAG as usize] = c_int::MAX; // flags never expire
    }

    Team_SetFlagStatus(team, FLAG_TAKEN);

    AddScore(other, &(*ent).r.currentOrigin, CTF_FLAG_BONUS);
    (*cl).pers.teamState.flagsince = (*addr_of!(level)).time as f32;
    Team_TakeFlagSound(ent, team);

    -1 // Do not respawn this automatically, but do delete it if it was FL_DROPPED
}

extern "C" {
    /// libc `int strcmp( const char *, const char * )` — the original `Pickup_Team` uses the
    /// libc `strcmp` (bg_lib's copy is `Q3_VM`-only), so bind it directly here.
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
}

/// `int Pickup_Team( gentity_t *ent, gentity_t *other )` (g_team.c:839) — dispatch a CTF flag
/// touch: identify which team's flag `ent` is by classname, then (in GT_CTF) route to
/// [`Team_TouchOurFlag`] when it is `other`'s own flag, else [`Team_TouchEnemyFlag`]. Returns 0
/// for an unrecognised flag.
///
/// No oracle — composes the two flag-touch handlers (their scoring side-effects).
///
/// # Safety
/// `ent`/`other` must be valid entities; `ent->classname` is NUL-terminated and `other->client`
/// is non-NULL (it is, for a touch).
pub unsafe fn Pickup_Team(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let team: c_int;
    let cl = (*other).client;

    // figure out what team this flag is
    if strcmp((*ent).classname, c"team_CTF_redflag".as_ptr()) == 0 {
        team = TEAM_RED;
    } else if strcmp((*ent).classname, c"team_CTF_blueflag".as_ptr()) == 0 {
        team = TEAM_BLUE;
    } else if strcmp((*ent).classname, c"team_CTF_neutralflag".as_ptr()) == 0 {
        team = TEAM_FREE;
    } else {
        //		PrintMsg ( other, "Don't know what team the flag is on.\n");
        return 0;
    }
    // GT_CTF
    if team == (*cl).sess.sessionTeam {
        return Team_TouchOurFlag(ent, other, team);
    }
    Team_TouchEnemyFlag(ent, other, team)
}

/// `gentity_t *Team_GetLocation(gentity_t *ent)` (g_team.c:871) — find the nearest `target_location`
/// entity to `ent` that is in `ent`'s PVS, walking the `level.locationHead` → `nextTrain` list and
/// keeping the closest by squared distance. Returns the best location entity, or NULL if none.
///
/// No oracle — traverses a `gentity_t` linked list and calls `trap_InPVS`.
pub unsafe fn Team_GetLocation(ent: *mut gentity_t) -> *mut gentity_t {
    let mut best: *mut gentity_t = null_mut();
    let mut bestlen: f32 = 3.0 * 8192.0 * 8192.0;

    let mut origin: vec3_t = [0.0; 3];
    VectorCopy(&(*ent).r.currentOrigin, &mut origin);

    let mut eloc = (*addr_of!(level)).locationHead;
    while !eloc.is_null() {
        let eo = (*eloc).r.currentOrigin;
        let len = (origin[0] - eo[0]) * (origin[0] - eo[0])
            + (origin[1] - eo[1]) * (origin[1] - eo[1])
            + (origin[2] - eo[2]) * (origin[2] - eo[2]);

        if len <= bestlen && trap::InPVS(&origin, &eo) == QTRUE {
            bestlen = len;
            best = eloc;
        }

        eloc = (*eloc).nextTrain;
    }

    best
}

/// `qboolean Team_GetLocationMsg(gentity_t *ent, char *loc, int loclen)` (g_team.c:910) — write the
/// nearest location's name into `loc` for `ent`, prefixed (when the location has a `count` colour
/// index, clamped to 0..=7) with a `^N` colour escape and suffixed with `S_COLOR_WHITE`. Returns
/// `qfalse` if no nearby `target_location` was found.
///
/// No oracle — calls `Team_GetLocation` (entity-list + trap) and `Com_sprintf`.
pub unsafe fn Team_GetLocationMsg(
    ent: *mut gentity_t,
    loc: *mut c_char,
    loclen: c_int,
) -> qboolean {
    let best = Team_GetLocation(ent);

    if best.is_null() {
        return QFALSE;
    }

    if (*best).count != 0 {
        if (*best).count < 0 {
            (*best).count = 0;
        }
        if (*best).count > 7 {
            (*best).count = 7;
        }
        // "%c%c%s" S_COLOR_WHITE  with  Q_COLOR_ESCAPE, best->count + '0', best->message
        let digit = ((*best).count as u8 + b'0') as char;
        Com_sprintf(
            loc,
            loclen,
            format_args!(
                "{}{}{}^7",
                Q_COLOR_ESCAPE as u8 as char,
                digit,
                Sz((*best).message)
            ),
        );
    } else {
        Com_sprintf(loc, loclen, format_args!("{}", Sz((*best).message)));
    }

    QTRUE
}

/// `void Team_ForceGesture(int team)` — flag every in-use client on `team` to play a
/// gesture by OR-ing `FL_FORCE_GESTURE` into its entity flags.
pub unsafe fn Team_ForceGesture(team: c_int) {
    for i in 0..MAX_CLIENTS as usize {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);
        if (*ent).inuse == QFALSE {
            continue;
        }
        if (*ent).client.is_null() {
            continue;
        }
        if (*(*ent).client).sess.sessionTeam != team {
            continue;
        }
        //
        (*ent).flags |= FL_FORCE_GESTURE;
    }
}

/// `void TeamplayInfoMessage( gentity_t *ent )` (g_team.c:1094) — build and send the
/// team-overlay (`tinfo`) message to `ent`. Gathers up to `TEAM_MAXOVERLAY` same-team
/// clients (top players, in client order from `level.sortedClients`), sorts them by
/// clientNum via [`SortClients`], then emits per-client `" %i %i %i %i %i %i"` entries
/// (client index, location, clamped health, clamped armor, weapon, powerups) into a
/// fixed 8192-byte buffer, breaking on overflow exactly as the C does.
///
/// No oracle: drives `trap_SendServerCommand` and dereferences the entity / client
/// arrays + `level.sortedClients` (entity/trap scaffolding, per the no-oracle precedent
/// for `SendScoreboardMessageToAllClients`-class fns). Faithful 1:1.
///
/// SAFETY: `ent` is a live client entity; the indices read from `level.sortedClients`
/// are in range for `g_entities` (populated by `CalculateRanks`).
pub unsafe fn TeamplayInfoMessage(ent: *mut gentity_t) {
    // C: char string[8192]; built up by per-client strcpy with an overflow guard.
    let mut string = String::new();
    let mut stringlength: usize;
    let mut cnt: c_int;
    let mut clients = [0 as c_int; TEAM_MAXOVERLAY as usize];

    if (*(*ent).client).pers.teamInfo == QFALSE {
        return;
    }

    let max = (*addr_of!(g_maxclients)).integer;
    let sorted = &(*addr_of!(level)).sortedClients;

    // figure out what client should be on the display
    // we are limited to 8, but we want to use the top eight players
    // but in client order (so they don't keep changing position on the overlay)
    let mut i = 0;
    cnt = 0;
    while i < max && cnt < TEAM_MAXOVERLAY {
        let player = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add(sorted[i as usize] as usize);
        if (*player).inuse == QTRUE
            && (*(*player).client).sess.sessionTeam == (*(*ent).client).sess.sessionTeam
        {
            clients[cnt as usize] = sorted[i as usize];
            cnt += 1;
        }
        i += 1;
    }

    // We have the top eight players, sort them by clientNum
    bg_lib::qsort(
        clients.as_mut_ptr() as *mut c_void,
        cnt as usize,
        core::mem::size_of::<c_int>(),
        SortClients,
    );

    // send the latest information on all clients
    stringlength = 0;

    let mut i = 0;
    cnt = 0;
    while i < max && cnt < TEAM_MAXOVERLAY {
        let player = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*player).inuse == QTRUE
            && (*(*player).client).sess.sessionTeam == (*(*ent).client).sess.sessionTeam
        {
            let mut h = (*(*player).client).ps.stats[STAT_HEALTH as usize];
            let mut a = (*(*player).client).ps.stats[STAT_ARMOR as usize];
            if h < 0 {
                h = 0;
            }
            if a < 0 {
                a = 0;
            }

            // C: Com_sprintf(entry, ..., " %i %i %i %i %i %i", i, location, h, a, weapon, powerups)
            let entry = format!(
                " {} {} {} {} {} {}",
                i,
                (*(*player).client).pers.teamState.location,
                h,
                a,
                (*(*player).client).ps.weapon,
                (*player).s.powerups
            );
            let j = entry.len();
            // C: if (stringlength + j > sizeof(string)) break;
            if stringlength + j > 8192 {
                break;
            }
            string.push_str(&entry);
            stringlength += j;
            cnt += 1;
        }
        i += 1;
    }

    trap::SendServerCommand(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        &format!("tinfo {cnt} {string}"),
    );
}

/// `void CheckTeamStatus(void)` (g_team.c:1152) — periodically (every
/// `TEAM_LOCATION_UPDATE_TIME` ms) refresh each connected red/blue client's
/// `pers.teamState.location` from the nearest `target_location` ([`Team_GetLocation`]),
/// then push the updated team overlay to each via [`TeamplayInfoMessage`].
///
/// No oracle: walks the entity array, calls the location-finding trap path and
/// `trap_SendServerCommand` (entity/trap scaffolding precedent). Faithful 1:1.
pub unsafe fn CheckTeamStatus() {
    if (*addr_of!(level)).time - (*addr_of!(level)).lastTeamLocationTime > TEAM_LOCATION_UPDATE_TIME
    {
        (*addr_of_mut!(level)).lastTeamLocationTime = (*addr_of!(level)).time;

        let max = (*addr_of!(g_maxclients)).integer;

        for i in 0..max {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if (*ent).client.is_null() {
                continue;
            }

            if (*(*ent).client).pers.connected != CON_CONNECTED {
                continue;
            }

            if (*ent).inuse == QTRUE
                && ((*(*ent).client).sess.sessionTeam == TEAM_RED
                    || (*(*ent).client).sess.sessionTeam == TEAM_BLUE)
            {
                let loc = Team_GetLocation(ent);
                if !loc.is_null() {
                    (*(*ent).client).pers.teamState.location = (*loc).health;
                } else {
                    (*(*ent).client).pers.teamState.location = 0;
                }
            }
        }

        for i in 0..max {
            let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

            if (*(*ent).client).pers.connected != CON_CONNECTED {
                continue;
            }

            if (*ent).inuse == QTRUE
                && ((*(*ent).client).sess.sessionTeam == TEAM_RED
                    || (*(*ent).client).sess.sessionTeam == TEAM_BLUE)
            {
                TeamplayInfoMessage(ent);
            }
        }
    }
}

/// `void SP_team_CTF_redplayer( gentity_t *ent )` (g_team.c:1200) — CTF red game-start spawn
/// marker. The spawn entity carries only its position/angles; the function body is empty (the
/// spawn-point selectors find these by classname). No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_team_CTF_redplayer(_ent: *mut gentity_t) {}

/// `void SP_team_CTF_blueplayer( gentity_t *ent )` (g_team.c:1207) — CTF blue game-start spawn
/// marker. Empty body, like its red counterpart. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_team_CTF_blueplayer(_ent: *mut gentity_t) {}

/// `void SP_team_CTF_redspawn( gentity_t *ent )` (g_team.c:1215) — CTF red respawn-point
/// marker. Empty body. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_team_CTF_redspawn(_ent: *mut gentity_t) {}

/// `void SP_team_CTF_bluespawn( gentity_t *ent )` (g_team.c:1222) — CTF blue respawn-point
/// marker. Empty body. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_team_CTF_bluespawn(_ent: *mut gentity_t) {}

/// `static int QDECL SortClients( const void *a, const void *b )` — ascending qsort
/// comparator over raw client-index ints.
pub extern "C" fn SortClients(a: *const c_void, b: *const c_void) -> c_int {
    unsafe { *(a as *const c_int) - *(b as *const c_int) }
}

const MAX_TEAM_SPAWN_POINTS: usize = 32;

/// `gentity_t *SelectRandomTeamSpawnPoint( int teamstate, team_t team, int siegeClass )`
/// (g_team.c:942). Picks a team/siege spawn marker: chooses the classname by gametype/team
/// (`info_player_siegeteamN` in siege — which additionally must be "enabled" via
/// `genericValue1`; else `team_CTF_{red,blue}{player,spawn}` keyed on whether this is a
/// round-start (`TEAM_BEGIN`) or in-game respawn), collects up to 32 non-telefragging spots,
/// and returns one at random. In siege, if any collected spot's `idealclass` matches the
/// requested class's name it restricts the random pick to those. Falls back to the first
/// matching spot (telefrag-allowed) if none are clear, or NULL for a bad team. No oracle
/// (walks `g_entities`, calls `SpotWouldTelefrag`/`G_Find`, draws `rand()`).
///
/// # Safety
/// The `g_entities`/`level` globals + `bgSiegeClasses` must be initialised.
pub unsafe fn SelectRandomTeamSpawnPoint(
    teamstate: playerTeamStateState_t,
    team: team_t,
    siegeClass: c_int,
) -> *mut gentity_t {
    let mut spots: [*mut gentity_t; MAX_TEAM_SPAWN_POINTS] = [null_mut(); MAX_TEAM_SPAWN_POINTS];
    let classname: *const c_char;
    let mut mustBeEnabled = QFALSE;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        if team == SIEGETEAM_TEAM1 {
            classname = c"info_player_siegeteam1".as_ptr();
        } else {
            classname = c"info_player_siegeteam2".as_ptr();
        }

        mustBeEnabled = QTRUE; //siege spawn points need to be "enabled" to be used (because multiple spawnpoint sets can be placed at once)
    } else if teamstate == TEAM_BEGIN {
        if team == TEAM_RED {
            classname = c"team_CTF_redplayer".as_ptr();
        } else if team == TEAM_BLUE {
            classname = c"team_CTF_blueplayer".as_ptr();
        } else {
            return null_mut();
        }
    } else if team == TEAM_RED {
        classname = c"team_CTF_redspawn".as_ptr();
    } else if team == TEAM_BLUE {
        classname = c"team_CTF_bluespawn".as_ptr();
    } else {
        return null_mut();
    }
    let mut count: i32 = 0;

    let mut spot: *mut gentity_t = null_mut();

    loop {
        spot = G_Find(spot, offset_of!(gentity_t, classname), classname);
        if spot.is_null() {
            break;
        }
        if SpotWouldTelefrag(spot) == QTRUE {
            continue;
        }

        if mustBeEnabled == QTRUE && (*spot).genericValue1 == 0 {
            //siege point that's not enabled, can't use it
            continue;
        }

        spots[count as usize] = spot;
        count += 1;
        if count as usize == MAX_TEAM_SPAWN_POINTS {
            break;
        }
    }

    if count == 0 {
        // no spots that won't telefrag
        return G_Find(null_mut(), offset_of!(gentity_t, classname), classname);
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && siegeClass >= 0
        && (*addr_of!(bgSiegeClasses))[siegeClass as usize].name[0] != 0
    {
        //out of the spots found, see if any have an idealclass to match our class name
        let mut classSpots: [*mut gentity_t; MAX_TEAM_SPAWN_POINTS] =
            [null_mut(); MAX_TEAM_SPAWN_POINTS];
        let mut classCount: i32 = 0;
        let mut i: i32 = 0;

        while i < count {
            let s = spots[i as usize];
            if !s.is_null()
                && !(*s).idealclass.is_null()
                && *(*s).idealclass != 0
                && Q_stricmp(
                    (*s).idealclass,
                    (*addr_of!(bgSiegeClasses))[siegeClass as usize]
                        .name
                        .as_ptr(),
                ) == 0
            {
                //this spot's idealclass matches the class name
                classSpots[classCount as usize] = s;
                classCount += 1;
            }
            i += 1;
        }

        if classCount > 0 {
            //found at least one
            let selection = bg_lib::rand() % classCount;
            return spots[selection as usize];
        }
    }

    let selection = bg_lib::rand() % count;
    spots[selection as usize]
}

/// `gentity_t *SelectCTFSpawnPoint ( team_t team, int teamstate, vec3_t origin, vec3_t angles )`
/// (g_team.c:1040). Thin wrapper over [`SelectRandomTeamSpawnPoint`] (siegeClass -1): on a
/// hit, writes the spot origin (raised 9 units) + angles into `origin`/`angles`; on a miss
/// falls back to [`SelectSpawnPoint`] from `vec3_origin`. No oracle.
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised; `origin`/`angles` must be valid.
pub unsafe fn SelectCTFSpawnPoint(
    team: team_t,
    teamstate: playerTeamStateState_t,
    origin: &mut vec3_t,
    angles: &mut vec3_t,
) -> *mut gentity_t {
    let spot = SelectRandomTeamSpawnPoint(teamstate, team, -1);

    if spot.is_null() {
        return SelectSpawnPoint(&vec3_origin, origin, angles, team);
    }

    VectorCopy(&(*spot).s.origin, origin);
    origin[2] += 9.0;
    VectorCopy(&(*spot).s.angles, angles);

    spot
}

/// `gentity_t *SelectSiegeSpawnPoint ( int siegeClass, team_t team, int teamstate, vec3_t origin, vec3_t angles )`
/// (g_team.c:1062). As [`SelectCTFSpawnPoint`] but threads `siegeClass` through to
/// [`SelectRandomTeamSpawnPoint`]; same origin/angles write + `SelectSpawnPoint` fallback.
/// No oracle.
///
/// # Safety
/// The `g_entities`/`level` globals must be initialised; `origin`/`angles` must be valid.
pub unsafe fn SelectSiegeSpawnPoint(
    siegeClass: c_int,
    team: team_t,
    teamstate: playerTeamStateState_t,
    origin: &mut vec3_t,
    angles: &mut vec3_t,
) -> *mut gentity_t {
    let spot = SelectRandomTeamSpawnPoint(teamstate, team, siegeClass);

    if spot.is_null() {
        return SelectSpawnPoint(&vec3_origin, origin, angles, team);
    }

    VectorCopy(&(*spot).s.origin, origin);
    origin[2] += 9.0;
    VectorCopy(&(*spot).s.angles, angles);

    spot
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use core::ffi::CStr;

    extern "C" {
        fn jka_OtherTeam(team: c_int) -> c_int;
        fn jka_TeamName(team: c_int) -> *const c_char;
        fn jka_OtherTeamName(team: c_int) -> *const c_char;
        fn jka_TeamColorString(team: c_int) -> *const c_char;
        fn jka_SortClients(a: *const c_void, b: *const c_void) -> c_int;
    }

    #[test]
    fn sort_clients_matches_c() {
        let cases: [(c_int, c_int); 7] =
            [(0, 0), (1, 0), (0, 1), (5, 3), (3, 5), (-2, 4), (-1, -7)];
        for (a, b) in cases {
            let rust = SortClients(
                &a as *const c_int as *const c_void,
                &b as *const c_int as *const c_void,
            );
            let c = unsafe {
                jka_SortClients(
                    &a as *const c_int as *const c_void,
                    &b as *const c_int as *const c_void,
                )
            };
            assert_eq!(rust, c, "SortClients({a}, {b})");
        }
    }

    // Cover the named teams plus a couple of out-of-range / default values.
    const CASES: [c_int; 6] = [TEAM_FREE, TEAM_RED, TEAM_BLUE, TEAM_SPECTATOR, -1, 99];

    #[test]
    fn other_team_matches_c() {
        for &t in &CASES {
            assert_eq!(OtherTeam(t), unsafe { jka_OtherTeam(t) }, "OtherTeam({t})");
        }
    }

    #[test]
    fn team_name_matches_c() {
        for &t in &CASES {
            let rust = unsafe { CStr::from_ptr(TeamName(t)) };
            let c = unsafe { CStr::from_ptr(jka_TeamName(t)) };
            assert_eq!(rust, c, "TeamName({t})");
        }
    }

    #[test]
    fn other_team_name_matches_c() {
        for &t in &CASES {
            let rust = unsafe { CStr::from_ptr(OtherTeamName(t)) };
            let c = unsafe { CStr::from_ptr(jka_OtherTeamName(t)) };
            assert_eq!(rust, c, "OtherTeamName({t})");
        }
    }

    #[test]
    fn team_color_string_matches_c() {
        for &t in &CASES {
            let rust = unsafe { CStr::from_ptr(TeamColorString(t)) };
            let c = unsafe { CStr::from_ptr(jka_TeamColorString(t)) };
            assert_eq!(rust, c, "TeamColorString({t})");
        }
    }
}
