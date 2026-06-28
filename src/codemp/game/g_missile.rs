//! Port of `g_missile.c` — the projectile runtime: per-frame movement
//! (`G_RunMissile`), impact resolution (`G_MissileImpact`), and the bounce / reflect /
//! deflect / explode helpers that the weapon-fire code spawns missiles into. Landed
//! incrementally: only the functions whose full dep-set is already ported.
//!
//! `G_BounceProjectile` lands first — a self-contained ray-reflection helper (pure vec3
//! math). The reflect/deflect/bounce/explode/stuck/create/effect helpers follow once
//! their entity + trap deps are in, then `G_MissileImpact` (impact resolution into the
//! saber block / deflect / reflect / laser-trap paths) and `G_RunMissile` (per-frame
//! movement + trace). The DEMP2-vs-`CLASS_VEHICLE` sub-branch of `G_MissileImpact` calls
//! `FighterIsLanded` (FighterNPC.c), which is unported; it is referenced through an
//! `extern "C"` forward-decl so the rest of the file compiles and the branch is faithful
//! once FighterNPC.c lands.

#![allow(non_snake_case)] // C function names (`G_BounceProjectile`, …) kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::bg_misc::{BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta};
use crate::codemp::game::bg_public::{
    EF_ALT_FIRING, EF_JETPACK_ACTIVE, EF_MISSILE_STICK, ET_GENERAL, ET_MISSILE, ET_NPC,
    EV_GRENADE_BOUNCE, EV_MISSILE_HIT, EV_MISSILE_MISS, EV_MISSILE_MISS_METAL, EV_MISSILE_STICK,
    EV_SABER_BLOCK, G2_MODEL_PART, MOD_CONC, MOD_CONC_ALT, MOD_CRUSH, MOD_DEMP2_ALT,
    MOD_DET_PACK_SPLASH, MOD_FLECHETTE_ALT_SPLASH, MOD_REPEATER_ALT, MOD_ROCKET, MOD_ROCKET_HOMING,
    MOD_SABER, MOD_TARGET_LASER, MOD_THERMAL, MOD_THERMAL_SPLASH, MOD_TIMED_MINE_SPLASH,
    MOD_TRIP_MINE_SPLASH, MOD_TURBLAST, MOD_VEHICLE, PW_CLOAKED,
};
use crate::codemp::game::bg_vehicles_h::{VH_FIGHTER, VH_SPEEDER};
use crate::codemp::game::bg_weapons_h::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_PISTOL, WP_DEMP2, WP_DET_PACK, WP_EMPLACED_GUN,
    WP_FLECHETTE, WP_NONE, WP_NUM_WEAPONS, WP_ROCKET_LAUNCHER, WP_SABER, WP_THERMAL, WP_TRIP_MINE,
};
use crate::codemp::game::g_combat::{G_Damage, G_RadiusDamage};
use crate::codemp::game::g_local::{
    gentity_t, DAMAGE_HALF_ABSORB, DAMAGE_HEAVY_WEAP_CLASS, FL_BOUNCE, FL_BOUNCE_HALF,
    FL_BOUNCE_SHRAPNEL, FL_DMG_BY_HEAVY_WEAP_ONLY, FL_SHIELDED,
};
use crate::codemp::game::g_main::{
    d_projectileGhoul2Collision, g_entities, g_g2TraceLod, level, G_RunThink,
};
use crate::codemp::game::g_public_h::{
    G2TRFLAG_DOGHOULTRACE, G2TRFLAG_GETSURFINDEX, G2TRFLAG_HITCORPSES, G2TRFLAG_THICK, Q3_INFINITE,
    SVF_OWNERNOTSHARED, SVF_USE_CURRENT_ORIGIN,
};
use crate::codemp::game::g_utils::{
    G_AddEvent, G_EffectIndex, G_FreeEntity, G_PlayEffectID, G_SetOrigin, G_Sound, G_SoundIndex,
    G_Spawn, G_TempEntity,
};
use crate::codemp::game::g_weapon::{laserTrapStick, LogAccuracyHit, SnapVectorTowards};
use crate::codemp::game::npc_ai_jedi::Jedi_Decloak;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, AngleVectors, DirToByte, DotProduct, VectorAdd, VectorClear, VectorCompare,
    VectorCopy, VectorLength, VectorMA, VectorNormalize, VectorScale, VectorSubtract,
};
use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::q_shared_h::{
    trace_t, vec3_t, CHAN_BODY, ENTITYNUM_NONE, ENTITYNUM_WORLD, FORCE_LEVEL_1, FORCE_LEVEL_2,
    FORCE_LEVEL_3, FP_SABER_DEFENSE, MAX_CLIENTS, PITCH, ROLL, TR_GRAVITY, TR_LINEAR,
    TR_STATIONARY,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_LIGHTSABER, SURF_FORCEFIELD, SURF_METALSTEPS, SURF_NOIMPACT,
};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_GONK, CLASS_INTERROGATOR, CLASS_MARK1, CLASS_MARK2, CLASS_MOUSE, CLASS_PROBE,
    CLASS_R2D2, CLASS_R5D2, CLASS_REMOTE, CLASS_SEEKER, CLASS_SENTRY, CLASS_VEHICLE,
};
use crate::codemp::game::w_saber::{RandFloat, WP_SaberBlockNonRandom, WP_SaberCanBlock};
use crate::codemp::game::w_saber_h::SEF_DEFLECTED;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// `qboolean FighterIsLanded( Vehicle_t *pVeh, playerState_t *parentPS )` (FighterNPC.c:297)
// — called by the DEMP2-vs-`CLASS_VEHICLE` sub-branch of `G_MissileImpact`. Now ported in
// fighternpc.rs, so imported directly (the prior `extern "C"` forward-decl is dropped).
use crate::codemp::game::fighternpc::FighterIsLanded;

/// `void G_BounceProjectile( vec3_t start, vec3_t impact, vec3_t dir, vec3_t endout )`
/// (g_missile.c:277) — reflect the ray `start`→`impact` about the plane whose normal is
/// `dir`, then return in `endout` a point 8192 units along the reflected (unit) direction
/// from `impact`. Pure vec3 arithmetic; the `-2*dot` and `8192` int literals promote to
/// `float` exactly as in the C.
pub fn G_BounceProjectile(start: &vec3_t, impact: &vec3_t, dir: &vec3_t, endout: &mut vec3_t) {
    let mut v: vec3_t = [0.0; 3];
    let mut newv: vec3_t = [0.0; 3];

    VectorSubtract(impact, start, &mut v);
    let dot = DotProduct(&v, dir);
    VectorMA(&v, -2.0 * dot, dir, &mut newv);

    VectorNormalize(&mut newv);
    VectorMA(impact, 8192.0, &newv, endout);
}

/// `void G_ReflectMissile( gentity_t *ent, gentity_t *missile, vec3_t forward )`
/// (g_missile.c:24) — bounce `missile` roughly back at its owner (or away from `ent` when
/// `ent` is the owner). Computes a new unit bounce direction, jitters it by
/// `RandFloat(-0.2, 0.2)` per axis, and re-launches the missile at its saved speed (×1.5
/// when shoved by its own owner). Reassigns ownership and stops rocket homing.
///
/// No oracle — mutates entities and reads the global `g_entities`/`level`.
///
/// # Safety
/// `ent`/`missile` must be valid entities; `forward` is the deflector's forward vector.
pub unsafe fn G_ReflectMissile(ent: *mut gentity_t, missile: *mut gentity_t, forward: &vec3_t) {
    let mut bounce_dir: vec3_t = [0.0; 3];
    let mut isowner = 0;

    // (JKA computes a dead `owner` local here — `ent` or `g_entities[ent->r.ownerNum]` —
    // that it never reads; dropped, no behavioral change.)

    if (*missile).r.ownerNum == (*ent).s.number {
        // the original owner is bouncing the missile, so don't try to bounce it back at him
        isowner = 1;
    }

    // save the original speed
    let mut speed = VectorNormalize(&mut (*missile).s.pos.trDelta);

    // the C guards on `&g_entities[missile->r.ownerNum]`, which (address-of an array
    // element) is always non-NULL; mirrored here as the never-null base+index pointer.
    let missile_owner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
        .add((*missile).r.ownerNum as usize);
    if !missile_owner.is_null()
        && (*missile).s.weapon != WP_SABER
        && (*missile).s.weapon != G2_MODEL_PART
        && isowner == 0
    {
        // bounce back at them if you can
        VectorSubtract(
            &(*missile_owner).r.currentOrigin,
            &(*missile).r.currentOrigin,
            &mut bounce_dir,
        );
        VectorNormalize(&mut bounce_dir);
    } else if isowner != 0 {
        // in this case, actually push the missile away from me, and since we're giving
        // boost to our own missile by pushing it, up the velocity
        let mut missile_dir: vec3_t = [0.0; 3];

        speed *= 1.5;

        VectorSubtract(
            &(*missile).r.currentOrigin,
            &(*ent).r.currentOrigin,
            &mut missile_dir,
        );
        VectorCopy(&(*missile).s.pos.trDelta, &mut bounce_dir);
        let dot = DotProduct(forward, &missile_dir);
        let bd = bounce_dir;
        VectorScale(&bd, dot, &mut bounce_dir);
        VectorNormalize(&mut bounce_dir);
    } else {
        let mut missile_dir: vec3_t = [0.0; 3];

        VectorSubtract(
            &(*ent).r.currentOrigin,
            &(*missile).r.currentOrigin,
            &mut missile_dir,
        );
        VectorCopy(&(*missile).s.pos.trDelta, &mut bounce_dir);
        let dot = DotProduct(forward, &missile_dir);
        let bd = bounce_dir;
        VectorScale(&bd, dot, &mut bounce_dir);
        VectorNormalize(&mut bounce_dir);
    }
    for i in 0..3 {
        bounce_dir[i] += RandFloat(-0.2, 0.2);
    }

    VectorNormalize(&mut bounce_dir);
    VectorScale(&bounce_dir, speed, &mut (*missile).s.pos.trDelta);
    (*missile).s.pos.trTime = (*addr_of!(level)).time; // move a bit on the very first frame
    VectorCopy(&(*missile).r.currentOrigin, &mut (*missile).s.pos.trBase);
    if (*missile).s.weapon != WP_SABER && (*missile).s.weapon != G2_MODEL_PART {
        // you are mine, now!
        (*missile).r.ownerNum = (*ent).s.number;
    }
    if (*missile).s.weapon == WP_ROCKET_LAUNCHER {
        // stop homing
        (*missile).think = None;
        (*missile).nextthink = 0;
    }
}

/// `void G_DeflectMissile( gentity_t *ent, gentity_t *missile, vec3_t forward )`
/// (g_missile.c:91) — like [`G_ReflectMissile`] but a wider, less accurate bounce: the new
/// direction comes from the deflector's view (or `forward` for non-clients), jittered by
/// `RandFloat(-1.0, 1.0)` per axis, at the missile's saved speed.
///
/// No oracle — mutates entities and reads the global `level`.
///
/// # Safety
/// `ent`/`missile` must be valid entities; `forward` is the deflector's forward vector.
pub unsafe fn G_DeflectMissile(ent: *mut gentity_t, missile: *mut gentity_t, forward: &vec3_t) {
    let mut bounce_dir: vec3_t = [0.0; 3];
    let mut missile_dir: vec3_t = [0.0; 3];

    // (JKA computes a dead `isowner` local here that it never reads; dropped, no
    // behavioral change.)

    // save the original speed
    let speed = VectorNormalize(&mut (*missile).s.pos.trDelta);

    if !(*ent).client.is_null() {
        AngleVectors(
            &(*(*ent).client).ps.viewangles,
            Some(&mut missile_dir),
            None,
            None,
        );
        VectorCopy(&missile_dir, &mut bounce_dir);
        let dot = DotProduct(forward, &missile_dir);
        let bd = bounce_dir;
        VectorScale(&bd, dot, &mut bounce_dir);
        VectorNormalize(&mut bounce_dir);
    } else {
        VectorCopy(forward, &mut bounce_dir);
        VectorNormalize(&mut bounce_dir);
    }

    for i in 0..3 {
        bounce_dir[i] += RandFloat(-1.0, 1.0);
    }

    VectorNormalize(&mut bounce_dir);
    VectorScale(&bounce_dir, speed, &mut (*missile).s.pos.trDelta);
    (*missile).s.pos.trTime = (*addr_of!(level)).time; // move a bit on the very first frame
    VectorCopy(&(*missile).r.currentOrigin, &mut (*missile).s.pos.trBase);
    if (*missile).s.weapon != WP_SABER && (*missile).s.weapon != G2_MODEL_PART {
        // you are mine, now!
        (*missile).r.ownerNum = (*ent).s.number;
    }
    if (*missile).s.weapon == WP_ROCKET_LAUNCHER {
        // stop homing
        (*missile).think = None;
        (*missile).nextthink = 0;
    }
}

/// `void G_BounceMissile( gentity_t *ent, trace_t *trace )` (g_missile.c:148) — reflect a
/// missile's velocity off the trace plane and damp it (¼ for shrapnel → `TR_GRAVITY`, 0.65
/// for half-bounce), settling it onto the surface when it has slowed on a near-flat plane.
/// Plays the per-weapon bounce sound and decrements `bounceCount` (unless it is the
/// infinite `-5` sentinel).
///
/// No oracle — mutates the entity, reads `level`, and drives sound traps.
///
/// # Safety
/// `ent` must be a valid missile entity; `trace` a valid trace result.
pub unsafe fn G_BounceMissile(ent: *mut gentity_t, trace: *mut trace_t) {
    let mut velocity: vec3_t = [0.0; 3];

    // reflect the velocity on the trace plane
    let hitTime = ((*addr_of!(level)).previousTime as f32
        + ((*addr_of!(level)).time - (*addr_of!(level)).previousTime) as f32 * (*trace).fraction)
        as c_int;
    BG_EvaluateTrajectoryDelta(&(*ent).s.pos, hitTime, &mut velocity);
    let dot = DotProduct(&velocity, &(*trace).plane.normal);
    VectorMA(
        &velocity,
        -2.0 * dot,
        &(*trace).plane.normal,
        &mut (*ent).s.pos.trDelta,
    );

    if (*ent).flags & FL_BOUNCE_SHRAPNEL != 0 {
        let trd = (*ent).s.pos.trDelta;
        VectorScale(&trd, 0.25, &mut (*ent).s.pos.trDelta);
        (*ent).s.pos.trType = TR_GRAVITY;

        // check for stop
        // this can happen even on very slightly sloped walls, so changed it from > 0 to > 0.7
        if (*trace).plane.normal[2] > 0.7 && (*ent).s.pos.trDelta[2] < 40.0 {
            G_SetOrigin(ent, &(*trace).endpos);
            (*ent).nextthink = (*addr_of!(level)).time + 100;
            return;
        }
    } else if (*ent).flags & FL_BOUNCE_HALF != 0 {
        let trd = (*ent).s.pos.trDelta;
        VectorScale(&trd, 0.65, &mut (*ent).s.pos.trDelta);
        // check for stop
        if (*trace).plane.normal[2] > 0.2 && VectorLength(&(*ent).s.pos.trDelta) < 40.0 {
            G_SetOrigin(ent, &(*trace).endpos);
            return;
        }
    }

    if (*ent).s.weapon == WP_THERMAL {
        // slight hack for hit sound
        G_Sound(
            ent,
            CHAN_BODY,
            G_SoundIndex(&format!(
                "sound/weapons/thermal/bounce{}.wav",
                Q_irand(1, 2)
            )),
        );
    } else if (*ent).s.weapon == WP_SABER {
        G_Sound(
            ent,
            CHAN_BODY,
            G_SoundIndex(&format!("sound/weapons/saber/bounce{}.wav", Q_irand(1, 3))),
        );
    } else if (*ent).s.weapon == G2_MODEL_PART {
        //Limb bounce sound?
    }

    let co = (*ent).r.currentOrigin;
    VectorAdd(&co, &(*trace).plane.normal, &mut (*ent).r.currentOrigin);
    VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.pos.trBase);
    (*ent).s.pos.trTime = (*addr_of!(level)).time;

    if (*ent).bounceCount != -5 {
        (*ent).bounceCount -= 1;
    }
}

/// `void G_ExplodeMissile( gentity_t *ent )` (g_missile.c:215) — explode a missile without
/// an impact: snap it to its current trajectory point, turn it into an `ET_GENERAL` event
/// entity carrying `EV_MISSILE_MISS` (straight-up dir), free it after the event, and apply
/// any splash damage, crediting the owner's accuracy on a hit.
///
/// No oracle — mutates the entity, reads `g_entities`/`level`, and links via trap.
///
/// # Safety
/// `ent` must be a valid missile entity.
pub unsafe fn G_ExplodeMissile(ent: *mut gentity_t) {
    let mut dir: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];

    BG_EvaluateTrajectory(&(*ent).s.pos, (*addr_of!(level)).time, &mut origin);
    trap::SnapVector(&mut origin);
    G_SetOrigin(ent, &origin);

    // we don't have a valid direction, so just point straight up
    dir[0] = 0.0;
    dir[1] = 0.0;
    dir[2] = 1.0;

    (*ent).s.eType = ET_GENERAL;
    G_AddEvent(ent, EV_MISSILE_MISS, DirToByte(&dir));

    (*ent).freeAfterEvent = QTRUE;

    (*ent).takedamage = QFALSE;
    // splash damage
    if (*ent).splashDamage != 0 {
        //NOTE: vehicle missiles don't have an ent->parent set, so check that here and set it
        if (*ent).s.eType == ET_MISSILE //missile
            && ((*ent).s.eFlags & EF_JETPACK_ACTIVE) != 0 //vehicle missile
            && (*ent).r.ownerNum < MAX_CLIENTS as c_int
        //valid client owner
        {
            //set my parent to my owner for purposes of damage credit...
            (*ent).parent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*ent).r.ownerNum as usize);
        }
        if G_RadiusDamage(
            &(*ent).r.currentOrigin,
            (*ent).parent,
            (*ent).splashDamage as f32,
            (*ent).splashRadius as f32,
            ent,
            ent,
            (*ent).splashMethodOfDeath,
        ) != QFALSE
        {
            if !(*ent).parent.is_null() {
                let pe = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*ent).parent).s.number as usize);
                (*(*pe).client).accuracy_hits += 1;
            } else if !(*ent).activator.is_null() {
                let ae = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*(*ent).activator).s.number as usize);
                (*(*ae).client).accuracy_hits += 1;
            }
        }
    }

    trap::LinkEntity(ent);
}

/// `void G_RunStuckMissile( gentity_t *ent )` (g_missile.c:252) — per-frame handler for a
/// missile stuck to a surface: if the thing it stuck to has started moving or rotating,
/// crush-kill the missile; otherwise just run its think.
///
/// No oracle — reads `g_entities` and mutates entities.
///
/// # Safety
/// `ent` must be a valid missile entity.
pub unsafe fn G_RunStuckMissile(ent: *mut gentity_t) {
    if (*ent).takedamage != QFALSE {
        if (*ent).s.groundEntityNum >= 0 && (*ent).s.groundEntityNum < ENTITYNUM_WORLD {
            let other = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*ent).s.groundEntityNum as usize);

            if (VectorCompare(&vec3_origin, &(*other).s.pos.trDelta) == QFALSE
                && (*other).s.pos.trType != TR_STATIONARY)
                || (VectorCompare(&vec3_origin, &(*other).s.apos.trDelta) == QFALSE
                    && (*other).s.apos.trType != TR_STATIONARY)
            {
                // thing I stuck to is moving or rotating now, kill me
                G_Damage(
                    ent,
                    other,
                    other,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    99999,
                    0,
                    MOD_CRUSH,
                );
                return;
            }
        }
    }
    // check think function
    G_RunThink(ent);
}

/// `gentity_t *CreateMissile( vec3_t org, vec3_t dir, float vel, int life, gentity_t *owner,
/// qboolean altFire )` (g_missile.c:291) — spawn a linear `ET_MISSILE` entity at `org`
/// heading `dir * vel`, owned by `owner`, that frees itself after `life` ms. `org` is
/// snapped in place (as in the C, where the array param is mutated).
///
/// No oracle — allocates via `G_Spawn` and reads `level`.
///
/// # Safety
/// `owner` must be a valid entity; `g_entities`/`level` must be initialised so `G_Spawn`
/// can allocate.
pub unsafe fn CreateMissile(
    org: &mut vec3_t,
    dir: &vec3_t,
    vel: f32,
    life: c_int,
    owner: *mut gentity_t,
    altFire: qboolean,
) -> *mut gentity_t {
    let missile = G_Spawn();

    (*missile).nextthink = (*addr_of!(level)).time + life;
    (*missile).think = Some(G_FreeEntity);
    (*missile).s.eType = ET_MISSILE;
    (*missile).r.svFlags = SVF_USE_CURRENT_ORIGIN;
    (*missile).parent = owner;
    (*missile).r.ownerNum = (*owner).s.number;

    if altFire != QFALSE {
        (*missile).s.eFlags |= EF_ALT_FIRING;
    }

    (*missile).s.pos.trType = TR_LINEAR;
    (*missile).s.pos.trTime = (*addr_of!(level)).time; // - MISSILE_PRESTEP_TIME; // NOTENOTE This is a Quake 3 addition over JK2
    (*missile).target_ent = core::ptr::null_mut();

    trap::SnapVector(org);
    VectorCopy(org, &mut (*missile).s.pos.trBase);
    VectorScale(dir, vel, &mut (*missile).s.pos.trDelta);
    VectorCopy(org, &mut (*missile).r.currentOrigin);
    trap::SnapVector(&mut (*missile).s.pos.trDelta);

    missile
}

/// `void G_MissileBounceEffect( gentity_t *ent, vec3_t org, vec3_t dir )` (g_missile.c:324)
/// — play the per-weapon deflect effect at the bounce point: bowcaster/blaster get their
/// dedicated effect at the missile origin; everything else spawns an `EV_SABER_BLOCK` temp
/// entity at `org`.
///
/// No oracle — drives effect/temp-entity traps.
///
/// # Safety
/// `ent` must be a valid missile entity; `org`/`dir` are the bounce point and normal.
pub unsafe fn G_MissileBounceEffect(ent: *mut gentity_t, org: &vec3_t, dir: &vec3_t) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
    let w = (*ent).s.weapon;
    if w == WP_BOWCASTER {
        G_PlayEffectID(
            G_EffectIndex("bowcaster/deflect"),
            &(*ent).r.currentOrigin,
            dir,
        );
    } else if w == WP_BLASTER || w == WP_BRYAR_PISTOL {
        G_PlayEffectID(
            G_EffectIndex("blaster/deflect"),
            &(*ent).r.currentOrigin,
            dir,
        );
    } else {
        let te = G_TempEntity(org, EV_SABER_BLOCK);
        VectorCopy(org, &mut (*te).s.origin);
        VectorCopy(dir, &mut (*te).s.angles);
        (*te).s.eventParm = 0;
        (*te).s.weapon = 0; //saberNum
        (*te).s.legsAnim = 0; //bladeNum
    }
}

/// `void G_MissileImpact( gentity_t *ent, trace_t *trace )` (g_missile.c:353) — resolve a
/// missile striking `trace->entityNum`: bounce/stick handling, saber-block / deflect /
/// reflect against shielded or saber-wielding targets, impact + splash damage, DEMP2
/// vehicle-disable / decloak, then turn the missile into the appropriate hit/miss event
/// entity. Mirrors the C `goto killProj` flow with a labeled block.
///
/// No oracle — entity-state heavy: reads/mutates `g_entities`/`level`, dispatches into
/// the saber, vehicle and laser-trap subsystems, and links via trap.
///
/// # Safety
/// `ent` must be a valid missile entity; `trace` a valid trace result.
pub unsafe fn G_MissileImpact(ent: *mut gentity_t, trace: *mut trace_t) {
    let mut hitClient: qboolean = QFALSE;
    let mut isKnockedSaber: qboolean = QFALSE;

    let other =
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*trace).entityNum as usize);

    // check for bounce
    if (*other).takedamage == QFALSE
        && ((*ent).bounceCount > 0 || (*ent).bounceCount == -5)
        && ((*ent).flags & (FL_BOUNCE | FL_BOUNCE_HALF)) != 0
    {
        G_BounceMissile(ent, trace);
        G_AddEvent(ent, EV_GRENADE_BOUNCE, 0);
        return;
    } else if (*ent).neverFree != QFALSE
        && (*ent).s.weapon == WP_SABER
        && ((*ent).flags & FL_BOUNCE_HALF) != 0
    {
        //this is a knocked-away saber
        if (*ent).bounceCount > 0 || (*ent).bounceCount == -5 {
            G_BounceMissile(ent, trace);
            G_AddEvent(ent, EV_GRENADE_BOUNCE, 0);
            return;
        }

        isKnockedSaber = QTRUE;
    }

    // I would glom onto the FL_BOUNCE code section above, but don't feel like risking breaking something else
    if ((*other).takedamage == QFALSE
        && ((*ent).bounceCount > 0 || (*ent).bounceCount == -5)
        && ((*ent).flags & FL_BOUNCE_SHRAPNEL) != 0)
        || (((*trace).surfaceFlags & SURF_FORCEFIELD) != 0
            && (*ent).splashDamage == 0
            && (*ent).splashRadius == 0
            && ((*ent).bounceCount > 0 || (*ent).bounceCount == -5))
    {
        G_BounceMissile(ent, trace);

        if (*ent).bounceCount < 1 {
            (*ent).flags &= !FL_BOUNCE_SHRAPNEL;
        }
        return;
    }

    /*
    if ( !other->takedamage && ent->s.weapon == WP_THERMAL && !ent->alt_fire )
    {//rolling thermal det - FIXME: make this an eFlag like bounce & stick!!!
        //G_BounceRollMissile( ent, trace );
        if ( ent->owner && ent->owner->s.number == 0 )
        {
            G_MissileAddAlerts( ent );
        }
        //gi.linkentity( ent );
        return;
    }
    */

    // The C `goto killProj` targets are emulated by `break 'kill_proj` out of this block;
    // a `return` inside it mirrors the C `return` (skipping the killProj tail entirely).
    'kill_proj: {
        if ((*other).r.contents & CONTENTS_LIGHTSABER) != 0 && isKnockedSaber == QFALSE {
            //hit this person's saber, so..
            let otherOwner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*other).r.ownerNum as usize);

            if (*otherOwner).takedamage != QFALSE
                && !(*otherOwner).client.is_null()
                && (*(*otherOwner).client).ps.duelInProgress != QFALSE
                && (*(*otherOwner).client).ps.duelIndex != (*ent).r.ownerNum
            {
                break 'kill_proj;
            }
        } else if isKnockedSaber == QFALSE {
            if (*other).takedamage != QFALSE
                && !(*other).client.is_null()
                && (*(*other).client).ps.duelInProgress != QFALSE
                && (*(*other).client).ps.duelIndex != (*ent).r.ownerNum
            {
                break 'kill_proj;
            }
        }

        if ((*other).flags & FL_DMG_BY_HEAVY_WEAP_ONLY) != 0 {
            if (*ent).methodOfDeath != MOD_REPEATER_ALT
                && (*ent).methodOfDeath != MOD_ROCKET
                && (*ent).methodOfDeath != MOD_FLECHETTE_ALT_SPLASH
                && (*ent).methodOfDeath != MOD_ROCKET_HOMING
                && (*ent).methodOfDeath != MOD_THERMAL
                && (*ent).methodOfDeath != MOD_THERMAL_SPLASH
                && (*ent).methodOfDeath != MOD_TRIP_MINE_SPLASH
                && (*ent).methodOfDeath != MOD_TIMED_MINE_SPLASH
                && (*ent).methodOfDeath != MOD_DET_PACK_SPLASH
                && (*ent).methodOfDeath != MOD_VEHICLE
                && (*ent).methodOfDeath != MOD_CONC
                && (*ent).methodOfDeath != MOD_CONC_ALT
                && (*ent).methodOfDeath != MOD_SABER
                && (*ent).methodOfDeath != MOD_TURBLAST
                && (*ent).methodOfDeath != MOD_TARGET_LASER
            // &&
            //ent->methodOfDeath != MOD_COLLISION)
            {
                let mut fwd: vec3_t = [0.0; 3];

                if !trace.is_null() {
                    VectorCopy(&(*trace).plane.normal, &mut fwd);
                } else {
                    //oh well
                    AngleVectors(&(*other).r.currentAngles, Some(&mut fwd), None, None);
                }

                G_DeflectMissile(other, ent, &fwd);
                G_MissileBounceEffect(ent, &(*ent).r.currentOrigin, &fwd);
                return;
            }
        }

        if ((*other).flags & FL_SHIELDED) != 0
            && (*ent).s.weapon != WP_ROCKET_LAUNCHER
            && (*ent).s.weapon != WP_THERMAL
            && (*ent).s.weapon != WP_TRIP_MINE
            && (*ent).s.weapon != WP_DET_PACK
            && (*ent).s.weapon != WP_DEMP2
            && (*ent).s.weapon != WP_EMPLACED_GUN
            && (*ent).methodOfDeath != MOD_REPEATER_ALT
            && (*ent).methodOfDeath != MOD_FLECHETTE_ALT_SPLASH
            && (*ent).methodOfDeath != MOD_TURBLAST
            && (*ent).methodOfDeath != MOD_TARGET_LASER
            && (*ent).methodOfDeath != MOD_VEHICLE
            && (*ent).methodOfDeath != MOD_CONC
            && (*ent).methodOfDeath != MOD_CONC_ALT
            && ((*ent).dflags & DAMAGE_HEAVY_WEAP_CLASS) == 0
        {
            let mut fwd: vec3_t = [0.0; 3];

            if !(*other).client.is_null() {
                AngleVectors(
                    &(*(*other).client).ps.viewangles,
                    Some(&mut fwd),
                    None,
                    None,
                );
            } else {
                AngleVectors(&(*other).r.currentAngles, Some(&mut fwd), None, None);
            }

            G_DeflectMissile(other, ent, &fwd);
            G_MissileBounceEffect(ent, &(*ent).r.currentOrigin, &fwd);
            return;
        }

        if (*other).takedamage != QFALSE
            && !(*other).client.is_null()
            && (*ent).s.weapon != WP_ROCKET_LAUNCHER
            && (*ent).s.weapon != WP_THERMAL
            && (*ent).s.weapon != WP_TRIP_MINE
            && (*ent).s.weapon != WP_DET_PACK
            && (*ent).s.weapon != WP_DEMP2
            && (*ent).methodOfDeath != MOD_REPEATER_ALT
            && (*ent).methodOfDeath != MOD_FLECHETTE_ALT_SPLASH
            && (*ent).methodOfDeath != MOD_CONC
            && (*ent).methodOfDeath != MOD_CONC_ALT
            && (*(*other).client).ps.saberBlockTime < (*addr_of!(level)).time
            && isKnockedSaber == QFALSE
            && WP_SaberCanBlock(other, &(*ent).r.currentOrigin, 0, 0, QTRUE, 0) != 0
        {
            //only block one projectile per 200ms (to prevent giant swarms of projectiles being blocked)
            let mut fwd: vec3_t = [0.0; 3];
            let mut otherDefLevel =
                (*(*other).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize];

            let te = G_TempEntity(&(*ent).r.currentOrigin, EV_SABER_BLOCK);
            VectorCopy(&(*ent).r.currentOrigin, &mut (*te).s.origin);
            VectorCopy(&(*trace).plane.normal, &mut (*te).s.angles);
            (*te).s.eventParm = 0;
            (*te).s.weapon = 0; //saberNum
            (*te).s.legsAnim = 0; //bladeNum

            /*if (other->client->ps.velocity[2] > 0 ||
            other->client->pers.cmd.forwardmove ||
            other->client->pers.cmd.rightmove)
            */
            if (*(*other).client).ps.velocity[2] > 0.0
                || (*(*other).client).pers.cmd.forwardmove < 0
            //now we only do it if jumping or running backward. Should be able to full-on charge.
            {
                otherDefLevel -= 1;
                if otherDefLevel < 0 {
                    otherDefLevel = 0;
                }
            }

            AngleVectors(
                &(*(*other).client).ps.viewangles,
                Some(&mut fwd),
                None,
                None,
            );
            if otherDefLevel == FORCE_LEVEL_1 {
                //if def is only level 1, instead of deflecting the shot it should just die here
            } else if otherDefLevel == FORCE_LEVEL_2 {
                G_DeflectMissile(other, ent, &fwd);
            } else {
                G_ReflectMissile(other, ent, &fwd);
            }
            (*(*other).client).ps.saberBlockTime =
                (*addr_of!(level)).time + (350 - (otherDefLevel * 100)); //200;

            //For jedi AI
            (*(*other).client).ps.saberEventFlags |= SEF_DEFLECTED;

            if otherDefLevel == FORCE_LEVEL_3 {
                (*(*other).client).ps.saberBlockTime = 0; //^_^
            }

            if otherDefLevel == FORCE_LEVEL_1 {
                break 'kill_proj;
            }
            return;
        } else if ((*other).r.contents & CONTENTS_LIGHTSABER) != 0 && isKnockedSaber == QFALSE {
            //hit this person's saber, so..
            let otherOwner = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add((*other).r.ownerNum as usize);

            if (*otherOwner).takedamage != QFALSE
                && !(*otherOwner).client.is_null()
                && (*ent).s.weapon != WP_ROCKET_LAUNCHER
                && (*ent).s.weapon != WP_THERMAL
                && (*ent).s.weapon != WP_TRIP_MINE
                && (*ent).s.weapon != WP_DET_PACK
                && (*ent).s.weapon != WP_DEMP2
                && (*ent).methodOfDeath != MOD_REPEATER_ALT
                && (*ent).methodOfDeath != MOD_FLECHETTE_ALT_SPLASH
                && (*ent).methodOfDeath != MOD_CONC
                && (*ent).methodOfDeath != MOD_CONC_ALT
            /*&&
            otherOwner->client->ps.saberBlockTime < level.time*/
            {
                //for now still deflect even if saberBlockTime >= level.time because it hit the actual saber
                let mut fwd: vec3_t = [0.0; 3];
                let mut otherDefLevel =
                    (*(*otherOwner).client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize];

                //in this case, deflect it even if we can't actually block it because it hit our saber
                //WP_SaberCanBlock(otherOwner, ent->r.currentOrigin, 0, 0, qtrue, 0);
                if !(*otherOwner).client.is_null() && (*(*otherOwner).client).ps.weaponTime <= 0 {
                    WP_SaberBlockNonRandom(otherOwner, &(*ent).r.currentOrigin, QTRUE);
                }

                let te = G_TempEntity(&(*ent).r.currentOrigin, EV_SABER_BLOCK);
                VectorCopy(&(*ent).r.currentOrigin, &mut (*te).s.origin);
                VectorCopy(&(*trace).plane.normal, &mut (*te).s.angles);
                (*te).s.eventParm = 0;
                (*te).s.weapon = 0; //saberNum
                (*te).s.legsAnim = 0; //bladeNum

                /*if (otherOwner->client->ps.velocity[2] > 0 ||
                otherOwner->client->pers.cmd.forwardmove ||
                otherOwner->client->pers.cmd.rightmove)*/
                if (*(*otherOwner).client).ps.velocity[2] > 0.0
                    || (*(*otherOwner).client).pers.cmd.forwardmove < 0
                //now we only do it if jumping or running backward. Should be able to full-on charge.
                {
                    otherDefLevel -= 1;
                    if otherDefLevel < 0 {
                        otherDefLevel = 0;
                    }
                }

                AngleVectors(
                    &(*(*otherOwner).client).ps.viewangles,
                    Some(&mut fwd),
                    None,
                    None,
                );

                if otherDefLevel == FORCE_LEVEL_1 {
                    //if def is only level 1, instead of deflecting the shot it should just die here
                } else if otherDefLevel == FORCE_LEVEL_2 {
                    G_DeflectMissile(otherOwner, ent, &fwd);
                } else {
                    G_ReflectMissile(otherOwner, ent, &fwd);
                }
                (*(*otherOwner).client).ps.saberBlockTime =
                    (*addr_of!(level)).time + (350 - (otherDefLevel * 100)); //200;

                //For jedi AI
                (*(*otherOwner).client).ps.saberEventFlags |= SEF_DEFLECTED;

                if otherDefLevel == FORCE_LEVEL_3 {
                    (*(*otherOwner).client).ps.saberBlockTime = 0; //^_^
                }

                if otherDefLevel == FORCE_LEVEL_1 {
                    break 'kill_proj;
                }
                return;
            }
        }

        // check for sticking
        if (*other).takedamage == QFALSE && ((*ent).s.eFlags & EF_MISSILE_STICK) != 0 {
            laserTrapStick(
                ent,
                (*trace).endpos.as_ptr(),
                (*trace).plane.normal.as_ptr(),
            );
            G_AddEvent(ent, EV_MISSILE_STICK, 0);
            return;
        }

        // impact damage
        if (*other).takedamage != QFALSE && isKnockedSaber == QFALSE {
            // FIXME: wrong damage direction?
            if (*ent).damage != 0 {
                let mut velocity: vec3_t = [0.0; 3];
                let mut didDmg: qboolean = QFALSE;

                if LogAccuracyHit(
                    other,
                    (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*ent).r.ownerNum as usize),
                ) != QFALSE
                {
                    (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add((*ent).r.ownerNum as usize))
                    .client)
                        .accuracy_hits += 1;
                    hitClient = QTRUE;
                }
                BG_EvaluateTrajectoryDelta(&(*ent).s.pos, (*addr_of!(level)).time, &mut velocity);
                if VectorLength(&velocity) == 0.0 {
                    velocity[2] = 1.0; // stepped on a grenade
                }

                if (*ent).s.weapon == WP_BOWCASTER
                    || (*ent).s.weapon == WP_FLECHETTE
                    || (*ent).s.weapon == WP_ROCKET_LAUNCHER
                {
                    if (*ent).s.weapon == WP_FLECHETTE && ((*ent).s.eFlags & EF_ALT_FIRING) != 0 {
                        (*ent).think.unwrap()(ent);
                    } else {
                        G_Damage(
                            other,
                            ent,
                            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                                .add((*ent).r.ownerNum as usize),
                            &mut velocity,
                            /*ent->s.origin*/ &mut (*ent).r.currentOrigin,
                            (*ent).damage,
                            DAMAGE_HALF_ABSORB,
                            (*ent).methodOfDeath,
                        );
                        didDmg = QTRUE;
                    }
                } else {
                    G_Damage(
                        other,
                        ent,
                        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                            .add((*ent).r.ownerNum as usize),
                        &mut velocity,
                        /*ent->s.origin*/ &mut (*ent).r.currentOrigin,
                        (*ent).damage,
                        0,
                        (*ent).methodOfDeath,
                    );
                    didDmg = QTRUE;
                }

                if didDmg != QFALSE && !other.is_null() && !(*other).client.is_null() {
                    //What I'm wondering is why this isn't in the NPC pain funcs. But this is what SP does, so whatever.
                    let npc_class = (*(*other).client).NPC_class;

                    // If we are a robot and we aren't currently doing the full body electricity...
                    if npc_class == CLASS_SEEKER
                        || npc_class == CLASS_PROBE
                        || npc_class == CLASS_MOUSE
                        || npc_class == CLASS_GONK
                        || npc_class == CLASS_R2D2
                        || npc_class == CLASS_R5D2
                        || npc_class == CLASS_REMOTE
                        || npc_class == CLASS_MARK1
                        || npc_class == CLASS_MARK2
                        || //npc_class == CLASS_PROTOCOL ||//no protocol, looks odd
                        npc_class == CLASS_INTERROGATOR
                        || npc_class == CLASS_ATST
                        || npc_class == CLASS_SENTRY
                    {
                        // special droid only behaviors
                        if (*(*other).client).ps.electrifyTime < (*addr_of!(level)).time + 100 {
                            // ... do the effect for a split second for some more feedback
                            (*(*other).client).ps.electrifyTime = (*addr_of!(level)).time + 450;
                        }
                        //FIXME: throw some sparks off droids,too
                    }
                }
            }

            if (*ent).s.weapon == WP_DEMP2 {
                //a hit with demp2 decloaks people, disables ships
                if !other.is_null()
                    && !(*other).client.is_null()
                    && (*(*other).client).NPC_class == CLASS_VEHICLE
                {
                    //hit a vehicle
                    if !(*other).m_pVehicle.is_null() //valid vehicle ent
                        && !(*(*other).m_pVehicle).m_pVehicleInfo.is_null()//valid stats
                        && ((*(*(*other).m_pVehicle).m_pVehicleInfo).r#type == VH_SPEEDER//always affect speeders
                            || ((*(*(*other).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER
                                && !(*ent).classname.is_null()
                                && Q_stricmp(c"vehicle_proj".as_ptr(), (*ent).classname) == 0))//only vehicle ion weapons affect a fighter in this manner
                        && FighterIsLanded((*other).m_pVehicle, &mut (*(*other).client).ps) == QFALSE//not landed
                        && ((*other).spawnflags & 2) == 0
                    //and not suspended
                    {
                        //vehicles hit by "ion cannons" lose control
                        if (*(*other).client).ps.electrifyTime > (*addr_of!(level)).time {
                            //add onto it
                            //FIXME: extern the length of the "out of control" time?
                            (*(*other).client).ps.electrifyTime += Q_irand(200, 500);
                            if (*(*other).client).ps.electrifyTime > (*addr_of!(level)).time + 4000
                            {
                                //cap it
                                (*(*other).client).ps.electrifyTime =
                                    (*addr_of!(level)).time + 4000;
                            }
                        } else {
                            //start it
                            //FIXME: extern the length of the "out of control" time?
                            (*(*other).client).ps.electrifyTime =
                                (*addr_of!(level)).time + Q_irand(200, 500);
                        }
                    }
                } else if !other.is_null()
                    && !(*other).client.is_null()
                    && (*(*other).client).ps.powerups[PW_CLOAKED as usize] != 0
                {
                    Jedi_Decloak(other);
                    if (*ent).methodOfDeath == MOD_DEMP2_ALT {
                        //direct hit with alt disables cloak forever
                        //permanently disable the saboteur's cloak
                        (*(*other).client).cloakToggleTime = Q3_INFINITE;
                    } else {
                        //temp disable
                        (*(*other).client).cloakToggleTime =
                            (*addr_of!(level)).time + Q_irand(3000, 10000);
                    }
                }
            }
        }
    } // killProj:
      // is it cheaper in bandwidth to just remove this ent and create a new
      // one, rather than changing the missile into the explosion?

    if (*other).takedamage != QFALSE && !(*other).client.is_null() && isKnockedSaber == QFALSE {
        G_AddEvent(ent, EV_MISSILE_HIT, DirToByte(&(*trace).plane.normal));
        (*ent).s.otherEntityNum = (*other).s.number;
    } else if ((*trace).surfaceFlags & SURF_METALSTEPS) != 0 {
        G_AddEvent(
            ent,
            EV_MISSILE_MISS_METAL,
            DirToByte(&(*trace).plane.normal),
        );
    } else if (*ent).s.weapon != G2_MODEL_PART && isKnockedSaber == QFALSE {
        G_AddEvent(ent, EV_MISSILE_MISS, DirToByte(&(*trace).plane.normal));
    }

    if isKnockedSaber == QFALSE {
        (*ent).freeAfterEvent = QTRUE;

        // change over to a normal entity right at the point of impact
        (*ent).s.eType = ET_GENERAL;
    }

    SnapVectorTowards(&mut (*trace).endpos, &(*ent).s.pos.trBase); // save net bandwidth

    G_SetOrigin(ent, &(*trace).endpos);

    (*ent).takedamage = QFALSE;
    // splash damage (doesn't apply to person directly hit)
    if (*ent).splashDamage != 0 {
        if G_RadiusDamage(
            &(*trace).endpos,
            (*ent).parent,
            (*ent).splashDamage as f32,
            (*ent).splashRadius as f32,
            other,
            ent,
            (*ent).splashMethodOfDeath,
        ) != QFALSE
        {
            if hitClient == QFALSE
                && !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*ent).r.ownerNum as usize))
                .client
                .is_null()
            {
                (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                    .add((*ent).r.ownerNum as usize))
                .client)
                    .accuracy_hits += 1;
            }
        }
    }

    if (*ent).s.weapon == G2_MODEL_PART {
        (*ent).freeAfterEvent = QFALSE; //it will free itself
    }

    trap::LinkEntity(ent);
}

/// `void G_RunMissile( gentity_t *ent )` (g_missile.c:792) — advance a missile one frame:
/// evaluate its trajectory, trace (ghoul2 or hull) from its old to its new position,
/// handle pass-through, surface marks and sky no-impact, route any hit into
/// [`G_MissileImpact`], settle a grounded limb part, and finally run its think.
///
/// No oracle — entity-state heavy: traces via trap, reads `g_entities`/`level`/cvars,
/// and mutates the missile entity.
///
/// # Safety
/// `ent` must be a valid missile entity.
pub unsafe fn G_RunMissile(ent: *mut gentity_t) {
    let mut origin: vec3_t = [0.0; 3];
    let mut groundSpot: vec3_t = [0.0; 3];
    let passent: c_int;
    let mut isKnockedSaber: qboolean = QFALSE;

    if (*ent).neverFree != QFALSE
        && (*ent).s.weapon == WP_SABER
        && ((*ent).flags & FL_BOUNCE_HALF) != 0
    {
        isKnockedSaber = QTRUE;
        (*ent).s.pos.trType = TR_GRAVITY;
    }

    // get current position
    BG_EvaluateTrajectory(&(*ent).s.pos, (*addr_of!(level)).time, &mut origin);

    // if this missile bounced off an invulnerability sphere
    if !(*ent).target_ent.is_null() {
        passent = (*(*ent).target_ent).s.number;
    } else {
        // ignore interactions with the missile owner
        if ((*ent).r.svFlags & SVF_OWNERNOTSHARED) != 0
            && ((*ent).s.eFlags & EF_JETPACK_ACTIVE) != 0
        {
            //A vehicle missile that should be solid to its owner
            //I don't care about hitting my owner
            passent = (*ent).s.number;
        } else {
            passent = (*ent).r.ownerNum;
        }
    }
    // trace a line from the previous position to the current position
    let mut tr: trace_t;
    if d_projectileGhoul2Collision.integer != 0 {
        tr = trap::G2Trace(
            &(*ent).r.currentOrigin,
            &(*ent).r.mins,
            &(*ent).r.maxs,
            &origin,
            passent,
            (*ent).clipmask,
            G2TRFLAG_DOGHOULTRACE | G2TRFLAG_GETSURFINDEX | G2TRFLAG_THICK | G2TRFLAG_HITCORPSES,
            g_g2TraceLod.integer,
        );

        if tr.fraction != 1.0 && (tr.entityNum as c_int) < ENTITYNUM_WORLD {
            let g2Hit = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                .add(tr.entityNum as usize);

            if (*g2Hit).inuse != QFALSE && !(*g2Hit).client.is_null() && !(*g2Hit).ghoul2.is_null()
            {
                //since we used G2TRFLAG_GETSURFINDEX, tr.surfaceFlags will actually contain the index of the surface on the ghoul2 model we collided with.
                (*(*g2Hit).client).g2LastSurfaceHit = tr.surfaceFlags;
                (*(*g2Hit).client).g2LastSurfaceTime = (*addr_of!(level)).time;
            }

            if !(*g2Hit).ghoul2.is_null() {
                tr.surfaceFlags = 0; //clear the surface flags after, since we actually care about them in here.
            }
        }
    } else {
        tr = trap::Trace(
            &(*ent).r.currentOrigin,
            &(*ent).r.mins,
            &(*ent).r.maxs,
            &origin,
            passent,
            (*ent).clipmask,
        );
    }

    if tr.startsolid != 0 || tr.allsolid != 0 {
        // make sure the tr.entityNum is set to the entity we're stuck in
        tr = trap::Trace(
            &(*ent).r.currentOrigin,
            &(*ent).r.mins,
            &(*ent).r.maxs,
            &(*ent).r.currentOrigin,
            passent,
            (*ent).clipmask,
        );
        tr.fraction = 0.0;
    } else {
        VectorCopy(&tr.endpos, &mut (*ent).r.currentOrigin);
    }

    // The C `goto passthrough` skips the trace-result handling below; emulated by a
    // labeled block that the pass-through case `break`s out of.
    'passthrough: {
        if (*ent).passThroughNum != 0 && tr.entityNum as c_int == ((*ent).passThroughNum - 1) {
            VectorCopy(&origin, &mut (*ent).r.currentOrigin);
            trap::LinkEntity(ent);
            break 'passthrough;
        }

        trap::LinkEntity(ent);

        if (*ent).s.weapon == G2_MODEL_PART && (*ent).bounceCount == 0 {
            let mut lowerOrg: vec3_t = [0.0; 3];

            VectorCopy(&(*ent).r.currentOrigin, &mut lowerOrg);
            lowerOrg[2] -= 1.0;
            let trG = trap::Trace(
                &(*ent).r.currentOrigin,
                &(*ent).r.mins,
                &(*ent).r.maxs,
                &lowerOrg,
                passent,
                (*ent).clipmask,
            );

            VectorCopy(&trG.endpos, &mut groundSpot);

            if trG.startsolid == 0 && trG.allsolid == 0 && trG.entityNum as c_int == ENTITYNUM_WORLD
            {
                (*ent).s.groundEntityNum = trG.entityNum as c_int;
            } else {
                (*ent).s.groundEntityNum = ENTITYNUM_NONE;
            }
        }

        if tr.fraction != 1.0 {
            // never explode or bounce on sky
            if (tr.surfaceFlags & SURF_NOIMPACT) != 0 {
                // If grapple, reset owner
                if !(*ent).parent.is_null()
                    && !(*(*ent).parent).client.is_null()
                    && (*(*(*ent).parent).client).hook == ent
                {
                    (*(*(*ent).parent).client).hook = core::ptr::null_mut();
                }

                if ((*ent).s.weapon == WP_SABER && (*ent).isSaberEntity != QFALSE)
                    || isKnockedSaber != QFALSE
                {
                    G_RunThink(ent);
                    return;
                } else if (*ent).s.weapon != G2_MODEL_PART {
                    G_FreeEntity(ent);
                    return;
                }
            }

            // #if 0 block (the EV_GHOUL2_MARK tempent path) is dead in JKA; the #else
            // branch below is the live one.
            if (*ent).s.weapon > WP_NONE
                && (*ent).s.weapon < WP_NUM_WEAPONS
                && ((tr.entityNum as c_int) < MAX_CLIENTS as c_int
                    || (*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
                        .add(tr.entityNum as usize))
                    .s
                    .eType
                        == ET_NPC)
            {
                //player or NPC, try making a mark on him
                //copy current pos to s.origin, and current projected traj to origin2
                VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.origin);
                BG_EvaluateTrajectory(
                    &(*ent).s.pos,
                    (*addr_of!(level)).time,
                    &mut (*ent).s.origin2,
                );

                if VectorCompare(&(*ent).s.origin, &(*ent).s.origin2) != QFALSE {
                    (*ent).s.origin2[2] += 2.0; //whatever, at least it won't mess up.
                }
            }

            G_MissileImpact(ent, &mut tr);

            if tr.entityNum as c_int == (*ent).s.otherEntityNum {
                //if the impact event other and the trace ent match then it's ok to do the g2 mark
                (*ent).s.trickedentindex = 1;
            }

            if (*ent).s.eType != ET_MISSILE && (*ent).s.weapon != G2_MODEL_PART {
                return; // exploded
            }
        }
    } // passthrough:

    if (*ent).s.pos.trType == TR_STATIONARY && ((*ent).s.eFlags & EF_MISSILE_STICK) != 0 {
        //stuck missiles should check some special stuff
        G_RunStuckMissile(ent);
        return;
    }

    if (*ent).s.weapon == G2_MODEL_PART {
        if (*ent).s.groundEntityNum == ENTITYNUM_WORLD {
            (*ent).s.pos.trType = TR_LINEAR;
            VectorClear(&mut (*ent).s.pos.trDelta);
            (*ent).s.pos.trTime = (*addr_of!(level)).time;

            VectorCopy(&groundSpot, &mut (*ent).s.pos.trBase);
            VectorCopy(&groundSpot, &mut (*ent).r.currentOrigin);

            if (*ent).s.apos.trType != TR_STATIONARY {
                (*ent).s.apos.trType = TR_STATIONARY;
                (*ent).s.apos.trTime = (*addr_of!(level)).time;

                (*ent).s.apos.trBase[ROLL] = 0.0;
                (*ent).s.apos.trBase[PITCH] = 0.0;
            }
        }
    }

    // check think function after bouncing
    G_RunThink(ent);
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    #[test]
    fn G_BounceProjectile_matches_oracle_bit_exact() {
        // ray endpoints and plane normals spanning signs, magnitudes and non-unit dirs —
        // the reflection formula and the final normalize must agree bit-for-bit.
        let cases: [([f32; 3], [f32; 3], [f32; 3]); 8] = [
            ([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            ([0.0, 0.0, 0.0], [10.0, 5.0, -3.0], [0.0, 1.0, 0.0]),
            ([-5.0, 2.0, 1.0], [5.0, -2.0, -1.0], [1.0, 0.0, 0.0]),
            ([1.5, 2.5, 3.5], [-1.5, -2.5, -3.5], [-1.0, 0.0, 0.0]),
            (
                [100.0, 200.0, 300.0],
                [0.0, 0.0, 0.0],
                [0.577, 0.577, 0.577],
            ),
            (
                [-1234.5, 9999.9, -0.001],
                [12.0, -34.0, 56.0],
                [0.0, -1.0, 0.0],
            ),
            ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, -3.0, 4.0]),
            ([7.0, 7.0, 7.0], [7.0, 7.0, 7.0], [0.0, 0.0, 1.0]),
        ];
        for (start, impact, dir) in cases {
            let mut rust: vec3_t = [0.0; 3];
            G_BounceProjectile(&start, &impact, &dir, &mut rust);
            let mut c: vec3_t = [0.0; 3];
            unsafe {
                oracle::jka_G_BounceProjectile(
                    start.as_ptr(),
                    impact.as_ptr(),
                    dir.as_ptr(),
                    c.as_mut_ptr(),
                )
            };
            for k in 0..3 {
                assert_eq!(
                    rust[k].to_bits(),
                    c[k].to_bits(),
                    "component {k} for start={start:?} impact={impact:?} dir={dir:?}"
                );
            }
        }
    }
}
