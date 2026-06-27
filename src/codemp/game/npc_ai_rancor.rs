//! Slice of `NPC_AI_Rancor.c` — the Rancor melee/grab monster's behavior state.
//! Opened bottom-up at the leaf seam: the sound-precache and the one-shot roar
//! gate are genuinely portable today, while the idle / patrol / move / combat /
//! attack / swing / smash / bite / grab-drop chain. This file is now COMPLETE.
//!
//! Ported here so far: `Rancor_SetBolts` (NPC_AI_Rancor.c:19),
//! `NPC_Rancor_Precache` (:36), `Rancor_CheckRoar` (:66),
//! `Rancor_DropVictim` (:140), `Rancor_CheckDropVictim` (:790),
//! `Rancor_Crush` (:811), `NPC_Rancor_Pain` (:709), `Rancor_Swing` (:196),
//! `Rancor_Smash` (:308), `Rancor_Bite` (:369), `Rancor_Attack` (:431).
//!
//! `Rancor_Move` (:115) also lands here now that `NPC_MoveToGoal` is ported.
//! `Rancor_Idle` (:53) and `Rancor_Patrol` (:83) land now that `UpdateGoal`
//! (npc_goal.rs) and `NPC_CheckEnemyExt` (npc_utils.rs) are ported.
//! `NPC_BSRancor_Default` (:833) lands now that `NPC_CheckEnemy`/`ValidEnemy`
//! (npc_combat.rs) and `AddSightEvent` (npc_senses.rs) are ported.
//! `Rancor_SetBolts` (:19) lands now that `trap_G2API_AddBolt` is wrapped.

#![allow(non_snake_case)] // C function names (`Rancor_CheckRoar`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::anims::{BOTH_ATTACK1, BOTH_ATTACK2, BOTH_ATTACK3, BOTH_DEATH17,
    BOTH_DEATHBACKWARD2, BOTH_FALLDEATH1, BOTH_MELEE1, BOTH_MELEE2, BOTH_PAIN1, BOTH_PAIN2,
    BOTH_STAND1TO2, BOTH_SWIM_IDLE1};
use crate::codemp::game::b_public_h::SCF_LOOK_FOR_ENEMIES;
use crate::codemp::game::bg_public::{EF2_ALERTED, EF2_GENERIC_NPC_FLAG, EF2_HELD_BY_MONSTER,
    EF2_USE_ALT_ANIM, EF_NODRAW, EV_DEATH1,
    EV_DEATH3, EV_JUMP, G2_MODELPART_HEAD, G2_MODELPART_RLEG, G2_MODELPART_WAIST, HANDEXTEND_NONE,
    MOD_CRUSH, MOD_MELEE, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE, SETANIM_TORSO};
use crate::codemp::game::g_client::SetClientViewAngle;
use crate::codemp::game::g_combat::{G_Damage, G_Dismember, G_Knockdown, TossClientItems};
use crate::codemp::game::g_local::{gentity_t, AEL_DANGER, AEL_DANGER_GREAT, DAMAGE_NO_ARMOR,
    DAMAGE_NO_HIT_LOC, DAMAGE_NO_KNOCKBACK, DAMAGE_NO_PROTECTION, FL_NOTARGET};
use crate::codemp::game::g_main::{g_entities, level};
use crate::codemp::game::g_public_h::Q3_INFINITE;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Done2, TIMER_Exists, TIMER_Remove, TIMER_Set};
use crate::codemp::game::g_utils::{G_AddEvent, G_ScreenShake, G_SetAngles, G_Sound, G_SoundIndex,
    G_Throw};
use crate::codemp::game::npc::{ucmd, NPC_SetAnim, NPC, NPCInfo};
use crate::codemp::game::npc_combat::{G_SetEnemy, NPC_CheckEnemy, ValidEnemy};
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_senses::{AddSightEvent, AddSoundEvent, InFOV3};
use crate::codemp::game::npc_utils::{G_GetBoltPosition, NPC_CheckEnemyExt, NPC_ClearLOS4,
    NPC_FaceEnemy, NPC_GetEntsNearBolt, NPC_UpdateAngles};
use crate::codemp::game::q_math::{flrand, vec3_origin, AngleVectors, Distance, DistanceSquared,
    VectorCopy, VectorScale, VectorSet};
use crate::codemp::game::q_shared::{crandom, random};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{trace_t, vec3_t, BUTTON_WALKING, CHAN_AUTO, ENTITYNUM_NONE,
    ENTITYNUM_WORLD, PITCH, ROLL, YAW};
use crate::codemp::game::teams_h::{CLASS_ATST, CLASS_GALAKMECH, CLASS_GONK, CLASS_INTERROGATOR,
    CLASS_MARK1, CLASS_MARK2, CLASS_MOUSE, CLASS_PROBE, CLASS_R2D2, CLASS_R5D2, CLASS_RANCOR,
    CLASS_REMOTE, CLASS_SEEKER, CLASS_SENTRY, CLASS_VEHICLE};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// These define the working combat range for these suckers
#[allow(dead_code)]
const MIN_DISTANCE: c_int = 128;
#[allow(dead_code)]
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const MAX_DISTANCE: c_int = 1024;
#[allow(dead_code)]
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

#[allow(dead_code)]
const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

/// `void Rancor_SetBolts( gentity_t *self )` (NPC_AI_Rancor.c:19) — cache the Ghoul2
/// bolt indices (hands / head-eyes / jaw) into the entity's `renderInfo`.
///
/// # Safety
/// `self_` may be null; `self_->client` is null-checked. `self_->ghoul2` must be a
/// valid Ghoul2 instance for any non-null client.
pub unsafe fn Rancor_SetBolts(self_: *mut gentity_t) {
    if !self_.is_null() && !(*self_).client.is_null() {
        let ri = &mut (*(*self_).client).renderInfo;
        ri.handRBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_hand");
        ri.handLBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_hand");
        ri.headBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*head_eyes");
        ri.torsoBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "jaw_bone");
    }
}

/*
-------------------------
NPC_Rancor_Precache
-------------------------
*/
pub unsafe fn NPC_Rancor_Precache() {
    let mut i: c_int = 1;
    while i < 3 {
        G_SoundIndex(&format!("sound/chars/rancor/snort_{}.wav", i));
        i += 1;
    }
    G_SoundIndex("sound/chars/rancor/swipehit.wav");
    G_SoundIndex("sound/chars/rancor/chomp.wav");
}

/*
-------------------------
Rancor_Idle
-------------------------
*/
pub unsafe fn Rancor_Idle() {
    (*NPCInfo).localState = LSTATE_CLEAR;

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    }
}

pub unsafe fn Rancor_CheckRoar(self_: *mut gentity_t) -> c_int {
    if (*self_).wait == 0.0 {
        // haven't ever gotten mad yet
        (*self_).wait = 1.0; // do this only once
        (*(*self_).client).ps.eFlags2 |= EF2_ALERTED;
        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            BOTH_STAND1TO2,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        TIMER_Set(self_, c"rageTime".as_ptr(), (*(*self_).client).ps.legsTimer);
        return QTRUE;
    }
    QFALSE
}

/*
-------------------------
Rancor_Patrol
-------------------------
*/
pub unsafe fn Rancor_Patrol() {
    (*NPCInfo).localState = LSTATE_CLEAR;

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    } else {
        if TIMER_Done(NPC, c"patrolTime".as_ptr()) != QFALSE {
            TIMER_Set(NPC, c"patrolTime".as_ptr(), (crandom() * 5000.0 + 5000.0) as c_int);
        }
    }

    if NPC_CheckEnemyExt(QTRUE) == QFALSE {
        Rancor_Idle();
        return;
    }
    Rancor_CheckRoar(NPC);
    TIMER_Set(NPC, c"lookForNewEnemy".as_ptr(), Q_irand(5000, 15000));
}

/*
-------------------------
Rancor_Move
-------------------------
*/
pub unsafe fn Rancor_Move(_visible: qboolean) {
    if (*NPCInfo).localState != LSTATE_WAITING {
        (*NPCInfo).goalEntity = (*NPC).enemy;
        if NPC_MoveToGoal(QTRUE) == QFALSE {
            (*NPCInfo).consecutiveBlockedMoves += 1;
        } else {
            (*NPCInfo).consecutiveBlockedMoves = 0;
        }
        (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range
    }
}

pub unsafe fn Rancor_Combat() {
    if (*NPC).count != 0 {
        //holding my enemy
        if TIMER_Done2(NPC, c"takingPain".as_ptr(), QTRUE) != QFALSE {
            (*NPCInfo).localState = LSTATE_CLEAR;
        } else {
            Rancor_Attack(0.0, QFALSE);
        }
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }
    // If we cannot see our target or we have somewhere to go, then do that
    if NPC_ClearLOS4((*NPC).enemy) == QFALSE
    //|| UpdateGoal( ))
    {
        (*NPCInfo).combatMove = QTRUE;
        (*NPCInfo).goalEntity = (*NPC).enemy;
        (*NPCInfo).goalRadius = MIN_DISTANCE; //MAX_DISTANCE;	// just get us within combat range

        if NPC_MoveToGoal(QTRUE) == QFALSE {
            //couldn't go after him?  Look for a new one
            TIMER_Set(NPC, c"lookForNewEnemy".as_ptr(), 0);
            (*NPCInfo).consecutiveBlockedMoves += 1;
        } else {
            (*NPCInfo).consecutiveBlockedMoves = 0;
        }
        return;
    }

    // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
    NPC_FaceEnemy(QTRUE);

    {
        let distance: f32;
        let mut advance: qboolean;
        let mut doCharge: qboolean;

        distance = Distance(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);
        advance = if distance > ((*NPC).r.maxs[0] + MIN_DISTANCE as f32) {
            QTRUE
        } else {
            QFALSE
        };
        doCharge = QFALSE;

        if advance != QFALSE {
            //have to get closer
            let mut yawOnlyAngles: vec3_t = [0.0; 3];
            VectorSet(
                &mut yawOnlyAngles,
                0.0,
                (*NPC).r.currentAngles[YAW],
                0.0,
            );
            if (*(*NPC).enemy).health > 0
                && (distance - 250.0).abs() <= 80.0
                && InFOV3(
                    &(*(*NPC).enemy).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &yawOnlyAngles,
                    30,
                    30,
                ) != QFALSE
            {
                if Q_irand(0, 9) == 0 {
                    //go for the charge
                    doCharge = QTRUE;
                    advance = QFALSE;
                }
            }
        }

        if (advance != QFALSE /*|| NPCInfo->localState == LSTATE_WAITING*/)
            && TIMER_Done(NPC, c"attacking".as_ptr()) != QFALSE
        // waiting monsters can't attack
        {
            if TIMER_Done2(NPC, c"takingPain".as_ptr(), QTRUE) != QFALSE {
                (*NPCInfo).localState = LSTATE_CLEAR;
            } else {
                Rancor_Move(1);
            }
        } else {
            Rancor_Attack(distance, doCharge);
        }
    }
}

pub unsafe fn Rancor_DropVictim(self_: *mut gentity_t) {
    //FIXME: if Rancor dies, it should drop its victim.
    //FIXME: if Rancor is removed, it must remove its victim.
    if !(*self_).activator.is_null() {
        let activator = (*self_).activator;
        if !(*activator).client.is_null() {
            (*(*activator).client).ps.eFlags2 &= !EF2_HELD_BY_MONSTER;
            (*(*activator).client).ps.hasLookTarget = QFALSE;
            (*(*activator).client).ps.lookTarget = ENTITYNUM_NONE;
            (*(*activator).client).ps.viewangles[ROLL] = 0.0;
            SetClientViewAngle(activator, &(*(*activator).client).ps.viewangles.clone());
            (*activator).r.currentAngles[PITCH] = 0.0;
            (*activator).r.currentAngles[ROLL] = 0.0;
            G_SetAngles(activator, &(*activator).r.currentAngles.clone());
        }
        if (*activator).health <= 0 {
            //if ( self->activator->s.number )
            {
                //never free player
                if (*self_).count == 1 {
                    //in my hand, just drop them
                    if !(*activator).client.is_null() {
                        (*(*activator).client).ps.legsTimer = 0;
                        (*(*activator).client).ps.torsoTimer = 0;
                        //FIXME: ragdoll?
                    }
                } else {
                    if !(*activator).client.is_null() {
                        (*(*activator).client).ps.eFlags |= EF_NODRAW; //so his corpse doesn't drop out of me...
                    }
                    //G_FreeEntity( self->activator );
                }
            }
        } else {
            if !(*activator).NPC.is_null() {
                //start thinking again
                (*(*activator).NPC).nextBStateThink = (*addr_of!(level)).time;
            }
            //clear their anim and let them fall
            (*(*activator).client).ps.legsTimer = 0;
            (*(*activator).client).ps.torsoTimer = 0;
        }
        if (*self_).enemy == (*self_).activator {
            (*self_).enemy = null_mut();
        }
        (*self_).activator = null_mut();
    }
    (*self_).count = 0; //drop him
}

pub unsafe fn Rancor_CheckDropVictim() {
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let trace: trace_t;

    let activator = (*NPC).activator;
    VectorSet(
        &mut mins,
        (*activator).r.mins[0] - 1.0,
        (*activator).r.mins[1] - 1.0,
        0.0,
    );
    VectorSet(
        &mut maxs,
        (*activator).r.maxs[0] + 1.0,
        (*activator).r.maxs[1] + 1.0,
        1.0,
    );
    VectorSet(
        &mut start,
        (*activator).r.currentOrigin[0],
        (*activator).r.currentOrigin[1],
        (*activator).r.absmin[2],
    );
    VectorSet(
        &mut end,
        (*activator).r.currentOrigin[0],
        (*activator).r.currentOrigin[1],
        (*activator).r.absmax[2] - 1.0,
    );

    trace = trap::Trace(
        &start,
        &mins,
        &maxs,
        &end,
        (*activator).s.number,
        (*activator).clipmask,
    );
    if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction >= 1.0 {
        Rancor_DropVictim(NPC);
    }
}

//if he's stepping on things then crush them -rww
pub unsafe fn Rancor_Crush() {
    let crush: *mut gentity_t;

    if NPC.is_null()
        || (*NPC).client.is_null()
        || (*(*NPC).client).ps.groundEntityNum >= ENTITYNUM_WORLD
    {
        //nothing to crush
        return;
    }

    crush = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add((*(*NPC).client).ps.groundEntityNum as usize);
    if (*crush).inuse != 0 && !(*crush).client.is_null() && (*crush).localAnimIndex == 0 {
        //a humanoid, smash them good.
        G_Damage(
            crush,
            NPC,
            NPC,
            null_mut(),
            addr_of_mut!((*NPC).r.currentOrigin),
            200,
            0,
            MOD_CRUSH,
        );
    }
}

pub unsafe extern "C" fn NPC_Rancor_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    let mut hitByRancor = QFALSE;
    if !attacker.is_null()
        && !(*attacker).client.is_null()
        && (*(*attacker).client).NPC_class == CLASS_RANCOR
    {
        hitByRancor = QTRUE;
    }
    if !attacker.is_null()
        && (*attacker).inuse != 0
        && attacker != (*self_).enemy
        && (*attacker).flags & FL_NOTARGET == 0
    {
        if (*self_).count == 0 {
            if ((*attacker).s.number == 0 && Q_irand(0, 3) == 0)
                || (*self_).enemy.is_null()
                || (*(*self_).enemy).health == 0
                || (!(*(*self_).enemy).client.is_null()
                    && (*(*(*self_).enemy).client).NPC_class == CLASS_RANCOR)
                || (!(*self_).NPC.is_null()
                    && (*(*self_).NPC).consecutiveBlockedMoves >= 10
                    && DistanceSquared(
                        &(*attacker).r.currentOrigin,
                        &(*self_).r.currentOrigin,
                    ) < DistanceSquared(
                        &(*(*self_).enemy).r.currentOrigin,
                        &(*self_).r.currentOrigin,
                    ))
            {
                //if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
                //FIXME: if can't nav to my enemy, take this guy if I can nav to him
                G_SetEnemy(self_, attacker);
                TIMER_Set(self_, c"lookForNewEnemy".as_ptr(), Q_irand(5000, 15000));
                if hitByRancor != QFALSE {
                    //stay mad at this Rancor for 2-5 secs before looking for attacker enemies
                    TIMER_Set(self_, c"rancorInfight".as_ptr(), Q_irand(2000, 5000));
                }
            }
        }
    }
    if (hitByRancor != QFALSE
        || ((*self_).count == 1 && !(*self_).activator.is_null() && Q_irand(0, 4) == 0)
        || Q_irand(0, 200) < damage)
        //hit by rancor, hit while holding live victim, or took a lot of damage
        && (*(*self_).client).ps.legsAnim != BOTH_STAND1TO2
        && TIMER_Done(self_, c"takingPain".as_ptr()) != QFALSE
    {
        if Rancor_CheckRoar(self_) == QFALSE {
            if (*(*self_).client).ps.legsAnim != BOTH_MELEE1
                && (*(*self_).client).ps.legsAnim != BOTH_MELEE2
                && (*(*self_).client).ps.legsAnim != BOTH_ATTACK2
            {
                //cant interrupt one of the big attack anims
                /*
                if ( self->count != 1
                    || attacker == self->activator
                    || (self->client->ps.legsAnim != BOTH_ATTACK1&&self->client->ps.legsAnim != BOTH_ATTACK3) )
                */
                {
                    //if going to bite our victim, only victim can interrupt that anim
                    if (*self_).health > 100 || hitByRancor != QFALSE {
                        TIMER_Remove(self_, c"attacking".as_ptr());

                        VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s.angles);

                        if (*self_).count == 1 {
                            NPC_SetAnim(
                                self_,
                                SETANIM_BOTH,
                                BOTH_PAIN2,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else {
                            NPC_SetAnim(
                                self_,
                                SETANIM_BOTH,
                                BOTH_PAIN1,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        TIMER_Set(
                            self_,
                            c"takingPain".as_ptr(),
                            (*(*self_).client).ps.legsTimer + Q_irand(0, 500),
                        );

                        if !(*self_).NPC.is_null() {
                            (*(*self_).NPC).localState = LSTATE_WAITING;
                        }
                    }
                }
            }
        }
        //let go
        /*
        if ( !Q_irand( 0, 3 ) && self->count == 1 )
        {
            Rancor_DropVictim( self );
        }
        */
    }
}

pub unsafe fn Rancor_Swing(tryGrab: c_int) {
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let numEnts: c_int;
    let radius: f32 = 88.0;
    let radiusSquared: f32 = radius * radius;
    let mut i: c_int;
    let mut boltOrg: vec3_t = [0.0; 3];

    numEnts = NPC_GetEntsNearBolt(
        radiusEntNums.as_mut_ptr(),
        radius,
        (*(*NPC).client).renderInfo.handRBolt,
        addr_of_mut!(boltOrg),
    );

    i = 0;
    while i < numEnts {
        let radiusEnt: *mut gentity_t = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(radiusEntNums[i as usize] as usize);
        if (*radiusEnt).inuse == 0 {
            i += 1;
            continue;
        }

        if radiusEnt == NPC {
            //Skip the rancor ent
            i += 1;
            continue;
        }

        if (*radiusEnt).client.is_null() {
            //must be a client
            i += 1;
            continue;
        }

        if (*(*radiusEnt).client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            //can't be one already being held
            i += 1;
            continue;
        }

        if DistanceSquared(&(*radiusEnt).r.currentOrigin, &boltOrg) <= radiusSquared {
            if tryGrab != QFALSE
                && (*NPC).count != 1 //don't have one in hand or in mouth already - FIXME: allow one in hand and any number in mouth!
                && (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                && (*(*radiusEnt).client).NPC_class != CLASS_GALAKMECH
                && (*(*radiusEnt).client).NPC_class != CLASS_ATST
                && (*(*radiusEnt).client).NPC_class != CLASS_GONK
                && (*(*radiusEnt).client).NPC_class != CLASS_R2D2
                && (*(*radiusEnt).client).NPC_class != CLASS_R5D2
                && (*(*radiusEnt).client).NPC_class != CLASS_MARK1
                && (*(*radiusEnt).client).NPC_class != CLASS_MARK2
                && (*(*radiusEnt).client).NPC_class != CLASS_MOUSE
                && (*(*radiusEnt).client).NPC_class != CLASS_PROBE
                && (*(*radiusEnt).client).NPC_class != CLASS_SEEKER
                && (*(*radiusEnt).client).NPC_class != CLASS_REMOTE
                && (*(*radiusEnt).client).NPC_class != CLASS_SENTRY
                && (*(*radiusEnt).client).NPC_class != CLASS_INTERROGATOR
                && (*(*radiusEnt).client).NPC_class != CLASS_VEHICLE
            {
                //grab
                if (*NPC).count == 2 {
                    //have one in my mouth, remove him
                    TIMER_Remove(NPC, c"clearGrabbed".as_ptr());
                    Rancor_DropVictim(NPC);
                }
                (*NPC).enemy = radiusEnt; //make him my new best friend
                (*(*radiusEnt).client).ps.eFlags2 |= EF2_HELD_BY_MONSTER;
                //FIXME: this makes it so that the victim can't hit us with shots!  Just use activator or something
                (*(*radiusEnt).client).ps.hasLookTarget = QTRUE;
                (*(*radiusEnt).client).ps.lookTarget = (*NPC).s.number;
                (*NPC).activator = radiusEnt; //remember him
                (*NPC).count = 1; //in my hand
                                  //wait to attack
                TIMER_Set(
                    NPC,
                    c"attacking".as_ptr(),
                    (*(*NPC).client).ps.legsTimer + Q_irand(500, 2500),
                );
                if (*radiusEnt).health > 0 && (*radiusEnt).pain.is_some() {
                    //do pain on enemy
                    ((*radiusEnt).pain.unwrap())(radiusEnt, NPC, 100);
                    //GEntity_PainFunc( radiusEnt, NPC, NPC, radiusEnt->r.currentOrigin, 0, MOD_CRUSH );
                } else if !(*radiusEnt).client.is_null() {
                    (*(*radiusEnt).client).ps.forceHandExtend = HANDEXTEND_NONE;
                    (*(*radiusEnt).client).ps.forceHandExtendTime = 0;
                    NPC_SetAnim(
                        radiusEnt,
                        SETANIM_BOTH,
                        BOTH_SWIM_IDLE1,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }
            } else {
                //smack
                let mut pushDir: vec3_t = [0.0; 3];
                let mut angs: vec3_t = [0.0; 3];

                G_Sound(
                    radiusEnt,
                    CHAN_AUTO,
                    G_SoundIndex("sound/chars/rancor/swipehit.wav"),
                );
                //actually push the enemy
                /*
                //VectorSubtract( radiusEnt->r.currentOrigin, boltOrg, pushDir );
                VectorSubtract( radiusEnt->r.currentOrigin, NPC->r.currentOrigin, pushDir );
                pushDir[2] = Q_flrand( 100, 200 );
                VectorNormalize( pushDir );
                */
                VectorCopy(&(*(*NPC).client).ps.viewangles, &mut angs);
                angs[YAW] += flrand(25.0, 50.0);
                angs[PITCH] = flrand(-25.0, -15.0);
                AngleVectors(&angs, Some(&mut pushDir), None, None);
                if (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                    && (*(*radiusEnt).client).NPC_class != CLASS_ATST
                {
                    G_Damage(
                        radiusEnt,
                        NPC,
                        NPC,
                        &mut vec3_origin.clone(),
                        &mut (*radiusEnt).r.currentOrigin.clone(),
                        Q_irand(25, 40),
                        DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                    G_Throw(radiusEnt, &pushDir, 250.0);
                    if (*radiusEnt).health > 0 {
                        //do pain on enemy
                        G_Knockdown(radiusEnt); //, NPC, pushDir, 100, qtrue );
                    }
                }
            }
        }
        i += 1;
    }
}

pub unsafe fn Rancor_Smash() {
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let numEnts: c_int;
    let radius: f32 = 128.0;
    let halfRadSquared: f32 = (radius / 2.0) * (radius / 2.0);
    let radiusSquared: f32 = radius * radius;
    let mut distSq: f32;
    let mut i: c_int;
    let mut boltOrg: vec3_t = [0.0; 3];

    AddSoundEvent(
        NPC,
        &(*NPC).r.currentOrigin,
        512.0,
        AEL_DANGER,
        QFALSE,
    ); //, qtrue );

    numEnts = NPC_GetEntsNearBolt(
        radiusEntNums.as_mut_ptr(),
        radius,
        (*(*NPC).client).renderInfo.handLBolt,
        addr_of_mut!(boltOrg),
    );

    i = 0;
    while i < numEnts {
        let radiusEnt: *mut gentity_t = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(radiusEntNums[i as usize] as usize);
        if (*radiusEnt).inuse == 0 {
            i += 1;
            continue;
        }

        if radiusEnt == NPC {
            //Skip the rancor ent
            i += 1;
            continue;
        }

        if (*radiusEnt).client.is_null() {
            //must be a client
            i += 1;
            continue;
        }

        if (*(*radiusEnt).client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            //can't be one being held
            i += 1;
            continue;
        }

        distSq = DistanceSquared(&(*radiusEnt).r.currentOrigin, &boltOrg);
        if distSq <= radiusSquared {
            G_Sound(
                radiusEnt,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/rancor/swipehit.wav"),
            );
            if distSq < halfRadSquared {
                //close enough to do damage, too
                G_Damage(
                    radiusEnt,
                    NPC,
                    NPC,
                    &mut vec3_origin.clone(),
                    &mut (*radiusEnt).r.currentOrigin.clone(),
                    Q_irand(10, 25),
                    DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                    MOD_MELEE,
                );
            }
            if (*radiusEnt).health > 0
                && !(*radiusEnt).client.is_null()
                && (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                && (*(*radiusEnt).client).NPC_class != CLASS_ATST
            {
                if distSq < halfRadSquared
                    || (*(*radiusEnt).client).ps.groundEntityNum != ENTITYNUM_NONE
                {
                    //within range of my fist or withing ground-shaking range and not in the air
                    G_Knockdown(radiusEnt); //, NPC, vec3_origin, 100, qtrue );
                }
            }
        }
        i += 1;
    }
}

pub unsafe fn Rancor_Bite() {
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let numEnts: c_int;
    let radius: f32 = 100.0;
    let radiusSquared: f32 = radius * radius;
    let mut i: c_int;
    let mut boltOrg: vec3_t = [0.0; 3];

    numEnts = NPC_GetEntsNearBolt(
        radiusEntNums.as_mut_ptr(),
        radius,
        (*(*NPC).client).renderInfo.crotchBolt,
        addr_of_mut!(boltOrg),
    ); //was gutBolt?

    i = 0;
    while i < numEnts {
        let radiusEnt: *mut gentity_t = core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>().add(radiusEntNums[i as usize] as usize);
        if (*radiusEnt).inuse == 0 {
            i += 1;
            continue;
        }

        if radiusEnt == NPC {
            //Skip the rancor ent
            i += 1;
            continue;
        }

        if (*radiusEnt).client.is_null() {
            //must be a client
            i += 1;
            continue;
        }

        if (*(*radiusEnt).client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            //can't be one already being held
            i += 1;
            continue;
        }

        if DistanceSquared(&(*radiusEnt).r.currentOrigin, &boltOrg) <= radiusSquared {
            G_Damage(
                radiusEnt,
                NPC,
                NPC,
                &mut vec3_origin.clone(),
                &mut (*radiusEnt).r.currentOrigin.clone(),
                Q_irand(15, 30),
                DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                MOD_MELEE,
            );
            if (*radiusEnt).health <= 0 && !(*radiusEnt).client.is_null() {
                //killed them, chance of dismembering
                if Q_irand(0, 1) == 0 {
                    //bite something off
                    let hitLoc = Q_irand(G2_MODELPART_HEAD, G2_MODELPART_RLEG);
                    if hitLoc == G2_MODELPART_HEAD {
                        NPC_SetAnim(
                            radiusEnt,
                            SETANIM_BOTH,
                            BOTH_DEATH17,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    } else if hitLoc == G2_MODELPART_WAIST {
                        NPC_SetAnim(
                            radiusEnt,
                            SETANIM_BOTH,
                            BOTH_DEATHBACKWARD2,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }
                    //radiusEnt->client->dismembered = qfalse;
                    //FIXME: the limb should just disappear, cuz I ate it
                    G_Dismember(
                        radiusEnt,
                        NPC,
                        &(*radiusEnt).r.currentOrigin,
                        hitLoc,
                        90.0,
                        0.0,
                        (*(*radiusEnt).client).ps.torsoAnim,
                        QTRUE,
                    );
                    //G_DoDismemberment( radiusEnt, radiusEnt->r.currentOrigin, MOD_SABER, 1000, hitLoc, qtrue );
                }
            }
            G_Sound(
                radiusEnt,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/rancor/chomp.wav"),
            );
        }
        i += 1;
    }
}

pub unsafe fn Rancor_Attack(distance: f32, doCharge: c_int) {
    if TIMER_Exists(NPC, c"attacking".as_ptr()) == QFALSE {
        if (*NPC).count == 2 && !(*NPC).activator.is_null() {
        } else if (*NPC).count == 1 && !(*NPC).activator.is_null() {
            //holding enemy
            if (*(*NPC).activator).health > 0 && Q_irand(0, 1) != 0 {
                //quick bite
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(NPC, c"attack_dmg".as_ptr(), 450);
            } else {
                //full eat
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK3,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                TIMER_Set(NPC, c"attack_dmg".as_ptr(), 900);
                //Make victim scream in fright
                if (*(*NPC).activator).health > 0 && !(*(*NPC).activator).client.is_null() {
                    G_AddEvent((*NPC).activator, Q_irand(EV_DEATH1, EV_DEATH3), 0);
                    NPC_SetAnim(
                        (*NPC).activator,
                        SETANIM_TORSO,
                        BOTH_FALLDEATH1,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    if !(*(*NPC).activator).NPC.is_null() {
                        //no more thinking for you
                        TossClientItems(NPC);
                        (*(*(*NPC).activator).NPC).nextBStateThink = Q3_INFINITE;
                    }
                }
            }
        } else if (*(*NPC).enemy).health > 0 && doCharge != QFALSE {
            //charge
            let mut fwd: vec3_t = [0.0; 3];
            let mut yawAng: vec3_t = [0.0; 3];
            VectorSet(&mut yawAng, 0.0, (*(*NPC).client).ps.viewangles[YAW], 0.0);
            AngleVectors(&yawAng, Some(&mut fwd), None, None);
            VectorScale(&fwd, distance * 1.5, &mut (*(*NPC).client).ps.velocity);
            (*(*NPC).client).ps.velocity[2] = 150.0;
            (*(*NPC).client).ps.groundEntityNum = ENTITYNUM_NONE;

            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_MELEE2,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(NPC, c"attack_dmg".as_ptr(), 1250);
        } else if Q_irand(0, 1) == 0 {
            //smash
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_MELEE1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(NPC, c"attack_dmg".as_ptr(), 1000);
        } else {
            //try to grab
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_ATTACK2,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(NPC, c"attack_dmg".as_ptr(), 1000);
        }

        TIMER_Set(
            NPC,
            c"attacking".as_ptr(),
            (*(*NPC).client).ps.legsTimer + (random() * 200.0) as c_int,
        );
    }

    // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

    if TIMER_Done2(NPC, c"attack_dmg".as_ptr(), QTRUE) != QFALSE {
        let mut shakePos: vec3_t = [0.0; 3];
        match (*(*NPC).client).ps.legsAnim {
            x if x == BOTH_MELEE1 => {
                Rancor_Smash();
                G_GetBoltPosition(
                    NPC,
                    (*(*NPC).client).renderInfo.handLBolt,
                    addr_of_mut!(shakePos),
                    0,
                );
                G_ScreenShake(&shakePos, null_mut(), 4.0, 1000, QFALSE);
                //CGCam_Shake( 1.0f*playerDist/128.0f, 1000 );
            }
            x if x == BOTH_MELEE2 => {
                Rancor_Bite();
                TIMER_Set(NPC, c"attack_dmg2".as_ptr(), 450);
            }
            x if x == BOTH_ATTACK1 => {
                if (*NPC).count == 1 && !(*NPC).activator.is_null() {
                    G_Damage(
                        (*NPC).activator,
                        NPC,
                        NPC,
                        &mut vec3_origin.clone(),
                        &mut (*(*NPC).activator).r.currentOrigin.clone(),
                        Q_irand(25, 40),
                        DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE,
                    );
                    if (*(*NPC).activator).health <= 0 {
                        //killed him
                        //make it look like we bit his head off
                        //NPC->activator->client->dismembered = qfalse;
                        G_Dismember(
                            (*NPC).activator,
                            NPC,
                            &(*(*NPC).activator).r.currentOrigin,
                            G2_MODELPART_HEAD,
                            90.0,
                            0.0,
                            (*(*(*NPC).activator).client).ps.torsoAnim,
                            QTRUE,
                        );
                        //G_DoDismemberment( NPC->activator, NPC->activator->r.currentOrigin, MOD_SABER, 1000, HL_HEAD, qtrue );
                        (*(*(*NPC).activator).client).ps.forceHandExtend = HANDEXTEND_NONE;
                        (*(*(*NPC).activator).client).ps.forceHandExtendTime = 0;
                        NPC_SetAnim(
                            (*NPC).activator,
                            SETANIM_BOTH,
                            BOTH_SWIM_IDLE1,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }
                    G_Sound(
                        (*NPC).activator,
                        CHAN_AUTO,
                        G_SoundIndex("sound/chars/rancor/chomp.wav"),
                    );
                }
            }
            x if x == BOTH_ATTACK2 => {
                //try to grab
                Rancor_Swing(QTRUE);
            }
            x if x == BOTH_ATTACK3 => {
                if (*NPC).count == 1 && !(*NPC).activator.is_null() {
                    //cut in half
                    if !(*(*NPC).activator).client.is_null() {
                        //NPC->activator->client->dismembered = qfalse;
                        G_Dismember(
                            (*NPC).activator,
                            NPC,
                            &(*(*NPC).activator).r.currentOrigin,
                            G2_MODELPART_WAIST,
                            90.0,
                            0.0,
                            (*(*(*NPC).activator).client).ps.torsoAnim,
                            QTRUE,
                        );
                        //G_DoDismemberment( NPC->activator, NPC->enemy->r.currentOrigin, MOD_SABER, 1000, HL_WAIST, qtrue );
                    }
                    //KILL
                    G_Damage(
                        (*NPC).activator,
                        NPC,
                        NPC,
                        &mut vec3_origin.clone(),
                        &mut (*(*NPC).activator).r.currentOrigin.clone(),
                        (*(*NPC).enemy).health + 10,
                        DAMAGE_NO_PROTECTION
                            | DAMAGE_NO_ARMOR
                            | DAMAGE_NO_KNOCKBACK
                            | DAMAGE_NO_HIT_LOC,
                        MOD_MELEE,
                    ); //, HL_NONE );//
                    if !(*(*NPC).activator).client.is_null() {
                        (*(*(*NPC).activator).client).ps.forceHandExtend = HANDEXTEND_NONE;
                        (*(*(*NPC).activator).client).ps.forceHandExtendTime = 0;
                        NPC_SetAnim(
                            (*NPC).activator,
                            SETANIM_BOTH,
                            BOTH_SWIM_IDLE1,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }
                    TIMER_Set(NPC, c"attack_dmg2".as_ptr(), 1350);
                    G_Sound(
                        (*NPC).activator,
                        CHAN_AUTO,
                        G_SoundIndex("sound/chars/rancor/swipehit.wav"),
                    );
                    G_AddEvent((*NPC).activator, EV_JUMP, (*(*NPC).activator).health);
                }
            }
            _ => {}
        }
    } else if TIMER_Done2(NPC, c"attack_dmg2".as_ptr(), QTRUE) != QFALSE {
        match (*(*NPC).client).ps.legsAnim {
            x if x == BOTH_MELEE1 => {}
            x if x == BOTH_MELEE2 => {
                Rancor_Bite();
            }
            x if x == BOTH_ATTACK1 => {}
            x if x == BOTH_ATTACK2 => {}
            x if x == BOTH_ATTACK3 => {
                if (*NPC).count == 1 && !(*NPC).activator.is_null() {
                    //swallow victim
                    G_Sound(
                        (*NPC).activator,
                        CHAN_AUTO,
                        G_SoundIndex("sound/chars/rancor/chomp.wav"),
                    );
                    //FIXME: sometimes end up with a live one in our mouths?
                    //just make sure they're dead
                    if (*(*NPC).activator).health > 0 {
                        //cut in half
                        //NPC->activator->client->dismembered = qfalse;
                        G_Dismember(
                            (*NPC).activator,
                            NPC,
                            &(*(*NPC).activator).r.currentOrigin,
                            G2_MODELPART_WAIST,
                            90.0,
                            0.0,
                            (*(*(*NPC).activator).client).ps.torsoAnim,
                            QTRUE,
                        );
                        //G_DoDismemberment( NPC->activator, NPC->enemy->r.currentOrigin, MOD_SABER, 1000, HL_WAIST, qtrue );
                        //KILL
                        G_Damage(
                            (*NPC).activator,
                            NPC,
                            NPC,
                            &mut vec3_origin.clone(),
                            &mut (*(*NPC).activator).r.currentOrigin.clone(),
                            (*(*NPC).enemy).health + 10,
                            DAMAGE_NO_PROTECTION
                                | DAMAGE_NO_ARMOR
                                | DAMAGE_NO_KNOCKBACK
                                | DAMAGE_NO_HIT_LOC,
                            MOD_MELEE,
                        ); //, HL_NONE );
                        (*(*(*NPC).activator).client).ps.forceHandExtend = HANDEXTEND_NONE;
                        (*(*(*NPC).activator).client).ps.forceHandExtendTime = 0;
                        NPC_SetAnim(
                            (*NPC).activator,
                            SETANIM_BOTH,
                            BOTH_SWIM_IDLE1,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                        G_AddEvent((*NPC).activator, EV_JUMP, (*(*NPC).activator).health);
                    }
                    if !(*(*NPC).activator).client.is_null() {
                        //*sigh*, can't get tags right, just remove them?
                        (*(*(*NPC).activator).client).ps.eFlags |= EF_NODRAW;
                    }
                    (*NPC).count = 2;
                    TIMER_Set(NPC, c"clearGrabbed".as_ptr(), 2600);
                }
            }
            _ => {}
        }
    } else if (*(*NPC).client).ps.legsAnim == BOTH_ATTACK2 {
        if (*(*NPC).client).ps.legsTimer >= 1200 && (*(*NPC).client).ps.legsTimer <= 1350 {
            if Q_irand(0, 2) != 0 {
                Rancor_Swing(QFALSE);
            } else {
                Rancor_Swing(QTRUE);
            }
        } else if (*(*NPC).client).ps.legsTimer >= 1100 && (*(*NPC).client).ps.legsTimer <= 1550 {
            Rancor_Swing(QTRUE);
        }
    }

    // Just using this to remove the attacking flag at the right time
    TIMER_Done2(NPC, c"attacking".as_ptr(), QTRUE);
}

/*
-------------------------
NPC_BSRancor_Default
-------------------------
*/
pub unsafe fn NPC_BSRancor_Default() {
    AddSightEvent(NPC, &(*NPC).r.currentOrigin, 1024.0, AEL_DANGER_GREAT, 50.0);

    Rancor_Crush();

    (*(*NPC).client).ps.eFlags2 &= !(EF2_USE_ALT_ANIM | EF2_GENERIC_NPC_FLAG);
    if (*NPC).count != 0 {
        //holding someone
        (*(*NPC).client).ps.eFlags2 |= EF2_USE_ALT_ANIM;
        if (*NPC).count == 2 {
            //in my mouth
            (*(*NPC).client).ps.eFlags2 |= EF2_GENERIC_NPC_FLAG;
        }
    } else {
        (*(*NPC).client).ps.eFlags2 &= !(EF2_USE_ALT_ANIM | EF2_GENERIC_NPC_FLAG);
    }

    if TIMER_Done2(NPC, c"clearGrabbed".as_ptr(), QTRUE) != QFALSE {
        Rancor_DropVictim(NPC);
    } else if (*(*NPC).client).ps.legsAnim == BOTH_PAIN2
        && (*NPC).count == 1
        && !(*NPC).activator.is_null()
    {
        if Q_irand(0, 3) == 0 {
            Rancor_CheckDropVictim();
        }
    }
    if TIMER_Done(NPC, c"rageTime".as_ptr()) == QFALSE {
        //do nothing but roar first time we see an enemy
        AddSoundEvent(NPC, &(*NPC).r.currentOrigin, 1024.0, AEL_DANGER_GREAT, QFALSE); //, qfalse );
        NPC_FaceEnemy(QTRUE);
        return;
    }
    if !(*NPC).enemy.is_null() {
        /*
        if ( NPC->enemy->client //enemy is a client
            && (NPC->enemy->client->NPC_class == CLASS_UGNAUGHT || NPC->enemy->client->NPC_class == CLASS_JAWA )//enemy is a lowly jawa or ugnaught
            && NPC->enemy->enemy != NPC//enemy's enemy is not me
            && (!NPC->enemy->enemy || !NPC->enemy->enemy->client || NPC->enemy->enemy->client->NPC_class!=CLASS_RANCOR) )//enemy's enemy is not a client or is not a rancor (which is as scary as me anyway)
        {//they should be scared of ME and no-one else
            G_SetEnemy( NPC->enemy, NPC );
        }
        */
        if TIMER_Done(NPC, c"angrynoise".as_ptr()) != QFALSE {
            G_Sound(
                NPC,
                CHAN_AUTO,
                G_SoundIndex(&format!(
                    "sound/chars/rancor/misc/anger{}.wav",
                    Q_irand(1, 3)
                )),
            );

            TIMER_Set(NPC, c"angrynoise".as_ptr(), Q_irand(5000, 10000));
        } else {
            AddSoundEvent(NPC, &(*NPC).r.currentOrigin, 512.0, AEL_DANGER_GREAT, QFALSE);
            //, qfalse );
        }
        if (*NPC).count == 2 && (*(*NPC).client).ps.legsAnim == BOTH_ATTACK3 {
            //we're still chewing our enemy up
            NPC_UpdateAngles(QTRUE, QTRUE);
            return;
        }
        //else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
        if !(*(*NPC).enemy).client.is_null()
            && (*(*(*NPC).enemy).client).NPC_class == CLASS_RANCOR
        {
            //got mad at another Rancor, look for a valid enemy
            if TIMER_Done(NPC, c"rancorInfight".as_ptr()) != QFALSE {
                NPC_CheckEnemyExt(QTRUE);
            }
        } else if (*NPC).count == 0 {
            if ValidEnemy((*NPC).enemy) == QFALSE {
                TIMER_Remove(NPC, c"lookForNewEnemy".as_ptr()); //make them look again right now
                if (*(*NPC).enemy).inuse == QFALSE
                    || (*addr_of!(level)).time - (*(*NPC).enemy).s.time > Q_irand(10000, 15000)
                {
                    //it's been a while since the enemy died, or enemy is completely gone, get bored with him
                    (*NPC).enemy = null_mut();
                    Rancor_Patrol();
                    NPC_UpdateAngles(QTRUE, QTRUE);
                    return;
                }
            }
            if TIMER_Done(NPC, c"lookForNewEnemy".as_ptr()) != QFALSE {
                let sav_enemy: *mut gentity_t = (*NPC).enemy; //FIXME: what about NPC->lastEnemy?
                (*NPC).enemy = null_mut();
                let newEnemy: *mut gentity_t = NPC_CheckEnemy(
                    ((*NPCInfo).confusionTime < (*addr_of!(level)).time) as qboolean,
                    QFALSE,
                    QFALSE,
                );
                (*NPC).enemy = sav_enemy;
                if !newEnemy.is_null() && newEnemy != sav_enemy {
                    //picked up a new enemy!
                    (*NPC).lastEnemy = (*NPC).enemy;
                    G_SetEnemy(NPC, newEnemy);
                    //hold this one for at least 5-15 seconds
                    TIMER_Set(NPC, c"lookForNewEnemy".as_ptr(), Q_irand(5000, 15000));
                } else {
                    //look again in 2-5 secs
                    TIMER_Set(NPC, c"lookForNewEnemy".as_ptr(), Q_irand(2000, 5000));
                }
            }
        }
        Rancor_Combat();
    } else {
        if TIMER_Done(NPC, c"idlenoise".as_ptr()) != QFALSE {
            G_Sound(
                NPC,
                CHAN_AUTO,
                G_SoundIndex(&format!("sound/chars/rancor/snort_{}.wav", Q_irand(1, 2))),
            );

            TIMER_Set(NPC, c"idlenoise".as_ptr(), Q_irand(2000, 4000));
            AddSoundEvent(NPC, &(*NPC).r.currentOrigin, 384.0, AEL_DANGER, QFALSE); //, qfalse );
        }
        if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            Rancor_Patrol();
        } else {
            Rancor_Idle();
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}
