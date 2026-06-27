//! Full port of `NPC_AI_Interrogator.c` (PC tree) — the floating torture/
//! interrogator droid's behavior state (COMPLETE, 10/10). The PC module ships
//! this file LIVE (the `refs/raven-jediacademy` Xbox tree `#if 0`s it out). The
//! maintain-height / parts-move / strafe / hunt / melee / attack-decision / idle
//! chain drives the droid's combat AI on top of `NPC_UpdateAngles`,
//! `NPC_SetBoneAngles`, `NPC_GetMoveDirection`, `NPC_FaceEnemy`,
//! `NPC_CheckEnemyExt`, `NPC_ClearLOS4`, `NPC_CheckPlayerTeamStealth`,
//! `NPC_BSIdle`, the timer/sound/effect helpers, and `G_Damage`.
//!
//! Ported here: `NPC_Interrogator_Precache` (NPC_AI_Interrogator.c:20),
//! `Interrogator_die` (:34), `Interrogator_PartsMove` (:64),
//! `Interrogator_MaintainHeight` (:137), `Interrogator_Strafe` (:238),
//! `Interrogator_Hunt` (:290), `Interrogator_Melee` (:345),
//! `Interrogator_Attack` (:381), `Interrogator_Idle` (:435),
//! `NPC_BSInterrogator_Default` (:454).

#![allow(non_snake_case)] // C function names (`NPC_BSInterrogator_Default`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::b_public_h::SCF_CHASE_ENEMIES;
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_public::{EF2_FLYING, MASK_SOLID, MOD_MELEE};
use crate::codemp::game::g_combat::G_Damage;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_NO_KNOCKBACK};
use crate::codemp::game::g_main::{g_spskill, level};
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_EffectIndex, G_Sound, G_SoundIndex, G_SoundOnEnt};
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC};
use crate::codemp::game::npc_ai_default::NPC_BSIdle;
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_move::NPC_GetMoveDirection;
use crate::codemp::game::npc_utils::{NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy,
    NPC_SetBoneAngles, NPC_UpdateAngles};
use crate::codemp::game::q_math::{AngleNormalize360, AngleVectors, DistanceHorizontalSquared,
    Q_irand, VectorMA, VectorNormalize, VectorSubtract};
use crate::codemp::game::q_shared::random;
use crate::codemp::game::q_shared_h::{vec3_t, CHAN_AUTO};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// Local state enum (scalpel blade motion)
const LSTATE_BLADESTOP: c_int = 0;
const LSTATE_BLADEUP: c_int = 1;
const LSTATE_BLADEDOWN: c_int = 2;

const _: () = assert!(LSTATE_BLADESTOP == 0); // silence unused-const lint while keeping the enum verbatim

/*
-------------------------
NPC_Interrogator_Precache
-------------------------
*/
pub unsafe fn NPC_Interrogator_Precache(_self_: *mut gentity_t) {
    G_SoundIndex("sound/chars/interrogator/misc/torture_droid_lp");
    G_SoundIndex("sound/chars/mark1/misc/anger.wav");
    G_SoundIndex("sound/chars/probe/misc/talk");
    G_SoundIndex("sound/chars/interrogator/misc/torture_droid_inject");
    G_SoundIndex("sound/chars/interrogator/misc/int_droid_explo");
    G_EffectIndex("explosions/droidexplosion1");
}

/*
-------------------------
Interrogator_die
-------------------------
*/
// C dieFunc is `void(self, inflictor, attacker, damage, mod, dFlags, hitLoc)`;
// this 7-arg orphan is never wired in MP (`self->e_DieFunc = dieF_Interrogator_die;`
// is commented out in NPC_BSInterrogator_Default), so it is ported as a plain
// `unsafe fn` rather than the 5-arg `extern "C"` die-callback ABI.
pub unsafe fn Interrogator_die(
    self_: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
    _dFlags: c_int,
    _hitLoc: c_int,
) {
    (*(*self_).client).ps.velocity[2] = -100.0;
    /*
    self->locationDamage[HL_NONE] += damage;
    if (self->locationDamage[HL_NONE] > 40)
    {
        DeathFX(self);
        self->client->ps.eFlags |= EF_NODRAW;
        self->contents = CONTENTS_CORPSE;
    }
    else
    */
    {
        (*(*self_).client).ps.eFlags2 &= !EF2_FLYING; //moveType = MT_WALK;
        (*(*self_).client).ps.velocity[0] = Q_irand(-10, -20) as f32;
        (*(*self_).client).ps.velocity[1] = Q_irand(-10, -20) as f32;
        (*(*self_).client).ps.velocity[2] = -100.0;
    }
    //self->takedamage = qfalse;
    //self->client->ps.eFlags |= EF_NODRAW;
    //self->contents = 0;
}

/*
-------------------------
Interrogator_PartsMove
-------------------------
*/
pub unsafe fn Interrogator_PartsMove() {
    // Syringe
    if TIMER_Done(NPC, c"syringeDelay".as_ptr()) != QFALSE {
        (*NPC).pos1[1] = AngleNormalize360((*NPC).pos1[1]);

        if (*NPC).pos1[1] < 60.0 || (*NPC).pos1[1] > 300.0 {
            (*NPC).pos1[1] += Q_irand(-20, 20) as f32; // Pitch
        } else if (*NPC).pos1[1] > 180.0 {
            (*NPC).pos1[1] = Q_irand(300, 360) as f32; // Pitch
        } else {
            (*NPC).pos1[1] = Q_irand(0, 60) as f32; // Pitch
        }

        //	gi.G2API_SetBoneAnglesIndex( &NPC->ghoul2[NPC->playerModel], NPC->genericBone1, NPC->pos1, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, NULL );
        NPC_SetBoneAngles(NPC, "left_arm", &(*NPC).pos1);

        TIMER_Set(NPC, c"syringeDelay".as_ptr(), Q_irand(100, 1000));
    }

    // Scalpel
    if TIMER_Done(NPC, c"scalpelDelay".as_ptr()) != QFALSE {
        // Change pitch
        if (*NPCInfo).localState == LSTATE_BLADEDOWN
        // Blade is moving down
        {
            (*NPC).pos2[0] -= 30.0;
            if (*NPC).pos2[0] < 180.0 {
                (*NPC).pos2[0] = 180.0;
                (*NPCInfo).localState = LSTATE_BLADEUP; // Make it move up
            }
        } else
        // Blade is coming back up
        {
            (*NPC).pos2[0] += 30.0;
            if (*NPC).pos2[0] >= 360.0 {
                (*NPC).pos2[0] = 360.0;
                (*NPCInfo).localState = LSTATE_BLADEDOWN; // Make it move down
                TIMER_Set(NPC, c"scalpelDelay".as_ptr(), Q_irand(100, 1000));
            }
        }

        (*NPC).pos2[0] = AngleNormalize360((*NPC).pos2[0]);
        //	gi.G2API_SetBoneAnglesIndex( &NPC->ghoul2[NPC->playerModel], NPC->genericBone2, NPC->pos2, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, NULL );

        NPC_SetBoneAngles(NPC, "right_arm", &(*NPC).pos2);
    }

    // Claw
    (*NPC).pos3[1] += Q_irand(10, 30) as f32;
    (*NPC).pos3[1] = AngleNormalize360((*NPC).pos3[1]);
    //gi.G2API_SetBoneAnglesIndex( &NPC->ghoul2[NPC->playerModel], NPC->genericBone3, NPC->pos3, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, NULL );

    NPC_SetBoneAngles(NPC, "claw", &(*NPC).pos3);
}

// These define the working friction behavior for the hovering droid
const VELOCITY_DECAY: f32 = 0.85;
const HUNTER_UPWARD_PUSH: f32 = 2.0;

/*
-------------------------
Interrogator_MaintainHeight
-------------------------
*/
pub unsafe fn Interrogator_MaintainHeight() {
    let mut dif: f32;
    //	vec3_t	endPos;
    //	trace_t	trace;

    (*NPC).s.loopSound = G_SoundIndex("sound/chars/interrogator/misc/torture_droid_lp");
    // Update our angles regardless
    NPC_UpdateAngles(QTRUE, QTRUE);

    // If we have an enemy, we should try to hover at about enemy eye level
    if !(*NPC).enemy.is_null() {
        // Find the height difference
        dif = ((*(*NPC).enemy).r.currentOrigin[2] + (*(*NPC).enemy).r.maxs[2])
            - (*NPC).r.currentOrigin[2];

        // cap to prevent dramatic height shifts
        if dif.abs() > 2.0 {
            if dif.abs() > 16.0 {
                dif = if dif < 0.0 { -16.0 } else { 16.0 };
            }

            (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
        }
    } else {
        let goal: *mut gentity_t;

        if !(*NPCInfo).goalEntity.is_null()
        // Is there a goal?
        {
            goal = (*NPCInfo).goalEntity;
        } else {
            goal = (*NPCInfo).lastGoalEntity;
        }
        if !goal.is_null() {
            dif = (*goal).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];

            if dif.abs() > 24.0 {
                ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
            } else if (*(*NPC).client).ps.velocity[2] != 0.0 {
                (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                if (*(*NPC).client).ps.velocity[2].abs() < 2.0 {
                    (*(*NPC).client).ps.velocity[2] = 0.0;
                }
            }
        }
        // Apply friction
        else if (*(*NPC).client).ps.velocity[2] != 0.0 {
            (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[2].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[2] = 0.0;
            }
        }
    }

    // Apply friction
    if (*(*NPC).client).ps.velocity[0] != 0.0 {
        (*(*NPC).client).ps.velocity[0] *= VELOCITY_DECAY;

        if (*(*NPC).client).ps.velocity[0].abs() < 1.0 {
            (*(*NPC).client).ps.velocity[0] = 0.0;
        }
    }

    if (*(*NPC).client).ps.velocity[1] != 0.0 {
        (*(*NPC).client).ps.velocity[1] *= VELOCITY_DECAY;

        if (*(*NPC).client).ps.velocity[1].abs() < 1.0 {
            (*(*NPC).client).ps.velocity[1] = 0.0;
        }
    }
}

const HUNTER_STRAFE_VEL: f32 = 32.0;
const HUNTER_STRAFE_DIS: f32 = 200.0;

/*
-------------------------
Interrogator_Strafe
-------------------------
*/
pub unsafe fn Interrogator_Strafe() {
    let dir: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let tr;
    let mut dif: f32;

    AngleVectors(&(*(*NPC).client).renderInfo.eyeAngles, None, Some(&mut right), None);

    // Pick a random strafe direction, then check to see if doing a strafe would be
    //	reasonable valid
    dir = if (rand() & 1) != 0 { -1 } else { 1 };
    VectorMA(&(*NPC).r.currentOrigin, HUNTER_STRAFE_DIS * dir as f32, &right, &mut end);

    tr = trap::Trace(
        &(*NPC).r.currentOrigin,
        &[0.0; 3],
        &[0.0; 3],
        &end,
        (*NPC).s.number,
        MASK_SOLID,
    );

    // Close enough
    if tr.fraction > 0.9 {
        let vel = (*(*NPC).client).ps.velocity;
        VectorMA(&vel, HUNTER_STRAFE_VEL * dir as f32, &right, &mut (*(*NPC).client).ps.velocity);

        // Add a slight upward push
        if !(*NPC).enemy.is_null() {
            // Find the height difference
            dif = ((*(*NPC).enemy).r.currentOrigin[2] + 32.0) - (*NPC).r.currentOrigin[2];

            // cap to prevent dramatic height shifts
            if dif.abs() > 8.0 {
                dif = if dif < 0.0 { -HUNTER_UPWARD_PUSH } else { HUNTER_UPWARD_PUSH };
            }

            (*(*NPC).client).ps.velocity[2] += dif;
        }

        // Set the strafe start time
        //NPC->fx_time = level.time;
        (*NPCInfo).standTime = (*addr_of!(level)).time + 3000 + (random() * 500.0) as c_int;
    }
}

/*
-------------------------
Interrogator_Hunt
-------------------------`
*/

const HUNTER_FORWARD_BASE_SPEED: f32 = 10.0;
const HUNTER_FORWARD_MULTIPLIER: f32 = 2.0;

pub unsafe fn Interrogator_Hunt(visible: qboolean, advance: qboolean) {
    let mut distance: f32 = 0.0;
    let speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    Interrogator_PartsMove();

    NPC_FaceEnemy(QFALSE);

    //If we're not supposed to stand still, pursue the player
    if (*NPCInfo).standTime < (*addr_of!(level)).time {
        // Only strafe when we can see the player
        if visible != QFALSE {
            Interrogator_Strafe();
            if (*NPCInfo).standTime > (*addr_of!(level)).time {
                //successfully strafed
                return;
            }
        }
    }

    //If we don't want to advance, stop here
    if advance == QFALSE {
        return;
    }

    //Only try and navigate if the player is visible
    if visible == QFALSE {
        // Move towards our goal
        (*NPCInfo).goalEntity = (*NPC).enemy;
        (*NPCInfo).goalRadius = 12;

        //Get our direction from the navigator if we can't see our target
        if NPC_GetMoveDirection(&mut forward, &mut distance) == QFALSE {
            return;
        }
    } else {
        VectorSubtract(&(*(*NPC).enemy).r.currentOrigin, &(*NPC).r.currentOrigin, &mut forward);
        distance = VectorNormalize(&mut forward);
    }
    let _ = distance;

    speed = HUNTER_FORWARD_BASE_SPEED
        + HUNTER_FORWARD_MULTIPLIER * (*addr_of!(g_spskill)).integer as f32;
    let curvel = (*(*NPC).client).ps.velocity;
    VectorMA(&curvel, speed, &forward, &mut (*(*NPC).client).ps.velocity);
}

const MIN_DISTANCE: c_int = 64;

/*
-------------------------
Interrogator_Melee
-------------------------
*/
pub unsafe fn Interrogator_Melee(visible: qboolean, advance: qboolean) {
    if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE
    // Attack?
    {
        // Make sure that we are within the height range before we allow any damage to happen
        if (*NPC).r.currentOrigin[2]
            >= (*(*NPC).enemy).r.currentOrigin[2] + (*(*NPC).enemy).r.mins[2]
            && (*NPC).r.currentOrigin[2] + (*NPC).r.mins[2] + 8.0
                < (*(*NPC).enemy).r.currentOrigin[2] + (*(*NPC).enemy).r.maxs[2]
        {
            //gentity_t *tent;

            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 3000));
            G_Damage((*NPC).enemy, NPC, NPC, core::ptr::null_mut(), core::ptr::null_mut(), 2,
                DAMAGE_NO_KNOCKBACK, MOD_MELEE);

            //	NPC->enemy->client->poisonDamage = 18;
            //	NPC->enemy->client->poisonTime = level.time + 1000;

            // Drug our enemy up and do the wonky vision thing
            //			tent = G_TempEntity( NPC->enemy->r.currentOrigin, EV_DRUGGED );
            //			tent->owner = NPC->enemy;

            //rwwFIXMEFIXME: poison damage

            G_Sound(NPC, CHAN_AUTO,
                G_SoundIndex("sound/chars/interrogator/misc/torture_droid_inject.mp3"));
        }
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        Interrogator_Hunt(visible, advance);
    }
}

/*
-------------------------
Interrogator_Attack
-------------------------
*/
pub unsafe fn Interrogator_Attack() {
    let distance: f32;
    let visible: qboolean;
    let mut advance: qboolean;

    // Always keep a good height off the ground
    Interrogator_MaintainHeight();

    //randomly talk
    if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
        if TIMER_Done(NPC, c"angerNoise".as_ptr()) != QFALSE {
            // C: `va("sound/chars/probe/misc/talk.wav", Q_irand(1, 3))` — the format
            // string has no conversion specifier, so va returns it verbatim; the
            // Q_irand call is still evaluated (RNG advance) then discarded by va.
            let _ = Q_irand(1, 3);
            G_SoundOnEnt(NPC, CHAN_AUTO, "sound/chars/probe/misc/talk.wav");

            TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(4000, 10000));
        }
    }

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE {
        Interrogator_Idle();
        return;
    }

    // Rate our distance to the target, and our visibilty
    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
        as c_int as f32;
    visible = NPC_ClearLOS4((*NPC).enemy);
    advance = (distance > (MIN_DISTANCE * MIN_DISTANCE) as f32) as qboolean;

    if visible == QFALSE {
        advance = QTRUE;
    }
    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        Interrogator_Hunt(visible, advance);
    }

    NPC_FaceEnemy(QTRUE);

    if advance == QFALSE {
        Interrogator_Melee(visible, advance);
    }
}

/*
-------------------------
Interrogator_Idle
-------------------------
*/
pub unsafe fn Interrogator_Idle() {
    if NPC_CheckPlayerTeamStealth() != QFALSE {
        G_SoundOnEnt(NPC, CHAN_AUTO, "sound/chars/mark1/misc/anger.wav");
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    Interrogator_MaintainHeight();

    NPC_BSIdle();
}

/*
-------------------------
NPC_BSInterrogator_Default
-------------------------
*/
pub unsafe fn NPC_BSInterrogator_Default() {
    //NPC->e_DieFunc = dieF_Interrogator_die;

    if !(*NPC).enemy.is_null() {
        Interrogator_Attack();
    } else {
        Interrogator_Idle();
    }
}
