// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// #include "g_headers.h"
// #include "b_local.h"
// #include "g_nav.h"

// Type stubs for external engine types
#[repr(C)]
pub struct gentity_t {
    // Stub: actual structure defined elsewhere
}

#[repr(C)]
pub struct gNPC_t {
    // Stub: actual structure defined elsewhere
}

#[repr(C)]
pub struct usercmd_t {
    // Stub: actual structure defined elsewhere
    pub upmove: i32,
}

#[repr(C)]
pub struct gameImport_t {
    // Stub: actual structure defined elsewhere
}

#[repr(C)]
pub struct level_locals_t {
    // Stub: actual structure defined elsewhere
    pub time: i32,
}

#[repr(C)]
pub struct cvar_t {
    // Stub: actual structure defined elsewhere
    pub integer: i32,
}

#[repr(C)]
pub struct trace_t {
    // Stub: actual structure defined elsewhere
    pub fraction: f32,
}

// External functions from the engine (forward declarations as stubs)
extern "C" {
    fn DeathFX(ent: *mut gentity_t);
    fn G_SoundIndex(filename: *const i8) -> i32;
    fn G_EffectIndex(filename: *const i8) -> i32;
    fn Q_irand(low: i32, high: i32) -> i32;
    fn AngleNormalize360(angle: f32) -> f32;
    fn NPC_UpdateAngles(doPitch: i32, doYaw: i32);
    fn NPC_FaceEnemy(doPitch: i32);
    fn Interrogator_Strafe();
    fn NPC_MoveToGoal(allowRun: i32);
    fn VectorSubtract(vec_a: [f32; 3], vec_b: [f32; 3], out: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn VectorMA(veca: [f32; 3], scale: f32, vecb: *const [f32; 3], vecc: *mut [f32; 3]);
    fn AngleVectors(angles: [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    fn DistanceHorizontalSquared(p1: [f32; 3], p2: [f32; 3]) -> f64;
    fn NPC_ClearLOS(ent: *mut gentity_t) -> i32;
    fn NPC_CheckEnemyExt() -> i32;
    fn NPC_CheckPlayerTeamStealth() -> i32;
    fn G_Damage(target: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: i32, dflags: i32, mod_: i32, mod2: i32, hitLoc: i32);
    fn G_TempEntity(origin: [f32; 3], event: i32) -> *mut gentity_t;
    fn G_Sound(ent: *mut gentity_t, index: i32);
    fn G_SoundOnEnt(ent: *mut gentity_t, channel: i32, soundfile: *const i8);
    fn va(fmt: *const i8, ...) -> *const i8;
    fn NPC_BSIdle();
    fn TIMER_Done(ent: *mut gentity_t, label: *const i8) -> i32;
    fn TIMER_Set(ent: *mut gentity_t, label: *const i8, duration: i32);
    fn rand() -> i32;
    fn random() -> f32;
    // gi function pointers accessed indirectly
    fn G2API_SetBoneAnglesIndex(ghoul2: *mut u8, boneIndex: i32, angles: [f32; 3], flags: i32, up: i32, right: i32, forward: i32, unused: *mut u8);
}

// Wrapper for gi.trace call
#[allow(non_snake_case)]
unsafe fn gi_trace(tr: *mut trace_t, start: [f32; 3], mins: *const u8, maxs: *const u8, end: [f32; 3], entNum: i32, mask: i32) {
    // This wraps gi.trace - in actual implementation would call through gi struct
    extern "C" {
        fn gi_trace_impl(tr: *mut trace_t, start: [f32; 3], mins: *const u8, maxs: *const u8, end: [f32; 3], entNum: i32, mask: i32);
    }
    gi_trace_impl(tr, start, mins, maxs, end, entNum, mask);
}

// External globals
extern "C" {
    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut gNPC_t;
    static mut ucmd: usercmd_t;
    static mut gi: gameImport_t;
    static mut level: level_locals_t;
    static mut g_spskill: *mut cvar_t;
}

// Constants for game definitions
const LSTATE_BLADESTOP: i32 = 0;
const LSTATE_BLADEUP: i32 = 1;
const LSTATE_BLADEDOWN: i32 = 2;

// Game engine constants
const qtrue: i32 = 1;
const qfalse: i32 = 0;

const MT_WALK: i32 = 0;
const BONE_ANGLES_POSTMULT: i32 = 0;
const POSITIVE_X: i32 = 0;
const NEGATIVE_Y: i32 = 0;
const NEGATIVE_Z: i32 = 0;

const MASK_SOLID: i32 = 0;
const EV_DRUGGED: i32 = 0;
const DAMAGE_NO_KNOCKBACK: i32 = 0;
const MOD_MELEE: i32 = 0;
const SCF_CHASE_ENEMIES: i32 = 0;
const CHAN_AUTO: i32 = 0;

// Type aliases for qboolean compatibility
type qboolean = i32;

/*
-------------------------
NPC_Interrogator_Precache
-------------------------
*/
fn NPC_Interrogator_Precache(self_: *mut gentity_t) -> () {
    G_SoundIndex("sound/chars/interrogator/misc/torture_droid_lp");
    G_SoundIndex("sound/chars/mark1/misc/anger.wav");
    G_SoundIndex("sound/chars/probe/misc/talk");
    G_SoundIndex("sound/chars/interrogator/misc/torture_droid_inject");
    G_SoundIndex("sound/chars/interrogator/misc/int_droid_explo");
    G_EffectIndex("explosions/droidexplosion1");
}

/*
-------------------------
Interrogator_die
-------------------------
*/
fn Interrogator_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: i32,
    mod_: i32,
    dFlags: i32,
    hitLoc: i32,
) -> () {
    unsafe {
        (*self_).client.ps.velocity[2] = -100;
        /*
        self->locationDamage[HL_NONE] += damage;
        if (self->locationDamage[HL_NONE] > 40)
        {
            DeathFX(self);
            self->client->ps.eFlags |= EF_NODRAW;
            self->contents = CONTENTS_CORPSE;
        }
        else
        */
        {
            (*self_).client.moveType = MT_WALK;
            (*self_).client.ps.velocity[0] = Q_irand(-10, -20) as f32;
            (*self_).client.ps.velocity[1] = Q_irand(-10, -20) as f32;
            (*self_).client.ps.velocity[2] = -100.0;
        }
        //self->takedamage = qfalse;
        //self->client->ps.eFlags |= EF_NODRAW;
        //self->contents = 0;
    }
}

/*
-------------------------
Interrogator_PartsMove
-------------------------
*/
fn Interrogator_PartsMove() -> () {
    unsafe {
        // Syringe
        if TIMER_Done(NPC, "syringeDelay") != 0 {
            (*NPC).pos1[1] = AngleNormalize360((*NPC).pos1[1]);

            if ((*NPC).pos1[1] < 60.0) || ((*NPC).pos1[1] > 300.0) {
                (*NPC).pos1[1] += Q_irand(-20, 20) as f32; // Pitch
            } else if (*NPC).pos1[1] > 180.0 {
                (*NPC).pos1[1] = Q_irand(300, 360) as f32; // Pitch
            } else {
                (*NPC).pos1[1] = Q_irand(0, 60) as f32; // Pitch
            }

            G2API_SetBoneAnglesIndex(
                &mut (*NPC).ghoul2[(*NPC).playerModel] as *mut u8,
                (*NPC).genericBone1,
                (*NPC).pos1,
                BONE_ANGLES_POSTMULT,
                POSITIVE_X,
                NEGATIVE_Y,
                NEGATIVE_Z,
                std::ptr::null_mut(),
            );
            TIMER_Set(NPC, "syringeDelay", Q_irand(100, 1000));
        }

        // Scalpel
        if TIMER_Done(NPC, "scalpelDelay") != 0 {
            // Change pitch
            if (*NPCInfo).localState == LSTATE_BLADEDOWN {
                // Blade is moving down
                (*NPC).pos2[0] -= 30.0;
                if (*NPC).pos2[0] < 180.0 {
                    (*NPC).pos2[0] = 180.0;
                    (*NPCInfo).localState = LSTATE_BLADEUP; // Make it move up
                }
            } else {
                // Blade is coming back up
                (*NPC).pos2[0] += 30.0;
                if (*NPC).pos2[0] >= 360.0 {
                    (*NPC).pos2[0] = 360.0;
                    (*NPCInfo).localState = LSTATE_BLADEDOWN; // Make it move down
                    TIMER_Set(NPC, "scalpelDelay", Q_irand(100, 1000));
                }
            }

            (*NPC).pos2[0] = AngleNormalize360((*NPC).pos2[0]);
            G2API_SetBoneAnglesIndex(
                &mut (*NPC).ghoul2[(*NPC).playerModel] as *mut u8,
                (*NPC).genericBone2,
                (*NPC).pos2,
                BONE_ANGLES_POSTMULT,
                POSITIVE_X,
                NEGATIVE_Y,
                NEGATIVE_Z,
                std::ptr::null_mut(),
            );
        }

        // Claw
        (*NPC).pos3[1] += Q_irand(10, 30) as f32;
        (*NPC).pos3[1] = AngleNormalize360((*NPC).pos3[1]);
        G2API_SetBoneAnglesIndex(
            &mut (*NPC).ghoul2[(*NPC).playerModel] as *mut u8,
            (*NPC).genericBone3,
            (*NPC).pos3,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X,
            NEGATIVE_Y,
            NEGATIVE_Z,
            std::ptr::null_mut(),
        );
    }
}

const VELOCITY_DECAY: f32 = 0.85;
const HUNTER_UPWARD_PUSH: i32 = 2;

/*
-------------------------
Interrogator_MaintainHeight
-------------------------
*/
fn Interrogator_MaintainHeight() -> () {
    unsafe {
        let mut dif: f32;
        //	vec3_t	endPos;
        //	trace_t	trace;

        (*NPC).s.loopSound = G_SoundIndex("sound/chars/interrogator/misc/torture_droid_lp");
        // Update our angles regardless
        NPC_UpdateAngles(qtrue, qtrue);

        // If we have an enemy, we should try to hover at about enemy eye level
        if !(*NPC).enemy.is_null() {
            // Find the height difference
            dif = ((*(*NPC).enemy).currentOrigin[2] + (*(*NPC).enemy).maxs[2])
                - (*NPC).currentOrigin[2];

            // cap to prevent dramatic height shifts
            if dif.abs() > 2.0 {
                if dif.abs() > 16.0 {
                    dif = if dif < 0.0 { -16.0 } else { 16.0 };
                }

                (*NPC).client.ps.velocity[2] = ((*NPC).client.ps.velocity[2] + dif) / 2.0;
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
                    ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
                } else {
                    if (*NPC).client.ps.velocity[2] != 0.0 {
                        (*NPC).client.ps.velocity[2] *= VELOCITY_DECAY;

                        if (*NPC).client.ps.velocity[2].abs() < 2.0 {
                            (*NPC).client.ps.velocity[2] = 0.0;
                        }
                    }
                }
            }
            // Apply friction
            else if (*NPC).client.ps.velocity[2] != 0.0 {
                (*NPC).client.ps.velocity[2] *= VELOCITY_DECAY;

                if (*NPC).client.ps.velocity[2].abs() < 1.0 {
                    (*NPC).client.ps.velocity[2] = 0.0;
                }
            }
        }

        // Apply friction
        if (*NPC).client.ps.velocity[0] != 0.0 {
            (*NPC).client.ps.velocity[0] *= VELOCITY_DECAY;

            if (*NPC).client.ps.velocity[0].abs() < 1.0 {
                (*NPC).client.ps.velocity[0] = 0.0;
            }
        }

        if (*NPC).client.ps.velocity[1] != 0.0 {
            (*NPC).client.ps.velocity[1] *= VELOCITY_DECAY;

            if (*NPC).client.ps.velocity[1].abs() < 1.0 {
                (*NPC).client.ps.velocity[1] = 0.0;
            }
        }
    }
}

const HUNTER_STRAFE_VEL: i32 = 32;
const HUNTER_STRAFE_DIS: i32 = 200;

/*
-------------------------
Interrogator_Strafe
-------------------------
*/
fn Interrogator_Strafe() -> () {
    unsafe {
        let mut dir: i32;
        let mut end: [f32; 3] = [0.0; 3];
        let mut right: [f32; 3] = [0.0; 3];
        let mut tr: trace_t;
        let mut dif: f32;

        AngleVectors(
            (*NPC).client.renderInfo.eyeAngles,
            std::ptr::null_mut(),
            &mut right,
            std::ptr::null_mut(),
        );

        // Pick a random strafe direction, then check to see if doing a strafe would be
        //	reasonable valid
        dir = if (rand() & 1) != 0 { -1 } else { 1 };
        VectorMA(
            (*NPC).currentOrigin,
            (HUNTER_STRAFE_DIS as f32 * dir as f32),
            &right,
            &mut end,
        );

        gi_trace(
            &mut tr,
            (*NPC).currentOrigin,
            std::ptr::null(),
            std::ptr::null(),
            end,
            (*NPC).s.number,
            MASK_SOLID,
        );

        // Close enough
        if tr.fraction > 0.9 {
            VectorMA(
                (*NPC).client.ps.velocity,
                (HUNTER_STRAFE_VEL as f32 * dir as f32),
                &right,
                &mut (*NPC).client.ps.velocity,
            );

            // Add a slight upward push
            if !(*NPC).enemy.is_null() {
                // Find the height difference
                dif = ((*(*NPC).enemy).currentOrigin[2] + 32.0) - (*NPC).currentOrigin[2];

                // cap to prevent dramatic height shifts
                if dif.abs() > 8.0 {
                    dif = if dif < 0.0 {
                        -(HUNTER_UPWARD_PUSH as f32)
                    } else {
                        HUNTER_UPWARD_PUSH as f32
                    };
                }

                (*NPC).client.ps.velocity[2] += dif;
            }

            // Set the strafe start time
            (*NPC).fx_time = level.time;
            (*NPCInfo).standTime = level.time + 3000 + (random() * 500.0) as i32;
        }
    }
}

/*
-------------------------
Interrogator_Hunt
-------------------------
*/

const HUNTER_FORWARD_BASE_SPEED: i32 = 10;
const HUNTER_FORWARD_MULTIPLIER: i32 = 2;

fn Interrogator_Hunt(visible: qboolean, advance: qboolean) -> () {
    unsafe {
        let mut distance: f32;
        let mut speed: f32;
        let mut forward: [f32; 3] = [0.0; 3];

        Interrogator_PartsMove();

        NPC_FaceEnemy(qfalse);

        //If we're not supposed to stand still, pursue the player
        if (*NPCInfo).standTime < level.time {
            // Only strafe when we can see the player
            if visible != qfalse {
                Interrogator_Strafe();
                if (*NPCInfo).standTime > level.time {
                    //successfully strafed
                    return;
                }
            }
        }

        //If we don't want to advance, stop here
        if advance == qfalse {
            return;
        }

        //Only try and navigate if the player is visible
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
                &mut forward,
            );
            distance = VectorNormalize(&mut forward);
        }

        speed = (HUNTER_FORWARD_BASE_SPEED as f32)
            + (HUNTER_FORWARD_MULTIPLIER as f32 * (*g_spskill).integer as f32);
        VectorMA(
            (*NPC).client.ps.velocity,
            speed,
            &forward,
            &mut (*NPC).client.ps.velocity,
        );
    }
}

const MIN_DISTANCE: i32 = 64;

/*
-------------------------
Interrogator_Melee
-------------------------
*/
fn Interrogator_Melee(visible: qboolean, advance: qboolean) -> () {
    unsafe {
        if TIMER_Done(NPC, "attackDelay") != 0 {
            // Attack?
            // Make sure that we are within the height range before we allow any damage to happen
            if (*NPC).currentOrigin[2] >= (*(*NPC).enemy).currentOrigin[2] + (*(*NPC).enemy).mins[2]
                && (*NPC).currentOrigin[2] + (*NPC).mins[2] + 8.0
                    < (*(*NPC).enemy).currentOrigin[2] + (*(*NPC).enemy).maxs[2]
            {
                TIMER_Set(NPC, "attackDelay", Q_irand(500, 3000));
                G_Damage(
                    (*NPC).enemy,
                    NPC,
                    NPC,
                    0,
                    0,
                    2,
                    DAMAGE_NO_KNOCKBACK,
                    MOD_MELEE,
                );

                (*(*NPC).enemy).client.poisonDamage = 18;
                (*(*NPC).enemy).client.poisonTime = level.time + 1000;

                // Drug our enemy up and do the wonky vision thing
                let mut tent: *mut gentity_t =
                    G_TempEntity((*(*NPC).enemy).currentOrigin, EV_DRUGGED);
                (*tent).owner = (*NPC).enemy;

                G_Sound(
                    NPC,
                    G_SoundIndex("sound/chars/interrogator/misc/torture_droid_inject.mp3"),
                );
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            Interrogator_Hunt(visible, advance);
        }
    }
}

/*
-------------------------
Interrogator_Attack
-------------------------
*/
fn Interrogator_Attack() -> () {
    unsafe {
        // Always keep a good height off the ground
        Interrogator_MaintainHeight();

        //randomly talk
        if TIMER_Done(NPC, "patrolNoise") != 0 {
            if TIMER_Done(NPC, "angerNoise") != 0 {
                G_SoundOnEnt(
                    NPC,
                    CHAN_AUTO,
                    va("sound/chars/probe/misc/talk.wav", Q_irand(1, 3)),
                );

                TIMER_Set(NPC, "patrolNoise", Q_irand(4000, 10000));
            }
        }

        // If we don't have an enemy, just idle
        if NPC_CheckEnemyExt() == qfalse {
            Interrogator_Idle();
            return;
        }

        // Rate our distance to the target, and our visibilty
        let distance: f32 = DistanceHorizontalSquared((*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin) as f32;
        let visible: qboolean = NPC_ClearLOS((*NPC).enemy);
        let mut advance: qboolean = if distance > ((MIN_DISTANCE * MIN_DISTANCE) as f32) {
            qtrue
        } else {
            qfalse
        };

        if visible == qfalse {
            advance = qtrue;
        }
        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            Interrogator_Hunt(visible, advance);
        }

        NPC_FaceEnemy(qtrue);

        if advance == qfalse {
            Interrogator_Melee(visible, advance);
        }
    }
}

/*
-------------------------
Interrogator_Idle
-------------------------
*/
fn Interrogator_Idle() -> () {
    unsafe {
        if NPC_CheckPlayerTeamStealth() != qfalse {
            G_SoundOnEnt(NPC, CHAN_AUTO, "sound/chars/mark1/misc/anger.wav");
            NPC_UpdateAngles(qtrue, qtrue);
            return;
        }

        Interrogator_MaintainHeight();

        NPC_BSIdle();
    }
}

/*
-------------------------
NPC_BSInterrogator_Default
-------------------------
*/
fn NPC_BSInterrogator_Default() -> () {
    unsafe {
        //NPC->e_DieFunc = dieF_Interrogator_die;

        if !(*NPC).enemy.is_null() {
            Interrogator_Attack();
        } else {
            Interrogator_Idle();
        }
    }
}
