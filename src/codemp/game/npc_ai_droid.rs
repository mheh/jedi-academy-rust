//! Slice of `NPC_AI_Droid.c` — the utility-droid (R2D2/R5D2/Gonk/Mouse/Protocol)
//! NPC behavior state. Opened greenfield at the leaf seam: the per-class precache
//! routines, the (empty) idle, the localState-reset pain timer, and the
//! self-contained eye-jitter / turn-anim helpers are genuinely portable today.
//! The patrol / run / spin chain (`Droid_Patrol`, `Droid_Run`, `Droid_Spin`,
//! `NPC_BSDroid_Default`) routes through `NPC_UpdateAngles`,
//! `NPC_MoveToGoal`/`UpdateGoal` (NAV), `G_SoundOnEnt`, `G_PlayEffectID`,
//! `trap_G2API_GetSurfaceRenderStatus` (Ghoul2) — all now ported.
//! `NPC_Droid_Pain` (:273) lands now that `NPC_SetSurfaceOnOff` and the
//! NPC-AI core (`NPC_GetPainChance`/`NPC_Pain`) are ported. This file is now
//! COMPLETE.
//!
//! Ported here so far: `NPC_Droid_Pain` (NPC_AI_Droid.c:273),
//! `NPC_Mouse_Precache` (:455),
//! `NPC_R5D2_Precache` (:474), `NPC_R2D2_Precache` (:497),
//! `NPC_Gonk_Precache` (:520), `NPC_Protocol_Precache` (:537),
//! `Droid_Idle` (:53), `Droid_Pain` (:442), `R2D2_PartsMove` (:24),
//! `R2D2_TurnAnims` (:65), `Droid_Patrol` (:102), `Droid_Run` (:175),
//! `Droid_Spin` (:207), `NPC_BSDroid_Default` (:597).
//!
//! Note: `R5D2_OffsetLook` (:544) and `R5D2_LookAround` (:564) are entirely
//! commented out in the C source and have no real definition.

#![allow(non_snake_case)] // C function names (`NPC_R2D2_Precache`) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::anims::{BOTH_PAIN1, BOTH_PAIN2, BOTH_RUN1, BOTH_STAND2, BOTH_TURN_LEFT1,
    BOTH_TURN_RIGHT1};
use crate::codemp::game::bg_public::{MOD_DEMP2, MOD_DEMP2_ALT, SETANIM_BOTH, SETANIM_FLAG_HOLD,
    SETANIM_FLAG_OVERRIDE};
use crate::codemp::game::b_public_h::SCF_LOOK_FOR_ENEMIES;
use crate::codemp::game::g_combat::gPainMOD;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::level;
use crate::codemp::game::g_timer::{TIMER_Done, TIMER_Set};
use crate::codemp::game::g_utils::{G_EffectIndex, G_PlayEffectID, G_SoundIndex, G_SoundOnEnt};
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC_SetAnim, NPC};
use crate::codemp::game::npc_goal::UpdateGoal;
use crate::codemp::game::npc_move::NPC_MoveToGoal;
use crate::codemp::game::npc_reactions::{NPC_GetPainChance, NPC_Pain};
use crate::codemp::game::npc_utils::{NPC_SetBoneAngles, NPC_SetSurfaceOnOff, NPC_UpdateAngles};
use crate::codemp::game::q_math::{AngleDelta, AngleNormalize360, AngleVectors, vec3_origin,
    VectorCopy, VectorMA, VectorNormalize, VectorSubtract};
use crate::codemp::game::q_shared::{crandom, random};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{vec3_t, BUTTON_WALKING, CHAN_AUTO, YAW};
use crate::codemp::game::teams_h::{CLASS_GONK, CLASS_INTERROGATOR, CLASS_MOUSE, CLASS_R2D2,
    CLASS_R5D2};
use crate::ffi::types::{QFALSE, QTRUE};
use crate::trap;

//Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_BACKINGUP: c_int = 1;
const LSTATE_SPINNING: c_int = 2;
const LSTATE_PAIN: c_int = 3;
const LSTATE_DROP: c_int = 4;

// #define TURN_OFF 0x00000100 (the surface-off bit passed to NPC_SetSurfaceOnOff;
// TURN_ON is 0 and never passed here).
const TURN_OFF: c_int = 0x00000100;

/*
-------------------------
R2D2_PartsMove
-------------------------
*/
pub unsafe fn R2D2_PartsMove() {
    // Front 'eye' lense
    if TIMER_Done(NPC, c"eyeDelay".as_ptr()) != QFALSE {
        (*NPC).pos1[1] = AngleNormalize360((*NPC).pos1[1]);

        (*NPC).pos1[0] += Q_irand(-20, 20) as f32; // Roll
        (*NPC).pos1[1] = Q_irand(-20, 20) as f32;
        (*NPC).pos1[2] = Q_irand(-20, 20) as f32;

        /*
        if (NPC->genericBone1)
        {
            gi.G2API_SetBoneAnglesIndex( &NPC->ghoul2[NPC->playerModel], NPC->genericBone1, NPC->pos1, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, NULL );
        }
        */
        NPC_SetBoneAngles(NPC, "f_eye", &(*NPC).pos1);

        TIMER_Set(NPC, c"eyeDelay".as_ptr(), Q_irand(100, 1000));
    }
}

/*
-------------------------
NPC_BSDroid_Idle
-------------------------
*/
pub fn Droid_Idle() {
    //	VectorCopy( NPCInfo->investigateGoal, lookPos );

    //	NPC_FacePosition( lookPos );
}

/*
-------------------------
R2D2_TurnAnims
-------------------------
*/
pub unsafe fn R2D2_TurnAnims() {
    let turndelta: f32;
    let anim: c_int;

    turndelta = AngleDelta((*NPC).r.currentAngles[YAW], (*NPCInfo).desiredYaw);

    if (turndelta.abs() > 20.0)
        && (((*(*NPC).client).NPC_class == CLASS_R2D2) || ((*(*NPC).client).NPC_class == CLASS_R5D2))
    {
        anim = (*(*NPC).client).ps.legsAnim;
        if turndelta < 0.0 {
            if anim != BOTH_TURN_LEFT1 {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_TURN_LEFT1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
        } else {
            if anim != BOTH_TURN_RIGHT1 {
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_TURN_RIGHT1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
        }
    } else {
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_RUN1,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    }
}

/*
-------------------------
Droid_Patrol
-------------------------
*/
pub unsafe fn Droid_Patrol() {
    (*NPC).pos1[1] = AngleNormalize360((*NPC).pos1[1]);

    if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class != CLASS_GONK {
        if (*(*NPC).client).NPC_class != CLASS_R5D2 {
            //he doesn't have an eye.
            R2D2_PartsMove(); // Get his eye moving.
        }
        R2D2_TurnAnims();
    }

    //If we have somewhere to go, then do that
    if !UpdateGoal().is_null() {
        ucmd.buttons |= BUTTON_WALKING;
        NPC_MoveToGoal(QTRUE);

        if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_MOUSE {
            (*NPCInfo).desiredYaw += ((level.time as f64 * 0.5).sin() * 25.0) as f32; // Weaves side to side a little

            if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
                G_SoundOnEnt(
                    NPC,
                    CHAN_AUTO,
                    &format!("sound/chars/mouse/misc/mousego{}.wav", Q_irand(1, 3)),
                );

                TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
            }
        } else if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_R2D2 {
            if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
                G_SoundOnEnt(
                    NPC,
                    CHAN_AUTO,
                    &format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", Q_irand(1, 3)),
                );

                TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
            }
        } else if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_R5D2 {
            if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
                G_SoundOnEnt(
                    NPC,
                    CHAN_AUTO,
                    &format!("sound/chars/r5d2/misc/r5talk{}.wav", Q_irand(1, 4)),
                );

                TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
            }
        }
        if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_GONK {
            if TIMER_Done(NPC, c"patrolNoise".as_ptr()) != QFALSE {
                G_SoundOnEnt(
                    NPC,
                    CHAN_AUTO,
                    &format!("sound/chars/gonk/misc/gonktalk{}.wav", Q_irand(1, 2)),
                );

                TIMER_Set(NPC, c"patrolNoise".as_ptr(), Q_irand(2000, 4000));
            }
        }
        //		else
        //		{
        //			R5D2_LookAround();
        //		}
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
Droid_Run
-------------------------
*/
pub unsafe fn Droid_Run() {
    R2D2_PartsMove();

    if (*NPCInfo).localState == LSTATE_BACKINGUP {
        ucmd.forwardmove = -127;
        (*NPCInfo).desiredYaw += 5.0;

        (*NPCInfo).localState = LSTATE_NONE; // So he doesn't constantly backup.
    } else {
        ucmd.forwardmove = 64;
        //If we have somewhere to go, then do that
        if !UpdateGoal().is_null() {
            if NPC_MoveToGoal(QFALSE) != QFALSE {
                (*NPCInfo).desiredYaw += ((level.time as f64 * 0.5).sin() * 5.0) as f32;
                // Weaves side to side a little
            }
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
void Droid_Spin( void )
-------------------------
*/
pub unsafe fn Droid_Spin() {
    let dir: vec3_t = [0.0, 0.0, 1.0];

    R2D2_TurnAnims();

    // Head is gone, spin and spark
    if (*(*NPC).client).NPC_class == CLASS_R5D2 || (*(*NPC).client).NPC_class == CLASS_R2D2 {
        // No head?
        if trap::G2API_GetSurfaceRenderStatus((*NPC).ghoul2, 0, c"head".as_ptr()) > 0 {
            if TIMER_Done(NPC, c"smoke".as_ptr()) != QFALSE
                && TIMER_Done(NPC, c"droidsmoketotal".as_ptr()) == QFALSE
            {
                TIMER_Set(NPC, c"smoke".as_ptr(), 100);
                G_PlayEffectID(
                    G_EffectIndex("volumetric/droid_smoke"),
                    &(*NPC).r.currentOrigin,
                    &dir,
                );
            }

            if TIMER_Done(NPC, c"droidspark".as_ptr()) != QFALSE {
                TIMER_Set(NPC, c"droidspark".as_ptr(), Q_irand(100, 500));
                G_PlayEffectID(G_EffectIndex("sparks/spark"), &(*NPC).r.currentOrigin, &dir);
            }

            ucmd.forwardmove = Q_irand(-64, 64) as i8;

            if TIMER_Done(NPC, c"roam".as_ptr()) != QFALSE {
                TIMER_Set(NPC, c"roam".as_ptr(), Q_irand(250, 1000));
                (*NPCInfo).desiredYaw = Q_irand(0, 360) as f32; // Go in random directions
            }
        } else {
            if TIMER_Done(NPC, c"roam".as_ptr()) != QFALSE {
                (*NPCInfo).localState = LSTATE_NONE;
            } else {
                (*NPCInfo).desiredYaw = AngleNormalize360((*NPCInfo).desiredYaw + 40.0);
                // Spin around
            }
        }
    } else {
        if TIMER_Done(NPC, c"roam".as_ptr()) != QFALSE {
            (*NPCInfo).localState = LSTATE_NONE;
        } else {
            (*NPCInfo).desiredYaw = AngleNormalize360((*NPCInfo).desiredYaw + 40.0);
            // Spin around
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/// `void NPC_Droid_Pain(gentity_t *self, gentity_t *attacker, int damage)`
/// (NPC_AI_Droid.c:273) — per-class droid pain reaction. R5D2/R2D2 may pop their
/// head (out of PVS / DEMP2 / low health → effects + electrify + spin) or play a
/// normal pain anim; the Mouse droid backs up or shocks; the Interrogator gets
/// knocked back by DEMP2. Always tails into `NPC_Pain`.
///
/// # Safety
/// `self_`/`self_->client`/`self_->NPC` must be valid; `attacker` may be null.
pub unsafe extern "C" fn NPC_Droid_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    let other = attacker;
    let anim: c_int;
    let mod_ = *addr_of!(gPainMOD);
    let pain_chance: f32;

    VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s.angles);

    if (*(*self_).client).NPC_class == CLASS_R5D2 {
        pain_chance = NPC_GetPainChance(self_, damage);

        // Put it in pain
        if mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT || random() < pain_chance
        // Spin around in pain? Demp2 always does this
        {
            // Health is between 0-30 or was hit by a DEMP2 so pop his head
            if (*self_).s.m_iVehicleNum == 0
                && ((*self_).health < 30 || mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT)
            {
                if (*self_).spawnflags & 2 == 0
                // Doesn't have to ALWAYSDIE
                {
                    if (*(*self_).NPC).localState != LSTATE_SPINNING
                        && trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"head".as_ptr())
                            == 0
                    {
                        NPC_SetSurfaceOnOff(self_, c"head".as_ptr(), TURN_OFF);

                        if (*(*self_).client).ps.m_iVehicleNum != 0 {
                            let mut up: vec3_t = [0.0; 3];
                            AngleVectors(&(*self_).r.currentAngles, None, None, Some(&mut up));
                            G_PlayEffectID(
                                G_EffectIndex("chunks/r5d2head_veh"),
                                &(*self_).r.currentOrigin,
                                &up,
                            );
                        } else {
                            G_PlayEffectID(
                                G_EffectIndex("small_chunks"),
                                &(*self_).r.currentOrigin,
                                &vec3_origin,
                            );
                            G_PlayEffectID(
                                G_EffectIndex("chunks/r5d2head"),
                                &(*self_).r.currentOrigin,
                                &vec3_origin,
                            );
                        }

                        //self->s.powerups |= ( 1 << PW_SHOCKED );
                        //self->client->ps.powerups[PW_SHOCKED] = level.time + 3000;
                        (*(*self_).client).ps.electrifyTime = (*addr_of!(level)).time + 3000;

                        TIMER_Set(self_, c"droidsmoketotal".as_ptr(), 5000);
                        TIMER_Set(self_, c"droidspark".as_ptr(), 100);
                        (*(*self_).NPC).localState = LSTATE_SPINNING;
                    }
                }
            }
            // Just give him normal pain for a little while
            else {
                anim = (*(*self_).client).ps.legsAnim;

                let anim = if anim == BOTH_STAND2 {
                    // On two legs?
                    BOTH_PAIN1
                } else {
                    // On three legs
                    BOTH_PAIN2
                };

                NPC_SetAnim(
                    self_,
                    SETANIM_BOTH,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                // Spin around in pain
                (*(*self_).NPC).localState = LSTATE_SPINNING;
                TIMER_Set(self_, c"roam".as_ptr(), Q_irand(1000, 2000));
            }
        }
    } else if (*(*self_).client).NPC_class == CLASS_MOUSE {
        if mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT {
            (*(*self_).NPC).localState = LSTATE_SPINNING;
            //self->s.powerups |= ( 1 << PW_SHOCKED );
            //self->client->ps.powerups[PW_SHOCKED] = level.time + 3000;
            (*(*self_).client).ps.electrifyTime = (*addr_of!(level)).time + 3000;
        } else {
            (*(*self_).NPC).localState = LSTATE_BACKINGUP;
        }

        (*(*self_).NPC).scriptFlags &= !SCF_LOOK_FOR_ENEMIES;
    } else if (*(*self_).client).NPC_class == CLASS_R2D2 {
        pain_chance = NPC_GetPainChance(self_, damage);

        if mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT || random() < pain_chance
        // Spin around in pain? Demp2 always does this
        {
            // Health is between 0-30 or was hit by a DEMP2 so pop his head
            if (*self_).s.m_iVehicleNum == 0
                && ((*self_).health < 30 || mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT)
            {
                if (*self_).spawnflags & 2 == 0
                // Doesn't have to ALWAYSDIE
                {
                    if (*(*self_).NPC).localState != LSTATE_SPINNING
                        && trap::G2API_GetSurfaceRenderStatus((*self_).ghoul2, 0, c"head".as_ptr())
                            == 0
                    {
                        NPC_SetSurfaceOnOff(self_, c"head".as_ptr(), TURN_OFF);

                        if (*(*self_).client).ps.m_iVehicleNum != 0 {
                            let mut up: vec3_t = [0.0; 3];
                            AngleVectors(&(*self_).r.currentAngles, None, None, Some(&mut up));
                            G_PlayEffectID(
                                G_EffectIndex("chunks/r2d2head_veh"),
                                &(*self_).r.currentOrigin,
                                &up,
                            );
                        } else {
                            G_PlayEffectID(
                                G_EffectIndex("small_chunks"),
                                &(*self_).r.currentOrigin,
                                &vec3_origin,
                            );
                            G_PlayEffectID(
                                G_EffectIndex("chunks/r2d2head"),
                                &(*self_).r.currentOrigin,
                                &vec3_origin,
                            );
                        }

                        //self->s.powerups |= ( 1 << PW_SHOCKED );
                        //self->client->ps.powerups[PW_SHOCKED] = level.time + 3000;
                        (*(*self_).client).ps.electrifyTime = (*addr_of!(level)).time + 3000;

                        TIMER_Set(self_, c"droidsmoketotal".as_ptr(), 5000);
                        TIMER_Set(self_, c"droidspark".as_ptr(), 100);
                        (*(*self_).NPC).localState = LSTATE_SPINNING;
                    }
                }
            }
            // Just give him normal pain for a little while
            else {
                anim = (*(*self_).client).ps.legsAnim;

                let anim = if anim == BOTH_STAND2 {
                    // On two legs?
                    BOTH_PAIN1
                } else {
                    // On three legs
                    BOTH_PAIN2
                };

                NPC_SetAnim(
                    self_,
                    SETANIM_BOTH,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                // Spin around in pain
                (*(*self_).NPC).localState = LSTATE_SPINNING;
                TIMER_Set(self_, c"roam".as_ptr(), Q_irand(1000, 2000));
            }
        }
    } else if (*(*self_).client).NPC_class == CLASS_INTERROGATOR
        && (mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT)
        && !other.is_null()
    {
        let mut dir: vec3_t = [0.0; 3];

        VectorSubtract(
            &(*self_).r.currentOrigin,
            &(*other).r.currentOrigin,
            &mut dir,
        );
        VectorNormalize(&mut dir);

        VectorMA(
            &(*(*self_).client).ps.velocity,
            550.0,
            &dir,
            &mut (*(*self_).client).ps.velocity,
        );
        (*(*self_).client).ps.velocity[2] -= 127.0;
    }

    NPC_Pain(self_, attacker, damage);
}

/*
-------------------------
Droid_Pain
-------------------------
*/
pub unsafe fn Droid_Pain() {
    if TIMER_Done(NPC, c"droidpain".as_ptr()) != QFALSE
    //He's done jumping around
    {
        (*NPCInfo).localState = LSTATE_NONE;
    }
}

/*
-------------------------
NPC_Mouse_Precache
-------------------------
*/
pub unsafe fn NPC_Mouse_Precache() {
    let mut i: c_int;

    i = 1;
    while i < 4 {
        G_SoundIndex(&format!("sound/chars/mouse/misc/mousego{}.wav", i));
        i += 1;
    }

    G_EffectIndex("env/small_explode");
    G_SoundIndex("sound/chars/mouse/misc/death1");
    G_SoundIndex("sound/chars/mouse/misc/mouse_lp");
}

/*
-------------------------
NPC_R5D2_Precache
-------------------------
*/
pub unsafe fn NPC_R5D2_Precache() {
    let mut i: c_int;

    i = 1;
    while i < 5 {
        G_SoundIndex(&format!("sound/chars/r5d2/misc/r5talk{}.wav", i));
        i += 1;
    }
    //G_SoundIndex( "sound/chars/r5d2/misc/falling1.wav" );
    G_SoundIndex("sound/chars/mark2/misc/mark2_explo"); // ??
    G_SoundIndex("sound/chars/r2d2/misc/r2_move_lp2.wav");
    G_EffectIndex("env/med_explode");
    G_EffectIndex("volumetric/droid_smoke");
    G_EffectIndex("sparks/spark");
    G_EffectIndex("chunks/r5d2head");
    G_EffectIndex("chunks/r5d2head_veh");
}

/*
-------------------------
NPC_R2D2_Precache
-------------------------
*/
pub unsafe fn NPC_R2D2_Precache() {
    let mut i: c_int;

    i = 1;
    while i < 4 {
        G_SoundIndex(&format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", i));
        i += 1;
    }
    //G_SoundIndex( "sound/chars/r2d2/misc/falling1.wav" );
    G_SoundIndex("sound/chars/mark2/misc/mark2_explo"); // ??
    G_SoundIndex("sound/chars/r2d2/misc/r2_move_lp.wav");
    G_EffectIndex("env/med_explode");
    G_EffectIndex("volumetric/droid_smoke");
    G_EffectIndex("sparks/spark");
    G_EffectIndex("chunks/r2d2head");
    G_EffectIndex("chunks/r2d2head_veh");
}

/*
-------------------------
NPC_Gonk_Precache
-------------------------
*/
pub unsafe fn NPC_Gonk_Precache() {
    G_SoundIndex("sound/chars/gonk/misc/gonktalk1.wav");
    G_SoundIndex("sound/chars/gonk/misc/gonktalk2.wav");

    G_SoundIndex("sound/chars/gonk/misc/death1.wav");
    G_SoundIndex("sound/chars/gonk/misc/death2.wav");
    G_SoundIndex("sound/chars/gonk/misc/death3.wav");

    G_EffectIndex("env/med_explode");
}

/*
-------------------------
NPC_Protocol_Precache
-------------------------
*/
pub unsafe fn NPC_Protocol_Precache() {
    G_SoundIndex("sound/chars/mark2/misc/mark2_explo");
    G_EffectIndex("env/med_explode");
}

/*
-------------------------
NPC_BSDroid_Default
-------------------------
*/
pub unsafe fn NPC_BSDroid_Default() {
    if (*NPCInfo).localState == LSTATE_SPINNING {
        Droid_Spin();
    } else if (*NPCInfo).localState == LSTATE_PAIN {
        Droid_Pain();
    } else if (*NPCInfo).localState == LSTATE_DROP {
        NPC_UpdateAngles(QTRUE, QTRUE);
        ucmd.upmove = (crandom() * 64.0) as i8;
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        Droid_Patrol();
    } else {
        Droid_Run();
    }
}
