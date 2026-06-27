//! Port of `NPC_AI_Mark2.c` — the Mark II droid NPC's behavior state.
//!
//! This whole file ships LIVE in the PC tree (it is `#if 0`/dead only on the
//! Xbox tree), so it is fully in scope for the MP build. Drained bottom-up at
//! the leaf seam: precache, the per-part explode helper, the pain handler, then
//! the hunt/fire/attack-decision/patrol/idle think helpers and the master
//! `NPC_BSMark2_Default` think.
//!
//! Ported here: `NPC_Mark2_Precache` (NPC_AI_Mark2.c:27),
//! `NPC_Mark2_Part_Explode` (:50), `NPC_Mark2_Pain` (:80),
//! `Mark2_Hunt` (:118), `Mark2_FireBlaster` (:137),
//! `Mark2_BlasterAttack` (:186), `Mark2_AttackDecision` (:212),
//! `Mark2_Patrol` (:303), `Mark2_Idle` (:337),
//! `NPC_BSMark2_Default` (:347).

#![allow(non_snake_case)] // C function names (`NPC_BSMark2_Default`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::{c_char, c_int};
use core::ptr::addr_of;

use crate::codemp::game::anims::{BOTH_RUN1START, BOTH_RUN1STOP};
use crate::codemp::game::bg_misc::{BG_FindItemForAmmo, BG_FindItemForWeapon};
use crate::codemp::game::bg_public::BG_GiveMeVectorFromMatrix;
use crate::codemp::game::bg_weapons_h::{
    AMMO_BLASTER, AMMO_METAL_BOLTS, AMMO_POWERCELL, WP_BRYAR_PISTOL,
};
use crate::codemp::game::g_combat::{gPainHitLoc, G_Damage};
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{
    gentity_t, DAMAGE_DEATH_KNOCKBACK, DAMAGE_NO_PROTECTION, FL_SHIELDED, HL_GENERIC1,
};
use crate::codemp::game::g_main::level;
use crate::codemp::game::g_missile::CreateMissile;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_EffectIndex, G_PlayEffectID, G_Sound, G_SoundIndex};
use crate::codemp::game::npc::{ucmd, NPC_SetAnim, NPC, NPCInfo};
use crate::codemp::game::npc_ai_default::NPC_BSIdle;
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_utils::{
    CalcEntitySpot, NPC_ClearLOS4, NPC_FaceEnemy, NPC_SetSurfaceOnOff, NPC_UpdateAngles,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::codemp::game::b_public_h::{SCF_LOOK_FOR_ENEMIES, SPOT_HEAD};
use crate::codemp::game::q_math::{
    vectoangles, AngleVectors, DistanceHorizontalSquared, Q_irand, VectorSubtract,
};
use crate::codemp::game::q_shared::va;
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, vec3_t, BUTTON_WALKING, CHAN_AUTO, NEGATIVE_Y, ORIGIN,
};
use crate::codemp::game::bg_public::{
    MASK_SHOT, MOD_BRYAR_PISTOL, MOD_UNKNOWN, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE,
};
use crate::codemp::game::surfaceflags_h::CONTENTS_LIGHTSABER;
use crate::trap;

//#define AMMO_POD_HEALTH				40
const AMMO_POD_HEALTH: c_int = 1;
const TURN_OFF: c_int = 0x00000100;

#[allow(dead_code)]
const VELOCITY_DECAY: f32 = 0.25;
#[allow(dead_code)]
const MAX_DISTANCE: c_int = 256;
#[allow(dead_code)]
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;
#[allow(dead_code)]
const MIN_DISTANCE: c_int = 24;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

//Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_DROPPINGDOWN: c_int = 1;
const LSTATE_DOWN: c_int = 2;
const LSTATE_RISINGUP: c_int = 3;

pub unsafe fn NPC_Mark2_Precache() {
    G_SoundIndex("sound/chars/mark2/misc/mark2_explo"); // blows up on death
    G_SoundIndex("sound/chars/mark2/misc/mark2_pain");
    G_SoundIndex("sound/chars/mark2/misc/mark2_fire");
    G_SoundIndex("sound/chars/mark2/misc/mark2_move_lp");

    G_EffectIndex("explosions/droidexplosion1");
    G_EffectIndex("env/med_explode2");
    G_EffectIndex("blaster/smoke_bolton");
    G_EffectIndex("bryar/muzzle_flash");

    RegisterItem(BG_FindItemForWeapon(WP_BRYAR_PISTOL));
    RegisterItem(BG_FindItemForAmmo(AMMO_METAL_BOLTS));
    RegisterItem(BG_FindItemForAmmo(AMMO_POWERCELL));
    RegisterItem(BG_FindItemForAmmo(AMMO_BLASTER));
}

/*
-------------------------
NPC_Mark2_Part_Explode
-------------------------
*/
pub unsafe fn NPC_Mark2_Part_Explode(self_: *mut gentity_t, bolt: c_int) {
    if bolt >= 0 {
        let mut boltMatrix = mdxaBone_t::default();
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &(*self_).r.currentAngles,
            &(*self_).r.currentOrigin,
            (*addr_of!(level)).time,
            core::ptr::null_mut(),
            &(*self_).modelScale,
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut org);
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y, &mut dir);

        G_PlayEffectID(G_EffectIndex("env/med_explode2"), &org, &dir);
        G_PlayEffectID(G_EffectIndex("blaster/smoke_bolton"), &org, &dir);
    }

    //G_PlayEffectID( G_EffectIndex("blaster/smoke_bolton"), self->playerModel, bolt, self->s.number);

    (*self_).count += 1; // Count of pods blown off
}

/*
-------------------------
NPC_Mark2_Pain
- look at what was hit and see if it should be removed from the model.
-------------------------
*/
pub unsafe extern "C" fn NPC_Mark2_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    let newBolt: c_int;
    let mut i: c_int;
    let hitLoc: c_int = *addr_of!(gPainHitLoc);

    NPC_Pain(self_, attacker, damage);

    i = 0;
    while i < 3 {
        if (hitLoc == HL_GENERIC1 + i)
            && ((*self_).locationDamage[(HL_GENERIC1 + i) as usize] > AMMO_POD_HEALTH)
        // Blow it up?
        {
            if (*self_).locationDamage[hitLoc as usize] >= AMMO_POD_HEALTH {
                newBolt =
                    trap::G2API_AddBolt((*self_).ghoul2, 0, &format!("torso_canister{}", i + 1));
                if newBolt != -1 {
                    NPC_Mark2_Part_Explode(self_, newBolt);
                }
                NPC_SetSurfaceOnOff(self_, va(format_args!("torso_canister{}", i + 1)), TURN_OFF);
                break;
            }
        }
        i += 1;
    }

    G_Sound(
        self_,
        CHAN_AUTO,
        G_SoundIndex("sound/chars/mark2/misc/mark2_pain"),
    );

    // If any pods were blown off, kill him
    if (*self_).count > 0 {
        G_Damage(
            self_,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (*self_).health,
            DAMAGE_NO_PROTECTION,
            MOD_UNKNOWN,
        );
    }
}

/*
-------------------------
Mark2_Hunt
-------------------------
*/
pub unsafe fn Mark2_Hunt() {
    if (*NPCInfo).goalEntity.is_null() {
        (*NPCInfo).goalEntity = (*NPC).enemy;
    }

    // Turn toward him before moving towards him.
    NPC_FaceEnemy(QTRUE);

    (*NPCInfo).combatMove = QTRUE;
    NPC_MoveToGoal(QTRUE);
}

/*
-------------------------
Mark2_FireBlaster
-------------------------
*/
pub unsafe fn Mark2_FireBlaster(_advance: qboolean) {
    let mut muzzle1: vec3_t = [0.0; 3];
    // C declares these `static`, but both code paths overwrite them every call,
    // so plain locals are faithful (cf. NPC_AI_Sentry's Sentry_Fire). The unused
    // `static vec3_t muzzle` C local is dropped.
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let missile: *mut gentity_t;
    let mut boltMatrix = mdxaBone_t::default();
    let bolt: c_int = trap::G2API_AddBolt((*NPC).ghoul2, 0, "*flash");

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

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle1);

    if (*NPC).health != 0 {
        let mut enemy_org1: vec3_t = [0.0; 3];
        let mut delta1: vec3_t = [0.0; 3];
        let mut angleToEnemy1: vec3_t = [0.0; 3];
        CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut enemy_org1);
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

    G_PlayEffectID(G_EffectIndex("bryar/muzzle_flash"), &muzzle1, &forward);

    G_Sound(
        NPC,
        CHAN_AUTO,
        G_SoundIndex("sound/chars/mark2/misc/mark2_fire"),
    );

    missile = CreateMissile(&mut muzzle1, &forward, 1600.0, 10000, NPC, QFALSE);

    (*missile).classname = c"bryar_proj".as_ptr() as *mut c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    (*missile).damage = 1;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_BRYAR_PISTOL;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
Mark2_BlasterAttack
-------------------------
*/
pub unsafe fn Mark2_BlasterAttack(advance: qboolean) {
    if TIMER_Done(NPC, c"attackDelay".as_ptr()) != QFALSE {
        // Attack?
        if (*NPCInfo).localState == LSTATE_NONE {
            // He's up so shoot less often.
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(500, 2000));
        } else {
            TIMER_Set(NPC, c"attackDelay".as_ptr(), Q_irand(100, 500));
        }
        Mark2_FireBlaster(advance);
        return;
    } else if advance != QFALSE {
        Mark2_Hunt();
    }
}

/*
-------------------------
Mark2_AttackDecision
-------------------------
*/
pub unsafe fn Mark2_AttackDecision() {
    let distance: f32;
    let visible: qboolean;
    let advance: qboolean;

    NPC_FaceEnemy(QTRUE);

    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
        as c_int as f32;
    visible = NPC_ClearLOS4((*NPC).enemy);
    advance = if distance > MIN_DISTANCE_SQR as f32 {
        QTRUE
    } else {
        QFALSE
    };

    // He's been ordered to get up
    if (*NPCInfo).localState == LSTATE_RISINGUP {
        (*NPC).flags &= !FL_SHIELDED;
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_RUN1START,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        if ((*(*NPC).client).ps.legsTimer <= 0)
            && (*(*NPC).client).ps.torsoAnim == BOTH_RUN1START
        {
            (*NPCInfo).localState = LSTATE_NONE; // He's up again.
        }
        return;
    }

    // If we cannot see our target, move to see it
    if (visible == QFALSE) || (NPC_FaceEnemy(QTRUE) == QFALSE) {
        // If he's going down or is down, make him get up
        if ((*NPCInfo).localState == LSTATE_DOWN) || ((*NPCInfo).localState == LSTATE_DROPPINGDOWN)
        {
            if TIMER_Done(NPC, c"downTime".as_ptr()) != QFALSE {
                // Down being down?? (The delay is so he doesn't pop up and down when the player goes in and out of range)
                (*NPCInfo).localState = LSTATE_RISINGUP;
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_RUN1STOP,
                    SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
                );
                TIMER_Set(NPC, c"runTime".as_ptr(), Q_irand(3000, 8000)); // So he runs for a while before testing to see if he should drop down.
            }
        } else {
            Mark2_Hunt();
        }
        return;
    }

    // He's down but he could advance if he wants to.
    if (advance != QFALSE)
        && (TIMER_Done(NPC, c"downTime".as_ptr()) != QFALSE)
        && ((*NPCInfo).localState == LSTATE_DOWN)
    {
        (*NPCInfo).localState = LSTATE_RISINGUP;
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_RUN1STOP,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        TIMER_Set(NPC, c"runTime".as_ptr(), Q_irand(3000, 8000)); // So he runs for a while before testing to see if he should drop down.
    }

    NPC_FaceEnemy(QTRUE);

    // Dropping down to shoot
    if (*NPCInfo).localState == LSTATE_DROPPINGDOWN {
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_RUN1STOP,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        TIMER_Set(NPC, c"downTime".as_ptr(), Q_irand(3000, 9000));

        if ((*(*NPC).client).ps.legsTimer <= 0) && (*(*NPC).client).ps.torsoAnim == BOTH_RUN1STOP {
            (*NPC).flags |= FL_SHIELDED;
            (*NPCInfo).localState = LSTATE_DOWN;
        }
    }
    // He's down and shooting
    else if (*NPCInfo).localState == LSTATE_DOWN {
        (*NPC).flags |= FL_SHIELDED; //only damagable by lightsabers and missiles

        Mark2_BlasterAttack(QFALSE);
    } else if TIMER_Done(NPC, c"runTime".as_ptr()) != QFALSE {
        // Lowering down to attack. But only if he's done running at you.
        (*NPCInfo).localState = LSTATE_DROPPINGDOWN;
    } else if advance != QFALSE {
        // We can see enemy so shoot him if timer lets you.
        Mark2_BlasterAttack(advance);
    }
}

/*
-------------------------
Mark2_Patrol
-------------------------
*/
pub unsafe fn Mark2_Patrol() {
    if NPC_CheckPlayerTeamStealth() != QFALSE {
        //		G_Sound( NPC, G_SoundIndex("sound/chars/mark1/misc/anger.wav"));
        NPC_UpdateAngles(QTRUE, QTRUE);
        return;
    }

    //If we have somewhere to go, then do that
    if (*NPC).enemy.is_null() {
        if !UpdateGoal().is_null() {
            ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(QTRUE);
            NPC_UpdateAngles(QTRUE, QTRUE);
        }

        //randomly talk
        if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
            //			G_Sound( NPC, G_SoundIndex(va("sound/chars/mark1/misc/talk%d.wav",	Q_irand(1, 4))));

            TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
        }
    }
}

/*
-------------------------
Mark2_Idle
-------------------------
*/
pub unsafe fn Mark2_Idle() {
    NPC_BSIdle();
}

/*
-------------------------
NPC_BSMark2_Default
-------------------------
*/
pub unsafe fn NPC_BSMark2_Default() {
    if !(*NPC).enemy.is_null() {
        (*NPCInfo).goalEntity = (*NPC).enemy;
        Mark2_AttackDecision();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        Mark2_Patrol();
    } else {
        Mark2_Idle();
    }
}
