// this include must remain at the top of every bg_xxxx file
// Equivalent to: #include "common_headers.h"
// Equivalent to: #include "q_shared.h"
// Equivalent to: #include "bg_public.h"
// Equivalent to: #include "bg_local.h"
// Equivalent to: #include "g_vehicles.h"

use core::ffi::{c_int, c_void};

// Local stub types for structural coherence
// These should be defined in other modules; kept here for mechanical translation

type vec3_t = [f32; 3];
type qboolean = c_int;

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub endpos: vec3_t,
    pub plane: cplane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub _type: c_int,
    pub signbits: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub clientNum: c_int,
    pub velocity: vec3_t,
    pub origin: vec3_t,
    pub gravity: c_int,
    pub groundEntityNum: c_int,
    pub legsAnim: c_int,
    pub eFlags: c_int,
    pub pm_flags: c_int,
    pub pm_time: c_int,
}

#[repr(C)]
pub struct gclient_t {
    pub NPC_class: c_int,
    pub playerTeam: c_int,
}

#[repr(C)]
pub struct vehicleInfo_t {
    pub maxSlope: f32,
    pub hoverHeight: f32,
}

#[repr(C)]
pub struct vehicle_t {
    pub m_pPilot: *mut gentity_t,
    pub m_pVehicleInfo: *mut vehicleInfo_t,
}

#[repr(C)]
pub struct gentity_t {
    pub client: *mut gclient_t,
    pub m_pVehicle: *mut vehicle_t,
    pub forcePushTime: c_int,
    pub spawnflags: c_int,
}

#[repr(C)]
pub struct pmove_local_t {
    pub walking: c_int,
    pub frametime: f32,
    pub groundPlane: c_int,
    pub groundTrace: trace_t,
    pub impactSpeed: f32,
}

#[repr(C)]
pub struct pmove_t {
    pub ps: *mut playerState_t,
    pub tracemask: c_int,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub clientNum: c_int,
    pub trace: extern "C" fn(*mut trace_t, *const vec3_t, *const vec3_t, *const vec3_t, *const vec3_t, c_int, c_int),
    pub debugLevel: c_int,
    pub gent: *mut gentity_t,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

// External function declarations
extern "C" {
    pub fn PM_ClientImpact(trace: *mut trace_t, damageSelf: qboolean) -> qboolean;
    pub fn PM_ControlledByPlayer() -> qboolean;
    pub fn PM_InReboundHold(anim: c_int) -> qboolean;
    pub fn PM_GroundSlideOkay(zNormal: f32) -> qboolean;
    pub fn PM_InSpecialJump(anim: c_int) -> qboolean;
    pub fn PM_ClipVelocity(in_: *mut vec3_t, normal: *const vec3_t, out: *mut vec3_t, overbounce: f32);
    pub fn PM_AddTouchEnt(entityNum: c_int);
    pub fn PM_AddEvent(event: c_int);
    pub fn Com_Printf(fmt: *const i8, ...);
    pub fn G_DebugLine(from: *const vec3_t, to: *const vec3_t, time: c_int, color: c_int, alphaFade: c_int);
    pub fn VectorCopy(in_: *const vec3_t, out: *mut vec3_t);
    pub fn VectorMA(v: *const vec3_t, scale: f32, dir: *const vec3_t, out: *mut vec3_t);
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn VectorNormalize2(in_: *const vec3_t, out: *mut vec3_t) -> f32;
    pub fn VectorAdd(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t);
    pub fn VectorScale(in_: *const vec3_t, scale: f32, out: *mut vec3_t);
    pub fn VectorClear(v: *mut vec3_t);
    pub fn VectorSubtract(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t);
    pub fn VectorSet(v: *mut vec3_t, x: f32, y: f32, z: f32);
    pub fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32;
    pub fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t);
    pub fn VectorLengthSquared(v: *const vec3_t) -> f32;

    // Global variables
    pub static mut pm: *mut pmove_t;
    pub static mut pml: pmove_local_t;
    pub static g_stepSlideFix: *const cvar_t;
    pub static mut g_entities: *mut gentity_t;
    pub static mut c_pmove: c_int;
}

// Constants
const MAX_CLIP_PLANES: usize = 5;
const MAX_CLIENTS: c_int = 64;
const ENTITYNUM_NONE: c_int = 1023;
const ENTITYNUM_WORLD: c_int = 1022;
const STEPSIZE: c_int = 18;
const OVERCLIP: f32 = 1.001;
const MIN_WALK_NORMAL: f32 = 0.7;
const EV_STEP_4: c_int = 0;
const EV_STEP_8: c_int = 1;
const EV_STEP_12: c_int = 2;
const EV_STEP_16: c_int = 3;

// Flags
const EF_FORCE_GRIPPED: c_int = 0x00000001;
const EF_FORCE_DRAINED: c_int = 0x00000002;
const PMF_STUCK_TO_WALL: c_int = 0x00000001;
const PMF_BUMPED: c_int = 0x00000002;
const PMF_TIME_KNOCKBACK: c_int = 0x00000004;
const CONTENTS_BOTCLIP: c_int = 0x00040000;

// NPC class constants
const CLASS_VEHICLE: c_int = 1;
const CLASS_ATST: c_int = 2;
const CLASS_RANCOR: c_int = 3;

// Surface flags
const SURF_NODAMAGE: c_int = 0x00040000;

// Macro stubs (these may need to be defined elsewhere)
extern "C" {
    pub fn level_time() -> c_int;
}

/*

input: origin, velocity, bounds, groundPlane, trace function

output: origin, velocity, impacts, stairup boolean

*/

/*
==================
PM_SlideMove

Returns qtrue if the velocity was clipped in some way
==================
*/
pub unsafe extern "C" fn PM_SlideMove(gravMod: f32) -> qboolean {
    let mut bumpcount: c_int;
    let mut numbumps: c_int;
    let mut dir: vec3_t = [0.0; 3];
    let mut d: f32;
    let mut numplanes: c_int;
    let mut normal: vec3_t = [0.0; 3];
    let mut planes: [vec3_t; MAX_CLIP_PLANES] = [[0.0; 3]; MAX_CLIP_PLANES];
    let mut primal_velocity: vec3_t = [0.0; 3];
    let mut clipVelocity: vec3_t = [0.0; 3];
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut trace: trace_t = core::mem::zeroed();
    let mut end: vec3_t = [0.0; 3];
    let mut time_left: f32;
    let mut into: f32;
    let mut endVelocity: vec3_t = [0.0; 3];
    let mut endClipVelocity: vec3_t = [0.0; 3];
    let mut damageSelf: qboolean = 1; // qtrue
    let mut slideMoveContents: c_int = (*pm).tracemask;

    if (*pm).ps as *const c_void as usize != 0 {
        if (*(*pm).ps).clientNum >= MAX_CLIENTS && PM_ControlledByPlayer() == 0
        {//a non-player client, not an NPC under player control
            if pml.walking != 0 //walking on the ground
                || ((*(*pm).ps).groundEntityNum != ENTITYNUM_NONE //in air
                    && PM_InSpecialJump((*(*pm).ps).legsAnim) != 0 //in a special jump
                    && ((*(*pm).ps).eFlags & EF_FORCE_GRIPPED) == 0 //not being gripped
                    && ((*(*pm).ps).pm_flags & PMF_TIME_KNOCKBACK) == 0
                    && (*pm).gent as *const c_void as usize != 0
                    && (*(*(*pm).gent).client).NPC_class != CLASS_VEHICLE
                    && (*(*pm).gent).m_pVehicle as *const c_void as usize != 0
                    && (*(*(*pm).gent).m_pVehicle).m_pPilot as *const c_void as usize != 0
                    && (*(*(*(*pm).gent).m_pVehicle).m_pPilot).clientNum >= MAX_CLIENTS) //not being pushed
            {//
                // If we're a vehicle, ignore this if we're being driven
                if (*pm).gent as *const c_void as usize == 0 //not an game ent
                    || (*(*pm).gent).client as *const c_void as usize == 0 //not a client
                    || (*(*(*pm).gent).client).NPC_class != CLASS_VEHICLE //not a vehicle
                    || (*(*pm).gent).m_pVehicle as *const c_void as usize == 0 //no vehicle
                    || (*(*(*pm).gent).m_pVehicle).m_pPilot as *const c_void as usize == 0 //no pilot
                    || (*(*(*(*pm).gent).m_pVehicle).m_pPilot).clientNum >= MAX_CLIENTS //pilot is not the player
                {//then treat do not enter brushes as SOLID
                    slideMoveContents |= CONTENTS_BOTCLIP;
                }
            }
        }
    }

    numbumps = 4;

    VectorCopy(&(*(*pm).ps).velocity, &mut primal_velocity);

    if gravMod != 0.0
    {
        VectorCopy(&(*(*pm).ps).velocity, &mut endVelocity);
        if ((*(*pm).ps).eFlags & EF_FORCE_GRIPPED) == 0 && ((*(*pm).ps).eFlags & EF_FORCE_DRAINED) == 0
        {
            endVelocity[2] -= ((*(*pm).ps).gravity as f32) * pml.frametime * gravMod;
        }
        (*(*pm).ps).velocity[2] = ((*(*pm).ps).velocity[2] + endVelocity[2]) * 0.5;
        primal_velocity[2] = endVelocity[2];
        if pml.groundPlane != 0
        {
            if PM_GroundSlideOkay(pml.groundTrace.plane.normal[2]) != 0
            {// slide along the ground plane
                PM_ClipVelocity(&mut (*(*pm).ps).velocity, &pml.groundTrace.plane.normal,
                    &mut (*(*pm).ps).velocity, OVERCLIP);
            }
        }
    }

    time_left = pml.frametime;

    // never turn against the ground plane
    if pml.groundPlane != 0
    {
        numplanes = 1;
        VectorCopy(&pml.groundTrace.plane.normal, &mut planes[0]);
        if PM_GroundSlideOkay(planes[0][2]) == 0
        {
            planes[0][2] = 0.0;
            VectorNormalize(&mut planes[0]);
        }
    }
    else
    {
        numplanes = 0;
    }

    // never turn against original velocity
    VectorNormalize2(&(*(*pm).ps).velocity, &mut planes[numplanes as usize]);
    numplanes += 1;

    bumpcount = 0;
    while bumpcount < numbumps {

        // calculate position we are trying to move to
        VectorMA(&(*(*pm).ps).origin, time_left, &(*(*pm).ps).velocity, &mut end);

        // see if we can make it there
        ((*pm).trace)(&mut trace, &(*(*pm).ps).origin, &(*pm).mins, &(*pm).maxs, &end, (*(*pm).ps).clientNum, slideMoveContents);
        if (trace.contents & CONTENTS_BOTCLIP) != 0
            && (slideMoveContents & CONTENTS_BOTCLIP) != 0
        {//hit a do not enter brush
            if trace.allsolid != 0 || trace.startsolid != 0 //inside the botclip
            {//crap, we're in a do not enter brush, take it out for the remainder of the traces and re-trace this one right now without it
                slideMoveContents &= !CONTENTS_BOTCLIP;
                ((*pm).trace)(&mut trace, &(*(*pm).ps).origin, &(*pm).mins, &(*pm).maxs, &end, (*(*pm).ps).clientNum, slideMoveContents);
            }
            else if trace.plane.normal[2] > 0.0
            {//on top of a do not enter brush, it, just redo this one trace without it
                ((*pm).trace)(&mut trace, &(*(*pm).ps).origin, &(*pm).mins, &(*pm).maxs, &end, (*(*pm).ps).clientNum, (slideMoveContents & !CONTENTS_BOTCLIP));
            }
        }

        if trace.allsolid != 0
        {// entity is completely trapped in another solid
            (*(*pm).ps).velocity[2] = 0.0;	// don't build up falling damage, but allow sideways acceleration
            return 1; // qtrue
        }

        if trace.fraction > 0.0
        {// actually covered some distance
            VectorCopy(&trace.endpos, &mut (*(*pm).ps).origin);
        }

        if trace.fraction == 1.0
        {
             break;		// moved the entire distance
        }



        // save entity for contact
        PM_AddTouchEnt(trace.entityNum);

        //Hit it
        if (trace.surfaceFlags & SURF_NODAMAGE) != 0
        {
            damageSelf = 0; // qfalse
        }
        else if trace.entityNum == ENTITYNUM_WORLD && trace.plane.normal[2] > 0.5
        {//if we land on the ground, let falling damage do it's thing itself, otherwise do impact damage
            damageSelf = 0; // qfalse
        }
        else
        {
            damageSelf = 1; // qtrue
        }

        if PM_ClientImpact(&mut trace, damageSelf) != 0
        {
            bumpcount += 1;
            continue;
        }

        if !(*pm).gent.is_null()
            && !(*(*pm).gent).client.is_null()
            && (*(*(*pm).gent).client).NPC_class == CLASS_VEHICLE
            && trace.plane.normal[2] < (*(*(*pm).gent).m_pVehicle).m_pVehicleInfo as *const c_void as usize as f32 // FIXME: this is wrong, need to properly dereference
        {
            (*(*pm).ps).pm_flags |= PMF_BUMPED;
        }

        time_left -= time_left * trace.fraction;

        if numplanes >= MAX_CLIP_PLANES as c_int
        {// this shouldn't really happen
            VectorClear(&mut (*(*pm).ps).velocity);
            return 1; // qtrue
        }

        VectorCopy(&trace.plane.normal, &mut normal);

         if PM_GroundSlideOkay(normal[2]) == 0
        {//wall-running
            //never push up off a sloped wall
            normal[2] = 0.0;
            VectorNormalize(&mut normal);
        }

        //
        // if this is the same plane we hit before, nudge velocity
        // out along it, which fixes some epsilon issues with
        // non-axial planes
        //
        if ((*(*pm).ps).pm_flags & PMF_STUCK_TO_WALL) == 0
        {//no sliding if stuck to wall!
            i = 0;
            while i < numplanes {
                if DotProduct(&normal, &planes[i as usize]) > 0.99 {
                    VectorAdd(&normal, &(*(*pm).ps).velocity, &mut (*(*pm).ps).velocity);
                    break;
                }
                i += 1;
            }
            if i < numplanes {
                bumpcount += 1;
                continue;
            }
        }
        VectorCopy(&normal, &mut planes[numplanes as usize]);
        numplanes += 1;

        //
        // modify velocity so it parallels all of the clip planes
        //

        // find a plane that it enters
        i = 0;
        while i < numplanes {
            into = DotProduct(&(*(*pm).ps).velocity, &planes[i as usize]);
            if into >= 0.1 {
                i += 1;
                continue;		// move doesn't interact with the plane
            }

            // see how hard we are hitting things
            if -into > pml.impactSpeed {
                pml.impactSpeed = -into;
            }

            // slide along the plane
            PM_ClipVelocity(&mut (*(*pm).ps).velocity, &planes[i as usize], &mut clipVelocity, OVERCLIP);

            // slide along the plane
            PM_ClipVelocity(&mut endVelocity, &planes[i as usize], &mut endClipVelocity, OVERCLIP);

            // see if there is a second plane that the new move enters
            j = 0;
            while j < numplanes {
                if j == i {
                    j += 1;
                    continue;
                }
                if DotProduct(&clipVelocity, &planes[j as usize]) >= 0.1 {
                    j += 1;
                    continue;		// move doesn't interact with the plane
                }

                // try clipping the move to the plane
                PM_ClipVelocity(&mut clipVelocity, &planes[j as usize], &mut clipVelocity, OVERCLIP);
                PM_ClipVelocity(&mut endClipVelocity, &planes[j as usize], &mut endClipVelocity, OVERCLIP);

                // see if it goes back into the first clip plane
                if DotProduct(&clipVelocity, &planes[i as usize]) >= 0.0 {
                    j += 1;
                    continue;
                }

                // slide the original velocity along the crease
                CrossProduct(&planes[i as usize], &planes[j as usize], &mut dir);
                VectorNormalize(&mut dir);
                d = DotProduct(&dir, &(*(*pm).ps).velocity);
                VectorScale(&dir, d, &mut clipVelocity);

                CrossProduct(&planes[i as usize], &planes[j as usize], &mut dir);
                VectorNormalize(&mut dir);
                d = DotProduct(&dir, &endVelocity);
                VectorScale(&dir, d, &mut endClipVelocity);

                // see if there is a third plane the the new move enters
                k = 0;
                while k < numplanes {
                    if k == i || k == j {
                        k += 1;
                        continue;
                    }
                    if DotProduct(&clipVelocity, &planes[k as usize]) >= 0.1 {
                        k += 1;
                        continue;		// move doesn't interact with the plane
                    }

                    // stop dead at a triple plane interaction
                    VectorClear(&mut (*(*pm).ps).velocity);
                    return 1; // qtrue
                }
                j += 1;
            }

            // if we have fixed all interactions, try another move
            VectorCopy(&clipVelocity, &mut (*(*pm).ps).velocity);
            VectorCopy(&endClipVelocity, &mut endVelocity);
            break;
            i += 1;
        }

        bumpcount += 1;
    }

    if gravMod != 0.0 {
        VectorCopy(&endVelocity, &mut (*(*pm).ps).velocity);
    }

    // don't change velocity if in a timer (FIXME: is this correct?)
    if (*(*pm).ps).pm_time != 0 {
        VectorCopy(&primal_velocity, &mut (*(*pm).ps).velocity);
    }

    return (if bumpcount != 0 { 1 } else { 0 }); // qtrue : qfalse
}

/*
==================
PM_StepSlideMove

==================
*/
pub unsafe extern "C" fn PM_StepSlideMove(gravMod: f32)
{
    let mut start_o: vec3_t = [0.0; 3];
    let mut start_v: vec3_t = [0.0; 3];
    let mut down_o: vec3_t = [0.0; 3];
    let mut down_v: vec3_t = [0.0; 3];
    let mut slideMove: vec3_t = [0.0; 3];
    let mut stepUpMove: vec3_t = [0.0; 3];
    let mut trace: trace_t = core::mem::zeroed();
    let mut up: vec3_t = [0.0; 3];
    let mut down: vec3_t = [0.0; 3];
    let mut cantStepUpFwd: qboolean;
    let mut isGiant: qboolean = 0; // qfalse
    let mut stepSize: c_int = STEPSIZE;

    VectorCopy(&(*(*pm).ps).origin, &mut start_o);
    VectorCopy(&(*(*pm).ps).velocity, &mut start_v);

    if PM_InReboundHold((*(*pm).ps).legsAnim) != 0
    {
        // gravMod = 0.0f; -- gravMod is reassigned, marked mut
    }

    if PM_SlideMove(gravMod) == 0 {
        return;		// we got exactly where we wanted to go first try
    }//else Bumped into something, see if we can step over it

    if !(*pm).gent.is_null()
        && !(*(*pm).gent).client.is_null()
        && (*(*(*pm).gent).client).NPC_class == CLASS_VEHICLE
        && !(*(*pm).gent).m_pVehicle.is_null()
        && (*(*(*pm).gent).m_pVehicle).m_pVehicleInfo as *const c_void as usize != 0
    {//Hovering vehicles don't do steps
        //FIXME: maybe make hovering vehicles go up steps, but not down them?
        return;
    }

    if !(*pm).gent.is_null()
        && !(*(*pm).gent).client.is_null()
        && ((*(*(*pm).gent).client).NPC_class == CLASS_ATST || (*(*(*pm).gent).client).NPC_class == CLASS_RANCOR)
    {
        isGiant = 1; // qtrue
        if (*(*(*pm).gent).client).NPC_class == CLASS_RANCOR
        {
            if ((*(*pm).gent).spawnflags & 1) != 0
            {
                stepSize = 64;//hack for Mutant Rancor stepping
            }
            else
            {
                stepSize = 48;//hack for Rancor stepping
            }
        }
        else
        {
            stepSize = 70;//hack for AT-ST stepping, slightly taller than a standing stormtrooper
        }
    }
    else if (*pm).maxs[2] <= 0.0
    {//short little guys can't go up steps... FIXME: just make this a flag for certain NPCs- especially ones that roll?
        stepSize = 4;
    }

    //Q3Final addition...
    VectorCopy(&start_o, &mut down);
    down[2] -= stepSize as f32;
    ((*pm).trace)(&mut trace, &start_o, &(*pm).mins, &(*pm).maxs, &down, (*(*pm).ps).clientNum, (*pm).tracemask);
    VectorSet(&mut up, 0.0, 0.0, 1.0);
    // never step up when you still have up velocity
    if (*(*pm).ps).velocity[2] > 0.0 && (trace.fraction == 1.0 ||
            DotProduct(&trace.plane.normal, &up) < 0.7) {
        return;
    }

    if (*(*pm).ps).velocity[0] == 0.0 && (*(*pm).ps).velocity[1] == 0.0
    {//All our velocity was cancelled sliding
        return;
    }

    VectorCopy(&(*(*pm).ps).origin, &mut down_o);
    VectorCopy(&(*(*pm).ps).velocity, &mut down_v);

    VectorCopy(&start_o, &mut up);
    up[2] += stepSize as f32;

    // test the player position if they were a stepheight higher

    ((*pm).trace)(&mut trace, &start_o, &(*pm).mins, &(*pm).maxs, &up, (*(*pm).ps).clientNum, (*pm).tracemask);
    if trace.allsolid != 0 || trace.startsolid != 0 || trace.fraction == 0.0 {
        if (*pm).debugLevel != 0 {
            Com_Printf("%i:bend can't step\n" as *const i8, c_pmove);
        }
        return;		// can't step up
    }

    if (*pm).debugLevel != 0
    {
        G_DebugLine(&start_o, &trace.endpos, 2000, 0xffffff, 1);
    }

//===Another slidemove forward================================================================================
    // try slidemove from this position
    VectorCopy(&trace.endpos, &mut (*(*pm).ps).origin);
    VectorCopy(&start_v, &mut (*(*pm).ps).velocity);
    cantStepUpFwd = PM_SlideMove(gravMod);
//===Another slidemove forward================================================================================

    if (*pm).debugLevel != 0
    {
        G_DebugLine(&trace.endpos, &(*(*pm).ps).origin, 2000, 0xffffff, 1);
    }
    //compare the initial slidemove and this slidemove from a step up position
    VectorSubtract(&down_o, &start_o, &mut slideMove);
    VectorSubtract(&trace.endpos, &(*(*pm).ps).origin, &mut stepUpMove);

    if (stepUpMove[0].abs() < 0.1) && (stepUpMove[1].abs() < 0.1) && (VectorLengthSquared(&slideMove) > VectorLengthSquared(&stepUpMove))
    {
        //slideMove was better, use it
        VectorCopy(&down_o, &mut (*(*pm).ps).origin);
        VectorCopy(&down_v, &mut (*(*pm).ps).velocity);
    }
    else
    {
        let mut skipStep: qboolean = 0; // qfalse
        // push down the final amount
        VectorCopy(&(*(*pm).ps).origin, &mut down);
        down[2] -= stepSize as f32;
        ((*pm).trace)(&mut trace, &(*(*pm).ps).origin, &(*pm).mins, &(*pm).maxs, &down, (*(*pm).ps).clientNum, (*pm).tracemask);
        if (*pm).debugLevel != 0
        {
            G_DebugLine(&(*(*pm).ps).origin, &trace.endpos, 2000, 0xffffff, 1);
        }
        if (*g_stepSlideFix).integer != 0
        {
            if (*(*pm).ps).clientNum < MAX_CLIENTS
                && trace.plane.normal[2] < MIN_WALK_NORMAL
            {//normal players cannot step up slopes that are too steep to walk on!
                let mut stepVec: vec3_t = [0.0; 3];
                //okay, the step up ends on a slope that it too steep to step up onto,
                //BUT:
                //If the step looks like this:
                //  (B)\__
                //        \_____(A)
                //Then it might still be okay, so we figure out the slope of the entire move
                //from (A) to (B) and if that slope is walk-upabble, then it's okay
                VectorSubtract(&trace.endpos, &down_o, &mut stepVec);
                VectorNormalize(&mut stepVec);
                if stepVec[2] > (1.0 - MIN_WALK_NORMAL)
                {
                    if (*pm).debugLevel != 0
                    {
                        G_DebugLine(&down_o, &trace.endpos, 2000, 0x0000ff, 1);
                    }
                    skipStep = 1; // qtrue
                }
            }
        }

        if trace.allsolid == 0
            && skipStep == 0 //normal players cannot step up slopes that are too steep to walk on!
        {
            if (*(*pm).ps).clientNum != 0
                && isGiant != 0
                && !(*g_entities.offset(trace.entityNum as isize)).client.is_null()
                && !(*pm).gent.is_null()
                && !(*(*pm).gent).client.is_null()
                && (*(*(*pm).gent).client).NPC_class == CLASS_RANCOR
            {//Rancor don't step on clients
                if (*g_stepSlideFix).integer != 0
                {
                    VectorCopy(&down_o, &mut (*(*pm).ps).origin);
                    VectorCopy(&down_v, &mut (*(*pm).ps).velocity);
                }
                else
                {
                    VectorCopy(&start_o, &mut (*(*pm).ps).origin);
                    VectorCopy(&start_v, &mut (*(*pm).ps).velocity);
                }
            }
            else if (*(*pm).ps).clientNum != 0
                && isGiant != 0
                && !(*g_entities.offset(trace.entityNum as isize)).client.is_null()
                && (*(*g_entities.offset(trace.entityNum as isize)).client).playerTeam == (*(*(*pm).gent).client).playerTeam
            {//AT-ST's don't step up on allies
                if (*g_stepSlideFix).integer != 0
                {
                    VectorCopy(&down_o, &mut (*(*pm).ps).origin);
                    VectorCopy(&down_v, &mut (*(*pm).ps).velocity);
                }
                else
                {
                    VectorCopy(&start_o, &mut (*(*pm).ps).origin);
                    VectorCopy(&start_v, &mut (*(*pm).ps).velocity);
                }
            }
            else
            {
                VectorCopy(&trace.endpos, &mut (*(*pm).ps).origin);
                if (*g_stepSlideFix).integer != 0
                {
                    if trace.fraction < 1.0
                    {
                        PM_ClipVelocity(&mut (*(*pm).ps).velocity, &trace.plane.normal, &mut (*(*pm).ps).velocity, OVERCLIP);
                    }
                }
            }
        }
        else
        {
            if (*g_stepSlideFix).integer != 0
            {
                VectorCopy(&down_o, &mut (*(*pm).ps).origin);
                VectorCopy(&down_v, &mut (*(*pm).ps).velocity);
            }
        }
        if (*g_stepSlideFix).integer == 0
        {
            if trace.fraction < 1.0
            {
                PM_ClipVelocity(&mut (*(*pm).ps).velocity, &trace.plane.normal, &mut (*(*pm).ps).velocity, OVERCLIP);
            }
        }
    }

    /*
    if(cantStepUpFwd && pm->ps->origin[2] < start_o[2] + stepSize && pm->ps->origin[2] >= start_o[2])
    {//We bumped into something we could not step up
        pm->ps->pm_flags |= PMF_BLOCKED;
    }
    else
    {//We did step up, clear the bumped flag
    }
    */
    // if the down trace can trace back to the original position directly, don't step
    // Disabled (#if 0 in C)
    // pm->trace( &trace, pm->ps->origin, pm->mins, pm->maxs, start_o, pm->ps->clientNum, pm->tracemask);
    // if ( trace.fraction == 1.0 ) {
    //   // use the original move
    //   VectorCopy (down_o, pm->ps->origin);
    //   VectorCopy (down_v, pm->ps->velocity);
    //   if ( pm->debugLevel ) {
    //     Com_Printf("%i:bend\n", c_pmove);
    //   }
    // } else
    {
        // use the step move
        let mut delta: f32;

        delta = (*(*pm).ps).origin[2] - start_o[2];
        if delta > 2.0 {
            if delta < 7.0 {
                PM_AddEvent(EV_STEP_4);
            } else if delta < 11.0 {
                PM_AddEvent(EV_STEP_8);
            } else if delta < 15.0 {
                PM_AddEvent(EV_STEP_12);
            } else {
                PM_AddEvent(EV_STEP_16);
            }
        }
        if (*pm).debugLevel != 0 {
            Com_Printf("%i:stepped\n" as *const i8, c_pmove);
        }
    }
}
