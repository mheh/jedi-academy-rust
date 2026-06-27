//! Port of `NPC_AI_Atst.c` — the AT-ST walker NPC's behavior state.
//!
//! In the PC tree (`refs/raven-jediacademy`) the bulk of this file is **live**,
//! not `#if 0` (the all-dead Xbox tree is a different source). Only one helper,
//! `ATST_PlayEffect`, sits inside an `#if 0` block (out of scope), plus the
//! commented-out arm-blowoff body inside `G_ATSTCheckPain`.
//!
//! Ported here: `G_ATSTCheckPain` (NPC_AI_Atst.c:68), `NPC_ATST_Pain` (:117),
//! `NPC_ATST_Precache` (:20), `ATST_Hunt` (:128), `ATST_Ranged` (:147),
//! `ATST_Attack` (:178), `ATST_Patrol` (:266), `ATST_Idle` (:289),
//! `NPC_BSATST_Default` (:300).

#![allow(non_snake_case)] // C function names (`G_ATSTCheckPain`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define`/enum constants kept verbatim

use core::ffi::c_int;

use crate::codemp::game::ai_h::{DIST_LONG, DIST_MELEE};
use crate::codemp::game::anims::BOTH_STAND1;
use crate::codemp::game::b_public_h::{SCF_CHASE_ENEMIES, SCF_LOOK_FOR_ENEMIES};
use crate::codemp::game::bg_lib::rand;
use crate::codemp::game::bg_misc::BG_FindItemForWeapon;
use crate::codemp::game::bg_public::{SETANIM_BOTH, SETANIM_FLAG_NORMAL};
use crate::codemp::game::bg_weapons_h::{WP_BOWCASTER, WP_NONE, WP_ROCKET_LAUNCHER};
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_EffectIndex, G_SoundIndex, G_SoundOnEnt};
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC_SetAnim, NPC};
use crate::codemp::game::npc_ai_default::NPC_BSIdle;
use crate::codemp::game::npc_ai_stormtrooper::NPC_CheckPlayerTeamStealth;
use crate::codemp::game::npc_combat::NPC_ChangeWeapon;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_reactions::NPC_Pain;
use crate::codemp::game::npc_utils::{NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy,
    NPC_UpdateAngles};
use crate::codemp::game::q_math::{DistanceHorizontalSquared, Q_irand};
use crate::codemp::game::q_shared_h::{BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_WALKING,
    CHAN_LESS_ATTEN};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// Local `#define`s from NPC_AI_Atst.c top (distance thresholds + the G2 surface
// "turn off" flag). `distance_e` (DIST_MELEE/DIST_LONG) lives in ai.h, imported above.
const MIN_MELEE_RANGE: c_int = 640;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const TURN_OFF: c_int = 0x00000100; // G2SURFACEFLAG_NODESCENDANTS

/*
-------------------------
NPC_ATST_Precache
-------------------------
*/
pub unsafe fn NPC_ATST_Precache() {
    G_SoundIndex("sound/chars/atst/atst_damaged1");
    G_SoundIndex("sound/chars/atst/atst_damaged2");

    //	RegisterItem( BG_FindItemForWeapon( WP_ATST_MAIN ));	//precache the weapon
    //rwwFIXMEFIXME: add this weapon
    RegisterItem(BG_FindItemForWeapon(WP_BOWCASTER)); //precache the weapon
    RegisterItem(BG_FindItemForWeapon(WP_ROCKET_LAUNCHER)); //precache the weapon

    G_EffectIndex("env/med_explode2");
    //	G_EffectIndex( "smaller_chunks" );
    G_EffectIndex("blaster/smoke_bolton");
    G_EffectIndex("explosions/droidexplosion1");
}

//-----------------------------------------------------------------
// `ATST_PlayEffect` (NPC_AI_Atst.c) sits inside an `#if 0` block — out of scope.

/*
-------------------------
G_ATSTCheckPain

Called by NPC's and player in an ATST
-------------------------
*/

pub unsafe fn G_ATSTCheckPain(self_: *mut gentity_t, _other: *mut gentity_t, _damage: c_int) {
    //int newBolt;
    //int hitLoc = gPainHitLoc;

    if rand() & 1 != 0 {
        G_SoundOnEnt(self_, CHAN_LESS_ATTEN, "sound/chars/atst/atst_damaged1");
    } else {
        G_SoundOnEnt(self_, CHAN_LESS_ATTEN, "sound/chars/atst/atst_damaged2");
    }

    // The arm-blowoff body (HL_ARM_LT/HL_ARM_RT, trap_G2API_AddBolt,
    // ATST_PlayEffect, NPC_SetSurfaceOnOff) is commented out in the C source.
}

/*
-------------------------
NPC_ATST_Pain
-------------------------
*/
pub unsafe extern "C" fn NPC_ATST_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    G_ATSTCheckPain(self_, attacker, damage);
    NPC_Pain(self_, attacker, damage);
}

/*
-------------------------
ATST_Hunt
-------------------------
*/
pub unsafe fn ATST_Hunt(_visible: qboolean, _advance: qboolean) {
    if (*NPCInfo).goalEntity.is_null() {
        //hunt
        (*NPCInfo).goalEntity = (*NPC).enemy;
    }

    (*NPCInfo).combatMove = QTRUE;

    NPC_MoveToGoal(QTRUE);
}

/*
-------------------------
ATST_Ranged
-------------------------
*/
pub unsafe fn ATST_Ranged(visible: qboolean, advance: qboolean, altAttack: qboolean) {
    if TIMER_Done(NPC, c"atkDelay".as_ptr()) != QFALSE && visible != QFALSE
    // Attack?
    {
        TIMER_Set(NPC, c"atkDelay".as_ptr(), Q_irand(500, 3000));

        if altAttack != QFALSE {
            ucmd.buttons |= BUTTON_ATTACK | BUTTON_ALT_ATTACK;
        } else {
            ucmd.buttons |= BUTTON_ATTACK;
        }
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        ATST_Hunt(visible, advance);
    }
}

/*
-------------------------
ATST_Patrol
-------------------------
*/
pub unsafe fn ATST_Patrol() {
    if NPC_CheckPlayerTeamStealth() != QFALSE {
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
    }
}

/*
-------------------------
ATST_Idle
-------------------------
*/
pub unsafe fn ATST_Idle() {
    NPC_BSIdle();

    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_STAND1, SETANIM_FLAG_NORMAL);
}

/*
-------------------------
ATST_Attack
-------------------------
*/
pub unsafe fn ATST_Attack() {
    let mut altAttack: qboolean = QFALSE;
    let blasterTest: c_int;
    let chargerTest: c_int;
    let weapon: c_int;
    let distance: f32;
    let distRate: c_int;
    let visible: qboolean;
    let advance: qboolean;

    if NPC_CheckEnemyExt(QFALSE) == QFALSE
    // !NPC->enemy )//
    {
        (*NPC).enemy = core::ptr::null_mut();
        return;
    }

    NPC_FaceEnemy(QTRUE);

    // Rate our distance to the target, and our visibilty
    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin)
        as c_int as f32;
    distRate = if distance > MIN_MELEE_RANGE_SQR as f32 {
        DIST_LONG
    } else {
        DIST_MELEE
    };
    visible = NPC_ClearLOS4((*NPC).enemy);
    advance = if distance > MIN_DISTANCE_SQR as f32 {
        QTRUE
    } else {
        QFALSE
    };

    // If we cannot see our target, move to see it
    if visible == QFALSE {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            ATST_Hunt(visible, advance);
            return;
        }
    }

    // Decide what type of attack to do
    match distRate {
        DIST_MELEE => {
            //		NPC_ChangeWeapon( WP_ATST_MAIN );
        }

        DIST_LONG => {
            //		NPC_ChangeWeapon( WP_ATST_SIDE );
            //rwwFIXMEFIXME: make atst weaps work.

            // See if the side weapons are there
            blasterTest =
                trap::G2API_GetSurfaceRenderStatus((*NPC).ghoul2, 0, c"head_light_blaster_cann".as_ptr());
            chargerTest =
                trap::G2API_GetSurfaceRenderStatus((*NPC).ghoul2, 0, c"head_concussion_charger".as_ptr());

            // It has both side weapons
            if blasterTest != -1
                && (blasterTest & TURN_OFF) == 0
                && chargerTest != -1
                && (chargerTest & TURN_OFF) == 0
            {
                weapon = Q_irand(0, 1); // 0 is blaster, 1 is charger (ALT SIDE)

                if weapon != 0 {
                    // Fire charger
                    altAttack = QTRUE;
                } else {
                    altAttack = QFALSE;
                }
            } else if blasterTest != -1 && (blasterTest & TURN_OFF) == 0 {
                // Blaster is on
                altAttack = QFALSE;
            } else if chargerTest != -1 && (chargerTest & TURN_OFF) == 0 {
                // Blaster is on
                altAttack = QTRUE;
            } else {
                NPC_ChangeWeapon(WP_NONE);
            }
        }

        _ => {}
    }

    NPC_FaceEnemy(QTRUE);

    ATST_Ranged(visible, advance, altAttack);
}

/*
-------------------------
NPC_BSDroid_Default
-------------------------
*/
pub unsafe fn NPC_BSATST_Default() {
    if !(*NPC).enemy.is_null() {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            (*NPCInfo).goalEntity = (*NPC).enemy;
        }
        ATST_Attack();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        ATST_Patrol();
    } else {
        ATST_Idle();
    }
}
