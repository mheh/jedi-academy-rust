// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_float, c_char, c_void};

extern "C" {
    pub fn G_EntIsUnlockedDoor(entityNum: c_int) -> c_int;
    pub fn FlyingCreature(ent: *mut gentity_t) -> c_int;
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorAdd(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorSet(v: *mut vec3_t, x: c_float, y: c_float, z: c_float);
    pub fn VectorScale(vin: *const vec3_t, scale: c_float, vout: *mut vec3_t);
    pub fn VectorLengthSquared(vec: *const vec3_t) -> c_float;
    pub fn DistanceSquared(a: *const vec3_t, b: *const vec3_t) -> c_float;
    pub fn Distance(a: *const vec3_t, b: *const vec3_t) -> c_float;
    pub fn G_BoundsOverlap(pmins: *const vec3_t, pmaxs: *const vec3_t, dmins: *const vec3_t, dmaxs: *const vec3_t) -> c_int;
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorMA(veca: *const vec3_t, scale: c_float, vecb: *const vec3_t, vecc: *mut vec3_t);
    pub fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    pub fn VectorNormalize(vec: *mut vec3_t) -> c_float;
    pub fn VectorCompare(v1: *const vec3_t, v2: *const vec3_t) -> c_int;

    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut gNPC_t;
    pub static mut g_entities: *mut gentity_t;
}

// Stub types for external dependencies
#[repr(C)]
pub struct gentity_t {
    pub mins: [c_float; 3],
    pub maxs: [c_float; 3],
    pub currentOrigin: [c_float; 3],
    pub currentAngles: [c_float; 3],
    pub s: entity_state_t,
    pub contents: c_int,
    pub clipmask: c_int,
    pub classname: *const c_char,
    pub client: *mut gclient_t,
    pub NPC: *mut gNPC_t,
}

#[repr(C)]
pub struct entity_state_t {
    pub number: c_int,
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
}

#[repr(C)]
pub struct playerState_t {
    pub speed: c_float,
    pub moveDir: [c_float; 3],
}

#[repr(C)]
pub struct gNPC_t {
    pub goalRadius: c_float,
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: c_float,
    pub endpos: [c_float; 3],
    pub contents: c_int,
    pub entityNum: c_int,
}

pub type vec3_t = [c_float; 3];

// Engine API - gi.trace
extern "C" {
    pub fn gi_trace(
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passent: c_int,
        contentmask: c_int,
    );
}

#[inline]
fn gi_trace_wrapper(
    results: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    passent: c_int,
    contentmask: c_int,
) {
    unsafe {
        gi_trace(results, start, mins, maxs, end, passent, contentmask);
    }
}

#[inline]
fn VALIDSTRING(a: *const c_char) -> bool {
    if a.is_null() {
        return false;
    }
    unsafe { *a != 0 }
}

const MIN_DOOR_BLOCK_DIST: c_int = 16;
const MIN_DOOR_BLOCK_DIST_SQR: c_int = MIN_DOOR_BLOCK_DIST * MIN_DOOR_BLOCK_DIST;
const STEPSIZE: c_float = 18.0;
const CONTENTS_BOTCLIP: c_int = 16384;
const SVF_NAVGOAL: c_int = 0x0020;
const ENTITYNUM_WORLD: c_int = 2047;
const YAW: c_int = 1;
const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

/*
-------------------------
NAV_HitNavGoal
-------------------------
*/

pub unsafe fn NAV_HitNavGoal(
    point: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    dest: *const vec3_t,
    radius: c_int,
    flying: c_int,
) -> c_int {
    let mut dmins: vec3_t = [0.0; 3];
    let mut dmaxs: vec3_t = [0.0; 3];
    let mut pmins: vec3_t = [0.0; 3];
    let mut pmaxs: vec3_t = [0.0; 3];

    if radius != 0 {
        let _ = radius;
        //NOTE:  This needs to do a DistanceSquared on navgoals that had
        //			a radius manually set! We can't do the smaller navgoals against
        //			walls to get around this because player-sized traces to them
        //			from angles will not work... - MCG
        if flying == 0 {
            //Allow for a little z difference
            let mut diff: vec3_t = [0.0; 3];
            VectorSubtract(point, dest, &mut diff);
            if libm::fabsf(diff[2]) <= 24.0 {
                diff[2] = 0.0;
            }
            return if VectorLengthSquared(&diff) <= ((radius as c_float) * (radius as c_float)) {
                1
            } else {
                0
            };
        } else {
            //must hit exactly
            return if DistanceSquared(dest, point) <= ((radius as c_float) * (radius as c_float)) {
                1
            } else {
                0
            };
        }
        //There is probably a better way to do this, either by preserving the original
        //		mins and maxs of the navgoal and doing this check ONLY if the radius
        //		is non-zero (like the original implementation) or some boolean to
        //		tell us to do this check rather than the fake bbox overlap check...
    } else {
        //Construct a dummy bounding box from our radius value
        VectorSet(&mut dmins, -(radius as c_float), -(radius as c_float), -(radius as c_float));
        VectorSet(&mut dmaxs, radius as c_float, radius as c_float, radius as c_float);

        //Translate it
        VectorAdd(&dmins, dest, &mut dmins);
        VectorAdd(&dmaxs, dest, &mut dmaxs);

        //Translate the starting box
        VectorAdd(point, mins, &mut pmins);
        VectorAdd(point, maxs, &mut pmaxs);

        //See if they overlap
        return G_BoundsOverlap(&pmins, &pmaxs, &dmins, &dmaxs);
    }
}

/*
-------------------------
NAV_CheckAhead
-------------------------
*/

pub unsafe fn NAV_CheckAhead(
    self_: *mut gentity_t,
    end: *const vec3_t,
    trace: *mut trace_t,
    clipmask: c_int,
) -> c_int {
    let mut mins: vec3_t = [0.0; 3];

    //Offset the step height
    VectorSet(
        &mut mins,
        (*self_).mins[0],
        (*self_).mins[1],
        (*self_).mins[2] + STEPSIZE,
    );

    gi_trace(trace, &(*self_).currentOrigin, &mins, &(*self_).maxs, end, (*self_).s.number, clipmask);

    if (*trace).startsolid != 0 && ((*trace).contents & CONTENTS_BOTCLIP) != 0 {
        //started inside do not enter, so ignore them
        let new_clipmask = clipmask & !CONTENTS_BOTCLIP;
        gi_trace(trace, &(*self_).currentOrigin, &mins, &(*self_).maxs, end, (*self_).s.number, new_clipmask);
    }
    //Do a simple check
    if ((*trace).allsolid == 0) && ((*trace).startsolid == 0) && ((*trace).fraction == 1.0) {
        return 1;
    }

    //See if we're too far above
    if libm::fabsf((*self_).currentOrigin[2] - (*end)[2]) > 48.0 {
        return 0;
    }

    //This is a work around
    let radius = if (*self_).maxs[0] > (*self_).maxs[1] {
        (*self_).maxs[0]
    } else {
        (*self_).maxs[1]
    };
    let dist = Distance(&(*self_).currentOrigin, end);
    let tFrac = 1.0 - (radius / dist);

    if (*trace).fraction >= tFrac {
        return 1;
    }

    //Do a special check for doors
    if (*trace).entityNum < ENTITYNUM_WORLD {
        let blocker = &mut (*g_entities.offset((*trace).entityNum as isize));

        if VALIDSTRING(blocker.classname) {
            if G_EntIsUnlockedDoor(blocker.s.number) != 0 {
                //if ( Q_stricmp( blocker->classname, "func_door" ) == 0 )
                //We're too close, try and avoid the door (most likely stuck on a lip)
                if DistanceSquared(&(*self_).currentOrigin, &(*trace).endpos) < (MIN_DOOR_BLOCK_DIST_SQR as c_float) {
                    return 0;
                }

                return 1;
            }
        }
    }

    return 0;
}

/*
-------------------------
NPC_ClearPathToGoal
-------------------------
*/

pub unsafe fn NPC_ClearPathToGoal(dir: *mut vec3_t, goal: *mut gentity_t) -> c_int {
    let mut trace: trace_t = core::mem::zeroed();

    //FIXME: What does do about area portals?  THIS IS BROKEN
    //if ( gi.inPVS( NPC->currentOrigin, goal->currentOrigin ) == qfalse )
    //	return qfalse;

    //Look ahead and see if we're clear to move to our goal position
    if NAV_CheckAhead(
        NPC,
        &(*goal).currentOrigin,
        &mut trace,
        ((*NPC).clipmask & !32) | CONTENTS_BOTCLIP,
    ) != 0
    {
        //VectorSubtract( goal->currentOrigin, NPC->currentOrigin, dir );
        return 1;
    }

    if FlyingCreature(NPC) == 0 {
        //See if we're too far above
        if libm::fabsf((*NPC).currentOrigin[2] - (*goal).currentOrigin[2]) > 48.0 {
            return 0;
        }
    }

    //This is a work around
    let radius = if (*NPC).maxs[0] > (*NPC).maxs[1] {
        (*NPC).maxs[0]
    } else {
        (*NPC).maxs[1]
    };
    let dist = Distance(&(*NPC).currentOrigin, &(*goal).currentOrigin);
    let tFrac = 1.0 - (radius / dist);

    if trace.fraction >= tFrac {
        return 1;
    }

    //See if we're looking for a navgoal
    if ((*goal).s.number & SVF_NAVGOAL) != 0 {
        //Okay, didn't get all the way there, let's see if we got close enough:
        if NAV_HitNavGoal(
            &trace.endpos,
            &(*NPC).mins,
            &(*NPC).maxs,
            &(*goal).currentOrigin,
            (*NPCInfo).goalRadius as c_int,
            FlyingCreature(NPC),
        ) != 0
        {
            //VectorSubtract(goal->currentOrigin, NPC->currentOrigin, dir);
            return 1;
        }
    }

    return 0;
}

pub unsafe fn NAV_DirSafe(self_: *mut gentity_t, dir: *const vec3_t, dist: c_float) -> c_int {
    let mut mins: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut trace: trace_t = core::mem::zeroed();

    VectorMA(&(*self_).currentOrigin, dist, dir, &mut end);

    //Offset the step height
    VectorSet(
        &mut mins,
        (*self_).mins[0],
        (*self_).mins[1],
        (*self_).mins[2] + STEPSIZE,
    );

    gi_trace(&mut trace, &(*self_).currentOrigin, &mins, &(*self_).maxs, &end, (*self_).s.number, CONTENTS_BOTCLIP);

    //Do a simple check
    if (trace.allsolid == 0) && (trace.startsolid == 0) && (trace.fraction == 1.0) {
        return 1;
    }

    return 0;
}

pub unsafe fn NAV_MoveDirSafe(self_: *mut gentity_t, cmd: *mut usercmd_t, distScale: c_float) -> c_int {
    let mut moveDir: vec3_t = [0.0; 3];

    if self_.is_null() || (*self_).client.is_null() {
        return 1;
    }
    if (*(*self_).client).ps.speed == 0.0 {
        return 1;
    }
    if FlyingCreature(self_) != 0 {
        return 1;
    }
    if VectorCompare(&(*(*self_).client).ps.moveDir, &vec3_origin) != 0 {
        //no movedir, build from cmd
        if (*cmd).forwardmove == 0 && (*cmd).rightmove == 0 {
            //not moving at all
            return 1;
        }
        let mut fwd: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut fwdAngs: vec3_t = [0.0, (*self_).currentAngles[YAW as usize], 0.0];
        AngleVectors(&fwdAngs, &mut fwd, &mut right, core::ptr::null_mut());
        VectorScale(&fwd, (*cmd).forwardmove as c_float, &mut fwd);
        VectorScale(&right, (*cmd).rightmove as c_float, &mut right);
        VectorAdd(&fwd, &right, &mut moveDir);
        VectorNormalize(&mut moveDir);
    } else {
        VectorCopy(&(*(*self_).client).ps.moveDir, &mut moveDir);
    }
    return NAV_DirSafe(self_, &moveDir, ((*(*self_).client).ps.speed / 10.0) * distScale);
}

// Stub types for dependencies that may need to be defined elsewhere
#[repr(C)]
pub struct usercmd_t {
    pub forwardmove: c_int,
    pub rightmove: c_int,
}

// External libm function for fabs (using Rust's libm)
mod libm {
    pub fn fabsf(x: f32) -> f32 {
        x.abs()
    }
}
