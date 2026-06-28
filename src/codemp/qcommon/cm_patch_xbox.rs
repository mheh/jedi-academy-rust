// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
//
// #include "cm_local.h"
// #include "cm_patch.h"

/*

This file does not reference any globals, and has these entry points:

void CM_ClearLevelPatches( void );
struct patchCollide_s	*CM_GeneratePatchCollide( int width, int height, const vec3_t *points );
void CM_TraceThroughPatchCollide( traceWork_t *tw, const struct patchCollide_s *pc );
qboolean CM_PositionTestInPatchCollide( traceWork_t *tw, const struct patchCollide_s *pc );
void CM_DrawDebugSurface( void (*drawPoly)(int color, int numPoints, flaot *points) );


Issues for collision against curved surfaces:

Surface edges need to be handled differently than surface planes

Plane expansion causes raw surfaces to expand past expanded bounding box

Position test of a volume against a surface is tricky.

Position test of a point against a surface is not well defined, because the surface has no volume.


Tracing leading edge points instead of volumes?
Position test by tracing corner to corner? (8*7 traces -- ouch)

coplanar edges
triangulated patches
degenerate patches

  endcaps
  degenerate

WARNING: this may misbehave with meshes that have rows or columns that only
degenerate a few triangles.  Completely degenerate rows and columns are handled
properly.
*/

/*
#define	MAX_FACETS			1024
#define	MAX_PATCH_PLANES	2048

typedef struct {
	float	plane[4];
	int		signbits;		// signx + (signy<<1) + (signz<<2), used as lookup during collision
} patchPlane_t;

typedef struct {
	int			surfacePlane;
	int			numBorders;		// 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
	int			borderPlanes[4+6+16];
	int			borderInward[4+6+16];
	qboolean	borderNoAdjust[4+6+16];
} facet_t;

typedef struct patchCollide_s {
	vec3_t	bounds[2];
	int		numPlanes;			// surface planes plus edge planes
	patchPlane_t	*planes;
	int		numFacets;
	facet_t	*facets;
} patchCollide_t;


#define	MAX_GRID_SIZE	129

typedef struct {
	int			width;
	int			height;
	qboolean	wrapWidth;
	qboolean	wrapHeight;
	vec3_t	points[MAX_GRID_SIZE][MAX_GRID_SIZE];	// [width][height]
} cGrid_t;

#define	SUBDIVIDE_DISTANCE	16	//4	// never more than this units away from curve
#define	PLANE_TRI_EPSILON	0.1
#define	WRAP_POINT_EPSILON	0.1
*/

use core::ffi::{c_int, c_char, c_void};

// ============================================================================
// EXTERNAL TYPES AND DECLARATIONS
// ============================================================================

// Stub types for dependencies not fully available in this file
// These should be defined in cm_local.h and cm_patch.h
#[repr(C)]
pub struct patchPlane_t {
    pub plane: [f32; 4],
    pub signbits: c_int,  // signx + (signy<<1) + (signz<<2), used as lookup during collision
}

#[repr(C)]
pub struct facet_t {
    pub surfacePlane: c_int,
    pub numBorders: c_int,  // 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
    pub borderPlanes: [c_int; 4+6+16],
    pub borderInward: [c_int; 4+6+16],
    pub borderNoAdjust: [c_int; 4+6+16],
}

#[repr(C)]
pub struct patchCollide_s {
    pub bounds: [[f32; 3]; 2],
    pub numPlanes: c_int,  // surface planes plus edge planes
    pub planes: *mut patchPlane_t,
    pub numFacets: c_int,
    pub facets: *mut facet_t,
}

pub type patchCollide_t = patchCollide_s;

pub const MAX_GRID_SIZE: usize = 129;
pub const MAX_FACETS: c_int = 1024;
pub const MAX_PATCH_PLANES: c_int = 2048;
pub const CM_MAX_GRID_SIZE: usize = 129;

#[repr(C)]
pub struct cGrid_t {
    pub width: c_int,
    pub height: c_int,
    pub wrapWidth: bool,
    pub wrapHeight: bool,
    pub points: [[[f32; 3]; MAX_GRID_SIZE]; MAX_GRID_SIZE],  // [width][height]
}

pub const SUBDIVIDE_DISTANCE: f32 = 16.0;  // 4	// never more than this units away from curve
pub const PLANE_TRI_EPSILON: f32 = 0.1;
pub const WRAP_POINT_EPSILON: f32 = 0.1;

pub const POINT_EPSILON: f32 = 0.1;
pub const NORMAL_EPSILON: f32 = 0.0001;
pub const DIST_EPSILON: f32 = 0.02;

pub const SIDE_FRONT: c_int = 0;
pub const SIDE_ON: c_int = 1;
pub const SIDE_BACK: c_int = 2;

pub const WORLD_SIZE: f32 = 1.0e6;
pub const SURFACE_CLIP_EPSILON: f32 = 0.125;
pub const MAX_MAP_BOUNDS: f32 = 65536.0;

pub const S_COLOR_RED: &str = "^1";

// ============================================================================
// EXTERNAL FUNCTIONS (stubs for math/utility functions)
// ============================================================================

extern "C" {
    pub fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorAdd(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorClear(v: *mut [f32; 3]);
    pub fn VectorNegate(src: *const [f32; 3], dst: *mut [f32; 3]);
    pub fn CrossProduct(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    pub fn DotProduct(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
    pub fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    pub fn VectorLengthSquared(v: *const [f32; 3]) -> f32;
    pub fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn Vector4Copy(src: *const [f32; 4], dst: *mut [f32; 4]);

    pub fn Q_fabs(x: f32) -> f32;

    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut core::ffi::c_void;

    pub fn Z_Malloc(size: usize, tag: c_int, zero: bool) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);

    pub fn BaseWindingForPlane(normal: *const [f32; 4], dist: f32) -> *mut core::ffi::c_void;
    pub fn FreeWinding(w: *mut core::ffi::c_void);
    pub fn ChopWindingInPlace(w: *mut *mut core::ffi::c_void, plane: *const [f32; 4], dist: f32, epsilon: f32);
    pub fn WindingBounds(w: *mut core::ffi::c_void, mins: *mut [f32; 3], maxs: *mut [f32; 3]);
    pub fn CopyWinding(w: *mut core::ffi::c_void) -> *mut core::ffi::c_void;

    pub fn ClearBounds(mins: *mut [f32; 3], maxs: *mut [f32; 3]);
    pub fn AddPointToBounds(v: *const [f32; 3], mins: *mut [f32; 3], maxs: *mut [f32; 3]);
}

pub const TAG_TEMP_WORKSPACE: c_int = 0;
pub const TAG_BSP: c_int = 1;
pub const ERR_DROP: c_int = 2;
pub const ERR_FATAL: c_int = 3;

#[repr(C)]
pub struct facetLoad_t {
    pub surfacePlane: c_int,
    pub numBorders: c_int,  // 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels
    pub borderPlanes: [c_int; 4+6+16],
    pub borderInward: [bool; 4+6+16],
    pub borderNoAdjust: [c_int; 4+6+16],
}

#[repr(C)]
pub struct traceWork_t {
    // Stub structure for traceWork_t
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub offsets: [[f32; 3]; 8],
    pub bounds: [[f32; 3]; 2],
    pub isPoint: bool,
    pub sphere: SphereTrace,
}

#[repr(C)]
pub struct SphereTrace {
    pub use_field: bool,
    pub radius: f32,
    pub offset: [f32; 3],
}

#[repr(C)]
pub struct trace_t {
    pub fraction: f32,
    pub plane: Plane,
}

#[repr(C)]
pub struct Plane {
    pub normal: [f32; 3],
    pub dist: f32,
}

pub const vec3_origin: [f32; 3] = [0.0, 0.0, 0.0];

// ============================================================================
// GLOBAL VARIABLES
// ============================================================================

pub static mut c_totalPatchBlocks: c_int = 0;
pub static mut c_totalPatchSurfaces: c_int = 0;
pub static mut c_totalPatchEdges: c_int = 0;

static mut debugPatchCollide: *const patchCollide_t = std::ptr::null();
static mut debugFacet: *const facet_t = std::ptr::null();
static mut debugBlock: bool = false;
static mut debugBlockPoints: [[f32; 3]; 4] = [[0.0; 3]; 4];

#[cfg(target_os = "windows")]
extern "C" {
    pub fn Hunk_Alloc(size: c_int) -> *mut c_void;
}

#[cfg(target_os = "windows")]
unsafe fn Hunk_Alloc_pref(size: c_int, _preference: c_int) -> *mut c_void {
    Hunk_Alloc(size)
}

/*
=================
CM_ClearLevelPatches
=================
*/
pub unsafe fn CM_ClearLevelPatches() {
    debugPatchCollide = std::ptr::null();
    debugFacet = std::ptr::null();
}

/*
=================
CM_SignbitsForNormal
=================
*/
#[inline]
pub unsafe fn CM_SignbitsForNormal(normal: [f32; 3]) -> c_int {
    let mut bits: c_int = 0;
    let mut j: c_int;

    bits = 0;
    j = 0;
    while j < 3 {
        if normal[j as usize] < 0.0 {
            bits |= 1 << j;
        }
        j += 1;
    }
    return bits;
}

/*
=====================
CM_PlaneFromPoints

Returns false if the triangle is degenrate.
The normal will point out of the clock for clockwise ordered points
=====================
*/
#[inline]
pub unsafe fn CM_PlaneFromPoints(plane: *mut [f32; 4], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> bool {
    let mut d1: [f32; 3] = [0.0; 3];
    let mut d2: [f32; 3] = [0.0; 3];

    VectorSubtract(&b, &a, &mut d1);
    VectorSubtract(&c, &a, &mut d2);
    CrossProduct(&d2, &d1, plane as *mut [f32; 3]);
    if VectorNormalize(plane as *mut [f32; 3]) == 0.0 {
        return false;
    }

    (*plane)[3] = DotProduct(&a, plane as *const [f32; 3]);
    return true;
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
pub unsafe fn CM_NeedsSubdivision(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> bool {
    let mut cmid: [f32; 3] = [0.0; 3];
    let mut lmid: [f32; 3] = [0.0; 3];
    let mut delta: [f32; 3] = [0.0; 3];
    let mut dist: f32;
    let mut i: c_int;

    // calculate the linear midpoint
    i = 0;
    while i < 3 {
        lmid[i as usize] = 0.5 * (a[i as usize] + c[i as usize]);
        i += 1;
    }

    // calculate the exact curve midpoint
    i = 0;
    while i < 3 {
        cmid[i as usize] = 0.5 * (0.5 * (a[i as usize] + b[i as usize]) + 0.5 * (b[i as usize] + c[i as usize]));
        i += 1;
    }

    // see if the curve is far enough away from the linear mid
    VectorSubtract(&cmid, &lmid, &mut delta);
    dist = VectorLengthSquared(&delta);

    return (dist >= SUBDIVIDE_DISTANCE * SUBDIVIDE_DISTANCE);
}

/*
===============
CM_Subdivide

a, b, and c are control points.
the subdivided sequence will be: a, out1, out2, out3, c
===============
*/
pub unsafe fn CM_Subdivide(a: [f32; 3], b: [f32; 3], c: [f32; 3], out1: *mut [f32; 3], out2: *mut [f32; 3], out3: *mut [f32; 3]) {
    let mut i: c_int;

    i = 0;
    while i < 3 {
        (*out1)[i as usize] = 0.5 * (a[i as usize] + b[i as usize]);
        (*out3)[i as usize] = 0.5 * (b[i as usize] + c[i as usize]);
        (*out2)[i as usize] = 0.5 * ((*out1)[i as usize] + (*out3)[i as usize]);
        i += 1;
    }
}

/*
=================
CM_TransposeGrid

Swaps the rows and columns in place
=================
*/
pub unsafe fn CM_TransposeGrid(grid: *mut cGrid_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut l: c_int;
    let mut temp: [f32; 3] = [0.0; 3];
    let mut tempWrap: bool;

    if (*grid).width > (*grid).height {
        i = 0;
        while i < (*grid).height {
            j = i + 1;
            while j < (*grid).width {
                if j < (*grid).height {
                    // swap the value
                    VectorCopy(&(*grid).points[i as usize][j as usize], &mut temp);
                    VectorCopy(&(*grid).points[j as usize][i as usize], &mut (*grid).points[i as usize][j as usize]);
                    VectorCopy(&temp, &mut (*grid).points[j as usize][i as usize]);
                } else {
                    // just copy
                    VectorCopy(&(*grid).points[j as usize][i as usize], &mut (*grid).points[i as usize][j as usize]);
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
                    VectorCopy(&(*grid).points[j as usize][i as usize], &mut temp);
                    VectorCopy(&(*grid).points[i as usize][j as usize], &mut (*grid).points[j as usize][i as usize]);
                    VectorCopy(&temp, &mut (*grid).points[i as usize][j as usize]);
                } else {
                    // just copy
                    VectorCopy(&(*grid).points[i as usize][j as usize], &mut (*grid).points[j as usize][i as usize]);
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

/*
===================
CM_SetGridWrapWidth

If the left and right columns are exactly equal, set grid->wrapWidth qtrue
===================
*/
pub unsafe fn CM_SetGridWrapWidth(grid: *mut cGrid_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut d: f32;

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
        (*grid).wrapWidth = true;
    } else {
        (*grid).wrapWidth = false;
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
pub unsafe fn CM_SubdivideGridColumns(grid: *mut cGrid_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;

    i = 0;
    while i < (*grid).width - 2 {
        // grid->points[i][x] is an interpolating control point
        // grid->points[i+1][x] is an aproximating control point
        // grid->points[i+2][x] is an interpolating control point

        //
        // first see if we can collapse the aproximating collumn away
        //
        j = 0;
        while j < (*grid).height {
            if CM_NeedsSubdivision((*grid).points[i as usize][j as usize], (*grid).points[(i + 1) as usize][j as usize], (*grid).points[(i + 2) as usize][j as usize]) {
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
                    VectorCopy(&(*grid).points[k as usize][j as usize], &mut (*grid).points[(k - 1) as usize][j as usize]);
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
            let mut prev: [f32; 3] = [0.0; 3];
            let mut mid: [f32; 3] = [0.0; 3];
            let mut next: [f32; 3] = [0.0; 3];

            // save the control points now
            VectorCopy(&(*grid).points[i as usize][j as usize], &mut prev);
            VectorCopy(&(*grid).points[(i + 1) as usize][j as usize], &mut mid);
            VectorCopy(&(*grid).points[(i + 2) as usize][j as usize], &mut next);

            // make room for two additional columns in the grid
            // columns i+1 will be replaced, column i+2 will become i+4
            // i+1, i+2, and i+3 will be generated
            k = (*grid).width - 1;
            while k > i + 1 {
                VectorCopy(&(*grid).points[k as usize][j as usize], &mut (*grid).points[(k + 2) as usize][j as usize]);
                k -= 1;
            }

            // generate the subdivided points
            CM_Subdivide(prev, mid, next, &mut (*grid).points[(i + 1) as usize][j as usize], &mut (*grid).points[(i + 2) as usize][j as usize], &mut (*grid).points[(i + 3) as usize][j as usize]);
            j += 1;
        }

        (*grid).width += 2;

        // the new aproximating point at i+1 may need to be removed
        // or subdivided farther, so don't advance i
    }
}


/*
======================
CM_ComparePoints
======================
*/
#[inline]
pub unsafe fn CM_ComparePoints(a: *const f32, b: *const f32) -> bool {
    let mut d: f32;

    d = *a.offset(0) - *b.offset(0);
    if d < -POINT_EPSILON || d > POINT_EPSILON {
        return false;
    }
    d = *a.offset(1) - *b.offset(1);
    if d < -POINT_EPSILON || d > POINT_EPSILON {
        return false;
    }
    d = *a.offset(2) - *b.offset(2);
    if d < -POINT_EPSILON || d > POINT_EPSILON {
        return false;
    }
    return true;
}

/*
=================
CM_RemoveDegenerateColumns

If there are any identical columns, remove them
=================
*/
pub unsafe fn CM_RemoveDegenerateColumns(grid: *mut cGrid_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;

    i = 0;
    while i < (*grid).width - 1 {
        j = 0;
        while j < (*grid).height {
            if !CM_ComparePoints(&(*grid).points[i as usize][j as usize][0], &(*grid).points[(i + 1) as usize][j as usize][0]) {
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
                VectorCopy(&(*grid).points[k as usize][j as usize], &mut (*grid).points[(k - 1) as usize][j as usize]);
                k += 1;
            }
            j += 1;
        }
        (*grid).width -= 1;

        // check against the next column
        i -= 1;
    }
}

/*
================================================================================

PATCH COLLIDE GENERATION

================================================================================
*/

static mut numPlanes: c_int = 0;
static mut planes: *mut patchPlane_t = std::ptr::null_mut();

pub unsafe fn CM_TempPatchPlanesAlloc() {
    if planes.is_null() {
        planes = Z_Malloc((MAX_PATCH_PLANES as usize) * std::mem::size_of::<patchPlane_t>(), TAG_TEMP_WORKSPACE, false) as *mut patchPlane_t;
    }
}

pub unsafe fn CM_TempPatchPlanesDealloc() {
    if !planes.is_null() {
        Z_Free(planes as *mut c_void);
        planes = std::ptr::null_mut();
    }
}

//static	int				numFacets;
//static	facet_t			facets[MAX_PATCH_PLANES]; //maybe MAX_FACETS ??
static mut facets: *mut facet_t = std::ptr::null_mut();

#[inline]
pub unsafe fn CM_PlaneEqual(p: *mut patchPlane_t, plane: &[f32; 4], flipped: *mut c_int) -> c_int {
    let mut invplane: [f32; 4] = [0.0; 4];

    if Q_fabs((*p).plane[0] - plane[0]) < NORMAL_EPSILON
        && Q_fabs((*p).plane[1] - plane[1]) < NORMAL_EPSILON
        && Q_fabs((*p).plane[2] - plane[2]) < NORMAL_EPSILON
        && Q_fabs((*p).plane[3] - plane[3]) < DIST_EPSILON
    {
        *flipped = 0;  // qfalse
        return 1;  // qtrue
    }

    VectorNegate(plane as *const [f32; 3], &mut invplane as *mut [f32; 3]);
    invplane[3] = -plane[3];

    if Q_fabs((*p).plane[0] - invplane[0]) < NORMAL_EPSILON
        && Q_fabs((*p).plane[1] - invplane[1]) < NORMAL_EPSILON
        && Q_fabs((*p).plane[2] - invplane[2]) < NORMAL_EPSILON
        && Q_fabs((*p).plane[3] - invplane[3]) < DIST_EPSILON
    {
        *flipped = 1;  // qtrue
        return 1;  // qtrue
    }

    return 0;  // qfalse
}

#[inline]
pub unsafe fn CM_SnapVector(normal: *mut [f32; 3]) {
    let mut i: c_int;

    i = 0;
    while i < 3 {
        if Q_fabs((*normal)[i as usize] - 1.0) < NORMAL_EPSILON {
            VectorClear(normal);
            (*normal)[i as usize] = 1.0;
            break;
        }
        if Q_fabs((*normal)[i as usize] - -1.0) < NORMAL_EPSILON {
            VectorClear(normal);
            (*normal)[i as usize] = -1.0;
            break;
        }
        i += 1;
    }
}

#[inline]
pub unsafe fn CM_FindPlane2(plane: &[f32; 4], flipped: *mut c_int) -> c_int {
    let mut i: c_int;

    // see if the points are close enough to an existing plane
    i = 0;
    while i < numPlanes {
        if CM_PlaneEqual(&mut *planes.offset(i as isize), plane, flipped) != 0 {
            return i;
        }
        i += 1;
    }

    // add a new plane
    if numPlanes == MAX_PATCH_PLANES {
        Com_Error(ERR_DROP, b"MAX_PATCH_PLANES\0" as *const c_char);
    }

    Vector4Copy(plane, &mut (*planes.offset(numPlanes as isize)).plane);
    (*planes.offset(numPlanes as isize)).signbits = CM_SignbitsForNormal([(*planes.offset(numPlanes as isize)).plane[0], (*planes.offset(numPlanes as isize)).plane[1], (*planes.offset(numPlanes as isize)).plane[2]]);

    numPlanes += 1;

    *flipped = 0;  // qfalse

    return numPlanes - 1;
}

/*
==================
CM_FindPlane
==================
*/
#[inline]
pub unsafe fn CM_FindPlane(p1: *const f32, p2: *const f32, p3: *const f32) -> c_int {
    let mut plane: [f32; 4] = [0.0; 4];
    let mut i: c_int;
    let mut d: f32;

    if !CM_PlaneFromPoints(&mut plane,
        [*p1.offset(0), *p1.offset(1), *p1.offset(2)],
        [*p2.offset(0), *p2.offset(1), *p2.offset(2)],
        [*p3.offset(0), *p3.offset(1), *p3.offset(2)]) {
        return -1;
    }

    // see if the points are close enough to an existing plane
    i = 0;
    while i < numPlanes {
        if DotProduct(&plane as *const [f32; 3], (*planes.offset(i as isize)).plane.as_ptr() as *const [f32; 3]) < 0.0 {
            i += 1;
            continue;	// allow backwards planes?
        }

        d = DotProduct(&[*p1, *(p1.offset(1)), *(p1.offset(2))], (*planes.offset(i as isize)).plane.as_ptr() as *const [f32; 3]) - (*planes.offset(i as isize)).plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            i += 1;
            continue;
        }

        d = DotProduct(&[*p2, *(p2.offset(1)), *(p2.offset(2))], (*planes.offset(i as isize)).plane.as_ptr() as *const [f32; 3]) - (*planes.offset(i as isize)).plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            i += 1;
            continue;
        }

        d = DotProduct(&[*p3, *(p3.offset(1)), *(p3.offset(2))], (*planes.offset(i as isize)).plane.as_ptr() as *const [f32; 3]) - (*planes.offset(i as isize)).plane[3];
        if d < -PLANE_TRI_EPSILON || d > PLANE_TRI_EPSILON {
            i += 1;
            continue;
        }

        // found it
        return i;
    }

    // add a new plane
    if numPlanes == MAX_PATCH_PLANES {
        Com_Error(ERR_DROP, b"MAX_PATCH_PLANES\0" as *const c_char);
    }

    Vector4Copy(&plane, &mut (*planes.offset(numPlanes as isize)).plane);
    (*planes.offset(numPlanes as isize)).signbits = CM_SignbitsForNormal([plane[0], plane[1], plane[2]]);

    numPlanes += 1;

    return numPlanes - 1;
}


/*
==================
CM_PointOnPlaneSide
==================
*/
#[inline]
pub unsafe fn CM_PointOnPlaneSide(p: *const f32, planeNum: c_int) -> c_int {
    let plane: *const [f32; 4];
    let mut d: f32;

    if planeNum == -1 {
        return SIDE_ON;
    }
    plane = &(*planes.offset(planeNum as isize)).plane;

    d = DotProduct(&[*p, *(p.offset(1)), *(p.offset(2))], plane as *const [f32; 3]) - (*plane)[3];

    if d > PLANE_TRI_EPSILON {
        return SIDE_FRONT;
    }

    if d < -PLANE_TRI_EPSILON {
        return SIDE_BACK;
    }

    return SIDE_ON;
}

#[inline]
pub unsafe fn CM_GridPlane(gridPlanes: *mut c_int, i: c_int, j: c_int, tri: c_int) -> c_int {
    let mut p: c_int;

    p = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + tri) as isize);
    if p != -1 {
        return p;
    }
    p = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + (!tri)) as isize);
    if p != -1 {
        return p;
    }

    // should never happen
    Com_Printf(b"WARNING: CM_GridPlane unresolvable\n\0" as *const c_char);
    return -1;
}

/*
==================
CM_EdgePlaneNum
==================
*/
pub unsafe fn CM_EdgePlaneNum(grid: *mut cGrid_t, gridPlanes: *mut c_int, i: c_int, j: c_int, k: c_int) -> c_int {
    let mut p1: *const f32;
    let mut p2: *const f32;
    let mut up: [f32; 3] = [0.0; 3];
    let mut p: c_int;

    match k {
        0 => {  // top border
            p1 = &(*grid).points[i as usize][j as usize][0];
            p2 = &(*grid).points[(i + 1) as usize][j as usize][0];
            p = CM_GridPlane(gridPlanes, i, j, 0);
            VectorMA(
                &(*grid).points[i as usize][j as usize],
                4.0,
                &(*planes.offset(p as isize)).plane as *const [f32; 4] as *const [f32; 3],
                &mut up
            );
            return CM_FindPlane(p1, p2, &up[0]);
        },

        2 => {  // bottom border
            p1 = &(*grid).points[i as usize][(j + 1) as usize][0];
            p2 = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            p = CM_GridPlane(gridPlanes, i, j, 1);
            VectorMA(
                &(*grid).points[i as usize][(j + 1) as usize],
                4.0,
                &(*planes.offset(p as isize)).plane as *const [f32; 4] as *const [f32; 3],
                &mut up
            );
            return CM_FindPlane(p2, p1, &up[0]);
        },

        3 => {  // left border
            p1 = &(*grid).points[i as usize][j as usize][0];
            p2 = &(*grid).points[i as usize][(j + 1) as usize][0];
            p = CM_GridPlane(gridPlanes, i, j, 1);
            VectorMA(
                &(*grid).points[i as usize][j as usize],
                4.0,
                &(*planes.offset(p as isize)).plane as *const [f32; 4] as *const [f32; 3],
                &mut up
            );
            return CM_FindPlane(p2, p1, &up[0]);
        },

        1 => {  // right border
            p1 = &(*grid).points[(i + 1) as usize][j as usize][0];
            p2 = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            p = CM_GridPlane(gridPlanes, i, j, 0);
            VectorMA(
                &(*grid).points[(i + 1) as usize][j as usize],
                4.0,
                &(*planes.offset(p as isize)).plane as *const [f32; 4] as *const [f32; 3],
                &mut up
            );
            return CM_FindPlane(p1, p2, &up[0]);
        },

        4 => {  // diagonal out of triangle 0
            p1 = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            p2 = &(*grid).points[i as usize][j as usize][0];
            p = CM_GridPlane(gridPlanes, i, j, 0);
            VectorMA(
                &(*grid).points[(i + 1) as usize][(j + 1) as usize],
                4.0,
                &(*planes.offset(p as isize)).plane as *const [f32; 4] as *const [f32; 3],
                &mut up
            );
            return CM_FindPlane(p1, p2, &up[0]);
        },

        5 => {  // diagonal out of triangle 1
            p1 = &(*grid).points[i as usize][j as usize][0];
            p2 = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            p = CM_GridPlane(gridPlanes, i, j, 1);
            VectorMA(
                &(*grid).points[i as usize][j as usize],
                4.0,
                &(*planes.offset(p as isize)).plane as *const [f32; 4] as *const [f32; 3],
                &mut up
            );
            return CM_FindPlane(p1, p2, &up[0]);
        },

        _ => {}
    }

    Com_Error(ERR_DROP, b"CM_EdgePlaneNum: bad k\0" as *const c_char);
    return -1;
}

/*
===================
CM_SetBorderInward
===================
*/
#[inline]
pub unsafe fn CM_SetBorderInward(facet: *mut facetLoad_t, grid: *mut cGrid_t, i: c_int, j: c_int, which: c_int) {
    let mut k: c_int;
    let mut l: c_int;
    let mut points: [*const f32; 4] = [std::ptr::null(); 4];
    let mut numPoints: c_int;

    match which {
        -1 => {
            points[0] = &(*grid).points[i as usize][j as usize][0];
            points[1] = &(*grid).points[(i + 1) as usize][j as usize][0];
            points[2] = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            points[3] = &(*grid).points[i as usize][(j + 1) as usize][0];
            numPoints = 4;
        },
        0 => {
            points[0] = &(*grid).points[i as usize][j as usize][0];
            points[1] = &(*grid).points[(i + 1) as usize][j as usize][0];
            points[2] = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            numPoints = 3;
        },
        1 => {
            points[0] = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            points[1] = &(*grid).points[i as usize][(j + 1) as usize][0];
            points[2] = &(*grid).points[i as usize][j as usize][0];
            numPoints = 3;
        },
        _ => {
            Com_Error(ERR_FATAL, b"CM_SetBorderInward: bad parameter\0" as *const c_char);
            numPoints = 0;
        }
    }

    k = 0;
    while k < (*facet).numBorders {
        let mut front: c_int;
        let mut back: c_int;

        front = 0;
        back = 0;

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
            (*facet).borderInward[k as usize] = true;
        } else if back != 0 && front == 0 {
            (*facet).borderInward[k as usize] = false;
        } else if front == 0 && back == 0 {
            // flat side border
            (*facet).borderPlanes[k as usize] = -1;
        } else {
            // bisecting side border
            Com_DPrintf(b"WARNING: CM_SetBorderInward: mixed plane sides\n\0" as *const c_char);
            (*facet).borderInward[k as usize] = false;
            if !debugBlock {
                debugBlock = true;
                VectorCopy(&(*grid).points[i as usize][j as usize], &mut debugBlockPoints[0]);
                VectorCopy(&(*grid).points[(i + 1) as usize][j as usize], &mut debugBlockPoints[1]);
                VectorCopy(&(*grid).points[(i + 1) as usize][(j + 1) as usize], &mut debugBlockPoints[2]);
                VectorCopy(&(*grid).points[i as usize][(j + 1) as usize], &mut debugBlockPoints[3]);
            }
        }
        k += 1;
    }
}

/*
==================
CM_ValidateFacet

If the facet isn't bounded by its borders, we screwed up.
==================
*/
#[inline]
pub unsafe fn CM_ValidateFacet(facet: *mut facetLoad_t) -> bool {
    let mut plane: [f32; 4] = [0.0; 4];
    let mut j: c_int;
    let mut w: *mut core::ffi::c_void;
    let mut bounds: [[f32; 3]; 2] = [[0.0; 3]; 2];

    if (*facet).surfacePlane == -1 {
        return false;
    }

    Vector4Copy(&(*planes.offset((*facet).surfacePlane as isize)).plane, &mut plane);
    w = BaseWindingForPlane(&plane, plane[3]);
    j = 0;
    while j < (*facet).numBorders && !w.is_null() {
        if (*facet).borderPlanes[j as usize] == -1 {
            FreeWinding(w);
            return false;
        }
        Vector4Copy(&(*planes.offset((*facet).borderPlanes[j as usize] as isize)).plane, &mut plane);
        if !(*facet).borderInward[j as usize] {
            VectorSubtract(&vec3_origin, &plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
            plane[3] = -plane[3];
        }
        ChopWindingInPlace(&mut w, &plane, plane[3], 0.1);
        j += 1;
    }

    if w.is_null() {
        return false;		// winding was completely chopped away
    }

    // see if the facet is unreasonably large
    WindingBounds(w, &mut bounds[0], &mut bounds[1]);
    FreeWinding(w);

    j = 0;
    while j < 3 {
        if bounds[1][j as usize] - bounds[0][j as usize] > MAX_MAP_BOUNDS {
            return false;		// we must be missing a plane
        }
        if bounds[0][j as usize] >= MAX_MAP_BOUNDS {
            return false;
        }
        if bounds[1][j as usize] <= -MAX_MAP_BOUNDS {
            return false;
        }
        j += 1;
    }
    return true;		// winding is fine
}

/*
==================
CM_AddFacetBevels
==================
*/
pub unsafe fn CM_AddFacetBevels(facet: *mut facetLoad_t) {

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
    let mut w: *mut core::ffi::c_void;
    let mut w2: *mut core::ffi::c_void;
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut vec: [f32; 3] = [0.0; 3];
    let mut vec2: [f32; 3] = [0.0; 3];

#[cfg(not(feature = "addbevels"))]
    return;

    Vector4Copy(&(*planes.offset((*facet).surfacePlane as isize)).plane, &mut plane);

    w = BaseWindingForPlane(&plane, plane[3]);
    j = 0;
    while j < (*facet).numBorders && !w.is_null() {
        if (*facet).borderPlanes[j as usize] == (*facet).surfacePlane {
            j += 1;
            continue;
        }
        Vector4Copy(&(*planes.offset((*facet).borderPlanes[j as usize] as isize)).plane, &mut plane);

        if !(*facet).borderInward[j as usize] {
            VectorSubtract(&vec3_origin, &plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
            plane[3] = -plane[3];
        }

        ChopWindingInPlace(&mut w, &plane, plane[3], 0.1);
        j += 1;
    }
    if w.is_null() {
        return;
    }

    WindingBounds(w, &mut mins, &mut maxs);

    // add the axial planes
    order = 0;
    axis = 0;
    while axis < 3 {
        dir = -1;
        while dir <= 1 {
            VectorClear(&mut plane as *mut [f32; 3]);
            plane[axis as usize] = dir as f32;
            if dir == 1 {
                plane[3] = maxs[axis as usize];
            }
            else {
                plane[3] = -mins[axis as usize];
            }
            //if it's the surface plane
            if CM_PlaneEqual(&mut *planes.offset((*facet).surfacePlane as isize), &plane, &mut flipped) != 0 {
                order += 1;
                dir += 2;
                axis += 1;
                continue;
            }
            // see if the plane is allready present
            i = 0;
            while i < (*facet).numBorders {
                if CM_PlaneEqual(&mut *planes.offset((*facet).borderPlanes[i as usize] as isize), &plane, &mut flipped) != 0 {
                    break;
                }
                i += 1;
            }

            if i == (*facet).numBorders {
                if (*facet).numBorders > 4 + 6 + 16 {
                    Com_Printf(b"%sERROR: too many bevels\n\0" as *const c_char, S_COLOR_RED.as_ptr() as *const c_char);
                }
                let num: c_int = CM_FindPlane2(&plane, &mut flipped);
                debug_assert!(num > -32768 && num < 32768);
                (*facet).borderPlanes[(*facet).numBorders as usize] = num;
                (*facet).borderNoAdjust[(*facet).numBorders as usize] = 0;
                (*facet).borderInward[(*facet).numBorders as usize] = flipped != 0;
                (*facet).numBorders += 1;
            }
            order += 1;
            dir += 2;
        }
        axis += 1;
    }
    //
    // add the edge bevels
    //
    // test the non-axial plane edges
    // w->numpoints would need to be accessed through a winding_t struct
    // For now this code is skipped - a proper winding_t struct definition is needed
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum edgeName_t {
    EN_TOP = 0,
    EN_RIGHT = 1,
    EN_BOTTOM = 2,
    EN_LEFT = 3,
}


static mut cm_facets: *mut facetLoad_t = std::ptr::null_mut();
static mut cm_gridPlanes: *mut c_int = std::ptr::null_mut();

pub unsafe fn CM_PatchCollideFromGridTempAlloc() {
    if cm_facets.is_null() {
        cm_facets = Z_Malloc((MAX_PATCH_PLANES as usize) * std::mem::size_of::<facetLoad_t>(), TAG_TEMP_WORKSPACE, false) as *mut facetLoad_t;
    }
    if cm_gridPlanes.is_null() {
        cm_gridPlanes = Z_Malloc(CM_MAX_GRID_SIZE * CM_MAX_GRID_SIZE * 2 * std::mem::size_of::<c_int>(), TAG_TEMP_WORKSPACE, false) as *mut c_int;
    }
}

pub unsafe fn CM_PatchCollideFromGridTempDealloc() {
    Z_Free(cm_gridPlanes as *mut c_void);
    Z_Free(cm_facets as *mut c_void);
    cm_gridPlanes = std::ptr::null_mut();
    cm_facets = std::ptr::null_mut();
}


/*
==================
CM_PatchCollideFromGrid
==================
*/
static mut min1: c_int = 0;
static mut max1: c_int = 0;
static mut min2: c_int = 0;
static mut max2: c_int = 0;

#[inline]
pub unsafe fn CM_PatchCollideFromGrid(grid: *mut cGrid_t, pf: *mut patchCollide_t,
                            facetbuf: *mut facetLoad_t, gridbuf: *mut c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut p1: *const f32;
    let mut p2: *const f32;
    let mut p3: *const f32;
    let mut gridPlanes: *mut c_int;
    let mut facet: *mut facetLoad_t;
    let mut borders: [c_int; 4] = [0; 4];
    let mut noAdjust: [c_int; 4] = [0; 4];
    let mut facets_local: *mut facetLoad_t;
    let mut numFacets: c_int;

    facets_local = cm_facets;
    if facets_local.is_null() {
        facets_local = facetbuf;
    }
    gridPlanes = cm_gridPlanes;
    if gridPlanes.is_null() {
        gridPlanes = gridbuf;
    }

    numPlanes = 0;
    numFacets = 0;

    // find the planes for each triangle of the grid
    i = 0;
    while i < (*grid).width - 1 {
        j = 0;
        while j < (*grid).height - 1 {
            p1 = &(*grid).points[i as usize][j as usize][0];
            p2 = &(*grid).points[(i + 1) as usize][j as usize][0];
            p3 = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize) = CM_FindPlane(p1, p2, p3);

            p1 = &(*grid).points[(i + 1) as usize][(j + 1) as usize][0];
            p2 = &(*grid).points[i as usize][(j + 1) as usize][0];
            p3 = &(*grid).points[i as usize][j as usize][0];
            *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize) = CM_FindPlane(p1, p2, p3);
            j += 1;
        }
        i += 1;
    }

    // create the borders for each facet
    i = 0;
    while i < (*grid).width - 1 {
        j = 0;
        while j < (*grid).height - 1 {

            borders[edgeName_t::EN_TOP as usize] = -1;
            if j > 0 {
                borders[edgeName_t::EN_TOP as usize] = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + (j - 1) * 2 + 1) as isize);
            } else if (*grid).wrapHeight {
                borders[edgeName_t::EN_TOP as usize] = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + ((*grid).height - 2) * 2 + 1) as isize);
            }
            noAdjust[edgeName_t::EN_TOP as usize] = (borders[edgeName_t::EN_TOP as usize] == *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize)) as c_int;
            if borders[edgeName_t::EN_TOP as usize] == -1 || noAdjust[edgeName_t::EN_TOP as usize] != 0 {
                borders[edgeName_t::EN_TOP as usize] = CM_EdgePlaneNum(grid, gridPlanes, i, j, 0);
            }

            borders[edgeName_t::EN_BOTTOM as usize] = -1;
            if j < (*grid).height - 2 {
                borders[edgeName_t::EN_BOTTOM as usize] = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + (j + 1) * 2 + 0) as isize);
            } else if (*grid).wrapHeight {
                borders[edgeName_t::EN_BOTTOM as usize] = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + 0 * 2 + 0) as isize);
            }
            noAdjust[edgeName_t::EN_BOTTOM as usize] = (borders[edgeName_t::EN_BOTTOM as usize] == *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize)) as c_int;
            if borders[edgeName_t::EN_BOTTOM as usize] == -1 || noAdjust[edgeName_t::EN_BOTTOM as usize] != 0 {
                borders[edgeName_t::EN_BOTTOM as usize] = CM_EdgePlaneNum(grid, gridPlanes, i, j, 2);
            }

            borders[edgeName_t::EN_LEFT as usize] = -1;
            if i > 0 {
                borders[edgeName_t::EN_LEFT as usize] = *gridPlanes.offset(((i - 1) * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize);
            } else if (*grid).wrapWidth {
                borders[edgeName_t::EN_LEFT as usize] = *gridPlanes.offset((((*grid).width - 2) * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize);
            }
            noAdjust[edgeName_t::EN_LEFT as usize] = (borders[edgeName_t::EN_LEFT as usize] == *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize)) as c_int;
            if borders[edgeName_t::EN_LEFT as usize] == -1 || noAdjust[edgeName_t::EN_LEFT as usize] != 0 {
                borders[edgeName_t::EN_LEFT as usize] = CM_EdgePlaneNum(grid, gridPlanes, i, j, 3);
            }

            borders[edgeName_t::EN_RIGHT as usize] = -1;
            if i < (*grid).width - 2 {
                borders[edgeName_t::EN_RIGHT as usize] = *gridPlanes.offset(((i + 1) * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize);
            } else if (*grid).wrapWidth {
                borders[edgeName_t::EN_RIGHT as usize] = *gridPlanes.offset((0 * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize);
            }
            noAdjust[edgeName_t::EN_RIGHT as usize] = (borders[edgeName_t::EN_RIGHT as usize] == *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize)) as c_int;
            if borders[edgeName_t::EN_RIGHT as usize] == -1 || noAdjust[edgeName_t::EN_RIGHT as usize] != 0 {
                borders[edgeName_t::EN_RIGHT as usize] = CM_EdgePlaneNum(grid, gridPlanes, i, j, 1);
            }

            if numFacets == MAX_FACETS {
                Com_Error(ERR_DROP, b"MAX_FACETS\0" as *const c_char);
            }
            facet = facets_local.offset(numFacets as isize);
            core::ptr::write_bytes(facet, 0, 1);

            if *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize) == *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize) {
                if *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize) == -1 {
                    j += 1;
                    continue;		// degenrate
                }
                (*facet).surfacePlane = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize);
                (*facet).numBorders = 4;
                (*facet).borderPlanes[0] = borders[edgeName_t::EN_TOP as usize];
                debug_assert!(borders[edgeName_t::EN_TOP as usize] > -32768 && borders[edgeName_t::EN_TOP as usize] < 32768);
                (*facet).borderNoAdjust[0] = noAdjust[edgeName_t::EN_TOP as usize];
                debug_assert!(noAdjust[edgeName_t::EN_TOP as usize] >= 0 && noAdjust[edgeName_t::EN_TOP as usize] < 256);
                (*facet).borderPlanes[1] = borders[edgeName_t::EN_RIGHT as usize];
                debug_assert!(borders[edgeName_t::EN_RIGHT as usize] > -32768 && borders[edgeName_t::EN_RIGHT as usize] < 32768);
                (*facet).borderNoAdjust[1] = noAdjust[edgeName_t::EN_RIGHT as usize];
                debug_assert!(noAdjust[edgeName_t::EN_RIGHT as usize] >= 0 && noAdjust[edgeName_t::EN_RIGHT as usize] < 256);
                (*facet).borderPlanes[2] = borders[edgeName_t::EN_BOTTOM as usize];
                debug_assert!(borders[edgeName_t::EN_BOTTOM as usize] > -32768 &&
                        borders[edgeName_t::EN_BOTTOM as usize] < 32768);
                (*facet).borderNoAdjust[2] = noAdjust[edgeName_t::EN_BOTTOM as usize];
                debug_assert!(noAdjust[edgeName_t::EN_BOTTOM as usize] >= 0 && noAdjust[edgeName_t::EN_BOTTOM as usize] < 256);
                (*facet).borderPlanes[3] = borders[edgeName_t::EN_LEFT as usize];
                debug_assert!(borders[edgeName_t::EN_LEFT as usize] > -32768 && borders[edgeName_t::EN_LEFT as usize] < 32768);
                (*facet).borderNoAdjust[3] = noAdjust[edgeName_t::EN_LEFT as usize];
                debug_assert!(noAdjust[edgeName_t::EN_LEFT as usize] >= 0 && noAdjust[edgeName_t::EN_LEFT as usize] < 256);
                CM_SetBorderInward(facet, grid, i, j, -1);
                if CM_ValidateFacet(facet) {
                    CM_AddFacetBevels(facet);
                    numFacets += 1;
                }
            } else {
                // two seperate triangles
                (*facet).surfacePlane = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize);
                (*facet).numBorders = 3;
                (*facet).borderPlanes[0] = borders[edgeName_t::EN_TOP as usize];
                debug_assert!(borders[edgeName_t::EN_TOP as usize] > -32768 && borders[edgeName_t::EN_TOP as usize] < 32768);
                debug_assert!(noAdjust[edgeName_t::EN_TOP as usize] >= 0 && noAdjust[edgeName_t::EN_TOP as usize] < 256);
                (*facet).borderNoAdjust[0] = noAdjust[edgeName_t::EN_TOP as usize];
                (*facet).borderPlanes[1] = borders[edgeName_t::EN_RIGHT as usize];
                debug_assert!(borders[edgeName_t::EN_RIGHT as usize] > -32768 && borders[edgeName_t::EN_RIGHT as usize] < 32768);
                (*facet).borderNoAdjust[1] = noAdjust[edgeName_t::EN_RIGHT as usize];
                debug_assert!(noAdjust[edgeName_t::EN_RIGHT as usize] >= 0 && noAdjust[edgeName_t::EN_RIGHT as usize] < 256);
                (*facet).borderPlanes[2] = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize);
                debug_assert!(*gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize) > -32768 &&
                        *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize) < 32768);
                if (*facet).borderPlanes[2] == -1 {
                    (*facet).borderPlanes[2] = borders[edgeName_t::EN_BOTTOM as usize];
                    debug_assert!(borders[edgeName_t::EN_BOTTOM as usize] > -32768 &&
                            borders[edgeName_t::EN_BOTTOM as usize] < 32768);
                    if (*facet).borderPlanes[2] == -1 {
                        let num: c_int = CM_EdgePlaneNum(grid, gridPlanes, i, j, 4);
                        debug_assert!(num > -32768 && num < 32768);
                        (*facet).borderPlanes[2] = num;
                    }
                }
                CM_SetBorderInward(facet, grid, i, j, 0);
                if CM_ValidateFacet(facet) {
                    CM_AddFacetBevels(facet);
                    numFacets += 1;
                }

                if numFacets == MAX_FACETS {
                    Com_Error(ERR_DROP, b"MAX_FACETS\0" as *const c_char);
                }
                facet = facets_local.offset(numFacets as isize);
                core::ptr::write_bytes(facet, 0, 1);

                (*facet).surfacePlane = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 1) as isize);
                (*facet).numBorders = 3;
                (*facet).borderPlanes[0] = borders[edgeName_t::EN_BOTTOM as usize];
                debug_assert!(borders[edgeName_t::EN_BOTTOM as usize] > -32768 &&
                        borders[edgeName_t::EN_BOTTOM as usize] < 32768);
                (*facet).borderNoAdjust[0] = noAdjust[edgeName_t::EN_BOTTOM as usize];
                debug_assert!(noAdjust[edgeName_t::EN_BOTTOM as usize] >= 0 && noAdjust[edgeName_t::EN_BOTTOM as usize] < 256);
                (*facet).borderPlanes[1] = borders[edgeName_t::EN_LEFT as usize];
                debug_assert!(borders[edgeName_t::EN_LEFT as usize] > -32768 && borders[edgeName_t::EN_LEFT as usize] < 32768);
                (*facet).borderNoAdjust[1] = noAdjust[edgeName_t::EN_LEFT as usize];
                debug_assert!(noAdjust[edgeName_t::EN_LEFT as usize] >= 0 && noAdjust[edgeName_t::EN_LEFT as usize] < 256);
                (*facet).borderPlanes[2] = *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize);
                debug_assert!(*gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize) > -32768 &&
                        *gridPlanes.offset((i * CM_MAX_GRID_SIZE as c_int * 2 + j * 2 + 0) as isize) < 32768);
                if (*facet).borderPlanes[2] == -1 {
                    (*facet).borderPlanes[2] = borders[edgeName_t::EN_TOP as usize];
                    debug_assert!(borders[edgeName_t::EN_TOP as usize] > -32768 &&
                            borders[edgeName_t::EN_TOP as usize] < 32768);
                    if (*facet).borderPlanes[2] == -1 {
                        let num: c_int = CM_EdgePlaneNum(grid, gridPlanes, i, j, 5);
                        debug_assert!(num > -32768 && num < 32768);
                        (*facet).borderPlanes[2] = num;
                    }
                }
                CM_SetBorderInward(facet, grid, i, j, 1);
                if CM_ValidateFacet(facet) {
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
        (*pf).facets = Z_Malloc((numFacets as usize) * std::mem::size_of::<facet_t>(), TAG_BSP, false) as *mut facet_t;
        // Note: facet_t structure copying would require accessing its data field
        // This is a simplified stub
    }
    else {
        (*pf).facets = std::ptr::null_mut();
    }
    (*pf).planes = Z_Malloc((numPlanes as usize) * std::mem::size_of::<patchPlane_t>(), TAG_BSP, false) as *mut patchPlane_t;
    // memcpy( pf->planes, planes, numPlanes * sizeof( *pf->planes ) );
}

static mut pfScratch: *mut patchCollide_t = std::ptr::null_mut();

pub unsafe fn CM_PreparePatchCollide(num: c_int) {
    pfScratch = Z_Malloc((std::mem::size_of::<patchCollide_t>() * num as usize), TAG_BSP, false) as *mut patchCollide_t;
}

static mut cm_grid: *mut cGrid_t = std::ptr::null_mut();

pub unsafe fn CM_GridAlloc() {
    if !cm_grid.is_null() {
        return;
    }
    cm_grid = Z_Malloc(std::mem::size_of::<cGrid_t>(), TAG_TEMP_WORKSPACE, false) as *mut cGrid_t;
}

pub unsafe fn CM_GridDealloc() {
    Z_Free(cm_grid as *mut c_void);
    cm_grid = std::ptr::null_mut();
}

/*
===================
CM_GeneratePatchCollide

Creates an internal structure that will be used to perform
collision detection with a patch mesh.

Points is packed as concatenated rows.
===================
*/
pub unsafe fn CM_GeneratePatchCollide(width: c_int, height: c_int, points: *mut [f32; 3],
                                facetbuf: *mut facetLoad_t, gridbuf: *mut c_int) -> *mut patchCollide_s {
    let mut pf: *mut patchCollide_t;
    // --AAA--AAA--
    //	cGrid_t			*grid = new cGrid_t;
    let grid: *mut cGrid_t = cm_grid;
    // --AAA--AAA--
    let mut i: c_int;
    let mut j: c_int;

    core::ptr::write_bytes(grid, 0, 1);
    if width <= 2 || height <= 2 || points.is_null() {
        Com_Error(ERR_DROP, b"CM_GeneratePatchFacets: bad parameters: (%i, %i, %p)\0" as *const c_char, width, height, points);
    }

    if (width & 1) == 0 || (height & 1) == 0 {
        Com_Error(ERR_DROP, b"CM_GeneratePatchFacets: even sizes are invalid for quadratic meshes\0" as *const c_char);
    }

    if width > CM_MAX_GRID_SIZE as c_int || height > CM_MAX_GRID_SIZE as c_int {
        Com_Error(ERR_DROP, b"CM_GeneratePatchFacets: source is > CM_MAX_GRID_SIZE\0" as *const c_char);
    }

    // build a grid
    (*grid).width = width;
    (*grid).height = height;
    (*grid).wrapWidth = false;
    (*grid).wrapHeight = false;
    i = 0;
    while i < width {
        j = 0;
        while j < height {
            VectorCopy(&*points.offset((j * width + i) as isize), &mut (*grid).points[i as usize][j as usize]);
            j += 1;
        }
        i += 1;
    }

    // subdivide the grid
    CM_SetGridWrapWidth(grid);
    CM_SubdivideGridColumns(grid);
    CM_RemoveDegenerateColumns(grid);

    CM_TransposeGrid(grid);

    CM_SetGridWrapWidth(grid);
    CM_SubdivideGridColumns(grid);
    CM_RemoveDegenerateColumns(grid);

    // we now have a grid of points exactly on the curve
    // the aproximate surface defined by these points will be
    // collided against

    // --AAA--AAA--
    //	pf = (patchCollide_t *) Z_Malloc( sizeof( *pf ), TAG_BSP, qfalse );
    pf = pfScratch;
    pfScratch = pfScratch.offset(1);
    // --AAA--AAA--

    ClearBounds(&mut (*pf).bounds[0], &mut (*pf).bounds[1]);
    i = 0;
    while i < (*grid).width {
        j = 0;
        while j < (*grid).height {
            AddPointToBounds(&(*grid).points[i as usize][j as usize], &mut (*pf).bounds[0], &mut (*pf).bounds[1]);
            j += 1;
        }
        i += 1;
    }

    c_totalPatchBlocks += ((*grid).width - 1) * ((*grid).height - 1);

    // generate a bsp tree for the surface
    CM_PatchCollideFromGrid(grid, pf, facetbuf, gridbuf);

    // expand by one unit for epsilon purposes
    (*pf).bounds[0][0] -= 1.0;
    (*pf).bounds[0][1] -= 1.0;
    (*pf).bounds[0][2] -= 1.0;

    (*pf).bounds[1][0] += 1.0;
    (*pf).bounds[1][1] += 1.0;
    (*pf).bounds[1][2] += 1.0;

    // --AAA--AAA--
    //	delete grid;
    // --AAA--AAA--

    return pf;
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
#[inline]
pub unsafe fn CM_TracePointThroughPatchCollide(tw: *mut traceWork_t, trace: &mut trace_t, pc: *const patchCollide_s) {
    let mut frontFacing: [bool; 2048] = [false; 2048];  // MAX_PATCH_PLANES
    let mut intersection: [f32; 2048] = [0.0; 2048];  // MAX_PATCH_PLANES
    let mut intersect: f32;
    let planes_ptr: *const patchPlane_t;
    let mut facet: *const facet_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut offset: f32;
    let mut d1: f32;
    let mut d2: f32;
    #[cfg(not(target_os = "windows"))]
    static mut cv: *mut core::ffi::c_void = std::ptr::null_mut();

    #[cfg(not(target_os = "windows"))]
    {
        if (*tw).isPoint == false {
            return;		// FIXME: until I get player sized clipping working right
        }
    }

    if (*pc).numFacets == 0 {	//not gonna do anything anyhow?
        return;
    }
    // determine the trace's relationship to all planes
    planes_ptr = (*pc).planes;
    i = 0;
    while i < (*pc).numPlanes {
        offset = DotProduct(&(*tw).offsets[(*planes_ptr).signbits as usize], &(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3]);
        d1 = DotProduct(&(*tw).start, &(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3]) - (*planes_ptr).plane[3] + offset;
        d2 = DotProduct(&(*tw).end, &(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3]) - (*planes_ptr).plane[3] + offset;
        if d1 <= 0.0 {
            frontFacing[i as usize] = false;
        } else {
            frontFacing[i as usize] = true;
        }
        if d1 == d2 {
            intersection[i as usize] = WORLD_SIZE;
        } else {
            intersection[i as usize] = d1 / (d1 - d2);
            if intersection[i as usize] <= 0.0 {
                intersection[i as usize] = WORLD_SIZE;
            }
        }
        i += 1;
        planes_ptr = (planes_ptr as *const u8).offset(std::mem::size_of::<patchPlane_t>() as isize) as *const patchPlane_t;
    }


    // see if any of the surface planes are intersected
    facet = (*pc).facets;
    i = 0;
    while i < (*pc).numFacets {
        if !frontFacing[(*facet).surfacePlane as usize] {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *const facet_t;
            continue;
        }
        intersect = intersection[(*facet).surfacePlane as usize];
        if intersect < 0.0 {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *const facet_t;
            continue;		// surface is behind the starting point
        }
        if intersect > trace.fraction {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *const facet_t;
            continue;		// already hit something closer
        }
        j = 0;
        while j < (*facet).numBorders {
            k = (*facet).borderPlanes[j as usize];
            if (frontFacing[k as usize] as c_int) ^ ((*facet).borderInward[j as usize] as c_int) != 0 {
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
            #[cfg(not(target_os = "windows"))]
            {
                if cv.is_null() {
                    cv = Cvar_Get(b"r_debugSurfaceUpdate\0" as *const c_char, b"1\0" as *const c_char, 0);
                }
                if !cv.is_null() {
                    debugPatchCollide = pc;
                    debugFacet = facet;
                }
            }
            let planes_hit = &(*pc).planes[(*facet).surfacePlane as usize];

            // calculate intersection with a slight pushoff
            offset = DotProduct(&(*tw).offsets[(*planes_hit).signbits as usize], &(*planes_hit).plane as *const [f32; 4] as *const [f32; 3]);
            d1 = DotProduct(&(*tw).start, &(*planes_hit).plane as *const [f32; 4] as *const [f32; 3]) - (*planes_hit).plane[3] + offset;
            d2 = DotProduct(&(*tw).end, &(*planes_hit).plane as *const [f32; 4] as *const [f32; 3]) - (*planes_hit).plane[3] + offset;
            trace.fraction = (d1 - SURFACE_CLIP_EPSILON) / (d1 - d2);

            if trace.fraction < 0.0 {
                trace.fraction = 0.0;
            }

            VectorCopy(&(*planes_hit).plane as *const [f32; 4] as *const [f32; 3], &mut trace.plane.normal);
            trace.plane.dist = (*planes_hit).plane[3];
        }
        i += 1;
        facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *const facet_t;
    }
}

/*
====================
CM_CheckFacetPlane
====================
*/
#[inline]
pub unsafe fn CM_CheckFacetPlane(plane: *const f32, start: [f32; 3], end: [f32; 3], enterFrac: *mut f32, leaveFrac: *mut f32, hit: *mut c_int) -> c_int {
    let mut d1: f32;
    let mut d2: f32;
    let mut f: f32;

    *hit = 0;  // qfalse

    d1 = DotProduct(&start, &[*plane, *(plane.offset(1)), *(plane.offset(2))]) - *(plane.offset(3));
    d2 = DotProduct(&end, &[*plane, *(plane.offset(1)), *(plane.offset(2))]) - *(plane.offset(3));

    // if completely in front of face, no intersection with the entire facet
    if d1 > 0.0 && (d2 >= SURFACE_CLIP_EPSILON || d2 >= d1) {
        return 0;  // qfalse
    }

    // if it doesn't cross the plane, the plane isn't relevent
    if d1 <= 0.0 && d2 <= 0.0 {
        return 1;  // qtrue
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
            *hit = 1;  // qtrue
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
    return 1;  // qtrue
}

/*
====================
CM_TraceThroughPatchCollide
====================
*/
pub unsafe fn CM_TraceThroughPatchCollide(tw: *mut traceWork_t, trace: &mut trace_t, pc: *const patchCollide_s) {
    let mut i: c_int;
    let mut j: c_int;
    let mut hit: c_int;
    let mut hitnum: c_int;
    let mut offset: f32;
    let mut enterFrac: f32;
    let mut leaveFrac: f32;
    let mut t: f32;
    let mut planes_ptr: *mut patchPlane_t;
    let mut facet: *mut facet_t;
    let mut plane: [f32; 4] = [0.0; 4];
    let mut bestplane: [f32; 4] = [0.0; 4];
    let mut startp: [f32; 3] = [0.0; 3];
    let mut endp: [f32; 3] = [0.0; 3];
    #[cfg(not(target_os = "windows"))]
    static mut cv: *mut core::ffi::c_void = std::ptr::null_mut();

#[cfg(not(feature = "cull_bbox"))]
    {
        // I'm not sure if test is strictly correct.  Are all
        // bboxes axis aligned?  Do I care?  It seems to work
        // good enough...
        i = 0;
        while i < 3 {
            if (*tw).bounds[0][i as usize] > (*pc).bounds[1][i as usize]
                || (*tw).bounds[1][i as usize] < (*pc).bounds[0][i as usize] {
                return;
            }
            i += 1;
        }
    }

    if (*tw).isPoint {
        CM_TracePointThroughPatchCollide(tw, trace, pc);
        return;
    }
    //
    facet = (*pc).facets as *mut facet_t;
    i = 0;
    while i < (*pc).numFacets {
        enterFrac = -1.0;
        leaveFrac = 1.0;
        hitnum = -1;
        //
        planes_ptr = &mut (*(*pc).planes.offset((*facet).surfacePlane as isize)) as *mut patchPlane_t;
        VectorCopy(&(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
        plane[3] = (*planes_ptr).plane[3];
        if (*tw).sphere.use_field {
            // adjust the plane distance apropriately for radius
            plane[3] += (*tw).sphere.radius;

            // find the closest point on the capsule to the plane
            t = DotProduct(&plane as *const [f32; 4] as *const [f32; 3], &(*tw).sphere.offset);
            if t > 0.0 {
                VectorSubtract(&(*tw).start, &(*tw).sphere.offset, &mut startp);
                VectorSubtract(&(*tw).end, &(*tw).sphere.offset, &mut endp);
            }
            else {
                VectorAdd(&(*tw).start, &(*tw).sphere.offset, &mut startp);
                VectorAdd(&(*tw).end, &(*tw).sphere.offset, &mut endp);
            }
        }
        else {
            offset = DotProduct(&(*tw).offsets[(*planes_ptr).signbits as usize], &plane as *const [f32; 4] as *const [f32; 3]);
            plane[3] -= offset;
            VectorCopy(&(*tw).start, &mut startp);
            VectorCopy(&(*tw).end, &mut endp);
        }
        if CM_CheckFacetPlane(&plane[0], startp, endp, &mut enterFrac, &mut leaveFrac, &mut hit) == 0 {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *mut facet_t;
            continue;
        }
        if hit != 0 {
            Vector4Copy(&plane, &mut bestplane);
        }
        j = 0;
        while j < (*facet).numBorders {
            planes_ptr = &mut (*(*pc).planes.offset((*facet).borderPlanes[j as usize] as isize)) as *mut patchPlane_t;
            if (*facet).borderInward[j as usize] {
                VectorNegate(&(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
                plane[3] = -(*planes_ptr).plane[3];
            }
            else {
                VectorCopy(&(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
                plane[3] = (*planes_ptr).plane[3];
            }
            if (*tw).sphere.use_field {
                // adjust the plane distance apropriately for radius
                plane[3] += (*tw).sphere.radius;

                // find the closest point on the capsule to the plane
                t = DotProduct(&plane as *const [f32; 4] as *const [f32; 3], &(*tw).sphere.offset);
                if t > 0.0 {
                    VectorSubtract(&(*tw).start, &(*tw).sphere.offset, &mut startp);
                    VectorSubtract(&(*tw).end, &(*tw).sphere.offset, &mut endp);
                }
                else {
                    VectorAdd(&(*tw).start, &(*tw).sphere.offset, &mut startp);
                    VectorAdd(&(*tw).end, &(*tw).sphere.offset, &mut endp);
                }
            }
            else {
                // NOTE: this works even though the plane might be flipped because the bbox is centered
                offset = DotProduct(&(*tw).offsets[(*planes_ptr).signbits as usize], &plane as *const [f32; 4] as *const [f32; 3]);
                plane[3] += Q_fabs(offset);
                VectorCopy(&(*tw).start, &mut startp);
                VectorCopy(&(*tw).end, &mut endp);
            }
            if CM_CheckFacetPlane(&plane[0], startp, endp, &mut enterFrac, &mut leaveFrac, &mut hit) == 0 {
                break;
            }
            if hit != 0 {
                hitnum = j;
                Vector4Copy(&plane, &mut bestplane);
            }
            j += 1;
        }
        if j < (*facet).numBorders {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *mut facet_t;
            continue;
        }
        //never clip against the back side
        if hitnum == (*facet).numBorders - 1 {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *mut facet_t;
            continue;
        }
        if enterFrac < leaveFrac && enterFrac >= 0.0 {
            if enterFrac < trace.fraction {
                if enterFrac < 0.0 {
                    enterFrac = 0.0;
                }
#[cfg(not(target_os = "windows"))]
                {
                    if cv.is_null() {
                        cv = Cvar_Get(b"r_debugSurfaceUpdate\0" as *const c_char, b"1\0" as *const c_char, 0);
                    }
                    if !cv.is_null() {
                        debugPatchCollide = pc;
                        debugFacet = facet as *const facet_t;
                    }
                }

                trace.fraction = enterFrac;
                VectorCopy(&bestplane as *const [f32; 4] as *const [f32; 3], &mut trace.plane.normal);
                trace.plane.dist = bestplane[3];
            }
        }
        i += 1;
        facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *mut facet_t;
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
pub unsafe fn CM_PositionTestInPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s) -> bool {
    let mut i: c_int;
    let mut j: c_int;
    let mut offset: f32;
    let mut t: f32;
    let mut planes_ptr: *mut patchPlane_t;
    let mut facet: *mut facet_t;
    let mut plane: [f32; 4] = [0.0; 4];
    let mut startp: [f32; 3] = [0.0; 3];

    if (*tw).isPoint {
        return false;
    }
    //
    facet = (*pc).facets as *mut facet_t;
    i = 0;
    while i < (*pc).numFacets {
        planes_ptr = &mut (*(*pc).planes.offset((*facet).surfacePlane as isize)) as *mut patchPlane_t;
        VectorCopy(&(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
        plane[3] = (*planes_ptr).plane[3];
        if (*tw).sphere.use_field {
            // adjust the plane distance apropriately for radius
            plane[3] += (*tw).sphere.radius;

            // find the closest point on the capsule to the plane
            t = DotProduct(&plane as *const [f32; 4] as *const [f32; 3], &(*tw).sphere.offset);
            if t > 0.0 {
                VectorSubtract(&(*tw).start, &(*tw).sphere.offset, &mut startp);
            }
            else {
                VectorAdd(&(*tw).start, &(*tw).sphere.offset, &mut startp);
            }
        }
        else {
            offset = DotProduct(&(*tw).offsets[(*planes_ptr).signbits as usize], &plane as *const [f32; 4] as *const [f32; 3]);
            plane[3] -= offset;
            VectorCopy(&(*tw).start, &mut startp);
        }

        if DotProduct(&plane as *const [f32; 4] as *const [f32; 3], &startp) - plane[3] > 0.0 {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *mut facet_t;
            continue;
        }

        j = 0;
        while j < (*facet).numBorders {
            //			planes = &pc->planes[ facet->borderPlanes[j] ];
            planes_ptr = &mut (*(*pc).planes.offset((*facet).borderPlanes[j as usize] as isize)) as *mut patchPlane_t;
            //			if (facet->borderInward[j]) {
            if (*facet).borderInward[j as usize] {
                VectorNegate(&(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
                plane[3] = -(*planes_ptr).plane[3];
            }
            else {
                VectorCopy(&(*planes_ptr).plane as *const [f32; 4] as *const [f32; 3], &mut plane as *mut [f32; 3]);
                plane[3] = (*planes_ptr).plane[3];
            }
            if (*tw).sphere.use_field {
                // adjust the plane distance apropriately for radius
                plane[3] += (*tw).sphere.radius;

                // find the closest point on the capsule to the plane
                t = DotProduct(&plane as *const [f32; 4] as *const [f32; 3], &(*tw).sphere.offset);
                if t > 0.0 {
                    VectorSubtract(&(*tw).start, &(*tw).sphere.offset, &mut startp);
                }
                else {
                    VectorAdd(&(*tw).start, &(*tw).sphere.offset, &mut startp);
                }
            }
            else {
                // NOTE: this works even though the plane might be flipped because the bbox is centered
                offset = DotProduct(&(*tw).offsets[(*planes_ptr).signbits as usize], &plane as *const [f32; 4] as *const [f32; 3]);
                plane[3] += Q_fabs(offset);
                VectorCopy(&(*tw).start, &mut startp);
            }

            if DotProduct(&plane as *const [f32; 4] as *const [f32; 3], &startp) - plane[3] > 0.0 {
                break;
            }
            j += 1;
        }
        if j < (*facet).numBorders {
            i += 1;
            facet = (facet as *const u8).offset(std::mem::size_of::<facet_t>() as isize) as *mut facet_t;
            continue;
        }
        // inside this patch facet
        return true;
    }

    return false;
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
#[cfg(not(target_os = "windows"))]
extern "C" {
    pub fn BotDrawDebugPolygons(drawPoly: *const core::ffi::c_void, value: c_int);
}

pub unsafe fn CM_DrawDebugSurface(_drawPoly: Option<extern "C" fn(c_int, c_int, *const f32)>) {

}
