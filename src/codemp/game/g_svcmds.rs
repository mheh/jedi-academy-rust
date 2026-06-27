//! `g_svcmds.c` — server-console commands (not reachable by remote clients).
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_svcmds.c`, incrementally as consumers
//! need it (the same lazy strategy used for `g_session.c` / `g_utils.c` / the
//! `trap_*` surface). This first slice is the **packet-filtering / IP-ban family**
//! that `G_InitGame` pulls in through its prologue: `G_ProcessIPBans` parses the
//! persistent `g_banIPs` cvar into the in-memory filter list at map start.
//!
//! Ported here: `StringToFilter` (the dotted-IP address parser), `UpdateIPBans`
//! (re-serialize the list back into `g_banIPs`), `AddIP` (insert one address),
//! the public `G_ProcessIPBans`, and `G_FilterPacket` (the connect-time allow/deny
//! check against the filter list, honoring `g_filterBan`).
//!
//! Also ported: the `addip`/`removeip`/`listip` console handlers `Svcmd_AddIP_f` /
//! `Svcmd_RemoveIP_f` / `Svcmd_ListIPs_f` (clean leaves over the IP-filter family
//! above), and `ClientForString` (the numeric-slot-or-name → `gclient_t` resolver
//! several server commands share).
//!
//! `ConsoleCommand` is ported here and dispatched from `vmMain`'s `GAME_CONSOLE_COMMAND`
//! (`listip` dispatches to `Svcmd_ListIPs_f`, per the PC source).
//! See DEVIATIONS.md "g_svcmds.c siblings".

#![allow(non_snake_case)] // C function names (`StringToFilter`, ...) kept verbatim
#![allow(non_upper_case_globals)] // C static names (`ipFilters`, `numIPFilters`) kept

use core::ffi::{c_char, c_int, c_uint, CStr};
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::bg_public::{
    ET_BEAM, ET_GENERAL, ET_INVISIBLE, ET_ITEM, ET_MISSILE, ET_MOVER, ET_NPC, ET_PLAYER, ET_PORTAL,
    ET_PUSH_TRIGGER, ET_SPEAKER, ET_TELEPORT_TRIGGER,
};
use crate::codemp::game::g_cmds::ConcatArgs;
use crate::codemp::game::g_local::{gclient_t, gentity_t, CON_DISCONNECTED};
use crate::codemp::game::g_main::{
    g_banIPs, g_dedicated, g_entities, g_filterBan, level, Com_Printf, G_Printf,
};
use crate::codemp::game::q_shared::{
    va, COM_BeginParseSession, COM_ParseExt, Com_sprintf, Q_stricmp, Q_strncpyz, Sz,
};
use crate::codemp::game::q_shared_h::{
    byte, FS_READ, FS_WRITE, MAX_INFO_STRING, MAX_QPATH, MAX_TOKEN_CHARS,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

extern "C" {
    // The retail (non-`Q3_VM`) build links the C library's `atoi`/`strlen`/`strchr`
    // (bg_lib.c's own copies are the `Q3_VM` path) — the same externs `q_shared.c`
    // and `g_session.c` use.
    fn atoi(s: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
}

/*
==============================================================================

PACKET FILTERING


You can add or remove addresses from the filter list with:

addip <ip>
removeip <ip>

The ip address is specified in dot format, and any unspecified digits will match any value, so you can specify an entire class C network with "addip 192.246.40".

Removeip will only remove an address specified exactly the same way.  You cannot addip a subnet, then removeip a single host.

listip
Prints the current list of filters.

g_filterban <0 or 1>

If 1 (the default), then ip addresses matching the current list will be prohibited from entering the game.  This is the default setting.

If 0, then only addresses matching the list will be allowed.  This lets you easily set up a private game, or a game that only allows players from your local network.


==============================================================================
*/

// extern	vmCvar_t	g_banIPs;
// extern	vmCvar_t	g_filterBan;

#[repr(C)]
#[derive(Clone, Copy)]
struct ipFilter_t {
    mask: c_uint,
    compare: c_uint,
}

// VVFIXME - We don't need this at all, but this is the quick way.
// (C: `#ifdef _XBOX` → 1, `#else` → 1024; this is the non-`_XBOX` retail build.)
const MAX_IPFILTERS: usize = 1024;

static mut ipFilters: [ipFilter_t; MAX_IPFILTERS] = [ipFilter_t { mask: 0, compare: 0 }; MAX_IPFILTERS];
static mut numIPFilters: c_int = 0;

/*
=================
StringToFilter
=================
*/
unsafe fn StringToFilter(mut s: *const c_char, f: *mut ipFilter_t) -> qboolean {
    let mut num: [c_char; 128] = [0; 128];
    // b[] and m[] are zero-initialized here (the C zeroes them in an explicit
    // `for (i=0;i<4;i++)` loop; the `[0; 4]` initializers have identical effect).
    let mut b: [byte; 4] = [0; 4];
    let mut m: [byte; 4] = [0; 4];

    for i in 0..4 {
        if (*s as c_int) < '0' as c_int || (*s as c_int) > '9' as c_int {
            G_Printf(&format!(
                "Bad filter address: {}\n",
                CStr::from_ptr(s).to_string_lossy()
            ));
            return QFALSE;
        }

        let mut j = 0usize;
        while (*s as c_int) >= '0' as c_int && (*s as c_int) <= '9' as c_int {
            num[j] = *s;
            j += 1;
            s = s.add(1);
        }
        num[j] = 0;
        b[i] = atoi(num.as_ptr()) as byte;
        if b[i] != 0 {
            m[i] = 255;
        }

        if *s == 0 {
            break;
        }
        s = s.add(1);
    }

    (*f).mask = u32::from_ne_bytes(m);
    (*f).compare = u32::from_ne_bytes(b);

    QTRUE
}

/*
=================
UpdateIPBans
=================
*/
unsafe fn UpdateIPBans() {
    let mut b: [byte; 4];
    let mut iplist: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];

    iplist[0] = 0;
    for i in 0..*addr_of!(numIPFilters) {
        let filter = *(addr_of!(ipFilters) as *const ipFilter_t).add(i as usize);
        if filter.compare == 0xffffffff {
            continue;
        }

        b = filter.compare.to_ne_bytes();
        Com_sprintf(
            iplist.as_mut_ptr().add(strlen(iplist.as_ptr())),
            (MAX_INFO_STRING - strlen(iplist.as_ptr())) as c_int,
            format_args!("{}.{}.{}.{} ", b[0], b[1], b[2], b[3]),
        );
    }

    // C passes the raw `iplist` char buffer; the trap wrapper takes a `&str`, so
    // bridge the (always-ASCII) buffer back through `CStr`.
    trap::Cvar_Set("g_banIPs", &CStr::from_ptr(iplist.as_ptr()).to_string_lossy());
}

/*
=================
G_FilterPacket
=================
*/
pub fn G_FilterPacket(from: *const c_char) -> qboolean {
    unsafe {
        // m[] is zeroed by the explicit `while (i < 4)` loop below (the C does the
        // same with `m[i] = 0`); the inline initializer comment in the C is dropped.
        let mut m: [byte; 4] = [0; 4];
        let mut i: c_int;
        let in_: c_uint;
        let mut p: *const c_char;

        i = 0;
        while i < 4 {
            m[i as usize] = 0;
            i += 1;
        }

        i = 0;
        p = from;
        while *p != 0 && i < 4 {
            while *p >= '0' as c_char && *p <= '9' as c_char {
                m[i as usize] = m[i as usize]
                    .wrapping_mul(10)
                    .wrapping_add((*p - '0' as c_char) as byte);
                p = p.add(1);
            }
            if *p == 0 || *p == ':' as c_char {
                break;
            }
            // C `i++, p++;` — comma operator, both increment.
            i += 1;
            p = p.add(1);
        }

        in_ = u32::from_ne_bytes(m);

        for i in 0..*addr_of!(numIPFilters) {
            let filter = *(addr_of!(ipFilters) as *const ipFilter_t).add(i as usize);
            if (in_ & filter.mask) == filter.compare {
                return if (*addr_of!(g_filterBan)).integer != 0 {
                    QTRUE
                } else {
                    QFALSE
                };
            }
        }

        if (*addr_of!(g_filterBan)).integer == 0 {
            QTRUE
        } else {
            QFALSE
        }
    }
}

/*
=================
AddIP
=================
*/
pub(crate) unsafe fn AddIP(str_: *const c_char) {
    // `str` in the C source (renamed: `str` is a Rust primitive type name).
    let mut i: c_int = 0;

    while i < *addr_of!(numIPFilters) {
        if (*(addr_of!(ipFilters) as *const ipFilter_t).add(i as usize)).compare == 0xffffffff {
            break; // free spot
        }
        i += 1;
    }
    if i == *addr_of!(numIPFilters) {
        if *addr_of!(numIPFilters) == MAX_IPFILTERS as c_int {
            G_Printf("IP filter list is full\n");
            return;
        }
        *addr_of_mut!(numIPFilters) += 1;
    }

    let f = (addr_of_mut!(ipFilters) as *mut ipFilter_t).add(i as usize);
    if StringToFilter(str_, f) == QFALSE {
        (*f).compare = 0xffffffffu32;
    }

    UpdateIPBans();
}

/*
=================
G_ProcessIPBans
=================
*/
pub fn G_ProcessIPBans() {
    unsafe {
        // `str` in the C source (renamed: `str` is a Rust primitive type name).
        let mut str_buf: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];

        // Carried bug: `str_buf` is filled and then never read (the loop walks
        // `g_banIPs.string`, not the copy). Faithful to JKA (OpenJK keeps it too).
        Q_strncpyz(
            str_buf.as_mut_ptr(),
            (*addr_of!(g_banIPs)).string.as_ptr(),
            MAX_TOKEN_CHARS as c_int,
        );

        let mut s: *mut c_char = (*addr_of_mut!(g_banIPs)).string.as_mut_ptr();
        let mut t: *mut c_char = s;
        while *t != 0 {
            s = strchr(s, ' ' as c_int);
            if s.is_null() {
                break;
            }
            while *s == ' ' as c_char {
                *s = 0;
                s = s.add(1);
            }
            if *t != 0 {
                AddIP(t);
            }
            t = s;
        }
    }
}

/// `G_SaveBanIP` (g_svcmds.c:299) — write the whole IP-filter table out to `banip.txt`
/// (a count line, then one line per slot; tombstoned slots become `"unused"`). Called by
/// `G_ShutdownGame`. No oracle (drives `trap_FS_*` file I/O over the module IP-filter
/// statics). Faithful note: the C extracts the dotted address with `*(unsigned*)b =
/// ipFilters[i].compare` then `va("%i.%i.%i.%i \n", b[0]..b[3])`, reproduced via
/// `compare.to_ne_bytes()` (the same idiom as `UpdateIPBans`/`Svcmd_ListIPs_f`), and the
/// `va` strings keep their trailing space before the newline.
pub fn G_SaveBanIP() {
    //save out all the banned IPs
    unsafe {
        let (_len, fh) = trap::FS_FOpenFile("banip.txt", FS_WRITE);
        if fh == 0 {
            G_Printf("G_SaveBanIP - ERROR: can't open banip.txt\n");
            return;
        }

        let str_ = va(format_args!("{} \n", *addr_of!(numIPFilters)));
        trap::FS_Write(core::slice::from_raw_parts(str_ as *const u8, strlen(str_)), fh);
        for i in 0..*addr_of!(numIPFilters) {
            let filter = *(addr_of!(ipFilters) as *const ipFilter_t).add(i as usize);
            if filter.compare == 0xffffffff {
                let s = c"unused \n".as_ptr();
                trap::FS_Write(core::slice::from_raw_parts(s as *const u8, strlen(s)), fh);
            } else {
                let b: [byte; 4] = filter.compare.to_ne_bytes();
                let str_ = va(format_args!("{}.{}.{}.{} \n", b[0], b[1], b[2], b[3]));
                trap::FS_Write(core::slice::from_raw_parts(str_ as *const u8, strlen(str_)), fh);
            }
        }

        trap::FS_FCloseFile(fh);
    }
}

/// `G_LoadIPBans` (g_svcmds.c:333) — read `banip.txt` back in and repopulate the IP-filter
/// table: the first token is the count, then each subsequent token is either `"unused"`
/// (tombstone) or a dotted address parsed by [`StringToFilter`]. Called by `G_InitGame`
/// (the PC source uses this in place of the Xbox `G_ProcessIPBans`). No oracle (drives
/// `trap_FS_*` + the `COM_Parse` text scanner over the module IP-filter statics). Faithful
/// quirks carried verbatim: the diagnostic spells the function `G_LoadBanIP`; `banIPFile`
/// is passed uninitialised to `COM_BeginParseSession` (only used for error text); and the
/// `banIPBuffer[len] = 0` terminator can write one past the buffer when the file fills it
/// exactly (same as JKA).
pub fn G_LoadIPBans() {
    //load in all the banned IPs
    unsafe {
        let mut banIPBuffer: [c_char; MAX_IPFILTERS * 32] = [0; MAX_IPFILTERS * 32]; //	The list of file names read in
        let mut banIPFile: [c_char; MAX_QPATH] = [0; MAX_QPATH];

        let (len, fh) = trap::FS_FOpenFile("banip.txt", FS_READ);
        if fh == 0 {
            G_Printf("G_LoadBanIP - ERROR: can't open banip.txt\n");
            return;
        }

        trap::FS_Read(
            core::slice::from_raw_parts_mut(banIPBuffer.as_mut_ptr() as *mut u8, len as usize),
            fh,
        );
        banIPBuffer[len as usize] = 0;
        trap::FS_FCloseFile(fh);
        let mut p: *const c_char = banIPBuffer.as_ptr();
        COM_BeginParseSession(banIPFile.as_mut_ptr());

        let mut token = COM_ParseExt(addr_of_mut!(p), QTRUE);
        if !token.is_null() {
            *addr_of_mut!(numIPFilters) = atoi(token);

            for i in 0..*addr_of!(numIPFilters) {
                token = COM_ParseExt(addr_of_mut!(p), QTRUE);
                if !token.is_null() {
                    //have an ip
                    if Q_stricmp(c"unused".as_ptr(), token) == 0 {
                        (*(addr_of_mut!(ipFilters) as *mut ipFilter_t).add(i as usize)).compare =
                            0xffffffffu32;
                    } else {
                        StringToFilter(
                            token,
                            (addr_of_mut!(ipFilters) as *mut ipFilter_t).add(i as usize),
                        );
                    }
                } else {
                    break;
                }
            }
        }
    }
}

/*
=================
Svcmd_AddIP_f
=================
*/
/// `Svcmd_AddIP_f` (g_svcmds.c:226) — the `addip <ip-mask>` console command:
/// read the mask argument and hand it to [`AddIP`]. No oracle (drives `trap_Argc`/
/// `trap_Argv` and mutates the module IP-filter statics). The C reads `trap_Argv`
/// into a fixed `str[MAX_TOKEN_CHARS]`; our `trap::Argv` already returns an owned
/// `String`, so we hold it in a [`CString`] and pass its pointer to `AddIP`.
pub fn Svcmd_AddIP_f() {
    if trap::Argc() < 2 {
        G_Printf("Usage:  addip <ip-mask>\n");
        return;
    }

    let str_ = trap::Argv(1);
    let cstr = std::ffi::CString::new(str_).unwrap_or_default();
    unsafe {
        AddIP(cstr.as_ptr());
    }
}

/*
=================
Svcmd_RemoveIP_f
=================
*/
/// `Svcmd_RemoveIP_f` (g_svcmds.c:246) — the `removeip <ip-mask>` console command:
/// parse the mask, then linear-scan the filter list for an exact `(mask, compare)`
/// match and tombstone it (`compare = 0xffffffff`) + re-serialize via [`UpdateIPBans`].
/// No oracle (drives the trap argv surface + module IP-filter statics).
pub fn Svcmd_RemoveIP_f() {
    if trap::Argc() < 2 {
        G_Printf("Usage:  sv removeip <ip-mask>\n");
        return;
    }

    let str_ = trap::Argv(1);
    let cstr = std::ffi::CString::new(str_).unwrap_or_default();

    unsafe {
        let mut f = ipFilter_t { mask: 0, compare: 0 };
        if StringToFilter(cstr.as_ptr(), &mut f) == QFALSE {
            return;
        }

        let arr = addr_of_mut!(ipFilters) as *mut ipFilter_t;
        for i in 0..*addr_of!(numIPFilters) {
            let filter = &mut *arr.add(i as usize);
            if filter.mask == f.mask && filter.compare == f.compare {
                filter.compare = 0xffffffffu32;
                G_Printf("Removed.\n");

                UpdateIPBans();
                return;
            }
        }
    }

    G_Printf(&format!("Didn't find {}.\n", cstr.to_string_lossy()));
}

/// `Svcmd_ListIPs_f` (g_svcmds.c:276) — the `listip` console command: print the
/// IP-filter table, one slot per line (tombstoned slots shown as `unused`). New in
/// the PC source (absent from the Xbox tree), and the callee `ConsoleCommand`'s
/// `listip` branch dispatches to. No oracle (drives `G_Printf`/`va` and reads the
/// module IP-filter statics). Faithful note: the C builds each address with
/// `va("%i.%i.%i.%i \n", ...)` then prints it via `G_Printf("%s\n", str)`, so the
/// trailing-space + double-newline is intentional and preserved.
pub fn Svcmd_ListIPs_f() {
    unsafe {
        G_Printf(&format!("{} IP slots used.\n", *addr_of!(numIPFilters)));
        for i in 0..*addr_of!(numIPFilters) {
            G_Printf(&format!("{}: ", i));
            let filter = *(addr_of!(ipFilters) as *const ipFilter_t).add(i as usize);
            if filter.compare == 0xffffffff {
                G_Printf("unused\n");
            } else {
                let b: [byte; 4] = filter.compare.to_ne_bytes();
                let str_ = va(format_args!("{}.{}.{}.{} \n", b[0], b[1], b[2], b[3]));
                G_Printf(&format!("{}\n", CStr::from_ptr(str_).to_string_lossy()));
            }
        }
    }
}

/// `ClientForString` (g_svcmds.c:340) — resolve a console argument to a connected
/// `gclient_t`: a purely-numeric string is a slot index (range-checked against
/// `level.maxclients`, rejected if that slot is disconnected); otherwise it is matched
/// case-insensitively against each connected client's `pers.netname`. Returns a null
/// pointer (and prints a diagnostic) when nothing matches. No oracle (walks the global
/// `level.clients`). Faithful note: the numeric branch prints via `Com_Printf` while
/// the not-connected / not-found branches print via `G_Printf`, exactly as in the C.
///
/// # Safety
/// The `level` global must be initialised; `s` must be a valid C string.
pub unsafe fn ClientForString(s: *const c_char) -> *mut gclient_t {
    let clients = (*addr_of!(level)).clients;
    let maxclients = (*addr_of!(level)).maxclients;

    // numeric values are just slot numbers
    if *s >= b'0' as c_char && *s <= b'9' as c_char {
        let idnum = atoi(s);
        if idnum < 0 || idnum >= maxclients {
            Com_Printf(&format!("Bad client slot: {}\n", idnum));
            return core::ptr::null_mut();
        }

        let cl = clients.offset(idnum as isize);
        if (*cl).pers.connected == CON_DISCONNECTED {
            G_Printf(&format!("Client {} is not connected\n", idnum));
            return core::ptr::null_mut();
        }
        return cl;
    }

    // check for a name match
    for i in 0..maxclients {
        let cl = clients.offset(i as isize);
        if (*cl).pers.connected == CON_DISCONNECTED {
            continue;
        }
        if Q_stricmp((*cl).pers.netname.as_ptr(), s) == 0 {
            return cl;
        }
    }

    G_Printf(&format!(
        "User {} is not on the server\n",
        CStr::from_ptr(s).to_string_lossy()
    ));

    core::ptr::null_mut()
}

/// `void Svcmd_EntityList_f( void )` (g_svcmds.c:281) — the `entitylist` console
/// command: dump every in-use entity (slot index, eType label, classname) to the
/// server console. Walks `g_entities[1 .. level.num_entities]` (slot 0 is the world
/// and is skipped, matching the C's `g_entities+1` start), printing one line per
/// `inuse` entity. The eType switch reproduces the C's fixed-width labels verbatim
/// (the same column padding); unknown types print the raw integer in the same
/// 3-wide field. A NULL `classname` is omitted, exactly as the C's pointer guard.
///
/// Clean leaf: the sole callee is `G_Printf`; everything else is reads of the
/// `g_entities` array / `level.num_entities` and the `inuse`/`s.eType`/`classname`
/// fields. No oracle (it drives the entity array + `G_Printf`, the same console-dump
/// precedent as the rest of g_svcmds).
pub unsafe fn Svcmd_EntityList_f() {
    let num_entities = (*addr_of!(level)).num_entities;

    for e in 1..num_entities {
        let check: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(e as isize);
        if (*check).inuse == QFALSE {
            continue;
        }
        G_Printf(&format!("{:3}:", e));
        match (*check).s.eType {
            ET_GENERAL => G_Printf("ET_GENERAL          "),
            ET_PLAYER => G_Printf("ET_PLAYER           "),
            ET_ITEM => G_Printf("ET_ITEM             "),
            ET_MISSILE => G_Printf("ET_MISSILE          "),
            ET_MOVER => G_Printf("ET_MOVER            "),
            ET_BEAM => G_Printf("ET_BEAM             "),
            ET_PORTAL => G_Printf("ET_PORTAL           "),
            ET_SPEAKER => G_Printf("ET_SPEAKER          "),
            ET_PUSH_TRIGGER => G_Printf("ET_PUSH_TRIGGER     "),
            ET_TELEPORT_TRIGGER => G_Printf("ET_TELEPORT_TRIGGER "),
            ET_INVISIBLE => G_Printf("ET_INVISIBLE        "),
            ET_NPC => G_Printf("ET_NPC              "),
            other => G_Printf(&format!("{:3}                 ", other)),
        }

        if !(*check).classname.is_null() {
            G_Printf(&format!("{}", CStr::from_ptr((*check).classname).to_string_lossy()));
        }
        G_Printf("\n");
    }
}

/*
===================
Svcmd_ForceTeam_f

forceteam <player> <team>
===================
*/
/// `void Svcmd_ForceTeam_f( void )` (g_svcmds.c:384) — the `forceteam <player>
/// <team>` console command: resolve argument 1 to a connected client via
/// [`ClientForString`], then force that client onto the named team via
/// [`SetTeam`]. No oracle (drives `trap_Argv` + the side-effecting `SetTeam`).
///
/// Faithful note: the C reads each `trap_Argv` into a fixed `str[MAX_TOKEN_CHARS]`
/// in turn; `trap::Argv` already returns an owned `String`, so we hold each in a
/// [`CString`]. `SetTeam` takes a `*mut c_char` and only reads it. The C's
/// `&g_entities[cl - level.clients]` (the client index recovered by pointer
/// subtraction) is mirrored with `offset_from` + `.offset` on the `g_entities`
/// array, the precedented client-pointer-to-index idiom.
pub fn Svcmd_ForceTeam_f() {
    unsafe {
        // find the player
        let str_ = trap::Argv(1);
        let cstr = std::ffi::CString::new(str_).unwrap_or_default();
        let cl = ClientForString(cstr.as_ptr());
        if cl.is_null() {
            return;
        }

        // set the team
        let str_ = trap::Argv(2);
        let cstr = std::ffi::CString::new(str_).unwrap_or_default();
        let idnum = cl.offset_from((*addr_of!(level)).clients) as isize;
        crate::codemp::game::g_cmds::SetTeam(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(idnum),
            cstr.as_ptr() as *mut c_char,
        );
    }
}

// ── Guarded stubs ─────────────────────────────────────────────────────────────
// `ConsoleCommand` dispatches three console commands into the bot / game-memory
// subsystems whose handlers have not been ported yet. They are void(void) with no
// Rust definition anywhere, so an `extern "C"` forward-decl is the wrong tool (it
// would try to link a nonexistent C symbol). Instead, mirror the proven
// guarded-stub pattern (cf. `FireVehicleWeapon` stubbed for `FireWeapon`, cycle 31,
// DEVIATIONS.md): a private placeholder that does nothing, so the dispatch branch
// still returns `qtrue` exactly as the C does. Replace each with the real port when
// the owning subsystem lands.

/// TODO: un-stub when `Svcmd_GameMem_f` lands (the `game_memory` diagnostic that
/// dumps the zone/hunk allocator stats; depends on the game-memory subsystem).
fn Svcmd_GameMem_f() {}

/// TODO: un-stub when the bot subsystem lands (`Svcmd_AddBot_f` — the `addbot`
/// console handler that spawns an AI bot client).
fn Svcmd_AddBot_f() {}

/// TODO: un-stub when the bot subsystem lands (`Svcmd_BotList_f` — the `botlist`
/// console handler that prints the registered bot definitions).
fn Svcmd_BotList_f() {}

/*
=================
ConsoleCommand

=================
*/
pub unsafe fn ConsoleCommand() -> qboolean {
    // char	cmd[MAX_TOKEN_CHARS];
    // trap_Argv( 0, cmd, sizeof( cmd ) );
    let cmd = trap::Argv(0);
    let cmd_c = std::ffi::CString::new(cmd).unwrap_or_default();
    let cmd = cmd_c.as_ptr();

    if Q_stricmp(cmd, c"entitylist".as_ptr()) == 0 {
        Svcmd_EntityList_f();
        return QTRUE;
    }

    if Q_stricmp(cmd, c"forceteam".as_ptr()) == 0 {
        Svcmd_ForceTeam_f();
        return QTRUE;
    }

    if Q_stricmp(cmd, c"game_memory".as_ptr()) == 0 {
        Svcmd_GameMem_f();
        return QTRUE;
    }

    if Q_stricmp(cmd, c"addbot".as_ptr()) == 0 {
        Svcmd_AddBot_f();
        return QTRUE;
    }

    if Q_stricmp(cmd, c"botlist".as_ptr()) == 0 {
        Svcmd_BotList_f();
        return QTRUE;
    }

    /*	if (Q_stricmp (cmd, "abort_podium") == 0) {
            Svcmd_AbortPodium_f();
            return qtrue;
        }
    */
    if Q_stricmp(cmd, c"addip".as_ptr()) == 0 {
        Svcmd_AddIP_f();
        return QTRUE;
    }

    if Q_stricmp(cmd, c"removeip".as_ptr()) == 0 {
        Svcmd_RemoveIP_f();
        return QTRUE;
    }

    if Q_stricmp(cmd, c"listip".as_ptr()) == 0 {
        Svcmd_ListIPs_f();
        //trap_SendConsoleCommand( EXEC_NOW, "g_banIPs\n" );
        return QTRUE;
    }

    if (*addr_of!(g_dedicated)).integer != 0 {
        if Q_stricmp(cmd, c"say".as_ptr()) == 0 {
            trap::SendServerCommand(-1, &format!("print \"server: {}\n\"", Sz(ConcatArgs(1))));
            return QTRUE;
        }
        // everything else will also be printed as a say command
        trap::SendServerCommand(-1, &format!("print \"server: {}\n\"", Sz(ConcatArgs(0))));
        return QTRUE;
    }

    QFALSE
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle::{jka_G_FilterPacket, jka_StringToFilter};
    use std::ffi::CString;

    // StringToFilter parity vs the real Raven C (oracle/g_svcmds_oracle.c).
    //
    // All inputs are VALID dotted addresses on purpose: an invalid one takes the
    // `G_Printf("Bad filter address")` branch, and G_Printf forwards through the
    // engine syscall pointer, which is unset under `cargo test` -> the syscall
    // layer's `debug_assert` fires. So the bad-address branch (return qfalse, leave
    // `f` untouched) is verified by inspection, not exercised here. The valid set
    // still covers the whole computation: 1-4 octets, the >4-octet tail (ignored),
    // a single octet, zero octets (mask byte stays 0), the 255 boundary, and the
    // `byte` truncation of out-of-range octets (256 -> 0, 300 -> 44).
    #[test]
    fn StringToFilter_matches_oracle() {
        let cases = [
            "1.2.3.4",
            "192.168.0.0",
            "10",
            "1.2",
            "255.255.255.255",
            "0.0.0.0",
            "0",
            "256",
            "300.1",
            "1.2.3.4.5",
            "192.168",
            "127.0.0.1",
        ];
        for case in cases {
            let c = CString::new(case).unwrap();

            // Rust port. `f` is caller-initialized to 0/0 (StringToFilter leaves it
            // untouched on the qfalse path; the oracle wrapper inits identically).
            let mut f = ipFilter_t { mask: 0, compare: 0 };
            let r_ret = unsafe { StringToFilter(c.as_ptr(), &mut f) };

            // Oracle.
            let mut o_mask: c_uint = 0;
            let mut o_compare: c_uint = 0;
            let o_ret = unsafe { jka_StringToFilter(c.as_ptr(), &mut o_mask, &mut o_compare) };

            assert_eq!(r_ret, o_ret, "return mismatch for {case:?}");
            assert_eq!(f.mask, o_mask, "mask mismatch for {case:?}");
            assert_eq!(f.compare, o_compare, "compare mismatch for {case:?}");
        }
    }

    // G_FilterPacket parity vs the real Raven C (oracle/g_svcmds_oracle.c).
    //
    // The Rust port reads three module statics (`ipFilters`/`numIPFilters` and the
    // `g_filterBan` vmCvar). The test seeds them, runs the port, then seeds the
    // oracle's own copies identically via jka_G_FilterPacket and compares the
    // qboolean. Cases cover: the dotted-IP -> u32 fold (1-4 octets, the `addr:port`
    // form, byte wraparound), an empty list (no match), an exact-match filter, a
    // masked class-C match, a non-match, and both g_filterBan polarities (1 = the
    // filter list is a deny-list, 0 = allow-list).
    #[test]
    fn G_FilterPacket_matches_oracle() {
        // (address, filter list [(mask, compare)], g_filterBan)
        let cases: &[(&str, &[(c_uint, c_uint)], c_int)] = &[
            // empty list, ban on/off -> falls through to the polarity default
            ("1.2.3.4", &[], 1),
            ("1.2.3.4", &[], 0),
            // exact-match deny-list: full mask, exact compare
            ("1.2.3.4", &[(0xffffffff, 0x04030201)], 1),
            ("1.2.3.4", &[(0xffffffff, 0x04030201)], 0),
            // class-C: mask first three octets only
            ("192.168.0.55", &[(0x00ffffff, 0x0000a8c0)], 1),
            ("192.168.1.55", &[(0x00ffffff, 0x0000a8c0)], 1),
            // non-matching exact filter
            ("10.0.0.1", &[(0xffffffff, 0x04030201)], 1),
            // addr:port form (parse stops at ':')
            ("1.2.3.4:29070", &[(0xffffffff, 0x04030201)], 1),
            // single/partial octet addresses
            ("127", &[(0x000000ff, 0x0000007f)], 1),
            ("192.168", &[(0x0000ffff, 0x0000a8c0)], 0),
            // multiple filters; second one matches
            (
                "5.6.7.8",
                &[(0xffffffff, 0x04030201), (0xffffffff, 0x08070605)],
                1,
            ),
        ];

        for (addr, filters, ban) in cases {
            let c = CString::new(*addr).unwrap();

            // Seed the Rust module statics.
            unsafe {
                *addr_of_mut!(numIPFilters) = filters.len() as c_int;
                let arr = addr_of_mut!(ipFilters) as *mut ipFilter_t;
                for (i, (mask, compare)) in filters.iter().enumerate() {
                    *arr.add(i) = ipFilter_t {
                        mask: *mask,
                        compare: *compare,
                    };
                }
                (*addr_of_mut!(g_filterBan)).integer = *ban;
            }

            let r_ret = G_FilterPacket(c.as_ptr());

            // Seed the oracle identically.
            let masks: Vec<c_uint> = filters.iter().map(|(m, _)| *m).collect();
            let compares: Vec<c_uint> = filters.iter().map(|(_, cmp)| *cmp).collect();
            let o_ret = unsafe {
                jka_G_FilterPacket(
                    c.as_ptr(),
                    filters.len() as c_int,
                    masks.as_ptr(),
                    compares.as_ptr(),
                    *ban,
                )
            };

            assert_eq!(
                r_ret, o_ret,
                "G_FilterPacket mismatch for addr={addr:?} ban={ban}"
            );
        }

        // Restore the statics to their pristine state for any later test ordering.
        unsafe {
            *addr_of_mut!(numIPFilters) = 0;
        }
    }
}
