//! Slice of `NPC_AI_MineMonster.c` — the burrowing "mine monster" melee NPC's
//! behavior state. Opened bottom-up at the leaf seam, then drained the full
//! idle / patrol / move / combat think chain once `UpdateGoal`, `NPC_MoveToGoal`,
//! `G_SetEnemy`, `NPC_CheckEnemyExt`, `NPC_ClearLOS4`, `NPC_FaceEnemy` and
//! `NPC_UpdateAngles` all landed.
//!
//! Ported here so far: `NPC_MineMonster_Precache` (NPC_AI_MineMonster.c:18),
//! `MineMonster_Idle` (:35), `MineMonster_Patrol` (:50), `MineMonster_Move` (:90),
//! `MineMonster_TryDamage` (:101), `MineMonster_Attack` (:129),
//! `MineMonster_Combat` (:189), `NPC_MineMonster_Pain` (:234),
//! `NPC_BSMineMonster_Default` (:262).

#![allow(non_snake_case)] // C function names (`MineMonster_TryDamage`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_int;

use crate::codemp::game::anims::{
    BOTH_ATTACK1, BOTH_ATTACK2, BOTH_ATTACK3, BOTH_ATTACK4, BOTH_PAIN1,
};
use crate::codemp::game::b_public_h::SCF_LOOK_FOR_ENEMIES;
use crate::codemp::game::bg_public::{
    EV_PAIN, MASK_SHOT, MOD_MELEE, SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE,
};
use crate::codemp::game::g_combat::G_Damage;
use crate::codemp::game::g_local::{gentity_t, DAMAGE_NO_KNOCKBACK};
use crate::codemp::game::g_main::g_entities;
use crate::codemp::game::g_timer::{
    TIMER_Done, TIMER_Done2, TIMER_Exists, TIMER_Remove, TIMER_Set,
};
use crate::codemp::game::g_utils::{G_AddEvent, G_EffectIndex, G_Sound, G_SoundIndex};
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC_SetAnim, NPC};
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_utils::{
    NPC_CheckEnemyExt, NPC_ClearLOS4, NPC_FaceEnemy, NPC_UpdateAngles,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, AngleVectors, DistanceHorizontalSquared, VectorCopy, VectorLengthSquared,
    VectorMA, VectorSubtract,
};
use crate::codemp::game::q_shared::{crandom, random};
use crate::codemp::game::q_shared_h::{trace_t, vec3_t, BUTTON_WALKING, CHAN_AUTO, ENTITYNUM_NONE};
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
NPC_MineMonster_Precache
-------------------------
*/
pub unsafe fn NPC_MineMonster_Precache() {
    let mut i: c_int = 0;

    while i < 4 {
        G_SoundIndex(&format!("sound/chars/mine/misc/bite{}.wav", i + 1));
        G_SoundIndex(&format!("sound/chars/mine/misc/miss{}.wav", i + 1));
        i += 1;
    }
}

/*
-------------------------
MineMonster_Idle
-------------------------
*/
pub unsafe fn MineMonster_Idle() {
    if !UpdateGoal().is_null() {
        ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    }
}

/*
-------------------------
MineMonster_Patrol
-------------------------
*/
pub unsafe fn MineMonster_Patrol() {
    let mut dif: vec3_t = [0.0; 3];

    (*NPCInfo).localState = LSTATE_CLEAR;

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    } else {
        if TIMER_Done(NPC, c"patrolTime".as_ptr()) != QFALSE {
            TIMER_Set(
                NPC,
                c"patrolTime".as_ptr(),
                (crandom() as f32 * 5000.0 + 5000.0) as c_int,
            );
        }
    }

    //rwwFIXMEFIXME: Care about all clients, not just client 0
    VectorSubtract(
        &(*core::ptr::addr_of_mut!(g_entities)
            .cast::<gentity_t>()
            .add(0))
        .r
        .currentOrigin,
        &(*NPC).r.currentOrigin,
        &mut dif,
    );

    if VectorLengthSquared(&dif) < (256 * 256) as f32 {
        G_SetEnemy(
            NPC,
            core::ptr::addr_of_mut!(g_entities)
                .cast::<gentity_t>()
                .add(0),
        );
    }

    if NPC_CheckEnemyExt(QTRUE) == QFALSE {
        MineMonster_Idle();
        return;
    }
}

/*
-------------------------
MineMonster_Move
-------------------------
*/
pub unsafe fn MineMonster_Move(_visible: qboolean) {
    if (*NPCInfo).localState != LSTATE_WAITING {
        (*NPCInfo).goalEntity = (*NPC).enemy;
        NPC_MoveToGoal(QTRUE);
        (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range
    }
}

//---------------------------------------------------------
pub unsafe fn MineMonster_TryDamage(enemy: *mut gentity_t, damage: c_int) {
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

    if tr.entityNum as c_int >= 0 && (tr.entityNum as c_int) < ENTITYNUM_NONE {
        G_Damage(
            &mut *core::ptr::addr_of_mut!(g_entities)
                .cast::<gentity_t>()
                .add(tr.entityNum as usize),
            NPC,
            NPC,
            &mut dir,
            &mut tr.endpos.clone(),
            damage,
            DAMAGE_NO_KNOCKBACK,
            MOD_MELEE,
        );
        G_Sound(
            NPC,
            CHAN_AUTO,
            G_EffectIndex(&format!("sound/chars/mine/misc/bite{}.wav", Q_irand(1, 4))),
        );
    } else {
        G_Sound(
            NPC,
            CHAN_AUTO,
            G_EffectIndex(&format!("sound/chars/mine/misc/miss{}.wav", Q_irand(1, 4))),
        );
    }
}

//------------------------------
pub unsafe fn MineMonster_Attack() {
    if TIMER_Exists(NPC, c"attacking".as_ptr()) == QFALSE {
        // usually try and play a jump attack if the player somehow got above them....or just really rarely
        if !(*NPC).enemy.is_null()
            && (((*(*NPC).enemy).r.currentOrigin[2] - (*NPC).r.currentOrigin[2] > 10.0
                && random() > 0.1)
                || random() > 0.8)
        {
            // Going to do ATTACK4
            TIMER_Set(
                NPC,
                c"attacking".as_ptr(),
                1750 + (random() * 200.0) as c_int,
            );
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_ATTACK4,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );

            TIMER_Set(NPC, c"attack2_dmg".as_ptr(), 950); // level two damage
        } else if random() > 0.5 {
            if random() > 0.8 {
                // Going to do ATTACK3, (rare)
                TIMER_Set(NPC, c"attacking".as_ptr(), 850);
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK3,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                TIMER_Set(NPC, c"attack2_dmg".as_ptr(), 400); // level two damage
            } else {
                // Going to do ATTACK1
                TIMER_Set(NPC, c"attacking".as_ptr(), 850);
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_ATTACK1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                TIMER_Set(NPC, c"attack1_dmg".as_ptr(), 450); // level one damage
            }
        } else {
            // Going to do ATTACK2
            TIMER_Set(NPC, c"attacking".as_ptr(), 1250);
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_ATTACK2,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );

            TIMER_Set(NPC, c"attack1_dmg".as_ptr(), 700); // level one damage
        }
    } else {
        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks
        if TIMER_Done2(NPC, c"attack1_dmg".as_ptr(), QTRUE) != QFALSE {
            MineMonster_TryDamage((*NPC).enemy, 5);
        } else if TIMER_Done2(NPC, c"attack2_dmg".as_ptr(), QTRUE) != QFALSE {
            MineMonster_TryDamage((*NPC).enemy, 10);
        }
    }

    // Just using this to remove the attacking flag at the right time
    TIMER_Done2(NPC, c"attacking".as_ptr(), QTRUE);
}

//----------------------------------
pub unsafe fn MineMonster_Combat() {
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
    // waiting monsters can't attack
    {
        if TIMER_Done2(NPC, c"takingPain".as_ptr(), QTRUE) != QFALSE {
            (*NPCInfo).localState = LSTATE_CLEAR;
        } else {
            MineMonster_Move(1);
        }
    } else {
        MineMonster_Attack();
    }
}

/*
-------------------------
NPC_MineMonster_Pain
-------------------------
*/
pub unsafe extern "C" fn NPC_MineMonster_Pain(
    self_: *mut gentity_t,
    _attacker: *mut gentity_t,
    damage: c_int,
) {
    G_AddEvent(
        self_,
        EV_PAIN,
        ((*self_).health as f32 / (*(*self_).client).pers.maxHealth as f32 * 100.0).floor()
            as c_int,
    );

    if damage >= 10 {
        TIMER_Remove(self_, c"attacking".as_ptr());
        TIMER_Remove(self_, c"attacking1_dmg".as_ptr());
        TIMER_Remove(self_, c"attacking2_dmg".as_ptr());
        TIMER_Set(self_, c"takingPain".as_ptr(), 1350);

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
NPC_BSMineMonster_Default
-------------------------
*/
pub unsafe fn NPC_BSMineMonster_Default() {
    if !(*NPC).enemy.is_null() {
        MineMonster_Combat();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        MineMonster_Patrol();
    } else {
        MineMonster_Idle();
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}
