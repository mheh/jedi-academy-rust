//! Slice of `NPC_AI_Remote.c` — the floating "remote" (lightsaber-training droid)
//! NPC behavior state. Opened bottom-up at the leaf seam: the precache, the strafe
//! evasion move, the missile-fire helper, and the height-maintenance / hunt /
//! ranged-attack / patrol movement chain are genuinely portable today.
//!
//! Ported here so far: `NPC_Remote_Precache` (NPC_AI_Remote.c:17),
//! `NPC_Remote_Pain` (:29), `Remote_MaintainHeight` (:44), `Remote_Strafe` (:139),
//! `Remote_Hunt` (:178), `Remote_Fire` (:229), `Remote_Ranged` (:264),
//! `Remote_Attack` (:290), `Remote_Idle` (:339), `Remote_Patrol` (:351),
//! `NPC_BSRemote_Default` (:375). The whole behavior file is now here.

#![allow(non_snake_case)] // C function names (`NPC_Remote_Precache`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::addr_of;

use crate::codemp::game::b_public_h::{SCF_CHASE_ENEMIES, SCF_LOOK_FOR_ENEMIES, SPOT_HEAD};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_public::{MASK_SHOT, MASK_SOLID, MOD_BRYAR_PISTOL};
use crate::codemp::game::bg_weapons_h::WP_BRYAR_PISTOL;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_DEATH_KNOCKBACK};
use crate::codemp::game::g_main::{g_spskill, level};
use crate::codemp::game::g_missile::CreateMissile;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex};
use crate::codemp::game::npc::{ucmd, RestoreNPCGlobals, SaveNPCGlobals, SetNPCGlobals, NPC, NPCInfo};
use crate::codemp::game::npc_ai_default::NPC_BSIdle;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::{NPC_GetMoveDirection, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_UpdateAngles,
};
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleVectors, DistanceHorizontalSquared, VectorCopy,
    VectorMA, VectorNormalize, VectorSubtract,
};
use crate::codemp::game::q_shared::{random};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{trace_t, vec3_t, BUTTON_WALKING, CHAN_AUTO};
use crate::codemp::game::surfaceflags_h::CONTENTS_LIGHTSABER;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

/// `#define VELOCITY_DECAY 0.85f` (NPC_AI_Remote.c:6).
const VELOCITY_DECAY: f32 = 0.85;

/*
-------------------------
NPC_Remote_Precache
-------------------------
*/
pub fn NPC_Remote_Precache() {
    G_SoundIndex("sound/chars/remote/misc/fire.wav");
    G_SoundIndex("sound/chars/remote/misc/hiss.wav");
    G_EffectIndex("env/small_explode");
}

/*
-------------------------
NPC_Remote_Pain
-------------------------
*/
pub unsafe extern "C" fn NPC_Remote_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    SaveNPCGlobals();
    SetNPCGlobals(self_);
    Remote_Strafe();
    RestoreNPCGlobals();

    NPC_Pain(self_, attacker, damage);
}

/*
-------------------------
Remote_MaintainHeight
-------------------------
*/
pub unsafe fn Remote_MaintainHeight() {
    let mut dif: f32;

    // Update our angles regardless
    NPC_UpdateAngles(QTRUE, QTRUE);

    if (*(*NPC).client).ps.velocity[2] != 0.0 {
        (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

        if (*(*NPC).client).ps.velocity[2].abs() < 2.0 {
            (*(*NPC).client).ps.velocity[2] = 0.0;
        }
    }
    // If we have an enemy, we should try to hover at or a little below enemy eye level
    if !(*NPC).enemy.is_null() {
        if TIMER_Done(NPC, c"heightChange".as_ptr()) != QFALSE {
            TIMER_Set(NPC, c"heightChange".as_ptr(), Q_irand(1000, 3000));

            // Find the height difference
            dif = ((*(*NPC).enemy).r.currentOrigin[2]
                + Q_irand(0, (*(*NPC).enemy).r.maxs[2] as c_int + 8) as f32)
                - (*NPC).r.currentOrigin[2];

            // cap to prevent dramatic height shifts
            if dif.abs() > 2.0 {
                if dif.abs() > 24.0 {
                    dif = if dif < 0.0 { -24.0 } else { 24.0 };
                }
                dif *= 10.0;
                (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
                //	NPC->fx_time = level.time;
                G_Sound(NPC, CHAN_AUTO, G_SoundIndex("sound/chars/remote/misc/hiss.wav"));
            }
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
                dif = if dif < 0.0 { -24.0 } else { 24.0 };
                (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
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

const REMOTE_STRAFE_VEL: f32 = 256.0;
const REMOTE_STRAFE_DIS: f32 = 200.0;
const REMOTE_UPWARD_PUSH: f32 = 32.0;

/*
-------------------------
Remote_Strafe
-------------------------
*/
pub unsafe fn Remote_Strafe() {
    let dir: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let tr: trace_t;

    AngleVectors(
        &(*(*NPC).client).renderInfo.eyeAngles,
        None,
        Some(&mut right),
        None,
    );

    // Pick a random strafe direction, then check to see if doing a strafe would be
    //	reasonable valid
    dir = if rand() & 1 != 0 { -1 } else { 1 };
    VectorMA(
        &(*NPC).r.currentOrigin,
        REMOTE_STRAFE_DIS * dir as f32,
        &right,
        &mut end,
    );

    tr = trap::Trace(
        &(*NPC).r.currentOrigin,
        &vec3_origin,
        &vec3_origin,
        &end,
        (*NPC).s.number,
        MASK_SOLID,
    );

    // Close enough
    if tr.fraction > 0.9 {
        let vel = (*(*NPC).client).ps.velocity;
        VectorMA(
            &vel,
            REMOTE_STRAFE_VEL * dir as f32,
            &right,
            &mut (*(*NPC).client).ps.velocity,
        );

        G_Sound(NPC, CHAN_AUTO, G_SoundIndex("sound/chars/remote/misc/hiss.wav"));

        // Add a slight upward push
        (*(*NPC).client).ps.velocity[2] += REMOTE_UPWARD_PUSH;

        // Set the strafe start time so we can do a controlled roll
        //	NPC->fx_time = level.time;
        (*NPCInfo).standTime = (*addr_of!(level)).time + 3000 + (random() * 500.0) as c_int;
    }
}

const REMOTE_FORWARD_BASE_SPEED: f32 = 10.0;
const REMOTE_FORWARD_MULTIPLIER: f32 = 5.0;

/*
-------------------------
Remote_Hunt
-------------------------
*/
pub unsafe fn Remote_Hunt(visible: qboolean, advance: qboolean, retreat: qboolean) {
    let mut distance: f32 = 0.0;
    let mut speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    // If we're not supposed to stand still, pursue the player
    if (*NPCInfo).standTime < (*addr_of!(level)).time {
        // Only strafe when we can see the player
        if visible != QFALSE {
            Remote_Strafe();
            return;
        }
    }

    // If we don't want to advance, stop here
    if advance == QFALSE && visible == QTRUE {
        return;
    }

    // Only try and navigate if the player is visible
    if visible == QFALSE {
        // Move towards our goal
        (*NPCInfo).goalEntity = (*NPC).enemy;
        (*NPCInfo).goalRadius = 12;

        // Get our direction from the navigator if we can't see our target
        if NPC_GetMoveDirection(&mut forward, &mut distance) == QFALSE {
            return;
        }
    } else {
        VectorSubtract(&(*(*NPC).enemy).r.currentOrigin, &(*NPC).r.currentOrigin, &mut forward);
        distance = VectorNormalize(&mut forward);
    }
    let _ = distance;

    speed = REMOTE_FORWARD_BASE_SPEED
        + REMOTE_FORWARD_MULTIPLIER * (*addr_of!(g_spskill)).integer as f32;
    if retreat == QTRUE {
        speed *= -1.0;
    }
    let curvel = (*(*NPC).client).ps.velocity;
    VectorMA(&curvel, speed, &forward, &mut (*(*NPC).client).ps.velocity);
}

/*
-------------------------
Remote_Fire
-------------------------
*/
pub unsafe fn Remote_Fire() {
    let mut delta1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    // C: static vec3_t forward, vright, up; static vec3_t muzzle; — the `muzzle`
    // static is dead (never read), elided here.
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let missile: *mut gentity_t;

    CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_org1);
    VectorCopy(&(*NPC).r.currentOrigin, &mut muzzle1);

    VectorSubtract(&enemy_org1, &muzzle1, &mut delta1);

    vectoangles(&delta1, &mut angleToEnemy1);
    AngleVectors(
        &angleToEnemy1,
        Some(&mut forward),
        Some(&mut vright),
        Some(&mut up),
    );

    missile = CreateMissile(
        &mut (*NPC).r.currentOrigin,
        &forward,
        1000.0,
        10000,
        NPC,
        QFALSE,
    );

    G_PlayEffectID(
        G_EffectIndex("bryar/muzzle_flash"),
        &(*NPC).r.currentOrigin,
        &forward,
    );

    (*missile).classname = c"briar".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    (*missile).damage = 10;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
Remote_Ranged
-------------------------
*/
pub unsafe fn Remote_Ranged(visible: qboolean, advance: qboolean, retreat: qboolean) {
    if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE
    // Attack?
    {
        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 3000));
        Remote_Fire();
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        Remote_Hunt(visible, advance, retreat);
    }
}

// NPC_AI_Remote.c also defines MIN_MELEE_RANGE / MIN_MELEE_RANGE_SQR (:279-280),
// used by no code in this file.
const MIN_DISTANCE: f32 = 80.0;
const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

/*
-------------------------
Remote_Attack
-------------------------
*/
pub unsafe fn Remote_Attack() {
    let distance: f32;
    let visible: qboolean;
    let idealDist: f32;
    let advance: qboolean;
    let retreat: qboolean;

    if TIMER_Done(NPC, c"spin".as_ptr()) != QFALSE {
        TIMER_Set(NPC, c"spin".as_ptr(), Q_irand(250, 1500));
        (*NPCInfo).desiredYaw += Q_irand(-200, 200) as f32;
    }
    // Always keep a good height off the ground
    Remote_MaintainHeight();

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE {
        Remote_Idle();
        return;
    }

    // Rate our distance to the target, and our visibilty
    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
        as c_int as f32;
    visible = NPC_ClearLOS4((*NPC).enemy);
    idealDist = MIN_DISTANCE_SQR + (MIN_DISTANCE_SQR * flrand(0.0, 1.0));
    advance = (distance > idealDist * 1.25) as qboolean;
    retreat = (distance < idealDist * 0.75) as qboolean;

    // If we cannot see our target, move to see it
    if visible == QFALSE {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            Remote_Hunt(visible, advance, retreat);
            return;
        }
    }

    Remote_Ranged(visible, advance, retreat);
}

/*
-------------------------
Remote_Idle
-------------------------
*/
pub unsafe fn Remote_Idle() {
    Remote_MaintainHeight();

    NPC_BSIdle();
}

/*
-------------------------
Remote_Patrol
-------------------------
*/
pub unsafe fn Remote_Patrol() {
    Remote_MaintainHeight();

    // If we have somewhere to go, then do that
    if (*NPC).enemy.is_null() {
        if !UpdateGoal().is_null() {
            // start loop sound once we move
            ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(QTRUE);
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
NPC_BSRemote_Default
-------------------------
*/
pub unsafe fn NPC_BSRemote_Default() {
    if !(*NPC).enemy.is_null() {
        Remote_Attack();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        Remote_Patrol();
    } else {
        Remote_Idle();
    }
}
