//! Port of `g_target.c` — the `target_*` entity classes: self-contained "reactor"
//! entities that fire when used (relays, speakers, counters, scriptrunners, …).
//!
//! Each entity is a `Use_*`/`Think_*` callback (an `unsafe extern "C" fn` stored in
//! `ent.r#use`/`ent.think`) plus an `SP_*` spawner that wires the callbacks up. The
//! `SP_*` functions are `unsafe extern "C" fn(*mut gentity_t)` to match the (still
//! gated) `G_CallSpawn` registry's `void (*spawn)(gentity_t*)` slot.
//!
//! Landed incrementally: only the classes whose callbacks reach already-ported deps.
//! All callbacks are No-oracle — engine-syscall plumbing over the global level/entity state.

#![allow(non_snake_case)] // C function names (`SP_target_position`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro names (`ACT_ACTIVE`, …) kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::bg_public::{
    CS_LOCATIONS, CS_MUSIC, EF_PERMANENT, ET_BEAM, ET_SPEAKER, EV_GENERAL_SOUND, EV_GLOBAL_SOUND,
    MOD_TARGET_LASER, MOD_TELEFRAG, PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG, TEAM_BLUE, TEAM_FREE,
    TEAM_RED,
};
use crate::codemp::game::g_ICARUScb::G_DebugPrint;
use crate::codemp::game::g_combat::{AddScore, G_Damage};
use crate::codemp::game::g_items::Touch_Item;
use crate::codemp::game::g_local::{
    gentity_s, gentity_t, DAMAGE_NO_KNOCKBACK, DAMAGE_NO_PROTECTION, FL_INACTIVE, FRAMETIME,
};
use crate::codemp::game::g_main::{
    g_developer, g_entities, level, Com_Error, Com_Printf, G_Error, G_Printf,
};
use crate::codemp::game::g_misc::TeleportPlayer;
use crate::codemp::game::g_public_h::{BSET_USE, SVF_BROADCAST};
use crate::codemp::game::g_spawn::{G_NewString, G_SpawnFloat, G_SpawnString};
use crate::codemp::game::g_team::Team_ReturnFlag;
use crate::codemp::game::g_utils::{
    vtos, GlobalUse, G_AddEvent, G_Find, G_FreeEntity, G_PickTarget, G_SetMovedir, G_SetOrigin,
    G_SoundIndex, G_SoundSetIndex, G_TeamCommand, G_UseTargets, G_UseTargets2,
};
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::q_math::{vec3_origin, VectorCopy, VectorMA, VectorNormalize, VectorSubtract};
use crate::codemp::game::q_shared::{crandom, va, Sz, Q_stricmp, Q_strncpyz};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{trace_t, vec3_t, ERR_DROP, MAX_QPATH, Q3_SCRIPT_DIR};
use crate::codemp::game::surfaceflags_h::{CONTENTS_BODY, CONTENTS_CORPSE, CONTENTS_SOLID};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `EXEC_NOW` (q_shared.h:410) — the `EXEC_*` enum's first value, so `0`. The trap's
/// `exec_when`: "don't return until completed." Kept local until a second consumer
/// wants the full enum in `q_shared_h.rs` (cf. `Q3_SCRIPT_DIR` in `npc_utils`).
const EXEC_NOW: i32 = 0;

/// `WL_VERBOSE` (q_shared.h `enum WL_e`) — the verbose debug-print level, `= 3`. Defined
/// locally because the enum lives in a header the ICARUS TU can't include (cf. the
/// `WL_*` consts in `g_ICARUScb.rs`); `G_DebugPrint` treats it as the default arm.
const WL_VERBOSE: c_int = 3;

//==========================================================

/*QUAKED target_give (1 0 0) (-8 -8 -8) (8 8 8)
Gives the activator all the items pointed to.
*/
/// `void Use_Target_Give( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_target.c:10). Hands the activator every item targeted by `ent->target`: walks the
/// `G_Find`/`FOFS(targetname)` chain, [`Touch_Item`]s each one onto the activator (with a
/// zeroed scratch `trace_t`), then suppresses respawn/events by zeroing `nextthink` and
/// unlinking. Bails if the activator isn't a client or `ent` has no `target`. No oracle
/// (drives the item-pickup + unlink plumbing over the global entity array).
///
/// # Safety
/// `ent`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn Use_Target_Give(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*activator).client.is_null() {
        return;
    }

    if (*ent).target.is_null() {
        return;
    }

    // memset( &trace, 0, sizeof( trace ) );
    let mut trace: trace_t = trace_t::default();
    let mut t: *mut gentity_t = null_mut();
    loop {
        t = G_Find(t, offset_of!(gentity_s, targetname), (*ent).target);
        if t.is_null() {
            break;
        }
        if (*t).item.is_null() {
            continue;
        }
        Touch_Item(t, activator, &mut trace);

        // make sure it isn't going to respawn or show any events
        (*t).nextthink = 0;
        trap::UnlinkEntity(t);
    }
}

/// `void SP_target_give( gentity_t *ent )` (g_target.c:36).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_give(ent: *mut gentity_t) {
    (*ent).r#use = Some(Use_Target_Give);
}

//==========================================================

/*QUAKED target_remove_powerups (1 0 0) (-8 -8 -8) (8 8 8)
takes away all the activators powerups.
Used to drop flight powerups into death puts.
*/
/// `void Use_target_remove_powerups( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_target.c:47). Strips every powerup off the activator: if it is carrying a CTF flag
/// powerup, returns that flag to base first ([`Team_ReturnFlag`]), then zeroes the whole
/// `ps.powerups` array (the C `memset(..., 0, sizeof(...))`). Bails if the activator is not a
/// client. No oracle (mutates engine-visible client playerState + drives the CTF flag-return
/// plumbing).
///
/// # Safety
/// `ent`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn Use_target_remove_powerups(
    _ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*activator).client.is_null() {
        return;
    }

    if (*(*activator).client).ps.powerups[PW_REDFLAG as usize] != 0 {
        Team_ReturnFlag(TEAM_RED);
    } else if (*(*activator).client).ps.powerups[PW_BLUEFLAG as usize] != 0 {
        Team_ReturnFlag(TEAM_BLUE);
    } else if (*(*activator).client).ps.powerups[PW_NEUTRALFLAG as usize] != 0 {
        Team_ReturnFlag(TEAM_FREE);
    }

    (*(*activator).client).ps.powerups = [0; crate::codemp::game::q_shared_h::MAX_POWERUPS];
}

/// `void SP_target_remove_powerups( gentity_t *ent )` (g_target.c:63).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_remove_powerups(ent: *mut gentity_t) {
    (*ent).r#use = Some(Use_target_remove_powerups);
}

//==========================================================

/*QUAKED target_position (0 0.5 0) (-4 -4 -4) (4 4 4)
Used as a positional target for in-game calculation, like jumppad targets.
*/
/// `void SP_target_position( gentity_t *self )` (g_target.c:531).
///
/// A bare position marker — no `use`/`think`, just pins its origin so other entities
/// (jumppads, etc.) can aim at it. The C `G_SetAngles`/`s.eType = ET_INVISIBLE` lines
/// are commented out in the original and stay omitted. No oracle (drives `G_SetOrigin`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_position(self_: *mut gentity_t) {
    G_SetOrigin(self_, &(*self_).s.origin);
}

//==========================================================

/*QUAKED target_score (1 0 0) (-8 -8 -8) (8 8 8)
"count" number of points to add, default 1

The activator is given this many points.
*/
/// `void Use_Target_Score( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_target.c:113). Awards `ent->count` points to the activator via [`AddScore`], using
/// the score-entity's current origin as the score-pop position. No oracle (wraps `AddScore`,
/// which drives the score-event plumbing).
///
/// # Safety
/// `ent`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn Use_Target_Score(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    AddScore(activator, &(*ent).r.currentOrigin, (*ent).count);
}

/// `void SP_target_score( gentity_t *ent )` (g_target.c:117). Defaults `count` to 1 (one
/// point) when unset, then wires the [`Use_Target_Score`] use callback. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_score(ent: *mut gentity_t) {
    if (*ent).count == 0 {
        (*ent).count = 1;
    }
    (*ent).r#use = Some(Use_Target_Score);
}

//==========================================================

/// `void G_SetActiveState( char *targetstring, qboolean actState )` (g_target.c:885).
///
/// Flip the `FL_INACTIVE` flag on every entity whose `targetname` matches
/// `targetstring`: clear it when `actState` is set (make usable/triggerable), set it
/// otherwise. Driven by `target_activate`/`target_deactivate`. No oracle (walks the
/// global entity array via `G_Find`).
///
/// # Safety
/// `targetstring` must be a valid C string (or null — `G_Find` handles it).
pub unsafe fn G_SetActiveState(targetstring: *mut c_char, act_state: qboolean) {
    let mut target: *mut gentity_t = null_mut();
    loop {
        target = G_Find(target, offset_of!(gentity_s, targetname), targetstring);
        if target.is_null() {
            break;
        }
        (*target).flags = if act_state != QFALSE {
            (*target).flags & !FL_INACTIVE
        } else {
            (*target).flags | FL_INACTIVE
        };
    }
}

const ACT_ACTIVE: qboolean = QTRUE; // #define ACT_ACTIVE   qtrue
const ACT_INACTIVE: qboolean = QFALSE; // #define ACT_INACTIVE qfalse

/// `void target_activate_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:897). Fires `self`'s use-script, then makes its targets usable.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`activator` are unused.
pub unsafe extern "C" fn target_activate_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);
    G_SetActiveState((*self_).target, ACT_ACTIVE);
}

/// `void target_deactivate_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:904). Fires `self`'s use-script, then makes its targets non-usable.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`activator` are unused.
pub unsafe extern "C" fn target_deactivate_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);
    G_SetActiveState((*self_).target, ACT_INACTIVE);
}

/*QUAKED target_activate (1 0 0) (-4 -4 -4) (4 4 4)
Will set the target(s) to be usable/triggerable
*/
/// `void SP_target_activate( gentity_t *self )` (g_target.c:915).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_activate(self_: *mut gentity_t) {
    G_SetOrigin(self_, &(*self_).s.origin);
    (*self_).r#use = Some(target_activate_use);
}

/*QUAKED target_deactivate (1 0 0) (-4 -4 -4) (4 4 4)
Will set the target(s) to be non-usable/triggerable
*/
/// `void SP_target_deactivate( gentity_t *self )` (g_target.c:924).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_deactivate(self_: *mut gentity_t) {
    G_SetOrigin(self_, &(*self_).s.origin);
    (*self_).r#use = Some(target_deactivate_use);
}

//==========================================================

/// `void target_level_change_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:930). Fires `self`'s use-script, then issues an immediate `map <message>`
/// console command to change level. No oracle (drives the console-command trap).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a non-null `message`.
pub unsafe extern "C" fn target_level_change_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);
    // trap_SendConsoleCommand(EXEC_NOW, va("map %s", self->message));
    let msg = CStr::from_ptr((*self_).message).to_string_lossy();
    trap::SendConsoleCommand(EXEC_NOW, &format!("map {msg}"));
}

/*QUAKED target_level_change (1 0 0) (-4 -4 -4) (4 4 4)
"mapname" - Name of map to change to
*/
/// `void SP_target_level_change( gentity_t *self )` (g_target.c:940).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_level_change(self_: *mut gentity_t) {
    let mut s: *mut c_char = null_mut();
    G_SpawnString(c"mapname".as_ptr(), c"".as_ptr(), &mut s);
    (*self_).message = G_NewString(s);

    if (*self_).message.is_null() || *(*self_).message == 0 {
        // G_Error never returns, so the C `return;` after it is unreachable here.
        G_Error("target_level_change with no mapname!\n");
    }

    G_SetOrigin(self_, &(*self_).s.origin);
    (*self_).r#use = Some(target_level_change_use);
}

/// `void target_play_music_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:957). Fires `self`'s use-script, then sets the `CS_MUSIC` configstring so
/// clients start playing `message`. No oracle (drives the configstring trap).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a non-null `message`.
pub unsafe extern "C" fn target_play_music_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);
    trap::SetConfigstring(CS_MUSIC, &CStr::from_ptr((*self_).message).to_string_lossy());
}

/*QUAKED target_play_music (1 0 0) (-4 -4 -4) (4 4 4)
target_play_music
Plays the requested music files when this target is used.
*/
/// `void SP_target_play_music( gentity_t *self )` (g_target.c:974).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_play_music(self_: *mut gentity_t) {
    G_SetOrigin(self_, &(*self_).s.origin);

    let mut s: *mut c_char = null_mut();
    if G_SpawnString(c"music".as_ptr(), c"".as_ptr(), &mut s) == QFALSE {
        G_Error(&format!(
            "target_play_music without a music key at {}",
            CStr::from_ptr(vtos(&(*self_).s.origin)).to_string_lossy()
        ));
    }

    (*self_).message = G_NewString(s);
    (*self_).r#use = Some(target_play_music_use);
}

//==========================================================

/// `void Think_Target_Delay( gentity_t *ent )` (g_target.c:78). The deferred fire:
/// once the delay elapses, use `ent`'s targets with the stored activator.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
unsafe extern "C" fn Think_Target_Delay(ent: *mut gentity_t) {
    G_UseTargets(ent, (*ent).activator);
}

/// `void Use_Target_Delay( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_target.c:82). Arms a delayed `G_UseTargets`: schedules [`Think_Target_Delay`] at
/// `wait +/- random` seconds out. With the `NO_RETRIGGER` spawnflag set, a re-use while
/// already counting down is ignored. No oracle (drives the script hook + think dispatch,
/// reads the global `level`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other` is unused.
pub unsafe extern "C" fn Use_Target_Delay(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*ent).nextthink > (*addr_of!(level)).time && ((*ent).spawnflags & 1) != 0 {
        // Leave me alone, I am thinking.
        return;
    }
    G_ActivateBehavior(ent, BSET_USE);
    // nextthink = level.time + ( wait + random * crandom() ) * 1000 — the C evaluates
    // this in `double` (crandom() is double, `1000` promotes), then truncates to the int
    // nextthink, so the f64 arithmetic here is load-bearing.
    let delay = ((*ent).wait as f64 + (*ent).random as f64 * crandom()) * 1000.0;
    (*ent).nextthink = ((*addr_of!(level)).time as f64 + delay) as c_int;
    (*ent).think = Some(Think_Target_Delay);
    (*ent).activator = activator;
}

/*QUAKED target_delay (1 0 0) (-8 -8 -8) (8 8 8) NO_RETRIGGER
"wait" seconds to pause before firing targets.
"random" delay variance, total delay = delay +/- random seconds
*/
/// `void SP_target_delay( gentity_t *ent )` (g_target.c:93).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_delay(ent: *mut gentity_t) {
    // check delay for backwards compatability
    if G_SpawnFloat(c"delay".as_ptr(), c"0".as_ptr(), &mut (*ent).wait) == QFALSE {
        G_SpawnFloat(c"wait".as_ptr(), c"1".as_ptr(), &mut (*ent).wait);
    }

    if (*ent).wait == 0.0 {
        (*ent).wait = 1.0;
    }
    (*ent).r#use = Some(Use_Target_Delay);
}

//==========================================================

/// `static void target_location_linkup( gentity_t *ent )` (g_target.c:539).
///
/// Runs once (200ms after spawn, gated by `level.locationLinked`): scans the whole
/// entity array for `target_location`s, assigns each an index (stashed in `health`),
/// publishes its name into the `CS_LOCATIONS+n` configstring, and threads them onto
/// `level.locationHead` via `nextTrain`. The `ent` parameter is the entity whose think
/// fired but is immediately overwritten by the C scan loop, so it is unused here.
/// File-local (`static` in C). No oracle (walks the global array, drives the trap).
///
/// DEVIATION: the C passes `ent->message` straight to `trap_SetConfigstring`; a
/// null message (mapper omitted the key) would be a null deref here, so we substitute
/// `""`. Every target_location is expected to carry a message, so this only hardens a
/// malformed map.
///
/// # Safety
/// Reads the global `level`/`g_entities`; all matched entities must be valid.
unsafe extern "C" fn target_location_linkup(_ent: *mut gentity_t) {
    if (*addr_of!(level)).locationLinked != QFALSE {
        return;
    }

    (*addr_of_mut!(level)).locationLinked = QTRUE;
    (*addr_of_mut!(level)).locationHead = null_mut();

    trap::SetConfigstring(CS_LOCATIONS, "unknown");

    let mut ent = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>();
    let mut n: c_int = 1;
    let mut i: c_int = 0;
    while i < (*addr_of!(level)).num_entities {
        if !(*ent).classname.is_null()
            && Q_stricmp((*ent).classname, c"target_location".as_ptr()) == 0
        {
            (*ent).health = n; // use for location marking
            let msg = (!(*ent).message.is_null())
                .then(|| CStr::from_ptr((*ent).message).to_string_lossy())
                .unwrap_or_default();
            trap::SetConfigstring(CS_LOCATIONS + n, &msg);
            n += 1;
            (*ent).nextTrain = (*addr_of!(level)).locationHead;
            (*addr_of_mut!(level)).locationHead = ent;
        }
        i += 1;
        ent = ent.add(1);
    }

    // All linked together now
}

/*QUAKED target_location (0 0.5 0) (-8 -8 -8) (8 8 8)
Set "message" to the name of this location.
*/
/// `void SP_target_location( gentity_t *self )` (g_target.c:577).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_location(self_: *mut gentity_t) {
    (*self_).think = Some(target_location_linkup);
    (*self_).nextthink = (*addr_of!(level)).time + 200; // Let them all spawn first

    G_SetOrigin(self_, &(*self_).s.origin);
}

//==========================================================

/*QUAKED target_counter (1.0 0 0) (-4 -4 -4) (4 4 4) x x x x x x x INACTIVE
Acts as an intermediary for an action that takes multiple inputs.

INACTIVE cannot be used until used by a target_activate

target2 - what the counter should fire each time it's incremented and does NOT reach it's count

After the counter has been triggered "count" times (default 2), it will fire all of it's targets and remove itself.

bounceCount - number of times the counter should reset to it's full count when it's done
*/
/// `void target_counter_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:595). Intermediary for an action with multiple inputs: decrements `count`
/// each time it's used, firing `target2` (via `G_UseTargets2`) on every non-final tick and
/// firing all of its targets — plus the `BSET_USE` behavior — once the count reaches zero.
/// The spawnflag-128 `INACTIVE` bit then disables the counter; a non-zero `bounceCount`
/// re-arms it from the cached `genericValue1` (a `bounceCount` of -1 bounces forever). No
/// oracle (global entity state + script hook).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `activator` may be null; `other` is unused.
pub unsafe extern "C" fn target_counter_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*self_).count == 0 {
        return;
    }

    //gi.Printf("target_counter %s used by %s, entnum %d\n", self->targetname, activator->targetname, activator->s.number );
    (*self_).count -= 1;

    if !activator.is_null() {
        G_DebugPrint(
            WL_VERBOSE,
            &format!(
                "target_counter {} used by {} ({}/{})\n",
                Sz((*self_).targetname),
                Sz((*activator).targetname),
                (*self_).genericValue1 - (*self_).count,
                (*self_).genericValue1
            ),
        );
    }

    if (*self_).count != 0 {
        if !(*self_).target2.is_null() {
            //gi.Printf("target_counter %s firing target2 from %s, entnum %d\n", self->targetname, activator->targetname, activator->s.number );
            G_UseTargets2(self_, activator, (*self_).target2);
        }
        return;
    }

    G_ActivateBehavior(self_, BSET_USE);

    if (*self_).spawnflags & 128 != 0 {
        (*self_).flags |= FL_INACTIVE;
    }

    (*self_).activator = activator;
    G_UseTargets(self_, activator);

    if (*self_).count == 0 {
        if (*self_).bounceCount == 0 {
            return;
        }
        (*self_).count = (*self_).genericValue1;
        if (*self_).bounceCount > 0 {
            //-1 means bounce back forever
            (*self_).bounceCount -= 1;
        }
    }
}

/// `void SP_target_counter( gentity_t *self )` (g_target.c:645). Wires up the counter's
/// `use` callback: forces `wait = -1`, defaults `count` to 2, and caches the initial count
/// in `genericValue1` so [`target_counter_use`] can re-arm on bounce. No oracle (spawn).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_counter(self_: *mut gentity_t) {
    (*self_).wait = -1.0;
    if (*self_).count == 0 {
        (*self_).count = 2;
    }
    //if ( self->bounceCount > 0 )//let's always set this anyway
    {
        //we will reset when we use up our count, remember our initial count
        (*self_).genericValue1 = (*self_).count;
    }

    (*self_).r#use = Some(target_counter_use);
}

//==========================================================

/// `void target_random_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:666). Fires exactly one randomly-chosen target (vs. the usual fire-all):
/// counts the entities matching `self->target` (excluding self), picks one with
/// [`Q_irand`], and uses it. The `USEONCE` spawnflag (bit 0) clears `use` so it never
/// fires again. No oracle (walks the global array, drives the script hook + GlobalUse).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other` is unused.
pub unsafe extern "C" fn target_random_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    let mut t_count: c_int = 0;
    let mut t: *mut gentity_t = null_mut();

    G_ActivateBehavior(self_, BSET_USE);

    if (*self_).spawnflags & 1 != 0 {
        (*self_).r#use = None;
    }

    loop {
        t = G_Find(t, offset_of!(gentity_s, targetname), (*self_).target);
        if t.is_null() {
            break;
        }
        if t != self_ {
            t_count += 1;
        }
    }

    if t_count == 0 {
        return;
    }

    if t_count == 1 {
        G_UseTargets(self_, activator);
        return;
    }

    //FIXME: need a seed
    let pick = Q_irand(1, t_count);
    t_count = 0;
    loop {
        t = G_Find(t, offset_of!(gentity_s, targetname), (*self_).target);
        if t.is_null() {
            break;
        }
        if t != self_ {
            t_count += 1;
        } else {
            continue;
        }

        if t == self_ {
            // gi.Printf ("WARNING: Entity used itself.\n");
        } else if t_count == pick && (*t).r#use.is_some() {
            // (the `t->use != NULL` check can be omitted, but is kept faithful)
            GlobalUse(t, self_, activator);
            return;
        }

        if (*self_).inuse == QFALSE {
            Com_Printf("entity was removed while using targets\n");
            return;
        }
    }
}

/*QUAKED target_random (.5 .5 .5) (-4 -4 -4) (4 4 4) USEONCE
Randomly fires off only one of it's targets each time used
*/
/// `void SP_target_random( gentity_t *self )` (g_target.c:733).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_random(self_: *mut gentity_t) {
    (*self_).r#use = Some(target_random_use);
}

//==========================================================

/// `void target_teleporter_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:425). Teleports the activator to this target's `target` destination: bails
/// if the activator isn't a client, fires the BSET_USE script behavior, looks up the
/// destination entity, and relocates the activator to its origin/angles via
/// [`TeleportPlayer`]. No oracle (script hook + engine teleport syscalls).
///
/// # Safety
/// `self_`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn target_teleporter_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*activator).client.is_null() {
        return;
    }

    G_ActivateBehavior(self_, BSET_USE);

    let dest = G_PickTarget((*self_).target);
    if dest.is_null() {
        G_Printf("Couldn't find teleporter destination\n");
        return;
    }

    TeleportPlayer(activator, &(*dest).s.origin, &(*dest).s.angles);
}

/*QUAKED target_teleporter (1 0 0) (-8 -8 -8) (8 8 8)
The activator will be teleported away.
*/
/// `void SP_target_teleporter( gentity_t *self )` (g_target.c:445). Warns about an
/// untargeted instance, then installs [`target_teleporter_use`]. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_teleporter(self_: *mut gentity_t) {
    if (*self_).targetname.is_null() {
        G_Printf(&format!(
            "untargeted {} at {}\n",
            CStr::from_ptr((*self_).classname).to_string_lossy(),
            CStr::from_ptr(vtos(&(*self_).s.origin)).to_string_lossy()
        ));
    }

    (*self_).r#use = Some(target_teleporter_use);
}

//==========================================================

/// `void Use_Target_Speaker( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_target.c:244). Fires the speaker: a looping speaker (spawnflags 1|2) toggles its
/// `loopSound` on/off; a one-shot adds an `EV_GENERAL_SOUND`/`EV_GLOBAL_SOUND` event on
/// the activator, the entity, or globally depending on spawnflags 8/4. No oracle (drives
/// the script hook + event queue).
///
/// # Safety
/// `ent`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn Use_Target_Speaker(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    G_ActivateBehavior(ent, BSET_USE);

    if (*ent).spawnflags & 3 != 0 {
        // looping sound toggles
        if (*ent).s.loopSound != 0 {
            (*ent).s.loopSound = 0; // turn it off
            (*ent).s.loopIsSoundset = QFALSE;
            (*ent).s.trickedentindex = 1;
        } else {
            (*ent).s.loopSound = (*ent).noise_index; // start it
            (*ent).s.loopIsSoundset = QFALSE;
            (*ent).s.trickedentindex = 0;
        }
    } else {
        // normal sound
        if (*ent).spawnflags & 8 != 0 {
            G_AddEvent(activator, EV_GENERAL_SOUND, (*ent).noise_index);
        } else if (*ent).spawnflags & 4 != 0 {
            G_AddEvent(ent, EV_GLOBAL_SOUND, (*ent).noise_index);
        } else {
            G_AddEvent(ent, EV_GENERAL_SOUND, (*ent).noise_index);
        }
    }
}

/*QUAKED target_speaker (1 0 0) (-8 -8 -8) (8 8 8) looped-on looped-off global activator
"noise"		wav file to play
*/
/// `void SP_target_speaker( gentity_t *ent )` (g_target.c:271).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_speaker(ent: *mut gentity_t) {
    let mut s: *mut c_char = null_mut();

    G_SpawnFloat(c"wait".as_ptr(), c"0".as_ptr(), &mut (*ent).wait);
    G_SpawnFloat(c"random".as_ptr(), c"0".as_ptr(), &mut (*ent).random);

    if G_SpawnString(c"soundSet".as_ptr(), c"".as_ptr(), &mut s) != QFALSE {
        // this is a sound set
        (*ent).s.soundSetIndex = G_SoundSetIndex(&CStr::from_ptr(s).to_string_lossy());
        (*ent).s.eFlags = EF_PERMANENT;
        let origin = (*ent).s.origin;
        VectorCopy(&origin, &mut (*ent).s.pos.trBase);
        trap::LinkEntity(ent);
        return;
    }

    if G_SpawnString(c"noise".as_ptr(), c"NOSOUND".as_ptr(), &mut s) == QFALSE {
        G_Error(&format!(
            "target_speaker without a noise key at {}",
            CStr::from_ptr(vtos(&(*ent).s.origin)).to_string_lossy()
        ));
    }

    // force all client reletive sounds to be "activator" speakers that
    // play on the entity that activates it
    if *s == b'*' as c_char {
        (*ent).spawnflags |= 8;
    }

    let mut buffer = [0 as c_char; MAX_QPATH];
    Q_strncpyz(buffer.as_mut_ptr(), s, MAX_QPATH as c_int);

    (*ent).noise_index = G_SoundIndex(&CStr::from_ptr(buffer.as_ptr()).to_string_lossy());

    // a repeating speaker can be done completely client side
    (*ent).s.eType = ET_SPEAKER;
    (*ent).s.eventParm = (*ent).noise_index;
    (*ent).s.frame = ((*ent).wait * 10.0) as c_int;
    (*ent).s.clientNum = ((*ent).random * 10.0) as c_int;

    // check for prestarted looping sound
    if (*ent).spawnflags & 1 != 0 {
        (*ent).s.loopSound = (*ent).noise_index;
        (*ent).s.loopIsSoundset = QFALSE;
    }

    (*ent).r#use = Some(Use_Target_Speaker);

    if (*ent).spawnflags & 4 != 0 {
        (*ent).r.svFlags |= SVF_BROADCAST;
    }

    let origin = (*ent).s.origin;
    VectorCopy(&origin, &mut (*ent).s.pos.trBase);

    // must link the entity so we get areas and clusters so
    // the server can determine who to send updates to
    trap::LinkEntity(ent);
}

//==========================================================

/// `void Use_Target_Print( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_target.c:132). Prints `ent->message` as a centerprint: to the activator (spawnflag
/// 4 + client), to red/blue teams (spawnflags 1/2), or to everyone. A leading `@` (but
/// not `@@`) selects the `cps` (string-id centerprint) command over plain `cp`. A `wait`
/// throttles re-prints. No oracle (drives the script hook + server commands, reads the
/// global level).
///
/// The `#ifndef FINAL_BUILD` quick-succession guard is included: this module is built
/// non-FINAL (cf. bg_panimate/bg_saberLoad), so the genericValue15 throttle is live.
///
/// # Safety
/// `ent`/`activator` must point to valid `gentity_t`s with a non-null `message`; `other`
/// is unused.
pub unsafe extern "C" fn Use_Target_Print(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if ent.is_null() || (*ent).inuse == QFALSE {
        Com_Printf("ERROR: Bad ent in Use_Target_Print");
        return;
    }

    if (*ent).wait != 0.0 {
        if (*ent).genericValue14 >= (*addr_of!(level)).time {
            return;
        }
        (*ent).genericValue14 = ((*addr_of!(level)).time as f32 + (*ent).wait) as c_int;
    }

    // #ifndef FINAL_BUILD — non-final module, so this throttle is compiled in.
    if ent.is_null() || (*ent).inuse == QFALSE {
        Com_Error(ERR_DROP, "Bad ent in Use_Target_Print");
    } else if activator.is_null() || (*activator).inuse == QFALSE {
        Com_Error(ERR_DROP, "Bad activator in Use_Target_Print");
    }

    if (*ent).genericValue15 > (*addr_of!(level)).time {
        Com_Printf("TARGET PRINT ERRORS:\n");
        if !activator.is_null()
            && !(*activator).classname.is_null()
            && *(*activator).classname != 0
        {
            Com_Printf(&format!(
                "activator classname: {}\n",
                CStr::from_ptr((*activator).classname).to_string_lossy()
            ));
        }
        if !activator.is_null() && !(*activator).target.is_null() && *(*activator).target != 0 {
            Com_Printf(&format!(
                "activator target: {}\n",
                CStr::from_ptr((*activator).target).to_string_lossy()
            ));
        }
        if !activator.is_null()
            && !(*activator).targetname.is_null()
            && *(*activator).targetname != 0
        {
            Com_Printf(&format!(
                "activator targetname: {}\n",
                CStr::from_ptr((*activator).targetname).to_string_lossy()
            ));
        }
        if !(*ent).targetname.is_null() && *(*ent).targetname != 0 {
            Com_Printf(&format!(
                "print targetname: {}\n",
                CStr::from_ptr((*ent).targetname).to_string_lossy()
            ));
        }
        Com_Error(
            ERR_DROP,
            "target_print used in quick succession, fix it! See the console for details.",
        );
    }
    (*ent).genericValue15 = (*addr_of!(level)).time + 5000;
    // #endif

    G_ActivateBehavior(ent, BSET_USE);

    // A leading '@' (but not "@@") marks a string-id message -> "cps" instead of "cp".
    let msg = CStr::from_ptr((*ent).message).to_string_lossy();
    let cmd = if *(*ent).message == b'@' as c_char && *(*ent).message.add(1) != b'@' as c_char {
        "cps"
    } else {
        "cp"
    };

    if (*ent).spawnflags & 4 != 0 {
        // private, to one client only
        if activator.is_null() || (*activator).inuse == QFALSE {
            Com_Printf("ERROR: Bad activator in Use_Target_Print");
        }
        if !activator.is_null() && !(*activator).client.is_null() {
            // make sure there's a valid client ent to send it to
            trap::SendServerCommand(
                activator.offset_from(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()) as c_int,
                &format!("{cmd} \"{msg}\""),
            );
        }
        // NOTE: change in functionality - if there *is* no valid client ent,
        // it won't send it to anyone at all
        return;
    }

    if (*ent).spawnflags & 3 != 0 {
        if (*ent).spawnflags & 1 != 0 {
            G_TeamCommand(TEAM_RED, va(format_args!("{cmd} \"{msg}\"")));
        }
        if (*ent).spawnflags & 2 != 0 {
            G_TeamCommand(TEAM_BLUE, va(format_args!("{cmd} \"{msg}\"")));
        }
        return;
    }

    trap::SendServerCommand(-1, &format!("{cmd} \"{msg}\""));
}

/*QUAKED target_print (1 0 0) (-8 -8 -8) (8 8 8) redteam blueteam private
"message"	text to print
*/
/// `void SP_target_print( gentity_t *ent )` (g_target.c:224).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_print(ent: *mut gentity_t) {
    (*ent).r#use = Some(Use_Target_Print);
}

//==========================================================

/// `int numNewICARUSEnts` (g_target.c:738) — counter for auto-generated ICARUS entity
/// names. A plain file-global `int` in C; kept as `static mut` (the game is
/// single-threaded), accessed via `addr_of`/`addr_of_mut` like `remapCount` in g_utils.
static mut numNewICARUSEnts: c_int = 0;

/// `void scriptrunner_run( gentity_t *self )` (g_target.c:739). The scriptrunner's
/// actual work, run inline or as a delayed think: decrements the use count (clearing
/// `use` when exhausted), then either runs an ICARUS script on the activator
/// (`runonactivator` spawnflag 1) or fires `self`'s own BSET_USE behavior. Re-arms a
/// `wait` interval. File-local. No oracle (drives the ICARUS traps + script hook).
///
/// The commented-out legacy `ICARUS_RunScript(self, ...)` block at the top of the C is
/// omitted (it is `/* */`-commented in the original).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn scriptrunner_run(self_: *mut gentity_t) {
    if (*self_).count != -1 {
        if (*self_).count <= 0 {
            (*self_).r#use = None;
            (*self_).behaviorSet[BSET_USE as usize] = null_mut();
            return;
        } else {
            (*self_).count -= 1;
        }
    }

    if !(*self_).behaviorSet[BSET_USE as usize].is_null() {
        if (*self_).spawnflags & 1 != 0 {
            if (*self_).activator.is_null() {
                if (*addr_of!(g_developer)).integer != 0 {
                    Com_Printf("target_scriptrunner tried to run on invalid entity!\n");
                }
                return;
            }

            //if ( !self->activator->sequencer || !self->activator->taskManager )
            if trap::ICARUS_IsInitialized((*self_).s.number) == QFALSE {
                // Need to be initialized through ICARUS
                if (*(*self_).activator).script_targetname.is_null()
                    || *(*(*self_).activator).script_targetname == 0
                {
                    // We don't have a script_targetname, so create a new one
                    let n = *addr_of!(numNewICARUSEnts);
                    *addr_of_mut!(numNewICARUSEnts) = n + 1;
                    (*(*self_).activator).script_targetname = va(format_args!("newICARUSEnt{n}"));
                }

                if trap::ICARUS_ValidEnt((*self_).activator) != QFALSE {
                    trap::ICARUS_InitEnt((*self_).activator);
                } else {
                    if (*addr_of!(g_developer)).integer != 0 {
                        Com_Printf(
                            "target_scriptrunner tried to run on invalid ICARUS activator!\n",
                        );
                    }
                    return;
                }
            }

            if (*addr_of!(g_developer)).integer != 0 {
                Com_Printf(&format!(
                    "target_scriptrunner running {} on activator {}\n",
                    Sz((*self_).behaviorSet[BSET_USE as usize]),
                    Sz((*(*self_).activator).targetname)
                ));
            }
            trap::ICARUS_RunScript(
                (*self_).activator,
                va(format_args!(
                    "{}/{}",
                    Q3_SCRIPT_DIR,
                    Sz((*self_).behaviorSet[BSET_USE as usize])
                )),
            );
        } else {
            if (*addr_of!(g_developer)).integer != 0 && !(*self_).activator.is_null() {
                Com_Printf(&format!(
                    "target_scriptrunner {} used by {}\n",
                    Sz((*self_).targetname),
                    Sz((*(*self_).activator).targetname)
                ));
            }
            G_ActivateBehavior(self_, BSET_USE);
        }
    }

    if (*self_).wait != 0.0 {
        (*self_).nextthink = ((*addr_of!(level)).time as f32 + (*self_).wait) as c_int;
    }
}

/// `void target_scriptrunner_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:824). Stores activator/enemy and either fires [`scriptrunner_run`] now or
/// schedules it after `delay`. A pending `nextthink` in the future blocks re-use. No
/// oracle (defers to scriptrunner_run / the think dispatch).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn target_scriptrunner_use(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*self_).nextthink > (*addr_of!(level)).time {
        return;
    }

    (*self_).activator = activator;
    (*self_).enemy = other;
    if (*self_).delay != 0 {
        // delay before firing scriptrunner
        (*self_).think = Some(scriptrunner_run);
        (*self_).nextthink = (*addr_of!(level)).time + (*self_).delay;
    } else {
        scriptrunner_run(self_);
    }
}

/*QUAKED target_scriptrunner (1 0 0) (-4 -4 -4) (4 4 4) runonactivator x x x x x x INACTIVE
runonactivator - Will run the script on the entity that used this or tripped the trigger
INACTIVE - start off
Usescript - Script to run when used
count - how many times to run, -1 = infinite.  Default is once
*/
/// `void SP_target_scriptrunner( gentity_t *self )` (g_target.c:856).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_scriptrunner(self_: *mut gentity_t) {
    if (*self_).spawnflags & 128 != 0 {
        (*self_).flags |= FL_INACTIVE;
    }

    if (*self_).count == 0 {
        (*self_).count = 1; //default 1 use only
    }

    // FIXME: this is a hack... because delay is read in as an int, so I'm bypassing that
    // because it's too late in the project to change it and I want to be able to set
    // less than a second delays
    let mut v: f32 = 0.0;
    G_SpawnFloat(c"delay".as_ptr(), c"0".as_ptr(), &mut v);
    (*self_).delay = (v * 1000.0) as c_int; //sec to ms
    (*self_).wait *= 1000.0; //sec to ms

    G_SetOrigin(self_, &(*self_).s.origin);
    (*self_).r#use = Some(target_scriptrunner_use);
}

//==========================================================

/*QUAKED target_relay (.5 .5 .5) (-8 -8 -8) (8 8 8) RED_ONLY BLUE_ONLY RANDOM x x x x INACTIVE
This doesn't do anything by itself, but can be used as a level of indirection
between an activator and its targets.
RED_ONLY - only red team players can fire this trigger
BLUE_ONLY - only blue team players can fire this trigger
RANDOM - one one of the targeted entities will be fired, not all of them
INACTIVE - start off, has to be activated to be usable
*/
/// `void target_relay_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:464). A level-of-indirection relay between an activator and its targets,
/// gated by the `RED_ONLY`/`BLUE_ONLY`/`INACTIVE` spawnflags. Runs its `BSET_USE` behavior;
/// a `wait == -1` relay then either drops its `use` (if a script ran, so it can't be freed)
/// or schedules itself for removal via `G_FreeEntity`. With the `RANDOM` (4) spawnflag it
/// fires a single `G_PickTarget`ed entity instead of all of them. No oracle (script-hook +
/// global entity state).
///
/// # Safety
/// `self_`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn target_relay_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*self_).spawnflags & 1 != 0
        && !(*activator).client.is_null()
        && (*(*activator).client).sess.sessionTeam != TEAM_RED
    {
        return;
    }
    if (*self_).spawnflags & 2 != 0
        && !(*activator).client.is_null()
        && (*(*activator).client).sess.sessionTeam != TEAM_BLUE
    {
        return;
    }

    if (*self_).flags & FL_INACTIVE != 0 {
        //set by target_deactivate
        return;
    }

    let ranscript = G_ActivateBehavior(self_, BSET_USE);
    if (*self_).wait == -1.0 {
        //never use again
        if ranscript != QFALSE {
            //crap, can't remove!
            (*self_).r#use = None;
        } else {
            //remove
            (*self_).think = Some(G_FreeEntity);
            (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
        }
    }
    if (*self_).spawnflags & 4 != 0 {
        let ent = G_PickTarget((*self_).target);
        if !ent.is_null() && (*ent).r#use.is_some() {
            GlobalUse(ent, self_, activator);
        }
        return;
    }
    G_UseTargets(self_, activator);
}

/// `void SP_target_relay( gentity_t *self )` (g_target.c:505). Wire up the relay's `use`
/// callback and honor the spawnflag-128 "start inactive" bit. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_relay(self_: *mut gentity_t) {
    (*self_).r#use = Some(target_relay_use);
    if (*self_).spawnflags & 128 != 0 {
        (*self_).flags |= FL_INACTIVE;
    }
}

//==========================================================

/*QUAKED target_laser (0 .5 .8) (-8 -8 -8) (8 8 8) START_ON
When triggered, fires a laser.  You can either set a target or a direction.
*/
/// `void target_laser_think( gentity_t *self )` (g_target.c:334). Re-aims at `enemy`
/// (if any), traces the beam forward 2048 units, damages whatever it hits, parks the
/// trace endpoint in `s.origin2` for the client beam render, and re-fires every frame.
/// No oracle (engine-syscall trace/link plumbing).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn target_laser_think(self_: *mut gentity_t) {
    let mut end: vec3_t = [0.0; 3];
    let mut point: vec3_t = [0.0; 3];

    // if pointed at another entity, set movedir to point at it
    if !(*self_).enemy.is_null() {
        let enemy = (*self_).enemy;
        VectorMA(&(*enemy).s.origin, 0.5, &(*enemy).r.mins, &mut point);
        let point_in = point;
        VectorMA(&point_in, 0.5, &(*enemy).r.maxs, &mut point);
        VectorSubtract(&point, &(*self_).s.origin, &mut (*self_).movedir);
        VectorNormalize(&mut (*self_).movedir);
    }

    // fire forward and see what we hit
    VectorMA(&(*self_).s.origin, 2048.0, &(*self_).movedir, &mut end);

    let mut tr = trap::Trace(
        &(*self_).s.origin,
        &vec3_origin,
        &vec3_origin,
        &end,
        (*self_).s.number,
        CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE,
    );

    if tr.entityNum != 0 {
        // hurt it if we can
        G_Damage(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize),
            self_,
            (*self_).activator,
            addr_of_mut!((*self_).movedir),
            addr_of_mut!(tr.endpos),
            (*self_).damage,
            DAMAGE_NO_KNOCKBACK,
            MOD_TARGET_LASER,
        );
    }

    VectorCopy(&tr.endpos, &mut (*self_).s.origin2);

    trap::LinkEntity(self_);
    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
}

/// `void target_laser_on( gentity_t *self )` (g_target.c:364). Self-activate if no
/// activator, then start firing.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn target_laser_on(self_: *mut gentity_t) {
    if (*self_).activator.is_null() {
        (*self_).activator = self_;
    }
    target_laser_think(self_);
}

/// `void target_laser_off( gentity_t *self )` (g_target.c:371). Stop firing: unlink and
/// clear the think timer.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn target_laser_off(self_: *mut gentity_t) {
    trap::UnlinkEntity(self_);
    (*self_).nextthink = 0;
}

/// `void target_laser_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:377). Toggle the beam on/off based on whether it is currently firing.
///
/// # Safety
/// `self_`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn target_laser_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    (*self_).activator = activator;
    if (*self_).nextthink > 0 {
        target_laser_off(self_);
    } else {
        target_laser_on(self_);
    }
}

/// `void target_laser_start( gentity_t *self )` (g_target.c:386). Deferred init (runs one
/// frame after spawn): resolves the `target` enemy or bakes the firing direction from
/// `angles`, wires the use/think callbacks, defaults `damage` to 1, and starts on/off per
/// the `START_ON` spawnflag.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn target_laser_start(self_: *mut gentity_t) {
    (*self_).s.eType = ET_BEAM;

    if !(*self_).target.is_null() {
        let ent = G_Find(null_mut(), offset_of!(gentity_s, targetname), (*self_).target);
        if ent.is_null() {
            Com_Printf(&format!(
                "{} at {}: {} is a bad target\n",
                CStr::from_ptr((*self_).classname).to_string_lossy(),
                CStr::from_ptr(vtos(&(*self_).s.origin)).to_string_lossy(),
                CStr::from_ptr((*self_).target).to_string_lossy()
            ));
        }
        (*self_).enemy = ent;
    } else {
        G_SetMovedir(&mut (*self_).s.angles, &mut (*self_).movedir);
    }

    (*self_).r#use = Some(target_laser_use);
    (*self_).think = Some(target_laser_think);

    if (*self_).damage == 0 {
        (*self_).damage = 1;
    }

    if (*self_).spawnflags & 1 != 0 {
        target_laser_on(self_);
    } else {
        target_laser_off(self_);
    }
}

/// `void SP_target_laser( gentity_t *self )` (g_target.c:415). Defer real init to
/// [`target_laser_start`] one frame out, so every other entity is spawned before the
/// beam tries to resolve its target.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_laser(self_: *mut gentity_t) {
    // let everything else get spawned before we start firing
    (*self_).think = Some(target_laser_start);
    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
}

//==========================================================

/*QUAKED target_kill (.5 .5 .5) (-8 -8 -8) (8 8 8)
Kills the activator.
*/
/// `void target_kill_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_target.c:519). Runs the BSET_USE behavior, then telefrags the activator outright
/// (100000 damage, ignoring all protection). No oracle.
///
/// # Safety
/// `self_`/`activator` must point to valid `gentity_t`s; `other` is unused.
pub unsafe extern "C" fn target_kill_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);
    G_Damage(
        activator,
        null_mut(),
        null_mut(),
        null_mut(),
        null_mut(),
        100000,
        DAMAGE_NO_PROTECTION,
        MOD_TELEFRAG,
    );
}

/// `void SP_target_kill( gentity_t *self )` (g_target.c:524). Wire up the kill `use`
/// callback.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_kill(self_: *mut gentity_t) {
    (*self_).r#use = Some(target_kill_use);
}
