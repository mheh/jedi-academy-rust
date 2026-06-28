//! Port of `g_trigger.c` — the `trigger_*`/`func_timer` brush entities: volumes that
//! fire their targets when a client touches them or they are used.
//!
//! Each class is a `touch`/`use`/`think` callback (an `unsafe extern "C" fn` stored in
//! the matching `gentity_t` slot) plus an `SP_*` spawner that wires the callbacks up and
//! installs the brush model via [`InitTrigger`]. The `SP_*` functions are
//! `unsafe extern "C" fn(*mut gentity_t)` to match the (still gated) `G_CallSpawn`
//! registry's `void (*spawn)(gentity_t*)` slot.
//!
//! Landed incrementally: only the classes whose callbacks reach already-ported deps.
//! `trigger_always`, `trigger_push`, and `target_push` landed once `G_FreeEntity`/`G_Sound`
//! did; `trigger_lightningstrike` landed once `G_Damage`/`G_RadiusDamage` did;
//! `trigger_multiple`/`_once` landed once their `Touch_Multi`→`multi_trigger` Siege deps
//! (`SiegeItemRemoveOwner`/`gSiegeRoundBegun`) did. `trigger_shipboundary`
//! (`shipboundary_touch`/`_think`) and `trigger_hyperspace` (`hyperspace_touch`) landed once
//! their vehicle struct fields (`m_pVehicle`/`m_pVehicleInfo->type`/`VH_FIGHTER`) and
//! `TeleportPlayer`/`G_Damage`/`G_Find` deps were available. `trigger_teleport`
//! landed once `TeleportPlayer` did. All callbacks here are No-oracle — engine-syscall/
//! script-hook plumbing over the global level/entity state.

#![allow(non_snake_case)] // C function names (`SP_trigger_space`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro names kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::anims::{BOTH_BUTTON_HOLD, BOTH_CONSOLE1};
use crate::codemp::game::bg_misc::BG_TouchJumpPad;
use crate::codemp::game::bg_public::HYPERSPACE_TIME;
use crate::codemp::game::bg_public::{
    CS_GLOBAL_AMBIENT_SET, EF_DEAD, EF_RAG, ET_NPC, ET_PUSH_TRIGGER, ET_TELEPORT_TRIGGER, GT_SIEGE,
    HANDEXTEND_NONE, MASK_PLAYERSOLID, MOD_FALLING, MOD_SUICIDE, MOD_TRIGGER_HURT, PMF_FOLLOW,
    PM_DEAD, PM_FLOAT, PM_FREEZE, PM_NORMAL, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE,
    SETANIM_TORSO, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_public::{EF2_HYPERSPACE, HYPERSPACE_TELEPORT_FRAC};
use crate::codemp::game::bg_saga::bgSiegeClasses;
use crate::codemp::game::bg_saga_h::{SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};
use crate::codemp::game::bg_vehicles_h::VH_FIGHTER;
use crate::codemp::game::bg_weapons_h::WP_NONE;
use crate::codemp::game::g_ICARUScb::Q3_Lerp2Origin;
use crate::codemp::game::g_client::respawn;
use crate::codemp::game::g_combat::{gSiegeRoundBegun, G_Damage, G_RadiusDamage};
use crate::codemp::game::g_items::Jetpack_Off;
use crate::codemp::game::g_local::gentity_s;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_NO_PROTECTION, FL_INACTIVE, FRAMETIME};
use crate::codemp::game::g_main::{
    g_entities, g_gametype, g_gravity, level, Com_Error, Com_Printf, G_Error, G_Printf,
};
use crate::codemp::game::g_misc::TeleportPlayer;
use crate::codemp::game::g_mover::SP_func_rotating;
use crate::codemp::game::g_public_h::SVF_NOCLIENT;
use crate::codemp::game::g_public_h::{BSET_USE, Q3_INFINITE};
use crate::codemp::game::g_saga::SiegeItemRemoveOwner;
use crate::codemp::game::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString};
use crate::codemp::game::g_utils::G_Find;
use crate::codemp::game::g_utils::{
    vtos, G_EffectIndex, G_EntitySound, G_FreeEntity, G_PickTarget, G_PlayEffectID,
    G_PointInBounds, G_ScaleNetHealth, G_SetAngles, G_SetAnim, G_SetMovedir, G_SetOrigin, G_Sound,
    G_SoundIndex, G_Spawn, G_UseTargets, G_UseTargets2,
};
use crate::codemp::game::npc_utils::G_ActivateBehavior;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::VectorMA;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, AngleVectors, Distance, DotProduct, VectorAdd, VectorCompare, VectorCopy,
    VectorLengthSquared, VectorNormalize, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{crandom, Q_stricmp};
use crate::codemp::game::q_shared_h::CHAN_LOCAL;
use crate::codemp::game::q_shared_h::{
    trace_t, vec3_t, BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_USE, CHAN_AUTO, CHAN_VOICE,
    ENTITYNUM_NONE, ENTITYNUM_WORLD, ERR_DROP, MAX_CLIENTS, MAX_GENTITIES, MAX_STRING_CHARS,
    TR_LINEAR, TR_LINEAR_STOP, TR_NONLINEAR_STOP, TR_STATIONARY,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_TRIGGER;
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;
use core::mem::offset_of;

// `SP_trigger_multiple`/`_once` resolve their `team` spawn key through libc `atoi`
// (the game build pulls it from <stdlib.h>), matching the original byte-for-byte.
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
}

/// `int gTrigFallSound;` (g_trigger.c file global). Set in [`SP_trigger_hurt`] and read by
/// the (still unported) `hurt_touch` to play a falling sound.
#[no_mangle]
pub static mut gTrigFallSound: c_int = 0;

/// `void InitTrigger( gentity_t *self )` (g_trigger.c:8). Shared setup for every brush
/// trigger: derive the move direction from `angles` (if any), install the brush model,
/// stamp `CONTENTS_TRIGGER`/`SVF_NOCLIENT`, and honor the spawnflag-128 "start inactive"
/// bit. No oracle (drives `trap_SetBrushModel`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a valid `model` C string.
pub unsafe fn InitTrigger(self_: *mut gentity_t) {
    if VectorCompare(&(*self_).s.angles, &vec3_origin) == 0 {
        G_SetMovedir(&mut (*self_).s.angles, &mut (*self_).movedir);
    }

    trap::SetBrushModel(self_, &CStr::from_ptr((*self_).model).to_string_lossy());
    (*self_).r.contents = CONTENTS_TRIGGER; // replaces the -1 from trap_SetBrushModel
    (*self_).r.svFlags = SVF_NOCLIENT;

    if (*self_).spawnflags & 128 != 0 {
        (*self_).flags |= FL_INACTIVE;
    }
}

/// `void multi_wait( gentity_t *ent )` (g_trigger.c:23). The wait time has passed, so set
/// back up for another activation: clears `nextthink` to disarm the think loop. No oracle
/// (entity-state mutation only).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn multi_wait(ent: *mut gentity_t) {
    (*ent).nextthink = 0;
}

/// `void multi_trigger_run( gentity_t *ent )` (g_trigger.c:32). The trigger was just
/// activated (`ent->activator` already points at the activator so it survives a delay), so
/// after the delay time has elapsed, fire: run the `BSET_USE` script hook, swap in the
/// trigger's ambient sound set, fire the Siege team-specific `target3`/`target4`, fire the
/// normal targets, play the noise, and then re-arm — either schedule the `trigger_cleared_fire`
/// follow-up (`target2`), restart the `wait` timer (one touch per frame via `painDebounceTime`),
/// or, for `wait < 0`, make itself non-solid (a one-shot, but it can't free itself mid-touch
/// while area links are being walked). Finally stamp `aimDebounceTime` if a player touched it.
/// No oracle (entity-state mutation + script/sound/configstring plumbing over global state).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn multi_trigger_run(ent: *mut gentity_t) {
    (*ent).think = None;

    G_ActivateBehavior(ent, BSET_USE);

    if !(*ent).soundSet.is_null() && *(*ent).soundSet != 0 {
        trap::SetConfigstring(
            CS_GLOBAL_AMBIENT_SET,
            &CStr::from_ptr((*ent).soundSet).to_string_lossy(),
        );
    }

    if (*ent).genericValue4 != 0 {
        //we want to activate target3 for team1 or target4 for team2
        if (*ent).genericValue4 == SIEGETEAM_TEAM1
            && !(*ent).target3.is_null()
            && *(*ent).target3 != 0
        {
            G_UseTargets2(ent, (*ent).activator, (*ent).target3);
        } else if (*ent).genericValue4 == SIEGETEAM_TEAM2
            && !(*ent).target4.is_null()
            && *(*ent).target4 != 0
        {
            G_UseTargets2(ent, (*ent).activator, (*ent).target4);
        }

        (*ent).genericValue4 = 0;
    }

    G_UseTargets(ent, (*ent).activator);
    if (*ent).noise_index != 0 {
        G_Sound((*ent).activator, CHAN_AUTO, (*ent).noise_index);
    }

    if !(*ent).target2.is_null() && *(*ent).target2 != 0 && (*ent).wait >= 0.0 {
        (*ent).think = Some(trigger_cleared_fire);
        // C: `level.time + ent->speed` — int promotes to float, then truncates to int nextthink.
        (*ent).nextthink = ((*addr_of!(level)).time as f32 + (*ent).speed) as c_int;
    } else if (*ent).wait > 0.0 {
        if (*ent).painDebounceTime != (*addr_of!(level)).time {
            //first ent to touch it this frame
            //ent->e_ThinkFunc = thinkF_multi_wait;
            // C evaluates this in `double`: `crandom()` is a double, so `wait + random *
            // crandom()` and the `* 1000` promote to double, as does `level.time`, before
            // truncating to the int nextthink.
            (*ent).nextthink = ((*addr_of!(level)).time as f64
                + ((*ent).wait as f64 + (*ent).random as f64 * crandom()) * 1000.0)
                as c_int;
            (*ent).painDebounceTime = (*addr_of!(level)).time;
        }
    } else if (*ent).wait < 0.0 {
        // we can't just remove (self) here, because this is a touch function
        // called while looping through area links...
        (*ent).r.contents &= !CONTENTS_TRIGGER; //so the EntityContact trace doesn't have to be done against me
        (*ent).think = None;
        (*ent).r#use = None;
        //Don't remove, Icarus may barf?
        //ent->nextthink = level.time + FRAMETIME;
        //ent->think = G_FreeEntity;
    }
    if !(*ent).activator.is_null() && !(*(*ent).activator).client.is_null() {
        // mark the trigger as being touched by the player
        (*ent).aimDebounceTime = (*addr_of!(level)).time;
    }
}

/// `qboolean G_NameInTriggerClassList(char *list, char *str)` (g_trigger.c:97).
/// Determine if the class given is listed in the string using the `|` formatting:
/// walks `list` splitting on `|`, case-insensitively comparing each token against
/// `str`. Returns `QTRUE` on the first match, `QFALSE` if a `|`-separated end is
/// reached without one. Oracle-tested (pure string parsing over `Q_stricmp`).
///
/// # Safety
/// `list` and `str` must point to NUL-terminated buffers, and no token in `list` may
/// exceed `MAX_STRING_CHARS - 1` bytes (matching the C's fixed `cmp` scratch buffer).
pub unsafe fn G_NameInTriggerClassList(list: *const c_char, str: *const c_char) -> qboolean {
    let mut cmp = [0 as c_char; MAX_STRING_CHARS];
    let mut i: usize = 0;
    let mut j: usize;

    while *list.add(i) != 0 {
        j = 0;
        while *list.add(i) != 0 && *list.add(i) != b'|' as c_char {
            cmp[j] = *list.add(i);
            i += 1;
            j += 1;
        }
        cmp[j] = 0;

        if Q_stricmp(str, cmp.as_ptr()) == 0 {
            // found it
            return QTRUE;
        }
        if *list.add(i) != b'|' as c_char {
            // reached the end and never found it
            return QFALSE;
        }
        i += 1;
    }

    QFALSE
}

/// `void multi_trigger( gentity_t *ent, gentity_t *activator )` (g_trigger.c:130). The core
/// trigger-fire gate behind both [`Use_Multi`] and [`Touch_Multi`]: bails if it is already
/// queued to run, enforces the Siege round/team/class rules, handles Siege objective-item
/// delivery (`genericValue1`) and team-balance ownership (`genericValue2`), applies the
/// `nextthink`/same-frame debounce, honors `FL_INACTIVE`, and finally either schedules the
/// delayed fire or runs [`multi_trigger_run`] immediately. No oracle (entity-fire side effects
/// over global level/entity state + the Siege subsystem).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `activator` may be null (only dereferenced after
/// the relevant null checks).
pub unsafe fn multi_trigger(ent: *mut gentity_t, activator: *mut gentity_t) {
    let mut haltTrigger: qboolean = QFALSE;

    if let Some(think) = (*ent).think {
        if core::ptr::fn_addr_eq(
            think,
            multi_trigger_run as unsafe extern "C" fn(*mut gentity_t),
        ) {
            //already triggered, just waiting to run
            return;
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE && *addr_of!(gSiegeRoundBegun) == QFALSE {
        //nothing can be used til the round starts.
        return;
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && !activator.is_null()
        && !(*activator).client.is_null()
        && (*ent).alliedTeam != 0
        && (*(*activator).client).sess.sessionTeam != (*ent).alliedTeam
    {
        //this team can't activate this trigger.
        return;
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && !(*ent).idealclass.is_null()
        && *(*ent).idealclass != 0
    {
        //only certain classes can activate it
        if activator.is_null()
            || (*activator).client.is_null()
            || (*(*activator).client).siegeClass < 0
        {
            //no class
            return;
        }

        if G_NameInTriggerClassList(
            bgSiegeClasses[(*(*activator).client).siegeClass as usize]
                .name
                .as_ptr(),
            (*ent).idealclass,
        ) == QFALSE
        {
            //wasn't in the list
            return;
        }
    }

    if (*addr_of!(g_gametype)).integer == GT_SIEGE && (*ent).genericValue1 != 0 {
        haltTrigger = QTRUE;

        if !activator.is_null()
            && !(*activator).client.is_null()
            && (*(*activator).client).holdingObjectiveItem != 0
            && !(*ent).targetname.is_null()
            && *(*ent).targetname != 0
        {
            let objItem = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*(*activator).client).holdingObjectiveItem as usize);

            if !objItem.is_null() && (*objItem).inuse != QFALSE {
                if !(*objItem).goaltarget.is_null()
                    && *(*objItem).goaltarget != 0
                    && Q_stricmp((*ent).targetname, (*objItem).goaltarget) == 0
                {
                    if (*objItem).genericValue7 != (*(*activator).client).sess.sessionTeam {
                        //The carrier of the item is not on the team which disallows objective scoring for it
                        if !(*objItem).target3.is_null() && *(*objItem).target3 != 0 {
                            //if it has a target3, fire it off instead of using the trigger
                            G_UseTargets2(objItem, objItem, (*objItem).target3);

                            //3-24-03 - want to fire off the target too I guess, if we have one.
                            if !(*ent).targetname.is_null() && *(*ent).targetname != 0 {
                                haltTrigger = QFALSE;
                            }
                        } else {
                            haltTrigger = QFALSE;
                        }

                        //now that the item has been delivered, it can go away.
                        SiegeItemRemoveOwner(objItem, activator);
                        (*objItem).nextthink = 0;
                        (*objItem).neverFree = QFALSE;
                        G_FreeEntity(objItem);
                    }
                }
            }
        }
    } else if (*ent).genericValue1 != 0 {
        //Never activate in non-siege gametype I guess.
        return;
    }

    if (*ent).genericValue2 != 0 {
        //has "teambalance" property
        let mut i = 0;
        let mut team1ClNum = 0;
        let mut team2ClNum = 0;
        let owningTeam = (*ent).genericValue3;
        let newOwningTeam;
        let numEnts;
        let mut entityList = [0i32; MAX_GENTITIES];

        if (*addr_of!(g_gametype)).integer != GT_SIEGE {
            return;
        }

        if (*activator).client.is_null()
            || ((*(*activator).client).sess.sessionTeam != SIEGETEAM_TEAM1
                && (*(*activator).client).sess.sessionTeam != SIEGETEAM_TEAM2)
        {
            //activator must be a valid client to begin with
            return;
        }

        //Count up the number of clients standing within the bounds of the trigger and the number of them on each team
        numEnts = trap::EntitiesInBox(&(*ent).r.absmin, &(*ent).r.absmax, &mut entityList);
        while i < numEnts {
            if entityList[i as usize] < MAX_CLIENTS as c_int {
                //only care about clients
                let cl = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add(entityList[i as usize] as usize);

                //the client is valid
                if (*cl).inuse != QFALSE
                    && !(*cl).client.is_null()
                    && ((*(*cl).client).sess.sessionTeam == SIEGETEAM_TEAM1
                        || (*(*cl).client).sess.sessionTeam == SIEGETEAM_TEAM2)
                    && (*cl).health > 0
                    && (*(*cl).client).ps.eFlags & EF_DEAD == 0
                {
                    //See which team he's on
                    if (*(*cl).client).sess.sessionTeam == SIEGETEAM_TEAM1 {
                        team1ClNum += 1;
                    } else {
                        team2ClNum += 1;
                    }
                }
            }
            i += 1;
        }

        if team1ClNum == 0 && team2ClNum == 0 {
            //no one in the box? How did we get activated? Oh well.
            return;
        }

        if team1ClNum == team2ClNum {
            //if equal numbers the ownership will remain the same as it is now
            return;
        }

        //decide who owns it now
        if team1ClNum > team2ClNum {
            newOwningTeam = SIEGETEAM_TEAM1;
        } else {
            newOwningTeam = SIEGETEAM_TEAM2;
        }

        if owningTeam == newOwningTeam {
            //it's the same one it already was, don't care then.
            return;
        }

        //Set the new owner and set the variable which will tell us to activate a team-specific target
        (*ent).genericValue3 = newOwningTeam;
        (*ent).genericValue4 = newOwningTeam;
    }

    if haltTrigger != QFALSE {
        //This is an objective trigger and the activator is not carrying an objective item that matches the targetname.
        return;
    }

    if (*ent).nextthink > (*addr_of!(level)).time {
        if (*ent).spawnflags & 2048 != 0 {
            // MULTIPLE - allow multiple entities to touch this trigger in a single frame
            if (*ent).painDebounceTime != 0 && (*ent).painDebounceTime != (*addr_of!(level)).time {
                //this should still allow subsequent ents to fire this trigger in the current frame
                return; // can't retrigger until the wait is over
            }
        } else {
            return;
        }
    }

    // if the player has already activated this trigger this frame
    if !activator.is_null()
        && (*activator).s.number == 0
        && (*ent).aimDebounceTime == (*addr_of!(level)).time
    {
        return;
    }

    if (*ent).flags & FL_INACTIVE != 0 {
        //Not active at this time
        return;
    }

    (*ent).activator = activator;

    if (*ent).delay != 0 && (*ent).painDebounceTime < ((*addr_of!(level)).time + (*ent).delay) {
        //delay before firing trigger
        (*ent).think = Some(multi_trigger_run);
        (*ent).nextthink = (*addr_of!(level)).time + (*ent).delay;
        (*ent).painDebounceTime = (*addr_of!(level)).time;
    } else {
        multi_trigger_run(ent);
    }
}

/// `void Use_Multi( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_trigger.c:346). The `use` callback for `trigger_multiple`/`_once`: forwards to
/// [`multi_trigger`] with the activator. No oracle (delegates to the entity-fire path).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other` is unused; `activator` may be null.
pub unsafe extern "C" fn Use_Multi(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    multi_trigger(ent, activator);
}

/// `void Touch_Multi( gentity_t *self, gentity_t *other, trace_t *trace )` (g_trigger.c:353).
/// The touch callback for `trigger_multiple`/`_once`: rejects non-clients and inactive
/// triggers, applies the team/NPC/script-targetname gates, then the spawnflag conditions —
/// FACING (within 45 deg), USE_BUTTON (incl. the hacking hold timer + Siege class check),
/// FIRE_BUTTON, and the radius head-in-volume hiding bonus — driving the use-button hold anim,
/// and finally fires via the `trigger_cleared_fire` follow-up or [`multi_trigger`]. No oracle
/// (entity-fire side effects + anim/global-state plumbing).
///
/// # Safety
/// `self_`/`other` must point to valid `gentity_t`s; `trace` is unused.
pub unsafe extern "C" fn Touch_Multi(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*other).client.is_null() {
        return;
    }

    if (*self_).flags & FL_INACTIVE != 0 {
        //set by target_deactivate
        return;
    }

    if (*self_).alliedTeam != 0 {
        if (*(*other).client).sess.sessionTeam != (*self_).alliedTeam {
            return;
        }
    }

    // moved to just above multi_trigger because up here it just checks if the trigger is not being touched
    // we want it to check any conditions set on the trigger, if one of those isn't met, the trigger is considered to be "cleared"
    //	if ( self->e_ThinkFunc == thinkF_trigger_cleared_fire )
    //	{//We're waiting to fire our target2 first
    //		self->nextthink = level.time + self->speed;
    //		return;
    //	}

    if (*self_).spawnflags & 1 != 0 {
        if (*other).s.eType == ET_NPC {
            return;
        }
    } else {
        if (*self_).spawnflags & 16 != 0 {
            //NPCONLY
            if (*other).NPC.is_null() {
                return;
            }
        }

        if !(*self_).NPC_targetname.is_null() && *(*self_).NPC_targetname != 0 {
            if !(*other).script_targetname.is_null() && *(*other).script_targetname != 0 {
                if Q_stricmp((*self_).NPC_targetname, (*other).script_targetname) != 0 {
                    //not the right guy to fire me off
                    return;
                }
            } else {
                return;
            }
        }
    }

    if (*self_).spawnflags & 2 != 0 {
        //FACING
        let mut forward: vec3_t = [0.0; 3];

        AngleVectors(
            &(*(*other).client).ps.viewangles,
            Some(&mut forward),
            None,
            None,
        );

        if DotProduct(&(*self_).movedir, &forward) < 0.5 {
            //Not Within 45 degrees
            return;
        }
    }

    if (*self_).spawnflags & 4 != 0 {
        //USE_BUTTON
        if (*(*other).client).pers.cmd.buttons & BUTTON_USE == 0 {
            //not pressing use button
            return;
        }

        if ((*(*other).client).ps.weaponTime > 0
            && (*(*other).client).ps.torsoAnim != BOTH_BUTTON_HOLD
            && (*(*other).client).ps.torsoAnim != BOTH_CONSOLE1)
            || (*other).health < 1
            || (*(*other).client).ps.pm_flags & PMF_FOLLOW != 0
            || (*(*other).client).sess.sessionTeam == TEAM_SPECTATOR
            || (*(*other).client).ps.forceHandExtend != HANDEXTEND_NONE
        {
            //player has to be free of other things to use.
            return;
        }

        if (*self_).genericValue7 != 0 {
            //we have to be holding the use key in this trigger for x milliseconds before firing
            if (*addr_of!(g_gametype)).integer == GT_SIEGE
                && !(*self_).idealclass.is_null()
                && *(*self_).idealclass != 0
            {
                //only certain classes can activate it
                if other.is_null() || (*other).client.is_null() || (*(*other).client).siegeClass < 0
                {
                    //no class
                    return;
                }

                if G_NameInTriggerClassList(
                    bgSiegeClasses[(*(*other).client).siegeClass as usize]
                        .name
                        .as_ptr(),
                    (*self_).idealclass,
                ) == QFALSE
                {
                    //wasn't in the list
                    return;
                }
            }

            if G_PointInBounds(
                &(*(*other).client).ps.origin,
                &(*self_).r.absmin,
                &(*self_).r.absmax,
            ) == QFALSE
            {
                return;
            } else if (*(*other).client).isHacking != (*self_).s.number
                && (*other).s.number < MAX_CLIENTS as c_int
            {
                //start the hack
                (*(*other).client).isHacking = (*self_).s.number;
                VectorCopy(
                    &(*(*other).client).ps.viewangles,
                    &mut (*(*other).client).hackingAngles,
                );
                (*(*other).client).ps.hackingTime =
                    (*addr_of!(level)).time + (*self_).genericValue7;
                (*(*other).client).ps.hackingBaseTime = (*self_).genericValue7;
                if (*(*other).client).ps.hackingBaseTime > 60000 {
                    //don't allow a bit overflow
                    (*(*other).client).ps.hackingTime = (*addr_of!(level)).time + 60000;
                    (*(*other).client).ps.hackingBaseTime = 60000;
                }
                return;
            } else if (*(*other).client).ps.hackingTime < (*addr_of!(level)).time {
                //finished with the hack, reset the hacking values and let it fall through
                (*(*other).client).isHacking = 0; //can't hack a client
                (*(*other).client).ps.hackingTime = 0;
            } else {
                //hack in progress
                return;
            }
        }
    }

    if (*self_).spawnflags & 8 != 0 {
        //FIRE_BUTTON
        if (*(*other).client).pers.cmd.buttons & BUTTON_ATTACK == 0
            && (*(*other).client).pers.cmd.buttons & BUTTON_ALT_ATTACK == 0
        {
            //not pressing fire button or altfire button
            return;
        }
    }

    if (*self_).radius != 0.0 {
        let mut eyeSpot: vec3_t = [0.0; 3];

        //Only works if your head is in it, but we allow leaning out
        //NOTE: We don't use CalcEntitySpot SPOT_HEAD because we don't want this
        //to be reliant on the physical model the player uses.
        VectorCopy(&(*(*other).client).ps.origin, &mut eyeSpot);
        eyeSpot[2] += (*(*other).client).ps.viewheight as f32;

        if G_PointInBounds(&eyeSpot, &(*self_).r.absmin, &(*self_).r.absmax) != QFALSE {
            if (*(*other).client).pers.cmd.buttons & BUTTON_ATTACK == 0
                && (*(*other).client).pers.cmd.buttons & BUTTON_ALT_ATTACK == 0
            {
                //not attacking, so hiding bonus
                /*
                //FIXME:  should really have sound events clear the hiddenDist
                other->client->hiddenDist = self->radius;
                //NOTE: movedir HAS to be normalized!
                if ( VectorLength( self->movedir ) )
                {//They can only be hidden from enemies looking in this direction
                    VectorCopy( self->movedir, other->client->hiddenDir );
                }
                else
                {
                    VectorClear( other->client->hiddenDir );
                }
                */
                //Not using this, at least not yet.
            }
        }
    }

    if (*self_).spawnflags & 4 != 0 {
        //USE_BUTTON
        if (*(*other).client).ps.torsoAnim != BOTH_BUTTON_HOLD
            && (*(*other).client).ps.torsoAnim != BOTH_CONSOLE1
        {
            G_SetAnim(
                other,
                null_mut(),
                SETANIM_TORSO,
                BOTH_BUTTON_HOLD,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
        } else {
            (*(*other).client).ps.torsoTimer = 500;
        }
        (*(*other).client).ps.weaponTime = (*(*other).client).ps.torsoTimer;
    }

    if let Some(think) = (*self_).think {
        if core::ptr::fn_addr_eq(
            think,
            trigger_cleared_fire as unsafe extern "C" fn(*mut gentity_t),
        ) {
            //We're waiting to fire our target2 first
            (*self_).nextthink = ((*addr_of!(level)).time as f32 + (*self_).speed) as c_int;
            return;
        }
    }

    multi_trigger(self_, other);
}

/*QUAKED trigger_multiple (.1 .5 .1) ? CLIENTONLY FACING USE_BUTTON FIRE_BUTTON NPCONLY x x INACTIVE MULTIPLE
CLIENTONLY - only a player can trigger this by touch
FACING - Won't fire unless triggering ent's view angles are within 45 degrees of trigger's angles (in addition to any other conditions)
USE_BUTTON - Won't fire unless player is in it and pressing use button (in addition to any other conditions)
FIRE_BUTTON - Won't fire unless player/NPC is in it and pressing fire button (in addition to any other conditions)
NPCONLY - only non-player NPCs can trigger this by touch
INACTIVE - Start off, has to be activated to be touchable/usable
MULTIPLE - multiple entities can touch this trigger in a single frame *and* if needed, the trigger can have a wait of > 0

"wait"		Seconds between triggerings, 0 default, number < 0 means one time only.
"random"	wait variance, default is 0
"delay"		how many seconds to wait to fire targets after tripped
"hiderange" As long as NPC's head is in this trigger, NPCs out of this hiderange cannot see him.  If you set an angle on the trigger, they're only hidden from enemies looking in that direction.  the player's crouch viewheight is 36, his standing viewheight is 54.  So a trigger thast should hide you when crouched but not standing should be 48 tall.
"target2"	The trigger will fire this only when the trigger has been activated and subsequently 'cleared'( once any of the conditions on the trigger have not been satisfied).  This will not fire the "target" more than once until the "target2" is fired (trigger field is 'cleared')
"speed"		How many seconds to wait to fire the target2, default is 1
"noise"		Sound to play when the trigger fires (plays at activator's origin)
"NPC_targetname"  Only the NPC with this NPC_targetname fires this trigger

Variable sized repeatable trigger.  Must be targeted at one or more entities.
so, the basic time between firing is a random time between
(wait - random) and (wait + random)

"team" - If set, only this team can trip this trigger
    0 - any
    1 - red
    2 - blue

"soundSet"	Ambient sound set to play when this trigger is activated

usetime		-	If specified (in milliseconds) along with the USE_BUTTON flag, will
                require a client to hold the use key for x amount of ms before firing.

Applicable only during Siege gametype:
teamuser	-	if 1, team 2 can't use this. If 2, team 1 can't use this.
siegetrig	-	if non-0, can only be activated by players carrying a misc_siege_item
                which is associated with this trigger by the item's goaltarget value.
teambalance	-	if non-0, is "owned" by the last team that activated. Can only be activated
                by the other team if the number of players on the other team inside	the
                trigger outnumber the number of players on the owning team inside the
                trigger.
target3		-	fire when activated by team1
target4		-	fire when activated by team2

idealclass	-	Can only be used by this class/these classes. You can specify use by
                multiple classes with the use of |, e.g.:
                "Imperial Medic|Imperial Assassin|Imperial Demolitionist"
*/
/// `void SP_trigger_multiple( gentity_t *ent )` (g_trigger.c:610). Spawn-initializer for
/// `trigger_multiple`: reads the `noise`/`usetime`/Siege/`delay` spawn keys, validates the
/// `wait`/`random` pairing, scales `delay`/`speed` into milliseconds, wires the
/// [`Touch_Multi`]/[`Use_Multi`] callbacks, resolves the `team` restriction into `alliedTeam`,
/// and installs the brush model via [`InitTrigger`]. No oracle (spawn-key plumbing over global
/// state + brush-model/link syscalls).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_trigger_multiple(ent: *mut gentity_t) {
    let mut s: *mut c_char = null_mut();
    if G_SpawnString(c"noise".as_ptr(), c"".as_ptr(), &mut s) != QFALSE {
        if !s.is_null() && *s != 0 {
            (*ent).noise_index = G_SoundIndex(&CStr::from_ptr(s).to_string_lossy());
        } else {
            (*ent).noise_index = 0;
        }
    }

    G_SpawnInt(
        c"usetime".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue7,
    );

    //For siege gametype
    G_SpawnInt(
        c"siegetrig".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue1,
    );
    G_SpawnInt(
        c"teambalance".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue2,
    );

    G_SpawnInt(c"delay".as_ptr(), c"0".as_ptr(), &mut (*ent).delay);

    if (*ent).wait > 0.0 && (*ent).random >= (*ent).wait {
        (*ent).random = (*ent).wait - FRAMETIME as f32;
        Com_Printf("^3trigger_multiple has random >= wait\n");
    }

    (*ent).delay *= 1000; //1 = 1 msec, 1000 = 1 sec
    if (*ent).speed == 0.0 && !(*ent).target2.is_null() && *(*ent).target2 != 0 {
        (*ent).speed = 1000.0;
    } else {
        (*ent).speed *= 1000.0;
    }

    (*ent).touch = Some(Touch_Multi);
    (*ent).r#use = Some(Use_Multi);

    if !(*ent).team.is_null() && *(*ent).team != 0 {
        (*ent).alliedTeam = atoi((*ent).team);
        (*ent).team = null_mut();
    }

    InitTrigger(ent);
    trap::LinkEntity(ent);
}

/*QUAKED trigger_once (.5 1 .5) ? CLIENTONLY FACING USE_BUTTON FIRE_BUTTON x x x INACTIVE MULTIPLE
CLIENTONLY - only a player can trigger this by touch
FACING - Won't fire unless triggering ent's view angles are within 45 degrees of trigger's angles (in addition to any other conditions)
USE_BUTTON - Won't fire unless player is in it and pressing use button (in addition to any other conditions)
FIRE_BUTTON - Won't fire unless player/NPC is in it and pressing fire button (in addition to any other conditions)
INACTIVE - Start off, has to be activated to be touchable/usable
MULTIPLE - multiple entities can touch this trigger in a single frame *and* if needed, the trigger can have a wait of > 0

"random"	wait variance, default is 0
"delay"		how many seconds to wait to fire targets after tripped
Variable sized repeatable trigger.  Must be targeted at one or more entities.
so, the basic time between firing is a random time between
(wait - random) and (wait + random)
"noise"		Sound to play when the trigger fires (plays at activator's origin)
"NPC_targetname"  Only the NPC with this NPC_targetname fires this trigger

"team" - If set, only this team can trip this trigger
    0 - any
    1 - red
    2 - blue

"soundSet"	Ambient sound set to play when this trigger is activated

usetime		-	If specified (in milliseconds) along with the USE_BUTTON flag, will
                require a client to hold the use key for x amount of ms before firing.

Applicable only during Siege gametype:
teamuser - if 1, team 2 can't use this. If 2, team 1 can't use this.
siegetrig - if non-0, can only be activated by players carrying a misc_siege_item
            which is associated with this trigger by the item's goaltarget value.

idealclass	-	Can only be used by this class/these classes. You can specify use by
                multiple classes with the use of |, e.g.:
                "Imperial Medic|Imperial Assassin|Imperial Demolitionist"
*/
/// `void SP_trigger_once( gentity_t *ent )` (g_trigger.c:697). Spawn-initializer for
/// `trigger_once`: like [`SP_trigger_multiple`] but forces `wait = -1` (a one-shot) and has no
/// `usetime`-scaling/`target2`-speed handling — reads `noise`/`usetime`/Siege/`delay`, forces
/// the one-shot wait, wires the [`Touch_Multi`]/[`Use_Multi`] callbacks, resolves the `team`
/// restriction into `alliedTeam`, scales `delay` to milliseconds, and installs the brush model.
/// No oracle (spawn-key plumbing over global state + brush-model/link syscalls).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_trigger_once(ent: *mut gentity_t) {
    let mut s: *mut c_char = null_mut();
    if G_SpawnString(c"noise".as_ptr(), c"".as_ptr(), &mut s) != QFALSE {
        if !s.is_null() && *s != 0 {
            (*ent).noise_index = G_SoundIndex(&CStr::from_ptr(s).to_string_lossy());
        } else {
            (*ent).noise_index = 0;
        }
    }

    G_SpawnInt(
        c"usetime".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue7,
    );

    //For siege gametype
    G_SpawnInt(
        c"siegetrig".as_ptr(),
        c"0".as_ptr(),
        &mut (*ent).genericValue1,
    );

    G_SpawnInt(c"delay".as_ptr(), c"0".as_ptr(), &mut (*ent).delay);

    (*ent).wait = -1.0;

    (*ent).touch = Some(Touch_Multi);
    (*ent).r#use = Some(Use_Multi);

    if !(*ent).team.is_null() && *(*ent).team != 0 {
        (*ent).alliedTeam = atoi((*ent).team);
        (*ent).team = null_mut();
    }

    (*ent).delay *= 1000; //1 = 1 msec, 1000 = 1 sec

    InitTrigger(ent);
    trap::LinkEntity(ent);
}

//==========================================================

/// `#define INITIAL_SUFFOCATION_DELAY 500` (g_trigger.c:1441) — half a second before a
/// freshly-entered client starts taking suffocation damage.
const INITIAL_SUFFOCATION_DELAY: c_int = 500;

/// `void space_touch( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (g_trigger.c:1442). Marks human clients standing inside the volume as "in space" so the
/// per-frame logic suffocates them and zeroes their gravity. A player riding a vehicle whose
/// `hideRider` is set is protected (the cockpit shields them). No oracle (reads the global
/// level/entity array).
///
/// # Safety
/// `self_`/`other` must point to valid `gentity_t`s; `trace` is unused.
pub unsafe extern "C" fn space_touch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if other.is_null() || (*other).inuse == QFALSE || (*other).client.is_null() {
        // NOTE: we need vehicles to know this, too...
        return;
    }

    if (*other).s.number < MAX_CLIENTS as c_int // player
        && (*(*other).client).ps.m_iVehicleNum != 0 // in a vehicle
        && (*(*other).client).ps.m_iVehicleNum >= MAX_CLIENTS as c_int
    {
        // a player client inside a vehicle
        let veh = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add((*(*other).client).ps.m_iVehicleNum as usize);

        if (*veh).inuse != QFALSE
            && !(*veh).client.is_null()
            && !(*veh).m_pVehicle.is_null()
            && (*(*(*veh).m_pVehicle).m_pVehicleInfo).hideRider != QFALSE
        {
            // if they are "inside" a vehicle, then let that protect them from THE HORRORS OF SPACE.
            (*(*other).client).inSpaceSuffocation = 0;
            (*(*other).client).inSpaceIndex = ENTITYNUM_NONE;
            return;
        }
    }

    if G_PointInBounds(
        &(*(*other).client).ps.origin,
        &(*self_).r.absmin,
        &(*self_).r.absmax,
    ) == QFALSE
    {
        // his origin must be inside the trigger
        return;
    }

    if (*(*other).client).inSpaceIndex == 0 || (*(*other).client).inSpaceIndex == ENTITYNUM_NONE {
        // freshly entering space
        (*(*other).client).inSpaceSuffocation = (*addr_of!(level)).time + INITIAL_SUFFOCATION_DELAY;
    }

    (*(*other).client).inSpaceIndex = (*self_).s.number;
}

/*QUAKED trigger_space (.5 .5 .5) ?
causes human clients to suffocate and have no gravity.
*/
/// `void SP_trigger_space( gentity_t *self )` (g_trigger.c:1484).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_trigger_space(self_: *mut gentity_t) {
    InitTrigger(self_);
    (*self_).r.contents = CONTENTS_TRIGGER;

    (*self_).touch = Some(space_touch);

    trap::LinkEntity(self_);
}

//==========================================================

/*QUAKED func_timer (0.3 0.1 0.6) (-8 -8 -8) (8 8 8) START_ON
This should be renamed trigger_timer...
Repeatedly fires its targets.
Can be turned on or off by using.

"wait"			base time between triggering all targets, default is 1
"random"		wait variance, default is 0
so, the basic time between firing is a random time between
(wait - random) and (wait + random)
*/
/// `void func_timer_think( gentity_t *self )` (g_trigger.c:1753). Fires the timer's targets,
/// then schedules the next firing a random `wait +/- random` seconds out. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn func_timer_think(self_: *mut gentity_t) {
    G_UseTargets(self_, (*self_).activator);
    // set time before next firing — the C evaluates this in `double` (crandom() is double,
    // the `1000` promotes), then truncates to the int nextthink.
    (*self_).nextthink = ((*addr_of!(level)).time as f64
        + 1000.0 * ((*self_).wait as f64 + crandom() * (*self_).random as f64))
        as c_int;
}

/// `void func_timer_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_trigger.c:1759). Toggles the timer: if running, stop; otherwise fire once via
/// [`func_timer_think`] to start the cycle. No oracle (drives the script hook).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other` is unused.
pub unsafe extern "C" fn func_timer_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    (*self_).activator = activator;

    G_ActivateBehavior(self_, BSET_USE);

    // if on, turn it off
    if (*self_).nextthink != 0 {
        (*self_).nextthink = 0;
        return;
    }

    // turn it on
    func_timer_think(self_);
}

/// `void SP_func_timer( gentity_t *self )` (g_trigger.c:1774).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_func_timer(self_: *mut gentity_t) {
    G_SpawnFloat(c"random".as_ptr(), c"1".as_ptr(), &mut (*self_).random);
    G_SpawnFloat(c"wait".as_ptr(), c"1".as_ptr(), &mut (*self_).wait);

    (*self_).r#use = Some(func_timer_use);
    (*self_).think = Some(func_timer_think);

    if (*self_).random >= (*self_).wait {
        (*self_).random = (*self_).wait - 1.0; // NOTE: was - FRAMETIME, but FRAMETIME is in
                                               // msec (100) and these numbers are in *seconds*!
        G_Printf(&format!(
            "func_timer at {} has random >= wait\n",
            CStr::from_ptr(vtos(&(*self_).s.origin)).to_string_lossy()
        ));
    }

    if (*self_).spawnflags & 1 != 0 {
        (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
        (*self_).activator = self_;
    }

    (*self_).r.svFlags = SVF_NOCLIENT;
}

//==========================================================

/// `void trigger_always_think( gentity_t *ent )` (g_trigger.c:875). Fire the targets once,
/// then remove the trigger. No oracle (script-hook plumbing).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn trigger_always_think(ent: *mut gentity_t) {
    G_UseTargets(ent, ent);
    G_FreeEntity(ent);
}

/*QUAKED trigger_always (.5 .5 .5) (-8 -8 -8) (8 8 8)
This trigger will always fire.  It is activated by the world.
*/
/// `void SP_trigger_always( gentity_t *ent )` (g_trigger.c:883). A one-shot trigger that
/// fires its targets shortly after the level starts (the 300ms delay ensures its targets are
/// spawned), via [`trigger_always_think`]. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_trigger_always(ent: *mut gentity_t) {
    // we must have some delay to make sure our use targets are present
    (*ent).nextthink = (*addr_of!(level)).time + 300;
    (*ent).think = Some(trigger_always_think);
}

//==========================================================
// trigger_push / target_push

// PUSH_* spawnflag bits (g_trigger.c:898-902).
//trigger_push
const PUSH_LINEAR: c_int = 4;
const PUSH_RELATIVE: c_int = 16;
const PUSH_MULTIPLE: c_int = 2048;
//target_push
const PUSH_CONSTANT: c_int = 2;

/// `void trigger_push_touch( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (g_trigger.c:904). Push a touching entity along the precomputed `s.origin2` velocity. A
/// non-`PUSH_LINEAR` trigger is a classic jump pad (delegates to [`BG_TouchJumpPad`]); the
/// linear variant honors the `wait`/`MULTIPLE` debounce and supports `RELATIVE`/`LINEAR`
/// direction modes for both clients and free-moving entities. No oracle (mutates client/entity
/// trajectory state).
///
/// # Safety
/// `self_`/`other` must point to valid `gentity_t`s; `trace` is unused.
pub unsafe extern "C" fn trigger_push_touch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*self_).flags & FL_INACTIVE != 0 {
        //set by target_deactivate
        return;
    }

    if (*self_).spawnflags & PUSH_LINEAR == 0 {
        //normal throw
        if (*other).client.is_null() {
            return;
        }
        BG_TouchJumpPad(&mut (*(*other).client).ps, &(*self_).s);
        return;
    }

    //linear
    if ((*addr_of!(level)).time as f32) < (*self_).painDebounceTime as f32 + (*self_).wait {
        // normal 'wait' check
        if (*self_).spawnflags & PUSH_MULTIPLE != 0 {
            // MULTIPLE - allow multiple entities to touch this trigger in one frame
            if (*self_).painDebounceTime != 0 && (*addr_of!(level)).time > (*self_).painDebounceTime
            {
                // if we haven't reached the next frame continue to let ents touch the trigger
                return;
            }
        } else {
            // only allowing one ent per frame to touch trigger
            return;
        }
    }

    if (*other).client.is_null() {
        if (*other).s.pos.trType != TR_STATIONARY
            && (*other).s.pos.trType != TR_LINEAR_STOP
            && (*other).s.pos.trType != TR_NONLINEAR_STOP
            && VectorLengthSquared(&(*other).s.pos.trDelta) != 0.0
        {
            //already moving
            VectorCopy(&(*other).r.currentOrigin, &mut (*other).s.pos.trBase);
            VectorCopy(&(*self_).s.origin2, &mut (*other).s.pos.trDelta);
            (*other).s.pos.trTime = (*addr_of!(level)).time;
        }
        return;
    }

    if (*(*other).client).ps.pm_type != PM_NORMAL
        && (*(*other).client).ps.pm_type != PM_DEAD
        && (*(*other).client).ps.pm_type != PM_FREEZE
    {
        return;
    }

    if (*self_).spawnflags & PUSH_RELATIVE != 0 {
        //relative, dir to it * speed
        let mut dir: vec3_t = [0.0; 3];
        VectorSubtract(&(*self_).s.origin2, &(*other).r.currentOrigin, &mut dir);
        if (*self_).speed != 0.0 {
            VectorNormalize(&mut dir);
            let d = dir;
            VectorScale(&d, (*self_).speed, &mut dir);
        }
        VectorCopy(&dir, &mut (*(*other).client).ps.velocity);
    } else if (*self_).spawnflags & PUSH_LINEAR != 0 {
        //linear dir * speed
        VectorScale(
            &(*self_).s.origin2,
            (*self_).speed,
            &mut (*(*other).client).ps.velocity,
        );
    } else {
        VectorCopy(&(*self_).s.origin2, &mut (*(*other).client).ps.velocity);
    }

    if (*self_).wait == -1.0 {
        (*self_).touch = None;
    } else if (*self_).wait > 0.0 {
        (*self_).painDebounceTime = (*addr_of!(level)).time;
    }
}

/// `void AimAtTarget( gentity_t *self )` (g_trigger.c:1042). Compute `s.origin2` (the push
/// velocity) from the trigger center to its `target`. `trigger_push`/`target_push` with the
/// relative/linear/constant spawnflags get a straight direction; otherwise solve a ballistic
/// arc whose apogee hits the target (using `g_gravity`). Frees self if it has no valid target.
/// No oracle (entity lookup + global state).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn AimAtTarget(self_: *mut gentity_t) {
    let mut origin: vec3_t = [0.0; 3];

    VectorAdd(&(*self_).r.absmin, &(*self_).r.absmax, &mut origin);
    let o = origin;
    VectorScale(&o, 0.5, &mut origin);

    let ent = G_PickTarget((*self_).target);
    if ent.is_null() {
        G_FreeEntity(self_);
        return;
    }

    if !(*self_).classname.is_null() && Q_stricmp(c"trigger_push".as_ptr(), (*self_).classname) == 0
    {
        if (*self_).spawnflags & PUSH_RELATIVE != 0 {
            //relative, not an arc or linear
            VectorCopy(&(*ent).r.currentOrigin, &mut (*self_).s.origin2);
            return;
        } else if (*self_).spawnflags & PUSH_LINEAR != 0 {
            //linear, not an arc
            VectorSubtract(&(*ent).r.currentOrigin, &origin, &mut (*self_).s.origin2);
            VectorNormalize(&mut (*self_).s.origin2);
            return;
        }
    }

    if !(*self_).classname.is_null() && Q_stricmp(c"target_push".as_ptr(), (*self_).classname) == 0
    {
        if (*self_).spawnflags & PUSH_CONSTANT != 0 {
            VectorSubtract(
                &(*ent).s.origin,
                &(*self_).s.origin,
                &mut (*self_).s.origin2,
            );
            VectorNormalize(&mut (*self_).s.origin2);
            let o2 = (*self_).s.origin2;
            VectorScale(&o2, (*self_).speed, &mut (*self_).s.origin2);
            return;
        }
    }

    let height = (*ent).s.origin[2] - origin[2];
    let gravity = (*addr_of!(g_gravity)).value;
    // C: sqrt( height / ( .5 * gravity ) ) — the `.5` is a double literal so the whole
    // expression evaluates in double, then truncates into the float `time`.
    let time = ((height as f64) / (0.5 * gravity as f64)).sqrt() as f32;
    if time == 0.0 {
        G_FreeEntity(self_);
        return;
    }

    // set s.origin2 to the push velocity
    VectorSubtract(&(*ent).s.origin, &origin, &mut (*self_).s.origin2);
    (*self_).s.origin2[2] = 0.0;
    let dist = VectorNormalize(&mut (*self_).s.origin2);

    let forward = dist / time;
    let o2 = (*self_).s.origin2;
    VectorScale(&o2, forward, &mut (*self_).s.origin2);

    (*self_).s.origin2[2] = time * gravity;
}

/*QUAKED trigger_push (.5 .5 .5) ? x x LINEAR x RELATIVE x x INACTIVE MULTIPLE
Must point at a target_position, which will be the apex of the leap.
This will be client side predicted, unlike target_push
*/
/// `void SP_trigger_push( gentity_t *self )` (g_trigger.c:1115). Set up a client-predicted
/// push volume: install the brush model, clear `SVF_NOCLIENT` (this one is networked), mark it
/// `ET_PUSH_TRIGGER`, and schedule [`AimAtTarget`] to compute the launch velocity once targets
/// exist. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_trigger_push(self_: *mut gentity_t) {
    InitTrigger(self_);

    // unlike other triggers, we need to send this one to the client
    (*self_).r.svFlags &= !SVF_NOCLIENT;

    // make sure the client precaches this sound
    G_SoundIndex("sound/weapons/force/jump.wav");

    (*self_).s.eType = ET_PUSH_TRIGGER;

    if (*self_).spawnflags & 2 == 0 {
        //start on
        (*self_).touch = Some(trigger_push_touch);
    }

    if (*self_).spawnflags & 4 != 0 {
        //linear
        (*self_).speed = 1000.0;
    }

    (*self_).think = Some(AimAtTarget);
    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
    trap::LinkEntity(self_);
}

/// `void Use_target_push( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_trigger.c:1141). Launch the activating client by copying the precomputed `s.origin2`
/// push velocity into its `ps.velocity` (no-op for non-clients or non-`PM_NORMAL`/`PM_FLOAT`
/// pm_types), firing the `BSET_USE` script hook, and replaying the bounce noise on
/// `CHAN_AUTO` at most every 1.5s (`fly_sound_debounce_time`). No oracle.
///
/// # Safety
/// `self_`/`activator` must point to valid `gentity_t`s.
pub unsafe extern "C" fn Use_target_push(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if (*activator).client.is_null() {
        return;
    }

    if (*(*activator).client).ps.pm_type != PM_NORMAL
        && (*(*activator).client).ps.pm_type != PM_FLOAT
    {
        return;
    }

    G_ActivateBehavior(self_, BSET_USE);

    VectorCopy(&(*self_).s.origin2, &mut (*(*activator).client).ps.velocity);

    // play fly sound every 1.5 seconds
    if (*activator).fly_sound_debounce_time < (*addr_of!(level)).time {
        (*activator).fly_sound_debounce_time = (*addr_of!(level)).time + 1500;
        if (*self_).noise_index != 0 {
            G_Sound(activator, CHAN_AUTO, (*self_).noise_index);
        }
    }
}

/*QUAKED target_push (.5 .5 .5) (-8 -8 -8) (8 8 8) bouncepad CONSTANT
CONSTANT will push activator in direction of 'target' at constant 'speed'

Pushes the activator in the direction.of angle, or towards a target apex.
"speed"		defaults to 1000
if "bouncepad", play bounce noise instead of none
*/
/// `void SP_target_push( gentity_t *self )` (g_trigger.c:1171). Spawn a `target_push`: derive
/// the push velocity `s.origin2` from `s.angles`×`speed` (default 1000), precache the jump
/// sound into `noise_index` when the `bouncepad` spawnflag is set, and — if it has a `target` —
/// seed its bounds and schedule [`AimAtTarget`] to recompute toward the target apex. Installs
/// [`Use_target_push`]. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_target_push(self_: *mut gentity_t) {
    if (*self_).speed == 0.0 {
        (*self_).speed = 1000.0;
    }
    G_SetMovedir(&mut (*self_).s.angles, &mut (*self_).s.origin2);
    let o2 = (*self_).s.origin2;
    VectorScale(&o2, (*self_).speed, &mut (*self_).s.origin2);

    if (*self_).spawnflags & 1 != 0 {
        (*self_).noise_index = G_SoundIndex("sound/weapons/force/jump.wav");
    } else {
        (*self_).noise_index = 0; //G_SoundIndex("sound/misc/windfly.wav");
    }
    if !(*self_).target.is_null() {
        VectorCopy(&(*self_).s.origin, &mut (*self_).r.absmin);
        VectorCopy(&(*self_).s.origin, &mut (*self_).r.absmax);
        (*self_).think = Some(AimAtTarget);
        (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
    }
    (*self_).r#use = Some(Use_target_push);
}

//==========================================================
// trigger_lightningstrike

/// `void Do_Strike( gentity_t *ent )` (g_trigger.c:742). Fire one lightning bolt: pick a
/// random point within the trigger's bounds, trace straight down to it, and either radius- or
/// line-damage whatever it hit (`MOD_SUICIDE`), then play the lightning effect at the strike
/// origin. A bad (in-solid) trace just reschedules the next frame. No oracle (engine
/// trace/effect over global state).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn Do_Strike(ent: *mut gentity_t) {
    let mut strike_from: vec3_t = [0.0; 3];
    let mut strike_point: vec3_t = [0.0; 3];
    let mut fx_ang: vec3_t = [0.0; 3];

    //maybe allow custom fx direction at some point?
    VectorSet(&mut fx_ang, 90.0, 0.0, 0.0);

    //choose a random point to strike within the bounds of the trigger
    strike_point[0] = flrand((*ent).r.absmin[0], (*ent).r.absmax[0]);
    strike_point[1] = flrand((*ent).r.absmin[1], (*ent).r.absmax[1]);

    //consider the bottom mins the ground level
    strike_point[2] = (*ent).r.absmin[2];

    //set the from point
    strike_from[0] = strike_point[0];
    strike_from[1] = strike_point[1];
    strike_from[2] = (*ent).r.absmax[2] - 4.0;

    //now trace for damaging stuff, and do the effect
    let local_trace = trap::Trace(
        &strike_from,
        &vec3_origin,
        &vec3_origin,
        &strike_point,
        (*ent).s.number,
        MASK_PLAYERSOLID,
    );
    VectorCopy(&local_trace.endpos, &mut strike_point);

    if local_trace.startsolid != 0 || local_trace.allsolid != 0 {
        //got a bad spot, think again next frame to try another strike
        (*ent).nextthink = (*addr_of!(level)).time;
        return;
    }

    if (*ent).radius != 0.0 {
        //do a radius damage at the end pos
        G_RadiusDamage(
            &strike_point,
            ent,
            (*ent).damage as f32,
            (*ent).radius,
            ent,
            null_mut(),
            MOD_SUICIDE,
        );
    } else {
        //only damage individuals
        let tr_hit = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add(local_trace.entityNum as usize);

        if (*tr_hit).inuse != QFALSE && (*tr_hit).takedamage != QFALSE {
            //damage it then
            G_Damage(
                tr_hit,
                ent,
                ent,
                null_mut(),
                addr_of_mut!((*tr_hit).r.currentOrigin),
                (*ent).damage,
                0,
                MOD_SUICIDE,
            );
        }
    }

    G_PlayEffectID((*ent).genericValue2, &strike_from, &fx_ang);
}

/// `void Think_Strike( gentity_t *ent )` (g_trigger.c:792). The per-frame loop: while turned
/// on (`genericValue1 == 0`), reschedule `wait + Q_irand(0, random)` ms out and fire one
/// [`Do_Strike`]. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn Think_Strike(ent: *mut gentity_t) {
    if (*ent).genericValue1 != 0 {
        //turned off currently
        return;
    }

    // C: `level.time + ent->wait + Q_irand(0, ent->random)` — `wait`/`random` are floats, so the
    // sum promotes to float (the int `Q_irand` result and `level.time` included), then truncates
    // back into the int `nextthink`.
    (*ent).nextthink = ((*addr_of!(level)).time as f32
        + (*ent).wait
        + Q_irand(0, (*ent).random as c_int) as f32) as c_int;
    Do_Strike(ent);
}

/// `void Use_Strike( gentity_t *ent, gentity_t *other, gentity_t *activator )`
/// (g_trigger.c:804). Toggle the strike on/off; turning it back on rearms the think loop for
/// this frame. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `other`/`activator` are unused.
pub unsafe extern "C" fn Use_Strike(
    ent: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    (*ent).genericValue1 = ((*ent).genericValue1 == 0) as c_int;

    if (*ent).genericValue1 == 0 {
        //turn it back on
        (*ent).nextthink = (*addr_of!(level)).time;
    }
}

/*QUAKED trigger_lightningstrike (.1 .5 .1) ? START_OFF
START_OFF - start trigger disabled

"lightningfx"	effect to use for lightning, MUST be specified
"wait"			Seconds between strikes, 1000 default
"random"		wait variance, default is 2000
"dmg"			damage on strike (default 50)
"radius"		if non-0, does a radius damage at the lightning strike
                impact point (using this value as the radius). otherwise
                will only do line trace damage. default 0.

use to toggle on and off
*/
/// `void SP_trigger_lightningstrike( gentity_t *ent )` (g_trigger.c:827). Spawn a lightning
/// trigger: wire the use/think callbacks, intern the (required) `lightningfx` effect into
/// `genericValue2`, honor `START_OFF`, and default `wait`/`random`/`damage`. No oracle.
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_trigger_lightningstrike(ent: *mut gentity_t) {
    (*ent).r#use = Some(Use_Strike);
    (*ent).think = Some(Think_Strike);
    (*ent).nextthink = (*addr_of!(level)).time + 500;

    let mut s: *mut c_char = null_mut();
    G_SpawnString(c"lightningfx".as_ptr(), c"".as_ptr(), &mut s);
    if s.is_null() || *s == 0 {
        Com_Error(ERR_DROP, "trigger_lightningstrike with no lightningfx");
    }

    //get a configstring index for it
    (*ent).genericValue2 = G_EffectIndex(&CStr::from_ptr(s).to_string_lossy());

    if (*ent).spawnflags & 1 != 0 {
        //START_OFF
        (*ent).genericValue1 = 1;
    }

    if (*ent).wait == 0.0 {
        //default 1000
        (*ent).wait = 1000.0;
    }
    if (*ent).random == 0.0 {
        //default 2000
        (*ent).random = 2000.0;
    }
    if (*ent).damage == 0 {
        //default 50
        (*ent).damage = 50;
    }

    InitTrigger(ent);
    trap::LinkEntity(ent);
}

/*
==============================================================================
trigger_teleport
==============================================================================
*/

/// `void trigger_teleporter_touch( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (g_trigger.c:1200). Teleports a touching client to the trigger's `target` destination:
/// bails if the trigger is `FL_INACTIVE` (set by `target_deactivate`), the toucher isn't a
/// live client, or (for a SPECTATOR-only trigger, spawnflag 1) the toucher isn't a
/// spectator; otherwise relocates them via [`TeleportPlayer`]. No oracle (engine teleport
/// syscalls).
///
/// # Safety
/// `self_`/`other` must point to valid `gentity_t`s; `trace` is unused.
pub unsafe extern "C" fn trigger_teleporter_touch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    if (*self_).flags & FL_INACTIVE != 0 {
        // set by target_deactivate
        return;
    }

    if (*other).client.is_null() {
        return;
    }
    if (*(*other).client).ps.pm_type == PM_DEAD {
        return;
    }
    // Spectators only?
    if (*self_).spawnflags & 1 != 0 && (*(*other).client).sess.sessionTeam != TEAM_SPECTATOR {
        return;
    }

    let dest = G_PickTarget((*self_).target);
    if dest.is_null() {
        G_Printf("Couldn't find teleporter destination\n");
        return;
    }

    TeleportPlayer(other, &(*dest).s.origin, &(*dest).s.angles);
}

/*QUAKED trigger_teleport (.5 .5 .5) ? SPECTATOR
Allows client side prediction of teleportation events.
Must point at a target_position, which will be the teleport destination.

If spectator is set, only spectators can use this teleport
Spectator teleporters are not normally placed in the editor, but are created
automatically near doors to allow spectators to move through them
*/
/// `void SP_trigger_teleport( gentity_t *self )` (g_trigger.c:1239). Sets up a teleport
/// volume: installs the brush model, sends it to the client (unless it's a spectator-only
/// trigger), precaches the speed sound, marks it `ET_TELEPORT_TRIGGER`, and wires the
/// touch callback. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn SP_trigger_teleport(self_: *mut gentity_t) {
    InitTrigger(self_);

    // unlike other triggers, we need to send this one to the client
    // unless is a spectator trigger
    if (*self_).spawnflags & 1 != 0 {
        (*self_).r.svFlags |= SVF_NOCLIENT;
    } else {
        (*self_).r.svFlags &= !SVF_NOCLIENT;
    }

    // make sure the client precaches this sound
    G_SoundIndex("sound/weapons/force/speed.wav");

    (*self_).s.eType = ET_TELEPORT_TRIGGER;
    (*self_).touch = Some(trigger_teleporter_touch);

    trap::LinkEntity(self_);
}

/// `void trigger_cleared_fire( gentity_t *self )` (g_trigger.c:552). Fires the trigger's
/// `target2` once the trigger has been "cleared" (its activation conditions stopped being
/// satisfied), then clears the `think` hook and — if the trigger has a positive `wait` —
/// starts the wait timer from this point. No oracle (entity-state `think` reset +
/// `level.time` plumbing + `G_UseTargets2` + `crandom`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn trigger_cleared_fire(self_: *mut gentity_t) {
    G_UseTargets2(self_, (*self_).activator, (*self_).target2);
    (*self_).think = None;
    // should start the wait timer now, because the trigger's just been cleared, so we must
    // "wait" from this point
    if (*self_).wait > 0.0 {
        // C evaluates this in `double`: `crandom()` is a double, so the whole `wait + random *
        // crandom()` subexpression and the `* 1000` promote to double, as does `level.time`,
        // before the result truncates to the int nextthink.
        (*self_).nextthink = ((*addr_of!(level)).time as f64
            + ((*self_).wait as f64 + (*self_).random as f64 * crandom()) * 1000.0)
            as c_int;
    }
}

/// `void hurt_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_trigger.c:1283). The `trigger_hurt` *use* callback (distinct from `hurt_touch`):
/// remembers the activating client as `self->activator` (only if it is a live client,
/// else clears it), fires the `BSET_USE` script hook, then toggles the trigger's solidity
/// by linking/unlinking it from the world. No oracle (entity-state mutation + link/unlink
/// syscalls).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other` is unused; `activator` may be null.
pub unsafe extern "C" fn hurt_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    activator: *mut gentity_t,
) {
    if !activator.is_null() && (*activator).inuse != QFALSE && !(*activator).client.is_null() {
        (*self_).activator = activator;
    } else {
        (*self_).activator = null_mut();
    }

    G_ActivateBehavior(self_, BSET_USE);

    if (*self_).r.linked != QFALSE {
        trap::UnlinkEntity(self_);
    } else {
        trap::LinkEntity(self_);
    }
}

/// `void hurt_touch( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (g_trigger.c:1302). The `trigger_hurt` *touch* callback: damages (or, for the
/// `damage == -1` fall-to-death special case, respawns/kills) the toucher. Honors Siege
/// team ownership, the `target_deactivate` `FL_INACTIVE` flag, `takedamage`, and a
/// per-volume re-trigger debounce (`timestamp`). No oracle (entity-state mutation +
/// `G_Damage`/`respawn`/`Jetpack_Off`/sound side effects).
///
/// # Safety
/// `self_` and `other` must point to valid `gentity_t`s; `trace` is unused.
pub unsafe extern "C" fn hurt_touch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let dflags: c_int;

    if (*addr_of!(g_gametype)).integer == GT_SIEGE
        && !(*self_).team.is_null()
        && *(*self_).team != 0
    {
        let team = atoi((*self_).team);

        if (*other).inuse != QFALSE
            && (*other).s.number < MAX_CLIENTS as c_int
            && !(*other).client.is_null()
            && (*(*other).client).sess.sessionTeam != team
        {
            //real client don't hurt
            return;
        } else if (*other).inuse != QFALSE
            && !(*other).client.is_null()
            && (*other).s.eType == ET_NPC
            && (*other).s.NPC_class == CLASS_VEHICLE
            && (*other).s.teamowner != team
        {
            //vehicle owned by team don't hurt
            return;
        }
    }

    if (*self_).flags & FL_INACTIVE != 0 {
        //set by target_deactivate
        return;
    }

    if (*other).takedamage == QFALSE {
        return;
    }

    if (*self_).timestamp > (*addr_of!(level)).time {
        return;
    }

    if (*self_).damage == -1
        && !other.is_null()
        && !(*other).client.is_null()
        && (*other).health < 1
    {
        (*(*other).client).ps.fallingToDeath = 0;
        respawn(other);
        return;
    }

    if (*self_).damage == -1
        && !other.is_null()
        && !(*other).client.is_null()
        && (*(*other).client).ps.fallingToDeath != 0
    {
        return;
    }

    if (*self_).spawnflags & 16 != 0 {
        (*self_).timestamp = (*addr_of!(level)).time + 1000;
    } else {
        (*self_).timestamp = (*addr_of!(level)).time + FRAMETIME;
    }

    // play sound
    /*
    if ( !(self->spawnflags & 4) && self->damage != -1 ) {
        G_Sound( other, CHAN_AUTO, self->noise_index );
    }
    */

    if (*self_).spawnflags & 8 != 0 {
        dflags = DAMAGE_NO_PROTECTION;
    } else {
        dflags = 0;
    }

    if (*self_).damage == -1 && !other.is_null() && !(*other).client.is_null() {
        if (*(*other).client).ps.otherKillerTime > (*addr_of!(level)).time {
            //we're as good as dead, so if someone pushed us into this then remember them
            (*(*other).client).ps.otherKillerTime = (*addr_of!(level)).time + 20000;
            (*(*other).client).ps.otherKillerDebounceTime = (*addr_of!(level)).time + 10000;
            (*(*other).client).otherKillerMOD = MOD_FALLING;
            (*(*other).client).otherKillerVehWeapon = 0;
            (*(*other).client).otherKillerWeaponType = WP_NONE;
        }
        (*(*other).client).ps.fallingToDeath = (*addr_of!(level)).time;

        //rag on the way down, this flag will automatically be cleared for us on respawn
        (*(*other).client).ps.eFlags |= EF_RAG;

        //make sure his jetpack is off
        Jetpack_Off(other);

        if !(*other).NPC.is_null() {
            //kill it now
            let mut vDir: vec3_t = [0.0; 3];

            VectorSet(&mut vDir, 0.0, 1.0, 0.0);
            G_Damage(
                other,
                other,
                other,
                addr_of_mut!(vDir),
                addr_of_mut!((*(*other).client).ps.origin),
                Q3_INFINITE,
                0,
                MOD_FALLING,
            );
        } else {
            G_EntitySound(other, CHAN_VOICE, G_SoundIndex("*falling1.wav"));
        }

        (*self_).timestamp = 0; //do not ignore others
    } else {
        let mut dmg = (*self_).damage;

        if dmg == -1 {
            //so fall-to-blackness triggers destroy evertyhing
            dmg = 99999;
            (*self_).timestamp = 0;
        }
        if !(*self_).activator.is_null()
            && (*(*self_).activator).inuse != QFALSE
            && !(*(*self_).activator).client.is_null()
        {
            G_Damage(
                other,
                (*self_).activator,
                (*self_).activator,
                null_mut(),
                null_mut(),
                dmg,
                dflags | DAMAGE_NO_PROTECTION,
                MOD_TRIGGER_HURT,
            );
        } else {
            G_Damage(
                other,
                self_,
                self_,
                null_mut(),
                null_mut(),
                dmg,
                dflags | DAMAGE_NO_PROTECTION,
                MOD_TRIGGER_HURT,
            );
        }
    }
}

/// `void SP_trigger_hurt( gentity_t *self )` (g_trigger.c:1413). Spawn a `trigger_hurt`
/// brush: shared trigger setup, register the falling/speed sounds, install `hurt_touch`,
/// default the damage to 5, and link in unless it starts inactive. No oracle (spawn fn:
/// entity-state side effects + `G_SoundIndex`/link traps).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a valid `model` C string.
pub unsafe extern "C" fn SP_trigger_hurt(self_: *mut gentity_t) {
    InitTrigger(self_);

    gTrigFallSound = G_SoundIndex("*falling1.wav");

    (*self_).noise_index = G_SoundIndex("sound/weapons/force/speed.wav");
    (*self_).touch = Some(hurt_touch);

    if (*self_).damage == 0 {
        (*self_).damage = 5;
    }

    (*self_).r.contents = CONTENTS_TRIGGER;

    if (*self_).spawnflags & 2 != 0 {
        (*self_).r#use = Some(hurt_use);
    }

    // link in to the world if starting active
    if (*self_).spawnflags & 1 == 0 {
        trap::LinkEntity(self_);
    } else if (*self_).r.linked != QFALSE {
        trap::UnlinkEntity(self_);
    }
}

/// `void shipboundary_touch( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (g_trigger.c:1493). When a piloted fighter vehicle hits the boundary brush, point it at the
/// trigger's `target` turnaround entity and stamp a `vehTurnaroundTime` so the pmove code steers
/// it there; an unpiloted/damaged vehicle is just blown up. No oracle (vehicle/entity-state +
/// `G_Find`/`G_Damage`/link traps; may `G_Error`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`trace` may be null.
pub unsafe extern "C" fn shipboundary_touch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let ent: *mut gentity_t;

    if other.is_null()
        || (*other).inuse == QFALSE
        || (*other).client.is_null()
        || (*other).s.number < MAX_CLIENTS as c_int
        || (*other).m_pVehicle.is_null()
    {
        //only let vehicles touch
        return;
    }

    if (*(*other).client).ps.hyperSpaceTime != 0
        && (*addr_of!(level)).time - (*(*other).client).ps.hyperSpaceTime < HYPERSPACE_TIME
    {
        //don't interfere with hyperspacing ships
        return;
    }

    ent = G_Find(
        null_mut(),
        offset_of!(gentity_s, targetname),
        (*self_).target,
    );
    if ent.is_null() || (*ent).inuse == QFALSE {
        //this is bad
        G_Error(&format!(
            "trigger_shipboundary has invalid target '{}'\n",
            CStr::from_ptr((*self_).target).to_string_lossy()
        ));
    }

    if (*(*other).client).ps.m_iVehicleNum == 0 || (*(*other).m_pVehicle).m_iRemovedSurfaces != 0 {
        //if a vehicle touches a boundary without a pilot in it or with parts missing, just blow the thing up
        G_Damage(
            other,
            other,
            other,
            null_mut(),
            &mut (*(*other).client).ps.origin,
            99999,
            DAMAGE_NO_PROTECTION,
            MOD_SUICIDE,
        );
        return;
    }

    //make sure this sucker is linked so the prediction knows where to go
    trap::LinkEntity(ent);

    (*(*other).client).ps.vehTurnaroundIndex = (*ent).s.number;
    (*(*other).client).ps.vehTurnaroundTime =
        (*addr_of!(level)).time + ((*self_).genericValue1 * 2);

    //keep up the detailed checks for another 2 seconds
    (*self_).genericValue7 = (*addr_of!(level)).time + 2000;
}

/// `void shipboundary_think(gentity_t *ent)` (g_trigger.c:1533). While a fighter has recently
/// touched (within 2s), re-scan the brush box for piloted fighter vehicles and re-apply
/// [`shipboundary_touch`] so the steering stays current. No oracle (`trap_EntitiesInBox` +
/// vehicle/entity-state).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`.
pub unsafe extern "C" fn shipboundary_think(ent: *mut gentity_t) {
    let mut iEntityList: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
    let numListedEntities: c_int;
    let mut i: c_int = 0;

    (*ent).nextthink = (*addr_of!(level)).time + 100;

    if (*ent).genericValue7 < (*addr_of!(level)).time {
        //don't need to be doing this check, no one has touched recently
        return;
    }

    numListedEntities = trap::EntitiesInBox(&(*ent).r.absmin, &(*ent).r.absmax, &mut iEntityList);
    while i < numListedEntities {
        let listedEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add(iEntityList[i as usize] as usize);
        if (*listedEnt).inuse != QFALSE
            && !(*listedEnt).client.is_null()
            && (*(*listedEnt).client).ps.m_iVehicleNum != 0
        {
            if (*listedEnt).s.eType == ET_NPC as c_int && (*listedEnt).s.NPC_class == CLASS_VEHICLE
            {
                let pVeh = (*listedEnt).m_pVehicle;
                if !pVeh.is_null() && (*(*pVeh).m_pVehicleInfo).r#type == VH_FIGHTER {
                    shipboundary_touch(ent, listedEnt, null_mut());
                }
            }
        }
        i += 1;
    }
}

/*QUAKED trigger_shipboundary (.5 .5 .5) ?
causes vehicle to turn toward target and travel in that direction for a set time when hit.

"target"		name of entity to turn toward (can be info_notnull, or whatever).
"traveltime"	time to travel in this direction

*/
/// `void SP_trigger_shipboundary( gentity_t *self )` (g_trigger.c:1574). Spawn a
/// `trigger_shipboundary` brush: shared trigger setup, require a `target` and a non-zero
/// `traveltime`, install `shipboundary_think`/`shipboundary_touch`, and link in. No oracle
/// (spawn fn: entity-state side effects + spawn-key reads / link trap; may `G_Error`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a valid `model` C string.
pub unsafe extern "C" fn SP_trigger_shipboundary(self_: *mut gentity_t) {
    InitTrigger(self_);
    (*self_).r.contents = CONTENTS_TRIGGER;

    if (*self_).target.is_null() || *(*self_).target == 0 {
        G_Error("trigger_shipboundary without a target.");
    }
    G_SpawnInt(
        c"traveltime".as_ptr(),
        c"0".as_ptr(),
        &mut (*self_).genericValue1,
    );

    if (*self_).genericValue1 == 0 {
        G_Error("trigger_shipboundary without traveltime.");
    }

    (*self_).think = Some(shipboundary_think);
    (*self_).nextthink = (*addr_of!(level)).time + 500;
    (*self_).touch = Some(shipboundary_touch);

    trap::LinkEntity(self_);
}

/// `void hyperspace_touch( gentity_t *self, gentity_t *other, trace_t *trace )`
/// (g_trigger.c:1597). Drives the vehicle hyperspace sequence: on first touch, stash the
/// destination angles and start the timer; once half-way through the effect, compute the
/// `target`-relative offset, re-apply it at the `target2` base position, and teleport the
/// vehicle (and its pilot) there. An unpiloted/damaged vehicle is blown up. No oracle
/// (vehicle/entity-state + `G_Find`/`G_Damage`/`TeleportPlayer`/sound traps; may `G_Error`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `other`/`trace` may be null.
pub unsafe extern "C" fn hyperspace_touch(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    _trace: *mut trace_t,
) {
    let mut ent: *mut gentity_t;

    if other.is_null()
        || (*other).inuse == QFALSE
        || (*other).client.is_null()
        || (*other).s.number < MAX_CLIENTS as c_int
        || (*other).m_pVehicle.is_null()
    {
        //only let vehicles touch
        return;
    }

    if (*(*other).client).ps.hyperSpaceTime != 0
        && (*addr_of!(level)).time - (*(*other).client).ps.hyperSpaceTime < HYPERSPACE_TIME
    {
        //already hyperspacing, just keep us moving
        if (*(*other).client).ps.eFlags2 & EF2_HYPERSPACE != 0 {
            //they've started the hyperspace but haven't been teleported yet
            let timeFrac: f32 = ((*addr_of!(level)).time - (*(*other).client).ps.hyperSpaceTime)
                as f32
                / HYPERSPACE_TIME as f32;
            if timeFrac >= HYPERSPACE_TELEPORT_FRAC {
                //half-way, now teleport them!
                let mut diff: vec3_t = [0.0; 3];
                let mut fwd: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];
                let mut up: vec3_t = [0.0; 3];
                let mut newOrg: vec3_t = [0.0; 3];
                let fDiff: f32;
                let rDiff: f32;
                let uDiff: f32;
                //take off the flag so we only do this once
                (*(*other).client).ps.eFlags2 &= !EF2_HYPERSPACE;
                //Get the offset from the local position
                ent = G_Find(
                    null_mut(),
                    offset_of!(gentity_s, targetname),
                    (*self_).target,
                );
                if ent.is_null() || (*ent).inuse == QFALSE {
                    //this is bad
                    G_Error(&format!(
                        "trigger_hyperspace has invalid target '{}'\n",
                        CStr::from_ptr((*self_).target).to_string_lossy()
                    ));
                }
                VectorSubtract(&(*(*other).client).ps.origin, &(*ent).s.origin, &mut diff);
                AngleVectors(
                    &(*ent).s.angles,
                    Some(&mut fwd),
                    Some(&mut right),
                    Some(&mut up),
                );
                fDiff = DotProduct(&fwd, &diff);
                rDiff = DotProduct(&right, &diff);
                uDiff = DotProduct(&up, &diff);
                //Now get the base position of the destination
                ent = G_Find(
                    null_mut(),
                    offset_of!(gentity_s, targetname),
                    (*self_).target2,
                );
                if ent.is_null() || (*ent).inuse == QFALSE {
                    //this is bad
                    G_Error(&format!(
                        "trigger_hyperspace has invalid target2 '{}'\n",
                        CStr::from_ptr((*self_).target2).to_string_lossy()
                    ));
                }
                VectorCopy(&(*ent).s.origin, &mut newOrg);
                //finally, add the offset into the new origin
                AngleVectors(
                    &(*ent).s.angles,
                    Some(&mut fwd),
                    Some(&mut right),
                    Some(&mut up),
                );
                VectorMA(&newOrg.clone(), fDiff * (*self_).radius, &fwd, &mut newOrg);
                VectorMA(
                    &newOrg.clone(),
                    rDiff * (*self_).radius,
                    &right,
                    &mut newOrg,
                );
                VectorMA(&newOrg.clone(), uDiff * (*self_).radius, &up, &mut newOrg);
                //G_Printf("hyperspace from %s to %s\n", vtos(other->client->ps.origin), vtos(newOrg) );
                //now put them in the offset position, facing the angles that position wants them to be facing
                TeleportPlayer(other, &newOrg, &(*ent).s.angles);
                if !(*other).m_pVehicle.is_null() && !(*(*other).m_pVehicle).m_pPilot.is_null() {
                    //teleport the pilot, too
                    TeleportPlayer(
                        (*(*other).m_pVehicle).m_pPilot as *mut gentity_t,
                        &newOrg,
                        &(*ent).s.angles,
                    );
                    //FIXME: and the passengers?
                }
                //make them face the new angle
                //other->client->ps.hyperSpaceIndex = ent->s.number;
                VectorCopy(
                    &(*ent).s.angles,
                    &mut (*(*other).client).ps.hyperSpaceAngles,
                );
                //sound
                G_Sound(
                    other,
                    CHAN_LOCAL,
                    G_SoundIndex("sound/vehicles/common/hyperend.wav"),
                );
            }
        }
        return;
    } else {
        ent = G_Find(
            null_mut(),
            offset_of!(gentity_s, targetname),
            (*self_).target,
        );
        if ent.is_null() || (*ent).inuse == QFALSE {
            //this is bad
            G_Error(&format!(
                "trigger_hyperspace has invalid target '{}'\n",
                CStr::from_ptr((*self_).target).to_string_lossy()
            ));
        }

        if (*(*other).client).ps.m_iVehicleNum == 0
            || (*(*other).m_pVehicle).m_iRemovedSurfaces != 0
        {
            //if a vehicle touches a boundary without a pilot in it or with parts missing, just blow the thing up
            G_Damage(
                other,
                other,
                other,
                null_mut(),
                &mut (*(*other).client).ps.origin,
                99999,
                DAMAGE_NO_PROTECTION,
                MOD_SUICIDE,
            );
            return;
        }
        //other->client->ps.hyperSpaceIndex = ent->s.number;
        VectorCopy(
            &(*ent).s.angles,
            &mut (*(*other).client).ps.hyperSpaceAngles,
        );
        (*(*other).client).ps.hyperSpaceTime = (*addr_of!(level)).time;
    }
}

/// `void SP_trigger_hyperspace( gentity_t *self )` (g_trigger.c:1707). Spawn a
/// `trigger_hyperspace` brush: register the hyperspace-end sound, shared trigger setup,
/// require both `target` and `target2`, stash the trigger's diagonal size in `delay`,
/// install `hyperspace_touch`, and link in. No oracle (spawn fn: entity-state side
/// effects + `G_SoundIndex`/link traps; may `G_Error`).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a valid `model` C string.
pub unsafe extern "C" fn SP_trigger_hyperspace(self_: *mut gentity_t) {
    G_SpawnFloat(c"exitscale".as_ptr(), c"1".as_ptr(), &mut (*self_).radius);

    //register the hyperspace end sound (start sounds are customized)
    G_SoundIndex("sound/vehicles/common/hyperend.wav");

    InitTrigger(self_);
    (*self_).r.contents = CONTENTS_TRIGGER;

    if (*self_).target.is_null() || *(*self_).target == 0 {
        G_Error("trigger_hyperspace without a target.");
    }
    if (*self_).target2.is_null() || *(*self_).target2 == 0 {
        G_Error("trigger_hyperspace without a target2.");
    }

    (*self_).delay = Distance(&(*self_).r.absmax, &(*self_).r.absmin) as c_int; //my size

    (*self_).touch = Some(hyperspace_touch);

    trap::LinkEntity(self_);

    //self->think = trigger_hyperspace_find_targets;
    //self->nextthink = level.time + FRAMETIME;
}

//==========================================================
// trigger_asteroid_field

/// `int asteroid_count_num_asteroids( gentity_t *self )` (g_trigger.c:1843). Count the live
/// asteroids this field owns: every in-use entity (between `MAX_CLIENTS` and `ENTITYNUM_WORLD`)
/// whose `r.ownerNum` is the field's entity number. No oracle (walks the global entity list).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn asteroid_count_num_asteroids(self_: *mut gentity_t) -> c_int {
    let mut count: c_int = 0;

    for i in (MAX_CLIENTS as c_int)..ENTITYNUM_WORLD {
        let e = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);
        if (*e).inuse == QFALSE {
            continue;
        }
        if (*e).r.ownerNum == (*self_).s.number {
            count += 1;
        }
    }
    count
}

/// `gentity_t *asteroid_pick_random_asteroid( gentity_t *self )` (g_trigger.c:1799). Pick one
/// of the `func_rotating` asteroids targeted by this field at random: walk every entity whose
/// `targetname` matches the field's `target` (excluding the field itself), returning NULL if
/// none, the sole match if exactly one, else a uniformly-chosen one (`Q_irand`). No oracle
/// (walks the global entity list + RNG).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn asteroid_pick_random_asteroid(self_: *mut gentity_t) -> *mut gentity_t {
    let mut t_count: c_int = 0;
    let pick: c_int;
    let mut t: *mut gentity_t = null_mut();

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
        return null_mut();
    }

    if t_count == 1 {
        return G_Find(
            null_mut(),
            offset_of!(gentity_s, targetname),
            (*self_).target,
        );
    }

    //FIXME: need a seed
    pick = Q_irand(1, t_count);
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

        if t_count == pick {
            return t;
        }
    }
    null_mut()
}

/// `void asteroid_move_to_start2( gentity_t *self, gentity_t *ownerTrigger )`
/// (g_trigger.c:1864). Move an asteroid to a fresh start position inside `ownerTrigger`'s
/// bounds: pick a random "cap" axis it travels fully across (min→max or max→min) and random
/// points on the other two, set the origin, schedule a `Q3_Lerp2Origin` glide to the end spot
/// (over `ceil(dist/speed)` seconds), spin it with random angles/angular velocity, then re-arm
/// [`asteroid_move_to_start`] for when it arrives. If the owner is gone, free the asteroid
/// next frame. No oracle (RNG + ICARUS lerp + entity-state mutation).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`; `ownerTrigger` may be null.
pub unsafe extern "C" fn asteroid_move_to_start2(
    self_: *mut gentity_t,
    ownerTrigger: *mut gentity_t,
) {
    //move asteroid to a new start position
    if !ownerTrigger.is_null() {
        //move it
        let mut startSpot: vec3_t = [0.0; 3];
        let mut endSpot: vec3_t = [0.0; 3];
        let mut startAngles: vec3_t = [0.0; 3];
        let dist: f32;
        let speed: f32 = flrand((*self_).speed * 0.25, (*self_).speed * 2.0);
        let capAxis: c_int;
        let time: c_int;

        capAxis = Q_irand(0, 2);
        for axis in 0..3usize {
            if axis as c_int == capAxis {
                if Q_irand(0, 1) != 0 {
                    startSpot[axis] = (*ownerTrigger).r.mins[axis];
                    endSpot[axis] = (*ownerTrigger).r.maxs[axis];
                } else {
                    startSpot[axis] = (*ownerTrigger).r.maxs[axis];
                    endSpot[axis] = (*ownerTrigger).r.mins[axis];
                }
            } else {
                startSpot[axis] = (*ownerTrigger).r.mins[axis]
                    + (flrand(0.0, 1.0)
                        * ((*ownerTrigger).r.maxs[axis] - (*ownerTrigger).r.mins[axis]));
                endSpot[axis] = (*ownerTrigger).r.mins[axis]
                    + (flrand(0.0, 1.0)
                        * ((*ownerTrigger).r.maxs[axis] - (*ownerTrigger).r.mins[axis]));
            }
        }
        //FIXME: maybe trace from start to end to make sure nothing is in the way?  How big of a trace?

        G_SetOrigin(self_, &startSpot);
        dist = Distance(&endSpot, &startSpot);
        // C: `ceil(dist/speed)*1000` — the float division promotes to double for `ceil`, the
        // `*1000` stays double, then truncates into the int `time`.
        time = (((dist / speed) as f64).ceil() * 1000.0) as c_int;
        Q3_Lerp2Origin(-1, (*self_).s.number, &endSpot, time as f32);

        //spin it
        startAngles[0] = flrand(-360.0, 360.0);
        startAngles[1] = flrand(-360.0, 360.0);
        startAngles[2] = flrand(-360.0, 360.0);
        G_SetAngles(self_, &startAngles);
        (*self_).s.apos.trDelta[0] = flrand(-100.0, 100.0);
        (*self_).s.apos.trDelta[1] = flrand(-100.0, 100.0);
        (*self_).s.apos.trDelta[2] = flrand(-100.0, 100.0);
        (*self_).s.apos.trTime = (*addr_of!(level)).time;
        (*self_).s.apos.trType = TR_LINEAR;
        //move itownerTrigger back to a new start when done
        (*self_).think = Some(asteroid_move_to_start);
        (*self_).nextthink = (*addr_of!(level)).time + time;
    } else {
        //crap, go bye-bye
        (*self_).think = Some(G_FreeEntity);
        (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
    }
}

/// `void asteroid_move_to_start( gentity_t *self )` (g_trigger.c:1922). The `think` callback
/// that moves the asteroid to a new start position: delegates to [`asteroid_move_to_start2`]
/// with its owning field (`g_entities[self->r.ownerNum]`). No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` whose `r.ownerNum` indexes `g_entities`.
pub unsafe extern "C" fn asteroid_move_to_start(self_: *mut gentity_t) {
    //move asteroid to a new start position
    asteroid_move_to_start2(
        self_,
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*self_).r.ownerNum as usize),
    );
}

/// `void asteroid_field_think( gentity_t *self )` (g_trigger.c:1927). The field's per-frame
/// loop: if it owns fewer than `count` asteroids, spawn a new one cloned from a random target
/// asteroid (model/health/mass/damage/speed/scale/material), run it through
/// [`SP_func_rotating`], take ownership, and kick it off with [`asteroid_move_to_start2`].
/// Re-thinks in 500ms (or 100ms while still under quota). No oracle (spawns + RNG + entity-state).
///
/// # Safety
/// `self_` must point to a valid `gentity_t`.
pub unsafe extern "C" fn asteroid_field_think(self_: *mut gentity_t) {
    let numAsteroids = asteroid_count_num_asteroids(self_);

    (*self_).nextthink = (*addr_of!(level)).time + 500;

    if numAsteroids < (*self_).count {
        //need to spawn a new asteroid
        let newAsteroid = G_Spawn();
        if !newAsteroid.is_null() {
            let copyAsteroid = asteroid_pick_random_asteroid(self_);
            if !copyAsteroid.is_null() {
                (*newAsteroid).model = (*copyAsteroid).model;
                (*newAsteroid).model2 = (*copyAsteroid).model2;
                (*newAsteroid).health = (*copyAsteroid).health;
                (*newAsteroid).spawnflags = (*copyAsteroid).spawnflags;
                (*newAsteroid).mass = (*copyAsteroid).mass;
                (*newAsteroid).damage = (*copyAsteroid).damage;
                (*newAsteroid).speed = (*copyAsteroid).speed;

                G_SetOrigin(newAsteroid, &(*copyAsteroid).s.origin);
                G_SetAngles(newAsteroid, &(*copyAsteroid).s.angles);
                (*newAsteroid).classname = c"func_rotating".as_ptr() as *mut c_char;

                SP_func_rotating(newAsteroid);

                (*newAsteroid).genericValue15 = (*copyAsteroid).genericValue15;
                (*newAsteroid).s.iModelScale = (*copyAsteroid).s.iModelScale;
                (*newAsteroid).maxHealth = (*newAsteroid).health;
                G_ScaleNetHealth(newAsteroid);
                (*newAsteroid).radius = (*copyAsteroid).radius;
                (*newAsteroid).material = (*copyAsteroid).material;
                //CacheChunkEffects( self->material );

                //keep track of it
                (*newAsteroid).r.ownerNum = (*self_).s.number;

                //position it
                asteroid_move_to_start2(newAsteroid, self_);

                //think again sooner if need even more
                if numAsteroids + 1 < (*self_).count {
                    //still need at least one more
                    //spawn it in 100ms
                    (*self_).nextthink = (*addr_of!(level)).time + 100;
                }
            }
        }
    }
}

/*QUAKED trigger_asteroid_field (.5 .5 .5) ?
speed - how fast, on average, the asteroid moves
count - how many asteroids, max, to have at one time
target - target this at func_rotating asteroids
*/
/// `void SP_trigger_asteroid_field( gentity_t *self )` (g_trigger.c:1986). Spawn-initializer for
/// the asteroid field: install the brush model but make it non-solid (`contents = 0`,
/// `SVF_NOCLIENT`), default `health`/`speed`, and schedule [`asteroid_field_think`] to start
/// populating the field. No oracle (brush-model/link syscalls + entity-state).
///
/// # Safety
/// `self_` must point to a valid `gentity_t` with a valid `model` C string.
pub unsafe extern "C" fn SP_trigger_asteroid_field(self_: *mut gentity_t) {
    trap::SetBrushModel(self_, &CStr::from_ptr((*self_).model).to_string_lossy());
    //	self->r.contents = CONTENTS_TRIGGER;		// replaces the -1 from trap_SetBrushModel
    (*self_).r.contents = 0;
    (*self_).r.svFlags = SVF_NOCLIENT;

    if (*self_).count == 0 {
        (*self_).health = 20;
    }

    if (*self_).speed == 0.0 {
        (*self_).speed = 10000.0;
    }

    (*self_).think = Some(asteroid_field_think);
    (*self_).nextthink = (*addr_of!(level)).time + 100;

    trap::LinkEntity(self_);
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;

    extern "C" {
        fn jka_G_NameInTriggerClassList(list: *mut c_char, str: *mut c_char) -> qboolean;
    }

    /// NUL-terminated C buffer from a byte slice (which should NOT include its own NUL).
    fn cbuf(s: &[u8]) -> Vec<c_char> {
        let mut v: Vec<c_char> = s.iter().map(|&b| b as c_char).collect();
        v.push(0);
        v
    }

    #[test]
    fn name_in_trigger_class_list_matches_oracle() {
        // (list, str) pairs exercising: empty list, single token (hit/miss),
        // multi-token with the match in front/middle/end, case-insensitivity,
        // empty str, trailing '|', prefix-but-not-equal, and an empty token.
        let cases: &[(&[u8], &[u8])] = &[
            (b"", b"foo"),
            (b"foo", b"foo"),
            (b"foo", b"bar"),
            (b"foo|bar|baz", b"foo"),
            (b"foo|bar|baz", b"bar"),
            (b"foo|bar|baz", b"baz"),
            (b"foo|bar|baz", b"qux"),
            (b"FOO|Bar|BAZ", b"foo"),
            (b"foo|bar", b"FOO"),
            (b"trigger_multiple|trigger_once", b"trigger_once"),
            (b"foo|bar|baz", b""),
            (b"", b""),
            (b"foo|bar|", b"baz"),
            (b"foo|bar|", b""),
            (b"|foo|bar", b""),
            (b"|foo|bar", b"foo"),
            (b"foobar|baz", b"foo"),
            (b"foo|foobar", b"foobar"),
        ];

        for (list, s) in cases {
            let rl = cbuf(list);
            let rs = cbuf(s);
            let mut cl = cbuf(list);
            let mut cs = cbuf(s);
            let r = unsafe { G_NameInTriggerClassList(rl.as_ptr(), rs.as_ptr()) };
            let o = unsafe { jka_G_NameInTriggerClassList(cl.as_mut_ptr(), cs.as_mut_ptr()) };
            assert_eq!(
                r,
                o,
                "G_NameInTriggerClassList({:?}, {:?})",
                String::from_utf8_lossy(list),
                String::from_utf8_lossy(s)
            );
        }
    }
}
