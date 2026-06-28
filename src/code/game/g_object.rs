// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// #include "g_local.h"
// #include "g_functions.h"

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void};

// ============================================================================
// Type Definitions and Stubs
// ============================================================================

/// Trajectory type enumeration for entity movement
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum trType_e {
    TR_STATIONARY = 0,
    TR_INTERPOLATE = 1,
    TR_LINEAR = 2,
    TR_LINEAR_STOP = 3,
    TR_NONLINEAR_STOP = 4,
    TR_SINE = 5,
    TR_GRAVITY = 6,
}

pub type trType_t = trType_e;

/// Trajectory structure - tracks position/rotation over time
#[repr(C)]
pub struct trajectory_t {
    pub trType: trType_t,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: [f32; 3],
    pub trDelta: [f32; 3],
}

/// Plane structure for collision
#[repr(C)]
pub struct cplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: c_int,
    pub signbits: c_int,
    pub pad: [c_char; 4],
}

/// Trace result structure
#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: cplane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
}

/// Entity state - networked data
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub pos: trajectory_t,
    pub apos: trajectory_t,
    pub time: c_int,
    pub time2: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub frame: c_int,
    pub solid: c_int,
    pub event: c_int,
    pub eventParm: c_int,
    pub powerups: c_int,
    pub loopSound: c_int,
}

/// Game entity structure
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub client: *mut c_void,
    pub inuse: c_int,
    pub linked: c_int,
    pub svFlags: c_int,
    pub bmodel: c_int,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub contents: c_int,
    pub absmin: [f32; 3],
    pub absmax: [f32; 3],
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
    pub owner: *mut gentity_t,
    // ... (ghoul2 and other fields omitted for brevity)
    _padding1: [u8; 1024], // Placeholder for fields we don't access
    pub classname: *mut c_char,
    pub spawnflags: c_int,
    pub flags: c_int,
    pub model: *mut c_char,
    pub model2: *mut c_char,
    pub freetime: c_int,
    pub eventTime: c_int,
    pub freeAfterEvent: c_int,
    pub physicsBounce: f32,
    pub clipmask: c_int,
    pub speed: f32,
    pub resultspeed: f32,
    pub lastMoveTime: c_int,
    pub movedir: [f32; 3],
    pub lastOrigin: [f32; 3],
    pub lastAngles: [f32; 3],
    pub mass: f32,
    pub lastImpact: c_int,
    pub watertype: c_int,
    pub waterlevel: c_int,
    pub angle: f32,
    pub target: *mut c_char,
    pub targetname: *mut c_char,
    pub health: c_int,
    pub nextthink: c_int,
    pub takedamage: c_int,
    pub count: c_int,
    pub damage: c_int,
    pub pos1: [f32; 3],
    pub pos2: [f32; 3],
    pub e_ThinkFunc: c_int,
    pub startFrame: c_int,
    pub endFrame: c_int,
}

/// CVars - console variables
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
}

/// Game level structure
#[repr(C)]
pub struct level_s {
    pub time: c_int,
    pub previousTime: c_int,
    // ... other fields not needed
}

// Type for trace function pointer
pub type TraceFn = unsafe extern "C" fn(
    *mut trace_t,
    *const f32,
    *const f32,
    *const f32,
    *const f32,
    c_int,
    c_int,
) -> ();

pub type LinkEntityFn = unsafe extern "C" fn(*mut gentity_t) -> ();

/// Game import structure (engine syscalls)
#[repr(C)]
pub struct gameImport_t {
    pub trace: Option<TraceFn>,
    pub linkentity: Option<LinkEntityFn>,
}

// Entity flags
pub const EF_BOUNCE: u32 = 0x00000010;
pub const EF_BOUNCE_HALF: u32 = 0x00000020;
pub const EF_AUTO_SIZE: u32 = 0x00000800;
pub const EF_TELEPORT_BIT: u32 = 0x00000004;

pub const SURF_NODAMAGE: c_int = 0x00000001;

pub const MASK_SOLID: c_int = 1;

pub const FRAMETIME: c_int = 100;

// ============================================================================
// External Functions and Globals
// ============================================================================

extern "C" {
    pub static level: level_s;
    pub static mut g_entities: [gentity_t; 1024];
    pub static g_gravity: *mut cvar_t;
    pub static gi: gameImport_t;

    // Forward declarations
    pub fn G_MoverTouchPushTriggers(ent: *mut gentity_t, oldOrg: [f32; 3]);
    pub fn DoImpact(
        self_: *mut gentity_t,
        other: *mut gentity_t,
        damageSelf: c_int,
        trace: *mut trace_t,
    );
    pub fn G_Spawn() -> *mut gentity_t;
    pub fn G_SetOrigin(ent: *mut gentity_t, origin: [f32; 3]);
    pub fn G_PlayEffect(fxID: c_int, origin: [f32; 3], forward: [f32; 3]);
    pub fn G_EffectIndex(name: *const c_char) -> c_int;
    pub fn G_Sound(ent: *mut gentity_t, soundIndex: c_int);
    pub fn G_SoundIndex(name: *const c_char) -> c_int;
    pub fn pitch_roll_for_slope(ent: *mut gentity_t, normal: [f32; 3]);
    pub fn GEntity_TouchFunc(
        ent: *mut gentity_t,
        other: *mut gentity_t,
        trace: *mut trace_t,
    );
    pub fn EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: *mut [f32; 3]);
    pub fn EvaluateTrajectoryDelta(tr: *const trajectory_t, atTime: c_int, result: *mut [f32; 3]);
    pub fn VectorMA(veca: [f32; 3], scale: f32, vecb: [f32; 3], vecc: *mut [f32; 3]);
    pub fn VectorScale(v: [f32; 3], scale: f32, out: *mut [f32; 3]);
    pub fn VectorCopy(src: [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    pub fn VectorNormalize2(v: [f32; 3], out: *mut [f32; 3]) -> f32;
    pub fn VectorClear(v: *mut [f32; 3]);
    pub fn VectorCompare(v1: [f32; 3], v2: [f32; 3]) -> c_int;
    pub fn DotProduct(v1: [f32; 3], v2: [f32; 3]) -> f32;
    pub fn Q_flrand(min: f32, max: f32) -> f32;
}

// Local inline implementations for common math operations
#[inline]
fn VectorCopy_inline(src: [f32; 3], dst: &mut [f32; 3]) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

#[inline]
fn VectorScale_inline(v: [f32; 3], scale: f32, out: &mut [f32; 3]) {
    out[0] = v[0] * scale;
    out[1] = v[1] * scale;
    out[2] = v[2] * scale;
}

#[inline]
fn VectorMA_inline(veca: [f32; 3], scale: f32, vecb: [f32; 3], vecc: &mut [f32; 3]) {
    vecc[0] = veca[0] + vecb[0] * scale;
    vecc[1] = veca[1] + vecb[1] * scale;
    vecc[2] = veca[2] + vecb[2] * scale;
}

#[inline]
fn VectorClear_inline(v: &mut [f32; 3]) {
    v[0] = 0.0;
    v[1] = 0.0;
    v[2] = 0.0;
}

#[inline]
fn VectorSet_inline(v: &mut [f32; 3], x: f32, y: f32, z: f32) {
    v[0] = x;
    v[1] = y;
    v[2] = z;
}

#[inline]
fn VectorNormalize_inline(v: &mut [f32; 3]) -> f32 {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0.0 {
        let inv_length = 1.0 / length;
        v[0] *= inv_length;
        v[1] *= inv_length;
        v[2] *= inv_length;
    }
    length
}

#[inline]
fn VectorNormalize2_inline(v: [f32; 3], out: &mut [f32; 3]) -> f32 {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0.0 {
        let inv_length = 1.0 / length;
        out[0] = v[0] * inv_length;
        out[1] = v[1] * inv_length;
        out[2] = v[2] * inv_length;
    } else {
        out[0] = 0.0;
        out[1] = 0.0;
        out[2] = 0.0;
    }
    length
}

#[inline]
fn DotProduct_inline(v1: [f32; 3], v2: [f32; 3]) -> f32 {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

#[inline]
fn VectorCompare_inline(v1: [f32; 3], v2: [f32; 3]) -> bool {
    v1[0] == v2[0] && v1[1] == v2[1] && v1[2] == v2[2]
}

// ============================================================================
// Translated Functions
// ============================================================================

/*
================
G_BounceMissile

================
*/
pub unsafe fn G_BounceObject(ent: *mut gentity_t, trace: *mut trace_t) {
    let mut velocity = [0.0; 3];
    let mut dot: f32;
    let hitTime: c_int;

    // reflect the velocity on the trace plane
    hitTime = level.previousTime + ((level.time - level.previousTime) as f32 * (*trace).fraction) as c_int;
    EvaluateTrajectoryDelta(&(*ent).s.pos, hitTime, &mut velocity);
    dot = DotProduct_inline(velocity, (*trace).plane.normal);
    let mut bounceFactor = 60.0 / (*ent).mass;
    if bounceFactor > 1.0 {
        bounceFactor = 1.0;
    }
    VectorMA_inline(velocity, -2.0 * dot * bounceFactor, (*trace).plane.normal, &mut (*ent).s.pos.trDelta);

    //FIXME: customized or material-based impact/bounce sounds
    if (*ent).s.eFlags & EF_BOUNCE_HALF != 0 {
        VectorScale_inline((*ent).s.pos.trDelta, 0.5, &mut (*ent).s.pos.trDelta);

        // check for stop
        if (((*trace).plane.normal[2] > 0.7 && (*g_gravity).value > 0.0)
            || ((*trace).plane.normal[2] < -0.7 && (*g_gravity).value < 0.0))
            && (((*ent).s.pos.trDelta[2] < 40.0 && (*g_gravity).value > 0.0)
                || ((*ent).s.pos.trDelta[2] > -40.0 && (*g_gravity).value < 0.0))
        {
            //this can happen even on very slightly sloped walls, so changed it from > 0 to > 0.7
            //G_SetOrigin( ent, trace->endpos );
            //ent->nextthink = level.time + 500;
            (*ent).s.apos.trType = TR_STATIONARY;
            VectorCopy_inline((*ent).currentAngles, &mut (*ent).s.apos.trBase);
            VectorCopy_inline((*trace).endpos, &mut (*ent).currentOrigin);
            VectorCopy_inline((*trace).endpos, &mut (*ent).s.pos.trBase);
            (*ent).s.pos.trTime = level.time;
            return;
        }
    }

    // NEW--It would seem that we want to set our trBase to the trace endpos
    //	and set the trTime to the actual time of impact....
    //	FIXME: Should we still consider adding the normal though??
    VectorCopy_inline((*trace).endpos, &mut (*ent).currentOrigin);
    (*ent).s.pos.trTime = hitTime;

    VectorCopy_inline((*ent).currentOrigin, &mut (*ent).s.pos.trBase);
    VectorCopy_inline((*trace).plane.normal, &mut (*ent).pos1); //???
}

/*
================
G_RunObject

  TODO:  When transition to 0 grav, push away from surface you were resting on
  TODO:  When free-floating in air, apply some friction to your trDelta (based on mass?)
================
*/
pub unsafe fn G_RunObject(ent: *mut gentity_t) {
    let mut origin = [0.0; 3];
    let mut oldOrg = [0.0; 3];
    let mut tr = core::mem::zeroed::<trace_t>();
    let mut traceEnt: *mut gentity_t = core::ptr::null_mut();

    //FIXME: floaters need to stop floating up after a while, even if gravity stays negative?
    if (*ent).s.pos.trType == TR_STATIONARY {
        //g_gravity->value <= 0 &&
        (*ent).s.pos.trType = TR_GRAVITY;
        VectorCopy_inline((*ent).currentOrigin, &mut (*ent).s.pos.trBase);
        (*ent).s.pos.trTime = level.previousTime; //?necc?
        if (*g_gravity).value == 0.0 {
            (*ent).s.pos.trDelta[2] += 100.0;
        }
    }

    (*ent).nextthink = level.time + FRAMETIME;

    VectorCopy_inline((*ent).currentOrigin, &mut oldOrg);
    // get current position
    EvaluateTrajectory(&(*ent).s.pos, level.time, &mut origin);
    //Get current angles?
    EvaluateTrajectory(&(*ent).s.apos, level.time, &mut (*ent).currentAngles);

    if VectorCompare_inline((*ent).currentOrigin, origin) {
        //error - didn't move at all!
        return;
    }
    // trace a line from the previous position to the current position,
    // ignoring interactions with the missile owner
    if let Some(trace_fn) = gi.trace {
        trace_fn(
            &mut tr,
            (*ent).currentOrigin.as_ptr(),
            (*ent).mins.as_ptr(),
            (*ent).maxs.as_ptr(),
            origin.as_ptr(),
            if !(*ent).owner.is_null() {
                (*(*ent).owner).s.number
            } else {
                (*ent).s.number
            },
            (*ent).clipmask,
        );
    }

    if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction != 0.0 {
        VectorCopy_inline(tr.endpos, &mut (*ent).currentOrigin);
        if let Some(linkent_fn) = gi.linkentity {
            linkent_fn(ent);
        }
    } else
    //if ( tr.startsolid )
    {
        tr.fraction = 0.0;
    }

    G_MoverTouchPushTriggers(ent, oldOrg);
    /*
    if ( !(ent->s.eFlags & EF_TELEPORT_BIT) && !(ent->svFlags & SVF_NO_TELEPORT) )
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

    if tr.fraction == 1.0 {
        if (*g_gravity).value <= 0.0 {
            if (*ent).s.apos.trType == TR_STATIONARY {
                VectorCopy_inline((*ent).currentAngles, &mut (*ent).s.apos.trBase);
                (*ent).s.apos.trType = TR_LINEAR;
                (*ent).s.apos.trDelta[1] = Q_flrand(-300.0, 300.0);
                (*ent).s.apos.trDelta[0] = Q_flrand(-10.0, 10.0);
                (*ent).s.apos.trDelta[2] = Q_flrand(-10.0, 10.0);
                (*ent).s.apos.trTime = level.time;
            }
        }
        //friction in zero-G
        if (*g_gravity).value == 0.0 {
            let friction = 0.975;
            /*friction -= ent->mass/1000.0f;
            if ( friction < 0.1 )
            {
                friction = 0.1f;
            }
            */
            VectorScale_inline((*ent).s.pos.trDelta, friction, &mut (*ent).s.pos.trDelta);
            VectorCopy_inline((*ent).currentOrigin, &mut (*ent).s.pos.trBase);
            (*ent).s.pos.trTime = level.time;
        }
        return;
    }

    //hit something

    //Do impact damage
    traceEnt = &mut g_entities[tr.entityNum as usize];
    if tr.fraction != 0.0 || (!traceEnt.is_null() && (*traceEnt).takedamage != 0) {
        if !VectorCompare_inline((*ent).currentOrigin, oldOrg) {
            //moved and impacted
            if !traceEnt.is_null() && (*traceEnt).takedamage != 0 {
                //hurt someone
                let mut fxDir = [0.0; 3];
                VectorNormalize2_inline((*ent).s.pos.trDelta, &mut fxDir);
                VectorScale_inline(fxDir, -1.0, &mut fxDir);
                G_PlayEffect(
                    G_EffectIndex(b"melee/kick_impact\0".as_ptr() as *const c_char),
                    tr.endpos,
                    fxDir,
                );
                //G_Sound( ent, G_SoundIndex( va( "sound/weapons/melee/punch%d", Q_irand( 1, 4 ) ) ) );
            } else {
                G_PlayEffect(
                    G_EffectIndex(b"melee/kick_impact_silent\0".as_ptr() as *const c_char),
                    tr.endpos,
                    tr.plane.normal,
                );
            }
            if (*ent).mass > 100.0 {
                G_Sound(
                    ent,
                    G_SoundIndex(b"sound/movers/objects/objectHitHeavy.wav\0".as_ptr() as *const c_char),
                );
            } else {
                G_Sound(
                    ent,
                    G_SoundIndex(b"sound/movers/objects/objectHit.wav\0".as_ptr() as *const c_char),
                );
            }
        }
        DoImpact(
            ent,
            traceEnt,
            if (tr.surfaceFlags & SURF_NODAMAGE as c_int) == 0 { 1 } else { 0 },
            &mut tr,
        );
    }

    if ent.is_null() || ((*ent).takedamage != 0 && (*ent).health <= 0) {
        //been destroyed by impact
        //chunks?
        G_Sound(
            ent,
            G_SoundIndex(b"sound/movers/objects/objectBreak.wav\0".as_ptr() as *const c_char),
        );
        return;
    }

    //do impact physics
    if (*ent).s.pos.trType == TR_GRAVITY {
        //tr.fraction < 1.0 &&
        //FIXME: only do this if no trDelta
        if (*g_gravity).value <= 0.0 || tr.plane.normal[2] < 0.7 {
            if ((*ent).s.eFlags & (EF_BOUNCE | EF_BOUNCE_HALF)) != 0 {
                if tr.fraction <= 0.0 {
                    VectorCopy_inline(tr.endpos, &mut (*ent).currentOrigin);
                    VectorCopy_inline(tr.endpos, &mut (*ent).s.pos.trBase);
                    VectorClear_inline(&mut (*ent).s.pos.trDelta);
                    (*ent).s.pos.trTime = level.time;
                } else {
                    G_BounceObject(ent, &mut tr);
                }
            } else {
                //slide down?
                //FIXME: slide off the slope
            }
        } else {
            (*ent).s.apos.trType = TR_STATIONARY;
            pitch_roll_for_slope(ent, tr.plane.normal);
            //ent->currentAngles[0] = 0;//FIXME: match to slope
            //ent->currentAngles[2] = 0;//FIXME: match to slope
            VectorCopy_inline((*ent).currentAngles, &mut (*ent).s.apos.trBase);
            //okay, we hit the floor, might as well stop or prediction will
            //make us go through the floor!
            //FIXME: this means we can't fall if something is pulled out from under us...
            G_StopObjectMoving(ent);
        }
    } else {
        (*ent).s.apos.trType = TR_STATIONARY;
        pitch_roll_for_slope(ent, tr.plane.normal);
        //ent->currentAngles[0] = 0;//FIXME: match to slope
        //ent->currentAngles[2] = 0;//FIXME: match to slope
        VectorCopy_inline((*ent).currentAngles, &mut (*ent).s.apos.trBase);
    }

    //call touch func
    GEntity_TouchFunc(ent, &mut g_entities[tr.entityNum as usize], &mut tr);
}

pub unsafe fn G_StopObjectMoving(object: *mut gentity_t) {
    (*object).s.pos.trType = TR_STATIONARY;
    VectorCopy_inline((*object).currentOrigin, &mut (*object).s.origin);
    VectorCopy_inline((*object).currentOrigin, &mut (*object).s.pos.trBase);
    VectorClear_inline(&mut (*object).s.pos.trDelta);

    /*
    //Stop spinning
    VectorClear( self->s.apos.trDelta );
    vectoangles(trace->plane.normal, self->s.angles);
    VectorCopy(self->s.angles, self->currentAngles );
    VectorCopy(self->s.angles, self->s.apos.trBase);
    */
}

pub unsafe fn G_StartObjectMoving(
    object: *mut gentity_t,
    mut dir: [f32; 3],
    speed: f32,
    trType: trType_t,
) {
    VectorNormalize_inline(&mut dir);

    //object->s.eType = ET_GENERAL;
    (*object).s.pos.trType = trType;
    VectorCopy_inline((*object).currentOrigin, &mut (*object).s.pos.trBase);
    VectorScale_inline(dir, speed, &mut (*object).s.pos.trDelta);
    (*object).s.pos.trTime = level.time;

    /*
    //FIXME: incorporate spin?
    vectoangles(dir, object->s.angles);
    VectorCopy(object->s.angles, object->s.apos.trBase);
    VectorSet(object->s.apos.trDelta, 300, 0, 0 );
    object->s.apos.trTime = level.time;
    */

    //FIXME: make these objects go through G_RunObject automatically, like missiles do
    if (*object).e_ThinkFunc == thinkF_NULL {
        (*object).nextthink = level.time + FRAMETIME;
        (*object).e_ThinkFunc = thinkF_G_RunObject;
    } else {
        //You're responsible for calling RunObject
    }
}

pub unsafe fn G_CreateObject(
    owner: *mut gentity_t,
    origin: [f32; 3],
    angles: [f32; 3],
    modelIndex: c_int,
    frame: c_int,
    trType: trType_t,
    effectID: c_int,
) -> *mut gentity_t {
    let object: *mut gentity_t;

    object = G_Spawn();

    if object.is_null() {
        return core::ptr::null_mut();
    }

    (*object).classname = b"object\0".as_ptr() as *mut c_char; //?
    (*object).nextthink = level.time + FRAMETIME;
    (*object).e_ThinkFunc = thinkF_G_RunObject;
    (*object).s.eType = ET_GENERAL;
    (*object).s.eFlags |= EF_AUTO_SIZE; //CG_Ents will create the mins & max itself based on model bounds
    (*object).s.modelindex = modelIndex;
    //FIXME: allow to set a targetname/script_targetname and animation info?
    (*object).s.frame = frame;
    (*object).startFrame = frame;
    (*object).endFrame = frame;
    (*object).owner = owner;
    //object->damage = 100;
    //object->splashDamage = 200;
    //object->splashRadius = 200;
    //object->methodOfDeath = MOD_EXPLOSIVE;
    //object->splashMethodOfDeath = MOD_EXPLOSIVE_SPLASH;
    (*object).clipmask = MASK_SOLID; //?
    //object->e_TouchFunc = touchF_charge_stick;

    // The effect to play.
    (*object).count = effectID;

    //Give it SOME size for now
    VectorSet_inline(&mut (*object).mins, -4.0, -4.0, -4.0);
    VectorSet_inline(&mut (*object).maxs, 4.0, 4.0, 4.0);

    //Origin
    G_SetOrigin(object, origin);
    (*object).s.pos.trType = trType;
    VectorCopy_inline(origin, &mut (*object).s.pos.trBase);
    //Velocity
    VectorClear_inline(&mut (*object).s.pos.trDelta);
    (*object).s.pos.trTime = level.time;
    //VectorScale( dir, 300, object->s.pos.trDelta );
    //object->s.pos.trTime = level.time;

    //Angles
    VectorCopy_inline(angles, &mut (*object).s.angles);
    VectorCopy_inline((*object).s.angles, &mut (*object).s.apos.trBase);
    //Angular Velocity
    VectorClear_inline(&mut (*object).s.apos.trDelta);
    (*object).s.apos.trTime = level.time;
    //VectorSet( object->s.apos.trDelta, 300, 0, 0 );
    //object->s.apos.trTime = level.time;

    if let Some(linkent_fn) = gi.linkentity {
        linkent_fn(object);
    }

    return object;
}

// Entity type constants
pub const ET_GENERAL: c_int = 0;

// Think function indices (from g_functions.rs)
pub const thinkF_NULL: c_int = 0;
pub const thinkF_G_RunObject: c_int = 7;
