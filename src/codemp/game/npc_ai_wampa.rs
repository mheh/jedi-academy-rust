//! Slice of `NPC_AI_Wampa.c` — the Wampa (yeti-like melee monster) NPC behavior
//! state. Opened greenfield at the leaf seam, then drained the move/slash/attack
//! think helpers, then the idle/patrol/combat think states once `UpdateGoal`
//! landed, then the Ghoul2 bolt setup `Wampa_SetBolts`, and finally the master
//! `NPC_BSWampa_Default` think once `NPC_BSSearch`/`NPC_BSWander`/`NPC_CheckEnemy`
//! landed. File is now fully drained.
//!
//! Ported here: `Wampa_SetBolts` (NPC_AI_Wampa.c:16),
//! `NPC_Wampa_Precache` (:43), `Wampa_Idle` (:66), `Wampa_CheckRoar` (:78),
//! `Wampa_Patrol` (:94), `Wampa_Move` (:126), `Wampa_Slash` (:177),
//! `Wampa_Attack` (:267), `Wampa_Combat` (:344), `NPC_Wampa_Pain` (:433),
//! `NPC_BSWampa_Default` (:506).

#![allow(non_snake_case)] // C function names (`NPC_Wampa_Pain`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_int;

use crate::codemp::game::anims::{
    BOTH_ATTACK1, BOTH_ATTACK2, BOTH_ATTACK3, BOTH_DEATH17, BOTH_DEATHBACKWARD2, BOTH_GESTURE1,
    BOTH_GESTURE2, BOTH_PAIN1, BOTH_PAIN2,
};
use crate::codemp::game::b_public_h::{BS_DEFAULT, BS_SEARCH, BS_WANDER, SCF_LOOK_FOR_ENEMIES};
use crate::codemp::game::bg_pmove::BG_KnockDownable;
use crate::codemp::game::bg_public::EF2_USE_ALT_ANIM;
use crate::codemp::game::bg_public::{
    G2_MODELPART_HEAD, G2_MODELPART_RLEG, G2_MODELPART_WAIST, HANDEXTEND_KNOCKDOWN, MOD_MELEE,
    SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE,
};
use crate::codemp::game::g_combat::{G_Damage, G_Dismember, G_Knockdown};
use crate::codemp::game::g_local::{gentity_t, DAMAGE_NO_ARMOR, DAMAGE_NO_KNOCKBACK, FL_NOTARGET};
use crate::codemp::game::g_main::{g_entities, level};
use crate::codemp::game::g_nav::WAYPOINT_NONE;
use crate::codemp::game::g_timer::{
    TIMER_Done, TIMER_Done2, TIMER_Exists, TIMER_Remove, TIMER_Set,
};
use crate::codemp::game::g_utils::{G_Sound, G_SoundIndex, G_Throw};
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC_SetAnim, NPC};
use crate::codemp::game::npc_behavior::{NPC_BSSearch, NPC_BSSearchStart, NPC_BSWander};
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::npc_combat::{NPC_CheckEnemy, ValidEnemy};
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_senses::InFOV3;
use crate::codemp::game::npc_utils::NPC_UpdateAngles;
use crate::codemp::game::npc_utils::{
    NPC_CheckEnemyExt, NPC_ClearLOS, NPC_FaceEnemy, NPC_GetEntsNearBolt,
};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, AngleVectors, Distance, DistanceSquared, VectorCopy, VectorScale,
    VectorSet,
};
use crate::codemp::game::q_shared::{crandom, random};
use crate::codemp::game::q_shared_h::{
    vec3_t, BUTTON_WALKING, CHAN_AUTO, CHAN_VOICE, CHAN_WEAPON, ENTITYNUM_NONE, PITCH, YAW,
};
use crate::codemp::game::teams_h::{CLASS_ATST, CLASS_RANCOR, CLASS_WAMPA};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 48;
#[allow(dead_code)]
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const MAX_DISTANCE: c_int = 1024;
#[allow(dead_code)]
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

pub static mut enemyDist: f32 = 0.0;

pub unsafe fn Wampa_SetBolts(self_: *mut gentity_t) {
    if !self_.is_null() && !(*self_).client.is_null() {
        let ri = &mut (*(*self_).client).renderInfo;
        ri.headBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*head_eyes");
        //ri->cervicalBolt = trap_G2API_AddBolt(self->ghoul2, 0, "neck_bone" );
        //ri->chestBolt = trap_G2API_AddBolt(self->ghoul2, 0, "upper_spine");
        //ri->gutBolt = trap_G2API_AddBolt(self->ghoul2, 0, "mid_spine");
        ri.torsoBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "lower_spine");
        ri.crotchBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "rear_bone");
        //ri->elbowLBolt = trap_G2API_AddBolt(self->ghoul2, 0, "*l_arm_elbow");
        //ri->elbowRBolt = trap_G2API_AddBolt(self->ghoul2, 0, "*r_arm_elbow");
        ri.handLBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_hand");
        ri.handRBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_hand");
        //ri->kneeLBolt = trap_G2API_AddBolt(self->ghoul2, 0, "*hips_l_knee");
        //ri->kneeRBolt = trap_G2API_AddBolt(self->ghoul2, 0, "*hips_r_knee");
        ri.footLBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*l_leg_foot");
        ri.footRBolt = trap::G2API_AddBolt((*self_).ghoul2, 0, "*r_leg_foot");
    }
}

/*
-------------------------
NPC_Wampa_Precache
-------------------------
*/
pub fn NPC_Wampa_Precache() {
    /*
    int i;
    for ( i = 1; i < 4; i ++ )
    {
        G_SoundIndex( va("sound/chars/wampa/growl%d.wav", i) );
    }
    for ( i = 1; i < 3; i ++ )
    {
        G_SoundIndex( va("sound/chars/wampa/snort%d.wav", i) );
    }
    */
    G_SoundIndex("sound/chars/rancor/swipehit.wav");
    //G_SoundIndex( "sound/chars/wampa/chomp.wav" );
}

/*
-------------------------
Wampa_Idle
-------------------------
*/
pub unsafe fn Wampa_Idle() {
    (*NPCInfo).localState = LSTATE_CLEAR;

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons &= !BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    }
}

pub unsafe fn Wampa_CheckRoar(self_: *mut gentity_t) -> qboolean {
    if (*self_).wait < level.time as f32 {
        (*self_).wait = (level.time + Q_irand(5000, 20000)) as f32;
        NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            Q_irand(BOTH_GESTURE1, BOTH_GESTURE2),
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        TIMER_Set(self_, c"rageTime".as_ptr(), (*(*self_).client).ps.legsTimer);
        return QTRUE;
    }
    QFALSE
}

/*
-------------------------
Wampa_Patrol
-------------------------
*/
pub unsafe fn Wampa_Patrol() {
    (*NPCInfo).localState = LSTATE_CLEAR;

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons |= BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);
    } else if TIMER_Done(NPC, c"patrolTime".as_ptr()) != QFALSE {
        TIMER_Set(
            NPC,
            c"patrolTime".as_ptr(),
            (crandom() * 5000.0 + 5000.0) as c_int,
        );
    }

    if NPC_CheckEnemyExt(QTRUE) == QFALSE {
        Wampa_Idle();
        return;
    }
    Wampa_CheckRoar(NPC);
    TIMER_Set(NPC, c"lookForNewEnemy".as_ptr(), Q_irand(5000, 15000));
}

/*
-------------------------
Wampa_Move
-------------------------
*/
pub unsafe fn Wampa_Move(visible: qboolean) {
    if (*NPCInfo).localState != LSTATE_WAITING {
        (*NPCInfo).goalEntity = (*NPC).enemy;

        if !(*NPC).enemy.is_null() {
            //pick correct movement speed and anim
            //run by default
            ucmd.buttons &= !BUTTON_WALKING;
            if TIMER_Done(NPC, c"runfar".as_ptr()) == QFALSE
                || TIMER_Done(NPC, c"runclose".as_ptr()) == QFALSE
            {
                //keep running with this anim & speed for a bit
            } else if TIMER_Done(NPC, c"walk".as_ptr()) == QFALSE {
                //keep walking for a bit
                ucmd.buttons |= BUTTON_WALKING;
            } else if visible != QFALSE && enemyDist > 384.0 && (*NPCInfo).stats.runSpeed == 180 {
                //fast run, all fours
                (*NPCInfo).stats.runSpeed = 300;
                TIMER_Set(NPC, c"runfar".as_ptr(), Q_irand(2000, 4000));
            } else if enemyDist > 256.0 && (*NPCInfo).stats.runSpeed == 300 {
                //slow run, upright
                (*NPCInfo).stats.runSpeed = 180;
                TIMER_Set(NPC, c"runclose".as_ptr(), Q_irand(3000, 5000));
            } else if enemyDist < 128.0 {
                //walk
                (*NPCInfo).stats.runSpeed = 180;
                ucmd.buttons |= BUTTON_WALKING;
                TIMER_Set(NPC, c"walk".as_ptr(), Q_irand(4000, 6000));
            }
        }

        if (*NPCInfo).stats.runSpeed == 300 {
            //need to use the alternate run - hunched over on all fours
            (*(*NPC).client).ps.eFlags2 |= EF2_USE_ALT_ANIM;
        }
        NPC_MoveToGoal(QTRUE);
        (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range
    }
}

//---------------------------------------------------------
//extern void G_Knockdown( gentity_t *self, gentity_t *attacker, const vec3_t pushDir, float strength, qboolean breakSaberLock );
//extern void G_Knockdown( gentity_t *victim );
//extern void G_Dismember( gentity_t *ent, gentity_t *enemy, vec3_t point, int limbType, float limbRollBase, float limbPitchBase, int deathAnim, qboolean postDeath );
//extern int NPC_GetEntsNearBolt( int *radiusEnts, float radius, int boltIndex, vec3_t boltOrg );

pub unsafe fn Wampa_Slash(boltIndex: c_int, backhand: qboolean) {
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let numEnts: c_int;
    let radius: f32 = 88.0;
    let radiusSquared: f32 = radius * radius;
    let mut i: c_int;
    let mut boltOrg: vec3_t = [0.0; 3];
    let damage: c_int = if backhand != QFALSE {
        Q_irand(10, 15)
    } else {
        Q_irand(20, 30)
    };

    numEnts = NPC_GetEntsNearBolt(radiusEntNums.as_mut_ptr(), radius, boltIndex, &mut boltOrg);

    i = 0;
    while i < numEnts {
        let radiusEnt: *mut gentity_t = &mut *core::ptr::addr_of_mut!(g_entities)
            .cast::<gentity_t>()
            .add(radiusEntNums[i as usize] as usize);
        if (*radiusEnt).inuse == QFALSE {
            i += 1;
            continue;
        }

        if radiusEnt == NPC {
            //Skip the wampa ent
            i += 1;
            continue;
        }

        if (*radiusEnt).client.is_null() {
            //must be a client
            i += 1;
            continue;
        }

        if DistanceSquared(&(*radiusEnt).r.currentOrigin, &boltOrg) <= radiusSquared {
            //smack
            G_Damage(
                radiusEnt,
                NPC,
                NPC,
                &mut vec3_origin.clone(),
                &mut (*radiusEnt).r.currentOrigin.clone(),
                damage,
                if backhand != QFALSE {
                    DAMAGE_NO_ARMOR
                } else {
                    DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK
                },
                MOD_MELEE,
            );
            if backhand != QFALSE {
                //actually push the enemy
                let mut pushDir: vec3_t = [0.0; 3];
                let mut angs: vec3_t = [0.0; 3];
                VectorCopy(&(*(*NPC).client).ps.viewangles, &mut angs);
                angs[YAW as usize] += flrand(25.0, 50.0);
                angs[PITCH as usize] = flrand(-25.0, -15.0);
                AngleVectors(&angs, Some(&mut pushDir), None, None);
                if (*(*radiusEnt).client).NPC_class != CLASS_WAMPA
                    && (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                    && (*(*radiusEnt).client).NPC_class != CLASS_ATST
                {
                    G_Throw(radiusEnt, &pushDir, 65.0);
                    if BG_KnockDownable(&mut (*(*radiusEnt).client).ps) != QFALSE
                        && (*radiusEnt).health > 0
                        && Q_irand(0, 1) != 0
                    {
                        //do pain on enemy
                        (*(*radiusEnt).client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN;
                        (*(*radiusEnt).client).ps.forceDodgeAnim = 0;
                        (*(*radiusEnt).client).ps.forceHandExtendTime = level.time + 1100;
                        (*(*radiusEnt).client).ps.quickerGetup = QFALSE;
                    }
                }
            } else if (*radiusEnt).health <= 0 && !(*radiusEnt).client.is_null() {
                //killed them, chance of dismembering
                if Q_irand(0, 1) == 0 {
                    //bite something off
                    let hitLoc: c_int = Q_irand(G2_MODELPART_HEAD, G2_MODELPART_RLEG);
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
                }
            } else if Q_irand(0, 3) == 0 && (*radiusEnt).health > 0 {
                //one out of every 4 normal hits does a knockdown, too
                let mut pushDir: vec3_t = [0.0; 3];
                let mut angs: vec3_t = [0.0; 3];
                VectorCopy(&(*(*NPC).client).ps.viewangles, &mut angs);
                angs[YAW as usize] += flrand(25.0, 50.0);
                angs[PITCH as usize] = flrand(-25.0, -15.0);
                AngleVectors(&angs, Some(&mut pushDir), None, None);
                G_Knockdown(radiusEnt);
            }
            G_Sound(
                radiusEnt,
                CHAN_WEAPON,
                G_SoundIndex("sound/chars/rancor/swipehit.wav"),
            );
        }
        i += 1;
    }
}

/*
-------------------------
NPC_Wampa_Pain
-------------------------
*/
//void NPC_Wampa_Pain( gentity_t *self, gentity_t *inflictor, gentity_t *other, const vec3_t point, int damage, int mod,int hitLoc )
pub unsafe extern "C" fn NPC_Wampa_Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    let mut hitByWampa = QFALSE;
    if !attacker.is_null()
        && !(*attacker).client.is_null()
        && (*(*attacker).client).NPC_class == CLASS_WAMPA
    {
        hitByWampa = QTRUE;
    }
    if !attacker.is_null()
        && (*attacker).inuse != QFALSE
        && attacker != (*self_).enemy
        && (*attacker).flags & FL_NOTARGET == 0
    {
        if (((*attacker).s.number == 0) && Q_irand(0, 3) == 0)
            || (*self_).enemy.is_null()
            || (*(*self_).enemy).health == 0
            || (!(*(*self_).enemy).client.is_null()
                && (*(*(*self_).enemy).client).NPC_class == CLASS_WAMPA)
            || (Q_irand(0, 4) == 0
                && DistanceSquared(&(*attacker).r.currentOrigin, &(*self_).r.currentOrigin)
                    < DistanceSquared(
                        &(*(*self_).enemy).r.currentOrigin,
                        &(*self_).r.currentOrigin,
                    ))
        {
            //if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
            //FIXME: if can't nav to my enemy, take this guy if I can nav to him
            G_SetEnemy(self_, attacker);
            TIMER_Set(self_, c"lookForNewEnemy".as_ptr(), Q_irand(5000, 15000));
            if hitByWampa != QFALSE {
                //stay mad at this Wampa for 2-5 secs before looking for attacker enemies
                TIMER_Set(self_, c"wampaInfight".as_ptr(), Q_irand(2000, 5000));
            }
        }
    }
    if (hitByWampa != QFALSE || Q_irand(0, 100) < damage) //hit by wampa, hit while holding live victim, or took a lot of damage
        && (*(*self_).client).ps.legsAnim != BOTH_GESTURE1
        && (*(*self_).client).ps.legsAnim != BOTH_GESTURE2
        && TIMER_Done(self_, c"takingPain".as_ptr()) != QFALSE
    {
        if Wampa_CheckRoar(self_) == QFALSE {
            if (*(*self_).client).ps.legsAnim != BOTH_ATTACK1
                && (*(*self_).client).ps.legsAnim != BOTH_ATTACK2
                && (*(*self_).client).ps.legsAnim != BOTH_ATTACK3
            {
                //cant interrupt one of the big attack anims
                if (*self_).health > 100 || hitByWampa != QFALSE {
                    TIMER_Remove(self_, c"attacking".as_ptr());

                    VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s.angles);

                    if Q_irand(0, 1) == 0 {
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
                    //allow us to re-evaluate our running speed/anim
                    TIMER_Set(self_, c"runfar".as_ptr(), -1);
                    TIMER_Set(self_, c"runclose".as_ptr(), -1);
                    TIMER_Set(self_, c"walk".as_ptr(), -1);

                    if !(*self_).NPC.is_null() {
                        (*(*self_).NPC).localState = LSTATE_WAITING;
                    }
                }
            }
        }
    }
}

//------------------------------
pub unsafe fn Wampa_Attack(distance: f32, doCharge: qboolean) {
    if TIMER_Exists(NPC, c"attacking".as_ptr()) == QFALSE {
        if Q_irand(0, 2) != 0 && doCharge == QFALSE {
            //double slash
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_ATTACK1,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(NPC, c"attack_dmg".as_ptr(), 750);
        } else if doCharge != QFALSE || (distance > 270.0 && distance < 430.0 && Q_irand(0, 1) != 0)
        {
            //leap
            let mut fwd: vec3_t = [0.0; 3];
            let mut yawAng: vec3_t = [0.0; 3];
            VectorSet(
                &mut yawAng,
                0.0,
                (*(*NPC).client).ps.viewangles[YAW as usize],
                0.0,
            );
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_ATTACK2,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(NPC, c"attack_dmg".as_ptr(), 500);
            AngleVectors(&yawAng, Some(&mut fwd), None, None);
            VectorScale(&fwd, distance * 1.5, &mut (*(*NPC).client).ps.velocity);
            (*(*NPC).client).ps.velocity[2] = 150.0;
            (*(*NPC).client).ps.groundEntityNum = ENTITYNUM_NONE;
        } else {
            //backhand
            NPC_SetAnim(
                NPC,
                SETANIM_BOTH,
                BOTH_ATTACK3,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            TIMER_Set(NPC, c"attack_dmg".as_ptr(), 250);
        }

        TIMER_Set(
            NPC,
            c"attacking".as_ptr(),
            (*(*NPC).client).ps.legsTimer + (random() * 200.0) as c_int,
        );
        //allow us to re-evaluate our running speed/anim
        TIMER_Set(NPC, c"runfar".as_ptr(), -1);
        TIMER_Set(NPC, c"runclose".as_ptr(), -1);
        TIMER_Set(NPC, c"walk".as_ptr(), -1);
    }

    // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

    if TIMER_Done2(NPC, c"attack_dmg".as_ptr(), QTRUE) != QFALSE {
        match (*(*NPC).client).ps.legsAnim {
            BOTH_ATTACK1 => {
                Wampa_Slash((*(*NPC).client).renderInfo.handRBolt, QFALSE);
                //do second hit
                TIMER_Set(NPC, c"attack_dmg2".as_ptr(), 100);
            }
            BOTH_ATTACK2 => {
                Wampa_Slash((*(*NPC).client).renderInfo.handRBolt, QFALSE);
                TIMER_Set(NPC, c"attack_dmg2".as_ptr(), 100);
            }
            BOTH_ATTACK3 => {
                Wampa_Slash((*(*NPC).client).renderInfo.handLBolt, QTRUE);
            }
            _ => {}
        }
    } else if TIMER_Done2(NPC, c"attack_dmg2".as_ptr(), QTRUE) != QFALSE {
        match (*(*NPC).client).ps.legsAnim {
            BOTH_ATTACK1 => {
                Wampa_Slash((*(*NPC).client).renderInfo.handLBolt, QFALSE);
            }
            BOTH_ATTACK2 => {
                Wampa_Slash((*(*NPC).client).renderInfo.handLBolt, QFALSE);
            }
            _ => {}
        }
    }

    // Just using this to remove the attacking flag at the right time
    TIMER_Done2(NPC, c"attacking".as_ptr(), QTRUE);

    if (*(*NPC).client).ps.legsAnim == BOTH_ATTACK1
        && distance > ((*NPC).r.maxs[0] + MIN_DISTANCE as f32)
    {
        //okay to keep moving
        ucmd.buttons |= BUTTON_WALKING;
        Wampa_Move(1);
    }
}

/*
-------------------------
NPC_BSWampa_Default
-------------------------
*/
pub unsafe fn NPC_BSWampa_Default() {
    (*(*NPC).client).ps.eFlags2 &= !EF2_USE_ALT_ANIM;
    //NORMAL ANIMS
    //	stand1 = normal stand
    //	walk1 = normal, non-angry walk
    //	walk2 = injured
    //	run1 = far away run
    //	run2 = close run
    //VICTIM ANIMS
    //	grabswipe = melee1 - sweep out and grab
    //	stand2 attack = attack4 - while holding victim, swipe at him
    //	walk3_drag = walk5 - walk with drag
    //	stand2 = hold victim
    //	stand2to1 = drop victim
    if TIMER_Done(NPC, c"rageTime".as_ptr()) == QFALSE {
        //do nothing but roar first time we see an enemy
        NPC_FaceEnemy(QTRUE);
        return;
    }
    if !(*NPC).enemy.is_null() {
        if TIMER_Done(NPC, c"attacking".as_ptr()) == QFALSE {
            //in middle of attack
            //face enemy
            NPC_FaceEnemy(QTRUE);
            //continue attack logic
            enemyDist = Distance(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);
            Wampa_Attack(enemyDist, QFALSE);
            return;
        } else {
            if TIMER_Done(NPC, c"angrynoise".as_ptr()) != QFALSE {
                G_Sound(
                    NPC,
                    CHAN_VOICE,
                    G_SoundIndex(&format!(
                        "sound/chars/wampa/misc/anger{}.wav",
                        Q_irand(1, 2)
                    )),
                );

                TIMER_Set(NPC, c"angrynoise".as_ptr(), Q_irand(5000, 10000));
            }
            //else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
            if !(*(*NPC).enemy).client.is_null()
                && (*(*(*NPC).enemy).client).NPC_class == CLASS_WAMPA
            {
                //got mad at another Wampa, look for a valid enemy
                if TIMER_Done(NPC, c"wampaInfight".as_ptr()) != QFALSE {
                    NPC_CheckEnemyExt(QTRUE);
                }
            } else {
                if ValidEnemy((*NPC).enemy) == QFALSE {
                    TIMER_Remove(NPC, c"lookForNewEnemy".as_ptr()); //make them look again right now
                    if (*(*NPC).enemy).inuse == QFALSE
                        || level.time - (*(*NPC).enemy).s.time > Q_irand(10000, 15000)
                    {
                        //it's been a while since the enemy died, or enemy is completely gone, get bored with him
                        (*NPC).enemy = core::ptr::null_mut();
                        Wampa_Patrol();
                        NPC_UpdateAngles(QTRUE, QTRUE);
                        //just lost my enemy
                        if (*NPC).spawnflags & 2 != 0 {
                            //search around me if I don't have an enemy
                            NPC_BSSearchStart((*NPC).waypoint, BS_SEARCH);
                            (*NPCInfo).tempBehavior = BS_DEFAULT;
                        } else if (*NPC).spawnflags & 1 != 0 {
                            //wander if I don't have an enemy
                            NPC_BSSearchStart((*NPC).waypoint, BS_WANDER);
                            (*NPCInfo).tempBehavior = BS_DEFAULT;
                        }
                        return;
                    }
                }
                if TIMER_Done(NPC, c"lookForNewEnemy".as_ptr()) != QFALSE {
                    let newEnemy: *mut gentity_t;
                    let sav_enemy: *mut gentity_t = (*NPC).enemy; //FIXME: what about NPC->lastEnemy?
                    (*NPC).enemy = core::ptr::null_mut();
                    newEnemy = NPC_CheckEnemy(
                        ((*NPCInfo).confusionTime < level.time) as qboolean,
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
            Wampa_Combat();
            return;
        }
    } else {
        if TIMER_Done(NPC, c"idlenoise".as_ptr()) != QFALSE {
            G_Sound(
                NPC,
                CHAN_AUTO,
                G_SoundIndex("sound/chars/wampa/misc/anger3.wav"),
            );

            TIMER_Set(NPC, c"idlenoise".as_ptr(), Q_irand(2000, 4000));
        }
        if (*NPC).spawnflags & 2 != 0 {
            //search around me if I don't have an enemy
            if (*NPCInfo).homeWp == WAYPOINT_NONE {
                //no homewap, initialize the search behavior
                NPC_BSSearchStart(WAYPOINT_NONE, BS_SEARCH);
                (*NPCInfo).tempBehavior = BS_DEFAULT;
            }
            ucmd.buttons |= BUTTON_WALKING;
            NPC_BSSearch(); //this automatically looks for enemies
        } else if (*NPC).spawnflags & 1 != 0 {
            //wander if I don't have an enemy
            if (*NPCInfo).homeWp == WAYPOINT_NONE {
                //no homewap, initialize the wander behavior
                NPC_BSSearchStart(WAYPOINT_NONE, BS_WANDER);
                (*NPCInfo).tempBehavior = BS_DEFAULT;
            }
            ucmd.buttons |= BUTTON_WALKING;
            NPC_BSWander();
            if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
                if NPC_CheckEnemyExt(QTRUE) == QFALSE {
                    Wampa_Idle();
                } else {
                    Wampa_CheckRoar(NPC);
                    TIMER_Set(NPC, c"lookForNewEnemy".as_ptr(), Q_irand(5000, 15000));
                }
            }
        } else {
            if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
                Wampa_Patrol();
            } else {
                Wampa_Idle();
            }
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

//----------------------------------
pub unsafe fn Wampa_Combat() {
    // If we cannot see our target or we have somewhere to go, then do that
    if NPC_ClearLOS(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin) == QFALSE {
        if Q_irand(0, 10) == 0 {
            if Wampa_CheckRoar(NPC) != QFALSE {
                return;
            }
        }
        (*NPCInfo).combatMove = QTRUE;
        (*NPCInfo).goalEntity = (*NPC).enemy;
        (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range

        Wampa_Move(0);
        return;
    } else if !UpdateGoal().is_null() {
        (*NPCInfo).combatMove = QTRUE;
        (*NPCInfo).goalEntity = (*NPC).enemy;
        (*NPCInfo).goalRadius = MAX_DISTANCE; // just get us within combat range

        Wampa_Move(1);
        return;
    } else {
        enemyDist = Distance(&(*NPC).r.currentOrigin, &(*(*NPC).enemy).r.currentOrigin);
        let distance: f32 = enemyDist;
        let mut advance: qboolean = if distance > ((*NPC).r.maxs[0] + MIN_DISTANCE as f32) {
            QTRUE
        } else {
            QFALSE
        };
        let mut doCharge: qboolean = QFALSE;

        // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
        //FIXME: always seems to face off to the left or right?!!!!
        NPC_FaceEnemy(QTRUE);

        if advance != QFALSE {
            //have to get closer
            let mut yawOnlyAngles: vec3_t = [0.0; 3];
            VectorSet(
                &mut yawOnlyAngles,
                0.0,
                (*NPC).r.currentAngles[YAW as usize],
                0.0,
            );
            if (*(*NPC).enemy).health > 0//enemy still alive
                && (distance - 350.0).abs() <= 80.0 //enemy anywhere from 270 to 430 away
                && InFOV3(&(*(*NPC).enemy).r.currentOrigin, &(*NPC).r.currentOrigin, &yawOnlyAngles, 20, 20) != QFALSE
            //enemy generally in front
            {
                //10% chance of doing charge anim
                if Q_irand(0, 9) == 0 {
                    //go for the charge
                    doCharge = QTRUE;
                    advance = QFALSE;
                }
            }
        }

        if (advance != QFALSE || (*NPCInfo).localState == LSTATE_WAITING)
            && TIMER_Done(NPC, c"attacking".as_ptr()) != QFALSE
        // waiting monsters can't attack
        {
            if TIMER_Done2(NPC, c"takingPain".as_ptr(), QTRUE) != QFALSE {
                (*NPCInfo).localState = LSTATE_CLEAR;
            } else {
                Wampa_Move(1);
            }
        } else {
            if Q_irand(0, 20) == 0 {
                //FIXME: only do this if we just damaged them or vice-versa?
                if Wampa_CheckRoar(NPC) != QFALSE {
                    return;
                }
            }
            if Q_irand(0, 1) == 0 {
                //FIXME: base on skill
                Wampa_Attack(distance, doCharge);
            }
        }
    }
}
