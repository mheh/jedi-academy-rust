//! Slice of `NPC_AI_Sentry.c` — the hovering sentry droid's behavior state.
//! Fully ported: the precache, the use/pain handlers, the maintain-height /
//! strafe / hunt / patrol routines, and the fire / idle / ranged / attack-decision
//! chain. The Ghoul2 bolt API (`trap_G2API_AddBolt` / `trap_G2API_GetBoltMatrix`)
//! and the NPC-AI core (`NPC_BSIdle`) have since landed, so the whole behavior
//! file is here.
//!
//! Ported: `NPC_Sentry_Precache` (NPC_AI_Sentry.c:37),
//! `sentry_use` (:64), `NPC_Sentry_Pain` (:79),
//! `Sentry_Fire` (:112), `Sentry_MaintainHeight` (:210),
//! `Sentry_Idle` (:311), `Sentry_Strafe` (:338), `Sentry_Hunt` (:372),
//! `Sentry_RangedAttack` (:418), `Sentry_AttackDecision` (:455),
//! `NPC_Sentry_Patrol` (:519), `NPC_BSSentry_Default` (:557).

#![allow(non_snake_case)] // C function names (`NPC_Sentry_Precache`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::addr_of;

use crate::codemp::game::anims::{BOTH_ATTACK1, BOTH_FLY_SHIELDED, BOTH_POWERUP1, BOTH_SLEEP1};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_misc::{BG_FindItemForAmmo, BG_GiveMeVectorFromMatrix};
use crate::codemp::game::bg_public::{MASK_SHOT, MASK_SOLID, MOD_BRYAR_PISTOL, MOD_DEMP2,
    MOD_DEMP2_ALT, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE};
use crate::codemp::game::bg_weapons_h::{AMMO_BLASTER, WP_BRYAR_PISTOL};
use crate::codemp::game::b_public_h::{SCF_CHASE_ENEMIES, SCF_LOOK_FOR_ENEMIES};
use crate::codemp::game::g_combat::gPainMOD;
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_DEATH_KNOCKBACK, FL_SHIELDED};
use crate::codemp::game::g_main::{g_spskill, level};
use crate::codemp::game::g_missile::CreateMissile;
use crate::codemp::game::g_public_h::BSET_USE;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{
    G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex, G_SoundOnEnt,
};
use crate::codemp::game::npc::{ucmd, NPC_SetAnim, NPCInfo, NPC};
use crate::codemp::game::npc_ai_default::NPC_BSIdle;
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::{NPC_GetMoveDirection, NPC_MoveToGoal};
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_utils::{
    G_ActivateBehavior, NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
use crate::codemp::game::q_math::{AngleVectors, DistanceHorizontalSquared, VectorMA,
    VectorNormalize, VectorSubtract};
use crate::codemp::game::q_shared::{random};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{mdxaBone_t, vec3_t, BUTTON_WALKING, CHAN_AUTO, ORIGIN};
use crate::codemp::game::surfaceflags_h::CONTENTS_LIGHTSABER;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

const MIN_DISTANCE: c_int = 256;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const SENTRY_FORWARD_BASE_SPEED: f32 = 10.0;
const SENTRY_FORWARD_MULTIPLIER: f32 = 5.0;

const SENTRY_VELOCITY_DECAY: f32 = 0.85;
const SENTRY_STRAFE_VEL: f32 = 256.0;
const SENTRY_STRAFE_DIS: f32 = 200.0;
const SENTRY_UPWARD_PUSH: f32 = 32.0;
const SENTRY_HOVER_HEIGHT: f32 = 24.0;

// Local state enums
const LSTATE_NONE: c_int = 0;
#[allow(dead_code)]
const LSTATE_ASLEEP: c_int = 1;
const LSTATE_WAKEUP: c_int = 2;
const LSTATE_ACTIVE: c_int = 3;
const LSTATE_POWERING_UP: c_int = 4;
const LSTATE_ATTACKING: c_int = 5;

const _: () = assert!(LSTATE_NONE == 0); // silence unused-const lint while keeping the enum verbatim

/*
-------------------------
NPC_Sentry_Precache
-------------------------
*/
pub unsafe fn NPC_Sentry_Precache() {
    let mut i: c_int;

    G_SoundIndex("sound/chars/sentry/misc/sentry_explo");
    G_SoundIndex("sound/chars/sentry/misc/sentry_pain");
    G_SoundIndex("sound/chars/sentry/misc/sentry_shield_open");
    G_SoundIndex("sound/chars/sentry/misc/sentry_shield_close");
    G_SoundIndex("sound/chars/sentry/misc/sentry_hover_1_lp");
    G_SoundIndex("sound/chars/sentry/misc/sentry_hover_2_lp");

    i = 1;
    while i < 4 {
        G_SoundIndex(&format!("sound/chars/sentry/misc/talk{}", i));
        i += 1;
    }

    G_EffectIndex("bryar/muzzle_flash");
    G_EffectIndex("env/med_explode");

    RegisterItem(BG_FindItemForAmmo(AMMO_BLASTER));
}

/*
================
sentry_use
================
*/
pub unsafe extern "C" fn sentry_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    G_ActivateBehavior(self_, BSET_USE);

    (*self_).flags &= !FL_SHIELDED;
    NPC_SetAnim(
        self_,
        SETANIM_BOTH,
        BOTH_POWERUP1,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
    );
    //	self->NPC->localState = LSTATE_WAKEUP;
    (*(*self_).NPC).localState = LSTATE_ACTIVE;
}

/*
-------------------------
NPC_Sentry_Pain
-------------------------
*/
pub unsafe extern "C" fn NPC_Sentry_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    let mod_: c_int = *addr_of!(gPainMOD);

    NPC_Pain(self_, attacker, damage);

    if mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT {
        (*(*self_).NPC).burstCount = 0;
        TIMER_Set(self_, c"attackDelay".as_ptr(), Q_irand(9000, 12000));
        (*self_).flags |= FL_SHIELDED;
        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            BOTH_FLY_SHIELDED,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        G_Sound(
            self_,
            CHAN_AUTO,
            G_SoundIndex("sound/chars/sentry/misc/sentry_pain"),
        );

        (*(*self_).NPC).localState = LSTATE_ACTIVE;
    }

    // You got hit, go after the enemy
    //	if (self->NPC->localState == LSTATE_ASLEEP)
    //	{
    //		G_Sound( self, G_SoundIndex("sound/chars/sentry/misc/shieldsopen.wav"));
    //
    //		self->flags &= ~FL_SHIELDED;
    //		NPC_SetAnim( self, SETANIM_BOTH, BOTH_POWERUP1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
    //		self->NPC->localState = LSTATE_WAKEUP;
    //	}
}

/*
-------------------------
Sentry_Fire
-------------------------
*/
pub unsafe fn Sentry_Fire() {
    let mut muzzle: vec3_t = [0.0; 3];
    // C declares `static vec3_t forward, vright, up;` but the static storage is
    // dead (recomputed via AngleVectors every call) — plain locals, matching the
    // `Remote_Fire`/`ImperialProbe_FireBlaster` precedent (avoids static_mut_refs).
    let mut FORWARD: vec3_t = [0.0; 3];
    let mut VRIGHT: vec3_t = [0.0; 3];
    let mut UP: vec3_t = [0.0; 3];
    let missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let bolt: c_int;
    let which: c_int;

    (*NPC).flags &= !FL_SHIELDED;

    if (*NPCInfo).localState == LSTATE_POWERING_UP {
        if TIMER_Done(NPC, c"powerup".as_ptr()) != QFALSE {
            (*NPCInfo).localState = LSTATE_ATTACKING;
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_ATTACK1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        } else {
            // can't do anything right now
            return;
        }
    } else if (*NPCInfo).localState == LSTATE_ACTIVE {
        (*NPCInfo).localState = LSTATE_POWERING_UP;

        G_Sound(
            NPC,
            CHAN_AUTO,
            G_SoundIndex("sound/chars/sentry/misc/sentry_shield_open"),
        );
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_POWERUP1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        TIMER_Set(NPC, c"powerup".as_ptr(), 250);
        return;
    } else if (*NPCInfo).localState != LSTATE_ATTACKING {
        // bad because we are uninitialized
        (*NPCInfo).localState = LSTATE_ACTIVE;
        return;
    }

    // Which muzzle to fire from?
    which = (*NPCInfo).burstCount % 3;
    match which {
        0 => {
            bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash1");
        }
        1 => {
            bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash2");
        }
        2 | _ => {
            bolt = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash03");
        }
    }

    trap::G2API_GetBoltMatrix(
        (*NPC).ghoul2,
        0,
        bolt,
        &mut boltMatrix,
        &(*NPC).r.currentAngles,
        &(*NPC).r.currentOrigin,
        (*addr_of!(level)).time,
        core::ptr::null_mut(),
        &(*NPC).modelScale,
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle);

    AngleVectors(
        &(*NPC).r.currentAngles,
        Some(&mut FORWARD),
        Some(&mut VRIGHT),
        Some(&mut UP),
    );
    //	G_Sound( NPC, G_SoundIndex("sound/chars/sentry/misc/shoot.wav"));

    G_PlayEffectID(G_EffectIndex("bryar/muzzle_flash"), &muzzle, &FORWARD);

    missile = CreateMissile(&mut muzzle, &FORWARD, 1600.0, 10000, NPC, QFALSE);

    (*missile).classname = c"bryar_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    (*NPCInfo).burstCount += 1;
    (*NPC).attackDebounceTime = (*addr_of!(level)).time + 50;
    (*missile).damage = 5;

    // now scale for difficulty
    if (*addr_of!(g_spskill)).integer == 0 {
        (*NPC).attackDebounceTime += 200;
        (*missile).damage = 1;
    } else if (*addr_of!(g_spskill)).integer == 1 {
        (*NPC).attackDebounceTime += 100;
        (*missile).damage = 3;
    }
}

/*
-------------------------
Sentry_MaintainHeight
-------------------------
*/
pub unsafe fn Sentry_MaintainHeight() {
    let mut dif: f32;

    (*NPC).s.loopSound = G_SoundIndex("sound/chars/sentry/misc/sentry_hover_1_lp");

    // Update our angles regardless
    NPC_UpdateAngles(QTRUE, QTRUE);

    // If we have an enemy, we should try to hover at about enemy eye level
    if !(*NPC).enemy.is_null() {
        // Find the height difference
        dif = ((*(*NPC).enemy).r.currentOrigin[2] + (*(*NPC).enemy).r.maxs[2])
            - (*NPC).r.currentOrigin[2];

        // cap to prevent dramatic height shifts
        if dif.abs() > 8.0 {
            if dif.abs() > SENTRY_HOVER_HEIGHT {
                dif = if dif < 0.0 { -24.0 } else { 24.0 };
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

            if dif.abs() > SENTRY_HOVER_HEIGHT {
                ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
            } else if (*(*NPC).client).ps.velocity[2] != 0.0 {
                (*(*NPC).client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

                if (*(*NPC).client).ps.velocity[2].abs() < 2.0 {
                    (*(*NPC).client).ps.velocity[2] = 0.0;
                }
            }
        }
        // Apply friction to Z
        else if (*(*NPC).client).ps.velocity[2] != 0.0 {
            (*(*NPC).client).ps.velocity[2] *= SENTRY_VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[2].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[2] = 0.0;
            }
        }
    }

    // Apply friction
    if (*(*NPC).client).ps.velocity[0] != 0.0 {
        (*(*NPC).client).ps.velocity[0] *= SENTRY_VELOCITY_DECAY;

        if (*(*NPC).client).ps.velocity[0].abs() < 1.0 {
            (*(*NPC).client).ps.velocity[0] = 0.0;
        }
    }

    if (*(*NPC).client).ps.velocity[1] != 0.0 {
        (*(*NPC).client).ps.velocity[1] *= SENTRY_VELOCITY_DECAY;

        if (*(*NPC).client).ps.velocity[1].abs() < 1.0 {
            (*(*NPC).client).ps.velocity[1] = 0.0;
        }
    }

    NPC_FaceEnemy(QTRUE);
}

/*
-------------------------
Sentry_Idle
-------------------------
*/
pub unsafe fn Sentry_Idle() {
    Sentry_MaintainHeight();

    // Is he waking up?
    if (*NPCInfo).localState == LSTATE_WAKEUP {
        if (*(*NPC).client).ps.torsoTimer <= 0 {
            (*NPCInfo).scriptFlags |= SCF_LOOK_FOR_ENEMIES;
            (*NPCInfo).burstCount = 0;
        }
    } else {
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_SLEEP1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        (*NPC).flags |= FL_SHIELDED;

        NPC_BSIdle();
    }
}

/*
-------------------------
Sentry_Strafe
-------------------------
*/
pub unsafe fn Sentry_Strafe() {
    let dir: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let tr;

    AngleVectors(&(*(*NPC).client).renderInfo.eyeAngles, None, Some(&mut right), None);

    // Pick a random strafe direction, then check to see if doing a strafe would be
    //	reasonable valid
    dir = if (rand() & 1) != 0 { -1 } else { 1 };
    VectorMA(&(*NPC).r.currentOrigin, SENTRY_STRAFE_DIS * dir as f32, &right, &mut end);

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
        VectorMA(&vel, SENTRY_STRAFE_VEL * dir as f32, &right, &mut (*(*NPC).client).ps.velocity);

        // Add a slight upward push
        (*(*NPC).client).ps.velocity[2] += SENTRY_UPWARD_PUSH;

        // Set the strafe start time so we can do a controlled roll
        //	NPC->fx_time = level.time;
        (*NPCInfo).standTime = (*addr_of!(level)).time + 3000 + (random() * 500.0) as c_int;
    }
}

/*
-------------------------
Sentry_Hunt
-------------------------
*/
pub unsafe fn Sentry_Hunt(visible: qboolean, advance: qboolean) {
    let mut distance: f32 = 0.0;
    let speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    //If we're not supposed to stand still, pursue the player
    if (*NPCInfo).standTime < (*addr_of!(level)).time {
        // Only strafe when we can see the player
        if visible != QFALSE {
            Sentry_Strafe();
            return;
        }
    }

    //If we don't want to advance, stop here
    if advance == QFALSE && visible != QFALSE {
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

    speed = SENTRY_FORWARD_BASE_SPEED
        + SENTRY_FORWARD_MULTIPLIER * (*addr_of!(g_spskill)).integer as f32;
    let curvel = (*(*NPC).client).ps.velocity;
    VectorMA(&curvel, speed, &forward, &mut (*(*NPC).client).ps.velocity);
}

/*
-------------------------
Sentry_RangedAttack
-------------------------
*/
pub unsafe fn Sentry_RangedAttack(visible: qboolean, advance: qboolean) {
    if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE
        && (*NPC).attackDebounceTime < (*addr_of!(level)).time
        && visible != QFALSE
    {
        // Attack?
        if (*NPCInfo).burstCount > 6 {
            if (*NPC).fly_sound_debounce_time == 0 {
                //delay closing down to give the player an opening
                (*NPC).fly_sound_debounce_time = (*addr_of!(level)).time + Q_irand(500, 2000);
            } else if (*NPC).fly_sound_debounce_time < (*addr_of!(level)).time {
                (*NPCInfo).localState = LSTATE_ACTIVE;
                (*NPC).fly_sound_debounce_time = 0;
                (*NPCInfo).burstCount = 0;
                TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(2000, 3500));
                (*NPC).flags |= FL_SHIELDED;
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_FLY_SHIELDED,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                G_SoundOnEnt(NPC, CHAN_AUTO, "sound/chars/sentry/misc/sentry_shield_close");
            }
        } else {
            Sentry_Fire();
        }
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        Sentry_Hunt(visible, advance);
    }
}

/*
-------------------------
Sentry_AttackDecision
-------------------------
*/
pub unsafe fn Sentry_AttackDecision() {
    let distance: f32;
    let visible: qboolean;
    let advance: qboolean;

    // Always keep a good height off the ground
    Sentry_MaintainHeight();

    (*NPC).s.loopSound = G_SoundIndex("sound/chars/sentry/misc/sentry_hover_2_lp");

    //randomly talk
    if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE
        && TIMER_Done(NPC, c"angerNoise".as_ptr()) != QFALSE
    {
        G_SoundOnEnt(
            NPC,
            CHAN_AUTO,
            &format!("sound/chars/sentry/misc/talk{}", Q_irand(1, 3)),
        );

        TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(4000, 10000));
    }

    // He's dead.
    if (*(*NPC).enemy).health < 1 {
        (*NPC).enemy = core::ptr::null_mut();
        Sentry_Idle();
        return;
    }

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt(QFALSE) == QFALSE {
        Sentry_Idle();
        return;
    }

    // Rate our distance to the target and visibilty
    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
        as c_int as f32;
    visible = NPC_ClearLOS4((*NPC).enemy);
    advance = if distance > MIN_DISTANCE_SQR as f32 {
        QTRUE
    } else {
        QFALSE
    };

    // If we cannot see our target, move to see it
    if visible == QFALSE && (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        Sentry_Hunt(visible, advance);
        return;
    }

    NPC_FaceEnemy(QTRUE);

    Sentry_RangedAttack(visible, advance);
}

/*
-------------------------
NPC_Sentry_Patrol
-------------------------
*/
pub unsafe fn NPC_Sentry_Patrol() {
    Sentry_MaintainHeight();

    //If we have somewhere to go, then do that
    if (*NPC).enemy.is_null() {
        if NPC_CheckPlayerTeamStealth() != QFALSE {
            //NPC_AngerSound();
            NPC_UpdateAngles(QTRUE, QTRUE);
            return;
        }

        if !UpdateGoal().is_null() {
            //start loop sound once we move
            ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(QTRUE);
        }

        //randomly talk
        if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
            G_SoundOnEnt(
                NPC,
                CHAN_AUTO,
                &format!("sound/chars/sentry/misc/talk{}", Q_irand(1, 3)),
            );

            TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
NPC_BSSentry_Default
-------------------------
*/
pub unsafe fn NPC_BSSentry_Default() {
    if !(*NPC).targetname.is_null() {
        (*NPC).r#use = Some(sentry_use);
    }

    if !(*NPC).enemy.is_null() && (*NPCInfo).localState != LSTATE_WAKEUP {
        // Don't attack if waking up or if no enemy
        Sentry_AttackDecision();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        NPC_Sentry_Patrol();
    } else {
        Sentry_Idle();
    }
}
