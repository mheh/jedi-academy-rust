//! Slice of `NPC_AI_Howler.c` — the Howler (acid-spitting quadruped) NPC behavior
//! state. Fully ported: the (empty) precache, the (empty) idle, the patrol/move/
//! combat think chain, the pain reaction, and the `NPC_BSHowler_Default` dispatcher.
//! Its NAV/AI callees (`UpdateGoal`/`NPC_MoveToGoal`, `NPC_CheckEnemyExt`,
//! `NPC_ClearLOS4`, `NPC_FaceEnemy`, `NPC_UpdateAngles`, `G_SetEnemy`) and the
//! `trap_Trace` combat-move have since landed, so the whole behavior file is here.
//!
//! Ported: `NPC_Howler_Precache` (NPC_AI_Howler.c:18), `Howler_Idle` (:28),
//! `Howler_Patrol` (:38), `Howler_Move` (:78), `Howler_TryDamage` (:89),
//! `Howler_Attack` (:112), `Howler_Combat` (:134), `NPC_Howler_Pain` (:178),
//! `NPC_BSHowler_Default` (:202).

#![allow(non_snake_case)] // C function names (`NPC_Howler_Pain`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_int;

use crate::codemp::game::anims::{BOTH_ATTACK1, BOTH_PAIN1};
use crate::codemp::game::b_public_h::SCF_LOOK_FOR_ENEMIES;
use crate::codemp::game::bg_public::{
    MASK_SHOT, MOD_MELEE, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE,
};
use crate::codemp::game::g_combat::G_Damage;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_NO_KNOCKBACK};
use crate::codemp::game::g_main::g_entities;
use crate::codemp::game::g_timer::{
    TIMER_Done, TIMER_Done2, TIMER_Exists, TIMER_Remove, TIMER_Set,
};
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC_SetAnim, NPC};
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_utils::{
    NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
use crate::codemp::game::q_math::{
    vec3_origin, AngleVectors, DistanceHorizontalSquared, VectorCopy, VectorLengthSquared,
    VectorMA, VectorSubtract,
};
use crate::codemp::game::q_shared::{crandom, random};
use crate::codemp::game::q_shared_h::{trace_t, vec3_t, BUTTON_WALKING, ENTITYNUM_WORLD};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 54;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const MAX_DISTANCE: c_int = 128;
#[allow(dead_code)]
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

/*
-------------------------
NPC_Howler_Precache
-------------------------
*/
pub fn NPC_Howler_Precache() {}

/*
-------------------------
Howler_Idle
-------------------------
*/
pub fn Howler_Idle() {}

/*
-------------------------
Howler_Patrol
-------------------------
*/
pub unsafe fn Howler_Patrol() {
    let mut dif: vec3_t = [0.0; 3];

    (*NPCInfo).localState = LSTATE_CLEAR;

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    } else if TIMER_Done(NPC, c"patrolTime".as_ptr()) != QFALSE {
        TIMER_Set(
            NPC,
            c"patrolTime".as_ptr(),
            (crandom() * 5000.0 + 5000.0) as c_int,
        );
    }

    //rwwFIXMEFIXME: Care about all clients, not just client 0
    VectorSubtract(
        &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0))
            .r
            .currentOrigin,
        &(*NPC).r.currentOrigin,
        &mut dif,
    );

    if VectorLengthSquared(&dif) < (256 * 256) as f32 {
        G_SetEnemy(
            NPC,
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0),
        );
    }

    if NPC_CheckEnemyExt(QTRUE) == QFALSE {
        Howler_Idle();
        return;
    }
}

/*
-------------------------
Howler_Move
-------------------------
*/
pub unsafe fn Howler_Move(_visible: qboolean) {
    if (*NPCInfo).localState != LSTATE_WAITING {
        (*NPCInfo).goalEntity = (*NPC).enemy;
        NPC_MoveToGoal(QTRUE);
        (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range
    }
}

//---------------------------------------------------------
pub unsafe fn Howler_TryDamage(enemy: *mut gentity_t, damage: c_int) {
    let mut end: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let tr: trace_t;

    if enemy.is_null() {
        return;
    }

    AngleVectors(&(*(*NPC).client).ps.viewangles, Some(&mut dir), None, None);
    VectorMA(&(*NPC).r.currentOrigin, MIN_DISTANCE as f32, &dir, &mut end);

    // Should probably trace from the mouth, but, ah well.
    tr = trap::Trace(
        &(*NPC).r.currentOrigin,
        &vec3_origin,
        &vec3_origin,
        &end,
        (*NPC).s.number,
        MASK_SHOT,
    );

    if tr.entityNum as c_int != ENTITYNUM_WORLD {
        let mut endpos: vec3_t = tr.endpos;
        G_Damage(
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize),
            NPC,
            NPC,
            &mut dir,
            &mut endpos,
            damage,
            DAMAGE_NO_KNOCKBACK,
            MOD_MELEE,
        );
    }
}

//------------------------------
pub unsafe fn Howler_Attack() {
    if TIMER_Exists(NPC, c"attacking".as_ptr()) == QFALSE {
        // Going to do ATTACK1
        TIMER_Set(
            NPC,
            c"attacking".as_ptr(),
            1700 + (random() * 200.0) as c_int,
        );
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_ATTACK1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );

        TIMER_Set(NPC, c"attack_dmg".as_ptr(), 200); // level two damage
    }

    // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks
    if TIMER_Done2(NPC, c"attack_dmg".as_ptr(), QTRUE) != QFALSE {
        Howler_TryDamage((*NPC).enemy, 5);
    }

    // Just using this to remove the attacking flag at the right time
    TIMER_Done2(NPC, c"attacking".as_ptr(), QTRUE);
}

//----------------------------------
pub unsafe fn Howler_Combat() {
    let distance: f32;
    let advance: qboolean;

    // If we cannot see our target or we have somewhere to go, then do that
    if NPC_ClearLOS4((*NPC).enemy) == QFALSE || !UpdateGoal().is_null() {
        (*NPCInfo).combatMove = QTRUE;
        (*NPCInfo).goalEntity = (*NPC).enemy;
        (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range

        NPC_MoveToGoal(QTRUE);
        return;
    }

    // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
    NPC_FaceEnemy(QTRUE);

    distance = DistanceHorizontalSquared(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);
    advance = if distance > MIN_DISTANCE_SQR as f32 {
        QTRUE
    } else {
        QFALSE
    };

    if (advance != QFALSE || (*NPCInfo).localState == LSTATE_WAITING)
        && TIMER_Done(NPC, c"attacking".as_ptr()) != QFALSE
    {
        // waiting monsters can't attack
        if TIMER_Done2(NPC, c"takingPain".as_ptr(), QTRUE) != QFALSE {
            (*NPCInfo).localState = LSTATE_CLEAR;
        } else {
            Howler_Move(1);
        }
    } else {
        Howler_Attack();
    }
}

/*
-------------------------
NPC_Howler_Pain
-------------------------
*/
pub unsafe extern "C" fn NPC_Howler_Pain(
    self_: *mut gentity_t,
    _attacker: *mut gentity_t,
    damage: c_int,
) {
    if damage >= 10 {
        TIMER_Remove(self_, c"attacking".as_ptr());
        TIMER_Set(self_, c"takingPain".as_ptr(), 2900);

        VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s.angles);

        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            BOTH_PAIN1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );

        if !(*self_).NPC.is_null() {
            (*(*self_).NPC).localState = LSTATE_WAITING;
        }
    }
}

/*
-------------------------
NPC_BSHowler_Default
-------------------------
*/
pub unsafe fn NPC_BSHowler_Default() {
    if !(*NPC).enemy.is_null() {
        Howler_Combat();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        Howler_Patrol();
    } else {
        Howler_Idle();
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}
