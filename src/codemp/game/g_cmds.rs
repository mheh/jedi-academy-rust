//! Ported `codemp/game/g_cmds.c` — client console-command handlers and the
//! team/spectator-management helpers the connect/spawn machinery sits on.
//!
//! This is the bottom-up entry point into the client/cmds/session infra cluster:
//! [`BroadcastTeamChange`] is the first leaf, opening the file for `SetTeam` /
//! `ClientUserinfoChanged` and the scoring-chain unblockers above them.

#![allow(non_upper_case_globals)] // C global names (`g_preventTeamBegin`, ...) kept verbatim

use core::ffi::{c_char, c_int, c_uint, CStr};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    /// libc `int tolower(int c)`. `SanitizeString` calls it directly (not the
    /// `Q3_VM` bg_lib shim), so we bind the same C library function the oracle
    /// links against for bit-exact parity.
    fn tolower(c: c_int) -> c_int;
    /// libc `int atoi(const char *)`. `ClientNumberFromString` parses a numeric
    /// slot string with it, matching the C verbatim.
    fn atoi(s: *const c_char) -> c_int;
    /// libc `int strcmp(const char *, const char *)`. `ClientNumberFromString`
    /// compares two sanitized NUL-terminated names with it.
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    /// libc `char *strcpy(char *dest, const char *src)`. `G_SetSaber` copies the
    /// chosen saber-type literals/names into the session buffers with it, matching
    /// the C `strcpy` calls verbatim (sibling bg files bind it the same way).
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    /// libc `double atof(const char *)`. `Cmd_SetViewpos_f` parses the x/y/z/yaw
    /// arguments with it, matching the C verbatim.
    fn atof(s: *const c_char) -> f64;
    /// libc `char *strchr(const char *, int)`. `Cmd_CallTeamVote_f` rejects vote
    /// strings containing `;` with it, matching the C verbatim.
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
}

/// `MAX_STRING_TOKENS` (`q_shared.h`) — max tokens from `Cmd_TokenizeString`; same
/// value as [`MAX_STRING_CHARS`] (1024). `Cmd_CallTeamVote_f` declares its `arg1`/
/// `arg2` token buffers with it.
const MAX_STRING_TOKENS: usize = MAX_STRING_CHARS;

/// `MAX_SAY_TEXT` (`q_shared.h:406`) — max length of a single chat line; `G_Say`
/// declares its `text` buffer with it ("don't let text be too long for malicious
/// reasons").
const MAX_SAY_TEXT: usize = 150;

/// `EC` (`match.h:12`) — the chat "extended-color"/escape lead byte `"\x19"` used to
/// bracket team/tell names in `G_Say`'s `Com_sprintf` format strings.
const EC: &str = "\x19";

/// `qboolean g_dontPenalizeTeam` / `qboolean g_preventTeamBegin` are both defined in
/// `g_cmds.c` (lines 664-665). `g_dontPenalizeTeam` lives with its consumer in
/// `g_combat.rs`; `g_preventTeamBegin` is referenced only by `SetTeam` here, so it
/// stays in this module as a module-static (`Cmd_EngageDuel_f` toggles it in the C —
/// not yet ported, but the static must already exist for `SetTeam`).
static mut g_preventTeamBegin: qboolean = QFALSE;

// `SetTeamQuick` (forced/siege quick team change) and `player_die` are the real
// ports living in g_saga.rs / g_combat.rs; re-exported / imported here for `SetTeam`'s
// GT_SIEGE and kill-on-team-change paths. `SetTeamQuick` stays reachable as
// `g_cmds::SetTeamQuick` (g_client.rs imports it via this module).
use crate::codemp::game::g_combat::player_die;
pub(crate) use crate::codemp::game::g_saga::SetTeamQuick;

use crate::codemp::cgame::animtable::animTable;
use crate::codemp::game::anims::{BOTH_KYLE_GRAB, MAX_ANIMATIONS};
use crate::codemp::game::bg_misc::vectoyaw;
use crate::codemp::game::bg_misc::{
    bg_customSiegeSoundNames, bg_itemlist, BG_FindItem, BG_IsItemSelectable,
};
use crate::codemp::game::bg_pmove::BG_KnockDownable;
use crate::codemp::game::bg_public::{
    gitem_t, CS_TEAMVOTE_NO, CS_TEAMVOTE_STRING, CS_TEAMVOTE_TIME, CS_TEAMVOTE_YES, CS_VOTE_NO,
    CS_VOTE_STRING, CS_VOTE_TIME, CS_VOTE_YES, DUELTEAM_DOUBLE, DUELTEAM_FREE, DUELTEAM_LONE,
    EF_SEEKERDRONE, EV_ITEMUSEFAIL, EV_PLAYER_TELEPORT_OUT, EV_PRIVATE_DUEL, EV_VOICECMD_SOUND,
    GT_DUEL, GT_FFA, GT_MAX_GAME_TYPE, GT_POWERDUEL, GT_SIEGE, GT_SINGLE_PLAYER, GT_TEAM,
    HANDEXTEND_DUELCHALLENGE, HANDEXTEND_KNOCKDOWN, HANDEXTEND_NONE, HANDEXTEND_PRETHROW,
    HANDEXTEND_PRETHROWN, HI_MEDPAC, HI_MEDPAC_BIG, HI_NUM_HOLDABLE, HI_SEEKER, HI_SENTRY_GUN,
    HI_SHIELD, LS_MOVE_MAX, MASK_PLAYERSOLID, MASK_SHOT, MASK_SOLID, MAX_CLIENT_SCORE_SEND,
    MAX_CUSTOM_SIEGE_SOUNDS, MOD_SUICIDE, MOD_TEAM_CHANGE, PERS_ASSIST_COUNT, PERS_CAPTURES,
    PERS_DEFEND_COUNT, PERS_EXCELLENT_COUNT, PERS_GAUNTLET_FRAG_COUNT, PERS_IMPRESSIVE_COUNT,
    PERS_KILLED, PERS_RANK, PERS_SCORE, PERS_TEAM, PMF_FOLLOW, PMF_USE_ITEM_HELD, SETANIM_BOTH,
    SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, STAT_ARMOR, STAT_HEALTH, STAT_HOLDABLE_ITEM,
    STAT_HOLDABLE_ITEMS, STAT_MAX_HEALTH, STAT_WEAPONS, TEAM_BLUE, TEAM_FREE, TEAM_NUM_TEAMS,
    TEAM_RED, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_saber::saberMoveData;
use crate::codemp::game::bg_saberLoad::{
    WP_SaberStyleValidForSaber, WP_SetSaber, WP_UseFirstValidSaberStyle,
};
use crate::codemp::game::bg_saga::{
    bgSiegeClasses, BG_SiegeCheckClassLegality, BG_SiegeFindThemeForTeam,
};
use crate::codemp::game::bg_saga_h::{
    siegeClass_t, siegeTeam_t, MAX_SIEGE_CLASSES, SIEGETEAM_TEAM1, SIEGETEAM_TEAM2,
};
use crate::codemp::game::bg_vehicles_h::SHIPSURF_FRONT;
use crate::codemp::game::bg_weapons_h::{LAST_USEABLE_WEAPON, WP_MELEE, WP_NONE, WP_SABER};
use crate::codemp::game::g_bot::{G_DoesMapSupportGametype, G_GetArenaInfoByMap};
use crate::codemp::game::g_client::ClientUserinfoChanged;
use crate::codemp::game::g_client::SetClientViewAngle;
use crate::codemp::game::g_client::{
    ClientBegin, MaintainBodyQueue, PickTeam, TeamCount, TeamLeader,
};
use crate::codemp::game::g_combat::g_dontPenalizeTeam;
use crate::codemp::game::g_combat::{DismembermentByNum, G_Damage};
use crate::codemp::game::g_items::{FinishSpawningItem, G_SpawnItem, Touch_Item};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_local::{
    gclient_t, spectatorState_t, CON_CONNECTED, CON_CONNECTING, CON_DISCONNECTED,
    DAMAGE_NO_PROTECTION, FL_GODMODE, FL_NOTARGET, MAX_NETNAME, MAX_VOTE_COUNT, PSG_TEAMVOTED,
    PSG_VOTED, SPECTATOR_FOLLOW, SPECTATOR_FREE, SPECTATOR_NOT, SPECTATOR_SCOREBOARD, TEAM_BEGIN,
};
use crate::codemp::game::g_main::{
    d_saberStanceDebug, g_allowDuelSuicide, g_allowVote, g_autoBanKillSpammers,
    g_autoBanTKSpammers, g_autoKickKillSpammers, g_autoKickTKSpammers, g_cheats, g_dedicated,
    g_entities, g_gametype, g_maxGameClients, g_privateDuel, g_teamForceBalance, g_trueJedi, level,
    BeginIntermission, CheckTeamLeader, Com_Printf, G_Error, G_GetStringEdString, G_LogPrintf,
    G_PowerDuelCount, G_Printf,
};
use crate::codemp::game::g_misc::{gEscaping, TeleportPlayer};
use crate::codemp::game::g_public_h::{SVF_BOT, SVF_BROADCAST};
use crate::codemp::game::g_saga::SiegeClearSwitchData;
use crate::codemp::game::g_svcmds::AddIP;
use crate::codemp::game::g_team::{OnSameTeam, TeamName, Team_GetLocationMsg};
use crate::codemp::game::g_utils::{
    vtos, G_AddEvent, G_FreeEntity, G_SetAnim, G_Sound, G_SoundIndex, G_Spawn, G_TempEntity,
};
use crate::codemp::game::g_utils::{G_EntitySound, G_Find};
use crate::codemp::game::npc_spawn::Cmd_NPC_f;
use crate::codemp::game::q_math::{
    vec3_origin, AngleVectors, VectorClear, VectorCopy, VectorMA, VectorNormalize, VectorSet,
    VectorSubtract,
};
use crate::codemp::game::q_shared::{
    va, Com_sprintf, Info_SetValueForKey, Info_ValueForKey, Q_CleanStr, Q_stricmp, Q_strncpyz, Sz,
};
use crate::codemp::game::q_shared_h::{
    playerState_t, trace_t, vec3_t, BLOCKED_BOUNCE_MOVE, CHAN_AUTO, CHAN_VOICE, COLOR_CYAN,
    COLOR_GREEN, COLOR_MAGENTA, COLOR_WHITE, ENTITYNUM_NONE, FORCE_LEVEL_1, FP_SABER_OFFENSE,
    MAX_CLIENTS, MAX_GENTITIES, MAX_INFO_STRING, MAX_NAME_LENGTH, MAX_STRING_CHARS,
    MAX_TOKEN_CHARS, MAX_WEAPONS, PITCH, Q_COLOR_ESCAPE, ROLL, SAY_ALL, SAY_TEAM, SAY_TELL,
    SEEKER_ALREADYDEPLOYED, SENTRY_ALREADYPLACED, SENTRY_NOROOM, SHIELD_NOROOM, SS_DUAL, SS_FAST,
    SS_NONE, SS_NUM_SABER_STYLES, YAW,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_SOLID;
use crate::codemp::game::w_force::WP_InitForcePowers;
use crate::codemp::game::w_saber::saberKnockDown;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `void BroadcastTeamChange( gclient_t *client, int oldTeam )` (g_cmds.c:670).
/// Announce a client's team change to everyone (a centre-print) and log it.
///
/// First flags `forceDoInit` so the client's force powers get re-set on every team
/// change. Returns early in Siege (no announcements there). Otherwise it
/// centre-prints the appropriate "JOINEDTHE…TEAM"/"…SPECTATORS"/"…BATTLE" stringed
/// reference (the `GT_DUEL`/`GT_POWERDUEL` "vs." path is commented out in the C —
/// "Just doing a vs. once it counts two players up"). Finally logs a `setteam:`
/// line with the old and new team names.
///
/// No oracle: drives `trap_SendServerCommand` plus the `level`/`g_gametype`
/// globals (the connect/team-infra precedent). Faithful 1:1 with original JKA.
///
/// # Safety
/// `client` must point to a valid `gclient_t` (a `level.clients[]` entry).
pub unsafe fn BroadcastTeamChange(client: *mut gclient_t, old_team: c_int) {
    // every time we change teams make sure our force powers are set right
    (*client).ps.fd.forceDoInit = 1;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        // don't announce these things in siege
        return;
    }

    let netname = Sz((*client).pers.netname.as_ptr());
    let session_team = (*client).sess.sessionTeam;

    if session_team == TEAM_RED {
        trap::SendServerCommand(
            -1,
            &format!(
                "cp \"{}^7 {}\n\"",
                netname,
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"JOINEDTHEREDTEAM".as_ptr() as *mut c_char,
                )),
            ),
        );
    } else if session_team == TEAM_BLUE {
        trap::SendServerCommand(
            -1,
            &format!(
                "cp \"{}^7 {}\n\"",
                netname,
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"JOINEDTHEBLUETEAM".as_ptr() as *mut c_char,
                )),
            ),
        );
    } else if session_team == TEAM_SPECTATOR && old_team != TEAM_SPECTATOR {
        trap::SendServerCommand(
            -1,
            &format!(
                "cp \"{}^7 {}\n\"",
                netname,
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"JOINEDTHESPECTATORS".as_ptr() as *mut c_char,
                )),
            ),
        );
    } else if session_team == TEAM_FREE {
        // The GT_DUEL/GT_POWERDUEL "vs." centre-print is commented out in the C
        // ("NOTE: Just doing a vs. once it counts two players up"); only the
        // non-duel "JOINEDTHEBATTLE" branch is live.
        if (*addr_of!(g_gametype)).integer != GT_DUEL
            && (*addr_of!(g_gametype)).integer != GT_POWERDUEL
        {
            trap::SendServerCommand(
                -1,
                &format!(
                    "cp \"{}^7 {}\n\"",
                    netname,
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"JOINEDTHEBATTLE".as_ptr() as *mut c_char,
                    )),
                ),
            );
        }
    }

    // G_LogPrintf("setteam:  %i %s %s\n", client - &level.clients[0],
    //             TeamName(oldTeam), TeamName(client->sess.sessionTeam));
    let client_num = client.offset_from((*addr_of!(level)).clients) as c_int;
    G_LogPrintf(&format!(
        "setteam:  {} {} {}\n",
        client_num,
        Sz(TeamName(old_team)),
        Sz(TeamName(session_team)),
    ));
}

/// `void Cmd_TeamTask_f( gentity_t *ent )` (g_cmds.c:506). Sets the client's
/// `teamtask` userinfo key from command arg 1 and republishes via
/// [`ClientUserinfoChanged`].
///
/// No oracle: the body is pure trap/userinfo plumbing (`trap_GetUserinfo` /
/// `trap_SetUserinfo` / `ClientUserinfoChanged`), driven by `level.clients`. The
/// userinfo round-trip mirrors `ClientUserinfoChanged`'s buffer idiom: `trap::Argv`
/// and `trap::GetUserinfo` return owned `String`s, so the `arg`/`userinfo` C char
/// buffers are filled byte-for-byte; `atoi` parses through a NUL-terminated
/// `CString` to match the C bit-for-bit; the mutated `userinfo` is handed back to
/// `trap::SetUserinfo` (which takes `&str`) via `CStr`.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level` must be initialized.
pub unsafe fn Cmd_TeamTask_f(ent: *mut gentity_t) {
    let mut userinfo = [0 as c_char; MAX_INFO_STRING];
    let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
    let client = (*ent).client.offset_from((*addr_of!(level)).clients) as c_int;

    if trap::Argc() != 2 {
        return;
    }

    // trap_Argv( 1, arg, sizeof( arg ) );
    {
        let s = trap::Argv(1);
        let bytes = s.as_bytes();
        let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
        for k in 0..n {
            arg[k] = bytes[k] as c_char;
        }
        arg[n] = 0;
    }
    let task = atoi(arg.as_ptr());

    // trap_GetUserinfo(client, userinfo, sizeof(userinfo));
    {
        let info = trap::GetUserinfo(client);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_STRING - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }

    Info_SetValueForKey(
        userinfo.as_mut_ptr(),
        c"teamtask".as_ptr(),
        va(format_args!("{}", task)),
    );
    trap::SetUserinfo(client, &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy());
    ClientUserinfoChanged(client);
}

/// `void Cmd_Team_f( gentity_t *ent )` (g_cmds.c:1058). Report the client's current
/// team when called with no argument, otherwise switch teams via [`SetTeam`].
///
/// With no arg, prints the localized "you are on team X" line for the session team.
/// Otherwise: enforces the `switchTeamTime` debounce (prints `NOSWITCH` and returns
/// when a recent switch is still on cooldown), bails while a level is escaping
/// ([`gEscaping`]), refuses team changes in `GT_DUEL` (a tournament game where the
/// client is `TEAM_FREE`) and `GT_POWERDUEL` (handled by automated stuff), then reads
/// arg 1, hands off to [`SetTeam`], and arms `switchTeamTime = level.time + 5000`.
///
/// No oracle: drives `trap_SendServerCommand` plus the `g_gametype`/`gEscaping`
/// globals and [`SetTeam`] (itself a no-oracle monolith). Faithful 1:1 with original
/// JKA. [`trap::Argv`]'s owned `String` is copied byte-for-byte into the `s[]` token.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `g_entities` must be initialized.
pub unsafe fn Cmd_Team_f(ent: *mut gentity_t) {
    let old_team: c_int;
    let mut s = [0 as c_char; MAX_TOKEN_CHARS];

    if trap::Argc() != 2 {
        old_team = (*(*ent).client).sess.sessionTeam;
        let ent_num =
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;
        match old_team {
            TEAM_BLUE => {
                trap::SendServerCommand(
                    ent_num,
                    &format!(
                        "print \"{}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr() as *mut c_char,
                            c"PRINTBLUETEAM".as_ptr() as *mut c_char,
                        )),
                    ),
                );
            }
            TEAM_RED => {
                trap::SendServerCommand(
                    ent_num,
                    &format!(
                        "print \"{}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr() as *mut c_char,
                            c"PRINTREDTEAM".as_ptr() as *mut c_char,
                        )),
                    ),
                );
            }
            TEAM_FREE => {
                trap::SendServerCommand(
                    ent_num,
                    &format!(
                        "print \"{}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr() as *mut c_char,
                            c"PRINTFREETEAM".as_ptr() as *mut c_char,
                        )),
                    ),
                );
            }
            TEAM_SPECTATOR => {
                trap::SendServerCommand(
                    ent_num,
                    &format!(
                        "print \"{}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr() as *mut c_char,
                            c"PRINTSPECTEAM".as_ptr() as *mut c_char,
                        )),
                    ),
                );
            }
            _ => {}
        }
        return;
    }

    if (*(*ent).client).switchTeamTime > (*addr_of!(level)).time {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOSWITCH".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    if gEscaping != QFALSE {
        return;
    }

    // if they are playing a tournement game, count as a loss
    if (*addr_of!(g_gametype)).integer == GT_DUEL && (*(*ent).client).sess.sessionTeam == TEAM_FREE
    {
        //in a tournament game
        //disallow changing teams
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            "print \"Cannot switch teams in Duel\n\"",
        );
        return;
        //FIXME: why should this be a loss???
        //ent->client->sess.losses++;
    }

    if (*addr_of!(g_gametype)).integer == GT_POWERDUEL {
        //don't let clients change teams manually at all in powerduel, it will be taken care of through automated stuff
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            "print \"Cannot switch teams in Power Duel\n\"",
        );
        return;
    }

    // trap_Argv( 1, s, sizeof( s ) );
    {
        let arg = trap::Argv(1);
        let bytes = arg.as_bytes();
        let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
        for k in 0..n {
            s[k] = bytes[k] as c_char;
        }
        s[n] = 0;
    }

    SetTeam(ent, s.as_mut_ptr());

    (*(*ent).client).switchTeamTime = (*addr_of!(level)).time + 5000;
}

/*
=================
Cmd_Kill_f
=================
*/
/// `void Cmd_Kill_f( gentity_t *ent )` (g_cmds.c:531). Client self-kill ("suicide")
/// command: refused for spectators and already-dead clients, and in an active
/// (`GT_DUEL`/`GT_POWERDUEL`) non-warmup match it is gated behind the
/// `g_allowDuelSuicide` cvar (printing the localized `ATTEMPTDUELKILL` refusal when
/// off). Otherwise it clears `FL_GODMODE`, drives health to `-999`, and runs
/// [`player_die`] with `MOD_SUICIDE`.
///
/// No oracle: an entity-state/command side-effecting fn — it drives the `g_entities`/
/// `g_gametype`/`level`/`g_allowDuelSuicide` globals, `trap_SendServerCommand`, and the
/// real [`player_die`] port (g_combat). Mirrors the C control flow
/// exactly.
///
/// # Safety
/// `ent`/`ent->client` must be valid; the game globals must be initialized.
pub unsafe fn Cmd_Kill_f(ent: *mut gentity_t) {
    // `EXEC_INSERT` (q_shared.h) — the `EXEC_*` enum's second value, so `1`.
    const EXEC_INSERT: c_int = 1;

    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        return;
    }
    if (*ent).health <= 0 {
        return;
    }

    if ((*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL)
        && (*addr_of!(level)).numPlayingClients > 1
        && (*addr_of!(level)).warmupTime == 0
    {
        if (*addr_of!(g_allowDuelSuicide)).integer == 0 {
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"ATTEMPTDUELKILL".as_ptr() as *mut c_char,
                    )),
                ),
            );
            return;
        }
    }

    if (*addr_of!(g_autoKickKillSpammers)).integer > 0
        || (*addr_of!(g_autoBanKillSpammers)).integer > 0
    {
        let client = (*ent).client;
        (*client).sess.killCount += 1;
        if (*addr_of!(g_autoBanKillSpammers)).integer > 0
            && (*client).sess.killCount >= (*addr_of!(g_autoBanKillSpammers)).integer
        {
            // C tests `if ( ent->client->sess.IPstring )`; `IPstring` is a
            // `char[32]` array, so the test is always true — ban their IP.
            AddIP((*client).sess.IPstring.as_ptr());

            trap::SendServerCommand(
                -1,
                &format!(
                    "print \"{} {}\n\"",
                    Sz((*client).pers.netname.as_ptr()),
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"SUICIDEBAN".as_ptr() as *mut c_char,
                    )),
                ),
            );
            //Com_sprintf ( level.voteString, sizeof(level.voteString ), "clientkick %d", ent->s.number );
            //Com_sprintf ( level.voteDisplayString, sizeof(level.voteDisplayString), "kick %s", ent->client->pers.netname );
            //trap_SendConsoleCommand( EXEC_INSERT, va( "banClient %d\n", ent->s.number ) );
            trap::SendConsoleCommand(EXEC_INSERT, &format!("clientkick {}\n", (*ent).s.number));
            return;
        }
        if (*addr_of!(g_autoKickKillSpammers)).integer > 0
            && (*client).sess.killCount >= (*addr_of!(g_autoKickKillSpammers)).integer
        {
            trap::SendServerCommand(
                -1,
                &format!(
                    "print \"{} {}\n\"",
                    Sz((*client).pers.netname.as_ptr()),
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"SUICIDEKICK".as_ptr() as *mut c_char,
                    )),
                ),
            );
            //Com_sprintf ( level.voteString, sizeof(level.voteString ), "clientkick %d", ent->s.number );
            //Com_sprintf ( level.voteDisplayString, sizeof(level.voteDisplayString), "kick %s", ent->client->pers.netname );
            trap::SendConsoleCommand(EXEC_INSERT, &format!("clientkick {}\n", (*ent).s.number));
            return;
        }
        //okay, not gone (yet), but warn them...
        if (*addr_of!(g_autoBanKillSpammers)).integer > 0
            && ((*addr_of!(g_autoKickKillSpammers)).integer <= 0
                || (*addr_of!(g_autoBanKillSpammers)).integer
                    < (*addr_of!(g_autoKickKillSpammers)).integer)
        {
            //warn about ban
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"WARNINGSUICIDEBAN".as_ptr() as *mut c_char,
                    )),
                ),
            );
        } else if (*addr_of!(g_autoKickKillSpammers)).integer > 0 {
            //warn about kick
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"WARNINGSUICIDEKICK".as_ptr() as *mut c_char,
                    )),
                ),
            );
        }
    }

    (*ent).flags &= !FL_GODMODE;
    (*(*ent).client).ps.stats[STAT_HEALTH as usize] = -999;
    (*ent).health = -999;
    player_die(ent, ent, ent, 100000, MOD_SUICIDE);
}

/// `void Cmd_DuelTeam_f( gentity_t *ent )` (g_cmds.c:1042). Power-duel-only: report
/// or change the client's `sess.duelTeam` (`free`/`single`/`double`); a live change
/// suicides the client (so they respawn under the new team), resets wins/losses, and
/// republishes via [`ClientUserinfoChanged`].
///
/// No oracle: drives `level`/`g_entities`/`g_gametype` globals, `trap_SendServerCommand`,
/// `G_Damage` (itself a no-oracle monolith), and `ClientUserinfoChanged`. The
/// commented-out spectator-gate block is kept verbatim as a comment (matching the C);
/// the `switchDuelTeamTime` debounce (prints `NOSWITCH`) and the
/// `switchDuelTeamTime = level.time + 5000` arm are live. [`trap::Argv`]'s owned `String` is parsed
/// with [`Q_stricmp`] through a NUL-terminated buffer to match the C `s[]` token.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_entities` must be initialized.
pub unsafe fn Cmd_DuelTeam_f(ent: *mut gentity_t) {
    let old_team: c_int;
    let mut s = [0 as c_char; MAX_TOKEN_CHARS];

    if (*addr_of!(g_gametype)).integer != GT_POWERDUEL {
        //don't bother doing anything if this is not power duel
        return;
    }

    /*
    if (ent->client->sess.sessionTeam != TEAM_SPECTATOR)
    {
        trap_SendServerCommand( ent-g_entities, va("print \"You cannot change your duel team unless you are a spectator.\n\""));
        return;
    }
    */

    if trap::Argc() != 2 {
        //No arg so tell what team we're currently on.
        let ent_num =
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;
        match (*(*ent).client).sess.duelTeam {
            DUELTEAM_FREE => {
                trap::SendServerCommand(ent_num, "print \"None\n\"");
            }
            DUELTEAM_LONE => {
                trap::SendServerCommand(ent_num, "print \"Single\n\"");
            }
            DUELTEAM_DOUBLE => {
                trap::SendServerCommand(ent_num, "print \"Double\n\"");
            }
            _ => {}
        }
        return;
    }

    if (*(*ent).client).switchDuelTeamTime > (*addr_of!(level)).time {
        //debounce for changing
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOSWITCH".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    // trap_Argv( 1, s, sizeof( s ) );
    {
        let arg = trap::Argv(1);
        let bytes = arg.as_bytes();
        let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
        for k in 0..n {
            s[k] = bytes[k] as c_char;
        }
        s[n] = 0;
    }

    old_team = (*(*ent).client).sess.duelTeam;

    if Q_stricmp(s.as_ptr(), c"free".as_ptr()) == 0 {
        (*(*ent).client).sess.duelTeam = DUELTEAM_FREE;
    } else if Q_stricmp(s.as_ptr(), c"single".as_ptr()) == 0 {
        (*(*ent).client).sess.duelTeam = DUELTEAM_LONE;
    } else if Q_stricmp(s.as_ptr(), c"double".as_ptr()) == 0 {
        (*(*ent).client).sess.duelTeam = DUELTEAM_DOUBLE;
    } else {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"'{}' not a valid duel team.\n\"",
                CStr::from_ptr(s.as_ptr()).to_string_lossy()
            ),
        );
    }

    if old_team == (*(*ent).client).sess.duelTeam {
        //didn't actually change, so don't care.
        return;
    }

    if (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR {
        //ok..die
        let cur_team = (*(*ent).client).sess.duelTeam;
        (*(*ent).client).sess.duelTeam = old_team;
        G_Damage(
            ent,
            ent,
            ent,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*(*ent).client).ps.origin),
            99999,
            DAMAGE_NO_PROTECTION,
            MOD_SUICIDE,
        );
        (*(*ent).client).sess.duelTeam = cur_team;
    }
    //reset wins and losses
    (*(*ent).client).sess.wins = 0;
    (*(*ent).client).sess.losses = 0;

    //get and distribute relevent paramters
    ClientUserinfoChanged((*ent).s.number);

    (*(*ent).client).switchDuelTeamTime = (*addr_of!(level)).time + 5000;
}

/// `void Cmd_Vote_f( gentity_t *ent )` (g_cmds.c:2111). Casts the calling
/// client's vote on an active map vote: gated by an in-progress vote
/// (`level.voteTime`), the per-client already-voted flag (`PSG_VOTED` in
/// `mGameFlags`), and (outside duel/power-duel) a non-spectator check; then sets
/// `PSG_VOTED`, reads arg 1, and bumps `level.voteYes`/`level.voteNo` with the
/// matching `CS_VOTE_*` configstring.
///
/// No oracle: pure trap/`level`-global plumbing (`trap_SendServerCommand` /
/// `trap_Argv` / `trap_SetConfigstring`) plus `G_GetStringEdString` — the same
/// no-oracle class as the surrounding cmd handlers. The original yes/no test
/// `msg[0] == 'y' || msg[1] == 'Y' || msg[1] == '1'` is kept verbatim (note it
/// checks `msg[0]` only for `'y'` and `msg[1]` for `'Y'`/`'1'` — a faithful quirk
/// of the C). [`trap::Argv`]'s owned `String` is copied byte-for-byte into the
/// `msg[64]` buffer to reproduce the C `trap_Argv(1, msg, sizeof(msg))` fill.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_entities` must be initialized.
pub unsafe fn Cmd_Vote_f(ent: *mut gentity_t) {
    let mut msg = [0 as c_char; 64];

    let ent_num = ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;

    if (*addr_of!(level)).voteTime == 0 {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOVOTEINPROG".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if (*(*ent).client).mGameFlags & (PSG_VOTED as c_uint) != 0 {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"VOTEALREADY".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if (*addr_of!(g_gametype)).integer != GT_DUEL && (*addr_of!(g_gametype)).integer != GT_POWERDUEL
    {
        if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
            trap::SendServerCommand(
                ent_num,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"NOVOTEASSPEC".as_ptr() as *mut c_char,
                    )),
                ),
            );
            return;
        }
    }

    trap::SendServerCommand(
        ent_num,
        &format!(
            "print \"{}\n\"",
            Sz(G_GetStringEdString(
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"PLVOTECAST".as_ptr() as *mut c_char,
            )),
        ),
    );

    (*(*ent).client).mGameFlags |= PSG_VOTED as c_uint;

    // trap_Argv( 1, msg, sizeof( msg ) );
    {
        let arg = trap::Argv(1);
        let bytes = arg.as_bytes();
        let n = bytes.len().min(msg.len() - 1);
        for k in 0..n {
            msg[k] = bytes[k] as c_char;
        }
        msg[n] = 0;
    }

    if msg[0] == b'y' as c_char || msg[1] == b'Y' as c_char || msg[1] == b'1' as c_char {
        (*addr_of_mut!(level)).voteYes += 1;
        trap::SetConfigstring(CS_VOTE_YES, &format!("{}", (*addr_of!(level)).voteYes));
    } else {
        (*addr_of_mut!(level)).voteNo += 1;
        trap::SetConfigstring(CS_VOTE_NO, &format!("{}", (*addr_of!(level)).voteNo));
    }

    // a majority will be determined in CheckVote, which will also account
    // for players entering or leaving
}

/// `void Cmd_TeamVote_f( gentity_t *ent )` (g_cmds.c:2281). Casts the calling
/// client's vote on an active team vote. The client's `sess.sessionTeam` selects
/// the per-team config-string offset (`TEAM_RED`->0, `TEAM_BLUE`->1; any other
/// team returns immediately). Gated by an in-progress team vote
/// (`level.teamVoteTime[cs_offset]`), the per-client already-team-voted flag
/// (`PSG_TEAMVOTED` in `mGameFlags`), and a non-spectator check, each emitting the
/// localized `G_GetStringEdString("MP_SVGAME", …)` print; then sets
/// `PSG_TEAMVOTED`, reads arg 1, and bumps `level.teamVoteYes`/`level.teamVoteNo`
/// at `cs_offset` with the matching `CS_TEAMVOTE_*` configstring.
///
/// No oracle: pure trap/`level`-global plumbing (`trap_SendServerCommand` /
/// `trap_Argv` / `trap_SetConfigstring`) plus `G_GetStringEdString` — the same
/// no-oracle class as the sibling [`Cmd_Vote_f`]. The original yes/no test
/// `msg[0] == 'y' || msg[1] == 'Y' || msg[1] == '1'` is kept verbatim (the same
/// faithful C quirk: `'y'` at index 0, `'Y'`/`'1'` at index 1). [`trap::Argv`]'s
/// owned `String` is copied byte-for-byte into the `msg[64]` buffer to reproduce
/// the C `trap_Argv(1, msg, sizeof(msg))` fill.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_entities` must be initialized.
pub unsafe fn Cmd_TeamVote_f(ent: *mut gentity_t) {
    let mut msg = [0 as c_char; 64];

    let ent_num = ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;

    let team = (*(*ent).client).sess.sessionTeam;
    let cs_offset: usize = if team == TEAM_RED {
        0
    } else if team == TEAM_BLUE {
        1
    } else {
        return;
    };

    if (*addr_of!(level)).teamVoteTime[cs_offset] == 0 {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOTEAMVOTEINPROG".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if (*(*ent).client).mGameFlags & (PSG_TEAMVOTED as c_uint) != 0 {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"TEAMVOTEALREADYCAST".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOVOTEASSPEC".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    trap::SendServerCommand(
        ent_num,
        &format!(
            "print \"{}\n\"",
            Sz(G_GetStringEdString(
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"PLTEAMVOTECAST".as_ptr() as *mut c_char,
            )),
        ),
    );

    (*(*ent).client).mGameFlags |= PSG_TEAMVOTED as c_uint;

    // trap_Argv( 1, msg, sizeof( msg ) );
    {
        let arg = trap::Argv(1);
        let bytes = arg.as_bytes();
        let n = bytes.len().min(msg.len() - 1);
        for k in 0..n {
            msg[k] = bytes[k] as c_char;
        }
        msg[n] = 0;
    }

    if msg[0] == b'y' as c_char || msg[1] == b'Y' as c_char || msg[1] == b'1' as c_char {
        (*addr_of_mut!(level)).teamVoteYes[cs_offset] += 1;
        trap::SetConfigstring(
            CS_TEAMVOTE_YES + cs_offset as c_int,
            &format!("{}", (*addr_of!(level)).teamVoteYes[cs_offset]),
        );
    } else {
        (*addr_of_mut!(level)).teamVoteNo[cs_offset] += 1;
        trap::SetConfigstring(
            CS_TEAMVOTE_NO + cs_offset as c_int,
            &format!("{}", (*addr_of!(level)).teamVoteNo[cs_offset]),
        );
    }

    // a majority will be determined in TeamCheckVote, which will also account
    // for players entering or leaving
}

/// `void Cmd_CallTeamVote_f( gentity_t *ent )` (g_cmds.c:2154). Seeds a per-team
/// vote on the calling client's team. `sess.sessionTeam` selects the per-team
/// config-string offset (`TEAM_RED`->0, `TEAM_BLUE`->1; any other team returns
/// immediately). Gated by `g_allowVote`, a not-already-running team vote
/// (`level.teamVoteTime[cs_offset]`), the per-client `teamVoteCount` limit
/// (`MAX_VOTE_COUNT`), and a non-spectator check, each emitting a localized
/// `G_GetStringEdString("MP_SVGAME", …)` print. The only valid team-vote command is
/// `leader <player>`: with no argument the caller becomes leader; a 1-3-digit numeric
/// is a client slot (range/`inuse`-checked); otherwise the (color-cleaned) name is
/// matched against connected same-team clients. The resolved slot is rendered into
/// `arg2`, the vote string is built, every connected same-team client is told a team
/// vote was called, the team-vote `level` fields are seeded (caller auto-votes yes),
/// `PSG_TEAMVOTED` is cleared for the team then set for the caller, and the four
/// `CS_TEAMVOTE_*` (+`cs_offset`) configstrings are published. The actual team change
/// happens later in `TeamCheckVote` when the vote passes — this function only
/// initiates, so it touches no `SetTeam`/`G_Say`-class callee.
///
/// No oracle: pure trap/`level`-global plumbing (`trap_SendServerCommand` /
/// `trap_Argv` / `trap_Argc` / `trap_SetConfigstring`) plus `G_GetStringEdString`,
/// the same no-oracle class as the sibling [`Cmd_TeamVote_f`]. The C `arg1`/`arg2`
/// `[MAX_STRING_TOKENS]` token buffers and the `strlen`/`strcat`-style append in the
/// `trap_Argv` loop are kept as raw `c_char` buffers operated on with the same C
/// helpers (`Q_stricmp`/`Q_strncpyz`/`Q_CleanStr`/`Com_sprintf`/`strchr`/`atoi`) for
/// byte-for-byte parity. [`trap::Argv`]'s owned `String` is copied byte-for-byte into
/// each buffer to reproduce the C `trap_Argv(i, buf, size)` fill.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_entities` must be initialized.
pub unsafe fn Cmd_CallTeamVote_f(ent: *mut gentity_t) {
    let mut arg1 = [0 as c_char; MAX_STRING_TOKENS];
    let mut arg2 = [0 as c_char; MAX_STRING_TOKENS];

    let ent_num = ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;

    let team = (*(*ent).client).sess.sessionTeam;
    let cs_offset: usize = if team == TEAM_RED {
        0
    } else if team == TEAM_BLUE {
        1
    } else {
        return;
    };

    if (*addr_of!(g_allowVote)).integer == 0 {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOVOTE".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    if (*addr_of!(level)).teamVoteTime[cs_offset] != 0 {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"TEAMVOTEALREADY".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if (*(*ent).client).pers.teamVoteCount >= MAX_VOTE_COUNT {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"MAXTEAMVOTES".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOSPECVOTE".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    // make sure it is a valid command to vote on
    // trap_Argv( 1, arg1, sizeof( arg1 ) );
    {
        let a = trap::Argv(1);
        let bytes = a.as_bytes();
        let n = bytes.len().min(arg1.len() - 1);
        for k in 0..n {
            arg1[k] = bytes[k] as c_char;
        }
        arg1[n] = 0;
    }
    arg2[0] = b'\0' as c_char;
    // `strlen` over the c_char buffer: index of the first NUL.
    let cstrlen =
        |buf: &[c_char]| -> usize { buf.iter().position(|&b| b == 0).unwrap_or(buf.len()) };
    let argc = trap::Argc();
    let mut i: c_int = 2;
    while i < argc {
        if i > 2 {
            // strcat(arg2, " ");
            let l = cstrlen(&arg2);
            if l + 1 < arg2.len() {
                arg2[l] = b' ' as c_char;
                arg2[l + 1] = 0;
            }
        }
        // trap_Argv( i, &arg2[strlen(arg2)], sizeof( arg2 ) - strlen(arg2) );
        let off = cstrlen(&arg2);
        let a = trap::Argv(i);
        let bytes = a.as_bytes();
        let avail = arg2.len() - off;
        let n = bytes.len().min(avail.saturating_sub(1));
        for k in 0..n {
            arg2[off + k] = bytes[k] as c_char;
        }
        arg2[off + n] = 0;
        i += 1;
    }

    if !strchr(arg1.as_ptr(), b';' as c_int).is_null()
        || !strchr(arg2.as_ptr(), b';' as c_int).is_null()
    {
        trap::SendServerCommand(ent_num, "print \"Invalid vote string.\n\"");
        return;
    }

    if Q_stricmp(arg1.as_ptr(), c"leader".as_ptr()) == 0 {
        let mut netname = [0 as c_char; MAX_NETNAME];
        let mut leader = [0 as c_char; MAX_NETNAME];

        if arg2[0] == 0 {
            i = (*(*ent).client).ps.clientNum;
        } else {
            // numeric values are just slot numbers
            let mut j: usize = 0;
            while j < 3 {
                if arg2[j] == 0 || arg2[j] < b'0' as c_char || arg2[j] > b'9' as c_char {
                    break;
                }
                j += 1;
            }
            if j >= 3 || arg2[j] == 0 {
                i = atoi(arg2.as_ptr());
                if i < 0 || i >= (*addr_of!(level)).maxclients {
                    trap::SendServerCommand(ent_num, &format!("print \"Bad client slot: {i}\n\""));
                    return;
                }

                if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize))
                    .inuse
                    == QFALSE
                {
                    trap::SendServerCommand(
                        ent_num,
                        &format!("print \"Client {i} is not active\n\""),
                    );
                    return;
                }
            } else {
                Q_strncpyz(leader.as_mut_ptr(), arg2.as_ptr(), leader.len() as c_int);
                Q_CleanStr(leader.as_mut_ptr());
                i = 0;
                while i < (*addr_of!(level)).maxclients {
                    let cl = (*addr_of!(level)).clients.add(i as usize);
                    if (*cl).pers.connected == CON_DISCONNECTED {
                        i += 1;
                        continue;
                    }
                    if (*cl).sess.sessionTeam != team {
                        i += 1;
                        continue;
                    }
                    Q_strncpyz(
                        netname.as_mut_ptr(),
                        (*cl).pers.netname.as_ptr(),
                        netname.len() as c_int,
                    );
                    Q_CleanStr(netname.as_mut_ptr());
                    if Q_stricmp(netname.as_ptr(), leader.as_ptr()) == 0 {
                        break;
                    }
                    i += 1;
                }
                if i >= (*addr_of!(level)).maxclients {
                    trap::SendServerCommand(
                        ent_num,
                        &format!(
                            "print \"{} is not a valid player on your team.\n\"",
                            Sz(arg2.as_ptr())
                        ),
                    );
                    return;
                }
            }
        }
        Com_sprintf(arg2.as_mut_ptr(), arg2.len() as c_int, format_args!("{i}"));
    } else {
        trap::SendServerCommand(ent_num, "print \"Invalid vote string.\n\"");
        trap::SendServerCommand(
            ent_num,
            "print \"Team vote commands are: leader <player>.\n\"",
        );
        return;
    }

    // Com_sprintf( level.teamVoteString[cs_offset], ..., "%s %s", arg1, arg2 );
    Com_sprintf(
        (*addr_of_mut!(level)).teamVoteString[cs_offset].as_mut_ptr(),
        (*addr_of!(level)).teamVoteString[cs_offset].len() as c_int,
        format_args!("{} {}", Sz(arg1.as_ptr()), Sz(arg2.as_ptr())),
    );

    i = 0;
    while i < (*addr_of!(level)).maxclients {
        let cl = (*addr_of!(level)).clients.add(i as usize);
        if (*cl).pers.connected == CON_DISCONNECTED {
            i += 1;
            continue;
        }
        if (*cl).sess.sessionTeam == team {
            trap::SendServerCommand(
                i,
                &format!(
                    "print \"{} called a team vote.\n\"",
                    Sz((*(*ent).client).pers.netname.as_ptr())
                ),
            );
        }
        i += 1;
    }

    // start the voting, the caller autoamtically votes yes
    (*addr_of_mut!(level)).teamVoteTime[cs_offset] = (*addr_of!(level)).time;
    (*addr_of_mut!(level)).teamVoteYes[cs_offset] = 1;
    (*addr_of_mut!(level)).teamVoteNo[cs_offset] = 0;

    i = 0;
    while i < (*addr_of!(level)).maxclients {
        let cl = (*addr_of!(level)).clients.add(i as usize);
        if (*cl).sess.sessionTeam == team {
            (*cl).mGameFlags &= !(PSG_TEAMVOTED as c_uint);
        }
        i += 1;
    }
    (*(*ent).client).mGameFlags |= PSG_TEAMVOTED as c_uint;

    trap::SetConfigstring(
        CS_TEAMVOTE_TIME + cs_offset as c_int,
        &format!("{}", (*addr_of!(level)).teamVoteTime[cs_offset]),
    );
    trap::SetConfigstring(
        CS_TEAMVOTE_STRING + cs_offset as c_int,
        &Sz((*addr_of!(level)).teamVoteString[cs_offset].as_ptr()).to_string(),
    );
    trap::SetConfigstring(
        CS_TEAMVOTE_YES + cs_offset as c_int,
        &format!("{}", (*addr_of!(level)).teamVoteYes[cs_offset]),
    );
    trap::SetConfigstring(
        CS_TEAMVOTE_NO + cs_offset as c_int,
        &format!("{}", (*addr_of!(level)).teamVoteNo[cs_offset]),
    );
}

/// `char *ConcatArgs( int start )` (g_cmds.c:127). Re-join the command tokens from
/// index `start` onward into a single space-separated string and return a pointer to
/// it.
///
/// The C accumulates into a `static char line[MAX_STRING_CHARS]` via
/// `trap_Argv`/`memcpy`, stopping before it would overflow (`len + tlen >=
/// MAX_STRING_CHARS - 1`), and returns that static buffer. We keep the exact same
/// static-buffer semantics: a module-static `LINE` is filled byte-for-byte and a raw
/// pointer into it is returned, so callers see the same "valid until the next
/// `ConcatArgs` call" lifetime as the C. [`trap::Argv`] returns an owned Rust
/// `String` (the engine fills the C out-buffer; the wrapper copies it out), so the C
/// `trap_Argv(i, arg, sizeof(arg))` fill becomes "take the returned string's bytes" —
/// behaviourally identical for the bounded copy that follows.
///
/// No oracle: drives `trap_Argc`/`trap_Argv` (the trap-boundary precedent). Faithful
/// 1:1 with original JKA.
///
/// # Safety
/// Returns a pointer into a mutable module static; the previous return value is
/// invalidated by the next call, exactly like the C `static` buffer. Single-threaded
/// game-module use only.
pub unsafe fn ConcatArgs(start: c_int) -> *mut c_char {
    static mut LINE: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
    // Raw element pointer into the static — avoids forming a `&mut` to the static.
    let line = core::ptr::addr_of_mut!(LINE) as *mut c_char;

    let mut len: usize = 0;
    let c = trap::Argc();
    let mut i = start;
    while i < c {
        let arg = trap::Argv(i);
        let bytes = arg.as_bytes();
        let tlen = bytes.len();
        if len + tlen >= MAX_STRING_CHARS - 1 {
            break;
        }
        // memcpy(line + len, arg, tlen)
        let mut k = 0;
        while k < tlen {
            *line.add(len + k) = bytes[k] as c_char;
            k += 1;
        }
        len += tlen;
        if i != c - 1 {
            *line.add(len) = b' ' as c_char;
            len += 1;
        }
        i += 1;
    }

    *line.add(len) = 0;

    line
}

/// `void SanitizeString( char *in, char *out )` (g_cmds.c:161). Remove case and
/// control characters: skip a two-byte color code (`^x`, signalled by the escape
/// byte `27`), drop any control char (`< 32`), and lowercase everything else.
/// Writes a NUL-terminated result to `out`.
///
/// Oracle-tested (`jka_SanitizeString`): a pure byte transform that calls libc
/// [`tolower`] just like the C. Faithful 1:1 with original JKA.
///
/// # Safety
/// `in_` must be a valid NUL-terminated C string; `out` must point to a buffer
/// large enough to hold the (never-longer) sanitized copy plus its NUL.
pub unsafe fn SanitizeString(mut in_: *const c_char, mut out: *mut c_char) {
    while *in_ != 0 {
        if *in_ == 27 {
            in_ = in_.offset(2); // skip color code
            continue;
        }
        if *in_ < 32 {
            in_ = in_.offset(1);
            continue;
        }
        // tolower((unsigned char)*in++)
        *out = tolower(*in_ as u8 as c_int) as c_char;
        out = out.offset(1);
        in_ = in_.offset(1);
    }

    *out = 0;
}

/// `int ClientNumberFromString( gentity_t *to, char *s )` (g_cmds.c:185). Resolve
/// a player reference — either a numeric slot index or a (sanitized) name — to a
/// client number, or `-1` if invalid. A leading digit means the whole string is an
/// [`atoi`]'d slot number, bounds-checked against `level.maxclients` and required to
/// be `CON_CONNECTED`; otherwise the input is [`SanitizeString`]'d and matched
/// (also sanitized) against every connected client's `pers.netname`. On every
/// failure path it prints a `print` server command back to `to`.
///
/// No oracle: drives `level.clients`/`level.maxclients` and `trap_SendServerCommand`
/// (the connect/team-infra precedent); the only computable inner pieces
/// ([`SanitizeString`]/[`atoi`]/[`strcmp`]) are libc/already-oracled. Faithful 1:1
/// with original JKA.
///
/// # Safety
/// `to` must point to a valid `gentity_t` in the `g_entities` array; `s` must be a
/// valid NUL-terminated C string. `level` must be initialized.
pub unsafe fn ClientNumberFromString(to: *mut gentity_t, s: *const c_char) -> c_int {
    let lvl = addr_of!(level);
    let mut s2 = [0 as c_char; MAX_STRING_CHARS];
    let mut n2 = [0 as c_char; MAX_STRING_CHARS];

    // numeric values are just slot numbers
    if *s.offset(0) >= b'0' as c_char && *s.offset(0) <= b'9' as c_char {
        let idnum = atoi(s);
        if idnum < 0 || idnum >= (*lvl).maxclients {
            trap::SendServerCommand(
                to.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!("print \"Bad client slot: {}\n\"", idnum),
            );
            return -1;
        }

        let cl = (*lvl).clients.offset(idnum as isize);
        if (*cl).pers.connected != CON_CONNECTED {
            trap::SendServerCommand(
                to.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!("print \"Client {} is not active\n\"", idnum),
            );
            return -1;
        }
        return idnum;
    }

    // check for a name match
    SanitizeString(s, s2.as_mut_ptr());
    let mut idnum: c_int = 0;
    let mut cl = (*lvl).clients;
    while idnum < (*lvl).maxclients {
        if (*cl).pers.connected != CON_CONNECTED {
            idnum += 1;
            cl = cl.offset(1);
            continue;
        }
        SanitizeString((*cl).pers.netname.as_ptr(), n2.as_mut_ptr());
        if strcmp(n2.as_ptr(), s2.as_ptr()) == 0 {
            return idnum;
        }
        idnum += 1;
        cl = cl.offset(1);
    }

    trap::SendServerCommand(
        to.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        &format!("print \"User {} is not on the server\n\"", Sz(s)),
    );
    -1
}

/// `gentity_t *G_GetDuelWinner(gclient_t *client)` (g_cmds.c:554). Find the "other"
/// duelist still standing: walk `level.clients[0..maxclients]` and return the entity
/// of the first connected, non-spectator client that is not `client`, or `NULL` if
/// none. Used to name the surviving opponent on a duel end.
///
/// No oracle: walks `level.clients` and indexes `g_entities` (the duel-walker
/// precedent in g_main). Faithful 1:1 with original JKA (the redundant `wCl` non-null
/// check on a `&level.clients[i]` address is kept verbatim).
///
/// # Safety
/// `level` must be initialized; `client` must be a valid `level.clients[]` pointer (or
/// null). Returns a pointer into the `g_entities` array.
pub unsafe fn G_GetDuelWinner(client: *mut gclient_t) -> *mut gentity_t {
    let lvl = addr_of!(level);

    let mut i: c_int = 0;
    while i < (*lvl).maxclients {
        let w_cl = (*lvl).clients.offset(i as isize);

        if !w_cl.is_null()
            && w_cl != client
            && (*w_cl).pers.connected == CON_CONNECTED
            && (*w_cl).sess.sessionTeam != TEAM_SPECTATOR
        {
            return (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .offset((*w_cl).ps.clientNum as isize);
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/// `qboolean G_PowerDuelCheckFail(gentity_t *ent)` (g_cmds.c:634). Decide whether a
/// client may *not* join the power-duel team it has selected: fails (`qtrue`) if the
/// client has no duel team assigned (`DUELTEAM_FREE`), or if its chosen side is
/// already full — one `DUELTEAM_LONE` or two `DUELTEAM_DOUBLE` already present (counted
/// by [`G_PowerDuelCount`] with `count_spec == qfalse`). Otherwise returns `qfalse`.
///
/// No oracle: control-flow over the `sess.duelTeam` field plus a [`G_PowerDuelCount`]
/// (already-ported) entity walk. Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; its `client` may be null (handled).
pub unsafe fn G_PowerDuelCheckFail(ent: *mut gentity_t) -> qboolean {
    let mut loners: c_int = 0;
    let mut doubles: c_int = 0;

    if (*ent).client.is_null() || (*(*ent).client).sess.duelTeam == DUELTEAM_FREE {
        return QTRUE;
    }

    G_PowerDuelCount(&mut loners, &mut doubles, QFALSE);

    if (*(*ent).client).sess.duelTeam == DUELTEAM_LONE && loners >= 1 {
        return QTRUE;
    }

    if (*(*ent).client).sess.duelTeam == DUELTEAM_DOUBLE && doubles >= 2 {
        return QTRUE;
    }

    QFALSE
}

/// `qboolean G_OtherPlayersDueling(void)` (g_cmds.c:2752). Return `qtrue` if any
/// in-use client entity currently has a private duel in progress
/// (`ps.duelInProgress`), else `qfalse`. Walks the first `MAX_CLIENTS` entity slots.
///
/// No oracle: walks the `g_entities` array (the entity-walker precedent). Faithful 1:1
/// with original JKA (the redundant `ent` non-null check on a `&g_entities[i]` address
/// is kept verbatim).
///
/// # Safety
/// `g_entities` must be a valid module static of at least `MAX_CLIENTS` entries (true
/// after `G_InitGame`).
pub unsafe fn G_OtherPlayersDueling() -> qboolean {
    let mut i: usize = 0;
    while i < MAX_CLIENTS {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);

        if !ent.is_null()
            && (*ent).inuse != QFALSE
            && !(*ent).client.is_null()
            && (*(*ent).client).ps.duelInProgress != QFALSE
        {
            return QTRUE;
        }
        i += 1;
    }

    QFALSE
}

/// `int G_ClientNumberFromName( const char* name )` (g_cmds.c:1818). Find the client
/// number whose [`SanitizeString`]'d `pers.netname` matches the [`SanitizeString`]'d
/// `name`, walking `level.clients[0..numConnectedClients]`; returns `-1` on no match.
///
/// No oracle: walks `level.clients` (the connect/team-infra precedent); the only
/// computable inner pieces ([`SanitizeString`]/[`strcmp`]) are already-oracled/libc.
/// Faithful 1:1 with original JKA.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string; `level` must be initialized.
pub unsafe fn G_ClientNumberFromName(name: *const c_char) -> c_int {
    let lvl = addr_of!(level);
    let mut s2 = [0 as c_char; MAX_STRING_CHARS];
    let mut n2 = [0 as c_char; MAX_STRING_CHARS];

    // check for a name match
    SanitizeString(name, s2.as_mut_ptr());
    let mut i: c_int = 0;
    let mut cl = (*lvl).clients;
    while i < (*lvl).numConnectedClients {
        SanitizeString((*cl).pers.netname.as_ptr(), n2.as_mut_ptr());
        if strcmp(n2.as_ptr(), s2.as_ptr()) == 0 {
            return i;
        }
        i += 1;
        cl = cl.offset(1);
    }

    -1
}

/// `void SanitizeString2( char *in, char *out )` (g_cmds.c:1846). Rich's revised
/// `SanitizeString` variant: copies `in` to `out` truncated at `MAX_NAME_LENGTH-1`
/// (matching the UI's name cap), dropping a `^`-color escape (`^0`..`^9` skips both
/// bytes; a lone `^` skips one) and any control char (`< 32`). Unlike [`SanitizeString`]
/// it does **not** lowercase. Writes a NUL-terminated result to `out`.
///
/// Oracle-tested (`jka_SanitizeString2`): a pure byte transform with no `level`/`trap`
/// deps. Faithful 1:1 with original JKA.
///
/// # Safety
/// `in_` must be a valid NUL-terminated C string; `out` must point to a buffer large
/// enough to hold the (never-longer, capped at `MAX_NAME_LENGTH`) result plus its NUL.
pub unsafe fn SanitizeString2(in_: *mut c_char, out: *mut c_char) {
    let mut i: isize = 0;
    let mut r: isize = 0;

    while *in_.offset(i) != 0 {
        if i >= MAX_NAME_LENGTH as isize - 1 {
            // the ui truncates the name here..
            break;
        }

        if *in_.offset(i) == b'^' as c_char {
            let next = *in_.offset(i + 1);
            if (48..=57).contains(&(next as c_int)) {
                // only skip it if there's a number after it for the color
                i += 2;
                continue;
            } else {
                // just skip the ^
                i += 1;
                continue;
            }
        }

        if *in_.offset(i) < 32 {
            i += 1;
            continue;
        }

        *out.offset(r) = *in_.offset(i);
        r += 1;
        i += 1;
    }
    *out.offset(r) = 0;
}

/// `int G_ClientNumberFromStrippedName( const char* name )` (g_cmds.c:1893). Same as
/// [`G_ClientNumberFromName`] but uses [`SanitizeString2`] (strip color/control,
/// truncate at `MAX_NAME_LENGTH`) on both the query and each connected client's
/// `pers.netname` before comparing; returns `-1` on no match.
///
/// No oracle: walks `level.clients` (the connect/team-infra precedent); the inner
/// [`SanitizeString2`]/[`strcmp`] are oracled/libc. Faithful 1:1 with original JKA.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string; `level` must be initialized.
pub unsafe fn G_ClientNumberFromStrippedName(name: *const c_char) -> c_int {
    let lvl = addr_of!(level);
    let mut s2 = [0 as c_char; MAX_STRING_CHARS];
    let mut n2 = [0 as c_char; MAX_STRING_CHARS];

    // check for a name match
    SanitizeString2(name as *mut c_char, s2.as_mut_ptr());
    let mut i: c_int = 0;
    let mut cl = (*lvl).clients;
    while i < (*lvl).numConnectedClients {
        SanitizeString2((*cl).pers.netname.as_ptr() as *mut c_char, n2.as_mut_ptr());
        if strcmp(n2.as_ptr(), s2.as_ptr()) == 0 {
            return i;
        }
        i += 1;
        cl = cl.offset(1);
    }

    -1
}

/// `void StandardSetBodyAnim(gentity_t *self, int anim, int flags)` (g_cmds.c:2991).
/// Thin wrapper: drive `self`'s torso+legs ([`SETANIM_BOTH`]) to `anim` with `flags`
/// and a zero blend time via [`G_SetAnim`]. Used by the kiss-emote command to put both
/// participants into a stand pose.
///
/// No oracle: pure delegation to [`G_SetAnim`] (already-landed anim infra). Faithful
/// 1:1 with original JKA.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a non-null `client` (the [`G_SetAnim`]
/// contract).
pub unsafe fn StandardSetBodyAnim(self_: *mut gentity_t, anim: c_int, flags: c_int) {
    G_SetAnim(self_, core::ptr::null_mut(), SETANIM_BOTH, anim, flags, 0);
}

/// `static int G_ClientNumFromNetname(char *name)` (g_cmds.c:3005). Find the client
/// number whose `pers.netname` exactly (case-insensitively, [`Q_stricmp`]) matches
/// `name`, scanning `g_entities[0..MAX_CLIENTS]` for an `inuse` ent with a `client`;
/// returns `ent->s.number` on the first match, else `-1`.
///
/// Unlike [`G_ClientNumberFromName`] this walks the raw entity array (not the packed
/// `level.clients`) and does no sanitization — a direct netname compare.
///
/// No oracle: walks `g_entities` (the connect/team-infra precedent); the inner
/// [`Q_stricmp`] is libc. Faithful 1:1 with original JKA.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string; `g_entities` must be a valid module
/// static of at least `MAX_CLIENTS` entries (true after `G_InitGame`).
pub unsafe fn G_ClientNumFromNetname(name: *mut c_char) -> c_int {
    let mut i: usize = 0;

    while i < MAX_CLIENTS {
        let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i);

        if !ent.is_null()
            && (*ent).inuse != QFALSE
            && !(*ent).client.is_null()
            && Q_stricmp((*(*ent).client).pers.netname.as_ptr(), name) == 0
        {
            return (*ent).s.number;
        }
        i += 1;
    }

    -1
}

/// `int G_ItemUsable(playerState_t *ps, int forcedUse)` (g_cmds.c:2380). The
/// server-side gate for using a holdable inventory item: rejects while on a vehicle or
/// with the use button still held, resolves `forcedUse==0` to the currently selected
/// `STAT_HOLDABLE_ITEM`'s [`giTag`](crate::codemp::game::bg_public::gitem_t), rejects if
/// it is not selectable ([`BG_IsItemSelectable`]), then per-item: the medpacs need
/// non-full, non-zero health; the seeker drone must not already be deployed; the sentry
/// gun and portable shield trace ([`trap::Trace`]) for clear ground/space, emitting an
/// [`EV_ITEMUSEFAIL`] event (with the failure reason) on the owning entity on failure;
/// everything else is always usable. Returns 1 if usable, else 0.
///
/// Mirrors the engine-side [`PM_ItemUsable`](crate::codemp::game::bg_pmove::PM_ItemUsable)
/// but does **not** include its `duelInProgress` gate (a deliberate difference in the C)
/// and uses [`trap::Trace`]/[`G_AddEvent`] rather than the pmove `trace`/event paths.
///
/// No oracle: drives [`trap::Trace`] and [`G_AddEvent`] on `g_entities` (the server-trap
/// precedent); the computable inner pieces ([`BG_IsItemSelectable`] / vector math) are
/// already landed. Faithful 1:1 with original JKA.
///
/// # Safety
/// `ps` must point to a valid `playerState_t` whose `clientNum` indexes a live
/// `g_entities[]` slot; `g_entities` must be initialized (true after `G_InitGame`).
pub unsafe fn G_ItemUsable(ps: *mut playerState_t, mut forcedUse: c_int) -> c_int {
    let mut fwd: vec3_t = [0.0; 3];
    let mut fwdorg: vec3_t = [0.0; 3];
    let mut dest: vec3_t = [0.0; 3];
    let mut pos: vec3_t = [0.0; 3];
    let mut yawonly: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut trtest: vec3_t = [0.0; 3];

    if (*ps).m_iVehicleNum != 0 {
        return 0;
    }

    if (*ps).pm_flags & PMF_USE_ITEM_HELD != 0 {
        //force to let go first
        return 0;
    }

    if forcedUse == 0 {
        forcedUse =
            (*addr_of!(bg_itemlist))[(*ps).stats[STAT_HOLDABLE_ITEM as usize] as usize].giTag;
    }

    if BG_IsItemSelectable(ps, forcedUse) == QFALSE {
        return 0;
    }

    match forcedUse {
        HI_MEDPAC | HI_MEDPAC_BIG => {
            if (*ps).stats[STAT_HEALTH as usize] >= (*ps).stats[STAT_MAX_HEALTH as usize] {
                return 0;
            }

            if (*ps).stats[STAT_HEALTH as usize] <= 0 {
                return 0;
            }

            1
        }
        HI_SEEKER => {
            if (*ps).eFlags & EF_SEEKERDRONE != 0 {
                G_AddEvent(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*ps).clientNum as usize),
                    EV_ITEMUSEFAIL,
                    SEEKER_ALREADYDEPLOYED,
                );
                return 0;
            }

            1
        }
        HI_SENTRY_GUN => {
            if (*ps).fd.sentryDeployed != QFALSE {
                G_AddEvent(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*ps).clientNum as usize),
                    EV_ITEMUSEFAIL,
                    SENTRY_ALREADYPLACED,
                );
                return 0;
            }

            yawonly[ROLL] = 0.0;
            yawonly[PITCH] = 0.0;
            yawonly[YAW] = (*ps).viewangles[YAW];

            VectorSet(&mut mins, -8.0, -8.0, 0.0);
            VectorSet(&mut maxs, 8.0, 8.0, 24.0);

            AngleVectors(&yawonly, Some(&mut fwd), None, None);

            fwdorg[0] = (*ps).origin[0] + fwd[0] * 64.0;
            fwdorg[1] = (*ps).origin[1] + fwd[1] * 64.0;
            fwdorg[2] = (*ps).origin[2] + fwd[2] * 64.0;

            trtest[0] = fwdorg[0] + fwd[0] * 16.0;
            trtest[1] = fwdorg[1] + fwd[1] * 16.0;
            trtest[2] = fwdorg[2] + fwd[2] * 16.0;

            let tr = trap::Trace(
                &(*ps).origin,
                &mins,
                &maxs,
                &trtest,
                (*ps).clientNum,
                MASK_PLAYERSOLID,
            );

            if (tr.fraction != 1.0 && tr.entityNum as c_int != (*ps).clientNum)
                || tr.startsolid != 0
                || tr.allsolid != 0
            {
                G_AddEvent(
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*ps).clientNum as usize),
                    EV_ITEMUSEFAIL,
                    SENTRY_NOROOM,
                );
                return 0;
            }

            1
        }
        HI_SHIELD => {
            mins[0] = -8.0;
            mins[1] = -8.0;
            mins[2] = 0.0;

            maxs[0] = 8.0;
            maxs[1] = 8.0;
            maxs[2] = 8.0;

            AngleVectors(&(*ps).viewangles, Some(&mut fwd), None, None);
            fwd[2] = 0.0;
            VectorMA(&(*ps).origin, 64.0, &fwd, &mut dest);
            let tr = trap::Trace(
                &(*ps).origin,
                &mins,
                &maxs,
                &dest,
                (*ps).clientNum,
                MASK_SHOT,
            );
            if tr.fraction > 0.9 && tr.startsolid == 0 && tr.allsolid == 0 {
                VectorCopy(&tr.endpos, &mut pos);
                VectorSet(&mut dest, pos[0], pos[1], pos[2] - 4096.0);
                let tr = trap::Trace(&pos, &mins, &maxs, &dest, (*ps).clientNum, MASK_SOLID);
                if tr.startsolid == 0 && tr.allsolid == 0 {
                    return 1;
                }
            }
            G_AddEvent(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*ps).clientNum as usize),
                EV_ITEMUSEFAIL,
                SHIELD_NOROOM,
            );
            0
        }
        _ => {
            //HI_JETPACK / HI_HEALTHDISP / HI_AMMODISP / HI_EWEB / HI_CLOAK / default
            1
        }
    }
}

/// `void DeathmatchScoreboardMessage( gentity_t *ent )` (g_cmds.c:25). Send the
/// `scores` server command to `ent` with one space-separated entry per connected
/// client (capped at [`MAX_CLIENT_SCORE_SEND`]): slot, score, ping, minutes-played,
/// score-flags, powerups, accuracy, and the various award/capture persistant counts.
///
/// The C builds each entry with `Com_sprintf` into a 1024-byte buffer and concatenates
/// into a 1400-byte string, breaking once a further entry would exceed 1022 bytes; the
/// total connected-client count and the red/blue team scores prefix the entry list.
/// `scoreFlags` is always `0` in retail JKA. Modelled here by accumulating the entries
/// in a `String` (each `%i` field is integer-exact under `format!`) with the same
/// `stringlength + j > 1022` truncation guard, then handing the assembled command to
/// [`trap::SendServerCommand`] (which takes a `&str`, so no `va` buffer is needed).
///
/// No oracle: drives `trap_SendServerCommand` plus the `level`/`g_entities` globals
/// (the connect/scoring-infra precedent). Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` indexed within `g_entities`; the `level`
/// client/sorted-client tables must be initialized.
pub unsafe fn DeathmatchScoreboardMessage(ent: *mut gentity_t) {
    // send the latest information on all clients
    let mut string = String::new();
    let mut stringlength: c_int = 0;
    let score_flags: c_int = 0;

    let mut num_sorted = (*addr_of!(level)).numConnectedClients;
    if num_sorted > MAX_CLIENT_SCORE_SEND {
        num_sorted = MAX_CLIENT_SCORE_SEND;
    }

    for i in 0..num_sorted {
        let sorted = (*addr_of!(level)).sortedClients[i as usize];
        let cl: *mut gclient_t = (*addr_of!(level)).clients.add(sorted as usize);

        let ping: c_int = if (*cl).pers.connected == CON_CONNECTING {
            -1
        } else if (*cl).ps.ping < 999 {
            (*cl).ps.ping
        } else {
            999
        };

        let accuracy: c_int = if (*cl).accuracy_shots != 0 {
            (*cl).accuracy_hits * 100 / (*cl).accuracy_shots
        } else {
            0
        };
        let perfect: c_int = if (*cl).ps.persistant[PERS_RANK as usize] == 0
            && (*cl).ps.persistant[PERS_KILLED as usize] == 0
        {
            1
        } else {
            0
        };

        let entry = format!(
            " {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            sorted,
            (*cl).ps.persistant[PERS_SCORE as usize],
            ping,
            ((*addr_of!(level)).time - (*cl).pers.enterTime) / 60000,
            score_flags,
            (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(sorted as usize))
                .s
                .powerups,
            accuracy,
            (*cl).ps.persistant[PERS_IMPRESSIVE_COUNT as usize],
            (*cl).ps.persistant[PERS_EXCELLENT_COUNT as usize],
            (*cl).ps.persistant[PERS_GAUNTLET_FRAG_COUNT as usize],
            (*cl).ps.persistant[PERS_DEFEND_COUNT as usize],
            (*cl).ps.persistant[PERS_ASSIST_COUNT as usize],
            perfect,
            (*cl).ps.persistant[PERS_CAPTURES as usize],
        );
        let j = entry.len() as c_int;
        if stringlength + j > 1022 {
            break;
        }
        string.push_str(&entry);
        stringlength += j;
    }

    // still want to know the total # of clients
    let i = (*addr_of!(level)).numConnectedClients;

    trap::SendServerCommand(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        &format!(
            "scores {} {} {}{}",
            i,
            (*addr_of!(level)).teamScores[TEAM_RED as usize],
            (*addr_of!(level)).teamScores[TEAM_BLUE as usize],
            string,
        ),
    );
}

/// `void Cmd_Score_f( gentity_t *ent )` (g_cmds.c:98). The `score` console command —
/// request current scoreboard information; a thin wrapper that just resends the
/// scoreboard via [`DeathmatchScoreboardMessage`].
///
/// No oracle: pure delegation to [`DeathmatchScoreboardMessage`]. Faithful 1:1.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` indexed within `g_entities`.
pub unsafe fn Cmd_Score_f(ent: *mut gentity_t) {
    DeathmatchScoreboardMessage(ent);
}

/// `void Cmd_LevelShot_f( gentity_t *ent )` (g_cmds.c:482).
///
/// This is just to help generate the level pictures for the menus. It goes to the
/// intermission immediately and sends over a command to the client to resize the
/// view, hide the scoreboard, and take a special screenshot.
///
/// No oracle: cheat-gated, drives [`BeginIntermission`] plus `trap_SendServerCommand`.
/// Faithful 1:1 with original JKA — the `g_gametype != 0` guard ("doesn't work in
/// single player") is preserved verbatim, comparing the raw integer like the C.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` indexed within `g_entities` (so
/// `ent - g_entities` is a valid client slot).
pub unsafe fn Cmd_LevelShot_f(ent: *mut gentity_t) {
    if CheatsOk(ent) == QFALSE {
        return;
    }

    // doesn't work in single player
    if (*addr_of!(g_gametype)).integer != 0 {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            "print \"Must be in g_gametype 0 for levelshot\n\"",
        );
        return;
    }

    BeginIntermission();
    trap::SendServerCommand(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        "clientLevelShot",
    );
}

/// `qboolean CheatsOk( gentity_t *ent )` (g_cmds.c:109). Gate for cheat console
/// commands: require `g_cheats` (the `sv_cheats` cvar) to be set and the entity to be
/// alive, printing the localized "NOCHEATS" / "MUSTBEALIVE" message and returning
/// [`QFALSE`] otherwise; [`QTRUE`] when both pass.
///
/// No oracle: drives the `g_cheats` cvar and `trap_SendServerCommand`. Faithful 1:1
/// with original JKA — the `va("print \"%s\n\"", …)` format is inlined into the Rust
/// `format!`, matching this file's established print precedent.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` indexed within `g_entities` (so
/// `ent - g_entities` is a valid client slot).
pub unsafe fn CheatsOk(ent: *mut gentity_t) -> qboolean {
    if (*addr_of!(g_cheats)).integer == 0 {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOCHEATS".as_ptr() as *mut c_char,
                )),
            ),
        );
        return QFALSE;
    }
    if (*ent).health <= 0 {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"MUSTBEALIVE".as_ptr() as *mut c_char,
                )),
            ),
        );
        return QFALSE;
    }
    QTRUE
}

/// `void Cmd_Give_f (gentity_t *cmdent, int baseArg)` (g_cmds.c:229).
///
/// Give items to a client. The cheat-gated `give`/`giveother` command: optionally
/// retargets to a client index (when `baseArg` is set, i.e. `giveother`), then matches
/// the item name against the magic strings (`all`, `health`, `weapons`, `weaponnum`,
/// `ammo`, `armor`, the award counters) and finally falls back to spawning a named
/// item right on the player and `Touch_Item`-ing it.
///
/// No oracle: pure entity-state side effects — mutates `ent->client->ps` stats/ammo/
/// persistant and `ent->health`, drives `trap_Argc`/`trap_Argv`, and spawns/frees a
/// real entity via `G_Spawn`/`G_SpawnItem`/`FinishSpawningItem`/`Touch_Item`/
/// `G_FreeEntity`. Faithful 1:1 with the C control flow. `Com_Printf("%i ...", i)`
/// collapses to `format!` (the print precedent); `trap_Argv(N, buf, sizeof(buf))`
/// becomes the established owned-`String`→fixed-`c_char`-buffer copy so the later
/// `Q_stricmp`/`atoi` see a NUL-terminated buffer byte-for-byte; `memset(&trace, 0,
/// sizeof(trace))` is the codebase `trace_t = core::mem::zeroed()` zero-init idiom.
///
/// # Safety
/// `cmdent` must point to a valid `gentity_t` indexed within `g_entities` with a
/// non-null `client`; with `baseArg` set the resolved target client is validated
/// (`inuse`/`client`) before use, mirroring the C.
#[allow(unused_assignments)] // faithful: C dead-resets `i = 0;` after the give_all holdable loop
pub unsafe fn Cmd_Give_f(cmdent: *mut gentity_t, base_arg: c_int) {
    let mut name = [0 as c_char; MAX_TOKEN_CHARS];
    let ent: *mut gentity_t;
    let it: *mut gitem_t;
    let mut i: c_int;
    let give_all: qboolean;
    let it_ent: *mut gentity_t;
    let trace: trace_t;
    let mut arg = [0 as c_char; MAX_TOKEN_CHARS];

    if CheatsOk(cmdent) == QFALSE {
        return;
    }

    if base_arg != 0 {
        let mut otherindex = [0 as c_char; MAX_TOKEN_CHARS];

        // trap_Argv( 1, otherindex, sizeof( otherindex ) );
        {
            let a = trap::Argv(1);
            let bytes = a.as_bytes();
            let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
            for k in 0..n {
                otherindex[k] = bytes[k] as c_char;
            }
            otherindex[n] = 0;
        }

        if otherindex[0] == 0 {
            Com_Printf("giveother requires that the second argument be a client index number.\n");
            return;
        }

        i = atoi(otherindex.as_ptr());

        if i < 0 || i >= MAX_CLIENTS as c_int {
            Com_Printf(&format!("{} is not a client index\n", i));
            return;
        }

        ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*ent).inuse == QFALSE || (*ent).client.is_null() {
            Com_Printf(&format!("{} is not an active client\n", i));
            return;
        }
    } else {
        ent = cmdent;
    }

    // trap_Argv( 1+baseArg, name, sizeof( name ) );
    {
        let a = trap::Argv(1 + base_arg);
        let bytes = a.as_bytes();
        let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
        for k in 0..n {
            name[k] = bytes[k] as c_char;
        }
        name[n] = 0;
    }

    if Q_stricmp(name.as_ptr(), c"all".as_ptr()) == 0 {
        give_all = QTRUE;
    } else {
        give_all = QFALSE;
    }

    if give_all != QFALSE {
        i = 0;
        while i < HI_NUM_HOLDABLE {
            (*(*ent).client).ps.stats[STAT_HOLDABLE_ITEMS as usize] |= 1 << i;
            i += 1;
        }
        i = 0;
    }

    if give_all != QFALSE || Q_stricmp(name.as_ptr(), c"health".as_ptr()) == 0 {
        if trap::Argc() == 3 + base_arg {
            // trap_Argv( 2+baseArg, arg, sizeof( arg ) );
            {
                let a = trap::Argv(2 + base_arg);
                let bytes = a.as_bytes();
                let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
                for k in 0..n {
                    arg[k] = bytes[k] as c_char;
                }
                arg[n] = 0;
            }
            (*ent).health = atoi(arg.as_ptr());
            if (*ent).health > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] {
                (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
            }
        } else {
            (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
        }
        if give_all == QFALSE {
            return;
        }
    }

    if give_all != QFALSE || Q_stricmp(name.as_ptr(), c"weapons".as_ptr()) == 0 {
        (*(*ent).client).ps.stats[STAT_WEAPONS as usize] =
            (1 << (LAST_USEABLE_WEAPON + 1)) - (1 << WP_NONE);
        if give_all == QFALSE {
            return;
        }
    }

    if give_all == QFALSE && Q_stricmp(name.as_ptr(), c"weaponnum".as_ptr()) == 0 {
        // trap_Argv( 2+baseArg, arg, sizeof( arg ) );
        {
            let a = trap::Argv(2 + base_arg);
            let bytes = a.as_bytes();
            let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
            for k in 0..n {
                arg[k] = bytes[k] as c_char;
            }
            arg[n] = 0;
        }
        (*(*ent).client).ps.stats[STAT_WEAPONS as usize] |= 1 << atoi(arg.as_ptr());
        return;
    }

    if give_all != QFALSE || Q_stricmp(name.as_ptr(), c"ammo".as_ptr()) == 0 {
        let mut num = 999;
        if trap::Argc() == 3 + base_arg {
            // trap_Argv( 2+baseArg, arg, sizeof( arg ) );
            {
                let a = trap::Argv(2 + base_arg);
                let bytes = a.as_bytes();
                let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
                for k in 0..n {
                    arg[k] = bytes[k] as c_char;
                }
                arg[n] = 0;
            }
            num = atoi(arg.as_ptr());
        }
        for i in 0..MAX_WEAPONS {
            (*(*ent).client).ps.ammo[i] = num;
        }
        if give_all == QFALSE {
            return;
        }
    }

    if give_all != QFALSE || Q_stricmp(name.as_ptr(), c"armor".as_ptr()) == 0 {
        if trap::Argc() == 3 + base_arg {
            // trap_Argv( 2+baseArg, arg, sizeof( arg ) );
            {
                let a = trap::Argv(2 + base_arg);
                let bytes = a.as_bytes();
                let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
                for k in 0..n {
                    arg[k] = bytes[k] as c_char;
                }
                arg[n] = 0;
            }
            (*(*ent).client).ps.stats[STAT_ARMOR as usize] = atoi(arg.as_ptr());
        } else {
            (*(*ent).client).ps.stats[STAT_ARMOR as usize] =
                (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
        }

        if give_all == QFALSE {
            return;
        }
    }

    if Q_stricmp(name.as_ptr(), c"excellent".as_ptr()) == 0 {
        (*(*ent).client).ps.persistant[PERS_EXCELLENT_COUNT as usize] += 1;
        return;
    }
    if Q_stricmp(name.as_ptr(), c"impressive".as_ptr()) == 0 {
        (*(*ent).client).ps.persistant[PERS_IMPRESSIVE_COUNT as usize] += 1;
        return;
    }
    if Q_stricmp(name.as_ptr(), c"gauntletaward".as_ptr()) == 0 {
        (*(*ent).client).ps.persistant[PERS_GAUNTLET_FRAG_COUNT as usize] += 1;
        return;
    }
    if Q_stricmp(name.as_ptr(), c"defend".as_ptr()) == 0 {
        (*(*ent).client).ps.persistant[PERS_DEFEND_COUNT as usize] += 1;
        return;
    }
    if Q_stricmp(name.as_ptr(), c"assist".as_ptr()) == 0 {
        (*(*ent).client).ps.persistant[PERS_ASSIST_COUNT as usize] += 1;
        return;
    }

    // spawn a specific item right on the player
    if give_all == QFALSE {
        it = BG_FindItem(name.as_ptr());
        if it.is_null() {
            return;
        }

        it_ent = G_Spawn();
        VectorCopy(&(*ent).r.currentOrigin, &mut (*it_ent).s.origin);
        (*it_ent).classname = (*it).classname;
        G_SpawnItem(it_ent, it);
        FinishSpawningItem(it_ent);
        trace = core::mem::zeroed();
        Touch_Item(it_ent, ent, addr_of!(trace) as *mut trace_t);
        if (*it_ent).inuse != QFALSE {
            G_FreeEntity(it_ent);
        }
    }
}

/// `void Cmd_God_f (gentity_t *ent)` (g_cmds.c:403). The `god` cheat command: gated by
/// [`CheatsOk`], toggle the `FL_GODMODE` entity flag and print the new on/off state.
///
/// No oracle: [`CheatsOk`] + entity-flag toggle + `trap_SendServerCommand`. Faithful
/// 1:1; the `va("print \"%s\"", msg)` collapses to `format!` (the print precedent).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` indexed within `g_entities`.
pub unsafe fn Cmd_God_f(ent: *mut gentity_t) {
    if CheatsOk(ent) == QFALSE {
        return;
    }

    (*ent).flags ^= FL_GODMODE;
    let msg = if (*ent).flags & FL_GODMODE == 0 {
        "godmode OFF\n"
    } else {
        "godmode ON\n"
    };

    trap::SendServerCommand(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        &format!("print \"{}\"", msg),
    );
}

/// `void Cmd_Notarget_f( gentity_t *ent )` (g_cmds.c:430). The `notarget` cheat
/// command: gated by [`CheatsOk`], toggle the `FL_NOTARGET` entity flag (NPCs ignore
/// the player) and print the new on/off state.
///
/// No oracle: [`CheatsOk`] + entity-flag toggle + `trap_SendServerCommand`. Faithful
/// 1:1.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` indexed within `g_entities`.
pub unsafe fn Cmd_Notarget_f(ent: *mut gentity_t) {
    if CheatsOk(ent) == QFALSE {
        return;
    }

    (*ent).flags ^= FL_NOTARGET;
    let msg = if (*ent).flags & FL_NOTARGET == 0 {
        "notarget OFF\n"
    } else {
        "notarget ON\n"
    };

    trap::SendServerCommand(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        &format!("print \"{}\"", msg),
    );
}

/// `void Cmd_Noclip_f( gentity_t *ent )` (g_cmds.c:454). The `noclip` cheat command:
/// gated by [`CheatsOk`], print the new state then toggle the client's `noclip` flag.
///
/// No oracle: [`CheatsOk`] + client-field toggle + `trap_SendServerCommand`. Faithful
/// 1:1 — the message reflects the pre-toggle state exactly as the C does (it reads
/// `noclip` for the message, then flips it).
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`, indexed within
/// `g_entities`.
pub unsafe fn Cmd_Noclip_f(ent: *mut gentity_t) {
    if CheatsOk(ent) == QFALSE {
        return;
    }

    let msg = if (*(*ent).client).noclip != QFALSE {
        "noclip OFF\n"
    } else {
        "noclip ON\n"
    };
    (*(*ent).client).noclip = if (*(*ent).client).noclip != QFALSE {
        QFALSE
    } else {
        QTRUE
    };

    trap::SendServerCommand(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        &format!("print \"{}\"", msg),
    );
}

/// `void SetTeam( gentity_t *ent, char *s )` (g_cmds.c:752). Apply a requested team /
/// spectator-state change for a client: parse the request string, enforce team-game
/// balance + game-mode player limits, kill the player when leaving a playing team,
/// commit the new session team/spectator state, fix up team leaders, broadcast the
/// change, teleport-out the old body, republish userinfo, and re-`ClientBegin`.
///
/// Faithful 1:1 with original JKA. Several commented-out `g_forceBasedTeams` branches
/// are preserved verbatim as comments. Callees [`SetTeamQuick`] (g_saga), [`ClientBegin`],
/// and [`player_die`] (g_combat) are the real ports.
///
/// No oracle: side-effecting team/session-state mutation driving
/// `trap_SendServerCommand` plus the `level`/`g_entities`/cvar globals (the
/// connect/team-infra precedent).
///
/// # Safety
/// `ent`/`ent->client` must be valid and indexed within `g_entities`/`level.clients`;
/// `s` must be a NUL-terminated C string.
pub unsafe fn SetTeam(ent: *mut gentity_t, s: *mut c_char) {
    let team: c_int;
    let old_team: c_int;
    let client: *mut gclient_t;
    let client_num: c_int;
    let spec_state: spectatorState_t;
    let mut spec_client: c_int;
    let team_leader: c_int;

    //
    // see what change is requested
    //
    client = (*ent).client;

    client_num = client.offset_from((*addr_of!(level)).clients) as c_int;
    spec_client = 0;
    let mut spec_state_v = SPECTATOR_NOT;
    if Q_stricmp(s, c"scoreboard".as_ptr()) == 0 || Q_stricmp(s, c"score".as_ptr()) == 0 {
        team = TEAM_SPECTATOR;
        spec_state_v = SPECTATOR_SCOREBOARD;
    } else if Q_stricmp(s, c"follow1".as_ptr()) == 0 {
        team = TEAM_SPECTATOR;
        spec_state_v = SPECTATOR_FOLLOW;
        spec_client = -1;
    } else if Q_stricmp(s, c"follow2".as_ptr()) == 0 {
        team = TEAM_SPECTATOR;
        spec_state_v = SPECTATOR_FOLLOW;
        spec_client = -2;
    } else if Q_stricmp(s, c"spectator".as_ptr()) == 0 || Q_stricmp(s, c"s".as_ptr()) == 0 {
        team = TEAM_SPECTATOR;
        spec_state_v = SPECTATOR_FREE;
    } else if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        // if running a team game, assign player to one of the teams
        spec_state_v = SPECTATOR_NOT;
        if Q_stricmp(s, c"red".as_ptr()) == 0 || Q_stricmp(s, c"r".as_ptr()) == 0 {
            team = TEAM_RED;
        } else if Q_stricmp(s, c"blue".as_ptr()) == 0 || Q_stricmp(s, c"b".as_ptr()) == 0 {
            team = TEAM_BLUE;
        } else {
            // pick the team with the least number of players
            //For now, don't do this. The legalize function will set powers properly now.
            /*
            if (g_forceBasedTeams.integer)
            {
                if (ent->client->ps.fd.forceSide == FORCE_LIGHTSIDE)
                {
                    team = TEAM_BLUE;
                }
                else
                {
                    team = TEAM_RED;
                }
            }
            else
            {
            */
            team = PickTeam(client_num);
            //}
        }

        if (*addr_of!(g_teamForceBalance)).integer != 0 && (*addr_of!(g_trueJedi)).integer == 0 {
            let mut counts = [0 as c_int; TEAM_NUM_TEAMS as usize];

            counts[TEAM_BLUE as usize] = TeamCount((*(*ent).client).ps.clientNum, TEAM_BLUE);
            counts[TEAM_RED as usize] = TeamCount((*(*ent).client).ps.clientNum, TEAM_RED);

            // We allow a spread of two
            if team == TEAM_RED && counts[TEAM_RED as usize] - counts[TEAM_BLUE as usize] > 1 {
                //For now, don't do this. The legalize function will set powers properly now.
                /*
                if (g_forceBasedTeams.integer && ent->client->ps.fd.forceSide == FORCE_DARKSIDE)
                {
                    trap_SendServerCommand( ent->client->ps.clientNum,
                        va("print \"%s\n\"", G_GetStringEdString("MP_SVGAME", "TOOMANYRED_SWITCH")) );
                }
                else
                */
                {
                    trap::SendServerCommand(
                        (*(*ent).client).ps.clientNum,
                        &format!(
                            "print \"{}\n\"",
                            Sz(G_GetStringEdString(
                                c"MP_SVGAME".as_ptr() as *mut c_char,
                                c"TOOMANYRED".as_ptr() as *mut c_char,
                            )),
                        ),
                    );
                }
                return; // ignore the request
            }
            if team == TEAM_BLUE && counts[TEAM_BLUE as usize] - counts[TEAM_RED as usize] > 1 {
                //For now, don't do this. The legalize function will set powers properly now.
                /*
                if (g_forceBasedTeams.integer && ent->client->ps.fd.forceSide == FORCE_LIGHTSIDE)
                {
                    trap_SendServerCommand( ent->client->ps.clientNum,
                        va("print \"%s\n\"", G_GetStringEdString("MP_SVGAME", "TOOMANYBLUE_SWITCH")) );
                }
                else
                */
                {
                    trap::SendServerCommand(
                        (*(*ent).client).ps.clientNum,
                        &format!(
                            "print \"{}\n\"",
                            Sz(G_GetStringEdString(
                                c"MP_SVGAME".as_ptr() as *mut c_char,
                                c"TOOMANYBLUE".as_ptr() as *mut c_char,
                            )),
                        ),
                    );
                }
                return; // ignore the request
            }

            // It's ok, the team we are switching to has less or same number of players
        }

        //For now, don't do this. The legalize function will set powers properly now.
        /*
        if (g_forceBasedTeams.integer)
        {
            if (team == TEAM_BLUE && ent->client->ps.fd.forceSide != FORCE_LIGHTSIDE)
            {
                trap_SendServerCommand( ent-g_entities, va("print \"%s\n\"", G_GetStringEdString("MP_SVGAME", "MUSTBELIGHT")) );
                return;
            }
            if (team == TEAM_RED && ent->client->ps.fd.forceSide != FORCE_DARKSIDE)
            {
                trap_SendServerCommand( ent-g_entities, va("print \"%s\n\"", G_GetStringEdString("MP_SVGAME", "MUSTBEDARK")) );
                return;
            }
        }
        */
    } else {
        // force them to spectators if there aren't any spots free
        team = TEAM_FREE;
    }

    spec_state = spec_state_v;
    let mut team = team;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        if (*client).tempSpectate >= (*addr_of!(level)).time && team == TEAM_SPECTATOR {
            //sorry, can't do that.
            return;
        }

        (*client).sess.siegeDesiredTeam = team;
        //oh well, just let them go.
        /*
        if (team != TEAM_SPECTATOR)
        { //can't switch to anything in siege unless you want to switch to being a fulltime spectator
            //fill them in on their objectives for this team now
            trap_SendServerCommand(ent-g_entities, va("sb %i", client->sess.siegeDesiredTeam));

            trap_SendServerCommand( ent-g_entities, va("print \"You will be on the selected team the next time the round begins.\n\"") );
            return;
        }
        */
        if (*client).sess.sessionTeam != TEAM_SPECTATOR && team != TEAM_SPECTATOR {
            //not a spectator now, and not switching to spec, so you have to wait til you die.
            //trap_SendServerCommand( ent-g_entities, va("print \"You will be on the selected team the next time you respawn.\n\"") );
            let do_begin: qboolean;
            if (*(*ent).client).tempSpectate >= (*addr_of!(level)).time {
                do_begin = QFALSE;
            } else {
                do_begin = QTRUE;
            }

            if do_begin != QFALSE {
                // Kill them so they automatically respawn in the team they wanted.
                if (*ent).health > 0 {
                    (*ent).flags &= !FL_GODMODE;
                    (*(*ent).client).ps.stats[STAT_HEALTH as usize] = 0;
                    (*ent).health = 0;
                    player_die(ent, ent, ent, 100000, MOD_TEAM_CHANGE);
                }
            }

            if (*(*ent).client).sess.sessionTeam != (*(*ent).client).sess.siegeDesiredTeam {
                SetTeamQuick(ent, (*(*ent).client).sess.siegeDesiredTeam, QFALSE);
            }

            return;
        }
    }

    // override decision if limiting the players
    if (*addr_of!(g_gametype)).integer == GT_DUEL && (*addr_of!(level)).numNonSpectatorClients >= 2
    {
        team = TEAM_SPECTATOR;
    } else if (*addr_of!(g_gametype)).integer == GT_POWERDUEL
        && ((*addr_of!(level)).numPlayingClients >= 3 || G_PowerDuelCheckFail(ent) != QFALSE)
    {
        team = TEAM_SPECTATOR;
    } else if (*addr_of!(g_maxGameClients)).integer > 0
        && (*addr_of!(level)).numNonSpectatorClients >= (*addr_of!(g_maxGameClients)).integer
    {
        team = TEAM_SPECTATOR;
    }

    //
    // decide if we will allow the change
    //
    old_team = (*client).sess.sessionTeam;
    if team == old_team && team != TEAM_SPECTATOR {
        return;
    }

    //
    // execute the team change
    //

    //If it's siege then show the mission briefing for the team you just joined.
    //	if (g_gametype.integer == GT_SIEGE && team != TEAM_SPECTATOR)
    //	{
    //		trap_SendServerCommand(clientNum, va("sb %i", team));
    //	}

    // if the player was dead leave the body
    if (*client).ps.stats[STAT_HEALTH as usize] <= 0 && (*client).sess.sessionTeam != TEAM_SPECTATOR
    {
        MaintainBodyQueue(ent);
    }

    // he starts at 'base'
    (*client).pers.teamState.state = TEAM_BEGIN;
    if old_team != TEAM_SPECTATOR {
        // Kill him (makes sure he loses flags, etc)
        (*ent).flags &= !FL_GODMODE;
        (*(*ent).client).ps.stats[STAT_HEALTH as usize] = 0;
        (*ent).health = 0;
        g_dontPenalizeTeam = QTRUE;
        player_die(ent, ent, ent, 100000, MOD_SUICIDE);
        g_dontPenalizeTeam = QFALSE;
    }
    // they go to the end of the line for tournements
    if team == TEAM_SPECTATOR {
        if (*addr_of!(g_gametype)).integer != GT_DUEL || old_team != TEAM_SPECTATOR {
            //so you don't get dropped to the bottom of the queue for changing skins, etc.
            (*client).sess.spectatorTime = (*addr_of!(level)).time;
        }
    }

    (*client).sess.sessionTeam = team;
    (*client).sess.spectatorState = spec_state;
    (*client).sess.spectatorClient = spec_client;

    (*client).sess.teamLeader = QFALSE;
    if team == TEAM_RED || team == TEAM_BLUE {
        team_leader = TeamLeader(team);
        // if there is no team leader or the team leader is a bot and this client is not a bot
        if team_leader == -1
            || ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(client_num as usize))
            .r
            .svFlags
                & SVF_BOT)
                == 0
                && ((*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(team_leader as usize))
                .r
                .svFlags
                    & SVF_BOT)
                    != 0
        {
            //SetLeader( team, clientNum );
        }
    }
    // make sure there is a team leader on the team the player came from
    if old_team == TEAM_RED || old_team == TEAM_BLUE {
        CheckTeamLeader(old_team);
    }

    BroadcastTeamChange(client, old_team);

    //make a disappearing effect where they were before teleporting them to the appropriate spawn point,
    //if we were not on the spec team
    if old_team != TEAM_SPECTATOR {
        let tent = G_TempEntity(&(*client).ps.origin, EV_PLAYER_TELEPORT_OUT);
        (*tent).s.clientNum = client_num;
    }

    // get and distribute relevent paramters
    ClientUserinfoChanged(client_num);

    if g_preventTeamBegin == QFALSE {
        ClientBegin(client_num, QFALSE);
    }
}

/// `void StopFollowing( gentity_t *ent )` (g_cmds.c:953). Drop a follow-spectator to
/// free-floating spectator mode: reset team/spectator session state, clear the
/// `PMF_FOLLOW` flag and `SVF_BOT`, point `clientNum` back at the entity's own slot,
/// and zero out weapon/vehicle/view/zoom/hand-extend/anim playerState fields.
///
/// No oracle: pure `client->ps`/`sess`/`r` field assignments on a live entity (the
/// connect/team-infra precedent). Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`, indexed within
/// `g_entities`.
pub unsafe fn StopFollowing(ent: *mut gentity_t) {
    let client = (*ent).client;
    (*client).ps.persistant[PERS_TEAM as usize] = TEAM_SPECTATOR;
    (*client).sess.sessionTeam = TEAM_SPECTATOR;
    (*client).sess.spectatorState = SPECTATOR_FREE;
    (*client).ps.pm_flags &= !PMF_FOLLOW;
    (*ent).r.svFlags &= !SVF_BOT;
    (*client).ps.clientNum =
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;
    (*client).ps.weapon = WP_NONE;
    (*client).ps.m_iVehicleNum = 0;
    (*client).ps.viewangles[ROLL as usize] = 0.0;
    (*client).ps.forceHandExtend = HANDEXTEND_NONE;
    (*client).ps.forceHandExtendTime = 0;
    (*client).ps.zoomMode = 0;
    (*client).ps.zoomLocked = 0;
    (*client).ps.zoomLockTime = 0;
    (*client).ps.legsAnim = 0;
    (*client).ps.legsTimer = 0;
    (*client).ps.torsoAnim = 0;
    (*client).ps.torsoTimer = 0;
}

/// `void Cmd_Follow_f( gentity_t *ent )` (g_cmds.c:1409). The `follow` console
/// command: start spectating a named/numbered client. With no argument, toggles off
/// an active follow via [`StopFollowing`]; otherwise resolves the target slot with
/// [`ClientNumberFromString`], rejecting self, spectators, and disconnected slots,
/// then forces the follower to spectator team ([`SetTeam`]) and latches the follow
/// state.
///
/// No oracle: drives `level.clients`/session state plus [`SetTeam`]. Faithful 1:1 with
/// original JKA — the tournament-loss `//WTF???` quirk is preserved verbatim, and the
/// `arg` token buffer is filled from [`trap::Argv`]'s owned `String` (this file's
/// established `trap_Argv` idiom) before [`ClientNumberFromString`].
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_gametype` must be initialized.
pub unsafe fn Cmd_Follow_f(ent: *mut gentity_t) {
    let i: c_int;
    let mut arg = [0 as c_char; MAX_TOKEN_CHARS];

    if trap::Argc() != 2 {
        if (*(*ent).client).sess.spectatorState == SPECTATOR_FOLLOW {
            StopFollowing(ent);
        }
        return;
    }

    // trap_Argv( 1, arg, sizeof( arg ) );
    {
        let s = trap::Argv(1);
        let bytes = s.as_bytes();
        let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
        for k in 0..n {
            arg[k] = bytes[k] as c_char;
        }
        arg[n] = 0;
    }
    i = ClientNumberFromString(ent, arg.as_ptr());
    if i == -1 {
        return;
    }

    // can't follow self
    if core::ptr::eq((*addr_of!(level)).clients.offset(i as isize), (*ent).client) {
        return;
    }

    // can't follow another spectator
    if (*(*addr_of!(level)).clients.offset(i as isize))
        .sess
        .sessionTeam
        == TEAM_SPECTATOR
    {
        return;
    }

    // if they are playing a tournement game, count as a loss
    if ((*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL)
        && (*(*ent).client).sess.sessionTeam == TEAM_FREE
    {
        //WTF???
        (*(*ent).client).sess.losses += 1;
    }

    // first set them to spectator
    if (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR {
        SetTeam(ent, c"spectator".as_ptr() as *mut c_char);
    }

    (*(*ent).client).sess.spectatorState = SPECTATOR_FOLLOW;
    (*(*ent).client).sess.spectatorClient = i;
}

/// `void Cmd_FollowCycle_f( gentity_t *ent, int dir )` (g_cmds.c:1457). Advance the
/// follow target one connected, non-spectator client in direction `dir` (+1/-1),
/// wrapping at `level.maxclients` and stopping when it loops back to the original.
/// Forces the follower to spectator team ([`SetTeam`]) on entry, [`G_Error`]s on a bad
/// `dir`, and leaves the target unchanged when no valid client is found.
///
/// No oracle: drives `level.clients`/session state plus [`SetTeam`]/[`G_Error`].
/// Faithful 1:1 with original JKA — the tournament-loss `//WTF???` quirk and the
/// do/while scan with the rollover branches are preserved verbatim. `G_Error`'s
/// `va`-style `%i` is rendered via `format!` (this file/cluster's `G_Error` idiom).
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_gametype` must be initialized.
pub unsafe fn Cmd_FollowCycle_f(ent: *mut gentity_t, dir: c_int) {
    let mut clientnum: c_int;
    let original: c_int;

    // if they are playing a tournement game, count as a loss
    if ((*addr_of!(g_gametype)).integer == GT_DUEL
        || (*addr_of!(g_gametype)).integer == GT_POWERDUEL)
        && (*(*ent).client).sess.sessionTeam == TEAM_FREE
    {
        //WTF???
        (*(*ent).client).sess.losses += 1;
    }
    // first set them to spectator
    if (*(*ent).client).sess.spectatorState == SPECTATOR_NOT {
        SetTeam(ent, c"spectator".as_ptr() as *mut c_char);
    }

    if dir != 1 && dir != -1 {
        G_Error(&format!("Cmd_FollowCycle_f: bad dir {}", dir));
    }

    clientnum = (*(*ent).client).sess.spectatorClient;
    original = clientnum;
    // C `do { … } while ( clientnum != original )`: body always runs once, then
    // re-tests the condition. The C `continue`s jump to that `while` test, so each is
    // mapped to a `clientnum != original` check guarding the next iteration.
    loop {
        clientnum += dir;
        if clientnum >= (*addr_of!(level)).maxclients {
            clientnum = 0;
        }
        if clientnum < 0 {
            clientnum = (*addr_of!(level)).maxclients - 1;
        }

        // can only follow connected clients
        if (*(*addr_of!(level)).clients.offset(clientnum as isize))
            .pers
            .connected
            != CON_CONNECTED
        {
            // continue;
        } else if (*(*addr_of!(level)).clients.offset(clientnum as isize))
            .sess
            .sessionTeam
            == TEAM_SPECTATOR
        {
            // can't follow another spectator
            // continue;
        } else {
            // this is good, we can use it
            (*(*ent).client).sess.spectatorClient = clientnum;
            (*(*ent).client).sess.spectatorState = SPECTATOR_FOLLOW;
            return;
        }

        if clientnum == original {
            break;
        }
    }

    // leave it where it was
}

/// `int G_TeamForSiegeClass(const char *clName)` (g_cmds.c:1129). Find which siege
/// team ([`SIEGETEAM_TEAM1`] / [`SIEGETEAM_TEAM2`]) owns the class named `clName`,
/// scanning each team's theme classes by name ([`Q_stricmp`]); returns `0` when no
/// theme exists for team 1 or no class matches.
///
/// No oracle: walks the loaded `bgSiegeTeams` themes via [`BG_SiegeFindThemeForTeam`].
/// Faithful 1:1 with original JKA — the `i >= MAX_SIEGE_CLASSES || i >= numClasses`
/// loop-rollover and team-switch logic preserved exactly.
///
/// # Safety
/// `cl_name` must be a valid NUL-terminated C string; the siege theme statics must be
/// initialized (true after the siege class/team load).
pub unsafe fn G_TeamForSiegeClass(cl_name: *const c_char) -> c_int {
    let mut i: c_int = 0;
    let mut team: c_int = SIEGETEAM_TEAM1;
    let mut stm: *mut siegeTeam_t = BG_SiegeFindThemeForTeam(team);

    if stm.is_null() {
        return 0;
    }

    while team <= SIEGETEAM_TEAM2 {
        let scl: *mut siegeClass_t = (*stm).classes[i as usize];

        if !scl.is_null() && (*scl).name[0] != 0 {
            if Q_stricmp(cl_name, (*scl).name.as_ptr()) == 0 {
                return team;
            }
        }

        i += 1;
        if i >= MAX_SIEGE_CLASSES as c_int || i >= (*stm).numClasses {
            if team == SIEGETEAM_TEAM2 {
                break;
            }
            team = SIEGETEAM_TEAM2;
            stm = BG_SiegeFindThemeForTeam(team);
            i = 0;
        }
    }

    0
}

/// `qboolean G_SetSaber(gentity_t *ent, int saberNum, char *saberName,
/// qboolean siegeOverride)` (g_cmds.c:1349). Set the client's saber slot `saberNum`
/// to `saberName` (defaults applied by [`WP_SetSaber`] when the name is invalid) and
/// record the resulting hilt names into `sess.saberType` / `sess.saber2Type`.
///
/// In Siege, refuse (return [`QFALSE`]) when the client's siege class has forced any
/// saber-related thing (stance / saber1 / saber2) unless `siegeOverride`. The name is
/// first truncated to 63 chars via the verbatim char-by-char loop; `"none"`/`"remove"`
/// on slot 0 is coerced to `"Kyle"` (saber 0 can't be removed this way). After
/// [`WP_SetSaber`], an empty saber-0 model trips the C `assert(0)` "should never
/// happen" path (modelled as [`debug_assert!`], matching C `assert`'s NDEBUG-gated
/// semantics) and sets `saberType` to `"none"`; otherwise the live hilt names are
/// copied out, and saber-1's model emptiness picks `"none"` vs. its name.
///
/// No oracle: drives `bgSiegeClasses`/`g_gametype` and mutates the client saber/session
/// fields via [`WP_SetSaber`] (the connect/team-infra precedent). Faithful 1:1.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`; `saber_name` must
/// be a valid NUL-terminated C string.
pub unsafe fn G_SetSaber(
    ent: *mut gentity_t,
    saber_num: c_int,
    saber_name: *mut c_char,
    siege_override: qboolean,
) -> qboolean {
    let mut trunc_saber_name = [0 as c_char; 64];
    let mut i: c_int = 0;

    if siege_override == QFALSE
        && (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*(*ent).client).siegeClass != -1
        && (bgSiegeClasses[(*(*ent).client).siegeClass as usize].saberStance != 0
            || bgSiegeClasses[(*(*ent).client).siegeClass as usize].saber1[0] != 0
            || bgSiegeClasses[(*(*ent).client).siegeClass as usize].saber2[0] != 0)
    {
        //don't let it be changed if the siege class has forced any saber-related things
        return QFALSE;
    }

    while *saber_name.offset(i as isize) != 0 && i < 64 - 1 {
        trunc_saber_name[i as usize] = *saber_name.offset(i as isize);
        i += 1;
    }
    trunc_saber_name[i as usize] = 0;

    if saber_num == 0
        && (Q_stricmp(c"none".as_ptr(), trunc_saber_name.as_ptr()) == 0
            || Q_stricmp(c"remove".as_ptr(), trunc_saber_name.as_ptr()) == 0)
    {
        //can't remove saber 0 like this
        strcpy(trunc_saber_name.as_mut_ptr(), c"Kyle".as_ptr());
    }

    //Set the saber with the arg given. If the arg is
    //not a valid sabername defaults will be used.
    WP_SetSaber(
        (*ent).s.number,
        (*(*ent).client).saber.as_mut_ptr(),
        saber_num,
        trunc_saber_name.as_ptr(),
    );

    if (*(*ent).client).saber[0].model[0] == 0 {
        debug_assert!(false); //should never happen!
        strcpy(
            (*(*ent).client).sess.saberType.as_mut_ptr(),
            c"none".as_ptr(),
        );
    } else {
        strcpy(
            (*(*ent).client).sess.saberType.as_mut_ptr(),
            (*(*ent).client).saber[0].name.as_ptr(),
        );
    }

    if (*(*ent).client).saber[1].model[0] == 0 {
        strcpy(
            (*(*ent).client).sess.saber2Type.as_mut_ptr(),
            c"none".as_ptr(),
        );
    } else {
        strcpy(
            (*(*ent).client).sess.saber2Type.as_mut_ptr(),
            (*(*ent).client).saber[1].name.as_ptr(),
        );
    }

    if WP_SaberStyleValidForSaber(
        addr_of_mut!((*(*ent).client).saber[0]),
        addr_of_mut!((*(*ent).client).saber[1]),
        (*(*ent).client).ps.saberHolstered,
        (*(*ent).client).ps.fd.saberAnimLevel,
    ) == QFALSE
    {
        WP_UseFirstValidSaberStyle(
            addr_of_mut!((*(*ent).client).saber[0]),
            addr_of_mut!((*(*ent).client).saber[1]),
            (*(*ent).client).ps.saberHolstered,
            addr_of_mut!((*(*ent).client).ps.fd.saberAnimLevel),
        );
        // C: saberAnimLevelBase = saberCycleQueue = saberAnimLevel
        let lvl = (*(*ent).client).ps.fd.saberAnimLevel;
        (*(*ent).client).ps.fd.saberAnimLevelBase = lvl;
        (*(*ent).client).saberCycleQueue = lvl;
    }

    QTRUE
}

/// `void Cmd_GameCommand_f( gentity_t *ent )` (g_cmds.c:1769). The `gc` command — a
/// player issues a canned bot-order ("game command"): arg 1 is the target player slot,
/// arg 2 indexes `gc_orders`. Sends the order as a private [`SAY_TELL`] to the named
/// player and echoes it back to the issuer.
///
/// No oracle: `trap_Argv`/`atoi` plumbing into [`G_Say`] over `g_entities`. Faithful
/// 1:1 with original JKA. The C `static char *gc_orders[]` (g_cmds.c:1759) becomes a
/// function-local `[*const c_char; 7]` of NUL-terminated literals (raw pointers are not
/// `Sync`, so a module-static cannot hold them; the array is referenced only here, so
/// the locality is behaviour-equivalent). The bounds check `order >
/// sizeof(gc_orders)/sizeof(char*)` uses `>` (not `>=`) — an off-by-one that in C
/// allows `order == 7` to read one element past the 7-entry array (UB). Rust would
/// instead panic on the OOB index, so the `>` gate is preserved verbatim for the live
/// `0..=6` range and the lone pathological `order == 7` is clamped to the last entry
/// (it can never deliver a meaningful order either way); see DEVIATIONS.md. The `atoi`
/// of args 1/2 is parsed through a NUL-terminated `CString` to match the C bit-for-bit.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `g_entities` must be initialized.
pub unsafe fn Cmd_GameCommand_f(ent: *mut gentity_t) {
    let gc_orders: [*const c_char; 7] = [
        c"hold your position".as_ptr(),
        c"hold this position".as_ptr(),
        c"come here".as_ptr(),
        c"cover me".as_ptr(),
        c"guard location".as_ptr(),
        c"search and destroy".as_ptr(),
        c"report".as_ptr(),
    ];

    let player: c_int;
    let order: c_int;

    // trap_Argv( 1, str, sizeof( str ) );
    let str_ = trap::Argv(1);
    let str_c = std::ffi::CString::new(str_).unwrap_or_default();
    player = atoi(str_c.as_ptr());
    // trap_Argv( 2, str, sizeof( str ) );
    let str_ = trap::Argv(2);
    let str_c = std::ffi::CString::new(str_).unwrap_or_default();
    order = atoi(str_c.as_ptr());

    if player < 0 || player >= MAX_CLIENTS as c_int {
        return;
    }
    if order < 0 || order > (gc_orders.len() as c_int) {
        return;
    }
    // C indexes gc_orders[order] where `order` may equal the element count (off-by-one
    // above); clamp to the last valid index in that single pathological case to mirror
    // the deliverable C behaviour without a Rust OOB panic.
    let idx = (order as usize).min(gc_orders.len() - 1);
    G_Say(
        ent,
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(player as isize),
        SAY_TELL,
        gc_orders[idx],
    );
    G_Say(ent, ent, SAY_TELL, gc_orders[idx]);
}

/// `void Cmd_Where_f( gentity_t *ent )` (g_cmds.c:1794). The `where` command — print
/// the player's current origin (via [`vtos`]) back to them.
///
/// No oracle: drives `trap_SendServerCommand` and reads `ent->s.origin`. Faithful 1:1;
/// the `va("print \"%s\n\"", vtos(…))` collapses to `format!` over [`Sz`]`(vtos(…))`.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` indexed within `g_entities`.
pub unsafe fn Cmd_Where_f(ent: *mut gentity_t) {
    trap::SendServerCommand(
        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
        &format!("print \"{}\n\"", Sz(vtos(&(*ent).s.origin))),
    );
}

/// `void Cmd_SetViewpos_f( gentity_t *ent )` (g_cmds.c:2330). The `setviewpos` cheat
/// command: require `g_cheats` and exactly 4 args (`x y z yaw`), then teleport the
/// player to that origin/yaw via [`TeleportPlayer`].
///
/// No oracle: `g_cheats` gate + `trap_Argc`/`trap_Argv` + [`TeleportPlayer`]. Faithful
/// 1:1; the [`trap::Argv`] owned-`String` is parsed through libc [`atof`] (a NUL-
/// terminated `CString`) to match the C `atof(buffer)` bit-for-bit.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`, indexed within
/// `g_entities`.
pub unsafe fn Cmd_SetViewpos_f(ent: *mut gentity_t) {
    let mut origin: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];

    if (*addr_of!(g_cheats)).integer == 0 {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOCHEATS".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if trap::Argc() != 5 {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            "print \"usage: setviewpos x y z yaw\n\"",
        );
        return;
    }

    VectorClear(&mut angles);
    for i in 0..3 {
        let buffer = trap::Argv(i + 1);
        let cbuf = std::ffi::CString::new(buffer).unwrap_or_default();
        origin[i as usize] = atof(cbuf.as_ptr()) as f32;
    }

    let buffer = trap::Argv(4);
    let cbuf = std::ffi::CString::new(buffer).unwrap_or_default();
    angles[YAW as usize] = atof(cbuf.as_ptr()) as f32;

    TeleportPlayer(ent, &origin, &angles);
}

/// `void Cmd_Stats_f( gentity_t *ent )` (g_cmds.c:2363). The `stats` command — in
/// retail JKA its entire body is commented out (it depended on the bot AAS
/// area-reachability traps), so it is a no-op. Ported as an empty function to preserve
/// the command-table slot 1:1; `ent` is accepted but unused, exactly as in the C.
///
/// No oracle: empty body. Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent` is unused; any value is accepted (the C body is fully commented out).
pub unsafe fn Cmd_Stats_f(_ent: *mut gentity_t) {
    /*
        int max, n, i;

        max = trap_AAS_PointReachabilityAreaIndex( NULL );

        n = 0;
        for ( i = 0; i < max; i++ ) {
            if ( ent->client->areabits[i >> 3] & (1 << (i & 7)) )
                n++;
        }

        //trap_SendServerCommand( ent-g_entities, va("print \"visited %d of %d areas\n\"", n, max));
        trap_SendServerCommand( ent-g_entities, va("print \"%d%% level coverage\n\"", n * 100 / max));
    */
}

/// `void Cmd_SaberAttackCycle_f(gentity_t *ent)` (g_cmds.c:2585). The `saberAttackCycle`
/// command — cycle the player's lightsaber combat stance / blade-toggle state. Bails on
/// a null entity/client. With dual sabers: toggle blade 2 on/off (sets `SS_DUAL`/`SS_FAST`
/// stance). With a staff (`numBlades > 1`): toggle the second staff blade (refusing to
/// re-enable an in-flight blade), applying the saber's `style`/`singleBladeStyle` now (or
/// queuing it via `saberCycleQueue` if busy). Otherwise cycle the animation stance: in
/// Siege with a class stance flag, scan upward through `SS_NUM_SABER_STYLES` for the next
/// allowed stance; else bump `saberAnimLevel` capped at `forcePowerLevel[FP_SABER_OFFENSE]`
/// (wrapping to `FORCE_LEVEL_1`). The chosen level is set immediately when not busy, else
/// queued. Emits `SABERSTANCEDEBUG:` prints when `d_saberStanceDebug` is set.
///
/// No oracle: drives `g_gametype`/`d_saberStanceDebug`/`bgSiegeClasses` and mutates the
/// client saber/playerState fields via [`G_Sound`] (the connect/team-infra precedent).
/// Faithful 1:1 — the two `#ifndef FINAL_BUILD`/commented blocks stay comments as in C.
///
/// # Safety
/// `ent` may be null or have a null `client` (handled); otherwise it must point to a
/// valid `gentity_t` indexed within `g_entities`.
pub unsafe fn Cmd_SaberAttackCycle_f(ent: *mut gentity_t) {
    // C: `int selectLevel = 0;` — the `= 0` is dead (both branches below assign it
    // before it is read), so it is dropped to keep the build warning-clean.
    let mut select_level: c_int;

    if ent.is_null() || (*ent).client.is_null() {
        return;
    }
    /*
    if (ent->client->ps.weaponTime > 0)
    { //no switching attack level when busy
        return;
    }
    */

    let client = (*ent).client;

    if (*client).saber[0].model[0] != 0 && (*client).saber[1].model[0] != 0 {
        //no cycling for akimbo
        if (*client).ps.saberHolstered == 1 {
            //have one holstered
            //unholster it
            G_Sound(ent, CHAN_AUTO, (*client).saber[1].soundOn);
            (*client).ps.saberHolstered = 0;
            //g_active should take care of this, but...
            (*client).ps.fd.saberAnimLevel = SS_DUAL;
        } else if (*client).ps.saberHolstered == 0 {
            //have none holstered
            //holster it
            G_Sound(ent, CHAN_AUTO, (*client).saber[1].soundOff);
            (*client).ps.saberHolstered = 1;
            //g_active should take care of this, but...
            (*client).ps.fd.saberAnimLevel = SS_FAST;
        }

        if (*addr_of!(d_saberStanceDebug)).integer != 0 {
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                "print \"SABERSTANCEDEBUG: Attempted to toggle dual saber blade.\n\"",
            );
        }
        return;
    }

    if (*client).saber[0].numBlades > 1 {
        //use staff stance then.
        if (*client).ps.saberHolstered == 1 {
            //second blade off
            if (*client).ps.saberInFlight != QFALSE {
                //can't turn second blade back on if it's in the air, you naughty boy!
                if (*addr_of!(d_saberStanceDebug)).integer != 0 {
                    trap::SendServerCommand(
                        ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            as c_int,
                        "print \"SABERSTANCEDEBUG: Attempted to toggle staff blade in air.\n\"",
                    );
                }
                return;
            }
            //turn it on
            G_Sound(ent, CHAN_AUTO, (*client).saber[0].soundOn);
            (*client).ps.saberHolstered = 0;
            //g_active should take care of this, but...
            // Xbox->PC stopgap: the Xbox `saber[0].style` (single locked style) is
            // gone; PC stores it as the (1<<style) bit in `stylesLearned` (see the
            // `saberStyle` parser handler). Recover the locked style by inverting that
            // encoding. PC's bg_saberLoad WP_UseFirstValidSaberStyle / the full PC
            // restructure of Cmd_SaberAttackCycle_f remain deferred (see DEVIATIONS).
            if (*client).saber[0].stylesLearned != 0 {
                let style = (*client).saber[0].stylesLearned.trailing_zeros() as c_int;
                if (*client).ps.weaponTime <= 0 {
                    //not busy, set it now
                    (*client).ps.fd.saberAnimLevel = style;
                } else {
                    //can't set it now or we might cause unexpected chaining, so queue it
                    (*client).saberCycleQueue = style;
                }
            }
        } else if (*client).ps.saberHolstered == 0 {
            //both blades on
            //turn second one off
            G_Sound(ent, CHAN_AUTO, (*client).saber[0].soundOff);
            (*client).ps.saberHolstered = 1;
            //g_active should take care of this, but...
            if (*client).saber[0].singleBladeStyle != SS_NONE {
                if (*client).ps.weaponTime <= 0 {
                    //not busy, set it now
                    (*client).ps.fd.saberAnimLevel = (*client).saber[0].singleBladeStyle;
                } else {
                    //can't set it now or we might cause unexpected chaining, so queue it
                    (*client).saberCycleQueue = (*client).saber[0].singleBladeStyle;
                }
            }
        }
        if (*addr_of!(d_saberStanceDebug)).integer != 0 {
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                "print \"SABERSTANCEDEBUG: Attempted to toggle staff blade.\n\"",
            );
        }
        return;
    }

    if (*client).saberCycleQueue != 0 {
        //resume off of the queue if we haven't gotten a chance to update it yet
        select_level = (*client).saberCycleQueue;
    } else {
        select_level = (*client).ps.fd.saberAnimLevel;
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && (*client).siegeClass != -1
        && bgSiegeClasses[(*client).siegeClass as usize].saberStance != 0
    {
        //we have a flag of useable stances so cycle through it instead
        let mut i = select_level + 1;

        while i != select_level {
            //cycle around upward til we hit the next style or end up back on this one
            if i >= SS_NUM_SABER_STYLES {
                //loop back around to the first valid
                i = SS_FAST;
            }

            if bgSiegeClasses[(*client).siegeClass as usize].saberStance & (1 << i) != 0 {
                //we can use this one, select it and break out.
                select_level = i;
                break;
            }
            i += 1;
        }

        if (*addr_of!(d_saberStanceDebug)).integer != 0 {
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                "print \"SABERSTANCEDEBUG: Attempted to cycle given class stance.\n\"",
            );
        }
    } else {
        select_level += 1;
        if select_level > (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] {
            select_level = FORCE_LEVEL_1;
        }
        if (*addr_of!(d_saberStanceDebug)).integer != 0 {
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                "print \"SABERSTANCEDEBUG: Attempted to cycle stance normally.\n\"",
            );
        }
    }
    /*
    #ifndef FINAL_BUILD
        switch ( selectLevel )
        {
        case FORCE_LEVEL_1:
            trap_SendServerCommand( ent-g_entities, va("print \"Lightsaber Combat Style: %sfast\n\"", S_COLOR_BLUE) );
            break;
        case FORCE_LEVEL_2:
            trap_SendServerCommand( ent-g_entities, va("print \"Lightsaber Combat Style: %smedium\n\"", S_COLOR_YELLOW) );
            break;
        case FORCE_LEVEL_3:
            trap_SendServerCommand( ent-g_entities, va("print \"Lightsaber Combat Style: %sstrong\n\"", S_COLOR_RED) );
            break;
        }
    #endif
    */
    if (*client).ps.weaponTime <= 0 {
        //not busy, set it now
        (*client).ps.fd.saberAnimLevel = select_level;
        (*client).ps.fd.saberAnimLevelBase = (*client).ps.fd.saberAnimLevel;
    } else {
        //can't set it now or we might cause unexpected chaining, so queue it
        (*client).saberCycleQueue = select_level;
        (*client).ps.fd.saberAnimLevelBase = (*client).saberCycleQueue;
    }
}

/// `void Cmd_EngageDuel_f( gentity_t *ent )` (g_cmds.c:2771). The `+engage_duel`
/// command — initiate (or accept) a private one-on-one saber duel with whoever the
/// caller is looking at. Bails early when `g_privateDuel` is off, in any duel/team
/// gametype, while on cooldown (`ps.duelTime`/`fd.privateDuelTime`), when not wielding
/// a saber, mid-throw (`saberInFlight`), already dueling, or when someone else is
/// dueling. Otherwise traces 256u along the view forward against `MASK_PLAYERSOLID`;
/// if it lands on a live, saber-wielding, non-teammate client it either *accepts* a
/// pending reciprocal challenge (both flagged `duelInProgress`, sabers holstered with
/// their off-sounds, broadcast "X accepted Y") or *issues* a fresh challenge
/// (centre-prints to both, `forceHandExtend = HANDEXTEND_DUELCHALLENGE`, 5s timer).
///
/// No oracle: drives `trap_Trace`/`trap_SendServerCommand`/`G_AddEvent`/`G_Sound` plus
/// the `g_entities`/`level`/`g_gametype`/`g_privateDuel` globals (the
/// connect/team-infra precedent). Faithful 1:1 with original JKA; the `va("...")`
/// formats are inlined into `format!` per this file's print precedent, and the
/// NULL `mins`/`maxs` of the C `trap_Trace` are passed as [`vec3_origin`].
///
/// # Safety
/// `ent` must be a valid in-use player entity with a non-null `client`; `g_entities`
/// must be a valid module static of at least `MAX_CLIENTS` entries.
pub unsafe fn Cmd_EngageDuel_f(ent: *mut gentity_t) {
    if (*addr_of!(g_privateDuel)).integer == 0 {
        return;
    }

    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        //rather pointless in this mode..
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NODUEL_GAMETYPE".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    //if (g_gametype.integer >= GT_TEAM && g_gametype.integer != GT_SIEGE)
    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        //no private dueling in team modes
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NODUEL_GAMETYPE".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    if (*(*ent).client).ps.duelTime >= (*addr_of!(level)).time {
        return;
    }

    if (*(*ent).client).ps.weapon != WP_SABER as c_int {
        return;
    }

    /*
    if (!ent->client->ps.saberHolstered)
    { //must have saber holstered at the start of the duel
        return;
    }
    */
    //NOTE: No longer doing this..

    if (*(*ent).client).ps.saberInFlight != QFALSE {
        return;
    }

    if (*(*ent).client).ps.duelInProgress != QFALSE {
        return;
    }

    //New: Don't let a player duel if he just did and hasn't waited 10 seconds yet (note: If someone challenges him, his duel timer will reset so he can accept)
    if (*(*ent).client).ps.fd.privateDuelTime > (*addr_of!(level)).time {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"CANTDUEL_JUSTDID".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    if G_OtherPlayersDueling() != QFALSE {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"CANTDUEL_BUSY".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    let mut forward: vec3_t = [0.0; 3];
    AngleVectors(
        &(*(*ent).client).ps.viewangles,
        Some(&mut forward),
        None,
        None,
    );

    let mut fwd_org: vec3_t = [0.0; 3];
    fwd_org[0] = (*(*ent).client).ps.origin[0] + forward[0] * 256.0;
    fwd_org[1] = (*(*ent).client).ps.origin[1] + forward[1] * 256.0;
    fwd_org[2] = ((*(*ent).client).ps.origin[2] + (*(*ent).client).ps.viewheight as f32)
        + forward[2] * 256.0;

    let tr = trap::Trace(
        &(*(*ent).client).ps.origin,
        &vec3_origin,
        &vec3_origin,
        &fwd_org,
        (*ent).s.number,
        MASK_PLAYERSOLID,
    );

    if tr.fraction != 1.0 && (tr.entityNum as c_int) < MAX_CLIENTS as c_int {
        let challenged =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(tr.entityNum as isize);

        if challenged.is_null()
            || (*challenged).client.is_null()
            || (*challenged).inuse == QFALSE
            || (*challenged).health < 1
            || (*(*challenged).client).ps.stats[STAT_HEALTH as usize] < 1
            || (*(*challenged).client).ps.weapon != WP_SABER as c_int
            || (*(*challenged).client).ps.duelInProgress != QFALSE
            || (*(*challenged).client).ps.saberInFlight != QFALSE
        {
            return;
        }

        if (*addr_of!(g_gametype)).integer >= GT_TEAM && OnSameTeam(ent, challenged) != QFALSE {
            return;
        }

        if (*(*challenged).client).ps.duelIndex == (*ent).s.number
            && (*(*challenged).client).ps.duelTime >= (*addr_of!(level)).time
        {
            trap::SendServerCommand(
                /*challenged-g_entities*/ -1,
                &format!(
                    "print \"{} {} {}!\n\"",
                    Sz((*(*challenged).client).pers.netname.as_ptr()),
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"PLDUELACCEPT".as_ptr() as *mut c_char,
                    )),
                    Sz((*(*ent).client).pers.netname.as_ptr()),
                ),
            );

            (*(*ent).client).ps.duelInProgress = QTRUE;
            (*(*challenged).client).ps.duelInProgress = QTRUE;

            (*(*ent).client).ps.duelTime = (*addr_of!(level)).time + 2000;
            (*(*challenged).client).ps.duelTime = (*addr_of!(level)).time + 2000;

            G_AddEvent(ent, EV_PRIVATE_DUEL, 1);
            G_AddEvent(challenged, EV_PRIVATE_DUEL, 1);

            //Holster their sabers now, until the duel starts (then they'll get auto-turned on to look cool)

            if (*(*ent).client).ps.saberHolstered == 0 {
                if (*(*ent).client).saber[0].soundOff != 0 {
                    G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[0].soundOff);
                }
                if (*(*ent).client).saber[1].soundOff != 0
                    && (*(*ent).client).saber[1].model[0] != 0
                {
                    G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[1].soundOff);
                }
                (*(*ent).client).ps.weaponTime = 400;
                (*(*ent).client).ps.saberHolstered = 2;
            }
            if (*(*challenged).client).ps.saberHolstered == 0 {
                if (*(*challenged).client).saber[0].soundOff != 0 {
                    G_Sound(
                        challenged,
                        CHAN_AUTO,
                        (*(*challenged).client).saber[0].soundOff,
                    );
                }
                if (*(*challenged).client).saber[1].soundOff != 0
                    && (*(*challenged).client).saber[1].model[0] != 0
                {
                    G_Sound(
                        challenged,
                        CHAN_AUTO,
                        (*(*challenged).client).saber[1].soundOff,
                    );
                }
                (*(*challenged).client).ps.weaponTime = 400;
                (*(*challenged).client).ps.saberHolstered = 2;
            }
        } else {
            //Print the message that a player has been challenged in private, only announce the actual duel initiation in private
            trap::SendServerCommand(
                challenged.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    as c_int,
                &format!(
                    "cp \"{} {}\n\"",
                    Sz((*(*ent).client).pers.netname.as_ptr()),
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"PLDUELCHALLENGE".as_ptr() as *mut c_char,
                    )),
                ),
            );
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!(
                    "cp \"{} {}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"PLDUELCHALLENGED".as_ptr() as *mut c_char,
                    )),
                    Sz((*(*challenged).client).pers.netname.as_ptr()),
                ),
            );
        }

        (*(*challenged).client).ps.fd.privateDuelTime = 0; //reset the timer in case this player just got out of a duel. He should still be able to accept the challenge.

        (*(*ent).client).ps.forceHandExtend = HANDEXTEND_DUELCHALLENGE;
        (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1000;

        (*(*ent).client).ps.duelIndex = (*challenged).s.number;
        (*(*ent).client).ps.duelTime = (*addr_of!(level)).time + 5000;
    }
}

/// `void Cmd_DebugSetSaberMove_f( gentity_t *self )` (g_cmds.c:2924, `#ifndef
/// FINAL_BUILD`). Developer command: force `self`'s `ps.saberMove` to the integer in
/// arg 1 (clamped below [`LS_MOVE_MAX`]), flag the block as
/// [`BLOCKED_BOUNCE_MOVE`], then `Com_Printf` the animation name the chosen move maps
/// to via [`saberMoveData`]→[`animTable`]. No-ops with fewer than 2 args or an empty
/// arg. Built as the live path per the non-`FINAL_BUILD` retail-module precedent
/// (`bg_saberLoad`).
///
/// No oracle: drives `trap_Argc`/`trap_Argv`/`Com_Printf` and the `saberMoveData`/
/// `animTable` globals. Faithful 1:1 with original JKA; [`trap::Argv`]'s owned
/// `String` is parsed through libc [`atoi`] (matching the C buffer) and the empty
/// check mirrors the C `if (!arg[0])`.
///
/// # Safety
/// `self_` must be a valid player entity with a non-null `client`; `saberMoveData`/
/// `animTable` must be valid module statics.
pub unsafe fn Cmd_DebugSetSaberMove_f(self_: *mut gentity_t) {
    let arg_num = trap::Argc();

    if arg_num < 2 {
        return;
    }

    let arg = trap::Argv(1);

    if arg.is_empty() {
        return;
    }

    let cbuf = std::ffi::CString::new(arg).unwrap_or_default();
    (*(*self_).client).ps.saberMove = atoi(cbuf.as_ptr());
    (*(*self_).client).ps.saberBlocked = BLOCKED_BOUNCE_MOVE;

    if (*(*self_).client).ps.saberMove >= LS_MOVE_MAX {
        (*(*self_).client).ps.saberMove = LS_MOVE_MAX - 1;
    }

    let table = &*addr_of!(animTable);
    let smd = &*addr_of!(saberMoveData);
    Com_Printf(&format!(
        "Anim for move: {}\n",
        core::ffi::CStr::from_ptr(
            table[smd[(*(*self_).client).ps.saberMove as usize].animToUse as usize].name
        )
        .to_string_lossy()
    ));
}

/// `void Cmd_DebugSetBodyAnim_f( gentity_t *self, int flags )` (g_cmds.c:2952,
/// `#ifndef FINAL_BUILD`). Developer command: look up arg 1 (case-insensitively) in
/// [`animTable`]; on a hit drive `self`'s torso+legs to that animation index via
/// [`G_SetAnim`] (`SETANIM_BOTH`, the caller's `flags`), else `Com_Printf` that the
/// animation does not exist. No-ops with fewer than 2 args or an empty arg. Built as
/// the live path per the non-`FINAL_BUILD` retail-module precedent.
///
/// No oracle: drives `trap_Argc`/`trap_Argv`/`Com_Printf`/`G_SetAnim` and the
/// `animTable` global. Faithful 1:1 with original JKA; [`trap::Argv`]'s owned `String`
/// is compared through [`Q_stricmp`] (matching the C buffer), the empty check mirrors
/// the C `if (!arg[0])`, and the C `NULL` `usercmd_t*` becomes a null pointer.
///
/// # Safety
/// `self_` must be a valid player entity with a non-null `client`; `animTable` must be
/// a valid module static.
pub unsafe fn Cmd_DebugSetBodyAnim_f(self_: *mut gentity_t, flags: c_int) {
    let arg_num = trap::Argc();

    if arg_num < 2 {
        return;
    }

    let arg = trap::Argv(1);

    if arg.is_empty() {
        return;
    }

    let cbuf = std::ffi::CString::new(arg.clone()).unwrap_or_default();
    let table = &*addr_of!(animTable);
    let mut i: c_int = 0;
    while i < MAX_ANIMATIONS {
        if Q_stricmp(cbuf.as_ptr(), table[i as usize].name) == 0 {
            break;
        }
        i += 1;
    }

    if i == MAX_ANIMATIONS {
        Com_Printf(&format!("Animation '{}' does not exist\n", arg));
        return;
    }

    G_SetAnim(self_, core::ptr::null_mut(), SETANIM_BOTH, i, flags, 0);

    Com_Printf(&format!("Set body anim to {}\n", arg));
}

/// `static void Cmd_VoiceCommand_f( gentity_t *ent )` (g_cmds.c:1699). The siege
/// voice-command (`voicechat`) handler — in team gametypes only, a non-spectator
/// player names a custom siege sound (arg 1); if `*<arg>` is in the validated
/// [`bg_customSiegeSoundNames`] whitelist it spawns a broadcast [`EV_VOICECMD_SOUND`]
/// temp-entity carrying the player's number and the sound index. No-ops below
/// [`GT_TEAM`], with fewer than 2 args, for spectators / `tempSpectate`, on an
/// already-`*`-prefixed arg, or on an unrecognised sound.
///
/// No oracle: drives `trap_Argc`/`trap_Argv`/`trap_SendServerCommand`/`G_TempEntity`/
/// `G_SoundIndex` plus the `bg_customSiegeSoundNames`/`level` globals (the
/// connect/team-infra precedent). Faithful 1:1 with original JKA; `va("*%s", arg)` is
/// inlined into `format!` and the whitelist name is fed to [`G_SoundIndex`] via
/// `CStr::to_string_lossy` (the established `*const c_char`→`&str` adaptation).
///
/// # Safety
/// `ent` must be a valid player entity with a non-null `client`; `bg_customSiegeSoundNames`
/// must be a valid module static and `level` initialized.
pub unsafe fn Cmd_VoiceCommand_f(ent: *mut gentity_t) {
    if (*addr_of!(g_gametype)).integer < GT_TEAM {
        return;
    }

    if trap::Argc() < 2 {
        return;
    }

    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR
        || (*(*ent).client).tempSpectate >= (*addr_of!(level)).time
    {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOVOICECHATASSPEC".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    let arg = trap::Argv(1);

    if arg.as_bytes().first() == Some(&b'*') {
        //hmm.. don't expect a * to be prepended already. maybe someone is trying to be sneaky.
        return;
    }

    let s = format!("*{}", arg);
    let s_cbuf = std::ffi::CString::new(s).unwrap_or_default();

    //now, make sure it's a valid sound to be playing like this.. so people can't go around
    //screaming out death sounds or whatever.
    let names = &*addr_of!(bg_customSiegeSoundNames);
    let mut i: usize = 0;
    while i < MAX_CUSTOM_SIEGE_SOUNDS {
        if names[i].is_null() {
            break;
        }
        if Q_stricmp(names[i], s_cbuf.as_ptr()) == 0 {
            //it matches this one, so it's ok
            break;
        }
        i += 1;
    }

    if i == MAX_CUSTOM_SIEGE_SOUNDS || names[i].is_null() {
        //didn't find it in the list
        return;
    }

    let te = G_TempEntity(&vec3_origin, EV_VOICECMD_SOUND);
    (*te).s.groundEntityNum = (*ent).s.number;
    (*te).s.eventParm = G_SoundIndex(&core::ffi::CStr::from_ptr(names[i]).to_string_lossy());
    (*te).r.svFlags |= SVF_BROADCAST;
}

/*
==================
G_Say
==================
*/

/// `void G_Say( gentity_t *ent, gentity_t *target, int mode, const char *chatText )`
/// (g_cmds.c:1565). Format and deliver one chat message from `ent`: build the colored
/// speaker `name` prefix per `mode` (all / team / tell), append the located form when
/// in a team game with a known location, log the line, copy the (length-capped)
/// `chatText`, then deliver — to a single `target` if given, else to every client via
/// [`G_SayTo`] (echoing to the dedicated-server console first).
///
/// No oracle: side-effecting chat formatting/dispatch over `trap`/`G_LogPrintf` and
/// the `level`/`g_entities`/`g_gametype`/`g_dedicated` globals (the chat/team-infra
/// precedent). Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `target` may be null (broadcast) or a valid
/// `g_entities[]` slot with a non-null client; `chatText` must be a NUL-terminated C
/// string.
pub unsafe fn G_Say(
    ent: *mut gentity_t,
    target: *mut gentity_t,
    mut mode: c_int,
    chatText: *const c_char,
) {
    let color: c_int;
    let mut name = [0 as c_char; 64];
    // don't let text be too long for malicious reasons
    let mut text = [0 as c_char; MAX_SAY_TEXT];
    let mut location = [0 as c_char; 64];
    let mut locMsg: *mut c_char = core::ptr::null_mut();

    if (*addr_of!(g_gametype)).integer < GT_TEAM && mode == SAY_TEAM {
        mode = SAY_ALL;
    }

    match mode {
        SAY_TEAM => {
            G_LogPrintf(&format!(
                "sayteam: {}: {}\n",
                Sz((*(*ent).client).pers.netname.as_ptr()),
                Sz(chatText),
            ));
            if Team_GetLocationMsg(ent, location.as_mut_ptr(), location.len() as c_int) != QFALSE {
                Com_sprintf(
                    name.as_mut_ptr(),
                    name.len() as c_int,
                    format_args!(
                        "{EC}({}{}{}{EC}){EC}: ",
                        Sz((*(*ent).client).pers.netname.as_ptr()),
                        Q_COLOR_ESCAPE as u8 as char,
                        COLOR_WHITE as u8 as char,
                    ),
                );
                locMsg = location.as_mut_ptr();
            } else {
                Com_sprintf(
                    name.as_mut_ptr(),
                    name.len() as c_int,
                    format_args!(
                        "{EC}({}{}{}{EC}){EC}: ",
                        Sz((*(*ent).client).pers.netname.as_ptr()),
                        Q_COLOR_ESCAPE as u8 as char,
                        COLOR_WHITE as u8 as char,
                    ),
                );
            }
            color = COLOR_CYAN as c_int;
        }
        SAY_TELL => {
            if !target.is_null()
                && (*addr_of!(g_gametype)).integer >= GT_TEAM
                && (*(*target).client).sess.sessionTeam == (*(*ent).client).sess.sessionTeam
                && Team_GetLocationMsg(ent, location.as_mut_ptr(), location.len() as c_int)
                    != QFALSE
            {
                Com_sprintf(
                    name.as_mut_ptr(),
                    name.len() as c_int,
                    format_args!(
                        "{EC}[{}{}{}{EC}]{EC}: ",
                        Sz((*(*ent).client).pers.netname.as_ptr()),
                        Q_COLOR_ESCAPE as u8 as char,
                        COLOR_WHITE as u8 as char,
                    ),
                );
                locMsg = location.as_mut_ptr();
            } else {
                Com_sprintf(
                    name.as_mut_ptr(),
                    name.len() as c_int,
                    format_args!(
                        "{EC}[{}{}{}{EC}]{EC}: ",
                        Sz((*(*ent).client).pers.netname.as_ptr()),
                        Q_COLOR_ESCAPE as u8 as char,
                        COLOR_WHITE as u8 as char,
                    ),
                );
            }
            color = COLOR_MAGENTA as c_int;
        }
        // default | SAY_ALL
        _ => {
            G_LogPrintf(&format!(
                "say: {}: {}\n",
                Sz((*(*ent).client).pers.netname.as_ptr()),
                Sz(chatText),
            ));
            Com_sprintf(
                name.as_mut_ptr(),
                name.len() as c_int,
                format_args!(
                    "{}{}{}{EC}: ",
                    Sz((*(*ent).client).pers.netname.as_ptr()),
                    Q_COLOR_ESCAPE as u8 as char,
                    COLOR_WHITE as u8 as char,
                ),
            );
            color = COLOR_GREEN as c_int;
        }
    }

    Q_strncpyz(text.as_mut_ptr(), chatText, text.len() as c_int);

    if !target.is_null() {
        G_SayTo(
            ent,
            target,
            mode,
            color,
            name.as_ptr(),
            text.as_ptr(),
            locMsg,
        );
        return;
    }

    // echo the text to the console
    if (*addr_of!(g_dedicated)).integer != 0 {
        G_Printf(&format!("{}{}\n", Sz(name.as_ptr()), Sz(text.as_ptr())));
    }

    // send it to all the apropriate clients
    let mut j = 0;
    while j < (*addr_of!(level)).maxclients {
        let other = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(j as usize);
        G_SayTo(
            ent,
            other,
            mode,
            color,
            name.as_ptr(),
            text.as_ptr(),
            locMsg,
        );
        j += 1;
    }
}

/// `static void G_SayTo( gentity_t *ent, gentity_t *other, int mode, int color,
/// const char *name, const char *message, char *locMsg )` (g_cmds.c:1513).
/// Deliver one chat line from `ent` to a single recipient `other`.
///
/// Bails on any recipient that is absent / not in use / has no client / is not
/// fully `CON_CONNECTED`, on a `SAY_TEAM` message to someone not [`OnSameTeam`],
/// and on a Siege temp-spectator trying to reach an in-game player. The
/// tournament-mute block is `#if 0`'d out in the C ("They've requested I take
/// this out.") and is preserved as a comment here. Otherwise it emits the
/// reliable per-client server command — the located `ltchat`/`lchat` form when
/// `locMsg` is set, else the plain `tchat`/`chat` form with the `^N` color code.
///
/// No oracle: pure recipient-filter control flow ending in `trap_SendServerCommand`
/// over the `level`/`g_entities`/`g_gametype` globals (the chat/team-infra
/// precedent). Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent`/`other` must be valid `g_entities[]` slots (`other` is null-checked);
/// `name`/`message`/`locMsg` must be null or NUL-terminated C strings.
pub unsafe fn G_SayTo(
    ent: *mut gentity_t,
    other: *mut gentity_t,
    mode: c_int,
    color: c_int,
    name: *const c_char,
    message: *const c_char,
    locMsg: *mut c_char,
) {
    if other.is_null() {
        return;
    }
    if (*other).inuse == 0 {
        return;
    }
    if (*other).client.is_null() {
        return;
    }
    if (*(*other).client).pers.connected != CON_CONNECTED {
        return;
    }
    if mode == SAY_TEAM && OnSameTeam(ent, other) == QFALSE {
        return;
    }
    /*
    // no chatting to players in tournements
    if ( (g_gametype.integer == GT_DUEL || g_gametype.integer == GT_POWERDUEL)
        && other->client->sess.sessionTeam == TEAM_FREE
        && ent->client->sess.sessionTeam != TEAM_FREE ) {
        //Hmm, maybe some option to do so if allowed?  Or at least in developer mode...
        return;
    }
    */
    //They've requested I take this out.

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && !(*ent).client.is_null()
        && ((*(*ent).client).tempSpectate >= (*addr_of!(level)).time
            || (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR)
        && (*(*other).client).sess.sessionTeam != TEAM_SPECTATOR
        && (*(*other).client).tempSpectate < (*addr_of!(level)).time
    {
        //siege temp spectators should not communicate to ingame players
        return;
    }

    if !locMsg.is_null() {
        trap::SendServerCommand(
            other.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "{} \"{}\" \"{}\" \"{}\" \"{}\"",
                if mode == SAY_TEAM { "ltchat" } else { "lchat" },
                Sz(name),
                Sz(locMsg as *const c_char),
                color as u8 as char,
                Sz(message),
            ),
        );
    } else {
        trap::SendServerCommand(
            other.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "{} \"{}{}{}{}\"",
                if mode == SAY_TEAM { "tchat" } else { "chat" },
                Sz(name),
                Q_COLOR_ESCAPE as u8 as char,
                color as u8 as char,
                Sz(message),
            ),
        );
    }
}

/// `static void Cmd_Say_f( gentity_t *ent, int mode, qboolean arg0 )` (g_cmds.c:1642).
/// The `say`/`say_team` handler: gathers the chat text and forwards it to [`G_Say`].
///
/// Bails when there is no chat payload (fewer than 2 args and not invoked via `arg0`).
/// `arg0` selects whether to include argv[0] (`ConcatArgs(0)`) or skip it
/// (`ConcatArgs(1)`) — the engine passes `arg0` true for the bare client message
/// form. File-local `static` in the C, so ported as a private `fn`.
///
/// No oracle: [`ConcatArgs`] reads the trap arg globals and [`G_Say`] is a
/// side-effecting monolith. Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent`/`ent->client` must be valid; the trap arg state must be initialized.
#[allow(dead_code)] // dispatched by ClientCommand (not yet ported)
unsafe fn Cmd_Say_f(ent: *mut gentity_t, mode: c_int, arg0: qboolean) {
    let p: *mut c_char;

    if trap::Argc() < 2 && arg0 == QFALSE {
        return;
    }

    if arg0 != QFALSE {
        p = ConcatArgs(0);
    } else {
        p = ConcatArgs(1);
    }

    G_Say(ent, core::ptr::null_mut(), mode, p);
}

/// `static void Cmd_Tell_f( gentity_t *ent )` (g_cmds.c:1666). The `tell` handler:
/// send a private `SAY_TELL` chat line to one target client.
///
/// Bails without a target/message, parses arg 1 as the target client number
/// (bounds-checked against `level.maxclients`), validates the target slot
/// (`inuse`/`client`), logs a `tell:` line, then delivers the message to the target
/// and — unless the target is the sender or a bot — echoes it back to the sender so
/// they see their own tell. File-local `static` in the C, so ported as a private `fn`.
///
/// No oracle: [`ConcatArgs`]/[`G_Say`]/[`G_LogPrintf`] are all side-effecting over
/// the trap-arg/`level`/`g_entities` globals. Faithful 1:1 with original JKA.
/// [`trap::Argv`]'s owned `String` is copied byte-for-byte into the `arg[]` buffer.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_entities` must be initialized.
#[allow(dead_code)] // dispatched by ClientCommand (not yet ported)
unsafe fn Cmd_Tell_f(ent: *mut gentity_t) {
    let target_num: c_int;
    let target: *mut gentity_t;
    let p: *mut c_char;
    let mut arg = [0 as c_char; MAX_TOKEN_CHARS];

    if trap::Argc() < 2 {
        return;
    }

    // trap_Argv( 1, arg, sizeof( arg ) );
    {
        let s = trap::Argv(1);
        let bytes = s.as_bytes();
        let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
        for k in 0..n {
            arg[k] = bytes[k] as c_char;
        }
        arg[n] = 0;
    }
    target_num = atoi(arg.as_ptr());
    if target_num < 0 || target_num >= (*addr_of!(level)).maxclients {
        return;
    }

    target = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(target_num as usize);
    if target.is_null() || (*target).inuse == QFALSE || (*target).client.is_null() {
        return;
    }

    p = ConcatArgs(2);

    G_LogPrintf(&format!(
        "tell: {} to {}: {}\n",
        Sz((*(*ent).client).pers.netname.as_ptr()),
        Sz((*(*target).client).pers.netname.as_ptr()),
        Sz(p),
    ));
    G_Say(ent, target, SAY_TELL, p);
    // don't tell to the player self if it was already directed to this player
    // also don't send the chat back to a bot
    if ent != target && ((*ent).r.svFlags & SVF_BOT) == 0 {
        G_Say(ent, ent, SAY_TELL, p);
    }
}

// ── Guarded stubs ─────────────────────────────────────────────────────────────
// `ClientCommand` dispatches a handful of commands into subsystems whose handlers
// have not been ported yet. Mirroring the proven `ConsoleCommand` guarded-stub
// pattern (cf. g_svcmds.rs `Svcmd_AddBot_f`/`Svcmd_GameMem_f`): a placeholder that
// reproduces the C control flow (the dispatch branch still runs and returns) so the
// keystone if-chain lands intact. Replace each with the real port when the owning
// subsystem lands.

// AcceptBotCommand (the `bot_*` waypoint-editing console commands) is ported in
// ai_wpnav.rs — imported here. It is gated by gBotEdit (returns 0 in normal play),
// so this is inert unless waypoint editing is active.
use crate::codemp::game::ai_wpnav::AcceptBotCommand;

// Bot_SetForcedMovement (the `debugBMove_*` developer commands) is ported in
// ai_main.rs — imported here so the commands actually drive the bot.
use crate::codemp::game::ai_main::Bot_SetForcedMovement;

// saberKnockOutOfHand (the `debugDropSaber` developer command) is ported in
// w_saber.rs — imported here so the saber is actually ejected.
use crate::codemp::game::w_saber::saberKnockOutOfHand;

// G_SetVehDamageFlags (the `debugShipDamage` developer command) is ported in
// g_vehicles.rs — imported here so the command sets real per-surface damage flags.
use crate::codemp::game::g_vehicles::G_SetVehDamageFlags;

/// `void Cmd_ToggleSaber_f( gentity_t *ent )` (g_cmds.c) — holster or ignite the player's
/// saber. Bails out while gripped (and already lit), knocks a thrown saber out of midair,
/// and refuses while the hand is busy, not wielding a saber, mid-duel, or saber-locked.
/// `saberHolstered == 2` is the fully-off state; toggling plays the per-blade on/off sounds.
pub(crate) unsafe fn Cmd_ToggleSaber_f(ent: *mut gentity_t) {
    if (*(*ent).client).ps.fd.forceGripCripple != 0 {
        //if they are being gripped, don't let them unholster their saber
        if (*(*ent).client).ps.saberHolstered != 0 {
            return;
        }
    }

    if (*(*ent).client).ps.saberInFlight != 0 {
        if (*(*ent).client).ps.saberEntityNum != 0 {
            //turn it off in midair
            saberKnockDown(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*ent).client).ps.saberEntityNum as usize),
                ent,
                ent,
            );
        }
        return;
    }

    if (*(*ent).client).ps.forceHandExtend != HANDEXTEND_NONE {
        return;
    }

    if (*(*ent).client).ps.weapon != WP_SABER {
        return;
    }

    //	if (ent->client->ps.duelInProgress && !ent->client->ps.saberHolstered)
    //	{
    //		return;
    //	}

    if (*(*ent).client).ps.duelTime >= (*addr_of!(level)).time {
        return;
    }

    if (*(*ent).client).ps.saberLockTime >= (*addr_of!(level)).time {
        return;
    }

    if !(*ent).client.is_null() && (*(*ent).client).ps.weaponTime < 1 {
        if (*(*ent).client).ps.saberHolstered == 2 {
            (*(*ent).client).ps.saberHolstered = 0;

            if (*(*ent).client).saber[0].soundOn != 0 {
                G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[0].soundOn);
            }
            if (*(*ent).client).saber[1].soundOn != 0 {
                G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[1].soundOn);
            }
        } else {
            (*(*ent).client).ps.saberHolstered = 2;
            if (*(*ent).client).saber[0].soundOff != 0 {
                G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[0].soundOff);
            }
            if (*(*ent).client).saber[1].soundOff != 0 && (*(*ent).client).saber[1].model[0] != 0 {
                G_Sound(ent, CHAN_AUTO, (*(*ent).client).saber[1].soundOff);
            }
            //prevent anything from being done for 400ms after holster
            (*(*ent).client).ps.weaponTime = 400;
        }
    }
}

/// `static const char *gameNames[]` (g_cmds.c:1798) — display names for the
/// gametypes, indexed by `g_gametype` value for the `g_gametype` vote-display string.
static GAME_NAMES: [&CStr; 10] = [
    c"Free For All",
    c"Holocron FFA",
    c"Jedi Master",
    c"Duel",
    c"Power Duel",
    c"Single Player",
    c"Team FFA",
    c"Siege",
    c"Capture the Flag",
    c"Capture the Ysalamiri",
];

/// `void Cmd_CallVote_f( gentity_t *ent )` (g_cmds.c:1921). Validates and starts a
/// server-wide vote: checks `g_allowVote`, that no vote is already in progress, the
/// caller's `voteCount` cap, and (outside duel) the spectator ban, then parses the
/// vote command (`map_restart`/`nextmap`/`map`/`g_gametype`/`kick`/`clientkick`/
/// `g_doWarmup`/`timelimit`/`fraglimit`), builds `level.voteString`/`voteDisplayString`,
/// announces it to everyone, and publishes the `CS_VOTE_*` configstrings.
///
/// No oracle: pure trap/`level`-global plumbing (`trap_SendServerCommand` /
/// `trap_Argv` / `trap_Argc` / `trap_SendConsoleCommand` / `trap_Cvar_*` /
/// `trap_SetConfigstring`) plus `G_GetStringEdString` and the already-ported
/// `G_DoesMapSupportGametype` / `G_GetArenaInfoByMap` / `G_ClientNumberFromName` /
/// `G_ClientNumberFromStrippedName` / `SiegeClearSwitchData` helpers — the same
/// no-oracle class as the sibling [`Cmd_CallTeamVote_f`]. The C `arg1`/`arg2`
/// `[MAX_STRING_TOKENS]` token buffers are kept as raw `c_char` buffers operated on
/// with the same C helpers (`Q_stricmp`/`strchr`/`atoi`/`Com_sprintf`) for
/// byte-for-byte parity.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_entities` must be initialized.
unsafe fn Cmd_CallVote_f(ent: *mut gentity_t) {
    // `EXEC_APPEND` (q_shared.h) — the `EXEC_*` enum's third value, so `2`.
    const EXEC_APPEND: c_int = 2;

    let mut arg1 = [0 as c_char; MAX_STRING_TOKENS];
    let mut arg2 = [0 as c_char; MAX_STRING_TOKENS];
    //	int		n = 0;
    //	char*	type = NULL;
    let mut map_name: *const c_char = core::ptr::null();
    let arena_info: *const c_char;

    let ent_num = ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int;

    if (*addr_of!(g_allowVote)).integer == 0 {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOVOTE".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    if (*addr_of!(level)).voteTime != 0
        || (*addr_of!(level)).voteExecuteTime >= (*addr_of!(level)).time
    {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"VOTEINPROGRESS".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }
    if (*(*ent).client).pers.voteCount >= MAX_VOTE_COUNT {
        trap::SendServerCommand(
            ent_num,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"MAXVOTES".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    if (*addr_of!(g_gametype)).integer != GT_DUEL && (*addr_of!(g_gametype)).integer != GT_POWERDUEL
    {
        if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
            trap::SendServerCommand(
                ent_num,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"NOSPECVOTE".as_ptr() as *mut c_char,
                    )),
                ),
            );
            return;
        }
    }

    // make sure it is a valid command to vote on
    // trap_Argv( 1, arg1, sizeof( arg1 ) );
    {
        let a = trap::Argv(1);
        let bytes = a.as_bytes();
        let n = bytes.len().min(arg1.len() - 1);
        for k in 0..n {
            arg1[k] = bytes[k] as c_char;
        }
        arg1[n] = 0;
    }
    // trap_Argv( 2, arg2, sizeof( arg2 ) );
    {
        let a = trap::Argv(2);
        let bytes = a.as_bytes();
        let n = bytes.len().min(arg2.len() - 1);
        for k in 0..n {
            arg2[k] = bytes[k] as c_char;
        }
        arg2[n] = 0;
    }

    if !strchr(arg1.as_ptr(), b';' as c_int).is_null()
        || !strchr(arg2.as_ptr(), b';' as c_int).is_null()
    {
        trap::SendServerCommand(ent_num, "print \"Invalid vote string.\n\"");
        return;
    }

    if Q_stricmp(arg1.as_ptr(), c"map_restart".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"nextmap".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"map".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"g_gametype".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"kick".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"clientkick".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"g_doWarmup".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"timelimit".as_ptr()) == 0 {
    } else if Q_stricmp(arg1.as_ptr(), c"fraglimit".as_ptr()) == 0 {
    } else {
        trap::SendServerCommand(ent_num, "print \"Invalid vote string.\n\"");
        trap::SendServerCommand(ent_num, "print \"Vote commands are: map_restart, nextmap, map <mapname>, g_gametype <n>, kick <player>, clientkick <clientnum>, g_doWarmup, timelimit <time>, fraglimit <frags>.\n\"");
        return;
    }

    // if there is still a vote to be executed
    if (*addr_of!(level)).voteExecuteTime != 0 {
        (*addr_of_mut!(level)).voteExecuteTime = 0;
        trap::SendConsoleCommand(
            EXEC_APPEND,
            &format!("{}\n", Sz((*addr_of!(level)).voteString.as_ptr())),
        );
    }

    // special case for g_gametype, check for bad values
    if Q_stricmp(arg1.as_ptr(), c"g_gametype".as_ptr()) == 0 {
        let i = atoi(arg2.as_ptr());
        if i == GT_SINGLE_PLAYER || i < GT_FFA || i >= GT_MAX_GAME_TYPE {
            trap::SendServerCommand(ent_num, "print \"Invalid gametype.\n\"");
            return;
        }

        (*addr_of_mut!(level)).votingGametype = QTRUE;
        (*addr_of_mut!(level)).votingGametypeTo = i;

        Com_sprintf(
            (*addr_of_mut!(level)).voteString.as_mut_ptr(),
            (*addr_of!(level)).voteString.len() as c_int,
            format_args!("{} {}", Sz(arg1.as_ptr()), i),
        );
        Com_sprintf(
            (*addr_of_mut!(level)).voteDisplayString.as_mut_ptr(),
            (*addr_of!(level)).voteDisplayString.len() as c_int,
            format_args!(
                "{} {}",
                Sz(arg1.as_ptr()),
                Sz(GAME_NAMES[i as usize].as_ptr())
            ),
        );
    } else if Q_stricmp(arg1.as_ptr(), c"map".as_ptr()) == 0 {
        // special case for map changes, we want to reset the nextmap setting
        // this allows a player to change maps, but not upset the map rotation
        let mut s = [0 as c_char; MAX_STRING_CHARS];

        if G_DoesMapSupportGametype(arg2.as_ptr(), trap::Cvar_VariableIntegerValue("g_gametype"))
            == QFALSE
        {
            //trap_SendServerCommand( ent-g_entities, "print \"You can't vote for this map, it isn't supported by the current gametype.\n\"" );
            trap::SendServerCommand(
                ent_num,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"NOVOTE_MAPNOTSUPPORTEDBYGAME".as_ptr() as *mut c_char,
                    )),
                ),
            );
            return;
        }

        trap::Cvar_VariableStringBuffer("nextmap", &mut s);
        if s[0] != 0 {
            Com_sprintf(
                (*addr_of_mut!(level)).voteString.as_mut_ptr(),
                (*addr_of!(level)).voteString.len() as c_int,
                format_args!(
                    "{} {}; set nextmap \"{}\"",
                    Sz(arg1.as_ptr()),
                    Sz(arg2.as_ptr()),
                    Sz(s.as_ptr())
                ),
            );
        } else {
            Com_sprintf(
                (*addr_of_mut!(level)).voteString.as_mut_ptr(),
                (*addr_of!(level)).voteString.len() as c_int,
                format_args!("{} {}", Sz(arg1.as_ptr()), Sz(arg2.as_ptr())),
            );
        }

        arena_info = G_GetArenaInfoByMap(arg2.as_ptr());
        if !arena_info.is_null() {
            map_name = Info_ValueForKey(arena_info, c"longname".as_ptr());
        }

        if map_name.is_null() || *map_name == 0 {
            map_name = c"ERROR".as_ptr();
        }

        Com_sprintf(
            (*addr_of_mut!(level)).voteDisplayString.as_mut_ptr(),
            (*addr_of!(level)).voteDisplayString.len() as c_int,
            format_args!("map {}", Sz(map_name)),
        );
    } else if Q_stricmp(arg1.as_ptr(), c"clientkick".as_ptr()) == 0 {
        let n = atoi(arg2.as_ptr());

        if n < 0 || n >= MAX_CLIENTS as c_int {
            trap::SendServerCommand(ent_num, &format!("print \"invalid client number {n}.\n\""));
            return;
        }

        if (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(n as usize)).client)
            .pers
            .connected
            == CON_DISCONNECTED
        {
            trap::SendServerCommand(
                ent_num,
                &format!("print \"there is no client with the client number {n}.\n\""),
            );
            return;
        }

        Com_sprintf(
            (*addr_of_mut!(level)).voteString.as_mut_ptr(),
            (*addr_of!(level)).voteString.len() as c_int,
            format_args!("{} {}", Sz(arg1.as_ptr()), Sz(arg2.as_ptr())),
        );
        Com_sprintf(
            (*addr_of_mut!(level)).voteDisplayString.as_mut_ptr(),
            (*addr_of!(level)).voteDisplayString.len() as c_int,
            format_args!(
                "kick {}",
                Sz(
                    (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(n as usize))
                        .client)
                        .pers
                        .netname
                        .as_ptr()
                )
            ),
        );
    } else if Q_stricmp(arg1.as_ptr(), c"kick".as_ptr()) == 0 {
        let mut clientid = G_ClientNumberFromName(arg2.as_ptr());

        if clientid == -1 {
            clientid = G_ClientNumberFromStrippedName(arg2.as_ptr());

            if clientid == -1 {
                trap::SendServerCommand(
                    ent_num,
                    &format!(
                        "print \"there is no client named '{}' currently on the server.\n\"",
                        Sz(arg2.as_ptr())
                    ),
                );
                return;
            }
        }

        Com_sprintf(
            (*addr_of_mut!(level)).voteString.as_mut_ptr(),
            (*addr_of!(level)).voteString.len() as c_int,
            format_args!("clientkick {clientid}"),
        );
        Com_sprintf(
            (*addr_of_mut!(level)).voteDisplayString.as_mut_ptr(),
            (*addr_of!(level)).voteDisplayString.len() as c_int,
            format_args!(
                "kick {}",
                Sz(
                    (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(clientid as usize))
                    .client)
                        .pers
                        .netname
                        .as_ptr()
                )
            ),
        );
    } else if Q_stricmp(arg1.as_ptr(), c"nextmap".as_ptr()) == 0 {
        let mut s = [0 as c_char; MAX_STRING_CHARS];

        trap::Cvar_VariableStringBuffer("nextmap", &mut s);
        if s[0] == 0 {
            trap::SendServerCommand(ent_num, "print \"nextmap not set.\n\"");
            return;
        }
        SiegeClearSwitchData();
        Com_sprintf(
            (*addr_of_mut!(level)).voteString.as_mut_ptr(),
            (*addr_of!(level)).voteString.len() as c_int,
            format_args!("vstr nextmap"),
        );
        Com_sprintf(
            (*addr_of_mut!(level)).voteDisplayString.as_mut_ptr(),
            (*addr_of!(level)).voteDisplayString.len() as c_int,
            format_args!("{}", Sz((*addr_of!(level)).voteString.as_ptr())),
        );
    } else {
        Com_sprintf(
            (*addr_of_mut!(level)).voteString.as_mut_ptr(),
            (*addr_of!(level)).voteString.len() as c_int,
            format_args!("{} \"{}\"", Sz(arg1.as_ptr()), Sz(arg2.as_ptr())),
        );
        Com_sprintf(
            (*addr_of_mut!(level)).voteDisplayString.as_mut_ptr(),
            (*addr_of!(level)).voteDisplayString.len() as c_int,
            format_args!("{}", Sz((*addr_of!(level)).voteString.as_ptr())),
        );
    }

    trap::SendServerCommand(
        -1,
        &format!(
            "print \"{}^7 {}\n\"",
            Sz((*(*ent).client).pers.netname.as_ptr()),
            Sz(G_GetStringEdString(
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"PLCALLEDVOTE".as_ptr() as *mut c_char,
            )),
        ),
    );

    // start the voting, the caller autoamtically votes yes
    (*addr_of_mut!(level)).voteTime = (*addr_of!(level)).time;
    (*addr_of_mut!(level)).voteYes = 1;
    (*addr_of_mut!(level)).voteNo = 0;

    let mut i: c_int = 0;
    while i < (*addr_of!(level)).maxclients {
        (*(*addr_of!(level)).clients.add(i as usize)).mGameFlags &= !(PSG_VOTED as c_uint);
        i += 1;
    }
    (*(*ent).client).mGameFlags |= PSG_VOTED as c_uint;

    trap::SetConfigstring(CS_VOTE_TIME, &format!("{}", (*addr_of!(level)).voteTime));
    trap::SetConfigstring(
        CS_VOTE_STRING,
        &Sz((*addr_of!(level)).voteDisplayString.as_ptr()).to_string(),
    );
    trap::SetConfigstring(CS_VOTE_YES, &format!("{}", (*addr_of!(level)).voteYes));
    trap::SetConfigstring(CS_VOTE_NO, &format!("{}", (*addr_of!(level)).voteNo));
    // ABI DEVIATION: retail JKA sets CS_VOTE_CALLER (g_cmds.c:2103) here, but
    // OpenJK removed that configstring entirely. We target the OpenJK config-
    // string ABI (see bg_public.rs), where index 12 is CS_TEAMVOTE_TIME — writing
    // the caller's netname there would corrupt it — so this write is dropped to
    // match OpenJK. See crate/DEVIATIONS.md.
}

/// `void Cmd_ForceChanged_f( gentity_t *ent )` (g_cmds.c:1278). Applies a queued
/// force-power change. For spectators the change applies instantly via
/// [`WP_InitForcePowers`]. Otherwise it prints the localized "FORCEPOWERCHANGED"
/// notice and defers the re-init to the next spawn (`forceDoInit = 1`). In
/// duel/power-duel it then returns without touching teams; otherwise, if a second arg
/// is present it is assumed to be a combo team command from the UI and dispatches to
/// [`Cmd_Team_f`].
///
/// No oracle: entity-state mutation over `gclient_t`/`playerState_t` driving the
/// already-ported [`WP_InitForcePowers`] / [`Cmd_Team_f`] and the
/// `trap_SendServerCommand` trap (the entity-state/anim-infra precedent). Faithful 1:1
/// with original JKA.
///
/// # Safety
/// `ent`/`ent->client` must be valid.
unsafe fn Cmd_ForceChanged_f(ent: *mut gentity_t) {
    let mut fp_ch_str = [0 as c_char; 1024];
    let buf: *const c_char;
    //	Cmd_Kill_f(ent);
    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        //if it's a spec, just make the changes now
        //trap_SendServerCommand( ent-g_entities, va("print \"%s\n\"", G_GetStringEdString("MP_SVGAME", "FORCEAPPLIED")) );
        //No longer print it, as the UI calls this a lot.
        WP_InitForcePowers(ent);
        // goto argCheck;
    } else {
        buf = G_GetStringEdString(
            c"MP_SVGAME".as_ptr() as *mut c_char,
            c"FORCEPOWERCHANGED".as_ptr() as *mut c_char,
        );

        strcpy(fp_ch_str.as_mut_ptr(), buf);

        // S_COLOR_GREEN == "^2"
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!("print \"{}{}\n\n\"", "^2", Sz(fp_ch_str.as_ptr())),
        );

        (*(*ent).client).ps.fd.forceDoInit = 1;
    }

    // argCheck:
    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        //If this is duel, don't even bother changing team in relation to this.
        return;
    }

    if trap::Argc() > 1 {
        let mut arg = [0 as c_char; MAX_TOKEN_CHARS];

        // trap_Argv( 1, arg, sizeof( arg ) );
        {
            let a = trap::Argv(1);
            let bytes = a.as_bytes();
            let n = bytes.len().min(arg.len() - 1);
            for k in 0..n {
                arg[k] = bytes[k] as c_char;
            }
            arg[n] = 0;
        }

        if arg[0] != 0 {
            //if there's an arg, assume it's a combo team command from the UI.
            Cmd_Team_f(ent);
        }
    }
}

/// `void Cmd_SiegeClass_f( gentity_t *ent )` (g_cmds.c:1251). Sets the player's Siege
/// class from command arg 1. Only valid in `GT_SIEGE`. Maps the class name to a team
/// via [`G_TeamForSiegeClass`]; if the player is not already on that team it tries to
/// switch (via [`SetTeam`], with `g_preventTeamBegin` latched), bailing with a
/// "NOCLASSTEAM" notice on failure. It then validates the class for the team with
/// [`BG_SiegeCheckClassLegality`], stores it in the session data, republishes via
/// [`ClientUserinfoChanged`], and — unless temp-spectating — kills the player (to drop
/// flags etc.) and respawns them instantly via [`ClientBegin`] when appropriate,
/// preserving the pre-switch `PERS_SCORE`.
///
/// No oracle: entity-state/team-infra side effects over `gclient_t` driving the
/// already-ported [`G_TeamForSiegeClass`] / [`BG_SiegeCheckClassLegality`] / [`SetTeam`]
/// / [`ClientUserinfoChanged`] / [`ClientBegin`] and the real [`player_die`] port (the
/// entity-state/team-infra precedent). Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent`/`ent->client` must be valid; `level`/`g_entities` must be initialized.
unsafe fn Cmd_SiegeClass_f(ent: *mut gentity_t) {
    let mut class_name = [0 as c_char; 64];
    let team: c_int;
    let pre_score: c_int;
    let mut started_as_spec: qboolean = QFALSE;

    if (*addr_of!(g_gametype)).integer != GT_SIEGE {
        //classes are only valid for this gametype
        return;
    }

    if (*ent).client.is_null() {
        return;
    }

    if trap::Argc() < 1 {
        return;
    }

    if (*(*ent).client).switchClassTime > (*addr_of!(level)).time {
        trap::SendServerCommand(
            ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
            &format!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOCLASSSWITCH".as_ptr() as *mut c_char,
                )),
            ),
        );
        return;
    }

    if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR {
        started_as_spec = QTRUE;
    }

    // trap_Argv( 1, className, sizeof( className ) );
    {
        let a = trap::Argv(1);
        let bytes = a.as_bytes();
        let n = bytes.len().min(class_name.len() - 1);
        for k in 0..n {
            class_name[k] = bytes[k] as c_char;
        }
        class_name[n] = 0;
    }

    team = G_TeamForSiegeClass(class_name.as_ptr());

    if team == 0 {
        //not a valid class name
        return;
    }

    if (*(*ent).client).sess.sessionTeam != team {
        //try changing it then
        g_preventTeamBegin = QTRUE;
        if team == TEAM_RED {
            SetTeam(ent, c"red".as_ptr() as *mut c_char);
        } else if team == TEAM_BLUE {
            SetTeam(ent, c"blue".as_ptr() as *mut c_char);
        }
        g_preventTeamBegin = QFALSE;

        if (*(*ent).client).sess.sessionTeam != team {
            //failed, oh well
            if (*(*ent).client).sess.sessionTeam != TEAM_SPECTATOR
                || (*(*ent).client).sess.siegeDesiredTeam != team
            {
                trap::SendServerCommand(
                    ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        as c_int,
                    &format!(
                        "print \"{}\n\"",
                        Sz(G_GetStringEdString(
                            c"MP_SVGAME".as_ptr() as *mut c_char,
                            c"NOCLASSTEAM".as_ptr() as *mut c_char,
                        )),
                    ),
                );
                return;
            }
        }
    }

    //preserve 'is score
    pre_score = (*(*ent).client).ps.persistant[PERS_SCORE as usize];

    //Make sure the class is valid for the team
    BG_SiegeCheckClassLegality(team, class_name.as_mut_ptr());

    //Set the session data
    strcpy(
        (*(*ent).client).sess.siegeClass.as_mut_ptr(),
        class_name.as_ptr(),
    );

    // get and distribute relevent paramters
    ClientUserinfoChanged((*ent).s.number);

    if (*(*ent).client).tempSpectate < (*addr_of!(level)).time {
        // Kill him (makes sure he loses flags, etc)
        if (*ent).health > 0 && started_as_spec == QFALSE {
            (*ent).flags &= !FL_GODMODE;
            (*ent).health = 0;
            (*(*ent).client).ps.stats[STAT_HEALTH as usize] = (*ent).health;
            player_die(ent, ent, ent, 100000, MOD_SUICIDE);
        }

        if (*(*ent).client).sess.sessionTeam == TEAM_SPECTATOR || started_as_spec == QTRUE {
            //respawn them instantly.
            ClientBegin((*ent).s.number, QFALSE);
        }
    }
    //set it back after we do all the stuff
    (*(*ent).client).ps.persistant[PERS_SCORE as usize] = pre_score;

    (*(*ent).client).switchClassTime = (*addr_of!(level)).time + 5000;
}

/// `qboolean TryGrapple( gentity_t *ent )` (g_cmds.c:3025). Saber/melee grapple
/// initiator: bails if the weapon is busy (`weaponTime > 0`), a force-hand action is in
/// progress (`forceHandExtend != HANDEXTEND_NONE`), the client is already grappling, or
/// the active weapon is neither saber nor melee. With the saber drawn it holsters via
/// [`Cmd_ToggleSaber_f`] first (and bails if it failed to holster), then plays
/// `BOTH_KYLE_GRAB` (`SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD`); if the torso anim
/// took, it extends the hold timers by 500ms, latches `weaponTime` to the torso timer,
/// and returns `qtrue`.
///
/// No oracle: entity-state mutation over `gclient_t`/`playerState_t` driving
/// [`G_SetAnim`] and the local [`Cmd_ToggleSaber_f`] stub (the entity-state/anim-infra
/// precedent). Faithful 1:1 with original JKA.
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a non-null `client`.
pub unsafe fn TryGrapple(ent: *mut gentity_t) -> qboolean {
    let client = (*ent).client;
    if (*client).ps.weaponTime > 0 {
        // weapon busy
        return QFALSE;
    }
    if (*client).ps.forceHandExtend != HANDEXTEND_NONE {
        // force power or knockdown or something
        return QFALSE;
    }
    if (*client).grappleState != 0 {
        // already grappling? but weapontime should be > 0 then..
        return QFALSE;
    }

    if (*client).ps.weapon != WP_SABER as c_int && (*client).ps.weapon != WP_MELEE as c_int {
        return QFALSE;
    }

    if (*client).ps.weapon == WP_SABER as c_int && (*client).ps.saberHolstered == 0 {
        Cmd_ToggleSaber_f(ent);
        if (*client).ps.saberHolstered == 0 {
            // must have saber holstered
            return QFALSE;
        }
    }

    //G_SetAnim(ent, &ent->client->pers.cmd, SETANIM_BOTH, BOTH_KYLE_PA_1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD, 0);
    G_SetAnim(
        ent,
        addr_of_mut!((*client).pers.cmd),
        SETANIM_BOTH,
        BOTH_KYLE_GRAB as c_int,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        0,
    );
    if (*client).ps.torsoAnim == BOTH_KYLE_GRAB as c_int {
        // providing the anim set succeeded..
        (*client).ps.torsoTimer += 500; // make the hand stick out a little longer than it normally would
        if (*client).ps.legsAnim == (*client).ps.torsoAnim {
            (*client).ps.legsTimer = (*client).ps.torsoTimer;
        }
        (*client).ps.weaponTime = (*client).ps.torsoTimer;
        return QTRUE;
    }

    QFALSE
}

/*
=================
ClientCommand
=================
*/
/// `void ClientCommand( int clientNum )` (g_cmds.c:3079). The keystone client-command
/// dispatcher: tokenizes the command word (`trap_Argv(0)`) and routes it to the right
/// `Cmd_*_f` handler (say/tell/score/give/god/team/follow/vote/gc/setviewpos/stats/…),
/// with the intermission-time gate that rejects most commands and the developer-only
/// `debug*`/`debugBMove_*` cheats.
///
/// No oracle: pure `trap`-driven command dispatch over `g_entities`/`level`/
/// `g_gametype` (the entity-state/command precedent). Faithful 1:1 with original JKA,
/// with these conditional-compilation decisions matching the retail PC module build
/// (cf. the `#ifndef FINAL_BUILD` live-path precedent already used for
/// `Cmd_DebugSetSaberMove_f`/`Cmd_DebugSetBodyAnim_f`, and the `#ifdef _DEBUG` omission
/// in `g_active.rs`):
///   - `#ifdef _DEBUG` blocks (`relax`, `holdme`, `limb_break`, `headexplodey`,
///     `debugstupidthing`, `arbitraryprint`, `handcut`, `loveandpeace`) — omitted
///     (debug-build only, not in the shipped module).
///   - `#ifdef VM_MEMALLOC_DEBUG` (`debugTestAlloc`) — omitted (alloc stress test).
///   - `#ifndef FINAL_BUILD` blocks (`debugSetSaberMove`/`debugSetBodyAnim`/
///     `debugDismemberment`/`debugDropSaber`/`debugKnockMeDown`/`debugSaberSwitch`/
///     `debugIK*`/`debugThrow`/`debugShipDamage`) — built live.
/// `AcceptBotCommand`, `Bot_SetForcedMovement`, `saberKnockOutOfHand`, and
/// `G_SetVehDamageFlags` are guarded stubs (see above); `Cmd_CallVote_f`/`Cmd_Vote_f`
/// etc. that funnel to voting are ported handlers in this file.
///
/// # Safety
/// `clientNum` must index a valid `g_entities[]` slot; `level`/`g_gametype` must be
/// initialized.
pub unsafe fn ClientCommand(clientNum: c_int) {
    let ent: *mut gentity_t =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(clientNum as isize);
    if (*ent).client.is_null() {
        return; // not fully in game yet
    }

    // trap_Argv( 0, cmd, sizeof( cmd ) );
    let cmd_s = trap::Argv(0);
    let cmd_c = std::ffi::CString::new(cmd_s.as_str()).unwrap_or_default();
    let cmd = cmd_c.as_ptr();

    //rww - redirect bot commands
    if cmd_s.contains("bot_") && AcceptBotCommand(cmd as *mut c_char, ent) != QFALSE {
        return;
    }
    //end rww

    if Q_stricmp(cmd, c"say".as_ptr()) == 0 {
        Cmd_Say_f(ent, SAY_ALL, QFALSE);
        return;
    }
    if Q_stricmp(cmd, c"say_team".as_ptr()) == 0 {
        if (*addr_of!(g_gametype)).integer < GT_TEAM {
            //not a team game, just refer to regular say.
            Cmd_Say_f(ent, SAY_ALL, QFALSE);
        } else {
            Cmd_Say_f(ent, SAY_TEAM, QFALSE);
        }
        return;
    }
    if Q_stricmp(cmd, c"tell".as_ptr()) == 0 {
        Cmd_Tell_f(ent);
        return;
    }

    if Q_stricmp(cmd, c"voice_cmd".as_ptr()) == 0 {
        Cmd_VoiceCommand_f(ent);
        return;
    }

    if Q_stricmp(cmd, c"score".as_ptr()) == 0 {
        Cmd_Score_f(ent);
        return;
    }

    // ignore all other commands when at intermission
    if (*addr_of!(level)).intermissiontime != 0 {
        let mut giveError: qboolean = QFALSE;
        //rwwFIXMEFIXME: This is terrible, write it differently

        if Q_stricmp(cmd, c"give".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"giveother".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"god".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"notarget".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"noclip".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"kill".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"teamtask".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"levelshot".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"follow".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"follownext".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"followprev".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"team".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"duelteam".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"siegeclass".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"forcechanged".as_ptr()) == 0 {
            //special case: still update force change
            Cmd_ForceChanged_f(ent);
            return;
        } else if Q_stricmp(cmd, c"where".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"callvote".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"vote".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"callteamvote".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"teamvote".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"gc".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"setviewpos".as_ptr()) == 0 {
            giveError = QTRUE;
        } else if Q_stricmp(cmd, c"stats".as_ptr()) == 0 {
            giveError = QTRUE;
        }

        if giveError != QFALSE {
            trap::SendServerCommand(
                clientNum,
                &format!(
                    "print \"{} ({}) \n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"CANNOT_TASK_INTERMISSION".as_ptr() as *mut c_char,
                    )),
                    Sz(cmd),
                ),
            );
        } else {
            Cmd_Say_f(ent, QFALSE, QTRUE);
        }
        return;
    }

    if Q_stricmp(cmd, c"give".as_ptr()) == 0 {
        Cmd_Give_f(ent, 0);
    } else if Q_stricmp(cmd, c"giveother".as_ptr()) == 0 {
        //for debugging pretty much
        Cmd_Give_f(ent, 1);
    } else if Q_stricmp(cmd, c"t_use".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        //debug use map object
        if trap::Argc() > 1 {
            let s_arg = trap::Argv(1);
            let s_arg_c = std::ffi::CString::new(s_arg).unwrap_or_default();

            let mut targ = G_Find(
                core::ptr::null_mut(),
                offset_of!(gentity_t, targetname),
                s_arg_c.as_ptr(),
            );

            while !targ.is_null() {
                if let Some(use_fn) = (*targ).r#use {
                    use_fn(targ, ent, ent);
                }
                targ = G_Find(targ, offset_of!(gentity_t, targetname), s_arg_c.as_ptr());
            }
        }
    } else if Q_stricmp(cmd, c"god".as_ptr()) == 0 {
        Cmd_God_f(ent);
    } else if Q_stricmp(cmd, c"notarget".as_ptr()) == 0 {
        Cmd_Notarget_f(ent);
    } else if Q_stricmp(cmd, c"noclip".as_ptr()) == 0 {
        Cmd_Noclip_f(ent);
    } else if Q_stricmp(cmd, c"NPC".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        Cmd_NPC_f(ent);
    } else if Q_stricmp(cmd, c"kill".as_ptr()) == 0 {
        Cmd_Kill_f(ent);
    } else if Q_stricmp(cmd, c"teamtask".as_ptr()) == 0 {
        Cmd_TeamTask_f(ent);
    } else if Q_stricmp(cmd, c"levelshot".as_ptr()) == 0 {
        Cmd_LevelShot_f(ent);
    } else if Q_stricmp(cmd, c"follow".as_ptr()) == 0 {
        Cmd_Follow_f(ent);
    } else if Q_stricmp(cmd, c"follownext".as_ptr()) == 0 {
        Cmd_FollowCycle_f(ent, 1);
    } else if Q_stricmp(cmd, c"followprev".as_ptr()) == 0 {
        Cmd_FollowCycle_f(ent, -1);
    } else if Q_stricmp(cmd, c"team".as_ptr()) == 0 {
        Cmd_Team_f(ent);
    } else if Q_stricmp(cmd, c"duelteam".as_ptr()) == 0 {
        Cmd_DuelTeam_f(ent);
    } else if Q_stricmp(cmd, c"siegeclass".as_ptr()) == 0 {
        Cmd_SiegeClass_f(ent);
    } else if Q_stricmp(cmd, c"forcechanged".as_ptr()) == 0 {
        Cmd_ForceChanged_f(ent);
    } else if Q_stricmp(cmd, c"where".as_ptr()) == 0 {
        Cmd_Where_f(ent);
    } else if Q_stricmp(cmd, c"callvote".as_ptr()) == 0 {
        Cmd_CallVote_f(ent);
    } else if Q_stricmp(cmd, c"vote".as_ptr()) == 0 {
        Cmd_Vote_f(ent);
    } else if Q_stricmp(cmd, c"callteamvote".as_ptr()) == 0 {
        Cmd_CallTeamVote_f(ent);
    } else if Q_stricmp(cmd, c"teamvote".as_ptr()) == 0 {
        Cmd_TeamVote_f(ent);
    } else if Q_stricmp(cmd, c"gc".as_ptr()) == 0 {
        Cmd_GameCommand_f(ent);
    } else if Q_stricmp(cmd, c"setviewpos".as_ptr()) == 0 {
        Cmd_SetViewpos_f(ent);
    } else if Q_stricmp(cmd, c"stats".as_ptr()) == 0 {
        Cmd_Stats_f(ent);
    }
    /*
    else if (Q_stricmp (cmd, "kylesmash") == 0)
    {
        TryGrapple(ent);
    }
    */
    //for convenient powerduel testing in release
    else if Q_stricmp(cmd, c"killother".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        if trap::Argc() > 1 {
            let s_arg = trap::Argv(1);
            let s_arg_c = std::ffi::CString::new(s_arg).unwrap_or_default();

            let entNum = G_ClientNumFromNetname(s_arg_c.as_ptr() as *mut c_char);

            if entNum >= 0 && entNum < MAX_GENTITIES as c_int {
                let kEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .offset(entNum as isize);

                if (*kEnt).inuse != QFALSE && !(*kEnt).client.is_null() {
                    (*kEnt).flags &= !FL_GODMODE;
                    (*(*kEnt).client).ps.stats[STAT_HEALTH as usize] = -999;
                    (*kEnt).health = -999;
                    player_die(kEnt, kEnt, kEnt, 100000, MOD_SUICIDE);
                }
            }
        }
    }
    // #ifdef _DEBUG block (relax/holdme/limb_break/headexplodey/debugstupidthing/
    // arbitraryprint/handcut/loveandpeace) omitted — debug-build only.
    else if Q_stricmp(cmd, c"thedestroyer".as_ptr()) == 0
        && CheatsOk(ent) != QFALSE
        && !ent.is_null()
        && !(*ent).client.is_null()
        && (*(*ent).client).ps.saberHolstered != 0
        && (*(*ent).client).ps.weapon == WP_SABER
    {
        Cmd_ToggleSaber_f(ent);

        if (*(*ent).client).ps.saberHolstered == 0 {}
    }
    //begin bot debug cmds
    else if Q_stricmp(cmd, c"debugBMove_Forward".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        let arg = 4000;
        assert!(trap::Argc() > 1);
        let sarg = trap::Argv(1);
        let sarg_c = std::ffi::CString::new(sarg).unwrap_or_default();

        assert!(!sarg_c.as_bytes().is_empty());
        let bCl = atoi(sarg_c.as_ptr());
        Bot_SetForcedMovement(bCl, arg, -1, -1);
    } else if Q_stricmp(cmd, c"debugBMove_Back".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        let arg = -4000;
        assert!(trap::Argc() > 1);
        let sarg = trap::Argv(1);
        let sarg_c = std::ffi::CString::new(sarg).unwrap_or_default();

        assert!(!sarg_c.as_bytes().is_empty());
        let bCl = atoi(sarg_c.as_ptr());
        Bot_SetForcedMovement(bCl, arg, -1, -1);
    } else if Q_stricmp(cmd, c"debugBMove_Right".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        let arg = 4000;
        assert!(trap::Argc() > 1);
        let sarg = trap::Argv(1);
        let sarg_c = std::ffi::CString::new(sarg).unwrap_or_default();

        assert!(!sarg_c.as_bytes().is_empty());
        let bCl = atoi(sarg_c.as_ptr());
        Bot_SetForcedMovement(bCl, -1, arg, -1);
    } else if Q_stricmp(cmd, c"debugBMove_Left".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        let arg = -4000;
        assert!(trap::Argc() > 1);
        let sarg = trap::Argv(1);
        let sarg_c = std::ffi::CString::new(sarg).unwrap_or_default();

        assert!(!sarg_c.as_bytes().is_empty());
        let bCl = atoi(sarg_c.as_ptr());
        Bot_SetForcedMovement(bCl, -1, arg, -1);
    } else if Q_stricmp(cmd, c"debugBMove_Up".as_ptr()) == 0 && CheatsOk(ent) != QFALSE {
        let arg = 4000;
        assert!(trap::Argc() > 1);
        let sarg = trap::Argv(1);
        let sarg_c = std::ffi::CString::new(sarg).unwrap_or_default();

        assert!(!sarg_c.as_bytes().is_empty());
        let bCl = atoi(sarg_c.as_ptr());
        Bot_SetForcedMovement(bCl, -1, -1, arg);
    }
    //end bot debug cmds
    // #ifndef FINAL_BUILD (developer commands — built live per the retail-module precedent)
    else if Q_stricmp(cmd, c"debugSetSaberMove".as_ptr()) == 0 {
        Cmd_DebugSetSaberMove_f(ent);
    } else if Q_stricmp(cmd, c"debugSetBodyAnim".as_ptr()) == 0 {
        Cmd_DebugSetBodyAnim_f(ent, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
    } else if Q_stricmp(cmd, c"debugDismemberment".as_ptr()) == 0 {
        Cmd_Kill_f(ent);
        if (*ent).health < 1 {
            let mut iArg = 0;

            if trap::Argc() > 1 {
                let arg = trap::Argv(1);
                let arg_c = std::ffi::CString::new(arg).unwrap_or_default();

                if !arg_c.as_bytes().is_empty() {
                    iArg = atoi(arg_c.as_ptr());
                }
            }

            DismembermentByNum(ent, iArg);
        }
    } else if Q_stricmp(cmd, c"debugDropSaber".as_ptr()) == 0 {
        if (*(*ent).client).ps.weapon == WP_SABER
            && (*(*ent).client).ps.saberEntityNum != 0
            && (*(*ent).client).ps.saberInFlight == QFALSE
        {
            saberKnockOutOfHand(
                (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .offset((*(*ent).client).ps.saberEntityNum as isize),
                ent,
                &vec3_origin,
            );
        }
    } else if Q_stricmp(cmd, c"debugKnockMeDown".as_ptr()) == 0 {
        if BG_KnockDownable(&mut (*(*ent).client).ps) != QFALSE {
            (*(*ent).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
            (*(*ent).client).ps.forceDodgeAnim = 0;
            if trap::Argc() > 1 {
                (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 1100;
                (*(*ent).client).ps.quickerGetup = QFALSE;
            } else {
                (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 700;
                (*(*ent).client).ps.quickerGetup = QTRUE;
            }
        }
    } else if Q_stricmp(cmd, c"debugSaberSwitch".as_ptr()) == 0 {
        let mut targ: *mut gentity_t = core::ptr::null_mut();

        if trap::Argc() > 1 {
            let arg = trap::Argv(1);
            let arg_c = std::ffi::CString::new(arg).unwrap_or_default();

            if !arg_c.as_bytes().is_empty() {
                let x = atoi(arg_c.as_ptr());

                if x >= 0 && x < MAX_CLIENTS as c_int {
                    targ = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .offset(x as isize);
                }
            }
        }

        if !targ.is_null() && (*targ).inuse != QFALSE && !(*targ).client.is_null() {
            Cmd_ToggleSaber_f(targ);
        }
    } else if Q_stricmp(cmd, c"debugIKGrab".as_ptr()) == 0 {
        let mut targ: *mut gentity_t = core::ptr::null_mut();

        if trap::Argc() > 1 {
            let arg = trap::Argv(1);
            let arg_c = std::ffi::CString::new(arg).unwrap_or_default();

            if !arg_c.as_bytes().is_empty() {
                let x = atoi(arg_c.as_ptr());

                if x >= 0 && x < MAX_CLIENTS as c_int {
                    targ = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .offset(x as isize);
                }
            }
        }

        if !targ.is_null()
            && (*targ).inuse != QFALSE
            && !(*targ).client.is_null()
            && (*ent).s.number != (*targ).s.number
        {
            (*(*targ).client).ps.heldByClient = (*ent).s.number + 1;
        }
    } else if Q_stricmp(cmd, c"debugIKBeGrabbedBy".as_ptr()) == 0 {
        let mut targ: *mut gentity_t = core::ptr::null_mut();

        if trap::Argc() > 1 {
            let arg = trap::Argv(1);
            let arg_c = std::ffi::CString::new(arg).unwrap_or_default();

            if !arg_c.as_bytes().is_empty() {
                let x = atoi(arg_c.as_ptr());

                if x >= 0 && x < MAX_CLIENTS as c_int {
                    targ = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .offset(x as isize);
                }
            }
        }

        if !targ.is_null()
            && (*targ).inuse != QFALSE
            && !(*targ).client.is_null()
            && (*ent).s.number != (*targ).s.number
        {
            (*(*ent).client).ps.heldByClient = (*targ).s.number + 1;
        }
    } else if Q_stricmp(cmd, c"debugIKRelease".as_ptr()) == 0 {
        let mut targ: *mut gentity_t = core::ptr::null_mut();

        if trap::Argc() > 1 {
            let arg = trap::Argv(1);
            let arg_c = std::ffi::CString::new(arg).unwrap_or_default();

            if !arg_c.as_bytes().is_empty() {
                let x = atoi(arg_c.as_ptr());

                if x >= 0 && x < MAX_CLIENTS as c_int {
                    targ = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .offset(x as isize);
                }
            }
        }

        if !targ.is_null() && (*targ).inuse != QFALSE && !(*targ).client.is_null() {
            (*(*targ).client).ps.heldByClient = 0;
        }
    } else if Q_stricmp(cmd, c"debugThrow".as_ptr()) == 0 {
        let mut tTo: vec3_t = [0.0; 3];
        let mut fwd: vec3_t = [0.0; 3];

        if (*(*ent).client).ps.weaponTime > 0
            || (*(*ent).client).ps.forceHandExtend != HANDEXTEND_NONE
            || (*(*ent).client).ps.groundEntityNum == ENTITYNUM_NONE
            || (*ent).health < 1
        {
            return;
        }

        AngleVectors(&(*(*ent).client).ps.viewangles, Some(&mut fwd), None, None);
        tTo[0] = (*(*ent).client).ps.origin[0] + fwd[0] * 32.0;
        tTo[1] = (*(*ent).client).ps.origin[1] + fwd[1] * 32.0;
        tTo[2] = (*(*ent).client).ps.origin[2] + fwd[2] * 32.0;

        let tr = trap::Trace(
            &(*(*ent).client).ps.origin,
            &vec3_origin,
            &vec3_origin,
            &tTo,
            (*ent).s.number,
            MASK_PLAYERSOLID,
        );

        if tr.fraction != 1.0 {
            let other = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .offset(tr.entityNum as isize);

            if (*other).inuse != QFALSE
                && !(*other).client.is_null()
                && (*(*other).client).ps.forceHandExtend == HANDEXTEND_NONE
                && (*(*other).client).ps.groundEntityNum != ENTITYNUM_NONE
                && (*other).health > 0
                && (*(*ent).client).ps.origin[2] as c_int
                    == (*(*other).client).ps.origin[2] as c_int
            {
                let pDif: f32 = 40.0;
                let mut entAngles: vec3_t = [0.0; 3];
                let mut entDir: vec3_t = [0.0; 3];
                let mut otherAngles: vec3_t = [0.0; 3];
                let mut otherDir: vec3_t = [0.0; 3];
                let mut intendedOrigin: vec3_t = [0.0; 3];
                let mut boltOrg: vec3_t = [0.0; 3];
                let mut pBoltOrg: vec3_t = [0.0; 3];
                let mut tAngles: vec3_t = [0.0; 3];
                let mut vDif: vec3_t = [0.0; 3];
                let mut fwd: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];

                VectorSubtract(
                    &(*(*other).client).ps.origin,
                    &(*(*ent).client).ps.origin,
                    &mut otherDir,
                );
                VectorCopy(&(*(*ent).client).ps.viewangles, &mut entAngles);
                entAngles[YAW as usize] = vectoyaw(&otherDir);
                SetClientViewAngle(ent, &entAngles);

                (*(*ent).client).ps.forceHandExtend = HANDEXTEND_PRETHROW;
                (*(*ent).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 5000;

                (*(*ent).client).throwingIndex = (*other).s.number;
                (*(*ent).client).doingThrow = (*addr_of!(level)).time + 5000;
                (*(*ent).client).beingThrown = 0;

                VectorSubtract(
                    &(*(*ent).client).ps.origin,
                    &(*(*other).client).ps.origin,
                    &mut entDir,
                );
                VectorCopy(&(*(*other).client).ps.viewangles, &mut otherAngles);
                otherAngles[YAW as usize] = vectoyaw(&entDir);
                SetClientViewAngle(other, &otherAngles);

                (*(*other).client).ps.forceHandExtend = HANDEXTEND_PRETHROWN;
                (*(*other).client).ps.forceHandExtendTime = (*addr_of!(level)).time + 5000;

                (*(*other).client).throwingIndex = (*ent).s.number;
                (*(*other).client).beingThrown = (*addr_of!(level)).time + 5000;
                (*(*other).client).doingThrow = 0;

                //Doing this now at a stage in the throw, isntead of initially.
                //other->client->ps.heldByClient = ent->s.number+1;

                G_EntitySound(other, CHAN_VOICE, G_SoundIndex("*pain100.wav"));
                G_EntitySound(ent, CHAN_VOICE, G_SoundIndex("*jump1.wav"));
                G_Sound(
                    other,
                    CHAN_AUTO,
                    G_SoundIndex("sound/movers/objects/objectHit.wav"),
                );

                //see if we can move to be next to the hand.. if it's not clear, break the throw.
                VectorClear(&mut tAngles);
                tAngles[YAW as usize] = (*(*ent).client).ps.viewangles[YAW as usize];
                VectorCopy(&(*(*ent).client).ps.origin, &mut pBoltOrg);
                AngleVectors(&tAngles, Some(&mut fwd), Some(&mut right), None);
                boltOrg[0] = pBoltOrg[0] + fwd[0] * 8.0 + right[0] * pDif;
                boltOrg[1] = pBoltOrg[1] + fwd[1] * 8.0 + right[1] * pDif;
                boltOrg[2] = pBoltOrg[2];

                VectorSubtract(&boltOrg, &pBoltOrg, &mut vDif);
                VectorNormalize(&mut vDif);

                VectorClear(&mut (*(*other).client).ps.velocity);
                intendedOrigin[0] = pBoltOrg[0] + vDif[0] * pDif;
                intendedOrigin[1] = pBoltOrg[1] + vDif[1] * pDif;
                intendedOrigin[2] = (*(*other).client).ps.origin[2];

                let tr = trap::Trace(
                    &intendedOrigin,
                    &(*other).r.mins,
                    &(*other).r.maxs,
                    &intendedOrigin,
                    (*other).s.number,
                    (*other).clipmask,
                );
                let tr2 = trap::Trace(
                    &(*(*ent).client).ps.origin,
                    &(*ent).r.mins,
                    &(*ent).r.maxs,
                    &intendedOrigin,
                    (*ent).s.number,
                    CONTENTS_SOLID,
                );

                if tr.fraction == 1.0
                    && tr.startsolid == 0
                    && tr2.fraction == 1.0
                    && tr2.startsolid == 0
                {
                    VectorCopy(&intendedOrigin, &mut (*(*other).client).ps.origin);
                } else {
                    //if the guy can't be put here then it's time to break the throw off.
                    let mut oppDir: vec3_t = [0.0; 3];
                    let strength = 4;

                    (*(*other).client).ps.heldByClient = 0;
                    (*(*other).client).beingThrown = 0;
                    (*(*ent).client).doingThrow = 0;

                    (*(*ent).client).ps.forceHandExtend = HANDEXTEND_NONE;
                    G_EntitySound(ent, CHAN_VOICE, G_SoundIndex("*pain25.wav"));

                    (*(*other).client).ps.forceHandExtend = HANDEXTEND_NONE;
                    VectorSubtract(
                        &(*(*other).client).ps.origin,
                        &(*(*ent).client).ps.origin,
                        &mut oppDir,
                    );
                    VectorNormalize(&mut oppDir);
                    (*(*other).client).ps.velocity[0] = oppDir[0] * (strength as f32 * 40.0);
                    (*(*other).client).ps.velocity[1] = oppDir[1] * (strength as f32 * 40.0);
                    (*(*other).client).ps.velocity[2] = 150.0;

                    VectorSubtract(
                        &(*(*ent).client).ps.origin,
                        &(*(*other).client).ps.origin,
                        &mut oppDir,
                    );
                    VectorNormalize(&mut oppDir);
                    (*(*ent).client).ps.velocity[0] = oppDir[0] * (strength as f32 * 40.0);
                    (*(*ent).client).ps.velocity[1] = oppDir[1] * (strength as f32 * 40.0);
                    (*(*ent).client).ps.velocity[2] = 150.0;
                }
            }
        }
    }
    // #ifdef VM_MEMALLOC_DEBUG (debugTestAlloc) omitted — alloc stress test, debug-only.
    else if Q_stricmp(cmd, c"debugShipDamage".as_ptr()) == 0 {
        let arg = trap::Argv(1);
        let arg_c = std::ffi::CString::new(arg).unwrap_or_default();
        let arg2 = trap::Argv(2);
        let arg2_c = std::ffi::CString::new(arg2).unwrap_or_default();
        let shipSurf = SHIPSURF_FRONT + atoi(arg_c.as_ptr());
        let damageLevel = atoi(arg2_c.as_ptr());

        G_SetVehDamageFlags(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .offset((*ent).s.m_iVehicleNum as isize),
            shipSurf,
            damageLevel,
        );
    } else {
        if Q_stricmp(cmd, c"addbot".as_ptr()) == 0 {
            //because addbot isn't a recognized command unless you're the server, but it is in the menus regardless
            //			trap_SendServerCommand( clientNum, va("print \"You can only add bots as the server.\n\"" ) );
            trap::SendServerCommand(
                clientNum,
                &format!(
                    "print \"{}.\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"ONLY_ADD_BOTS_AS_SERVER".as_ptr() as *mut c_char,
                    )),
                ),
            );
        } else {
            trap::SendServerCommand(clientNum, &format!("print \"unknown cmd {}\n\"", Sz(cmd)));
        }
    }
}

/// `void G_CheckTKAutoKickBan( gentity_t *ent )` (g_cmds.c:527) — team-kill spam
/// check: bump the client's `sess.TKCount`, then auto-ban (`AddIP` + `clientkick`)
/// once it reaches `g_autoBanTKSpammers`, auto-kick (`clientkick`) once it reaches
/// `g_autoKickTKSpammers`, otherwise warn the player about the impending ban/kick.
/// No oracle — mutates live game/admin state (drives `trap_SendServerCommand`/
/// `trap_SendConsoleCommand` and `AddIP` off the kick/ban cvars), not oracle-testable.
///
/// # Safety
/// `ent` may be null; otherwise it must point to a valid `gentity_t`, and
/// `g_entities` must be initialized.
pub unsafe fn G_CheckTKAutoKickBan(ent: *mut gentity_t) {
    // `EXEC_INSERT` (q_shared.h) — the `EXEC_*` enum's second value, so `1`.
    const EXEC_INSERT: c_int = 1;

    if ent.is_null() || (*ent).client.is_null() || (*ent).s.number >= MAX_CLIENTS as c_int {
        return;
    }

    if (*addr_of!(g_autoKickTKSpammers)).integer > 0 || (*addr_of!(g_autoBanTKSpammers)).integer > 0
    {
        let client = (*ent).client;
        (*client).sess.TKCount += 1;
        if (*addr_of!(g_autoBanTKSpammers)).integer > 0
            && (*client).sess.TKCount >= (*addr_of!(g_autoBanTKSpammers)).integer
        {
            // C tests `if ( ent->client->sess.IPstring )`; `IPstring` is a
            // `char[32]` array, so the test is always true — ban their IP.
            AddIP((*client).sess.IPstring.as_ptr());

            trap::SendServerCommand(
                -1,
                &format!(
                    "print \"{} {}\n\"",
                    Sz((*client).pers.netname.as_ptr()),
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"TKBAN".as_ptr() as *mut c_char,
                    )),
                ),
            );
            //Com_sprintf ( level.voteString, sizeof(level.voteString ), "clientkick %d", ent->s.number );
            //Com_sprintf ( level.voteDisplayString, sizeof(level.voteDisplayString), "kick %s", ent->client->pers.netname );
            //trap_SendConsoleCommand( EXEC_INSERT, va( "banClient %d\n", ent->s.number ) );
            trap::SendConsoleCommand(EXEC_INSERT, &format!("clientkick {}\n", (*ent).s.number));
            return;
        }
        if (*addr_of!(g_autoKickTKSpammers)).integer > 0
            && (*client).sess.TKCount >= (*addr_of!(g_autoKickTKSpammers)).integer
        {
            trap::SendServerCommand(
                -1,
                &format!(
                    "print \"{} {}\n\"",
                    Sz((*client).pers.netname.as_ptr()),
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"TKKICK".as_ptr() as *mut c_char,
                    )),
                ),
            );
            //Com_sprintf ( level.voteString, sizeof(level.voteString ), "clientkick %d", ent->s.number );
            //Com_sprintf ( level.voteDisplayString, sizeof(level.voteDisplayString), "kick \"%s\"\n", ent->client->pers.netname );
            trap::SendConsoleCommand(EXEC_INSERT, &format!("clientkick {}\n", (*ent).s.number));
            return;
        }
        //okay, not gone (yet), but warn them...
        if (*addr_of!(g_autoBanTKSpammers)).integer > 0
            && ((*addr_of!(g_autoKickTKSpammers)).integer <= 0
                || (*addr_of!(g_autoBanTKSpammers)).integer
                    < (*addr_of!(g_autoKickTKSpammers)).integer)
        {
            //warn about ban
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"WARNINGTKBAN".as_ptr() as *mut c_char,
                    )),
                ),
            );
        } else if (*addr_of!(g_autoKickTKSpammers)).integer > 0 {
            //warn about kick
            trap::SendServerCommand(
                ent.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!(
                    "print \"{}\n\"",
                    Sz(G_GetStringEdString(
                        c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                        c"WARNINGTKKICK".as_ptr() as *mut c_char,
                    )),
                ),
            );
        }
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use core::ffi::CStr;

    extern "C" {
        fn jka_SanitizeString(in_: *mut c_char, out: *mut c_char);
        fn jka_SanitizeString2(in_: *mut c_char, out: *mut c_char);
    }

    #[test]
    fn sanitize_string_matches_c() {
        // Color codes (^x), control chars, mixed case, escape-at-end, empties.
        let cases: &[&[u8]] = &[
            b"\0",
            b"Hello World\0",
            b"MixedCASE123\0",
            b"\x1b1Red\x1b7White\0",   // two color codes
            b"tab\there\0",            // embedded control char (9 < 32)
            b"trailing\x1b\0",         // escape with one byte after (the NUL)
            b"\x1b\0",                 // lone escape then NUL
            b"\x01\x02\x1f visible\0", // run of control chars
            b"ALLCAPS\0",
            b"sym!@#$%^&*()\0", // '^' (94) is NOT the escape (27); passes
        ];

        for case in cases {
            let input = case.to_vec();
            // out buffer can never exceed input length + NUL.
            let mut rust_out = vec![0u8; input.len() + 16];
            let mut c_out = vec![0u8; input.len() + 16];

            unsafe {
                let rin = input.clone();
                SanitizeString(
                    rin.as_ptr() as *const c_char,
                    rust_out.as_mut_ptr() as *mut c_char,
                );
                let mut cin = input.clone();
                jka_SanitizeString(
                    cin.as_mut_ptr() as *mut c_char,
                    c_out.as_mut_ptr() as *mut c_char,
                );
            }

            let r = unsafe { CStr::from_ptr(rust_out.as_ptr() as *const c_char) };
            let c = unsafe { CStr::from_ptr(c_out.as_ptr() as *const c_char) };
            assert_eq!(r, c, "SanitizeString({:?})", String::from_utf8_lossy(case));
        }
    }

    #[test]
    fn sanitize_string2_matches_c() {
        // Color codes (^x with/without digit), control chars, mixed case (NOT
        // lowercased), MAX_NAME_LENGTH truncation, lone-^, empties.
        let cases: &[&[u8]] = &[
            b"\0",
            b"Hello World\0",
            b"MixedCASE123\0",         // case preserved, unlike SanitizeString
            b"^1Red^7White\0",         // two color codes (digit after ^)
            b"a^bc\0",                 // lone ^ (no digit) -> skip just the ^
            b"^\0",                    // trailing ^ then NUL
            b"tab\there\0",            // embedded control char (9 < 32)
            b"\x01\x02\x1f visible\0", // run of control chars
            b"^9end^\0",               // color code then trailing lone ^
            b"0123456789012345678901234567890123456789\0", // > MAX_NAME_LENGTH, truncates
            b"^1^2^3^4^5^6^7^8^9^0abc\0", // many color codes (indices count toward cap)
        ];

        for case in cases {
            let input = case.to_vec();
            let mut rust_out = vec![0u8; input.len() + 16];
            let mut c_out = vec![0u8; input.len() + 16];

            unsafe {
                let mut rin = input.clone();
                SanitizeString2(
                    rin.as_mut_ptr() as *mut c_char,
                    rust_out.as_mut_ptr() as *mut c_char,
                );
                let mut cin = input.clone();
                jka_SanitizeString2(
                    cin.as_mut_ptr() as *mut c_char,
                    c_out.as_mut_ptr() as *mut c_char,
                );
            }

            let r = unsafe { CStr::from_ptr(rust_out.as_ptr() as *const c_char) };
            let c = unsafe { CStr::from_ptr(c_out.as_ptr() as *const c_char) };
            assert_eq!(r, c, "SanitizeString2({:?})", String::from_utf8_lossy(case));
        }
    }
}
