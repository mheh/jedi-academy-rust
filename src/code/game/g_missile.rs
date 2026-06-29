// leave this line at the top for all g_xxxx.cpp files...
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_variables, dead_code, unused_mut, unused_imports)]
#![allow(clippy::all)]

use crate::code::game::g_headers_h::*;
use crate::code::game::g_local_h::*;
use crate::code::game::g_functions_h::*;
use crate::code::game::wp_saber_h::*;
use crate::code::game::bg_local_h::*;
// #ifdef _DEBUG
// #include <float.h>   — system header; _isnan declared extern below

use core::ffi::{c_int, c_char};
use core::ptr::{addr_of, addr_of_mut};

// _isnan from MSVC <float.h>; used only in _DEBUG builds
#[cfg(debug_assertions)]
extern "C" {
    fn _isnan(x: f64) -> c_int;
}

// strcmp from C standard library (used in G_MissileImpact for classname comparison)
extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// Forward declarations from other translation units —
// resolved through glob imports above:
// extern qboolean InFront( vec3_t spot, vec3_t from, vec3_t fromAngles, float threshHold = 0.0f );
// qboolean LogAccuracyHit( gentity_t *target, gentity_t *attacker );
// extern qboolean PM_SaberInParry( int move );
// extern qboolean PM_SaberInReflect( int move );
// extern qboolean PM_SaberInIdle( int move );
// extern qboolean PM_SaberInAttack( int move );
// extern qboolean PM_SaberInTransitionAny( int move );
// extern qboolean PM_SaberInSpecialAttack( int anim );
// extern gentity_t *Jedi_FindEnemyInCone( gentity_t *self, gentity_t *fallback, float minDot );
// extern void WP_SaberBlock( gentity_t *saber, vec3_t hitloc, qboolean missleBlock );
// extern void laserTrapStick( gentity_t *ent, vec3_t endpos, vec3_t normal );
// extern qboolean W_AccuracyLoggableWeapon( int weapon, qboolean alt_fire, int mod );
// void G_MoverTouchPushTriggers( gentity_t *ent, vec3_t oldOrg );

//-------------------------------------------------------------------------
#[cfg(feature = "immersion")]
pub unsafe fn G_MissileBounceEffect(ent: *mut gentity_t, hitEntNum: c_int, org: *mut vec3_t, dir: *mut vec3_t, hitWorld: qboolean) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
    match (*ent).s.weapon {
        WP_BOWCASTER => {
            if hitWorld != 0 {
                G_PlayEffect(b"bowcaster/bounce_wall\0".as_ptr() as *const c_char, hitEntNum, org, dir);
            } else {
                G_PlayEffect(b"bowcaster/deflect\0".as_ptr() as *const c_char, hitEntNum, addr_of_mut!((*ent).currentOrigin), dir);
            }
        }
        WP_BLASTER | WP_BRYAR_PISTOL | WP_BLASTER_PISTOL => {
            G_PlayEffect(b"blaster/deflect\0".as_ptr() as *const c_char, hitEntNum, addr_of_mut!((*ent).currentOrigin), dir);
        }
        _ => {
            let tent = G_TempEntity(org, EV_GRENADE_BOUNCE);
            VectorCopy(dir, addr_of_mut!((*tent).pos1));
            (*tent).s.weapon = (*ent).s.weapon;
            if hitEntNum != -1 {
                (*tent).s.saberActive = 1;
                (*tent).s.otherEntityNum = hitEntNum;
            }
        }
    }
}

#[cfg(not(feature = "immersion"))]
pub unsafe fn G_MissileBounceEffect(ent: *mut gentity_t, org: *mut vec3_t, dir: *mut vec3_t, hitWorld: qboolean) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
    match (*ent).s.weapon {
        WP_BOWCASTER => {
            if hitWorld != 0 {
                G_PlayEffect(b"bowcaster/bounce_wall\0".as_ptr() as *const c_char, org, dir);
            } else {
                G_PlayEffect(b"bowcaster/deflect\0".as_ptr() as *const c_char, addr_of_mut!((*ent).currentOrigin), dir);
            }
        }
        WP_BLASTER | WP_BRYAR_PISTOL | WP_BLASTER_PISTOL => {
            G_PlayEffect(b"blaster/deflect\0".as_ptr() as *const c_char, addr_of_mut!((*ent).currentOrigin), dir);
        }
        _ => {
            let tent = G_TempEntity(org, EV_GRENADE_BOUNCE);
            VectorCopy(dir, addr_of_mut!((*tent).pos1));
            (*tent).s.weapon = (*ent).s.weapon;
        }
    }
}

#[cfg(feature = "immersion")]
pub unsafe fn G_MissileReflectEffect(ent: *mut gentity_t, hitEntNum: c_int, org: *mut vec3_t, dir: *mut vec3_t) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
    match (*ent).s.weapon {
        WP_BOWCASTER => {
            G_PlayEffect(b"bowcaster/deflect\0".as_ptr() as *const c_char, hitEntNum, addr_of_mut!((*ent).currentOrigin), dir);
        }
        // WP_BLASTER | WP_BRYAR_PISTOL | WP_BLASTER_PISTOL | default:
        _ => {
            G_PlayEffect(b"blaster/deflect\0".as_ptr() as *const c_char, hitEntNum, addr_of_mut!((*ent).currentOrigin), dir);
        }
    }
}

#[cfg(not(feature = "immersion"))]
pub unsafe fn G_MissileReflectEffect(ent: *mut gentity_t, org: *mut vec3_t, dir: *mut vec3_t) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
    match (*ent).s.weapon {
        WP_BOWCASTER => {
            G_PlayEffect(b"bowcaster/deflect\0".as_ptr() as *const c_char, addr_of_mut!((*ent).currentOrigin), dir);
        }
        // WP_BLASTER | WP_BRYAR_PISTOL | WP_BLASTER_PISTOL | default:
        _ => {
            G_PlayEffect(b"blaster/deflect\0".as_ptr() as *const c_char, addr_of_mut!((*ent).currentOrigin), dir);
        }
    }
}

//-------------------------------------------------------------------------
unsafe fn G_MissileStick(missile: *mut gentity_t, other: *mut gentity_t, tr: *const trace_t) {
    if !(*other).NPC.is_null() || Q_stricmp((*other).classname, b"misc_model_breakable\0".as_ptr() as *const c_char) == 0 {
        // we bounce off of NPC's and misc model breakables because sticking to them requires too much effort
        let mut velocity: vec3_t = [0.0; 3];

        let level_prev = (*addr_of!(level)).previousTime;
        let level_time = (*addr_of!(level)).time;
        let hitTime: c_int = (level_prev as f32 + (level_time - level_prev) as f32 * (*tr).fraction) as c_int;

        EvaluateTrajectoryDelta(addr_of!((*missile).s.pos), hitTime, velocity.as_mut_ptr());

        let dot: f32 = DotProduct(velocity.as_ptr(), (*tr).plane.normal.as_ptr());
        G_SetOrigin(missile, addr_of!((*tr).endpos));
        VectorMA(velocity.as_ptr(), -1.6_f32 * dot, (*tr).plane.normal.as_ptr(), (*missile).s.pos.trDelta.as_mut_ptr());
        VectorMA((*missile).s.pos.trDelta.as_ptr(), 10.0_f32, (*tr).plane.normal.as_ptr(), (*missile).s.pos.trDelta.as_mut_ptr());
        (*missile).s.pos.trTime = (*addr_of!(level)).time - 10; // move a bit on the first frame

        // check for stop
        if (*tr).entityNum >= 0 && (*tr).entityNum < ENTITYNUM_WORLD &&
                (*tr).plane.normal[2] > 0.7 && (*missile).s.pos.trDelta[2] < 40.0 //this can happen even on very slightly sloped walls, so changed it from > 0 to > 0.7
        {
            (*missile).nextthink = (*addr_of!(level)).time + 100;
        } else {
            // fall till we hit the ground
            (*missile).s.pos.trType = TR_GRAVITY;
        }

        return; // don't stick yet
    }

    if (*missile).e_TouchFunc != touchF_NULL {
        GEntity_TouchFunc(missile, other, tr);
    }

    G_AddEvent(missile, EV_MISSILE_STICK, 0);

    if (*other).s.eType == ET_MOVER || (*other).e_DieFunc == dieF_funcBBrushDie || (*other).e_DieFunc == dieF_funcGlassDie {
        // movers and breakable brushes need extra info...so sticky missiles can ride lifts and blow up when the thing they are attached to goes away.
        (*missile).s.groundEntityNum = (*tr).entityNum;
    }
}

/*
================
G_ReflectMissile

  Reflect the missile roughly back at it's owner
================
*/
// extern gentity_t *Jedi_FindEnemyInCone — resolved via glob import
pub unsafe fn G_ReflectMissile(ent: *mut gentity_t, missile: *mut gentity_t, forward: *mut vec3_t) {
    let mut bounce_dir: vec3_t = [0.0; 3];
    let mut i: c_int;
    let speed: f32;
    let mut reflected: qboolean = qfalse;
    let mut owner: *mut gentity_t = ent;

    if !(*ent).owner.is_null() {
        owner = (*ent).owner;
    }

    //save the original speed
    speed = VectorNormalize((*missile).s.pos.trDelta.as_mut_ptr());

    if !ent.is_null() && !owner.is_null() && !(*owner).client.is_null() && (*(*owner).client).ps.saberInFlight == 0 &&
        ((*(*owner).client).ps.forcePowerLevel[FP_SABER_DEFENSE as usize] > FORCE_LEVEL_2 || ((*(*owner).client).ps.forcePowerLevel[FP_SABER_DEFENSE as usize] > FORCE_LEVEL_1 && Q_irand(0, 3) == 0))
    {
        //if high enough defense skill and saber in-hand (100% at level 3, 25% at level 2, 0% at level 1), reflections are perfectly deflected toward an enemy
        let enemy: *mut gentity_t;
        if !(*owner).enemy.is_null() && Q_irand(0, 3) != 0 {
            //toward current enemy 75% of the time
            enemy = (*owner).enemy;
        } else {
            //find another enemy
            enemy = Jedi_FindEnemyInCone(owner, (*owner).enemy, 0.3_f32);
        }
        if !enemy.is_null() {
            let mut bullseye: vec3_t = [0.0; 3];
            CalcEntitySpot(enemy, SPOT_HEAD, addr_of_mut!(bullseye));
            bullseye[0] += Q_irand(-4, 4) as f32;
            bullseye[1] += Q_irand(-4, 4) as f32;
            bullseye[2] += Q_irand(-16, 4) as f32;
            VectorSubtract(bullseye.as_ptr(), (*missile).currentOrigin.as_ptr(), bounce_dir.as_mut_ptr());
            VectorNormalize(bounce_dir.as_mut_ptr());
            if PM_SaberInParry((*(*owner).client).ps.saberMove) == 0
                && PM_SaberInReflect((*(*owner).client).ps.saberMove) == 0
                && PM_SaberInIdle((*(*owner).client).ps.saberMove) == 0
            {
                //a bit more wild
                if PM_SaberInAttack((*(*owner).client).ps.saberMove) != 0
                    || PM_SaberInTransitionAny((*(*owner).client).ps.saberMove) != 0
                    || PM_SaberInSpecialAttack((*(*owner).client).ps.torsoAnim) != 0
                {
                    //moderately more wild
                    for i in 0..3 {
                        bounce_dir[i] += Q_flrand(-0.2_f32, 0.2_f32);
                    }
                } else {
                    //mildly more wild
                    for i in 0..3 {
                        bounce_dir[i] += Q_flrand(-0.1_f32, 0.1_f32);
                    }
                }
            }
            VectorNormalize(bounce_dir.as_mut_ptr());
            reflected = qtrue;
        }
    }
    if reflected == 0 {
        if !(*missile).owner.is_null() && (*missile).s.weapon != WP_SABER {
            //bounce back at them if you can
            VectorSubtract((*(*missile).owner).currentOrigin.as_ptr(), (*missile).currentOrigin.as_ptr(), bounce_dir.as_mut_ptr());
            VectorNormalize(bounce_dir.as_mut_ptr());
        } else {
            let mut missile_dir: vec3_t = [0.0; 3];

            VectorSubtract((*ent).currentOrigin.as_ptr(), (*missile).currentOrigin.as_ptr(), missile_dir.as_mut_ptr());
            VectorCopy((*missile).s.pos.trDelta.as_ptr(), bounce_dir.as_mut_ptr());
            VectorScale(bounce_dir.as_ptr(), DotProduct(forward, missile_dir.as_ptr()), bounce_dir.as_mut_ptr());
            VectorNormalize(bounce_dir.as_mut_ptr());
        }
        if (*owner).s.weapon == WP_SABER && !(*owner).client.is_null() {
            //saber
            if (*(*owner).client).ps.saberInFlight != 0 {
                //reflecting off a thrown saber is totally wild
                for i in 0..3 {
                    bounce_dir[i] += Q_flrand(-0.8_f32, 0.8_f32);
                }
            } else if (*(*owner).client).ps.forcePowerLevel[FP_SABER_DEFENSE as usize] <= FORCE_LEVEL_1 {
                // at level 1
                for i in 0..3 {
                    bounce_dir[i] += Q_flrand(-0.4_f32, 0.4_f32);
                }
            } else {
                // at level 2
                for i in 0..3 {
                    bounce_dir[i] += Q_flrand(-0.2_f32, 0.2_f32);
                }
            }
            if PM_SaberInParry((*(*owner).client).ps.saberMove) == 0
                && PM_SaberInReflect((*(*owner).client).ps.saberMove) == 0
                && PM_SaberInIdle((*(*owner).client).ps.saberMove) == 0
            {
                //a bit more wild
                if PM_SaberInAttack((*(*owner).client).ps.saberMove) != 0
                    || PM_SaberInTransitionAny((*(*owner).client).ps.saberMove) != 0
                    || PM_SaberInSpecialAttack((*(*owner).client).ps.torsoAnim) != 0
                {
                    //really wild
                    for i in 0..3 {
                        bounce_dir[i] += Q_flrand(-0.3_f32, 0.3_f32);
                    }
                } else {
                    //mildly more wild
                    for i in 0..3 {
                        bounce_dir[i] += Q_flrand(-0.1_f32, 0.1_f32);
                    }
                }
            }
        } else {
            //some other kind of reflection
            for i in 0..3 {
                bounce_dir[i] += Q_flrand(-0.2_f32, 0.2_f32);
            }
        }
    }
    VectorNormalize(bounce_dir.as_mut_ptr());
    VectorScale(bounce_dir.as_ptr(), speed, (*missile).s.pos.trDelta.as_mut_ptr());
    #[cfg(debug_assertions)]
    {
        assert!(_isnan((*missile).s.pos.trDelta[0] as f64) == 0
            && _isnan((*missile).s.pos.trDelta[1] as f64) == 0
            && _isnan((*missile).s.pos.trDelta[2] as f64) == 0);
    }// _DEBUG
    (*missile).s.pos.trTime = (*addr_of!(level)).time - 10; // move a bit on the very first frame
    VectorCopy((*missile).currentOrigin.as_ptr(), (*missile).s.pos.trBase.as_mut_ptr());
    if (*missile).s.weapon != WP_SABER {
        //you are mine, now!
        if (*missile).lastEnemy.is_null() {
            //remember who originally shot this missile
            (*missile).lastEnemy = (*missile).owner;
        }
        (*missile).owner = owner;
    }
    if (*missile).s.weapon == WP_ROCKET_LAUNCHER {
        //stop homing
        (*missile).e_ThinkFunc = thinkF_NULL;
    }
}

/*
================
G_BounceRollMissile

================
*/
pub unsafe fn G_BounceRollMissile(ent: *mut gentity_t, trace: *const trace_t) {
    let mut velocity: vec3_t = [0.0; 3];
    let mut normal: vec3_t = [0.0; 3];
    let dot: f32;
    let speedXY: f32;
    let velocityZ: f32;
    let normalZ: f32;
    let hitTime: c_int;

    // reflect the velocity on the trace plane
    let level_prev = (*addr_of!(level)).previousTime;
    let level_time = (*addr_of!(level)).time;
    hitTime = (level_prev as f32 + (level_time - level_prev) as f32 * (*trace).fraction) as c_int;
    EvaluateTrajectoryDelta(addr_of!((*ent).s.pos), hitTime, velocity.as_mut_ptr());
    //Do horizontal
    //FIXME: Need to roll up, down slopes
    let velocityZ = velocity[2];
    velocity[2] = 0.0;
    let speedXY = VectorLength(velocity.as_ptr()); //friction
    VectorCopy((*trace).plane.normal.as_ptr(), normal.as_mut_ptr());
    let normalZ = normal[2];
    normal[2] = 0.0;
    let dot = DotProduct(velocity.as_ptr(), normal.as_ptr());
    VectorMA(velocity.as_ptr(), -2.0_f32 * dot, normal.as_ptr(), (*ent).s.pos.trDelta.as_mut_ptr());
    //now do the z reflection
    //FIXME: Bobbles when it stops
    VectorSet(velocity.as_mut_ptr(), 0.0, 0.0, velocityZ);
    VectorSet(normal.as_mut_ptr(), 0.0, 0.0, normalZ);
    let dot = DotProduct(velocity.as_ptr(), normal.as_ptr()) * -1.0_f32;
    if dot > 10.0 {
        (*ent).s.pos.trDelta[2] = dot * 0.3_f32; //not very bouncy
    } else {
        (*ent).s.pos.trDelta[2] = 0.0;
    }

    // check for stop
    if speedXY <= 0.0 {
        G_SetOrigin(ent, addr_of!((*trace).endpos));
        VectorCopy((*ent).currentAngles.as_ptr(), (*ent).s.apos.trBase.as_mut_ptr());
        VectorClear((*ent).s.apos.trDelta.as_mut_ptr());
        (*ent).s.apos.trType = TR_STATIONARY;
        return;
    }

    //FIXME: rolling needs to match direction
    VectorCopy((*ent).currentAngles.as_ptr(), (*ent).s.apos.trBase.as_mut_ptr());
    VectorCopy((*ent).s.pos.trDelta.as_ptr(), (*ent).s.apos.trDelta.as_mut_ptr());

    //remember this spot
    VectorCopy((*trace).endpos.as_ptr(), (*ent).currentOrigin.as_mut_ptr());
    (*ent).s.pos.trTime = hitTime - 10;
    VectorCopy((*ent).currentOrigin.as_ptr(), (*ent).s.pos.trBase.as_mut_ptr());
    //VectorCopy( trace->plane.normal, ent->pos1 );
}

/*
================
G_BounceMissile

================
*/
pub unsafe fn G_BounceMissile(ent: *mut gentity_t, trace: *const trace_t) {
    let mut velocity: vec3_t = [0.0; 3];
    let dot: f32;
    let hitTime: c_int;

    // reflect the velocity on the trace plane
    let level_prev = (*addr_of!(level)).previousTime;
    let level_time = (*addr_of!(level)).time;
    hitTime = (level_prev as f32 + (level_time - level_prev) as f32 * (*trace).fraction) as c_int;
    EvaluateTrajectoryDelta(addr_of!((*ent).s.pos), hitTime, velocity.as_mut_ptr());
    let dot = DotProduct(velocity.as_ptr(), (*trace).plane.normal.as_ptr());
    VectorMA(velocity.as_ptr(), -2.0_f32 * dot, (*trace).plane.normal.as_ptr(), (*ent).s.pos.trDelta.as_mut_ptr());

    if (*ent).s.eFlags & EF_BOUNCE_SHRAPNEL != 0 {
        VectorScale((*ent).s.pos.trDelta.as_ptr(), 0.25_f32, (*ent).s.pos.trDelta.as_mut_ptr());
        (*ent).s.pos.trType = TR_GRAVITY;

        // check for stop
        if (*trace).plane.normal[2] > 0.7 && (*ent).s.pos.trDelta[2] < 40.0 //this can happen even on very slightly sloped walls, so changed it from > 0 to > 0.7
        {
            G_SetOrigin(ent, addr_of!((*trace).endpos));
            (*ent).nextthink = (*addr_of!(level)).time + 100;
            return;
        }
    } else if (*ent).s.eFlags & EF_BOUNCE_HALF != 0 {
        VectorScale((*ent).s.pos.trDelta.as_ptr(), 0.5_f32, (*ent).s.pos.trDelta.as_mut_ptr());

        // check for stop
        if (*trace).plane.normal[2] > 0.7 && (*ent).s.pos.trDelta[2] < 40.0 //this can happen even on very slightly sloped walls, so changed it from > 0 to > 0.7
        {
            if (*ent).s.weapon == WP_THERMAL {
                //roll when you "stop"
                (*ent).s.pos.trType = TR_INTERPOLATE;
            } else {
                G_SetOrigin(ent, addr_of!((*trace).endpos));
                (*ent).nextthink = (*addr_of!(level)).time + 500;
                return;
            }
        }

        if (*ent).s.weapon == WP_THERMAL {
            (*ent).has_bounced = qtrue;
        }
    }

    /* #if 0
     * // OLD--this looks so wrong.  It looked wrong in EF.  It just must be wrong.
     * VectorAdd( ent->currentOrigin, trace->plane.normal, ent->currentOrigin);
     *
     * ent->s.pos.trTime = level.time - 10;
     * #else */
    // NEW--It would seem that we want to set our trBase to the trace endpos
    //  and set the trTime to the actual time of impact....
    VectorAdd((*trace).endpos.as_ptr(), (*trace).plane.normal.as_ptr(), (*ent).currentOrigin.as_mut_ptr());
    if hitTime >= (*addr_of!(level)).time {
        //trace fraction must have been 1
        (*ent).s.pos.trTime = (*addr_of!(level)).time - 10;
    } else {
        (*ent).s.pos.trTime = hitTime - 10; // this is kinda dumb hacking, but it pushes the missile away from the impact plane a bit
    }
    // #endif

    VectorCopy((*ent).currentOrigin.as_ptr(), (*ent).s.pos.trBase.as_mut_ptr());
    VectorCopy((*trace).plane.normal.as_ptr(), (*ent).pos1.as_mut_ptr());

    if (*ent).s.weapon != WP_SABER
        && (*ent).s.weapon != WP_THERMAL
        && (*ent).e_clThinkFunc != clThinkF_CG_Limb
        && (*ent).e_ThinkFunc != thinkF_LimbThink
    {
        //not a saber, bouncing thermal or limb
        //now you can damage the guy you came from
        (*ent).owner = core::ptr::null_mut();
    }
}

/*
================
G_MissileImpact

================
*/

pub unsafe fn NoghriGasCloudThink(self_: *mut gentity_t) {
    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;

    AddSightEvent((*self_).owner, (*self_).currentOrigin.as_ptr(), 200, AEL_DANGER, 50);

    if (*self_).fx_time < (*addr_of!(level)).time {
        let up: vec3_t = [0.0, 0.0, 1.0];
        G_PlayEffect(b"noghri_stick/gas_cloud\0".as_ptr() as *const c_char, (*self_).currentOrigin.as_ptr(), up.as_ptr());
        (*self_).fx_time = (*addr_of!(level)).time + 250;
    }

    if (*addr_of!(level)).time - (*self_).s.time <= 2500 {
        if Q_irand(0, 3 - (*(*addr_of!(g_spskill))).integer) == 0 {
            G_RadiusDamage((*self_).currentOrigin.as_ptr(), (*self_).owner, Q_irand(1, 4), (*self_).splashRadius,
                (*self_).owner, (*self_).splashMethodOfDeath);
        }
    }

    if (*addr_of!(level)).time - (*self_).s.time > 3000 {
        G_FreeEntity(self_);
    }
}

pub unsafe fn G_SpawnNoghriGasCloud(ent: *mut gentity_t) {
    //FIXME: force-pushable/dispersable?
    (*ent).freeAfterEvent = qfalse;
    (*ent).e_TouchFunc = touchF_NULL;
    //ent->s.loopSound = G_SoundIndex( "sound/weapons/noghri/smoke.wav" );
    //G_SoundOnEnt( ent, CHAN_AUTO, "sound/weapons/noghri/smoke.wav" );

    G_SetOrigin(ent, addr_of!((*ent).currentOrigin));
    (*ent).e_ThinkFunc = thinkF_NoghriGasCloudThink;
    (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME;

    let up: vec3_t = [0.0, 0.0, 1.0];
    G_PlayEffect(b"noghri_stick/gas_cloud\0".as_ptr() as *const c_char, (*ent).currentOrigin.as_ptr(), up.as_ptr());

    (*ent).fx_time = (*addr_of!(level)).time + 250;
    (*ent).s.time = (*addr_of!(level)).time;
}

// extern void WP_SaberBlock — resolved via glob import
// extern void laserTrapStick — resolved via glob import
// extern qboolean W_AccuracyLoggableWeapon — resolved via glob import
pub unsafe fn G_MissileImpacted(ent: *mut gentity_t, other: *mut gentity_t, impactPos: *const vec3_t, normal: *const vec3_t, hitLoc: c_int) {
    // impact damage
    if (*other).takedamage != 0 {
        // FIXME: wrong damage direction?
        if (*ent).damage != 0 {
            let mut velocity: vec3_t = [0.0; 3];

            EvaluateTrajectoryDelta(addr_of!((*ent).s.pos), (*addr_of!(level)).time, velocity.as_mut_ptr());
            if VectorLength(velocity.as_ptr()) == 0.0 {
                velocity[2] = 1.0; // stepped on a grenade
            }

            let mut damage: c_int = (*ent).damage;

            if !(*other).client.is_null() {
                let npc_class: class_t = (*(*other).client).NPC_class;

                // If we are a robot and we aren't currently doing the full body electricity...
                if npc_class == CLASS_SEEKER || npc_class == CLASS_PROBE || npc_class == CLASS_MOUSE ||
                    npc_class == CLASS_GONK || npc_class == CLASS_R2D2 || npc_class == CLASS_R5D2 || npc_class == CLASS_REMOTE ||
                    npc_class == CLASS_MARK1 || npc_class == CLASS_MARK2 || //npc_class == CLASS_PROTOCOL ||//no protocol, looks odd
                    npc_class == CLASS_INTERROGATOR || npc_class == CLASS_ATST || npc_class == CLASS_SENTRY
                {
                    // special droid only behaviors
                    if (*(*other).client).ps.powerups[PW_SHOCKED as usize] < (*addr_of!(level)).time + 100 {
                        // ... do the effect for a split second for some more feedback
                        (*other).s.powerups |= 1 << PW_SHOCKED;
                        (*(*other).client).ps.powerups[PW_SHOCKED as usize] = (*addr_of!(level)).time + 450;
                    }
                    //FIXME: throw some sparks off droids,too
                }
            }

            G_Damage(other, ent, (*ent).owner, velocity.as_ptr(),
                    impactPos, damage,
                    (*ent).dflags, (*ent).methodOfDeath, hitLoc);

            if (*ent).s.weapon == WP_DEMP2 {
                //a hit with demp2 decloaks saboteurs
                if !other.is_null() && !(*other).client.is_null() && (*(*other).client).NPC_class == CLASS_SABOTEUR {
                    //FIXME: make this disabled cloak hold for some amount of time?
                    Saboteur_Decloak(other, Q_irand(3000, 10000));
                    if (*ent).methodOfDeath == MOD_DEMP2_ALT {
                        //direct hit with alt disabled cloak forever
                        if !(*other).NPC.is_null() {
                            //permanently disable the saboteur's cloak
                            (*(*other).NPC).aiFlags &= !NPCAI_SHIELDS;
                        }
                    }
                }
            }
        }
    }

    // is it cheaper in bandwidth to just remove this ent and create a new
    // one, rather than changing the missile into the explosion?
    //G_FreeEntity(ent);

    if ((*other).takedamage != 0 && !(*other).client.is_null()) || ((*ent).s.weapon == WP_FLECHETTE && (*other).contents & CONTENTS_LIGHTSABER != 0) {
        G_AddEvent(ent, EV_MISSILE_HIT, DirToByte(normal));
        (*ent).s.otherEntityNum = (*other).s.number;
    } else {
        G_AddEvent(ent, EV_MISSILE_MISS, DirToByte(normal));
        (*ent).s.otherEntityNum = (*other).s.number;
    }

    VectorCopy(normal, (*ent).pos1.as_mut_ptr());

    if !(*ent).owner.is_null() {
        //&& ent->owner->s.number == 0 )
        //Add the event
        AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 256, AEL_SUSPICIOUS, qfalse, qtrue);
        AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 512, AEL_DISCOVERED, 75);
    }

    (*ent).freeAfterEvent = qtrue;

    // change over to a normal entity right at the point of impact
    (*ent).s.eType = ET_GENERAL;

    //SnapVectorTowards( trace->endpos, ent->s.pos.trBase );	// save net bandwidth
    VectorCopy(impactPos, (*ent).s.pos.trBase.as_mut_ptr());

    G_SetOrigin(ent, impactPos);

    // splash damage (doesn't apply to person directly hit)
    if (*ent).splashDamage != 0 {
        G_RadiusDamage(impactPos, (*ent).owner, (*ent).splashDamage, (*ent).splashRadius,
            other, (*ent).splashMethodOfDeath);
    }

    if (*ent).s.weapon == WP_NOGHRI_STICK {
        G_SpawnNoghriGasCloud(ent);
    }

    ((*addr_of!(gi)).linkentity)(ent);
}

//------------------------------------------------
unsafe fn G_MissileAddAlerts(ent: *mut gentity_t) {
    //Add the event
    if (*ent).s.weapon == WP_THERMAL && (((*ent).delay - (*addr_of!(level)).time) < 2000 || (*ent).s.pos.trType == TR_INTERPOLATE) {
        //a thermal about to explode or rolling
        if ((*ent).delay - (*addr_of!(level)).time) < 500 {
            //half a second before it explodes!
            AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), ((*ent).splashRadius * 2.0_f32) as c_int, AEL_DANGER_GREAT, qfalse, qtrue);
            AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), ((*ent).splashRadius * 2.0_f32) as c_int, AEL_DANGER_GREAT, 20);
        } else {
            //2 seconds until it explodes or it's rolling
            AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), ((*ent).splashRadius * 2.0_f32) as c_int, AEL_DANGER, qfalse, qtrue);
            AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), ((*ent).splashRadius * 2.0_f32) as c_int, AEL_DANGER, 20);
        }
    } else {
        AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 128, AEL_DISCOVERED, qfalse, qfalse);
        AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 256, AEL_DISCOVERED, 40);
    }
}

//------------------------------------------------------
pub unsafe fn G_MissileImpact(ent: *mut gentity_t, trace: *mut trace_t, hitLoc: c_int) {
    let other: *mut gentity_t;
    let mut diff: vec3_t = [0.0; 3];

    other = (addr_of_mut!(g_entities) as *mut gentity_t).add((*trace).entityNum as usize);
    if other == ent {
        assert!(false, "missile hit itself!!!");
        return;
    }
    if (*trace).plane.normal[0] == 0.0_f32 &&
        (*trace).plane.normal[1] == 0.0_f32 &&
        (*trace).plane.normal[2] == 0.0_f32
    {
        //model moved into missile in flight probably...
        (*trace).plane.normal[0] = -(*ent).s.pos.trDelta[0];
        (*trace).plane.normal[1] = -(*ent).s.pos.trDelta[1];
        (*trace).plane.normal[2] = -(*ent).s.pos.trDelta[2];
        VectorNormalize((*trace).plane.normal.as_mut_ptr());
    }

    if !(*ent).owner.is_null() && ((*other).takedamage != 0 || !(*other).client.is_null()) {
        if (*ent).lastEnemy.is_null() || (*ent).lastEnemy == (*ent).owner {
            //a missile that was not reflected or, if so, still is owned by original owner
            if LogAccuracyHit(other, (*ent).owner) != 0 {
                (*(*(*ent).owner).client).ps.persistant[PERS_ACCURACY_HITS as usize] += 1;
            }
            if !(*(*ent).owner).client.is_null() && (*(*ent).owner).s.number == 0 {
                if W_AccuracyLoggableWeapon((*ent).s.weapon, qfalse, (*ent).methodOfDeath) != 0 {
                    (*(*(*ent).owner).client).sess.missionStats.hits += 1;
                }
            }
        }
    }
    // check for bounce
    //OR: if the surfaceParm is has a reflect property (magnetic shielding) and the missile isn't an exploding missile
    let mut bounce: qboolean = if ((*other).takedamage == 0 && ((*ent).s.eFlags & (EF_BOUNCE | EF_BOUNCE_HALF) != 0))
        || ((((*trace).surfaceFlags & SURF_FORCEFIELD != 0) || ((*other).flags & FL_SHIELDED != 0))
            && (*ent).splashDamage == 0 && (*ent).splashRadius == 0.0 && (*ent).s.weapon != WP_NOGHRI_STICK)
    { qtrue } else { qfalse };

    if (*ent).dflags & DAMAGE_HEAVY_WEAP_CLASS != 0 {
        // heavy class missiles generally never bounce.
        bounce = qfalse;
    }

    if (*other).flags & (FL_DMG_BY_HEAVY_WEAP_ONLY | FL_SHIELDED) != 0 {
        // Dumb assumption, but I guess we must be a shielded ion_cannon??  We should probably verify
        // if it's an ion_cannon that's Heavy Weapon only, we don't want to make it shielded do we...?
        if strcmp(b"misc_ion_cannon\0".as_ptr() as *const c_char, (*other).classname) == 0 && (*other).flags & FL_SHIELDED != 0 {
            // Anything will bounce off of us.
            bounce = qtrue;

            // Not exactly the debounce time, but rather the impact time for the shield effect...play effect for 1 second
            (*other).painDebounceTime = (*addr_of!(level)).time + 1000;
        }
    }

    if (*ent).s.weapon == WP_DEMP2 {
        // demp2 shots can never bounce
        bounce = qfalse;

        // in fact, alt-charge shots will not call the regular impact functions
        if (*ent).alt_fire != 0 {
            // detonate at the trace end
            VectorCopy((*trace).endpos.as_ptr(), (*ent).currentOrigin.as_mut_ptr());
            VectorCopy((*trace).plane.normal.as_ptr(), (*ent).pos1.as_mut_ptr());
            DEMP2_AltDetonate(ent);
            return;
        }
    }

    if bounce != 0 {
        // Check to see if there is a bounce count
        if (*ent).bounceCount != 0 {
            // decrement number of bounces and then see if it should be done bouncing
            (*ent).bounceCount -= 1;
            if (*ent).bounceCount == 0 {
                // He (or she) will bounce no more (after this current bounce, that is).
                (*ent).s.eFlags &= !(EF_BOUNCE | EF_BOUNCE_HALF);
            }
        }

        if !(*other).NPC.is_null() {
            G_Damage(other, ent, (*ent).owner, (*ent).currentOrigin.as_ptr(), (*ent).s.pos.trDelta.as_ptr(), 0, DAMAGE_NO_DAMAGE, MOD_UNKNOWN, HL_NONE);
        }

        G_BounceMissile(ent, trace);

        if !(*ent).owner.is_null() {
            //&& ent->owner->s.number == 0 )
            G_MissileAddAlerts(ent);
        }
        #[cfg(feature = "immersion")]
        G_MissileBounceEffect(ent,
            if (*other).contents & CONTENTS_LIGHTSABER != 0 && (*(*other).owner).s.saberInFlight == 0 {
                (*(*other).owner).s.number
            } else { -1 },
            addr_of_mut!((*trace).endpos),
            (*trace).plane.normal.as_mut_ptr(),
            ((*trace).entityNum == ENTITYNUM_WORLD) as qboolean);
        #[cfg(not(feature = "immersion"))]
        G_MissileBounceEffect(ent, addr_of_mut!((*trace).endpos), (*trace).plane.normal.as_mut_ptr(), ((*trace).entityNum == ENTITYNUM_WORLD) as qboolean);

        return;
    }

    // I would glom onto the EF_BOUNCE code section above, but don't feel like risking breaking something else
    if ((*other).takedamage == 0 && ((*ent).s.eFlags & EF_BOUNCE_SHRAPNEL != 0))
        || (((*trace).surfaceFlags & SURF_FORCEFIELD != 0) && (*ent).splashDamage == 0 && (*ent).splashRadius == 0.0)
    {
        if (*other).contents & CONTENTS_LIGHTSABER == 0
            || (*(*addr_of!(g_spskill))).integer <= 0 //on easy, it reflects all shots
            || ((*(*addr_of!(g_spskill))).integer == 1 && (*ent).s.weapon != WP_FLECHETTE && (*ent).s.weapon != WP_DEMP2) //on medium it won't reflect flechette or demp shots
            || ((*(*addr_of!(g_spskill))).integer >= 2 && (*ent).s.weapon != WP_FLECHETTE && (*ent).s.weapon != WP_DEMP2 && (*ent).s.weapon != WP_BOWCASTER && (*ent).s.weapon != WP_REPEATER) //on hard it won't reflect flechette, demp, repeater or bowcaster shots
        {
            G_BounceMissile(ent, trace);

            (*ent).bounceCount -= 1;
            if (*ent).bounceCount < 0 {
                (*ent).s.eFlags &= !EF_BOUNCE_SHRAPNEL;
            }
            #[cfg(feature = "immersion")]
            // might deflect flechette spam in bespin_platform.bsp
            G_MissileBounceEffect(ent,
                if (*other).contents & CONTENTS_LIGHTSABER != 0 && (*(*other).owner).s.saberInFlight == 0 {
                    (*(*other).owner).s.number
                } else { -1 },
                addr_of_mut!((*trace).endpos),
                (*trace).plane.normal.as_mut_ptr(),
                ((*trace).entityNum == ENTITYNUM_WORLD) as qboolean);
            #[cfg(not(feature = "immersion"))]
            G_MissileBounceEffect(ent, addr_of_mut!((*trace).endpos), (*trace).plane.normal.as_mut_ptr(), ((*trace).entityNum == ENTITYNUM_WORLD) as qboolean);
            return;
        }
    }

    if ((*other).takedamage == 0 || (!(*other).client.is_null() && (*other).health <= 0))
        && (*ent).s.weapon == WP_THERMAL
        && (*ent).alt_fire == 0
    {
        //rolling thermal det - FIXME: make this an eFlag like bounce & stick!!!
        //G_BounceRollMissile( ent, trace );
        if !(*ent).owner.is_null() {
            //&& ent->owner->s.number == 0 )
            G_MissileAddAlerts(ent);
        }
        //gi.linkentity( ent );
        return;
    }

    // check for sticking
    if (*ent).s.eFlags & EF_MISSILE_STICK != 0 {
        if !(*ent).owner.is_null() {
            //&& ent->owner->s.number == 0 )
            //Add the event
            if (*ent).s.weapon == WP_TRIP_MINE {
                AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), ((*ent).splashRadius / 2.0_f32) as c_int, AEL_DISCOVERED /*AEL_DANGER*/, qfalse, qtrue);
                AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), ((*ent).splashRadius * 2.0_f32) as c_int, AEL_DISCOVERED /*AEL_DANGER*/, 60);
                /*
                AddSoundEvent( ent->owner, ent->currentOrigin, ent->splashRadius*2, AEL_DANGER, qfalse, qtrue );
                AddSightEvent( ent->owner, ent->currentOrigin, ent->splashRadius*2, AEL_DANGER, 60 );
                */
            } else {
                AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 128, AEL_DISCOVERED, qfalse, qtrue);
                AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 256, AEL_DISCOVERED, 10);
            }
        }

        G_MissileStick(ent, other, trace);
        return;
    }

    // check for hitting a lightsaber
    if (*other).contents & CONTENTS_LIGHTSABER != 0 {
        if !(*other).owner.is_null() && (*(*other).owner).s.number == 0 && !(*(*other).owner).client.is_null() {
            (*(*(*other).owner).client).sess.missionStats.saberBlocksCnt += 1;
        }
        if ((*(*addr_of!(g_spskill))).integer <= 0 //on easy, it reflects all shots
                || ((*(*addr_of!(g_spskill))).integer == 1 && (*ent).s.weapon != WP_FLECHETTE && (*ent).s.weapon != WP_DEMP2) //on medium it won't reflect flechette or demp shots
                || ((*(*addr_of!(g_spskill))).integer >= 2 && (*ent).s.weapon != WP_FLECHETTE && (*ent).s.weapon != WP_DEMP2 && (*ent).s.weapon != WP_BOWCASTER && (*ent).s.weapon != WP_REPEATER) //on hard it won't reflect flechette, demp, repeater or bowcaster shots
            )
            && ((*ent).splashDamage == 0 || (*ent).splashRadius == 0.0) //this would be cool, though, to "bat" the thermal det away...
            && (*ent).s.weapon != WP_NOGHRI_STICK //gas bomb, don't reflect
        {
            //FIXME: take other's owner's FP_SABER_DEFENSE into account here somehow?
            if (*other).owner.is_null() || (*(*other).owner).client.is_null() || (*(*(*other).owner).client).ps.saberInFlight != 0
                || InFront((*ent).currentOrigin.as_ptr(), (*(*other).owner).currentOrigin.as_ptr(), (*(*(*other).owner).client).ps.viewangles.as_ptr(), SABER_REFLECT_MISSILE_CONE) != 0
            //other->owner->s.number != 0 ||
            {
                //Jedi cannot block shots from behind!
                let mut blockChance: c_int = 0;
                match (*(*(*other).owner).client).ps.forcePowerLevel[FP_SABER_DEFENSE as usize] {
                    //level 1 reflects 50% of the time, level 2 reflects 75% of the time
                    FORCE_LEVEL_3 => {
                        blockChance = 10;
                    }
                    FORCE_LEVEL_2 => {
                        blockChance = 3;
                    }
                    FORCE_LEVEL_1 => {
                        blockChance = 1;
                    }
                    _ => {}
                }
                if blockChance != 0 && ((*(*(*other).owner).client).ps.forcePowersActive & (1 << FP_SPEED) != 0) {
                    //in in force speed, better chance of deflecting the shot
                    blockChance += (*(*(*other).owner).client).ps.forcePowerLevel[FP_SPEED as usize] * 2;
                }
                if Q_irand(0, blockChance) != 0 {
                    VectorSubtract((*ent).currentOrigin.as_ptr(), (*other).currentOrigin.as_ptr(), diff.as_mut_ptr());
                    VectorNormalize(diff.as_mut_ptr());
                    G_ReflectMissile(other, ent, diff.as_mut_ptr());
                    //WP_SaberBlock( other, ent->currentOrigin, qtrue );
                    if !(*other).owner.is_null() && !(*(*other).owner).client.is_null() {
                        (*(*(*other).owner).client).ps.saberEventFlags |= SEF_DEFLECTED;
                    }
                    //do the effect
                    VectorCopy((*ent).s.pos.trDelta.as_ptr(), diff.as_mut_ptr());
                    VectorNormalize(diff.as_mut_ptr());
                    #[cfg(feature = "immersion")]
                    G_MissileReflectEffect(ent, (*(*other).owner).s.number, addr_of_mut!((*trace).endpos), (*trace).plane.normal.as_mut_ptr());
                    #[cfg(not(feature = "immersion"))]
                    G_MissileReflectEffect(ent, addr_of_mut!((*trace).endpos), (*trace).plane.normal.as_mut_ptr());
                    return;
                }
            }
        } else {
            //still do the bounce effect
            #[cfg(feature = "immersion")]
            G_MissileReflectEffect(ent, (*(*other).owner).s.number, addr_of_mut!((*trace).endpos), (*trace).plane.normal.as_mut_ptr());
            #[cfg(not(feature = "immersion"))]
            G_MissileReflectEffect(ent, addr_of_mut!((*trace).endpos), (*trace).plane.normal.as_mut_ptr());
        }
    }

    G_MissileImpacted(ent, other, addr_of!((*trace).endpos), (*trace).plane.normal.as_ptr(), hitLoc);
}

/*
================
G_ExplodeMissile

Explode a missile without an impact
================
*/
pub unsafe fn G_ExplodeMissile(ent: *mut gentity_t) {
    let mut dir: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];

    EvaluateTrajectory(addr_of!((*ent).s.pos), (*addr_of!(level)).time, origin.as_mut_ptr());
    SnapVector(origin.as_mut_ptr());
    G_SetOrigin(ent, addr_of!(origin));

    // we don't have a valid direction, so just point straight up
    dir[0] = 0.0;
    dir[1] = 0.0;
    dir[2] = 1.0;

    if !(*ent).owner.is_null() {
        //&& ent->owner->s.number == 0 )
        //Add the event
        AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 256, AEL_DISCOVERED, qfalse, qtrue); //FIXME: are we on ground or not?
        AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 512, AEL_DISCOVERED, 100);
    }
    /*	ent->s.eType = ET_GENERAL;
    	G_AddEvent( ent, EV_MISSILE_MISS, DirToByte( dir ) );

    	ent->freeAfterEvent = qtrue;*/

    // splash damage
    if (*ent).splashDamage != 0 {
        G_RadiusDamage((*ent).currentOrigin.as_ptr(), (*ent).owner, (*ent).splashDamage, (*ent).splashRadius, core::ptr::null_mut()
            , (*ent).splashMethodOfDeath);
    }

    G_FreeEntity(ent);
    //gi.linkentity( ent );
}


pub unsafe fn G_RunStuckMissile(ent: *mut gentity_t) {
    if (*ent).takedamage != 0 {
        if (*ent).s.groundEntityNum >= 0 && (*ent).s.groundEntityNum < ENTITYNUM_WORLD {
            let other: *mut gentity_t = (addr_of_mut!(g_entities) as *mut gentity_t).add((*ent).s.groundEntityNum as usize);

            if (VectorCompare(addr_of!(vec3_origin) as *const vec3_t, (*other).s.pos.trDelta.as_ptr()) == 0
                && (*other).s.pos.trType != TR_STATIONARY)
                || (VectorCompare(addr_of!(vec3_origin) as *const vec3_t, (*other).s.apos.trDelta.as_ptr()) == 0
                    && (*other).s.apos.trType != TR_STATIONARY)
            {
                //thing I stuck to is moving or rotating now, kill me
                G_Damage(ent, other, other, core::ptr::null(), core::ptr::null(), 99999, 0, MOD_CRUSH, HL_NONE);
                return;
            }
        }
    }
    // check think function
    G_RunThink(ent);
}

/*
==================

G_GroundTrace

==================
*/
pub unsafe fn G_GroundTrace(ent: *mut gentity_t, pPml: *mut pml_t) -> c_int {
    let mut point: vec3_t = [0.0; 3];
    let mut trace: trace_t = core::mem::zeroed();

    point[0] = (*ent).currentOrigin[0];
    point[1] = (*ent).currentOrigin[1];
    point[2] = (*ent).currentOrigin[2] - 0.25;

    ((*addr_of!(gi)).trace)(addr_of_mut!(trace), (*ent).currentOrigin.as_ptr(), (*ent).mins.as_ptr(), (*ent).maxs.as_ptr(), point.as_ptr(), (*ent).s.number, (*ent).clipmask, G2_NOCOLLIDE, 0);
    (*pPml).groundTrace = trace;

    // do something corrective if the trace starts in a solid...
    if trace.allsolid != 0 {
        (*pPml).groundPlane = qfalse;
        (*pPml).walking = qfalse;
        return ENTITYNUM_NONE;
    }

    // if the trace didn't hit anything, we are in free fall
    if trace.fraction == 1.0 {
        (*pPml).groundPlane = qfalse;
        (*pPml).walking = qfalse;
        return ENTITYNUM_NONE;
    }

    // check if getting thrown off the ground
    if (*ent).s.pos.trDelta[2] > 0.0 && DotProduct((*ent).s.pos.trDelta.as_ptr(), trace.plane.normal.as_ptr()) > 10.0 {
        (*pPml).groundPlane = qfalse;
        (*pPml).walking = qfalse;
        return ENTITYNUM_NONE;
    }

    // slopes that are too steep will not be considered onground
    if trace.plane.normal[2] < MIN_WALK_NORMAL {
        (*pPml).groundPlane = qtrue;
        (*pPml).walking = qfalse;
        return ENTITYNUM_NONE;
    }

    (*pPml).groundPlane = qtrue;
    (*pPml).walking = qtrue;

    /*
    if ( ent->s.groundEntityNum == ENTITYNUM_NONE )
    {
    	// just hit the ground
    }
    */

    return trace.entityNum;
}

pub unsafe fn G_ClipVelocity(in_: *const vec3_t, normal: *const vec3_t, out: *mut vec3_t, overbounce: f32) {
    let backoff: f32;
    let change: f32;
    let i: c_int;

    let mut backoff = DotProduct(in_ as *const f32, normal as *const f32);

    if backoff < 0.0 {
        backoff *= overbounce;
    } else {
        backoff /= overbounce;
    }

    for i in 0..3 {
        let change = (*normal)[i] * backoff;
        (*out)[i] = (*in_)[i] - change;
    }
}
/*
==================

G_RollMissile

reworking the rolling object code,
still needs to stop bobbling up & down,
need to get roll angles right,
and need to maybe make the transfer of velocity happen on impacts?
Also need bounce sound for bounces off a floor.
Also need to not bounce as much off of enemies
Also gets stuck inside thrower if looking down when thrown

==================
*/
const MAX_CLIP_PLANES: usize = 5;
const BUMPCLIP: f32 = 1.5_f32;
pub unsafe fn G_RollMissile(ent: *mut gentity_t) {
    let bumpcount: c_int;
    let numbumps: c_int;
    let mut dir: vec3_t = [0.0; 3];
    let d: f32;
    let numplanes: c_int;
    let mut planes: [vec3_t; MAX_CLIP_PLANES] = [[0.0; 3]; MAX_CLIP_PLANES];
    let mut primal_velocity: vec3_t = [0.0; 3];
    let mut clipVelocity: vec3_t = [0.0; 3];
    let i: c_int;
    let j: c_int;
    let k: c_int;
    let mut trace: trace_t = core::mem::zeroed();
    let mut end: vec3_t = [0.0; 3];
    let time_left: f32;
    let into: f32;
    let mut endVelocity: vec3_t = [0.0; 3];
    let mut endClipVelocity: vec3_t = [0.0; 3];
    let mut objPML: pml_t = core::mem::zeroed();
    let mut bounceAmt: f32 = BUMPCLIP;
    let mut hitEnt: *mut gentity_t = core::ptr::null_mut();

    // memset( &objPML, 0, sizeof( objPML ) ); — zeroed() above handles this

    G_GroundTrace(ent, addr_of_mut!(objPML));

    (*addr_of_mut!(objPML)).frametime = ((*addr_of!(level)).time - (*addr_of!(level)).previousTime) as f32 * 0.001_f32;

    let numbumps: c_int = 4;

    VectorCopy((*ent).s.pos.trDelta.as_ptr(), primal_velocity.as_mut_ptr());

    VectorCopy((*ent).s.pos.trDelta.as_ptr(), endVelocity.as_mut_ptr());
    endVelocity[2] -= (*(*addr_of!(g_gravity))).value * (*addr_of!(objPML)).frametime;
    (*ent).s.pos.trDelta[2] = ((*ent).s.pos.trDelta[2] + endVelocity[2]) * 0.5_f32;
    primal_velocity[2] = endVelocity[2];
    if (*addr_of!(objPML)).groundPlane != 0 {
        //FIXME: never happens!
        // slide along the ground plane
        G_ClipVelocity((*ent).s.pos.trDelta.as_ptr(), (*addr_of!(objPML)).groundTrace.plane.normal.as_ptr(), (*ent).s.pos.trDelta.as_mut_ptr(), BUMPCLIP);
        VectorScale((*ent).s.pos.trDelta.as_ptr(), 0.9_f32, (*ent).s.pos.trDelta.as_mut_ptr());
    }

    let mut time_left: f32 = (*addr_of!(objPML)).frametime;

    // never turn against the ground plane
    let mut numplanes: usize;
    if (*addr_of!(objPML)).groundPlane != 0 {
        numplanes = 1;
        VectorCopy((*addr_of!(objPML)).groundTrace.plane.normal.as_ptr(), planes[0].as_mut_ptr());
    } else {
        numplanes = 0;
    }

    // never turn against original velocity
    /*
    VectorNormalize2( ent->s.pos.trDelta, planes[numplanes] );
    numplanes++;
    */

    let mut bumpcount: usize = 0;
    while bumpcount < numbumps as usize {
        // calculate position we are trying to move to
        VectorMA((*ent).currentOrigin.as_ptr(), time_left, (*ent).s.pos.trDelta.as_ptr(), end.as_mut_ptr());

        // see if we can make it there
        ((*addr_of!(gi)).trace)(addr_of_mut!(trace), (*ent).currentOrigin.as_ptr(), (*ent).mins.as_ptr(), (*ent).maxs.as_ptr(), end.as_ptr(), (*ent).s.number, (*ent).clipmask, G2_RETURNONHIT, 10);

        //TEMP HACK:
        //had to move this up above the trace.allsolid check now that for some reason ghoul2 impacts tell me I'm allsolid?!
        //this needs to be fixed, really
        if trace.entityNum < ENTITYNUM_WORLD {
            //hit another ent
            hitEnt = (addr_of_mut!(g_entities) as *mut gentity_t).add(trace.entityNum as usize);
            if !hitEnt.is_null() && ((*hitEnt).takedamage != 0 || ((*hitEnt).contents & CONTENTS_LIGHTSABER != 0)) {
                G_MissileImpact(ent, addr_of_mut!(trace), HL_NONE);
                if (*ent).s.eType == ET_GENERAL {
                    //exploded
                    return;
                }
            }
        }

        if trace.allsolid != 0 {
            // entity is completely trapped in another solid
            //FIXME: this happens a lot now when we hit a G2 ent... WTF?
            (*ent).s.pos.trDelta[2] = 0.0; // don't build up falling damage, but allow sideways acceleration
            return; // qtrue;
        }

        if trace.fraction > 0.0 {
            // actually covered some distance
            VectorCopy(trace.endpos.as_ptr(), (*ent).currentOrigin.as_mut_ptr());
        }

        if trace.fraction == 1.0 {
            break; // moved the entire distance
        }

        //pm->ps->pm_flags |= PMF_BUMPED;

        // save entity for contact
        //PM_AddTouchEnt( trace.entityNum );

        //Hit it
        /*
        if ( PM_ClientImpact( trace.entityNum, qtrue ) )
        {
        	continue;
        }
        */

        time_left -= time_left * trace.fraction;

        if numplanes >= MAX_CLIP_PLANES {
            // this shouldn't really happen
            VectorClear((*ent).s.pos.trDelta.as_mut_ptr());
            return; // qtrue;
        }

        //
        // if this is the same plane we hit before, nudge velocity
        // out along it, which fixes some epsilon issues with
        // non-axial planes
        //
        let mut found_plane = false;
        for i in 0..numplanes {
            if DotProduct(trace.plane.normal.as_ptr(), planes[i].as_ptr()) > 0.99 {
                VectorAdd(trace.plane.normal.as_ptr(), (*ent).s.pos.trDelta.as_ptr(), (*ent).s.pos.trDelta.as_mut_ptr());
                found_plane = true;
                break;
            }
        }
        if found_plane {
            bumpcount += 1;
            continue;
        }
        VectorCopy(trace.plane.normal.as_ptr(), planes[numplanes].as_mut_ptr());
        numplanes += 1;

        //
        // modify velocity so it parallels all of the clip planes
        //
        if !(addr_of_mut!(g_entities) as *mut gentity_t).add(trace.entityNum as usize).is_null()
            && !(*((addr_of_mut!(g_entities) as *mut gentity_t).add(trace.entityNum as usize))).client.is_null()
        {
            //hit a person, bounce off much less
            bounceAmt = OVERCLIP;
        } else {
            bounceAmt = BUMPCLIP;
        }

        // find a plane that it enters
        let mut broke_inner = false;
        for i in 0..numplanes {
            let into = DotProduct((*ent).s.pos.trDelta.as_ptr(), planes[i].as_ptr());
            if into >= 0.1 {
                continue; // move doesn't interact with the plane
            }

            // see how hard we are hitting things
            if -into > (*addr_of_mut!(pml)).impactSpeed {
                (*addr_of_mut!(pml)).impactSpeed = -into;
            }

            // slide along the plane
            G_ClipVelocity((*ent).s.pos.trDelta.as_ptr(), planes[i].as_ptr(), clipVelocity.as_mut_ptr(), bounceAmt);

            // slide along the plane
            G_ClipVelocity(endVelocity.as_ptr(), planes[i].as_ptr(), endClipVelocity.as_mut_ptr(), bounceAmt);

            // see if there is a second plane that the new move enters
            for j in 0..numplanes {
                if j == i {
                    continue;
                }
                if DotProduct(clipVelocity.as_ptr(), planes[j].as_ptr()) >= 0.1 {
                    continue; // move doesn't interact with the plane
                }

                // try clipping the move to the plane
                G_ClipVelocity(clipVelocity.as_ptr(), planes[j].as_ptr(), clipVelocity.as_mut_ptr(), bounceAmt);
                G_ClipVelocity(endClipVelocity.as_ptr(), planes[j].as_ptr(), endClipVelocity.as_mut_ptr(), bounceAmt);

                // see if it goes back into the first clip plane
                if DotProduct(clipVelocity.as_ptr(), planes[i].as_ptr()) >= 0.0 {
                    continue;
                }

                // slide the original velocity along the crease
                CrossProduct(planes[i].as_ptr(), planes[j].as_ptr(), dir.as_mut_ptr());
                VectorNormalize(dir.as_mut_ptr());
                let d = DotProduct(dir.as_ptr(), (*ent).s.pos.trDelta.as_ptr());
                VectorScale(dir.as_ptr(), d, clipVelocity.as_mut_ptr());

                CrossProduct(planes[i].as_ptr(), planes[j].as_ptr(), dir.as_mut_ptr());
                VectorNormalize(dir.as_mut_ptr());
                let d = DotProduct(dir.as_ptr(), endVelocity.as_ptr());
                VectorScale(dir.as_ptr(), d, endClipVelocity.as_mut_ptr());

                // see if there is a third plane the the new move enters
                let mut triple_stop = false;
                for k in 0..numplanes {
                    if k == i || k == j {
                        continue;
                    }
                    if DotProduct(clipVelocity.as_ptr(), planes[k].as_ptr()) >= 0.1 {
                        continue; // move doesn't interact with the plane
                    }

                    // stop dead at a triple plane interaction
                    VectorClear((*ent).s.pos.trDelta.as_mut_ptr());
                    return; // qtrue;
                }
            }

            // if we have fixed all interactions, try another move
            VectorCopy(clipVelocity.as_ptr(), (*ent).s.pos.trDelta.as_mut_ptr());
            VectorCopy(endClipVelocity.as_ptr(), endVelocity.as_mut_ptr());
            broke_inner = true;
            break;
        }
        VectorScale(endVelocity.as_ptr(), 0.975_f32, endVelocity.as_mut_ptr());

        bumpcount += 1;
    }

    VectorCopy(endVelocity.as_ptr(), (*ent).s.pos.trDelta.as_mut_ptr());

    // don't change velocity if in a timer (FIXME: is this correct?)
    /*
    if ( pm->ps->pm_time )
    {
    	VectorCopy( primal_velocity, ent->s.pos.trDelta );
    }
    */

    return; // ( bumpcount != 0 );
}
/*
================
G_RunMissile

================
*/
// void G_MoverTouchPushTriggers — resolved via glob import
pub unsafe fn G_RunMissile(ent: *mut gentity_t) {
    let mut oldOrg: vec3_t = [0.0; 3];
    let mut tr: trace_t = core::mem::zeroed();
    let mut trHitLoc: c_int = HL_NONE;

    if (*ent).s.eFlags & EF_HELD_BY_SAND_CREATURE != 0 {
        //in a sand creature's mouth
        if !(*ent).activator.is_null() {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            // Getting the bolt here
            //in hand
            let mut scAngles: vec3_t = [0.0; 3];
            scAngles[YAW as usize] = (*(*ent).activator).currentAngles[YAW as usize];
            ((*addr_of!(gi)).G2API_GetBoltMatrix)(
                addr_of_mut!((*(*ent).activator).ghoul2),
                (*(*ent).activator).playerModel,
                (*(*ent).activator).gutBolt,
                addr_of_mut!(boltMatrix),
                scAngles.as_ptr(),
                (*(*ent).activator).currentOrigin.as_ptr(),
                {
                    let cg_time = (*addr_of!(cg)).time;
                    if cg_time != 0 { cg_time } else { (*addr_of!(level)).time }
                },
                core::ptr::null_mut(),
                (*(*ent).activator).s.modelScale,
            );
            // Storing ent position, bolt position, and bolt axis
            ((*addr_of!(gi)).G2API_GiveMeVectorFromMatrix)(addr_of_mut!(boltMatrix), ORIGIN, (*ent).currentOrigin.as_mut_ptr());
            G_SetOrigin(ent, addr_of!((*ent).currentOrigin));
        }
        // check think function
        G_RunThink(ent);
        return;
    }

    VectorCopy((*ent).currentOrigin.as_ptr(), oldOrg.as_mut_ptr());

    // get current position
    if (*ent).s.pos.trType == TR_INTERPOLATE {
        //rolling missile?
        //FIXME: WTF?!!  Sticks to stick missiles?
        //FIXME: they stick inside the player
        G_RollMissile(ent);
        if (*ent).s.eType != ET_GENERAL {
            //didn't explode
            VectorCopy((*ent).currentOrigin.as_ptr(), (*ent).s.pos.trBase.as_mut_ptr());
            ((*addr_of!(gi)).trace)(addr_of_mut!(tr), oldOrg.as_ptr(), (*ent).mins.as_ptr(), (*ent).maxs.as_ptr(), (*ent).currentOrigin.as_ptr(), (*ent).s.number, (*ent).clipmask, G2_RETURNONHIT, 10);
            if VectorCompare((*ent).s.pos.trDelta.as_ptr(), addr_of!(vec3_origin) as *const vec3_t) != 0 {
                //VectorCopy( ent->currentAngles, ent->s.apos.trBase );
                VectorClear((*ent).s.apos.trDelta.as_mut_ptr());
            } else {
                let mut ang: vec3_t = [0.0; 3];
                let mut fwdDir: vec3_t = [0.0; 3];
                let mut rtDir: vec3_t = [0.0; 3];
                let speed: f32;

                (*ent).s.apos.trType = TR_INTERPOLATE;
                VectorSet(ang.as_mut_ptr(), 0.0, (*ent).s.apos.trBase[1], 0.0);
                AngleVectors(ang.as_ptr(), fwdDir.as_mut_ptr(), rtDir.as_mut_ptr(), core::ptr::null_mut());
                let speed = VectorLength((*ent).s.pos.trDelta.as_ptr()) * 4.0_f32;

                //HMM, this works along an axis-aligned dir, but not along diagonals
                //This is because when roll gets to 90, pitch becomes yaw, and vice-versa
                //Maybe need to just set the angles directly?
                (*ent).s.apos.trDelta[0] = DotProduct(fwdDir.as_ptr(), (*ent).s.pos.trDelta.as_ptr());
                (*ent).s.apos.trDelta[1] = 0.0; //never spin!
                (*ent).s.apos.trDelta[2] = DotProduct(rtDir.as_ptr(), (*ent).s.pos.trDelta.as_ptr());

                VectorNormalize((*ent).s.apos.trDelta.as_mut_ptr());
                VectorScale((*ent).s.apos.trDelta.as_ptr(), speed, (*ent).s.apos.trDelta.as_mut_ptr());

                (*ent).s.apos.trTime = (*addr_of!(level)).previousTime;
            }
        }
    } else {
        let mut origin: vec3_t = [0.0; 3];
        EvaluateTrajectory(addr_of!((*ent).s.pos), (*addr_of!(level)).time, origin.as_mut_ptr());
        // trace a line from the previous position to the current position,
        // ignoring interactions with the missile owner
        ((*addr_of!(gi)).trace)(
            addr_of_mut!(tr),
            (*ent).currentOrigin.as_ptr(),
            (*ent).mins.as_ptr(),
            (*ent).maxs.as_ptr(),
            origin.as_ptr(),
            if !(*ent).owner.is_null() { (*(*ent).owner).s.number } else { (*ent).s.number },
            (*ent).clipmask,
            G2_COLLIDE,
            10,
        );

        if tr.entityNum != ENTITYNUM_NONE {
            let other: *mut gentity_t = (addr_of_mut!(g_entities) as *mut gentity_t).add(tr.entityNum as usize);
            // check for hitting a lightsaber
            if (*other).contents & CONTENTS_LIGHTSABER != 0 {
                //hit a lightsaber bbox
                if !(*other).owner.is_null()
                    && !(*(*other).owner).client.is_null()
                    && (*(*(*other).owner).client).ps.saberInFlight == 0
                    && (Q_irand(0, (*(*(*other).owner).client).ps.forcePowerLevel[FP_SABER_DEFENSE as usize]
                        * (*(*(*other).owner).client).ps.forcePowerLevel[FP_SABER_DEFENSE as usize]) == 0
                        || InFront((*ent).currentOrigin.as_ptr(), (*(*other).owner).currentOrigin.as_ptr(), (*(*(*other).owner).client).ps.viewangles.as_ptr(), SABER_REFLECT_MISSILE_CONE) == 0)
                //other->owner->s.number == 0 &&
                {
                    //Jedi cannot block shots from behind!
                    //re-trace from here, ignoring the lightsaber
                    ((*addr_of!(gi)).trace)(addr_of_mut!(tr), tr.endpos.as_ptr(), (*ent).mins.as_ptr(), (*ent).maxs.as_ptr(), origin.as_ptr(), tr.entityNum, (*ent).clipmask, G2_RETURNONHIT, 10);
                }
            }
        }

        VectorCopy(tr.endpos.as_ptr(), (*ent).currentOrigin.as_mut_ptr());
    }

    // get current angles
    VectorMA((*ent).s.apos.trBase.as_ptr(), ((*addr_of!(level)).time - (*ent).s.apos.trTime) as f32 * 0.001_f32, (*ent).s.apos.trDelta.as_ptr(), (*ent).s.apos.trBase.as_mut_ptr());

    //FIXME: Rolling things hitting G2 polys is weird
    ///////////////////////////////////////////////////////
    //?	if ( tr.fraction != 1 )
    {
        // did we hit or go near a Ghoul2 model?
        //		qboolean hitModel = qfalse;
        for i in 0..MAX_G2_COLLISIONS {
            if tr.G2CollisionMap[i].mEntityNum == -1 {
                break;
            }

            let coll: &CCollisionRecord = &tr.G2CollisionMap[i];
            let hitEnt: *mut gentity_t = (addr_of_mut!(g_entities) as *mut gentity_t).add(coll.mEntityNum as usize);

            // process collision records here...
            // make sure we only do this once, not for all the entrance wounds we might generate
            if (coll.mFlags & G2_FRONTFACE != 0) /* && !(hitModel)*/ && (*hitEnt).health != 0 {
                if trHitLoc == HL_NONE {
                    G_GetHitLocFromSurfName(
                        (addr_of_mut!(g_entities) as *mut gentity_t).add(coll.mEntityNum as usize),
                        ((*addr_of!(gi)).G2API_GetSurfaceName)(
                            addr_of_mut!((*((addr_of_mut!(g_entities) as *mut gentity_t).add(coll.mEntityNum as usize))).ghoul2[coll.mModelIndex as usize]),
                            coll.mSurfaceIndex,
                        ),
                        addr_of_mut!(trHitLoc),
                        coll.mCollisionPosition.as_ptr(),
                        core::ptr::null(),
                        core::ptr::null(),
                        (*ent).methodOfDeath,
                    );
                }

                break; // NOTE: the way this whole section was working, it would only get inside of this IF once anyway, might as well break out now
            }
        }
    }
    /////////////////////////////////////////////////////////

    if tr.startsolid != 0 {
        tr.fraction = 0.0;
    }

    ((*addr_of!(gi)).linkentity)(ent);

    if (*ent).s.pos.trType == TR_STATIONARY && ((*ent).s.eFlags & EF_MISSILE_STICK != 0) {
        //stuck missiles should check some special stuff
        G_RunStuckMissile(ent);
        return;
    }

    // check think function
    G_RunThink(ent);

    if (*ent).s.eType != ET_MISSILE {
        return; // exploded
    }

    if (*ent).mass != 0 {
        G_MoverTouchPushTriggers(ent, oldOrg.as_ptr());
    }
    /*
    if ( !(ent->s.eFlags & EF_TELEPORT_BIT) )
    {
    	G_MoverTouchTeleportTriggers( ent, oldOrg );
    	if ( ent->s.eFlags & EF_TELEPORT_BIT )
    	{//was teleported
    		return;
    	}
    }
    else
    {
    	ent->s.eFlags &= ~EF_TELEPORT_BIT;
    }
    */

    AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 512, AEL_DISCOVERED, 75); //wakes them up when see a shot passes in front of them
    if Q_irand(0, 10) == 0 {
        //not so often...
        if (*ent).splashDamage != 0 && (*ent).splashRadius != 0.0 {
            //I'm an exploder, let people around me know danger is coming
            if (*ent).s.weapon == WP_TRIP_MINE {
                //???
            } else {
                if (*ent).s.weapon == WP_ROCKET_LAUNCHER && (*ent).e_ThinkFunc == thinkF_rocketThink {
                    //homing rocket- run like hell!
                    AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), (*ent).splashRadius as c_int, AEL_DANGER_GREAT, 50);
                } else {
                    AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), (*ent).splashRadius as c_int, AEL_DANGER, 50);
                }
                AddSoundEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), (*ent).splashRadius as c_int, AEL_DANGER, qfalse, qfalse);
            }
        } else {
            //makes them run from near misses
            AddSightEvent((*ent).owner, (*ent).currentOrigin.as_ptr(), 48, AEL_DANGER, 50);
        }
    }

    if tr.fraction == 1.0 {
        if (*ent).s.weapon == WP_THERMAL && (*ent).s.pos.trType == TR_INTERPOLATE {
            //a rolling thermal that didn't hit anything
            G_MissileAddAlerts(ent);
        }
        return;
    }

    // never explode or bounce on sky
    if tr.surfaceFlags & SURF_NOIMPACT != 0 {
        G_FreeEntity(ent);
        return;
    }

    G_MissileImpact(ent, addr_of_mut!(tr), trHitLoc);
}
