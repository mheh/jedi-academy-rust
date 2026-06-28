//! `bg_misc.c` — background (shared client/server) miscellany.
//!
//! Ported from `refs/raven-jediacademy/codemp/game/bg_misc.c`. This module starts with the
//! **bg memory subsystem** (`bg_misc.c:3325-3413`, the non-`_XBOX` branch): the
//! fixed-pool head/tail bump allocator that backs `BG_Alloc`/`BG_TempAlloc` and the
//! whole vehicle/animation/NPC-data load path. Porting it is what unblocks the
//! `bg_vehicleLoad.c` parser (every `VF_LSTRING` field calls `BG_Alloc`).
//!
//! The early "rest of `bg_misc.c` is deferred on `bg_strap.h`" caveat is **long
//! superseded** — the file's functions (`BG_FindItem*`, the item selectors,
//! `BG_EvaluateTrajectory*`, `BG_PlayerStateToEntityState*`, the force/inventory
//! cyclers, `BG_LegalizedForcePowers`, `BG_ParseField`, `BG_ModelCache`, …), the
//! `bg_itemlist`/`gitem_t` master table, and all file-scope data tables have landed.
//! The only excluded piece is the `eventnames[]` table, which is reachable solely from
//! the `#ifdef _DEBUG` `showevents` block (undefined in this build) and is carried as a
//! comment inside [`BG_AddPredictableEventToPlayerstate`].
//!
//! One pool `bg_pool` is divided in two: `BG_Alloc`/`BG_AllocUnaligned` hand out
//! from the head (`bg_poolSize`, growing up), `BG_TempAlloc`/`BG_TempFree` from the
//! tail (`bg_poolTail`, growing down); they meet in the middle. Like `g_mem.c`'s
//! `G_Alloc`, nothing is freed individually (temp allocs excepted). The game module
//! is `QAGAME`, so `MAX_POOL_SIZE` is the 3 MB QAGAME size.
//!
//! No C oracle: like `g_mem.rs` / the trap layer, the behaviour is engine-I/O
//! (`Com_Error`) plus mutation of a file-private static pool, not a computable data
//! table. The only arithmetic — the `(n + 3) & ~3` 4-byte round-up and the bounds
//! checks — is a verbatim transcription.

#![allow(non_snake_case)] // C function names (`BG_Alloc`, ...) kept verbatim
#![allow(non_upper_case_globals)] // C static names (`bg_pool`, `bg_poolSize`) kept

use core::ffi::{c_char, c_int, c_void, CStr};
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::anims::{
    BOTH_ATTACK1, BOTH_ATTACK2, BOTH_ATTACK3, BOTH_STAND1, BOTH_STAND2, BOTH_THERMAL_THROW,
    TORSO_DROPWEAP1, TORSO_WEAPONREADY1, TORSO_WEAPONREADY10, TORSO_WEAPONREADY2,
    TORSO_WEAPONREADY3,
};
// Glob: the `bg_itemlist` table + item selectors pull in the full `IT_*`/`HI_*`/`PW_*`
// /`STAT_*` constant families plus `gitem_t`/`MAX_ITEM_MODELS` (verified collision-free
// against this file's explicit `anims`/`q_shared_h` imports — the `bg_saber.rs` precedent).
use crate::codemp::game::bg_public::*;
// Glob: the table's `giTag`s span the entire `WP_*`/`AMMO_*` weapon/ammo enum.
use crate::codemp::game::bg_weapons_h::*;
// `BG_CanItemBeGrabbed` reads the ammo-capacity columns of these two tables.
use crate::codemp::game::bg_weapons::{ammoData, weaponData};
use crate::codemp::game::g_main::Com_Error;
use crate::codemp::game::q_math::{
    vectoangles, AngleNormalize180, AngleSubtract, VectorClear, VectorCopy, VectorMA, VectorScale,
};
use crate::codemp::game::q_shared::{
    va, Q_strcat, Q_stricmp, Q_stricmpn, Q_strncmp, Q_strncpyz, Sz,
};
use crate::codemp::game::q_shared_h::{
    byte, entityState_t, playerState_t, qboolean, snap_vector, trajectory_t, vec3_t, vec_t,
    DEG2RAD, ERR_DROP, FORCE_DARKSIDE, FORCE_LEVEL_3, FORCE_LIGHTSIDE, FP_ABSORB, FP_DRAIN,
    FP_GRIP, FP_HEAL, FP_LEVITATION, FP_LIGHTNING, FP_PROTECT, FP_PULL, FP_PUSH, FP_RAGE,
    FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE, FP_SEE, FP_SPEED, FP_TEAM_FORCE,
    FP_TEAM_HEAL, FP_TELEPATHY, FS_READ, MAX_POWERUPS, MAX_PS_EVENTS, MAX_QPATH, M_PI,
    NUM_FORCE_POWERS, NUM_FORCE_POWER_LEVELS, PITCH, QFALSE, QTRUE, TR_GRAVITY, TR_INTERPOLATE,
    TR_LINEAR, TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_SINE, TR_STATIONARY, YAW,
};
// `BG_ParseField`'s parser deps: `G_NewString` (the `F_LSTRING` allocator the C
// forward-declares at bg_misc.c:342) and `sscanf` (the `F_VECTOR` parse; `vm` shim vs
// native libc binding split inside `bg_lib`).
use crate::codemp::game::bg_lib::sscanf;
use crate::codemp::game::g_spawn::G_NewString;
// `BG_GiveMeVectorFromMatrix` deps: the ghoul2 bolt transform and the Eorientations
// axis flags (q_shared.h).
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, NEGATIVE_X, NEGATIVE_Y, NEGATIVE_Z, ORIGIN, POSITIVE_X, POSITIVE_Y, POSITIVE_Z,
};
use crate::trap;

extern "C" {
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
}

// #ifdef QAGAME
// #define MAX_POOL_SIZE	3000000 //1024000
const MAX_POOL_SIZE: c_int = 3000000;

//I am using this for all the stuff like NPC client structures on server/client and
//non-humanoid animations as well until/if I can get dynamic memory working properly
//with casted datatypes, which is why it is so large.

/// 64-bit-soundness deviation (see DEVIATIONS): the bare `[c_char; N]` static has
/// alignment 1, so even a perfectly pointer-width-rounded offset could resolve to an
/// under-aligned absolute address. Wrapping the pool in an `align(8)` newtype forces its
/// base address to pointer-width alignment (over-aligned on 32-bit, harmless).
#[repr(C, align(8))]
struct BgPool([c_char; MAX_POOL_SIZE as usize]);
static mut bg_pool: BgPool = BgPool([0; MAX_POOL_SIZE as usize]);
static mut bg_poolSize: c_int = 0;
static mut bg_poolTail: c_int = MAX_POOL_SIZE;

/// 64-bit-soundness deviation (see DEVIATIONS): the C aligns the bump offset to 4 bytes
/// (`& 0xfffffffc`), which suffices on 32-bit JKA where every struct aligns to ≤4. On
/// 64-bit, pointer-bearing structs handed out by `BG_Alloc` (`gclient_t`, `Vehicle_t`, …)
/// require 8-byte alignment, and Rust's typed `write_bytes`/reference creation is UB on an
/// under-aligned pointer (C's `memset` is not — hence "32-bit worked, 64-bit aborted").
/// Aligning to pointer width keeps the C math verbatim on 32-bit (4) and makes it sound on
/// 64-bit (8).
const BG_ALLOC_ALIGN: c_int = core::mem::size_of::<*const c_void>() as c_int;

pub fn BG_Alloc(size: c_int) -> *mut c_void {
    // SAFETY: the module is single-threaded; `bg_pool`/`bg_poolSize`/`bg_poolTail`
    // are this file's own statics, taken by raw pointer (never `&mut`) to stay sound.
    unsafe {
        *addr_of_mut!(bg_poolSize) =
            (*addr_of!(bg_poolSize) + (BG_ALLOC_ALIGN - 1)) & !(BG_ALLOC_ALIGN - 1);

        if *addr_of!(bg_poolSize) + size > *addr_of!(bg_poolTail) {
            Com_Error(
                ERR_DROP,
                &format!(
                    "BG_Alloc: buffer exceeded tail ({} > {})",
                    *addr_of!(bg_poolSize) + size,
                    *addr_of!(bg_poolTail)
                ),
            );
            // return 0;  -- unreachable: Com_Error (-> !) does not return.
        }

        *addr_of_mut!(bg_poolSize) += size;

        (addr_of_mut!(bg_pool.0) as *mut c_char).add((*addr_of!(bg_poolSize) - size) as usize)
            as *mut c_void
    }
}

pub fn BG_AllocUnaligned(size: c_int) -> *mut c_void {
    // SAFETY: single-threaded module; file-private statics via raw pointer.
    unsafe {
        if *addr_of!(bg_poolSize) + size > *addr_of!(bg_poolTail) {
            Com_Error(
                ERR_DROP,
                &format!(
                    "BG_AllocUnaligned: buffer exceeded tail ({} > {})",
                    *addr_of!(bg_poolSize) + size,
                    *addr_of!(bg_poolTail)
                ),
            );
            // return 0;  -- unreachable: Com_Error (-> !) does not return.
        }

        *addr_of_mut!(bg_poolSize) += size;

        (addr_of_mut!(bg_pool.0) as *mut c_char).add((*addr_of!(bg_poolSize) - size) as usize)
            as *mut c_void
    }
}

pub fn BG_TempAlloc(size: c_int) -> *mut c_void {
    // SAFETY: single-threaded module; file-private statics via raw pointer.
    unsafe {
        let size = (size + 0x00000003) & !3;

        if *addr_of!(bg_poolTail) - size < *addr_of!(bg_poolSize) {
            Com_Error(
                ERR_DROP,
                &format!(
                    "BG_TempAlloc: buffer exceeded head ({} > {})",
                    *addr_of!(bg_poolTail) - size,
                    *addr_of!(bg_poolSize)
                ),
            );
            // return 0;  -- unreachable: Com_Error (-> !) does not return.
        }

        *addr_of_mut!(bg_poolTail) -= size;

        (addr_of_mut!(bg_pool.0) as *mut c_char).add(*addr_of!(bg_poolTail) as usize) as *mut c_void
    }
}

pub fn BG_TempFree(size: c_int) {
    // SAFETY: single-threaded module; file-private statics via raw pointer.
    unsafe {
        let size = (size + 0x00000003) & !3;

        if *addr_of!(bg_poolTail) + size > MAX_POOL_SIZE {
            Com_Error(
                ERR_DROP,
                &format!(
                    "BG_TempFree: tail greater than size ({} > {})",
                    *addr_of!(bg_poolTail) + size,
                    MAX_POOL_SIZE
                ),
            );
        }

        *addr_of_mut!(bg_poolTail) += size;
    }
}

/// # Safety
/// `source` must point to a NUL-terminated string.
pub unsafe fn BG_StringAlloc(source: *const c_char) -> *mut c_char {
    let dest = BG_Alloc(strlen(source) as c_int + 1) as *mut c_char;
    strcpy(dest, source);
    dest
}

pub fn BG_OutOfMemory() -> qboolean {
    // SAFETY: single-threaded module; reads `bg_poolSize`.
    if unsafe { *addr_of!(bg_poolSize) } >= MAX_POOL_SIZE {
        QTRUE
    } else {
        QFALSE
    }
}

/// `BG_HasYsalamiri` (bg_misc.c:1794) — `qtrue` if the player is under an ysalamiri
/// effect (carrying a flag in CTY, or holding the `PW_YSALAMIRI` powerup), which
/// suppresses force-power use. Ported out of source order (ahead of the rest of the
/// still-`bg_strap.h`-blocked `bg_misc.c`) because `bg_pmove.c`'s `PM_TryRoll` needs it.
///
/// # Safety
/// `ps` must point to a valid `playerState_t`.
pub unsafe fn BG_HasYsalamiri(gametype: c_int, ps: *mut playerState_t) -> qboolean {
    if gametype == GT_CTY
        && ((*ps).powerups[PW_REDFLAG as usize] != 0 || (*ps).powerups[PW_BLUEFLAG as usize] != 0)
    {
        return QTRUE;
    }

    if (*ps).powerups[PW_YSALAMIRI as usize] != 0 {
        return QTRUE;
    }

    QFALSE
}

/// `BG_CanUseFPNow` (bg_misc.c:1810) — gate on whether `power` may be used this instant:
/// blocked by ysalamiri, force restriction / true-non-jedi, emplaced guns, vehicles,
/// duels (except saber offense/defense/levitation, or push during a saberlock), an
/// active saberlock, falling-to-death, and a broken arm (for push/pull/grip/lightning/
/// drain). Ported out of source order to unblock `bg_pmove.c`'s `PM_TryRoll`.
///
/// # Safety
/// `ps` must point to a valid `playerState_t`.
pub unsafe fn BG_CanUseFPNow(
    gametype: c_int,
    ps: *mut playerState_t,
    time: c_int,
    power: c_int,
) -> qboolean {
    if BG_HasYsalamiri(gametype, ps) != 0 {
        return QFALSE;
    }

    if (*ps).forceRestricted != 0 || (*ps).trueNonJedi != 0 {
        return QFALSE;
    }

    if (*ps).weapon == WP_EMPLACED_GUN {
        //can't use any of your powers while on an emplaced weapon
        return QFALSE;
    }

    if (*ps).m_iVehicleNum != 0 {
        //can't use powers while riding a vehicle (this may change, I don't know)
        return QFALSE;
    }

    if (*ps).duelInProgress != 0 {
        if power != FP_SABER_OFFENSE
            && power != FP_SABER_DEFENSE
            /*&& power != FP_SABERTHROW*/
            && power != FP_LEVITATION
        {
            if (*ps).saberLockFrame == 0 || power != FP_PUSH {
                return QFALSE;
            }
        }
    }

    if (*ps).saberLockFrame != 0 || (*ps).saberLockTime > time {
        if power != FP_PUSH {
            return QFALSE;
        }
    }

    if (*ps).fallingToDeath != 0 {
        return QFALSE;
    }

    if ((*ps).brokenLimbs & (1 << BROKENLIMB_RARM)) != 0
        || ((*ps).brokenLimbs & (1 << BROKENLIMB_LARM)) != 0
    {
        //powers we can't use with a broken arm
        match power {
            FP_PUSH | FP_PULL | FP_GRIP | FP_LIGHTNING | FP_DRAIN => return QFALSE,
            _ => {}
        }
    }

    QTRUE
}

/*
===============
BG_AddPredictableEventToPlayerstate

Handles the sequence numbers
===============
*/
/// `BG_AddPredictableEventToPlayerstate` (bg_misc.c:2644) — records a pmove-generated
/// event (and its parm) into the playerstate's `MAX_PS_EVENTS`-deep ring buffer and
/// bumps the sequence counter. Ported out of source order (ahead of the rest of
/// `bg_misc.c`, which is still `bg_strap.h`-blocked) because it is the dependency that
/// unblocks `bg_pmove.c`'s `PM_AddEvent`/`PM_AddEventWithParm` event helpers.
///
/// The `#ifdef _DEBUG` `showevents`-cvar diagnostic block is carried as a source-order
/// comment below: it is not compiled in this build (it needs `trap_Cvar_Register`,
/// `Com_Printf`, and the `eventnames[]` table), and `_DEBUG` is undefined for the
/// shipping module.
///
/// # Safety
/// `ps` must point to a valid `playerState_t`.
pub unsafe fn BG_AddPredictableEventToPlayerstate(
    newEvent: c_int,
    eventParm: c_int,
    ps: *mut playerState_t,
) {
    // #ifdef _DEBUG
    //   static vmCvar_t showEvents; static qboolean isRegistered = qfalse;
    //   if (!isRegistered) { trap_Cvar_Register(&showEvents,"showevents","0",0); isRegistered = qtrue; }
    //   if ( showEvents.integer != 0 )
    //     Com_Printf(" game event svt %5d -> %5d: num = %20s parm %d\n",
    //                ps->pmove_framecount, ps->eventSequence, eventnames[newEvent], eventParm);
    // #endif
    let i = ((*ps).eventSequence & (MAX_PS_EVENTS as c_int - 1)) as usize;
    (*ps).events[i] = newEvent;
    (*ps).eventParms[i] = eventParm;
    (*ps).eventSequence += 1;
}

// Pulled in out of source order (bg_misc.c:242): the torso ready-stance lookup that
// `bg_pmove.c`'s `PM_CrashLand`/`PM_Weapon` index by `ps->weapon`. A pure data table
// (no `bg_strap.h`), so it lands ahead of the not-yet-ported bulk of bg_misc.c.
#[rustfmt::skip]
pub static WeaponReadyAnim: [c_int; WP_NUM_WEAPONS as usize] = [
    TORSO_DROPWEAP1,    //WP_NONE,

    TORSO_WEAPONREADY3, //WP_STUN_BATON,
    TORSO_WEAPONREADY3, //WP_MELEE,
    BOTH_STAND2,        //WP_SABER,
    TORSO_WEAPONREADY2, //WP_BRYAR_PISTOL,
    TORSO_WEAPONREADY3, //WP_BLASTER,
    TORSO_WEAPONREADY3, //TORSO_WEAPONREADY4,//WP_DISRUPTOR,
    TORSO_WEAPONREADY3, //TORSO_WEAPONREADY5,//WP_BOWCASTER,
    TORSO_WEAPONREADY3, //TORSO_WEAPONREADY6,//WP_REPEATER,
    TORSO_WEAPONREADY3, //TORSO_WEAPONREADY7,//WP_DEMP2,
    TORSO_WEAPONREADY3, //TORSO_WEAPONREADY8,//WP_FLECHETTE,
    TORSO_WEAPONREADY3, //TORSO_WEAPONREADY9,//WP_ROCKET_LAUNCHER,
    TORSO_WEAPONREADY10,//WP_THERMAL,
    TORSO_WEAPONREADY10,//TORSO_WEAPONREADY11,//WP_TRIP_MINE,
    TORSO_WEAPONREADY10,//TORSO_WEAPONREADY12,//WP_DET_PACK,
    TORSO_WEAPONREADY3, //WP_CONCUSSION
    TORSO_WEAPONREADY2, //WP_BRYAR_OLD,

    //NOT VALID (e.g. should never really be used):
    BOTH_STAND1,        //WP_EMPLACED_GUN,
    TORSO_WEAPONREADY1, //WP_TURRET
];

// Pulled in out of source order (bg_misc.c:294): the torso attack-anim lookup that
// `bg_pmove.c`'s `PM_Weapon` indexes by `ps->weapon`. A pure data table (no `bg_strap.h`),
// so it lands ahead of the not-yet-ported bulk of bg_misc.c alongside `WeaponReadyAnim`.
//
// CARRIED BUG (faithful to original Raven JKA): the C initializer at bg_misc.c:294 has only
// 18 entries for a `WP_NUM_WEAPONS` (== 19) array — it OMITS the `WP_CONCUSSION` row that
// `WeaponReadyAnim` above has. In C the remaining initializers therefore shift up by one and
// the trailing element zero-fills, so by index the table is NOT what the (now-misaligned)
// `//WP_*` comments claim. The actual original-JKA contents are reproduced here by index:
//   [15] WP_CONCUSSION   = BOTH_ATTACK2 (the value the comment pins to WP_BRYAR_OLD)
//   [16] WP_BRYAR_OLD    = BOTH_STAND1  (the value the comment pins to WP_EMPLACED_GUN)
//   [17] WP_EMPLACED_GUN = BOTH_ATTACK1 (the value the comment pins to WP_TURRET)
//   [18] WP_TURRET       = 0            (C zero-fill — no initializer was provided)
// OpenJK recognised this and fixes it (inserts the `BOTH_ATTACK3,//WP_CONCUSSION` row,
// realigning to 19 entries) behind the `g_fixWeaponAttackAnim` cvar / `BG_FixWeaponAttackAnim`.
// We target original JKA, so we keep the bug; the comments below mark the real per-index weapon.
#[rustfmt::skip]
pub static WeaponAttackAnim: [c_int; WP_NUM_WEAPONS as usize] = [
    BOTH_ATTACK1,       //[ 0] WP_NONE //(shouldn't happen)

    BOTH_ATTACK3,       //[ 1] WP_STUN_BATON
    BOTH_ATTACK3,       //[ 2] WP_MELEE
    BOTH_STAND2,        //[ 3] WP_SABER //(has its own handling)
    BOTH_ATTACK2,       //[ 4] WP_BRYAR_PISTOL
    BOTH_ATTACK3,       //[ 5] WP_BLASTER
    BOTH_ATTACK3,       //[ 6] WP_DISRUPTOR
    BOTH_ATTACK3,       //[ 7] WP_BOWCASTER
    BOTH_ATTACK3,       //[ 8] WP_REPEATER
    BOTH_ATTACK3,       //[ 9] WP_DEMP2
    BOTH_ATTACK3,       //[10] WP_FLECHETTE
    BOTH_ATTACK3,       //[11] WP_ROCKET_LAUNCHER
    BOTH_THERMAL_THROW, //[12] WP_THERMAL
    BOTH_ATTACK3,       //[13] WP_TRIP_MINE
    BOTH_ATTACK3,       //[14] WP_DET_PACK
    BOTH_ATTACK2,       //[15] WP_CONCUSSION   <- carried bug: shifted from WP_BRYAR_OLD's slot
    BOTH_STAND1,        //[16] WP_BRYAR_OLD    <- carried bug: shifted from WP_EMPLACED_GUN's slot
    BOTH_ATTACK1,       //[17] WP_EMPLACED_GUN <- carried bug: shifted from WP_TURRET's slot
    0,                  //[18] WP_TURRET       <- carried bug: C zero-fill (no initializer)
];

// Pulled in out of source order (bg_misc.c:268): the legs ready-stance lookup that
// `bg_pmove.c`'s `PM_Footsteps` indexes by `ps->weapon` (declared there as
// `extern int WeaponReadyLegsAnim[WP_NUM_WEAPONS]`). A pure data table (no `bg_strap.h`),
// so it lands ahead of the not-yet-ported bulk of bg_misc.c alongside `WeaponReadyAnim`.
#[rustfmt::skip]
pub static WeaponReadyLegsAnim: [c_int; WP_NUM_WEAPONS as usize] = [
    BOTH_STAND1,//WP_NONE,

    BOTH_STAND1,//WP_STUN_BATON,
    BOTH_STAND1,//WP_MELEE,
    BOTH_STAND2,//WP_SABER,
    BOTH_STAND1,//WP_BRYAR_PISTOL,
    BOTH_STAND1,//WP_BLASTER,
    BOTH_STAND1,//TORSO_WEAPONREADY4,//WP_DISRUPTOR,
    BOTH_STAND1,//TORSO_WEAPONREADY5,//WP_BOWCASTER,
    BOTH_STAND1,//TORSO_WEAPONREADY6,//WP_REPEATER,
    BOTH_STAND1,//TORSO_WEAPONREADY7,//WP_DEMP2,
    BOTH_STAND1,//TORSO_WEAPONREADY8,//WP_FLECHETTE,
    BOTH_STAND1,//TORSO_WEAPONREADY9,//WP_ROCKET_LAUNCHER,
    BOTH_STAND1,//WP_THERMAL,
    BOTH_STAND1,//TORSO_WEAPONREADY11,//WP_TRIP_MINE,
    BOTH_STAND1,//TORSO_WEAPONREADY12,//WP_DET_PACK,
    BOTH_STAND1,//WP_CONCUSSION
    BOTH_STAND1,//WP_BRYAR_OLD,

    //NOT VALID (e.g. should never really be used):
    BOTH_STAND1,//WP_EMPLACED_GUN,
    BOTH_STAND1//WP_TURRET,
];

/// Resolve a nullable `'static` C-string literal to the raw `char *` a `gitem_t`
/// field holds: `Some(c"...")` → the literal's pointer, `None` → C `NULL`. The table
/// never writes through these (read-only item definitions), so the `*const`→`*mut`
/// cast is sound — it matches the C `char *` fields, every one of which is initialised
/// from a string literal or `NULL`/`0`. (The `bg_saber.rs` `smd()`/`CStr` precedent.)
const fn cs(p: Option<&'static CStr>) -> *mut c_char {
    match p {
        Some(c) => c.as_ptr() as *mut c_char,
        None => core::ptr::null_mut(),
    }
}

/// Builds one [`gitem_t`] row positionally, mirroring the C aggregate initializer
/// (classname, pickup_sound, world_model[0..4], view_model, icon, quantity, giType,
/// giTag, precaches, sounds, description). `world_model` is spread to four `wm*`
/// args (the C brace group). Nullable `char *` fields take `Option<&CStr>`.
#[allow(clippy::too_many_arguments)]
const fn gi(
    classname: Option<&'static CStr>,
    pickup_sound: Option<&'static CStr>,
    wm0: Option<&'static CStr>,
    wm1: Option<&'static CStr>,
    wm2: Option<&'static CStr>,
    wm3: Option<&'static CStr>,
    view_model: Option<&'static CStr>,
    icon: Option<&'static CStr>,
    quantity: c_int,
    giType: itemType_t,
    giTag: c_int,
    precaches: Option<&'static CStr>,
    sounds: Option<&'static CStr>,
    description: Option<&'static CStr>,
) -> gitem_t {
    gitem_t {
        classname: cs(classname),
        pickup_sound: cs(pickup_sound),
        world_model: [cs(wm0), cs(wm1), cs(wm2), cs(wm3)],
        view_model: cs(view_model),
        icon: cs(icon),
        quantity,
        giType,
        giTag,
        precaches: cs(precaches),
        sounds: cs(sounds),
        description: cs(description),
    }
}

/// `bg_itemlist[]` (bg_misc.c:795) — the master item-definition table: every pickup,
/// holdable, weapon, ammo and team flag, keyed by `(giType, giTag)`. Index 0 is the
/// reserved NULL sentinel and the final `{NULL}` is the end-of-list terminator (both
/// excluded from [`bg_numItems`]). Carries raw `char *` fields, so — like
/// `saberMoveData[]` — the array is `!Sync` and mirrors the C mutable global as
/// `static mut`; rows are built positionally through [`gi()`].
///
/// CARRIED QUIRK (faithful to original Raven JKA): the `item_shield_sm_instant` row
/// (index 1, bg_misc.c:833-834) has a MISSING COMMA between its `sounds` and
/// `description` initializers — C concatenates the two adjacent `""` literals into the
/// single `sounds` value, leaving `description` with no initializer (zero-filled to
/// `NULL`). Every other row supplies all three trailing strings as `""`. Reproduced
/// here as `description: None` for that one row.
#[rustfmt::skip]
pub static mut bg_itemlist: [gitem_t; BG_ITEMLIST_LEN] = [
    // leave index 0 alone (reserved NULL sentinel; note precaches/sounds/description are "")
    gi(None, None, None, None, None, None, None, None, 0, IT_BAD, 0, Some(c""), Some(c""), Some(c"")),

    //
    // Pickups
    //
    gi(Some(c"item_shield_sm_instant"), Some(c"sound/player/pickupshield.wav"),
        Some(c"models/map_objects/mp/psd_sm.md3"), None, None, None,
        None, Some(c"gfx/mp/small_shield"),
        25, IT_ARMOR, 1, // special for shield - max on pickup is maxhealth*tag (small → 100)
        Some(c""), Some(c""), None), // <- carried quirk: missing comma → description = NULL
    gi(Some(c"item_shield_lrg_instant"), Some(c"sound/player/pickupshield.wav"),
        Some(c"models/map_objects/mp/psd.md3"), None, None, None,
        None, Some(c"gfx/mp/large_shield"),
        100, IT_ARMOR, 2, // special for shield - max on pickup is maxhealth*tag (large → 200)
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_medpak_instant"), Some(c"sound/player/pickuphealth.wav"),
        Some(c"models/map_objects/mp/medpac.md3"), None, None, None,
        None, Some(c"gfx/hud/i_icon_medkit"),
        25, IT_HEALTH, 0,
        Some(c""), Some(c""), Some(c"")),

    //
    // ITEMS
    //
    gi(Some(c"item_seeker"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/items/remote.md3"), None, None, None,
        None, Some(c"gfx/hud/i_icon_seeker"),
        120, IT_HOLDABLE, HI_SEEKER,
        Some(c""), Some(c""), Some(c"@MENUS_AN_ATTACK_DRONE_SIMILAR")),
    gi(Some(c"item_shield"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/map_objects/mp/shield.md3"), None, None, None,
        None, Some(c"gfx/hud/i_icon_shieldwall"),
        120, IT_HOLDABLE, HI_SHIELD,
        Some(c""),
        Some(c"sound/weapons/detpack/stick.wav sound/movers/doors/forcefield_on.wav sound/movers/doors/forcefield_off.wav sound/movers/doors/forcefield_lp.wav sound/effects/bumpfield.wav"),
        Some(c"@MENUS_THIS_STATIONARY_ENERGY")),
    gi(Some(c"item_medpac"), Some(c"sound/weapons/w_pkup.wav"), // should be item_bacta
        Some(c"models/map_objects/mp/bacta.md3"), None, None, None,
        None, Some(c"gfx/hud/i_icon_bacta"),
        25, IT_HOLDABLE, HI_MEDPAC,
        Some(c""), Some(c""), Some(c"@SP_INGAME_BACTA_DESC")),
    gi(Some(c"item_medpac_big"), Some(c"sound/weapons/w_pkup.wav"), // should be item_bacta
        Some(c"models/items/big_bacta.md3"), None, None, None,
        None, Some(c"gfx/hud/i_icon_big_bacta"),
        25, IT_HOLDABLE, HI_MEDPAC_BIG,
        Some(c""), Some(c""), Some(c"@SP_INGAME_BACTA_DESC")),
    gi(Some(c"item_binoculars"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/items/binoculars.md3"), None, None, None,
        None, Some(c"gfx/hud/i_icon_zoom"),
        60, IT_HOLDABLE, HI_BINOCULARS,
        Some(c""), Some(c""), Some(c"@SP_INGAME_LA_GOGGLES_DESC")),
    gi(Some(c"item_sentry_gun"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/items/psgun.glm"), None, None, None,
        None, Some(c"gfx/hud/i_icon_sentrygun"),
        120, IT_HOLDABLE, HI_SENTRY_GUN,
        Some(c""), Some(c""), Some(c"@MENUS_THIS_DEADLY_WEAPON_IS")),
    gi(Some(c"item_jetpack"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/items/psgun.glm"), None, None, None, // FIXME: no model
        None, Some(c"gfx/hud/i_icon_jetpack"),
        120, IT_HOLDABLE, HI_JETPACK,
        Some(c"effects/boba/jet.efx"),
        Some(c"sound/chars/boba/JETON.wav sound/chars/boba/JETHOVER.wav sound/effects/fire_lp.wav"),
        Some(c"@MENUS_JETPACK_DESC")),
    gi(Some(c"item_healthdisp"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/map_objects/mp/bacta.md3"), None, None, None, // replace me
        None, Some(c"gfx/hud/i_icon_healthdisp"),
        120, IT_HOLDABLE, HI_HEALTHDISP,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_ammodisp"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/map_objects/mp/bacta.md3"), None, None, None, // replace me
        None, Some(c"gfx/hud/i_icon_ammodisp"),
        120, IT_HOLDABLE, HI_AMMODISP,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_eweb_holdable"), Some(c"sound/interface/shieldcon_empty"),
        Some(c"models/map_objects/hoth/eweb_model.glm"), None, None, None,
        None, Some(c"gfx/hud/i_icon_eweb"),
        120, IT_HOLDABLE, HI_EWEB,
        Some(c""), Some(c""), Some(c"@MENUS_EWEB_DESC")),
    gi(Some(c"item_cloak"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/items/psgun.glm"), None, None, None, // FIXME: no model
        None, Some(c"gfx/hud/i_icon_cloak"),
        120, IT_HOLDABLE, HI_CLOAK,
        Some(c""), Some(c""), Some(c"@MENUS_CLOAK_DESC")),
    gi(Some(c"item_force_enlighten_light"), Some(c"sound/player/enlightenment.wav"),
        Some(c"models/map_objects/mp/jedi_enlightenment.md3"), None, None, None,
        None, Some(c"gfx/hud/mpi_jlight"),
        25, IT_POWERUP, PW_FORCE_ENLIGHTENED_LIGHT,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_force_enlighten_dark"), Some(c"sound/player/enlightenment.wav"),
        Some(c"models/map_objects/mp/dk_enlightenment.md3"), None, None, None,
        None, Some(c"gfx/hud/mpi_dklight"),
        25, IT_POWERUP, PW_FORCE_ENLIGHTENED_DARK,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_force_boon"), Some(c"sound/player/boon.wav"),
        Some(c"models/map_objects/mp/force_boon.md3"), None, None, None,
        None, Some(c"gfx/hud/mpi_fboon"),
        25, IT_POWERUP, PW_FORCE_BOON,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_ysalimari"), Some(c"sound/player/ysalimari.wav"),
        Some(c"models/map_objects/mp/ysalimari.md3"), None, None, None,
        None, Some(c"gfx/hud/mpi_ysamari"),
        25, IT_POWERUP, PW_YSALAMIRI,
        Some(c""), Some(c""), Some(c"")),

    //
    // WEAPONS
    //
    gi(Some(c"weapon_stun_baton"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/stun_baton/baton_w.glm"), None, None, None,
        Some(c"models/weapons2/stun_baton/baton.md3"), Some(c"gfx/hud/w_icon_stunbaton"),
        100, IT_WEAPON, WP_STUN_BATON,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"weapon_melee"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/stun_baton/baton_w.glm"), None, None, None,
        Some(c"models/weapons2/stun_baton/baton.md3"), Some(c"gfx/hud/w_icon_melee"),
        100, IT_WEAPON, WP_MELEE,
        Some(c""), Some(c""), Some(c"@MENUS_MELEE_DESC")),
    gi(Some(c"weapon_saber"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/saber/saber_w.glm"), None, None, None,
        Some(c"models/weapons2/saber/saber_w.md3"), Some(c"gfx/hud/w_icon_lightsaber"),
        100, IT_WEAPON, WP_SABER,
        Some(c""), Some(c""), Some(c"@MENUS_AN_ELEGANT_WEAPON_FOR")),
    gi(Some(c"weapon_blaster_pistol"), Some(c"sound/weapons/w_pkup.wav"), // was weapon_bryar_pistol
        Some(c"models/weapons2/blaster_pistol/blaster_pistol_w.glm"), None, None, None,
        Some(c"models/weapons2/blaster_pistol/blaster_pistol.md3"), Some(c"gfx/hud/w_icon_blaster_pistol"),
        100, IT_WEAPON, WP_BRYAR_PISTOL,
        Some(c""), Some(c""), Some(c"@MENUS_BLASTER_PISTOL_DESC")),
    gi(Some(c"weapon_concussion_rifle"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/concussion/c_rifle_w.glm"), None, None, None,
        Some(c"models/weapons2/concussion/c_rifle.md3"), Some(c"gfx/hud/w_icon_c_rifle"),
        50, IT_WEAPON, WP_CONCUSSION,
        Some(c""), Some(c""), Some(c"@MENUS_CONC_RIFLE_DESC")),
    gi(Some(c"weapon_bryar_pistol"), Some(c"sound/weapons/w_pkup.wav"), // the "old" bryar
        Some(c"models/weapons2/briar_pistol/briar_pistol_w.glm"), None, None, None,
        Some(c"models/weapons2/briar_pistol/briar_pistol.md3"), Some(c"gfx/hud/w_icon_briar"),
        100, IT_WEAPON, WP_BRYAR_OLD,
        Some(c""), Some(c""), Some(c"@SP_INGAME_BLASTER_PISTOL")),
    gi(Some(c"weapon_blaster"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/blaster_r/blaster_w.glm"), None, None, None,
        Some(c"models/weapons2/blaster_r/blaster.md3"), Some(c"gfx/hud/w_icon_blaster"),
        100, IT_WEAPON, WP_BLASTER,
        Some(c""), Some(c""), Some(c"@MENUS_THE_PRIMARY_WEAPON_OF")),
    gi(Some(c"weapon_disruptor"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/disruptor/disruptor_w.glm"), None, None, None,
        Some(c"models/weapons2/disruptor/disruptor.md3"), Some(c"gfx/hud/w_icon_disruptor"),
        100, IT_WEAPON, WP_DISRUPTOR,
        Some(c""), Some(c""), Some(c"@MENUS_THIS_NEFARIOUS_WEAPON")),
    gi(Some(c"weapon_bowcaster"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/bowcaster/bowcaster_w.glm"), None, None, None,
        Some(c"models/weapons2/bowcaster/bowcaster.md3"), Some(c"gfx/hud/w_icon_bowcaster"),
        100, IT_WEAPON, WP_BOWCASTER,
        Some(c""), Some(c""), Some(c"@MENUS_THIS_ARCHAIC_LOOKING")),
    gi(Some(c"weapon_repeater"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/heavy_repeater/heavy_repeater_w.glm"), None, None, None,
        Some(c"models/weapons2/heavy_repeater/heavy_repeater.md3"), Some(c"gfx/hud/w_icon_repeater"),
        100, IT_WEAPON, WP_REPEATER,
        Some(c""), Some(c""), Some(c"@MENUS_THIS_DESTRUCTIVE_PROJECTILE")),
    gi(Some(c"weapon_demp2"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/demp2/demp2_w.glm"), None, None, None,
        Some(c"models/weapons2/demp2/demp2.md3"), Some(c"gfx/hud/w_icon_demp2"),
        100, IT_WEAPON, WP_DEMP2,
        Some(c""), Some(c""), Some(c"@MENUS_COMMONLY_REFERRED_TO")),
    gi(Some(c"weapon_flechette"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/golan_arms/golan_arms_w.glm"), None, None, None,
        Some(c"models/weapons2/golan_arms/golan_arms.md3"), Some(c"gfx/hud/w_icon_flechette"),
        100, IT_WEAPON, WP_FLECHETTE,
        Some(c""), Some(c""), Some(c"@MENUS_WIDELY_USED_BY_THE_CORPORATE")),
    gi(Some(c"weapon_rocket_launcher"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/merr_sonn/merr_sonn_w.glm"), None, None, None,
        Some(c"models/weapons2/merr_sonn/merr_sonn.md3"), Some(c"gfx/hud/w_icon_merrsonn"),
        3, IT_WEAPON, WP_ROCKET_LAUNCHER,
        Some(c""), Some(c""), Some(c"@MENUS_THE_PLX_2M_IS_AN_EXTREMELY")),
    gi(Some(c"ammo_thermal"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/thermal/thermal_pu.md3"), Some(c"models/weapons2/thermal/thermal_w.glm"), None, None,
        Some(c"models/weapons2/thermal/thermal.md3"), Some(c"gfx/hud/w_icon_thermal"),
        4, IT_AMMO, AMMO_THERMAL,
        Some(c""), Some(c""), Some(c"@MENUS_THE_THERMAL_DETONATOR")),
    gi(Some(c"ammo_tripmine"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/laser_trap/laser_trap_pu.md3"), Some(c"models/weapons2/laser_trap/laser_trap_w.glm"), None, None,
        Some(c"models/weapons2/laser_trap/laser_trap.md3"), Some(c"gfx/hud/w_icon_tripmine"),
        3, IT_AMMO, AMMO_TRIPMINE,
        Some(c""), Some(c""), Some(c"@MENUS_TRIP_MINES_CONSIST_OF")),
    gi(Some(c"ammo_detpack"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/detpack/det_pack_pu.md3"), Some(c"models/weapons2/detpack/det_pack_proj.glm"), Some(c"models/weapons2/detpack/det_pack_w.glm"), None,
        Some(c"models/weapons2/detpack/det_pack.md3"), Some(c"gfx/hud/w_icon_detpack"),
        3, IT_AMMO, AMMO_DETPACK,
        Some(c""), Some(c""), Some(c"@MENUS_A_DETONATION_PACK_IS")),
    gi(Some(c"weapon_thermal"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/thermal/thermal_w.glm"), Some(c"models/weapons2/thermal/thermal_pu.md3"), None, None,
        Some(c"models/weapons2/thermal/thermal.md3"), Some(c"gfx/hud/w_icon_thermal"),
        4, IT_WEAPON, WP_THERMAL,
        Some(c""), Some(c""), Some(c"@MENUS_THE_THERMAL_DETONATOR")),
    gi(Some(c"weapon_trip_mine"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/laser_trap/laser_trap_w.glm"), Some(c"models/weapons2/laser_trap/laser_trap_pu.md3"), None, None,
        Some(c"models/weapons2/laser_trap/laser_trap.md3"), Some(c"gfx/hud/w_icon_tripmine"),
        3, IT_WEAPON, WP_TRIP_MINE,
        Some(c""), Some(c""), Some(c"@MENUS_TRIP_MINES_CONSIST_OF")),
    gi(Some(c"weapon_det_pack"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/detpack/det_pack_proj.glm"), Some(c"models/weapons2/detpack/det_pack_pu.md3"), Some(c"models/weapons2/detpack/det_pack_w.glm"), None,
        Some(c"models/weapons2/detpack/det_pack.md3"), Some(c"gfx/hud/w_icon_detpack"),
        3, IT_WEAPON, WP_DET_PACK,
        Some(c""), Some(c""), Some(c"@MENUS_A_DETONATION_PACK_IS")),
    gi(Some(c"weapon_emplaced"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/blaster_r/blaster_w.glm"), None, None, None,
        Some(c"models/weapons2/blaster_r/blaster.md3"), Some(c"gfx/hud/w_icon_blaster"),
        50, IT_WEAPON, WP_EMPLACED_GUN,
        Some(c""), Some(c""), Some(c"")),
    // NOTE: keeps things from messing up because the turret weapon type isn't real
    gi(Some(c"weapon_turretwp"), Some(c"sound/weapons/w_pkup.wav"),
        Some(c"models/weapons2/blaster_r/blaster_w.glm"), None, None, None,
        Some(c"models/weapons2/blaster_r/blaster.md3"), Some(c"gfx/hud/w_icon_blaster"),
        50, IT_WEAPON, WP_TURRET,
        Some(c""), Some(c""), Some(c"")),

    //
    // AMMO ITEMS
    //
    gi(Some(c"ammo_force"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/items/energy_cell.md3"), None, None, None,
        None, Some(c"gfx/hud/w_icon_blaster"),
        100, IT_AMMO, AMMO_FORCE,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"ammo_blaster"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/items/energy_cell.md3"), None, None, None,
        None, Some(c"gfx/hud/i_icon_battery"),
        100, IT_AMMO, AMMO_BLASTER,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"ammo_powercell"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/items/power_cell.md3"), None, None, None,
        None, Some(c"gfx/mp/ammo_power_cell"),
        100, IT_AMMO, AMMO_POWERCELL,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"ammo_metallic_bolts"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/items/metallic_bolts.md3"), None, None, None,
        None, Some(c"gfx/mp/ammo_metallic_bolts"),
        100, IT_AMMO, AMMO_METAL_BOLTS,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"ammo_rockets"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/items/rockets.md3"), None, None, None,
        None, Some(c"gfx/mp/ammo_rockets"),
        3, IT_AMMO, AMMO_ROCKETS,
        Some(c""), Some(c""), Some(c"")),
    // ammo_all: DO NOT PLACE — siege classes with ammo-dispensing ability only
    gi(Some(c"ammo_all"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/items/battery.md3"), None, None, None, // replace me
        None, Some(c"gfx/mp/ammo_rockets"), // replace me
        0, IT_AMMO, -1,
        Some(c""), Some(c""), Some(c"")),

    //
    // POWERUP ITEMS
    //
    gi(Some(c"team_CTF_redflag"), None,
        Some(c"models/flags/r_flag.md3"), Some(c"models/flags/r_flag_ysal.md3"), None, None,
        None, Some(c"gfx/hud/mpi_rflag"),
        0, IT_TEAM, PW_REDFLAG,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"team_CTF_blueflag"), None,
        Some(c"models/flags/b_flag.md3"), Some(c"models/flags/b_flag_ysal.md3"), None, None,
        None, Some(c"gfx/hud/mpi_bflag"),
        0, IT_TEAM, PW_BLUEFLAG,
        Some(c""), Some(c""), Some(c"")),

    //
    // PERSISTANT POWERUP ITEMS
    //
    gi(Some(c"team_CTF_neutralflag"), None,
        Some(c"models/flags/n_flag.md3"), None, None, None,
        None, Some(c"icons/iconf_neutral1"),
        0, IT_TEAM, PW_NEUTRALFLAG,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_redcube"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/powerups/orb/r_orb.md3"), None, None, None,
        None, Some(c"icons/iconh_rorb"),
        0, IT_TEAM, 0,
        Some(c""), Some(c""), Some(c"")),
    gi(Some(c"item_bluecube"), Some(c"sound/player/pickupenergy.wav"),
        Some(c"models/powerups/orb/b_orb.md3"), None, None, None,
        None, Some(c"icons/iconh_borb"),
        0, IT_TEAM, 0,
        Some(c""), Some(c""), Some(c"")),

    // end of list marker — C `{NULL}` zero-inits every field (all strings NULL, not "")
    gi(None, None, None, None, None, None, None, None, 0, IT_BAD, 0, None, None, None),
];

/// Length of [`bg_itemlist`] including the index-0 sentinel and the `{NULL}`
/// terminator (50 real items + 2). The array type enforces the count.
const BG_ITEMLIST_LEN: usize = 52;

/// `bg_numItems` (bg_misc.c:1771) — count of real entries in [`bg_itemlist`]:
/// `sizeof(bg_itemlist)/sizeof(bg_itemlist[0]) - 1` (the `-1` drops the `{NULL}`
/// terminator; the index-0 sentinel is still counted, exactly as in C).
pub static bg_numItems: c_int = (BG_ITEMLIST_LEN - 1) as c_int;

// Pulled in out of source order (bg_misc.c:1773): the direction→yaw helper the
// `bg_pmove.c` wall-run/wall-jump angle adjusters (`PM_AdjustAngleForWallRun` &c.)
// index `trace.plane.normal` through. Pure float math (no `bg_strap.h`), so it lands
// ahead of the not-yet-ported bulk of bg_misc.c. `atan2`/`*180/M_PI` are computed in double
// then narrowed to `vec_t`, matching the `(float)` assignment in the C (the
// `vectoangles` precedent in q_math.rs).
pub fn vectoyaw(vec: &vec3_t) -> vec_t {
    let mut yaw: vec_t;

    if vec[YAW] == 0.0 && vec[PITCH] == 0.0 {
        yaw = 0.0;
    } else {
        if vec[PITCH] != 0.0 {
            yaw = ((vec[YAW] as f64).atan2(vec[PITCH] as f64) * 180.0 / (M_PI as f64)) as vec_t;
        } else if vec[YAW] > 0.0 {
            yaw = 90.0;
        } else {
            yaw = 270.0;
        }
        if yaw < 0.0 {
            yaw += 360.0;
        }
    }

    yaw
}

/// `BG_FindItemForPowerup` (bg_misc.c:1881) — first [`bg_itemlist`] entry that is a
/// powerup or team item carrying the given `giTag`; `NULL` if none. Returns a raw
/// pointer into the `static mut` table (the C `gitem_t *`).
pub unsafe fn BG_FindItemForPowerup(pw: powerup_t) -> *mut gitem_t {
    let list = addr_of_mut!(bg_itemlist) as *mut gitem_t;
    let mut i = 0;
    while i < bg_numItems {
        let it = list.add(i as usize);
        if ((*it).giType == IT_POWERUP || (*it).giType == IT_TEAM) && (*it).giTag == pw {
            return it;
        }
        i += 1;
    }
    core::ptr::null_mut()
}

/// `BG_FindItemForHoldable` (bg_misc.c:1901) — the [`bg_itemlist`] holdable entry
/// carrying the given `giTag`. `Com_Error(ERR_DROP)` if absent (the C body then
/// `return NULL`s — unreachable here, `Com_Error` diverges).
pub unsafe fn BG_FindItemForHoldable(pw: holdable_t) -> *mut gitem_t {
    let list = addr_of_mut!(bg_itemlist) as *mut gitem_t;
    let mut i = 0;
    while i < bg_numItems {
        let it = list.add(i as usize);
        if (*it).giType == IT_HOLDABLE && (*it).giTag == pw {
            return it;
        }
        i += 1;
    }
    Com_Error(ERR_DROP, "HoldableItem not found");
}

/// `BG_FindItemForWeapon` (bg_misc.c:1922) — the [`bg_itemlist`] weapon entry for the
/// given `weapon`. Walks `bg_itemlist + 1` until the `{NULL}` terminator (classname
/// NULL); `Com_Error(ERR_DROP)` if absent (the C `return NULL` is unreachable).
pub unsafe fn BG_FindItemForWeapon(weapon: weapon_t) -> *mut gitem_t {
    let mut it = (addr_of_mut!(bg_itemlist) as *mut gitem_t).add(1);
    while !(*it).classname.is_null() {
        if (*it).giType == IT_WEAPON && (*it).giTag == weapon {
            return it;
        }
        it = it.add(1);
    }
    Com_Error(
        ERR_DROP,
        &format!("Couldn't find item for weapon {}", weapon),
    );
}

/// `BG_FindItemForAmmo` (bg_misc.c:1941) — the [`bg_itemlist`] ammo entry for the given
/// `ammo`. Same `bg_itemlist + 1` walk; `Com_Error(ERR_DROP)` if absent.
pub unsafe fn BG_FindItemForAmmo(ammo: ammo_t) -> *mut gitem_t {
    let mut it = (addr_of_mut!(bg_itemlist) as *mut gitem_t).add(1);
    while !(*it).classname.is_null() {
        if (*it).giType == IT_AMMO && (*it).giTag == ammo {
            return it;
        }
        it = it.add(1);
    }
    Com_Error(ERR_DROP, &format!("Couldn't find item for ammo {}", ammo));
}

/// `BG_FindItem` (bg_misc.c:1960) — the [`bg_itemlist`] entry whose `classname`
/// matches (case-insensitive); `NULL` if none. Walks `bg_itemlist + 1` to the
/// terminator.
pub unsafe fn BG_FindItem(classname: *const c_char) -> *mut gitem_t {
    let mut it = (addr_of_mut!(bg_itemlist) as *mut gitem_t).add(1);
    while !(*it).classname.is_null() {
        if Q_stricmp((*it).classname, classname) == 0 {
            return it;
        }
        it = it.add(1);
    }
    core::ptr::null_mut()
}

/// `bgToggleableSurfaces` (bg_misc.c:34) — the parallel name table for the
/// breakable/toggleable G2 surfaces (`BG_NUM_TOGGLEABLE_SURFACES` entries: 30 surface
/// names at indices 0..29 then a `NULL` terminator at 30). The index is the
/// toggleable-surface id; consumers (`NPC_utils.c`, cgame) `Q_stricmp` against the
/// names and walk to the `NULL`. The galak head/arm block (old indices 15..23) is
/// commented out upstream — the slots were reclaimed for vehicle surfs — so r_wing1
/// resumes at index 15; the gap is carried as comments to keep the index annotations.
///
/// `static mut` (not a plain `static`) because the array holds raw `*const c_char`,
/// which is `!Sync`; it is never mutated, mirroring the `animTable`/`SaberTable`
/// raw-pointer-table precedent. The trailing `NULL` is `core::ptr::null()`.
pub static mut bgToggleableSurfaces: [*const c_char; BG_NUM_TOGGLEABLE_SURFACES] = [
    c"l_arm_key".as_ptr(), // 0
    c"torso_canister1".as_ptr(),
    c"torso_canister2".as_ptr(),
    c"torso_canister3".as_ptr(),
    c"torso_tube1".as_ptr(),
    c"torso_tube2".as_ptr(), // 5
    c"torso_tube3".as_ptr(),
    c"torso_tube4".as_ptr(),
    c"torso_tube5".as_ptr(),
    c"torso_tube6".as_ptr(),
    c"r_arm".as_ptr(), // 10
    c"l_arm".as_ptr(),
    c"torso_shield".as_ptr(),
    c"torso_galaktorso".as_ptr(),
    c"torso_collar".as_ptr(),
    // "torso_eyes_mouth",            // 15
    // "torso_galakhead",
    // "torso_galakface",
    // "torso_antenna_base_cap",
    // "torso_antenna",
    // "l_arm_augment",               // 20
    // "l_arm_middle",
    // "l_arm_wrist",
    // "r_arm_middle", // yeah.. galak's surf stuff is no longer auto, sorry! need the space for vehicle surfs.
    c"r_wing1".as_ptr(), // 15
    c"r_wing2".as_ptr(),
    c"l_wing1".as_ptr(),
    c"l_wing2".as_ptr(),
    c"r_gear".as_ptr(),
    c"l_gear".as_ptr(), // 20
    c"nose".as_ptr(),
    c"blah4".as_ptr(),
    c"blah5".as_ptr(),
    c"l_hand".as_ptr(),
    c"r_hand".as_ptr(), // 25
    c"helmet".as_ptr(),
    c"head".as_ptr(),
    c"head_concussion_charger".as_ptr(),
    c"head_light_blaster_cann".as_ptr(), // 29
    core::ptr::null(),
];

/// `bgToggleableSurfaceDebris` (bg_misc.c:78) — parallel to [`bgToggleableSurfaces`],
/// indexed by toggleable-surface id: the debris/effect type spawned when that surface
/// is destroyed (`>= 2` means a flame trail, for vehicles). The final `-1` is the
/// terminator. Read-only in C → plain `static`.
pub static bgToggleableSurfaceDebris: [c_int; BG_NUM_TOGGLEABLE_SURFACES] = [
    0, // 0
    0, 0, 0, 0, // 1..4
    0, // 5
    0, 0, 0, 0, // 6..9
    0, // 10
    0, 0, 0, // 11..13
    0, // 14 -- >= 2 means it should create a flame trail when destroyed (for vehicles)
    3, // 15
    5, // rwing2
    4, // 17
    6, // lwing2
    0, // rgear
    0, // lgear  // 20
    7, // nose
    0, // blah
    0, // blah
    0, // 24
    0, // 25
    0, 0, 0,  // 26..28
    0,  // 29
    -1, // terminator
];

/// `bg_customSiegeSoundNames` (bg_misc.c:113) — the `MAX_CUSTOM_SIEGE_SOUNDS` siege
/// voice-command sound aliases (`*att_attack`, `*reply_yes`, …; 29 names + `NULL`
/// terminator). The server registers / matches these in `g_main.c`/`g_cmds.c`. The `*`
/// prefix is a custom-sound path marker; rww's note: no `@` prefix because these are
/// not cgame `StringEd` lookups. Same `static mut [*const c_char]` raw-pointer-table
/// representation as [`bgToggleableSurfaces`].
pub static mut bg_customSiegeSoundNames: [*const c_char; MAX_CUSTOM_SIEGE_SOUNDS] = [
    c"*att_attack".as_ptr(),
    c"*att_primary".as_ptr(),
    c"*att_second".as_ptr(),
    c"*def_guns".as_ptr(),
    c"*def_position".as_ptr(),
    c"*def_primary".as_ptr(),
    c"*def_second".as_ptr(),
    c"*reply_coming".as_ptr(),
    c"*reply_go".as_ptr(),
    c"*reply_no".as_ptr(),
    c"*reply_stay".as_ptr(),
    c"*reply_yes".as_ptr(),
    c"*req_assist".as_ptr(),
    c"*req_demo".as_ptr(),
    c"*req_hvy".as_ptr(),
    c"*req_medic".as_ptr(),
    c"*req_sup".as_ptr(),
    c"*req_tech".as_ptr(),
    c"*spot_air".as_ptr(),
    c"*spot_defenses".as_ptr(),
    c"*spot_emplaced".as_ptr(),
    c"*spot_sniper".as_ptr(),
    c"*spot_troops".as_ptr(),
    c"*tac_cover".as_ptr(),
    c"*tac_fallback".as_ptr(),
    c"*tac_follow".as_ptr(),
    c"*tac_hold".as_ptr(),
    c"*tac_split".as_ptr(),
    c"*tac_together".as_ptr(),
    core::ptr::null(),
];

/// `forceMasteryLevels` (bg_misc.c:150) — the `StringEd` lookup keys for each
/// `FORCE_MASTERY_*` rank name (`"MASTERY0"`..`"MASTERY7"`; the C comments preserve the
/// human-readable names — note the upstream Adept/Guardian swap relative to the
/// `FORCE_MASTERY_*` enum order, carried verbatim). Only `ui`/`cgame` read it
/// (no server caller), but `bg_misc.c` compiles it into the QAGAME module so it is
/// ported for parity. Non-`const` `char *[]` in C → same `static mut [*const c_char]`
/// representation; no `NULL` terminator (exactly `NUM_FORCE_MASTERY_LEVELS` entries).
pub static mut forceMasteryLevels: [*const c_char; NUM_FORCE_MASTERY_LEVELS as usize] = [
    c"MASTERY0".as_ptr(), // "Uninitiated"   // FORCE_MASTERY_UNINITIATED,
    c"MASTERY1".as_ptr(), // "Initiate"      // FORCE_MASTERY_INITIATE,
    c"MASTERY2".as_ptr(), // "Padawan"       // FORCE_MASTERY_PADAWAN,
    c"MASTERY3".as_ptr(), // "Jedi"          // FORCE_MASTERY_JEDI,
    c"MASTERY4".as_ptr(), // "Jedi Adept"    // FORCE_MASTERY_JEDI_GUARDIAN,
    c"MASTERY5".as_ptr(), // "Jedi Guardian" // FORCE_MASTERY_JEDI_ADEPT,
    c"MASTERY6".as_ptr(), // "Jedi Knight"   // FORCE_MASTERY_JEDI_KNIGHT,
    c"MASTERY7".as_ptr(), // "Jedi Master"   // FORCE_MASTERY_JEDI_MASTER,
];

/// `forceMasteryPoints` (bg_misc.c:162) — force-point budget granted at each
/// force-mastery rank, indexed `0..NUM_FORCE_MASTERY_LEVELS` by the `FORCE_MASTERY_*`
/// rank. Read-only in C → plain `static`. (`extern` in bg_public.h; nothing else
/// references it yet, so kept `pub` here without a separate declaration.)
pub static forceMasteryPoints: [c_int; NUM_FORCE_MASTERY_LEVELS as usize] = [
    0,   // FORCE_MASTERY_UNINITIATED,
    5,   // FORCE_MASTERY_INITIATE,
    10,  // FORCE_MASTERY_PADAWAN,
    20,  // FORCE_MASTERY_JEDI,
    30,  // FORCE_MASTERY_JEDI_GUARDIAN,
    50,  // FORCE_MASTERY_JEDI_ADEPT,
    75,  // FORCE_MASTERY_JEDI_KNIGHT,
    100, // FORCE_MASTERY_JEDI_MASTER,
];

/// `bgForcePowerCost` (bg_misc.c:174) — cumulative point cost to hold force power
/// `[FP_*]` at level `[0..NUM_FORCE_POWER_LEVELS)`; `0 == neutral` (level 0, not owned).
/// Read-only in C → plain `static`.
pub static bgForcePowerCost: [[c_int; NUM_FORCE_POWER_LEVELS]; NUM_FORCE_POWERS] = [
    [0, 2, 4, 6], // Heal          // FP_HEAL
    [0, 0, 2, 6], // Jump          // FP_LEVITATION, hold/duration
    [0, 2, 4, 6], // Speed         // FP_SPEED, duration
    [0, 1, 3, 6], // Push          // FP_PUSH, hold/duration
    [0, 1, 3, 6], // Pull          // FP_PULL, hold/duration
    [0, 4, 6, 8], // Mind Trick    // FP_TELEPATHY, instant
    [0, 1, 3, 6], // Grip          // FP_GRIP, hold/duration
    [0, 2, 5, 8], // Lightning     // FP_LIGHTNING, hold/duration
    [0, 4, 6, 8], // Dark Rage     // FP_RAGE, duration
    [0, 2, 5, 8], // Protection    // FP_PROTECT, duration
    [0, 1, 3, 6], // Absorb        // FP_ABSORB, duration
    [0, 1, 3, 6], // Team Heal     // FP_TEAM_HEAL, instant
    [0, 1, 3, 6], // Team Force    // FP_TEAM_FORCE, instant
    [0, 2, 4, 6], // Drain         // FP_DRAIN, hold/duration
    [0, 2, 5, 8], // Sight         // FP_SEE, duration
    [0, 1, 5, 8], // Saber Attack  // FP_SABER_OFFENSE,
    [0, 1, 5, 8], // Saber Defend  // FP_SABER_DEFENSE,
    [0, 4, 6, 8], // Saber Throw   // FP_SABERTHROW,
];

/// `forcePowerSorted` (bg_misc.c:197) — rww's canonical draw/cycle order for the
/// `FP_*` powers. Indexed 0..[`NUM_FORCE_POWERS`]; each entry is the `FP_*` value at that
/// "sorted" slot. Immutable in C (never written), so a plain `static`.
pub static forcePowerSorted: [c_int; NUM_FORCE_POWERS] = [
    //rww - always use this order when drawing force powers for any reason
    FP_TELEPATHY,
    FP_HEAL,
    FP_ABSORB,
    FP_PROTECT,
    FP_TEAM_HEAL,
    FP_LEVITATION,
    FP_SPEED,
    FP_PUSH,
    FP_PULL,
    FP_SEE,
    FP_LIGHTNING,
    FP_DRAIN,
    FP_RAGE,
    FP_GRIP,
    FP_TEAM_FORCE,
    FP_SABER_OFFENSE,
    FP_SABER_DEFENSE,
    FP_SABERTHROW,
];

/// `forcePowerDarkLight` (bg_misc.c:219) — the alignment each force power belongs to:
/// [`FORCE_LIGHTSIDE`], [`FORCE_DARKSIDE`], or `0` (neutral / usable by either side).
/// Indexed by `FP_*`. Read-only in C → plain `static`.
pub static forcePowerDarkLight: [c_int; NUM_FORCE_POWERS] = [
    //nothing should be usable at rank 0..
    FORCE_LIGHTSIDE, // FP_HEAL, instant
    0,               // FP_LEVITATION, hold/duration
    0,               // FP_SPEED, duration
    0,               // FP_PUSH, hold/duration
    0,               // FP_PULL, hold/duration
    FORCE_LIGHTSIDE, // FP_TELEPATHY, instant
    FORCE_DARKSIDE,  // FP_GRIP, hold/duration
    FORCE_DARKSIDE,  // FP_LIGHTNING, hold/duration
    FORCE_DARKSIDE,  // FP_RAGE, duration
    FORCE_LIGHTSIDE, // FP_PROTECT, duration
    FORCE_LIGHTSIDE, // FP_ABSORB, duration
    FORCE_LIGHTSIDE, // FP_TEAM_HEAL, instant
    FORCE_DARKSIDE,  // FP_TEAM_FORCE, instant
    FORCE_DARKSIDE,  // FP_DRAIN, hold/duration
    0,               // FP_SEE, duration
    0,               // FP_SABER_OFFENSE,
    0,               // FP_SABER_DEFENSE,
    0,               // FP_SABERTHROW,
];

/// `BG_LegalizedForcePowers` (bg_misc.c:439) — "the magical function to end all
/// functions": parse the force-config string in `powerOut` (`"rank-side-PPPPPPPPPPPPPPPPPP"`,
/// 18 power-level digits), then **legalize** it against `maxRank`'s point budget and the
/// supplied rules, writing the legalized config back into `powerOut`. Returns whether the
/// input was already legal (`qtrue`) or had to be altered (`qfalse`).
///
/// `fpDisabled` is a bitmask of server-disabled `FP_*` powers (only meaningful from the
/// server). `teamForce` (non-zero) forces the side; `gametype < GT_TEAM` strips the team
/// powers; `freeSaber` grants free saber attack/defense at level 1.
///
/// **Faithful note:** C leaves the tail of `final_Powers[]` uninitialized when the input
/// supplies fewer than `NUM_FORCE_POWERS` power digits, then reads all 18. Well-formed
/// configs always carry exactly 18 digits (so every slot is written); the Rust port
/// zero-initializes the array (no UB), which is observably identical for any 18-digit input.
#[allow(unused_assignments)] // faithful redundant inits: `usedPoints`/`countDown`/`allowedPoints`/`final_Side`
pub unsafe fn BG_LegalizedForcePowers(
    powerOut: *mut c_char,
    maxRank: c_int,
    freeSaber: qboolean,
    teamForce: c_int,
    gametype: c_int,
    fpDisabled: c_int,
) -> qboolean {
    let mut powerBuf: [c_char; 128] = [0; 128];
    let mut readBuf: [c_char; 128] = [0; 128];
    let mut maintainsValidity: qboolean = QTRUE;
    let powerLen: c_int = strlen(powerOut) as c_int;
    let mut i: c_int = 0;
    let mut c: c_int = 0;
    let mut allowedPoints: c_int = 0;
    let mut usedPoints: c_int = 0;
    let mut countDown: c_int = 0;

    let mut final_Side: c_int;
    let mut final_Powers: [c_int; NUM_FORCE_POWERS] = [0; NUM_FORCE_POWERS];

    if powerLen >= 128 {
        //This should not happen. If it does, this is obviously a bogus string.
        //They can have this string. Because I said so.
        strcpy(powerBuf.as_mut_ptr(), c"7-1-032330000000001333".as_ptr());
        maintainsValidity = QFALSE;
    } else {
        strcpy(powerBuf.as_mut_ptr(), powerOut); //copy it as the original
    }

    //first of all, print the max rank into the string as the rank
    strcpy(powerOut, va(format_args!("{}-", maxRank)) as *const c_char);

    while i < 128 && powerBuf[i as usize] != 0 && powerBuf[i as usize] != b'-' as c_char {
        i += 1;
    }
    i += 1;
    while i < 128 && powerBuf[i as usize] != 0 && powerBuf[i as usize] != b'-' as c_char {
        readBuf[c as usize] = powerBuf[i as usize];
        c += 1;
        i += 1;
    }
    readBuf[c as usize] = 0;
    i += 1;
    //at this point, readBuf contains the intended side
    final_Side = atoi(readBuf.as_ptr());

    if final_Side != FORCE_LIGHTSIDE && final_Side != FORCE_DARKSIDE {
        //Not a valid side. You will be dark. Because I said so. (this is something that should never actually happen unless you purposely feed in an invalid config)
        final_Side = FORCE_DARKSIDE;
        maintainsValidity = QFALSE;
    }

    if teamForce != 0 {
        //If we are under force-aligned teams, make sure we're on the right side.
        if final_Side != teamForce {
            final_Side = teamForce;
            //maintainsValidity = qfalse;
            //Not doing this, for now. Let them join the team with their filtered powers.
        }
    }

    //Now we have established a valid rank, and a valid side.
    //Read the force powers in, and cut them down based on the various rules supplied.
    c = 0;
    while i < 128
        && powerBuf[i as usize] != 0
        && powerBuf[i as usize] != b'\n' as c_char
        && c < NUM_FORCE_POWERS as c_int
    {
        readBuf[0] = powerBuf[i as usize];
        readBuf[1] = 0;
        final_Powers[c as usize] = atoi(readBuf.as_ptr());
        c += 1;
        i += 1;
    }

    //final_Powers now contains all the stuff from the string
    //Set the maximum allowed points used based on the max rank level, and count the points actually used.
    allowedPoints = forceMasteryPoints[maxRank as usize];

    i = 0;
    while i < NUM_FORCE_POWERS as c_int {
        //if this power doesn't match the side we're on, then 0 it now.
        if final_Powers[i as usize] != 0
            && forcePowerDarkLight[i as usize] != 0
            && forcePowerDarkLight[i as usize] != final_Side
        {
            final_Powers[i as usize] = 0;
            //This is only likely to happen with g_forceBasedTeams. Let it slide.
        }

        if final_Powers[i as usize] != 0 && (fpDisabled & (1 << i)) != 0 {
            //if this power is disabled on the server via said server option, then we don't get it.
            final_Powers[i as usize] = 0;
        }

        i += 1;
    }

    if gametype < GT_TEAM {
        //don't bother with team powers then
        final_Powers[FP_TEAM_HEAL as usize] = 0;
        final_Powers[FP_TEAM_FORCE as usize] = 0;
    }

    usedPoints = 0;
    i = 0;
    while i < NUM_FORCE_POWERS as c_int {
        countDown = 0;

        countDown = final_Powers[i as usize];

        while countDown > 0 {
            usedPoints += bgForcePowerCost[i as usize][countDown as usize]; //[fp index][fp level]
                                                                            //if this is jump, or we have a free saber and it's offense or defense, take the level back down on level 1
            if countDown == 1
                && (i == FP_LEVITATION
                    || (i == FP_SABER_OFFENSE && freeSaber != 0)
                    || (i == FP_SABER_DEFENSE && freeSaber != 0))
            {
                usedPoints -= bgForcePowerCost[i as usize][countDown as usize];
            }
            countDown -= 1;
        }

        i += 1;
    }

    if usedPoints > allowedPoints {
        //Time to do the fancy stuff. (meaning, slowly cut parts off while taking a guess at what is most or least important in the config)
        let mut attemptedCycles: c_int = 0;
        let mut powerCycle: c_int = 2;
        let mut minPow: c_int = 0;

        if freeSaber != 0 {
            minPow = 1;
        }

        maintainsValidity = QFALSE;

        while usedPoints > allowedPoints {
            c = 0;

            while c < NUM_FORCE_POWERS as c_int && usedPoints > allowedPoints {
                if final_Powers[c as usize] != 0 && final_Powers[c as usize] < powerCycle {
                    //kill in order of lowest powers, because the higher powers are probably more important
                    if c == FP_SABER_OFFENSE
                        && (final_Powers[FP_SABER_DEFENSE as usize] > minPow
                            || final_Powers[FP_SABERTHROW as usize] > 0)
                    {
                        //if we're on saber attack, only suck it down if we have no def or throw either
                        let mut whichOne: c_int = FP_SABERTHROW; //first try throw

                        if final_Powers[whichOne as usize] == 0 {
                            whichOne = FP_SABER_DEFENSE; //if no throw, drain defense
                        }

                        while final_Powers[whichOne as usize] > 0 && usedPoints > allowedPoints {
                            if final_Powers[whichOne as usize] > 1
                                || ((whichOne != FP_SABER_OFFENSE || freeSaber == 0)
                                    && (whichOne != FP_SABER_DEFENSE || freeSaber == 0))
                            {
                                //don't take attack or defend down on level 1 still, if it's free
                                usedPoints -= bgForcePowerCost[whichOne as usize]
                                    [final_Powers[whichOne as usize] as usize];
                                final_Powers[whichOne as usize] -= 1;
                            } else {
                                break;
                            }
                        }
                    } else {
                        while final_Powers[c as usize] > 0 && usedPoints > allowedPoints {
                            if final_Powers[c as usize] > 1
                                || ((c != FP_LEVITATION)
                                    && (c != FP_SABER_OFFENSE || freeSaber == 0)
                                    && (c != FP_SABER_DEFENSE || freeSaber == 0))
                            {
                                usedPoints -=
                                    bgForcePowerCost[c as usize][final_Powers[c as usize] as usize];
                                final_Powers[c as usize] -= 1;
                            } else {
                                break;
                            }
                        }
                    }
                }

                c += 1;
            }

            powerCycle += 1;
            attemptedCycles += 1;

            if attemptedCycles > NUM_FORCE_POWERS as c_int {
                //I think this should be impossible. But just in case.
                break;
            }
        }

        if usedPoints > allowedPoints {
            //Still? Fine then.. we will kill all of your powers, except the freebies.
            i = 0;

            while i < NUM_FORCE_POWERS as c_int {
                final_Powers[i as usize] = 0;
                if i == FP_LEVITATION
                    || (i == FP_SABER_OFFENSE && freeSaber != 0)
                    || (i == FP_SABER_DEFENSE && freeSaber != 0)
                {
                    final_Powers[i as usize] = 1;
                }
                i += 1;
            }
            usedPoints = 0;
        }
    }

    if freeSaber != 0 {
        if final_Powers[FP_SABER_OFFENSE as usize] < 1 {
            final_Powers[FP_SABER_OFFENSE as usize] = 1;
        }
        if final_Powers[FP_SABER_DEFENSE as usize] < 1 {
            final_Powers[FP_SABER_DEFENSE as usize] = 1;
        }
    }
    if final_Powers[FP_LEVITATION as usize] < 1 {
        final_Powers[FP_LEVITATION as usize] = 1;
    }

    i = 0;
    while i < NUM_FORCE_POWERS as c_int {
        if final_Powers[i as usize] > FORCE_LEVEL_3 {
            final_Powers[i as usize] = FORCE_LEVEL_3;
        }
        i += 1;
    }

    if fpDisabled != 0 {
        //If we specifically have attack or def disabled, force them up to level 3. It's the way
        //things work for the case of all powers disabled.
        //If jump is disabled, down-cap it to level 1. Otherwise don't do a thing.
        if (fpDisabled & (1 << FP_LEVITATION)) != 0 {
            final_Powers[FP_LEVITATION as usize] = 1;
        }
        if (fpDisabled & (1 << FP_SABER_OFFENSE)) != 0 {
            final_Powers[FP_SABER_OFFENSE as usize] = 3;
        }
        if (fpDisabled & (1 << FP_SABER_DEFENSE)) != 0 {
            final_Powers[FP_SABER_DEFENSE as usize] = 3;
        }
    }

    if final_Powers[FP_SABER_OFFENSE as usize] < 1 {
        final_Powers[FP_SABER_DEFENSE as usize] = 0;
        final_Powers[FP_SABERTHROW as usize] = 0;
    }

    //We finally have all the force powers legalized and stored locally.
    //Put them all into the string and return the result. We already have
    //the rank there, so print the side and the powers now.
    Q_strcat(
        powerOut,
        128,
        va(format_args!("{}-", final_Side)) as *const c_char,
    );

    i = strlen(powerOut) as c_int;
    c = 0;
    while c < NUM_FORCE_POWERS as c_int {
        strcpy(
            readBuf.as_mut_ptr(),
            va(format_args!("{}", final_Powers[c as usize])) as *const c_char,
        );
        *powerOut.add(i as usize) = readBuf[0];
        c += 1;
        i += 1;
    }
    *powerOut.add(i as usize) = 0;

    maintainsValidity
}

/// `BG_ProperForceIndex` (bg_misc.c:1997) — the "sorted" slot in [`forcePowerSorted`]
/// holding `power` (an `FP_*`), or `-1` if absent. Inverse of indexing the table.
pub fn BG_ProperForceIndex(power: c_int) -> c_int {
    let mut i = 0;

    while i < NUM_FORCE_POWERS as c_int {
        if forcePowerSorted[i as usize] == power {
            return i;
        }

        i += 1;
    }

    -1
}

/// `BG_CycleForce` (bg_misc.c:2014) — advance `ps->fd.forcePowerSelected` to the next/prev
/// (`direction == 1` next, else previous) *known* selectable power in [`forcePowerSorted`]
/// order, wrapping the table; leaves the selection unchanged if none qualifies. The four
/// always-on/passive powers (levitation, saber offense/defense, throw) are skipped.
///
/// **Carried bug (faithful):** the early-out tests `!ps->fd.forcePowersKnown & (1 << x)`.
/// In C `!` binds tighter than `&`, so this is `(!forcePowersKnown) & (1 << x)` — a logical
/// NOT (0/1) bit-anded with the mask, *not* the intended `!(forcePowersKnown & (1 << x))`.
/// Reproduced exactly: `((forcePowersKnown == 0) as c_int) & (1 << x)`. Shifts use
/// `wrapping_shl` to match C's wrapping shift (and avoid debug-overflow panics).
// C declares `int presel = i;` then overwrites it (`presel = x;`) after the early-out,
// before any read — the initializer is faithfully transcribed but provably dead.
#[allow(unused_assignments)]
pub unsafe fn BG_CycleForce(ps: *mut playerState_t, direction: c_int) {
    let mut i = (*ps).fd.forcePowerSelected;
    let mut x = i;
    let mut presel = i;
    let mut foundnext = -1;

    // faithful `!`-precedence bug — see the "Carried bug" note in the doc comment above.
    if ((((*ps).fd.forcePowersKnown == 0) as c_int) & 1i32.wrapping_shl(x as u32)) != 0
        || x >= NUM_FORCE_POWERS as c_int
        || x == -1
    {
        //apparently we have no valid force powers
        return;
    }

    x = BG_ProperForceIndex(x);
    presel = x;

    if direction == 1 {
        //get the next power
        x += 1;
    } else {
        //get the previous power
        x -= 1;
    }

    if x >= NUM_FORCE_POWERS as c_int {
        //cycled off the end.. cycle around to the first
        x = 0;
    }
    if x < 0 {
        //cycled off the beginning.. cycle around to the last
        x = NUM_FORCE_POWERS as c_int - 1;
    }

    i = forcePowerSorted[x as usize]; //the "sorted" value of this power

    while x != presel {
        //loop around to the current force power
        if (*ps).fd.forcePowersKnown & 1i32.wrapping_shl(i as u32) != 0
            && i != (*ps).fd.forcePowerSelected
        {
            //we have the force power
            if i != FP_LEVITATION
                && i != FP_SABER_OFFENSE
                && i != FP_SABER_DEFENSE
                && i != FP_SABERTHROW
            {
                //it's selectable
                foundnext = i;
                break;
            }
        }

        if direction == 1 {
            //next
            x += 1;
        } else {
            //previous
            x -= 1;
        }

        if x >= NUM_FORCE_POWERS as c_int {
            //loop around
            x = 0;
        }
        if x < 0 {
            //loop around
            x = NUM_FORCE_POWERS as c_int - 1;
        }

        i = forcePowerSorted[x as usize]; //set to the sorted value again
    }

    if foundnext != -1 {
        //found one, select it
        (*ps).fd.forcePowerSelected = foundnext;
    }
}

/// `BG_GetItemIndexByTag` (bg_misc.c:2092) — the [`bg_itemlist`] index matching both
/// `tag` (giTag) and `type` (giType); `0` if none. (`type` is a Rust keyword → `r#type`.)
pub fn BG_GetItemIndexByTag(tag: c_int, r#type: c_int) -> c_int {
    let list = addr_of!(bg_itemlist) as *const gitem_t;
    let mut i = 0;
    while i < bg_numItems {
        let it = unsafe { &*list.add(i as usize) };
        if it.giTag == tag && it.giType == r#type {
            return i;
        }
        i += 1;
    }
    0
}

//yeah..
/// `BG_IsItemSelectable` (bg_misc.c:2111) — can this holdable (`item`, an `HI_*`) be
/// cycled to? The dispensers and jetpack are auto-only, so not selectable. `ps` is
/// part of the C signature but unused by the body.
pub fn BG_IsItemSelectable(_ps: *mut playerState_t, item: c_int) -> qboolean {
    if item == HI_HEALTHDISP || item == HI_AMMODISP || item == HI_JETPACK {
        return QFALSE;
    }
    QTRUE
}

/// `BG_CycleInven` (bg_misc.c:2121) — advance the selected holdable (`STAT_HOLDABLE_ITEM`)
/// to the next/previous owned, selectable `HI_*` in the given `direction` (+1 next,
/// else previous), wrapping the `1..HI_NUM_HOLDABLE` range; selects nothing if a full
/// loop finds none. The `dontFreeze` counter is the C's paranoia bound (≥32 → bail).
pub unsafe fn BG_CycleInven(ps: *mut playerState_t, direction: c_int) {
    let mut dontFreeze = 0;

    let list = addr_of!(bg_itemlist) as *const gitem_t;
    let mut i = (*list.add((*ps).stats[STAT_HOLDABLE_ITEM as usize] as usize)).giTag;
    let original = i;

    if direction == 1 {
        //next
        i += 1;
        if i == HI_NUM_HOLDABLE {
            i = 1;
        }
    } else {
        //previous
        i -= 1;
        if i == 0 {
            i = HI_NUM_HOLDABLE - 1;
        }
    }

    while i != original {
        //go in a full loop until hitting something, if hit nothing then select nothing
        if (*ps).stats[STAT_HOLDABLE_ITEMS as usize] & (1 << i) != 0 {
            //we have it, select it.
            if BG_IsItemSelectable(ps, i) != QFALSE {
                (*ps).stats[STAT_HOLDABLE_ITEM as usize] = BG_GetItemIndexByTag(i, IT_HOLDABLE);
                break;
            }
        }

        if direction == 1 {
            //next
            i += 1;
        } else {
            //previous
            i -= 1;
        }

        if i <= 0 {
            //wrap around to the last
            i = HI_NUM_HOLDABLE - 1;
        } else if i >= HI_NUM_HOLDABLE {
            //wrap around to the first
            i = 1;
        }

        dontFreeze += 1;
        if dontFreeze >= 32 {
            //yeah, sure, whatever (it's 2 am and I'm paranoid and can't frickin think)
            break;
        }
    }
}

/// `BG_EvaluateTrajectory` (bg_misc.c:2355) — sample a [`trajectory_t`]'s value
/// (position or angles) at `atTime` into `result`. Pure parametric math shared by game
/// and cgame. FP faithfulness: the unsuffixed `0.001`/`0.5` literals are *doubles* in C,
/// so `TR_LINEAR`/`TR_LINEAR_STOP`/`TR_GRAVITY` compute those products in `f64` then
/// narrow (the `as f64`/`as vec_t` dance); `TR_NONLINEAR_STOP`'s `0.001f` is a *float*,
/// so that one stays `f32`. `sin`/`cos` are the libm `double` versions. `DEFAULT_GRAVITY`
/// is 800. Unknown `trType` is `Com_Error(ERR_DROP)` (diverges → the C trailing `break`
/// is unreachable). Takes the QAGAME side of the `#ifdef`'d error message.
pub fn BG_EvaluateTrajectory(tr: &trajectory_t, mut atTime: c_int, result: &mut vec3_t) {
    match tr.trType {
        TR_STATIONARY | TR_INTERPOLATE => {
            VectorCopy(&tr.trBase, result);
        }
        TR_LINEAR => {
            let deltaTime = ((atTime - tr.trTime) as f64 * 0.001) as vec_t; // milliseconds to seconds
            VectorMA(&tr.trBase, deltaTime, &tr.trDelta, result);
        }
        TR_SINE => {
            let deltaTime = (atTime - tr.trTime) as vec_t / tr.trDuration as vec_t;
            let phase = ((deltaTime * M_PI * 2.0) as f64).sin() as vec_t;
            VectorMA(&tr.trBase, phase, &tr.trDelta, result);
        }
        TR_LINEAR_STOP => {
            if atTime > tr.trTime + tr.trDuration {
                atTime = tr.trTime + tr.trDuration;
            }
            let mut deltaTime = ((atTime - tr.trTime) as f64 * 0.001) as vec_t; // milliseconds to seconds
            if deltaTime < 0.0 {
                deltaTime = 0.0;
            }
            VectorMA(&tr.trBase, deltaTime, &tr.trDelta, result);
        }
        TR_NONLINEAR_STOP => {
            if atTime > tr.trTime + tr.trDuration {
                atTime = tr.trTime + tr.trDuration;
            }
            //new slow-down at end
            let deltaTime = if atTime - tr.trTime > tr.trDuration || atTime - tr.trTime <= 0 {
                0.0
            } else {
                //FIXME: maybe scale this somehow?  So that it starts out faster and stops faster?
                let inner =
                    90.0 - (90.0 * (atTime as vec_t - tr.trTime as vec_t) / tr.trDuration as vec_t);
                tr.trDuration as vec_t * 0.001 * ((DEG2RAD(inner) as f64).cos() as vec_t)
            };
            VectorMA(&tr.trBase, deltaTime, &tr.trDelta, result);
        }
        TR_GRAVITY => {
            let deltaTime = ((atTime - tr.trTime) as f64 * 0.001) as vec_t; // milliseconds to seconds
            VectorMA(&tr.trBase, deltaTime, &tr.trDelta, result);
            // FIXME: local gravity... computed in f64 (the `0.5` literal is a double)
            result[2] = (result[2] as f64
                - 0.5 * DEFAULT_GRAVITY as f64 * deltaTime as f64 * deltaTime as f64)
                as vec_t;
        }
        _ => {
            Com_Error(
                ERR_DROP,
                &format!(
                    "BG_EvaluateTrajectory: [GAME SIDE] unknown trType: {}",
                    tr.trType
                ),
            );
        }
    }
}

/// `BG_EvaluateTrajectoryDelta` (bg_misc.c:2421) — sample the *velocity* (the derivative
/// of [`BG_EvaluateTrajectory`]) of `tr` at `atTime`. Same FP discipline as its sibling
/// (double `0.001`/`0.5`, float `0.001f`, libm `cos`). Carried original-JKA quirk: the
/// default-case `Com_Error` says "unknown trType" but prints `tr->trTime`, not `trType`
/// (recorded in DEVIATIONS). The `TR_LINEAR_STOP`/`TR_NONLINEAR_STOP` early-out cases
/// `return` after clearing `result`.
pub fn BG_EvaluateTrajectoryDelta(tr: &trajectory_t, atTime: c_int, result: &mut vec3_t) {
    match tr.trType {
        TR_STATIONARY | TR_INTERPOLATE => {
            VectorClear(result);
        }
        TR_LINEAR => {
            VectorCopy(&tr.trDelta, result);
        }
        TR_SINE => {
            let deltaTime = (atTime - tr.trTime) as vec_t / tr.trDuration as vec_t;
            let mut phase = ((deltaTime * M_PI * 2.0) as f64).cos() as vec_t; // derivative of sin = cos
            phase = (phase as f64 * 0.5) as vec_t;
            VectorScale(&tr.trDelta, phase, result);
        }
        TR_LINEAR_STOP => {
            if atTime > tr.trTime + tr.trDuration {
                VectorClear(result);
                return;
            }
            VectorCopy(&tr.trDelta, result);
        }
        TR_NONLINEAR_STOP => {
            if atTime - tr.trTime > tr.trDuration || atTime - tr.trTime <= 0 {
                VectorClear(result);
                return;
            }
            let inner =
                90.0 - (90.0 * (atTime as vec_t - tr.trTime as vec_t) / tr.trDuration as vec_t);
            let deltaTime =
                tr.trDuration as vec_t * 0.001 * ((DEG2RAD(inner) as f64).cos() as vec_t);
            VectorScale(&tr.trDelta, deltaTime, result);
        }
        TR_GRAVITY => {
            let deltaTime = ((atTime - tr.trTime) as f64 * 0.001) as vec_t; // milliseconds to seconds
            VectorCopy(&tr.trDelta, result);
            result[2] -= DEFAULT_GRAVITY as vec_t * deltaTime; // FIXME: local gravity...
        }
        _ => {
            Com_Error(
                ERR_DROP,
                &format!("BG_EvaluateTrajectoryDelta: unknown trType: {}", tr.trTime),
            );
        }
    }
}

/// `BG_PlayerTouchesItem` (bg_misc.c:1979) — whether `ps` is close enough to pick up
/// `item` at `atTime`. Items can be grabbed without touching their physical bounds, so
/// this samples the item's position via [`BG_EvaluateTrajectory`] and tests a fixed
/// (deliberately ducked-difference-ignoring) AABB around the player origin: +44/-50 on x,
/// ±36 on y and z. Pure read of `ps.origin` and `item.pos`.
pub fn BG_PlayerTouchesItem(ps: &playerState_t, item: &entityState_t, atTime: c_int) -> qboolean {
    let mut origin: vec3_t = [0.0; 3];

    BG_EvaluateTrajectory(&item.pos, atTime, &mut origin);

    // we are ignoring ducked differences here
    if ps.origin[0] - origin[0] > 44.0
        || ps.origin[0] - origin[0] < -50.0
        || ps.origin[1] - origin[1] > 36.0
        || ps.origin[1] - origin[1] < -36.0
        || ps.origin[2] - origin[2] > 36.0
        || ps.origin[2] - origin[2] < -36.0
    {
        return QFALSE;
    }

    QTRUE
}

/// `BG_CanItemBeGrabbed` (bg_misc.c:2192) — whether the item identified by `ent`
/// (`ent.modelindex` selects a [`bg_itemlist`] entry) may be picked up by player `ps`.
/// Shared by client-side prediction and the server, so it must stay identical on both.
/// A `NULL` `ps` is rejected outright. Otherwise the rules layer up: trueJedi/trueNonJedi
/// pickup gating, no pickups mid-duel, then a per-`giType` switch (weapon-stay, ammo /
/// armor / health caps, ysalamiri-only powerups, CTF/CTY flag-carry logic,
/// holdable-already-owned). Reads the static `bg_itemlist`, [`weaponData`] and
/// [`ammoData`] tables. The original JKA `isJediMaster` block is commented out in the C
/// (the field does not exist in this ABI) and so is not ported — see `DEVIATIONS.md`.
///
/// # Safety
/// `ps` may be null — the C `if (ps)` guard is preserved — and when non-null must point
/// to a valid `playerState_t`. Indexes the `static mut bg_itemlist` by `ent.modelindex`.
pub unsafe fn BG_CanItemBeGrabbed(
    gametype: c_int,
    ent: &entityState_t,
    ps: *const playerState_t,
) -> qboolean {
    if ent.modelindex < 1 || ent.modelindex >= bg_numItems {
        Com_Error(ERR_DROP, "BG_CanItemBeGrabbed: index out of range");
    }

    let item = (addr_of_mut!(bg_itemlist) as *mut gitem_t).add(ent.modelindex as usize);

    if !ps.is_null() {
        if (*ps).trueJedi != QFALSE {
            //force powers and saber only
            if (*item).giType != IT_TEAM //not a flag
                && (*item).giType != IT_ARMOR//not shields
                && ((*item).giType != IT_WEAPON || (*item).giTag != WP_SABER)//not a saber
                && ((*item).giType != IT_HOLDABLE || (*item).giTag != HI_SEEKER)//not a seeker
                && ((*item).giType != IT_POWERUP || (*item).giTag == PW_YSALAMIRI)
            //not a force pick-up
            {
                return QFALSE;
            }
        } else if (*ps).trueNonJedi != QFALSE {
            //can't pick up force powerups
            if ((*item).giType == IT_POWERUP && (*item).giTag != PW_YSALAMIRI) //if a powerup, can only can pick up ysalamiri
                || ((*item).giType == IT_HOLDABLE && (*item).giTag == HI_SEEKER) //if holdable, cannot pick up seeker
                || ((*item).giType == IT_WEAPON && (*item).giTag == WP_SABER)
            //or if it's a saber
            {
                return QFALSE;
            }
        }
        if (*ps).isJediMaster != QFALSE
            && !item.is_null()
            && ((*item).giType == IT_WEAPON || (*item).giType == IT_AMMO)
        {
            //jedi master cannot pick up weapons
            return QFALSE;
        }
        if (*ps).duelInProgress != QFALSE {
            //no picking stuff up while in a duel, no matter what the type is
            return QFALSE;
        }
    } else {
        //safety return since below code assumes a non-null ps
        return QFALSE;
    }

    match (*item).giType {
        IT_WEAPON => {
            if ent.generic1 == (*ps).clientNum && ent.powerups != 0 {
                return QFALSE;
            }
            if (ent.eFlags & EF_DROPPEDWEAPON) == 0
                && ((*ps).stats[STAT_WEAPONS as usize] & (1 << (*item).giTag)) != 0
                && (*item).giTag != WP_THERMAL
                && (*item).giTag != WP_TRIP_MINE
                && (*item).giTag != WP_DET_PACK
            {
                //weaponstay stuff.. if this isn't dropped, and you already have it, you don't get it.
                return QFALSE;
            }
            if (*item).giTag == WP_THERMAL
                || (*item).giTag == WP_TRIP_MINE
                || (*item).giTag == WP_DET_PACK
            {
                //check to see if full on ammo for this, if so, then..
                let ammoIndex = weaponData[(*item).giTag as usize].ammoIndex;
                if (*ps).ammo[ammoIndex as usize] >= ammoData[ammoIndex as usize].max {
                    //don't need it
                    return QFALSE;
                }
            }
            QTRUE // weapons are always picked up
        }

        IT_AMMO => {
            if (*item).giTag == -1 {
                //special case for "all ammo" packs
                return QTRUE;
            }
            if (*ps).ammo[(*item).giTag as usize] >= ammoData[(*item).giTag as usize].max {
                return QFALSE; // can't hold any more
            }
            QTRUE
        }

        IT_ARMOR => {
            if (*ps).stats[STAT_ARMOR as usize] >= (*ps).stats[STAT_MAX_HEALTH as usize]
            /* * item->giTag*/
            {
                return QFALSE;
            }
            QTRUE
        }

        IT_HEALTH => {
            // small and mega healths will go over the max, otherwise
            // don't pick up if already at max
            if ((*ps).fd.forcePowersActive & (1 << FP_RAGE)) != 0 {
                return QFALSE;
            }

            if (*item).quantity == 5 || (*item).quantity == 100 {
                if (*ps).stats[STAT_HEALTH as usize] >= (*ps).stats[STAT_MAX_HEALTH as usize] * 2 {
                    return QFALSE;
                }
                return QTRUE;
            }

            if (*ps).stats[STAT_HEALTH as usize] >= (*ps).stats[STAT_MAX_HEALTH as usize] {
                return QFALSE;
            }
            QTRUE
        }

        IT_POWERUP => {
            if !ps.is_null() && (*ps).powerups[PW_YSALAMIRI as usize] != 0 {
                if (*item).giTag != PW_YSALAMIRI {
                    return QFALSE;
                }
            }
            QTRUE // powerups are always picked up
        }

        IT_TEAM => {
            // team items, such as flags
            if gametype == GT_CTF || gametype == GT_CTY {
                // ent->modelindex2 is non-zero on items if they are dropped
                // we need to know this because we can pick up our dropped flag (and return it)
                // but we can't pick up our flag at base
                if (*ps).persistant[PERS_TEAM as usize] == TEAM_RED {
                    if (*item).giTag == PW_BLUEFLAG
                        || ((*item).giTag == PW_REDFLAG && ent.modelindex2 != 0)
                        || ((*item).giTag == PW_REDFLAG
                            && (*ps).powerups[PW_BLUEFLAG as usize] != 0)
                    {
                        return QTRUE;
                    }
                } else if (*ps).persistant[PERS_TEAM as usize] == TEAM_BLUE {
                    if (*item).giTag == PW_REDFLAG
                        || ((*item).giTag == PW_BLUEFLAG && ent.modelindex2 != 0)
                        || ((*item).giTag == PW_BLUEFLAG
                            && (*ps).powerups[PW_REDFLAG as usize] != 0)
                    {
                        return QTRUE;
                    }
                }
            }

            QFALSE
        }

        IT_HOLDABLE => {
            if ((*ps).stats[STAT_HOLDABLE_ITEMS as usize] & (1 << (*item).giTag)) != 0 {
                return QFALSE;
            }
            QTRUE
        }

        IT_BAD => {
            Com_Error(ERR_DROP, "BG_CanItemBeGrabbed: IT_BAD");
        }

        _ => {
            // Original: #ifndef Q3_VM / #ifndef NDEBUG -- a debug-only diagnostic; it has
            // no effect on the return value (the C `break` falls through to `return qfalse`).
            #[cfg(all(not(feature = "vm"), debug_assertions))]
            crate::codemp::game::g_main::Com_Printf(&format!(
                "BG_CanItemBeGrabbed: unknown enum {}\n",
                (*item).giType
            ));
            QFALSE
        }
    }
}

/// `BG_TouchJumpPad` (bg_misc.c:2676) — shared jump-pad response. Spectator pm_types are
/// ignored; otherwise the player is given the pad's `origin2` as their velocity and the
/// pad is recorded (`jumppad_ent`/`jumppad_frame`) so the event sound isn't replayed while
/// sitting in a fat trigger. The original computes an `effectNum` from the pad's pitch on
/// the first-touch frame but never uses it (vestigial) — kept for fidelity, see
/// `DEVIATIONS.md`. Pure: [`vectoangles`]/[`AngleNormalize180`]/[`VectorCopy`] math, no
/// `trap_*`.
pub fn BG_TouchJumpPad(ps: &mut playerState_t, jumppad: &entityState_t) {
    // spectators don't use jump pads
    if ps.pm_type != PM_NORMAL && ps.pm_type != PM_JETPACK && ps.pm_type != PM_FLOAT {
        return;
    }

    // if we didn't hit this same jumppad the previous frame
    // then don't play the event sound again if we are in a fat trigger
    if ps.jumppad_ent != jumppad.number {
        let mut angles: vec3_t = [0.0; 3];

        vectoangles(&jumppad.origin2, &mut angles);
        let p = AngleNormalize180(angles[PITCH]).abs();
        // `effectNum` is assigned but never read in the original C body (vestigial);
        // underscore-bound here so the dead store doesn't warn.
        let _effectNum = if p < 45.0 { 0 } else { 1 };
    }
    // remember hitting this jumppad this frame
    ps.jumppad_ent = jumppad.number;
    ps.jumppad_frame = ps.pmove_framecount;
    // give the player the velocity from the jumppad
    VectorCopy(&jumppad.origin2, &mut ps.velocity);
}

/// `BG_EmplacedView` (bg_misc.c:2712) — shared emplaced-gun yaw constriction. Measures
/// how far the gunner's yaw (`angles`) has swung from the mount's base yaw (`baseAngles`)
/// and clamps it to `±constraint`: returns `0` when in range, `1` when only slightly out
/// (the clamp's `amt` overshoot is within `±1`), and `2` when significantly out (the
/// caller should force the view). `*newYaw` is written with the clamped yaw only when out
/// of range — left untouched on the `0` return. Pure [`AngleSubtract`] yaw math, reading
/// only the `[YAW]` component of each angle vector.
pub fn BG_EmplacedView(
    baseAngles: &vec3_t,
    angles: &vec3_t,
    newYaw: &mut vec_t,
    constraint: vec_t,
) -> c_int {
    let mut dif = AngleSubtract(baseAngles[YAW], angles[YAW]);

    if dif > constraint || dif < -constraint {
        let amt;

        if dif > constraint {
            amt = dif - constraint;
            dif = constraint;
        } else if dif < -constraint {
            amt = dif + constraint;
            dif = -constraint;
        } else {
            amt = 0.0;
        }

        *newYaw = AngleSubtract(angles[YAW], -dif);

        if amt > 1.0 || amt < -1.0 {
            // significant, force the view
            return 2;
        } else {
            // just a little out of range
            return 1;
        }
    }

    0
}

/// `BG_FileExists` (bg_misc.c:319) — true iff `fileName` names a file the engine can open
/// for reading. Opens it with [`trap_FS_FOpenFile`](crate::trap::FS_FOpenFile) in `FS_READ`
/// mode and, on a non-zero handle, closes it again and reports success; a NULL or empty name
/// is rejected up front. No oracle — pure engine file I/O (mutates no game state, like the
/// memory subsystem above). [`BG_ValidateSkinForTeam`] is its only in-file caller.
///
/// The C passes the `const char *` straight to the trap; the Rust trap takes a `&str`, so the
/// pointer is round-tripped through [`CStr`] here (lossy on non-UTF-8 — JKA asset paths are
/// ASCII). See `DEVIATIONS.md`.
///
/// # Safety
/// `fileName` must be NULL or a valid NUL-terminated C string.
pub unsafe fn BG_FileExists(fileName: *const c_char) -> qboolean {
    if !fileName.is_null() && *fileName != 0 {
        let name = CStr::from_ptr(fileName).to_string_lossy();
        let (_len, fh) = trap::FS_FOpenFile(&name, FS_READ);
        if fh > 0 {
            trap::FS_FCloseFile(fh);
            return QTRUE;
        }
    }

    QFALSE
}

/// `BG_ParseField` (bg_misc.c:358) — set one binary spawn field from a key/value text
/// pair. Walks the NUL-`name`-terminated `l_fields` descriptor table; on the first
/// case-insensitive [`Q_stricmp`] name match it decodes `value` per the field's
/// [`fieldtype_t`] and writes it into `ent` at `f->ofs` bytes, then returns. Backs the
/// entity/vehicle/saber spawn-string parsers. This is the `#ifndef UI_EXPORTS` / `QAGAME`
/// branch (this is the server game module), so `F_LSTRING` allocates via [`G_NewString`].
///
/// **`F_PARM1..=F_PARM16` are a deliberate no-op.** In C (QAGAME) they call ICARUS's
/// `Q3_SetParm` (which is ported, in `g_ICARUScb.rs`), but that is a live engine-trap call;
/// the extracted oracle C for this fn is compiled non-QAGAME (the F_PARM arm no-ops), so
/// calling it here would both break bit-exact oracle parity and SIGSEGV the trap-less test
/// harness. Kept as a no-op == `F_IGNORE`, matching the oracle. See `DEVIATIONS.md`.
///
/// `atoi`/`atof` are the platform libc (same choice as [`BG_LegalizedForcePowers`] above).
/// `sscanf` is [`bg_lib::sscanf`](crate::codemp::game::bg_lib::sscanf): the `vm` shim takes
/// an explicit output-pointer slice, the native build a true variadic libc call, so the
/// `F_VECTOR` parse has a small `#[cfg]` split — both fill the same three floats.
///
/// # Safety
/// `l_fields` must be a valid NUL-`name`-terminated [`BG_field_t`] table; `key`/`value`
/// valid NUL-terminated C strings; `ent` must point to a buffer in which every matched
/// field's `ofs`+width is in bounds — the contract the C spawner already upholds.
pub unsafe fn BG_ParseField(
    l_fields: *mut BG_field_t,
    key: *const c_char,
    value: *const c_char,
    ent: *mut byte,
) {
    let mut f = l_fields;
    while !(*f).name.is_null() {
        if Q_stricmp((*f).name, key) == 0 {
            // found it
            let b = ent;

            match (*f).r#type {
                F_LSTRING => {
                    *(b.add((*f).ofs as usize) as *mut *mut c_char) = G_NewString(value);
                }
                F_VECTOR => {
                    let mut vec: vec3_t = [0.0; 3];
                    #[cfg(feature = "vm")]
                    sscanf(
                        value,
                        c"%f %f %f".as_ptr(),
                        &[
                            addr_of_mut!(vec[0]) as *mut c_void,
                            addr_of_mut!(vec[1]) as *mut c_void,
                            addr_of_mut!(vec[2]) as *mut c_void,
                        ],
                    );
                    #[cfg(not(feature = "vm"))]
                    sscanf(
                        value,
                        c"%f %f %f".as_ptr(),
                        addr_of_mut!(vec[0]),
                        addr_of_mut!(vec[1]),
                        addr_of_mut!(vec[2]),
                    );
                    let p = b.add((*f).ofs as usize) as *mut f32;
                    *p.add(0) = vec[0];
                    *p.add(1) = vec[1];
                    *p.add(2) = vec[2];
                }
                F_INT => {
                    *(b.add((*f).ofs as usize) as *mut c_int) = atoi(value);
                }
                F_FLOAT => {
                    *(b.add((*f).ofs as usize) as *mut f32) = atof(value) as f32;
                }
                F_ANGLEHACK => {
                    let v = atof(value) as f32;
                    let p = b.add((*f).ofs as usize) as *mut f32;
                    *p.add(0) = 0.0;
                    *p.add(1) = v;
                    *p.add(2) = 0.0;
                }
                // F_PARM1..=F_PARM16 (QAGAME) -> Q3_SetParm(ent->s.number, type-F_PARM1,
                // value). Deliberate no-op (== F_IGNORE): Q3_SetParm exists (g_ICARUScb) but is
                // a live engine-trap; the oracle C is non-QAGAME, so calling it breaks parity +
                // SIGSEGVs the test harness. See doc above / DEVIATIONS.md.
                F_PARM1 | F_PARM2 | F_PARM3 | F_PARM4 | F_PARM5 | F_PARM6 | F_PARM7 | F_PARM8
                | F_PARM9 | F_PARM10 | F_PARM11 | F_PARM12 | F_PARM13 | F_PARM14 | F_PARM15
                | F_PARM16 => {}
                // default / F_IGNORE
                _ => {}
            }
            return;
        }
        f = f.add(1);
    }
}

/// `BG_IsValidCharacterModel` (bg_misc.c:2753) — rejects the single-player-only "kyle"
/// first-person-look skins (`fpls`/`fpls2`/`fpls3`) that shipped in the assets but aren't
/// meant for MP; every other model/skin pair is allowed. Pure [`Q_stricmp`] comparisons.
///
/// # Safety
/// `modelName`/`skinName` must be valid NUL-terminated C strings (passed straight to
/// [`Q_stricmp`]).
pub unsafe fn BG_IsValidCharacterModel(
    modelName: *const c_char,
    skinName: *const c_char,
) -> qboolean {
    if Q_stricmp(skinName, c"menu".as_ptr()) == 0 {
        return QFALSE;
    } else if Q_stricmp(modelName, c"kyle".as_ptr()) == 0 {
        if Q_stricmp(skinName, c"fpls".as_ptr()) == 0 {
            return QFALSE;
        } else if Q_stricmp(skinName, c"fpls2".as_ptr()) == 0 {
            return QFALSE;
        } else if Q_stricmp(skinName, c"fpls3".as_ptr()) == 0 {
            return QFALSE;
        }
    }
    QTRUE
}

/// `BG_ValidateSkinForTeam` (bg_misc.c:2773) — in team games, coerce a player's chosen
/// `skinName` into the team's mandated red/blue variant (mutated in place), returning
/// [`QTRUE`] only if no change was needed. Custom `jedi_*` player skins are exempt: they
/// keep their skin and instead receive a flat team `colors` tint (red/blue) when `colors`
/// is non-NULL. Otherwise, a skin that is already the right team color passes; "blue"/"red"
/// /"default", a multi-skin (`|`-joined) name, or a non-character model
/// ([`BG_IsValidCharacterModel`]) is forced straight to the bare team name; anything else
/// gets a `_red`/`_blue` suffix appended (unless too long for [`MAX_QPATH`], or it already
/// ends in the suffix), then falls back to the bare team name if the resulting
/// `model_<skin>.skin` file doesn't exist ([`BG_FileExists`]).
///
/// No C oracle — like [`BG_FileExists`], the deciding branches route through that engine
/// file-I/O trap, so behaviour isn't a computable pure function. See `DEVIATIONS.md`.
///
/// # Safety
/// `modelName` must be a valid C string of length ≥ 5 (the C copies 5 bytes unconditionally).
/// `skinName` must point to a writable NUL-terminated buffer of [`MAX_QPATH`] bytes (it is
/// rewritten via [`Q_strncpyz`]/[`Q_strcat`]). `colors`, if non-NULL, must point to at least
/// 3 writable [`vec_t`].
pub unsafe fn BG_ValidateSkinForTeam(
    modelName: *const c_char,
    skinName: *mut c_char,
    team: c_int,
    colors: *mut vec_t,
) -> qboolean {
    if Q_stricmpn(modelName, c"jedi_".as_ptr(), 5) == 0 {
        // argh, it's a custom player skin!
        if team == TEAM_RED && !colors.is_null() {
            *colors.add(0) = 1.0;
            *colors.add(1) = 0.0;
            *colors.add(2) = 0.0;
        } else if team == TEAM_BLUE && !colors.is_null() {
            *colors.add(0) = 0.0;
            *colors.add(1) = 0.0;
            *colors.add(2) = 1.0;
        }
        return QTRUE;
    }

    if team == TEAM_RED {
        if Q_stricmp(c"red".as_ptr(), skinName) != 0 {
            // not "red"
            if Q_stricmp(c"blue".as_ptr(), skinName) == 0
                || Q_stricmp(c"default".as_ptr(), skinName) == 0
                || !strchr(skinName, '|' as c_int).is_null() // a multi-skin playerModel
                || BG_IsValidCharacterModel(modelName, skinName) == QFALSE
            {
                Q_strncpyz(skinName, c"red".as_ptr(), MAX_QPATH as c_int);
                return QFALSE;
            } else {
                // need to set it to red
                let len = strlen(skinName) as c_int;
                if len < 3 {
                    // too short to be "red"
                    Q_strcat(skinName, MAX_QPATH as c_int, c"_red".as_ptr());
                } else {
                    let start = skinName.add((len - 3) as usize);
                    if Q_strncmp(c"red".as_ptr(), start, 3) != 0 {
                        // doesn't already end in "red"
                        if len + 4 >= MAX_QPATH as c_int {
                            // too big to append "_red"
                            Q_strncpyz(skinName, c"red".as_ptr(), MAX_QPATH as c_int);
                            return QFALSE;
                        } else {
                            Q_strcat(skinName, MAX_QPATH as c_int, c"_red".as_ptr());
                        }
                    }
                }
                // if file does not exist, set to "red"
                if BG_FileExists(va(format_args!(
                    "models/players/{}/model_{}.skin",
                    Sz(modelName),
                    Sz(skinName)
                )) as *const c_char)
                    == QFALSE
                {
                    Q_strncpyz(skinName, c"red".as_ptr(), MAX_QPATH as c_int);
                }
                return QFALSE;
            }
        }
    } else if team == TEAM_BLUE {
        if Q_stricmp(c"blue".as_ptr(), skinName) != 0 {
            if Q_stricmp(c"red".as_ptr(), skinName) == 0
                || Q_stricmp(c"default".as_ptr(), skinName) == 0
                || !strchr(skinName, '|' as c_int).is_null() // a multi-skin playerModel
                || BG_IsValidCharacterModel(modelName, skinName) == QFALSE
            {
                Q_strncpyz(skinName, c"blue".as_ptr(), MAX_QPATH as c_int);
                return QFALSE;
            } else {
                // need to set it to blue
                let len = strlen(skinName) as c_int;
                if len < 4 {
                    // too short to be "blue"
                    Q_strcat(skinName, MAX_QPATH as c_int, c"_blue".as_ptr());
                } else {
                    let start = skinName.add((len - 4) as usize);
                    if Q_strncmp(c"blue".as_ptr(), start, 4) != 0 {
                        // doesn't already end in "blue"
                        if len + 5 >= MAX_QPATH as c_int {
                            // too big to append "_blue"
                            Q_strncpyz(skinName, c"blue".as_ptr(), MAX_QPATH as c_int);
                            return QFALSE;
                        } else {
                            Q_strcat(skinName, MAX_QPATH as c_int, c"_blue".as_ptr());
                        }
                    }
                }
                // if file does not exist, set to "blue"
                if BG_FileExists(va(format_args!(
                    "models/players/{}/model_{}.skin",
                    Sz(modelName),
                    Sz(skinName)
                )) as *const c_char)
                    == QFALSE
                {
                    Q_strncpyz(skinName, c"blue".as_ptr(), MAX_QPATH as c_int);
                }
                return QFALSE;
            }
        }
    }
    QTRUE
}

/// `BG_PlayerStateToEntityState` (bg_misc.c:2901) — collapse a `playerState_t` into the
/// networked `entityState_t`. Done on the server after each set of `usercmd_t` and on the
/// client after local prediction. Straight field copying plus a few derived bits: the
/// invisible/player `eType` gate, the interpolated position/angle trajectories (optionally
/// `snap`ped via the inline [`snap_vector`]), the powerup bitmask, the seeker-drone/dead
/// `eFlags`, and the external/sequenced event latch (which advances `ps.entityEventSequence`,
/// the one mutation of `ps`). Retail-PC populates `s.isJediMaster = ps.isJediMaster` and
/// `s.time2 = ps.holocronBits` (Xbox left both commented with `time2 = 0`). No `trap_*`.
pub fn BG_PlayerStateToEntityState(ps: &mut playerState_t, s: &mut entityState_t, snap: qboolean) {
    if ps.pm_type == PM_INTERMISSION || ps.pm_type == PM_SPECTATOR {
        s.eType = ET_INVISIBLE;
    } else if ps.stats[STAT_HEALTH as usize] <= GIB_HEALTH {
        s.eType = ET_INVISIBLE;
    } else {
        s.eType = ET_PLAYER;
    }

    s.number = ps.clientNum;

    s.pos.trType = TR_INTERPOLATE;
    VectorCopy(&ps.origin, &mut s.pos.trBase);
    if snap != QFALSE {
        snap_vector(&mut s.pos.trBase);
    }
    // set the trDelta for flag direction
    VectorCopy(&ps.velocity, &mut s.pos.trDelta);

    s.apos.trType = TR_INTERPOLATE;
    VectorCopy(&ps.viewangles, &mut s.apos.trBase);
    if snap != QFALSE {
        snap_vector(&mut s.apos.trBase);
    }

    s.trickedentindex = ps.fd.forceMindtrickTargetIndex;
    s.trickedentindex2 = ps.fd.forceMindtrickTargetIndex2;
    s.trickedentindex3 = ps.fd.forceMindtrickTargetIndex3;
    s.trickedentindex4 = ps.fd.forceMindtrickTargetIndex4;

    s.forceFrame = ps.saberLockFrame;

    s.emplacedOwner = ps.electrifyTime;

    s.speed = ps.speed;

    s.genericenemyindex = ps.genericEnemyIndex;

    s.activeForcePass = ps.activeForcePass;

    s.angles2[YAW] = ps.movementDir as vec_t;
    s.legsAnim = ps.legsAnim;
    s.torsoAnim = ps.torsoAnim;

    s.legsFlip = ps.legsFlip;
    s.torsoFlip = ps.torsoFlip;

    s.clientNum = ps.clientNum; // ET_PLAYER looks here instead of at number
                                // so corpses can also reference the proper config
    s.eFlags = ps.eFlags;
    s.eFlags2 = ps.eFlags2;

    s.saberInFlight = ps.saberInFlight;
    s.saberEntityNum = ps.saberEntityNum;
    s.saberMove = ps.saberMove;
    s.forcePowersActive = ps.fd.forcePowersActive;

    if ps.duelInProgress != QFALSE {
        s.bolt1 = 1;
    } else {
        s.bolt1 = 0;
    }

    s.otherEntityNum2 = ps.emplacedIndex;

    s.saberHolstered = ps.saberHolstered;

    if ps.genericEnemyIndex != -1 {
        s.eFlags |= EF_SEEKERDRONE;
    }

    if ps.stats[STAT_HEALTH as usize] <= 0 {
        s.eFlags |= EF_DEAD;
    } else {
        s.eFlags &= !EF_DEAD;
    }

    if ps.externalEvent != 0 {
        s.event = ps.externalEvent;
        s.eventParm = ps.externalEventParm;
    } else if ps.entityEventSequence < ps.eventSequence {
        if ps.entityEventSequence < ps.eventSequence - MAX_PS_EVENTS as c_int {
            ps.entityEventSequence = ps.eventSequence - MAX_PS_EVENTS as c_int;
        }
        let seq = (ps.entityEventSequence & (MAX_PS_EVENTS as c_int - 1)) as usize;
        s.event = ps.events[seq] | ((ps.entityEventSequence & 3) << 8);
        s.eventParm = ps.eventParms[seq];
        ps.entityEventSequence += 1;
    }

    s.weapon = ps.weapon;
    s.groundEntityNum = ps.groundEntityNum;

    s.powerups = 0;
    for i in 0..MAX_POWERUPS {
        if ps.powerups[i] != 0 {
            s.powerups |= 1 << i;
        }
    }

    s.loopSound = ps.loopSound;
    s.generic1 = ps.generic1;

    // NOT INCLUDED IN ENTITYSTATETOPLAYERSTATE:
    s.modelindex2 = ps.weaponstate;
    s.constantLight = ps.weaponChargeTime;

    VectorCopy(&ps.lastHitLoc, &mut s.origin2);

    s.isJediMaster = ps.isJediMaster;

    s.time2 = ps.holocronBits;

    s.fireflag = ps.fd.saberAnimLevel;

    s.heldByClient = ps.heldByClient;
    s.ragAttach = ps.ragAttach;

    s.iModelScale = ps.iModelScale;

    s.brokenLimbs = ps.brokenLimbs;

    s.hasLookTarget = ps.hasLookTarget;
    s.lookTarget = ps.lookTarget;

    s.customRGBA[0] = ps.customRGBA[0];
    s.customRGBA[1] = ps.customRGBA[1];
    s.customRGBA[2] = ps.customRGBA[2];
    s.customRGBA[3] = ps.customRGBA[3];

    s.m_iVehicleNum = ps.m_iVehicleNum;
}

/// `BG_PlayerStateToEntityStateExtraPolate` (bg_misc.c:3052) — the extrapolating sibling of
/// [`BG_PlayerStateToEntityState`]. Identical field-by-field to that fn save the *position*
/// trajectory: it sets `pos.trType = TR_LINEAR_STOP` (vs `TR_INTERPOLATE`) and stamps
/// `pos.trTime = time` + `pos.trDuration = 50` (`1000 / sv_fps`, default 20) so the client can
/// linearly extrapolate the entity's motion ahead of the next snapshot. The angle trajectory
/// (`apos`) stays `TR_INTERPOLATE` exactly as in the sibling, and every other field copy,
/// `eType`/`eFlags` gate, event latch (the one `ps` mutation), and the retail-PC
/// `isJediMaster`/`time2 = holocronBits` populations are verbatim-identical. No `trap_*`.
pub fn BG_PlayerStateToEntityStateExtraPolate(
    ps: &mut playerState_t,
    s: &mut entityState_t,
    time: c_int,
    snap: qboolean,
) {
    if ps.pm_type == PM_INTERMISSION || ps.pm_type == PM_SPECTATOR {
        s.eType = ET_INVISIBLE;
    } else if ps.stats[STAT_HEALTH as usize] <= GIB_HEALTH {
        s.eType = ET_INVISIBLE;
    } else {
        s.eType = ET_PLAYER;
    }

    s.number = ps.clientNum;

    s.pos.trType = TR_LINEAR_STOP;
    VectorCopy(&ps.origin, &mut s.pos.trBase);
    if snap != QFALSE {
        snap_vector(&mut s.pos.trBase);
    }
    // set the trDelta for flag direction and linear prediction
    VectorCopy(&ps.velocity, &mut s.pos.trDelta);
    // set the time for linear prediction
    s.pos.trTime = time;
    // set maximum extra polation time
    s.pos.trDuration = 50; // 1000 / sv_fps (default = 20)

    s.apos.trType = TR_INTERPOLATE;
    VectorCopy(&ps.viewangles, &mut s.apos.trBase);
    if snap != QFALSE {
        snap_vector(&mut s.apos.trBase);
    }

    s.trickedentindex = ps.fd.forceMindtrickTargetIndex;
    s.trickedentindex2 = ps.fd.forceMindtrickTargetIndex2;
    s.trickedentindex3 = ps.fd.forceMindtrickTargetIndex3;
    s.trickedentindex4 = ps.fd.forceMindtrickTargetIndex4;

    s.forceFrame = ps.saberLockFrame;

    s.emplacedOwner = ps.electrifyTime;

    s.speed = ps.speed;

    s.genericenemyindex = ps.genericEnemyIndex;

    s.activeForcePass = ps.activeForcePass;

    s.angles2[YAW] = ps.movementDir as vec_t;
    s.legsAnim = ps.legsAnim;
    s.torsoAnim = ps.torsoAnim;

    s.legsFlip = ps.legsFlip;
    s.torsoFlip = ps.torsoFlip;

    s.clientNum = ps.clientNum; // ET_PLAYER looks here instead of at number
                                // so corpses can also reference the proper config
    s.eFlags = ps.eFlags;
    s.eFlags2 = ps.eFlags2;

    s.saberInFlight = ps.saberInFlight;
    s.saberEntityNum = ps.saberEntityNum;
    s.saberMove = ps.saberMove;
    s.forcePowersActive = ps.fd.forcePowersActive;

    if ps.duelInProgress != QFALSE {
        s.bolt1 = 1;
    } else {
        s.bolt1 = 0;
    }

    s.otherEntityNum2 = ps.emplacedIndex;

    s.saberHolstered = ps.saberHolstered;

    if ps.genericEnemyIndex != -1 {
        s.eFlags |= EF_SEEKERDRONE;
    }

    if ps.stats[STAT_HEALTH as usize] <= 0 {
        s.eFlags |= EF_DEAD;
    } else {
        s.eFlags &= !EF_DEAD;
    }

    if ps.externalEvent != 0 {
        s.event = ps.externalEvent;
        s.eventParm = ps.externalEventParm;
    } else if ps.entityEventSequence < ps.eventSequence {
        if ps.entityEventSequence < ps.eventSequence - MAX_PS_EVENTS as c_int {
            ps.entityEventSequence = ps.eventSequence - MAX_PS_EVENTS as c_int;
        }
        let seq = (ps.entityEventSequence & (MAX_PS_EVENTS as c_int - 1)) as usize;
        s.event = ps.events[seq] | ((ps.entityEventSequence & 3) << 8);
        s.eventParm = ps.eventParms[seq];
        ps.entityEventSequence += 1;
    }

    s.weapon = ps.weapon;
    s.groundEntityNum = ps.groundEntityNum;

    s.powerups = 0;
    for i in 0..MAX_POWERUPS {
        if ps.powerups[i] != 0 {
            s.powerups |= 1 << i;
        }
    }

    s.loopSound = ps.loopSound;
    s.generic1 = ps.generic1;

    // NOT INCLUDED IN ENTITYSTATETOPLAYERSTATE:
    s.modelindex2 = ps.weaponstate;
    s.constantLight = ps.weaponChargeTime;

    VectorCopy(&ps.lastHitLoc, &mut s.origin2);

    s.isJediMaster = ps.isJediMaster;

    s.time2 = ps.holocronBits;

    s.fireflag = ps.fd.saberAnimLevel;

    s.heldByClient = ps.heldByClient;
    s.ragAttach = ps.ragAttach;

    s.iModelScale = ps.iModelScale;

    s.brokenLimbs = ps.brokenLimbs;

    s.hasLookTarget = ps.hasLookTarget;
    s.lookTarget = ps.lookTarget;

    s.customRGBA[0] = ps.customRGBA[0];
    s.customRGBA[1] = ps.customRGBA[1];
    s.customRGBA[2] = ps.customRGBA[2];
    s.customRGBA[3] = ps.customRGBA[3];

    s.m_iVehicleNum = ps.m_iVehicleNum;
}

/// `BG_ModelCache` (bg_misc.c:3216) — precache a model (and optional skin) by name. This is
/// the `#ifdef QAGAME` branch (this is the server game module): register the skin if one was
/// given, then briefly instance the ghoul2 model via [`trap_G2API_InitGhoul2Model`] and tear
/// it straight back down with [`trap_G2API_CleanGhoul2Models`] — a load-and-discard that warms
/// the engine's caches. Always returns `0` (the cgame/ui branch returns a real model handle).
/// No oracle — pure engine asset I/O, mutating no game state (like [`BG_FileExists`] above).
///
/// [`trap_G2API_InitGhoul2Model`]: crate::trap::G2API_InitGhoul2Model
/// [`trap_G2API_CleanGhoul2Models`]: crate::trap::G2API_CleanGhoul2Models
///
/// The C passes the `const char *` names straight to the traps; the Rust traps take `&str`, so
/// the pointers are round-tripped through [`CStr`] here (lossy on non-UTF-8 — JKA asset paths
/// are ASCII). See `DEVIATIONS.md`.
///
/// # Safety
/// `modelName` must be a valid NUL-terminated C string; `skinName` must be NULL or one.
pub unsafe fn BG_ModelCache(modelName: *const c_char, skinName: *const c_char) -> c_int {
    let mut g2: *mut c_void = core::ptr::null_mut();

    if !skinName.is_null() && *skinName != 0 {
        let skin = CStr::from_ptr(skinName).to_string_lossy();
        trap::R_RegisterSkin(&skin);
    }

    // I could hook up a precache ghoul2 function, but oh well, this works
    trap::G2API_InitGhoul2Model(addr_of_mut!(g2), modelName, 0, 0, 0, 0, 0);
    if !g2.is_null() {
        // now get rid of it
        trap::G2API_CleanGhoul2Models(addr_of_mut!(g2));
    }
    0
}

// In C this function is wrapped in `#ifdef __LCC__` (the VM/LCC-only build path); we
// port it unconditionally.
// given a boltmatrix, return in vec a normalised vector for the axis requested in flags
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
        // No default in the C switch: an unmatched flag leaves `vec` untouched.
        _ => {}
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::codemp::game::g_mem::{G_InitMemory, POOL_LOCK};
    use crate::codemp::game::q_shared_h::{MAX_PERSISTANT, MAX_POWERUPS, MAX_STATS, MAX_WEAPONS};
    use crate::oracle::{
        jka_BG_ParseField, jka_bg_add_pred_event, jka_bg_emplaced_view, jka_bg_eval_traj,
        jka_bg_eval_traj_delta, jka_bg_is_valid_character_model, jka_bg_legalize_force_powers,
        jka_bg_player_state_to_entity_state, jka_bg_player_state_to_entity_state_extrapolate,
        jka_bg_player_touches_item, jka_bg_touch_jump_pad, jka_bg_vectoyaw,
        jka_bgitem_can_item_be_grabbed,
    };
    use std::ffi::{CStr, CString};

    /// Parity for `BG_CanItemBeGrabbed`: run EVERY real `bg_itemlist` entry
    /// (`1..bg_numItems`) through a curated set of player/entity scenarios and compare
    /// the Rust port against the verbatim C oracle. Iterating the whole table means each
    /// `giType` branch (weapon-stay, ammo/armor/health caps, ysalamiri powerups, CTF/CTY
    /// flag carry, holdable-owned) is driven by the real item that selects it, while the
    /// scenarios exercise the trueJedi/trueNonJedi gates, the duel block, the NULL-`ps`
    /// safety return, and the per-branch capacity/ownership conditions.
    #[test]
    fn bg_can_item_be_grabbed_matches_oracle() {
        struct Scn {
            gametype: c_int,
            has_ps: bool,
            trueJedi: c_int,
            trueNonJedi: c_int,
            isJediMaster: c_int,
            duelInProgress: c_int,
            clientNum: c_int,
            forcePowersActive: c_int,
            generic1: c_int,
            es_powerups: c_int,
            eFlags: c_int,
            modelindex2: c_int,
            stats: [c_int; MAX_STATS],
            persistant: [c_int; MAX_PERSISTANT],
            ammo: [c_int; MAX_WEAPONS],
            powerups: [c_int; MAX_POWERUPS],
        }
        let mk = || Scn {
            gametype: 0,
            has_ps: true,
            trueJedi: 0,
            trueNonJedi: 0,
            isJediMaster: 0,
            duelInProgress: 0,
            clientNum: 3,
            forcePowersActive: 0,
            generic1: -1,
            es_powerups: 0,
            eFlags: 0,
            modelindex2: 0,
            stats: [0; MAX_STATS],
            persistant: [0; MAX_PERSISTANT],
            ammo: [0; MAX_WEAPONS],
            powerups: [0; MAX_POWERUPS],
        };

        let mut scns: Vec<Scn> = Vec::new();
        scns.push(mk()); // baseline: empty player, FFA
        scns.push(Scn {
            has_ps: false,
            ..mk()
        }); // NULL ps -> safety return
        scns.push(Scn {
            trueJedi: 1,
            ..mk()
        }); // force/saber-only gate
        scns.push(Scn {
            trueNonJedi: 1,
            ..mk()
        }); // no-force-powerups gate
        scns.push(Scn {
            isJediMaster: 1,
            ..mk()
        }); // jedi master: no weapons/ammo
        scns.push(Scn {
            duelInProgress: 1,
            ..mk()
        }); // mid-duel: nothing
        scns.push(Scn {
            stats: {
                let mut s = [0; MAX_STATS];
                s[STAT_WEAPONS as usize] = !0;
                s
            },
            ..mk()
        }); // owns every weapon (weaponstay)
        scns.push(Scn {
            eFlags: EF_DROPPEDWEAPON,
            stats: {
                let mut s = [0; MAX_STATS];
                s[STAT_WEAPONS as usize] = !0;
                s
            },
            ..mk()
        }); // dropped weapon bypasses weaponstay
        scns.push(Scn {
            generic1: 3,
            clientNum: 3,
            es_powerups: 1,
            ..mk()
        }); // own un-grabbable dropped weapon
        scns.push(Scn {
            ammo: [9999; MAX_WEAPONS],
            ..mk()
        }); // full on all ammo
        scns.push(Scn {
            stats: {
                let mut s = [0; MAX_STATS];
                s[STAT_ARMOR as usize] = 100;
                s[STAT_MAX_HEALTH as usize] = 100;
                s
            },
            ..mk()
        }); // full armor
        scns.push(Scn {
            stats: {
                let mut s = [0; MAX_STATS];
                s[STAT_HEALTH as usize] = 100;
                s[STAT_MAX_HEALTH as usize] = 100;
                s
            },
            ..mk()
        }); // full health
        scns.push(Scn {
            forcePowersActive: 1 << FP_RAGE,
            ..mk()
        }); // rage active (health refusal)
        scns.push(Scn {
            powerups: {
                let mut p = [0; MAX_POWERUPS];
                p[PW_YSALAMIRI as usize] = 1;
                p
            },
            ..mk()
        }); // already has ysalamiri
        scns.push(Scn {
            stats: {
                let mut s = [0; MAX_STATS];
                s[STAT_HOLDABLE_ITEMS as usize] = !0;
                s
            },
            ..mk()
        }); // owns every holdable
            // CTF/CTY flag-carry logic
        scns.push(Scn {
            gametype: GT_CTF,
            persistant: {
                let mut p = [0; MAX_PERSISTANT];
                p[PERS_TEAM as usize] = TEAM_RED;
                p
            },
            ..mk()
        });
        scns.push(Scn {
            gametype: GT_CTF,
            persistant: {
                let mut p = [0; MAX_PERSISTANT];
                p[PERS_TEAM as usize] = TEAM_BLUE;
                p
            },
            ..mk()
        });
        scns.push(Scn {
            gametype: GT_CTY,
            modelindex2: 1,
            persistant: {
                let mut p = [0; MAX_PERSISTANT];
                p[PERS_TEAM as usize] = TEAM_RED;
                p
            },
            ..mk()
        }); // dropped flag
        scns.push(Scn {
            gametype: GT_CTY,
            persistant: {
                let mut p = [0; MAX_PERSISTANT];
                p[PERS_TEAM as usize] = TEAM_BLUE;
                p
            },
            powerups: {
                let mut p = [0; MAX_POWERUPS];
                p[PW_REDFLAG as usize] = 1;
                p
            },
            ..mk()
        }); // carrying enemy flag
        scns.push(Scn {
            gametype: GT_CTF,
            persistant: {
                let mut p = [0; MAX_PERSISTANT];
                p[PERS_TEAM as usize] = TEAM_RED;
                p
            },
            powerups: {
                let mut p = [0; MAX_POWERUPS];
                p[PW_BLUEFLAG as usize] = 1;
                p
            },
            ..mk()
        });

        for mi in 1..bg_numItems {
            for s in &scns {
                // Rust: real entityState_t / playerState_t, ps null when !has_ps.
                let mut es: entityState_t = unsafe { core::mem::zeroed() };
                es.modelindex = mi;
                es.modelindex2 = s.modelindex2;
                es.generic1 = s.generic1;
                es.powerups = s.es_powerups;
                es.eFlags = s.eFlags;

                let mut psv: playerState_t = unsafe { core::mem::zeroed() };
                psv.trueJedi = s.trueJedi;
                psv.trueNonJedi = s.trueNonJedi;
                psv.isJediMaster = s.isJediMaster;
                psv.duelInProgress = s.duelInProgress;
                psv.clientNum = s.clientNum;
                psv.fd.forcePowersActive = s.forcePowersActive;
                psv.stats = s.stats;
                psv.persistant = s.persistant;
                psv.ammo = s.ammo;
                psv.powerups = s.powerups;

                let ps_ptr = if s.has_ps {
                    &psv as *const playerState_t
                } else {
                    core::ptr::null()
                };
                let rust = unsafe { BG_CanItemBeGrabbed(s.gametype, &es, ps_ptr) };

                // C: same scenario marshalled into the verbatim-body oracle.
                let c = unsafe {
                    jka_bgitem_can_item_be_grabbed(
                        s.gametype,
                        mi,
                        s.has_ps as c_int,
                        s.trueJedi,
                        s.trueNonJedi,
                        s.isJediMaster,
                        s.duelInProgress,
                        s.clientNum,
                        s.forcePowersActive,
                        s.generic1,
                        s.es_powerups,
                        s.eFlags,
                        s.modelindex2,
                        s.stats.as_ptr(),
                        s.persistant.as_ptr(),
                        s.ammo.as_ptr(),
                        s.powerups.as_ptr(),
                    )
                };

                assert_eq!(rust, c, "modelindex={mi}, scenario gametype={}", s.gametype);
            }
        }
    }

    /// Logic parity for `BG_AddPredictableEventToPlayerstate`: drive one call on a
    /// zeroed `playerState_t` (with the sequence pre-set) and compare the resulting
    /// event ring (`eventSequence` + both `events`/`eventParms` slots) against the
    /// verbatim C body. Sweeps several initial `eventSequence` values — crucially the
    /// `MAX_PS_EVENTS-1` wrap (0,1,2,3 all index slot `seq & 1`) — and distinct
    /// event/parm payloads.
    #[test]
    fn bg_add_predictable_event_matches_oracle() {
        for &seq in &[0i32, 1, 2, 3, 7, 100] {
            for &(ev, parm) in &[(0i32, 0i32), (5, 11), (255, -1), (42, 99)] {
                // Rust: run on a real zeroed playerState_t with the seq pre-set.
                let mut ps: playerState_t = unsafe { core::mem::zeroed() };
                ps.eventSequence = seq;
                unsafe { BG_AddPredictableEventToPlayerstate(ev, parm, &mut ps) };

                // C: same starting state through the verbatim oracle body.
                let mut c_seq = seq;
                let mut c_events = [0i32; MAX_PS_EVENTS];
                let mut c_parms = [0i32; MAX_PS_EVENTS];
                unsafe {
                    jka_bg_add_pred_event(
                        ev,
                        parm,
                        &mut c_seq,
                        c_events.as_mut_ptr(),
                        c_parms.as_mut_ptr(),
                    );
                }

                assert_eq!(ps.eventSequence, c_seq, "eventSequence (seq={seq})");
                assert_eq!(ps.events, c_events, "events (seq={seq})");
                assert_eq!(ps.eventParms, c_parms, "eventParms (seq={seq})");
            }
        }
    }

    /// Parity for `BG_HasYsalamiri`: sweep the gametype (incl. the `GT_CTY` flag-carry
    /// branch and a non-CTY type) against every combination of the three powerup slots
    /// it reads. Compares the Rust port (on a real zeroed `playerState_t`) against the
    /// verbatim C body.
    #[test]
    fn bg_has_ysalamiri_matches_oracle() {
        for &gt in &[0i32, GT_CTY, 6] {
            for &pw_red in &[0i32, 1] {
                for &pw_blue in &[0i32, 1] {
                    for &pw_ysa in &[0i32, 1] {
                        let mut ps: playerState_t = unsafe { core::mem::zeroed() };
                        ps.powerups[PW_REDFLAG as usize] = pw_red;
                        ps.powerups[PW_BLUEFLAG as usize] = pw_blue;
                        ps.powerups[PW_YSALAMIRI as usize] = pw_ysa;
                        let got = unsafe { BG_HasYsalamiri(gt, &mut ps) };
                        let want = unsafe {
                            crate::oracle::jka_bg_has_ysalamiri(gt, pw_red, pw_blue, pw_ysa)
                        };
                        assert_eq!(
                            got, want,
                            "gt={gt} red={pw_red} blue={pw_blue} ysa={pw_ysa}"
                        );
                    }
                }
            }
        }
    }

    /// Parity for `BG_CanUseFPNow`: an exhaustive-ish nested sweep that exercises every
    /// early-return branch and their interactions — ysalamiri (via gametype + powerups),
    /// force-restriction / true-non-jedi, emplaced gun, vehicle, the duel exceptions
    /// (incl. push-during-saberlock), the saberlock `> time` gate, falling-to-death, and
    /// the broken-arm power switch. The Rust port runs on a real zeroed `playerState_t`;
    /// the C body runs over the int-marshalled minimal struct.
    #[test]
    fn bg_can_use_fp_now_matches_oracle() {
        use crate::codemp::game::bg_weapons_h::WP_SABER;
        let rarm = 1 << BROKENLIMB_RARM;
        let larm = 1 << BROKENLIMB_LARM;
        for &gt in &[0i32, GT_CTY] {
            for &(pw_red, pw_blue, pw_ysa) in &[(0i32, 0i32, 0i32), (1, 0, 0), (0, 0, 1)] {
                for &power in &[
                    FP_LEVITATION,
                    FP_PUSH,
                    FP_PULL,
                    FP_GRIP,
                    FP_DRAIN,
                    FP_SABER_OFFENSE,
                ] {
                    for &forceRestricted in &[0i32, 1] {
                        for &trueNonJedi in &[0i32, 1] {
                            for &weapon in &[WP_SABER, WP_EMPLACED_GUN] {
                                for &m_iVehicleNum in &[0i32, 1] {
                                    for &duelInProgress in &[0i32, 1] {
                                        for &saberLockFrame in &[0i32, 1] {
                                            for &(saberLockTime, time) in &[(0i32, 0i32), (100, 50)]
                                            {
                                                for &fallingToDeath in &[0i32, 1] {
                                                    for &brokenLimbs in
                                                        &[0i32, rarm, larm, rarm | larm]
                                                    {
                                                        let mut ps: playerState_t =
                                                            unsafe { core::mem::zeroed() };
                                                        ps.powerups[PW_REDFLAG as usize] = pw_red;
                                                        ps.powerups[PW_BLUEFLAG as usize] = pw_blue;
                                                        ps.powerups[PW_YSALAMIRI as usize] = pw_ysa;
                                                        ps.forceRestricted = forceRestricted;
                                                        ps.trueNonJedi = trueNonJedi;
                                                        ps.weapon = weapon;
                                                        ps.m_iVehicleNum = m_iVehicleNum;
                                                        ps.duelInProgress = duelInProgress;
                                                        ps.saberLockFrame = saberLockFrame;
                                                        ps.saberLockTime = saberLockTime;
                                                        ps.fallingToDeath = fallingToDeath;
                                                        ps.brokenLimbs = brokenLimbs;
                                                        let got = unsafe {
                                                            BG_CanUseFPNow(gt, &mut ps, time, power)
                                                        };
                                                        let want = unsafe {
                                                            crate::oracle::jka_bg_can_use_fp_now(
                                                                gt,
                                                                time,
                                                                power,
                                                                pw_red,
                                                                pw_blue,
                                                                pw_ysa,
                                                                forceRestricted,
                                                                trueNonJedi,
                                                                weapon,
                                                                m_iVehicleNum,
                                                                duelInProgress,
                                                                saberLockFrame,
                                                                saberLockTime,
                                                                fallingToDeath,
                                                                brokenLimbs,
                                                            )
                                                        };
                                                        assert_eq!(
                                                            got, want,
                                                            "gt={gt} power={power} fr={forceRestricted} tnj={trueNonJedi} wpn={weapon} veh={m_iVehicleNum} duel={duelInProgress} slf={saberLockFrame} slt={saberLockTime} t={time} ftd={fallingToDeath} bl={brokenLimbs}"
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Bit-exact parity for `vectoyaw`: sweep direction vectors that exercise every
    /// branch — the (0,0) early-out, all four quadrants of the `atan2` path (including
    /// negative-yaw `+360` wrap), and the `vec[PITCH]==0` axis cases (`YAW>0`→90,
    /// `YAW<0`→270). `vec[2]` is irrelevant to the result but swept to confirm so.
    #[test]
    fn vectoyaw_matches_oracle() {
        let coords = [-128.0f32, -1.0, -0.0, 0.0, 1.0, 64.0, 128.0];
        for &x in &coords {
            for &y in &coords {
                for &z in &[0.0f32, 50.0] {
                    let v: vec3_t = [x, y, z];
                    let got = vectoyaw(&v);
                    let want = unsafe { jka_bg_vectoyaw(x, y, z) };
                    assert_eq!(
                        got.to_bits(),
                        want.to_bits(),
                        "vectoyaw([{x},{y},{z}]) = {got} (rust) vs {want} (c)"
                    );
                }
            }
        }
    }

    /// Bit-exact parity for the [`BG_EvaluateTrajectory`] / [`BG_EvaluateTrajectoryDelta`]
    /// pair. Sweeps every `trType` (0..=6, plus the verbatim-`break` default), several
    /// `(trTime, trDuration, atTime)` triples chosen to straddle each branch boundary —
    /// `atTime` before/at/after `trTime + trDuration` (the `_STOP` clamps), the
    /// `atTime - trTime <= 0` / `> trDuration` NONLINEAR early-outs, and a zero/negative
    /// `deltaTime` — and a couple of `trBase`/`trDelta` vectors. Each component is
    /// compared by raw bits, so any `f32`-vs-`f64` slip in the `0.001`/`0.001f`/`0.5`/
    /// `sin`/`cos`/gravity arithmetic surfaces immediately.
    #[test]
    fn bg_evaluate_trajectory_matches_oracle() {
        let base: vec3_t = [10.0, -20.0, 30.0];
        let delta: vec3_t = [3.0, -4.0, 5.0];
        // Only the 7 real trTypes (0..=6); the default arm is a diverging `Com_Error`
        // (engine syscall) and so isn't oracle-testable, like the other error paths.
        let trtypes = [0, 1, 2, 3, 4, 5, 6];
        let trtimes = [0, 1000, -500];
        let durations = [1, 250, 1000, 4000];
        let attimes = [-500, 0, 1, 125, 250, 999, 1000, 1001, 5000];

        for &trType in &trtypes {
            for &trTime in &trtimes {
                for &trDuration in &durations {
                    for &atTime in &attimes {
                        let tr = trajectory_t {
                            trType,
                            trTime,
                            trDuration,
                            trBase: base,
                            trDelta: delta,
                        };

                        let mut got: vec3_t = [111.0, 222.0, 333.0];
                        BG_EvaluateTrajectory(&tr, atTime, &mut got);
                        let mut want: vec3_t = [111.0, 222.0, 333.0];
                        unsafe {
                            jka_bg_eval_traj(
                                trType,
                                trTime,
                                trDuration,
                                base[0],
                                base[1],
                                base[2],
                                delta[0],
                                delta[1],
                                delta[2],
                                atTime,
                                want.as_mut_ptr(),
                            );
                        }
                        for k in 0..3 {
                            assert_eq!(
                                got[k].to_bits(),
                                want[k].to_bits(),
                                "BG_EvaluateTrajectory trType={trType} trTime={trTime} \
                                 trDuration={trDuration} atTime={atTime} [{k}]: {} (rust) vs {} (c)",
                                got[k],
                                want[k]
                            );
                        }

                        let mut gotd: vec3_t = [111.0, 222.0, 333.0];
                        BG_EvaluateTrajectoryDelta(&tr, atTime, &mut gotd);
                        let mut wantd: vec3_t = [111.0, 222.0, 333.0];
                        unsafe {
                            jka_bg_eval_traj_delta(
                                trType,
                                trTime,
                                trDuration,
                                base[0],
                                base[1],
                                base[2],
                                delta[0],
                                delta[1],
                                delta[2],
                                atTime,
                                wantd.as_mut_ptr(),
                            );
                        }
                        for k in 0..3 {
                            assert_eq!(
                                gotd[k].to_bits(),
                                wantd[k].to_bits(),
                                "BG_EvaluateTrajectoryDelta trType={trType} trTime={trTime} \
                                 trDuration={trDuration} atTime={atTime} [{k}]: {} (rust) vs {} (c)",
                                gotd[k],
                                wantd[k]
                            );
                        }
                    }
                }
            }
        }
    }

    /// Parity for `BG_GiveMeVectorFromMatrix`: the ghoul2 bolt-axis selector. Drives a
    /// populated 3x4 `mdxaBone_t` matrix (12 distinct floats incl. negatives) through
    /// every one of the 7 axis flags plus an unmatched flag (the C switch has no default,
    /// so the unmatched path must leave `vec` untouched — checked by pre-seeding it). The
    /// flag constants and `mdxaBone_t` are imported in this fn's own scope.
    #[test]
    fn bg_give_me_vector_from_matrix_matches_oracle() {
        use crate::codemp::game::q_shared_h::{
            mdxaBone_t, NEGATIVE_X, NEGATIVE_Y, NEGATIVE_Z, ORIGIN, POSITIVE_X, POSITIVE_Y,
            POSITIVE_Z,
        };
        use crate::oracle::jka_bg_give_me_vector_from_matrix;

        // 12 distinct, sign-varied floats laid out row-major (matrix[3][4]).
        let m12: [f32; 12] = [
            1.5, -2.25, 3.75, -4.5, 5.0, -6.125, 7.5, -8.0, 9.25, -10.5, 11.0, -12.75,
        ];
        let mut bone = mdxaBone_t::default();
        for r in 0..3 {
            for c in 0..4 {
                bone.matrix[r][c] = m12[r * 4 + c];
            }
        }

        // The 7 real flags plus an out-of-range flag (untouched path).
        let flags = [
            ORIGIN, POSITIVE_X, POSITIVE_Y, POSITIVE_Z, NEGATIVE_X, NEGATIVE_Y, NEGATIVE_Z, 99,
        ];

        for &flag in &flags {
            let mut got: vec3_t = [111.0, 222.0, 333.0];
            BG_GiveMeVectorFromMatrix(&bone, flag, &mut got);
            let mut want: vec3_t = [111.0, 222.0, 333.0];
            unsafe {
                jka_bg_give_me_vector_from_matrix(m12.as_ptr(), flag, want.as_mut_ptr());
            }
            for k in 0..3 {
                assert_eq!(
                    got[k].to_bits(),
                    want[k].to_bits(),
                    "BG_GiveMeVectorFromMatrix flag={flag} [{k}]: {} (rust) vs {} (c)",
                    got[k],
                    want[k]
                );
            }
        }
    }

    /// Parity for `BG_PlayerTouchesItem`: the pickup-proximity AABB test. Sweeps a
    /// few `trType`s/atTimes through the embedded `BG_EvaluateTrajectory` and, crucially,
    /// per-axis offsets chosen to straddle each asymmetric bound (`+44`/`-50` on x, `±36`
    /// on y/z) — exactly on, just inside, and just outside — so a flipped comparison or a
    /// swapped constant surfaces. Rust runs on real zeroed `playerState_t`/`entityState_t`;
    /// the C oracle marshals the same origin + pos trajectory.
    #[test]
    fn bg_player_touches_item_matches_oracle() {
        let player: vec3_t = [100.0, 200.0, 300.0];
        // offsets added to the player origin to place the item base on each axis,
        // straddling the bounds (item = player - diff, so diff = -offset hits the test).
        let offsets = [
            -51.0f32, -50.0, -49.0, -45.0, -1.0, 0.0, 1.0, 35.0, 36.0, 37.0, 44.0, 45.0,
        ];
        let trtypes = [TR_STATIONARY, TR_LINEAR, TR_GRAVITY];
        let attimes = [0, 250, 1000];

        for &trType in &trtypes {
            for &atTime in &attimes {
                for &ox in &offsets {
                    for &oy in &offsets {
                        for &oz in &offsets {
                            let base: vec3_t = [player[0] + ox, player[1] + oy, player[2] + oz];
                            // small delta so TR_LINEAR/TR_GRAVITY actually move the item
                            let delta: vec3_t = [2.0, -3.0, 4.0];

                            let mut ps: playerState_t = unsafe { core::mem::zeroed() };
                            ps.origin = player;
                            let mut item: entityState_t = unsafe { core::mem::zeroed() };
                            item.pos = trajectory_t {
                                trType,
                                trTime: 0,
                                trDuration: 1000,
                                trBase: base,
                                trDelta: delta,
                            };

                            let got = BG_PlayerTouchesItem(&ps, &item, atTime);
                            let want = unsafe {
                                jka_bg_player_touches_item(
                                    player[0], player[1], player[2], trType, 0, 1000, base[0],
                                    base[1], base[2], delta[0], delta[1], delta[2], atTime,
                                )
                            };
                            assert_eq!(
                                got as c_int, want,
                                "trType={trType} atTime={atTime} off=({ox},{oy},{oz})"
                            );
                        }
                    }
                }
            }
        }
    }

    /// Logic parity for `BG_ProperForceIndex`: sweep `power` over the full `FP_*` range
    /// plus out-of-range / sentinel values (`-2..=20`), comparing the returned sorted-table
    /// slot (or `-1`) against the verbatim C body. Confirms the `forcePowerSorted` table
    /// was transcribed in the right order.
    #[test]
    fn bg_proper_force_index_matches_oracle() {
        use crate::oracle::jka_bg_proper_force_index;
        for power in -2..=20 {
            let got = BG_ProperForceIndex(power);
            let want = unsafe { jka_bg_proper_force_index(power) };
            assert_eq!(got, want, "BG_ProperForceIndex({power})");
        }
    }

    /// Logic parity for `BG_CycleForce`: drive both directions (next / previous / the
    /// `else`-is-previous `0`) over a spread of `forcePowersKnown` bitmasks and every
    /// `forcePowerSelected` in `-1..=NUM_FORCE_POWERS`, comparing the resulting selection.
    /// The masks deliberately include the empty set, the full set, only-skipped powers
    /// (levitation + saber offense/defense/throw → must find nothing), and mixes — and the
    /// `-1`/`==NUM_FORCE_POWERS` selections exercise the early-out (incl. the carried
    /// `!known & (1<<x)` bug, whose `1 << -1` masks to `1 << 31` identically on both sides).
    #[test]
    fn bg_cycle_force_matches_oracle() {
        use crate::oracle::jka_bg_cycle_force;
        let masks = [
            0i32,
            !0,
            (1 << FP_TELEPATHY) | (1 << FP_HEAL),
            1 << FP_LEVITATION,
            (1 << FP_LEVITATION)
                | (1 << FP_SABER_OFFENSE)
                | (1 << FP_SABER_DEFENSE)
                | (1 << FP_SABERTHROW),
            (1 << FP_PUSH) | (1 << FP_PULL) | (1 << FP_LIGHTNING),
            (1 << FP_HEAL) | (1 << FP_DRAIN) | (1 << FP_GRIP) | (1 << FP_RAGE),
        ];
        for &known in &masks {
            for selected in -1..=NUM_FORCE_POWERS as c_int {
                for &direction in &[1i32, -1, 0] {
                    let mut ps: playerState_t = unsafe { core::mem::zeroed() };
                    ps.fd.forcePowersKnown = known;
                    ps.fd.forcePowerSelected = selected;
                    unsafe { BG_CycleForce(&mut ps, direction) };
                    let got = ps.fd.forcePowerSelected;

                    let want = unsafe { jka_bg_cycle_force(known, selected, direction) };
                    assert_eq!(
                        got, want,
                        "BG_CycleForce(known={known:#x}, selected={selected}, dir={direction})"
                    );
                }
            }
        }
    }

    /// Element-wise parity of the whole `bg_itemlist` table against the authentic C
    /// (verbatim copy in `oracle/bg_misc_items_oracle.c`). `gitem_t` carries raw
    /// `char *` (no `PartialEq`), so each row is compared field-by-field; pointer
    /// fields are matched by NULL-ness and then by `CStr` contents (the strings live at
    /// different addresses). Covers the index-0 sentinel and the `{NULL}` terminator —
    /// and so catches the carried `item_shield_sm_instant` missing-comma quirk
    /// (`description == NULL`) directly.
    #[test]
    fn bg_itemlist_matches_c() {
        use crate::oracle::{jka_bgitem_itemlist_ptr, jka_bgitem_numItems};
        unsafe {
            assert_eq!(bg_numItems, jka_bgitem_numItems(), "bg_numItems");

            // full array length = bg_numItems + 1 (the {NULL} terminator)
            let n = (jka_bgitem_numItems() + 1) as usize;
            let rust = addr_of!(bg_itemlist) as *const gitem_t;
            let c = jka_bgitem_itemlist_ptr();

            // NULL-aware string-pointer compare.
            let cmp = |r: *const c_char, cc: *const c_char, what: &str, i: usize| {
                if r.is_null() || cc.is_null() {
                    assert!(
                        r.is_null() && cc.is_null(),
                        "bg_itemlist[{i}].{what}: null mismatch (rust_null={}, c_null={})",
                        r.is_null(),
                        cc.is_null()
                    );
                } else {
                    assert_eq!(
                        CStr::from_ptr(r),
                        CStr::from_ptr(cc),
                        "bg_itemlist[{i}].{what}"
                    );
                }
            };

            for i in 0..n {
                let r = &*rust.add(i);
                let cc = &*c.add(i);
                cmp(r.classname, cc.classname, "classname", i);
                cmp(r.pickup_sound, cc.pickup_sound, "pickup_sound", i);
                for m in 0..MAX_ITEM_MODELS {
                    cmp(r.world_model[m], cc.world_model[m], "world_model", i);
                }
                cmp(r.view_model, cc.view_model, "view_model", i);
                cmp(r.icon, cc.icon, "icon", i);
                assert_eq!(r.quantity, cc.quantity, "bg_itemlist[{i}].quantity");
                assert_eq!(r.giType, cc.giType, "bg_itemlist[{i}].giType");
                assert_eq!(r.giTag, cc.giTag, "bg_itemlist[{i}].giTag");
                cmp(r.precaches, cc.precaches, "precaches", i);
                cmp(r.sounds, cc.sounds, "sounds", i);
                cmp(r.description, cc.description, "description", i);
            }
        }
    }

    /// Logic parity for the three item selectors against verbatim C bodies.
    /// `BG_GetItemIndexByTag` is swept over a wide `(tag, type)` grid;
    /// `BG_IsItemSelectable` over the full `HI_*` range (plus out-of-range); and
    /// `BG_CycleInven` driven both directions over every *holdable* selected index and a
    /// spread of owned-item bitmasks. The CycleInven sweep stays on holdable indices
    /// (`giTag` 1..11) because that is the only domain real play produces — entries with
    /// `giTag <= 0` (the sentinel / `ammo_all`) would drive the C `1 << i` to a negative
    /// shift (UB in C, a debug panic in Rust); `STAT_HOLDABLE_ITEM` never indexes those.
    #[test]
    fn bg_item_selectors_match_c() {
        use crate::oracle::{
            jka_bgitem_CycleInven, jka_bgitem_GetItemIndexByTag, jka_bgitem_IsItemSelectable,
        };
        unsafe {
            for tag in -2..20 {
                for ty in 0..=8 {
                    assert_eq!(
                        BG_GetItemIndexByTag(tag, ty),
                        jka_bgitem_GetItemIndexByTag(tag, ty),
                        "BG_GetItemIndexByTag(tag={tag}, type={ty})"
                    );
                }
            }

            for item in -1..=HI_NUM_HOLDABLE + 1 {
                assert_eq!(
                    BG_IsItemSelectable(core::ptr::null_mut(), item),
                    jka_bgitem_IsItemSelectable(item),
                    "BG_IsItemSelectable(item={item})"
                );
            }

            // Holdable table indices (giTag 1..11): item_seeker(4) .. item_cloak(14).
            for hold_item in 4..=14 {
                for &hold_items in &[0i32, 0x2, 0xE, 0xFE, 0x1FE, 0xAAA, !0] {
                    for &direction in &[1i32, -1, 0] {
                        let mut ps: playerState_t = core::mem::zeroed();
                        ps.stats[STAT_HOLDABLE_ITEM as usize] = hold_item;
                        ps.stats[STAT_HOLDABLE_ITEMS as usize] = hold_items;
                        BG_CycleInven(&mut ps, direction);
                        let r_item = ps.stats[STAT_HOLDABLE_ITEM as usize];
                        let r_items = ps.stats[STAT_HOLDABLE_ITEMS as usize];

                        let mut c_item = hold_item;
                        let mut c_items = hold_items;
                        jka_bgitem_CycleInven(direction, &mut c_item, &mut c_items);

                        assert_eq!(
                            r_item, c_item,
                            "CycleInven dir={direction} item={hold_item} items={hold_items:#x}: STAT_HOLDABLE_ITEM"
                        );
                        assert_eq!(
                            r_items, c_items,
                            "CycleInven dir={direction} item={hold_item} items={hold_items:#x}: STAT_HOLDABLE_ITEMS"
                        );
                    }
                }
            }
        }
    }

    /// Logic parity for the `BG_FindItem*` family against verbatim C bodies, comparing
    /// the returned `bg_itemlist` index (pointers can't cross between the two table
    /// copies). `FindItemForPowerup`/`FindItem` return NULL on a miss (swept widely);
    /// the Holdable/Weapon/Ammo variants `Com_Error` on a miss, so they are driven only
    /// over tags that exist in the table.
    #[test]
    fn bg_item_finders_match_c() {
        use crate::oracle::{
            jka_bgitem_FindItem, jka_bgitem_FindItemForAmmo, jka_bgitem_FindItemForHoldable,
            jka_bgitem_FindItemForPowerup, jka_bgitem_FindItemForWeapon,
        };
        let base = addr_of!(bg_itemlist) as *const gitem_t;
        let idx_of = |p: *mut gitem_t| -> c_int {
            if p.is_null() {
                -1
            } else {
                unsafe { p.offset_from(base) as c_int }
            }
        };
        unsafe {
            // FindItemForPowerup — NULL on miss, safe to sweep widely.
            for pw in -1..20 {
                assert_eq!(
                    idx_of(BG_FindItemForPowerup(pw)),
                    jka_bgitem_FindItemForPowerup(pw),
                    "BG_FindItemForPowerup({pw})"
                );
            }
            // FindItemForHoldable — HI_SEEKER..HI_CLOAK (1..=11) all present.
            for pw in 1..HI_NUM_HOLDABLE {
                assert_eq!(
                    idx_of(BG_FindItemForHoldable(pw)),
                    jka_bgitem_FindItemForHoldable(pw),
                    "BG_FindItemForHoldable({pw})"
                );
            }
            // FindItemForWeapon — WP_STUN_BATON..=WP_TURRET all present.
            for w in WP_STUN_BATON..=WP_TURRET {
                assert_eq!(
                    idx_of(BG_FindItemForWeapon(w)),
                    jka_bgitem_FindItemForWeapon(w),
                    "BG_FindItemForWeapon({w})"
                );
            }
            // FindItemForAmmo — only the ammo tags actually in the table (AMMO_EMPLACED
            // has no entry → would Com_Error).
            for &a in &[
                AMMO_FORCE,
                AMMO_BLASTER,
                AMMO_POWERCELL,
                AMMO_METAL_BOLTS,
                AMMO_ROCKETS,
                AMMO_THERMAL,
                AMMO_TRIPMINE,
                AMMO_DETPACK,
            ] {
                assert_eq!(
                    idx_of(BG_FindItemForAmmo(a)),
                    jka_bgitem_FindItemForAmmo(a),
                    "BG_FindItemForAmmo({a})"
                );
            }
            // FindItem by classname — exact hit, case-insensitive hit, and a miss.
            for &name in &[
                c"weapon_blaster",
                c"WEAPON_SABER",
                c"item_seeker",
                c"ammo_all",
                c"does_not_exist",
            ] {
                assert_eq!(
                    idx_of(BG_FindItem(name.as_ptr())),
                    jka_bgitem_FindItem(name.as_ptr()),
                    "BG_FindItem({name:?})"
                );
            }
        }
    }

    /// Bit-exact parity for `BG_EmplacedView`. Sweeps `baseAngles[YAW]` × `angles[YAW]`
    /// across a grid that straddles the `AngleSubtract` ±180 wrap (negative, zero, the
    /// ±180 boundary, and >360 overflow) and `constraint` across `0` up to a half-turn —
    /// hitting the in-range `0` return, both clamp sides, and the `amt` ±1 split between
    /// the `1` and `2` codes. Both the int return AND the `*newYaw` out-param are compared
    /// by raw bits (the out-param is pre-seeded to a sentinel so the untouched `0`-return
    /// path is asserted to leave it unwritten, matching the C body).
    #[test]
    fn bg_emplaced_view_matches_oracle() {
        let yaws = [
            -360.0f32, -270.0, -181.0, -180.0, -90.0, -45.5, -1.0, -0.0, 0.0, 1.0, 45.5, 90.0,
            179.0, 180.0, 181.0, 270.0, 360.0,
        ];
        let constraints = [0.0f32, 0.5, 1.0, 1.5, 5.0, 30.0, 90.0, 180.0];
        const SENTINEL: f32 = -123456.0;
        for &base in &yaws {
            for &ang in &yaws {
                for &c in &constraints {
                    let bb: vec3_t = [0.0, base, 0.0];
                    let aa: vec3_t = [0.0, ang, 0.0];

                    let mut got_yaw = SENTINEL;
                    let got = BG_EmplacedView(&bb, &aa, &mut got_yaw, c);

                    let mut want_yaw = SENTINEL;
                    let want = unsafe { jka_bg_emplaced_view(base, ang, c, &mut want_yaw) };

                    assert_eq!(
                        got, want,
                        "BG_EmplacedView ret (base={base}, ang={ang}, c={c})"
                    );
                    assert_eq!(
                        got_yaw.to_bits(),
                        want_yaw.to_bits(),
                        "BG_EmplacedView newYaw (base={base}, ang={ang}, c={c})"
                    );
                }
            }
        }
    }

    /// Parity for `BG_TouchJumpPad`. Sweeps `pm_type` across the three pad-eligible types
    /// and several spectator/dead values (the early-return), `jumppad_ent` vs the pad's
    /// `number` (equal → skips the vestigial first-touch block, unequal → runs it),
    /// `pmove_framecount`, and a spread of `origin2` vectors (driving the dead
    /// `vectoangles`/pitch path identically on both sides). The three observable mutations —
    /// `jumppad_ent`, `jumppad_frame`, and the bit-exact `velocity` copy — are compared
    /// against the verbatim C body (run on a `jumppad_frame`-seeded-0 minimal struct,
    /// matching the freshly zeroed Rust `playerState_t`).
    #[test]
    fn bg_touch_jump_pad_matches_oracle() {
        let origins: &[vec3_t] = &[
            [0.0, 0.0, 0.0],
            [100.0, 0.0, 0.0],
            [0.0, 0.0, 300.0],
            [50.0, -60.0, 70.0],
            [-200.0, 150.0, -90.0],
        ];
        for &pm_type in &[0i32, 1, 2, 3, 4, 5] {
            for &in_ent in &[-1i32, 0, 5, 7] {
                for &number in &[5i32, 7] {
                    for &framecount in &[0i32, 1, 42, -3] {
                        for &o2 in origins {
                            let mut ps: playerState_t = unsafe { core::mem::zeroed() };
                            ps.pm_type = pm_type;
                            ps.jumppad_ent = in_ent;
                            ps.jumppad_frame = 0;
                            ps.pmove_framecount = framecount;
                            let mut es: entityState_t = unsafe { core::mem::zeroed() };
                            es.number = number;
                            es.origin2 = o2;

                            BG_TouchJumpPad(&mut ps, &es);

                            let mut c_ent = 0;
                            let mut c_frame = 0;
                            let mut c_vel = [0.0f32; 3];
                            unsafe {
                                jka_bg_touch_jump_pad(
                                    pm_type,
                                    in_ent,
                                    number,
                                    framecount,
                                    o2[0],
                                    o2[1],
                                    o2[2],
                                    &mut c_ent,
                                    &mut c_frame,
                                    c_vel.as_mut_ptr(),
                                );
                            }
                            assert_eq!(
                                ps.jumppad_ent, c_ent,
                                "jumppad_ent pm={pm_type} ent={in_ent} num={number}"
                            );
                            assert_eq!(
                                ps.jumppad_frame, c_frame,
                                "jumppad_frame pm={pm_type} fc={framecount}"
                            );
                            for k in 0..3 {
                                assert_eq!(
                                    ps.velocity[k].to_bits(),
                                    c_vel[k].to_bits(),
                                    "velocity[{k}] pm={pm_type} o2={o2:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Parity for `BG_IsValidCharacterModel`: the SP-only "kyle" `fpls*` skin block. Sweeps
    /// model names that hit and miss the (case-insensitive) "kyle" gate and skins covering
    /// the three rejected `fpls`/`fpls2`/`fpls3` values, a case variant, and several
    /// allowed names, comparing the qboolean against the verbatim C body.
    #[test]
    fn bg_is_valid_character_model_matches_oracle() {
        let models = ["kyle", "KYLE", "Kyle", "jan", "", "kyleX"];
        let skins = [
            "fpls", "fpls2", "fpls3", "FPLS", "default", "fpls4", "", "red", "menu", "MENU",
        ];
        for m in &models {
            for s in &skins {
                let cm = std::ffi::CString::new(*m).unwrap();
                let cs = std::ffi::CString::new(*s).unwrap();
                let got = unsafe { BG_IsValidCharacterModel(cm.as_ptr(), cs.as_ptr()) };
                let want = unsafe { jka_bg_is_valid_character_model(cm.as_ptr(), cs.as_ptr()) };
                assert_eq!(got as c_int, want, "model={m:?} skin={s:?}");
            }
        }
    }

    /// Parity for `BG_PlayerStateToEntityState`: the ps→es field collapse. A "rich" base
    /// `playerState_t` is built with a DISTINCT sentinel in every copied field (so any
    /// mis-wired field-to-field mapping — Rust copying A where the oracle reads B — shows up
    /// as a divergence), then run through `snap` ∈ {off, on} × a set of mode overrides that
    /// drive each branch: the `eType` gate (intermission / spectator / gibbed / alive), the
    /// seeker-drone and EF_DEAD `eFlags` bits, the duel `bolt1`, the external-event latch,
    /// and the sequenced-event path (in-window, the catch-up clamp, and the no-event case).
    /// Every written `entityState_t` field (incl. the snapped `pos`/`apos` trajectories and
    /// the powerup bitmask) plus the one mutated `ps.entityEventSequence` is compared
    /// bit-exact against the verbatim C body (both sides run over zeroed structs).
    #[test]
    fn bg_player_state_to_entity_state_matches_oracle() {
        // A base ps with a unique value in every field the function reads. Fractional
        // origin/velocity/viewangles carry `.5` ties so the inline SnapVector's
        // round-to-nearest-ties-even is exercised (and `velocity`→trDelta is left unsnapped).
        let base = || -> playerState_t {
            let mut ps: playerState_t = unsafe { core::mem::zeroed() };
            ps.pm_type = PM_NORMAL;
            ps.stats[STAT_HEALTH as usize] = 100;
            ps.clientNum = 11;
            ps.origin = [10.4, 20.6, 30.5];
            ps.velocity = [1.25, -2.75, 3.5];
            ps.viewangles = [45.5, 90.5, 135.5];
            ps.fd.forceMindtrickTargetIndex = 21;
            ps.fd.forceMindtrickTargetIndex2 = 22;
            ps.fd.forceMindtrickTargetIndex3 = 23;
            ps.fd.forceMindtrickTargetIndex4 = 24;
            ps.fd.forcePowersActive = 121;
            ps.fd.saberAnimLevel = 211;
            ps.saberLockFrame = 31;
            ps.electrifyTime = 41;
            ps.speed = 51.5;
            ps.genericEnemyIndex = 61;
            ps.activeForcePass = 71;
            ps.movementDir = 5;
            ps.legsAnim = 81;
            ps.torsoAnim = 91;
            ps.legsFlip = QTRUE;
            ps.torsoFlip = QFALSE;
            ps.eFlags = EF_DEAD | 0x100;
            ps.eFlags2 = 0x55;
            ps.saberInFlight = QTRUE;
            ps.saberEntityNum = 101;
            ps.saberMove = 111;
            ps.duelInProgress = QTRUE;
            ps.emplacedIndex = 131;
            ps.saberHolstered = 2;
            ps.externalEvent = 0;
            ps.externalEventParm = 141;
            ps.entityEventSequence = 9;
            ps.eventSequence = 11;
            ps.events = [1001, 1002];
            ps.eventParms = [2001, 2002];
            ps.weapon = 151;
            ps.groundEntityNum = 161;
            ps.powerups[0] = 5;
            ps.powerups[3] = 7;
            ps.powerups[15] = 9;
            ps.loopSound = 171;
            ps.generic1 = 181;
            ps.weaponstate = 191;
            ps.weaponChargeTime = 201;
            ps.lastHitLoc = [1.5, 2.5, 3.5];
            ps.heldByClient = 221;
            ps.ragAttach = 231;
            ps.iModelScale = 241;
            ps.brokenLimbs = 251;
            ps.hasLookTarget = QTRUE;
            ps.lookTarget = 261;
            ps.customRGBA = [12, 34, 56, 78];
            ps.m_iVehicleNum = 271;
            ps.isJediMaster = QTRUE;
            ps.holocronBits = 0x2a5;
            ps
        };

        // (name, mutator) — each drives a different branch off the rich base.
        let modes: &[(&str, fn(&mut playerState_t))] = &[
            ("alive", |_ps| {}),
            ("intermission", |ps| ps.pm_type = PM_INTERMISSION),
            ("spectator", |ps| ps.pm_type = PM_SPECTATOR),
            ("gibbed", |ps| ps.stats[STAT_HEALTH as usize] = -50),
            ("dead_zero", |ps| ps.stats[STAT_HEALTH as usize] = 0),
            ("no_seeker", |ps| ps.genericEnemyIndex = -1),
            ("no_duel", |ps| ps.duelInProgress = QFALSE),
            ("external_event", |ps| ps.externalEvent = 555),
            ("seq_clamp", |ps| {
                ps.entityEventSequence = 2;
                ps.eventSequence = 20;
            }),
            ("no_event", |ps| {
                ps.entityEventSequence = 11;
                ps.eventSequence = 11;
            }),
        ];

        for &snap in &[QFALSE, QTRUE] {
            for (name, mutate) in modes {
                let mut ps = base();
                mutate(&mut ps);

                // Pack the oracle inputs from `ps` BEFORE the Rust call mutates
                // entityEventSequence (index maps documented in bg_misc_oracle.c).
                let in_i: [c_int; 45] = [
                    ps.pm_type,
                    ps.stats[STAT_HEALTH as usize],
                    ps.clientNum,
                    ps.fd.forceMindtrickTargetIndex,
                    ps.fd.forceMindtrickTargetIndex2,
                    ps.fd.forceMindtrickTargetIndex3,
                    ps.fd.forceMindtrickTargetIndex4,
                    ps.saberLockFrame,
                    ps.electrifyTime,
                    ps.genericEnemyIndex,
                    ps.activeForcePass,
                    ps.movementDir,
                    ps.legsAnim,
                    ps.torsoAnim,
                    ps.legsFlip,
                    ps.torsoFlip,
                    ps.eFlags,
                    ps.eFlags2,
                    ps.saberInFlight,
                    ps.saberEntityNum,
                    ps.saberMove,
                    ps.fd.forcePowersActive,
                    ps.duelInProgress,
                    ps.emplacedIndex,
                    ps.saberHolstered,
                    ps.externalEvent,
                    ps.externalEventParm,
                    ps.entityEventSequence,
                    ps.eventSequence,
                    ps.weapon,
                    ps.groundEntityNum,
                    ps.loopSound,
                    ps.generic1,
                    ps.weaponstate,
                    ps.weaponChargeTime,
                    ps.fd.saberAnimLevel,
                    ps.heldByClient,
                    ps.ragAttach,
                    ps.iModelScale,
                    ps.brokenLimbs,
                    ps.hasLookTarget,
                    ps.lookTarget,
                    ps.m_iVehicleNum,
                    ps.isJediMaster,
                    ps.holocronBits,
                ];
                let in_f: [f32; 12] = [
                    ps.origin[0],
                    ps.origin[1],
                    ps.origin[2],
                    ps.velocity[0],
                    ps.velocity[1],
                    ps.velocity[2],
                    ps.viewangles[0],
                    ps.viewangles[1],
                    ps.viewangles[2],
                    ps.lastHitLoc[0],
                    ps.lastHitLoc[1],
                    ps.lastHitLoc[2],
                ];
                let in_events = ps.events;
                let in_eventParms = ps.eventParms;
                let in_powerups = ps.powerups;
                let in_rgba = ps.customRGBA;
                let in_speed = ps.speed;

                let mut out_i = [0 as c_int; 45];
                let mut out_f = [0.0f32; 16];
                let mut out_rgba = [0 as c_int; 4];
                let mut out_ees = 0 as c_int;
                unsafe {
                    jka_bg_player_state_to_entity_state(
                        in_i.as_ptr(),
                        in_f.as_ptr(),
                        in_speed,
                        in_events.as_ptr(),
                        in_eventParms.as_ptr(),
                        in_powerups.as_ptr(),
                        in_rgba.as_ptr(),
                        snap,
                        out_i.as_mut_ptr(),
                        out_f.as_mut_ptr(),
                        out_rgba.as_mut_ptr(),
                        &mut out_ees,
                    );
                }

                let mut es: entityState_t = unsafe { core::mem::zeroed() };
                BG_PlayerStateToEntityState(&mut ps, &mut es, snap);

                let r_i: [c_int; 45] = [
                    es.eType,
                    es.number,
                    es.pos.trType,
                    es.apos.trType,
                    es.trickedentindex,
                    es.trickedentindex2,
                    es.trickedentindex3,
                    es.trickedentindex4,
                    es.forceFrame,
                    es.emplacedOwner,
                    es.genericenemyindex,
                    es.activeForcePass,
                    es.legsAnim,
                    es.torsoAnim,
                    es.legsFlip,
                    es.torsoFlip,
                    es.clientNum,
                    es.eFlags,
                    es.eFlags2,
                    es.saberInFlight,
                    es.saberEntityNum,
                    es.saberMove,
                    es.forcePowersActive,
                    es.bolt1,
                    es.otherEntityNum2,
                    es.saberHolstered,
                    es.event,
                    es.eventParm,
                    es.weapon,
                    es.groundEntityNum,
                    es.powerups,
                    es.loopSound,
                    es.generic1,
                    es.modelindex2,
                    es.constantLight,
                    es.time2,
                    es.fireflag,
                    es.heldByClient,
                    es.ragAttach,
                    es.iModelScale,
                    es.brokenLimbs,
                    es.hasLookTarget,
                    es.lookTarget,
                    es.m_iVehicleNum,
                    es.isJediMaster,
                ];
                for k in 0..45 {
                    assert_eq!(r_i[k], out_i[k], "out_i[{k}] mode={name} snap={snap}");
                }

                let r_f: [f32; 16] = [
                    es.pos.trBase[0],
                    es.pos.trBase[1],
                    es.pos.trBase[2],
                    es.pos.trDelta[0],
                    es.pos.trDelta[1],
                    es.pos.trDelta[2],
                    es.apos.trBase[0],
                    es.apos.trBase[1],
                    es.apos.trBase[2],
                    es.angles2[0],
                    es.angles2[1],
                    es.angles2[2],
                    es.origin2[0],
                    es.origin2[1],
                    es.origin2[2],
                    es.speed,
                ];
                for k in 0..16 {
                    assert_eq!(
                        r_f[k].to_bits(),
                        out_f[k].to_bits(),
                        "out_f[{k}] mode={name} snap={snap}"
                    );
                }

                for k in 0..4 {
                    assert_eq!(
                        es.customRGBA[k], out_rgba[k],
                        "customRGBA[{k}] mode={name} snap={snap}"
                    );
                }

                assert_eq!(
                    ps.entityEventSequence, out_ees,
                    "entityEventSequence mode={name} snap={snap}"
                );
            }
        }
    }

    /// Parity for `BG_PlayerStateToEntityStateExtraPolate`: same rich base + branch matrix
    /// as the plain-variant test, additionally swept over a set of `time` values, asserting
    /// that the position trajectory comes out as `TR_LINEAR_STOP` with `pos.trTime == time`
    /// and `pos.trDuration == 50` (the two appended `out_i[44]`/`out_i[45]` slots) while every
    /// other field, the snapped `apos` (still `TR_INTERPOLATE`), the powerup bitmask, and the
    /// mutated `ps.entityEventSequence` stay bit-identical to the verbatim C body.
    #[test]
    fn bg_player_state_to_entity_state_extrapolate_matches_oracle() {
        let base = || -> playerState_t {
            let mut ps: playerState_t = unsafe { core::mem::zeroed() };
            ps.pm_type = PM_NORMAL;
            ps.stats[STAT_HEALTH as usize] = 100;
            ps.clientNum = 11;
            ps.origin = [10.4, 20.6, 30.5];
            ps.velocity = [1.25, -2.75, 3.5];
            ps.viewangles = [45.5, 90.5, 135.5];
            ps.fd.forceMindtrickTargetIndex = 21;
            ps.fd.forceMindtrickTargetIndex2 = 22;
            ps.fd.forceMindtrickTargetIndex3 = 23;
            ps.fd.forceMindtrickTargetIndex4 = 24;
            ps.fd.forcePowersActive = 121;
            ps.fd.saberAnimLevel = 211;
            ps.saberLockFrame = 31;
            ps.electrifyTime = 41;
            ps.speed = 51.5;
            ps.genericEnemyIndex = 61;
            ps.activeForcePass = 71;
            ps.movementDir = 5;
            ps.legsAnim = 81;
            ps.torsoAnim = 91;
            ps.legsFlip = QTRUE;
            ps.torsoFlip = QFALSE;
            ps.eFlags = EF_DEAD | 0x100;
            ps.eFlags2 = 0x55;
            ps.saberInFlight = QTRUE;
            ps.saberEntityNum = 101;
            ps.saberMove = 111;
            ps.duelInProgress = QTRUE;
            ps.emplacedIndex = 131;
            ps.saberHolstered = 2;
            ps.externalEvent = 0;
            ps.externalEventParm = 141;
            ps.entityEventSequence = 9;
            ps.eventSequence = 11;
            ps.events = [1001, 1002];
            ps.eventParms = [2001, 2002];
            ps.weapon = 151;
            ps.groundEntityNum = 161;
            ps.powerups[0] = 5;
            ps.powerups[3] = 7;
            ps.powerups[15] = 9;
            ps.loopSound = 171;
            ps.generic1 = 181;
            ps.weaponstate = 191;
            ps.weaponChargeTime = 201;
            ps.lastHitLoc = [1.5, 2.5, 3.5];
            ps.heldByClient = 221;
            ps.ragAttach = 231;
            ps.iModelScale = 241;
            ps.brokenLimbs = 251;
            ps.hasLookTarget = QTRUE;
            ps.lookTarget = 261;
            ps.customRGBA = [12, 34, 56, 78];
            ps.m_iVehicleNum = 271;
            ps.isJediMaster = QTRUE;
            ps.holocronBits = 0x2a5;
            ps
        };

        let modes: &[(&str, fn(&mut playerState_t))] = &[
            ("alive", |_ps| {}),
            ("intermission", |ps| ps.pm_type = PM_INTERMISSION),
            ("spectator", |ps| ps.pm_type = PM_SPECTATOR),
            ("gibbed", |ps| ps.stats[STAT_HEALTH as usize] = -50),
            ("dead_zero", |ps| ps.stats[STAT_HEALTH as usize] = 0),
            ("no_seeker", |ps| ps.genericEnemyIndex = -1),
            ("no_duel", |ps| ps.duelInProgress = QFALSE),
            ("external_event", |ps| ps.externalEvent = 555),
            ("seq_clamp", |ps| {
                ps.entityEventSequence = 2;
                ps.eventSequence = 20;
            }),
            ("no_event", |ps| {
                ps.entityEventSequence = 11;
                ps.eventSequence = 11;
            }),
        ];

        // `time` values: zero, a typical level time, a negative, and a large stamp.
        for &time in &[0 as c_int, 1000, -50, 1_234_567] {
            for &snap in &[QFALSE, QTRUE] {
                for (name, mutate) in modes {
                    let mut ps = base();
                    mutate(&mut ps);

                    let in_i: [c_int; 45] = [
                        ps.pm_type,
                        ps.stats[STAT_HEALTH as usize],
                        ps.clientNum,
                        ps.fd.forceMindtrickTargetIndex,
                        ps.fd.forceMindtrickTargetIndex2,
                        ps.fd.forceMindtrickTargetIndex3,
                        ps.fd.forceMindtrickTargetIndex4,
                        ps.saberLockFrame,
                        ps.electrifyTime,
                        ps.genericEnemyIndex,
                        ps.activeForcePass,
                        ps.movementDir,
                        ps.legsAnim,
                        ps.torsoAnim,
                        ps.legsFlip,
                        ps.torsoFlip,
                        ps.eFlags,
                        ps.eFlags2,
                        ps.saberInFlight,
                        ps.saberEntityNum,
                        ps.saberMove,
                        ps.fd.forcePowersActive,
                        ps.duelInProgress,
                        ps.emplacedIndex,
                        ps.saberHolstered,
                        ps.externalEvent,
                        ps.externalEventParm,
                        ps.entityEventSequence,
                        ps.eventSequence,
                        ps.weapon,
                        ps.groundEntityNum,
                        ps.loopSound,
                        ps.generic1,
                        ps.weaponstate,
                        ps.weaponChargeTime,
                        ps.fd.saberAnimLevel,
                        ps.heldByClient,
                        ps.ragAttach,
                        ps.iModelScale,
                        ps.brokenLimbs,
                        ps.hasLookTarget,
                        ps.lookTarget,
                        ps.m_iVehicleNum,
                        ps.isJediMaster,
                        ps.holocronBits,
                    ];
                    let in_f: [f32; 12] = [
                        ps.origin[0],
                        ps.origin[1],
                        ps.origin[2],
                        ps.velocity[0],
                        ps.velocity[1],
                        ps.velocity[2],
                        ps.viewangles[0],
                        ps.viewangles[1],
                        ps.viewangles[2],
                        ps.lastHitLoc[0],
                        ps.lastHitLoc[1],
                        ps.lastHitLoc[2],
                    ];
                    let in_events = ps.events;
                    let in_eventParms = ps.eventParms;
                    let in_powerups = ps.powerups;
                    let in_rgba = ps.customRGBA;
                    let in_speed = ps.speed;

                    let mut out_i = [0 as c_int; 47];
                    let mut out_f = [0.0f32; 16];
                    let mut out_rgba = [0 as c_int; 4];
                    let mut out_ees = 0 as c_int;
                    unsafe {
                        jka_bg_player_state_to_entity_state_extrapolate(
                            in_i.as_ptr(),
                            in_f.as_ptr(),
                            in_speed,
                            in_events.as_ptr(),
                            in_eventParms.as_ptr(),
                            in_powerups.as_ptr(),
                            in_rgba.as_ptr(),
                            time,
                            snap,
                            out_i.as_mut_ptr(),
                            out_f.as_mut_ptr(),
                            out_rgba.as_mut_ptr(),
                            &mut out_ees,
                        );
                    }

                    let mut es: entityState_t = unsafe { core::mem::zeroed() };
                    BG_PlayerStateToEntityStateExtraPolate(&mut ps, &mut es, time, snap);

                    let r_i: [c_int; 47] = [
                        es.eType,
                        es.number,
                        es.pos.trType,
                        es.apos.trType,
                        es.trickedentindex,
                        es.trickedentindex2,
                        es.trickedentindex3,
                        es.trickedentindex4,
                        es.forceFrame,
                        es.emplacedOwner,
                        es.genericenemyindex,
                        es.activeForcePass,
                        es.legsAnim,
                        es.torsoAnim,
                        es.legsFlip,
                        es.torsoFlip,
                        es.clientNum,
                        es.eFlags,
                        es.eFlags2,
                        es.saberInFlight,
                        es.saberEntityNum,
                        es.saberMove,
                        es.forcePowersActive,
                        es.bolt1,
                        es.otherEntityNum2,
                        es.saberHolstered,
                        es.event,
                        es.eventParm,
                        es.weapon,
                        es.groundEntityNum,
                        es.powerups,
                        es.loopSound,
                        es.generic1,
                        es.modelindex2,
                        es.constantLight,
                        es.time2,
                        es.fireflag,
                        es.heldByClient,
                        es.ragAttach,
                        es.iModelScale,
                        es.brokenLimbs,
                        es.hasLookTarget,
                        es.lookTarget,
                        es.m_iVehicleNum,
                        es.pos.trTime,
                        es.pos.trDuration,
                        es.isJediMaster,
                    ];
                    for k in 0..47 {
                        assert_eq!(
                            r_i[k], out_i[k],
                            "out_i[{k}] mode={name} snap={snap} time={time}"
                        );
                    }

                    let r_f: [f32; 16] = [
                        es.pos.trBase[0],
                        es.pos.trBase[1],
                        es.pos.trBase[2],
                        es.pos.trDelta[0],
                        es.pos.trDelta[1],
                        es.pos.trDelta[2],
                        es.apos.trBase[0],
                        es.apos.trBase[1],
                        es.apos.trBase[2],
                        es.angles2[0],
                        es.angles2[1],
                        es.angles2[2],
                        es.origin2[0],
                        es.origin2[1],
                        es.origin2[2],
                        es.speed,
                    ];
                    for k in 0..16 {
                        assert_eq!(
                            r_f[k].to_bits(),
                            out_f[k].to_bits(),
                            "out_f[{k}] mode={name} snap={snap} time={time}"
                        );
                    }

                    for k in 0..4 {
                        assert_eq!(
                            es.customRGBA[k], out_rgba[k],
                            "customRGBA[{k}] mode={name} snap={snap} time={time}"
                        );
                    }

                    assert_eq!(
                        ps.entityEventSequence, out_ees,
                        "entityEventSequence mode={name} snap={snap} time={time}"
                    );
                }
            }
        }
    }

    /// Parity for `BG_LegalizedForcePowers`: drive the Rust port and the verbatim C
    /// oracle over identical 128-byte config buffers across a matrix of inputs that
    /// exercises every branch — valid/invalid/wrong-team sides, ranks 0..7 (the whole
    /// `forceMasteryPoints` budget ladder), over-budget configs that trip the cost-cutting
    /// reduction loop (including the saber-attack special case and the all-powers-killed
    /// fallthrough), `freeSaber` freebies, the `gametype < GT_TEAM` team-power strip, the
    /// `fpDisabled` per-power masks (zero + force-to-3 / cap-to-1), and the over-128-char
    /// bogus-string path. Asserts both the rewritten buffer bytes and the qboolean return.
    /// Every input carries exactly `NUM_FORCE_POWERS` (18) power digits so the C never reads
    /// the uninitialized tail of `final_Powers[]` (see the fn's faithful note).
    #[test]
    fn bg_legalized_force_powers_matches_oracle() {
        // (label, config string, maxRank, freeSaber, teamForce, gametype, fpDisabled)
        let cases: &[(&str, &str, c_int, c_int, c_int, c_int, c_int)] = &[
            // light side, low rank, modest powers (all within budget at rank 0? no — rank-gated)
            ("light-r3", "3-1-300000000000000033", 3, 0, 0, 0, 0),
            ("dark-r3", "3-2-000003330000000033", 3, 0, 0, 0, 0),
            // invalid side (0/9) -> defaults to dark + invalidates
            ("badside0", "5-0-333333333333333333", 5, 0, 0, 0, 0),
            ("badside9", "5-9-333333333333333333", 5, 0, 0, 0, 0),
            // teamForce forces side regardless of config
            ("teamlight", "5-2-333333333333333333", 5, 0, 1, 0, 0),
            ("teamdark", "5-1-333333333333333333", 5, 0, 2, 0, 0),
            // every rank with a maxed config -> heavy reduction-loop exercise
            ("max-r0", "0-1-333333333333333333", 0, 0, 0, 0, 0),
            ("max-r1", "1-1-333333333333333333", 1, 0, 0, 0, 0),
            ("max-r2", "2-2-333333333333333333", 2, 0, 0, 0, 0),
            ("max-r3", "3-1-333333333333333333", 3, 0, 0, 0, 0),
            ("max-r4", "4-2-333333333333333333", 4, 0, 0, 0, 0),
            ("max-r5", "5-1-333333333333333333", 5, 0, 0, 0, 0),
            ("max-r6", "6-2-333333333333333333", 6, 0, 0, 0, 0),
            ("max-r7", "7-1-333333333333333333", 7, 0, 0, 0, 0),
            // freeSaber: forces saber off/def to >=1, changes reduction freebies
            ("free-max-r2", "2-1-333333333333333333", 2, 1, 0, 0, 0),
            ("free-max-r7", "7-2-333333333333333333", 7, 1, 0, 0, 0),
            // team gametype keeps team powers; non-team strips them
            ("team-gt", "5-1-333330033333003333", 5, 0, 0, GT_TEAM, 0),
            ("nonteam-gt", "5-1-333330033333003333", 5, 0, 0, 0, 0),
            // fpDisabled masks: disable a few powers (bit per FP_*)
            ("fpdis-heal", "7-1-333333333333333333", 7, 0, 0, 0, 1 << FP_HEAL),
            (
                "fpdis-saber",
                "7-1-333333333333333333",
                7,
                0,
                0,
                0,
                (1 << FP_SABER_OFFENSE) | (1 << FP_SABER_DEFENSE) | (1 << FP_LEVITATION),
            ),
            ("fpdis-all", "7-1-333333333333333333", 7, 1, 0, 0, 0x3FFFF),
            // saber-attack special-case in the reduction loop (def+throw present)
            ("saberheavy", "3-2-000000000000000333", 3, 0, 0, 0, 0),
            // all zeros (only the forced jump freebie should appear)
            ("zeros", "4-1-000000000000000000", 4, 0, 0, 0, 0),
            // over-128-char input -> bogus-default branch + invalidates
            (
                "toolong",
                "9-9-3333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333",
                6,
                0,
                0,
                0,
                0,
            ),
        ];

        for &(label, cfg, maxRank, freeSaber, teamForce, gametype, fpDisabled) in cases {
            // In/out buffers, identical contents for both sides. Sized 256 so the
            // over-128-char "toolong" input fits; the fn only ever writes back the short
            // legalized string (well within 128), so the tail stays equal for both.
            let mut rust_buf = [0 as c_char; 256];
            let mut orac_buf = [0 as c_char; 256];
            let bytes = cfg.as_bytes();
            assert!(bytes.len() < 256, "test buffer too small for {label}");
            for (i, &b) in bytes.iter().enumerate() {
                rust_buf[i] = b as c_char;
                orac_buf[i] = b as c_char;
            }

            let rust_ret = unsafe {
                BG_LegalizedForcePowers(
                    rust_buf.as_mut_ptr(),
                    maxRank,
                    freeSaber,
                    teamForce,
                    gametype,
                    fpDisabled,
                )
            };
            let orac_ret = unsafe {
                jka_bg_legalize_force_powers(
                    orac_buf.as_mut_ptr(),
                    maxRank,
                    freeSaber,
                    teamForce,
                    gametype,
                    fpDisabled,
                )
            };

            assert_eq!(rust_ret, orac_ret, "return mismatch for {label}");
            assert_eq!(rust_buf, orac_buf, "output buffer mismatch for {label}");
        }
    }

    /// Parity for `BG_ParseField`: drive every ported field type (`F_INT`,
    /// `F_FLOAT`, `F_VECTOR`, `F_ANGLEHACK`, `F_LSTRING`) plus the no-op arms (`F_IGNORE`,
    /// the not-yet-ported `F_PARM1`, and an unknown key) through one shared field table, parsing
    /// the same key/value stream into a Rust buffer and the verbatim-C oracle buffer, then
    /// compare. The `F_LSTRING` slot holds a freshly-allocated `char *` (a different
    /// address per side), so its 8 pointer bytes are compared by pointed-to string content
    /// rather than raw bytes; everything else is byte-for-byte. Values are all exactly
    /// representable in `f32` so the `vm`-feature `sscanf`/`atof` shims agree with libc.
    #[test]
    fn bg_parse_field_matches_oracle() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        G_InitMemory();

        // Field names (kept alive for the table's lifetime).
        let n_ival = CString::new("ival").unwrap();
        let n_fval = CString::new("fval").unwrap();
        let n_org = CString::new("origin").unwrap();
        let n_ang = CString::new("angle").unwrap();
        let n_msg = CString::new("message").unwrap();
        let n_ign = CString::new("ignoreme").unwrap();
        let n_parm = CString::new("parm1").unwrap();

        // Offsets into a 64-byte ent buffer; the F_LSTRING pointer lands 8-aligned at 32.
        const OFS_I: c_int = 0;
        const OFS_F: c_int = 4;
        const OFS_V: c_int = 8; // 3 floats: 8..20
        const OFS_A: c_int = 20; // 3 floats: 20..32
        const OFS_S: c_int = 32; // char*: 32..40
        const OFS_IGN: c_int = 40;
        const OFS_PARM: c_int = 44;

        let mk = |name: &CString, ofs: c_int, ty: fieldtype_t| BG_field_t {
            name: name.as_ptr() as *mut c_char,
            ofs,
            r#type: ty,
            flags: 0,
        };
        let mut fields = [
            mk(&n_ival, OFS_I, F_INT),
            mk(&n_fval, OFS_F, F_FLOAT),
            mk(&n_org, OFS_V, F_VECTOR),
            mk(&n_ang, OFS_A, F_ANGLEHACK),
            mk(&n_msg, OFS_S, F_LSTRING),
            mk(&n_ign, OFS_IGN, F_IGNORE),
            mk(&n_parm, OFS_PARM, F_PARM1),
            BG_field_t {
                name: core::ptr::null_mut(),
                ofs: 0,
                r#type: F_IGNORE,
                flags: 0,
            },
        ];

        // key/value stream (the unknown key must leave the buffer untouched).
        let kvs: &[(&str, &str)] = &[
            ("ival", "-42"),
            ("fval", "3.5"),
            ("origin", "1.5 -2.25 1024"),
            ("angle", "90"),
            ("message", "hello\\nworld"),
            ("ignoreme", "discarded"),
            ("parm1", "deferred"),
            ("nosuchkey", "x"),
        ];

        let mut rust_buf = [0u8; 64];
        let mut orac_buf = [0u8; 64];
        for &(k, v) in kvs {
            let key = CString::new(k).unwrap();
            let val = CString::new(v).unwrap();
            unsafe {
                BG_ParseField(
                    fields.as_mut_ptr(),
                    key.as_ptr(),
                    val.as_ptr(),
                    rust_buf.as_mut_ptr(),
                );
                jka_BG_ParseField(
                    fields.as_mut_ptr(),
                    key.as_ptr(),
                    val.as_ptr(),
                    orac_buf.as_mut_ptr(),
                );
            }
        }

        // Everything except the F_LSTRING pointer (bytes 32..40) is byte-exact.
        assert_eq!(
            rust_buf[..OFS_S as usize],
            orac_buf[..OFS_S as usize],
            "scalar region"
        );
        assert_eq!(
            rust_buf[40..],
            orac_buf[40..],
            "F_IGNORE/F_PARM/tail region (untouched)"
        );

        // The F_LSTRING slot: compare the strings the two pointers point at.
        unsafe {
            let rp = *(rust_buf.as_ptr().add(OFS_S as usize) as *const *const c_char);
            let op = *(orac_buf.as_ptr().add(OFS_S as usize) as *const *const c_char);
            assert!(!rp.is_null() && !op.is_null(), "F_LSTRING allocated");
            assert_eq!(
                CStr::from_ptr(rp).to_bytes(),
                CStr::from_ptr(op).to_bytes(),
                "F_LSTRING decoded string"
            );
        }
    }

    /// Element-wise parity for the four `bg_misc.c` file-scope data tables against
    /// their verbatim C copies in `bg_misc_oracle.c`. For the three `*const c_char`
    /// tables each slot is compared as either both-`NULL` (the terminators) or equal
    /// NUL-terminated bytes; the `bgToggleableSurfaceDebris` ints are compared directly.
    #[test]
    fn bg_misc_data_tables_match_oracle() {
        use crate::oracle::{
            jka_bg_custom_siege_sound_names_ptr, jka_bg_force_mastery_levels_ptr,
            jka_bg_toggleable_surface_debris_ptr, jka_bg_toggleable_surfaces_ptr,
        };

        // Compare a Rust `[*const c_char; N]` against an oracle base pointer, slot by slot.
        unsafe fn cmp_str_table(
            rust: &[*const c_char],
            oracle_base: *const *const c_char,
            name: &str,
        ) {
            for i in 0..rust.len() {
                let r = rust[i];
                let o = unsafe { *oracle_base.add(i) };
                assert_eq!(r.is_null(), o.is_null(), "{name}[{i}] NULL-ness");
                if !r.is_null() {
                    assert_eq!(
                        unsafe { CStr::from_ptr(r).to_bytes() },
                        unsafe { CStr::from_ptr(o).to_bytes() },
                        "{name}[{i}] string"
                    );
                }
            }
        }

        unsafe {
            cmp_str_table(
                &*addr_of!(bgToggleableSurfaces),
                jka_bg_toggleable_surfaces_ptr(),
                "bgToggleableSurfaces",
            );
            cmp_str_table(
                &*addr_of!(bg_customSiegeSoundNames),
                jka_bg_custom_siege_sound_names_ptr(),
                "bg_customSiegeSoundNames",
            );
            cmp_str_table(
                &*addr_of!(forceMasteryLevels),
                jka_bg_force_mastery_levels_ptr(),
                "forceMasteryLevels",
            );

            let debris_oracle = jka_bg_toggleable_surface_debris_ptr();
            for (i, &got) in bgToggleableSurfaceDebris.iter().enumerate() {
                assert_eq!(got, *debris_oracle.add(i), "bgToggleableSurfaceDebris[{i}]");
            }
        }
    }
}
