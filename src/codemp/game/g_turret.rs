//! Port of `g_turret.c` — the two-piece `misc_turret` turbolaser (a Hoth-style
//! auto-targeting turret built from a stationary base plus a rotating top). COMPLETE: all of
//! the turret's pain/die/use callbacks, the fire/aim/think machinery
//! (`turret_fire`/`turret_aim`/`turret_head_think`/`turret_find_enemies`/`turret_base_think`),
//! and the spawn plumbing (`SP_misc_turret`/`turret_base_spawn_top`) are ported here.
//!
//! All callbacks here are No-oracle (engine-syscall / global level/entity plumbing).

#![allow(non_snake_case)] // C function names (`TurretPain`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro names kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::addr_of;

use crate::codemp::game::bg_misc::{BG_EvaluateTrajectory, BG_FindItemForWeapon};
use crate::codemp::game::bg_public::{
    EFFECT_EXPLOSION_TURRET, EFFECT_SPARKS, ET_GENERAL, ET_MISSILE, ET_NPC, MASK_SHOT,
    MOD_TARGET_LASER, MOD_UNKNOWN, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_vehicles_h::VH_WALKER;
use crate::codemp::game::bg_weapons_h::{WP_DEMP2, WP_EMPLACED_GUN};
use crate::codemp::game::g_combat::{G_RadiusDamage, ObjectDie};
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::{gentity_t, FL_NOTARGET, FRAMETIME};
use crate::codemp::game::g_main::{g_entities, level};
use crate::codemp::game::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString};
use crate::codemp::game::g_utils::{
    G_EffectIndex, G_FreeEntity, G_IconIndex, G_ModelIndex, G_PlayEffect, G_PlayEffectID,
    G_RadiusList, G_ScaleNetHealth, G_SetAngles, G_SetOrigin, G_SoundIndex, G_Spawn, G_UseTargets,
    G_UseTargets2,
};
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, vectoangles, AngleNormalize180, AngleSubtract, AngleVectors, VectorClear,
    VectorCopy, VectorLengthSquared, VectorMA, VectorScale, VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::{random, Q_stricmp};
use crate::codemp::game::q_shared_h::{
    vec3_t, MAT_METAL, MAX_GENTITIES, PITCH, TR_LINEAR, TR_LINEAR_STOP, TR_STATIONARY, YAW,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_BODY, CONTENTS_LIGHTSABER};
use crate::codemp::game::teams_h::CLASS_VEHICLE;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// `atoi` is the C-library implementation (the toolchain links it from <stdlib.h>),
// matching the original's locale-default parse — see g_spawn.rs / bg_misc.rs.
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
}

//------------------------------------------------------------------------------------------------------------
/// `void TurretPain( gentity_t *self, gentity_t *attacker, int damage )` (g_turret.c:11) —
/// the turret top's `pain` callback. Mirrors `health` into the linked base entity
/// (`target_ent`) so the crosshair health bar tracks the damage, stuns the turret for a
/// random window when hit by the DEMP2, and acquires the attacker as an enemy if we don't
/// already have one. `damage` is unused, matching the C signature. No oracle (mutates
/// `gentity_t`s and reads the file-static `level`).
//------------------------------------------------------------------------------------------------------------
pub unsafe extern "C" fn TurretPain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
) {
    if !(*self_).target_ent.is_null() {
        (*(*self_).target_ent).health = (*self_).health;
        if (*(*self_).target_ent).maxHealth != 0 {
            G_ScaleNetHealth((*self_).target_ent);
        }
    }

    if !(*attacker).client.is_null() && (*(*attacker).client).ps.weapon == WP_DEMP2 {
        (*self_).attackDebounceTime = (*addr_of!(level)).time + 800 + (random() * 500.0) as c_int;
        (*self_).painDebounceTime = (*self_).attackDebounceTime;
    }
    if (*self_).enemy.is_null() {
        //react to being hit
        G_SetEnemy(self_, attacker);
    }
}

//------------------------------------------------------------------------------------------------------------
/// `void TurretBasePain( gentity_t *self, gentity_t *attacker, int damage )` (g_turret.c:35) —
/// the turret base's `pain` callback. Mirrors `health` into the linked top entity
/// (`target_ent`) for the crosshair health bar, then forwards the pain to the top via
/// [`TurretPain`]. No oracle (mutates `gentity_t`s).
//------------------------------------------------------------------------------------------------------------
pub unsafe extern "C" fn TurretBasePain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    if !(*self_).target_ent.is_null() {
        (*(*self_).target_ent).health = (*self_).health;
        if (*(*self_).target_ent).maxHealth != 0 {
            G_ScaleNetHealth((*self_).target_ent);
        }

        TurretPain((*self_).target_ent, attacker, damage);
    }
}

//------------------------------------------------------------------------------------------------------------
/// `void auto_turret_die ( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int meansOfDeath )`
/// (g_turret.c:51) — the turret top's `die` callback. Shuts down the base's thinking,
/// clears its own combat data, plays the explosion effect, dishes out splash damage, and
/// then either switches to the damaged model (and fires its `target`) or falls through to
/// [`ObjectDie`]. No oracle (mutates `gentity_t`s and the `g_entities` array).
//------------------------------------------------------------------------------------------------------------
pub unsafe extern "C" fn auto_turret_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    means_of_death: c_int,
) {
    let forward: vec3_t = [0.0, 0.0, 1.0];
    let mut pos: vec3_t = [0.0; 3];

    // Turn off the thinking of the base & use it's targets
    (*core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add((*self_).r.ownerNum as usize))
    .think = None;
    (*core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add((*self_).r.ownerNum as usize))
    .r#use = None;

    // clear my data
    (*self_).die = None;
    (*self_).takedamage = QFALSE;
    (*self_).s.health = 0;
    (*self_).health = 0;
    (*self_).s.loopSound = 0;
    (*self_).s.shouldtarget = QFALSE;
    //self->s.owner = MAX_CLIENTS; //not owned by any client

    VectorCopy(&(*self_).r.currentOrigin, &mut pos);
    pos[2] += (*self_).r.maxs[2] * 0.5;
    G_PlayEffect(EFFECT_EXPLOSION_TURRET as c_int, &pos, &forward);
    G_PlayEffectID(G_EffectIndex("turret/explode"), &pos, &forward);

    if (*self_).splashDamage > 0 && (*self_).splashRadius > 0 {
        G_RadiusDamage(
            &(*self_).r.currentOrigin,
            attacker,
            (*self_).splashDamage as f32,
            (*self_).splashRadius as f32,
            attacker,
            core::ptr::null_mut(),
            MOD_UNKNOWN,
        );
    }

    (*self_).s.weapon = 0; // crosshair code uses this to mark crosshair red

    if (*self_).s.modelindex2 != 0 {
        // switch to damage model if we should
        (*self_).s.modelindex = (*self_).s.modelindex2;

        if !(*self_).target_ent.is_null() && (*(*self_).target_ent).s.modelindex2 != 0 {
            (*(*self_).target_ent).s.modelindex = (*(*self_).target_ent).s.modelindex2;
        }

        VectorCopy(&(*self_).r.currentAngles, &mut (*self_).s.apos.trBase);
        VectorClear(&mut (*self_).s.apos.trDelta);

        if !(*self_).target.is_null() {
            G_UseTargets(self_, attacker);
        }
    } else {
        ObjectDie(self_, inflictor, attacker, damage, means_of_death);
    }
}

//------------------------------------------------------------------------------------------------------------
/// `void bottom_die ( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int meansOfDeath )`
/// (g_turret.c:113) — the turret base's `die` callback. Mirrors `health` into the top
/// (`target_ent`) and forwards the death to the top via [`auto_turret_die`]. No oracle
/// (mutates `gentity_t`s).
//------------------------------------------------------------------------------------------------------------
pub unsafe extern "C" fn bottom_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    means_of_death: c_int,
) {
    if !(*self_).target_ent.is_null() && (*(*self_).target_ent).health > 0 {
        (*(*self_).target_ent).health = (*self_).health;
        if (*(*self_).target_ent).maxHealth != 0 {
            G_ScaleNetHealth((*self_).target_ent);
        }
        auto_turret_die(
            (*self_).target_ent,
            inflictor,
            attacker,
            damage,
            means_of_death,
        );
    }
}

//-----------------------------------------------------------------------------
/// `void turret_base_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_turret.c:608) — the turret base's `use` callback: toggles the turret on/off by
/// flipping the START_OFF spawnflag. `other`/`activator` are unused. No oracle (mutates a
/// `gentity_t`).
//-----------------------------------------------------------------------------
pub unsafe extern "C" fn turret_base_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    // Toggle on and off
    (*self_).spawnflags ^= 1;

    /*
    if (( self->s.eFlags & EF_SHADER_ANIM ) && ( self->spawnflags & 1 )) // Start_Off
    {
        self->s.frame = 1; // black
    }
    else
    {
        self->s.frame = 0; // glow
    }
    */
}

//-----------------------------------------------------
/// `static void turret_aim( gentity_t *self )` (g_turret.c:232) — drives the turret top's
/// angular trajectory each frame. Evaluates the base (`top`) current angles, then computes
/// desired yaw/pitch: jittering randomly when stunned (`painDebounceTime`), aiming at the
/// enemy (with a `+32` height hack when the enemy is a `VH_WALKER` vehicle NPC) when one
/// exists, or sweeping back and forth (`sin`) while searching. Caps the per-frame turn by
/// `turnSpeed`, writes a `TR_LINEAR_STOP` apos trajectory into the base, and toggles the
/// turn loop-sound. The Ghoul2 eye-bolt matrix lookup is preserved as a comment. No oracle
/// (entity-state: mutates `gentity_t`s, reads `g_entities`/`level`, plays a sound index).
//-----------------------------------------------------
#[allow(dead_code)] // only caller is turret_base_think (g_turret.c:604)
unsafe fn turret_aim(self_: *mut gentity_t) {
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];
    let mut desiredAngles: vec3_t = [0.0; 3];
    let mut setAngle: vec3_t = [0.0; 3];
    // C: `float diffYaw = 0.0f, diffPitch = 0.0f` — the initializers are dead here because
    // the if/else-if/else below assigns both on every path; declared uninitialized to keep a
    // clean build while preserving the original control flow.
    let mut diffYaw: f32;
    let mut diffPitch: f32;
    let mut turnSpeed: f32;
    let pitchCap: f32 = 40.0;
    let top = core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add((*self_).r.ownerNum as usize);
    if top.is_null() {
        return;
    }

    // move our gun base yaw to where we should be at this time....
    BG_EvaluateTrajectory(
        &(*top).s.apos,
        (*addr_of!(level)).time,
        &mut (*top).r.currentAngles,
    );
    (*top).r.currentAngles[YAW] = AngleNormalize180((*top).r.currentAngles[YAW]);
    (*top).r.currentAngles[PITCH] = AngleNormalize180((*top).r.currentAngles[PITCH]);
    turnSpeed = (*top).speed;

    if (*self_).painDebounceTime > (*addr_of!(level)).time {
        desiredAngles[YAW] = (*top).r.currentAngles[YAW] + flrand(-45.0, 45.0);
        desiredAngles[PITCH] = (*top).r.currentAngles[PITCH] + flrand(-10.0, 10.0);

        if desiredAngles[PITCH] < -pitchCap {
            desiredAngles[PITCH] = -pitchCap;
        } else if desiredAngles[PITCH] > pitchCap {
            desiredAngles[PITCH] = pitchCap;
        }

        diffYaw = AngleSubtract(desiredAngles[YAW], (*top).r.currentAngles[YAW]);
        diffPitch = AngleSubtract(desiredAngles[PITCH], (*top).r.currentAngles[PITCH]);
        turnSpeed = flrand(-5.0, 5.0);
    } else if !(*self_).enemy.is_null() {
        // ...then we'll calculate what new aim adjustments we should attempt to make this frame
        // Aim at enemy
        VectorCopy(&(*(*self_).enemy).r.currentOrigin, &mut org);
        org[2] += (*(*self_).enemy).r.maxs[2] * 0.5;
        if (*(*self_).enemy).s.eType == ET_NPC
            && (*(*self_).enemy).s.NPC_class == CLASS_VEHICLE
            && !(*(*self_).enemy).m_pVehicle.is_null()
            && (*(*(*(*self_).enemy).m_pVehicle).m_pVehicleInfo).r#type == VH_WALKER
        {
            //hack!
            org[2] += 32.0;
        }
        /*
        mdxaBone_t	boltMatrix;

        // Getting the "eye" here
        gi.G2API_GetBoltMatrix( self->ghoul2, self->playerModel,
                    self->torsoBolt,
                    &boltMatrix, self->r.currentAngles, self->s.origin, (cg.time?cg.time:level.time),
                    NULL, self->s.modelScale );

        gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, org2 );
        */
        VectorCopy(&(*top).r.currentOrigin, &mut org2);

        VectorSubtract(&org, &org2, &mut enemyDir);
        vectoangles(&enemyDir, &mut desiredAngles);
        desiredAngles[PITCH] = AngleNormalize180(desiredAngles[PITCH]);

        if desiredAngles[PITCH] < -pitchCap {
            desiredAngles[PITCH] = -pitchCap;
        } else if desiredAngles[PITCH] > pitchCap {
            desiredAngles[PITCH] = pitchCap;
        }

        diffYaw = AngleSubtract(desiredAngles[YAW], (*top).r.currentAngles[YAW]);
        diffPitch = AngleSubtract(desiredAngles[PITCH], (*top).r.currentAngles[PITCH]);
    } else {
        //FIXME: Pan back and forth in original facing
        // no enemy, so make us slowly sweep back and forth as if searching for a new one
        // `sin` is the C-library `double` sin; the argument is float-evaluated then
        // promoted to double, and the result truncated back to float on store.
        desiredAngles[YAW] =
            (((*addr_of!(level)).time as f32 * 0.0001 + (*top).count as f32) as f64).sin() as f32;
        desiredAngles[YAW] *= 60.0;
        desiredAngles[YAW] += (*self_).s.angles[YAW];
        desiredAngles[YAW] = AngleNormalize180(desiredAngles[YAW]);
        diffYaw = AngleSubtract(desiredAngles[YAW], (*top).r.currentAngles[YAW]);
        diffPitch = AngleSubtract(0.0, (*top).r.currentAngles[PITCH]);
        turnSpeed = 1.0;
    }

    if diffYaw != 0.0 {
        // cap max speed....
        if diffYaw.abs() > turnSpeed {
            diffYaw = if diffYaw >= 0.0 {
                turnSpeed
            } else {
                -turnSpeed
            };
        }
    }
    if diffPitch != 0.0 {
        if diffPitch.abs() > turnSpeed {
            // cap max speed
            diffPitch = if diffPitch > 0.0 {
                turnSpeed
            } else {
                -turnSpeed
            };
        }
    }
    // ...then set up our desired yaw
    VectorSet(&mut setAngle, diffPitch, diffYaw, 0.0);

    VectorCopy(&(*top).r.currentAngles, &mut (*top).s.apos.trBase);
    VectorScale(
        &setAngle,
        (1000 / FRAMETIME) as f32,
        &mut (*top).s.apos.trDelta,
    );
    (*top).s.apos.trTime = (*addr_of!(level)).time;
    (*top).s.apos.trType = TR_LINEAR_STOP;
    (*top).s.apos.trDuration = FRAMETIME;

    if diffYaw != 0.0 || diffPitch != 0.0 {
        (*top).s.loopSound = G_SoundIndex("sound/vehicles/weapons/hoth_turret/turn.wav");
    } else {
        (*top).s.loopSound = 0;
    }
}

#[allow(dead_code)] // used by turret_head_think (g_turret.c:226), not yet ported
const START_DIS: f32 = 15.0;

//----------------------------------------------------------------
/// `static void turret_fire ( gentity_t *ent, vec3_t start, vec3_t dir )` (g_turret.c:133) —
/// spawns the turret's `turret_proj` missile. Bails if the muzzle point is already in solid,
/// plays the muzzle-flash effect, then builds an `ET_MISSILE` bolt with custom shot/impact
/// effect indices, the emplaced-gun weapon id, and a linear trajectory along `dir` at the
/// turret's configured `mass` (shot speed). No oracle (spawns/mutates `gentity_t`s, calls
/// trap_PointContents).
//----------------------------------------------------------------
#[allow(dead_code)] // only caller is turret_head_think (g_turret.c:226), not yet ported
unsafe fn turret_fire(ent: *mut gentity_t, start: &vec3_t, dir: &vec3_t) {
    let mut org: vec3_t = [0.0; 3];

    if trap::PointContents(start, (*ent).s.number) & MASK_SHOT != 0 {
        return;
    }

    VectorMA(start, -START_DIS, dir, &mut org); // dumb....
    G_PlayEffectID((*ent).genericValue13, &org, dir);

    let bolt = G_Spawn();

    //use a custom shot effect
    (*bolt).s.otherEntityNum2 = (*ent).genericValue14;
    //use a custom impact effect
    (*bolt).s.emplacedOwner = (*ent).genericValue15;

    (*bolt).classname = c"turret_proj".as_ptr() as *mut c_char;
    (*bolt).nextthink = (*addr_of!(level)).time + 10000;
    (*bolt).think = Some(G_FreeEntity);
    (*bolt).s.eType = ET_MISSILE;
    (*bolt).s.weapon = WP_EMPLACED_GUN;
    (*bolt).r.ownerNum = (*ent).s.number;
    (*bolt).damage = (*ent).damage;
    (*bolt).alliedTeam = (*ent).alliedTeam;
    (*bolt).teamnodmg = (*ent).teamnodmg;
    //bolt->dflags = DAMAGE_NO_KNOCKBACK;// | DAMAGE_HEAVY_WEAP_CLASS;		// Don't push them around, or else we are constantly re-aiming
    (*bolt).splashDamage = (*ent).damage;
    (*bolt).splashRadius = 100;
    (*bolt).methodOfDeath = MOD_TARGET_LASER;
    (*bolt).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    //bolt->trigger_formation = qfalse;		// don't draw tail on first frame

    VectorSet(&mut (*bolt).r.maxs, 1.5, 1.5, 1.5);
    let maxs = (*bolt).r.maxs;
    VectorScale(&maxs, -1.0, &mut (*bolt).r.mins);
    (*bolt).s.pos.trType = TR_LINEAR;
    (*bolt).s.pos.trTime = (*addr_of!(level)).time;
    VectorCopy(start, &mut (*bolt).s.pos.trBase);
    VectorScale(dir, (*ent).mass, &mut (*bolt).s.pos.trDelta);
    trap::SnapVector(&mut (*bolt).s.pos.trDelta); // save net bandwidth
    VectorCopy(start, &mut (*bolt).r.currentOrigin);

    (*bolt).parent = ent;
}

//-----------------------------------------------------
/// `void turret_head_think( gentity_t *self )` (g_turret.c:183) — the turret top's per-frame
/// fire logic. When stunned (`painDebounceTime`) it sparks and has a 75% chance to skip
/// firing; otherwise, if it has an enemy and its fire/setup timers have elapsed, it computes
/// the muzzle point/forward off the base (`top`) and calls [`turret_fire`]. The original
/// Ghoul2 flash-bolt matrix lookup is preserved as a comment (the muzzle is derived from the
/// base bounds instead). No oracle (mutates `gentity_t`s, reads `g_entities`/`level`).
//-----------------------------------------------------
#[allow(dead_code)] // installed/called via turret_base_think (g_turret.c:588), not yet ported
pub unsafe extern "C" fn turret_head_think(self_: *mut gentity_t) {
    let top = core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add((*self_).r.ownerNum as usize);
    if top.is_null() {
        return;
    }
    if (*self_).painDebounceTime > (*addr_of!(level)).time {
        let mut v_up: vec3_t = [0.0; 3];
        VectorSet(&mut v_up, 0.0, 0.0, 1.0);
        G_PlayEffect(EFFECT_SPARKS as c_int, &(*self_).r.currentOrigin, &v_up);
        if Q_irand(0, 3) != 0 {
            //25% chance of still firing
            return;
        }
    }
    // if it's time to fire and we have an enemy, then gun 'em down!  pushDebounce time controls next fire time
    if !(*self_).enemy.is_null()
        && (*self_).setTime < (*addr_of!(level)).time
        && (*self_).attackDebounceTime < (*addr_of!(level)).time
    {
        let mut fwd: vec3_t = [0.0; 3];
        let mut org: vec3_t = [0.0; 3];
        // set up our next fire time
        (*self_).setTime = (*addr_of!(level)).time + (*self_).wait as c_int;

        /*
        mdxaBone_t	boltMatrix;

        // Getting the flash bolt here
        gi.G2API_GetBoltMatrix( self->ghoul2, self->playerModel,
                    self->torsoBolt,
                    &boltMatrix, self->r.currentAngles, self->r.currentOrigin, (cg.time?cg.time:level.time),
                    NULL, self->s.modelScale );

        gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, org );
        gi.G2API_GiveMeVectorFromMatrix( boltMatrix, POSITIVE_Y, fwd );
        */
        VectorCopy(&(*top).r.currentOrigin, &mut org);
        org[2] += (*top).r.maxs[2] - 8.0;
        AngleVectors(&(*top).r.currentAngles, Some(&mut fwd), None, None);

        let org_copy = org;
        VectorMA(&org_copy, START_DIS, &fwd, &mut org);

        turret_fire(top, &org, &fwd);
        (*self_).fly_sound_debounce_time = (*addr_of!(level)).time; //used as lastShotTime
    }
}

//-----------------------------------------------------
/// `static void turret_turnoff( gentity_t *self )` (g_turret.c:359) — stops the turret top
/// (`top`) rotating (stationary trajectory), kills its loop sound, and clears the enemy.
/// No oracle (mutates `gentity_t`s, reads `g_entities`/`level`).
//-----------------------------------------------------
#[allow(dead_code)] // only caller is turret_base_think (g_turret.c:519), not yet ported
unsafe fn turret_turnoff(self_: *mut gentity_t) {
    let top = core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add((*self_).r.ownerNum as usize);
    if !top.is_null() {
        //still have a top
        //stop it from rotating
        VectorCopy(&(*top).r.currentAngles, &mut (*top).s.apos.trBase);
        VectorClear(&mut (*top).s.apos.trDelta);
        (*top).s.apos.trTime = (*addr_of!(level)).time;
        (*top).s.apos.trType = TR_STATIONARY;
    }

    (*self_).s.loopSound = 0;
    // shut-down sound
    //G_Sound( self, CHAN_BODY, G_SoundIndex( "sound/chars/turret/shutdown.wav" ));

    // Clear enemy
    (*self_).enemy = core::ptr::null_mut();
}

//-----------------------------------------------------
/// `static void turret_sleep( gentity_t *self )` (g_turret.c:381) — puts the turret to sleep:
/// if it had an enemy, arms a 5-second ping window (`aimDebounceTime`) and clears the enemy.
/// No oracle (mutates a `gentity_t`, reads `level`).
//-----------------------------------------------------
#[allow(dead_code)] // only caller is turret_base_think (g_turret.c:595), not yet ported
unsafe fn turret_sleep(self_: *mut gentity_t) {
    if (*self_).enemy.is_null() {
        // we don't need to play sound
        return;
    }

    // make turret play ping sound for 5 seconds
    (*self_).aimDebounceTime = (*addr_of!(level)).time + 5000;

    // Clear enemy
    (*self_).enemy = core::ptr::null_mut();
}

//-----------------------------------------------------
/// `static qboolean turret_find_enemies( gentity_t *self )` (g_turret.c:398) — scans the
/// turret's radius for a valid client target. Skips non-clients, dead/notarget/spectator
/// ents, and same-allied-team ents; requires PVS + a clear trace; prefers the nearest (with
/// an AT-ST override). On acquisition it sets the enemy and fires `target2`. Returns whether
/// an enemy was found. No oracle (trap PVS/trace, mutates `gentity_t`s).
//-----------------------------------------------------
#[allow(dead_code)] // only caller is turret_base_think (g_turret.c:536), not yet ported
unsafe fn turret_find_enemies(self_: *mut gentity_t) -> qboolean {
    let mut found: qboolean = QFALSE;
    let count: c_int;
    let mut bestDist: f32 = (*self_).radius * (*self_).radius;
    let mut enemyDist: f32;
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];
    let mut entity_list: [*mut gentity_t; MAX_GENTITIES] = [core::ptr::null_mut(); MAX_GENTITIES];
    let mut target: *mut gentity_t;
    let mut bestTarget: *mut gentity_t = core::ptr::null_mut();
    let top = core::ptr::addr_of_mut!(g_entities)
        .cast::<gentity_t>()
        .add((*self_).r.ownerNum as usize);
    if top.is_null() {
        return QFALSE;
    }

    if (*self_).aimDebounceTime > (*addr_of!(level)).time {
        // time since we've been shut off
        // We were active and alert, i.e. had an enemy in the last 3 secs
        if (*self_).timestamp < (*addr_of!(level)).time {
            //G_Sound(self, CHAN_BODY, G_SoundIndex( "sound/chars/turret/ping.wav" ));
            (*self_).timestamp = (*addr_of!(level)).time + 1000;
        }
    }

    VectorCopy(&(*top).r.currentOrigin, &mut org2);

    count = G_RadiusList(&org2, (*self_).radius, self_, QTRUE, &mut entity_list);

    let mut i: c_int = 0;
    while i < count {
        target = entity_list[i as usize];

        if (*target).client.is_null() {
            // only attack clients
            i += 1;
            continue;
        }
        if target == self_
            || (*target).takedamage == QFALSE
            || (*target).health <= 0
            || ((*target).flags & FL_NOTARGET) != 0
        {
            i += 1;
            continue;
        }
        if (*(*target).client).sess.sessionTeam == TEAM_SPECTATOR {
            i += 1;
            continue;
        }
        if (*self_).alliedTeam != 0 {
            if !(*target).client.is_null() {
                if (*(*target).client).sess.sessionTeam == (*self_).alliedTeam {
                    // A bot/client/NPC we don't want to shoot
                    i += 1;
                    continue;
                }
            } else if (*target).teamnodmg == (*self_).alliedTeam {
                // An ent we don't want to shoot
                i += 1;
                continue;
            }
        }
        if trap::InPVS(&org2, &(*target).r.currentOrigin) == QFALSE {
            i += 1;
            continue;
        }

        VectorCopy(&(*target).r.currentOrigin, &mut org);
        org[2] += (*target).r.maxs[2] * 0.5;

        let tr = trap::Trace(
            &org2,
            &vec3_origin,
            &vec3_origin,
            &org,
            (*self_).s.number,
            MASK_SHOT,
        );

        if tr.allsolid == 0
            && tr.startsolid == 0
            && (tr.fraction == 1.0 || tr.entityNum as c_int == (*target).s.number)
        {
            // Only acquire if have a clear shot, Is it in range and closer than our best?
            VectorSubtract(
                &(*target).r.currentOrigin,
                &(*top).r.currentOrigin,
                &mut enemyDir,
            );
            enemyDist = VectorLengthSquared(&enemyDir);

            if enemyDist < bestDist // all things equal, keep current
                || (Q_stricmp(c"atst_vehicle".as_ptr(), (*target).NPC_type) == 0
                    && !bestTarget.is_null()
                    && Q_stricmp(c"atst_vehicle".as_ptr(), (*bestTarget).NPC_type) != 0)
            //target AT-STs over non-AT-STs... FIXME: must be a better, easier way to tell this, no?
            {
                if (*self_).attackDebounceTime < (*addr_of!(level)).time {
                    // We haven't fired or acquired an enemy in the last 2 seconds-start-up sound
                    //G_Sound( self, CHAN_BODY, G_SoundIndex( "sound/chars/turret/startup.wav" ));

                    // Wind up turrets for a bit
                    (*self_).attackDebounceTime = (*addr_of!(level)).time + 1400;
                }

                bestTarget = target;
                bestDist = enemyDist;
                found = QTRUE;
            }
        }
        i += 1;
    }

    if found != QFALSE {
        G_SetEnemy(self_, bestTarget);
        if !(*self_).target2.is_null() && *(*self_).target2 != 0 {
            G_UseTargets2(self_, self_, (*self_).target2);
        }
    }

    found
}

//-----------------------------------------------------
/// `qboolean turret_base_spawn_top( gentity_t *base )` (g_turret.c:698) — spawns and wires up
/// the turret's rotating top half, linking it bi-directionally to `base`, copying over all
/// the team/health/timing/damage/effect config (with defaults), installing the
/// pain/die/use/think callbacks, and registering the emplaced-gun item for the missile
/// effect. Returns `qfalse` if the top couldn't be spawned. No oracle (spawns/mutates
/// `gentity_t`s, model/sound/effect indices, trap_LinkEntity).
//-----------------------------------------------------
pub unsafe extern "C" fn turret_base_spawn_top(base: *mut gentity_t) -> qboolean {
    let mut org: vec3_t = [0.0; 3];
    let mut t: c_int = 0;

    let top = G_Spawn();
    if top.is_null() {
        return QFALSE;
    }

    (*top).s.modelindex = G_ModelIndex("models/map_objects/hoth/turret_top_new.md3");
    (*top).s.modelindex2 = G_ModelIndex("models/map_objects/hoth/turret_top.md3");
    G_SetAngles(top, &(*base).s.angles);
    VectorCopy(&(*base).s.origin, &mut org);
    org[2] += 128.0;
    G_SetOrigin(top, &org);

    (*base).r.ownerNum = (*top).s.number;
    (*top).r.ownerNum = (*base).s.number;

    if !(*base).team.is_null() && *(*base).team != 0 && //g_gametype.integer == GT_SIEGE &&
        (*base).teamnodmg == 0
    {
        (*base).teamnodmg = atoi((*base).team);
    }
    (*base).team = core::ptr::null_mut();
    (*top).teamnodmg = (*base).teamnodmg;
    (*top).alliedTeam = (*base).alliedTeam;

    (*base).s.eType = ET_GENERAL;

    // Set up our explosion effect for the ExplodeDeath code....
    G_EffectIndex("turret/explode");
    G_EffectIndex("sparks/spark_exp_nosnd");
    G_EffectIndex("turret/hoth_muzzle_flash");

    // this is really the pitch angle.....
    (*top).speed = 0.0;

    // this is a random time offset for the no-enemy-search-around-mode
    (*top).count = (random() * 9000.0) as c_int;

    if (*base).health == 0 {
        (*base).health = 3000;
    }
    (*top).health = (*base).health;

    G_SpawnInt(c"showhealth".as_ptr(), c"0".as_ptr(), &mut t);

    if t != 0 {
        //a non-0 maxhealth value will mean we want to show the health on the hud
        (*top).maxHealth = (*base).health; //acts as "maxhealth"
        G_ScaleNetHealth(top);

        (*base).maxHealth = (*base).health;
        G_ScaleNetHealth(base);
    }

    (*base).takedamage = QTRUE;
    (*base).pain = Some(TurretBasePain);
    (*base).die = Some(bottom_die);

    //design specified shot speed
    G_SpawnFloat(c"shotspeed".as_ptr(), c"1100".as_ptr(), &mut (*base).mass);
    (*top).mass = (*base).mass;

    //even if we don't want to show health, let's at least light the crosshair up properly over ourself
    if (*top).s.teamowner == 0 {
        (*top).s.teamowner = (*top).alliedTeam;
    }

    (*base).alliedTeam = (*top).alliedTeam;
    (*base).s.teamowner = (*top).s.teamowner;

    (*base).s.shouldtarget = QTRUE;
    (*top).s.shouldtarget = QTRUE;

    //link them to each other
    (*base).target_ent = top;
    (*top).target_ent = base;

    //top->s.owner = MAX_CLIENTS; //not owned by any client

    // search radius
    if (*base).radius == 0.0 {
        (*base).radius = 1024.0;
    }
    (*top).radius = (*base).radius;

    // How quickly to fire
    if (*base).wait == 0.0 {
        (*base).wait = 300.0 + random() * 55.0;
    }
    (*top).wait = (*base).wait;

    if (*base).splashDamage == 0 {
        (*base).splashDamage = 300;
    }
    (*top).splashDamage = (*base).splashDamage;

    if (*base).splashRadius == 0 {
        (*base).splashRadius = 128;
    }
    (*top).splashRadius = (*base).splashRadius;

    // how much damage each shot does
    if (*base).damage == 0 {
        (*base).damage = 100;
    }
    (*top).damage = (*base).damage;

    // how fast it turns
    if (*base).speed == 0.0 {
        (*base).speed = 20.0;
    }
    (*top).speed = (*base).speed;

    VectorSet(&mut (*top).r.maxs, 48.0, 48.0, 16.0);
    VectorSet(&mut (*top).r.mins, -48.0, -48.0, 0.0);
    // Precache moving sounds
    //G_SoundIndex( "sound/chars/turret/startup.wav" );
    //G_SoundIndex( "sound/chars/turret/shutdown.wav" );
    //G_SoundIndex( "sound/chars/turret/ping.wav" );
    G_SoundIndex("sound/vehicles/weapons/hoth_turret/turn.wav");
    (*top).genericValue13 = G_EffectIndex("turret/hoth_muzzle_flash");
    (*top).genericValue14 = G_EffectIndex("turret/hoth_shot");
    (*top).genericValue15 = G_EffectIndex("turret/hoth_impact");

    (*top).r.contents = CONTENTS_BODY;

    //base->max_health = base->health;
    (*top).takedamage = QTRUE;
    (*top).pain = Some(TurretPain);
    (*top).die = Some(auto_turret_die);

    (*top).material = MAT_METAL;
    //base->r.svFlags |= SVF_NO_TELEPORT|SVF_NONNPC_ENEMY|SVF_SELF_ANIMATING;

    // Register this so that we can use it for the missile effect
    RegisterItem(BG_FindItemForWeapon(WP_EMPLACED_GUN));

    // But set us as a turret so that we can be identified as a turret
    (*top).s.weapon = WP_EMPLACED_GUN;

    trap::LinkEntity(top);
    QTRUE
}

//-----------------------------------------------------
/// `void turret_base_think( gentity_t *self )` (g_turret.c:509) — the turret base's per-frame
/// `think` callback. When START_OFF it turns the turret off and stops thinking; otherwise it
/// re-arms its think, then either acquires an enemy ([`turret_find_enemies`]), drops a
/// spectator enemy, or (for a live enemy in radius + PVS + a clear trace) keeps the lock and
/// runs the head fire logic ([`turret_head_think`]). It then sleeps ([`turret_sleep`]) after a
/// debounce when no valid target remains (else extends the 2-second keep-enemy window) and
/// finally drives the aim trajectory ([`turret_aim`]). No oracle (entity-state: mutates
/// `gentity_t`s, trap_InPVS/trap_Trace, reads `level`).
//-----------------------------------------------------
pub unsafe extern "C" fn turret_base_think(self_: *mut gentity_t) {
    let mut turnOff: qboolean = QTRUE;
    let enemyDist: f32;
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];

    if (*self_).spawnflags & 1 != 0 {
        // not turned on
        turret_turnoff(self_);

        // No target
        (*self_).flags |= FL_NOTARGET;
        (*self_).nextthink = -1; //never think again
        return;
    } else {
        // I'm all hot and bothered
        (*self_).flags &= !FL_NOTARGET;
        //remember to keep thinking!
        (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;
    }

    if (*self_).enemy.is_null() {
        if turret_find_enemies(self_) != QFALSE {
            turnOff = QFALSE;
        }
    } else if !(*(*self_).enemy).client.is_null()
        && (*(*(*self_).enemy).client).sess.sessionTeam == TEAM_SPECTATOR
    {
        //don't keep going after spectators
        (*self_).enemy = core::ptr::null_mut();
    } else {
        //FIXME: remain single-minded or look for a new enemy every now and then?
        if (*(*self_).enemy).health > 0 {
            // enemy is alive
            VectorSubtract(
                &(*(*self_).enemy).r.currentOrigin,
                &(*self_).r.currentOrigin,
                &mut enemyDir,
            );
            enemyDist = VectorLengthSquared(&enemyDir);

            if enemyDist < ((*self_).radius * (*self_).radius) {
                // was in valid radius
                if trap::InPVS(
                    &(*self_).r.currentOrigin,
                    &(*(*self_).enemy).r.currentOrigin,
                ) != QFALSE
                {
                    // Every now and again, check to see if we can even trace to the enemy

                    if !(*(*self_).enemy).client.is_null() {
                        VectorCopy(&(*(*(*self_).enemy).client).renderInfo.eyePoint, &mut org);
                    } else {
                        VectorCopy(&(*(*self_).enemy).r.currentOrigin, &mut org);
                    }
                    VectorCopy(&(*self_).r.currentOrigin, &mut org2);
                    if (*self_).spawnflags & 2 != 0 {
                        org2[2] += 10.0;
                    } else {
                        org2[2] -= 10.0;
                    }
                    let tr = trap::Trace(
                        &org2,
                        &vec3_origin,
                        &vec3_origin,
                        &org,
                        (*self_).s.number,
                        MASK_SHOT,
                    );

                    if tr.allsolid == 0
                        && tr.startsolid == 0
                        && tr.entityNum as c_int == (*(*self_).enemy).s.number
                    {
                        turnOff = QFALSE; // Can see our enemy
                    }
                }
            }
        }

        turret_head_think(self_);
    }

    if turnOff != QFALSE {
        if (*self_).bounceCount < (*addr_of!(level)).time
        // bounceCount is used to keep the thing from ping-ponging from on to off
        {
            turret_sleep(self_);
        }
    } else {
        // keep our enemy for a minimum of 2 seconds from now
        (*self_).bounceCount = (*addr_of!(level)).time + 2000 + (random() * 150.0) as c_int;
    }

    turret_aim(self_);
}

/*QUAKED misc_turret (1 0 0) (-48 -48 0) (48 48 144) START_OFF
Large 2-piece turbolaser turret

  START_OFF - Starts off

  radius - How far away an enemy can be for it to pick it up (default 1024)
  wait	- Time between shots (default 300 ms)
  dmg	- How much damage each shot does (default 100)
  health - How much damage it can take before exploding (default 3000)
  speed - how fast it turns (default 10)

  splashDamage - How much damage the explosion does (300)
  splashRadius - The radius of the explosion (128)

  shotspeed - speed at which projectiles will move

  targetname - Toggles it on/off
  target - What to use when destroyed
  target2 - What to use when it decides to start shooting at an enemy

  showhealth - set to 1 to show health bar on this entity when crosshair is over it

  teamowner - crosshair shows green for this team, red for opposite team
    0 - none
    1 - red
    2 - blue

  alliedTeam - team that this turret won't target
    0 - none
    1 - red
    2 - blue

  teamnodmg - team that turret does not take damage from
    0 - none
    1 - red
    2 - blue

"icon" - icon that represents the objective on the radar
*/
//-----------------------------------------------------
/// `void SP_misc_turret( gentity_t *base )` (g_turret.c:665) — spawn function for the
/// `misc_turret` 2-piece turbolaser. Sets the base models/bounds/contents, indexes the
/// optional radar `"icon"` (into `s.genericenemyindex`), installs the on/off `use` and
/// per-frame `think` callbacks, links it, then spawns the rotating top via
/// [`turret_base_spawn_top`] (freeing the base on failure). No oracle (entity-state spawn:
/// model indices, `G_SetAngles`/`G_SetOrigin`/`trap_LinkEntity`/`G_FreeEntity`).
//-----------------------------------------------------
pub unsafe extern "C" fn SP_misc_turret(base: *mut gentity_t) {
    let mut s: *mut c_char = core::ptr::null_mut();

    (*base).s.modelindex2 = G_ModelIndex("models/map_objects/hoth/turret_bottom.md3");
    (*base).s.modelindex = G_ModelIndex("models/map_objects/hoth/turret_base.md3");
    //base->playerModel = gi.G2API_InitGhoul2Model( base->ghoul2, "models/map_objects/imp_mine/turret_canon.glm", base->s.modelindex );
    //base->s.radius = 80.0f;

    //gi.G2API_SetBoneAngles( &base->ghoul2[base->playerModel], "Bone_body", vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_Y, POSITIVE_Z, POSITIVE_X, NULL );
    //base->torsoBolt = gi.G2API_AddBolt( &base->ghoul2[base->playerModel], "*flash03" );

    G_SpawnString(c"icon".as_ptr(), c"".as_ptr(), &mut s);
    if !s.is_null() && *s != 0 {
        // We have an icon, so index it now.  We are reusing the genericenemyindex
        // variable rather than adding a new one to the entity state.
        (*base).s.genericenemyindex = G_IconIndex(&CStr::from_ptr(s).to_string_lossy());
    }

    G_SetAngles(base, &(*base).s.angles);
    G_SetOrigin(base, &(*base).s.origin);

    (*base).r.contents = CONTENTS_BODY;

    VectorSet(&mut (*base).r.maxs, 32.0, 32.0, 128.0);
    VectorSet(&mut (*base).r.mins, -32.0, -32.0, 0.0);

    (*base).r#use = Some(turret_base_use);
    (*base).think = Some(turret_base_think);
    // don't start working right away
    (*base).nextthink = (*addr_of!(level)).time + FRAMETIME * 5;

    trap::LinkEntity(base);

    if turret_base_spawn_top(base) == QFALSE {
        G_FreeEntity(base);
    }
}
