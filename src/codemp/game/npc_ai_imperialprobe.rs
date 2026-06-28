//! Full port of `NPC_AI_ImperialProbe.c` — the floating imperial probe droid's
//! behavior state (COMPLETE, 12/12). The maintain-height / strafe / hunt / fire /
//! ranged / attack-decision / idle / patrol / wait chain drives the probe droid's
//! combat AI on top of `NPC_UpdateAngles`, `NPC_GetMoveDirection`, `NPC_MoveToGoal`,
//! `UpdateGoal`, `NPC_ClearLOS4`, `NPC_FaceEnemy`, `NPC_CheckEnemyExt`,
//! `NPC_CheckPlayerTeamStealth`, `NPC_BSIdle`, the `trap_G2API_*` Ghoul2 bolt API,
//! and the NPC-AI / NAV core.
//!
//! Ported here: `NPC_Probe_Precache` (NPC_AI_ImperialProbe.c:21),
//! `ImperialProbe_MaintainHeight` (:49), `ImperialProbe_Strafe` (:182),
//! `ImperialProbe_Hunt` (:220), `ImperialProbe_FireBlaster` (:268),
//! `ImperialProbe_Ranged` (:331), `ImperialProbe_AttackDecision` (:377),
//! `NPC_Probe_Pain` (:433), `ImperialProbe_Idle` (:506),
//! `ImperialProbe_Patrol` (:518), `ImperialProbe_Wait` (:563),
//! `NPC_BSImperialProbe_Default` (:589).

#![allow(non_snake_case)] // C function names (`NPC_Probe_Pain`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, null_mut};

use crate::codemp::game::anims::{BOTH_PAIN1, BOTH_RUN1};
use crate::codemp::game::b_public_h::{SCF_CHASE_ENEMIES, SCF_LOOK_FOR_ENEMIES, SPOT_CHEST};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_misc::{BG_FindItemForAmmo, BG_FindItemForWeapon};
use crate::codemp::game::bg_public::{
    BG_GiveMeVectorFromMatrix, MASK_SHOT, MASK_SOLID, MOD_DEMP2, MOD_DEMP2_ALT, MOD_UNKNOWN,
    SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_NORMAL, SETANIM_FLAG_OVERRIDE,
};
use crate::codemp::game::bg_weapons_h::{AMMO_BLASTER, WP_BRYAR_PISTOL};
use crate::codemp::game::g_combat::{gPainMOD, G_Damage};
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_DEATH_KNOCKBACK};
use crate::codemp::game::g_main::{g_spskill, level};
use crate::codemp::game::g_missile::CreateMissile;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{
    G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex, G_SoundOnEnt,
};
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC_SetAnim, NPC};
use crate::codemp::game::npc_ai_default::NPC_BSIdle;
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::{NPC_GetMoveDirection, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::{NPC_GetPainChance, NPC_Pain};
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleNormalize360, AngleVectors, DistanceHorizontalSquared,
    VectorCopy, VectorMA, VectorNormalize, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::random;
use crate::codemp::game::q_shared_h::{mdxaBone_t, vec3_t, BUTTON_WALKING, CHAN_AUTO, ORIGIN};
use crate::codemp::game::surfaceflags_h::CONTENTS_LIGHTSABER;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// These define the working friction behavior for the hovering probe droid
const VELOCITY_DECAY: f32 = 0.85;

const HUNTER_STRAFE_VEL: f32 = 256.0;
const HUNTER_STRAFE_DIS: f32 = 200.0;
const HUNTER_UPWARD_PUSH: f32 = 32.0;

// Local state enums
const LSTATE_NONE: c_int = 0;
#[allow(dead_code)]
const LSTATE_BACKINGUP: c_int = 1;
#[allow(dead_code)]
const LSTATE_SPINNING: c_int = 2;
#[allow(dead_code)]
const LSTATE_PAIN: c_int = 3;
const LSTATE_DROP: c_int = 4;

const _: () = assert!(LSTATE_NONE == 0); // silence unused-const lint while keeping the enum verbatim

pub unsafe fn NPC_Probe_Precache() {
    let mut i: c_int = 1;

    while i < 4 {
        G_SoundIndex(&format!("sound/chars/probe/misc/probetalk{}", i));
        i += 1;
    }
    G_SoundIndex("sound/chars/probe/misc/probedroidloop");
    G_SoundIndex("sound/chars/probe/misc/anger1");
    G_SoundIndex("sound/chars/probe/misc/fire");

    G_EffectIndex("chunks/probehead");
    G_EffectIndex("env/med_explode2");
    G_EffectIndex("explosions/probeexplosion1");
    G_EffectIndex("bryar/muzzle_flash");

    RegisterItem(BG_FindItemForAmmo(AMMO_BLASTER));
    RegisterItem(BG_FindItemForWeapon(WP_BRYAR_PISTOL));
}

/*
-------------------------
Hunter_MaintainHeight
-------------------------
*/
pub unsafe fn ImperialProbe_MaintainHeight() {
    let mut dif: f32;
    //	vec3_t	endPos;
    //	trace_t	trace;

    // Update our angles regardless
    NPC_UpdateAngles(QTRUE, QTRUE);

    // If we have an enemy, we should try to hover at about enemy eye level
    if !(*NPC).enemy.is_null() {
        // Find the height difference
        dif = (*(*NPC).enemy).r.currentOrigin[2] - (*NPC).r.currentOrigin[2];

        // cap to prevent dramatic height shifts
        if dif.abs() > 8.0 {
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

        // Stay at a given height until we take on an enemy
        /*		VectorSet( endPos, NPC->r.currentOrigin[0], NPC->r.currentOrigin[1], NPC->r.currentOrigin[2] - 512 );
        trap_Trace( &trace, NPC->r.currentOrigin, NULL, NULL, endPos, NPC->s.number, MASK_SOLID );

        if ( trace.fraction != 1.0f )
        {
            float	length = ( trace.fraction * 512 );

            if ( length < 80 )
            {
                ucmd.upmove = 32;
            }
            else if ( length > 120 )
            {
                ucmd.upmove = -32;
            }
            else
            {
                if ( NPC->client->ps.velocity[2] )
                {
                    NPC->client->ps.velocity[2] *= VELOCITY_DECAY;

                    if ( fabs( NPC->client->ps.velocity[2] ) < 1 )
                    {
                        NPC->client->ps.velocity[2] = 0;
                    }
                }
            }
        } */
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

/*
-------------------------
ImperialProbe_Strafe
-------------------------
*/
pub unsafe fn ImperialProbe_Strafe() {
    let dir: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let tr;

    AngleVectors(
        &(*(*NPC).client).renderInfo.eyeAngles,
        None,
        Some(&mut right),
        None,
    );

    // Pick a random strafe direction, then check to see if doing a strafe would be
    //	reasonable valid
    dir = if (rand() & 1) != 0 { -1 } else { 1 };
    VectorMA(
        &(*NPC).r.currentOrigin,
        HUNTER_STRAFE_DIS * dir as f32,
        &right,
        &mut end,
    );

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
        VectorMA(
            &vel,
            HUNTER_STRAFE_VEL * dir as f32,
            &right,
            &mut (*(*NPC).client).ps.velocity,
        );

        // Add a slight upward push
        (*(*NPC).client).ps.velocity[2] += HUNTER_UPWARD_PUSH;

        // Set the strafe start time so we can do a controlled roll
        //NPC->fx_time = level.time;
        (*NPCInfo).standTime = (*addr_of!(level)).time + 3000 + (random() * 500.0) as c_int;
    }
}

/*
-------------------------
ImperialProbe_Hunt
-------------------------`
*/

const HUNTER_FORWARD_BASE_SPEED: f32 = 10.0;
const HUNTER_FORWARD_MULTIPLIER: f32 = 5.0;

pub unsafe fn ImperialProbe_Hunt(visible: qboolean, advance: qboolean) {
    let mut distance: f32 = 0.0;
    let speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    NPC_SetAnim(
        NPC,
        SETANIM_BOTH,
        BOTH_RUN1,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );

    //If we're not supposed to stand still, pursue the player
    if (*NPCInfo).standTime < (*addr_of!(level)).time {
        // Only strafe when we can see the player
        if visible != QFALSE {
            ImperialProbe_Strafe();
            return;
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
        VectorSubtract(
            &(*(*NPC).enemy).r.currentOrigin,
            &(*NPC).r.currentOrigin,
            &mut forward,
        );
        distance = VectorNormalize(&mut forward);
    }
    let _ = distance;

    speed = HUNTER_FORWARD_BASE_SPEED
        + HUNTER_FORWARD_MULTIPLIER * (*addr_of!(g_spskill)).integer as f32;
    let curvel = (*(*NPC).client).ps.velocity;
    VectorMA(&curvel, speed, &forward, &mut (*(*NPC).client).ps.velocity);
}

/*
-------------------------
ImperialProbe_FireBlaster
-------------------------
*/
pub unsafe fn ImperialProbe_FireBlaster() {
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut delta1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    // C: static vec3_t forward, vright, up; static vec3_t muzzle; — written then
    // read within each call (the `static` storage is dead, the `muzzle` static is
    // never used), demoted to plain locals to match the Remote_Fire precedent.
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let genBolt1: c_int;
    let missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

    genBolt1 = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash");

    //FIXME: use {0, NPC->client->ps.legsYaw, 0}
    trap::G2API_GetBoltMatrix(
        (*NPC).ghoul2,
        0,
        genBolt1,
        &mut boltMatrix,
        &(*NPC).r.currentAngles,
        &(*NPC).r.currentOrigin,
        (*addr_of!(level)).time,
        null_mut(),
        &(*NPC).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle1);

    G_PlayEffectID(G_EffectIndex("bryar/muzzle_flash"), &muzzle1, &vec3_origin);

    G_Sound(NPC, CHAN_AUTO, G_SoundIndex("sound/chars/probe/misc/fire"));

    if (*NPC).health != 0 {
        CalcEntitySpot((*NPC).enemy, SPOT_CHEST, &mut enemy_org1);
        enemy_org1[0] += Q_irand(0, 10) as f32;
        enemy_org1[1] += Q_irand(0, 10) as f32;
        VectorSubtract(&enemy_org1, &muzzle1, &mut delta1);
        vectoangles(&delta1, &mut angleToEnemy1);
        AngleVectors(
            &angleToEnemy1,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    } else {
        AngleVectors(
            &(*NPC).r.currentAngles,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    }

    missile = CreateMissile(&mut muzzle1, &forward, 1600.0, 10000, NPC, QFALSE);

    (*missile).classname = c"bryar_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    if (*addr_of!(g_spskill)).integer <= 1 {
        (*missile).damage = 5;
    } else {
        (*missile).damage = 10;
    }

    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_UNKNOWN;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
ImperialProbe_Ranged
-------------------------
*/
pub unsafe fn ImperialProbe_Ranged(visible: qboolean, advance: qboolean) {
    let delay_min: c_int;
    let delay_max: c_int;

    if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE
    // Attack?
    {
        if (*addr_of!(g_spskill)).integer == 0 {
            delay_min = 500;
            delay_max = 3000;
        } else if (*addr_of!(g_spskill)).integer > 1 {
            delay_min = 500;
            delay_max = 2000;
        } else {
            delay_min = 300;
            delay_max = 1500;
        }
        let _ = (delay_min, delay_max);

        TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 3000));
        ImperialProbe_FireBlaster();
        //		ucmd.buttons |= BUTTON_ATTACK;
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        ImperialProbe_Hunt(visible, advance);
    }
}

/*
-------------------------
ImperialProbe_AttackDecision
-------------------------
*/

const MIN_MELEE_RANGE: c_int = 320;
#[allow(dead_code)]
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

pub unsafe fn ImperialProbe_AttackDecision() {
    let distance: f32;
    let visible: qboolean;
    let advance: qboolean;

    // Always keep a good height off the ground
    ImperialProbe_MaintainHeight();

    //randomly talk
    if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
        if TIMER_Done(NPC, c"angerNoise".as_ptr()) != QFALSE {
            G_SoundOnEnt(
                NPC,
                CHAN_AUTO,
                &format!("sound/chars/probe/misc/probetalk{}", Q_irand(1, 3)),
            );

            TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(4000, 10000));
        }
    }

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE {
        ImperialProbe_Idle();
        return;
    }

    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_RUN1, SETANIM_FLAG_NORMAL);

    // Rate our distance to the target, and our visibilty
    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
        as c_int as f32;
    visible = NPC_ClearLOS4((*NPC).enemy);
    advance = (distance > MIN_DISTANCE_SQR as f32) as qboolean;

    // If we cannot see our target, move to see it
    if visible == QFALSE {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            ImperialProbe_Hunt(visible, advance);
            return;
        }
    }

    // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
    NPC_FaceEnemy(QTRUE);

    // Decide what type of attack to do
    ImperialProbe_Ranged(visible, advance);
}

/*
-------------------------
NPC_BSDroid_Pain
-------------------------
*/
pub unsafe extern "C" fn NPC_Probe_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    let pain_chance: f32;
    let other: *mut gentity_t = attacker;
    let mod_: c_int = *addr_of!(gPainMOD);

    VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s.angles);

    if (*self_).health < 30 || mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT
    // demp2 always messes them up real good
    {
        let mut endPos: vec3_t = [0.0; 3];
        let trace;

        VectorSet(
            &mut endPos,
            (*self_).r.currentOrigin[0],
            (*self_).r.currentOrigin[1],
            (*self_).r.currentOrigin[2] - 128.0,
        );
        trace = trap::Trace(
            &(*self_).r.currentOrigin,
            &[0.0; 3],
            &[0.0; 3],
            &endPos,
            (*self_).s.number,
            MASK_SOLID,
        );

        if trace.fraction == 1.0 || mod_ == MOD_DEMP2
        // demp2 always does this
        {
            /*
                        if (self->client->clientInfo.headModel != 0)
                        {
                            vec3_t origin;

                            VectorCopy(self->r.currentOrigin,origin);
                            origin[2] +=50;
            //				G_PlayEffect( "small_chunks", origin );
                            G_PlayEffect( "chunks/probehead", origin );
                            G_PlayEffect( "env/med_explode2", origin );
                            self->client->clientInfo.headModel = 0;
                            self->client->moveType = MT_RUNJUMP;
                            self->client->ps.gravity = g_gravity->value*.1;
                        }
                        */

            if (mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT) && !other.is_null() {
                let mut dir: vec3_t = [0.0; 3];

                NPC_SetAnim(
                    self_,
                    SETANIM_BOTH,
                    BOTH_PAIN1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                VectorSubtract(
                    &(*self_).r.currentOrigin,
                    &(*other).r.currentOrigin,
                    &mut dir,
                );
                VectorNormalize(&mut dir);

                let vel = (*(*self_).client).ps.velocity;
                VectorMA(&vel, 550.0, &dir, &mut (*(*self_).client).ps.velocity);
                (*(*self_).client).ps.velocity[2] -= 127.0;
            }

            //self->s.powerups |= ( 1 << PW_SHOCKED );
            //self->client->ps.powerups[PW_SHOCKED] = level.time + 3000;
            (*(*self_).client).ps.electrifyTime = (*addr_of!(level)).time + 3000;

            (*(*self_).NPC).localState = LSTATE_DROP;
        }
    } else {
        pain_chance = NPC_GetPainChance(self_, damage);

        if random() < pain_chance
        // Spin around in pain?
        {
            NPC_SetAnim(self_, SETANIM_BOTH, BOTH_PAIN1, SETANIM_FLAG_OVERRIDE);
        }
    }

    NPC_Pain(self_, attacker, damage);
}

/*
-------------------------
ImperialProbe_Idle
-------------------------
*/
pub unsafe fn ImperialProbe_Idle() {
    ImperialProbe_MaintainHeight();

    NPC_BSIdle();
}

/*
-------------------------
NPC_BSImperialProbe_Patrol
-------------------------
*/
pub unsafe fn ImperialProbe_Patrol() {
    ImperialProbe_MaintainHeight();

    if NPC_CheckPlayerTeamStealth() != QFALSE {
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    //If we have somewhere to go, then do that
    if (*NPC).enemy.is_null() {
        NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_RUN1, SETANIM_FLAG_NORMAL);

        if !UpdateGoal().is_null() {
            //start loop sound once we move
            (*NPC).s.loopSound = G_SoundIndex("sound/chars/probe/misc/probedroidloop");
            ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(QTRUE);
        }
        //randomly talk
        if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
            G_SoundOnEnt(
                NPC,
                CHAN_AUTO,
                &format!("sound/chars/probe/misc/probetalk{}", Q_irand(1, 3)),
            );

            TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
        }
    } else
    // He's got an enemy. Make him angry.
    {
        G_SoundOnEnt(NPC, CHAN_AUTO, "sound/chars/probe/misc/anger1");
        TIMER_Set(NPC, c"angerNoise".as_ptr(), Q_irand(2000, 4000));
        //NPCInfo->behaviorState = BS_HUNT_AND_KILL;
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
ImperialProbe_Wait
-------------------------
*/
pub unsafe fn ImperialProbe_Wait() {
    if (*NPCInfo).localState == LSTATE_DROP {
        let mut endPos: vec3_t = [0.0; 3];
        let trace;

        (*NPCInfo).desiredYaw = AngleNormalize360((*NPCInfo).desiredYaw + 25.0);

        VectorSet(
            &mut endPos,
            (*NPC).r.currentOrigin[0],
            (*NPC).r.currentOrigin[1],
            (*NPC).r.currentOrigin[2] - 32.0,
        );
        trace = trap::Trace(
            &(*NPC).r.currentOrigin,
            &[0.0; 3],
            &[0.0; 3],
            &endPos,
            (*NPC).s.number,
            MASK_SOLID,
        );

        if trace.fraction != 1.0 {
            G_Damage(
                NPC,
                (*NPC).enemy,
                (*NPC).enemy,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                2000,
                0,
                MOD_UNKNOWN,
            );
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
NPC_BSImperialProbe_Default
-------------------------
*/
pub unsafe fn NPC_BSImperialProbe_Default() {
    if !(*NPC).enemy.is_null() {
        (*NPCInfo).goalEntity = (*NPC).enemy;
        ImperialProbe_AttackDecision();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        ImperialProbe_Patrol();
    } else if (*NPCInfo).localState == LSTATE_DROP {
        ImperialProbe_Wait();
    } else {
        ImperialProbe_Idle();
    }
}
