//! Port of `g_bot.c` — the bot roster / connection-management module: it loads
//! the `.bot`/`.arena` roster files, parses their `{ key value }` info blocks,
//! tracks the live bot/human player counts, runs the delayed bot-spawn queue,
//! and drives the `addbot`/`botlist` server commands plus `bot_minplayers`
//! auto-fill. This is the roster/connection layer — it is independent of the
//! `ai_main` acting tree (the per-frame bot "brain").
//!
//! The two module statics that back the roster (`g_botInfos`, `g_arenaInfos`)
//! and the spawn queue (`botSpawnQueue`) live here, matching upstream. `g_numBots`
//! is file-static in C; `g_numArenas` is external (read by the arena code). The
//! parsed info strings are `G_Alloc`'d (never freed for the level's lifetime),
//! exactly as in C.
//!
//! Opened leaves-up: the pure helpers (`G_GetMapTypeBits`), the roster lookups
//! (`G_GetBotInfoByName`/`...ByNumber`/`G_GetArenaInfoByMap`), the info parser
//! (`G_ParseInfos`), the file loaders, the player counters, the random-bot
//! add/remove + spawn-queue machinery, and `G_AddBot`/`Svcmd_AddBot_f`/
//! `Svcmd_BotList_f`, `G_InitBots` (the GAME_INIT roster bring-up), and
//! `G_BotConnect` (the per-bot GAME_CLIENT_CONNECT setup).

#![allow(non_snake_case)] // C function names (`G_AddBot`) kept verbatim
#![allow(non_upper_case_globals)] // C global names (`g_botInfos`) kept verbatim
#![allow(non_camel_case_types)] // C struct names (`botSpawnQueue_t`) kept verbatim
#![allow(dead_code)] // roster/queue fns consumed by the bot-spawn + svcmd paths once wired

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::ai_main::BotAISetupClient;
use crate::codemp::game::ai_wpnav::LoadPath_ThisLevel;
use crate::codemp::game::bg_public::{
    DUELTEAM_DOUBLE, DUELTEAM_LONE, GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER,
    GT_POWERDUEL, GT_SIEGE, GT_TEAM, MAX_ARENAS, MAX_ARENAS_TEXT, MAX_BOTS, MAX_BOTS_TEXT,
    PERS_TEAM, TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR,
};
use crate::codemp::game::g_client::{ClientBegin, ClientConnect, ClientUserinfoChanged, PickTeam};
use crate::codemp::game::g_cmds::SetTeam;
use crate::codemp::game::g_local::bot_settings_t;
use crate::codemp::game::g_local::{gclient_t, gentity_t, CON_CONNECTED};
use crate::codemp::game::g_main::{
    g_autoMapCycle, g_entities, g_gametype, g_maxclients, level, Com_Printf, G_GetStringEdString,
    G_PowerDuelCount, G_Printf,
};
use crate::codemp::game::g_mem::G_Alloc;
use crate::codemp::game::g_public_h::SVF_BOT;
use crate::codemp::game::g_session::G_ReadSessionData;
use crate::codemp::game::q_shared::{
    random, va, COM_Parse, COM_ParseExt, Info_SetValueForKey, Info_ValueForKey, Q_CleanStr,
    Q_stricmp, Q_strncpyz, Sz,
};
use crate::codemp::game::q_shared_h::{
    CVAR_INIT, CVAR_ROM, CVAR_SERVERINFO, FS_READ, MAX_INFO_STRING, MAX_TOKEN_CHARS,
};
use crate::ffi::types::{fileHandle_t, qboolean, vmCvar_t, QFALSE, QTRUE};
use crate::trap;

const S_COLOR_RED: &str = "^1";
const S_COLOR_YELLOW: &str = "^3";

// q_shared.h `EXEC_*` enum: EXEC_NOW=0, EXEC_INSERT=1, EXEC_APPEND=2. Used by
// `trap_SendConsoleCommand`. Mirrors g_main.rs's module-private consts.
const EXEC_INSERT: c_int = 1;

extern "C" {
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn atof(s: *const c_char) -> f64;
    fn atoi(s: *const c_char) -> c_int;
}

// g_bot.c:8-9 — the bot roster: `g_numBots` parsed entries, each a `G_Alloc`'d
// info string. `g_numBots` is file-static in C.
static mut g_numBots: c_int = 0;
static mut g_botInfos: [*mut c_char; MAX_BOTS as usize] = [null_mut(); MAX_BOTS as usize];

// g_bot.c:12-13 — the arena roster. `g_numArenas` has external linkage in C (the
// arena/intermission code reads it), so it is `pub`.
pub static mut g_numArenas: c_int = 0;
static mut g_arenaInfos: [*mut c_char; MAX_ARENAS as usize] = [null_mut(); MAX_ARENAS as usize];

// g_bot.c:16-19
const BOT_BEGIN_DELAY_BASE: c_int = 2000;
const BOT_BEGIN_DELAY_INCREMENT: c_int = 1500;
const BOT_SPAWN_QUEUE_DEPTH: usize = 16;

// g_bot.c:21-24
#[derive(Clone, Copy)]
struct botSpawnQueue_t {
    clientNum: c_int,
    spawnTime: c_int,
}

// g_bot.c:27
static mut botSpawnQueue: [botSpawnQueue_t; BOT_SPAWN_QUEUE_DEPTH] = [botSpawnQueue_t {
    clientNum: 0,
    spawnTime: 0,
}; BOT_SPAWN_QUEUE_DEPTH];

// g_bot.c:29
pub static mut bot_minplayers: vmCvar_t = vmCvar_t::zeroed();

// g_bot.c:566 — file-scope throttle for `G_CheckMinimumPlayers`.
static mut checkminimumplayers_time: c_int = 0;

/// `int G_GetMapTypeBits(char *type)` (g_bot.c:129). Maps an arena's `type`
/// string (e.g. `"ffa duel"`) to a bitmask of the `GT_*` gametypes it supports;
/// an empty/`NULL`-content `type` defaults to FFA-only. No oracle: pure
/// string-scan over the `GT_*` enum.
///
/// SAFETY: `type` must be a valid NUL-terminated C string.
pub unsafe fn G_GetMapTypeBits(type_: *mut c_char) -> c_int {
    let mut typeBits: c_int = 0;

    if *type_ != 0 {
        if !strstr(type_, c"ffa".as_ptr()).is_null() {
            typeBits |= 1 << GT_FFA;
            typeBits |= 1 << GT_TEAM;
        }
        if !strstr(type_, c"team".as_ptr()).is_null() {
            typeBits |= 1 << GT_TEAM;
        }
        if !strstr(type_, c"holocron".as_ptr()).is_null() {
            typeBits |= 1 << GT_HOLOCRON;
        }
        if !strstr(type_, c"jedimaster".as_ptr()).is_null() {
            typeBits |= 1 << GT_JEDIMASTER;
        }
        if !strstr(type_, c"duel".as_ptr()).is_null() {
            typeBits |= 1 << GT_DUEL;
            typeBits |= 1 << GT_POWERDUEL;
        }
        if !strstr(type_, c"powerduel".as_ptr()).is_null() {
            typeBits |= 1 << GT_DUEL;
            typeBits |= 1 << GT_POWERDUEL;
        }
        if !strstr(type_, c"siege".as_ptr()).is_null() {
            typeBits |= 1 << GT_SIEGE;
        }
        if !strstr(type_, c"ctf".as_ptr()).is_null() {
            typeBits |= 1 << GT_CTF;
        }
        if !strstr(type_, c"cty".as_ptr()).is_null() {
            typeBits |= 1 << GT_CTY;
        }
    } else {
        typeBits |= 1 << GT_FFA;
    }

    typeBits
}

/// `char *G_GetBotInfoByNumber( int num )` (g_bot.c:1270). Returns the `num`-th
/// parsed bot info string, or `NULL` (with a console warning) if out of range.
/// No oracle: a bounds check + index into the roster static.
///
/// SAFETY: reads the module-static roster; the returned pointer aliases a
/// `G_Alloc`'d info string owned by the roster.
pub unsafe fn G_GetBotInfoByNumber(num: c_int) -> *mut c_char {
    if num < 0 || num >= g_numBots {
        trap::Printf(&format!("{S_COLOR_RED}Invalid bot number: {num}\n"));
        return null_mut();
    }
    g_botInfos[num as usize]
}

/// `char *G_GetBotInfoByName( const char *name )` (g_bot.c:1284). Linear search
/// of the roster for the entry whose `name` key case-insensitively matches
/// `name`; `NULL` if none. No oracle: roster scan over `Info_ValueForKey`.
///
/// SAFETY: `name` must be a valid NUL-terminated C string; reads the roster
/// static.
pub unsafe fn G_GetBotInfoByName(name: *const c_char) -> *mut c_char {
    let mut value: *mut c_char;

    let mut n: c_int = 0;
    while n < g_numBots {
        value = Info_ValueForKey((*addr_of!(g_botInfos))[n as usize], c"name".as_ptr());
        if Q_stricmp(value, name) == 0 {
            return (*addr_of!(g_botInfos))[n as usize];
        }
        n += 1;
    }

    null_mut()
}

/// `const char *G_GetArenaInfoByMap( const char *map )` (g_bot.c:326). Linear
/// search of the arena roster for the entry whose `map` key case-insensitively
/// matches `map`; `NULL` if none. No oracle: roster scan.
///
/// SAFETY: `map` must be a valid NUL-terminated C string; reads the arena
/// roster static.
pub unsafe fn G_GetArenaInfoByMap(map: *const c_char) -> *const c_char {
    let mut n: c_int = 0;
    while n < g_numArenas {
        if Q_stricmp(
            Info_ValueForKey((*addr_of!(g_arenaInfos))[n as usize], c"map".as_ptr()),
            map,
        ) == 0
        {
            return (*addr_of!(g_arenaInfos))[n as usize];
        }
        n += 1;
    }

    null_mut()
}

/// `qboolean G_DoesMapSupportGametype(const char *mapname, int gametype)`
/// (g_bot.c:168). Looks `mapname` up in the arena roster, then tests whether the
/// arena's `type` bitmask (`G_GetMapTypeBits`) includes `gametype`. `qfalse` if
/// the roster is empty, `mapname` is empty, the map isn't in the roster, or it
/// doesn't support the gametype. No oracle: roster scan + bit test.
///
/// SAFETY: `mapname` must be a valid (possibly empty/NULL) C string; reads the
/// arena roster static.
pub unsafe fn G_DoesMapSupportGametype(mapname: *const c_char, gametype: c_int) -> qboolean {
    let typeBits: c_int;
    let mut thisLevel: c_int = -1;
    let mut n: c_int;
    let mut type_: *mut c_char;

    if (*addr_of!(g_arenaInfos))[0].is_null() {
        return QFALSE;
    }

    if mapname.is_null() || *mapname == 0 {
        return QFALSE;
    }

    n = 0;
    while n < g_numArenas {
        type_ = Info_ValueForKey((*addr_of!(g_arenaInfos))[n as usize], c"map".as_ptr());

        if Q_stricmp(mapname, type_) == 0 {
            thisLevel = n;
            break;
        }
        n += 1;
    }

    if thisLevel == -1 {
        return QFALSE;
    }

    type_ = Info_ValueForKey(
        (*addr_of!(g_arenaInfos))[thisLevel as usize],
        c"type".as_ptr(),
    );

    typeBits = G_GetMapTypeBits(type_);
    if typeBits & (1 << gametype) != 0 {
        //the map in question supports the gametype in question, so..
        return QTRUE;
    }

    QFALSE
}

/// `int G_ParseInfos( char *buf, int max, char *infos[] )` (g_bot.c:50). Parses a
/// `.bot`/`.arena` roster text buffer into up to `max` `G_Alloc`'d info strings
/// stored in `infos[]`, returning the count parsed. Each `{ ... }` block becomes
/// one info string of `key value` pairs. No oracle: trap-free token parsing, but
/// it allocates via `G_Alloc` (the level zone) so it's exercised only on a live
/// game.
///
/// SAFETY: `buf` must be a mutable, NUL-terminated parse buffer; `infos` must
/// point at room for `max` entries.
pub unsafe fn G_ParseInfos(buf: *mut c_char, max: c_int, infos: *mut *mut c_char) -> c_int {
    let mut token: *mut c_char;
    let mut count: c_int;
    let mut key = [0 as c_char; MAX_TOKEN_CHARS as usize];
    let mut info = [0 as c_char; MAX_INFO_STRING as usize];

    // C passes `&buf` (a `char **`) to COM_Parse, which advances the pointer.
    let mut buf_p: *const c_char = buf;

    count = 0;

    loop {
        token = COM_Parse(&mut buf_p);
        if *token == 0 {
            break;
        }
        if strcmp(token, c"{".as_ptr()) != 0 {
            Com_Printf("Missing { in info file\n");
            break;
        }

        if count == max {
            Com_Printf("Max infos exceeded\n");
            break;
        }

        info[0] = b'\0' as c_char;
        loop {
            token = COM_ParseExt(&mut buf_p, QTRUE);
            if *token == 0 {
                Com_Printf("Unexpected end of info file\n");
                break;
            }
            if strcmp(token, c"}".as_ptr()) == 0 {
                break;
            }
            Q_strncpyz(
                key.as_mut_ptr(),
                token,
                core::mem::size_of_val(&key) as c_int,
            );

            token = COM_ParseExt(&mut buf_p, QFALSE);
            if *token == 0 {
                strcpy(token, c"<NULL>".as_ptr());
            }
            Info_SetValueForKey(info.as_mut_ptr(), key.as_ptr(), token);
        }
        //NOTE: extra space for arena number
        *infos.add(count as usize) = G_Alloc(
            strlen(info.as_ptr()) as c_int
                + strlen(c"\\num\\".as_ptr()) as c_int
                + strlen(va(format_args!("{}", MAX_ARENAS))) as c_int
                + 1,
        ) as *mut c_char;
        if !(*infos.add(count as usize)).is_null() {
            strcpy(*infos.add(count as usize), info.as_ptr());
            count += 1;
        }
    }
    count
}

/// `static void G_LoadArenasFromFile( char *filename )` (g_bot.c:106). Reads one
/// `.arena` file into a fixed buffer and appends its parsed info blocks to the
/// arena roster via `G_ParseInfos`. Bails (with a colored console message) if the
/// file is missing or too large. No oracle: filesystem trap + roster mutation.
///
/// SAFETY: `filename` must be a valid NUL-terminated C string; mutates the arena
/// roster statics.
unsafe fn G_LoadArenasFromFile(filename: *mut c_char) {
    let len: c_int;
    let f: fileHandle_t;
    let mut buf = [0 as c_char; MAX_ARENAS_TEXT as usize];

    let fname = CStr::from_ptr(filename).to_string_lossy();
    (len, f) = trap::FS_FOpenFile(&fname, FS_READ);
    if f == 0 {
        trap::Printf(&format!("{S_COLOR_RED}file not found: {}\n", Sz(filename)));
        return;
    }
    if len >= MAX_ARENAS_TEXT {
        trap::Printf(&format!(
            "{S_COLOR_RED}file too large: {} is {len}, max allowed is {MAX_ARENAS_TEXT}",
            Sz(filename)
        ));
        trap::FS_FCloseFile(f);
        return;
    }

    let rbuf = core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, len as usize);
    trap::FS_Read(rbuf, f);
    buf[len as usize] = 0;
    trap::FS_FCloseFile(f);

    g_numArenas += G_ParseInfos(
        buf.as_mut_ptr(),
        MAX_ARENAS - g_numArenas,
        addr_of_mut!((*addr_of_mut!(g_arenaInfos))[g_numArenas as usize]),
    );
}

/// `static void G_LoadBotsFromFile( char *filename )` (g_bot.c:1199). Reads one
/// `.bot` file into a fixed buffer and appends its parsed info blocks to the bot
/// roster via `G_ParseInfos`. Bails (with a colored console message) if the file
/// is missing or too large. No oracle: filesystem trap + roster mutation.
///
/// SAFETY: `filename` must be a valid NUL-terminated C string; mutates the bot
/// roster statics.
unsafe fn G_LoadBotsFromFile(filename: *mut c_char) {
    let len: c_int;
    let f: fileHandle_t;
    let mut buf = [0 as c_char; MAX_BOTS_TEXT as usize];

    let fname = CStr::from_ptr(filename).to_string_lossy();
    (len, f) = trap::FS_FOpenFile(&fname, FS_READ);
    if f == 0 {
        trap::Printf(&format!("{S_COLOR_RED}file not found: {}\n", Sz(filename)));
        return;
    }
    if len >= MAX_BOTS_TEXT {
        trap::Printf(&format!(
            "{S_COLOR_RED}file too large: {} is {len}, max allowed is {MAX_BOTS_TEXT}",
            Sz(filename)
        ));
        trap::FS_FCloseFile(f);
        return;
    }

    let rbuf = core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, len as usize);
    trap::FS_Read(rbuf, f);
    buf[len as usize] = 0;
    trap::FS_FCloseFile(f);

    g_numBots += G_ParseInfos(
        buf.as_mut_ptr(),
        MAX_BOTS - g_numBots,
        addr_of_mut!((*addr_of_mut!(g_botInfos))[g_numBots as usize]),
    );
}

/// `const char *G_RefreshNextMap(int gametype, qboolean forced)` (g_bot.c:213).
/// rww — auto-obtain the next map: cycle forward through the arena roster from
/// the current `mapname` to the next arena that supports `gametype`, set
/// `nextmap` to `map <name>` (or `map_restart 0` if no other map qualifies), and
/// return the chosen map name. Honors `g_autoMapCycle` unless `forced`. No
/// oracle: roster scan + cvar traps.
///
/// SAFETY: reads the arena roster + `g_autoMapCycle` statics; uses a local
/// `vmCvar_t`.
pub unsafe fn G_RefreshNextMap(gametype: c_int, forced: qboolean) -> *const c_char {
    let mut typeBits: c_int;
    let mut thisLevel: c_int = 0;
    let mut desiredMap: c_int;
    let mut n: c_int;
    let mut type_: *mut c_char;
    let mut loopingUp: qboolean = QFALSE;
    let mut mapname: vmCvar_t = vmCvar_t::zeroed();

    if (*addr_of!(g_autoMapCycle)).integer == 0 && forced == QFALSE {
        return null_mut();
    }

    if (*addr_of!(g_arenaInfos))[0].is_null() {
        return null_mut();
    }

    trap::Cvar_Register(
        addr_of_mut!(mapname).as_mut(),
        "mapname",
        "",
        CVAR_SERVERINFO | CVAR_ROM,
    );
    n = 0;
    while n < g_numArenas {
        type_ = Info_ValueForKey((*addr_of!(g_arenaInfos))[n as usize], c"map".as_ptr());

        if Q_stricmp(mapname.string.as_ptr(), type_) == 0 {
            thisLevel = n;
            break;
        }
        n += 1;
    }

    desiredMap = thisLevel;

    n = thisLevel + 1;
    while n != thisLevel {
        //now cycle through the arena list and find the next map that matches the gametype we're in
        if (*addr_of!(g_arenaInfos))[n as usize].is_null() || n >= g_numArenas {
            if loopingUp != QFALSE {
                //this shouldn't happen, but if it does we have a null entry break in the arena file
                //if this is the case just break out of the loop instead of sticking in an infinite loop
                break;
            }
            n = 0;
            loopingUp = QTRUE;
        }

        type_ = Info_ValueForKey((*addr_of!(g_arenaInfos))[n as usize], c"type".as_ptr());

        typeBits = G_GetMapTypeBits(type_);
        if typeBits & (1 << gametype) != 0 {
            desiredMap = n;
            break;
        }

        n += 1;
    }

    if desiredMap == thisLevel {
        //If this is the only level for this game mode or we just can't find a map for this game mode, then nextmap
        //will always restart.
        trap::Cvar_Set("nextmap", "map_restart 0");
    } else {
        //otherwise we have a valid nextmap to cycle to, so use it.
        type_ = Info_ValueForKey(
            (*addr_of!(g_arenaInfos))[desiredMap as usize],
            c"map".as_ptr(),
        );
        trap::Cvar_Set(
            "nextmap",
            &CStr::from_ptr(va(format_args!("map {}", Sz(type_)))).to_string_lossy(),
        );
    }

    Info_ValueForKey(
        (*addr_of!(g_arenaInfos))[desiredMap as usize],
        c"map".as_ptr(),
    )
}

/// `static void G_LoadArenas( void )` (g_bot.c:292). Resets the arena roster, then
/// loads every `scripts/*.arena` file into it, stamps each entry's `num` key, and
/// primes `nextmap` via `G_RefreshNextMap`. No oracle: filesystem trap + roster
/// mutation.
///
/// SAFETY: mutates the arena roster statics.
unsafe fn G_LoadArenas() {
    let numdirs: c_int;
    let mut filename = [0 as c_char; 128];
    let mut dirlist = [0 as c_char; 1024];
    let mut dirptr: *mut c_char;
    let mut i: c_int;
    let mut n: c_int;

    g_numArenas = 0;

    // get all arenas from .arena files
    // C: `for (i=0; i<numdirs; i++, dirptr += dirlen+1)` with `dirlen` updated at
    // the top of the body. The final post-increment's `dirptr += dirlen+1` is a
    // dead store in C; we fold the advance into the loop body, scoping `dirlen`
    // to it so the trailing write doesn't read as a dead assignment.
    numdirs = trap::FS_GetFileList("scripts", ".arena", &mut dirlist);
    dirptr = dirlist.as_mut_ptr();
    i = 0;
    while i < numdirs {
        let dirlen = strlen(dirptr) as c_int;
        strcpy(filename.as_mut_ptr(), c"scripts/".as_ptr());
        strcat(filename.as_mut_ptr(), dirptr);
        G_LoadArenasFromFile(filename.as_mut_ptr());
        dirptr = dirptr.add((dirlen + 1) as usize);
        i += 1;
    }
    //	trap_Printf( va( "%i arenas parsed\n", g_numArenas ) );

    n = 0;
    while n < g_numArenas {
        Info_SetValueForKey(
            (*addr_of_mut!(g_arenaInfos))[n as usize],
            c"num".as_ptr(),
            va(format_args!("{n}")),
        );
        n += 1;
    }

    G_RefreshNextMap((*addr_of!(g_gametype)).integer, QFALSE);
}

/// `static void G_LoadBots( void )` (g_bot.c:1227). When `bot_enable` is set,
/// resets the bot roster and loads the bot definitions: from the `g_botsFile`
/// cvar if set, else `botfiles/bots.txt`, plus every `scripts/*.bot` file. No
/// oracle: cvar/filesystem traps + roster mutation.
///
/// SAFETY: mutates the bot roster statics; uses a local `vmCvar_t`.
unsafe fn G_LoadBots() {
    let mut botsFile: vmCvar_t = vmCvar_t::zeroed();
    let numdirs: c_int;
    let mut filename = [0 as c_char; 128];
    let mut dirlist = [0 as c_char; 1024];
    let mut dirptr: *mut c_char;
    let mut i: c_int;

    if trap::Cvar_VariableIntegerValue("bot_enable") == 0 {
        return;
    }

    g_numBots = 0;

    trap::Cvar_Register(
        addr_of_mut!(botsFile).as_mut(),
        "g_botsFile",
        "",
        CVAR_INIT | CVAR_ROM,
    );
    if botsFile.string[0] != 0 {
        G_LoadBotsFromFile(botsFile.string.as_mut_ptr());
    } else {
        //G_LoadBotsFromFile("scripts/bots.txt");
        G_LoadBotsFromFile(c"botfiles/bots.txt".as_ptr() as *mut c_char);
    }

    // get all bots from .bot files
    numdirs = trap::FS_GetFileList("scripts", ".bot", &mut dirlist);
    dirptr = dirlist.as_mut_ptr();
    i = 0;
    while i < numdirs {
        let dirlen = strlen(dirptr) as c_int;
        strcpy(filename.as_mut_ptr(), c"scripts/".as_ptr());
        strcat(filename.as_mut_ptr(), dirptr);
        G_LoadBotsFromFile(filename.as_mut_ptr());
        dirptr = dirptr.add((dirlen + 1) as usize);
        i += 1;
    }
    //	trap_Printf( va( "%i bots parsed\n", g_numBots ) );
}

/// `int G_CountHumanPlayers( int team )` (g_bot.c:496). Counts connected,
/// non-bot clients on `team` (or all teams when `team < 0`). No oracle: scans the
/// live client/entity arrays.
///
/// SAFETY: reads `level.clients`, `g_entities`, and `g_maxclients`; valid only
/// once the client array is allocated.
pub unsafe fn G_CountHumanPlayers(team: c_int) -> c_int {
    let mut num: c_int;
    let mut cl: *mut gclient_t;

    num = 0;
    let mut i: c_int = 0;
    while i < (*addr_of!(g_maxclients)).integer {
        cl = (*addr_of!(level)).clients.add(i as usize);
        if (*cl).pers.connected != CON_CONNECTED {
            i += 1;
            continue;
        }
        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*cl).ps.clientNum as usize))
        .r
        .svFlags
            & SVF_BOT
            != 0
        {
            i += 1;
            continue;
        }
        if team >= 0 && (*cl).sess.sessionTeam != team {
            i += 1;
            continue;
        }
        num += 1;
        i += 1;
    }
    num
}

/// `int G_CountBotPlayers( int team )` (g_bot.c:522). Counts connected bots on
/// `team` (Siege uses `siegeDesiredTeam`, else `sessionTeam`; `team < 0` counts
/// all), plus any bots already queued to spawn whose delay has elapsed. No
/// oracle: scans the live client/entity arrays and the spawn queue.
///
/// SAFETY: reads `level`, `g_entities`, `g_maxclients`, and the spawn-queue
/// static.
pub unsafe fn G_CountBotPlayers(team: c_int) -> c_int {
    let mut num: c_int;
    let mut cl: *mut gclient_t;

    num = 0;
    let mut i: c_int = 0;
    while i < (*addr_of!(g_maxclients)).integer {
        cl = (*addr_of!(level)).clients.add(i as usize);
        if (*cl).pers.connected != CON_CONNECTED {
            i += 1;
            continue;
        }
        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*cl).ps.clientNum as usize))
        .r
        .svFlags
            & SVF_BOT
            == 0
        {
            i += 1;
            continue;
        }
        if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            if team >= 0 && (*cl).sess.siegeDesiredTeam != team {
                i += 1;
                continue;
            }
        } else if team >= 0 && (*cl).sess.sessionTeam != team {
            i += 1;
            continue;
        }
        num += 1;
        i += 1;
    }
    let mut n: c_int = 0;
    while (n as usize) < BOT_SPAWN_QUEUE_DEPTH {
        if (*addr_of!(botSpawnQueue))[n as usize].spawnTime == 0 {
            n += 1;
            continue;
        }
        if (*addr_of!(botSpawnQueue))[n as usize].spawnTime > (*addr_of!(level)).time {
            n += 1;
            continue;
        }
        num += 1;
        n += 1;
    }
    num
}

/// `void G_RemoveQueuedBotBegin( int clientNum )` (g_bot.c:756). Called on client
/// disconnect to cancel a pending delayed spawn so it doesn't fire on a freed
/// index. No oracle: spawn-queue scan.
///
/// SAFETY: mutates the spawn-queue static.
pub unsafe fn G_RemoveQueuedBotBegin(clientNum: c_int) {
    let mut n: c_int = 0;
    while (n as usize) < BOT_SPAWN_QUEUE_DEPTH {
        if (*addr_of!(botSpawnQueue))[n as usize].clientNum == clientNum {
            (*addr_of_mut!(botSpawnQueue))[n as usize].spawnTime = 0;
            return;
        }
        n += 1;
    }
}

/// `static void AddBotToSpawnQueue( int clientNum, int delay )` (g_bot.c:732).
/// Schedules `clientNum` to begin `delay` ms from now in the first free
/// spawn-queue slot; if the queue is full, warns and begins immediately. No
/// oracle: spawn-queue mutation + `ClientBegin`.
///
/// SAFETY: mutates the spawn-queue static; reads `level.time`.
unsafe fn AddBotToSpawnQueue(clientNum: c_int, delay: c_int) {
    let mut n: c_int = 0;
    while (n as usize) < BOT_SPAWN_QUEUE_DEPTH {
        if (*addr_of!(botSpawnQueue))[n as usize].spawnTime == 0 {
            (*addr_of_mut!(botSpawnQueue))[n as usize].spawnTime = (*addr_of!(level)).time + delay;
            (*addr_of_mut!(botSpawnQueue))[n as usize].clientNum = clientNum;
            return;
        }
        n += 1;
    }

    G_Printf(&format!("{S_COLOR_YELLOW}Unable to delay spawn\n"));
    ClientBegin(clientNum, QFALSE);
}

/// `float trap_Cvar_VariableValue( const char *var_name )` (g_bot.c:36). A
/// module-local convenience wrapper (defined inside the bg `namespace_begin`):
/// read the cvar's string value into a buffer and `atof` it. No oracle: cvar
/// trap + libc `atof`. Named without the `trap_` prefix per the trap-wrapper
/// convention, though it lives in g_bot rather than the trap layer (it is a
/// game-side helper in C).
///
// =====================================================================================
// `float trap_Cvar_VariableValue( const char *var_name )` — g_bot.c:36
// -------------------------------------------------------------------------------------
// FALSE-POSITIVE (wiring dup). `ported_index.py` flags `trap_Cvar_VariableValue` as
// missing because the port drops the `trap_` prefix per the trap-wrapper naming
// convention. It IS ported, live, as `Cvar_VariableValue` immediately below — same body
// (fill a 128-byte buffer from the cvar string trap, then `atof`). The C source is a
// module-local helper defined inside the bg `namespace_begin`, hence the `trap_` prefix
// despite living in g_bot rather than the trap layer. Faithful translation of the C body,
// for 1:1-file self-documentation only (the live version is `Cvar_VariableValue`):
//
// pub unsafe fn trap_Cvar_VariableValue(var_name: *const c_char) -> f32 {
//     let mut buf = [0 as c_char; 128];
//     trap_Cvar_VariableStringBuffer(var_name, buf.as_mut_ptr(), buf.len() as c_int);
//     atof(buf.as_ptr()) as f32
// }
// =====================================================================================

/// SAFETY: calls the cvar string trap and libc `atof`.
pub unsafe fn Cvar_VariableValue(var_name: &str) -> f32 {
    let mut buf = [0 as c_char; 128];

    let s = trap::Cvar_VariableString(var_name);
    // Copy into the fixed 128-byte buffer exactly as the C `buf[128]` would be
    // filled by trap_Cvar_VariableStringBuffer (truncating to fit), then atof.
    let bytes = s.as_bytes();
    let n = bytes.len().min(buf.len() - 1);
    for i in 0..n {
        buf[i] = bytes[i] as c_char;
    }
    buf[n] = 0;
    atof(buf.as_ptr()) as f32
}

/// `int G_RemoveRandomBot( int team )` (g_bot.c:458). Kicks the first connected
/// bot on `team` (Siege uses `siegeDesiredTeam`, else `sessionTeam`; `team < 0`
/// matches any), returning `qtrue` if one was kicked. No oracle: scans the live
/// client/entity arrays and issues a `kick` console command.
///
/// SAFETY: reads `level`, `g_entities`, `g_maxclients`; issues a console trap.
pub unsafe fn G_RemoveRandomBot(team: c_int) -> c_int {
    let mut netname = [0 as c_char; 36];
    let mut cl: *mut gclient_t;

    let mut i: c_int = 0;
    while i < (*addr_of!(g_maxclients)).integer {
        cl = (*addr_of!(level)).clients.add(i as usize);
        if (*cl).pers.connected != CON_CONNECTED {
            i += 1;
            continue;
        }
        if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*cl).ps.clientNum as usize))
        .r
        .svFlags
            & SVF_BOT
            == 0
        {
            i += 1;
            continue;
        }
        if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            if team >= 0 && (*cl).sess.siegeDesiredTeam != team {
                i += 1;
                continue;
            }
        } else if team >= 0 && (*cl).sess.sessionTeam != team {
            i += 1;
            continue;
        }
        strcpy(netname.as_mut_ptr(), (*cl).pers.netname.as_ptr());
        Q_CleanStr(netname.as_mut_ptr());
        trap::SendConsoleCommand(
            EXEC_INSERT,
            &CStr::from_ptr(va(format_args!("kick \"{}\"\n", Sz(netname.as_ptr()))))
                .to_string_lossy(),
        );
        return QTRUE;
    }
    QFALSE
}

/// `void G_AddRandomBot( int team )` (g_bot.c:370). Picks a roster bot not already
/// in the game on `team` (Siege uses `siegeDesiredTeam`, else `sessionTeam`;
/// `team < 0` matches any) and issues an `addbot` console command for it at the
/// `g_spSkill` skill. Two passes: count eligible bots, then `random()`-select one.
/// No oracle: scans the live arrays + roster and issues a console command.
///
/// SAFETY: reads `level`, `g_entities`, `g_maxclients`, and the bot roster.
pub unsafe fn G_AddRandomBot(team: c_int) {
    let mut i: c_int;
    let mut n: c_int;
    let mut num: c_int;
    let skill: f32;
    let mut value: *mut c_char;
    let mut netname = [0 as c_char; 36];
    let teamstr: *const c_char;
    let mut cl: *mut gclient_t;

    num = 0;
    n = 0;
    while n < g_numBots {
        value = Info_ValueForKey((*addr_of!(g_botInfos))[n as usize], c"name".as_ptr());
        //
        i = 0;
        while i < (*addr_of!(g_maxclients)).integer {
            cl = (*addr_of!(level)).clients.add(i as usize);
            if (*cl).pers.connected != CON_CONNECTED {
                i += 1;
                continue;
            }
            if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*cl).ps.clientNum as usize))
            .r
            .svFlags
                & SVF_BOT
                == 0
            {
                i += 1;
                continue;
            }
            if (*addr_of!(g_gametype)).integer == GT_SIEGE {
                if team >= 0 && (*cl).sess.siegeDesiredTeam != team {
                    i += 1;
                    continue;
                }
            } else if team >= 0 && (*cl).sess.sessionTeam != team {
                i += 1;
                continue;
            }
            if Q_stricmp(value, (*cl).pers.netname.as_ptr()) == 0 {
                break;
            }
            i += 1;
        }
        if i >= (*addr_of!(g_maxclients)).integer {
            num += 1;
        }
        n += 1;
    }
    num = (random() * num as f32) as c_int;
    n = 0;
    while n < g_numBots {
        value = Info_ValueForKey((*addr_of!(g_botInfos))[n as usize], c"name".as_ptr());
        //
        i = 0;
        while i < (*addr_of!(g_maxclients)).integer {
            cl = (*addr_of!(level)).clients.add(i as usize);
            if (*cl).pers.connected != CON_CONNECTED {
                i += 1;
                continue;
            }
            if (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*cl).ps.clientNum as usize))
            .r
            .svFlags
                & SVF_BOT
                == 0
            {
                i += 1;
                continue;
            }
            if (*addr_of!(g_gametype)).integer == GT_SIEGE {
                if team >= 0 && (*cl).sess.siegeDesiredTeam != team {
                    i += 1;
                    continue;
                }
            } else if team >= 0 && (*cl).sess.sessionTeam != team {
                i += 1;
                continue;
            }
            if Q_stricmp(value, (*cl).pers.netname.as_ptr()) == 0 {
                break;
            }
            i += 1;
        }
        if i >= (*addr_of!(g_maxclients)).integer {
            num -= 1;
            if num <= 0 {
                skill = Cvar_VariableValue("g_spSkill");
                if team == TEAM_RED {
                    teamstr = c"red".as_ptr();
                } else if team == TEAM_BLUE {
                    teamstr = c"blue".as_ptr();
                } else {
                    teamstr = c"".as_ptr();
                }
                strncpy(netname.as_mut_ptr(), value, netname.len() - 1);
                netname[netname.len() - 1] = b'\0' as c_char;
                Q_CleanStr(netname.as_mut_ptr());
                trap::SendConsoleCommand(
                    EXEC_INSERT,
                    &CStr::from_ptr(va(format_args!(
                        "addbot \"{}\" {:.6} {} {}\n",
                        Sz(netname.as_ptr()),
                        skill,
                        Sz(teamstr),
                        0
                    )))
                    .to_string_lossy(),
                );
                return;
            }
        }
        n += 1;
    }
}

/// `void G_CheckMinimumPlayers( void )` (g_bot.c:568). Throttled to once per 10s,
/// keeps the live player count near `bot_minplayers` by adding/removing random
/// bots (skipped in Siege and during intermission). The C body's per-gametype
/// `/* ... */` block is commented out upstream and is not ported. No oracle:
/// reads/writes the throttle + cvar and drives the bot add/remove helpers.
///
/// SAFETY: reads `level`, `g_maxclients`, and the `bot_minplayers` cvar.
pub unsafe fn G_CheckMinimumPlayers() {
    let mut minplayers: c_int;
    let humanplayers: c_int;
    let botplayers: c_int;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        return;
    }

    if (*addr_of!(level)).intermissiontime != 0 {
        return;
    }
    //only check once each 10 seconds
    if (*addr_of!(checkminimumplayers_time)) > (*addr_of!(level)).time - 10000 {
        return;
    }
    *addr_of_mut!(checkminimumplayers_time) = (*addr_of!(level)).time;
    trap::Cvar_Update(&mut *addr_of_mut!(bot_minplayers));
    minplayers = (*addr_of!(bot_minplayers)).integer;
    if minplayers <= 0 {
        return;
    }

    if minplayers > (*addr_of!(g_maxclients)).integer {
        minplayers = (*addr_of!(g_maxclients)).integer;
    }

    humanplayers = G_CountHumanPlayers(-1);
    botplayers = G_CountBotPlayers(-1);

    if (humanplayers + botplayers) < minplayers {
        G_AddRandomBot(-1);
    } else if (humanplayers + botplayers) > minplayers && botplayers != 0 {
        // try to remove spectators first
        if G_RemoveRandomBot(TEAM_SPECTATOR) == 0 {
            // just remove the bot that is playing
            G_RemoveRandomBot(-1);
        }
    }
}

/// `void G_CheckBotSpawn( void )` (g_bot.c:702). Per-frame: tops up min players,
/// then begins any queued bots whose delay has elapsed, clearing their slots. The
/// `GT_SINGLE_PLAYER` intro-sound `/* ... */` block is commented out upstream and
/// is not ported. No oracle: spawn-queue drain + `ClientBegin`.
///
/// SAFETY: reads `level.time` and mutates the spawn-queue static.
pub unsafe fn G_CheckBotSpawn() {
    G_CheckMinimumPlayers();

    let mut n: c_int = 0;
    while (n as usize) < BOT_SPAWN_QUEUE_DEPTH {
        if (*addr_of!(botSpawnQueue))[n as usize].spawnTime == 0 {
            n += 1;
            continue;
        }
        if (*addr_of!(botSpawnQueue))[n as usize].spawnTime > (*addr_of!(level)).time {
            n += 1;
            continue;
        }
        ClientBegin((*addr_of!(botSpawnQueue))[n as usize].clientNum, QFALSE);
        (*addr_of_mut!(botSpawnQueue))[n as usize].spawnTime = 0;

        /*
        if( g_gametype.integer == GT_SINGLE_PLAYER ) {
            trap_GetUserinfo( botSpawnQueue[n].clientNum, userinfo, sizeof(userinfo) );
            PlayerIntroSound( Info_ValueForKey (userinfo, "model") );
        }
        */
        n += 1;
    }
}

/// `void Svcmd_BotList_f( void )` (g_bot.c:1105). Prints the parsed bot roster
/// (name / model / personality / funname), substituting defaults for empty
/// fields. No oracle: roster scan + console print.
///
/// SAFETY: reads the bot roster static.
pub unsafe fn Svcmd_BotList_f() {
    let mut name = [0 as c_char; MAX_TOKEN_CHARS as usize];
    let mut funname = [0 as c_char; MAX_TOKEN_CHARS as usize];
    let mut model = [0 as c_char; MAX_TOKEN_CHARS as usize];
    let mut personality = [0 as c_char; MAX_TOKEN_CHARS as usize];

    trap::Printf("^1name             model            personality              funname\n");
    let mut i: c_int = 0;
    while i < g_numBots {
        strcpy(
            name.as_mut_ptr(),
            Info_ValueForKey((*addr_of!(g_botInfos))[i as usize], c"name".as_ptr()),
        );
        if name[0] == 0 {
            strcpy(name.as_mut_ptr(), c"Padawan".as_ptr());
        }
        strcpy(
            funname.as_mut_ptr(),
            Info_ValueForKey((*addr_of!(g_botInfos))[i as usize], c"funname".as_ptr()),
        );
        if funname[0] == 0 {
            strcpy(funname.as_mut_ptr(), c"".as_ptr());
        }
        strcpy(
            model.as_mut_ptr(),
            Info_ValueForKey((*addr_of!(g_botInfos))[i as usize], c"model".as_ptr()),
        );
        if model[0] == 0 {
            strcpy(model.as_mut_ptr(), c"kyle/default".as_ptr());
        }
        strcpy(
            personality.as_mut_ptr(),
            Info_ValueForKey((*addr_of!(g_botInfos))[i as usize], c"personality".as_ptr()),
        );
        if personality[0] == 0 {
            strcpy(personality.as_mut_ptr(), c"botfiles/kyle.jkb".as_ptr());
        }
        trap::Printf(
            &CStr::from_ptr(va(format_args!(
                "{:<16} {:<16} {:<20} {:<20}\n",
                Sz(name.as_ptr()),
                Sz(model.as_ptr()),
                Sz(personality.as_ptr()),
                Sz(funname.as_ptr())
            )))
            .to_string_lossy(),
        );
        i += 1;
    }
}

/// `static void G_AddBot( const char *name, float skill, const char *team, int delay, char *altname)`
/// (g_bot.c:797). Builds a bot's userinfo from its `bots.txt` definition,
/// allocates a client slot, connects it as a normal client, assigns its team
/// (with duel/powerduel/siege special-casing), and either begins it immediately
/// or queues a delayed spawn. No oracle: full client-connect path over the live
/// entity/client state + traps.
///
/// SAFETY: reads/writes the entity array and client state; issues client +
/// userinfo traps.
unsafe fn G_AddBot(
    name: *const c_char,
    skill: f32,
    mut team: *const c_char,
    delay: c_int,
    altname: *mut c_char,
) {
    let clientNum: c_int;
    let botinfo: *mut c_char;
    let bot: *mut gentity_t;
    let mut key: *const c_char;
    let mut s: *mut c_char;
    let mut botname: *mut c_char;
    let mut model: *mut c_char;
    //	char			*headmodel;
    let mut userinfo = [0 as c_char; MAX_INFO_STRING];
    let preTeam: c_int;

    // get the botinfo from bots.txt
    botinfo = G_GetBotInfoByName(name);
    if botinfo.is_null() {
        G_Printf(&format!(
            "{S_COLOR_RED}Error: Bot '{}' not defined\n",
            Sz(name)
        ));
        return;
    }

    // create the bot's userinfo
    userinfo[0] = b'\0' as c_char;

    botname = Info_ValueForKey(botinfo, c"funname".as_ptr());
    if botname.is_null() || *botname == 0 {
        botname = Info_ValueForKey(botinfo, c"name".as_ptr());
    }
    // check for an alternative name
    if !altname.is_null() && *altname != 0 {
        botname = altname;
    }
    Info_SetValueForKey(userinfo.as_mut_ptr(), c"name".as_ptr(), botname);
    Info_SetValueForKey(userinfo.as_mut_ptr(), c"rate".as_ptr(), c"25000".as_ptr());
    Info_SetValueForKey(userinfo.as_mut_ptr(), c"snaps".as_ptr(), c"20".as_ptr());
    Info_SetValueForKey(
        userinfo.as_mut_ptr(),
        c"skill".as_ptr(),
        va(format_args!("{skill:.2}")),
    );

    if (1.0..2.0).contains(&skill) {
        Info_SetValueForKey(userinfo.as_mut_ptr(), c"handicap".as_ptr(), c"50".as_ptr());
    } else if (2.0..3.0).contains(&skill) {
        Info_SetValueForKey(userinfo.as_mut_ptr(), c"handicap".as_ptr(), c"70".as_ptr());
    } else if (3.0..4.0).contains(&skill) {
        Info_SetValueForKey(userinfo.as_mut_ptr(), c"handicap".as_ptr(), c"90".as_ptr());
    }

    key = c"model".as_ptr();
    model = Info_ValueForKey(botinfo, key);
    if *model == 0 {
        model = c"kyle/default".as_ptr() as *mut c_char;
    }
    Info_SetValueForKey(userinfo.as_mut_ptr(), key, model);

    /*	key = "headmodel";
        headmodel = Info_ValueForKey( botinfo, key );
        if ( !*headmodel ) {
            headmodel = model;
        }
        Info_SetValueForKey( userinfo, key, headmodel );
        key = "team_headmodel";
        Info_SetValueForKey( userinfo, key, headmodel );
    */
    key = c"gender".as_ptr();
    s = Info_ValueForKey(botinfo, key);
    if *s == 0 {
        s = c"male".as_ptr() as *mut c_char;
    }
    Info_SetValueForKey(userinfo.as_mut_ptr(), c"sex".as_ptr(), s);

    key = c"color1".as_ptr();
    s = Info_ValueForKey(botinfo, key);
    if *s == 0 {
        s = c"4".as_ptr() as *mut c_char;
    }
    Info_SetValueForKey(userinfo.as_mut_ptr(), key, s);

    key = c"color2".as_ptr();
    s = Info_ValueForKey(botinfo, key);
    if *s == 0 {
        s = c"4".as_ptr() as *mut c_char;
    }
    Info_SetValueForKey(userinfo.as_mut_ptr(), key, s);

    key = c"saber1".as_ptr();
    s = Info_ValueForKey(botinfo, key);
    if *s == 0 {
        s = c"single_1".as_ptr() as *mut c_char;
    }
    Info_SetValueForKey(userinfo.as_mut_ptr(), key, s);

    key = c"saber2".as_ptr();
    s = Info_ValueForKey(botinfo, key);
    if *s == 0 {
        s = c"none".as_ptr() as *mut c_char;
    }
    Info_SetValueForKey(userinfo.as_mut_ptr(), key, s);

    s = Info_ValueForKey(botinfo, c"personality".as_ptr());
    if *s == 0 {
        Info_SetValueForKey(
            userinfo.as_mut_ptr(),
            c"personality".as_ptr(),
            c"botfiles/default.jkb".as_ptr(),
        );
    } else {
        Info_SetValueForKey(userinfo.as_mut_ptr(), c"personality".as_ptr(), s);
    }

    // have the server allocate a client slot
    clientNum = trap::BotAllocateClient();
    if clientNum == -1 {
        //		G_Printf( S_COLOR_RED "Unable to add bot.  All player slots are in use.\n" );
        //		G_Printf( S_COLOR_RED "Start server with more 'open' slots.\n" );
        trap::SendServerCommand(
            -1,
            &CStr::from_ptr(va(format_args!(
                "print \"{}\n\"",
                Sz(G_GetStringEdString(
                    c"MP_SVGAME".as_ptr(),
                    c"UNABLE_TO_ADD_BOT".as_ptr()
                ))
            )))
            .to_string_lossy(),
        );
        return;
    }

    // initialize the bot settings
    if team.is_null() || *team == 0 {
        if (*addr_of!(g_gametype)).integer >= GT_TEAM {
            if PickTeam(clientNum) == TEAM_RED {
                team = c"red".as_ptr();
            } else {
                team = c"blue".as_ptr();
            }
        } else {
            team = c"red".as_ptr();
        }
    }
    //	Info_SetValueForKey( userinfo, "characterfile", Info_ValueForKey( botinfo, "aifile" ) );
    Info_SetValueForKey(
        userinfo.as_mut_ptr(),
        c"skill".as_ptr(),
        va(format_args!("{skill:5.2}")),
    );
    Info_SetValueForKey(userinfo.as_mut_ptr(), c"team".as_ptr(), team);

    bot = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(clientNum as usize);
    (*bot).r.svFlags |= SVF_BOT;
    (*bot).inuse = QTRUE;

    // register the userinfo
    trap::SetUserinfo(
        clientNum,
        &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
    );

    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        if !team.is_null() && Q_stricmp(team, c"red".as_ptr()) == 0 {
            (*(*bot).client).sess.sessionTeam = TEAM_RED;
        } else if !team.is_null() && Q_stricmp(team, c"blue".as_ptr()) == 0 {
            (*(*bot).client).sess.sessionTeam = TEAM_BLUE;
        } else {
            (*(*bot).client).sess.sessionTeam = PickTeam(-1);
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE {
        (*(*bot).client).sess.siegeDesiredTeam = (*(*bot).client).sess.sessionTeam;
        (*(*bot).client).sess.sessionTeam = TEAM_SPECTATOR;
    }

    preTeam = (*(*bot).client).sess.sessionTeam;

    // have it connect to the game as a normal client
    if !ClientConnect(clientNum, QTRUE, QTRUE).is_null() {
        return;
    }

    if (*(*bot).client).sess.sessionTeam != preTeam {
        {
            let info = trap::GetUserinfo(clientNum);
            let bytes = info.as_bytes();
            let n = bytes.len().min(MAX_INFO_STRING - 1);
            for k in 0..n {
                userinfo[k] = bytes[k] as c_char;
            }
            userinfo[n] = 0;
        }

        if (*(*bot).client).sess.sessionTeam == TEAM_SPECTATOR {
            (*(*bot).client).sess.sessionTeam = preTeam;
        }

        if (*(*bot).client).sess.sessionTeam == TEAM_RED {
            team = c"Red".as_ptr();
        } else if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            if (*(*bot).client).sess.sessionTeam == TEAM_BLUE {
                team = c"Blue".as_ptr();
            } else {
                team = c"s".as_ptr();
            }
        } else {
            team = c"Blue".as_ptr();
        }

        Info_SetValueForKey(userinfo.as_mut_ptr(), c"team".as_ptr(), team);

        trap::SetUserinfo(
            clientNum,
            &CStr::from_ptr(userinfo.as_ptr()).to_string_lossy(),
        );

        (*(*bot).client).ps.persistant[PERS_TEAM as usize] = (*(*bot).client).sess.sessionTeam;

        G_ReadSessionData((*bot).client);
        ClientUserinfoChanged(clientNum);
    }

    if (*addr_of!(g_gametype)).integer == GT_DUEL || (*addr_of!(g_gametype)).integer == GT_POWERDUEL
    {
        let mut loners: c_int = 0;
        let mut doubles: c_int = 0;

        (*(*bot).client).sess.duelTeam = 0;
        G_PowerDuelCount(&mut loners, &mut doubles, QTRUE);

        if doubles == 0 || loners > (doubles / 2) {
            (*(*bot).client).sess.duelTeam = DUELTEAM_DOUBLE;
        } else {
            (*(*bot).client).sess.duelTeam = DUELTEAM_LONE;
        }

        (*(*bot).client).sess.sessionTeam = TEAM_SPECTATOR;
        SetTeam(bot, c"s".as_ptr() as *mut c_char);
    } else {
        if delay == 0 {
            ClientBegin(clientNum, QFALSE);
            return;
        }

        AddBotToSpawnQueue(clientNum, delay);
    }
}

/// `void Svcmd_AddBot_f( void )` (g_bot.c:1046). The `addbot` server command:
/// parse `<botname> [skill] [team] [delay] [altname]` from the console args and
/// hand off to `G_AddBot` (no-op if `bot_enable` is off). When issued mid-game on
/// a local server, also forces a deferred media load. No oracle: cvar/arg traps +
/// `G_AddBot`.
///
/// SAFETY: reads console args + cvars and the `level` global.
pub unsafe fn Svcmd_AddBot_f() {
    let skill: f32;
    let delay: c_int;
    let mut name = [0 as c_char; MAX_TOKEN_CHARS as usize];
    let mut altname = [0 as c_char; MAX_TOKEN_CHARS as usize];
    let mut string = [0 as c_char; MAX_TOKEN_CHARS as usize];
    let mut team = [0 as c_char; MAX_TOKEN_CHARS as usize];

    // trap_Argv( n, buf, size ): fill a fixed C buffer from console arg `n`.
    unsafe fn argv(n: c_int, buf: &mut [c_char]) {
        let s = trap::Argv(n);
        let bytes = s.as_bytes();
        let cnt = bytes.len().min(buf.len() - 1);
        for k in 0..cnt {
            buf[k] = bytes[k] as c_char;
        }
        buf[cnt] = 0;
    }

    // are bots enabled?
    if trap::Cvar_VariableIntegerValue("bot_enable") == 0 {
        return;
    }

    // name
    argv(1, &mut name);
    if name[0] == 0 {
        trap::Printf("Usage: Addbot <botname> [skill 1-5] [team] [msec delay] [altname]\n");
        return;
    }

    // skill
    argv(2, &mut string);
    if string[0] == 0 {
        skill = 4.0;
    } else {
        skill = atof(string.as_ptr()) as f32;
    }

    // team
    argv(3, &mut team);

    // delay
    argv(4, &mut string);
    if string[0] == 0 {
        delay = 0;
    } else {
        delay = atoi(string.as_ptr());
    }

    // alternative name
    argv(5, &mut altname);

    G_AddBot(
        name.as_ptr(),
        skill,
        team.as_ptr(),
        delay,
        altname.as_mut_ptr(),
    );

    // if this was issued during gameplay and we are playing locally,
    // go ahead and load the bot's media immediately
    if (*addr_of!(level)).time - (*addr_of!(level)).startTime > 1000
        && trap::Cvar_VariableIntegerValue("cl_running") != 0
    {
        trap::SendServerCommand(-1, "loaddefered\n"); // FIXME: spelled wrong, but not changing for demo
    }
}

/// `void G_InitBots( qboolean restart )` (g_bot.c:1307). GAME_INIT-time roster
/// bring-up: load the bot + arena rosters, register the `bot_minplayers` cvar,
/// then (rww) prime the level's bot route via `LoadPath_ThisLevel`. The `restart`
/// argument is unused in the C body (kept for ABI parity). No oracle: drives the
/// loaders + a cvar register + the route loader.
///
/// SAFETY: mutates the roster + `bot_minplayers` statics and the route state.
pub unsafe fn G_InitBots(_restart: qboolean) {
    G_LoadBots();
    G_LoadArenas();

    trap::Cvar_Register(
        addr_of_mut!(bot_minplayers).as_mut(),
        "bot_minplayers",
        "0",
        CVAR_SERVERINFO,
    );

    //rww - new bot route stuff
    LoadPath_ThisLevel();
    //end rww
}

/// `qboolean G_BotConnect( int clientNum, qboolean restart )` (g_bot.c:773) — the bot half of
/// `GAME_CLIENT_CONNECT`: read the client's userinfo, pull `personality`/`skill`/`team` into a
/// [`bot_settings_t`], and hand them to [`BotAISetupClient`]. Drops the client if setup fails.
/// No oracle: `trap_GetUserinfo`/`trap_DropClient` syscalls + the botlib client bring-up.
///
/// # Safety
/// `clientNum` must be a valid client slot.
pub unsafe fn G_BotConnect(clientNum: c_int, restart: qboolean) -> qboolean {
    let mut settings: bot_settings_t = core::mem::zeroed();
    let mut userinfo = [0 as c_char; MAX_INFO_STRING as usize];

    // trap_GetUserinfo( clientNum, userinfo, sizeof(userinfo) );
    {
        let info = trap::GetUserinfo(clientNum);
        let bytes = info.as_bytes();
        let n = bytes.len().min(MAX_INFO_STRING as usize - 1);
        for k in 0..n {
            userinfo[k] = bytes[k] as c_char;
        }
        userinfo[n] = 0;
    }

    Q_strncpyz(
        settings.personalityfile.as_mut_ptr(),
        Info_ValueForKey(userinfo.as_ptr(), c"personality".as_ptr()),
        settings.personalityfile.len() as c_int,
    );
    settings.skill = atof(Info_ValueForKey(userinfo.as_ptr(), c"skill".as_ptr())) as f32;
    Q_strncpyz(
        settings.team.as_mut_ptr(),
        Info_ValueForKey(userinfo.as_ptr(), c"team".as_ptr()),
        settings.team.len() as c_int,
    );

    if BotAISetupClient(clientNum, &mut settings, restart) == 0 {
        trap::DropClient(clientNum, "BotAISetupClient failed");
        return QFALSE;
    }

    QTRUE
}
