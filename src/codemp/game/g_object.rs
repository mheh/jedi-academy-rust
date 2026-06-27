//! Port of `g_object.c` — physics for free-moving "objects" (thrown/dropped props that
//! arc under gravity, bounce, and come to rest on slopes).
//!
//! Landed incrementally: only the helpers whose deps are already ported. The
//! trajectory helpers below are pure state mutation over a `gentity_t` (no oracle —
//! same convention as [`crate::codemp::game::g_utils::G_SetOrigin`]). `G_RunObject`
//! is the per-frame think driving them through the engine trace/link calls.

#![allow(non_snake_case)] // C function names (`G_BounceObject`, …) kept verbatim
#![allow(non_upper_case_globals)] // C macro names kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::bg_misc::{BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta};
use crate::codemp::game::bg_weapons_h::WP_SABER;
use crate::codemp::game::g_active::{DoImpact, G_MoverTouchPushTriggers};
use crate::codemp::game::g_local::{
    gentity_t, FL_BOUNCE, FL_BOUNCE_HALF, FRAMETIME,
};
use crate::codemp::game::g_main::{g_entities, g_gravity, level};
use crate::codemp::game::npc::pitch_roll_for_slope;
use crate::codemp::game::q_math::{
    flrand, DotProduct, VectorClear, VectorCompare, VectorCopy, VectorMA, VectorNormalize,
    VectorScale,
};
use crate::codemp::game::q_shared_h::{
    trace_t, trType_t, vec3_t, TR_GRAVITY, TR_LINEAR, TR_STATIONARY,
};
use crate::ffi::types::QTRUE;
use crate::trap;

/// `void G_BounceObject( gentity_t *ent, trace_t *trace )` (g_object.c:14). Reflect the
/// object's velocity off the surface it just hit, writing the new `s.pos.trDelta`/origin
/// trajectory. `FL_BOUNCE_HALF` objects keep half their speed and come to rest once they
/// settle onto a near-flat surface with little vertical speed (sign-flipped under inverted
/// gravity). No oracle (mutates a `gentity_t`).
///
/// # Safety
/// `ent` must point to a valid `gentity_t`; `trace` to a valid `trace_t`.
pub unsafe fn G_BounceObject(ent: *mut gentity_t, trace: *mut trace_t) {
    let mut velocity: vec3_t = [0.0; 3];

    // reflect the velocity on the trace plane
    let hitTime = ((*addr_of!(level)).previousTime as f32
        + ((*addr_of!(level)).time - (*addr_of!(level)).previousTime) as f32 * (*trace).fraction)
        as c_int;
    BG_EvaluateTrajectoryDelta(&(*ent).s.pos, hitTime, &mut velocity);
    let dot = DotProduct(&velocity, &(*trace).plane.normal);
    //	bounceFactor = 60/ent->mass;		// NOTENOTE Mass is not yet implemented
    let mut bounceFactor: f32 = 1.0;
    if bounceFactor > 1.0 {
        bounceFactor = 1.0;
    }
    VectorMA(
        &velocity,
        -2.0 * dot * bounceFactor,
        &(*trace).plane.normal,
        &mut (*ent).s.pos.trDelta,
    );

    //FIXME: customized or material-based impact/bounce sounds
    if (*ent).flags & FL_BOUNCE_HALF != 0 {
        let trDelta = (*ent).s.pos.trDelta;
        VectorScale(&trDelta, 0.5, &mut (*ent).s.pos.trDelta);

        // check for stop
        let gravity = (*addr_of!(g_gravity)).value;
        if (((*trace).plane.normal[2] > 0.7 && gravity > 0.0)
            || ((*trace).plane.normal[2] < -0.7 && gravity < 0.0))
            && (((*ent).s.pos.trDelta[2] < 40.0 && gravity > 0.0)
                || ((*ent).s.pos.trDelta[2] > -40.0 && gravity < 0.0))
        {
            // this can happen even on very slightly sloped walls, so changed it from > 0 to > 0.7
            //G_SetOrigin( ent, trace->endpos );
            //ent->nextthink = level.time + 500;
            (*ent).s.apos.trType = TR_STATIONARY;
            VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
            VectorCopy(&(*trace).endpos, &mut (*ent).r.currentOrigin);
            VectorCopy(&(*trace).endpos, &mut (*ent).s.pos.trBase);
            (*ent).s.pos.trTime = (*addr_of!(level)).time;
            return;
        }
    }

    // NEW--It would seem that we want to set our trBase to the trace endpos
    //	and set the trTime to the actual time of impact....
    //	FIXME: Should we still consider adding the normal though??
    VectorCopy(&(*trace).endpos, &mut (*ent).r.currentOrigin);
    (*ent).s.pos.trTime = hitTime;

    let currentOrigin = (*ent).r.currentOrigin;
    VectorCopy(&currentOrigin, &mut (*ent).s.pos.trBase);
    VectorCopy(&(*trace).plane.normal, &mut (*ent).pos1); //???
}

/// `void G_RunObject( gentity_t *ent )` (g_object.c:72). Per-frame `think` for a free-moving
/// object: advance its position/angle trajectories, trace the move, fire push triggers, apply
/// impact damage and bounce/settle physics, and call the entity's `touch`. No oracle
/// (entity-state mutation driven by engine `trap_Trace`/`trap_LinkEntity` and the `touch`
/// callback).
///
/// TODO:  When transition to 0 grav, push away from surface you were resting on
/// TODO:  When free-floating in air, apply some friction to your trDelta (based on mass?)
///
/// # Safety
/// `ent` must point to a valid `gentity_t` with a valid `touch` callback installed.
pub unsafe extern "C" fn G_RunObject(ent: *mut gentity_t) {
    let mut origin: vec3_t = [0.0; 3];
    let mut oldOrg: vec3_t = [0.0; 3];

    //FIXME: floaters need to stop floating up after a while, even if gravity stays negative?
    if (*ent).s.pos.trType == TR_STATIONARY {
        //g_gravity.value <= 0 &&
        (*ent).s.pos.trType = TR_GRAVITY;
        VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.pos.trBase);
        (*ent).s.pos.trTime = (*addr_of!(level)).previousTime; //?necc?
        if (*addr_of!(g_gravity)).value == 0.0 {
            (*ent).s.pos.trDelta[2] += 100.0;
        }
    }

    (*ent).nextthink = (*addr_of!(level)).time + FRAMETIME;

    VectorCopy(&(*ent).r.currentOrigin, &mut oldOrg);
    // get current position
    BG_EvaluateTrajectory(&(*ent).s.pos, (*addr_of!(level)).time, &mut origin);
    //Get current angles?
    BG_EvaluateTrajectory(
        &(*ent).s.apos,
        (*addr_of!(level)).time,
        &mut (*ent).r.currentAngles,
    );

    if VectorCompare(&(*ent).r.currentOrigin, &origin) != 0 {
        //error - didn't move at all!
        return;
    }
    // trace a line from the previous position to the current position,
    // ignoring interactions with the missile owner
    let mut tr = trap::Trace(
        &(*ent).r.currentOrigin,
        &(*ent).r.mins,
        &(*ent).r.maxs,
        &origin,
        if !(*ent).parent.is_null() {
            (*(*ent).parent).s.number
        } else {
            (*ent).s.number
        },
        (*ent).clipmask,
    );

    if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction != 0.0 {
        VectorCopy(&tr.endpos, &mut (*ent).r.currentOrigin);
        trap::LinkEntity(ent);
    } else
    //if ( tr.startsolid )
    {
        tr.fraction = 0.0;
    }

    G_MoverTouchPushTriggers(ent, &oldOrg);
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
        if (*addr_of!(g_gravity)).value <= 0.0 {
            if (*ent).s.apos.trType == TR_STATIONARY {
                VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
                (*ent).s.apos.trType = TR_LINEAR;
                (*ent).s.apos.trDelta[1] = flrand(-300.0, 300.0);
                (*ent).s.apos.trDelta[0] = flrand(-10.0, 10.0);
                (*ent).s.apos.trDelta[2] = flrand(-10.0, 10.0);
                (*ent).s.apos.trTime = (*addr_of!(level)).time;
            }
        }
        //friction in zero-G
        if (*addr_of!(g_gravity)).value == 0.0 {
            let mut friction: f32 = 0.975;
            //friction -= ent->mass/1000.0f;
            if friction < 0.1 {
                friction = 0.1;
            }

            let trDelta = (*ent).s.pos.trDelta;
            VectorScale(&trDelta, friction, &mut (*ent).s.pos.trDelta);
            VectorCopy(&(*ent).r.currentOrigin, &mut (*ent).s.pos.trBase);
            (*ent).s.pos.trTime = (*addr_of!(level)).time;
        }
        return;
    }

    //hit something

    //Do impact damage
    let traceEnt = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize);
    if tr.fraction != 0.0 || (!traceEnt.is_null() && (*traceEnt).takedamage != 0) {
        if VectorCompare(&(*ent).r.currentOrigin, &oldOrg) == 0 {
            //moved and impacted
            if !traceEnt.is_null() && (*traceEnt).takedamage != 0 {
                //hurt someone
                //				G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHurt.wav" ) );
            }
            //			G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHit.wav" ) );
        }

        if (*ent).s.weapon != WP_SABER {
            DoImpact(ent, traceEnt, QTRUE);
        }
    }

    if ent.is_null() || ((*ent).takedamage != 0 && (*ent).health <= 0) {
        //been destroyed by impact
        //chunks?
        //		G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectBreak.wav" ) );
        return;
    }

    //do impact physics
    if (*ent).s.pos.trType == TR_GRAVITY {
        //tr.fraction < 1.0 &&
        //FIXME: only do this if no trDelta
        if (*addr_of!(g_gravity)).value <= 0.0 || tr.plane.normal[2] < 0.7 {
            if (*ent).flags & (FL_BOUNCE | FL_BOUNCE_HALF) != 0 {
                if tr.fraction <= 0.0 {
                    VectorCopy(&tr.endpos, &mut (*ent).r.currentOrigin);
                    VectorCopy(&tr.endpos, &mut (*ent).s.pos.trBase);
                    VectorClear(&mut (*ent).s.pos.trDelta);
                    (*ent).s.pos.trTime = (*addr_of!(level)).time;
                } else {
                    G_BounceObject(ent, &mut tr);
                }
            } else {
                //slide down?
                //FIXME: slide off the slope
            }
        } else {
            (*ent).s.apos.trType = TR_STATIONARY;
            pitch_roll_for_slope(ent, &tr.plane.normal as *const vec3_t);
            //ent->r.currentAngles[0] = 0;//FIXME: match to slope
            //ent->r.currentAngles[2] = 0;//FIXME: match to slope
            VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
            //okay, we hit the floor, might as well stop or prediction will
            //make us go through the floor!
            //FIXME: this means we can't fall if something is pulled out from under us...
            G_StopObjectMoving(ent);
        }
    } else if (*ent).s.weapon != WP_SABER {
        (*ent).s.apos.trType = TR_STATIONARY;
        pitch_roll_for_slope(ent, &tr.plane.normal as *const vec3_t);
        //ent->r.currentAngles[0] = 0;//FIXME: match to slope
        //ent->r.currentAngles[2] = 0;//FIXME: match to slope
        VectorCopy(&(*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
    }

    //call touch func
    if let Some(touch) = (*ent).touch {
        touch(
            ent,
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(tr.entityNum as usize),
            &mut tr,
        );
    }
}

/// `void G_StopObjectMoving( gentity_t *object )` (g_object.c:244). Freeze the object in
/// place: switch its position trajectory to `TR_STATIONARY`, snap the trajectory base to the
/// current origin, and zero the velocity delta. No oracle (mutates a `gentity_t`).
///
/// # Safety
/// `object` must point to a valid `gentity_t`.
pub unsafe fn G_StopObjectMoving(object: *mut gentity_t) {
    (*object).s.pos.trType = TR_STATIONARY;
    VectorCopy(&(*object).r.currentOrigin, &mut (*object).s.origin);
    VectorCopy(&(*object).r.currentOrigin, &mut (*object).s.pos.trBase);
    VectorClear(&mut (*object).s.pos.trDelta);

    /*
    //Stop spinning
    VectorClear( self->s.apos.trDelta );
    vectoangles(trace->plane.normal, self->s.angles);
    VectorCopy(self->s.angles, self->r.currentAngles );
    VectorCopy(self->s.angles, self->s.apos.trBase);
    */
}

/// `void G_StartObjectMoving( gentity_t *object, vec3_t dir, float speed, trType_t trType )`
/// (g_object.c:260). Launch a free-moving object: normalize `dir`, set its position
/// trajectory type/base/delta and start time, and — if it has no `think` yet — schedule it to
/// run through [`G_RunObject`] next frame. No oracle (mutates a `gentity_t`).
///
/// # Safety
/// `object` must point to a valid `gentity_t`.
pub unsafe fn G_StartObjectMoving(
    object: *mut gentity_t,
    dir: &mut vec3_t,
    speed: f32,
    trType: trType_t,
) {
    VectorNormalize(dir);

    //object->s.eType = ET_GENERAL;
    (*object).s.pos.trType = trType;
    VectorCopy(&(*object).r.currentOrigin, &mut (*object).s.pos.trBase);
    VectorScale(dir, speed, &mut (*object).s.pos.trDelta);
    (*object).s.pos.trTime = (*addr_of!(level)).time;

    /*
    //FIXME: incorporate spin?
    vectoangles(dir, object->s.angles);
    VectorCopy(object->s.angles, object->s.apos.trBase);
    VectorSet(object->s.apos.trDelta, 300, 0, 0 );
    object->s.apos.trTime = level.time;
    */

    //FIXME: make these objects go through G_RunObject automatically, like missiles do
    if (*object).think.is_none() {
        (*object).nextthink = (*addr_of!(level)).time + FRAMETIME;
        (*object).think = Some(G_RunObject);
    } else {
        //You're responsible for calling RunObject
    }
}
