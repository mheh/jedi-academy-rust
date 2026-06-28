// leave this line at the top of all AI_xxxx.rs files for PCH reasons...
// Corresponds to: #include "g_headers.h"
// Corresponds to: #include "b_local.h"
// Corresponds to: #include "g_nav.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// Engine/game externs - minimal stubs for structural coherence
#[repr(C)]
pub struct gentity_t {
    // Stub: full definition should come from translated headers
}

#[repr(C)]
pub struct gitem_t {
    // Stub: full definition should come from translated headers
}

#[repr(C)]
pub struct trace_t {
    // Stub: full definition should come from translated headers
}

#[repr(C)]
pub struct mdxaBone_t {
    // Stub: full definition should come from translated headers
}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

const QTRUE: qboolean = 1;
const QFALSE: qboolean = 0;

// Forward declaration from engine
extern "C" {
    fn CreateMissile(
        org: *const vec3_t,
        dir: *const vec3_t,
        vel: f32,
        life: c_int,
        owner: *mut gentity_t,
        altFire: qboolean,
    ) -> *mut gentity_t;
    fn FindItemForAmmo(ammo: c_int) -> *mut gitem_t;

    // Engine callbacks
    fn G_SoundIndex(sound: *const c_char) -> c_int;
    fn G_EffectIndex(effect: *const c_char) -> c_int;
    fn RegisterItem(item: *mut gitem_t);
    fn FindItemForWeapon(weapon: c_int) -> *mut gitem_t;
    fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean);
    fn NPC_SetAnim(
        ent: *mut gentity_t,
        setAnimType: c_int,
        anim: c_int,
        flags: c_int,
    );
    fn VectorMA(veca: *mut vec3_t, scale: f32, vecb: *const vec3_t, out: *mut vec3_t);
    fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    fn VectorSubtract(veca: *const vec3_t, vecb: *const vec3_t, out: *mut vec3_t);
    fn VectorNormalize(v: *mut vec3_t) -> f32;
    fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    fn VectorSet(v: *mut vec3_t, x: f32, y: f32, z: f32);
    fn CalcEntitySpot(ent: *const gentity_t, spot: c_int, spot_org: *mut vec3_t);
    fn vectoangles(value: *const vec3_t, angles: *mut vec3_t);
    fn G_PlayEffect(effect: *const c_char, org: *const vec3_t);
    fn G_Sound(ent: *mut gentity_t, index: c_int);
    fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundIndex: *const c_char);
    fn NPC_MoveToGoal(allowPathfind: qboolean);
    fn NPC_CheckEnemyExt() -> qboolean;
    fn NPC_ClearLOS(ent: *const gentity_t) -> qboolean;
    fn NPC_FaceEnemy(doPitch: qboolean);
    fn NPC_GetPainChance(self_: *const gentity_t, damage: c_int) -> f32;
    fn NPC_Pain(
        self_: *mut gentity_t,
        inflictor: *mut gentity_t,
        other: *mut gentity_t,
        point: *const vec3_t,
        damage: c_int,
        mod_: c_int,
    );
    fn NPC_BSIdle();
    fn NPC_CheckPlayerTeamStealth() -> qboolean;
    fn UpdateGoal() -> qboolean;
    fn AngleNormalize360(angle: f32) -> f32;
    fn G_Damage(
        target: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        dir: *const vec3_t,
        point: *const vec3_t,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    );
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn random() -> f32;
    fn rand() -> c_int;
    fn fabs(x: f32) -> f32;
    fn DistanceHorizontalSquared(p1: *const vec3_t, p2: *const vec3_t) -> f32;
}

// Stub structures for NPC and level globals
// These would normally come from b_local_h.rs and other headers
#[repr(C)]
pub struct NPCInfo_t {
    // Stub
}

extern "C" {
    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut NPCInfo_t;
    pub static mut ucmd: usercmd_t;

    // Level globals
    pub static mut level: level_locals_t;
    pub static mut cg: cg_t;
    pub static mut g_spskill: *mut cvar_t;
    pub static mut g_gravity: *mut cvar_t;
}

#[repr(C)]
pub struct usercmd_t {
    // Stub
}

#[repr(C)]
pub struct level_locals_t {
    // Stub
}

#[repr(C)]
pub struct cg_t {
    // Stub
}

#[repr(C)]
pub struct cvar_t {
    // Stub
}

// Stub for trace function
extern "C" {
    fn gi_trace(
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passent: c_int,
        contentmask: c_int,
    );
}

extern "C" {
    fn gi_G2API_GetBoltMatrix(
        ghoul2: *mut c_char,
        modelindex: c_int,
        bolt: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        origin: *const vec3_t,
        time: c_int,
        scale: *const vec3_t,
        scale_val: f32,
    );
    fn gi_G2API_GiveMeVectorFromMatrix(
        matrix: *const mdxaBone_t,
        which: c_int,
        vec: *mut vec3_t,
    );
}

// Constants and defines
const VELOCITY_DECAY: f32 = 0.85;
const HUNTER_STRAFE_VEL: c_int = 256;
const HUNTER_STRAFE_DIS: c_int = 200;
const HUNTER_UPWARD_PUSH: c_int = 32;
const HUNTER_FORWARD_BASE_SPEED: c_int = 10;
const HUNTER_FORWARD_MULTIPLIER: c_int = 5;
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

// Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_BACKINGUP: c_int = 1;
const LSTATE_SPINNING: c_int = 2;
const LSTATE_PAIN: c_int = 3;
const LSTATE_DROP: c_int = 4;

// Macros for timer management - declared as stubs since they're engine functions
extern "C" {
    fn TIMER_Done(ent: *mut gentity_t, timer: *const c_char) -> qboolean;
    fn TIMER_Set(ent: *mut gentity_t, timer: *const c_char, duration: c_int);
}

// Local stub for weapon/ammo constants
const WP_BRYAR_PISTOL: c_int = 1;
const AMMO_BLASTER: c_int = 0;

// Damage and effect constants
const DAMAGE_DEATH_KNOCKBACK: c_int = 1;
const MOD_ENERGY: c_int = 10;
const MOD_DEMP2: c_int = 15;
const MOD_DEMP2_ALT: c_int = 16;
const MOD_UNKNOWN: c_int = 0;

// Animation constants
const SETANIM_BOTH: c_int = 0;
const SETANIM_FLAG_OVERRIDE: c_int = 0x100;
const SETANIM_FLAG_HOLD: c_int = 0x200;
const SETANIM_FLAG_NORMAL: c_int = 0;
const BOTH_RUN1: c_int = 1;
const BOTH_PAIN1: c_int = 2;

// Movement constants
const BUTTON_WALKING: c_int = 0x1;
const BUTTON_ATTACK: c_int = 0x2;

// Mask constants
const MASK_SOLID: c_int = 1;
const MASK_SHOT: c_int = 2;
const CONTENTS_LIGHTSABER: c_int = 4;

// Channel constants
const CHAN_AUTO: c_int = 0;

// AI state flags
const SCF_CHASE_ENEMIES: c_int = 0x1;
const SCF_LOOK_FOR_ENEMIES: c_int = 0x2;

// Power-up constants
const PW_SHOCKED: c_int = 0;

// Spot constants
const SPOT_CHEST: c_int = 1;

// Vector matrix constants
const ORIGIN: c_int = 0;

// Forward declarations
unsafe fn ImperialProbe_Idle();

pub unsafe fn NPC_Probe_Precache() {
    G_SoundIndex("sound/chars/probe/misc/probetalk1\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/probe/misc/probetalk2\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/probe/misc/probetalk3\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/probe/misc/probedroidloop\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/probe/misc/anger1\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/probe/misc/fire\0".as_ptr() as *const c_char);

    G_EffectIndex("chunks/probehead\0".as_ptr() as *const c_char);
    G_EffectIndex("env/med_explode2\0".as_ptr() as *const c_char);
    G_EffectIndex("explosions/probeexplosion1\0".as_ptr() as *const c_char);
    G_EffectIndex("bryar/muzzle_flash\0".as_ptr() as *const c_char);

    let ammo_item = FindItemForAmmo(AMMO_BLASTER);
    RegisterItem(ammo_item);
    let weapon_item = FindItemForWeapon(WP_BRYAR_PISTOL);
    RegisterItem(weapon_item);
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
        dif = (*(*(*NPC).enemy).currentOrigin)[2] - (*(*NPC).currentOrigin)[2];

        // cap to prevent dramatic height shifts
        if fabs(dif) > 8.0 {
            if fabs(dif) > 16.0 {
                dif = if dif < 0.0 { -16.0 } else { 16.0 };
            }

            (*(*(*NPC).client).ps.velocity)[2] = ((*(*(*NPC).client).ps.velocity)[2] + dif) / 2.0;
        }
    } else {
        let mut goal: *mut gentity_t = core::ptr::null_mut();

        if !(*NPCInfo).goalEntity.is_null() {
            // Is there a goal?
            goal = (*NPCInfo).goalEntity;
        } else {
            goal = (*NPCInfo).lastGoalEntity;
        }
        if !goal.is_null() {
            dif = (*goal.currentOrigin)[2] - (*(*NPC).currentOrigin)[2];

            if fabs(dif) > 24.0 {
                ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
            } else {
                if (*(*(*NPC).client).ps.velocity)[2] != 0.0 {
                    (*(*(*NPC).client).ps.velocity)[2] *= VELOCITY_DECAY;

                    if fabs((*(*(*NPC).client).ps.velocity)[2]) < 2.0 {
                        (*(*(*NPC).client).ps.velocity)[2] = 0.0;
                    }
                }
            }
        }
        // Apply friction
        else if (*(*(*NPC).client).ps.velocity)[2] != 0.0 {
            (*(*(*NPC).client).ps.velocity)[2] *= VELOCITY_DECAY;

            if fabs((*(*(*NPC).client).ps.velocity)[2]) < 1.0 {
                (*(*(*NPC).client).ps.velocity)[2] = 0.0;
            }
        }

        // Stay at a given height until we take on an enemy
        /*		VectorSet( endPos, NPC->currentOrigin[0], NPC->currentOrigin[1], NPC->currentOrigin[2] - 512 );
        gi.trace( &trace, NPC->currentOrigin, NULL, NULL, endPos, NPC->s.number, MASK_SOLID );

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
    if (*(*(*NPC).client).ps.velocity)[0] != 0.0 {
        (*(*(*NPC).client).ps.velocity)[0] *= VELOCITY_DECAY;

        if fabs((*(*(*NPC).client).ps.velocity)[0]) < 1.0 {
            (*(*(*NPC).client).ps.velocity)[0] = 0.0;
        }
    }

    if (*(*(*NPC).client).ps.velocity)[1] != 0.0 {
        (*(*(*NPC).client).ps.velocity)[1] *= VELOCITY_DECAY;

        if fabs((*(*(*NPC).client).ps.velocity)[1]) < 1.0 {
            (*(*(*NPC).client).ps.velocity)[1] = 0.0;
        }
    }
}

/*
-------------------------
ImperialProbe_Strafe
-------------------------
*/

pub unsafe fn ImperialProbe_Strafe() {
    let mut dir: c_int;
    let mut end: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut tr: trace_t = core::mem::zeroed();

    AngleVectors(
        &(*(*(*NPC).client).renderInfo.eyeAngles) as *const vec3_t,
        core::ptr::null_mut(),
        &mut right,
        core::ptr::null_mut(),
    );

    // Pick a random strafe direction, then check to see if doing a strafe would be
    //	reasonable valid
    dir = if (rand() & 1) != 0 { -1 } else { 1 };
    VectorMA(
        &(*NPC).currentOrigin as *const vec3_t as *mut vec3_t,
        (HUNTER_STRAFE_DIS * dir) as f32,
        &right,
        &mut end,
    );

    gi_trace(
        &mut tr,
        &(*NPC).currentOrigin,
        core::ptr::null(),
        core::ptr::null(),
        &end,
        (*NPC).s.number,
        MASK_SOLID,
    );

    // Close enough
    if tr.fraction > 0.9 {
        VectorMA(
            &mut (*(*(*NPC).client).ps.velocity),
            (HUNTER_STRAFE_VEL * dir) as f32,
            &right,
            &mut (*(*(*NPC).client).ps.velocity),
        );

        // Add a slight upward push
        (*(*(*NPC).client).ps.velocity)[2] += HUNTER_UPWARD_PUSH as f32;

        // Set the strafe start time so we can do a controlled roll
        (*NPC).fx_time = level.time;
        (*NPCInfo).standTime = level.time + 3000 + (random() * 500.0) as c_int;
    }
}

/*
-------------------------
ImperialProbe_Hunt
-------------------------`
*/

pub unsafe fn ImperialProbe_Hunt(visible: qboolean, advance: qboolean) {
    let mut distance: f32;
    let mut speed: f32;
    let mut forward: vec3_t = [0.0; 3];

    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_RUN1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);

    //If we're not supposed to stand still, pursue the player
    if (*NPCInfo).standTime < level.time {
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

        NPC_MoveToGoal(QTRUE);
        return;
    } else {
        VectorSubtract(
            &(*(*NPC).enemy).currentOrigin,
            &(*NPC).currentOrigin,
            &mut forward,
        );
        distance = VectorNormalize(&mut forward);
    }

    speed = HUNTER_FORWARD_BASE_SPEED as f32 + HUNTER_FORWARD_MULTIPLIER as f32 * (*g_spskill).integer as f32;
    VectorMA(
        &mut (*(*(*NPC).client).ps.velocity),
        speed,
        &forward,
        &mut (*(*(*NPC).client).ps.velocity),
    );
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
    static mut forward: vec3_t = [0.0; 3];
    static mut vright: vec3_t = [0.0; 3];
    static mut up: vec3_t = [0.0; 3];
    static mut muzzle: vec3_t = [0.0; 3];
    let mut missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

    //FIXME: use {0, NPC->client->ps.legsYaw, 0}
    gi_G2API_GetBoltMatrix(
        (*NPC).ghoul2 as *mut c_char,
        (*NPC).playerModel,
        (*NPC).genericBolt1,
        &mut boltMatrix,
        &(*NPC).currentAngles,
        &(*NPC).currentOrigin,
        if cg.time != 0 { cg.time } else { level.time },
        core::ptr::null(),
        (*NPC).s.modelScale,
    );

    gi_G2API_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut muzzle1);

    G_PlayEffect("bryar/muzzle_flash\0".as_ptr() as *const c_char, &muzzle1);

    G_Sound(NPC, G_SoundIndex("sound/chars/probe/misc/fire\0".as_ptr() as *const c_char));

    if (*NPC).health != 0 {
        CalcEntitySpot((*NPC).enemy, SPOT_CHEST, &mut enemy_org1);
        enemy_org1[0] += Q_irand(0, 10) as f32;
        enemy_org1[1] += Q_irand(0, 10) as f32;
        VectorSubtract(&enemy_org1, &muzzle1, &mut delta1);
        vectoangles(&delta1, &mut angleToEnemy1);
        AngleVectors(&angleToEnemy1, &mut forward, &mut vright, &mut up);
    } else {
        AngleVectors(&(*NPC).currentAngles, &mut forward, &mut vright, &mut up);
    }

    missile = CreateMissile(&muzzle1, &forward, 1600.0, 10000, NPC, QFALSE);

    (*missile).classname = "bryar_proj\0".as_ptr() as *const c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    if (*g_spskill).integer <= 1 {
        (*missile).damage = 5;
    } else {
        (*missile).damage = 10;
    }

    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_ENERGY;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
ImperialProbe_Ranged
-------------------------
*/
pub unsafe fn ImperialProbe_Ranged(visible: qboolean, advance: qboolean) {
    let mut delay_min: c_int;
    let mut delay_max: c_int;

    if TIMER_Done(NPC, "attackDelay\0".as_ptr() as *const c_char) != QFALSE {
        // Attack?

        if (*g_spskill).integer == 0 {
            delay_min = 500;
            delay_max = 3000;
        } else if (*g_spskill).integer > 1 {
            delay_min = 500;
            delay_max = 2000;
        } else {
            delay_min = 300;
            delay_max = 1500;
        }

        TIMER_Set(
            NPC,
            "attackDelay\0".as_ptr() as *const c_char,
            Q_irand(500, 3000),
        );
        ImperialProbe_FireBlaster();
        //		ucmd.buttons |= BUTTON_ATTACK;
    }

    if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
        ImperialProbe_Hunt(visible, advance);
    }
}

/*
-------------------------
ImperialProbe_AttackDecision
-------------------------
*/

pub unsafe fn ImperialProbe_AttackDecision() {
    // Always keep a good height off the ground
    ImperialProbe_MaintainHeight();

    //randomly talk
    if TIMER_Done(NPC, "patrolNoise\0".as_ptr() as *const c_char) != QFALSE {
        if TIMER_Done(NPC, "angerNoise\0".as_ptr() as *const c_char) != QFALSE {
            let sound_idx = Q_irand(1, 3);
            let sound_file = match sound_idx {
                1 => "sound/chars/probe/misc/probetalk1\0",
                2 => "sound/chars/probe/misc/probetalk2\0",
                _ => "sound/chars/probe/misc/probetalk3\0",
            };
            G_SoundOnEnt(NPC, CHAN_AUTO, sound_file.as_ptr() as *const c_char);

            TIMER_Set(NPC, "patrolNoise\0".as_ptr() as *const c_char, Q_irand(4000, 10000));
        }
    }

    // If we don't have an enemy, just idle
    if NPC_CheckEnemyExt() == QFALSE {
        ImperialProbe_Idle();
        return;
    }

    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_RUN1, SETANIM_FLAG_NORMAL);

    // Rate our distance to the target, and our visibilty
    let distance: f32 = DistanceHorizontalSquared(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin) as f32;
    //	distance_e	distRate	= ( distance > MIN_MELEE_RANGE_SQR ) ? DIST_LONG : DIST_MELEE;
    let visible: qboolean = NPC_ClearLOS((*NPC).enemy);
    let advance: qboolean = if distance > MIN_DISTANCE_SQR as f32 {
        QTRUE
    } else {
        QFALSE
    };

    // If we cannot see our target, move to see it
    if visible == QFALSE {
        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
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
pub unsafe fn NPC_Probe_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    let mut pain_chance: f32;

    VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s.angles);

    if (*self_).health < 30 || mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT {
        // demp2 always messes them up real good
        let mut endPos: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();

        VectorSet(
            &mut endPos,
            (*self_).currentOrigin[0],
            (*self_).currentOrigin[1],
            (*self_).currentOrigin[2] - 128.0,
        );
        gi_trace(
            &mut trace,
            &(*self_).currentOrigin,
            core::ptr::null(),
            core::ptr::null(),
            &endPos,
            (*self_).s.number,
            MASK_SOLID,
        );

        if trace.fraction == 1.0 || mod_ == MOD_DEMP2 {
            // demp2 always does this
            if (*(*self_).client).clientInfo.headModel != 0 {
                let mut origin: vec3_t = [0.0; 3];

                VectorCopy(&(*self_).currentOrigin, &mut origin);
                origin[2] += 50.0;
                //				G_PlayEffect( "small_chunks", origin );
                G_PlayEffect("chunks/probehead\0".as_ptr() as *const c_char, &origin);
                G_PlayEffect("env/med_explode2\0".as_ptr() as *const c_char, &origin);
                (*(*self_).client).clientInfo.headModel = 0;
                (*(*self_).client).moveType = 1; // MT_RUNJUMP
                (*(*self_).client).ps.gravity = (*g_gravity).value * 0.1;
            }

            if (mod_ == MOD_DEMP2 || mod_ == MOD_DEMP2_ALT) && !other.is_null() {
                let mut dir: vec3_t = [0.0; 3];

                NPC_SetAnim(
                    self_,
                    SETANIM_BOTH,
                    BOTH_PAIN1,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );

                VectorSubtract(&(*self_).currentOrigin, &(*other).currentOrigin, &mut dir);
                let _dist = VectorNormalize(&mut dir);

                VectorMA(
                    &mut (*(*self_).client).ps.velocity,
                    550.0,
                    &dir,
                    &mut (*(*self_).client).ps.velocity,
                );
                (*(*self_).client).ps.velocity[2] -= 127.0;
            }

            (*self_).s.powerups |= 1 << PW_SHOCKED;
            (*(*self_).client).ps.powerups[PW_SHOCKED as usize] = level.time + 3000;

            (*(*self_).NPC).localState = LSTATE_DROP;
        }
    } else {
        pain_chance = NPC_GetPainChance(self_, damage);

        if random() < pain_chance {
            // Spin around in pain?
            NPC_SetAnim(self_, SETANIM_BOTH, BOTH_PAIN1, SETANIM_FLAG_OVERRIDE);
        }
    }

    NPC_Pain(self_, inflictor, other, point, damage, mod_);
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

        if UpdateGoal() != QFALSE {
            //start loop sound once we move
            (*NPC).s.loopSound = G_SoundIndex("sound/chars/probe/misc/probedroidloop\0".as_ptr() as *const c_char);
            ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(QTRUE);
        }
        //randomly talk
        if TIMER_Done(NPC, "patrolNoise\0".as_ptr() as *const c_char) != QFALSE {
            let sound_idx = Q_irand(1, 3);
            let sound_file = match sound_idx {
                1 => "sound/chars/probe/misc/probetalk1\0",
                2 => "sound/chars/probe/misc/probetalk2\0",
                _ => "sound/chars/probe/misc/probetalk3\0",
            };
            G_SoundOnEnt(NPC, CHAN_AUTO, sound_file.as_ptr() as *const c_char);

            TIMER_Set(NPC, "patrolNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
        }
    } else {
        // He's got an enemy. Make him angry.
        G_SoundOnEnt(NPC, CHAN_AUTO, "sound/chars/probe/misc/anger1\0".as_ptr() as *const c_char);
        TIMER_Set(NPC, "angerNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
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
        let mut trace: trace_t = core::mem::zeroed();

        (*NPCInfo).desiredYaw = AngleNormalize360((*NPCInfo).desiredYaw + 25.0);

        VectorSet(
            &mut endPos,
            (*NPC).currentOrigin[0],
            (*NPC).currentOrigin[1],
            (*NPC).currentOrigin[2] - 32.0,
        );
        gi_trace(
            &mut trace,
            &(*NPC).currentOrigin,
            core::ptr::null(),
            core::ptr::null(),
            &endPos,
            (*NPC).s.number,
            MASK_SOLID,
        );

        if trace.fraction != 1.0 {
            G_Damage(NPC, (*NPC).enemy, (*NPC).enemy, core::ptr::null(), core::ptr::null(), 2000, 0, MOD_UNKNOWN);
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
    } else if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
        ImperialProbe_Patrol();
    } else if (*NPCInfo).localState == LSTATE_DROP {
        ImperialProbe_Wait();
    } else {
        ImperialProbe_Idle();
    }
}
