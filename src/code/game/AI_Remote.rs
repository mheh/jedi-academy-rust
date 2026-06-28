// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

#[allow(non_snake_case)]

use crate::b_local::*;
use crate::g_nav::*;
use core::ffi::*;

// Forward declarations
extern "C" {
    pub fn CreateMissile(
        org: *const [c_float; 3],
        dir: *const [c_float; 3],
        vel: c_float,
        life: c_int,
        owner: *mut gentity_t,
    ) -> *mut gentity_t;
    fn Remote_Strafe();
}

const VELOCITY_DECAY: c_float = 0.85;

// Local state enums
const LSTATE_NONE: c_int = 0;

extern "C" {
    fn Remote_Idle();
}

pub extern "C" fn NPC_Remote_Precache() {
    unsafe {
        G_SoundIndex(b"sound/chars/remote/misc/fire.wav\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/remote/misc/hiss.wav\0".as_ptr() as *const c_char);
        G_EffectIndex(b"env/small_explode\0".as_ptr() as *const c_char);
    }
}

/*
-------------------------
NPC_Remote_Pain
-------------------------
*/
pub extern "C" fn NPC_Remote_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const [c_float; 3],
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    unsafe {
        SaveNPCGlobals();
        SetNPCGlobals(self_);
        Remote_Strafe();
        RestoreNPCGlobals();

        NPC_Pain(self_, inflictor, other, point, damage, mod_);
    }
}

/*
-------------------------
Remote_MaintainHeight
-------------------------
*/
pub extern "C" fn Remote_MaintainHeight() {
    unsafe {
        let mut dif: c_float;

        // Update our angles regardless
        NPC_UpdateAngles(qtrue, qtrue);

        if (*(*NPC).client).ps.velocity[2] != 0.0 {
            (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

            if ((*(*NPC).client).ps.velocity[2]).abs() < 2.0 {
                (*(*NPC).client).ps.velocity[2] = 0.0;
            }
        }
        // If we have an enemy, we should try to hover at or a little below enemy eye level
        if !(*NPC).enemy.is_null() {
            if TIMER_Done(NPC, b"heightChange\0".as_ptr() as *const c_char) {
                TIMER_Set(
                    NPC,
                    b"heightChange\0".as_ptr() as *const c_char,
                    Q_irand(1000, 3000),
                );

                // Find the height difference
                dif = ((*(*(*NPC).enemy)).currentOrigin[2]
                    + Q_irand(0, (*(*NPC).enemy).maxs[2] as c_int + 8) as c_float)
                    - (*NPC).currentOrigin[2];

                // cap to prevent dramatic height shifts
                if dif.abs() > 2.0 {
                    if dif.abs() > 24.0 {
                        dif = if dif < 0.0 { -24.0 } else { 24.0 };
                    }
                    dif *= 10.0;
                    (*(*NPC).client).ps.velocity[2] =
                        ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
                    (*NPC).fx_time = level.time;
                    G_Sound(
                        NPC,
                        G_SoundIndex(b"sound/chars/remote/misc/hiss.wav\0".as_ptr() as *const c_char),
                    );
                }
            }
        } else {
            let mut goal: *mut gentity_t = std::ptr::null_mut();

            if !(*NPCInfo).goalEntity.is_null() {
                // Is there a goal?
                goal = (*NPCInfo).goalEntity;
            } else {
                goal = (*NPCInfo).lastGoalEntity;
            }
            if !goal.is_null() {
                dif = (*goal).currentOrigin[2] - (*NPC).currentOrigin[2];

                if dif.abs() > 24.0 {
                    dif = if dif < 0.0 { -24.0 } else { 24.0 };
                    (*(*NPC).client).ps.velocity[2] =
                        ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
                }
            }
        }

        // Apply friction
        if (*(*NPC).client).ps.velocity[0] != 0.0 {
            (*(*NPC).client).ps.velocity[0] *= VELOCITY_DECAY;

            if ((*(*NPC).client).ps.velocity[0]).abs() < 1.0 {
                (*(*NPC).client).ps.velocity[0] = 0.0;
            }
        }

        if (*(*NPC).client).ps.velocity[1] != 0.0 {
            (*(*NPC).client).ps.velocity[1] *= VELOCITY_DECAY;

            if ((*(*NPC).client).ps.velocity[1]).abs() < 1.0 {
                (*(*NPC).client).ps.velocity[1] = 0.0;
            }
        }
    }
}

const REMOTE_STRAFE_VEL: c_int = 256;
const REMOTE_STRAFE_DIS: c_int = 200;
const REMOTE_UPWARD_PUSH: c_int = 32;

/*
-------------------------
Remote_Strafe
-------------------------
*/
pub extern "C" fn Remote_Strafe() {
    unsafe {
        let mut dir: c_int;
        let mut end: [c_float; 3] = [0.0; 3];
        let mut right: [c_float; 3] = [0.0; 3];
        let mut tr: trace_t = std::mem::zeroed();

        AngleVectors(
            (*(*NPC).client).renderInfo.eyeAngles,
            std::ptr::null_mut(),
            right.as_mut_ptr(),
            std::ptr::null_mut(),
        );

        // Pick a random strafe direction, then check to see if doing a strafe would be
        //	reasonable valid
        dir = if (rand() & 1) != 0 { -1 } else { 1 };
        VectorMA(
            (*NPC).currentOrigin,
            (REMOTE_STRAFE_DIS * dir) as c_float,
            right.as_ptr(),
            end.as_mut_ptr(),
        );

        gi.trace(
            &mut tr,
            (*NPC).currentOrigin,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            end.as_ptr(),
            (*NPC).s.number,
            MASK_SOLID,
        );

        // Close enough
        if tr.fraction > 0.9 {
            VectorMA(
                (*(*NPC).client).ps.velocity,
                (REMOTE_STRAFE_VEL * dir) as c_float,
                right.as_ptr(),
                (*(*NPC).client).ps.velocity,
            );

            G_Sound(
                NPC,
                G_SoundIndex(b"sound/chars/remote/misc/hiss.wav\0".as_ptr() as *const c_char),
            );

            // Add a slight upward push
            (*(*NPC).client).ps.velocity[2] += REMOTE_UPWARD_PUSH as c_float;

            // Set the strafe start time so we can do a controlled roll
            (*NPC).fx_time = level.time;
            (*NPCInfo).standTime = level.time + 3000 + (random() * 500.0) as c_int;
        }
    }
}

const REMOTE_FORWARD_BASE_SPEED: c_int = 10;
const REMOTE_FORWARD_MULTIPLIER: c_int = 5;

/*
-------------------------
Remote_Hunt
-------------------------
*/
pub extern "C" fn Remote_Hunt(visible: qboolean, advance: qboolean, retreat: qboolean) {
    unsafe {
        let mut distance: c_float;
        let mut speed: c_float;
        let mut forward: [c_float; 3] = [0.0; 3];

        // If we're not supposed to stand still, pursue the player
        if (*NPCInfo).standTime < level.time {
            // Only strafe when we can see the player
            if visible {
                Remote_Strafe();
                return;
            }
        }

        // If we don't want to advance, stop here
        if advance == qfalse && visible == qtrue {
            return;
        }

        // Only try and navigate if the player is visible
        if visible == qfalse {
            // Move towards our goal
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = 12;

            NPC_MoveToGoal(qtrue);
            return;
        } else {
            VectorSubtract(
                (*(*NPC).enemy).currentOrigin,
                (*NPC).currentOrigin,
                forward.as_mut_ptr(),
            );
            distance = VectorNormalize(forward.as_mut_ptr());
        }

        speed = (REMOTE_FORWARD_BASE_SPEED + REMOTE_FORWARD_MULTIPLIER * (*g_spskill).integer)
            as c_float;
        if retreat == qtrue {
            speed *= -1.0;
        }
        VectorMA(
            (*(*NPC).client).ps.velocity,
            speed,
            forward.as_ptr(),
            (*(*NPC).client).ps.velocity,
        );
    }
}

/*
-------------------------
Remote_Fire
-------------------------
*/
pub extern "C" fn Remote_Fire() {
    unsafe {
        let mut delta1: [c_float; 3] = [0.0; 3];
        let mut enemy_org1: [c_float; 3] = [0.0; 3];
        let mut muzzle1: [c_float; 3] = [0.0; 3];
        let mut angleToEnemy1: [c_float; 3] = [0.0; 3];
        static mut forward: [c_float; 3] = [0.0; 3];
        static mut vright: [c_float; 3] = [0.0; 3];
        static mut up: [c_float; 3] = [0.0; 3];
        static mut muzzle: [c_float; 3] = [0.0; 3];
        let mut missile: *mut gentity_t;

        CalcEntitySpot((*NPC).enemy, SPOT_HEAD, enemy_org1.as_mut_ptr());
        VectorCopy((*NPC).currentOrigin, muzzle1.as_mut_ptr());

        VectorSubtract(
            enemy_org1.as_ptr(),
            muzzle1.as_ptr(),
            delta1.as_mut_ptr(),
        );

        vectoangles(delta1.as_ptr(), angleToEnemy1.as_mut_ptr());
        AngleVectors(
            angleToEnemy1.as_ptr(),
            forward.as_mut_ptr(),
            vright.as_mut_ptr(),
            up.as_mut_ptr(),
        );

        missile = CreateMissile((*NPC).currentOrigin, forward.as_ptr(), 1000.0, 10000, NPC);

        G_PlayEffect(
            b"bryar/muzzle_flash\0".as_ptr() as *const c_char,
            (*NPC).currentOrigin,
            forward.as_ptr(),
        );

        (*missile).classname = b"briar\0".as_ptr() as *const c_char;
        (*missile).s.weapon = WP_BRYAR_PISTOL;

        (*missile).damage = 10;
        (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
        (*missile).methodOfDeath = MOD_ENERGY;
        (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    }
}

/*
-------------------------
Remote_Ranged
-------------------------
*/
pub extern "C" fn Remote_Ranged(visible: qboolean, advance: qboolean, retreat: qboolean) {
    unsafe {
        if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) {
            // Attack?
            TIMER_Set(
                NPC,
                b"attackDelay\0".as_ptr() as *const c_char,
                Q_irand(500, 3000),
            );
            Remote_Fire();
        }

        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            Remote_Hunt(visible, advance, retreat);
        }
    }
}

const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

const MIN_DISTANCE: c_int = 80;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

/*
-------------------------
Remote_Attack
-------------------------
*/
pub extern "C" fn Remote_Attack() {
    unsafe {
        if TIMER_Done(NPC, b"spin\0".as_ptr() as *const c_char) {
            TIMER_Set(NPC, b"spin\0".as_ptr() as *const c_char, Q_irand(250, 1500));
            (*NPCInfo).desiredYaw += Q_irand(-200, 200);
        }
        // Always keep a good height off the ground
        Remote_MaintainHeight();

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt() == qfalse {
            Remote_Idle();
            return;
        }

        // Rate our distance to the target, and our visibilty
        let distance: c_float =
            DistanceHorizontalSquared((*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin)
                as c_float;
        // distance_e	distRate	= ( distance > MIN_MELEE_RANGE_SQR ) ? DIST_LONG : DIST_MELEE;
        let visible: qboolean = NPC_ClearLOS((*NPC).enemy);
        let idealDist: c_float =
            (MIN_DISTANCE_SQR as c_float + (MIN_DISTANCE_SQR as c_float * Q_flrand(0.0, 1.0)))
                as c_float;
        let advance: qboolean = (distance > idealDist * 1.25) as qboolean;
        let retreat: qboolean = (distance < idealDist * 0.75) as qboolean;

        // If we cannot see our target, move to see it
        if visible == qfalse {
            if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
                Remote_Hunt(visible, advance, retreat);
                return;
            }
        }

        Remote_Ranged(visible, advance, retreat);
    }
}

/*
-------------------------
Remote_Idle
-------------------------
*/
pub extern "C" fn Remote_Idle() {
    unsafe {
        Remote_MaintainHeight();

        NPC_BSIdle();
    }
}

/*
-------------------------
Remote_Patrol
-------------------------
*/
pub extern "C" fn Remote_Patrol() {
    unsafe {
        Remote_MaintainHeight();

        // If we have somewhere to go, then do that
        if (*NPC).enemy.is_null() {
            if UpdateGoal() {
                // start loop sound once we move
                ucmd.buttons |= BUTTON_WALKING;
                NPC_MoveToGoal(qtrue);
            }
        }

        NPC_UpdateAngles(qtrue, qtrue);
    }
}

/*
-------------------------
NPC_BSRemote_Default
-------------------------
*/
pub extern "C" fn NPC_BSRemote_Default() {
    unsafe {
        if !(*NPC).enemy.is_null() {
            Remote_Attack();
        } else if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            Remote_Patrol();
        } else {
            Remote_Idle();
        }
    }
}
