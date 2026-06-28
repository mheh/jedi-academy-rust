//! Slice of `NPC_behavior.c` — the NPC behaviour-state (`NPC_BS*`) track. The handlers
//! whose entire callee set is already ported are pulled forward here:
//!
//!   * `Disappear`   (NPC_behavior.c:185) — entity think that hides + parks an entity.
//!   * `BeamOut`     (NPC_behavior.c:194) — entity think that schedules `Disappear`.
//!   * `NPC_BSSleep` (NPC_behavior.c:500) — BState handler; only touches the global
//!                    `NPC` entity, `NPC_CheckAlertEvents`, and `G_ActivateBehavior`.
//!   * `NPC_BSInvestigate` (NPC_behavior.c:252) — its entire body is commented out in
//!                    the original (a no-op), so it has no callees to block on.
//!   * `NPC_Surrender`        (NPC_behavior.c:1317) — force the NPC into surrender.
//!   * `NPC_BSAdvanceFight`   (NPC_behavior.c:29)  — advance to captureGoal, shoot along the way.
//!   * `NPC_BSCinematic`      (NPC_behavior.c:213) — cinematic move/watch handler.
//!   * `NPC_BSWait`           (NPC_behavior.c:246) — just face current angles.
//!   * `NPC_CheckInvestigate` (NPC_behavior.c:409) — promote an alert to investigate/enemy.
//!   * `NPC_BSRemove`         (NPC_behavior.c:921) — fade out when out of PVS of client 0.
//!   * `NPC_BSWander`         (NPC_behavior.c:1193) — wander between nav nodes.
//!   * `NPC_BSSearchStart`    (NPC_behavior.c:1132) — seed a homeWp search.
//!   * `NPC_CheckSurrender`   (NPC_behavior.c:1341) — decide whether to surrender.
//!   * `NPC_BSFlee`           (NPC_behavior.c:1444) — flee toward a goal away from danger.
//!   * `NPC_StartFlee`        (NPC_behavior.c:1560) — pick a flee combat-point/goal.
//!   * `G_StartFlee`          (NPC_behavior.c:1636) — `SetNPCGlobals` wrapper for `NPC_StartFlee`.
//!   * `NPC_BSEmplaced`       (NPC_behavior.c:1650) — emplaced-gun idle/aim/fire handler.
//!   * `NPC_BSNoClip`         (NPC_behavior.c:1160) — no-clip move straight to the goal.
//!   * `NPC_BSFollowLeader`   (NPC_behavior.c:524)  — follow your leader, fight along the way.
//!   * `NPC_BSSearch`         (NPC_behavior.c:939)  — search waypoint branches for enemies.
//!   * `NPC_BSJump`           (NPC_behavior.c:733)  — parabola-jump handler (face/crouch/jump/land).
//!
//! No oracle: every fn here is pure entity-state / global-state mutation (no
//! return value to parity-check, or driven by `trap_Trace`/`trap_InPVS`/`Q_irand`).

#![allow(non_snake_case)] // C function names (`Disappear`, `BeamOut`, `NPC_BSSleep`) kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

use crate::codemp::game::ai_h::{SQUAD_IDLE, SQUAD_RETREAT};
use crate::codemp::game::anims::{
    BOTH_ATTACK1, BOTH_ATTACK2, BOTH_ATTACK3, BOTH_CROUCH1, BOTH_GUARD_IDLE1,
    BOTH_GUARD_LOOKAROUND1, BOTH_INAIR1, BOTH_LAND1, BOTH_MELEE1, BOTH_MELEE2,
};
use crate::codemp::game::b_local_h::{CP_AVOID, CP_COVER, CP_HAS_ROUTE, CP_NO_PVS};
use crate::codemp::game::b_public_h::{
    BS_DEFAULT, BS_FLEE, BS_FOLLOW_LEADER, BS_HUNT_AND_KILL, BS_INVESTIGATE, BS_SEARCH,
    BS_STAND_GUARD, JS_CROUCHING, JS_FACING, JS_JUMPING, JS_LANDING, JS_WAITING,
    NPCAI_ENROUTE_TO_HOMEWP, NPCAI_MOVING, SCF_DONT_FIRE, SCF_FIRE_WEAPON, SCF_IGNORE_ALERTS,
    SCF_LOOK_FOR_ENEMIES, SPOT_HEAD, SPOT_HEAD_LEAN, SPOT_WEAPON, VIS_FOV, VIS_PVS, VIS_SHOOT,
};
use crate::codemp::game::bg_misc::vectoyaw;
use crate::codemp::game::bg_panimate::PM_InKnockDown;
use crate::codemp::game::bg_public::{
    EF_NODRAW, ET_INVISIBLE, ET_NPC, ET_PLAYER, EV_PUSHED1, EV_PUSHED3, MASK_SHOT, SETANIM_BOTH,
    SETANIM_FLAG_HOLD, SETANIM_FLAG_NORMAL, SETANIM_FLAG_OVERRIDE, SETANIM_LEGS, TEAM_FREE,
};
use crate::codemp::game::bg_weapons_h::{
    WP_FLECHETTE, WP_NONE, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON,
};
use crate::codemp::game::g_local::{
    gentity_t, AEL_DANGER, AEL_MINOR, AEL_SUSPICIOUS, FL_NOTARGET, FL_NO_KNOCKBACK, FRAMETIME,
};
use crate::codemp::game::g_main::{g_entities, level};
use crate::codemp::game::g_nav::{
    G_Cube, NAV_FindClosestWaypointForEnt, NAV_GetNearestNode, NPC_SetMoveGoal, WAYPOINT_NONE,
};
use crate::codemp::game::g_public_h::{
    BSET_AWAKE, BSET_FLEE, BSET_LOSTENEMY, TID_BSTATE, TID_MOVE_NAV,
};
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_FreeEntity, G_UseTargets2};
use crate::codemp::game::npc::NPC_SetAnim;
use crate::codemp::game::npc::{
    ucmd, NPCInfo, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals, NPC,
};
use crate::codemp::game::npc_ai_default::{NPC_BSRunAndShoot, NPC_BSStandGuard};
use crate::codemp::game::npc_ai_jedi::NPC_MoveDirClear;
use crate::codemp::game::npc_combat::{
    enemyVisibility, G_AddVoiceEvent, G_ClearEnemy, G_SetEnemy, NPC_AimAdjust, NPC_CheckAttack,
    NPC_CheckEnemy, NPC_CheckGetNewWeapon, NPC_EnemyTooFar, NPC_FindCombatPoint,
    NPC_SetCombatPoint, NPC_ShotEntity, ValidEnemy, WeaponThink,
};
use crate::codemp::game::npc_goal::{NPC_ClearGoal, UpdateGoal};
use crate::codemp::game::npc_move::{NPC_MoveToGoal, NPC_SlideMoveToGoal};
use crate::codemp::game::npc_senses::{
    InFOV, NPC_CheckAlertEvents, NPC_CheckVisibility, NPC_GetHFOVPercentage,
};
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, G_ActivateBehavior, NPC_AimWiggle, NPC_CheckEnemyExt, NPC_ClearLOS4,
    NPC_FaceEnemy, NPC_SomeoneLookingAtMe, NPC_UpdateAngles, NPC_UpdateFiringAngles,
    NPC_UpdateShootAngles,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleDelta, AngleNormalize360, AngleVectors, DistanceSquared,
    DotProduct, VectorAdd, VectorClear, VectorCompare, VectorCopy, VectorLength,
    VectorLengthSquared, VectorMA, VectorNormalize, VectorScale, VectorSubtract,
};
use crate::codemp::game::q_shared::random;
use crate::codemp::game::q_shared_h::{
    vec3_t, BUTTON_WALKING, ENTITYNUM_NONE, FP_SABER_DEFENSE, PITCH, YAW,
};
use crate::codemp::game::teams_h::CLASS_PROTOCOL;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `void Disappear( gentity_t *self )` (NPC_behavior.c:185).
///
/// Entity think: flag the entity `EF_NODRAW` (hidden) and park it — clear `think` and
/// set `nextthink` to `-1` so it never thinks again. The leading commented-out
/// `ClientDisconnect(self)` is carried as-is from the original (dead). No oracle —
/// pure entity-state mutation.
///
/// # Safety
/// `self` must point to a valid `gentity_t`.
pub unsafe extern "C" fn Disappear(self_: *mut gentity_t) {
    //	ClientDisconnect(self);
    (*self_).s.eFlags |= EF_NODRAW;
    (*self_).think = None;
    (*self_).nextthink = -1;
}

/// `void BeamOut( gentity_t *self )` (NPC_behavior.c:194).
///
/// Entity think: schedule the entity to vanish 1.5s from now via [`Disappear`], and
/// drop it from its squad/team (`squadname = NULL`, `playerTeam` and `s.teamowner`
/// → `TEAM_FREE`). The commented-out temp-entity teleport-effect block (and the
/// `MakeOwnerInvis` forward-decl / `SVF_BEAMING` note) is dead in SP as well and
/// carried verbatim. No oracle — pure entity/global-state mutation.
///
/// # Safety
/// `self` must point to a valid `gentity_t` with a non-null `client`.
pub unsafe extern "C" fn BeamOut(self_: *mut gentity_t) {
    //	gentity_t *tent = G_Spawn();

    /*
        tent->owner = self;
        tent->think = MakeOwnerInvis;
        tent->nextthink = level.time + 1800;
        //G_AddEvent( ent, EV_PLAYER_TELEPORT, 0 );
        tent = G_TempEntity( self->client->pcurrentOrigin, EV_PLAYER_TELEPORT );
    */
    //fixme: doesn't actually go away!
    (*self_).nextthink = (*addr_of!(level)).time + 1500;
    (*self_).think = Some(Disappear);
    (*(*self_).client).squadname = core::ptr::null_mut();
    (*self_).s.teamowner = TEAM_FREE;
    (*(*self_).client).playerTeam = TEAM_FREE;
    //self->r.svFlags |= SVF_BEAMING; //this appears unused in SP as well
}

/*
void NPC_BSSleep( void )
*/
/// `void NPC_BSSleep( void )` (NPC_behavior.c:500).
///
/// BState handler for a sleeping NPC: look for the most minor (or louder) alert event;
/// if one exists, fire the entity's `BSET_AWAKE` behaviour script and return. The
/// commented-out sound-event/vigilance debounce block below is dead in the original and
/// carried verbatim. No oracle — drives the global `NPC` entity through
/// [`NPC_CheckAlertEvents`] / [`G_ActivateBehavior`].
///
/// # Safety
/// `NPC` must be set to the current think entity.
pub unsafe fn NPC_BSSleep() {
    let alertEvent: c_int = NPC_CheckAlertEvents(QTRUE, QFALSE, -1, QFALSE, AEL_MINOR);

    //There is an event to look at
    if alertEvent >= 0 {
        G_ActivateBehavior(NPC, BSET_AWAKE);
        return;
    }

    /*
    if ( level.time > NPCInfo->enemyCheckDebounceTime )
    {
        if ( NPC_CheckSoundEvents() != -1 )
        {//only 1 alert per second per 0.1 of vigilance
            NPCInfo->enemyCheckDebounceTime = level.time + (NPCInfo->stats.vigilance * 10000);
            G_ActivateBehavior(NPC, BSET_AWAKE);
        }
    }
    */
}

/// `void NPC_BSInvestigate( void )` (NPC_behavior.c:252).
///
/// A no-op: the entire body is commented out in the original (the investigate-goal
/// turn/look/move logic was never re-enabled — *"FIXME: Reimplement"*). Carried as an
/// empty function so the bState-table slot keeps its identity. No oracle (nothing runs).
///
/// # Safety
/// Trivially safe (does nothing), but kept `unsafe` to match the bState-handler family.
pub unsafe fn NPC_BSInvestigate() {
    /*
        //FIXME: maybe allow this to be set as a tempBState in a script?  Just specify the
        //investigateGoal, investigateDebounceTime and investigateCount? (Needs a macro)
        ... (entire original body is commented out — see NPC_behavior.c:254-406) ...
    */
}

/// `void NPC_Surrender( void )` (NPC_behavior.c:1317).
///
/// Drives the global `NPC` into the surrender state: bail if mid-weapon-fire or in a
/// knockdown; (the `WP_DropWeapon` branch is left commented out — *rwwFIXMEFIXME*); if
/// we haven't surrendered for at least ~6 seconds, force-say a "don't shoot!" voice event
/// (`EV_PUSHED1..EV_PUSHED3`); then stamp `surrenderTime` to stay surrendered for at least
/// a second. All comments carried verbatim. No oracle — pure entity/`NPCInfo` state
/// mutation plus a `Q_irand`-driven voice event, no return value to parity-check.
///
/// # Safety
/// `NPC`, `NPCInfo`, and `NPC->client` must be valid for the current think entity.
pub unsafe fn NPC_Surrender() {
    //FIXME: say "don't shoot!" if we weren't already surrendering
    if (*(*NPC).client).ps.weaponTime != 0 || PM_InKnockDown(&mut (*(*NPC).client).ps) != QFALSE {
        return;
    }
    if (*NPC).s.weapon != WP_NONE && (*NPC).s.weapon != WP_STUN_BATON && (*NPC).s.weapon != WP_SABER
    {
        //WP_DropWeapon( NPC, NULL ); //rwwFIXMEFIXME: Do this (gonna need a system for notifying client of removal)
    }
    if (*NPCInfo).surrenderTime < (*addr_of!(level)).time - 5000 {
        //haven't surrendered for at least 6 seconds, tell them what you're doing
        //FIXME: need real dialogue EV_SURRENDER
        (*NPCInfo).blockedSpeechDebounceTime = 0; //make sure we say this
        G_AddVoiceEvent(NPC, Q_irand(EV_PUSHED1, EV_PUSHED3), 3000);
    }
    //	NPC_SetAnim( NPC, SETANIM_TORSO, TORSO_SURRENDER_START, SETANIM_FLAG_HOLD|SETANIM_FLAG_OVERRIDE );
    //	NPC->client->ps.torsoTimer = 1000;
    (*NPCInfo).surrenderTime = (*addr_of!(level)).time + 1000; //stay surrendered for at least 1 second
                                                               //FIXME: while surrendering, make a big sight/sound alert? Or G_AlertTeam?
}

/// `CHECK_*` LOS/visibility flags (NPC_behavior.c via b_local.h) — kept as file-local
/// `const`s to mirror the sibling NPC_*.c slices (`npc_combat.rs`/`npc_senses.rs`); they are
/// not exported from any header module.
const CHECK_PVS: c_int = 1;
const CHECK_360: c_int = 2;
const CHECK_FOV: c_int = 4;
const CHECK_SHOOT: c_int = 8;

/// `void NPC_BSAdvanceFight (void)` (NPC_behavior.c:29).
///
/// Advance towards your `captureGoal` and shoot anyone you can along the way. Faithful 1:1
/// of the original (the leading `//FIXME: IMPLEMENT` and all interior FIXMEs carried). No
/// oracle — process-global `NPC`/`NPCInfo`/`ucmd`/`enemyVisibility` + `trap_Trace`/ICARUS.
///
/// # Safety
/// `NPC`/`NPCInfo` set for the current think entity; `level`/`g_entities` initialised.
pub unsafe fn NPC_BSAdvanceFight() {
    //FIXME: IMPLEMENT
    //Head to Goal if I can

    //Make sure we're still headed where we want to capture
    if !(*NPCInfo).captureGoal.is_null() {
        //FIXME: if no captureGoal, what do we do?
        //VectorCopy( NPCInfo->captureGoal->r.currentOrigin, NPCInfo->tempGoal->r.currentOrigin );
        //NPCInfo->goalEntity = NPCInfo->tempGoal;

        NPC_SetMoveGoal(
            NPC,
            &(*(*NPCInfo).captureGoal).r.currentOrigin,
            16,
            QTRUE,
            -1,
            core::ptr::null_mut(),
        );

        //		NAV_ClearLastRoute(NPC);
        (*NPCInfo).goalTime = (*addr_of!(level)).time + 100000;
    }

    //	NPC_BSRun();

    NPC_CheckEnemy(QTRUE, QFALSE, QTRUE);

    //FIXME: Need melee code
    if !(*NPC).enemy.is_null() {
        //See if we can shoot him
        let mut delta: vec3_t = [0.0; 3];
        let mut forward: vec3_t = [0.0; 3];
        let mut angleToEnemy: vec3_t = [0.0; 3];
        let mut hitspot: vec3_t = [0.0; 3];
        let mut muzzle: vec3_t = [0.0; 3];
        let mut diff: vec3_t = [0.0; 3];
        let mut enemy_org: vec3_t = [0.0; 3];
        let mut enemy_head: vec3_t = [0.0; 3];
        let distanceToEnemy: f32;
        let mut attack_ok: qboolean = QFALSE;
        let mut dead_on: qboolean = QFALSE;
        let mut attack_scale: f32 = 1.0;
        let mut aim_off: f32;
        let max_aim_off: f32 = 64.0;

        //Yaw to enemy
        VectorMA(
            &(*(*NPC).enemy).r.absmin,
            0.5,
            &(*(*NPC).enemy).r.maxs,
            &mut enemy_org,
        );
        CalcEntitySpot(NPC, SPOT_WEAPON, &mut muzzle);

        VectorSubtract(&enemy_org, &muzzle, &mut delta);
        vectoangles(&delta, &mut angleToEnemy);
        distanceToEnemy = VectorNormalize(&mut delta);

        if NPC_EnemyTooFar((*NPC).enemy, distanceToEnemy * distanceToEnemy, QTRUE) == QFALSE {
            attack_ok = QTRUE;
        }

        if attack_ok != QFALSE {
            NPC_UpdateShootAngles(&mut angleToEnemy, QFALSE, QTRUE);

            (*NPCInfo).enemyLastVisibility = enemyVisibility;
            enemyVisibility = NPC_CheckVisibility((*NPC).enemy, CHECK_FOV); //CHECK_360|//CHECK_PVS|

            if enemyVisibility == VIS_FOV {
                //He's in our FOV

                attack_ok = QTRUE;
                CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_head);

                if attack_ok != QFALSE {
                    //are we gonna hit him if we shoot at his center?
                    let mut tr = trap::Trace(
                        &muzzle,
                        &vec3_origin,
                        &vec3_origin,
                        &enemy_org,
                        (*NPC).s.number,
                        MASK_SHOT,
                    );
                    let mut traceEnt: *mut gentity_t = (core::ptr::addr_of_mut!(g_entities)
                        .cast::<gentity_t>())
                    .offset(tr.entityNum as isize);
                    if traceEnt != (*NPC).enemy
                        && (traceEnt.is_null()
                            || (*traceEnt).client.is_null()
                            || (*(*NPC).client).enemyTeam == 0
                            || (*(*NPC).client).enemyTeam != (*(*traceEnt).client).playerTeam)
                    {
                        //no, so shoot for the head
                        attack_scale *= 0.75;
                        tr = trap::Trace(
                            &muzzle,
                            &vec3_origin,
                            &vec3_origin,
                            &enemy_head,
                            (*NPC).s.number,
                            MASK_SHOT,
                        );
                        traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .offset(tr.entityNum as isize);
                    }

                    VectorCopy(&tr.endpos, &mut hitspot);

                    if traceEnt == (*NPC).enemy
                        || (!(*traceEnt).client.is_null()
                            && (*(*NPC).client).enemyTeam != 0
                            && (*(*NPC).client).enemyTeam == (*(*traceEnt).client).playerTeam)
                    {
                        dead_on = QTRUE;
                    } else {
                        attack_scale *= 0.5;
                        if (*(*NPC).client).playerTeam != 0
                            && !traceEnt.is_null()
                            && !(*traceEnt).client.is_null()
                            && (*(*traceEnt).client).playerTeam != 0
                            && (*(*NPC).client).playerTeam == (*(*traceEnt).client).playerTeam
                        {
                            //Don't shoot our own team
                            attack_ok = QFALSE;
                        }
                    }
                }

                if attack_ok != QFALSE {
                    //ok, now adjust pitch aim
                    VectorSubtract(&hitspot, &muzzle, &mut delta);
                    vectoangles(&delta, &mut angleToEnemy);
                    (*(*NPC).NPC).desiredPitch = angleToEnemy[PITCH];
                    NPC_UpdateShootAngles(&mut angleToEnemy, QTRUE, QFALSE);

                    if dead_on == QFALSE {
                        //We're not going to hit him directly, try a suppressing fire
                        //see if where we're going to shoot is too far from his origin
                        AngleVectors(&(*NPCInfo).shootAngles, Some(&mut forward), None, None);
                        VectorMA(&muzzle, distanceToEnemy, &forward, &mut hitspot);
                        VectorSubtract(&hitspot, &enemy_org, &mut diff);
                        aim_off = VectorLength(&diff);
                        if aim_off > random() * max_aim_off
                        //FIXME: use aim value to allow poor aim?
                        {
                            attack_scale *= 0.75;
                            //see if where we're going to shoot is too far from his head
                            VectorSubtract(&hitspot, &enemy_head, &mut diff);
                            aim_off = VectorLength(&diff);
                            if aim_off > random() * max_aim_off {
                                attack_ok = QFALSE;
                            }
                        }
                        attack_scale *= (max_aim_off - aim_off + 1.0) / max_aim_off;
                    }
                }
            }
        }

        if attack_ok != QFALSE {
            if NPC_CheckAttack(attack_scale) != QFALSE {
                //check aggression to decide if we should shoot
                enemyVisibility = VIS_SHOOT;
                WeaponThink(QTRUE);
            }
            //else
            //    attack_ok = qfalse;
        }
    //Don't do this- only for when stationary and trying to shoot an enemy
    //		else
    //			NPC->cantHitEnemyCounter++;
    } else {
        //FIXME:
        NPC_UpdateShootAngles(&mut (*(*NPC).client).ps.viewangles, QTRUE, QTRUE);
    }

    if (*addr_of!(ucmd)).forwardmove == 0 && (*addr_of!(ucmd)).rightmove == 0 {
        //We reached our captureGoal
        if trap::ICARUS_IsInitialized((*NPC).s.number) != QFALSE {
            trap::ICARUS_TaskIDComplete(NPC, TID_BSTATE);
        }
    }
}

/// `void NPC_BSCinematic( void )` (NPC_behavior.c:213).
///
/// Cinematic behaviour: optionally fire, move to goal, and keep facing `watchTarget`.
/// Faithful 1:1; the `//NOTE: this will override...` comment carried. No oracle.
///
/// # Safety
/// `NPC`/`NPCInfo` set for the current think entity.
pub unsafe fn NPC_BSCinematic() {
    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(QTRUE);
    }

    if UpdateGoal() != core::ptr::null_mut() {
        //have a goalEntity
        //move toward goal, should also face that goal
        NPC_MoveToGoal(QTRUE);
    }

    if !(*NPCInfo).watchTarget.is_null() {
        //have an entity which we want to keep facing
        //NOTE: this will override any angles set by NPC_MoveToGoal
        let mut eyes: vec3_t = [0.0; 3];
        let mut viewSpot: vec3_t = [0.0; 3];
        let mut viewvec: vec3_t = [0.0; 3];
        let mut viewangles: vec3_t = [0.0; 3];

        CalcEntitySpot(NPC, SPOT_HEAD_LEAN, &mut eyes);
        CalcEntitySpot((*NPCInfo).watchTarget, SPOT_HEAD_LEAN, &mut viewSpot);

        VectorSubtract(&viewSpot, &eyes, &mut viewvec);

        vectoangles(&viewvec, &mut viewangles);

        (*NPCInfo).desiredYaw = viewangles[YAW];
        (*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;
        (*NPCInfo).desiredPitch = viewangles[PITCH];
        (*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/// `void NPC_BSWait( void )` (NPC_behavior.c:246).
///
/// Just keep facing our current desired angles. No oracle.
///
/// # Safety
/// `NPC`/`NPCInfo` set for the current think entity.
pub unsafe fn NPC_BSWait() {
    NPC_UpdateAngles(QTRUE, QTRUE);
}

/// `qboolean NPC_CheckInvestigate( int alertEventNum )` (NPC_behavior.c:409).
///
/// Promotes an alert event to an investigate (or, if the NPC is vigilant enough and the
/// owner is a valid enemy, straight to a hunt). Returns `qtrue` if it triggered a behaviour
/// change. Faithful 1:1; the commented-out `NPC_AngerSound` block and the
/// `trap_InPVSIgnorePortals` note are carried. No oracle (`trap_InPVS` + process-global
/// `level`/`NPC`/`NPCInfo`).
///
/// # Safety
/// `NPC`/`NPCInfo` set; `level.alertEvents[alertEventNum]` valid.
pub unsafe fn NPC_CheckInvestigate(alertEventNum: c_int) -> qboolean {
    let owner: *mut gentity_t = (*addr_of!(level)).alertEvents[alertEventNum as usize].owner;
    let invAdd: c_int = (*addr_of!(level)).alertEvents[alertEventNum as usize].level;
    let mut soundPos: vec3_t = [0.0; 3];
    let soundRad: f32 = (*addr_of!(level)).alertEvents[alertEventNum as usize].radius;
    let earshot: f32 = (*NPCInfo).stats.earshot;

    VectorCopy(
        &(*addr_of!(level)).alertEvents[alertEventNum as usize].position,
        &mut soundPos,
    );

    //NOTE: Trying to preserve previous investigation behavior
    if owner.is_null() {
        return QFALSE;
    }

    if (*owner).s.eType != ET_PLAYER && (*owner).s.eType != ET_NPC && owner == (*NPCInfo).goalEntity
    {
        return QFALSE;
    }

    if (*owner).s.eFlags & EF_NODRAW != 0 {
        return QFALSE;
    }

    if (*owner).flags & FL_NOTARGET != 0 {
        return QFALSE;
    }

    if soundRad < earshot {
        return QFALSE;
    }

    //if(!trap_InPVSIgnorePortals(ent->r.currentOrigin, NPC->r.currentOrigin))//should we be able to hear through areaportals?
    if trap::InPVS(&soundPos, &(*NPC).r.currentOrigin) == QFALSE {
        //can hear through doors?
        return QFALSE;
    }

    if !(*owner).client.is_null()
        && (*(*owner).client).playerTeam != 0
        && (*(*NPC).client).playerTeam != 0
        && (*(*owner).client).playerTeam != (*(*NPC).client).playerTeam
    {
        if (*NPCInfo).investigateCount as f32 >= ((*NPCInfo).stats.vigilance * 200.0)
            && !owner.is_null()
        {
            //If investigateCount == 10, just take it as enemy and go
            if ValidEnemy(owner) != QFALSE {
                //FIXME: run angerscript
                G_SetEnemy(NPC, owner);
                (*NPCInfo).goalEntity = (*NPC).enemy;
                (*NPCInfo).goalRadius = 12;
                (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
                return QTRUE;
            }
        } else {
            (*NPCInfo).investigateCount += invAdd;
        }
        //run awakescript
        G_ActivateBehavior(NPC, BSET_AWAKE);

        /*
        if ( Q_irand(0, 10) > 7 )
        {
            NPC_AngerSound();
        }
        */

        //NPCInfo->hlookCount = NPCInfo->vlookCount = 0;
        (*NPCInfo).eventOwner = owner;
        VectorCopy(&soundPos, &mut (*NPCInfo).investigateGoal);
        if (*NPCInfo).investigateCount > 20 {
            (*NPCInfo).investigateDebounceTime = (*addr_of!(level)).time + 10000;
        } else {
            (*NPCInfo).investigateDebounceTime =
                (*addr_of!(level)).time + ((*NPCInfo).investigateCount * 500);
        }
        (*NPCInfo).tempBehavior = BS_INVESTIGATE;
        return QTRUE;
    }

    QFALSE
}

// `#define MIN_ANGLE_ERROR 0.01f` (b_local.h:29) — kept file-local (the copy in
// `npc_utils.rs` is private); used by [`NPC_BSJump`]'s facing check.
const MIN_ANGLE_ERROR: f32 = 0.01;

// `#define APEX_HEIGHT 200.0f` (NPC_behavior.c:730).
const APEX_HEIGHT: f32 = 200.0;
// PARA_WIDTH / JUMP_SPEED (NPC_behavior.c:731-732) are unused outside the
// commented-out apex-height block below, so they are not carried as live consts.

// `qboolean showBBoxes` (defined in NPC_spawn.c:4211) — the debug bbox-render toggle.
// Carried as a file-local `static` (it is never set in the MP build → always `qfalse`),
// matching the C global it gates [`NPC_BSJump`]'s `G_Cube` debug draw on.
#[allow(non_upper_case_globals)] // C global name kept verbatim
static showBBoxes: qboolean = QFALSE;

// `vec3_t NPCDEBUG_BLUE = {0.0, 0.0, 1.0}` (defined in NPC.c:660) — the blue debug colour
// passed to `G_Cube` while the jumping NPC is airborne. Carried as a file-local `static`.
static NPCDEBUG_BLUE: vec3_t = [0.0, 0.0, 1.0];

/// `void NPC_BSJump (void)` (NPC_behavior.c:733).
///
/// Parabola-jump BState handler. Runs a small state machine over `NPCInfo->jumpState`:
/// face the navgoal (`JS_FACING`) → play the crouch anim (`JS_CROUCHING`) → solve a
/// parabola through an apex and launch the NPC (`JS_JUMPING`) → land and complete the
/// nav task (`JS_LANDING`). The commented-out "most desirable apex height" block and the
/// `Com_Printf` debug lines are carried verbatim. The C `assert(z >= 0)` / `assert(xy > 0)`
/// sanity checks → `debug_assert!` (matching C `assert`'s NDEBUG-gated semantics — active
/// in debug, compiled out in release; the `g_cmds.c`/`animalnpc.rs` precedent). No oracle —
/// pure entity/player-state mutation driven by the global `NPC`/`NPCInfo`.
///
/// # Safety
/// `NPC` must be set with a non-null `client`; `level` initialised.
pub unsafe fn NPC_BSJump() {
    let mut dir: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut p1: vec3_t = [0.0; 3];
    let mut p2: vec3_t = [0.0; 3];
    let mut apex: vec3_t = [0.0; 3];
    let time: f32;
    let height: f32;
    let forward: f32;
    let mut z: f32;
    let mut xy: f32;
    let dist: f32;
    let yawError: f32;
    let apexHeight: f32;

    if (*NPCInfo).goalEntity.is_null() {
        //Should have task completed the navgoal
        return;
    }

    if (*NPCInfo).jumpState != JS_JUMPING && (*NPCInfo).jumpState != JS_LANDING {
        //Face navgoal
        VectorSubtract(
            &(*(*NPCInfo).goalEntity).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut dir,
        );
        vectoangles(&dir, &mut angles);
        (*NPCInfo).lockedDesiredPitch = AngleNormalize360(angles[PITCH as usize]);
        (*NPCInfo).desiredPitch = (*NPCInfo).lockedDesiredPitch;
        (*NPCInfo).lockedDesiredYaw = AngleNormalize360(angles[YAW as usize]);
        (*NPCInfo).desiredYaw = (*NPCInfo).lockedDesiredYaw;
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
    yawError = AngleDelta(
        (*(*NPC).client).ps.viewangles[YAW as usize],
        (*NPCInfo).desiredYaw,
    );
    //We don't really care about pitch here

    match (*NPCInfo).jumpState {
        JS_FACING => {
            if yawError < MIN_ANGLE_ERROR {
                //Facing it, Start crouching
                NPC_SetAnim(
                    NPC,
                    SETANIM_LEGS,
                    BOTH_CROUCH1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                (*NPCInfo).jumpState = JS_CROUCHING;
            }
        }
        JS_CROUCHING => {
            if (*(*NPC).client).ps.legsTimer > 0 {
                //Still playing crouching anim
                return;
            }

            //Create a parabola

            if (*NPC).r.currentOrigin[2] > (*(*NPCInfo).goalEntity).r.currentOrigin[2] {
                VectorCopy(&(*NPC).r.currentOrigin, &mut p1);
                VectorCopy(&(*(*NPCInfo).goalEntity).r.currentOrigin, &mut p2);
            } else if (*NPC).r.currentOrigin[2] < (*(*NPCInfo).goalEntity).r.currentOrigin[2] {
                VectorCopy(&(*(*NPCInfo).goalEntity).r.currentOrigin, &mut p1);
                VectorCopy(&(*NPC).r.currentOrigin, &mut p2);
            } else {
                VectorCopy(&(*NPC).r.currentOrigin, &mut p1);
                VectorCopy(&(*(*NPCInfo).goalEntity).r.currentOrigin, &mut p2);
            }

            //z = xy*xy
            VectorSubtract(&p2, &p1, &mut dir);
            dir[2] = 0.0;

            //Get xy and z diffs
            xy = VectorNormalize(&mut dir);
            z = p1[2] - p2[2];

            apexHeight = APEX_HEIGHT / 2.0;
            /*
            //Determine most desirable apex height
            apexHeight = (APEX_HEIGHT * PARA_WIDTH/xy) + (APEX_HEIGHT * z/128);
            if ( apexHeight < APEX_HEIGHT * 0.5 )
            {
                apexHeight = APEX_HEIGHT*0.5;
            }
            else if ( apexHeight > APEX_HEIGHT * 2 )
            {
                apexHeight = APEX_HEIGHT*2;
            }
            */

            //FIXME: length of xy will change curve of parabola, need to account for this
            //somewhere... PARA_WIDTH

            z = ((apexHeight + z).sqrt()) - (apexHeight.sqrt());

            debug_assert!(z >= 0.0);

            //		Com_Printf("apex is %4.2f percent from p1: ", (xy-z)*0.5/xy*100.0f);

            xy -= z;
            xy *= 0.5;

            debug_assert!(xy > 0.0);

            VectorMA(&p1, xy, &dir, &mut apex);
            apex[2] += apexHeight;

            VectorCopy(&apex, &mut (*NPC).pos1);

            //Now we have the apex, aim for it
            height = apex[2] - (*NPC).r.currentOrigin[2];
            time = (height / (0.5 * (*(*NPC).client).ps.gravity as f32)).sqrt();
            if time == 0.0 {
                //			Com_Printf("ERROR no time in jump\n");
                return;
            }

            // set s.origin2 to the push velocity
            VectorSubtract(
                &apex,
                &(*NPC).r.currentOrigin,
                &mut (*(*NPC).client).ps.velocity,
            );
            (*(*NPC).client).ps.velocity[2] = 0.0;
            dist = VectorNormalize(&mut (*(*NPC).client).ps.velocity);

            forward = dist / time;
            VectorScale(
                &(*(*NPC).client).ps.velocity,
                forward,
                &mut (*(*NPC).client).ps.velocity,
            );

            (*(*NPC).client).ps.velocity[2] = time * (*(*NPC).client).ps.gravity as f32;

            //		Com_Printf( "%s jumping %s, gravity at %4.0f percent\n", NPC->targetname, vtos(NPC->client->ps.velocity), NPC->client->ps.gravity/8.0f );

            (*NPC).flags |= FL_NO_KNOCKBACK;
            (*NPCInfo).jumpState = JS_JUMPING;
            //FIXME: jumpsound?
        }
        JS_JUMPING => {
            if showBBoxes != QFALSE {
                VectorAdd(&(*NPC).r.mins, &(*NPC).pos1, &mut p1);
                VectorAdd(&(*NPC).r.maxs, &(*NPC).pos1, &mut p2);
                G_Cube(&p1, &p2, &NPCDEBUG_BLUE, 0.5);
            }

            if (*NPC).s.groundEntityNum != ENTITYNUM_NONE {
                //Landed, start landing anim
                //FIXME: if the
                VectorClear(&mut (*(*NPC).client).ps.velocity);
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_LAND1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                (*NPCInfo).jumpState = JS_LANDING;
                //FIXME: landsound?
            } else if (*(*NPC).client).ps.legsTimer > 0 {
                //Still playing jumping anim
                //FIXME: apply jump velocity here, a couple frames after start, not right away
                return;
            } else {
                //still in air, but done with jump anim, play inair anim
                NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_INAIR1, SETANIM_FLAG_OVERRIDE);
            }
        }
        JS_LANDING => {
            if (*(*NPC).client).ps.legsTimer > 0 {
                //Still playing landing anim
            } else {
                (*NPCInfo).jumpState = JS_WAITING;

                //task complete no matter what...
                NPC_ClearGoal();
                (*NPCInfo).goalTime = (*addr_of!(level)).time;
                (*NPCInfo).aiFlags &= !NPCAI_MOVING;
                (*addr_of_mut!(ucmd)).forwardmove = 0;
                (*NPC).flags &= !FL_NO_KNOCKBACK;
                //Return that the goal was reached
                trap::ICARUS_TaskIDComplete(NPC, TID_MOVE_NAV);

                //Or should we keep jumping until reached goal?

                /*
                NPCInfo->goalEntity = UpdateGoal();
                if ( !NPCInfo->goalEntity )
                {
                    NPC->flags &= ~FL_NO_KNOCKBACK;
                    Q3_TaskIDComplete( NPC, TID_MOVE_NAV );
                }
                */
            }
        }
        // JS_WAITING and default
        _ => {
            (*NPCInfo).jumpState = JS_FACING;
        }
    }
}

/// `void NPC_BSRemove (void)` (NPC_behavior.c:921).
///
/// If out of PVS of client 0, fire `target3`, hide the NPC, strip it solid/health, and
/// schedule it to free itself in `FRAMETIME`. Faithful 1:1; the `rwwFIXMEFIXME` and
/// `cg.vieworg` notes are carried. No oracle (`trap_InPVS` + entity-state mutation).
///
/// # Safety
/// `NPC` set; `g_entities`/`level` initialised.
pub unsafe fn NPC_BSRemove() {
    NPC_UpdateAngles(QTRUE, QTRUE);
    if trap::InPVS(
        &(*NPC).r.currentOrigin,
        &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(0))
            .r
            .currentOrigin,
    ) == QFALSE
    //FIXME: use cg.vieworg?
    {
        //rwwFIXMEFIXME: Care about all clients instead of just 0?
        G_UseTargets2(NPC, NPC, (*NPC).target3);
        (*NPC).s.eFlags |= EF_NODRAW;
        (*NPC).s.eType = ET_INVISIBLE;
        (*NPC).r.contents = 0;
        (*NPC).health = 0;
        (*NPC).targetname = core::ptr::null_mut();

        //Disappear in half a second
        (*NPC).think = Some(G_FreeEntity);
        (*NPC).nextthink = (*addr_of!(level)).time + FRAMETIME;
    } //FIXME: else allow for out of FOV???
}

/// `void NPC_BSSearchStart( int homeWp, bState_t bState )` (NPC_behavior.c:1132).
///
/// Seed a homeWp-anchored search: pick a home waypoint if none given, stamp the temp
/// behaviour, and aim the tempGoal at the home node. Faithful 1:1 (the `//FIXME:
/// Reimplement` and `Com_Printf` notes carried). No oracle (`trap_Nav_*` + state).
///
/// # Safety
/// `NPC`/`NPCInfo` set; nav system available.
pub unsafe fn NPC_BSSearchStart(mut homeWp: c_int, bState: c_int) {
    //FIXME: Reimplement
    if homeWp == WAYPOINT_NONE {
        homeWp = NAV_FindClosestWaypointForEnt(NPC, WAYPOINT_NONE);
        if (*NPC).waypoint == WAYPOINT_NONE {
            (*NPC).waypoint = homeWp;
        }
    }
    (*NPCInfo).homeWp = homeWp;
    (*NPCInfo).tempBehavior = bState;
    (*NPCInfo).aiFlags |= NPCAI_ENROUTE_TO_HOMEWP;
    (*NPCInfo).investigateDebounceTime = 0;
    trap::Nav_GetNodePosition(homeWp, &mut (*(*NPCInfo).tempGoal).r.currentOrigin);
    (*(*NPCInfo).tempGoal).waypoint = homeWp;
    //Com_Printf("\nHeading for wp %d...\n", NPCInfo->homeWp);
}

/// `void NPC_BSWander (void)` (NPC_behavior.c:1193).
///
/// Wander between nav nodes, pausing to look around at each. Faithful 1:1 (the leading
/// `//FIXME: don't actually go all the way...` and interior comments carried). No oracle
/// (`trap_Nav_*` + process-global state + `Q_irand`/`flrand`).
///
/// # Safety
/// `NPC`/`NPCInfo` set; nav system available.
pub unsafe fn NPC_BSWander() {
    //FIXME: don't actually go all the way to the next waypoint, just move in fits and jerks...?
    if (*NPCInfo).investigateDebounceTime == 0 {
        //Starting out
        let mut minGoalReachedDistSquared: f32 = 64.0; //32*32;
        let mut vec: vec3_t = [0.0; 3];

        //Keep moving toward our tempGoal
        (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;

        VectorSubtract(
            &(*(*NPCInfo).tempGoal).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut vec,
        );

        if (*(*NPCInfo).tempGoal).waypoint != WAYPOINT_NONE {
            minGoalReachedDistSquared = 64.0;
        }

        if VectorLengthSquared(&vec) < minGoalReachedDistSquared {
            //Close enough, just got there
            (*NPC).waypoint = NAV_FindClosestWaypointForEnt(NPC, WAYPOINT_NONE);

            if Q_irand(0, 1) == 0 {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_GUARD_LOOKAROUND1,
                    SETANIM_FLAG_NORMAL,
                );
            } else {
                NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_IDLE1, SETANIM_FLAG_NORMAL);
            }
            //Just got here, so Look around for a while
            (*NPCInfo).investigateDebounceTime = (*addr_of!(level)).time + Q_irand(3000, 10000);
        } else {
            //Keep moving toward goal
            NPC_MoveToGoal(QTRUE);
        }
    } else {
        //We're there
        if (*NPCInfo).investigateDebounceTime > (*addr_of!(level)).time {
            //Still waiting around for a bit
            //Turn angles every now and then to look around
            if (*(*NPCInfo).tempGoal).waypoint != WAYPOINT_NONE {
                if Q_irand(0, 30) == 0 {
                    let numEdges: c_int =
                        trap::Nav_GetNodeNumEdges((*(*NPCInfo).tempGoal).waypoint);

                    if numEdges != WAYPOINT_NONE {
                        let branchNum: c_int = Q_irand(0, numEdges - 1);

                        let mut branchPos: vec3_t = [0.0; 3];
                        let mut lookDir: vec3_t = [0.0; 3];

                        let nextWp: c_int =
                            trap::Nav_GetNodeEdge((*(*NPCInfo).tempGoal).waypoint, branchNum);
                        trap::Nav_GetNodePosition(nextWp, &mut branchPos);

                        VectorSubtract(
                            &branchPos,
                            &(*(*NPCInfo).tempGoal).r.currentOrigin,
                            &mut lookDir,
                        );
                        (*NPCInfo).desiredYaw =
                            AngleNormalize360(vectoyaw(&lookDir) + flrand(-45.0, 45.0));
                    }
                }
            }
        } else {
            //Just finished waiting
            (*NPC).waypoint = NAV_FindClosestWaypointForEnt(NPC, WAYPOINT_NONE);

            if (*NPC).waypoint != WAYPOINT_NONE {
                let numEdges: c_int = trap::Nav_GetNodeNumEdges((*NPC).waypoint);

                if numEdges != WAYPOINT_NONE {
                    let branchNum: c_int = Q_irand(0, numEdges - 1);

                    let nextWp: c_int = trap::Nav_GetNodeEdge((*NPC).waypoint, branchNum);
                    trap::Nav_GetNodePosition(nextWp, &mut (*(*NPCInfo).tempGoal).r.currentOrigin);
                    (*(*NPCInfo).tempGoal).waypoint = nextWp;
                }

                (*NPCInfo).investigateDebounceTime = 0;
                //Start moving toward our tempGoal
                (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;
                NPC_MoveToGoal(QTRUE);
            }
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/// `qboolean NPC_CheckSurrender( void )` (NPC_behavior.c:1341).
///
/// Decide whether to surrender to a close, threatening, attacking enemy. Returns the
/// short-circuit `qfalse` for every disqualifying condition; the big commented-out
/// group-based surrender block (`NPCInfo->group`) is carried verbatim — so this body
/// always returns `qfalse` once it reaches the fall-throughs (matching the C, whose only
/// live `qtrue` paths are inside that commented block). No oracle (`trap_ICARUS_*`/
/// `trap_InPVS` + process-global state).
///
/// # Safety
/// `NPC` set; `NPC->client` non-null; `level` initialised.
pub unsafe fn NPC_CheckSurrender() -> qboolean {
    if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE
        && (*(*NPC).client).ps.groundEntityNum != ENTITYNUM_NONE
        && (*(*NPC).client).ps.weaponTime == 0
        && PM_InKnockDown(&mut (*(*NPC).client).ps) == QFALSE
        && !(*NPC).enemy.is_null()
        && !(*(*NPC).enemy).client.is_null()
        && (*(*NPC).enemy).enemy == NPC
        && (*(*NPC).enemy).s.weapon != WP_NONE
        && (*(*NPC).enemy).s.weapon != WP_STUN_BATON
        && (*(*NPC).enemy).health > 20
        && (*(*NPC).enemy).painDebounceTime < (*addr_of!(level)).time - 3000
        && (*(*(*NPC).enemy).client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize]
            < (*addr_of!(level)).time - 1000
    {
        //don't surrender if scripted to run somewhere or if we're in the air or if we're busy or if we don't have an enemy or if the enemy is not mad at me or is hurt or not a threat or busy being attacked
        //FIXME: even if not in a group, don't surrender if there are other enemies in the PVS and within a certain range?
        if (*NPC).s.weapon != WP_ROCKET_LAUNCHER
            && (*NPC).s.weapon != WP_REPEATER
            && (*NPC).s.weapon != WP_FLECHETTE
            && (*NPC).s.weapon != WP_SABER
        {
            //jedi and heavy weapons guys never surrender
            //FIXME: rework all this logic into some orderly fashion!!!
            if (*NPC).s.weapon != WP_NONE {
                //they have a weapon so they'd have to drop it to surrender
                //don't give up unless low on health
                if (*NPC).health > 25
                /*|| NPC->health >= NPC->max_health*/
                {
                    //rwwFIXMEFIXME: Keep max health not a ps state?
                    return QFALSE;
                }
                //if ( g_crosshairEntNum == NPC->s.number && NPC->painDebounceTime > level.time )
                if NPC_SomeoneLookingAtMe(NPC) != QFALSE
                    && (*NPC).painDebounceTime > (*addr_of!(level)).time
                {
                    //if he just shot me, always give up
                    //fall through
                } else {
                    //don't give up unless facing enemy and he's very close
                    if InFOV((*NPC).enemy, NPC, 60, 30) == QFALSE {
                        //I'm not looking at them
                        return QFALSE;
                    } else if DistanceSquared(
                        &(*NPC).r.currentOrigin,
                        &(*(*NPC).enemy).r.currentOrigin,
                    ) < 65536.0
                    /*256*256*/
                    {
                        //they're not close
                        return QFALSE;
                    } else if trap::InPVS(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
                        == QFALSE
                    {
                        //they're not in the same room
                        return QFALSE;
                    }
                }
            }
            //fixme: this logic keeps making npc's randomly surrender
            /*
            if ( NPCInfo->group && NPCInfo->group->numGroup <= 1 )
            {//I'm alone but I was in a group//FIXME: surrender anyway if just melee or no weap?
                if ( NPC->s.weapon == WP_NONE
                    //NPC has a weapon
                    || (NPC->enemy && NPC->enemy->s.number < MAX_CLIENTS)
                    || (NPC->enemy->s.weapon == WP_SABER&&NPC->enemy->client&&!NPC->enemy->client->ps.saberHolstered)
                    || (NPC->enemy->NPC && NPC->enemy->NPC->group && NPC->enemy->NPC->group->numGroup > 2) )
                {//surrender only if have no weapon or fighting a player or jedi or if we are outnumbered at least 3 to 1
                    if ( (NPC->enemy && NPC->enemy->s.number < MAX_CLIENTS) )
                    {//player is the guy I'm running from
                        //if ( g_crosshairEntNum == NPC->s.number )
                        if (NPC_SomeoneLookingAtMe(NPC))
                        {//give up if player is aiming at me
                            NPC_Surrender();
                            NPC_UpdateAngles( qtrue, qtrue );
                            return qtrue;
                        }
                        else if ( NPC->enemy->s.weapon == WP_SABER )
                        {//player is using saber
                            if ( InFOV( NPC, NPC->enemy, 60, 30 ) )
                            {//they're looking at me
                                if ( DistanceSquared( NPC->r.currentOrigin, NPC->enemy->r.currentOrigin ) < 16384 )
                                {//they're close
                                    if ( trap_InPVS( NPC->r.currentOrigin, NPC->enemy->r.currentOrigin ) )
                                    {//they're in the same room
                                        NPC_Surrender();
                                        NPC_UpdateAngles( qtrue, qtrue );
                                        return qtrue;
                                    }
                                }
                            }
                        }
                    }
                    else if ( NPC->enemy )
                    {//???
                        //should NPC's surrender to others?
                        if ( InFOV( NPC, NPC->enemy, 30, 30 ) )
                        {//they're looking at me
                            if ( DistanceSquared( NPC->r.currentOrigin, NPC->enemy->r.currentOrigin ) < 4096 )
                            {//they're close
                                if ( trap_InPVS( NPC->r.currentOrigin, NPC->enemy->r.currentOrigin ) )
                                {//they're in the same room
                                    //FIXME: should player-team NPCs not fire on surrendered NPCs?
                                    NPC_Surrender();
                                    NPC_UpdateAngles( qtrue, qtrue );
                                    return qtrue;
                                }
                            }
                        }
                    }
                }
            }
            */
        }
    }
    QFALSE
}

/// `void NPC_BSFlee( void )` (NPC_behavior.c:1444).
///
/// Flee toward a goal away from danger: drop out of the temp flee state when the timer
/// expires, surrender if appropriate, then pick a waypoint branch that runs away from the
/// danger direction and move there (cowering/surrendering when weaponless and trapped).
/// Faithful 1:1; interior FIXMEs carried. No oracle (`trap_Nav_*` + process-global state).
///
/// # Safety
/// `NPC`/`NPCInfo` set; nav system available.
pub unsafe fn NPC_BSFlee() {
    //FIXME: keep checking for danger
    let mut goal: *mut gentity_t;

    if TIMER_Done(NPC, c"flee".as_ptr()) != QFALSE && (*NPCInfo).tempBehavior == BS_FLEE {
        (*NPCInfo).tempBehavior = BS_DEFAULT;
        (*NPCInfo).squadState = SQUAD_IDLE;
        //FIXME: should we set some timer to make him stay in this spot for a bit,
        //so he doesn't just suddenly turn around and come back at the enemy?
        //OR, just stop running toward goal for last second or so of flee?
    }
    if NPC_CheckSurrender() != QFALSE {
        return;
    }
    goal = (*NPCInfo).goalEntity;
    if goal.is_null() {
        goal = (*NPCInfo).lastGoalEntity;
        if goal.is_null() {
            //???!!!
            goal = (*NPCInfo).tempGoal;
        }
    }

    if !goal.is_null() {
        let moved: qboolean;
        let mut reverseCourse: qboolean = QTRUE;

        //FIXME: if no weapon, find one and run to pick it up?

        //Let's try to find a waypoint that gets me away from this thing
        if (*NPC).waypoint == WAYPOINT_NONE {
            (*NPC).waypoint = NAV_GetNearestNode(NPC, (*NPC).lastWaypoint);
        }
        if (*NPC).waypoint != WAYPOINT_NONE {
            let numEdges: c_int = trap::Nav_GetNodeNumEdges((*NPC).waypoint);

            if numEdges != WAYPOINT_NONE {
                let mut dangerDir: vec3_t = [0.0; 3];
                let mut branchNum: c_int;

                VectorSubtract(
                    &(*NPCInfo).investigateGoal,
                    &(*NPC).r.currentOrigin,
                    &mut dangerDir,
                );
                VectorNormalize(&mut dangerDir);

                branchNum = 0;
                while branchNum < numEdges {
                    let mut branchPos: vec3_t = [0.0; 3];
                    let mut runDir: vec3_t = [0.0; 3];

                    let nextWp: c_int = trap::Nav_GetNodeEdge((*NPC).waypoint, branchNum);
                    trap::Nav_GetNodePosition(nextWp, &mut branchPos);

                    VectorSubtract(&branchPos, &(*NPC).r.currentOrigin, &mut runDir);
                    VectorNormalize(&mut runDir);
                    if DotProduct(&runDir, &dangerDir) > flrand(0.0, 0.5) {
                        //don't run toward danger
                        branchNum += 1;
                        continue;
                    }
                    //FIXME: don't want to ping-pong back and forth
                    NPC_SetMoveGoal(NPC, &branchPos, 0, QTRUE, -1, core::ptr::null_mut());
                    reverseCourse = QFALSE;
                    break;
                }
            }
        }

        moved = NPC_MoveToGoal(QFALSE); //qtrue? (do try to move straight to (away from) goal)

        if (*NPC).s.weapon == WP_NONE && (moved == QFALSE || reverseCourse != QFALSE) {
            //No weapon and no escape route... Just cower?  Need anim.
            NPC_Surrender();
            NPC_UpdateAngles(QTRUE, QTRUE);
            return;
        }
        //If our move failed, then just run straight away from our goal
        //FIXME: We really shouldn't do this.
        if moved == QFALSE {
            let mut dir: vec3_t = [0.0; 3];
            let dist: f32;
            if reverseCourse != QFALSE {
                VectorSubtract(&(*NPC).r.currentOrigin, &(*goal).r.currentOrigin, &mut dir);
            } else {
                VectorSubtract(&(*goal).r.currentOrigin, &(*NPC).r.currentOrigin, &mut dir);
            }
            dist = VectorNormalize(&mut dir);
            (*NPCInfo).distToGoal = dist;
            (*NPCInfo).desiredYaw = vectoyaw(&dir);
            (*NPCInfo).desiredPitch = 0.0;
            (*addr_of_mut!(ucmd)).forwardmove = 127;
        } else if reverseCourse != QFALSE {
            //ucmd.forwardmove *= -1;
            //ucmd.rightmove *= -1;
            //VectorScale( NPC->client->ps.moveDir, -1, NPC->client->ps.moveDir );
            (*NPCInfo).desiredYaw *= -1.0;
        }
        //FIXME: can stop after a safe distance?
        //ucmd.upmove = 0;
        (*addr_of_mut!(ucmd)).buttons &= !BUTTON_WALKING;
        //FIXME: what do we do once we've gotten to our goal?
    }
    NPC_UpdateAngles(QTRUE, QTRUE);

    NPC_CheckGetNewWeapon();
}

/// `void NPC_StartFlee( gentity_t *enemy, vec3_t dangerPoint, int dangerLevel, int fleeTimeMin, int fleeTimeMax )` (NPC_behavior.c:1560).
///
/// Begin fleeing: prefer a script, else find a combat point (relaxing cover/avoid/no-PVS
/// constraints in turn) or run straight away; set the flee/attack/panic timers. Faithful
/// 1:1; interior FIXMEs carried. No oracle (`trap_ICARUS_*`/nav + process-global state).
///
/// # Safety
/// `NPC`/`NPCInfo`/`NPC->client` set; `level` + nav available.
pub unsafe fn NPC_StartFlee(
    enemy: *mut gentity_t,
    dangerPoint: &vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    let mut cp: c_int = -1;

    if trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) != QFALSE {
        //running somewhere that a script requires us to go, don't interrupt that!
        return;
    }

    //if have a fleescript, run that instead
    if G_ActivateBehavior(NPC, BSET_FLEE) != QFALSE {
        return;
    }
    //FIXME: play a flee sound?  Appropriate to situation?
    if !enemy.is_null() {
        G_SetEnemy(NPC, enemy);
    }

    //FIXME: if don't have a weapon, find nearest one we have a route to and run for it?
    if dangerLevel > AEL_DANGER
        || (*NPC).s.weapon == WP_NONE
        || (((*NPCInfo).group.is_null() || (*(*NPCInfo).group).numGroup <= 1)
            && (*NPC).health <= 10)
    {
        //IF either great danger OR I have no weapon OR I'm alone and low on health, THEN try to find a combat point out of PVS
        cp = NPC_FindCombatPoint(
            &(*NPC).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            dangerPoint,
            CP_COVER | CP_AVOID | CP_HAS_ROUTE | CP_NO_PVS,
            128.0,
            -1,
        );
    }
    //FIXME: still happens too often...
    if cp == -1 {
        //okay give up on the no PVS thing
        cp = NPC_FindCombatPoint(
            &(*NPC).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            dangerPoint,
            CP_COVER | CP_AVOID | CP_HAS_ROUTE,
            128.0,
            -1,
        );
        if cp == -1 {
            //okay give up on the avoid
            cp = NPC_FindCombatPoint(
                &(*NPC).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                dangerPoint,
                CP_COVER | CP_HAS_ROUTE,
                128.0,
                -1,
            );
            if cp == -1 {
                //okay give up on the cover
                cp = NPC_FindCombatPoint(
                    &(*NPC).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    dangerPoint,
                    CP_HAS_ROUTE,
                    128.0,
                    -1,
                );
            }
        }
    }

    //see if we got a valid one
    if cp != -1 {
        //found a combat point
        NPC_SetCombatPoint(cp);
        NPC_SetMoveGoal(
            NPC,
            &(*addr_of!(level)).combatPoints[cp as usize].origin,
            8,
            QTRUE,
            cp,
            core::ptr::null_mut(),
        );
        (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
        (*NPCInfo).tempBehavior = BS_DEFAULT;
    } else {
        //need to just run like hell!
        if (*NPC).s.weapon != WP_NONE {
            return; //let's just not flee?
        } else {
            //FIXME: other evasion AI?  Duck?  Strafe?  Dodge?
            (*NPCInfo).tempBehavior = BS_FLEE;
            //Run straight away from here... FIXME: really want to find farthest waypoint/navgoal from this pos... maybe based on alert event radius?
            NPC_SetMoveGoal(NPC, dangerPoint, 0, QTRUE, -1, core::ptr::null_mut());
            //store the danger point
            VectorCopy(dangerPoint, &mut (*NPCInfo).investigateGoal); //FIXME: make a new field for this?
        }
    }
    //FIXME: localize this Timer?
    TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2500));
    //FIXME: is this always applicable?
    (*NPCInfo).squadState = SQUAD_RETREAT;
    TIMER_Set(NPC, c"flee".as_ptr(), Q_irand(fleeTimeMin, fleeTimeMax));
    TIMER_Set(NPC, c"panic".as_ptr(), Q_irand(1000, 4000)); //how long to wait before trying to nav to a dropped weapon

    if (*(*NPC).client).NPC_class != CLASS_PROTOCOL {
        TIMER_Set(NPC, c"duck".as_ptr(), 0);
    }
}

/// `void G_StartFlee( gentity_t *self, gentity_t *enemy, vec3_t dangerPoint, int dangerLevel, int fleeTimeMin, int fleeTimeMax )` (NPC_behavior.c:1636).
///
/// `SetNPCGlobals` wrapper so any caller can make an arbitrary NPC `self` flee. No oracle.
///
/// # Safety
/// `self` valid; if `self->NPC` is non-null it must be set up for the NPC globals.
pub unsafe fn G_StartFlee(
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
    dangerPoint: &vec3_t,
    dangerLevel: c_int,
    fleeTimeMin: c_int,
    fleeTimeMax: c_int,
) {
    if (*self_).NPC.is_null() {
        //player
        return;
    }
    SaveNPCGlobals();
    SetNPCGlobals(self_);

    NPC_StartFlee(enemy, dangerPoint, dangerLevel, fleeTimeMin, fleeTimeMax);

    RestoreNPCGlobals();
}

/// `void NPC_BSFollowLeader (void)` (NPC_behavior.c:524).
///
/// Follow your leader, staying within visibility and a certain distance band: if no leader,
/// fall back to stand-guard; find/clear an enemy (or inherit the leader's), face and fire if
/// armed (lightsaber users switch to a temp hunt-and-kill), otherwise face the leader; then
/// close/back-off to maintain the follow distance and clear do-not-enter brushes. Faithful
/// 1:1; all interior FIXMEs/rwwFIXMEs carried. No oracle — process-global `NPC`/`NPCInfo`/
/// `ucmd`/`enemyVisibility` + `trap_*`/nav, no return value to parity-check.
///
/// Dispatched from `npc.rs`, so `pub(crate)`.
///
/// # Safety
/// `NPC`/`NPCInfo`/`NPC->client` set for the current think entity; `level`/nav available.
pub(crate) unsafe fn NPC_BSFollowLeader() {
    let mut vec: vec3_t = [0.0; 3];
    let leaderDist: f32;
    let leaderVis: c_int;
    let curAnim: c_int;

    if (*(*NPC).client).leader.is_null() {
        //ok, stand guard until we find an enemy
        if (*NPCInfo).tempBehavior == BS_HUNT_AND_KILL {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        } else {
            (*NPCInfo).tempBehavior = BS_STAND_GUARD;
            NPC_BSStandGuard();
        }
        return;
    }

    if (*NPC).enemy.is_null() {
        //no enemy, find one
        NPC_CheckEnemy(
            ((*NPCInfo).confusionTime < (*addr_of!(level)).time) as qboolean,
            QFALSE,
            QTRUE,
        ); //don't find new enemy if this is tempbehav
        if !(*NPC).enemy.is_null() {
            //just found one
            (*NPCInfo).enemyCheckDebounceTime = (*addr_of!(level)).time + Q_irand(3000, 10000);
        } else {
            if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
                let eventID: c_int = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QFALSE, AEL_MINOR);
                if (*addr_of!(level)).alertEvents[eventID as usize].level >= AEL_SUSPICIOUS
                    && (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0
                {
                    (*NPCInfo).lastAlertID = (*addr_of!(level)).alertEvents[eventID as usize].ID;
                    if (*addr_of!(level)).alertEvents[eventID as usize]
                        .owner
                        .is_null()
                        || (*(*addr_of!(level)).alertEvents[eventID as usize].owner)
                            .client
                            .is_null()
                        || (*(*addr_of!(level)).alertEvents[eventID as usize].owner).health <= 0
                        || (*(*(*addr_of!(level)).alertEvents[eventID as usize].owner).client)
                            .playerTeam
                            != (*(*NPC).client).enemyTeam
                    {
                        //not an enemy
                    } else {
                        //FIXME: what if can't actually see enemy, don't know where he is... should we make them just become very alert and start looking for him?  Or just let combat AI handle this... (act as if you lost him)
                        G_SetEnemy(NPC, (*addr_of!(level)).alertEvents[eventID as usize].owner);
                        (*NPCInfo).enemyCheckDebounceTime =
                            (*addr_of!(level)).time + Q_irand(3000, 10000);
                        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
                        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 1000));
                    }
                }
            }
        }
        if (*NPC).enemy.is_null() {
            if !(*(*NPC).client).leader.is_null()
                && !(*(*(*NPC).client).leader).enemy.is_null()
                && (*(*(*NPC).client).leader).enemy != NPC
                && ((!(*(*(*(*NPC).client).leader).enemy).client.is_null()
                    && (*(*(*(*(*NPC).client).leader).enemy).client).playerTeam
                        == (*(*NPC).client).enemyTeam)
                    || (/*NPC->client->leader->enemy->r.svFlags&SVF_NONNPC_ENEMY*/false
                        && (*(*(*(*NPC).client).leader).enemy).alliedTeam
                            == (*(*NPC).client).enemyTeam))
                && (*(*(*(*NPC).client).leader).enemy).health > 0
            {
                //rwwFIXMEFIXME: use SVF_NONNPC_ENEMY?
                G_SetEnemy(NPC, (*(*(*NPC).client).leader).enemy);
                (*NPCInfo).enemyCheckDebounceTime = (*addr_of!(level)).time + Q_irand(3000, 10000);
                (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
            }
        }
    } else {
        if (*(*NPC).enemy).health <= 0 || (*(*NPC).enemy).flags & FL_NOTARGET != 0 {
            G_ClearEnemy(NPC);
            if (*NPCInfo).enemyCheckDebounceTime > (*addr_of!(level)).time + 1000 {
                (*NPCInfo).enemyCheckDebounceTime = (*addr_of!(level)).time + Q_irand(1000, 2000);
            }
        } else if (*(*NPC).client).ps.weapon != 0
            && (*NPCInfo).enemyCheckDebounceTime < (*addr_of!(level)).time
        {
            NPC_CheckEnemy(
                ((*NPCInfo).confusionTime < (*addr_of!(level)).time
                    || (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER) as qboolean,
                QFALSE,
                QTRUE,
            ); //don't find new enemy if this is tempbehav
        }
    }

    if !(*NPC).enemy.is_null() && (*(*NPC).client).ps.weapon != 0 {
        //If have an enemy, face him and fire
        if (*(*NPC).client).ps.weapon == WP_SABER {
            //|| NPCInfo->confusionTime>level.time )
            //lightsaber user or charmed enemy
            if (*NPCInfo).tempBehavior != BS_FOLLOW_LEADER {
                //not already in a temp bState
                //go after the guy
                (*NPCInfo).tempBehavior = BS_HUNT_AND_KILL;
                NPC_UpdateAngles(QTRUE, QTRUE);
                return;
            }
        }

        enemyVisibility = NPC_CheckVisibility((*NPC).enemy, CHECK_FOV | CHECK_SHOOT); //CHECK_360|CHECK_PVS|
        if enemyVisibility > VIS_PVS {
            //face
            let mut enemy_org: vec3_t = [0.0; 3];
            let mut muzzle: vec3_t = [0.0; 3];
            let mut delta: vec3_t = [0.0; 3];
            let mut angleToEnemy: vec3_t = [0.0; 3];
            let _distanceToEnemy: f32;

            CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_org);
            NPC_AimWiggle(&mut enemy_org);

            CalcEntitySpot(NPC, SPOT_WEAPON, &mut muzzle);

            VectorSubtract(&enemy_org, &muzzle, &mut delta);
            vectoangles(&delta, &mut angleToEnemy);
            _distanceToEnemy = VectorNormalize(&mut delta);

            (*NPCInfo).desiredYaw = angleToEnemy[YAW];
            (*NPCInfo).desiredPitch = angleToEnemy[PITCH];
            NPC_UpdateFiringAngles(QTRUE, QTRUE);

            if enemyVisibility >= VIS_SHOOT {
                //shoot
                NPC_AimAdjust(2);
                if NPC_GetHFOVPercentage(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &(*(*NPC).client).ps.viewangles,
                    (*NPCInfo).stats.hfov as f32,
                ) > 0.6f32
                    && NPC_GetHFOVPercentage(
                        &(*(*NPC).enemy).r.currentOrigin,
                        &(*NPC).r.currentOrigin,
                        &(*(*NPC).client).ps.viewangles,
                        (*NPCInfo).stats.vfov as f32,
                    ) > 0.5f32
                {
                    //actually withing our front cone
                    WeaponThink(QTRUE);
                }
            } else {
                NPC_AimAdjust(1);
            }

            //NPC_CheckCanAttack(1.0, qfalse);
        } else {
            NPC_AimAdjust(-1);
        }
    } else {
        //FIXME: combine with vector calc below
        let mut head: vec3_t = [0.0; 3];
        let mut leaderHead: vec3_t = [0.0; 3];
        let mut delta: vec3_t = [0.0; 3];
        let mut angleToLeader: vec3_t = [0.0; 3];

        CalcEntitySpot((*(*NPC).client).leader, SPOT_HEAD, &mut leaderHead);
        CalcEntitySpot(NPC, SPOT_HEAD, &mut head);
        VectorSubtract(&leaderHead, &head, &mut delta);
        vectoangles(&delta, &mut angleToLeader);
        VectorNormalize(&mut delta);
        (*(*NPC).NPC).desiredYaw = angleToLeader[YAW];
        (*(*NPC).NPC).desiredPitch = angleToLeader[PITCH];

        NPC_UpdateAngles(QTRUE, QTRUE);
    }

    //leader visible?
    leaderVis = NPC_CheckVisibility((*(*NPC).client).leader, CHECK_PVS | CHECK_360 | CHECK_SHOOT); //			ent->e_UseFunc = useF_NULL;

    //Follow leader, stay within visibility and a certain distance, maintain a distance from.
    curAnim = (*(*NPC).client).ps.legsAnim;
    if curAnim != BOTH_ATTACK1
        && curAnim != BOTH_ATTACK2
        && curAnim != BOTH_ATTACK3
        && curAnim != BOTH_MELEE1
        && curAnim != BOTH_MELEE2
    {
        //Don't move toward leader if we're in a full-body attack anim
        //FIXME, use IdealDistance to determine if we need to close distance
        let mut followDist: f32 = 96.0; //FIXME:  If there are enmies, make this larger?
        let backupdist: f32;
        let walkdist: f32;
        let minrundist: f32;
        let leaderHDist: f32;

        if (*NPCInfo).followDist != 0.0 {
            followDist = (*NPCInfo).followDist;
        }
        backupdist = followDist / 2.0;
        walkdist = followDist * 0.83;
        minrundist = followDist * 1.33;

        VectorSubtract(
            &(*(*(*NPC).client).leader).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut vec,
        );
        leaderDist = VectorLength(&vec); //FIXME: make this just nav distance?
                                         //never get within their radius horizontally
        vec[2] = 0.0;
        leaderHDist = VectorLength(&vec);
        if leaderHDist > backupdist && (leaderVis != VIS_SHOOT || leaderDist > walkdist) {
            //We should close in?
            (*NPCInfo).goalEntity = (*(*NPC).client).leader;

            NPC_SlideMoveToGoal();
            if leaderVis == VIS_SHOOT && leaderDist < minrundist {
                (*addr_of_mut!(ucmd)).buttons |= BUTTON_WALKING;
            }
        } else if leaderDist < backupdist {
            //We should back off?
            (*NPCInfo).goalEntity = (*(*NPC).client).leader;
            NPC_SlideMoveToGoal();

            //reversing direction
            (*addr_of_mut!(ucmd)).forwardmove = -(*addr_of!(ucmd)).forwardmove;
            (*addr_of_mut!(ucmd)).rightmove = -(*addr_of!(ucmd)).rightmove;
            let mut neg_move: vec3_t = [0.0; 3];
            VectorScale(&(*(*NPC).client).ps.moveDir, -1.0, &mut neg_move);
            (*(*NPC).client).ps.moveDir = neg_move;
        } //otherwise, stay where we are
          //check for do not enter and stop if there's one there...
        if (*addr_of!(ucmd)).forwardmove != 0
            || (*addr_of!(ucmd)).rightmove != 0
            || VectorCompare(&vec3_origin, &(*(*NPC).client).ps.moveDir) != 0
        {
            NPC_MoveDirClear(
                (*addr_of!(ucmd)).forwardmove as c_int,
                (*addr_of!(ucmd)).rightmove as c_int,
                QTRUE,
            );
        }
    }
}

/// `void NPC_BSSearch (void)` (NPC_behavior.c:939).
///
/// Search the immediate branches of waypoints for enemies: if an enemy turns up, switch to
/// hunt-and-kill (or drop the temp behaviour); otherwise navigate to the tempGoal, look
/// around for a while on arrival (running the lost-enemy script the first time we reach the
/// homeWp), then pick a neighbouring waypoint branch and head out / return home. Faithful
/// 1:1; all `//FIXME: Reimplement` / commented-out waypoint-radius blocks carried verbatim.
/// No oracle — process-global `NPC`/`NPCInfo` + `trap_Nav_*` + `Q_irand`/`flrand`.
///
/// Dispatched from `npc.rs`, so `pub(crate)`.
///
/// # Safety
/// `NPC`/`NPCInfo` set for the current think entity; `level`/nav available.
pub(crate) unsafe fn NPC_BSSearch() {
    NPC_CheckEnemy(QTRUE, QFALSE, QTRUE);
    //Look for enemies, if find one:
    if !(*NPC).enemy.is_null() {
        if (*NPCInfo).tempBehavior == BS_SEARCH {
            //if tempbehavior, set tempbehavior to default
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        } else {
            //if bState, change to run and shoot
            (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
            NPC_BSRunAndShoot();
        }
        return;
    }

    //FIXME: what if our goalEntity is not NULL and NOT our tempGoal - they must
    //want us to do something else?  If tempBehavior, just default, else set
    //to run and shoot...?

    //FIXME: Reimplement

    if (*NPCInfo).investigateDebounceTime == 0 {
        //On our way to a tempGoal
        let mut minGoalReachedDistSquared: f32 = (32 * 32) as f32;
        let mut vec: vec3_t = [0.0; 3];

        //Keep moving toward our tempGoal
        (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;

        VectorSubtract(
            &(*(*NPCInfo).tempGoal).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut vec,
        );
        if vec[2] < 24.0 {
            vec[2] = 0.0;
        }

        if (*(*NPCInfo).tempGoal).waypoint != WAYPOINT_NONE {
            /*
            //FIXME: can't get the radius...
            float	wpRadSq = waypoints[NPCInfo->tempGoal->waypoint].radius * waypoints[NPCInfo->tempGoal->waypoint].radius;
            if ( minGoalReachedDistSquared > wpRadSq )
            {
                minGoalReachedDistSquared = wpRadSq;
            }
            */

            minGoalReachedDistSquared = (32 * 32) as f32; //12*12;
        }

        if VectorLengthSquared(&vec) < minGoalReachedDistSquared {
            //Close enough, just got there
            (*NPC).waypoint = NAV_FindClosestWaypointForEnt(NPC, WAYPOINT_NONE);

            if (*NPCInfo).homeWp == WAYPOINT_NONE || (*NPC).waypoint == WAYPOINT_NONE {
                //Heading for or at an invalid waypoint, get out of this bState
                if (*NPCInfo).tempBehavior == BS_SEARCH {
                    //if tempbehavior, set tempbehavior to default
                    (*NPCInfo).tempBehavior = BS_DEFAULT;
                } else {
                    //if bState, change to stand guard
                    (*NPCInfo).behaviorState = BS_STAND_GUARD;
                    NPC_BSRunAndShoot();
                }
                return;
            }

            if (*NPC).waypoint == (*NPCInfo).homeWp {
                //Just Reached our homeWp, if this is the first time, run your lostenemyscript
                if (*NPCInfo).aiFlags & NPCAI_ENROUTE_TO_HOMEWP != 0 {
                    (*NPCInfo).aiFlags &= !NPCAI_ENROUTE_TO_HOMEWP;
                    G_ActivateBehavior(NPC, BSET_LOSTENEMY);
                }
            }

            //Com_Printf("Got there.\n");
            //Com_Printf("Looking...");
            if Q_irand(0, 1) == 0 {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_GUARD_LOOKAROUND1,
                    SETANIM_FLAG_NORMAL,
                );
            } else {
                NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_GUARD_IDLE1, SETANIM_FLAG_NORMAL);
            }
            (*NPCInfo).investigateDebounceTime = (*addr_of!(level)).time + Q_irand(3000, 10000);
        } else {
            NPC_MoveToGoal(QTRUE);
        }
    } else {
        //We're there
        if (*NPCInfo).investigateDebounceTime > (*addr_of!(level)).time {
            //Still waiting around for a bit
            //Turn angles every now and then to look around
            if (*(*NPCInfo).tempGoal).waypoint != WAYPOINT_NONE {
                if Q_irand(0, 30) == 0 {
                    let numEdges: c_int =
                        trap::Nav_GetNodeNumEdges((*(*NPCInfo).tempGoal).waypoint);

                    if numEdges != WAYPOINT_NONE {
                        let branchNum: c_int = Q_irand(0, numEdges - 1);

                        let mut branchPos: vec3_t = [0.0; 3];
                        let mut lookDir: vec3_t = [0.0; 3];

                        let nextWp: c_int =
                            trap::Nav_GetNodeEdge((*(*NPCInfo).tempGoal).waypoint, branchNum);
                        trap::Nav_GetNodePosition(nextWp, &mut branchPos);

                        VectorSubtract(
                            &branchPos,
                            &(*(*NPCInfo).tempGoal).r.currentOrigin,
                            &mut lookDir,
                        );
                        (*NPCInfo).desiredYaw =
                            AngleNormalize360(vectoyaw(&lookDir) + flrand(-45.0, 45.0));
                    }

                    //pick an angle +-45 degrees off of the dir of a random branch
                    //from NPCInfo->tempGoal->waypoint
                    //int branch = Q_irand( 0, (waypoints[NPCInfo->tempGoal->waypoint].numNeighbors - 1) );
                    //int	nextWp = waypoints[NPCInfo->tempGoal->waypoint].nextWaypoint[branch][NPC->client->moveType];
                    //vec3_t	lookDir;

                    //VectorSubtract( waypoints[nextWp].origin, NPCInfo->tempGoal->r.currentOrigin, lookDir );
                    //Look in that direction +- 45 degrees
                    //NPCInfo->desiredYaw = AngleNormalize360( vectoyaw( lookDir ) + Q_flrand( -45, 45 ) );
                }
            }
            //Com_Printf(".");
        } else {
            //Just finished waiting
            (*NPC).waypoint = NAV_FindClosestWaypointForEnt(NPC, WAYPOINT_NONE);

            if (*NPC).waypoint == (*NPCInfo).homeWp {
                let numEdges: c_int = trap::Nav_GetNodeNumEdges((*(*NPCInfo).tempGoal).waypoint);

                if numEdges != WAYPOINT_NONE {
                    let branchNum: c_int = Q_irand(0, numEdges - 1);

                    let nextWp: c_int = trap::Nav_GetNodeEdge((*NPCInfo).homeWp, branchNum);
                    trap::Nav_GetNodePosition(nextWp, &mut (*(*NPCInfo).tempGoal).r.currentOrigin);
                    (*(*NPCInfo).tempGoal).waypoint = nextWp;
                }

                /*
                //Pick a random branch
                int branch = Q_irand( 0, (waypoints[NPCInfo->homeWp].numNeighbors - 1) );
                int	nextWp = waypoints[NPCInfo->homeWp].nextWaypoint[branch][NPC->client->moveType];

                VectorCopy( waypoints[nextWp].origin, NPCInfo->tempGoal->r.currentOrigin );
                NPCInfo->tempGoal->waypoint = nextWp;
                //Com_Printf("\nHeading for wp %d...\n", waypoints[NPCInfo->homeWp].nextWaypoint[branch][NPC->client->moveType]);
                */
            } else {
                //At a branch, so return home
                trap::Nav_GetNodePosition(
                    (*NPCInfo).homeWp,
                    &mut (*(*NPCInfo).tempGoal).r.currentOrigin,
                );
                (*(*NPCInfo).tempGoal).waypoint = (*NPCInfo).homeWp;
                /*
                VectorCopy( waypoints[NPCInfo->homeWp].origin, NPCInfo->tempGoal->r.currentOrigin );
                NPCInfo->tempGoal->waypoint = NPCInfo->homeWp;
                //Com_Printf("\nHeading for wp %d...\n", NPCInfo->homeWp);
                */
            }

            (*NPCInfo).investigateDebounceTime = 0;
            //Start moving toward our tempGoal
            (*NPCInfo).goalEntity = (*NPCInfo).tempGoal;
            NPC_MoveToGoal(QTRUE);
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/// `void NPC_BSEmplaced( void )` (NPC_behavior.c:1650).
///
/// Emplaced-gun handler: idle if hurt or enemy-less; otherwise check line-of-sight to the
/// enemy, face and (if a clear shot and not `SCF_DONT_FIRE`, and not two duelling jedi)
/// fire. Faithful 1:1; the commented-out `else if trap_InPVS` block and the ANH-duel note
/// carried. No oracle (`trap_Trace`/LOS + process-global state).
///
/// # Safety
/// `NPC`/`NPCInfo` set; `level` initialised.
pub unsafe fn NPC_BSEmplaced() {
    let mut enemyLOS: qboolean = QFALSE;
    let mut enemyCS: qboolean = QFALSE;
    let mut faceEnemy: qboolean = QFALSE;
    let mut shoot: qboolean = QFALSE;
    let mut impactPos: vec3_t = [0.0; 3];

    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > (*addr_of!(level)).time {
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(QTRUE);
    }

    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE {
        if Q_irand(0, 30) == 0 {
            (*NPCInfo).desiredYaw = (*NPC).s.angles[1] + Q_irand(-90, 90) as f32;
        }
        if Q_irand(0, 30) == 0 {
            (*NPCInfo).desiredPitch = Q_irand(-20, 20) as f32;
        }
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    if NPC_ClearLOS4((*NPC).enemy) != QFALSE {
        let hit: c_int;
        let hitEnt: *mut gentity_t;

        enemyLOS = QTRUE;

        hit = NPC_ShotEntity((*NPC).enemy, &mut impactPos);
        hitEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).offset(hit as isize);

        if hit == (*(*NPC).enemy).s.number || (!hitEnt.is_null() && (*hitEnt).takedamage != QFALSE)
        {
            //can hit enemy or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
            enemyCS = QTRUE;
            NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
            VectorCopy(
                &(*(*NPC).enemy).r.currentOrigin,
                &mut (*NPCInfo).enemyLastSeenLocation,
            );
        }
    }
    /*
        else if ( trap_InPVS( NPC->enemy->r.currentOrigin, NPC->r.currentOrigin ) )
        {
            NPCInfo->enemyLastSeenTime = level.time;
            faceEnemy = qtrue;
            NPC_AimAdjust( -1 );//adjust aim worse longer we cannot see enemy
        }
    */

    if enemyLOS != QFALSE {
        //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
        faceEnemy = QTRUE;
    }
    if enemyCS != QFALSE {
        shoot = QTRUE;
    }

    if faceEnemy != QFALSE {
        //face the enemy
        NPC_FaceEnemy(QTRUE);
    } else {
        //we want to face in the dir we're running
        NPC_UpdateAngles(QTRUE, QTRUE);
    }

    if (*NPCInfo).scriptFlags & SCF_DONT_FIRE != 0 {
        shoot = QFALSE;
    }

    if !(*NPC).enemy.is_null() && !(*(*NPC).enemy).enemy.is_null() {
        if (*(*NPC).enemy).s.weapon == WP_SABER && (*(*(*NPC).enemy).enemy).s.weapon == WP_SABER {
            //don't shoot at an enemy jedi who is fighting another jedi, for fear of injuring one or causing rogue blaster deflections (a la Obi Wan/Vader duel at end of ANH)
            shoot = QFALSE;
        }
    }
    if shoot != QFALSE {
        //try to shoot if it's time
        if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON == 0 {
            // we've already fired, no need to do it again here
            WeaponThink(QTRUE);
        }
    }
}

/// `void NPC_BSNoClip( void )` (NPC_behavior.c:1160).
///
/// No-clip movement: head straight toward the goal entity (if `UpdateGoal()` finds one),
/// converting the goal-relative direction into `ucmd` forward/right/up moves; otherwise
/// clear velocity. "Use in extreme circumstances only." No oracle — pure entity/global
/// state (`NPC`/`NPCInfo`/`ucmd`) with no return value to parity-check.
///
/// # Safety
/// `NPC`/`NPCInfo`/`ucmd` set; `UpdateGoal`/`NPC_UpdateAngles` ported.
pub unsafe fn NPC_BSNoClip() {
    if !UpdateGoal().is_null() {
        let mut dir: vec3_t = [0.0; 3];
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0; 3];
        let up: vec3_t = [0.0, 0.0, 1.0];
        let fDot: f32;
        let rDot: f32;
        let uDot: f32;

        VectorSubtract(
            &(*(*NPCInfo).goalEntity).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut dir,
        );

        vectoangles(&dir, &mut angles);
        (*NPCInfo).desiredYaw = angles[YAW];

        AngleVectors(
            &(*NPC).r.currentAngles,
            Some(&mut forward),
            Some(&mut right),
            None,
        );

        VectorNormalize(&mut dir);

        fDot = DotProduct(&forward, &dir) * 127.0;
        rDot = DotProduct(&right, &dir) * 127.0;
        uDot = DotProduct(&up, &dir) * 127.0;

        (*addr_of_mut!(ucmd)).forwardmove = (fDot as f64).floor() as i8;
        (*addr_of_mut!(ucmd)).rightmove = (rDot as f64).floor() as i8;
        (*addr_of_mut!(ucmd)).upmove = (uDot as f64).floor() as i8;
    } else {
        //Cut velocity?
        VectorClear(&mut (*(*NPC).client).ps.velocity);
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}
