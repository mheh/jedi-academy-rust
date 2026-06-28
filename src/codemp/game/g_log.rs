//! `g_log.c` — weapon-statistic logging.
//!
//! Nothing super-fancy here, the original just keeps track of, per player: how many
//! times a weapon/item is picked up, used/fired, the total damage done by that weapon,
//! the number of kills/deaths with it, and the time spent with it; plus how many times
//! each powerup/item is picked up.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/g_log.c` — the full file. The
//! `G_InitGame`/`G_ShutdownGame` entry points ([`G_LogWeaponInit`] zeroes the per-player
//! counters, [`G_LogWeaponOutput`] dumps the accumulated stats to the console log and to
//! `g_statLogFile`), the per-event increment helpers (`G_LogWeaponPickup`/`Fire`/`Damage`/…)
//! that feed those counters, the end-of-round award calculations (`CalculateEfficiency`,
//! the team awards, `CalculateAwards`, …), the `Get*ForClient` stat queries, and
//! `G_ClearClientLog`. `CalculateStreak` is a faithful `#if 0` empty body (returns 0).
//!
//! `LOGGING_WEAPONS` is `#define`d at the top of the **PC** `g_log.c`, so these bodies are
//! live (the Xbox tree comments the define out, which is why the Xbox `g_main.c` comments
//! out both calls — see DEVIATIONS). No oracle: every path is `trap_FS_*` file I/O / cvar
//! reads over module statics (entity-state/trap surface), so there is no extractable pure-C
//! oracle.

#![allow(non_snake_case)] // C function names (`G_LogWeaponInit`, ...) kept verbatim
#![allow(non_upper_case_globals)] // C global names (`weaponFromMOD`, ...) kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::bg_public::{
    GT_CTF, GT_CTY, GT_JEDIMASTER, GT_TEAM, HI_NONE, HI_NUM_HOLDABLE, MOD_DET_PACK_SPLASH,
    MOD_DISRUPTOR_SNIPER, MOD_FORCE_DARK, MOD_MAX, MOD_ROCKET, MOD_ROCKET_HOMING,
    MOD_ROCKET_HOMING_SPLASH, MOD_ROCKET_SPLASH, MOD_SENTRY, MOD_STUN_BATON, MOD_THERMAL,
    MOD_THERMAL_SPLASH, MOD_TIMED_MINE_SPLASH, MOD_TRIP_MINE_SPLASH, PERS_KILLED, PERS_RANK,
    PERS_SCORE, PERS_TEAM, PW_NONE, PW_NUM_POWERUPS,
};
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DEMP2, WP_DET_PACK, WP_DISRUPTOR, WP_FLECHETTE,
    WP_MELEE, WP_NONE, WP_NUM_WEAPONS, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON,
    WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use crate::codemp::game::g_combat::modNames;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{
    g_entities, g_gametype, g_maxclients, g_statLog, g_statLogFile, level, G_LogPrintf,
};
use crate::codemp::game::q_shared::{Com_sprintf, Info_ValueForKey, Sz};
use crate::codemp::game::q_shared_h::{FS_APPEND, MAX_CLIENTS};
use crate::codemp::game::w_saber::HasSetSaberOnly;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

extern "C" {
    // The retail (non-`Q3_VM`) build links the C library's `strlen`/`strncpy` (bg_lib.c's
    // own copies are the `Q3_VM` path) — the same externs the rest of the game module uses.
    fn strlen(s: *const c_char) -> usize;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

// ---------------------------------------------------------------------------
// #ifdef LOGGING_WEAPONS — file-scope counters (one row per client).
// ---------------------------------------------------------------------------
static mut G_WeaponLogPickups: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS] =
    [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
static mut G_WeaponLogFired: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS] =
    [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
static mut G_WeaponLogDamage: [[c_int; MOD_MAX as usize]; MAX_CLIENTS] =
    [[0; MOD_MAX as usize]; MAX_CLIENTS];
static mut G_WeaponLogKills: [[c_int; MOD_MAX as usize]; MAX_CLIENTS] =
    [[0; MOD_MAX as usize]; MAX_CLIENTS];
static mut G_WeaponLogDeaths: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS] =
    [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
static mut G_WeaponLogFrags: [[c_int; MAX_CLIENTS]; MAX_CLIENTS] = [[0; MAX_CLIENTS]; MAX_CLIENTS];
static mut G_WeaponLogTime: [[c_int; WP_NUM_WEAPONS as usize]; MAX_CLIENTS] =
    [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
static mut G_WeaponLogLastTime: [c_int; MAX_CLIENTS] = [0; MAX_CLIENTS];
static mut G_WeaponLogClientTouch: [qboolean; MAX_CLIENTS] = [0; MAX_CLIENTS];
static mut G_WeaponLogPowerups: [[c_int; HI_NUM_HOLDABLE as usize]; MAX_CLIENTS] =
    [[0; HI_NUM_HOLDABLE as usize]; MAX_CLIENTS];
static mut G_WeaponLogItems: [[c_int; PW_NUM_POWERUPS as usize]; MAX_CLIENTS] =
    [[0; PW_NUM_POWERUPS as usize]; MAX_CLIENTS];

// MOD-weapon mapping array. `int weaponFromMOD[MOD_MAX]`. The PC source supplies these 40
// positional initializers (the Xbox tree omits the `MOD_COLLISION` slot); on retail PC
// `MOD_MAX` is 45, so the trailing slots are zero-filled to `WP_NONE`. Faithful quirk: the
// list predates the `MOD_VEHICLE`/`MOD_CONC*` enum inserts, so past `MOD_DET_PACK_SPLASH`
// the comments no longer line up with the current enumerators — reproduced verbatim. The
// only indices that matter are `0..=MOD_SENTRY`, which the accumulation below reads.
static weaponFromMOD: [c_int; MOD_MAX as usize] = [
    WP_NONE,            //MOD_UNKNOWN,
    WP_STUN_BATON,      //MOD_STUN_BATON,
    WP_MELEE,           //MOD_MELEE,
    WP_SABER,           //MOD_SABER,
    WP_BRYAR_PISTOL,    //MOD_BRYAR_PISTOL,
    WP_BRYAR_PISTOL,    //MOD_BRYAR_PISTOL_ALT,
    WP_BLASTER,         //MOD_BLASTER,
    WP_TURRET,          //MOD_TURBLAST
    WP_DISRUPTOR,       //MOD_DISRUPTOR,
    WP_DISRUPTOR,       //MOD_DISRUPTOR_SPLASH,
    WP_DISRUPTOR,       //MOD_DISRUPTOR_SNIPER,
    WP_BOWCASTER,       //MOD_BOWCASTER,
    WP_REPEATER,        //MOD_REPEATER,
    WP_REPEATER,        //MOD_REPEATER_ALT,
    WP_REPEATER,        //MOD_REPEATER_ALT_SPLASH,
    WP_DEMP2,           //MOD_DEMP2,
    WP_DEMP2,           //MOD_DEMP2_ALT,
    WP_FLECHETTE,       //MOD_FLECHETTE,
    WP_FLECHETTE,       //MOD_FLECHETTE_ALT_SPLASH,
    WP_ROCKET_LAUNCHER, //MOD_ROCKET,
    WP_ROCKET_LAUNCHER, //MOD_ROCKET_SPLASH,
    WP_ROCKET_LAUNCHER, //MOD_ROCKET_HOMING,
    WP_ROCKET_LAUNCHER, //MOD_ROCKET_HOMING_SPLASH,
    WP_THERMAL,         //MOD_THERMAL,
    WP_THERMAL,         //MOD_THERMAL_SPLASH,
    WP_TRIP_MINE,       //MOD_TRIP_MINE_SPLASH,
    WP_TRIP_MINE,       //MOD_TIMED_MINE_SPLASH,
    WP_DET_PACK,        //MOD_DET_PACK_SPLASH,
    WP_NONE,            //MOD_FORCE_DARK,
    WP_NONE,            //MOD_SENTRY,
    WP_NONE,            //MOD_WATER,
    WP_NONE,            //MOD_SLIME,
    WP_NONE,            //MOD_LAVA,
    WP_NONE,            //MOD_CRUSH,
    WP_NONE,            //MOD_TELEFRAG,
    WP_NONE,            //MOD_FALLING,
    WP_NONE,            //MOD_COLLISION,
    WP_NONE,            //MOD_SUICIDE,
    WP_NONE,            //MOD_TARGET_LASER,
    WP_NONE,            //MOD_TRIGGER_HURT,
    // trailing MOD_MAX slots beyond the C initializer list, zero-filled to WP_NONE:
    WP_NONE,
    WP_NONE,
    WP_NONE,
    WP_NONE,
    WP_NONE,
];

// `char *weaponNameFromIndex[WP_NUM_WEAPONS]`. The C supplies these 16 positional string
// literals; on retail PC `WP_NUM_WEAPONS` is 19, so the trailing 3 slots are NULL. Faithful
// quirk: the literals fill indices 0..15 by position, so "Turret" sits at index 15 while
// `WP_TURRET == 18` — reproduced verbatim (matching modNames' same misalignment quirk).
static mut weaponNameFromIndex: [*const c_char; WP_NUM_WEAPONS as usize] = [
    c"No Weapon".as_ptr(),
    c"Stun Baton".as_ptr(),
    c"Saber".as_ptr(),
    c"Bryar Pistol".as_ptr(),
    c"Blaster".as_ptr(),
    c"Disruptor".as_ptr(),
    c"Bowcaster".as_ptr(),
    c"Repeater".as_ptr(),
    c"Demp2".as_ptr(),
    c"Flechette".as_ptr(),
    c"Rocket Launcher".as_ptr(),
    c"Thermal".as_ptr(),
    c"Tripmine".as_ptr(),
    c"Detpack".as_ptr(),
    c"Emplaced gun".as_ptr(),
    c"Turret".as_ptr(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
];

/// Render a `weaponNameFromIndex`/`modNames` entry to an owned `String` for the width-padded
/// `%15s`/`%25s` log columns; a NULL entry renders empty. DEVIATION: glibc's `printf("%s")`
/// prints `(null)` for a NULL pointer, but only the phantom trailing slots are NULL and this
/// is log-only output, so an empty rendering is harmless.
unsafe fn name_or_empty(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}

/*
=================
G_LogWeaponInit
=================
*/
pub fn G_LogWeaponInit() {
    // #ifdef LOGGING_WEAPONS — `memset` every per-client counter back to zero.
    unsafe {
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogPickups) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogPickups)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogFired) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogFired)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogDamage) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogDamage)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogKills) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogKills)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogDeaths) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogDeaths)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogFrags) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogFrags)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogTime) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogTime)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogLastTime) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogLastTime)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogPowerups) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogPowerups)),
        );
        core::ptr::write_bytes(
            addr_of_mut!(G_WeaponLogItems) as *mut u8,
            0,
            core::mem::size_of_val(&*addr_of!(G_WeaponLogItems)),
        );
        // Note: the C zeroes the ten counters above but deliberately does NOT memset
        // `G_WeaponLogClientTouch`; that flag is managed by the per-event helpers, so it is
        // left untouched here to match the C exactly.
    }
}

// ---------------------------------------------------------------------------
// Per-event increment helpers (`#ifdef LOGGING_WEAPONS`). Each bumps a counter
// and marks the client "touched". DEVIATION: the C `G_LogWeaponDamage`/`Kill`
// `MAX_CLIENTS`-guard their `client` index, but `G_LogWeaponPickup`/`Fire` do
// **not** — Raven trusts the caller there. That is a latent buffer overflow: the
// callers pass `ent->s.number`, which for a non-client entity (an NPC) exceeds
// `MAX_CLIENTS`, so the C writes out of bounds into adjacent statics (UB that the
// C build happens to tolerate — `FireWeapon`, g_weapon.c:4607, fires for NPCs).
// Rust's bounds check turns that same write into a hard panic, so we add the
// missing `client >= MAX_CLIENTS` guard here to match the C's *observable*
// behaviour (the game keeps running; only out-of-range stat rows — which have no
// gameplay effect — are skipped). `QDECL` (cdecl) in C; called directly from Rust
// here, so plain `pub fn` matches [`G_LogWeaponInit`].
// ---------------------------------------------------------------------------

pub fn G_LogWeaponPickup(client: c_int, weaponid: c_int) {
    unsafe {
        if client >= MAX_CLIENTS as c_int {
            return; // see block comment: guards the C's unguarded OOB write
        }
        (*addr_of_mut!(G_WeaponLogPickups))[client as usize][weaponid as usize] += 1;
        (*addr_of_mut!(G_WeaponLogClientTouch))[client as usize] = QTRUE;
    }
}

pub fn G_LogWeaponFire(client: c_int, weaponid: c_int) {
    unsafe {
        if client >= MAX_CLIENTS as c_int {
            return; // see block comment: guards the C's unguarded OOB write
        }
        (*addr_of_mut!(G_WeaponLogFired))[client as usize][weaponid as usize] += 1;
        let dur = (*addr_of!(level)).time - (*addr_of!(G_WeaponLogLastTime))[client as usize];
        if dur > 5000 {
            // 5 second max.
            (*addr_of_mut!(G_WeaponLogTime))[client as usize][weaponid as usize] += 5000;
        } else {
            (*addr_of_mut!(G_WeaponLogTime))[client as usize][weaponid as usize] += dur;
        }
        (*addr_of_mut!(G_WeaponLogLastTime))[client as usize] = (*addr_of!(level)).time;
        (*addr_of_mut!(G_WeaponLogClientTouch))[client as usize] = QTRUE;
    }
}

pub fn G_LogWeaponDamage(client: c_int, mod_: c_int, amount: c_int) {
    unsafe {
        if client >= MAX_CLIENTS as c_int {
            return;
        }
        (*addr_of_mut!(G_WeaponLogDamage))[client as usize][mod_ as usize] += amount;
        (*addr_of_mut!(G_WeaponLogClientTouch))[client as usize] = QTRUE;
    }
}

pub fn G_LogWeaponKill(client: c_int, mod_: c_int) {
    unsafe {
        if client >= MAX_CLIENTS as c_int {
            return;
        }
        (*addr_of_mut!(G_WeaponLogKills))[client as usize][mod_ as usize] += 1;
        (*addr_of_mut!(G_WeaponLogClientTouch))[client as usize] = QTRUE;
    }
}

pub fn G_LogWeaponFrag(attacker: c_int, deadguy: c_int) {
    unsafe {
        if attacker >= MAX_CLIENTS as c_int || deadguy >= MAX_CLIENTS as c_int {
            return;
        }
        (*addr_of_mut!(G_WeaponLogFrags))[attacker as usize][deadguy as usize] += 1;
        (*addr_of_mut!(G_WeaponLogClientTouch))[attacker as usize] = QTRUE;
    }
}

pub fn G_LogWeaponDeath(client: c_int, weaponid: c_int) {
    unsafe {
        if client >= MAX_CLIENTS as c_int {
            return;
        }
        (*addr_of_mut!(G_WeaponLogDeaths))[client as usize][weaponid as usize] += 1;
        (*addr_of_mut!(G_WeaponLogClientTouch))[client as usize] = QTRUE;
    }
}

pub fn G_LogWeaponPowerup(client: c_int, powerupid: c_int) {
    unsafe {
        if client >= MAX_CLIENTS as c_int {
            return;
        }
        (*addr_of_mut!(G_WeaponLogPowerups))[client as usize][powerupid as usize] += 1;
        (*addr_of_mut!(G_WeaponLogClientTouch))[client as usize] = QTRUE;
    }
}

pub fn G_LogWeaponItem(client: c_int, itemid: c_int) {
    unsafe {
        if client >= MAX_CLIENTS as c_int {
            return;
        }
        (*addr_of_mut!(G_WeaponLogItems))[client as usize][itemid as usize] += 1;
        (*addr_of_mut!(G_WeaponLogClientTouch))[client as usize] = QTRUE;
    }
}

// Run through each player.  Print out:
//	-- Most commonly picked up weapon.
//  -- Weapon with which the most time was spent.
//  -- Weapon that was most often died with.
//  -- Damage type with which the most damage was done.
//  -- Damage type with the most kills.
//  -- Weapon with which the most damage was done.
//	-- Weapon with which the most damage was done per shot.
//
// For the whole game, print out:
//  -- Total pickups of each weapon.
//  -- Total time spent with each weapon.
//  -- Total damage done with each weapon.
//  -- Total damage done for each damage type.
//  -- Number of kills with each weapon.
//  -- Number of kills for each damage type.
//  -- Damage per shot with each weapon.
//  -- Number of deaths with each weapon.
pub fn G_LogWeaponOutput() {
    // #ifdef LOGGING_WEAPONS
    unsafe {
        let mut curwp: c_int;
        let weaponfile;
        let mut string: [c_char; 1024] = [0; 1024];

        let mut totalpickups: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
        let mut totaltime: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
        let mut totaldeaths: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
        let mut totaldamageMOD: [c_int; MOD_MAX as usize] = [0; MOD_MAX as usize];
        let mut totalkillsMOD: [c_int; MOD_MAX as usize] = [0; MOD_MAX as usize];
        let mut totaldamage: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
        let mut totalkills: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
        let mut totalshots: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
        let mut percharacter: [c_int; WP_NUM_WEAPONS as usize];
        let mut mapname: [c_char; 128] = [0; 128];
        let mut nameptr: *const c_char;
        let unknownname: *const c_char = c"<Unknown>".as_ptr();

        if (*addr_of!(g_statLog)).integer == 0 {
            return;
        }

        G_LogPrintf("*****************************Weapon Log:\n");

        // (the totals are zero-initialised above, matching the C `memset`s.)

        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!
                for j in 0..WP_NUM_WEAPONS as usize {
                    totalpickups[j] += (*addr_of!(G_WeaponLogPickups))[i][j];
                    totaltime[j] += (*addr_of!(G_WeaponLogTime))[i][j];
                    totaldeaths[j] += (*addr_of!(G_WeaponLogDeaths))[i][j];
                    totalshots[j] += (*addr_of!(G_WeaponLogFired))[i][j];
                }

                for j in 0..MOD_MAX as usize {
                    totaldamageMOD[j] += (*addr_of!(G_WeaponLogDamage))[i][j];
                    totalkillsMOD[j] += (*addr_of!(G_WeaponLogKills))[i][j];
                }
            }
        }

        // Now total the weapon data from the MOD data.
        for j in 0..MOD_MAX {
            if j <= MOD_SENTRY {
                curwp = weaponFromMOD[j as usize];
                totaldamage[curwp as usize] += totaldamageMOD[j as usize];
                totalkills[curwp as usize] += totalkillsMOD[j as usize];
            }
        }

        G_LogPrintf("\n****Data by Weapon:\n");
        for j in 0..WP_NUM_WEAPONS as usize {
            G_LogPrintf(&format!(
                "{:>15}:  Pickups: {:>4},  Time:  {:>5},  Deaths: {:>5}\n",
                name_or_empty((*addr_of!(weaponNameFromIndex))[j]),
                totalpickups[j],
                totaltime[j] / 1000,
                totaldeaths[j]
            ));
        }

        G_LogPrintf("\n****Combat Data by Weapon:\n");
        for j in 0..WP_NUM_WEAPONS as usize {
            let pershot = if totalshots[j] > 0 {
                (totaldamage[j] as f32) / (totalshots[j] as f32)
            } else {
                0.0
            };
            G_LogPrintf(&format!(
                "{:>15}:  Damage: {:>6},  Kills: {:>5},  Dmg per Shot: {:.6}\n",
                name_or_empty((*addr_of!(weaponNameFromIndex))[j]),
                totaldamage[j],
                totalkills[j],
                pershot
            ));
        }

        G_LogPrintf("\n****Combat Data By Damage Type:\n");
        for j in 0..MOD_MAX as usize {
            G_LogPrintf(&format!(
                "{:>25}:  Damage: {:>6},  Kills: {:>5}\n",
                name_or_empty((*addr_of!(modNames))[j]),
                totaldamageMOD[j],
                totalkillsMOD[j]
            ));
        }

        G_LogPrintf("\n");

        // Write the whole weapon statistic log out to a file.
        let (_l, wf) = trap::FS_FOpenFile(
            &CStr::from_ptr((*addr_of!(g_statLogFile)).string.as_ptr()).to_string_lossy(),
            FS_APPEND,
        );
        weaponfile = wf;
        if weaponfile == 0 {
            // failed to open file, let's not crash, shall we?
            return;
        }

        // `Com_sprintf(string, sizeof(string), fmt, ...); trap_FS_Write(string, strlen, wf)`.
        macro_rules! emit {
            ($($arg:tt)*) => {{
                Com_sprintf(string.as_mut_ptr(), 1024, format_args!($($arg)*));
                let n = strlen(string.as_ptr());
                trap::FS_Write(
                    core::slice::from_raw_parts(string.as_ptr() as *const u8, n),
                    weaponfile,
                );
            }};
        }
        // `trap_FS_Write(nameptr, strlen(nameptr), wf)` — write a raw C buffer directly.
        macro_rules! wbytes {
            ($p:expr) => {{
                let p_ = $p;
                trap::FS_Write(
                    core::slice::from_raw_parts(p_ as *const u8, strlen(p_)),
                    weaponfile,
                );
            }};
        }

        // Write out the level name
        let info = trap::GetServerinfo();
        let info_c = std::ffi::CString::new(info).unwrap_or_default();
        strncpy(
            mapname.as_mut_ptr(),
            Info_ValueForKey(info_c.as_ptr(), c"mapname".as_ptr()),
            128 - 1,
        );
        mapname[128 - 1] = b'\0' as c_char;

        emit!("\n\n\nLevel:\t{}\n\n\n", Sz(mapname.as_ptr()));

        // Combat data per character

        // Start with Pickups per character
        emit!("Weapon Pickups per Player:\n\n");

        emit!("Player");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", Sz((*addr_of!(weaponNameFromIndex))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!
                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..WP_NUM_WEAPONS as usize {
                    emit!("\t{}", (*addr_of!(G_WeaponLogPickups))[i][j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", totalpickups[j]);
        }

        emit!("\n\n\n");

        // Weapon fires per character
        emit!("Weapon Shots per Player:\n\n");

        emit!("Player");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", Sz((*addr_of!(weaponNameFromIndex))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!
                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..WP_NUM_WEAPONS as usize {
                    emit!("\t{}", (*addr_of!(G_WeaponLogFired))[i][j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", totalshots[j]);
        }

        emit!("\n\n\n");

        // Weapon time per character
        emit!("Weapon Use Time per Player:\n\n");

        emit!("Player");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", Sz((*addr_of!(weaponNameFromIndex))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!
                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..WP_NUM_WEAPONS as usize {
                    emit!("\t{}", (*addr_of!(G_WeaponLogTime))[i][j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", totaltime[j]);
        }

        emit!("\n\n\n");

        // Weapon deaths per character
        emit!("Weapon Deaths per Player:\n\n");

        emit!("Player");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", Sz((*addr_of!(weaponNameFromIndex))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!
                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..WP_NUM_WEAPONS as usize {
                    emit!("\t{}", (*addr_of!(G_WeaponLogDeaths))[i][j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", totaldeaths[j]);
        }

        emit!("\n\n\n");

        // Weapon damage per character

        emit!("Weapon Damage per Player:\n\n");

        emit!("Player");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", Sz((*addr_of!(weaponNameFromIndex))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!

                // We must grab the totals from the damage types for the player and map them to the weapons.
                percharacter = [0; WP_NUM_WEAPONS as usize];
                for j in 0..MOD_MAX {
                    if j <= MOD_SENTRY {
                        curwp = weaponFromMOD[j as usize];
                        percharacter[curwp as usize] +=
                            (*addr_of!(G_WeaponLogDamage))[i][j as usize];
                    }
                }

                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..WP_NUM_WEAPONS as usize {
                    emit!("\t{}", percharacter[j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", totaldamage[j]);
        }

        emit!("\n\n\n");

        // Weapon kills per character

        emit!("Weapon Kills per Player:\n\n");

        emit!("Player");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", Sz((*addr_of!(weaponNameFromIndex))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!

                // We must grab the totals from the damage types for the player and map them to the weapons.
                percharacter = [0; WP_NUM_WEAPONS as usize];
                for j in 0..MOD_MAX {
                    if j <= MOD_SENTRY {
                        curwp = weaponFromMOD[j as usize];
                        percharacter[curwp as usize] +=
                            (*addr_of!(G_WeaponLogKills))[i][j as usize];
                    }
                }

                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..WP_NUM_WEAPONS as usize {
                    emit!("\t{}", percharacter[j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..WP_NUM_WEAPONS as usize {
            emit!("\t{}", totalkills[j]);
        }

        emit!("\n\n\n");

        // Damage type damage per character
        emit!("Typed Damage per Player:\n\n");

        emit!("Player");

        for j in 0..MOD_MAX as usize {
            emit!("\t{}", Sz((*addr_of!(modNames))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!
                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..MOD_MAX as usize {
                    emit!("\t{}", (*addr_of!(G_WeaponLogDamage))[i][j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..MOD_MAX as usize {
            emit!("\t{}", totaldamageMOD[j]);
        }

        emit!("\n\n\n");

        // Damage type kills per character
        emit!("Damage-Typed Kills per Player:\n\n");

        emit!("Player");

        for j in 0..MOD_MAX as usize {
            emit!("\t{}", Sz((*addr_of!(modNames))[j]));
        }
        emit!("\n");

        // Cycle through each player, give their name and the number of times they picked up each weapon.
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogClientTouch))[i] != 0 {
                // Ignore any entity/clients we don't care about!
                if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i))
                    .client
                    .is_null()
                {
                    nameptr = (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(i))
                    .client)
                        .pers
                        .netname
                        .as_ptr();
                } else {
                    nameptr = unknownname;
                }
                wbytes!(nameptr);

                for j in 0..MOD_MAX as usize {
                    emit!("\t{}", (*addr_of!(G_WeaponLogKills))[i][j]);
                }

                emit!("\n");
            }
        }

        // Sum up the totals.
        emit!("\n***TOTAL:");

        for j in 0..MOD_MAX as usize {
            emit!("\t{}", totalkillsMOD[j]);
        }

        emit!("\n\n\n");

        trap::FS_FCloseFile(weaponfile);
    }
}

// ===========================================================================
// End-of-round award calculations. All `#ifdef LOGGING_WEAPONS`; no oracle
// (entity/cvar/`level` global reads). Locals keep their C `camelCase` names for
// diff fidelity (the file-level `non_snake_case` allow covers them).
//
// DEVIATION: faithful to the C, the per-minute ratio tests divide by `playTime`
// (an `int`) without a zero-guard, so a sub-minute `playTime` of 0 yields an
// IEEE inf/nan exactly as the original float math does.
// ===========================================================================

pub unsafe fn CalculateEfficiency(ent: *mut gentity_t, efficiency: *mut c_int) -> qboolean {
    let mut fBestRatio: f32 = 0.0;
    let mut nBestPlayer: c_int = -1;

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE {
            continue;
        }
        let nShotsFired = (*(*player).client).accuracy_shots; // ps.persistant[PERS_ACCURACY_SHOTS]
        let nShotsHit = (*(*player).client).accuracy_hits; // ps.persistant[PERS_ACCURACY_HITS]
        let fAccuracyRatio = (nShotsHit as f32) / (nShotsFired as f32);
        if fAccuracyRatio > fBestRatio {
            fBestRatio = fAccuracyRatio;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        // huh?
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        let tempEff = (100.0 * fBestRatio) as c_int;
        if tempEff > 50 {
            *efficiency = tempEff;
            return QTRUE;
        }
        return QFALSE;
    }
    QFALSE
}

// did this player earn the sharpshooter award?
pub unsafe fn CalculateSharpshooter(ent: *mut gentity_t, frags: *mut c_int) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nMostKills: c_int = 0;
    let playTime: c_int = ((*addr_of!(level)).time - (*(*ent).client).pers.enterTime) / 60000;

    let entIdx = ent.offset_from(addr_of_mut!(g_entities).cast::<gentity_t>());
    // if this guy didn't get one kill per minute, reject him right now
    if ((*addr_of!(G_WeaponLogKills))[entIdx as usize][MOD_DISRUPTOR_SNIPER as usize] as f32)
        / (playTime as f32)
        < 1.0
    {
        return QFALSE;
    }

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE {
            continue;
        }
        let nKills = (*addr_of!(G_WeaponLogKills))[i as usize][MOD_DISRUPTOR_SNIPER as usize];
        if nKills > nMostKills {
            nMostKills = nKills;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        *frags = nMostKills;
        return QTRUE;
    }
    QFALSE
}

// did this player earn the untouchable award?
pub unsafe fn CalculateUntouchable(ent: *mut gentity_t) -> qboolean {
    let playTime: c_int = ((*addr_of!(level)).time - (*(*ent).client).pers.enterTime) / 60000;

    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER
        && (*(*ent).client).ps.isJediMaster != QFALSE
    {
        // Jedi Master (was Borg queen) can only be killed once anyway
        return QFALSE;
    }
    //------------------------------------------------------ MUST HAVE ACHIEVED 2 KILLS PER MINUTE
    if ((*(*ent).client).ps.persistant[PERS_SCORE as usize] as f32) / (playTime as f32) < 2.0
        || playTime == 0
    {
        return QFALSE;
    }
    //------------------------------------------------------ MUST HAVE ACHIEVED 2 KILLS PER MINUTE

    // if this guy was never killed...  Award Away!!!
    if (*(*ent).client).ps.persistant[PERS_KILLED as usize] == 0 {
        return QTRUE;
    }
    QFALSE
}

// did this player earn the logistics award?
pub unsafe fn CalculateLogistics(ent: *mut gentity_t, stuffUsed: *mut c_int) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nMostStuffUsed: c_int = 0;
    let mut nMostDifferent: c_int = 0;

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let mut nStuffUsed: c_int = 0;
        let mut nDifferent: c_int = 0;
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE {
            continue;
        }
        for j in (HI_NONE + 1)..HI_NUM_HOLDABLE {
            if (*addr_of!(G_WeaponLogPowerups))[i as usize][j as usize] != 0 {
                nDifferent += 1;
            }
            nStuffUsed += (*addr_of!(G_WeaponLogPowerups))[i as usize][j as usize];
        }
        for j in (PW_NONE + 1)..PW_NUM_POWERUPS {
            if (*addr_of!(G_WeaponLogItems))[i as usize][j as usize] != 0 {
                nDifferent += 1;
            }
            nStuffUsed += (*addr_of!(G_WeaponLogItems))[i as usize][j as usize];
        }
        if nDifferent >= 4 && nDifferent >= nMostDifferent && nStuffUsed > nMostStuffUsed {
            nMostDifferent = nDifferent;
            nMostStuffUsed = nStuffUsed;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        *stuffUsed = nMostDifferent;
        return QTRUE;
    }
    QFALSE
}

// did this player earn the tactician award?
pub unsafe fn CalculateTactician(ent: *mut gentity_t, kills: *mut c_int) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nMostKills: c_int = 0;
    let mut wasPickedUpBySomeone: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
    let mut killsWithWeapon: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];
    let playTime: c_int = ((*addr_of!(level)).time - (*(*ent).client).pers.enterTime) / 60000;

    if HasSetSaberOnly() != QFALSE {
        // duh, only 1 weapon
        return QFALSE;
    }
    if (*addr_of!(g_gametype)).integer == GT_JEDIMASTER
        && (*(*ent).client).ps.isJediMaster != QFALSE
    {
        // Jedi Master (was Borg queen) has only 1 weapon
        return QFALSE;
    }
    //------------------------------------------------------ MUST HAVE ACHIEVED 2 KILLS PER MINUTE
    if (playTime as f64) < 0.3 {
        return QFALSE;
    }

    if ((*(*ent).client).ps.persistant[PERS_SCORE as usize] as f32) / (playTime as f32) < 2.0 {
        return QFALSE;
    }
    //------------------------------------------------------ MUST HAVE ACHIEVED 2 KILLS PER MINUTE

    //------------------------------------------------------ FOR EVERY WEAPON, ADD UP TOTAL PICKUPS
    // (wasPickedUpBySomeone already CLEAR)
    for person in 0..(*addr_of!(g_maxclients)).integer {
        for weapon in 0..WP_NUM_WEAPONS as usize {
            if (*addr_of!(G_WeaponLogPickups))[person as usize][weapon] > 0 {
                wasPickedUpBySomeone[weapon] += 1;
            }
        }
    }
    //------------------------------------------------------ FOR EVERY WEAPON, ADD UP TOTAL PICKUPS

    //------------------------------------------------------ FOR EVERY PERSON, CHECK FOR CANDIDATE
    for person in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities)
            .cast::<gentity_t>()
            .add(person as usize);
        if (*player).inuse == QFALSE {
            continue;
        }

        let mut nKills: c_int = 0; // This Persons's Kills
        for w in killsWithWeapon.iter_mut() {
            *w = 0; // CLEAR
        }

        for i in 0..MOD_MAX as usize {
            let weapon = weaponFromMOD[i]; // Select Weapon
            killsWithWeapon[weapon as usize] += (*addr_of!(G_WeaponLogKills))[person as usize][i];
        }

        let mut weapon = WP_STUN_BATON; // Start At Stun Baton
                                        //   keep looking through weapons if weapon is not on map, or if it is and we used it
        while weapon < WP_NUM_WEAPONS
            && (wasPickedUpBySomeone[weapon as usize] == 0 || killsWithWeapon[weapon as usize] > 0)
        {
            weapon += 1;
            nKills += killsWithWeapon[weapon as usize]; // Update the number of kills
        }
        //
        // At this point we have either successfully gone through every weapon on the map and saw it had
        // been used, or we found one that WAS on the map and was NOT used
        //
        if weapon >= WP_NUM_WEAPONS && nKills > nMostKills {
            // WE ARE A TACTICION CANDIDATE
            nMostKills = nKills;
            nBestPlayer = person;
        }
    }
    //------------------------------------------------------ FOR EVERY PERSON, CHECK FOR CANDIDATE

    // Now, if we are the best player, return true and the number of kills we got
    if nBestPlayer == (*ent).s.number {
        *kills = nMostKills;
        return QTRUE;
    }
    QFALSE
}

// did this player earn the demolitionist award?
pub unsafe fn CalculateDemolitionist(ent: *mut gentity_t, kills: *mut c_int) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nMostKills: c_int = 0;
    let playTime: c_int = ((*addr_of!(level)).time - (*(*ent).client).pers.enterTime) / 60000;

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE {
            continue;
        }

        let logKills = &(*addr_of!(G_WeaponLogKills))[i as usize];
        let mut nKills: c_int = logKills[MOD_THERMAL as usize];
        nKills += logKills[MOD_THERMAL_SPLASH as usize];
        nKills += logKills[MOD_ROCKET as usize];
        nKills += logKills[MOD_ROCKET_SPLASH as usize];
        nKills += logKills[MOD_ROCKET_HOMING as usize];
        nKills += logKills[MOD_ROCKET_HOMING_SPLASH as usize];
        nKills += logKills[MOD_TRIP_MINE_SPLASH as usize];
        nKills += logKills[MOD_TIMED_MINE_SPLASH as usize];
        nKills += logKills[MOD_DET_PACK_SPLASH as usize];

        // if this guy didn't get two explosive kills per minute, reject him right now
        if (nKills as f32) / (playTime as f32) < 2.0 {
            continue;
        }

        if nKills > nMostKills {
            nMostKills = nKills;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        *kills = nMostKills;
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn CalculateStreak(_ent: *mut gentity_t) -> c_int {
    // The streak body is `#if 0`'d out in the C source (PERS_STREAK_COUNT machinery).
    // No streak calculation, at least for now.
    0
}

pub unsafe fn CalculateTeamMVP(ent: *mut gentity_t) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nHighestScore: c_int = 0;
    let team = (*(*ent).client).ps.persistant[PERS_TEAM as usize];

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE
            || (*(*player).client).ps.persistant[PERS_TEAM as usize] != team
        {
            continue;
        }
        let nScore = (*(*player).client).ps.persistant[PERS_SCORE as usize];
        if nScore > nHighestScore {
            nHighestScore = nScore;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn CalculateTeamMVPByRank(ent: *mut gentity_t) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nHighestScore: c_int = 0;
    let team = (*(*ent).client).ps.persistant[PERS_RANK as usize] + 1;
    let bTied: qboolean = if team == 3 { QTRUE } else { QFALSE };

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE {
            continue;
        }
        if bTied == QFALSE && (*(*player).client).ps.persistant[PERS_TEAM as usize] != team {
            continue;
        }
        let nScore = (*(*player).client).ps.persistant[PERS_SCORE as usize];
        if nScore > nHighestScore {
            nHighestScore = nScore;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn CalculateTeamDefender(ent: *mut gentity_t) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nHighestScore: c_int = 0;
    let team = (*(*ent).client).ps.persistant[PERS_TEAM as usize];

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE
            || (*(*player).client).ps.persistant[PERS_TEAM as usize] != team
        {
            continue;
        }
        let nScore = (*(*player).client).pers.teamState.basedefense;
        if nScore > nHighestScore {
            nHighestScore = nScore;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn CalculateTeamWarrior(ent: *mut gentity_t) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nHighestScore: c_int = 0;
    let team = (*(*ent).client).ps.persistant[PERS_TEAM as usize];

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE
            || (*(*player).client).ps.persistant[PERS_TEAM as usize] != team
        {
            continue;
        }
        let nScore = (*(*player).client).ps.persistant[PERS_SCORE as usize];
        if nScore > nHighestScore {
            nHighestScore = nScore;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn CalculateTeamCarrier(ent: *mut gentity_t) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nHighestScore: c_int = 0;
    let team = (*(*ent).client).ps.persistant[PERS_TEAM as usize];

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE
            || (*(*player).client).ps.persistant[PERS_TEAM as usize] != team
        {
            continue;
        }
        let nScore = (*(*player).client).pers.teamState.captures;
        if nScore > nHighestScore {
            nHighestScore = nScore;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn CalculateTeamInterceptor(ent: *mut gentity_t) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nHighestScore: c_int = 0;
    let team = (*(*ent).client).ps.persistant[PERS_TEAM as usize];

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE
            || (*(*player).client).ps.persistant[PERS_TEAM as usize] != team
        {
            continue;
        }
        let mut nScore = (*(*player).client).pers.teamState.flagrecovery;
        nScore += (*(*player).client).pers.teamState.fragcarrier;
        if nScore > nHighestScore {
            nHighestScore = nScore;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        return QTRUE;
    }
    QFALSE
}

pub unsafe fn CalculateTeamRedShirt(ent: *mut gentity_t) -> qboolean {
    let mut nBestPlayer: c_int = -1;
    let mut nHighestScore: c_int = 0;
    let team = (*(*ent).client).ps.persistant[PERS_TEAM as usize];

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE
            || (*(*player).client).ps.persistant[PERS_TEAM as usize] != team
        {
            continue;
        }
        let mut nScore = (*(*player).client).ps.persistant[PERS_KILLED as usize];
        nScore -= (*(*player).client).ps.fd.suicides; // suicides don't count, you big cheater.
        if nScore > nHighestScore {
            nHighestScore = nScore;
            nBestPlayer = i;
        }
    }
    if nBestPlayer == -1 {
        return QFALSE;
    }
    if nBestPlayer == (*ent).s.number {
        return QTRUE;
    }
    QFALSE
}

// awardType_t — bit positions in the `awardFlags` mask built by CalculateAwards.
const AWARD_EFFICIENCY: c_int = 0; // Accuracy
const AWARD_SHARPSHOOTER: c_int = 1; // Most compression rifle frags
const AWARD_UNTOUCHABLE: c_int = 2; // Perfect (no deaths)
const AWARD_LOGISTICS: c_int = 3; // Most pickups
const AWARD_TACTICIAN: c_int = 4; // Kills with all weapons
const AWARD_DEMOLITIONIST: c_int = 5; // Most explosive damage kills
const AWARD_STREAK: c_int = 6; // Ace/Expert/Master/Champion
const AWARD_TEAM: c_int = 7; // MVP/Defender/Warrior/Carrier/Interceptor/Bravery
const AWARD_SECTION31: c_int = 8; // All-around god

// teamAward_e — bit positions in the team-award mask returned by CalculateTeamAward.
const TEAM_MVP: c_int = 1; // most overall points
const TEAM_DEFENDER: c_int = 2; // killed the most baddies near your flag
const TEAM_WARRIOR: c_int = 3; // most frags
const TEAM_CARRIER: c_int = 4; // infected the most people with plague
const TEAM_INTERCEPTOR: c_int = 5; // returned your own flag the most
const TEAM_BRAVERY: c_int = 6; // Red Shirt Award (tm). you died more than anybody.

pub unsafe fn CalculateTeamAward(ent: *mut gentity_t) -> c_int {
    let mut teamAwards: c_int = 0;

    if CalculateTeamMVP(ent) != QFALSE {
        teamAwards |= 1 << TEAM_MVP;
    }
    if GT_CTF == (*addr_of!(g_gametype)).integer || GT_CTY == (*addr_of!(g_gametype)).integer {
        if CalculateTeamDefender(ent) != QFALSE {
            teamAwards |= 1 << TEAM_DEFENDER;
        }
        if CalculateTeamWarrior(ent) != QFALSE {
            teamAwards |= 1 << TEAM_WARRIOR;
        }
        if CalculateTeamCarrier(ent) != QFALSE {
            teamAwards |= 1 << TEAM_CARRIER;
        }
        if CalculateTeamInterceptor(ent) != QFALSE {
            teamAwards |= 1 << TEAM_INTERCEPTOR;
        }
    }
    if teamAwards == 0 && CalculateTeamRedShirt(ent) != QFALSE {
        // if you got nothing else and died a lot, at least get bravery
        teamAwards |= 1 << TEAM_BRAVERY;
    }
    teamAwards
}

pub unsafe fn CalculateSection31Award(ent: *mut gentity_t) -> qboolean {
    let mut frags: c_int = 0;
    let mut efficiency: c_int = 0;

    for i in 0..(*addr_of!(g_maxclients)).integer {
        let player = addr_of_mut!(g_entities).cast::<gentity_t>().add(i as usize);
        if (*player).inuse == QFALSE {
            continue;
        }
        // kef -- heh. (the JaxxonPhred netname check is commented out in the C)
        // Faithful quirk: these all test `ent`, not the loop's `player`.
        CalculateEfficiency(ent, &mut efficiency);
        if CalculateSharpshooter(ent, &mut frags) == QFALSE
            || CalculateUntouchable(ent) == QFALSE
            || efficiency < 75
        {
            continue;
        }
        return QTRUE;
    }
    QFALSE
}

const AWARDS_MSG_LENGTH: c_int = 256;

pub unsafe fn CalculateAwards(ent: *mut gentity_t, msg: *mut c_char) {
    let mut buf1: [c_char; AWARDS_MSG_LENGTH as usize] = [0; AWARDS_MSG_LENGTH as usize];
    let mut buf2: [c_char; AWARDS_MSG_LENGTH as usize] = [0; AWARDS_MSG_LENGTH as usize];
    let mut awardFlags: c_int = 0;
    let mut efficiency: c_int = 0;
    let mut stuffUsed: c_int = 0;
    let mut kills: c_int = 0;

    // buf1/buf2 already zeroed (the C memsets them).
    if CalculateEfficiency(ent, &mut efficiency) != QFALSE {
        awardFlags |= 1 << AWARD_EFFICIENCY;
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!(" {}", efficiency),
        );
    }
    if CalculateSharpshooter(ent, &mut kills) != QFALSE {
        awardFlags |= 1 << AWARD_SHARPSHOOTER;
        strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!("{} {}", Sz(buf2.as_ptr()), kills),
        );
    }
    if CalculateUntouchable(ent) != QFALSE {
        awardFlags |= 1 << AWARD_UNTOUCHABLE;
        strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!("{} {}", Sz(buf2.as_ptr()), 0),
        );
    }
    if CalculateLogistics(ent, &mut stuffUsed) != QFALSE {
        awardFlags |= 1 << AWARD_LOGISTICS;
        strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!("{} {}", Sz(buf2.as_ptr()), stuffUsed),
        );
    }
    if CalculateTactician(ent, &mut kills) != QFALSE {
        awardFlags |= 1 << AWARD_TACTICIAN;
        strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!("{} {}", Sz(buf2.as_ptr()), kills),
        );
    }
    if CalculateDemolitionist(ent, &mut kills) != QFALSE {
        awardFlags |= 1 << AWARD_DEMOLITIONIST;
        strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!("{} {}", Sz(buf2.as_ptr()), kills),
        );
    }
    let streak = CalculateStreak(ent);
    if streak != 0 {
        awardFlags |= 1 << AWARD_STREAK;
        strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!("{} {}", Sz(buf2.as_ptr()), streak),
        );
    }
    if (*addr_of!(g_gametype)).integer >= GT_TEAM {
        let teamAwards = CalculateTeamAward(ent);
        if teamAwards != 0 {
            awardFlags |= 1 << AWARD_TEAM;
            strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
            Com_sprintf(
                buf1.as_mut_ptr(),
                AWARDS_MSG_LENGTH,
                format_args!("{} {}", Sz(buf2.as_ptr()), teamAwards),
            );
        }
    }
    if CalculateSection31Award(ent) != QFALSE {
        awardFlags |= 1 << AWARD_SECTION31;
        strcpy(buf2.as_mut_ptr(), buf1.as_ptr());
        Com_sprintf(
            buf1.as_mut_ptr(),
            AWARDS_MSG_LENGTH,
            format_args!("{} {}", Sz(buf2.as_ptr()), 0),
        );
    }
    strcpy(buf2.as_mut_ptr(), msg);
    Com_sprintf(
        msg,
        AWARDS_MSG_LENGTH,
        format_args!("{} {}{}", Sz(buf2.as_ptr()), awardFlags, Sz(buf1.as_ptr())),
    );
}

pub fn GetMaxDeathsForClient(nClient: c_int) -> c_int {
    let mut nMostDeaths: c_int = 0;

    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogFrags))[i][nClient as usize] > nMostDeaths {
                nMostDeaths = (*addr_of!(G_WeaponLogFrags))[i][nClient as usize];
            }
        }
    }
    nMostDeaths
}

pub fn GetMaxKillsForClient(nClient: c_int) -> c_int {
    let mut nMostKills: c_int = 0;

    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogFrags))[nClient as usize][i] > nMostKills {
                nMostKills = (*addr_of!(G_WeaponLogFrags))[nClient as usize][i];
            }
        }
    }
    nMostKills
}

pub fn GetFavoriteTargetForClient(nClient: c_int) -> c_int {
    let mut nMostKills: c_int = 0;
    let mut nFavoriteTarget: c_int = -1;

    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        for i in 0..MAX_CLIENTS {
            if (*addr_of!(G_WeaponLogFrags))[nClient as usize][i] > nMostKills {
                nMostKills = (*addr_of!(G_WeaponLogFrags))[nClient as usize][i];
                nFavoriteTarget = i as c_int;
            }
        }
    }
    nFavoriteTarget
}

pub fn GetWorstEnemyForClient(nClient: c_int) -> c_int {
    let mut nMostDeaths: c_int = 0;
    let mut nWorstEnemy: c_int = -1;

    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        for i in 0..MAX_CLIENTS {
            // If there is a tie for most deaths, we want to choose anybody else
            // over the client...  I.E. Most deaths should not tie with yourself and
            // have yourself show up...
            let frags = (*addr_of!(G_WeaponLogFrags))[i][nClient as usize];
            if frags > nMostDeaths
                || (frags == nMostDeaths && i as c_int != nClient && nMostDeaths != 0)
            {
                nMostDeaths = frags;
                nWorstEnemy = i as c_int;
            }
        }
    }
    nWorstEnemy
}

pub fn GetFavoriteWeaponForClient(nClient: c_int) -> c_int {
    let mut nMostKills: c_int;
    let mut fav: c_int = 0;
    let mut killsWithWeapon: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];

    // First thing we need to do is cycle through all the MOD types and convert
    // number of kills to a single weapon.
    //----------------------------------------------------------------
    // (killsWithWeapon already CLEAR)
    unsafe {
        for i in MOD_STUN_BATON..=MOD_FORCE_DARK {
            let weapon = weaponFromMOD[i as usize]; // Select Weapon
            if weapon != WP_NONE {
                // Store Num Kills With Weapon
                killsWithWeapon[weapon as usize] +=
                    (*addr_of!(G_WeaponLogKills))[nClient as usize][i as usize];
            }
        }
    }

    // now look through our list of kills per weapon and pick the biggest
    //----------------------------------------------------------------
    nMostKills = 0;
    for weapon in WP_STUN_BATON..WP_NUM_WEAPONS {
        if killsWithWeapon[weapon as usize] > nMostKills {
            nMostKills = killsWithWeapon[weapon as usize];
            fav = weapon;
        }
    }
    fav
}

// kef -- if a client leaves the game, clear out all counters he may have set
pub fn G_ClearClientLog(client: c_int) {
    let c = client as usize;
    unsafe {
        for i in 0..WP_NUM_WEAPONS as usize {
            (*addr_of_mut!(G_WeaponLogPickups))[c][i] = 0;
        }
        for i in 0..WP_NUM_WEAPONS as usize {
            (*addr_of_mut!(G_WeaponLogFired))[c][i] = 0;
        }
        for i in 0..MOD_MAX as usize {
            (*addr_of_mut!(G_WeaponLogDamage))[c][i] = 0;
        }
        for i in 0..MOD_MAX as usize {
            (*addr_of_mut!(G_WeaponLogKills))[c][i] = 0;
        }
        for i in 0..WP_NUM_WEAPONS as usize {
            (*addr_of_mut!(G_WeaponLogDeaths))[c][i] = 0;
        }
        for i in 0..MAX_CLIENTS {
            (*addr_of_mut!(G_WeaponLogFrags))[c][i] = 0;
        }
        for i in 0..MAX_CLIENTS {
            (*addr_of_mut!(G_WeaponLogFrags))[i][c] = 0;
        }
        for i in 0..WP_NUM_WEAPONS as usize {
            (*addr_of_mut!(G_WeaponLogTime))[c][i] = 0;
        }
        (*addr_of_mut!(G_WeaponLogLastTime))[c] = 0;
        (*addr_of_mut!(G_WeaponLogClientTouch))[c] = QFALSE;
        for i in 0..HI_NUM_HOLDABLE as usize {
            (*addr_of_mut!(G_WeaponLogPowerups))[c][i] = 0;
        }
        for i in 0..PW_NUM_POWERUPS as usize {
            (*addr_of_mut!(G_WeaponLogItems))[c][i] = 0;
        }
    }
}
