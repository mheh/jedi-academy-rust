//! Port of `g_turret_G2.c` — the Ghoul2-model `misc_turretG2` (a ceiling/floor-mounted
//! auto-targeting turret built from a single Ghoul2 entity, plus a boxy "TURBO" Death-Star
//! turbolaser variant). COMPLETE: the bone-angle tracking helper (`G2Tur_SetBoneAngles`),
//! model setup (`turretG2_set_models`), the pain/die callbacks
//! (`TurretG2Pain`/`turretG2_die`), the turbolaser bone-anim helper
//! (`TurboLaser_SetBoneAnim`), the fire/aim/think machinery
//! (`turretG2_fire`/`turretG2_aim`/`turretG2_head_think`/`turretG2_find_enemies`/`turretG2_base_think`),
//! the on/off use + respawn + turnoff callbacks
//! (`turretG2_base_use`/`turretG2_respawn`/`turretG2_turnoff`), and the spawn plumbing
//! (`SP_misc_turretG2`/`finish_spawning_turretG2`).
//!
//! All callbacks here are No-oracle (engine-syscall / Ghoul2 / global level/entity plumbing).
//!
//! NOTE: `SP_misc_turretG2` must be wired into the `g_spawn.rs` classname→spawn table
//! (`"misc_turretG2"`) centrally — that table is owned elsewhere and is not edited here.

#![allow(non_snake_case)] // C function names (`TurretG2Pain`, …) kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::addr_of;

use crate::codemp::game::bg_misc::{
    BG_EvaluateTrajectory, BG_FindItemForWeapon, BG_GiveMeVectorFromMatrix,
};
use crate::codemp::game::bg_public::{
    EFFECT_EXPLOSION_TURRET, EF2_BRACKET_ENTITY, EF_G2ANIMATING, EF_RADAROBJECT, EF_SHADER_ANIM,
    ET_GENERAL, ET_MISSILE, GT_SIEGE, MASK_SHOT, MOD_TARGET_LASER, MOD_UNKNOWN, TEAM_SPECTATOR,
};
use crate::codemp::game::bg_weapons_h::{WP_BLASTER, WP_DEMP2, WP_TURRET};
use crate::codemp::game::g_combat::{AddScore, ObjectDie, G_RadiusDamage};
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_team::OnSameTeam;
use crate::codemp::game::g_local::{
    gentity_t, DAMAGE_HEAVY_WEAP_CLASS, DAMAGE_NO_KNOCKBACK, FL_BBRUSH, FL_NOTARGET, FRAMETIME,
};
use crate::codemp::game::g_main::{g_gametype, level, Com_Printf};
use crate::codemp::game::g_spawn::{G_SpawnFloat, G_SpawnInt, G_SpawnString};
use crate::codemp::game::g_utils::{
    G_BoneIndex, G_EffectIndex, G_FreeEntity, G_IconIndex, G_KillG2Queue, G_ModelIndex, G_PlayEffect,
    G_PlayEffectID, G_RadiusList, G_ScaleNetHealth, G_SetAngles, G_SetOrigin, G_Sound,
    G_SoundIndex, G_Spawn, G_UseTargets, G_UseTargets2,
};
use crate::codemp::game::g_weapon::WP_FireTurboLaserMissile;
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::q_math::{
    flrand, vec3_origin, AngleNormalize360, AngleSubtract, AngleVectors, VectorClear, VectorCopy,
    VectorLengthSquared, VectorMA, VectorNormalize, VectorScale, VectorSet, VectorSubtract,
    vectoangles,
};
use crate::codemp::game::q_shared::{random, Q_stricmp};
use crate::codemp::game::q_shared_h::{
    vec3_t, mdxaBone_t, CHAN_BODY, MAT_METAL, MAX_CLIENTS, MAX_GENTITIES, NEGATIVE_X, NEGATIVE_Z,
    ORIGIN, PITCH,
    POSITIVE_X, POSITIVE_Y, ROLL, TR_LINEAR, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_LIGHTSABER, CONTENTS_MONSTERCLIP, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP,
};
use crate::codemp::ghoul2::g2_h::{BONE_ANGLES_POSTMULT, BONE_ANIM_BLEND, BONE_ANIM_OVERRIDE_FREEZE};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// `atoi` is the C-library implementation (the toolchain links it from <stdlib.h>),
// matching the original's locale-default parse — see g_turret.rs / g_spawn.rs.
extern "C" {
    fn atoi(s: *const c_char) -> c_int;
}

// void G_SetEnemy( gentity_t *self, gentity_t *enemy );        — npc_combat::G_SetEnemy
// void finish_spawning_turretG2( gentity_t *base );            — below
// void ObjectDie (...);                                        — g_combat::ObjectDie
// void turretG2_base_use( ... );                               — below
// extern void WP_FireTurboLaserMissile( ... );                 — g_weapon::WP_FireTurboLaserMissile

const SPF_TURRETG2_CANRESPAWN: c_int = 4;
const SPF_TURRETG2_TURBO: c_int = 8;
const SPF_TURRETG2_LEAD_ENEMY: c_int = 16;
const SPF_SHOWONRADAR: c_int = 32;

// #define ARM_ANGLE_RANGE   60   — unused
// #define HEAD_ANGLE_RANGE  90   — unused

const name: &str = "models/map_objects/imp_mine/turret_canon.glm";
const name2: &str = "models/map_objects/imp_mine/turret_damage.md3";
const name3: &str = "models/map_objects/wedge/laser_cannon_model.glm";

//special routine for tracking angles between client and server -rww
/// `void G2Tur_SetBoneAngles(gentity_t *ent, char *bone, vec3_t angles)`
/// (g_turret_G2.c:24). Finds (or allocates) a free bone-index slot in the entity-state
/// (`boneIndex1`..`boneIndex4`), copies `angles` into the matching `boneAngles*` slot so the
/// client can replay them, then sets the angle override on the server Ghoul2 instance if one
/// exists. No oracle (Ghoul2 syscall + entity-state plumbing).
pub unsafe fn G2Tur_SetBoneAngles(ent: *mut gentity_t, bone: &str, angles: &vec3_t) {
    let mut thebone: *mut c_int = &mut (*ent).s.boneIndex1;
    let mut firstFree: *mut c_int = core::ptr::null_mut();
    let mut i: c_int = 0;
    let boneIndex: c_int = G_BoneIndex(bone);
    let flags: c_int;
    let up: c_int;
    let right: c_int;
    let forward: c_int;
    let mut boneVector: *mut vec3_t = &mut (*ent).s.boneAngles1;
    let mut freeBoneVec: *mut vec3_t = core::ptr::null_mut();

    while !thebone.is_null() {
        if *thebone == 0 && firstFree.is_null() {
            //if the value is 0 then this index is clear, we can use it if we don't find the bone we want already existing.
            firstFree = thebone;
            freeBoneVec = boneVector;
        } else if *thebone != 0 {
            if *thebone == boneIndex {
                //this is it
                break;
            }
        }

        match i {
            0 => {
                thebone = &mut (*ent).s.boneIndex2;
                boneVector = &mut (*ent).s.boneAngles2;
            }
            1 => {
                thebone = &mut (*ent).s.boneIndex3;
                boneVector = &mut (*ent).s.boneAngles3;
            }
            2 => {
                thebone = &mut (*ent).s.boneIndex4;
                boneVector = &mut (*ent).s.boneAngles4;
            }
            _ => {
                thebone = core::ptr::null_mut();
                boneVector = core::ptr::null_mut();
            }
        }

        i += 1;
    }

    if thebone.is_null() {
        //didn't find it, create it
        if firstFree.is_null() {
            //no free bones.. can't do a thing then.
            Com_Printf("WARNING: NPC has no free bone indexes\n");
            return;
        }

        thebone = firstFree;

        *thebone = boneIndex;
        boneVector = freeBoneVec;
    }

    //If we got here then we have a vector and an index.

    //Copy the angles over the vector in the entitystate, so we can use the corresponding index
    //to set the bone angles on the client.
    VectorCopy(angles, &mut *boneVector);

    //Now set the angles on our server instance if we have one.

    if (*ent).ghoul2.is_null() {
        return;
    }

    flags = BONE_ANGLES_POSTMULT;
    up = POSITIVE_Y;
    right = NEGATIVE_Z;
    forward = NEGATIVE_X;

    //first 3 bits is forward, second 3 bits is right, third 3 bits is up
    (*ent).s.boneOrient = (forward) | (right << 3) | (up << 6);

    trap::G2API_SetBoneAngles(
        (*ent).ghoul2,
        0,
        bone,
        angles,
        flags,
        up,
        right,
        forward,
        core::ptr::null_mut(),
        100,
        (*addr_of!(level)).time,
    );
}

/// `void turretG2_set_models( gentity_t *self, qboolean dying )` (g_turret_G2.c:129).
/// Swaps the turret's render models between the live and damaged states. When `dying`, it
/// drops the Ghoul2 model; otherwise it (re)inits the Ghoul2 model (canon or turbolaser),
/// sets the g2 radius, and registers the pitch bone + muzzle-flash bolts. No oracle (Ghoul2
/// syscalls + entity-state plumbing).
pub unsafe fn turretG2_set_models(self_: *mut gentity_t, dying: qboolean) {
    if dying != QFALSE {
        if (*self_).spawnflags & SPF_TURRETG2_TURBO == 0 {
            (*self_).s.modelindex = G_ModelIndex(name2);
            (*self_).s.modelindex2 = G_ModelIndex(name);
        }

        trap::G2API_RemoveGhoul2Model(&mut (*self_).ghoul2 as *mut _ as *mut core::ffi::c_void, 0);
        G_KillG2Queue((*self_).s.number);
        (*self_).s.modelGhoul2 = 0;
        /*
        trap_G2API_InitGhoul2Model( &self->ghoul2,
                                    name2,
                                    0, //base->s.modelindex,
                                    0,
                                    0,
                                    0,
                                    0);
        */
    } else {
        if (*self_).spawnflags & SPF_TURRETG2_TURBO == 0 {
            (*self_).s.modelindex = G_ModelIndex(name);
            (*self_).s.modelindex2 = G_ModelIndex(name2);
            //set the new onw
            trap::G2API_InitGhoul2Model(
                &mut (*self_).ghoul2,
                c"models/map_objects/imp_mine/turret_canon.glm".as_ptr(),
                0, //base->s.modelindex,
                //note, this is not the same kind of index - this one's referring to the actual
                //index of the model in the g2 instance, whereas modelindex is the index of a
                //configstring -rww
                0,
                0,
                0,
                0,
            );
        } else {
            (*self_).s.modelindex = G_ModelIndex(name3);
            //set the new onw
            trap::G2API_InitGhoul2Model(
                &mut (*self_).ghoul2,
                c"models/map_objects/wedge/laser_cannon_model.glm".as_ptr(),
                0, //base->s.modelindex,
                //note, this is not the same kind of index - this one's referring to the actual
                //index of the model in the g2 instance, whereas modelindex is the index of a
                //configstring -rww
                0,
                0,
                0,
                0,
            );
        }

        (*self_).s.modelGhoul2 = 1;
        if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
            //larger
            (*self_).s.g2radius = 128;
        } else {
            (*self_).s.g2radius = 80;
        }

        if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
            //different pitch bone and muzzle flash points
            G2Tur_SetBoneAngles(self_, "pitch", &vec3_origin);
            (*self_).genericValue11 = trap::G2API_AddBolt((*self_).ghoul2, 0, "*muzzle1");
            (*self_).genericValue12 = trap::G2API_AddBolt((*self_).ghoul2, 0, "*muzzle2");
        } else {
            G2Tur_SetBoneAngles(self_, "Bone_body", &vec3_origin);
            (*self_).genericValue11 = trap::G2API_AddBolt((*self_).ghoul2, 0, "*flash03");
        }
    }
}

//------------------------------------------------------------------------------------------------------------
/// `void TurretG2Pain( gentity_t *self, gentity_t *attacker, int damage )` (g_turret_G2.c:214).
/// The turret's `pain` callback: fires its `paintarget` (rate-limited by `genericValue4`),
/// stuns it for a random window when hit by the DEMP2, and acquires the attacker as an enemy
/// if it has none. `damage` is unused, matching the C signature. No oracle.
//------------------------------------------------------------------------------------------------------------
pub unsafe extern "C" fn TurretG2Pain(
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
) {
    if !(*self_).paintarget.is_null() && *(*self_).paintarget != 0 {
        if (*self_).genericValue8 < (*addr_of!(level)).time {
            G_UseTargets2(self_, self_, (*self_).paintarget);
            (*self_).genericValue8 = (*addr_of!(level)).time + (*self_).genericValue4;
        }
    }

    if !(*attacker).client.is_null() && (*(*attacker).client).ps.weapon == WP_DEMP2 {
        (*self_).attackDebounceTime = (*addr_of!(level)).time + 2000 + (random() * 500.0) as c_int;
        (*self_).painDebounceTime = (*self_).attackDebounceTime;
    }
    if (*self_).enemy.is_null() {
        //react to being hit
        G_SetEnemy(self_, attacker);
    }
    //self->s.health = self->health;
    //mmm..yes..bad.
}

//------------------------------------------------------------------------------------------------------------
/// `void turretG2_die ( gentity_t *self, gentity_t *inflictor, gentity_t *attacker, int damage, int meansOfDeath )`
/// (g_turret_G2.c:240). The turret's `die` callback: clears its combat data, plays the
/// explosion effect, dishes out splash damage, and then either switches to the damaged model
/// (firing its `target` and arming the respawn timer) or falls through to [`ObjectDie`].
/// No oracle.
//------------------------------------------------------------------------------------------------------------
pub unsafe extern "C" fn turretG2_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    means_of_death: c_int,
) {
    let mut forward: vec3_t = [0.0, 0.0, -1.0];
    let mut pos: vec3_t = [0.0; 3];

    // Turn off the thinking of the base & use it's targets
    //self->think = NULL;
    (*self_).r#use = None;

    // clear my data
    (*self_).die = None;
    (*self_).pain = None;
    (*self_).takedamage = QFALSE;
    (*self_).s.health = 0;
    (*self_).health = 0;
    (*self_).s.loopSound = 0;
    (*self_).s.shouldtarget = QFALSE;
    //self->s.owner = MAX_CLIENTS; //not owned by any client

    if !attacker.is_null()
        && (*attacker).s.number < MAX_CLIENTS as c_int
        && OnSameTeam(attacker, self_) == QFALSE
    {
        //give them a point for the kill
        AddScore(attacker, &(*self_).r.currentOrigin, 1);
        //should we send an obit?  nah...
    }

    // hack the effect angle so that explode death can orient the effect properly
    if (*self_).spawnflags & 2 != 0 {
        VectorSet(&mut forward, 0.0, 0.0, 1.0);
    }

    //	VectorCopy( self->r.currentOrigin, self->s.pos.trBase );

    VectorMA(&(*self_).r.currentOrigin, 12.0, &forward, &mut pos);
    G_PlayEffect(EFFECT_EXPLOSION_TURRET as c_int, &pos, &forward);

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

    if (*self_).s.eFlags & EF_SHADER_ANIM != 0 {
        (*self_).s.frame = 1; // black
    }

    (*self_).s.weapon = 0; // crosshair code uses this to mark crosshair red

    if (*self_).s.modelindex2 != 0 {
        // switch to damage model if we should
        turretG2_set_models(self_, QTRUE);

        VectorCopy(&(*self_).r.currentAngles, &mut (*self_).s.apos.trBase);
        VectorClear(&mut (*self_).s.apos.trDelta);

        if !(*self_).target.is_null() {
            G_UseTargets(self_, attacker);
        }

        if (*self_).spawnflags & SPF_TURRETG2_CANRESPAWN != 0 {
            //respawn
            if (*self_).health < 1 && (*self_).genericValue5 == 0 {
                //we are dead, set our respawn delay if we have one
                (*self_).genericValue5 = (*addr_of!(level)).time + (*self_).count;
            }
        }
    } else {
        ObjectDie(self_, inflictor, attacker, damage, means_of_death);
    }
}

const START_DIS: f32 = 15.0;

//start an animation on model_root both server side and client side
/// `void TurboLaser_SetBoneAnim(gentity_t *eweb, int startFrame, int endFrame)`
/// (g_turret_G2.c:317). Flags the entity for client-side g2 animation, sets/restarts the
/// torso/legs anim frames, and plays the bone anim on the server Ghoul2 instance. No oracle.
pub unsafe fn TurboLaser_SetBoneAnim(eweb: *mut gentity_t, startFrame: c_int, endFrame: c_int) {
    //set info on the entity so it knows to start the anim on the client next snapshot.
    (*eweb).s.eFlags |= EF_G2ANIMATING;

    if (*eweb).s.torsoAnim == startFrame && (*eweb).s.legsAnim == endFrame {
        //already playing this anim, let's flag it to restart
        (*eweb).s.torsoFlip = if (*eweb).s.torsoFlip == QFALSE {
            QTRUE
        } else {
            QFALSE
        };
    } else {
        (*eweb).s.torsoAnim = startFrame;
        (*eweb).s.legsAnim = endFrame;
    }

    //now set the animation on the server ghoul2 instance.
    assert!(!(*eweb).ghoul2.is_null());
    trap::G2API_SetBoneAnim(
        (*eweb).ghoul2,
        0,
        "model_root",
        startFrame,
        endFrame,
        BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
        1.0,
        (*addr_of!(level)).time,
        -1.0,
        100,
    );
}

//----------------------------------------------------------------
/// `static void turretG2_fire ( gentity_t *ent, vec3_t start, vec3_t dir )` (g_turret_G2.c:340).
/// Fires the turret: bails if the muzzle is in solid, optionally jitters `dir` by `random`,
/// then either fires a turbolaser missile (with a muzzle effect + bone anim) or spawns a
/// `turret_proj` blaster bolt. No oracle.
//----------------------------------------------------------------
unsafe fn turretG2_fire(ent: *mut gentity_t, start: &mut vec3_t, dir: &vec3_t) {
    let mut org: vec3_t = [0.0; 3];
    let mut ang: vec3_t = [0.0; 3];
    let mut dir = *dir;

    if trap::PointContents(start, (*ent).s.number) & MASK_SHOT != 0 {
        return;
    }

    VectorMA(start, -START_DIS, &dir, &mut org); // dumb....

    if (*ent).random != 0.0 {
        vectoangles(&dir, &mut ang);
        ang[PITCH] += flrand(-(*ent).random, (*ent).random);
        ang[YAW] += flrand(-(*ent).random, (*ent).random);
        AngleVectors(&ang, Some(&mut dir), None, None);
    }

    vectoangles(&dir, &mut ang);

    if (*ent).spawnflags & SPF_TURRETG2_TURBO != 0 {
        //muzzle flash
        G_PlayEffectID((*ent).genericValue13, &org, &ang);
        WP_FireTurboLaserMissile(ent, start, &dir);
        if (*ent).alt_fire != QFALSE {
            TurboLaser_SetBoneAnim(ent, 2, 3);
        } else {
            TurboLaser_SetBoneAnim(ent, 0, 1);
        }
    } else {
        G_PlayEffectID(G_EffectIndex("blaster/muzzle_flash"), &org, &ang);
        let bolt = G_Spawn();

        (*bolt).classname = c"turret_proj".as_ptr() as *mut c_char;
        (*bolt).nextthink = (*addr_of!(level)).time + 10000;
        (*bolt).think = Some(G_FreeEntity);
        (*bolt).s.eType = ET_MISSILE;
        (*bolt).s.weapon = WP_BLASTER;
        (*bolt).r.ownerNum = (*ent).s.number;
        (*bolt).damage = (*ent).damage;
        (*bolt).alliedTeam = (*ent).alliedTeam;
        (*bolt).teamnodmg = (*ent).teamnodmg;
        (*bolt).dflags = DAMAGE_NO_KNOCKBACK | DAMAGE_HEAVY_WEAP_CLASS; // Don't push them around, or else we are constantly re-aiming
        (*bolt).splashDamage = (*ent).splashDamage;
        (*bolt).splashRadius = (*ent).splashDamage;
        (*bolt).methodOfDeath = MOD_TARGET_LASER; //MOD_ENERGY;
        (*bolt).splashMethodOfDeath = MOD_TARGET_LASER; //MOD_ENERGY;
        (*bolt).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
        //bolt->trigger_formation = qfalse;		// don't draw tail on first frame

        VectorSet(&mut (*bolt).r.maxs, 1.5, 1.5, 1.5);
        let maxs = (*bolt).r.maxs;
        VectorScale(&maxs, -1.0, &mut (*bolt).r.mins);
        (*bolt).s.pos.trType = TR_LINEAR;
        (*bolt).s.pos.trTime = (*addr_of!(level)).time;
        VectorCopy(start, &mut (*bolt).s.pos.trBase);
        VectorScale(&dir, (*ent).mass, &mut (*bolt).s.pos.trDelta);
        trap::SnapVector(&mut (*bolt).s.pos.trDelta); // save net bandwidth
        VectorCopy(start, &mut (*bolt).r.currentOrigin);
    }
}

/// `void turretG2_respawn( gentity_t *self )` (g_turret_G2.c:410). Re-arms a killed turret:
/// restores its callbacks/takedamage flags, resets the shader frame and crosshair weapon,
/// rebuilds the live model, and restores health. No oracle.
pub unsafe fn turretG2_respawn(self_: *mut gentity_t) {
    (*self_).r#use = Some(turretG2_base_use);
    (*self_).pain = Some(TurretG2Pain);
    (*self_).die = Some(turretG2_die);
    (*self_).takedamage = QTRUE;
    (*self_).s.shouldtarget = QTRUE;
    //self->s.owner = MAX_CLIENTS; //not owned by any client
    if (*self_).s.eFlags & EF_SHADER_ANIM != 0 {
        (*self_).s.frame = 0; // normal
    }
    (*self_).s.weapon = WP_TURRET; // crosshair code uses this to mark crosshair red

    turretG2_set_models(self_, QFALSE);
    (*self_).s.health = (*self_).genericValue6;
    (*self_).health = (*self_).genericValue6;
    if (*self_).maxHealth != 0 {
        G_ScaleNetHealth(self_);
    }
    (*self_).genericValue5 = 0; //clear this now
}

//-----------------------------------------------------
/// `void turretG2_head_think( gentity_t *self )` (g_turret_G2.c:433). The turret's per-frame
/// fire logic: when it has an enemy and its timers have elapsed, it reads the muzzle bolt
/// matrix off the Ghoul2 instance, derives the muzzle origin + forward, and calls
/// [`turretG2_fire`]. No oracle.
//-----------------------------------------------------
pub unsafe fn turretG2_head_think(self_: *mut gentity_t) {
    // if it's time to fire and we have an enemy, then gun 'em down!  pushDebounce time controls next fire time
    if !(*self_).enemy.is_null()
        && (*self_).setTime < (*addr_of!(level)).time
        && (*self_).attackDebounceTime < (*addr_of!(level)).time
    {
        let mut fwd: vec3_t = [0.0; 3];
        let mut org: vec3_t = [0.0; 3];
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

        // set up our next fire time
        (*self_).setTime = (*addr_of!(level)).time + (*self_).wait as c_int;

        // Getting the flash bolt here
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            if (*self_).alt_fire != QFALSE {
                (*self_).genericValue12
            } else {
                (*self_).genericValue11
            },
            &mut boltMatrix,
            &(*self_).r.currentAngles,
            &(*self_).r.currentOrigin,
            (*addr_of!(level)).time,
            core::ptr::null_mut(),
            &(*self_).modelScale,
        );
        if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
            (*self_).alt_fire = if (*self_).alt_fire == QFALSE {
                QTRUE
            } else {
                QFALSE
            };
        }

        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut org);
        //BG_GiveMeVectorFromMatrix( &boltMatrix, POSITIVE_Y, fwd );
        if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
            BG_GiveMeVectorFromMatrix(&boltMatrix, POSITIVE_X, &mut fwd);
        } else {
            BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_X, &mut fwd);
        }

        let org_copy = org;
        VectorMA(&org_copy, START_DIS, &fwd, &mut org);

        turretG2_fire(self_, &mut org, &fwd);
        (*self_).fly_sound_debounce_time = (*addr_of!(level)).time; //used as lastShotTime
    }
}

//-----------------------------------------------------
/// `static void turretG2_aim( gentity_t *self )` (g_turret_G2.c:481). Drives the turret's
/// per-frame yaw/pitch trajectory: evaluates current angles, aims at the enemy (eye-bolt
/// matrix, optional lead) when present, caps the per-frame turn by the max yaw/pitch speed,
/// writes a `TR_LINEAR` apos trajectory, drives the pitch bone angles, and toggles the turn
/// loop sound. No oracle.
//-----------------------------------------------------
unsafe fn turretG2_aim(self_: *mut gentity_t) {
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];
    let mut desiredAngles: vec3_t = [0.0; 3];
    let mut setAngle: vec3_t = [0.0; 3];
    let mut diffYaw: f32 = 0.0;
    let mut diffPitch: f32 = 0.0;
    let maxYawSpeed: f32 = if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
        30.0
    } else {
        14.0
    };
    let maxPitchSpeed: f32 = if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
        15.0
    } else {
        3.0
    };

    // move our gun base yaw to where we should be at this time....
    BG_EvaluateTrajectory(
        &(*self_).s.apos,
        (*addr_of!(level)).time,
        &mut (*self_).r.currentAngles,
    );
    (*self_).r.currentAngles[YAW] = AngleNormalize360((*self_).r.currentAngles[YAW]);
    (*self_).speed = AngleNormalize360((*self_).speed);

    if !(*self_).enemy.is_null() {
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        // ...then we'll calculate what new aim adjustments we should attempt to make this frame
        // Aim at enemy
        if !(*(*self_).enemy).client.is_null() {
            VectorCopy(&(*(*(*self_).enemy).client).renderInfo.eyePoint, &mut org);
        } else {
            VectorCopy(&(*(*self_).enemy).r.currentOrigin, &mut org);
        }
        if (*self_).spawnflags & 2 != 0 {
            org[2] -= 15.0;
        } else {
            org[2] -= 5.0;
        }

        if (*self_).spawnflags & SPF_TURRETG2_LEAD_ENEMY != 0 {
            //we want to lead them a bit
            let mut diff: vec3_t = [0.0; 3];
            let mut velocity: vec3_t = [0.0; 3];
            let dist: f32;
            VectorSubtract(&org, &(*self_).s.origin, &mut diff);
            dist = VectorNormalize(&mut diff);
            if !(*(*self_).enemy).client.is_null() {
                VectorCopy(&(*(*(*self_).enemy).client).ps.velocity, &mut velocity);
            } else {
                VectorCopy(&(*(*self_).enemy).s.pos.trDelta, &mut velocity);
            }
            let org_copy = org;
            VectorMA(&org_copy, dist / (*self_).mass, &velocity, &mut org);
        }

        // Getting the "eye" here
        trap::G2API_GetBoltMatrix(
            (*self_).ghoul2,
            0,
            if (*self_).alt_fire != QFALSE {
                (*self_).genericValue12
            } else {
                (*self_).genericValue11
            },
            &mut boltMatrix,
            &(*self_).r.currentAngles,
            &(*self_).s.origin,
            (*addr_of!(level)).time,
            core::ptr::null_mut(),
            &(*self_).modelScale,
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut org2);

        VectorSubtract(&org, &org2, &mut enemyDir);
        vectoangles(&enemyDir, &mut desiredAngles);

        diffYaw = AngleSubtract((*self_).r.currentAngles[YAW], desiredAngles[YAW]);
        diffPitch = AngleSubtract((*self_).speed, desiredAngles[PITCH]);
    } else {
        // no enemy, so make us slowly sweep back and forth as if searching for a new one
        //		diffYaw = sin( level.time * 0.0001f + self->count ) * 5.0f;	// don't do this for now since it can make it go into walls.
    }

    if diffYaw != 0.0 {
        // cap max speed....
        if diffYaw.abs() > maxYawSpeed {
            diffYaw = if diffYaw >= 0.0 {
                maxYawSpeed
            } else {
                -maxYawSpeed
            };
        }

        // ...then set up our desired yaw
        VectorSet(&mut setAngle, 0.0, diffYaw, 0.0);

        VectorCopy(&(*self_).r.currentAngles, &mut (*self_).s.apos.trBase);
        VectorScale(&setAngle, -5.0, &mut (*self_).s.apos.trDelta);
        (*self_).s.apos.trTime = (*addr_of!(level)).time;
        (*self_).s.apos.trType = TR_LINEAR;
    }

    if diffPitch != 0.0 {
        if diffPitch.abs() > maxPitchSpeed {
            // cap max speed
            (*self_).speed += if diffPitch > 0.0 {
                -maxPitchSpeed
            } else {
                maxPitchSpeed
            };
        } else {
            // small enough, so just add half the diff so we smooth out the stopping
            (*self_).speed -= diffPitch; //desiredAngles[PITCH];
        }

        // Note that this is NOT interpolated, so it will be less smooth...On the other hand, it does use Ghoul2 to blend, so it may smooth it out a bit?
        if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
            if (*self_).spawnflags & 2 != 0 {
                VectorSet(&mut desiredAngles, 0.0, 0.0, -(*self_).speed);
            } else {
                VectorSet(&mut desiredAngles, 0.0, 0.0, (*self_).speed);
            }
            G2Tur_SetBoneAngles(self_, "pitch", &desiredAngles);
        } else {
            if (*self_).spawnflags & 2 != 0 {
                VectorSet(&mut desiredAngles, (*self_).speed, 0.0, 0.0);
            } else {
                VectorSet(&mut desiredAngles, -(*self_).speed, 0.0, 0.0);
            }
            G2Tur_SetBoneAngles(self_, "Bone_body", &desiredAngles);
        }
        /*
        trap_G2API_SetBoneAngles( self->ghoul2,
                        0,
                        "Bone_body",
                        desiredAngles,
                        BONE_ANGLES_POSTMULT,
                        POSITIVE_Y,
                        POSITIVE_Z,
                        POSITIVE_X,
                        NULL,
                        100,
                        level.time );
                        */
    }

    if diffYaw != 0.0 || diffPitch != 0.0 {
        //FIXME: turbolaser sounds
        if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
            (*self_).s.loopSound = G_SoundIndex("sound/vehicles/weapons/turbolaser/turn.wav");
        } else {
            (*self_).s.loopSound = G_SoundIndex("sound/chars/turret/move.wav");
        }
    } else {
        (*self_).s.loopSound = 0;
    }
}

//-----------------------------------------------------
/// `static void turretG2_turnoff( gentity_t *self )` (g_turret_G2.c:647). Shuts the turret
/// down: stops the turbolaser anim, plays the shutdown sound, arms the 5-second ping window
/// (`aimDebounceTime`), and clears the enemy. No oracle.
//-----------------------------------------------------
unsafe fn turretG2_turnoff(self_: *mut gentity_t) {
    if (*self_).enemy.is_null() {
        // we don't need to turnoff
        return;
    }
    if (*self_).spawnflags & SPF_TURRETG2_TURBO != 0 {
        TurboLaser_SetBoneAnim(self_, 4, 5);
    }
    // shut-down sound
    if (*self_).spawnflags & SPF_TURRETG2_TURBO == 0 {
        G_Sound(
            self_,
            CHAN_BODY,
            G_SoundIndex("sound/chars/turret/shutdown.wav"),
        );
    }

    // make turret play ping sound for 5 seconds
    (*self_).aimDebounceTime = (*addr_of!(level)).time + 5000;

    // Clear enemy
    (*self_).enemy = core::ptr::null_mut();
}

//-----------------------------------------------------
/// `static qboolean turretG2_find_enemies( gentity_t *self )` (g_turret_G2.c:673). Scans the
/// turret's radius for a valid target (clients, or breakable brushes it's allowed to break),
/// skipping dead/notarget/spectator/allied ents; requires PVS + a clear trace; prefers the
/// nearest, preferring clients. On acquisition it sets the enemy and fires `target2`. Returns
/// whether an enemy was found. No oracle.
//-----------------------------------------------------
unsafe fn turretG2_find_enemies(self_: *mut gentity_t) -> qboolean {
    let mut found: qboolean = QFALSE;
    let count: c_int;
    let mut bestDist: f32 = (*self_).radius * (*self_).radius;
    let mut enemyDist: f32;
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];
    let mut foundClient: qboolean = QFALSE;
    let mut entity_list: [*mut gentity_t; MAX_GENTITIES] = [core::ptr::null_mut(); MAX_GENTITIES];
    let mut target: *mut gentity_t;
    let mut bestTarget: *mut gentity_t = core::ptr::null_mut();

    if (*self_).aimDebounceTime > (*addr_of!(level)).time {
        // time since we've been shut off
        // We were active and alert, i.e. had an enemy in the last 3 secs
        if (*self_).painDebounceTime < (*addr_of!(level)).time {
            if (*self_).spawnflags & SPF_TURRETG2_TURBO == 0 {
                G_Sound(self_, CHAN_BODY, G_SoundIndex("sound/chars/turret/ping.wav"));
            }
            (*self_).painDebounceTime = (*addr_of!(level)).time + 1000;
        }
    }

    VectorCopy(&(*self_).r.currentOrigin, &mut org2);
    if (*self_).spawnflags & 2 != 0 {
        org2[2] += 20.0;
    } else {
        org2[2] -= 20.0;
    }

    count = G_RadiusList(&org2, (*self_).radius, self_, QTRUE, &mut entity_list);

    let mut i: c_int = 0;
    while i < count {
        target = entity_list[i as usize];

        if (*target).client.is_null() {
            // only attack clients
            if (*target).flags & FL_BBRUSH == 0 //not a breakable brush
                || (*target).takedamage == QFALSE //is a bbrush, but invincible
                || (!(*target).NPC_targetname.is_null()
                    && !(*self_).targetname.is_null()
                    && Q_stricmp((*target).NPC_targetname, (*self_).targetname) != 0)
            //not in invicible bbrush, but can only be broken by an NPC that is not me
            {
                i += 1;
                continue;
            }
            //else: we will shoot at bbrushes!
        }
        if target == self_
            || (*target).takedamage == QFALSE
            || (*target).health <= 0
            || (*target).flags & FL_NOTARGET != 0
        {
            i += 1;
            continue;
        }
        if !(*target).client.is_null()
            && (*(*target).client).sess.sessionTeam == TEAM_SPECTATOR
        {
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

        if !(*target).client.is_null() {
            VectorCopy(&(*(*target).client).renderInfo.eyePoint, &mut org);
        } else {
            VectorCopy(&(*target).r.currentOrigin, &mut org);
        }

        if (*self_).spawnflags & 2 != 0 {
            org[2] -= 15.0;
        } else {
            org[2] += 5.0;
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
            && (tr.fraction == 1.0 || tr.entityNum as c_int == (*target).s.number)
        {
            // Only acquire if have a clear shot, Is it in range and closer than our best?
            VectorSubtract(
                &(*target).r.currentOrigin,
                &(*self_).r.currentOrigin,
                &mut enemyDir,
            );
            enemyDist = VectorLengthSquared(&enemyDir);

            if enemyDist < bestDist || (!(*target).client.is_null() && foundClient == QFALSE) {
                // all things equal, keep current
                if (*self_).attackDebounceTime < (*addr_of!(level)).time {
                    // We haven't fired or acquired an enemy in the last 2 seconds-start-up sound
                    if (*self_).spawnflags & SPF_TURRETG2_TURBO == 0 {
                        G_Sound(
                            self_,
                            CHAN_BODY,
                            G_SoundIndex("sound/chars/turret/startup.wav"),
                        );
                    }

                    // Wind up turrets for a bit
                    (*self_).attackDebounceTime = (*addr_of!(level)).time + 1400;
                }

                bestTarget = target;
                bestDist = enemyDist;
                found = QTRUE;
                if !(*target).client.is_null() {
                    //prefer clients over non-clients
                    foundClient = QTRUE;
                }
            }
        }
        i += 1;
    }

    if found != QFALSE {
        /*
        if ( !self->enemy )
        {//just aquired one
            AddSoundEvent( bestTarget, self->r.currentOrigin, 256, AEL_DISCOVERED );
            AddSightEvent( bestTarget, self->r.currentOrigin, 512, AEL_DISCOVERED, 20 );
        }
        */
        G_SetEnemy(self_, bestTarget);
        if !(*self_).target2.is_null() && *(*self_).target2 != 0 {
            G_UseTargets2(self_, self_, (*self_).target2);
        }
    }

    found
}

//-----------------------------------------------------
/// `void turretG2_base_think( gentity_t *self )` (g_turret_G2.c:825). The turret's per-frame
/// `think`: handles dead/respawn and START_OFF states, drops invalid/spectator enemies,
/// acquires new ones ([`turretG2_find_enemies`]), keeps the lock while the enemy stays in
/// radius/PVS with a clear trace, sleeps ([`turretG2_turnoff`]) after a debounce when nothing
/// valid remains, and finally aims ([`turretG2_aim`]) + fires ([`turretG2_head_think`]).
/// No oracle.
//-----------------------------------------------------
pub unsafe extern "C" fn turretG2_base_think(self_: *mut gentity_t) {
    let mut turnOff: qboolean = QTRUE;
    let enemyDist: f32;
    let mut enemyDir: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];
    let mut org2: vec3_t = [0.0; 3];

    (*self_).nextthink = (*addr_of!(level)).time + FRAMETIME;

    if (*self_).health <= 0 {
        //dead
        if (*self_).spawnflags & SPF_TURRETG2_CANRESPAWN != 0 {
            //can respawn
            if (*self_).genericValue5 != 0 && (*self_).genericValue5 < (*addr_of!(level)).time {
                //we are dead, see if it's time to respawn
                turretG2_respawn(self_);
            }
        }
        return;
    } else if (*self_).spawnflags & 1 != 0 {
        // not turned on
        turretG2_turnoff(self_);
        turretG2_aim(self_);

        // No target
        (*self_).flags |= FL_NOTARGET;
        return;
    } else {
        // I'm all hot and bothered
        (*self_).flags &= !FL_NOTARGET;
    }

    if !(*self_).enemy.is_null() {
        if (*(*self_).enemy).health < 0 || (*(*self_).enemy).inuse == QFALSE {
            (*self_).enemy = core::ptr::null_mut();
        }
    }

    if (*self_).last_move_time < (*addr_of!(level)).time {
        //MISNOMER: used a enemy recalcing debouncer
        if turretG2_find_enemies(self_) != QFALSE {
            //found one
            turnOff = QFALSE;
            if !(*(*self_).enemy).client.is_null() {
                //hold on to clients for a min of 3 seconds
                (*self_).last_move_time = (*addr_of!(level)).time + 3000;
            } else {
                //hold less
                (*self_).last_move_time = (*addr_of!(level)).time + 500;
            }
        }
    }

    if !(*self_).enemy.is_null() {
        if !(*(*self_).enemy).client.is_null()
            && (*(*(*self_).enemy).client).sess.sessionTeam == TEAM_SPECTATOR
        {
            //don't keep going after spectators
            (*self_).enemy = core::ptr::null_mut();
        } else {
            //FIXME: remain single-minded or look for a new enemy every now and then?
            // enemy is alive
            VectorSubtract(
                &(*(*self_).enemy).r.currentOrigin,
                &(*self_).r.currentOrigin,
                &mut enemyDir,
            );
            enemyDist = VectorLengthSquared(&enemyDir);

            if enemyDist < (*self_).radius * (*self_).radius {
                // was in valid radius
                if trap::InPVS(&(*self_).r.currentOrigin, &(*(*self_).enemy).r.currentOrigin)
                    != QFALSE
                {
                    // Every now and again, check to see if we can even trace to the enemy

                    if !(*(*self_).enemy).client.is_null() {
                        VectorCopy(
                            &(*(*(*self_).enemy).client).renderInfo.eyePoint,
                            &mut org,
                        );
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
    }

    if turnOff != QFALSE {
        if (*self_).bounceCount < (*addr_of!(level)).time {
            // bounceCount is used to keep the thing from ping-ponging from on to off
            turretG2_turnoff(self_);
        }
    } else {
        // keep our enemy for a minimum of 2 seconds from now
        (*self_).bounceCount = (*addr_of!(level)).time + 2000 + (random() * 150.0) as c_int;
    }

    turretG2_aim(self_);
    if turnOff == QFALSE {
        turretG2_head_think(self_);
    }
}

//-----------------------------------------------------------------------------
/// `void turretG2_base_use( gentity_t *self, gentity_t *other, gentity_t *activator )`
/// (g_turret_G2.c:955). The turret's `use` callback: toggles on/off by flipping the START_OFF
/// spawnflag and updates the shader frame. `other`/`activator` are unused. No oracle.
//-----------------------------------------------------------------------------
pub unsafe extern "C" fn turretG2_base_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    // Toggle on and off
    (*self_).spawnflags ^= 1;

    if (*self_).s.eFlags & EF_SHADER_ANIM != 0 && (*self_).spawnflags & 1 != 0 {
        // Start_Off
        (*self_).s.frame = 1; // black
    } else {
        (*self_).s.frame = 0; // glow
    }
}

/*QUAKED misc_turretG2 (1 0 0) (-8 -8 -22) (8 8 0) START_OFF UPSIDE_DOWN CANRESPAWN TURBO LEAD SHOWRADAR

Turret that hangs from the ceiling, will aim and shoot at enemies

  START_OFF - Starts off
  UPSIDE_DOWN - make it rest on a surface/floor instead of hanging from the ceiling
  CANRESPAWN - will respawn after being killed (use count)
  TURBO - Big-ass, Boxy Death Star Turbo Laser version
  LEAD - Turret will aim ahead of moving targets ("lead" them)
  SHOWRADAR - show on radar

  radius - How far away an enemy can be for it to pick it up (default 512)
  wait	- Time between shots (default 150 ms)
  dmg	- How much damage each shot does (default 5)
  health - How much damage it can take before exploding (default 100)
  count - if CANRESPAWN spawnflag, decides how long it is before gun respawns (in ms) - defaults to 20000 (20 seconds)

  paintarget - target to fire off upon being hurt
  painwait - ms to wait between firing off pain targets

  random - random error (in degrees) of projectile direction when it comes out of the muzzle (default is 2)

  shotspeed - the speed of the missile it fires travels at (default is 1100 for regular turrets, 20000 for TURBOLASERS)

  splashDamage - How much damage the explosion does
  splashRadius - The radius of the explosion

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

  customscale - custom scaling size. 100 is normal size, 1024 is the max scaling. this will change the bounding box size, so be careful of starting in solid!

"icon" - icon that represents the objective on the radar
*/
//-----------------------------------------------------
/// `void SP_misc_turretG2( gentity_t *base )` (g_turret_G2.c:1023). Spawn function for the
/// `misc_turretG2` Ghoul2 turret: sets up the live model, parses `painwait`/`customscale`
/// (applying the model scale), runs [`finish_spawning_turretG2`], and sets the initial shader
/// frame + radar/shader eFlags. No oracle.
///
/// Central wiring TODO: register `"misc_turretG2"` → `SP_misc_turretG2` in the g_spawn.rs
/// classname spawn table (owned elsewhere).
//-----------------------------------------------------
pub unsafe extern "C" fn SP_misc_turretG2(base: *mut gentity_t) {
    let mut customscaleVal: c_int = 0;
    let mut s: *mut c_char = core::ptr::null_mut();
    turretG2_set_models(base, QFALSE);

    G_SpawnInt(c"painwait".as_ptr(), c"0".as_ptr(), &mut (*base).genericValue4);
    (*base).genericValue8 = 0;

    G_SpawnInt(c"customscale".as_ptr(), c"0".as_ptr(), &mut customscaleVal);
    (*base).s.iModelScale = customscaleVal;
    if (*base).s.iModelScale != 0 {
        if (*base).s.iModelScale > 1023 {
            (*base).s.iModelScale = 1023;
        }
        (*base).modelScale[0] = (*base).s.iModelScale as f32 / 100.0;
        (*base).modelScale[1] = (*base).s.iModelScale as f32 / 100.0;
        (*base).modelScale[2] = (*base).s.iModelScale as f32 / 100.0;
    }

    G_SpawnString(c"icon".as_ptr(), c"".as_ptr(), &mut s);
    if !s.is_null() && *s != 0 {
        // We have an icon, so index it now.  We are reusing the genericenemyindex
        // variable rather than adding a new one to the entity state.
        (*base).s.genericenemyindex = G_IconIndex(&CStr::from_ptr(s).to_string_lossy());
    }

    finish_spawning_turretG2(base);

    if (*base).spawnflags & 1 != 0 {
        // Start_Off
        (*base).s.frame = 1; // black
    } else {
        (*base).s.frame = 0; // glow
    }
    if (*base).spawnflags & SPF_TURRETG2_TURBO == 0 {
        (*base).s.eFlags |= EF_SHADER_ANIM;
    }

    if (*base).spawnflags & SPF_SHOWONRADAR != 0 {
        (*base).s.eFlags |= EF_RADAROBJECT;
    }
}

//-----------------------------------------------------
/// `void finish_spawning_turretG2( gentity_t *base )` (g_turret_G2.c:1068). Finishes
/// configuring a spawned turret: orients/places it (inverting for UPSIDE_DOWN), parses team
/// no-damage, installs the use/pain/think/die callbacks, fills in all the
/// TURBO/regular default config (random/mass/health/radius/wait/splash/damage/bounds),
/// stashes respawn health, applies bbox scaling, precaches FX/sounds, and links it. No oracle.
//-----------------------------------------------------
pub unsafe fn finish_spawning_turretG2(base: *mut gentity_t) {
    let mut fwd: vec3_t = [0.0; 3];
    let mut t: c_int = 0;

    if (*base).spawnflags & 2 != 0 {
        (*base).s.angles[ROLL] += 180.0;
        (*base).s.origin[2] -= 22.0;
    }

    G_SetAngles(base, &(*base).s.angles);
    AngleVectors(&(*base).r.currentAngles, Some(&mut fwd), None, None);

    G_SetOrigin(base, &(*base).s.origin);

    (*base).s.eType = ET_GENERAL;

    if !(*base).team.is_null() && *(*base).team != 0 && //g_gametype.integer == GT_SIEGE &&
        (*base).teamnodmg == 0
    {
        (*base).teamnodmg = atoi((*base).team);
    }
    (*base).team = core::ptr::null_mut();

    // Set up our explosion effect for the ExplodeDeath code....
    G_EffectIndex("turret/explode");
    G_EffectIndex("sparks/spark_exp_nosnd");

    (*base).r#use = Some(turretG2_base_use);
    (*base).pain = Some(TurretG2Pain);

    // don't start working right away
    (*base).think = Some(turretG2_base_think);
    (*base).nextthink = (*addr_of!(level)).time + FRAMETIME * 5;

    // this is really the pitch angle.....
    (*base).speed = 0.0;

    // respawn time defaults to 20 seconds
    if (*base).spawnflags & SPF_TURRETG2_CANRESPAWN != 0 && (*base).count == 0 {
        (*base).count = 20000;
    }

    G_SpawnFloat(c"shotspeed".as_ptr(), c"0".as_ptr(), &mut (*base).mass);
    if (*base).spawnflags & SPF_TURRETG2_TURBO != 0 {
        if (*base).random == 0.0 {
            //error worked into projectile direction
            (*base).random = 2.0;
        }

        if (*base).mass == 0.0 {
            //misnomer: speed of projectile
            (*base).mass = 20000.0;
        }

        if (*base).health == 0 {
            (*base).health = 2000;
        }

        // search radius
        if (*base).radius == 0.0 {
            (*base).radius = 32768.0;
        }

        // How quickly to fire
        if (*base).wait == 0.0 {
            (*base).wait = 1000.0; // + random() * 500;
        }

        if (*base).splashDamage == 0 {
            (*base).splashDamage = 200;
        }

        if (*base).splashRadius == 0 {
            (*base).splashRadius = 500;
        }

        // how much damage each shot does
        if (*base).damage == 0 {
            (*base).damage = 500;
        }

        if (*base).spawnflags & SPF_TURRETG2_TURBO != 0 {
            VectorSet(&mut (*base).r.maxs, 64.0, 64.0, 30.0);
            VectorSet(&mut (*base).r.mins, -64.0, -64.0, -30.0);
        }
        //start in "off" anim
        TurboLaser_SetBoneAnim(base, 4, 5);
        if (*addr_of!(g_gametype)).integer == GT_SIEGE {
            //FIXME: designer-specified?
            //FIXME: put on other entities, too, particularly siege objectives and bbrushes...
            (*base).s.eFlags2 |= EF2_BRACKET_ENTITY;
        }
    } else {
        if (*base).random == 0.0 {
            //error worked into projectile direction
            (*base).random = 2.0;
        }

        if (*base).mass == 0.0 {
            //misnomer: speed of projectile
            (*base).mass = 1100.0;
        }

        if (*base).health == 0 {
            (*base).health = 100;
        }

        // search radius
        if (*base).radius == 0.0 {
            (*base).radius = 512.0;
        }

        // How quickly to fire
        if (*base).wait == 0.0 {
            (*base).wait = 150.0 + random() * 55.0;
        }

        if (*base).splashDamage == 0 {
            (*base).splashDamage = 10;
        }

        if (*base).splashRadius == 0 {
            (*base).splashRadius = 25;
        }

        // how much damage each shot does
        if (*base).damage == 0 {
            (*base).damage = 5;
        }

        if (*base).spawnflags & 2 != 0 {
            //upside-down, invert r.mins and maxe
            VectorSet(&mut (*base).r.maxs, 10.0, 10.0, 30.0);
            VectorSet(&mut (*base).r.mins, -10.0, -10.0, 0.0);
        } else {
            VectorSet(&mut (*base).r.maxs, 10.0, 10.0, 0.0);
            VectorSet(&mut (*base).r.mins, -10.0, -10.0, -30.0);
        }
    }

    //stash health off for respawn.  NOTE: cannot use maxhealth because that might not be set if not showing the health bar
    (*base).genericValue6 = (*base).health;

    G_SpawnInt(c"showhealth".as_ptr(), c"0".as_ptr(), &mut t);
    if t != 0 {
        //a non-0 maxhealth value will mean we want to show the health on the hud
        (*base).maxHealth = (*base).health;
        G_ScaleNetHealth(base);
        (*base).s.shouldtarget = QTRUE;
        //base->s.owner = MAX_CLIENTS; //not owned by any client
    }

    if (*base).s.iModelScale != 0 {
        //let's scale the bbox too...
        let fScale: f32 = (*base).s.iModelScale as f32 / 100.0;
        let mins = (*base).r.mins;
        let maxs = (*base).r.maxs;
        VectorScale(&mins, fScale, &mut (*base).r.mins);
        VectorScale(&maxs, fScale, &mut (*base).r.maxs);
    }

    // Precache special FX and moving sounds
    if (*base).spawnflags & SPF_TURRETG2_TURBO != 0 {
        (*base).genericValue13 = G_EffectIndex("turret/turb_muzzle_flash");
        (*base).genericValue14 = G_EffectIndex("turret/turb_shot");
        (*base).genericValue15 = G_EffectIndex("turret/turb_impact");
        //FIXME: Turbo Laser Cannon sounds!
        G_SoundIndex("sound/vehicles/weapons/turbolaser/turn.wav");
    } else {
        G_SoundIndex("sound/chars/turret/startup.wav");
        G_SoundIndex("sound/chars/turret/shutdown.wav");
        G_SoundIndex("sound/chars/turret/ping.wav");
        G_SoundIndex("sound/chars/turret/move.wav");
    }

    (*base).r.contents =
        CONTENTS_BODY | CONTENTS_PLAYERCLIP | CONTENTS_MONSTERCLIP | CONTENTS_SHOTCLIP;

    //base->max_health = base->health;
    (*base).takedamage = QTRUE;
    (*base).die = Some(turretG2_die);

    (*base).material = MAT_METAL;
    //base->r.svFlags |= SVF_NO_TELEPORT|SVF_NONNPC_ENEMY|SVF_SELF_ANIMATING;

    // Register this so that we can use it for the missile effect
    RegisterItem(BG_FindItemForWeapon(WP_BLASTER));

    // But set us as a turret so that we can be identified as a turret
    (*base).s.weapon = WP_TURRET;

    trap::LinkEntity(base);
}
