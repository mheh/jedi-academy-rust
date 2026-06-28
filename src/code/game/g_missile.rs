// leave this line at the top for all g_xxxx.cpp files...
#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void};

// Type aliases and opaque struct definitions
#[repr(C)]
pub struct gentity_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct gclient_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct gNPC_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct level_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct playerState_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct entityState_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct pml_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct mdxaBone_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CCollisionRecord {
    _opaque: [u8; 0],
}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// External C functions
extern "C" {
    pub fn InFront(spot: *const f32, from: *const f32, fromAngles: *const f32, threshHold: f32) -> qboolean;
    pub fn LogAccuracyHit(target: *mut gentity_t, attacker: *mut gentity_t) -> qboolean;
    pub fn PM_SaberInParry(move_: c_int) -> qboolean;
    pub fn PM_SaberInReflect(move_: c_int) -> qboolean;
    pub fn PM_SaberInIdle(move_: c_int) -> qboolean;
    pub fn PM_SaberInAttack(move_: c_int) -> qboolean;
    pub fn PM_SaberInTransitionAny(move_: c_int) -> qboolean;
    pub fn PM_SaberInSpecialAttack(anim: c_int) -> qboolean;
    pub fn Jedi_FindEnemyInCone(slf: *mut gentity_t, fallback: *mut gentity_t, minDot: f32) -> *mut gentity_t;
    pub fn CalcEntitySpot(ent: *mut gentity_t, spot: c_int, result: *mut vec3_t) -> ();
    pub fn G_TempEntity(org: *const vec3_t, ev: c_int) -> *mut gentity_t;
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t) -> ();
    pub fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32) -> ();
    pub fn VectorMA(veca: *const f32, scale: f32, vecb: *const f32, out: *mut f32) -> ();
    pub fn VectorNormalize(vec: *mut f32) -> f32;
    pub fn VectorLength(vec: *const f32) -> f32;
    pub fn VectorScale(in_vec: *const f32, scale: f32, out: *mut f32) -> ();
    pub fn VectorSet(vec: *mut f32, x: f32, y: f32, z: f32) -> ();
    pub fn VectorClear(vec: *mut f32) -> ();
    pub fn VectorCompare(a: *const f32, b: *const f32) -> qboolean;
    pub fn VectorAdd(a: *const f32, b: *const f32, out: *mut f32) -> ();
    pub fn DotProduct(a: *const f32, b: *const f32) -> f32;
    pub fn CrossProduct(a: *const f32, b: *const f32, out: *mut f32) -> ();
    pub fn DirToByte(dir: *const f32) -> c_int;
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32) -> ();
    pub fn SnapVector(vec: *mut f32) -> ();
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn Q_flrand(low: f32, high: f32) -> f32;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn EvaluateTrajectory(tr: *const c_void, time: c_int, result: *mut vec3_t) -> ();
    pub fn EvaluateTrajectoryDelta(tr: *const c_void, time: c_int, result: *mut f32) -> ();
    pub fn G_PlayEffect(name: *const c_char, org: *const vec3_t, dir: *const f32) -> ();
    pub fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int) -> ();
    pub fn G_SetOrigin(ent: *mut gentity_t, origin: *const vec3_t) -> ();
    pub fn G_Damage(target: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *const f32,
                point: *const f32, damage: c_int, dflags: c_int, mod_: c_int, hitLoc: c_int) -> ();
    pub fn G_RadiusDamage(origin: *const f32, attacker: *mut gentity_t, damage: c_int, radius: f32,
                      ignore: *mut gentity_t, mod_: c_int) -> ();
    pub fn G_FreeEntity(ent: *mut gentity_t) -> ();
    pub fn GEntity_TouchFunc(ent: *mut gentity_t, other: *mut gentity_t, tr: *const trace_t) -> ();
    pub fn AddSoundEvent(ent: *mut gentity_t, pos: *const f32, radius: c_int, level: c_int, bNoUse: qboolean, bNoSound: qboolean) -> ();
    pub fn AddSightEvent(ent: *mut gentity_t, pos: *const f32, radius: c_int, level: c_int, other: c_int) -> ();
    pub fn Saboteur_Decloak(ent: *mut gentity_t, time: c_int) -> ();
    pub fn DEMP2_AltDetonate(ent: *mut gentity_t) -> ();
    pub fn WP_SaberBlock(saber: *mut gentity_t, hitloc: *const f32, missleBlock: qboolean) -> ();
    pub fn laserTrapStick(ent: *mut gentity_t, endpos: *const f32, normal: *const f32) -> ();
    pub fn W_AccuracyLoggableWeapon(weapon: c_int, alt_fire: qboolean, mod_: c_int) -> qboolean;
    pub fn G_RunThink(ent: *mut gentity_t) -> ();
    pub fn G_MoverTouchPushTriggers(ent: *mut gentity_t, oldOrg: *const f32) -> ();
    pub fn G_GetHitLocFromSurfName(ent: *mut gentity_t, surf: *const c_char, hitLoc: *mut c_int,
                               pos: *const f32, a: *const c_void, b: *const c_void, mod_: c_int) -> ();
    pub fn gi_trace(tr: *mut trace_t, start: *const f32, mins: *const f32, maxs: *const f32,
                end: *const f32, passent: c_int, contentmask: c_int, collisionMap: c_int, radius: c_int) -> ();
    pub fn gi_linkentity(ent: *mut gentity_t) -> ();
    pub fn gi_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int,
                             result: *mut mdxaBone_t, angles: *const f32, position: *const f32,
                             time: c_int, modelList: *mut c_void, scale: f32) -> ();
    pub fn gi_G2API_GiveMeVectorFromMatrix(matrix: *const mdxaBone_t, flags: c_int, vec: *mut f32) -> ();
    pub fn gi_G2API_GetSurfaceName(ghoul2: *mut c_void, modelIndex: c_int, surfIndex: c_int) -> *const c_char;

    // Global variables
    pub static mut level: level_t;
    pub static mut g_entities: gentity_t;
    pub static mut cg: c_void;
    pub static mut pml: pml_t;
    pub static mut g_spskill: *mut c_void;
}

// Constants
pub const WP_BOWCASTER: c_int = 2;
pub const WP_BLASTER: c_int = 3;
pub const WP_BRYAR_PISTOL: c_int = 4;
pub const WP_BLASTER_PISTOL: c_int = 5;
pub const WP_THERMAL: c_int = 6;
pub const WP_TRIP_MINE: c_int = 7;
pub const WP_SABER: c_int = 1;
pub const WP_DEMP2: c_int = 8;
pub const WP_NOGHRI_STICK: c_int = 12;
pub const WP_FLECHETTE: c_int = 9;
pub const WP_ROCKET_LAUNCHER: c_int = 10;
pub const WP_REPEATER: c_int = 11;

pub const EV_GRENADE_BOUNCE: c_int = 1;
pub const EV_MISSILE_HIT: c_int = 2;
pub const EV_MISSILE_MISS: c_int = 3;
pub const EV_MISSILE_STICK: c_int = 4;
pub const ET_GENERAL: c_int = 1;
pub const ET_MISSILE: c_int = 2;
pub const ET_MOVER: c_int = 3;

pub const EF_BOUNCE: c_int = 1;
pub const EF_BOUNCE_HALF: c_int = 2;
pub const EF_BOUNCE_SHRAPNEL: c_int = 4;
pub const EF_MISSILE_STICK: c_int = 8;
pub const EF_HELD_BY_SAND_CREATURE: c_int = 16;
pub const EF_TELEPORT_BIT: c_int = 32;

pub const FL_DMG_BY_HEAVY_WEAP_ONLY: c_int = 1;
pub const FL_SHIELDED: c_int = 2;

pub const CONTENTS_LIGHTSABER: c_int = 1;
pub const SURF_FORCEFIELD: c_int = 1;
pub const SURF_NOIMPACT: c_int = 2;
pub const DAMAGE_HEAVY_WEAP_CLASS: c_int = 1;
pub const DAMAGE_NO_DAMAGE: c_int = 0;

pub const MOD_UNKNOWN: c_int = 0;
pub const MOD_CRUSH: c_int = 1;
pub const MOD_DEMP2_ALT: c_int = 2;

pub const HL_NONE: c_int = -1;

pub const ENTITYNUM_NONE: c_int = -1;
pub const ENTITYNUM_WORLD: c_int = 0x7fffffff;

pub const FORCE_LEVEL_1: c_int = 1;
pub const FORCE_LEVEL_2: c_int = 2;
pub const FORCE_LEVEL_3: c_int = 3;
pub const FP_SABER_DEFENSE: c_int = 0;
pub const FP_SPEED: c_int = 1;

pub const OVERCLIP: f32 = 1.5;
pub const BUMPCLIP: f32 = 1.5;
pub const MAX_CLIP_PLANES: usize = 5;
pub const MAX_G2_COLLISIONS: usize = 5;

pub const SPOT_HEAD: c_int = 0;
pub const ORIGIN: c_int = 0;
pub const YAW: usize = 0;

pub const AEL_DISCOVERED: c_int = 0;
pub const AEL_SUSPICIOUS: c_int = 1;
pub const AEL_DANGER: c_int = 2;
pub const AEL_DANGER_GREAT: c_int = 3;

pub const FRAMETIME: c_int = 16;

pub const TR_STATIONARY: c_int = 0;
pub const TR_INTERPOLATE: c_int = 1;
pub const TR_GRAVITY: c_int = 2;

pub const SABER_REFLECT_MISSILE_CONE: f32 = 0.0;

pub const CLASS_SEEKER: c_int = 0;
pub const CLASS_PROBE: c_int = 1;
pub const CLASS_MOUSE: c_int = 2;
pub const CLASS_GONK: c_int = 3;
pub const CLASS_R2D2: c_int = 4;
pub const CLASS_R5D2: c_int = 5;
pub const CLASS_REMOTE: c_int = 6;
pub const CLASS_MARK1: c_int = 7;
pub const CLASS_MARK2: c_int = 8;
pub const CLASS_INTERROGATOR: c_int = 9;
pub const CLASS_ATST: c_int = 10;
pub const CLASS_SENTRY: c_int = 11;
pub const CLASS_SABOTEUR: c_int = 12;

pub const PW_SHOCKED: c_int = 0;

pub const NPCAI_SHIELDS: c_int = 1;

pub const SEF_DEFLECTED: c_int = 1;

pub const PERS_ACCURACY_HITS: c_int = 0;

pub const MIN_WALK_NORMAL: f32 = 0.7;

pub const G2_COLLIDE: c_int = 0;
pub const G2_RETURNONHIT: c_int = 1;
pub const G2_FRONTFACE: c_int = 1;

//-------------------------------------------------------------------------
#[cfg(feature = "_IMMERSION")]
#[allow(unused_variables)]
pub unsafe fn G_MissileBounceEffect(ent: *mut gentity_t, hitEntNum: c_int, org: *const vec3_t, dir: *const f32, hitWorld: qboolean) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
}

#[cfg(not(feature = "_IMMERSION"))]
#[allow(unused_variables)]
pub unsafe fn G_MissileBounceEffect(ent: *mut gentity_t, org: *const vec3_t, dir: *const f32, hitWorld: qboolean) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
}

#[cfg(feature = "_IMMERSION")]
#[allow(unused_variables)]
pub unsafe fn G_MissileReflectEffect(ent: *mut gentity_t, hitEntNum: c_int, org: *const vec3_t, dir: *const f32) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
}

#[cfg(not(feature = "_IMMERSION"))]
#[allow(unused_variables)]
pub unsafe fn G_MissileReflectEffect(ent: *mut gentity_t, org: *const vec3_t, dir: *const f32) {
    //FIXME: have an EV_BOUNCE_MISSILE event that checks the s.weapon and does the appropriate effect
}

//-------------------------------------------------------------------------
#[allow(unused_variables)]
unsafe fn G_MissileStick(missile: *mut gentity_t, other: *mut gentity_t, tr: *const trace_t) {
    // we bounce off of NPC's and misc model breakables because sticking to them requires too much effort
}

/*
================
G_ReflectMissile

  Reflect the missile roughly back at it's owner
================
*/
#[allow(unused_variables)]
pub unsafe fn G_ReflectMissile(ent: *mut gentity_t, missile: *mut gentity_t, forward: *const f32) {
}

/*
================
G_BounceRollMissile

================
*/
#[allow(unused_variables)]
pub unsafe fn G_BounceRollMissile(ent: *mut gentity_t, trace: *const trace_t) {
}

/*
================
G_BounceMissile

================
*/
#[allow(unused_variables)]
pub unsafe fn G_BounceMissile(ent: *mut gentity_t, trace: *const trace_t) {
}

/*
================
G_MissileImpact

================
*/

#[allow(unused_variables)]
pub unsafe fn NoghriGasCloudThink(slf: *mut gentity_t) {
}

#[allow(unused_variables)]
pub unsafe fn G_SpawnNoghriGasCloud(ent: *mut gentity_t) {
    //FIXME: force-pushable/dispersable?
}

#[allow(unused_variables)]
pub unsafe fn G_MissileImpacted(ent: *mut gentity_t, other: *mut gentity_t, impactPos: *const vec3_t, normal: *const f32, hitLoc: c_int) {
}

//------------------------------------------------
#[allow(unused_variables)]
unsafe fn G_MissileAddAlerts(ent: *mut gentity_t) {
    //Add the event
}

//------------------------------------------------------
#[allow(unused_variables)]
pub unsafe fn G_MissileImpact(ent: *mut gentity_t, trace: *const trace_t, hitLoc: c_int) {
}

/*
================
G_ExplodeMissile

Explode a missile without an impact
================
*/
#[allow(unused_variables)]
pub unsafe fn G_ExplodeMissile(ent: *mut gentity_t) {
}


#[allow(unused_variables)]
pub unsafe fn G_RunStuckMissile(ent: *mut gentity_t) {
}

/*
==================

G_GroundTrace

==================
*/
#[allow(unused_variables)]
pub unsafe fn G_GroundTrace(ent: *mut gentity_t, pPml: *mut pml_t) -> c_int {
    ENTITYNUM_NONE
}

#[allow(unused_variables)]
pub unsafe fn G_ClipVelocity(in_vec: *const f32, normal: *const f32, out: *mut f32, overbounce: f32) {
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
#[allow(unused_variables)]
pub unsafe fn G_RollMissile(ent: *mut gentity_t) {
}

/*
================
G_RunMissile

================
*/
#[allow(unused_variables)]
pub unsafe fn G_RunMissile(ent: *mut gentity_t) {
}
