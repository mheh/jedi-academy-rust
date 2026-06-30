// oracle/code/qcommon/cm_trace.cpp — faithful blind port

#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_mut,
    unused_unsafe,
    clippy::all
)]

use crate::code::qcommon::cm_local_h::*;

// #ifdef _XBOX
// #include "../renderer/tr_local.h"
#[cfg(feature = "xbox")]
use crate::code::renderer::tr_local_h::*;

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

/*
===============================================================================

POSITION TESTING

===============================================================================
*/

extern "C" {
    pub static mut com_terrainPhysics: *mut cvar_t;
    // void VectorAdvance( const vec3_t veca, const float scale, const vec3_t vecb, vec3_t vecc);
    fn VectorAdvance(veca: &vec3_t, scale: f32, vecb: &vec3_t, vecc: &mut vec3_t);
}

// #define	MAX_POSITION_LEAFS	1024
const MAX_POSITION_LEAFS: usize = 1024;

/*
================
CM_TestBoxInBrush
================
*/
pub unsafe fn CM_TestBoxInBrush(tw: *mut traceWork_t, brush: *mut cbrush_t) {
    let mut i: c_int;
    let mut plane: *const cplane_t;
    let mut dist: f32;
    let mut d1: f32;
    let mut side: *const cbrushside_t;

    if (*brush).numsides == 0 {
        return;
    }

    // special test for axial
    if (*tw).bounds[0][0] > (*brush).bounds[1][0]
        || (*tw).bounds[0][1] > (*brush).bounds[1][1]
        || (*tw).bounds[0][2] > (*brush).bounds[1][2]
        || (*tw).bounds[1][0] < (*brush).bounds[0][0]
        || (*tw).bounds[1][1] < (*brush).bounds[0][1]
        || (*tw).bounds[1][2] < (*brush).bounds[0][2]
    {
        return;
    }

    // the first six planes are the axial planes, so we only
    // need to test the remainder
    i = 6;
    while i < (*brush).numsides as c_int {
        side = (*brush).sides.offset(i as isize);
        #[cfg(feature = "xbox")]
        {
            plane = (*addr_of!(cmg)).planes.offset((*side).planeNum.GetValue() as isize);
        }
        #[cfg(not(feature = "xbox"))]
        {
            plane = (*side).plane;
        }

        // adjust the plane distance apropriately for mins/maxs
        dist = (*plane).dist
            - DotProduct(
                &(*tw).offsets[(*plane).signbits as usize],
                &(*plane).normal,
            );

        d1 = DotProduct(&(*tw).start, &(*plane).normal) - dist;

        // if completely in front of face, no intersection
        if d1 > 0.0 {
            return;
        }
        i += 1;
    }

    // inside this brush
    (*tw).trace.startsolid = qtrue;
    (*tw).trace.allsolid = qtrue;
    (*tw).trace.fraction = 0.0;
    (*tw).trace.contents = (*brush).contents;
}

/*
================
CM_PlaneCollision

  Returns false for a quick getout
================
*/

pub unsafe fn CM_PlaneCollision(tw: *mut traceWork_t, side: *mut cbrushside_t) -> bool {
    let mut dist: f32;
    let mut f: f32;
    let mut d1: f32;
    let mut d2: f32;
    #[cfg(feature = "xbox")]
    let plane: *const cplane_t =
        (*addr_of!(cmg)).planes.offset((*side).planeNum.GetValue() as isize);
    #[cfg(not(feature = "xbox"))]
    let plane: *const cplane_t = (*side).plane;

    // adjust the plane distance apropriately for mins/maxs
    dist = (*plane).dist
        - DotProduct(&(*tw).offsets[(*plane).signbits as usize], &(*plane).normal);

    d1 = DotProduct(&(*tw).start, &(*plane).normal) - dist;
    d2 = DotProduct(&(*tw).end, &(*plane).normal) - dist;

    if d2 > 0.0 {
        // endpoint is not in solid
        (*tw).getout = true;
    }
    if d1 > 0.0 {
        // startpoint is not in solid
        (*tw).startout = true;
    }

    // if completely in front of face, no intersection with the entire brush
    if (d1 > 0.0) && ((d2 >= SURFACE_CLIP_EPSILON) || (d2 >= d1)) {
        return false;
    }

    // if it doesn't cross the plane, the plane isn't relevent
    if (d1 <= 0.0) && (d2 <= 0.0) {
        return true;
    }
    // crosses face
    if d1 > d2 {
        // enter
        f = d1 - SURFACE_CLIP_EPSILON;
        if f < 0.0 {
            f = 0.0;
            if f > (*tw).enterFrac {
                (*tw).enterFrac = f;
                (*tw).clipplane = plane as *mut cplane_t;
                (*tw).leadside = side;
            }
        } else if f > (*tw).enterFrac * (d1 - d2) {
            (*tw).enterFrac = f / (d1 - d2);
            (*tw).clipplane = plane as *mut cplane_t;
            (*tw).leadside = side;
        }
    } else {
        // leave
        f = d1 + SURFACE_CLIP_EPSILON;
        if f < (d1 - d2) {
            f = 1.0;
            if f < (*tw).leaveFrac {
                (*tw).leaveFrac = f;
            }
        } else if f > (*tw).leaveFrac * (d1 - d2) {
            (*tw).leaveFrac = f / (d1 - d2);
        }
    }
    true
}

/*
================
CM_TraceThroughBrush
================
*/
// porting note: C++ has two overloads of CM_TraceThroughBrush; this is the
// extended version (trace_t &trace, bool infoOnly). Renamed to
// CM_TraceThroughBrush_infoOnly to avoid the duplicate name. Callers in this
// file are updated accordingly.
pub unsafe fn CM_TraceThroughBrush_infoOnly(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    brush: *mut cbrush_t,
    infoOnly: bool,
) {
    let mut i: c_int;
    let mut side: *mut cbrushside_t;

    (*tw).enterFrac = -1.0;
    (*tw).leaveFrac = 1.0;
    (*tw).clipplane = core::ptr::null_mut();

    if (*brush).numsides == 0 {
        return;
    }

    (*tw).getout = false;
    (*tw).startout = false;
    (*tw).leadside = core::ptr::null_mut();

    //
    // compare the trace against all planes of the brush
    // find the latest time the trace crosses a plane towards the interior
    // and the earliest time the trace crosses a plane towards the exterior
    //
    i = 0;
    while i < (*brush).numsides as c_int {
        side = (*brush).sides.offset(i as isize);

        if !CM_PlaneCollision(tw, side) {
            return;
        }
        i += 1;
    }

    //
    // all planes have been checked, and the trace was not
    // completely outside the brush
    //
    if !(*tw).startout {
        if !infoOnly {
            // original point was inside brush
            (*trace).startsolid = qtrue;
            if !(*tw).getout {
                (*trace).allsolid = qtrue;
                (*trace).fraction = 0.0;
            }
        }
        (*tw).enterFrac = 0.0;
        return;
    }

    if (*tw).enterFrac < (*tw).leaveFrac {
        if ((*tw).enterFrac > -1.0) && ((*tw).enterFrac < (*trace).fraction) {
            if (*tw).enterFrac < 0.0 {
                (*tw).enterFrac = 0.0;
            }
            if !infoOnly {
                (*trace).fraction = (*tw).enterFrac;
                (*trace).plane = *(*tw).clipplane;
                (*trace).surfaceFlags = (*(*addr_of!(cmg))
                    .shaders
                    .offset((*(*tw).leadside).shaderNum as isize))
                .surfaceFlags;
                // tw->trace.sideNum = tw->leadside - cmg.brushsides;
                (*trace).contents = (*brush).contents;
            }
        }
    }
}

#[cfg(all(not(feature = "bspc"), not(feature = "xbox")))] // #ifndef BSPC / #ifndef _XBOX  // Removing terrain from Xbox
pub unsafe fn CM_TraceThroughTerrain(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    brush: *mut cbrush_t,
) {
    let mut landscape: *mut CCMLandScape;
    let mut tBegin: vec3_t = [0.0; 3];
    let mut tEnd: vec3_t = [0.0; 3];
    let mut tDistance: vec3_t = [0.0; 3];
    let mut tStep: vec3_t = [0.0; 3];
    let mut baseStart: vec3_t = [0.0; 3];
    let mut baseEnd: vec3_t = [0.0; 3];
    let mut count: c_int;
    let mut i: c_int;
    let mut fraction: f32;

    // At this point we know we may be colliding with a terrain brush (and we know we have a valid terrain structure)
    landscape = (*addr_of!(cmg)).landScape;

    if landscape.is_null() {
        assert!(!landscape.is_null());
        Com_Error(
            ERR_FATAL,
            c"Brush had surfaceparm terrain, but there is no Terrain entity on this map!".as_ptr(),
        );
    }
    // Check for absolutely no connection
    if !CM_GenericBoxCollide(&(*tw).bounds, (*landscape).GetBounds()) {
        return;
    }
    // Now we know that at least some part of the trace needs to collide with the terrain
    // The regular brush collision is handled elsewhere, so advance the ray to an edge in the terrain brush
    CM_TraceThroughBrush_infoOnly(tw, trace, brush, true);

    // Remember the base entering and leaving fractions
    (*tw).baseEnterFrac = (*tw).enterFrac;
    (*tw).baseLeaveFrac = (*tw).leaveFrac;
    // Reset to full spread within the brush
    (*tw).enterFrac = -1.0;
    (*tw).leaveFrac = 1.0;

    // Work out the corners of the AABB when the trace first hits the terrain brush and when it leaves
    VectorAdvance(&(*tw).start, (*tw).baseEnterFrac, &(*tw).end, &mut tBegin);
    VectorAdvance(&(*tw).start, (*tw).baseLeaveFrac, &(*tw).end, &mut tEnd);
    VectorSubtract(&tEnd, &tBegin, &mut tDistance);

    // Calculate number of iterations to process
    count = f32::ceil(
        VectorLength(&tDistance)
            / ((*landscape).GetPatchScalarSize() * TERRAIN_STEP_MAGIC),
    ) as c_int;
    count = 1;
    fraction = (*trace).fraction;
    VectorScale(&tDistance, 1.0 / count as f32, &mut tStep);

    // Save the base start and end vectors
    VectorCopy(&(*tw).start, &mut baseStart);
    VectorCopy(&(*tw).end, &mut baseEnd);

    // Use the terrain vectors.  Start both at the beginning since the
    // step will be added to the end as the first step of the loop
    VectorCopy(&tBegin, &mut (*tw).start);
    VectorCopy(&tBegin, &mut (*tw).end);

    // Step thru terrain patches moving on about 1 patch at a time
    i = 0;
    while i < count {
        // Add the step to the end
        let tw_end_tmp = (*tw).end;
        VectorAdd(&tw_end_tmp, &tStep, &mut (*tw).end);

        CM_CalcExtents(&tBegin, &(*tw).end, tw, &mut (*tw).localBounds);

        (*landscape).PatchCollide(
            tw,
            trace,
            &(*tw).start,
            &(*tw).end,
            (*brush).checkcount,
        );

        // If collision with something closer than water then just stop here
        if (*trace).fraction < fraction {
            // Convert the fraction of this sub tract into the full trace's fraction
            (*trace).fraction =
                i as f32 * (1.0 / count as f32) + (1.0 / count as f32) * (*trace).fraction;
            break;
        }

        // Move the end to the start so the next trace starts
        // where this one left off
        let tw_end_tmp = (*tw).end;
        VectorCopy(&tw_end_tmp, &mut (*tw).start);
        i += 1;
    }

    // Put the original start and end back
    VectorCopy(&baseStart, &mut (*tw).start);
    VectorCopy(&baseEnd, &mut (*tw).end);

    // Convert to global fraction only if something was hit along the way
    if (*trace).fraction != 1.0 {
        (*trace).fraction = (*tw).baseEnterFrac
            + (((*tw).baseLeaveFrac - (*tw).baseEnterFrac) * (*trace).fraction);
        (*trace).contents = (*brush).contents;
    }

    // Collide with any water
    if ((*tw).contents & CONTENTS_WATER) != 0 {
        fraction =
            (*landscape).WaterCollide(&(*tw).start, &(*tw).end, (*trace).fraction);
        if fraction < (*trace).fraction {
            VectorSet(&mut (*trace).plane.normal, 0.0, 0.0, 1.0);
            (*trace).contents = (*landscape).GetWaterContents();
            (*trace).fraction = fraction;
            (*trace).surfaceFlags = (*landscape).GetWaterSurfaceFlags();
        }
    }
}
// #endif	// _XBOX
// #endif

#[cfg(feature = "xbox")] // #ifdef _XBOX
unsafe fn CM_GetSurfaceIndex(firstLeafSurface: c_int) -> c_int {
    if tr.world.is_null()
        || firstLeafSurface > (*tr.world).nummarksurfaces
        || firstLeafSurface < 0
    {
        return *(*addr_of!(cmg)).leafsurfaces.offset(firstLeafSurface as isize);
    } else {
        return *(*tr.world).marksurfaces.offset(firstLeafSurface as isize)
            - (*tr.world).surfaces as c_int;
    }
}

/*
================
CM_TestInLeaf
================
*/
pub unsafe fn CM_TestInLeaf(tw: *mut traceWork_t, leaf: *mut cLeaf_t, local: *mut clipMap_t) {
    let mut k: c_int;
    let mut brushnum: c_int;
    let mut b: *mut cbrush_t;
    let mut patch: *mut cPatch_t;

    // test box position against all brushes in the leaf
    k = 0;
    while k < (*leaf).numLeafBrushes {
        brushnum = *(*local).leafbrushes.offset(((*leaf).firstLeafBrush + k) as isize);
        b = (*local).brushes.offset(brushnum as isize);
        if (*b).checkcount == (*local).checkcount {
            k += 1;
            continue; // already checked this brush in another leaf
        }
        (*b).checkcount = (*local).checkcount;

        if ((*b).contents & (*tw).contents) == 0 {
            k += 1;
            continue;
        }

        #[cfg(all(not(feature = "bspc"), not(feature = "xbox")))] // #ifndef BSPC / #ifndef _XBOX  // Removing terrain from Xbox
        {
            if (*com_terrainPhysics).integer != 0
                && !(*addr_of!(cmg)).landScape.is_null()
                && ((*b).contents & CONTENTS_TERRAIN) != 0
            {
                // Invalidate the checkcount for terrain as the terrain brush has to be processed
                // many times.
                (*b).checkcount -= 1;

                CM_TraceThroughTerrain(tw, &mut (*tw).trace, b);
                // If inside a terrain brush don't bother with regular brush collision
                k += 1;
                continue;
            }
        }

        CM_TestBoxInBrush(tw, b);
        if (*tw).trace.allsolid != 0 {
            return;
        }
        k += 1;
    }

    // test against all patches
    // #ifdef BSPC
    // if (1) {
    // #else
    // if ( !cm_noCurves->integer ) {
    // #endif //BSPC
    // porting note: cfg! avoids invalid #[cfg] on else-arm
    let check_patches: bool = cfg!(feature = "bspc")
        || (*cm_noCurves).integer == 0;
    if check_patches {
        k = 0;
        while k < (*leaf).numLeafSurfaces {
            #[cfg(feature = "xbox")] // #ifdef _XBOX
            {
                let index: c_int = CM_GetSurfaceIndex((*leaf).firstLeafSurface + k);
                patch = *(*addr_of!(cmg)).surfaces.offset(index as isize);
            }
            #[cfg(not(feature = "xbox"))] // #else
            {
                patch = *(*local)
                    .surfaces
                    .offset(*(*local).leafsurfaces.offset(((*leaf).firstLeafSurface + k) as isize) as isize);
            }
            if patch.is_null() {
                k += 1;
                continue;
            }
            if (*patch).checkcount == (*local).checkcount {
                k += 1;
                continue; // already checked this brush in another leaf
            }
            (*patch).checkcount = (*local).checkcount;

            if ((*patch).contents & (*tw).contents) == 0 {
                k += 1;
                continue;
            }

            if CM_PositionTestInPatchCollide(tw, (*patch).pc) != 0 {
                (*tw).trace.startsolid = qtrue;
                (*tw).trace.allsolid = qtrue;
                (*tw).trace.fraction = 0.0;
                (*tw).trace.contents = (*patch).contents;
                return;
            }
            k += 1;
        }
    }
}

/*
==================
CM_PositionTest
==================
*/
pub unsafe fn CM_PositionTest(tw: *mut traceWork_t) {
    let mut leafs: [c_int; MAX_POSITION_LEAFS] = [0; MAX_POSITION_LEAFS];
    let mut i: c_int;
    let mut ll: leafList_t = core::mem::zeroed();

    // identify the leafs we are touching
    VectorAdd(&(*tw).start, &(*tw).size[0], &mut ll.bounds[0]);
    VectorAdd(&(*tw).start, &(*tw).size[1], &mut ll.bounds[1]);

    i = 0;
    while i < 3 {
        ll.bounds[0][i as usize] -= 1.0;
        ll.bounds[1][i as usize] += 1.0;
        i += 1;
    }

    ll.count = 0;
    ll.maxcount = MAX_POSITION_LEAFS as c_int;
    ll.list = leafs.as_mut_ptr();
    ll.storeLeafs = Some(CM_StoreLeafs);
    ll.lastLeaf = 0;
    ll.overflowed = qfalse;

    (*addr_of_mut!(cmg)).checkcount += 1;

    CM_BoxLeafnums_r(&mut ll, 0);

    (*addr_of_mut!(cmg)).checkcount += 1;

    // test the contents of the leafs
    i = 0;
    while i < ll.count {
        CM_TestInLeaf(
            tw,
            (*addr_of!(cmg))
                .leafs
                .offset(*leafs.as_ptr().offset(i as isize) as isize),
            addr_of_mut!(cmg),
        );
        if (*tw).trace.allsolid != 0 {
            break;
        }
        i += 1;
    }
}

/*
===============================================================================

BOX TRACING

===============================================================================
*/


/*
================
CM_TraceThroughPatch
================
*/

pub unsafe fn CM_TraceThroughPatch(tw: *mut traceWork_t, patch: *mut cPatch_t) {
    let mut oldFrac: f32;

    c_patch_traces += 1;

    oldFrac = (*tw).trace.fraction;

    CM_TraceThroughPatchCollide(tw, (*patch).pc);

    if (*tw).trace.fraction < oldFrac {
        (*tw).trace.surfaceFlags = (*patch).surfaceFlags;
        (*tw).trace.contents = (*patch).contents;
    }
}


/*
================
CM_TraceThroughBrush
================
*/
// porting note: this is the second (standard Q3-era) overload of
// CM_TraceThroughBrush(tw, brush). The first overload with (tw, trace, brush,
// infoOnly) is named CM_TraceThroughBrush_infoOnly above.
pub unsafe fn CM_TraceThroughBrush(tw: *mut traceWork_t, brush: *mut cbrush_t) {
    let mut i: c_int;
    let mut plane: *const cplane_t;
    let mut clipplane: *const cplane_t;
    let mut dist: f32;
    let mut enterFrac: f32;
    let mut leaveFrac: f32;
    let mut d1: f32;
    let mut d2: f32;
    let mut getout: qboolean;
    let mut startout: qboolean;
    let mut f: f32;
    let mut side: *const cbrushside_t;
    let mut leadside: *const cbrushside_t;

    enterFrac = -1.0;
    leaveFrac = 1.0;
    clipplane = core::ptr::null();

    if (*brush).numsides == 0 {
        return;
    }

    // I'm not sure if test is strictly correct.  Are all
    // bboxes axis aligned?  Do I care?  It seems to work
    // good enough...
    if (*tw).bounds[0][0] > (*brush).bounds[1][0]
        || (*tw).bounds[0][1] > (*brush).bounds[1][1]
        || (*tw).bounds[0][2] > (*brush).bounds[1][2]
        || (*tw).bounds[1][0] < (*brush).bounds[0][0]
        || (*tw).bounds[1][1] < (*brush).bounds[0][1]
        || (*tw).bounds[1][2] < (*brush).bounds[0][2]
    {
        return;
    }

    c_brush_traces += 1;

    getout = qfalse;
    startout = qfalse;

    leadside = core::ptr::null();

    //
    // compare the trace against all planes of the brush
    // find the latest time the trace crosses a plane towards the interior
    // and the earliest time the trace crosses a plane towards the exterior
    //
    i = 0;
    while i < (*brush).numsides as c_int {
        side = (*brush).sides.offset(i as isize);
        #[cfg(feature = "xbox")]
        {
            plane = (*addr_of!(cmg)).planes.offset((*side).planeNum.GetValue() as isize);
        }
        #[cfg(not(feature = "xbox"))]
        {
            plane = (*side).plane;
        }

        // adjust the plane distance apropriately for mins/maxs
        dist = (*plane).dist
            - DotProduct(&(*tw).offsets[(*plane).signbits as usize], &(*plane).normal);

        d1 = DotProduct(&(*tw).start, &(*plane).normal) - dist;
        d2 = DotProduct(&(*tw).end, &(*plane).normal) - dist;

        if d2 > 0.0 {
            getout = qtrue; // endpoint is not in solid
        }
        if d1 > 0.0 {
            startout = qtrue;
        }

        // if completely in front of face, no intersection with the entire brush
        if d1 > 0.0 && (d2 >= SURFACE_CLIP_EPSILON || d2 >= d1) {
            return;
        }

        // if it doesn't cross the plane, the plane isn't relevent
        if d1 <= 0.0 && d2 <= 0.0 {
            i += 1;
            continue;
        }

        // crosses face
        if d1 > d2 {
            // enter
            f = (d1 - SURFACE_CLIP_EPSILON) / (d1 - d2);
            if f < 0.0 {
                f = 0.0;
            }
            if f > enterFrac {
                enterFrac = f;
                clipplane = plane;
                leadside = side;
            }
        } else {
            // leave
            f = (d1 + SURFACE_CLIP_EPSILON) / (d1 - d2);
            if f > 1.0 {
                f = 1.0;
            }
            if f < leaveFrac {
                leaveFrac = f;
            }
        }
        i += 1;
    }

    //
    // all planes have been checked, and the trace was not
    // completely outside the brush
    //
    if startout == qfalse {
        // original point was inside brush
        (*tw).trace.startsolid = qtrue;
        (*tw).trace.contents |= (*brush).contents; //note, we always want to know the contents of something we're inside of
        if getout == qfalse {
            //endpoint was inside brush
            (*tw).trace.allsolid = qtrue;
            (*tw).trace.fraction = 0.0;
        }
        return;
    }

    if enterFrac < leaveFrac {
        if enterFrac > -1.0 && enterFrac < (*tw).trace.fraction {
            if enterFrac < 0.0 {
                enterFrac = 0.0;
            }
            (*tw).trace.fraction = enterFrac;
            (*tw).trace.plane = *clipplane;
            (*tw).trace.surfaceFlags = (*(*addr_of!(cmg))
                .shaders
                .offset((*leadside).shaderNum as isize))
            .surfaceFlags;
            (*tw).trace.contents = (*brush).contents;
        }
    }
}

/*
================
CM_PatchCollide

  By the time we get here we know the AABB is within the patch AABB ie there is a chance of collision
  The collision data is made up of bounds, 2 triangle planes
  There is an BB check for the terxel check to see if it is worth checking the planes.
  Collide with both triangles to find the shortest fraction
================
*/

pub unsafe fn CM_HandlePatchCollision(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    tStart: *const vec3_t,
    tEnd: *const vec3_t,
    patch: *mut CCMPatch,
    checkcount: c_int,
) {
    let mut numBrushes: c_int;
    let mut i: c_int;
    let mut brush: *mut cbrush_t;

    // Get the collision data
    brush = (*patch).GetCollisionData();
    numBrushes = (*patch).GetNumBrushes();

    i = 0;
    while i < numBrushes {
        if (*brush).checkcount == checkcount {
            return;
        }

        // Generic collision of terxel bounds to line segment bounds
        if !CM_GenericBoxCollide(&(*brush).bounds, &(*tw).localBounds) {
            brush = brush.offset(1);
            i += 1;
            continue;
        }

        (*brush).checkcount = checkcount;

        //CM_TraceThroughBrush(tw, trace, brush, false );
        CM_TraceThroughBrush(tw, brush);
        if (*trace).fraction <= 0.0 {
            break;
        }
        brush = brush.offset(1);
        i += 1;
    }
}

/*
================
CM_GenericBoxCollide
================
*/

pub unsafe fn CM_GenericBoxCollide(abounds: &vec3pair_t, bbounds: &vec3pair_t) -> bool {
    let mut i: c_int;

    // Check for completely no intersection
    i = 0;
    while i < 3 {
        if abounds[1][i as usize] < bbounds[0][i as usize] {
            return false;
        }
        if abounds[0][i as usize] > bbounds[1][i as usize] {
            return false;
        }
        i += 1;
    }
    true
}

/*
================
CM_TraceToLeaf
================
*/
pub unsafe fn CM_TraceToLeaf(tw: *mut traceWork_t, leaf: *mut cLeaf_t, local: *mut clipMap_t) {
    let mut k: c_int;
    let mut brushnum: c_int;
    let mut b: *mut cbrush_t;
    let mut patch: *mut cPatch_t;

    // trace line against all brushes in the leaf
    k = 0;
    while k < (*leaf).numLeafBrushes {
        brushnum = *(*local).leafbrushes.offset(((*leaf).firstLeafBrush + k) as isize);

        b = (*local).brushes.offset(brushnum as isize);
        if (*b).checkcount == (*local).checkcount {
            k += 1;
            continue; // already checked this brush in another leaf
        }
        (*b).checkcount = (*local).checkcount;

        if ((*b).contents & (*tw).contents) == 0 {
            k += 1;
            continue;
        }

        #[cfg(all(not(feature = "bspc"), not(feature = "xbox")))] // #ifndef BSPC / #ifndef _XBOX  // Removing terrain from Xbox
        {
            if (*com_terrainPhysics).integer != 0
                && !(*addr_of!(cmg)).landScape.is_null()
                && ((*b).contents & CONTENTS_TERRAIN) != 0
            {
                // Invalidate the checkcount for terrain as the terrain brush has to be processed
                // many times.
                (*b).checkcount -= 1;

                CM_TraceThroughTerrain(tw, &mut (*tw).trace, b);
                // If inside a terrain brush don't bother with regular brush collision
                k += 1;
                continue;
            }
        }

        //if (b->contents & CONTENTS_PLAYERCLIP) continue;

        CM_TraceThroughBrush(tw, b);
        if (*tw).trace.fraction == 0.0 {
            return;
        }
        k += 1;
    }

    // trace line against all patches in the leaf
    // #ifdef BSPC
    // if (1) {
    // #else
    // if ( !cm_noCurves->integer ) {
    // #endif
    // porting note: cfg! avoids invalid #[cfg] on else-arm
    let check_patches: bool = cfg!(feature = "bspc")
        || (*cm_noCurves).integer == 0;
    if check_patches {
        k = 0;
        while k < (*leaf).numLeafSurfaces {
            #[cfg(feature = "xbox")] // #ifdef _XBOX
            {
                let index: c_int = CM_GetSurfaceIndex((*leaf).firstLeafSurface + k);
                patch = *(*addr_of!(cmg)).surfaces.offset(index as isize);
            }
            #[cfg(not(feature = "xbox"))] // #else
            {
                patch = *(*local)
                    .surfaces
                    .offset(*(*local).leafsurfaces.offset(((*leaf).firstLeafSurface + k) as isize) as isize);
            }
            if patch.is_null() {
                k += 1;
                continue;
            }
            if (*patch).checkcount == (*local).checkcount {
                k += 1;
                continue; // already checked this patch in another leaf
            }
            (*patch).checkcount = (*local).checkcount;

            if ((*patch).contents & (*tw).contents) == 0 {
                k += 1;
                continue;
            }

            CM_TraceThroughPatch(tw, patch);
            if (*tw).trace.fraction == 0.0 {
                return;
            }
            k += 1;
        }
    }
}

//=========================================================================================

/*
==================
CM_TraceThroughTree

Traverse all the contacted leafs from the start to the end position.
If the trace is a point, they will be exactly in order, but for larger
trace volumes it is possible to hit something in a later leaf with
a smaller intercept fraction.
==================
*/
pub unsafe fn CM_TraceThroughTree(
    tw: *mut traceWork_t,
    local: *mut clipMap_t,
    num: c_int,
    p1f: f32,
    p2f: f32,
    p1: *mut vec3_t,
    p2: *mut vec3_t,
) {
    let mut node: *const cNode_t;
    let mut plane: *const cplane_t;
    let mut t1: f32;
    let mut t2: f32;
    let mut offset: f32;
    let mut frac: f32;
    let mut frac2: f32;
    let mut idist: f32;
    let mut mid: vec3_t = [0.0; 3];
    let mut side: c_int;
    let mut midf: f32;

    #[cfg(feature = "xbox")] // #ifdef _XBOX
    {
        if tr.world.is_null() {
            return;
        }
    }

    if (*tw).trace.fraction <= p1f {
        return; // already hit something nearer
    }

    // if < 0, we are in a leaf node
    if num < 0 {
        CM_TraceToLeaf(tw, (*local).leafs.offset((-1 - num) as isize), local);
        return;
    }

    //
    // find the point distances to the seperating plane
    // and the offset for the size of the box
    //
    node = (*local).nodes.offset(num as isize);

    #[cfg(feature = "xbox")]
    {
        plane = (*addr_of!(cmg))
            .planes
            .offset((*(*tr.world).nodes.offset(num as isize)).planeNum as isize);
    }
    #[cfg(not(feature = "xbox"))] /* mnode_s  — stray identifier in original #else line, preserved */
    {
        plane = (*node).plane;
    }

    /* #if 0
    	// uncomment this to test against every leaf in the world for debugging
    CM_TraceThroughTree( tw, local, node->children[0], p1f, p2f, p1, p2 );
    CM_TraceThroughTree( tw, local, node->children[1], p1f, p2f, p1, p2 );
    return;
    #endif */

    // adjust the plane distance apropriately for mins/maxs
    if (*plane).r#type < 3 {
        t1 = (*p1)[(*plane).r#type as usize] - (*plane).dist;
        t2 = (*p2)[(*plane).r#type as usize] - (*plane).dist;
        offset = (*tw).extents[(*plane).r#type as usize];
    } else {
        t1 = DotProduct(&*p1, &(*plane).normal) - (*plane).dist;
        t2 = DotProduct(&*p2, &(*plane).normal) - (*plane).dist;
        if (*tw).isPoint != 0 {
            offset = 0.0;
        } else {
            // an axial brush right behind a slanted bsp plane
            // will poke through when expanded, so adjust
            // by sqrt(3)
            offset = f32::abs((*tw).extents[0] * (*plane).normal[0])
                + f32::abs((*tw).extents[1] * (*plane).normal[1])
                + f32::abs((*tw).extents[2] * (*plane).normal[2]);

            offset *= 2.0;
            /* #if 0
            CM_TraceThroughTree( tw, local, node->children[0], p1f, p2f, p1, p2 );
            CM_TraceThroughTree( tw, local, node->children[1], p1f, p2f, p1, p2 );
            return;
            #endif */
            offset = (*tw).maxOffset;
            offset = 2048.0;
        }
    }

    // see which sides we need to consider
    if t1 >= offset + 1.0 && t2 >= offset + 1.0 {
        CM_TraceThroughTree(tw, local, (*node).children[0], p1f, p2f, p1, p2);
        return;
    }
    if t1 < -offset - 1.0 && t2 < -offset - 1.0 {
        CM_TraceThroughTree(tw, local, (*node).children[1], p1f, p2f, p1, p2);
        return;
    }

    // put the crosspoint SURFACE_CLIP_EPSILON pixels on the near side
    if t1 < t2 {
        idist = 1.0 / (t1 - t2);
        side = 1;
        frac2 = (t1 + offset + SURFACE_CLIP_EPSILON) * idist;
        frac = (t1 - offset + SURFACE_CLIP_EPSILON) * idist;
    } else if t1 > t2 {
        idist = 1.0 / (t1 - t2);
        side = 0;
        frac2 = (t1 - offset - SURFACE_CLIP_EPSILON) * idist;
        frac = (t1 + offset + SURFACE_CLIP_EPSILON) * idist;
    } else {
        side = 0;
        frac = 1.0;
        frac2 = 0.0;
    }

    // move up to the node
    if frac < 0.0 {
        frac = 0.0;
    }
    if frac > 1.0 {
        frac = 1.0;
    }

    midf = p1f + (p2f - p1f) * frac;

    mid[0] = (*p1)[0] + frac * ((*p2)[0] - (*p1)[0]);
    mid[1] = (*p1)[1] + frac * ((*p2)[1] - (*p1)[1]);
    mid[2] = (*p1)[2] + frac * ((*p2)[2] - (*p1)[2]);

    CM_TraceThroughTree(tw, local, (*node).children[side as usize], p1f, midf, p1, &mut mid);


    // go past the node
    if frac2 < 0.0 {
        frac2 = 0.0;
    }
    if frac2 > 1.0 {
        frac2 = 1.0;
    }

    midf = p1f + (p2f - p1f) * frac2;

    mid[0] = (*p1)[0] + frac2 * ((*p2)[0] - (*p1)[0]);
    mid[1] = (*p1)[1] + frac2 * ((*p2)[1] - (*p1)[1]);
    mid[2] = (*p1)[2] + frac2 * ((*p2)[2] - (*p1)[2]);

    CM_TraceThroughTree(
        tw,
        local,
        (*node).children[(side ^ 1) as usize],
        midf,
        p2f,
        &mut mid,
        p2,
    );
}

pub unsafe fn CM_CalcExtents(
    start: *const vec3_t,
    end: *const vec3_t,
    tw: *const traceWork_t,
    bounds: *mut vec3pair_t,
) {
    let mut i: c_int;

    i = 0;
    while i < 3 {
        if (*start)[i as usize] < (*end)[i as usize] {
            (*bounds)[0][i as usize] = (*start)[i as usize] + (*tw).size[0][i as usize];
            (*bounds)[1][i as usize] = (*end)[i as usize] + (*tw).size[1][i as usize];
        } else {
            (*bounds)[0][i as usize] = (*end)[i as usize] + (*tw).size[0][i as usize];
            (*bounds)[1][i as usize] = (*start)[i as usize] + (*tw).size[1][i as usize];
        }
        i += 1;
    }
}

//======================================================================

/*
==================
CM_BoxTrace
==================
*/
pub unsafe fn CM_BoxTrace(
    results: *mut trace_t,
    start: *const vec3_t,
    end: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    model: clipHandle_t,
    brushmask: c_int,
) {
    let mut i: c_int;
    // C: memset( &tw, 0, sizeof(tw) - sizeof(tw.trace.G2CollisionMap))
    // porting deviation: zeroing the entire struct including G2CollisionMap;
    // original left G2CollisionMap uninitialized as a performance optimization.
    let mut tw: traceWork_t = core::mem::zeroed();
    let mut offset: vec3_t = [0.0; 3];
    let mut cmod: *mut cmodel_t;
    let mut local: *mut clipMap_t = core::ptr::null_mut();

    cmod = CM_ClipHandleToModel(model, &mut local);

    (*local).checkcount += 1; // for multi-check avoidance

    c_traces += 1; // for statistics, may be zeroed

    // fill in a default trace
    // (tw already zeroed above)
    tw.trace.fraction = 1.0; // assume it goes the entire distance until shown otherwise

    if (*local).numNodes == 0 {
        *results = tw.trace;
        return; // map not loaded, shouldn't happen
    }

    // allow NULL to be passed in for 0,0,0
    let mins: *const vec3_t = if mins.is_null() { addr_of!(vec3_origin) } else { mins };
    let maxs: *const vec3_t = if maxs.is_null() { addr_of!(vec3_origin) } else { maxs };

    // set basic parms
    tw.contents = brushmask;

    // adjust so that mins and maxs are always symetric, which
    // avoids some complications with plane expanding of rotated
    // bmodels
    i = 0;
    while i < 3 {
        offset[i as usize] = ((*mins)[i as usize] + (*maxs)[i as usize]) * 0.5;
        tw.size[0][i as usize] = (*mins)[i as usize] - offset[i as usize];
        tw.size[1][i as usize] = (*maxs)[i as usize] - offset[i as usize];
        tw.start[i as usize] = (*start)[i as usize] + offset[i as usize];
        tw.end[i as usize] = (*end)[i as usize] + offset[i as usize];
        i += 1;
    }

    tw.maxOffset = tw.size[1][0] + tw.size[1][1] + tw.size[1][2];

    // tw.offsets[signbits] = vector to apropriate corner from origin
    tw.offsets[0][0] = tw.size[0][0];
    tw.offsets[0][1] = tw.size[0][1];
    tw.offsets[0][2] = tw.size[0][2];

    tw.offsets[1][0] = tw.size[1][0];
    tw.offsets[1][1] = tw.size[0][1];
    tw.offsets[1][2] = tw.size[0][2];

    tw.offsets[2][0] = tw.size[0][0];
    tw.offsets[2][1] = tw.size[1][1];
    tw.offsets[2][2] = tw.size[0][2];

    tw.offsets[3][0] = tw.size[1][0];
    tw.offsets[3][1] = tw.size[1][1];
    tw.offsets[3][2] = tw.size[0][2];

    tw.offsets[4][0] = tw.size[0][0];
    tw.offsets[4][1] = tw.size[0][1];
    tw.offsets[4][2] = tw.size[1][2];

    tw.offsets[5][0] = tw.size[1][0];
    tw.offsets[5][1] = tw.size[0][1];
    tw.offsets[5][2] = tw.size[1][2];

    tw.offsets[6][0] = tw.size[0][0];
    tw.offsets[6][1] = tw.size[1][1];
    tw.offsets[6][2] = tw.size[1][2];

    tw.offsets[7][0] = tw.size[1][0];
    tw.offsets[7][1] = tw.size[1][1];
    tw.offsets[7][2] = tw.size[1][2];


    //
    // calculate bounds
    //
    i = 0;
    while i < 3 {
        if tw.start[i as usize] < tw.end[i as usize] {
            tw.bounds[0][i as usize] = tw.start[i as usize] + tw.size[0][i as usize];
            tw.bounds[1][i as usize] = tw.end[i as usize] + tw.size[1][i as usize];
        } else {
            tw.bounds[0][i as usize] = tw.end[i as usize] + tw.size[0][i as usize];
            tw.bounds[1][i as usize] = tw.start[i as usize] + tw.size[1][i as usize];
        }
        i += 1;
    }

    //
    // check for position test special case
    //
    if (*start)[0] == (*end)[0] && (*start)[1] == (*end)[1] && (*start)[2] == (*end)[2] {
        if model != 0 {
            CM_TestInLeaf(&mut tw, &mut (*cmod).leaf, local);
        } else {
            CM_PositionTest(&mut tw);
        }
    } else {
        //
        // check for point special case
        //
        if tw.size[0][0] == 0.0 && tw.size[0][1] == 0.0 && tw.size[0][2] == 0.0 {
            tw.isPoint = qtrue;
            VectorClear(&mut tw.extents);
        } else {
            tw.isPoint = qfalse;
            tw.extents[0] = tw.size[1][0];
            tw.extents[1] = tw.size[1][1];
            tw.extents[2] = tw.size[1][2];
        }

        //
        // general sweeping through world
        //
        if model != 0 {
            CM_TraceToLeaf(&mut tw, &mut (*cmod).leaf, local);
        } else {
            CM_TraceThroughTree(
                &mut tw,
                local,
                0,
                0.0,
                1.0,
                addr_of_mut!(tw.start),
                addr_of_mut!(tw.end),
            );
        }
    }

    // generate endpos from the original, unmodified start/end
    if tw.trace.fraction == 1.0 {
        VectorCopy(&*end, &mut tw.trace.endpos);
    } else {
        i = 0;
        while i < 3 {
            tw.trace.endpos[i as usize] = (*start)[i as usize]
                + tw.trace.fraction * ((*end)[i as usize] - (*start)[i as usize]);
            i += 1;
        }
    }

    *results = tw.trace;
}


/*
==================
CM_TransformedBoxTrace

Handles offseting and rotation of the end points for moving and
rotating entities
==================
*/
pub unsafe fn CM_TransformedBoxTrace(
    results: *mut trace_t,
    start: *const vec3_t,
    end: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    model: clipHandle_t,
    brushmask: c_int,
    origin: *const vec3_t,
    angles: *const vec3_t,
) {
    let mut trace: trace_t = core::mem::zeroed();
    let mut start_l: vec3_t = [0.0; 3];
    let mut end_l: vec3_t = [0.0; 3];
    let mut a: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];
    let mut rotated: qboolean;
    let mut offset: vec3_t = [0.0; 3];
    let mut symetricSize: [vec3_t; 2] = [[0.0; 3]; 2];
    let mut i: c_int;

    let mins: *const vec3_t = if mins.is_null() { addr_of!(vec3_origin) } else { mins };
    let maxs: *const vec3_t = if maxs.is_null() { addr_of!(vec3_origin) } else { maxs };

    // adjust so that mins and maxs are always symetric, which
    // avoids some complications with plane expanding of rotated
    // bmodels
    i = 0;
    while i < 3 {
        offset[i as usize] = ((*mins)[i as usize] + (*maxs)[i as usize]) * 0.5;
        symetricSize[0][i as usize] = (*mins)[i as usize] - offset[i as usize];
        symetricSize[1][i as usize] = (*maxs)[i as usize] - offset[i as usize];
        start_l[i as usize] = (*start)[i as usize] + offset[i as usize];
        end_l[i as usize] = (*end)[i as usize] + offset[i as usize];
        i += 1;
    }

    // subtract origin offset
    VectorSubtract(&start_l, &*origin, &mut start_l);
    VectorSubtract(&end_l, &*origin, &mut end_l);

    // rotate start and end into the models frame of reference
    if model != BOX_MODEL_HANDLE
        && ((*angles)[0] != 0.0 || (*angles)[1] != 0.0 || (*angles)[2] != 0.0)
    {
        rotated = qtrue;
    } else {
        rotated = qfalse;
    }

    if rotated != qfalse {
        AngleVectors(&*angles, Some(&mut forward), Some(&mut right), Some(&mut up));

        VectorCopy(&start_l, &mut temp);
        start_l[0] = DotProduct(&temp, &forward);
        start_l[1] = -DotProduct(&temp, &right);
        start_l[2] = DotProduct(&temp, &up);

        VectorCopy(&end_l, &mut temp);
        end_l[0] = DotProduct(&temp, &forward);
        end_l[1] = -DotProduct(&temp, &right);
        end_l[2] = DotProduct(&temp, &up);
    }

    // sweep the box through the model
    CM_BoxTrace(
        &mut trace,
        &start_l,
        &end_l,
        &symetricSize[0],
        &symetricSize[1],
        model,
        brushmask,
    );

    if rotated != qfalse && trace.fraction != 1.0 {
        // FIXME: figure out how to do this with existing angles
        VectorNegate(&*angles, &mut a);
        AngleVectors(&a, Some(&mut forward), Some(&mut right), Some(&mut up));

        VectorCopy(&trace.plane.normal, &mut temp);
        trace.plane.normal[0] = DotProduct(&temp, &forward);
        trace.plane.normal[1] = -DotProduct(&temp, &right);
        trace.plane.normal[2] = DotProduct(&temp, &up);
    }

    trace.endpos[0] = (*start)[0] + trace.fraction * ((*end)[0] - (*start)[0]);
    trace.endpos[1] = (*start)[1] + trace.fraction * ((*end)[1] - (*start)[1]);
    trace.endpos[2] = (*start)[2] + trace.fraction * ((*end)[2] - (*start)[2]);

    *results = trace;
}

/*
=================
CM_CullBox

Returns true if culled out
=================
*/

pub unsafe fn CM_CullBox(frustum: *const cplane_t, transformed: *const vec3_t) -> bool {
    let mut i: c_int;
    let mut j: c_int;
    let mut frust: *const cplane_t;

    // check against frustum planes
    i = 0;
    frust = frustum;
    while i < 4 {
        j = 0;
        while j < 8 {
            if DotProduct(&*transformed.offset(j as isize), &(*frust).normal)
                > (*frust).dist
            {
                // a point is in front
                break;
            }
            j += 1;
        }

        if j == 8 {
            // all points were behind one of the planes
            return true;
        }
        i += 1;
        frust = frust.offset(1);
    }
    false
}

/*
=================
CM_CullWorldBox

Returns true if culled out
=================
*/

pub unsafe fn CM_CullWorldBox(frustum: *const cplane_t, bounds: *const vec3pair_t) -> bool {
    let mut i: c_int;
    let mut transformed: [vec3_t; 8] = [[0.0; 3]; 8];

    i = 0;
    while i < 8 {
        transformed[i as usize][0] = (*bounds)[(i & 1) as usize][0];
        transformed[i as usize][1] = (*bounds)[((i >> 1) & 1) as usize][1];
        transformed[i as usize][2] = (*bounds)[((i >> 2) & 1) as usize][2];
        i += 1;
    }

    //rwwFIXMEFIXME: Was not ! before. But that seems the way it should be and it works that way. Why?
    !CM_CullBox(frustum, transformed.as_ptr())
}
