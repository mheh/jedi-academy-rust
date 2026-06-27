//! `g_session.c` — session data.
//!
//! Session data is the only data that stays persistent across level loads and
//! tournament restarts.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_session.c`, incrementally as consumers
//! need it (the same lazy strategy used for the `trap_*` surface and `g_utils.c`).
//! This slice is **`G_InitWorldSession`** — the one function `G_InitGame` calls
//! directly (g_main.c:980). It reads the persistent `"session"` cvar and, if the
//! gametype changed since the last map, raises `level.newSession` so stale
//! per-client session data is discarded.
//!
//! All five functions are now ported: the serialization pair
//! **`G_WriteClientSessionData`** (g_session.c:23) and **`G_ReadSessionData`**
//! (g_session.c:105), plus **`G_WriteSessionData`** (g_session.c:312),
//! **`G_InitWorldSession`** (g_session.c:291), and **`G_InitSessionData`**
//! (g_session.c:187, the first-connect team determination — landed once
//! `BroadcastTeamChange` opened g_cmds.rs). This file is **complete (5/5)**.

#![allow(non_snake_case)] // C function names (`G_InitWorldSession`, ...) kept verbatim

use core::ffi::{c_char, c_int};
#[cfg(feature = "vm")]
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut};
use std::ffi::{CStr, CString};

use crate::codemp::game::bg_lib::sscanf;
use crate::codemp::game::bg_public::{
    DUELTEAM_DOUBLE, DUELTEAM_LONE, GT_DUEL, GT_POWERDUEL, GT_TEAM, TEAM_BLUE, TEAM_FREE, TEAM_RED,
    TEAM_SPECTATOR,
};
use crate::codemp::game::g_client::PickTeam;
use crate::codemp::game::g_cmds::BroadcastTeamChange;
use crate::codemp::game::g_local::{gclient_t, SPECTATOR_FREE, CON_CONNECTED};
use crate::codemp::game::g_main::{
    g_gametype, g_maxGameClients, g_teamAutoJoin, level, G_PowerDuelCount, G_Printf,
};
use crate::codemp::game::q_shared::{va, Info_ValueForKey, Sz};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

extern "C" {
    /// `int atoi( const char *string )` — the retail (non-`Q3_VM`) build links the
    /// C library's `atoi` (bg_lib.c's own `atoi` is the `Q3_VM` path), the same
    /// extern `q_shared.c`'s parser uses.
    fn atoi(s: *const c_char) -> c_int;

    /// `char *strcpy( char *dest, const char *src )` — libc, linked in both builds
    /// (same local-extern pattern as `bg_misc` / `bg_saberLoad` / `bg_vehicleLoad`;
    /// `bg_lib.c`'s own `strcpy` is only the `Q3_VM` shim).
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

/// `void G_WriteClientSessionData( gclient_t *client )` (g_session.c:23) — called on
/// game shutdown (and at first-connect, by `G_InitSessionData`). Serializes the
/// client's [`clientSession_t`](crate::codemp::game::g_local::clientSession_t) into
/// the per-client `"session%i"` cvar.
///
/// The siege-class and both saber-type names can contain spaces, but the serialized
/// form is space-separated, so each name is copied to a scratch buffer with its
/// spaces hacked to char `1` (read back the same way by [`G_ReadSessionData`]); an
/// empty siege class is written as `"none"`. The 12 integer fields are written in
/// the fixed serialization order (which is **not** struct order — `teamLeader` /
/// `setForce` / `saberLevel` / `selectedFP` precede `duelTeam`).
///
/// # Safety
/// `client` must point to a live `gclient_t` whose `sess` is initialized, and must
/// lie within `level.clients` (the index `client - level.clients` is the cvar
/// suffix) — the contract the C call sites already uphold.
pub unsafe fn G_WriteClientSessionData(client: *mut gclient_t) {
    let mut siegeClass = [0 as c_char; 64];
    let mut saberType = [0 as c_char; 64];
    let mut saber2Type = [0 as c_char; 64];

    strcpy(siegeClass.as_mut_ptr(), (*client).sess.siegeClass.as_ptr());

    let mut i = 0;
    while siegeClass[i] != 0 {
        // sort of a hack.. we don't want spaces by siege class names have spaces so
        // convert them all to unused chars
        if siegeClass[i] == b' ' as c_char {
            siegeClass[i] = 1;
        }
        i += 1;
    }

    if siegeClass[0] == 0 {
        // make sure there's at least something
        strcpy(siegeClass.as_mut_ptr(), c"none".as_ptr());
    }

    // Do the same for the saber
    strcpy(saberType.as_mut_ptr(), (*client).sess.saberType.as_ptr());

    i = 0;
    while saberType[i] != 0 {
        if saberType[i] == b' ' as c_char {
            saberType[i] = 1;
        }
        i += 1;
    }

    strcpy(saber2Type.as_mut_ptr(), (*client).sess.saber2Type.as_ptr());

    i = 0;
    while saber2Type[i] != 0 {
        if saber2Type[i] == b' ' as c_char {
            saber2Type[i] = 1;
        }
        i += 1;
    }

    let s = va(format_args!(
        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        (*client).sess.sessionTeam,
        (*client).sess.spectatorTime,
        (*client).sess.spectatorState,
        (*client).sess.spectatorClient,
        (*client).sess.wins,
        (*client).sess.losses,
        (*client).sess.teamLeader,
        (*client).sess.setForce,
        (*client).sess.saberLevel,
        (*client).sess.selectedFP,
        (*client).sess.duelTeam,
        (*client).sess.siegeDesiredTeam,
        Sz(siegeClass.as_ptr()),
        Sz(saberType.as_ptr()),
        Sz(saber2Type.as_ptr()),
    ));

    // `va` rotates two static buffers, so `var` (buffer 1) does not clobber `s`
    // (buffer 0); both survive to the `Cvar_Set` below, exactly as in the C.
    let var = va(format_args!(
        "session{}",
        client.offset_from((*addr_of!(level)).clients)
    ));

    trap::Cvar_Set(
        &CStr::from_ptr(var).to_string_lossy(),
        &CStr::from_ptr(s).to_string_lossy(),
    );
}

/// `void G_ReadSessionData( gclient_t *client )` (g_session.c:105) — called on a
/// reconnect. Reads the per-client `"session%i"` cvar back into the client's
/// [`clientSession_t`](crate::codemp::game::g_local::clientSession_t), undoing the
/// space→char-`1` name hack [`G_WriteClientSessionData`] applied.
///
/// The C `sscanf`s into temporary `int`s for the three enum/`qboolean` fields
/// (`sessionTeam` / `spectatorState` / `teamLeader`) and casts them back afterward.
/// `sscanf` is [`bg_lib::sscanf`](crate::codemp::game::bg_lib::sscanf): a `#[cfg]`
/// split as in [`BG_ParseField`](crate::codemp::game::bg_misc::BG_ParseField) — the
/// native build is a true variadic libc call; the `vm` shim takes an output-pointer
/// slice and (faithful to `bg_lib.c`) handles only `%i/%d/%u/%f`, so under `vm` the
/// three `%s` name fields are left as-is, matching the original `Q3_VM` build.
///
/// # Safety
/// As [`G_WriteClientSessionData`] — `client` must be a live in-`level.clients`
/// `gclient_t`.
pub unsafe fn G_ReadSessionData(client: *mut gclient_t) {
    // bk001205 - format
    let mut teamLeader: c_int = 0;
    let mut spectatorState: c_int = 0;
    let mut sessionTeam: c_int = 0;

    let var = va(format_args!(
        "session{}",
        client.offset_from((*addr_of!(level)).clients)
    ));
    // The C reads into a `char s[MAX_STRING_CHARS]`; the trap wrapper returns an owned
    // String, bridged back to a NUL-terminated buffer for `sscanf`.
    let s = trap::Cvar_VariableString(&CStr::from_ptr(var).to_string_lossy());
    let s = CString::new(s).unwrap_or_default();

    #[cfg(feature = "vm")]
    sscanf(
        s.as_ptr(),
        c"%i %i %i %i %i %i %i %i %i %i %i %i %s %s %s".as_ptr(),
        &[
            addr_of_mut!(sessionTeam) as *mut c_void,
            addr_of_mut!((*client).sess.spectatorTime) as *mut c_void,
            addr_of_mut!(spectatorState) as *mut c_void,
            addr_of_mut!((*client).sess.spectatorClient) as *mut c_void,
            addr_of_mut!((*client).sess.wins) as *mut c_void,
            addr_of_mut!((*client).sess.losses) as *mut c_void,
            addr_of_mut!(teamLeader) as *mut c_void,
            addr_of_mut!((*client).sess.setForce) as *mut c_void,
            addr_of_mut!((*client).sess.saberLevel) as *mut c_void,
            addr_of_mut!((*client).sess.selectedFP) as *mut c_void,
            addr_of_mut!((*client).sess.duelTeam) as *mut c_void,
            addr_of_mut!((*client).sess.siegeDesiredTeam) as *mut c_void,
            (*client).sess.siegeClass.as_mut_ptr() as *mut c_void,
            (*client).sess.saberType.as_mut_ptr() as *mut c_void,
            (*client).sess.saber2Type.as_mut_ptr() as *mut c_void,
        ],
    );
    #[cfg(not(feature = "vm"))]
    sscanf(
        s.as_ptr(),
        c"%i %i %i %i %i %i %i %i %i %i %i %i %s %s %s".as_ptr(),
        addr_of_mut!(sessionTeam),                       // bk010221 - format
        addr_of_mut!((*client).sess.spectatorTime),
        addr_of_mut!(spectatorState),                    // bk010221 - format
        addr_of_mut!((*client).sess.spectatorClient),
        addr_of_mut!((*client).sess.wins),
        addr_of_mut!((*client).sess.losses),
        addr_of_mut!(teamLeader),                        // bk010221 - format
        addr_of_mut!((*client).sess.setForce),
        addr_of_mut!((*client).sess.saberLevel),
        addr_of_mut!((*client).sess.selectedFP),
        addr_of_mut!((*client).sess.duelTeam),
        addr_of_mut!((*client).sess.siegeDesiredTeam),
        (*client).sess.siegeClass.as_mut_ptr(),
        (*client).sess.saberType.as_mut_ptr(),
        (*client).sess.saber2Type.as_mut_ptr(),
    );

    let mut i = 0;
    while (*client).sess.siegeClass[i] != 0 {
        // convert back to spaces from unused chars, as session data is written that way.
        if (*client).sess.siegeClass[i] == 1 {
            (*client).sess.siegeClass[i] = b' ' as c_char;
        }
        i += 1;
    }

    i = 0;
    // And do the same for the saber type
    while (*client).sess.saberType[i] != 0 {
        if (*client).sess.saberType[i] == 1 {
            (*client).sess.saberType[i] = b' ' as c_char;
        }
        i += 1;
    }

    i = 0;
    while (*client).sess.saber2Type[i] != 0 {
        if (*client).sess.saber2Type[i] == 1 {
            (*client).sess.saber2Type[i] = b' ' as c_char;
        }
        i += 1;
    }

    // bk001205 - format issues: the three enum/qboolean fields were sscanf'd through
    // int temporaries (team_t / spectatorState_t / qboolean are all c_int here).
    (*client).sess.sessionTeam = sessionTeam;
    (*client).sess.spectatorState = spectatorState;
    (*client).sess.teamLeader = teamLeader;

    (*client).ps.fd.saberAnimLevel = (*client).sess.saberLevel;
    (*client).ps.fd.saberDrawAnimLevel = (*client).sess.saberLevel;
    (*client).ps.fd.forcePowerSelected = (*client).sess.selectedFP;
}

/// `void G_InitSessionData( gclient_t *client, char *userinfo, qboolean isBot )`
/// (g_session.c:187) — set up a client's fresh per-map session on first connect:
/// pick the initial team, default the spectator/saber/siege fields, then persist
/// via `G_WriteClientSessionData`.
///
/// Team gametypes (`>= GT_TEAM`): with `g_teamAutoJoin` set, `PickTeam(-1)` +
/// `BroadcastTeamChange`; otherwise humans spawn as spectators and bots read their
/// `"team"` userinfo key (`r`/`b`, else `PickTeam`) then announce. Below `GT_TEAM`:
/// a `"team"` value starting `s` is a willing spectator; else the gametype decides
/// (FFA/holocron/JM/SP cap on `g_maxGameClients`, Duel caps at 2 playing, PowerDuel
/// assigns a `duelTeam` from the live loner/double tally and waits as spectator).
/// Finally zero the spectator/siege/saber session fields and write them out.
///
/// No oracle: drives `PickTeam`/`BroadcastTeamChange`/`G_PowerDuelCount` plus the
/// `level`/`g_gametype` globals and `Info_ValueForKey` over engine userinfo — the
/// connect/session-infra precedent. Faithful 1:1 with original JKA.
///
/// # Safety
/// `client` must point to a valid `gclient_t`; `userinfo` a valid NUL-terminated
/// userinfo string.
pub unsafe fn G_InitSessionData(client: *mut gclient_t, userinfo: *mut c_char, is_bot: qboolean) {
    // clientSession_t *sess = &client->sess;
    let sess = addr_of_mut!((*client).sess);

    (*client).sess.siegeDesiredTeam = TEAM_FREE;

    // initial team determination
    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        if (*addr_of!(g_teamAutoJoin)).integer != 0 {
            (*sess).sessionTeam = PickTeam(-1);
            BroadcastTeamChange(client, -1);
        } else {
            // always spawn as spectator in team games
            if is_bot == QFALSE {
                (*sess).sessionTeam = TEAM_SPECTATOR;
            } else {
                // Bots choose their team on creation
                let value = Info_ValueForKey(userinfo, c"team".as_ptr());
                let c0 = *value;
                if c0 == b'r' as c_char || c0 == b'R' as c_char {
                    (*sess).sessionTeam = TEAM_RED;
                } else if c0 == b'b' as c_char || c0 == b'B' as c_char {
                    (*sess).sessionTeam = TEAM_BLUE;
                } else {
                    (*sess).sessionTeam = PickTeam(-1);
                }
                BroadcastTeamChange(client, -1);
            }
        }
    } else {
        let value = Info_ValueForKey(userinfo, c"team".as_ptr());
        if *value == b's' as c_char {
            // a willing spectator, not a waiting-in-line
            (*sess).sessionTeam = TEAM_SPECTATOR;
        } else {
            match (*addr_of!(g_gametype)).integer {
                // default: | GT_FFA | GT_HOLOCRON | GT_JEDIMASTER | GT_SINGLE_PLAYER
                GT_DUEL => {
                    // if the game is full, go into a waiting mode
                    if (*addr_of!(level)).numNonSpectatorClients >= 2 {
                        (*sess).sessionTeam = TEAM_SPECTATOR;
                    } else {
                        (*sess).sessionTeam = TEAM_FREE;
                    }
                }
                GT_POWERDUEL => {
                    // sess->duelTeam = DUELTEAM_LONE; // default
                    let mut loners = 0;
                    let mut doubles = 0;

                    G_PowerDuelCount(&mut loners, &mut doubles, QTRUE);

                    if doubles == 0 || loners > (doubles / 2) {
                        (*sess).duelTeam = DUELTEAM_DOUBLE;
                    } else {
                        (*sess).duelTeam = DUELTEAM_LONE;
                    }
                    (*sess).sessionTeam = TEAM_SPECTATOR;
                }
                // GT_FFA / GT_HOLOCRON / GT_JEDIMASTER / GT_SINGLE_PLAYER / default
                _ => {
                    if (*addr_of!(g_maxGameClients)).integer > 0
                        && (*addr_of!(level)).numNonSpectatorClients
                            >= (*addr_of!(g_maxGameClients)).integer
                    {
                        (*sess).sessionTeam = TEAM_SPECTATOR;
                    } else {
                        (*sess).sessionTeam = TEAM_FREE;
                    }
                }
            }
        }
    }

    (*sess).spectatorState = SPECTATOR_FREE;
    (*sess).spectatorTime = (*addr_of!(level)).time;

    (*sess).siegeClass[0] = 0;
    (*sess).saberType[0] = 0;
    (*sess).saber2Type[0] = 0;

    G_WriteClientSessionData(client);
}

/// `void G_InitWorldSession( void )` (g_session.c:291) — called once from
/// `G_InitGame`. Reads the persistent `"session"` cvar (the gametype in effect at
/// the last `G_WriteSessionData`); if it differs from the current `g_gametype`,
/// the old per-client session data is stale, so `level.newSession` is raised.
///
/// The C reads the cvar into a `char s[MAX_STRING_CHARS]` then `atoi`s it; the
/// trap wrapper returns an owned `String`, so the scratch buffer is gone and the
/// string is bridged back to a `*const c_char` (via `CString`) for the same libc
/// `atoi`.
pub fn G_InitWorldSession() {
    let s = trap::Cvar_VariableString("session");
    let cs = CString::new(s).unwrap_or_default();
    // SAFETY: single-threaded module init; `g_gametype` / `level` are the module's
    // own statics, and `atoi` is the libc extern above reading a NUL-terminated
    // CString that outlives the call.
    unsafe {
        let gt = atoi(cs.as_ptr());

        // if the gametype changed since the last session, don't use any
        // client sessions
        if (*addr_of!(g_gametype)).integer != gt {
            (*addr_of_mut!(level)).newSession = QTRUE;
            G_Printf("Gametype changed, clearing session data.\n");
        }
    }
}

/// `void G_WriteSessionData( void )` (g_session.c:312) — called from `G_ShutdownGame`
/// and `ExitLevel`. Writes the persistent `"session"` cvar (the current `g_gametype`,
/// read back by [`G_InitWorldSession`] on the next map) and then serializes every
/// `CON_CONNECTED` client's session via [`G_WriteClientSessionData`].
///
/// Faithful to the C: iterates `level.maxclients` (not the `g_maxclients` cvar) and
/// skips any client whose `pers.connected` is not `CON_CONNECTED`.
///
/// No-oracle: pure trap/global plumbing (`trap_Cvar_Set` + a scan of the mutable
/// `level` global). Nothing computable to compare bit-exact.
pub fn G_WriteSessionData() {
    // SAFETY: single-threaded module shutdown/exit-level; `g_gametype` / `level` are
    // the module's own statics, and `level.clients` is the engine-allocated array
    // indexed within `level.maxclients` — the contract `G_WriteClientSessionData`
    // (which takes `&level.clients[i]`) already upholds.
    unsafe {
        let lvl = addr_of!(level);

        trap::Cvar_Set(
            "session",
            &CStr::from_ptr(va(format_args!("{}", (*addr_of!(g_gametype)).integer)))
                .to_string_lossy(),
        );

        for i in 0..(*lvl).maxclients {
            if (*(*lvl).clients.add(i as usize)).pers.connected == CON_CONNECTED {
                G_WriteClientSessionData((*addr_of_mut!(level)).clients.add(i as usize));
            }
        }
    }
}
