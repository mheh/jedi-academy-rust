// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #include "cm_local.h"
// #include "cm_landscape.h"

// #ifdef _XBOX
// #include "../renderer/tr_local.h"
// #endif

// always use bbox vs. bbox collision and never capsule vs. bbox or vice versa
// #define ALWAYS_BBOX_VS_BBOX
// always use capsule vs. capsule collision and never capsule vs. bbox or vice versa
// #define ALWAYS_CAPSULE_VS_CAPSULE

// #define CAPSULE_DEBUG

extern "C" {
    fn CM_TraceThroughTerrain(tw: *mut traceWork_t, trace: *mut trace_t, brush: *mut cbrush_t);
}

// #define TEST_TERRAIN_PHYSICS

// #ifdef TEST_TERRAIN_PHYSICS
//
// Be sure to un-link entity in		void SP_terrain(gentity_t *ent)			(yeah, left this uncommented to cause error / attention )
//
//
// 	void CM_TraceThroughTerrain( traceWork_t *tw, trace_t &trace, CCMLandScape *landscape);
// #endif

/*
===============================================================================

BASIC MATH

===============================================================================
*/

/*
================
RotatePoint
================
*/
fn RotatePoint(point: &mut [f32; 3], matrix: &[[f32; 3]; 3]) {
    // bk: FIXME
    let mut tvec: [f32; 3] = [0.0; 3];

    VectorCopy(point, &mut tvec);
    point[0] = DotProduct(&matrix[0], &tvec);
    point[1] = DotProduct(&matrix[1], &tvec);
    point[2] = DotProduct(&matrix[2], &tvec);
}

/*
================
TransposeMatrix
================
*/
fn TransposeMatrix(matrix: &[[f32; 3]; 3], transpose: &mut [[f32; 3]; 3]) {
    // bk: FIXME
    for i in 0..3 {
        for j in 0..3 {
            transpose[i][j] = matrix[j][i];
        }
    }
}

/*
================
CreateRotationMatrix
================
*/
fn CreateRotationMatrix(angles: &[f32; 3], matrix: &mut [[f32; 3]; 3]) {
    AngleVectors(angles, &mut matrix[0], &mut matrix[1], &mut matrix[2]);
    VectorInverse(&mut matrix[1]);
}

/*
================
CM_ProjectPointOntoVector
================
*/
fn CM_ProjectPointOntoVector(point: &[f32; 3], vStart: &[f32; 3], vDir: &[f32; 3], vProj: &mut [f32; 3]) {
    let mut pVec: [f32; 3] = [0.0; 3];

    VectorSubtract(point, vStart, &mut pVec);
    // project onto the directional vector for this segment
    VectorMA(vStart, DotProduct(&pVec, vDir), vDir, vProj);
}

/*
================
CM_DistanceFromLineSquared
================
*/
fn CM_DistanceFromLineSquared(p: &[f32; 3], lp1: &[f32; 3], lp2: &[f32; 3], dir: &[f32; 3]) -> f32 {
    let mut proj: [f32; 3] = [0.0; 3];
    let mut t: [f32; 3] = [0.0; 3];
    let mut j: i32;

    CM_ProjectPointOntoVector(p, lp1, dir, &mut proj);
    j = 0;
    while j < 3 {
        if (proj[j as usize] > lp1[j as usize] && proj[j as usize] > lp2[j as usize])
            || (proj[j as usize] < lp1[j as usize] && proj[j as usize] < lp2[j as usize])
        {
            break;
        }
        j += 1;
    }
    if j < 3 {
        if (proj[j as usize] - lp1[j as usize]).abs() < (proj[j as usize] - lp2[j as usize]).abs() {
            VectorSubtract(p, lp1, &mut t);
        } else {
            VectorSubtract(p, lp2, &mut t);
        }
        return VectorLengthSquared(&t);
    }
    VectorSubtract(p, &proj, &mut t);
    return VectorLengthSquared(&t);
}

/*
================
CM_VectorDistanceSquared
================
*/
fn CM_VectorDistanceSquared(p1: &[f32; 3], p2: &[f32; 3]) -> f32 {
    let mut dir: [f32; 3] = [0.0; 3];

    VectorSubtract(p2, p1, &mut dir);
    return VectorLengthSquared(&dir);
}

/*
================
SquareRootFloat
================
*/
fn SquareRootFloat(number: f32) -> f32 {
    let mut i: i32;
    let mut x: f32;
    let mut y: f32;
    let f: f32 = 1.5f32;

    x = number * 0.5f32;
    y = number;
    i = unsafe { *((&y as *const f32) as *const i32) };
    i = 0x5f3759df - (i >> 1);
    y = unsafe { *((&i as *const i32) as *const f32) };
    y = y * (f - (x * y * y));
    y = y * (f - (x * y * y));
    return number * y;
}

/*
===============================================================================

POSITION TESTING

===============================================================================
*/

/*
================
CM_TestBoxInBrush
================
*/
fn CM_TestBoxInBrush(tw: *mut traceWork_t, trace: *mut trace_t, brush: *mut cbrush_t) {
    let mut i: i32;
    let mut plane: *const cplane_t;
    let mut dist: f32;
    let mut d1: f32;
    let mut side: *mut cbrushside_t;
    let mut t: f32;
    let mut startp: [f32; 3] = [0.0; 3];

    if unsafe { (*brush).numsides == 0 } {
        return;
    }

    // special test for axial
    if unsafe {
        (*tw).bounds[0][0] > (*brush).bounds[1][0]
            || (*tw).bounds[0][1] > (*brush).bounds[1][1]
            || (*tw).bounds[0][2] > (*brush).bounds[1][2]
            || (*tw).bounds[1][0] < (*brush).bounds[0][0]
            || (*tw).bounds[1][1] < (*brush).bounds[0][1]
            || (*tw).bounds[1][2] < (*brush).bounds[0][2]
    } {
        return;
    }

    if unsafe { (*tw).sphere.use } {
        // the first six planes are the axial planes, so we only
        // need to test the remainder
        i = 6;
        while i < unsafe { (*brush).numsides } {
            side = unsafe { (*brush).sides.as_mut().unwrap().as_mut_ptr().add(i as usize) };

            // #ifdef _XBOX
            //     plane = &cmg.planes[side->planeNum.GetValue()];
            // #else
            plane = unsafe { (*side).plane };
            // #endif

            // adjust the plane distance apropriately for radius
            dist = unsafe { (*plane).dist + (*tw).sphere.radius };
            // find the closest point on the capsule to the plane
            t = DotProduct(&unsafe { (*plane).normal }, &unsafe { (*tw).sphere.offset });
            if t > 0.0 {
                VectorSubtract(&unsafe { (*tw).start }, &unsafe { (*tw).sphere.offset }, &mut startp);
            } else {
                VectorAdd(&unsafe { (*tw).start }, &unsafe { (*tw).sphere.offset }, &mut startp);
            }
            d1 = DotProduct(&startp, &unsafe { (*plane).normal }) - dist;
            // if completely in front of face, no intersection
            if d1 > 0.0 {
                return;
            }
            i += 1;
        }
    } else {
        // the first six planes are the axial planes, so we only
        // need to test the remainder
        i = 6;
        while i < unsafe { (*brush).numsides } {
            side = unsafe { (*brush).sides.as_mut().unwrap().as_mut_ptr().add(i as usize) };

            // #ifdef _XBOX
            //     plane = &cmg.planes[side->planeNum.GetValue()];
            // #else
            plane = unsafe { (*side).plane };
            // #endif

            // adjust the plane distance apropriately for mins/maxs
            dist = unsafe { (*plane).dist - DotProduct(&unsafe { (*tw).offsets[(*plane).signbits as usize] }, &(*plane).normal) };

            d1 = DotProduct(&unsafe { (*tw).start }, &unsafe { (*plane).normal }) - dist;

            // if completely in front of face, no intersection
            if d1 > 0.0 {
                return;
            }
            i += 1;
        }
    }

    // inside this brush
    unsafe {
        (*trace).startsolid = true;
        (*trace).allsolid = true;
        (*trace).fraction = 0.0;
        (*trace).contents = (*brush).contents;
    }
}

// #ifdef _XBOX
// static int CM_GetSurfaceIndex(int firstLeafSurface)
// {
// 	if(firstLeafSurface > tr.world->nummarksurfaces || firstLeafSurface < 0) {
// 		return cmg.leafsurfaces[ firstLeafSurface ] ;
// 	} else {
// 		return tr.world->marksurfaces[firstLeafSurface] - tr.world->surfaces;
// 	}
// }
// #endif

/*
================
CM_TestInLeaf
================
*/
fn CM_TestInLeaf(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    leaf: *const cLeaf_t,
    local: *mut clipMap_t,
) {
    let mut k: i32;
    let mut brushnum: i32;
    let mut b: *mut cbrush_t;
    let mut patch: *mut cPatch_t;

    // test box position against all brushes in the leaf
    k = 0;
    while k < unsafe { (*leaf).numLeafBrushes } {
        brushnum = unsafe { (*local).leafbrushes[(*leaf).firstLeafBrush as usize + k as usize] };
        b = unsafe { &mut (*local).brushes[brushnum as usize] };
        if unsafe { (*b).checkcount == (*local).checkcount } {
            k += 1;
            continue; // already checked this brush in another leaf
        }
        unsafe { (*b).checkcount = (*local).checkcount };

        if unsafe { !((*b).contents & (*tw).contents) != 0 } {
            k += 1;
            continue;
        }

        // #ifndef BSPC
        // 	if (com_terrainPhysics->integer && cmg.landScape && (b->contents & CONTENTS_TERRAIN) )
        // 	{
        // 		// Invalidate the checkcount for terrain as the terrain brush has to be processed
        // 		// many times.
        // 		b->checkcount--;
        //
        // 		CM_TraceThroughTerrain( tw, trace, b );
        // 		// If inside a terrain brush don't bother with regular brush collision
        // 		continue;
        // 	}
        // #endif
        CM_TestBoxInBrush(tw, trace, b);
        if unsafe { (*trace).allsolid } {
            return;
        }
        k += 1;
    }

    // test against all patches
    // #ifdef BSPC
    // 	if (1) {
    // #else
    // 	if ( !cm_noCurves->integer ) {
    // #endif //BSPC
    k = 0;
    while k < unsafe { (*leaf).numLeafSurfaces } {
        // //#ifdef _XBOX
        // //		int index = CM_GetSurfaceIndex(leaf->firstLeafSurface + k);
        // //		patch = local->surfaces[ index ];
        // //#else
        patch = unsafe {
            (*local).surfaces[(*local).leafsurfaces[(*leaf).firstLeafSurface as usize + k as usize] as usize]
        };
        // //#endif
        if patch.is_null() {
            k += 1;
            continue;
        }
        if unsafe { (*patch).checkcount == (*local).checkcount } {
            k += 1;
            continue; // already checked this brush in another leaf
        }
        unsafe { (*patch).checkcount = (*local).checkcount };

        if unsafe { !((*patch).contents & (*tw).contents) != 0 } {
            k += 1;
            continue;
        }

        if CM_PositionTestInPatchCollide(tw, unsafe { (*patch).pc }) {
            unsafe {
                (*trace).startsolid = true;
                (*trace).allsolid = true;
                (*trace).fraction = 0.0;
                (*trace).contents = (*patch).contents;
            }
            return;
        }
        k += 1;
    }
}

/*
==================
CM_TestCapsuleInCapsule

capsule inside capsule check
==================
*/
fn CM_TestCapsuleInCapsule(tw: *mut traceWork_t, trace: *mut trace_t, model: clipHandle_t) {
    let mut i: i32;
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut top: [f32; 3] = [0.0; 3];
    let mut bottom: [f32; 3] = [0.0; 3];
    let mut p1: [f32; 3] = [0.0; 3];
    let mut p2: [f32; 3] = [0.0; 3];
    let mut tmp: [f32; 3] = [0.0; 3];
    let mut offset: [f32; 3] = [0.0; 3];
    let mut symetricSize: [[f32; 3]; 2] = [[0.0; 3]; 2];
    let mut radius: f32;
    let mut halfwidth: f32;
    let mut halfheight: f32;
    let mut offs: f32;
    let mut r: f32;

    CM_ModelBounds(model, &mut mins, &mut maxs);

    VectorAdd(&unsafe { (*tw).start }, &unsafe { (*tw).sphere.offset }, &mut top);
    VectorSubtract(&unsafe { (*tw).start }, &unsafe { (*tw).sphere.offset }, &mut bottom);
    i = 0;
    while i < 3 {
        offset[i as usize] = (mins[i as usize] + maxs[i as usize]) * 0.5;
        symetricSize[0][i as usize] = mins[i as usize] - offset[i as usize];
        symetricSize[1][i as usize] = maxs[i as usize] - offset[i as usize];
        i += 1;
    }
    halfwidth = symetricSize[1][0];
    halfheight = symetricSize[1][2];
    radius = if halfwidth > halfheight {
        halfheight
    } else {
        halfwidth
    };
    offs = halfheight - radius;

    r = Square(unsafe { (*tw).sphere.radius } + radius);
    // check if any of the spheres overlap
    VectorCopy(&offset, &mut p1);
    p1[2] += offs;
    VectorSubtract(&p1, &top, &mut tmp);
    if VectorLengthSquared(&tmp) < r {
        unsafe {
            (*trace).startsolid = true;
            (*trace).allsolid = true;
            (*trace).fraction = 0.0;
        }
    }
    VectorSubtract(&p1, &bottom, &mut tmp);
    if VectorLengthSquared(&tmp) < r {
        unsafe {
            (*trace).startsolid = true;
            (*trace).allsolid = true;
            (*trace).fraction = 0.0;
        }
    }
    VectorCopy(&offset, &mut p2);
    p2[2] -= offs;
    VectorSubtract(&p2, &top, &mut tmp);
    if VectorLengthSquared(&tmp) < r {
        unsafe {
            (*trace).startsolid = true;
            (*trace).allsolid = true;
            (*trace).fraction = 0.0;
        }
    }
    VectorSubtract(&p2, &bottom, &mut tmp);
    if VectorLengthSquared(&tmp) < r {
        unsafe {
            (*trace).startsolid = true;
            (*trace).allsolid = true;
            (*trace).fraction = 0.0;
        }
    }
    // if between cylinder up and lower bounds
    if (top[2] >= p1[2] && top[2] <= p2[2]) || (bottom[2] >= p1[2] && bottom[2] <= p2[2]) {
        // 2d coordinates
        top[2] = 0.0;
        p1[2] = 0.0;
        // if the cylinders overlap
        VectorSubtract(&top, &p1, &mut tmp);
        if VectorLengthSquared(&tmp) < r {
            unsafe {
                (*trace).startsolid = true;
                (*trace).allsolid = true;
                (*trace).fraction = 0.0;
            }
        }
    }
}

/*
==================
CM_TestBoundingBoxInCapsule

bounding box inside capsule check
==================
*/
fn CM_TestBoundingBoxInCapsule(tw: *mut traceWork_t, trace: *mut trace_t, model: clipHandle_t) {
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut offset: [f32; 3] = [0.0; 3];
    let mut size: [[f32; 3]; 2] = [[0.0; 3]; 2];
    let mut h: clipHandle_t;
    let mut cmod: *mut cmodel_t;
    let mut i: i32;

    // mins maxs of the capsule
    CM_ModelBounds(model, &mut mins, &mut maxs);

    // offset for capsule center
    i = 0;
    while i < 3 {
        offset[i as usize] = (mins[i as usize] + maxs[i as usize]) * 0.5;
        size[0][i as usize] = mins[i as usize] - offset[i as usize];
        size[1][i as usize] = maxs[i as usize] - offset[i as usize];
        unsafe {
            (*tw).start[i as usize] -= offset[i as usize];
            (*tw).end[i as usize] -= offset[i as usize];
        }
        i += 1;
    }

    // replace the bounding box with the capsule
    unsafe {
        (*tw).sphere.use = true;
        (*tw).sphere.radius = if size[1][0] > size[1][2] {
            size[1][2]
        } else {
            size[1][0]
        };
        (*tw).sphere.halfheight = size[1][2];
        VectorSet(&mut (*tw).sphere.offset, 0.0, 0.0, size[1][2] - (*tw).sphere.radius);
    }

    // replace the capsule with the bounding box
    h = CM_TempBoxModel(&unsafe { (*tw).size[0] }, &unsafe { (*tw).size[1] }, false);
    // calculate collision
    cmod = CM_ClipHandleToModel(h, std::ptr::null_mut());
    CM_TestInLeaf(tw, trace, &unsafe { (*cmod).leaf }, unsafe { &mut cmg });
}

/*
==================
CM_PositionTest
==================
*/
const MAX_POSITION_LEAFS: i32 = 1024;
fn CM_PositionTest(tw: *mut traceWork_t, trace: *mut trace_t) {
    let mut leafs: [i32; 1024] = [0; 1024];
    let mut i: i32;
    let mut ll: leafList_t = leafList_t {
        bounds: [[0.0; 3]; 2],
        count: 0,
        maxcount: 0,
        list: std::ptr::null_mut(),
        storeLeafs: None,
        lastLeaf: 0,
        overflowed: false,
    };

    // identify the leafs we are touching
    VectorAdd(&unsafe { (*tw).start }, &unsafe { (*tw).size[0] }, &mut ll.bounds[0]);
    VectorAdd(&unsafe { (*tw).start }, &unsafe { (*tw).size[1] }, &mut ll.bounds[1]);

    i = 0;
    while i < 3 {
        ll.bounds[0][i as usize] -= 1.0;
        ll.bounds[1][i as usize] += 1.0;
        i += 1;
    }

    ll.count = 0;
    ll.maxcount = MAX_POSITION_LEAFS;
    ll.list = leafs.as_mut_ptr();
    ll.storeLeafs = Some(CM_StoreLeafs);
    ll.lastLeaf = 0;
    ll.overflowed = false;

    unsafe { cmg.checkcount += 1 };

    CM_BoxLeafnums_r(&mut ll, 0);

    unsafe { cmg.checkcount += 1 };

    // test the contents of the leafs
    i = 0;
    while i < ll.count {
        CM_TestInLeaf(tw, trace, &unsafe { (*cmg.leafs.add(leafs[i as usize] as usize)) }, unsafe { &mut cmg });
        if unsafe { (*trace).allsolid } {
            break;
        }
        i += 1;
    }
}

/*
===============================================================================

TRACING

===============================================================================
*/

/*
================
CM_TraceThroughPatch
================
*/

fn CM_TraceThroughPatch(tw: *mut traceWork_t, trace: *mut trace_t, patch: *mut cPatch_t) {
    let mut oldFrac: f32;

    unsafe { c_patch_traces += 1 };

    oldFrac = unsafe { (*trace).fraction };

    CM_TraceThroughPatchCollide(tw, trace, unsafe { (*patch).pc });

    if unsafe { (*trace).fraction < oldFrac } {
        unsafe {
            (*trace).surfaceFlags = (*patch).surfaceFlags;
            (*trace).contents = (*patch).contents;
        }
    }
}

/*
================
CM_PlaneCollision

  Returns false for a quick getout
================
*/

fn CM_PlaneCollision(tw: *mut traceWork_t, side: *mut cbrushside_t) -> bool {
    let mut dist: f32;
    let mut f: f32;
    let mut d1: f32;
    let mut d2: f32;

    // #ifdef _XBOX
    // 	cplane_t		*plane = &cmg.planes[side->planeNum.GetValue()];
    // #else
    let plane: *const cplane_t = unsafe { (*side).plane };
    // #endif

    // adjust the plane distance apropriately for mins/maxs
    dist = unsafe { (*plane).dist - DotProduct(&(*tw).offsets[(*plane).signbits as usize], &(*plane).normal) };

    d1 = DotProduct(&unsafe { (*tw).start }, &unsafe { (*plane).normal }) - dist;
    d2 = DotProduct(&unsafe { (*tw).end }, &unsafe { (*plane).normal }) - dist;

    if d2 > 0.0f32 {
        // endpoint is not in solid
        unsafe { (*tw).getout = true };
    }
    if d1 > 0.0f32 {
        // startpoint is not in solid
        unsafe { (*tw).startout = true };
    }

    // if completely in front of face, no intersection with the entire brush
    if (d1 > 0.0f32) && ((d2 >= SURFACE_CLIP_EPSILON) || (d2 >= d1)) {
        return false;
    }

    // if it doesn't cross the plane, the plane isn't relevent
    if (d1 <= 0.0f32) && (d2 <= 0.0f32) {
        return true;
    }
    // crosses face
    if d1 > d2 {
        // enter
        f = d1 - SURFACE_CLIP_EPSILON;
        if f < 0.0f32 {
            f = 0.0f32;
            if f > unsafe { (*tw).enterFrac } {
                unsafe {
                    (*tw).enterFrac = f;
                    (*tw).clipplane = plane;
                    (*tw).leadside = side;
                }
            }
        } else if f > unsafe { (*tw).enterFrac * (d1 - d2) } {
            unsafe {
                (*tw).enterFrac = f / (d1 - d2);
                (*tw).clipplane = plane;
                (*tw).leadside = side;
            }
        }
    } else {
        // leave
        f = d1 + SURFACE_CLIP_EPSILON;
        if f < (d1 - d2) {
            f = 1.0f32;
            if f < unsafe { (*tw).leaveFrac } {
                unsafe { (*tw).leaveFrac = f };
            }
        } else if f > unsafe { (*tw).leaveFrac * (d1 - d2) } {
            unsafe { (*tw).leaveFrac = f / (d1 - d2) };
        }
    }
    return true;
}

/*
================
CM_TraceThroughBrush
================
*/
fn CM_TraceThroughBrush(tw: *mut traceWork_t, trace: *mut trace_t, brush: *mut cbrush_t, infoOnly: bool) {
    let mut i: i32;
    let mut side: *mut cbrushside_t;

    unsafe {
        (*tw).enterFrac = -1.0f32;
        (*tw).leaveFrac = 1.0f32;
        (*tw).clipplane = std::ptr::null();
    }

    if unsafe { (*brush).numsides == 0 } {
        return;
    }

    // I'm not sure if test is strictly correct.  Are all
    // bboxes axis aligned?  Do I care?  It seems to work
    // good enough...
    if unsafe {
        (*tw).bounds[0][0] > (*brush).bounds[1][0]
            || (*tw).bounds[0][1] > (*brush).bounds[1][1]
            || (*tw).bounds[0][2] > (*brush).bounds[1][2]
            || (*tw).bounds[1][0] < (*brush).bounds[0][0]
            || (*tw).bounds[1][1] < (*brush).bounds[0][1]
            || (*tw).bounds[1][2] < (*brush).bounds[0][2]
    } {
        return;
    }

    unsafe {
        (*tw).getout = false;
        (*tw).startout = false;
        (*tw).leadside = std::ptr::null_mut();
    }

    //
    // compare the trace against all planes of the brush
    // find the latest time the trace crosses a plane towards the interior
    // and the earliest time the trace crosses a plane towards the exterior
    //
    i = 0;
    while i < unsafe { (*brush).numsides } {
        side = unsafe { (*brush).sides.as_mut().unwrap().as_mut_ptr().add(i as usize) };

        if !CM_PlaneCollision(tw, side) {
            return;
        }
        i += 1;
    }

    //
    // all planes have been checked, and the trace was not
    // completely outside the brush
    //
    if !unsafe { (*tw).startout } {
        if !infoOnly {
            // original point was inside brush
            unsafe { (*trace).startsolid = true };
            if !unsafe { (*tw).getout } {
                unsafe {
                    (*trace).allsolid = true;
                    (*trace).fraction = 0.0f32;
                }
            }
        }
        unsafe { (*tw).enterFrac = 0.0f32 };
        return;
    }

    if unsafe { (*tw).enterFrac < (*tw).leaveFrac } {
        if (unsafe { (*tw).enterFrac > -1.0f32 }) && (unsafe { (*tw).enterFrac < (*trace).fraction }) {
            if unsafe { (*tw).enterFrac < 0.0f32 } {
                unsafe { (*tw).enterFrac = 0.0f32 };
            }
            if !infoOnly {
                unsafe {
                    (*trace).fraction = (*tw).enterFrac;
                    (*trace).plane = *(*tw).clipplane;
                    // TODO: cmg.shaders access
                    (*trace).contents = (*brush).contents;
                }
            }
        }
    }
}

/*
================
CM_TraceThroughTerrain

  During this routine the fraction is internal to the brush
  and converted to a global fraction on exit.
================
*/

// #ifndef BSPC

// void CM_TraceThroughTerrain( traceWork_t *tw, trace_t &trace, cbrush_t *brush )
// {
// 	CCMLandScape		*landscape;
// 	vec3_t				tBegin, tEnd, tDistance, tStep;
// 	vec3_t				baseStart;
// 	vec3_t				baseEnd;
// 	int					count;
// 	int					i;
// 	float				fraction;
//
// 	// At this point we know we may be colliding with a terrain brush (and we know we have a valid terrain structure)
// 	landscape = (CCMLandScape *)cmg.landScape;
//
// 	// Check for absolutely no connection
// 	if(!CM_GenericBoxCollide(tw->bounds, landscape->GetBounds()))
// 	{
// 		return;
// 	}
// 	// Now we know that at least some part of the trace needs to collide with the terrain
// 	// The regular brush collision is handled elsewhere, so advance the ray to an edge in the terrain brush
// 	CM_TraceThroughBrush( tw, trace, brush, true );
//
// 	// Remember the base entering and leaving fractions
// 	tw->baseEnterFrac = tw->enterFrac;
// 	tw->baseLeaveFrac = tw->leaveFrac;
// 	// Reset to full spread within the brush
// 	tw->enterFrac = -1.0f;
// 	tw->leaveFrac = 1.0f;
//
// 	// Work out the corners of the AABB when the trace first hits the terrain brush and when it leaves
// 	VectorAdvance(tw->start, tw->baseEnterFrac, tw->end, tBegin);
// 	VectorAdvance(tw->start, tw->baseLeaveFrac, tw->end, tEnd);
// 	VectorSubtract(tEnd, tBegin, tDistance);
//
// 	// Calculate number of iterations to process
// 	count = ceilf(VectorLength(tDistance) / (landscape->GetPatchScalarSize() * TERRAIN_STEP_MAGIC));
// 	count = 1;
// 	fraction = trace.fraction;
// 	VectorScale(tDistance, 1.0f / count, tStep);
//
// 	// Save the base start and end vectors
// 	VectorCopy ( tw->start, baseStart );
// 	VectorCopy ( tw->end, baseEnd );
//
// 	// Use the terrain vectors.  Start both at the beginning since the
// 	// step will be added to the end as the first step of the loop
// 	VectorCopy ( tBegin, tw->start );
// 	VectorCopy ( tBegin, tw->end );
//
// 	// Step thru terrain patches moving on about 1 patch at a time
// 	for ( i = 0; i < count; i ++ )
// 	{
// 		// Add the step to the end
// 		VectorAdd(tw->end, tStep, tw->end);
//
// 		CM_CalcExtents(tBegin, tw->end, tw, tw->localBounds);
//
// 		landscape->PatchCollide(tw, trace, tw->start, tw->end, brush->checkcount);
//
// 		// If collision with something closer than water then just stop here
// 		if ( trace.fraction < fraction )
// 		{
// 			// Convert the fraction of this sub tract into the full trace's fraction
// 			trace.fraction = i * (1.0f / count) + (1.0f / count) * trace.fraction;
// 			break;
// 		}
//
// 		// Move the end to the start so the next trace starts
// 		// where this one left off
// 		VectorCopy(tw->end, tw->start);
// 	}
//
// 	// Put the original start and end back
// 	VectorCopy ( baseStart, tw->start );
// 	VectorCopy ( baseEnd, tw->end );
//
// 	// Convert to global fraction only if something was hit along the way
// 	if ( trace.fraction != 1.0 )
// 	{
// 		trace.fraction = tw->baseEnterFrac + ((tw->baseLeaveFrac - tw->baseEnterFrac) * trace.fraction);
// 		trace.contents = brush->contents;
// 	}
//
// 	// Collide with any water
// 	if ( tw->contents & CONTENTS_WATER )
// 	{
// 		fraction = landscape->WaterCollide(tw->start, tw->end, trace.fraction);
// 		if( fraction < trace.fraction )
// 		{
// 			VectorSet(trace.plane.normal, 0.0f, 0.0f, 1.0f);
// 			trace.contents = landscape->GetWaterContents();
// 			trace.fraction = fraction;
// 			trace.surfaceFlags = landscape->GetWaterSurfaceFlags();
// 		}
// 	}
// }

// #ifdef TEST_TERRAIN_PHYSICS
//
// void CM_TraceThroughTerrain( traceWork_t *tw, trace_t &trace, CCMLandScape *landscape)
// {
// 	vec3_t				tBegin, tEnd, tDistance, tStep;
// 	vec3_t				baseStart;
// 	vec3_t				baseEnd;
// 	int					count;
// 	int					i;
// 	float				fraction;
//
// 	// Check for absolutely no connection
// 	if(!CM_GenericBoxCollide(tw->bounds, landscape->GetBounds()))
// 	{
// 		return;
// 	}
//
// 	tw->enterFrac = 0.0f;
// 	tw->leaveFrac = 1.0f;
// 	tw->clipplane = NULL;
// 	tw->getout = false;
// 	tw->startout = false;
// 	tw->leadside = NULL;
//
// 	// Remember the base entering and leaving fractions
// 	tw->baseEnterFrac = tw->enterFrac;
// 	tw->baseLeaveFrac = tw->leaveFrac;
// 	// Reset to full spread within the brush
// 	tw->enterFrac = -1.0f;
// 	tw->leaveFrac = 1.0f;
//
// 	// Work out the corners of the AABB when the trace first hits the terrain brush and when it leaves
// 	VectorAdvance(tw->start, tw->baseEnterFrac, tw->end, tBegin);
// 	VectorAdvance(tw->start, tw->baseLeaveFrac, tw->end, tEnd);
// 	VectorSubtract(tEnd, tBegin, tDistance);
//
// 	// Calculate number of iterations to process
// 	count = ceilf(VectorLength(tDistance) / (landscape->GetPatchScalarSize() * TERRAIN_STEP_MAGIC));
// 	count = 1;
// 	fraction = trace.fraction;
// 	VectorScale(tDistance, 1.0f / count, tStep);
//
// 	// Save the base start and end vectors
// 	VectorCopy ( tw->start, baseStart );
// 	VectorCopy ( tw->end, baseEnd );
//
// 	// Use the terrain vectors.  Start both at the beginning since the
// 	// step will be added to the end as the first step of the loop
// 	VectorCopy ( tBegin, tw->start );
// 	VectorCopy ( tBegin, tw->end );
//
// 	// Step thru terrain patches moving on about 1 patch at a time
// 	for ( i = 0; i < count; i ++ )
// 	{
// 		// Add the step to the end
// 		VectorAdd(tw->end, tStep, tw->end);
//
// 		CM_CalcExtents(tBegin, tw->end, tw, tw->localBounds);
//
// 		landscape->PatchCollide(tw, trace, tw->start, tw->end, cmg.checkcount);
//
// 		// If collision with something closer than water then just stop here
// 		if ( trace.fraction < fraction )
// 		{
// 			// Convert the fraction of this sub tract into the full trace's fraction
// 			trace.fraction = i * (1.0f / count) + (1.0f / count) * trace.fraction;
// 			break;
// 		}
//
// 		// Move the end to the start so the next trace starts
// 		// where this one left off
// 		VectorCopy(tw->end, tw->start);
// 	}
//
// 	// Put the original start and end back
// 	VectorCopy ( baseStart, tw->start );
// 	VectorCopy ( baseEnd, tw->end );
//
// 	// Convert to global fraction only if something was hit along the way
// 	if ( trace.fraction != 1.0 )
// 	{
// //		trace.fraction = tw->baseEnterFrac + ((tw->baseLeaveFrac - tw->baseEnterFrac) * trace.fraction);
// 		trace.contents = CONTENTS_TERRAIN | CONTENTS_OUTSIDE;
// 	}
//
// 	// Collide with any water
// 	if ( tw->contents & CONTENTS_WATER )
// 	{
// 		fraction = landscape->WaterCollide(tw->start, tw->end, trace.fraction);
// 		if( fraction < trace.fraction )
// 		{
// 			VectorSet(trace.plane.normal, 0.0f, 0.0f, 1.0f);
// 			trace.contents = landscape->GetWaterContents();
// 			trace.fraction = fraction;
// 			trace.surfaceFlags = landscape->GetWaterSurfaceFlags();
// 		}
// 	}
// }
//
// #endif // #ifdef TEST_TERRAIN_PHYSICS

// #endif

/*
================
CM_PatchCollide

  By the time we get here we know the AABB is within the patch AABB ie there is a chance of collision
  The collision data is made up of bounds, 2 triangle planes
  There is an BB check for the terxel check to see if it is worth checking the planes.
  Collide with both triangles to find the shortest fraction
================
*/

fn CM_HandlePatchCollision(
    tw: *mut traceWork_s,
    trace: *mut trace_t,
    tStart: &[f32; 3],
    tEnd: &[f32; 3],
    patch: *mut CCMPatch,
    checkcount: i32,
) {
    let mut numBrushes: i32;
    let mut i: i32;
    let mut brush: *mut cbrush_t;

    // Get the collision data
    brush = unsafe { (*patch).GetCollisionData() };
    numBrushes = unsafe { (*patch).GetNumBrushes() };

    i = 0;
    while i < numBrushes {
        if unsafe { (*brush).checkcount == checkcount } {
            return;
        }

        // Generic collision of terxel bounds to line segment bounds
        if !CM_GenericBoxCollide(&unsafe { (*brush).bounds }, &unsafe { (*tw).localBounds }) {
            unsafe { brush = brush.add(1) };
            i += 1;
            continue;
        }

        unsafe { (*brush).checkcount = checkcount };

        CM_TraceThroughBrush(tw, trace, brush, false);
        if unsafe { (*trace).fraction <= 0.0 } {
            break;
        }
        unsafe { brush = brush.add(1) };
        i += 1;
    }
}

/*
================
CM_GenericBoxCollide
================
*/

fn CM_GenericBoxCollide(abounds: &vec3pair_t, bbounds: &vec3pair_t) -> bool {
    let mut i: i32;

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
    return true;
}

/*
================
CM_TraceThroughLeaf
================
*/
fn CM_TraceThroughLeaf(tw: *mut traceWork_t, trace: *mut trace_t, local: *mut clipMap_t, leaf: *const cLeaf_t) {
    let mut k: i32;
    let mut brushnum: i32;
    let mut b: *mut cbrush_t;
    let mut patch: *mut cPatch_t;

    // trace line against all brushes in the leaf
    k = 0;
    while k < unsafe { (*leaf).numLeafBrushes } {
        brushnum = unsafe { (*local).leafbrushes[(*leaf).firstLeafBrush as usize + k as usize] };

        b = unsafe { &mut (*local).brushes[brushnum as usize] };
        if unsafe { (*b).checkcount == (*local).checkcount } {
            k += 1;
            continue; // already checked this brush in another leaf
        }
        unsafe { (*b).checkcount = (*local).checkcount };

        if unsafe { !((*b).contents & (*tw).contents) != 0 } {
            k += 1;
            continue;
        }

        // #ifndef BSPC
        // 	if (com_terrainPhysics->integer && cmg.landScape && (b->contents & CONTENTS_TERRAIN) )
        // 	{
        // 		// Invalidate the checkcount for terrain as the terrain brush has to be processed
        // 		// many times.
        // 		b->checkcount--;
        //
        // 		CM_TraceThroughTerrain( tw, trace, b );
        // 	}
        // 	else
        // #endif
        // 	{
        CM_TraceThroughBrush(tw, trace, b, false);
        // 	}

        if unsafe { (*trace).fraction == 0.0 } {
            return;
        }
        k += 1;
    }

    // trace line against all patches in the leaf
    // #ifdef BSPC
    // 	if (1) {
    // #else
    // 	if ( !cm_noCurves->integer ) {
    // #endif
    k = 0;
    while k < unsafe { (*leaf).numLeafSurfaces } {
        // //#ifdef _XBOX
        // //		int index = CM_GetSurfaceIndex(leaf->firstLeafSurface + k);
        // //		patch = local->surfaces[ index ];
        // //#else
        patch = unsafe {
            (*local).surfaces[(*local).leafsurfaces[(*leaf).firstLeafSurface as usize + k as usize] as usize]
        };
        // //#endif
        if patch.is_null() {
            k += 1;
            continue;
        }
        if unsafe { (*patch).checkcount == (*local).checkcount } {
            k += 1;
            continue; // already checked this patch in another leaf
        }
        unsafe { (*patch).checkcount = (*local).checkcount };

        if unsafe { !((*patch).contents & (*tw).contents) != 0 } {
            k += 1;
            continue;
        }

        CM_TraceThroughPatch(tw, trace, patch);
        if unsafe { (*trace).fraction == 0.0 } {
            return;
        }
        k += 1;
    }
}

const RADIUS_EPSILON: f32 = 1.0f32;

/*
================
CM_TraceThroughSphere

get the first intersection of the ray with the sphere
================
*/
fn CM_TraceThroughSphere(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    origin: &[f32; 3],
    radius: f32,
    start: &[f32; 3],
    end: &[f32; 3],
) {
    let mut l1: f32;
    let mut l2: f32;
    let mut length: f32;
    let mut scale: f32;
    let mut fraction: f32;
    let mut a: f32;
    let mut b: f32;
    let mut c: f32;
    let mut d: f32;
    let mut sqrtd: f32;
    let mut v1: [f32; 3] = [0.0; 3];
    let mut dir: [f32; 3] = [0.0; 3];
    let mut intersection: [f32; 3] = [0.0; 3];

    // if inside the sphere
    VectorSubtract(start, origin, &mut dir);
    l1 = VectorLengthSquared(&dir);
    if l1 < Square(radius) {
        unsafe { (*trace).fraction = 0.0 };
        unsafe { (*trace).startsolid = true };
        // test for allsolid
        VectorSubtract(end, origin, &mut dir);
        l1 = VectorLengthSquared(&dir);
        if l1 < Square(radius) {
            unsafe { (*trace).allsolid = true };
        }
        return;
    }
    //
    VectorSubtract(end, start, &mut dir);
    length = VectorNormalize(&mut dir);
    //
    l1 = CM_DistanceFromLineSquared(origin, start, end, &dir);
    VectorSubtract(end, origin, &mut v1);
    l2 = VectorLengthSquared(&v1);
    // if no intersection with the sphere and the end point is at least an epsilon away
    if l1 >= Square(radius) && l2 > Square(radius + SURFACE_CLIP_EPSILON) {
        return;
    }
    //
    //	| origin - (start + t * dir) | = radius
    //	a = dir[0]^2 + dir[1]^2 + dir[2]^2;
    //	b = 2 * (dir[0] * (start[0] - origin[0]) + dir[1] * (start[1] - origin[1]) + dir[2] * (start[2] - origin[2]));
    //	c = (start[0] - origin[0])^2 + (start[1] - origin[1])^2 + (start[2] - origin[2])^2 - radius^2;
    //
    VectorSubtract(start, origin, &mut v1);
    // dir is normalized so a = 1
    a = 1.0f32;
    b = 2.0f32 * (dir[0] * v1[0] + dir[1] * v1[1] + dir[2] * v1[2]);
    c = v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2] - (radius + RADIUS_EPSILON) * (radius + RADIUS_EPSILON);

    d = b * b - 4.0f32 * c;
    if d > 0.0 {
        sqrtd = SquareRootFloat(d);
        // = (- b + sqrtd) * 0.5f; // / (2.0f * a);
        fraction = (-b - sqrtd) * 0.5f32; // / (2.0f * a);
        //
        if fraction < 0.0 {
            fraction = 0.0;
        } else {
            fraction /= length;
        }
        if fraction < unsafe { (*trace).fraction } {
            unsafe { (*trace).fraction = fraction };
            VectorSubtract(end, start, &mut dir);
            VectorMA(start, fraction, &dir, &mut intersection);
            VectorSubtract(&intersection, origin, &mut dir);
            // #ifdef CAPSULE_DEBUG
            // 	l2 = VectorLength(dir);
            // 	if (l2 < radius) {
            // 		int bah = 1;
            // 	}
            // #endif
            scale = 1.0 / (radius + RADIUS_EPSILON);
            VectorScale(&dir, scale, &mut dir);
            VectorCopy(&dir, &mut unsafe { (*trace).plane.normal });
            VectorAdd(&unsafe { (*tw).modelOrigin }, &intersection, &mut intersection);
            unsafe { (*trace).plane.dist = DotProduct(&(*trace).plane.normal, &intersection) };
            unsafe { (*trace).contents = CONTENTS_BODY };
        }
    } else if d == 0.0 {
        //t1 = (- b ) / 2;
        // slide along the sphere
    }
    // no intersection at all
}

/*
================
CM_TraceThroughVerticalCylinder

get the first intersection of the ray with the cylinder
the cylinder extends halfheight above and below the origin
================
*/
fn CM_TraceThroughVerticalCylinder(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    origin: &[f32; 3],
    radius: f32,
    halfheight: f32,
    start: &[f32; 3],
    end: &[f32; 3],
) {
    let mut length: f32;
    let mut scale: f32;
    let mut fraction: f32;
    let mut l1: f32;
    let mut l2: f32;
    let mut a: f32;
    let mut b: f32;
    let mut c: f32;
    let mut d: f32;
    let mut sqrtd: f32;
    let mut v1: [f32; 3] = [0.0; 3];
    let mut dir: [f32; 3] = [0.0; 3];
    let mut start2d: [f32; 3] = [0.0; 3];
    let mut end2d: [f32; 3] = [0.0; 3];
    let mut org2d: [f32; 3] = [0.0; 3];
    let mut intersection: [f32; 3] = [0.0; 3];

    // 2d coordinates
    VectorSet(&mut start2d, start[0], start[1], 0.0);
    VectorSet(&mut end2d, end[0], end[1], 0.0);
    VectorSet(&mut org2d, origin[0], origin[1], 0.0);
    // if between lower and upper cylinder bounds
    if start[2] <= origin[2] + halfheight && start[2] >= origin[2] - halfheight {
        // if inside the cylinder
        VectorSubtract(&start2d, &org2d, &mut dir);
        l1 = VectorLengthSquared(&dir);
        if l1 < Square(radius) {
            unsafe { (*trace).fraction = 0.0 };
            unsafe { (*trace).startsolid = true };
            VectorSubtract(&end2d, &org2d, &mut dir);
            l1 = VectorLengthSquared(&dir);
            if l1 < Square(radius) {
                unsafe { (*trace).allsolid = true };
            }
            return;
        }
    }
    //
    VectorSubtract(&end2d, &start2d, &mut dir);
    length = VectorNormalize(&mut dir);
    //
    l1 = CM_DistanceFromLineSquared(&org2d, &start2d, &end2d, &dir);
    VectorSubtract(&end2d, &org2d, &mut v1);
    l2 = VectorLengthSquared(&v1);
    // if no intersection with the cylinder and the end point is at least an epsilon away
    if l1 >= Square(radius) && l2 > Square(radius + SURFACE_CLIP_EPSILON) {
        return;
    }
    //
    //
    // (start[0] - origin[0] - t * dir[0]) ^ 2 + (start[1] - origin[1] - t * dir[1]) ^ 2 = radius ^ 2
    // (v1[0] + t * dir[0]) ^ 2 + (v1[1] + t * dir[1]) ^ 2 = radius ^ 2;
    // v1[0] ^ 2 + 2 * v1[0] * t * dir[0] + (t * dir[0]) ^ 2 +
    //						v1[1] ^ 2 + 2 * v1[1] * t * dir[1] + (t * dir[1]) ^ 2 = radius ^ 2
    // t ^ 2 * (dir[0] ^ 2 + dir[1] ^ 2) + t * (2 * v1[0] * dir[0] + 2 * v1[1] * dir[1]) +
    //						v1[0] ^ 2 + v1[1] ^ 2 - radius ^ 2 = 0
    //
    VectorSubtract(start, origin, &mut v1);
    // dir is normalized so we can use a = 1
    a = 1.0f32;
    b = 2.0f32 * (v1[0] * dir[0] + v1[1] * dir[1]);
    c = v1[0] * v1[0] + v1[1] * v1[1] - (radius + RADIUS_EPSILON) * (radius + RADIUS_EPSILON);

    d = b * b - 4.0f32 * c;
    if d > 0.0 {
        sqrtd = SquareRootFloat(d);
        // = (- b + sqrtd) * 0.5f;// / (2.0f * a);
        fraction = (-b - sqrtd) * 0.5f32; // / (2.0f * a);
        //
        if fraction < 0.0 {
            fraction = 0.0;
        } else {
            fraction /= length;
        }
        if fraction < unsafe { (*trace).fraction } {
            VectorSubtract(end, start, &mut dir);
            VectorMA(start, fraction, &dir, &mut intersection);
            // if the intersection is between the cylinder lower and upper bound
            if intersection[2] <= origin[2] + halfheight && intersection[2] >= origin[2] - halfheight {
                //
                unsafe { (*trace).fraction = fraction };
                VectorSubtract(&intersection, origin, &mut dir);
                dir[2] = 0.0;
                // #ifdef CAPSULE_DEBUG
                // 	l2 = VectorLength(dir);
                // 	if (l2 <= radius) {
                // 		int bah = 1;
                // 	}
                // #endif
                scale = 1.0 / (radius + RADIUS_EPSILON);
                VectorScale(&dir, scale, &mut dir);
                VectorCopy(&dir, &mut unsafe { (*trace).plane.normal });
                VectorAdd(&unsafe { (*tw).modelOrigin }, &intersection, &mut intersection);
                unsafe { (*trace).plane.dist = DotProduct(&(*trace).plane.normal, &intersection) };
                unsafe { (*trace).contents = CONTENTS_BODY };
            }
        }
    } else if d == 0.0 {
        //t[0] = (- b ) / 2 * a;
        // slide along the cylinder
    }
    // no intersection at all
}

/*
================
CM_TraceCapsuleThroughCapsule

capsule vs. capsule collision (not rotated)
================
*/
fn CM_TraceCapsuleThroughCapsule(tw: *mut traceWork_t, trace: *mut trace_t, model: clipHandle_t) {
    let mut i: i32;
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut top: [f32; 3] = [0.0; 3];
    let mut bottom: [f32; 3] = [0.0; 3];
    let mut starttop: [f32; 3] = [0.0; 3];
    let mut startbottom: [f32; 3] = [0.0; 3];
    let mut endtop: [f32; 3] = [0.0; 3];
    let mut endbottom: [f32; 3] = [0.0; 3];
    let mut offset: [f32; 3] = [0.0; 3];
    let mut symetricSize: [[f32; 3]; 2] = [[0.0; 3]; 2];
    let mut radius: f32;
    let mut halfwidth: f32;
    let mut halfheight: f32;
    let mut offs: f32;
    let mut h: f32;

    CM_ModelBounds(model, &mut mins, &mut maxs);
    // test trace bounds vs. capsule bounds
    if unsafe {
        (*tw).bounds[0][0] > maxs[0] + RADIUS_EPSILON
            || (*tw).bounds[0][1] > maxs[1] + RADIUS_EPSILON
            || (*tw).bounds[0][2] > maxs[2] + RADIUS_EPSILON
            || (*tw).bounds[1][0] < mins[0] - RADIUS_EPSILON
            || (*tw).bounds[1][1] < mins[1] - RADIUS_EPSILON
            || (*tw).bounds[1][2] < mins[2] - RADIUS_EPSILON
    } {
        return;
    }
    // top origin and bottom origin of each sphere at start and end of trace
    VectorAdd(&unsafe { (*tw).start }, &unsafe { (*tw).sphere.offset }, &mut starttop);
    VectorSubtract(&unsafe { (*tw).start }, &unsafe { (*tw).sphere.offset }, &mut startbottom);
    VectorAdd(&unsafe { (*tw).end }, &unsafe { (*tw).sphere.offset }, &mut endtop);
    VectorSubtract(&unsafe { (*tw).end }, &unsafe { (*tw).sphere.offset }, &mut endbottom);

    // calculate top and bottom of the capsule spheres to collide with
    i = 0;
    while i < 3 {
        offset[i as usize] = (mins[i as usize] + maxs[i as usize]) * 0.5;
        symetricSize[0][i as usize] = mins[i as usize] - offset[i as usize];
        symetricSize[1][i as usize] = maxs[i as usize] - offset[i as usize];
        i += 1;
    }
    halfwidth = symetricSize[1][0];
    halfheight = symetricSize[1][2];
    radius = if halfwidth > halfheight {
        halfheight
    } else {
        halfwidth
    };
    offs = halfheight - radius;
    VectorCopy(&offset, &mut top);
    top[2] += offs;
    VectorCopy(&offset, &mut bottom);
    bottom[2] -= offs;
    // expand radius of spheres
    radius += unsafe { (*tw).sphere.radius };
    // if there is horizontal movement
    if unsafe { (*tw).start[0] != (*tw).end[0] || (*tw).start[1] != (*tw).end[1] } {
        // height of the expanded cylinder is the height of both cylinders minus the radius of both spheres
        h = halfheight + unsafe { (*tw).sphere.halfheight } - radius;
        // if the cylinder has a height
        if h > 0.0 {
            // test for collisions between the cylinders
            CM_TraceThroughVerticalCylinder(tw, trace, &offset, radius, h, &unsafe { (*tw).start }, &unsafe { (*tw).end });
        }
    }
    // test for collision between the spheres
    CM_TraceThroughSphere(tw, trace, &top, radius, &startbottom, &endbottom);
    CM_TraceThroughSphere(tw, trace, &bottom, radius, &starttop, &endtop);
}

/*
================
CM_TraceBoundingBoxThroughCapsule

bounding box vs. capsule collision
================
*/
fn CM_TraceBoundingBoxThroughCapsule(tw: *mut traceWork_t, trace: *mut trace_t, model: clipHandle_t) {
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut offset: [f32; 3] = [0.0; 3];
    let mut size: [[f32; 3]; 2] = [[0.0; 3]; 2];
    let mut h: clipHandle_t;
    let mut cmod: *mut cmodel_t;
    let mut i: i32;

    // mins maxs of the capsule
    CM_ModelBounds(model, &mut mins, &mut maxs);

    // offset for capsule center
    i = 0;
    while i < 3 {
        offset[i as usize] = (mins[i as usize] + maxs[i as usize]) * 0.5;
        size[0][i as usize] = mins[i as usize] - offset[i as usize];
        size[1][i as usize] = maxs[i as usize] - offset[i as usize];
        unsafe {
            (*tw).start[i as usize] -= offset[i as usize];
            (*tw).end[i as usize] -= offset[i as usize];
        }
        i += 1;
    }

    // replace the bounding box with the capsule
    unsafe {
        (*tw).sphere.use = true;
        (*tw).sphere.radius = if size[1][0] > size[1][2] {
            size[1][2]
        } else {
            size[1][0]
        };
        (*tw).sphere.halfheight = size[1][2];
        VectorSet(&mut (*tw).sphere.offset, 0.0, 0.0, size[1][2] - (*tw).sphere.radius);
    }

    // replace the capsule with the bounding box
    h = CM_TempBoxModel(&size[0], &size[1], false);
    // calculate collision
    cmod = CM_ClipHandleToModel(h, std::ptr::null_mut());
    CM_TraceThroughLeaf(tw, trace, unsafe { &mut cmg }, &unsafe { (*cmod).leaf });
}

//=========================================================================================

/*
================
CM_TraceToLeaf
================
*/
fn CM_TraceToLeaf(tw: *mut traceWork_t, trace: *mut trace_t, leaf: *const cLeaf_t, local: *mut clipMap_t) {
    let mut k: i32;
    let mut brushnum: i32;
    let mut b: *mut cbrush_t;
    let mut patch: *mut cPatch_t;

    // trace line against all brushes in the leaf
    k = 0;
    while k < unsafe { (*leaf).numLeafBrushes } {
        brushnum = unsafe { (*local).leafbrushes[(*leaf).firstLeafBrush as usize + k as usize] };

        b = unsafe { &mut (*local).brushes[brushnum as usize] };
        if unsafe { (*b).checkcount == (*local).checkcount } {
            k += 1;
            continue; // already checked this brush in another leaf
        }
        unsafe { (*b).checkcount = (*local).checkcount };

        if unsafe { !((*b).contents & (*tw).contents) != 0 } {
            k += 1;
            continue;
        }

        // #ifndef BSPC
        // 	if ( com_terrainPhysics->integer && cmg.landScape && (b->contents & CONTENTS_TERRAIN) )
        // 	{
        // 		// Invalidate the checkcount for terrain as the terrain brush has to be processed
        // 		// many times.
        // 		b->checkcount--;
        //
        // 		CM_TraceThroughTerrain( tw, trace, b );
        // 		// If inside a terrain brush don't bother with regular brush collision
        // 		continue;
        // 	}
        // #endif

        CM_TraceThroughBrush(tw, trace, b, false);
        if unsafe { (*trace).fraction == 0.0 } {
            return;
        }
        k += 1;
    }

    // trace line against all patches in the leaf
    // #ifdef BSPC
    // 	if (1) {
    // #else
    // 	if ( !cm_noCurves->integer ) {
    // #endif
    k = 0;
    while k < unsafe { (*leaf).numLeafSurfaces } {
        patch = unsafe {
            (*local).surfaces[(*local).leafsurfaces[(*leaf).firstLeafSurface as usize + k as usize] as usize]
        };
        if patch.is_null() {
            k += 1;
            continue;
        }
        if unsafe { (*patch).checkcount == (*local).checkcount } {
            k += 1;
            continue; // already checked this patch in another leaf
        }
        unsafe { (*patch).checkcount = (*local).checkcount };

        if unsafe { !((*patch).contents & (*tw).contents) != 0 } {
            k += 1;
            continue;
        }

        CM_TraceThroughPatch(tw, trace, patch);
        if unsafe { (*trace).fraction == 0.0 } {
            return;
        }
        k += 1;
    }
}

/*
==================
CM_TraceThroughTree

Traverse all the contacted leafs from the start to the end position.
If the trace is a point, they will be exactly in order, but for larger
trace volumes it is possible to hit something in a later leaf with
a smaller intercept fraction.
==================
*/
fn CM_TraceThroughTree(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    local: *mut clipMap_t,
    num: i32,
    p1f: f32,
    p2f: f32,
    p1: &[f32; 3],
    p2: &[f32; 3],
) {
    let mut node: *mut cNode_t;
    let mut plane: *const cplane_t;
    let mut t1: f32;
    let mut t2: f32;
    let mut offset: f32;
    let mut frac: f32;
    let mut frac2: f32;
    let mut idist: f32;
    let mut mid: [f32; 3] = [0.0; 3];
    let mut side: i32;
    let mut midf: f32;

    if unsafe { (*trace).fraction <= p1f } {
        return; // already hit something nearer
    }

    // if < 0, we are in a leaf node
    if num < 0 {
        CM_TraceToLeaf(tw, trace, unsafe { &(*local).leafs[-1 - num as usize] }, local);
        return;
    }

    //
    // find the point distances to the seperating plane
    // and the offset for the size of the box
    //
    node = unsafe { &mut (*local).nodes[num as usize] };
    // #ifdef _XBOX
    // 	plane = cmg.planes + node->planeNum;//tr.world->nodes[num].planeNum;
    // #else
    plane = unsafe { (*node).plane };
    // #endif

    // adjust the plane distance apropriately for mins/maxs
    if unsafe { (*plane).type_val } < 3 {
        t1 = p1[unsafe { (*plane).type_val } as usize] - unsafe { (*plane).dist };
        t2 = p2[unsafe { (*plane).type_val } as usize] - unsafe { (*plane).dist };
        offset = unsafe { (*tw).extents[(*plane).type_val as usize] };
    } else {
        t1 = DotProduct(plane, p1) - unsafe { (*plane).dist };
        t2 = DotProduct(plane, p2) - unsafe { (*plane).dist };
        if unsafe { (*tw).isPoint } {
            offset = 0.0;
        } else {
            // #if 0 // bk010201 - DEAD
            // 	// an axial brush right behind a slanted bsp plane
            // 	// will poke through when expanded, so adjust
            // 	// by sqrt(3)
            // 	offset = fabs(tw->extents[0]*plane->normal[0]) +
            // 		fabs(tw->extents[1]*plane->normal[1]) +
            // 		fabs(tw->extents[2]*plane->normal[2]);
            //
            // 	offset *= 2;
            // 	offset = tw->maxOffset;
            // #endif
            // this is silly
            offset = 2048.0;
        }
    }

    // see which sides we need to consider
    if t1 >= offset + 1.0 && t2 >= offset + 1.0 {
        CM_TraceThroughTree(tw, trace, local, unsafe { (*node).children[0] }, p1f, p2f, p1, p2);
        return;
    }
    if t1 < -offset - 1.0 && t2 < -offset - 1.0 {
        CM_TraceThroughTree(tw, trace, local, unsafe { (*node).children[1] }, p1f, p2f, p1, p2);
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

    mid[0] = p1[0] + frac * (p2[0] - p1[0]);
    mid[1] = p1[1] + frac * (p2[1] - p1[1]);
    mid[2] = p1[2] + frac * (p2[2] - p1[2]);

    CM_TraceThroughTree(tw, trace, local, unsafe { (*node).children[side as usize] }, p1f, midf, p1, &mid);

    // go past the node
    if frac2 < 0.0 {
        frac2 = 0.0;
    }
    if frac2 > 1.0 {
        frac2 = 1.0;
    }

    midf = p1f + (p2f - p1f) * frac2;

    mid[0] = p1[0] + frac2 * (p2[0] - p1[0]);
    mid[1] = p1[1] + frac2 * (p2[1] - p1[1]);
    mid[2] = p1[2] + frac2 * (p2[2] - p1[2]);

    CM_TraceThroughTree(tw, trace, local, unsafe { (*node).children[(side ^ 1) as usize] }, midf, p2f, &mid, p2);
}

fn CM_CalcExtents(start: &[f32; 3], end: &[f32; 3], tw: *const traceWork_t, bounds: &mut vec3pair_t) {
    let mut i: i32;

    i = 0;
    while i < 3 {
        if start[i as usize] < end[i as usize] {
            bounds[0][i as usize] = start[i as usize] + unsafe { (*tw).size[0][i as usize] };
            bounds[1][i as usize] = end[i as usize] + unsafe { (*tw).size[1][i as usize] };
        } else {
            bounds[0][i as usize] = end[i as usize] + unsafe { (*tw).size[0][i as usize] };
            bounds[1][i as usize] = start[i as usize] + unsafe { (*tw).size[1][i as usize] };
        }
        i += 1;
    }
}

//======================================================================

/*
==================
CM_Trace
==================
*/
fn CM_Trace(
    trace: *mut trace_t,
    start: &[f32; 3],
    end: &[f32; 3],
    mins: *const [f32; 3],
    maxs: *const [f32; 3],
    model: clipHandle_t,
    origin: &[f32; 3],
    brushmask: i32,
    capsule: i32,
    sphere: *const sphere_t,
) {
    let mut i: i32;
    let mut tw: traceWork_t = traceWork_t {
        contents: 0,
        isPoint: false,
        bounds: [[0.0; 3]; 2],
        size: [[0.0; 3]; 2],
        offsets: [[0.0; 3]; 8],
        start: [0.0; 3],
        end: [0.0; 3],
        extents: [0.0; 3],
        sphere: sphere_t {
            use_: false,
            radius: 0.0,
            halfheight: 0.0,
            offset: [0.0; 3],
        },
        maxOffset: 0.0,
        modelOrigin: [0.0; 3],
        enterFrac: 0.0,
        leaveFrac: 0.0,
        clipplane: std::ptr::null(),
        leadside: std::ptr::null_mut(),
        getout: false,
        startout: false,
        baseEnterFrac: 0.0,
        baseLeaveFrac: 0.0,
        localBounds: [[0.0; 3]; 2],
    };
    let mut offset: [f32; 3] = [0.0; 3];
    let mut cmod: *mut cmodel_t;
    let mut local: *mut clipMap_t = std::ptr::null_mut();

    cmod = CM_ClipHandleToModel(model, &mut local);

    unsafe {
        (*local).checkcount += 1; // for multi-check avoidance

        c_traces += 1; // for statistics, may be zeroed
    }

    // fill in a default trace
    unsafe {
        (*trace).fraction = 1.0; // assume it goes the entire distance until shown otherwise
        VectorCopy(origin, &mut (*tw).modelOrigin);
    }

    if unsafe { (*local).numNodes == 0 } {
        return; // map not loaded, shouldn't happen
    }

    // allow NULL to be passed in for 0,0,0
    let mins_ref: &[f32; 3] = if mins.is_null() {
        &vec3_origin
    } else {
        unsafe { &*mins }
    };
    let maxs_ref: &[f32; 3] = if maxs.is_null() {
        &vec3_origin
    } else {
        unsafe { &*maxs }
    };

    // set basic parms
    tw.contents = brushmask;

    // adjust so that mins and maxs are always symetric, which
    // avoids some complications with plane expanding of rotated
    // bmodels
    i = 0;
    while i < 3 {
        offset[i as usize] = (mins_ref[i as usize] + maxs_ref[i as usize]) * 0.5;
        tw.size[0][i as usize] = mins_ref[i as usize] - offset[i as usize];
        tw.size[1][i as usize] = maxs_ref[i as usize] - offset[i as usize];
        tw.start[i as usize] = start[i as usize] + offset[i as usize];
        tw.end[i as usize] = end[i as usize] + offset[i as usize];
        i += 1;
    }

    // if a sphere is already specified
    if !sphere.is_null() {
        tw.sphere = unsafe { *sphere };
    } else {
        tw.sphere.use = capsule != 0;
        tw.sphere.radius = if tw.size[1][0] > tw.size[1][2] {
            tw.size[1][2]
        } else {
            tw.size[1][0]
        };
        tw.sphere.halfheight = tw.size[1][2];
        VectorSet(&mut tw.sphere.offset, 0.0, 0.0, tw.size[1][2] - tw.sphere.radius);
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
    if tw.sphere.use {
        i = 0;
        while i < 3 {
            if tw.start[i as usize] < tw.end[i as usize] {
                tw.bounds[0][i as usize] = tw.start[i as usize] - tw.sphere.offset[i as usize].abs() - tw.sphere.radius;
                tw.bounds[1][i as usize] = tw.end[i as usize] + tw.sphere.offset[i as usize].abs() + tw.sphere.radius;
            } else {
                tw.bounds[0][i as usize] = tw.end[i as usize] - tw.sphere.offset[i as usize].abs() - tw.sphere.radius;
                tw.bounds[1][i as usize] = tw.start[i as usize] + tw.sphere.offset[i as usize].abs() + tw.sphere.radius;
            }
            i += 1;
        }
    } else {
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
    }

    //
    // check for position test special case
    //
    if start[0] == end[0] && start[1] == end[1] && start[2] == end[2]
        && tw.size[0][0] == 0.0 && tw.size[0][1] == 0.0 && tw.size[0][2] == 0.0
    {
        if !model.is_null() && unsafe { (*cmod).firstNode == -1 } {
            // #ifdef ALWAYS_BBOX_VS_BBOX // bk010201 - FIXME - compile time flag?
            // 	if ( model == BOX_MODEL_HANDLE || model == CAPSULE_MODEL_HANDLE)
            // 	{
            // 		tw.sphere.use = qfalse;
            // 		CM_TestInLeaf( &tw, &cmod->leaf );
            // 	}
            // 	else
            // #elif defined(ALWAYS_CAPSULE_VS_CAPSULE)
            // 	if ( model == BOX_MODEL_HANDLE || model == CAPSULE_MODEL_HANDLE)
            // 	{
            // 		CM_TestCapsuleInCapsule( &tw, model );
            // 	}
            // 	else
            // #endif
            if model == unsafe { BOX_MODEL_HANDLE } || model == unsafe { CAPSULE_MODEL_HANDLE } {
                if tw.sphere.use {
                    CM_TestCapsuleInCapsule(&mut tw, trace, model);
                } else {
                    CM_TestBoundingBoxInCapsule(&mut tw, trace, model);
                }
            } else {
                CM_TestInLeaf(&mut tw, trace, &unsafe { (*cmod).leaf }, local);
            }
        }
        // #ifdef TEST_TERRAIN_PHYSICS
        // 	else if (cmg.landScape && !model && !cmod->firstNode)
        // 	{
        // 		CM_TraceThroughTerrain( &tw, *trace, cmg.landScape );
        // 	}
        // #endif // #ifdef TEST_TERRAIN_PHYSICS
        else if unsafe { (*cmod).firstNode == -1 } {
            CM_PositionTest(&mut tw, trace);
        } else {
            CM_TraceThroughTree(&mut tw, trace, local, unsafe { (*cmod).firstNode }, 0.0, 1.0, &tw.start, &tw.end);
        }
    } else {
        //
        // check for point special case
        //
        if tw.size[0][0] == 0.0 && tw.size[0][1] == 0.0 && tw.size[0][2] == 0.0 {
            tw.isPoint = true;
            VectorClear(&mut tw.extents);
        } else {
            tw.isPoint = false;
            tw.extents[0] = tw.size[1][0];
            tw.extents[1] = tw.size[1][1];
            tw.extents[2] = tw.size[1][2];
        }

        //
        // general sweeping through world
        //
        if !model.is_null() && unsafe { (*cmod).firstNode == -1 } {
            // #ifdef ALWAYS_BBOX_VS_BBOX
            // 	if ( model == BOX_MODEL_HANDLE || model == CAPSULE_MODEL_HANDLE)
            // 	{
            // 		tw.sphere.use = qfalse;
            // 		CM_TraceThroughLeaf( &tw, &cmod->leaf );
            // 	}
            // 	else
            // #elif defined(ALWAYS_CAPSULE_VS_CAPSULE)
            // 	if ( model == BOX_MODEL_HANDLE || model == CAPSULE_MODEL_HANDLE)
            // 	{
            // 		CM_TraceCapsuleThroughCapsule( &tw, model );
            // 	}
            // 	else
            // #endif
            if model == unsafe { BOX_MODEL_HANDLE } || model == unsafe { CAPSULE_MODEL_HANDLE } {
                if tw.sphere.use {
                    CM_TraceCapsuleThroughCapsule(&mut tw, trace, model);
                } else {
                    CM_TraceBoundingBoxThroughCapsule(&mut tw, trace, model);
                }
            } else {
                CM_TraceThroughLeaf(&mut tw, trace, local, &unsafe { (*cmod).leaf });
            }
        }
        // #ifdef TEST_TERRAIN_PHYSICS
        // 	else if (cmg.landScape && !model && !cmod->firstNode)
        // 	{
        // 		CM_TraceThroughTerrain( &tw, *trace, cmg.landScape );
        // 	}
        // #endif // #ifdef TEST_TERRAIN_PHYSICS
        else {
            CM_TraceThroughTree(&mut tw, trace, local, unsafe { (*cmod).firstNode }, 0.0, 1.0, &tw.start, &tw.end);
        }
    }

    // generate endpos from the original, unmodified start/end
    if unsafe { (*trace).fraction == 1.0 } {
        VectorCopy(end, &mut unsafe { (*trace).endpos });
    } else {
        i = 0;
        while i < 3 {
            unsafe {
                (*trace).endpos[i as usize] = start[i as usize] + (*trace).fraction * (end[i as usize] - start[i as usize]);
            }
            i += 1;
        }
    }

    // If allsolid is set (was entirely inside something solid), the plane is not valid.
    // If fraction == 1.0, we never hit anything, and thus the plane is not valid.
    // Otherwise, the normal on the plane should have unit length
    // assert(trace->allsolid ||
    //        trace->fraction == 1.0 ||
    //        VectorLengthSquared(trace->plane.normal) > 0.9999);
}

/*
==================
CM_BoxTrace
==================
*/
fn CM_BoxTrace(
    results: *mut trace_t,
    start: &[f32; 3],
    end: &[f32; 3],
    mins: &[f32; 3],
    maxs: &[f32; 3],
    model: clipHandle_t,
    brushmask: i32,
    capsule: i32,
) {
    CM_Trace(results, start, end, mins, maxs, model, &vec3_origin, brushmask, capsule, std::ptr::null());
}

/*
==================
CM_TransformedBoxTrace

Handles offseting and rotation of the end points for moving and
rotating entities
==================
*/
fn CM_TransformedBoxTrace(
    trace: *mut trace_t,
    start: &[f32; 3],
    end: &[f32; 3],
    mins: *const [f32; 3],
    maxs: *const [f32; 3],
    model: clipHandle_t,
    brushmask: i32,
    origin: &[f32; 3],
    angles: &[f32; 3],
    capsule: i32,
) {
    let mut start_l: [f32; 3] = [0.0; 3];
    let mut end_l: [f32; 3] = [0.0; 3];
    let mut rotated: bool;
    let mut offset: [f32; 3] = [0.0; 3];
    let mut symetricSize: [[f32; 3]; 2] = [[0.0; 3]; 2];
    let mut matrix: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut transpose: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut i: i32;
    let mut halfwidth: f32;
    let mut halfheight: f32;
    let mut t: f32;
    let mut sphere: sphere_t = sphere_t {
        // Zero-initialize
        use_: false,
        radius: 0.0,
        halfheight: 0.0,
        offset: [0.0; 3],
    };

    let mins_ref: &[f32; 3] = if mins.is_null() {
        &vec3_origin
    } else {
        unsafe { &*mins }
    };
    let maxs_ref: &[f32; 3] = if maxs.is_null() {
        &vec3_origin
    } else {
        unsafe { &*maxs }
    };

    // adjust so that mins and maxs are always symetric, which
    // avoids some complications with plane expanding of rotated
    // bmodels
    i = 0;
    while i < 3 {
        offset[i as usize] = (mins_ref[i as usize] + maxs_ref[i as usize]) * 0.5;
        symetricSize[0][i as usize] = mins_ref[i as usize] - offset[i as usize];
        symetricSize[1][i as usize] = maxs_ref[i as usize] - offset[i as usize];
        start_l[i as usize] = start[i as usize] + offset[i as usize];
        end_l[i as usize] = end[i as usize] + offset[i as usize];
        i += 1;
    }

    // subtract origin offset
    VectorSubtract(&start_l, origin, &mut start_l);
    VectorSubtract(&end_l, origin, &mut end_l);

    // rotate start and end into the models frame of reference
    if model != unsafe { BOX_MODEL_HANDLE } && (angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0) {
        rotated = true;
    } else {
        rotated = false;
    }

    halfwidth = symetricSize[1][0];
    halfheight = symetricSize[1][2];

    sphere.use_ = capsule != 0;
    sphere.radius = if halfwidth > halfheight { halfheight } else { halfwidth };
    sphere.halfheight = halfheight;
    t = halfheight - sphere.radius;

    if rotated {
        // rotation on trace line (start-end) instead of rotating the bmodel
        // NOTE: This is still incorrect for bounding boxes because the actual bounding
        //		 box that is swept through the model is not rotated. We cannot rotate
        //		 the bounding box or the bmodel because that would make all the brush
        //		 bevels invalid.
        //		 However this is correct for capsules since a capsule itself is rotated too.
        CreateRotationMatrix(angles, &mut matrix);
        RotatePoint(&mut start_l, &matrix);
        RotatePoint(&mut end_l, &matrix);
        // rotated sphere offset for capsule
        sphere.offset[0] = matrix[0][2] * t;
        sphere.offset[1] = -matrix[1][2] * t;
        sphere.offset[2] = matrix[2][2] * t;
    } else {
        VectorSet(&mut sphere.offset, 0.0, 0.0, t);
    }

    // sweep the box through the model
    CM_Trace(trace, &start_l, &end_l, &symetricSize[0], &symetricSize[1], model, origin, brushmask, capsule, &sphere);

    // if the bmodel was rotated and there was a collision
    if rotated && unsafe { (*trace).fraction != 1.0 } {
        // rotation of bmodel collision plane
        TransposeMatrix(&matrix, &mut transpose);
        RotatePoint(&mut unsafe { (*trace).plane.normal }, &transpose);
    }

    // re-calculate the end position of the trace because the trace.endpos
    // calculated by CM_Trace could be rotated and have an offset
    unsafe {
        (*trace).endpos[0] = start[0] + (*trace).fraction * (end[0] - start[0]);
        (*trace).endpos[1] = start[1] + (*trace).fraction * (end[1] - start[1]);
        (*trace).endpos[2] = start[2] + (*trace).fraction * (end[2] - start[2]);
    }
}

/*
=================
CM_CullBox

Returns true if culled out
=================
*/

fn CM_CullBox(frustum: *const cplane_t, transformed: &[[f32; 3]; 8]) -> bool {
    let mut i: i32;
    let mut j: i32;
    let mut frust: *const cplane_t;

    // check against frustum planes
    frust = frustum;
    i = 0;
    while i < 4 {
        j = 0;
        while j < 8 {
            if DotProduct(&transformed[j as usize], unsafe { &(*frust).normal }) > unsafe { (*frust).dist } {
                // a point is in front
                break;
            }
            j += 1;
        }

        if j == 8 {
            // all points were behind one of the planes
            return true;
        }
        frust = unsafe { frust.add(1) };
        i += 1;
    }
    return false;
}

/*
=================
CM_CullWorldBox

Returns true if culled out
=================
*/

fn CM_CullWorldBox(frustum: *const cplane_t, bounds: &vec3pair_t) -> bool {
    let mut i: i32;
    let mut transformed: [[f32; 3]; 8] = [[0.0; 3]; 8];

    i = 0;
    while i < 8 {
        transformed[i as usize][0] = bounds[(i & 1) as usize][0];
        transformed[i as usize][1] = bounds[((i >> 1) & 1) as usize][1];
        transformed[i as usize][2] = bounds[((i >> 2) & 1) as usize][2];
        i += 1;
    }

    return CM_CullBox(frustum, &transformed);
}

use core::ffi::c_void;

// Type definitions for C ABI compatibility
type vec3pair_t = [[f32; 3]; 2];
type clipHandle_t = *mut c_void;

#[repr(C)]
pub struct cplane_t {
    normal: [f32; 3],
    dist: f32,
    type_val: u8,
    signbits: u8,
}

#[repr(C)]
pub struct cbrushside_t {
    plane: *const cplane_t,
    shaderNum: i32,
}

#[repr(C)]
pub struct cbrush_t {
    numsides: i32,
    sides: *mut cbrushside_t,
    contents: i32,
    bounds: vec3pair_t,
    checkcount: i32,
}

#[repr(C)]
pub struct cLeaf_t {
    numLeafBrushes: i32,
    firstLeafBrush: i32,
    numLeafSurfaces: i32,
    firstLeafSurface: i32,
}

#[repr(C)]
pub struct cPatch_t {
    contents: i32,
    surfaceFlags: i32,
    pc: *mut c_void,
    checkcount: i32,
}

#[repr(C)]
pub struct cNode_t {
    plane: *const cplane_t,
    children: [i32; 2],
    planeNum: i32,
}

#[repr(C)]
pub struct cmodel_t {
    firstNode: i32,
    leaf: cLeaf_t,
}

#[repr(C)]
pub struct clipMap_t {
    checkcount: i32,
    numNodes: i32,
    nodes: *mut cNode_t,
    leafs: *mut cLeaf_t,
    brushes: *mut cbrush_t,
    surfaces: *mut *mut cPatch_t,
    leafbrushes: *mut i32,
    leafsurfaces: *mut i32,
}

#[repr(C)]
pub struct trace_t {
    fraction: f32,
    startsolid: bool,
    allsolid: bool,
    contents: i32,
    entityNum: i32,
    plane: cplane_t,
    surfaceFlags: i32,
    endpos: [f32; 3],
}

#[repr(C)]
pub struct sphere_t {
    use_: bool,
    radius: f32,
    halfheight: f32,
    offset: [f32; 3],
}

#[repr(C)]
pub struct leafList_t {
    bounds: vec3pair_t,
    count: i32,
    maxcount: i32,
    list: *mut i32,
    storeLeafs: Option<fn(*mut leafList_t, i32)>,
    lastLeaf: i32,
    overflowed: bool,
}

#[repr(C)]
pub struct traceWork_t {
    contents: i32,
    isPoint: bool,
    bounds: vec3pair_t,
    size: [[f32; 3]; 2],
    offsets: [[f32; 3]; 8],
    start: [f32; 3],
    end: [f32; 3],
    extents: [f32; 3],
    sphere: sphere_t,
    maxOffset: f32,
    modelOrigin: [f32; 3],
    enterFrac: f32,
    leaveFrac: f32,
    clipplane: *const cplane_t,
    leadside: *mut cbrushside_t,
    getout: bool,
    startout: bool,
    baseEnterFrac: f32,
    baseLeaveFrac: f32,
    localBounds: vec3pair_t,
}

// Stub for CCMPatch
#[repr(C)]
pub struct CCMPatch {
    // Stub - actual implementation would be more complex
}

#[repr(C)]
pub struct traceWork_s {
    // Stub - same as traceWork_t
}

// Wrapper functions for C vector math - delegates to extern C ABI
fn VectorCopy(src: &[f32; 3], dst: &mut [f32; 3]) {
    unsafe {
        VectorCopy_c(src.as_ptr(), dst.as_mut_ptr());
    }
}

fn VectorAdd(a: &[f32; 3], b: &[f32; 3], out: &mut [f32; 3]) {
    unsafe {
        VectorAdd_c(a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
    }
}

fn VectorSubtract(a: &[f32; 3], b: &[f32; 3], out: &mut [f32; 3]) {
    unsafe {
        VectorSubtract_c(a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
    }
}

fn VectorClear(v: &mut [f32; 3]) {
    unsafe {
        VectorClear_c(v.as_mut_ptr());
    }
}

fn VectorSet(v: &mut [f32; 3], x: f32, y: f32, z: f32) {
    unsafe {
        VectorSet_c(v.as_mut_ptr(), x, y, z);
    }
}

fn VectorMA(base: &[f32; 3], scale: f32, dir: &[f32; 3], out: &mut [f32; 3]) {
    unsafe {
        VectorMA_c(base.as_ptr(), scale, dir.as_ptr(), out.as_mut_ptr());
    }
}

fn VectorScale(v: &[f32; 3], scale: f32, out: &mut [f32; 3]) {
    unsafe {
        VectorScale_c(v.as_ptr(), scale, out.as_mut_ptr());
    }
}

fn DotProduct(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    unsafe { DotProduct_c(a.as_ptr(), b.as_ptr()) }
}

fn VectorNormalize(v: &mut [f32; 3]) -> f32 {
    unsafe { VectorNormalize_c(v.as_mut_ptr()) }
}

fn VectorLength(v: &[f32; 3]) -> f32 {
    unsafe { VectorLength_c(v.as_ptr()) }
}

fn VectorLengthSquared(v: &[f32; 3]) -> f32 {
    unsafe { VectorLengthSquared_c(v.as_ptr()) }
}

fn VectorInverse(v: &mut [f32; 3]) {
    unsafe {
        VectorInverse_c(v.as_mut_ptr());
    }
}

fn AngleVectors(angles: &[f32; 3], forward: &mut [f32; 3], right: &mut [f32; 3], up: &mut [f32; 3]) {
    unsafe {
        AngleVectors_c(angles.as_ptr(), forward.as_mut_ptr(), right.as_mut_ptr(), up.as_mut_ptr());
    }
}

fn CM_ModelBounds(model: clipHandle_t, mins: &mut [f32; 3], maxs: &mut [f32; 3]) {
    unsafe {
        CM_ModelBounds_c(model, mins.as_mut_ptr(), maxs.as_mut_ptr());
    }
}

fn CM_TempBoxModel(mins: &[f32; 3], maxs: &[f32; 3], capsule: bool) -> clipHandle_t {
    unsafe { CM_TempBoxModel_c(mins.as_ptr(), maxs.as_ptr(), capsule) }
}

// External C declarations with C ABI
extern "C" {
    pub static mut cmg: clipMap_t;
    pub static mut c_traces: i32;
    pub static mut c_patch_traces: i32;
    pub static BOX_MODEL_HANDLE: clipHandle_t;
    pub static CAPSULE_MODEL_HANDLE: clipHandle_t;
    pub static vec3_origin: [f32; 3];
    pub const SURFACE_CLIP_EPSILON: f32;
    pub const CONTENTS_BODY: i32;
    pub const CONTENTS_WATER: i32;
    pub const CONTENTS_TERRAIN: i32;
    pub const CONTENTS_OUTSIDE: i32;

    #[link_name = "VectorCopy"]
    fn VectorCopy_c(src: *const f32, dst: *mut f32);
    #[link_name = "VectorAdd"]
    fn VectorAdd_c(a: *const f32, b: *const f32, out: *mut f32);
    #[link_name = "VectorSubtract"]
    fn VectorSubtract_c(a: *const f32, b: *const f32, out: *mut f32);
    #[link_name = "VectorClear"]
    fn VectorClear_c(v: *mut f32);
    #[link_name = "VectorSet"]
    fn VectorSet_c(v: *mut f32, x: f32, y: f32, z: f32);
    #[link_name = "VectorMA"]
    fn VectorMA_c(base: *const f32, scale: f32, dir: *const f32, out: *mut f32);
    #[link_name = "VectorScale"]
    fn VectorScale_c(v: *const f32, scale: f32, out: *mut f32);
    #[link_name = "DotProduct"]
    fn DotProduct_c(a: *const f32, b: *const f32) -> f32;
    #[link_name = "VectorNormalize"]
    fn VectorNormalize_c(v: *mut f32) -> f32;
    #[link_name = "VectorLength"]
    fn VectorLength_c(v: *const f32) -> f32;
    #[link_name = "VectorLengthSquared"]
    fn VectorLengthSquared_c(v: *const f32) -> f32;
    #[link_name = "VectorInverse"]
    fn VectorInverse_c(v: *mut f32);
    #[link_name = "AngleVectors"]
    fn AngleVectors_c(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    #[link_name = "Square"]
    pub fn Square(x: f32) -> f32;

    #[link_name = "CM_ModelBounds"]
    fn CM_ModelBounds_c(model: clipHandle_t, mins: *mut f32, maxs: *mut f32);
    pub fn CM_ClipHandleToModel(handle: clipHandle_t, local: *mut *mut clipMap_t) -> *mut cmodel_t;
    #[link_name = "CM_TempBoxModel"]
    fn CM_TempBoxModel_c(mins: *const f32, maxs: *const f32, capsule: bool) -> clipHandle_t;
    pub fn CM_PositionTestInPatchCollide(tw: *mut traceWork_t, pc: *mut c_void) -> bool;
    pub fn CM_TraceThroughPatchCollide(tw: *mut traceWork_t, trace: *mut trace_t, pc: *mut c_void);
    pub fn CM_BoxLeafnums_r(ll: *mut leafList_t, nodenum: i32);
    pub fn CM_StoreLeafs(ll: *mut leafList_t, leafnum: i32);
}
