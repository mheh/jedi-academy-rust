//Anything above this #include will be ignored by the compiler
// (oracle/codemp/qcommon/exe_headers.h)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use core::ffi::{c_int, c_char, c_void, c_float};
use core::ptr::{addr_of_mut, addr_of};

use crate::codemp::qcommon::cm_patch_h::{
    patchPlane_t, facet_t, patchCollide_s, patchCollide_t, cGrid_t,
    MAX_PATCH_PLANES, MAX_FACETS, MAX_GRID_SIZE, SUBDIVIDE_DISTANCE,
    PLANE_TRI_EPSILON, WRAP_POINT_EPSILON,
};
use crate::codemp::qcommon::cm_local_h::traceWork_t;
use crate::codemp::qcommon::cm_polylib_h::winding_t;
use crate::codemp::qcommon::files_h::cvar_t;
use crate::codemp::game::q_shared_h::{vec3_t, qboolean, qtrue, qfalse, trace_t};

// Constants
const POINT_EPSILON: f32 = 0.1;
const NORMAL_EPSILON: f32 = 0.0001;
const DIST_EPSILON: f32 = 0.02;
const SIDE_ON: c_int = 2;
const SIDE_FRONT: c_int = 0;
const SIDE_BACK: c_int = 1;
const MAX_MAP_BOUNDS: i32 = 65535;
const SURFACE_CLIP_EPSILON: f32 = 0.1f32;
const ERR_DROP: c_int = 0;
const ERR_FATAL: c_int = 1;

// External C functions
extern "C" {
    pub fn Com_Error(level: c_int, format: *const c_char, ...);
    pub fn Com_Printf(format: *const c_char, ...);
    pub fn Com_DPrintf(format: *const c_char, ...);
    pub fn Com_Memset(dest: *mut c_void, val: c_int, count: usize);
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
    pub fn Z_Malloc(size: usize, tag: c_int, clear: c_int, alignment: c_int) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Hunk_Alloc(size: usize, preference: c_int) -> *mut c_void;
    pub fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn Q_fabs(x: f32) -> f32;
    pub fn BaseWindingForPlane(normal: *mut vec3_t, dist: f32) -> *mut winding_t;
    pub fn FreeWinding(w: *mut winding_t);
    pub fn WindingBounds(w: *const winding_t, mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn ChopWindingInPlace(w: *mut *mut winding_t, plane: *mut vec3_t, dist: f32, epsilon: f32);
    pub fn CopyWinding(w: *const winding_t) -> *mut winding_t;
    pub fn ClearBounds(mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn AddPointToBounds(v: *const vec3_t, mins: *mut vec3_t, maxs: *mut vec3_t);
}

// Macros implemented as inline functions
#[inline]
fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32 {
    unsafe { (*v1)[0] * (*v2)[0] + (*v1)[1] * (*v2)[1] + (*v1)[2] * (*v2)[2] }
}

#[inline]
fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    unsafe {
        (*c)[0] = (*a)[0] - (*b)[0];
        (*c)[1] = (*a)[1] - (*b)[1];
        (*c)[2] = (*a)[2] - (*b)[2];
    }
}

#[inline]
fn VectorAdd(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    unsafe {
        (*c)[0] = (*a)[0] + (*b)[0];
        (*c)[1] = (*a)[1] + (*b)[1];
        (*c)[2] = (*a)[2] + (*b)[2];
    }
}

#[inline]
fn VectorCopy(a: *const vec3_t, b: *mut vec3_t) {
    unsafe {
        (*b)[0] = (*a)[0];
        (*b)[1] = (*a)[1];
        (*b)[2] = (*a)[2];
    }
}

#[inline]
fn VectorClear(v: *mut vec3_t) {
    unsafe {
        (*v)[0] = 0.0;
        (*v)[1] = 0.0;
        (*v)[2] = 0.0;
    }
}

#[inline]
fn VectorNegate(a: *const vec3_t, b: *mut vec3_t) {
    unsafe {
        (*b)[0] = -(*a)[0];
        (*b)[1] = -(*a)[1];
        (*b)[2] = -(*a)[2];
    }
}

#[inline]
fn VectorMA(v: *const vec3_t, s: f32, b: *const vec3_t, o: *mut vec3_t) {
    unsafe {
        (*o)[0] = (*v)[0] + s * (*b)[0];
        (*o)[1] = (*v)[1] + s * (*b)[1];
        (*o)[2] = (*v)[2] + s * (*b)[2];
    }
}

#[inline]
fn VectorLengthSquared(v: *const vec3_t) -> f32 {
    unsafe { (*v)[0] * (*v)[0] + (*v)[1] * (*v)[1] + (*v)[2] * (*v)[2] }
}

#[inline]
fn CrossProduct(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    unsafe {
        (*c)[0] = (*a)[1] * (*b)[2] - (*a)[2] * (*b)[1];
        (*c)[1] = (*a)[2] * (*b)[0] - (*a)[0] * (*b)[2];
        (*c)[2] = (*a)[0] * (*b)[1] - (*a)[1] * (*b)[0];
    }
}

#[inline]
fn Vector4Copy(a: *const [f32; 4], b: *mut [f32; 4]) {
    unsafe {
        (*b)[0] = (*a)[0];
        (*b)[1] = (*a)[1];
        (*b)[2] = (*a)[2];
        (*b)[3] = (*a)[3];
    }
}

// Global vec3_origin constant
const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

pub static mut c_totalPatchBlocks: c_int = 0;
pub static mut c_totalPatchSurfaces: c_int = 0;
pub static mut c_totalPatchEdges: c_int = 0;

static mut debugPatchCollide: *const patchCollide_t = core::ptr::null();
static mut debugFacet: *const facet_t = core::ptr::null();
static mut debugBlock: qboolean = qfalse;
static mut debugBlockPoints: [vec3_t; 4] = [[0.0; 3]; 4];

/*
=================
CM_ClearLevelPatches
=================
*/
pub fn CM_ClearLevelPatches() {
    unsafe {
        debugPatchCollide = core::ptr::null();
        debugFacet = core::ptr::null();
    }
}

/*
=================
CM_SignbitsForNormal
=================
*/
#[inline]
fn CM_SignbitsForNormal(normal: *const vec3_t) -> c_int {
    let mut bits: c_int = 0;
    let mut j: c_int;

    unsafe {
        j = 0;
        while j < 3 {
            if (*normal)[j as usize] < 0.0 {
                bits |= 1 << j;
            }
            j += 1;
        }
    }
    bits
}

/*
=====================
CM_PlaneFromPoints

Returns false if the triangle is degenrate.
The normal will point out of the clock for clockwise ordered points
=====================
*/
#[inline]
fn CM_PlaneFromPoints(plane: *mut [f32; 4], a: *const vec3_t, b: *const vec3_t, c: *const vec3_t) -> qboolean {
    let mut d1: vec3_t = [0.0; 3];
    let mut d2: vec3_t = [0.0; 3];

    VectorSubtract(b, a, addr_of_mut!(d1));
    VectorSubtract(c, a, addr_of_mut!(d2));
    CrossProduct(addr_of!(d2), addr_of!(d1), addr_of_mut!([(*plane)[0], (*plane)[1], (*plane)[2]]));

    unsafe {
        if VectorNormalize(addr_of_mut!([(*plane)[0], (*plane)[1], (*plane)[2]])) == 0.0 {
            return qfalse;
        }
    }

    unsafe {
        (*plane)[3] = DotProduct(a, addr_of!([(*plane)[0], (*plane)[1], (*plane)[2]]));
    }
    qtrue
}

/*
================================================================================

GRID SUBDIVISION

================================================================================
*/

/*
=================
CM_NeedsSubdivision

Returns true if the given quadratic curve is not flat enough for our
collision detection purposes
=================
*/
fn CM_NeedsSubdivision(a: *const vec3_t, b: *const vec3_t, c: *const vec3_t) -> qboolean {
    let mut cmid: vec3_t = [0.0; 3];
    let mut lmid: vec3_t = [0.0; 3];
    let mut delta: vec3_t = [0.0; 3];
    let mut dist: f32;
    let mut i: c_int;

    unsafe {
        // calculate the linear midpoint
        i = 0;
        while i < 3 {
            lmid[i as usize] = 0.5 * ((*a)[i as usize] + (*c)[i as usize]);
            i += 1;
        }

        // calculate the exact curve midpoint
        i = 0;
        while i < 3 {
            cmid[i as usize] = 0.5 * (0.5 * ((*a)[i as usize] + (*b)[i as usize]) + 0.5 * ((*b)[i as usize] + (*c)[i as usize]));
            i += 1;
        }

        // see if the curve is far enough away from the linear mid
        VectorSubtract(addr_of!(cmid), addr_of!(lmid), addr_of_mut!(delta));
        dist = VectorLengthSquared(addr_of!(delta));

        if dist >= SUBDIVIDE_DISTANCE as f32 * SUBDIVIDE_DISTANCE as f32 { qtrue } else { qfalse }
    }
}

/*
===============
CM_Subdivide

a, b, and c are control points.
the subdivided sequence will be: a, out1, out2, out3, c
===============
*/
fn CM_Subdivide(a: *const vec3_t, b: *const vec3_t, c: *const vec3_t, out1: *mut vec3_t, out2: *mut vec3_t, out3: *mut vec3_t) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < 3 {
            (*out1)[i as usize] = 0.5 * ((*a)[i as usize] + (*b)[i as usize]);
            (*out3)[i as usize] = 0.5 * ((*b)[i as usize] + (*c)[i as usize]);
            (*out2)[i as usize] = 0.5 * ((*out1)[i as usize] + (*out3)[i as usize]);
            i += 1;
        }
    }
}

/*
=================
CM_TransposeGrid

Swaps the rows and columns in place
=================
*/
fn CM_TransposeGrid(grid: *mut cGrid_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut l: c_int;
    let mut temp: vec3_t = [0.0; 3];
    let mut tempWrap: qboolean;

    unsafe {
        if (*grid).width > (*grid).height {
            i = 0;
            while i < (*grid).height {
                j = i + 1;
                while j < (*grid).width {
                    if j < (*grid).height {
                        // swap the value
                        VectorCopy(addr_of!((*grid).points[i as usize][j as usize]), addr_of_mut!(temp));
                        VectorCopy(addr_of!((*grid).points[j as usize][i as usize]), addr_of_mut!((*grid).points[i as usize][j as usize]));
                        VectorCopy(addr_of!(temp), addr_of_mut!((*grid).points[j as usize][i as usize]));
                    } else {
                        // just copy
                        VectorCopy(addr_of!((*grid).points[j as usize][i as usize]), addr_of_mut!((*grid).points[i as usize][j as usize]));
                    }
                    j += 1;
                }
                i += 1;
            }
        } else {
            i = 0;
            while i < (*grid).width {
                j = i + 1;
                while j < (*grid).height {
                    if j < (*grid).width {
                        // swap the value
                        VectorCopy(addr_of!((*grid).points[j as usize][i as usize]), addr_of_mut!(temp));
                        VectorCopy(addr_of!((*grid).points[i as usize][j as usize]), addr_of_mut!((*grid).points[j as usize][i as usize]));
                        VectorCopy(addr_of!(temp), addr_of_mut!((*grid).points[i as usize][j as usize]));
                    } else {
                        // just copy
                        VectorCopy(addr_of!((*grid).points[i as usize][j as usize]), addr_of_mut!((*grid).points[j as usize][i as usize]));
                    }
                    j += 1;
                }
                i += 1;
            }
        }

        l = (*grid).width;
        (*grid).width = (*grid).height;
        (*grid).height = l;

        tempWrap = (*grid).wrapWidth;
        (*grid).wrapWidth = (*grid).wrapHeight;
        (*grid).wrapHeight = tempWrap;
    }
}

/*
===================
CM_SetGridWrapWidth

If the left and right columns are exactly equal, set grid->wrapWidth qtrue
===================
*/
fn CM_SetGridWrapWidth(grid: *mut cGrid_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut d: f32;

    unsafe {
        i = 0;
        while i < (*grid).height {
            j = 0;
            while j < 3 {
                d = (*grid).points[0][i as usize][j as usize] - (*grid).points[((*grid).width - 1) as usize][i as usize][j as usize];
                if d < -WRAP_POINT_EPSILON || d > WRAP_POINT_EPSILON {
                    break;
                }
                j += 1;
            }
            if j != 3 {
                break;
            }
            i += 1;
        }
        if i == (*grid).height {
            (*grid).wrapWidth = qtrue;
        } else {
            (*grid).wrapWidth = qfalse;
        }
    }
}

/*
=================
CM_SubdivideGridColumns

Adds columns as necessary to the grid until
all the aproximating points are within SUBDIVIDE_DISTANCE
from the true curve
=================
*/
fn CM_SubdivideGridColumns(grid: *mut cGrid_t) {
    let mut i: c_int = 0;
    let mut j: c_int;
    let mut k: c_int;

    unsafe {
        while i < (*grid).width - 2 {
            // grid->points[i][x] is an interpolating control point
            // grid->points[i+1][x] is an aproximating control point
            // grid->points[i+2][x] is an interpolating control point

            //
            // first see if we can collapse the aproximating collumn away
            //
            j = 0;
            while j < (*grid).height {
                if CM_NeedsSubdivision(addr_of!((*grid).points[i as usize][j as usize]), addr_of!((*grid).points[(i + 1) as usize][j as usize]), addr_of!((*grid).points[(i + 2) as usize][j as usize])) != qfalse {
                    break;
                }
                j += 1;
            }
            if j == (*grid).height {
                // all of the points were close enough to the linear midpoints
                // that we can collapse the entire column away
                j = 0;
                while j < (*grid).height {
                    // remove the column
                    k = i + 2;
                    while k < (*grid).width {
                        VectorCopy(addr_of!((*grid).points[k as usize][j as usize]), addr_of_mut!((*grid).points[(k - 1) as usize][j as usize]));
                        k += 1;
                    }
                    j += 1;
                }

                (*grid).width -= 1;

                // go to the next curve segment
                i += 1;
                continue;
            }

            //
            // we need to subdivide the curve
            //
            j = 0;
            while j < (*grid).height {
                let mut prev: vec3_t = [0.0; 3];
                let mut mid: vec3_t = [0.0; 3];
                let mut next: vec3_t = [0.0; 3];

                // save the control points now
                VectorCopy(addr_of!((*grid).points[i as usize][j as usize]), addr_of_mut!(prev));
                VectorCopy(addr_of!((*grid).points[(i + 1) as usize][j as usize]), addr_of_mut!(mid));
                VectorCopy(addr_of!((*grid).points[(i + 2) as usize][j as usize]), addr_of_mut!(next));

                // make room for two additional columns in the grid
                // columns i+1 will be replaced, column i+2 will become i+4
                // i+1, i+2, and i+3 will be generated
                k = (*grid).width - 1;
                while k > i + 1 {
                    VectorCopy(addr_of!((*grid).points[k as usize][j as usize]), addr_of_mut!((*grid).points[(k + 2) as usize][j as usize]));
                    k -= 1;
                }

                // generate the subdivided points
                CM_Subdivide(addr_of!(prev), addr_of!(mid), addr_of!(next), addr_of_mut!((*grid).points[(i + 1) as usize][j as usize]), addr_of_mut!((*grid).points[(i + 2) as usize][j as usize]), addr_of_mut!((*grid).points[(i + 3) as usize][j as usize]));
                j += 1;
            }

            (*grid).width += 2;

            // the new aproximating point at i+1 may need to be removed
            // or subdivided farther, so don't advance i
        }
    }
}

/*
======================
CM_ComparePoints
======================
*/
fn CM_ComparePoints(a: *const [f32; 3], b: *const [f32; 3]) -> qboolean {
    let mut d: f32;

    unsafe {
        d = (*a)[0] - (*b)[0];
        if d < -POINT_EPSILON || d > POINT_EPSILON {
            return qfalse;
        }
        d = (*a)[1] - (*b)[1];
        if d < -POINT_EPSILON || d > POINT_EPSILON {
            return qfalse;
        }
        d = (*a)[2] - (*b)[2];
        if d < -POINT_EPSILON || d > POINT_EPSILON {
            return qfalse;
        }
    }
    qtrue
}

/*
=================
CM_RemoveDegenerateColumns

If there are any identical columns, remove them
=================
*/
fn CM_RemoveDegenerateColumns(grid: *mut cGrid_t) {
    let mut i: c_int = 0;
    let mut j: c_int;
    let mut k: c_int;

    unsafe {
        while i < (*grid).width - 1 {
            j = 0;
            while j < (*grid).height {
                if CM_ComparePoints(addr_of!((*grid).points[i as usize][j as usize]), addr_of!((*grid).points[(i + 1) as usize][j as usize])) == qfalse {
                    break;
                }
                j += 1;
            }

            if j != (*grid).height {
                i += 1;
                continue;	// not degenerate
            }

            j = 0;
            while j < (*grid).height {
                // remove the column
                k = i + 2;
                while k < (*grid).width {
                    VectorCopy(addr_of!((*grid).points[k as usize][j as usize]), addr_of_mut!((*grid).points[(k - 1) as usize][j as usize]));
                    k += 1;
                }
                j += 1;
            }
            (*grid).width -= 1;

            // check against the next column
            i -= 1;
            if i < 0 { break; }
        }
    }
}

/*
================================================================================

PATCH COLLIDE GENERATION

================================================================================
*/

static mut numPlanes: c_int = 0;
static mut planes: [patchPlane_t; MAX_PATCH_PLANES as usize] = unsafe { core::mem::zeroed() };

//static	int				numFacets;
//static	facet_t			facets[MAX_PATCH_PLANES]; //maybe MAX_FACETS ??
static mut facets: *mut facet_t = core::ptr::null_mut();

#[inline]
fn CM_PlaneEqual(p: *const patchPlane_t, plane: *const [f32; 4], flipped: *mut c_int) -> c_int {
    let mut invplane: [f32; 4] = [0.0; 4];

    unsafe {
        if Q_fabs((*p).plane[0] - (*plane)[0]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[1] - (*plane)[1]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[2] - (*plane)[2]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[3] - (*plane)[3]) < DIST_EPSILON
        {
            *flipped = qfalse;
            return qtrue;
        }

        let mut normal_neg = [0.0; 3];
        VectorNegate(addr_of!([(*plane)[0], (*plane)[1], (*plane)[2]]), addr_of_mut!(normal_neg));
        invplane[0] = normal_neg[0];
        invplane[1] = normal_neg[1];
        invplane[2] = normal_neg[2];
        invplane[3] = -(*plane)[3];

        if Q_fabs((*p).plane[0] - invplane[0]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[1] - invplane[1]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[2] - invplane[2]) < NORMAL_EPSILON
            && Q_fabs((*p).plane[3] - invplane[3]) < DIST_EPSILON
        {
            *flipped = qtrue;
            return qtrue;
        }
    }

    qfalse
}

#[inline]
fn CM_SnapVector(normal: *mut vec3_t) {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < 3 {
            if Q_fabs((*normal)[i as usize] - 1.0) < NORMAL_EPSILON {
                VectorClear(normal);
                (*normal)[i as usize] = 1.0;
                break;
            }
            if Q_fabs((*normal)[i as usize] - (-1.0)) < NORMAL_EPSILON {
                VectorClear(normal);
                (*normal)[i as usize] = -1.0;
                break;
            }
            i += 1;
        }
    }
}

#[inline]
fn CM_FindPlane2(plane: *const [f32; 4], flipped: *mut c_int) -> c_int {
    let mut i: c_int;

    unsafe {
        // see if the points are close enough to an existing plane
        i = 0;
        while i < numPlanes {
            if CM_PlaneEqual(addr_of!(planes[i as usize]), plane, flipped) != qfalse {
                return i;
            }
            i += 1;
        }

        // add a new plane
        if numPlanes == MAX_PATCH_PLANES {
            Com_Error(ERR_DROP, b"MAX_PATCH_PLANES\0".as_ptr() as *const c_char);
        }

        Vector4Copy(plane, addr_of_mut!(planes[numPlanes as usize].plane));
        planes[numPlanes as usize].signbits = CM_SignbitsForNormal(addr_of!([(*plane)[0], (*plane)[1], (*plane)[2]]));

        numPlanes += 1;
    }

    *flipped = qfalse;

    unsafe { numPlanes - 1 }
}

/*
==================
CM_FindPlane
==================
*/
#[inline]
fn CM_FindPlane(p1: *const vec3_t, p2: *const vec3_t, p3: *const vec3_t) -> c_int {
    let mut plane: [f32; 4] = [0.0; 4];
    let mut i: c_int;
    let mut d: f32;

    if CM_PlaneFromPoints(addr_of_mut!(plane), p1, p2, p3) == qfalse {
        return -1;
    }

    unsafe {
        // see if the points are close enough to an existing plane
        i = 0;
        while i < numPlanes {
            if DotProduct(addr_of!([plane[0], plane[1], plane[2]]), addr_of!(planes[i as usize].plane[..3] as *const [f32; 3])) < 0.0 {
                i += 1;
                continue;	// allow backwards planes?
            }

            d = DotProduct(p1, addr_of!([planes[i as usize].plane[0], planes[i as usize].plane[1], planes[i as usize].plane[2]])) - planes[i as usize].plane[3];
            if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
                i += 1;
                continue;
            }

            d = DotProduct(p2, addr_of!([planes[i as usize].plane[0], planes[i as usize].plane[1], planes[i as usize].plane[2]])) - planes[i as usize].plane[3];
            if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
                i += 1;
                continue;
            }

            d = DotProduct(p3, addr_of!([planes[i as usize].plane[0], planes[i as usize].plane[1], planes[i as usize].plane[2]])) - planes[i as usize].plane[3];
            if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
                i += 1;
                continue;
            }

            // found it
            return i;
        }

        // add a new plane
        if numPlanes == MAX_PATCH_PLANES {
            Com_Error(ERR_DROP, b"MAX_PATCH_PLANES\0".as_ptr() as *const c_char);
        }

        Vector4Copy(addr_of!(plane), addr_of_mut!(planes[numPlanes as usize].plane));
        planes[numPlanes as usize].signbits = CM_SignbitsForNormal(addr_of!([plane[0], plane[1], plane[2]]));

        numPlanes += 1;
    }

    unsafe { numPlanes - 1 }
}

/*
==================
CM_PointOnPlaneSide
==================
*/
#[inline]
fn CM_PointOnPlaneSide(p: *const vec3_t, planeNum: c_int) -> c_int {
    let mut d: f32;

    if planeNum == -1 {
        return SIDE_ON;
    }

    unsafe {
        d = DotProduct(p, addr_of!([planes[planeNum as usize].plane[0], planes[planeNum as usize].plane[1], planes[planeNum as usize].plane[2]])) - planes[planeNum as usize].plane[3];

        if d > PLANE_TRI_EPSILON {
            return SIDE_FRONT;
        }

        if d < -PLANE_TRI_EPSILON {
            return SIDE_BACK;
        }
    }

    SIDE_ON
}

#[inline]
fn CM_GridPlane(gridPlanes: *const [[[c_int; 2]; MAX_GRID_SIZE as usize]; MAX_GRID_SIZE as usize], i: c_int, j: c_int, tri: c_int) -> c_int {
    let mut p: c_int;

    unsafe {
        p = (*gridPlanes)[i as usize][j as usize][tri as usize];
        if p != -1 {
            return p;
        }
        p = (*gridPlanes)[i as usize][j as usize][(1 - tri) as usize];
        if p != -1 {
            return p;
        }

        // should never happen
        Com_Printf(b"WARNING: CM_GridPlane unresolvable\n\0".as_ptr() as *const c_char);
    }
    -1
}

/*
==================
CM_EdgePlaneNum
==================
*/
#[inline]
fn CM_EdgePlaneNum(grid: *const cGrid_t, gridPlanes: *const [[[c_int; 2]; MAX_GRID_SIZE as usize]; MAX_GRID_SIZE as usize], i: c_int, j: c_int, k: c_int) -> c_int {
    let p1: *const vec3_t;
    let p2: *const vec3_t;
    let mut up: vec3_t = [0.0; 3];
    let mut p: c_int;

    unsafe {
        match k {
            0 => {	// top border
                p1 = addr_of!((*grid).points[i as usize][j as usize]);
                p2 = addr_of!((*grid).points[(i + 1) as usize][j as usize]);
                p = CM_GridPlane(gridPlanes, i, j, 0);
                VectorMA(p1, 4.0, addr_of!([planes[p as usize].plane[0], planes[p as usize].plane[1], planes[p as usize].plane[2]]), addr_of_mut!(up));
                return CM_FindPlane(p1, p2, addr_of!(up));
            }
            2 => {	// bottom border
                p1 = addr_of!((*grid).points[i as usize][(j + 1) as usize]);
                p2 = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                p = CM_GridPlane(gridPlanes, i, j, 1);
                VectorMA(p1, 4.0, addr_of!([planes[p as usize].plane[0], planes[p as usize].plane[1], planes[p as usize].plane[2]]), addr_of_mut!(up));
                return CM_FindPlane(p2, p1, addr_of!(up));
            }
            3 => { // left border
                p1 = addr_of!((*grid).points[i as usize][j as usize]);
                p2 = addr_of!((*grid).points[i as usize][(j + 1) as usize]);
                p = CM_GridPlane(gridPlanes, i, j, 1);
                VectorMA(p1, 4.0, addr_of!([planes[p as usize].plane[0], planes[p as usize].plane[1], planes[p as usize].plane[2]]), addr_of_mut!(up));
                return CM_FindPlane(p2, p1, addr_of!(up));
            }
            1 => {	// right border
                p1 = addr_of!((*grid).points[(i + 1) as usize][j as usize]);
                p2 = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                p = CM_GridPlane(gridPlanes, i, j, 0);
                VectorMA(p1, 4.0, addr_of!([planes[p as usize].plane[0], planes[p as usize].plane[1], planes[p as usize].plane[2]]), addr_of_mut!(up));
                return CM_FindPlane(p1, p2, addr_of!(up));
            }
            4 => {	// diagonal out of triangle 0
                p1 = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                p2 = addr_of!((*grid).points[i as usize][j as usize]);
                p = CM_GridPlane(gridPlanes, i, j, 0);
                VectorMA(p1, 4.0, addr_of!([planes[p as usize].plane[0], planes[p as usize].plane[1], planes[p as usize].plane[2]]), addr_of_mut!(up));
                return CM_FindPlane(p1, p2, addr_of!(up));
            }
            5 => {	// diagonal out of triangle 1
                p1 = addr_of!((*grid).points[i as usize][j as usize]);
                p2 = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                p = CM_GridPlane(gridPlanes, i, j, 1);
                VectorMA(p1, 4.0, addr_of!([planes[p as usize].plane[0], planes[p as usize].plane[1], planes[p as usize].plane[2]]), addr_of_mut!(up));
                return CM_FindPlane(p1, p2, addr_of!(up));
            }
            _ => {}
        }

        Com_Error(ERR_DROP, b"CM_EdgePlaneNum: bad k\0".as_ptr() as *const c_char);
    }
    -1
}

/*
===================
CM_SetBorderInward
===================
*/
#[inline]
fn CM_SetBorderInward(facet: *mut facet_t, grid: *const cGrid_t, gridPlanes: *const [[[c_int; 2]; MAX_GRID_SIZE as usize]; MAX_GRID_SIZE as usize],
                          i: c_int, j: c_int, which: c_int) {
    let mut k: c_int;
    let mut l: c_int;
    let mut points: [*const vec3_t; 4] = [core::ptr::null(); 4];
    let mut numPoints: c_int;

    unsafe {
        match which {
            -1 => {
                points[0] = addr_of!((*grid).points[i as usize][j as usize]);
                points[1] = addr_of!((*grid).points[(i + 1) as usize][j as usize]);
                points[2] = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                points[3] = addr_of!((*grid).points[i as usize][(j + 1) as usize]);
                numPoints = 4;
            }
            0 => {
                points[0] = addr_of!((*grid).points[i as usize][j as usize]);
                points[1] = addr_of!((*grid).points[(i + 1) as usize][j as usize]);
                points[2] = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                numPoints = 3;
            }
            1 => {
                points[0] = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                points[1] = addr_of!((*grid).points[i as usize][(j + 1) as usize]);
                points[2] = addr_of!((*grid).points[i as usize][j as usize]);
                numPoints = 3;
            }
            _ => {
                Com_Error(ERR_FATAL, b"CM_SetBorderInward: bad parameter\0".as_ptr() as *const c_char);
                numPoints = 0;
            }
        }

        k = 0;
        while k < (*facet).numBorders {
            let mut front: c_int = 0;
            let mut back: c_int = 0;

            l = 0;
            while l < numPoints {
                let side: c_int;

                side = CM_PointOnPlaneSide(points[l as usize], (*facet).borderPlanes[k as usize]);
                if side == SIDE_FRONT {
                    front += 1;
                }
                if side == SIDE_BACK {
                    back += 1;
                }
                l += 1;
            }

            if front != 0 && back == 0 {
                (*facet).borderInward[k as usize] = qtrue;
            } else if back != 0 && front == 0 {
                (*facet).borderInward[k as usize] = qfalse;
            } else if front == 0 && back == 0 {
                // flat side border
                (*facet).borderPlanes[k as usize] = -1;
            } else {
                // bisecting side border
                #[cfg(not(feature = "bspc"))]
                Com_DPrintf(b"WARNING: CM_SetBorderInward: mixed plane sides\n\0".as_ptr() as *const c_char);
                (*facet).borderInward[k as usize] = qfalse;
                if debugBlock == qfalse {
                    debugBlock = qtrue;
                    VectorCopy(addr_of!((*grid).points[i as usize][j as usize]), addr_of_mut!(debugBlockPoints[0]));
                    VectorCopy(addr_of!((*grid).points[(i + 1) as usize][j as usize]), addr_of_mut!(debugBlockPoints[1]));
                    VectorCopy(addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]), addr_of_mut!(debugBlockPoints[2]));
                    VectorCopy(addr_of!((*grid).points[i as usize][(j + 1) as usize]), addr_of_mut!(debugBlockPoints[3]));
                }
            }
            k += 1;
        }
    }
}

/*
==================
CM_ValidateFacet

If the facet isn't bounded by its borders, we screwed up.
==================
*/
#[inline]
fn CM_ValidateFacet(facet: *const facet_t) -> qboolean {
    let mut plane: [f32; 4] = [0.0; 4];
    let mut j: c_int;
    let mut w: *mut winding_t;
    let mut bounds: [vec3_t; 2] = [[0.0; 3]; 2];

    unsafe {
        if (*facet).surfacePlane == -1 {
            return qfalse;
        }

        Vector4Copy(addr_of!(planes[(*facet).surfacePlane as usize].plane), addr_of_mut!(plane));
        w = BaseWindingForPlane(addr_of_mut!([plane[0], plane[1], plane[2]]), plane[3]);
        j = 0;
        while j < (*facet).numBorders && !w.is_null() {
            if (*facet).borderPlanes[j as usize] == -1 {
                FreeWinding(w);
                return qfalse;
            }
            Vector4Copy(addr_of!(planes[(*facet).borderPlanes[j as usize] as usize].plane), addr_of_mut!(plane));
            if (*facet).borderInward[j as usize] == qfalse {
                let mut plane_neg = [0.0; 3];
                VectorNegate(addr_of!([plane[0], plane[1], plane[2]]), addr_of_mut!(plane_neg));
                plane[0] = plane_neg[0];
                plane[1] = plane_neg[1];
                plane[2] = plane_neg[2];
                plane[3] = -plane[3];
            }
            ChopWindingInPlace(addr_of_mut!(w), addr_of_mut!([plane[0], plane[1], plane[2]]), plane[3], 0.1f32);
            j += 1;
        }

        if w.is_null() {
            return qfalse;		// winding was completely chopped away
        }

        // see if the facet is unreasonably large
        WindingBounds(w, addr_of_mut!(bounds[0]), addr_of_mut!(bounds[1]));
        FreeWinding(w);

        j = 0;
        while j < 3 {
            if bounds[1][j as usize] - bounds[0][j as usize] > MAX_MAP_BOUNDS as f32 {
                return qfalse;		// we must be missing a plane
            }
            if bounds[0][j as usize] >= MAX_MAP_BOUNDS as f32 {
                return qfalse;
            }
            if bounds[1][j as usize] <= -MAX_MAP_BOUNDS as f32 {
                return qfalse;
            }
            j += 1;
        }
    }
    qtrue		// winding is fine
}

/*
==================
CM_AddFacetBevels
==================
*/
#[inline]
fn CM_AddFacetBevels(facet: *mut facet_t) {

    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut l: c_int;
    let mut axis: c_int;
    let mut dir: c_int;
    let mut order: c_int;
    let mut flipped: c_int;
    let mut plane: [f32; 4] = [0.0; 4];
    let mut d: f32;
    let mut newplane: [f32; 4] = [0.0; 4];
    let mut w: *mut winding_t;
    let mut w2: *mut winding_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut vec: vec3_t = [0.0; 3];
    let mut vec2: vec3_t = [0.0; 3];

    unsafe {
        Vector4Copy(addr_of!(planes[(*facet).surfacePlane as usize].plane), addr_of_mut!(plane));

        w = BaseWindingForPlane(addr_of_mut!([plane[0], plane[1], plane[2]]), plane[3]);
        j = 0;
        while j < (*facet).numBorders && !w.is_null() {
            if (*facet).borderPlanes[j as usize] == (*facet).surfacePlane {
                j += 1;
                continue;
            }
            Vector4Copy(addr_of!(planes[(*facet).borderPlanes[j as usize] as usize].plane), addr_of_mut!(plane));

            if (*facet).borderInward[j as usize] == qfalse {
                let mut plane_neg = [0.0; 3];
                VectorNegate(addr_of!([plane[0], plane[1], plane[2]]), addr_of_mut!(plane_neg));
                plane[0] = plane_neg[0];
                plane[1] = plane_neg[1];
                plane[2] = plane_neg[2];
                plane[3] = -plane[3];
            }

            ChopWindingInPlace(addr_of_mut!(w), addr_of_mut!([plane[0], plane[1], plane[2]]), plane[3], 0.1f32);
            j += 1;
        }
        if w.is_null() {
            return;
        }

        WindingBounds(w, addr_of_mut!(mins), addr_of_mut!(maxs));

        // add the axial planes
        order = 0;
        axis = 0;
        while axis < 3 {
            dir = -1;
            while dir <= 1 {
                VectorClear(addr_of_mut!(plane));
                plane[axis as usize] = dir as f32;
                if dir == 1 {
                    plane[3] = maxs[axis as usize];
                }
                else {
                    plane[3] = -mins[axis as usize];
                }
                //if it's the surface plane
                if CM_PlaneEqual(addr_of!(planes[(*facet).surfacePlane as usize]), addr_of!(plane), addr_of_mut!(flipped)) != qfalse {
                    dir += 2;
                    order += 1;
                    continue;
                }
                // see if the plane is allready present
                i = 0;
                while i < (*facet).numBorders {
                    if CM_PlaneEqual(addr_of!(planes[(*facet).borderPlanes[i as usize] as usize]), addr_of!(plane), addr_of_mut!(flipped)) != qfalse {
                        break;
                    }
                    i += 1;
                }

                if i == (*facet).numBorders {
                    if (*facet).numBorders > 4 + 6 + 16 {
                        Com_Printf(b"ERROR: too many bevels\n\0".as_ptr() as *const c_char);
                    }
                    (*facet).borderPlanes[(*facet).numBorders as usize] = CM_FindPlane2(addr_of!(plane), addr_of_mut!(flipped));
                    (*facet).borderNoAdjust[(*facet).numBorders as usize] = 0;
                    (*facet).borderInward[(*facet).numBorders as usize] = flipped;
                    (*facet).numBorders += 1;
                }
                dir += 2;
                order += 1;
            }
            axis += 1;
        }
        //
        // add the edge bevels
        //
        // test the non-axial plane edges
        j = 0;
        while j < (*w).numpoints {
            k = ((j + 1) % (*w).numpoints);
            VectorSubtract(addr_of!((*w).p[j as usize]), addr_of!((*w).p[k as usize]), addr_of_mut!(vec));
            //if it's a degenerate edge
            if VectorNormalize(addr_of_mut!(vec)) < 0.5 {
                j += 1;
                continue;
            }
            CM_SnapVector(addr_of_mut!(vec));
            k = 0;
            while k < 3 {
                if vec[k as usize] == -1.0 || vec[k as usize] == 1.0 {
                    break;	// axial
                }
                k += 1;
            }
            if k < 3 {
                j += 1;
                continue;	// only test non-axial edges
            }

            // try the six possible slanted axials from this edge
            axis = 0;
            while axis < 3 {
                dir = -1;
                while dir <= 1 {
                    // construct a plane
                    VectorClear(addr_of_mut!(vec2));
                    vec2[axis as usize] = dir as f32;
                    CrossProduct(addr_of!(vec), addr_of!(vec2), addr_of_mut!(plane));
                    if VectorNormalize(addr_of_mut!(plane)) < 0.5 {
                        dir += 2;
                        continue;
                    }
                    plane[3] = DotProduct(addr_of!((*w).p[j as usize]), addr_of!([plane[0], plane[1], plane[2]]));

                    // if all the points of the facet winding are
                    // behind this plane, it is a proper edge bevel
                    l = 0;
                    while l < (*w).numpoints {
                        d = DotProduct(addr_of!((*w).p[l as usize]), addr_of!([plane[0], plane[1], plane[2]])) - plane[3];
                        if d > 0.1 {
                            break;	// point in front
                        }
                        l += 1;
                    }
                    if l < (*w).numpoints {
                        dir += 2;
                        continue;
                    }

                    //if it's the surface plane
                    if CM_PlaneEqual(addr_of!(planes[(*facet).surfacePlane as usize]), addr_of!(plane), addr_of_mut!(flipped)) != qfalse {
                        dir += 2;
                        continue;
                    }
                    // see if the plane is allready present
                    i = 0;
                    while i < (*facet).numBorders {
                        if CM_PlaneEqual(addr_of!(planes[(*facet).borderPlanes[i as usize] as usize]), addr_of!(plane), addr_of_mut!(flipped)) != qfalse {
                            break;
                        }
                        i += 1;
                    }

                    if i == (*facet).numBorders {
                        if (*facet).numBorders > 4 + 6 + 16 {
                            Com_Printf(b"ERROR: too many bevels\n\0".as_ptr() as *const c_char);
                        }
                        (*facet).borderPlanes[(*facet).numBorders as usize] = CM_FindPlane2(addr_of!(plane), addr_of_mut!(flipped));

                        k = 0;
                        while k < (*facet).numBorders {
                            if (*facet).borderPlanes[(*facet).numBorders as usize] ==
                                (*facet).borderPlanes[k as usize] {
                                Com_Printf(b"WARNING: bevel plane already used\n\0".as_ptr() as *const c_char);
                            }
                            k += 1;
                        }

                        (*facet).borderNoAdjust[(*facet).numBorders as usize] = 0;
                        (*facet).borderInward[(*facet).numBorders as usize] = flipped;
                        //
                        w2 = CopyWinding(w);
                        Vector4Copy(addr_of!(planes[(*facet).borderPlanes[(*facet).numBorders as usize] as usize].plane), addr_of_mut!(newplane));
                        if (*facet).borderInward[(*facet).numBorders as usize] == qfalse {
                            let mut newplane_neg = [0.0; 3];
                            VectorNegate(addr_of!([newplane[0], newplane[1], newplane[2]]), addr_of_mut!(newplane_neg));
                            newplane[0] = newplane_neg[0];
                            newplane[1] = newplane_neg[1];
                            newplane[2] = newplane_neg[2];
                            newplane[3] = -newplane[3];
                        }
                        ChopWindingInPlace(addr_of_mut!(w2), addr_of_mut!([newplane[0], newplane[1], newplane[2]]), newplane[3], 0.1f32);
                        if w2.is_null() {
                            #[cfg(not(feature = "bspc"))]
                            Com_DPrintf(b"WARNING: CM_AddFacetBevels... invalid bevel\n\0".as_ptr() as *const c_char);
                            dir += 2;
                            continue;
                        }
                        else {
                            FreeWinding(w2);
                        }
                        //
                        (*facet).numBorders += 1;
                        //already got a bevel
        //					break;
                    }
                    dir += 2;
                }
                axis += 1;
            }
            j += 1;
        }
        FreeWinding(w);

        #[cfg(not(feature = "bspc"))]
        {
            //add opposite plane
            (*facet).borderPlanes[(*facet).numBorders as usize] = (*facet).surfacePlane;
            (*facet).borderNoAdjust[(*facet).numBorders as usize] = 0;
            (*facet).borderInward[(*facet).numBorders as usize] = qtrue;
            (*facet).numBorders += 1;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
enum edgeName_t {
    EN_TOP = 0,
    EN_RIGHT = 1,
    EN_BOTTOM = 2,
    EN_LEFT = 3,
}

/*
==================
CM_PatchCollideFromGrid
==================
*/
fn CM_PatchCollideFromGrid(grid: *const cGrid_t, pf: *mut patchCollide_t) {
    let mut i: c_int;
    let mut j: c_int;
    let p1: *const vec3_t;
    let p2: *const vec3_t;
    let p3: *const vec3_t;
    let mut gridPlanes: [[[c_int; 2]; MAX_GRID_SIZE as usize]; MAX_GRID_SIZE as usize] = unsafe { core::mem::zeroed() };
    let mut facet: *mut facet_t;
    let mut borders: [c_int; 4] = [0; 4];
    let mut noAdjust: [c_int; 4] = [0; 4];

    let mut numFacets: c_int;

    unsafe {
        facets = Z_Malloc((MAX_FACETS as usize) * core::mem::size_of::<facet_t>(), 18, 0, 4) as *mut facet_t;	// TAG_TEMP_WORKSPACE = 18

        numPlanes = 0;
        numFacets = 0;

        // find the planes for each triangle of the grid
        i = 0;
        while i < (*grid).width - 1 {
            j = 0;
            while j < (*grid).height - 1 {
                p1 = addr_of!((*grid).points[i as usize][j as usize]);
                p2 = addr_of!((*grid).points[(i + 1) as usize][j as usize]);
                p3 = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                gridPlanes[i as usize][j as usize][0] = CM_FindPlane(p1, p2, p3);

                p1 = addr_of!((*grid).points[(i + 1) as usize][(j + 1) as usize]);
                p2 = addr_of!((*grid).points[i as usize][(j + 1) as usize]);
                p3 = addr_of!((*grid).points[i as usize][j as usize]);
                gridPlanes[i as usize][j as usize][1] = CM_FindPlane(p1, p2, p3);
                j += 1;
            }
            i += 1;
        }

        // create the borders for each facet
        i = 0;
        while i < (*grid).width - 1 {
            j = 0;
            while j < (*grid).height - 1 {

                borders[0] = -1;
                if j > 0 {
                    borders[0] = gridPlanes[i as usize][(j - 1) as usize][1];
                } else if (*grid).wrapHeight != qfalse {
                    borders[0] = gridPlanes[i as usize][((*grid).height - 2) as usize][1];
                }
                noAdjust[0] = if borders[0] == gridPlanes[i as usize][j as usize][0] { 1 } else { 0 };
                if borders[0] == -1 || noAdjust[0] != 0 {
                    borders[0] = CM_EdgePlaneNum(grid, addr_of!(gridPlanes), i, j, 0);
                }

                borders[2] = -1;
                if j < (*grid).height - 2 {
                    borders[2] = gridPlanes[i as usize][(j + 1) as usize][0];
                } else if (*grid).wrapHeight != qfalse {
                    borders[2] = gridPlanes[i as usize][0][0];
                }
                noAdjust[2] = if borders[2] == gridPlanes[i as usize][j as usize][1] { 1 } else { 0 };
                if borders[2] == -1 || noAdjust[2] != 0 {
                    borders[2] = CM_EdgePlaneNum(grid, addr_of!(gridPlanes), i, j, 2);
                }

                borders[3] = -1;
                if i > 0 {
                    borders[3] = gridPlanes[(i - 1) as usize][j as usize][0];
                } else if (*grid).wrapWidth != qfalse {
                    borders[3] = gridPlanes[((*grid).width - 2) as usize][j as usize][0];
                }
                noAdjust[3] = if borders[3] == gridPlanes[i as usize][j as usize][1] { 1 } else { 0 };
                if borders[3] == -1 || noAdjust[3] != 0 {
                    borders[3] = CM_EdgePlaneNum(grid, addr_of!(gridPlanes), i, j, 3);
                }

                borders[1] = -1;
                if i < (*grid).width - 2 {
                    borders[1] = gridPlanes[(i + 1) as usize][j as usize][1];
                } else if (*grid).wrapWidth != qfalse {
                    borders[1] = gridPlanes[0][j as usize][1];
                }
                noAdjust[1] = if borders[1] == gridPlanes[i as usize][j as usize][0] { 1 } else { 0 };
                if borders[1] == -1 || noAdjust[1] != 0 {
                    borders[1] = CM_EdgePlaneNum(grid, addr_of!(gridPlanes), i, j, 1);
                }

                if numFacets == MAX_FACETS {
                    Com_Error(ERR_DROP, b"MAX_FACETS\0".as_ptr() as *const c_char);
                }
                facet = facets.add(numFacets as usize);
                Com_Memset(facet as *mut c_void, 0, core::mem::size_of::<facet_t>());

                if gridPlanes[i as usize][j as usize][0] == gridPlanes[i as usize][j as usize][1] {
                    if gridPlanes[i as usize][j as usize][0] == -1 {
                        j += 1;
                        continue;		// degenrate
                    }
                    (*facet).surfacePlane = gridPlanes[i as usize][j as usize][0];
                    (*facet).numBorders = 4;
                    (*facet).borderPlanes[0] = borders[0];
                    (*facet).borderNoAdjust[0] = noAdjust[0] as c_int;
                    (*facet).borderPlanes[1] = borders[1];
                    (*facet).borderNoAdjust[1] = noAdjust[1] as c_int;
                    (*facet).borderPlanes[2] = borders[2];
                    (*facet).borderNoAdjust[2] = noAdjust[2] as c_int;
                    (*facet).borderPlanes[3] = borders[3];
                    (*facet).borderNoAdjust[3] = noAdjust[3] as c_int;
                    CM_SetBorderInward(facet, grid, addr_of!(gridPlanes), i, j, -1);
                    if CM_ValidateFacet(facet) != qfalse {
                        CM_AddFacetBevels(facet);
                        numFacets += 1;
                    }
                } else {
                    // two seperate triangles
                    (*facet).surfacePlane = gridPlanes[i as usize][j as usize][0];
                    (*facet).numBorders = 3;
                    (*facet).borderPlanes[0] = borders[0];
                    (*facet).borderNoAdjust[0] = noAdjust[0] as c_int;
                    (*facet).borderPlanes[1] = borders[1];
                    (*facet).borderNoAdjust[1] = noAdjust[1] as c_int;
                    (*facet).borderPlanes[2] = gridPlanes[i as usize][j as usize][1];
                    if (*facet).borderPlanes[2] == -1 {
                        (*facet).borderPlanes[2] = borders[2];
                        if (*facet).borderPlanes[2] == -1 {
                            (*facet).borderPlanes[2] = CM_EdgePlaneNum(grid, addr_of!(gridPlanes), i, j, 4);
                        }
                    }
     				CM_SetBorderInward(facet, grid, addr_of!(gridPlanes), i, j, 0);
                    if CM_ValidateFacet(facet) != qfalse {
                        CM_AddFacetBevels(facet);
                        numFacets += 1;
                    }

                    if numFacets == MAX_FACETS {
                        Com_Error(ERR_DROP, b"MAX_FACETS\0".as_ptr() as *const c_char);
                    }
                    facet = facets.add(numFacets as usize);
                    Com_Memset(facet as *mut c_void, 0, core::mem::size_of::<facet_t>());

                    (*facet).surfacePlane = gridPlanes[i as usize][j as usize][1];
                    (*facet).numBorders = 3;
                    (*facet).borderPlanes[0] = borders[2];
                    (*facet).borderNoAdjust[0] = noAdjust[2] as c_int;
                    (*facet).borderPlanes[1] = borders[3];
                    (*facet).borderNoAdjust[1] = noAdjust[3] as c_int;
                    (*facet).borderPlanes[2] = gridPlanes[i as usize][j as usize][0];
                    if (*facet).borderPlanes[2] == -1 {
                        (*facet).borderPlanes[2] = borders[0];
                        if (*facet).borderPlanes[2] == -1 {
                            (*facet).borderPlanes[2] = CM_EdgePlaneNum(grid, addr_of!(gridPlanes), i, j, 5);
                        }
                    }
                    CM_SetBorderInward(facet, grid, addr_of!(gridPlanes), i, j, 1);
                    if CM_ValidateFacet(facet) != qfalse {
                        CM_AddFacetBevels(facet);
                        numFacets += 1;
                    }
                }
                j += 1;
            }
            i += 1;
        }

        // copy the results out
        (*pf).numPlanes = numPlanes;
        (*pf).numFacets = numFacets;
        if numFacets != 0 {
            (*pf).facets = Hunk_Alloc((numFacets as usize) * core::mem::size_of::<facet_t>(), 1) as *mut facet_t;	// h_high = 1
            Com_Memcpy((*pf).facets as *mut c_void, facets as *const c_void, (numFacets as usize) * core::mem::size_of::<facet_t>());
        }
        else {
            (*pf).facets = core::ptr::null_mut();
        }
        (*pf).planes = Hunk_Alloc((numPlanes as usize) * core::mem::size_of::<patchPlane_t>(), 1) as *mut patchPlane_t;	// h_high = 1
        Com_Memcpy((*pf).planes as *mut c_void, addr_of!(planes) as *const c_void, (numPlanes as usize) * core::mem::size_of::<patchPlane_t>());

        Z_Free(facets as *mut c_void);
    }
}

/*
===================
CM_GeneratePatchCollide

Creates an internal structure that will be used to perform
collision detection with a patch mesh.

Points is packed as concatenated rows.
===================
*/
pub fn CM_GeneratePatchCollide(width: c_int, height: c_int, points: *const vec3_t) -> *mut patchCollide_s {
    let mut pf: *mut patchCollide_t;
    let mut grid: cGrid_t;
    let mut i: c_int;
    let mut j: c_int;

    unsafe {
        if width <= 2 || height <= 2 || points.is_null() {
            Com_Error(ERR_DROP, b"CM_GeneratePatchFacets: bad parameters: (%i, %i, %p)\0".as_ptr() as *const c_char,
                width, height, points);
        }

        if (width & 1) == 0 || (height & 1) == 0 {
            Com_Error(ERR_DROP, b"CM_GeneratePatchFacets: even sizes are invalid for quadratic meshes\0".as_ptr() as *const c_char);
        }

        if width > MAX_GRID_SIZE as c_int || height > MAX_GRID_SIZE as c_int {
            Com_Error(ERR_DROP, b"CM_GeneratePatchFacets: source is > MAX_GRID_SIZE\0".as_ptr() as *const c_char);
        }

        // build a grid
        grid = core::mem::zeroed();
        grid.width = width;
        grid.height = height;
        grid.wrapWidth = qfalse;
        grid.wrapHeight = qfalse;
        i = 0;
        while i < width {
            j = 0;
            while j < height {
                VectorCopy(points.add((j * width + i) as usize), addr_of_mut!(grid.points[i as usize][j as usize]));
                j += 1;
            }
            i += 1;
        }

        // subdivide the grid
        CM_SetGridWrapWidth(addr_of_mut!(grid));
        CM_SubdivideGridColumns(addr_of_mut!(grid));
        CM_RemoveDegenerateColumns(addr_of_mut!(grid));

        CM_TransposeGrid(addr_of_mut!(grid));

        CM_SetGridWrapWidth(addr_of_mut!(grid));
        CM_SubdivideGridColumns(addr_of_mut!(grid));
        CM_RemoveDegenerateColumns(addr_of_mut!(grid));

        // we now have a grid of points exactly on the curve
        // the aproximate surface defined by these points will be
        // collided against
        pf = Hunk_Alloc(core::mem::size_of::<patchCollide_t>(), 1) as *mut patchCollide_t;	// h_high = 1
        ClearBounds(addr_of_mut!((*pf).bounds[0]), addr_of_mut!((*pf).bounds[1]));
        i = 0;
        while i < grid.width {
            j = 0;
            while j < grid.height {
                AddPointToBounds(addr_of!(grid.points[i as usize][j as usize]), addr_of_mut!((*pf).bounds[0]), addr_of_mut!((*pf).bounds[1]));
                j += 1;
            }
            i += 1;
        }

        c_totalPatchBlocks += (grid.width - 1) * (grid.height - 1);

        // generate a bsp tree for the surface
        CM_PatchCollideFromGrid(addr_of!(grid), pf);

        // expand by one unit for epsilon purposes
        (*pf).bounds[0][0] -= 1.0;
        (*pf).bounds[0][1] -= 1.0;
        (*pf).bounds[0][2] -= 1.0;

        (*pf).bounds[1][0] += 1.0;
        (*pf).bounds[1][1] += 1.0;
        (*pf).bounds[1][2] += 1.0;

        pf as *mut patchCollide_s
    }
}

/*
================================================================================

TRACE TESTING

================================================================================
*/

/*
====================
CM_TracePointThroughPatchCollide

  special case for point traces because the patch collide "brushes" have no volume
====================
*/
fn CM_TracePointThroughPatchCollide(tw: *const traceWork_t, trace: *mut trace_t, pc: *const patchCollide_s) {
    let mut frontFacing: [qboolean; MAX_PATCH_PLANES as usize] = [qfalse; MAX_PATCH_PLANES as usize];
    let mut intersection: [f32; MAX_PATCH_PLANES as usize] = [0.0; MAX_PATCH_PLANES as usize];
    let mut intersect: f32;
    let planes_ptr: *const patchPlane_t;
    let facet: *const facet_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut offset: f32;
    let mut d1: f32;
    let mut d2: f32;
    #[cfg(not(feature = "bspc"))]
    static mut cv: *const cvar_t = core::ptr::null();

    unsafe {
        #[cfg(not(feature = "bspc"))]
        {
            if (*tw).isPoint == qfalse {
                return;
            }
            // Check cm_playerCurveClip cvar - for now we'll assume it's set
        }

        // determine the trace's relationship to all planes
        planes_ptr = (*pc).planes;
        i = 0;
        while i < (*pc).numPlanes {
            offset = DotProduct(addr_of!((*tw).offsets[(*planes_ptr.add(i as usize)).signbits as usize]), addr_of!((*planes_ptr.add(i as usize)).plane));
            d1 = DotProduct(addr_of!((*tw).start), addr_of!([(*planes_ptr.add(i as usize)).plane[0], (*planes_ptr.add(i as usize)).plane[1], (*planes_ptr.add(i as usize)).plane[2]])) - (*planes_ptr.add(i as usize)).plane[3] + offset;
            d2 = DotProduct(addr_of!((*tw).end), addr_of!([(*planes_ptr.add(i as usize)).plane[0], (*planes_ptr.add(i as usize)).plane[1], (*planes_ptr.add(i as usize)).plane[2]])) - (*planes_ptr.add(i as usize)).plane[3] + offset;
            if d1 <= 0.0 {
                frontFacing[i as usize] = qfalse;
            } else {
                frontFacing[i as usize] = qtrue;
            }
            if d1 == d2 {
                intersection[i as usize] = 99999.0;
            } else {
                intersection[i as usize] = d1 / (d1 - d2);
                if intersection[i as usize] <= 0.0 {
                    intersection[i as usize] = 99999.0;
                }
            }
            i += 1;
        }

        // see if any of the surface planes are intersected
        facet = (*pc).facets;
        i = 0;
        while i < (*pc).numFacets {
            if frontFacing[(*facet).surfacePlane as usize] == qfalse {
                facet = facet.add(1);
                i += 1;
                continue;
            }
            intersect = intersection[(*facet).surfacePlane as usize];
            if intersect < 0.0 {
                facet = facet.add(1);
                i += 1;
                continue;		// surface is behind the starting point
            }
            if intersect > (*trace).fraction {
                facet = facet.add(1);
                i += 1;
                continue;		// already hit something closer
            }
            j = 0;
            while j < (*facet).numBorders {
                k = (*facet).borderPlanes[j as usize];
                if (frontFacing[k as usize] as c_int ^ (*facet).borderInward[j as usize]) != 0 {
                    if intersection[k as usize] > intersect {
                        break;
                    }
                } else {
                    if intersection[k as usize] < intersect {
                        break;
                    }
                }
                j += 1;
            }
            if j == (*facet).numBorders {
                // we hit this facet
                #[cfg(not(feature = "bspc"))]
                {
                    if cv.is_null() {
                        cv = Cvar_Get(b"r_debugSurfaceUpdate\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
                    }
                    if !cv.is_null() && (*cv).integer != 0 {
                        debugPatchCollide = pc;
                        debugFacet = facet;
                    }
                }
                planes_ptr = addr_of!((*pc).planes[(*facet).surfacePlane as usize]);

                // calculate intersection with a slight pushoff
                offset = DotProduct(addr_of!((*tw).offsets[(*planes_ptr).signbits as usize]), addr_of!((*planes_ptr).plane));
                d1 = DotProduct(addr_of!((*tw).start), addr_of!([(*planes_ptr).plane[0], (*planes_ptr).plane[1], (*planes_ptr).plane[2]])) - (*planes_ptr).plane[3] + offset;
                d2 = DotProduct(addr_of!((*tw).end), addr_of!([(*planes_ptr).plane[0], (*planes_ptr).plane[1], (*planes_ptr).plane[2]])) - (*planes_ptr).plane[3] + offset;
                (*trace).fraction = (d1 - SURFACE_CLIP_EPSILON) / (d1 - d2);

                if (*trace).fraction < 0.0 {
                    (*trace).fraction = 0.0;
                }

                (*trace).plane.normal[0] = (*planes_ptr).plane[0];
                (*trace).plane.normal[1] = (*planes_ptr).plane[1];
                (*trace).plane.normal[2] = (*planes_ptr).plane[2];
                (*trace).plane.dist = (*planes_ptr).plane[3];
            }
            facet = facet.add(1);
            i += 1;
        }
    }
}

/*
====================
CM_CheckFacetPlane
====================
*/
#[inline]
fn CM_CheckFacetPlane(plane: *const [f32; 4], start: *const vec3_t, end: *const vec3_t, enterFrac: *mut f32, leaveFrac: *mut f32, hit: *mut c_int) -> c_int {
    let mut d1: f32;
    let mut d2: f32;
    let mut f: f32;

    unsafe {
        *hit = qfalse;

        d1 = DotProduct(start, addr_of!([(*plane)[0], (*plane)[1], (*plane)[2]])) - (*plane)[3];
        d2 = DotProduct(end, addr_of!([(*plane)[0], (*plane)[1], (*plane)[2]])) - (*plane)[3];

        // if completely in front of face, no intersection with the entire facet
        if d1 > 0.0 && (d2 >= SURFACE_CLIP_EPSILON || d2 >= d1) {
            return qfalse;
        }

        // if it doesn't cross the plane, the plane isn't relevent
        if d1 <= 0.0 && d2 <= 0.0 {
            return qtrue;
        }

        // crosses face
        if d1 > d2 {	// enter
            f = (d1 - SURFACE_CLIP_EPSILON) / (d1 - d2);
            if f < 0.0 {
                f = 0.0;
            }
            //always favor previous plane hits and thus also the surface plane hit
            if f > *enterFrac {
                *enterFrac = f;
                *hit = qtrue;
            }
        } else {	// leave
            f = (d1 + SURFACE_CLIP_EPSILON) / (d1 - d2);
            if f > 1.0 {
                f = 1.0;
            }
            if f < *leaveFrac {
                *leaveFrac = f;
            }
        }
    }
    qtrue
}

/*
====================
CM_TraceThroughPatchCollide
====================
*/
pub fn CM_TraceThroughPatchCollide(tw: *const traceWork_t, trace: *mut trace_t, pc: *const patchCollide_s) {
    let mut i: c_int;
    let mut j: c_int;
    let mut hit: c_int;
    let mut hitnum: c_int;
    let mut offset: f32;
    let mut enterFrac: f32;
    let mut leaveFrac: f32;
    let mut t: f32;
    let mut planes: *const patchPlane_t;
    let mut facet: *const facet_t;
    let mut plane: [f32; 4] = [0.0; 4];
    let mut bestplane: [f32; 4] = [0.0; 4];
    let mut startp: vec3_t = [0.0; 3];
    let mut endp: vec3_t = [0.0; 3];
    #[cfg(not(feature = "bspc"))]
    static mut cv: *const cvar_t = core::ptr::null();

    unsafe {
        #[cfg(not(feature = "cull_bbox"))]
        {
            // I'm not sure if test is strictly correct.  Are all
            // bboxes axis aligned?  Do I care?  It seems to work
            // good enough...
            i = 0;
            while i < 3 {
                if (*tw).bounds[0][i as usize] > (*pc).bounds[1][i as usize]
                    || (*tw).bounds[1][i as usize] < (*pc).bounds[0][i as usize]
                {
                    return;
                }
                i += 1;
            }
        }

        if (*tw).isPoint != qfalse {
            CM_TracePointThroughPatchCollide(tw, trace, pc);
            return;
        }
        //
        facet = (*pc).facets;
        i = 0;
        while i < (*pc).numFacets {
            enterFrac = -1.0;
            leaveFrac = 1.0;
            hitnum = -1;
            //
            planes = addr_of!((*pc).planes[(*facet).surfacePlane as usize]);
            plane[0] = (*planes).plane[0];
            plane[1] = (*planes).plane[1];
            plane[2] = (*planes).plane[2];
            plane[3] = (*planes).plane[3];
            if (*tw).sphere.use_ != qfalse {
                // adjust the plane distance apropriately for radius
                plane[3] += (*tw).sphere.radius;

                // find the closest point on the capsule to the plane
                t = DotProduct(addr_of!([plane[0], plane[1], plane[2]]), addr_of!((*tw).sphere.offset));
                if t > 0.0f32 {
                    VectorSubtract(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                    VectorSubtract(addr_of!((*tw).end), addr_of!((*tw).sphere.offset), addr_of_mut!(endp));
                }
                else {
                    VectorAdd(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                    VectorAdd(addr_of!((*tw).end), addr_of!((*tw).sphere.offset), addr_of_mut!(endp));
                }
            }
            else {
                offset = DotProduct(addr_of!((*tw).offsets[(*planes).signbits as usize]), addr_of!([plane[0], plane[1], plane[2]]));
                plane[3] -= offset;
                VectorCopy(addr_of!((*tw).start), addr_of_mut!(startp));
                VectorCopy(addr_of!((*tw).end), addr_of_mut!(endp));
            }
            //
            if CM_CheckFacetPlane(addr_of!(plane), addr_of!(startp), addr_of!(endp), addr_of_mut!(enterFrac), addr_of_mut!(leaveFrac), addr_of_mut!(hit)) == qfalse {
                facet = facet.add(1);
                i += 1;
                continue;
            }
            if hit != qfalse {
                Vector4Copy(addr_of!(plane), addr_of_mut!(bestplane));
            }
            //
            j = 0;
            while j < (*facet).numBorders {
                planes = addr_of!((*pc).planes[(*facet).borderPlanes[j as usize] as usize]);
                if (*facet).borderInward[j as usize] != qfalse {
                    let mut plane_neg = [0.0; 3];
                    VectorNegate(addr_of!([(*planes).plane[0], (*planes).plane[1], (*planes).plane[2]]), addr_of_mut!(plane_neg));
                    plane[0] = plane_neg[0];
                    plane[1] = plane_neg[1];
                    plane[2] = plane_neg[2];
                    plane[3] = -(*planes).plane[3];
                }
                else {
                    plane[0] = (*planes).plane[0];
                    plane[1] = (*planes).plane[1];
                    plane[2] = (*planes).plane[2];
                    plane[3] = (*planes).plane[3];
                }
                if (*tw).sphere.use_ != qfalse {
                    // adjust the plane distance apropriately for radius
                    plane[3] += (*tw).sphere.radius;

                    // find the closest point on the capsule to the plane
                    t = DotProduct(addr_of!([plane[0], plane[1], plane[2]]), addr_of!((*tw).sphere.offset));
                    if t > 0.0f32 {
                        VectorSubtract(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                        VectorSubtract(addr_of!((*tw).end), addr_of!((*tw).sphere.offset), addr_of_mut!(endp));
                    }
                    else {
                        VectorAdd(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                        VectorAdd(addr_of!((*tw).end), addr_of!((*tw).sphere.offset), addr_of_mut!(endp));
                    }
                }
                else {
                    // NOTE: this works even though the plane might be flipped because the bbox is centered
                    offset = DotProduct(addr_of!((*tw).offsets[(*planes).signbits as usize]), addr_of!([plane[0], plane[1], plane[2]]));
                    plane[3] += Q_fabs(offset);
                    VectorCopy(addr_of!((*tw).start), addr_of_mut!(startp));
                    VectorCopy(addr_of!((*tw).end), addr_of_mut!(endp));
                }
                //
                if CM_CheckFacetPlane(addr_of!(plane), addr_of!(startp), addr_of!(endp), addr_of_mut!(enterFrac), addr_of_mut!(leaveFrac), addr_of_mut!(hit)) == qfalse {
                    break;
                }
                if hit != qfalse {
                    hitnum = j;
                    Vector4Copy(addr_of!(plane), addr_of_mut!(bestplane));
                }
                j += 1;
            }
            if j < (*facet).numBorders {
                facet = facet.add(1);
                i += 1;
                continue;
            }
            //never clip against the back side
            if hitnum == (*facet).numBorders - 1 {
                facet = facet.add(1);
                i += 1;
                continue;
            }
            //
            if enterFrac < leaveFrac && enterFrac >= 0.0 {
                if enterFrac < (*trace).fraction {
                    if enterFrac < 0.0 {
                        enterFrac = 0.0;
                    }
                    #[cfg(not(feature = "bspc"))]
                    {
                        if cv.is_null() {
                            cv = Cvar_Get(b"r_debugSurfaceUpdate\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0);
                        }
                        if !cv.is_null() && (*cv).integer != 0 {
                            debugPatchCollide = pc;
                            debugFacet = facet;
                        }
                    }

                    (*trace).fraction = enterFrac;
                    (*trace).plane.normal[0] = bestplane[0];
                    (*trace).plane.normal[1] = bestplane[1];
                    (*trace).plane.normal[2] = bestplane[2];
                    (*trace).plane.dist = bestplane[3];
                }
            }
            facet = facet.add(1);
            i += 1;
        }
    }
}

/*
=======================================================================

POSITION DETECTION

=======================================================================
*/

/*
====================
CM_PositionTestInPatchCollide

Modifies tr->tr if any of the facets effect the trace
====================
*/
pub fn CM_PositionTestInPatchCollide(tw: *const traceWork_t, pc: *const patchCollide_s) -> qboolean {
    let mut i: c_int;
    let mut j: c_int;
    let mut offset: f32;
    let mut t: f32;
    let mut planes: *const patchPlane_t;
    let facet: *const facet_t;
    let mut plane: [f32; 4] = [0.0; 4];
    let mut startp: vec3_t = [0.0; 3];

    unsafe {
        if (*tw).isPoint != qfalse {
            return qfalse;
        }
        //
        facet = (*pc).facets;
        i = 0;
        while i < (*pc).numFacets {
            planes = addr_of!((*pc).planes[(*facet).surfacePlane as usize]);
            plane[0] = (*planes).plane[0];
            plane[1] = (*planes).plane[1];
            plane[2] = (*planes).plane[2];
            plane[3] = (*planes).plane[3];
            if (*tw).sphere.use_ != qfalse {
                // adjust the plane distance apropriately for radius
                plane[3] += (*tw).sphere.radius;

                // find the closest point on the capsule to the plane
                t = DotProduct(addr_of!([plane[0], plane[1], plane[2]]), addr_of!((*tw).sphere.offset));
                if t > 0.0 {
                    VectorSubtract(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                }
                else {
                    VectorAdd(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                }
            }
            else {
                offset = DotProduct(addr_of!((*tw).offsets[(*planes).signbits as usize]), addr_of!([plane[0], plane[1], plane[2]]));
                plane[3] -= offset;
                VectorCopy(addr_of!((*tw).start), addr_of_mut!(startp));
            }

            if DotProduct(addr_of!(startp), addr_of!([plane[0], plane[1], plane[2]])) - plane[3] > 0.0f32 {
                facet = facet.add(1);
                i += 1;
                continue;
            }

            j = 0;
            while j < (*facet).numBorders {
                planes = addr_of!((*pc).planes[(*facet).borderPlanes[j as usize] as usize]);
                if (*facet).borderInward[j as usize] != qfalse {
                    let mut plane_neg = [0.0; 3];
                    VectorNegate(addr_of!([(*planes).plane[0], (*planes).plane[1], (*planes).plane[2]]), addr_of_mut!(plane_neg));
                    plane[0] = plane_neg[0];
                    plane[1] = plane_neg[1];
                    plane[2] = plane_neg[2];
                    plane[3] = -(*planes).plane[3];
                }
                else {
                    plane[0] = (*planes).plane[0];
                    plane[1] = (*planes).plane[1];
                    plane[2] = (*planes).plane[2];
                    plane[3] = (*planes).plane[3];
                }
                if (*tw).sphere.use_ != qfalse {
                    // adjust the plane distance apropriately for radius
                    plane[3] += (*tw).sphere.radius;

                    // find the closest point on the capsule to the plane
                    t = DotProduct(addr_of!([plane[0], plane[1], plane[2]]), addr_of!((*tw).sphere.offset));
                    if t > 0.0f32 {
                        VectorSubtract(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                    }
                    else {
                        VectorAdd(addr_of!((*tw).start), addr_of!((*tw).sphere.offset), addr_of_mut!(startp));
                    }
                }
                else {
                    // NOTE: this works even though the plane might be flipped because the bbox is centered
                    offset = DotProduct(addr_of!((*tw).offsets[(*planes).signbits as usize]), addr_of!([plane[0], plane[1], plane[2]]));
                    plane[3] += Q_fabs(offset);
                    VectorCopy(addr_of!((*tw).start), addr_of_mut!(startp));
                }

                if DotProduct(addr_of!(startp), addr_of!([plane[0], plane[1], plane[2]])) - plane[3] > 0.0f32 {
                    break;
                }
                j += 1;
            }
            if j < (*facet).numBorders {
                facet = facet.add(1);
                i += 1;
                continue;
            }
            // inside this patch facet
            return qtrue;
        }
    }

    qfalse
}

/*
=======================================================================

DEBUGGING

=======================================================================
*/

/*
==================
CM_DrawDebugSurface

Called from the renderer
==================
*/
#[cfg(not(feature = "bspc"))]
extern "C" {
    fn BotDrawDebugPolygons(drawPoly: *const c_void, value: c_int);
}

pub fn CM_DrawDebugSurface(drawPoly: extern "C" fn(c_int, c_int, *mut f32)) {
    static mut cv: *const cvar_t = core::ptr::null();
    #[cfg(not(feature = "bspc"))]
    static mut cv2: *const cvar_t = core::ptr::null();
    let pc: *const patchCollide_t;
    let facet: *const facet_t;
    let mut w: *mut winding_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut n: c_int;
    let mut curplanenum: c_int;
    let mut planenum: c_int;
    let mut curinward: c_int;
    let mut inward: c_int;
    let mut plane: [f32; 4] = [0.0; 4];
    let mut mins: vec3_t = [-15.0, -15.0, -28.0];
    let mut maxs: vec3_t = [15.0, 15.0, 28.0];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    unsafe {
        #[cfg(not(feature = "bspc"))]
        {
            if cv2.is_null() {
                cv2 = Cvar_Get(b"r_debugSurface\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
            }

            if (*cv2).integer != 1 {
                BotDrawDebugPolygons(drawPoly as *const c_void, (*cv2).integer);
                return;
            }
        }

        if debugPatchCollide.is_null() {
            return;
        }

        #[cfg(not(feature = "bspc"))]
        {
            if cv.is_null() {
                cv = Cvar_Get(b"cm_debugSize\0".as_ptr() as *const c_char, b"2\0".as_ptr() as *const c_char, 0);
            }
        }
        pc = debugPatchCollide;

        i = 0;
        facet = (*pc).facets;
        while i < (*pc).numFacets {

            k = 0;
            while k < (*facet).numBorders + 1 {
                //
                if k < (*facet).numBorders {
                    planenum = (*facet).borderPlanes[k as usize];
                    inward = (*facet).borderInward[k as usize];
                } else {
                    planenum = (*facet).surfacePlane;
                    inward = qfalse;
                    //continue;
                }

                Vector4Copy(addr_of!((*pc).planes[planenum as usize].plane), addr_of_mut!(plane));

                //planenum = facet->surfacePlane;
                if inward != qfalse {
                    let mut plane_neg = [0.0; 3];
                    VectorNegate(addr_of!([plane[0], plane[1], plane[2]]), addr_of_mut!(plane_neg));
                    plane[0] = plane_neg[0];
                    plane[1] = plane_neg[1];
                    plane[2] = plane_neg[2];
                    plane[3] = -plane[3];
                }

                plane[3] += (*cv).value;
                //*
                n = 0;
                while n < 3 {
                    if plane[n as usize] > 0.0 {
                        v1[n as usize] = maxs[n as usize];
                    } else {
                        v1[n as usize] = mins[n as usize];
                    }
                    n += 1;
                }
                let mut v2_neg = [0.0; 3];
                VectorNegate(addr_of!([plane[0], plane[1], plane[2]]), addr_of_mut!(v2_neg));
                v2[0] = v2_neg[0];
                v2[1] = v2_neg[1];
                v2[2] = v2_neg[2];
                plane[3] += Q_fabs(DotProduct(addr_of!(v1), addr_of!(v2)));
                //*/

                w = BaseWindingForPlane(addr_of_mut!([plane[0], plane[1], plane[2]]), plane[3]);
                j = 0;
                while j < (*facet).numBorders + 1 && !w.is_null() {
                    //
                    if j < (*facet).numBorders {
                        curplanenum = (*facet).borderPlanes[j as usize];
                        curinward = (*facet).borderInward[j as usize];
                    } else {
                        curplanenum = (*facet).surfacePlane;
                        curinward = qfalse;
                        //continue;
                    }
                    //
                    if curplanenum == planenum {
                        j += 1;
                        continue;
                    }

                    Vector4Copy(addr_of!((*pc).planes[curplanenum as usize].plane), addr_of_mut!(plane));
                    if curinward == qfalse {
                        let mut plane_neg = [0.0; 3];
                        VectorNegate(addr_of!([plane[0], plane[1], plane[2]]), addr_of_mut!(plane_neg));
                        plane[0] = plane_neg[0];
                        plane[1] = plane_neg[1];
                        plane[2] = plane_neg[2];
                        plane[3] = -plane[3];
                    }
            //			if ( !facet->borderNoAdjust[j] ) {
                    plane[3] -= (*cv).value;
            //			}
                    n = 0;
                    while n < 3 {
                        if plane[n as usize] > 0.0 {
                            v1[n as usize] = maxs[n as usize];
                        } else {
                            v1[n as usize] = mins[n as usize];
                        }
                        n += 1;
                    }
                    let mut v2_neg = [0.0; 3];
                    VectorNegate(addr_of!([plane[0], plane[1], plane[2]]), addr_of_mut!(v2_neg));
                    v2[0] = v2_neg[0];
                    v2[1] = v2_neg[1];
                    v2[2] = v2_neg[2];
                    plane[3] -= Q_fabs(DotProduct(addr_of!(v1), addr_of!(v2)));

                    ChopWindingInPlace(addr_of_mut!(w), addr_of_mut!([plane[0], plane[1], plane[2]]), plane[3], 0.1f32);
                    j += 1;
                }
                if !w.is_null() {
                    if facet == debugFacet {
                        drawPoly(4, (*w).numpoints, (*w).p[0].as_mut_ptr());
                        //Com_Printf("blue facet has %d border planes\n", facet->numBorders);
                    } else {
                        drawPoly(1, (*w).numpoints, (*w).p[0].as_mut_ptr());
                    }
                    FreeWinding(w);
                }
                else {
                    Com_Printf(b"winding chopped away by border planes\n\0".as_ptr() as *const c_char);
                }
                k += 1;
            }
            facet = facet.add(1);
            i += 1;
        }

        // draw the debug block
        {
            let mut v: [vec3_t; 3] = [[0.0; 3]; 3];

            VectorCopy(addr_of!(debugBlockPoints[0]), addr_of_mut!(v[0]));
            VectorCopy(addr_of!(debugBlockPoints[1]), addr_of_mut!(v[1]));
            VectorCopy(addr_of!(debugBlockPoints[2]), addr_of_mut!(v[2]));
            drawPoly(2, 3, v[0].as_mut_ptr());

            VectorCopy(addr_of!(debugBlockPoints[2]), addr_of_mut!(v[0]));
            VectorCopy(addr_of!(debugBlockPoints[3]), addr_of_mut!(v[1]));
            VectorCopy(addr_of!(debugBlockPoints[0]), addr_of_mut!(v[2]));
            drawPoly(2, 3, v[0].as_mut_ptr());
        }
    }
}
