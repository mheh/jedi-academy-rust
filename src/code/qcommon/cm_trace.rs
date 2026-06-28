// cm_trace.cpp -- collision model tracing

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// Type definitions from q_shared.h and cm_local.h
// ============================================================================

pub type qboolean = c_int;
pub type byte = u8;
pub type vec_t = f32;
pub type vec3_t = [f32; 3];
pub type vec3pair_t = [vec3_t; 2];
pub type clipHandle_t = c_int;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// keep 1/8 unit away to keep the position valid before network snapping
// and to avoid various numeric issues
const SURFACE_CLIP_EPSILON: f32 = 0.125;

const BOX_MODEL_HANDLE: c_int = 511; // MAX_SUBMODELS - 1, where MAX_SUBMODELS = 512

const MAX_POSITION_LEAFS: usize = 1024;

// ============================================================================
// Structure definitions
// ============================================================================

#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub r#type: byte, // for fast side tests: 0,1,2 = axial, 3 = nonaxial
    pub signbits: byte, // signx + (signy<<1) + (signz<<2), used as lookup during collision
    pub pad: [byte; 2],
}

#[repr(C)]
pub struct cPatch_t {
    pub checkcount: c_int, // to avoid repeated testings
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub pc: *mut patchCollide_s,
}

#[repr(C)]
pub struct patchCollide_s {
    _private: [u8; 0],
}

#[repr(C)]
pub struct cbrushside_t {
    pub plane: *mut cplane_t,
    pub shaderNum: c_int,
}

#[repr(C)]
pub struct cbrush_t {
    pub shaderNum: c_int, // the shader that determined the contents
    pub contents: c_int,
    pub bounds: [vec3_t; 2],
    pub sides: *mut cbrushside_t,
    pub numsides: u16,
    pub checkcount: u16, // to avoid repeated testings
}

#[repr(C)]
pub struct CCMShader {
    pub shader: [c_char; 64], // MAX_QPATH
    pub mNext: *mut CCMShader,
    pub surfaceFlags: c_int,
    pub contentFlags: c_int,
}

#[repr(C)]
pub struct cLeaf_t {
    pub cluster: c_int,
    pub area: c_int,

    pub firstLeafBrush: c_int,
    pub numLeafBrushes: c_int,

    pub firstLeafSurface: c_int,
    pub numLeafSurfaces: c_int,
}

#[repr(C)]
pub struct cNode_t {
    pub plane: *mut cplane_t,
    pub children: [c_int; 2], // negative numbers are leafs
}

#[repr(C)]
pub struct cmodel_t {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub leaf: cLeaf_t, // submodels don't reference the main tree
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: qboolean,    // if true, plane is not valid
    pub startsolid: qboolean,  // if true, the initial point was in a solid area
    pub fraction: f32,         // time completed, 1.0 = didn't hit anything
    pub endpos: vec3_t,        // final position
    pub plane: cplane_t,       // surface normal at impact, transformed to world space
    pub surfaceFlags: c_int,   // surface hit
    pub contents: c_int,       // contents on other side of surface hit
    pub entityNum: c_int,      // entity the contacted surface is a part of
    pub G2CollisionMap: [u8; 16], // placeholder for CCollisionRecord[MAX_G2_COLLISIONS]
}

#[repr(C)]
pub struct sphere_t {
    pub r#use: qboolean,
    pub radius: f32,
    pub halfheight: f32,
    pub offset: vec3_t,
}

#[repr(C)]
pub struct traceWork_t {
    pub start: vec3_t,
    pub end: vec3_t,
    pub size: [vec3_t; 2], // size of the box being swept through the model
    pub offsets: [vec3_t; 8], // [signbits][x] = either size[0][x] or size[1][x]
    pub maxOffset: f32,    // longest corner length from origin
    pub extents: vec3_t,   // greatest of abs(size[0]) and abs(size[1])

    pub bounds: vec3pair_t, // enclosing box of start and end surrounding by size
    pub localBounds: vec3pair_t, // enclosing box of start and end surrounding by size for a segment

    pub modelOrigin: vec3_t, // origin of the model tracing through
    pub contents: c_int,     // ored contents of the model tracing through
    pub isPoint: qboolean,   // optimized case
    pub sphere: sphere_t,    // sphere for oriented capsule collision

    pub baseEnterFrac: f32, // global enter fraction (before processing subsections of the brush)
    pub baseLeaveFrac: f32, // global leave fraction (before processing subsections of the brush)
    pub enterFrac: f32,     // fraction where the ray enters the brush
    pub leaveFrac: f32,     // fraction where the ray leaves the brush
    pub leadside: *mut cbrushside_t,
    pub clipplane: *mut cplane_t,
    pub startout: bool,
    pub getout: bool,

    pub trace: trace_t, // returned from trace call
}

#[repr(C)]
pub struct clipMap_t {
    pub name: [c_char; 64],  // MAX_QPATH

    pub numShaders: c_int,
    pub shaders: *mut CCMShader,

    pub numBrushSides: c_int,
    pub brushsides: *mut cbrushside_t,

    pub numPlanes: c_int,
    pub planes: *mut cplane_t,

    pub numNodes: c_int,
    pub nodes: *mut cNode_t,

    pub numLeafs: c_int,
    pub leafs: *mut cLeaf_t,

    pub numLeafBrushes: c_int,
    pub leafbrushes: *mut c_int,

    pub numLeafSurfaces: c_int,
    pub leafsurfaces: *mut c_int,

    pub numSubModels: c_int,
    pub cmodels: *mut cmodel_t,

    pub numBrushes: c_int,
    pub brushes: *mut cbrush_t,

    pub numClusters: c_int,
    pub clusterBytes: c_int,
    pub visibility: *mut byte,
    pub vised: qboolean, // if false, visibility is just a single cluster of ffs

    pub numEntityChars: c_int,
    pub entityString: *mut c_char,

    pub numAreas: c_int,
    pub areas: *mut u8, // cArea_t placeholder
    pub areaPortals: *mut c_int, // [ numAreas*numAreas ] reference counts

    pub numSurfaces: c_int,
    pub surfaces: *mut *mut cPatch_t, // non-patches will be NULL

    pub floodvalid: c_int,
    pub checkcount: c_int, // incremented on each trace

    pub landScape: *mut c_void, // CCMLandScape
}

#[repr(C)]
pub struct leafList_t {
    pub count: c_int,
    pub maxcount: c_int,
    pub overflowed: qboolean,
    pub list: *mut c_int,
    pub bounds: [vec3_t; 2],
    pub lastLeaf: c_int, // for overflows where each leaf can't be stored individually
    pub storeLeafs: Option<unsafe extern "C" fn(*mut leafList_t, c_int)>,
}

// Forward declaration for cvar_t (defined elsewhere in codebase)
#[repr(C)]
pub struct cvar_t {
    _opaque: [u8; 0],
}

// ============================================================================
// Globals
// ============================================================================

// extern cvar_t *com_terrainPhysics;
pub static mut c_patch_traces: c_int = 0;
pub static mut c_brush_traces: c_int = 0;
pub static mut c_traces: c_int = 0;

// Global collision map instance (stub - needs to be wired in with actual cm_load.rs)
pub static mut cmg: clipMap_t = clipMap_t {
    name: [0; 64],
    numShaders: 0,
    shaders: core::ptr::null_mut(),
    numBrushSides: 0,
    brushsides: core::ptr::null_mut(),
    numPlanes: 0,
    planes: core::ptr::null_mut(),
    numNodes: 0,
    nodes: core::ptr::null_mut(),
    numLeafs: 0,
    leafs: core::ptr::null_mut(),
    numLeafBrushes: 0,
    leafbrushes: core::ptr::null_mut(),
    numLeafSurfaces: 0,
    leafsurfaces: core::ptr::null_mut(),
    numSubModels: 0,
    cmodels: core::ptr::null_mut(),
    numBrushes: 0,
    brushes: core::ptr::null_mut(),
    numClusters: 0,
    clusterBytes: 0,
    visibility: core::ptr::null_mut(),
    vised: qfalse,
    numEntityChars: 0,
    entityString: core::ptr::null_mut(),
    numAreas: 0,
    areas: core::ptr::null_mut(),
    areaPortals: core::ptr::null_mut(),
    numSurfaces: 0,
    surfaces: core::ptr::null_mut(),
    floodvalid: 0,
    checkcount: 0,
    landScape: core::ptr::null_mut(),
};

pub static mut cm_noCurves: *mut cvar_t = core::ptr::null_mut();
pub static mut com_terrainPhysics: *mut cvar_t = core::ptr::null_mut();

// ============================================================================
// External functions
// ============================================================================

extern "C" {
    fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
    fn VectorAdd(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    fn VectorCopy(in_: *const vec3_t, out: *mut vec3_t);
    fn VectorClear(v: *mut vec3_t);
    fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
    fn VectorSet(v: *mut vec3_t, x: f32, y: f32, z: f32);
    fn VectorLength(v: *const vec3_t) -> f32;
    fn VectorNegate(in_: *const vec3_t, out: *mut vec3_t);
    fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);

    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    fn CM_ClipHandleToModel(handle: clipHandle_t, local: *mut *mut clipMap_t) -> *mut cmodel_t;
    fn CM_StoreLeafs(ll: *mut leafList_t, nodenum: c_int);
    fn CM_BoxLeafnums_r(ll: *mut leafList_t, nodenum: c_int);
    fn CM_TraceThroughPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s);
    fn CM_PositionTestInPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s) -> qboolean;

    fn Com_Error(code: c_int, fmt: *const c_char, ...) -> !;
    fn ceilf(x: f32) -> f32;
}

// ============================================================================
// Helper function declarations (stubs that should be linked from elsewhere)
// ============================================================================

fn VectorAdvance(veca: *const vec3_t, scale: f32, vecb: *const vec3_t, vecc: *mut vec3_t) {
    unsafe {
        (*vecc)[0] = (*veca)[0] + scale * (*vecb)[0];
        (*vecc)[1] = (*veca)[1] + scale * (*vecb)[1];
        (*vecc)[2] = (*veca)[2] + scale * (*vecb)[2];
    }
}

// ============================================================================
// Position testing functions
// ============================================================================

/*
================
CM_TestBoxInBrush
================
*/
pub unsafe fn CM_TestBoxInBrush(tw: *mut traceWork_t, brush: *mut cbrush_t) {
    let tw = &mut *tw;
    let brush = &*brush;

    if brush.numsides == 0 {
        return;
    }

    // special test for axial
    if tw.bounds[0][0] > brush.bounds[1][0]
        || tw.bounds[0][1] > brush.bounds[1][1]
        || tw.bounds[0][2] > brush.bounds[1][2]
        || tw.bounds[1][0] < brush.bounds[0][0]
        || tw.bounds[1][1] < brush.bounds[0][1]
        || tw.bounds[1][2] < brush.bounds[0][2]
    {
        return;
    }

    // the first six planes are the axial planes, so we only
    // need to test the remainder
    for i in 6..brush.numsides as usize {
        let side = (*brush.sides.add(i));
        let plane = side.plane;

        // adjust the plane distance apropriately for mins/maxs
        let dist = (*plane).dist - DotProduct(&tw.offsets[(*plane).signbits as usize], &(*plane).normal);

        let d1 = DotProduct(&tw.start, &(*plane).normal) - dist;

        // if completely in front of face, no intersection
        if d1 > 0.0 {
            return;
        }
    }

    // inside this brush
    tw.trace.startsolid = qtrue;
    tw.trace.allsolid = qtrue;
    tw.trace.fraction = 0.0;
    tw.trace.contents = brush.contents;
}

/*
================
CM_PlaneCollision

  Returns false for a quick getout
================
*/
pub unsafe fn CM_PlaneCollision(tw: *mut traceWork_t, side: *mut cbrushside_t) -> bool {
    let tw = &mut *tw;
    let side = &*side;
    let plane = side.plane;

    // adjust the plane distance apropriately for mins/maxs
    let dist = (*plane).dist - DotProduct(&tw.offsets[(*plane).signbits as usize], &(*plane).normal);

    let d1 = DotProduct(&tw.start, &(*plane).normal) - dist;
    let d2 = DotProduct(&tw.end, &(*plane).normal) - dist;

    if d2 > 0.0 {
        // endpoint is not in solid
        tw.getout = true;
    }
    if d1 > 0.0 {
        // startpoint is not in solid
        tw.startout = true;
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
        let mut f = d1 - SURFACE_CLIP_EPSILON;
        if f < 0.0 {
            f = 0.0;
            if f > tw.enterFrac {
                tw.enterFrac = f;
                tw.clipplane = plane as *mut cplane_t;
                tw.leadside = side as *mut cbrushside_t;
            }
        } else if f > tw.enterFrac * (d1 - d2) {
            tw.enterFrac = f / (d1 - d2);
            tw.clipplane = plane as *mut cplane_t;
            tw.leadside = side as *mut cbrushside_t;
        }
    } else {
        // leave
        let mut f = d1 + SURFACE_CLIP_EPSILON;
        if f < (d1 - d2) {
            f = 1.0;
            if f < tw.leaveFrac {
                tw.leaveFrac = f;
            }
        } else if f > tw.leaveFrac * (d1 - d2) {
            tw.leaveFrac = f / (d1 - d2);
        }
    }

    true
}

/*
================
CM_TraceThroughBrush (with trace_t parameter version)
================
*/
pub unsafe fn CM_TraceThroughBrush_withTrace(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    brush: *mut cbrush_t,
    infoOnly: bool,
) {
    let tw = &mut *tw;
    let brush = &*brush;
    let trace = &mut *trace;

    tw.enterFrac = -1.0;
    tw.leaveFrac = 1.0;
    tw.clipplane = core::ptr::null_mut();

    if brush.numsides == 0 {
        return;
    }

    tw.getout = false;
    tw.startout = false;
    tw.leadside = core::ptr::null_mut();

    // compare the trace against all planes of the brush
    // find the latest time the trace crosses a plane towards the interior
    // and the earliest time the trace crosses a plane towards the exterior
    for i in 0..brush.numsides as usize {
        let side = brush.sides.add(i);

        if !CM_PlaneCollision(tw, side) {
            return;
        }
    }

    // all planes have been checked, and the trace was not
    // completely outside the brush
    if !tw.startout {
        if !infoOnly {
            // original point was inside brush
            trace.startsolid = qtrue;
            if !tw.getout {
                trace.allsolid = qtrue;
                trace.fraction = 0.0;
            }
        }
        tw.enterFrac = 0.0;
        return;
    }

    if tw.enterFrac < tw.leaveFrac {
        if (tw.enterFrac > -1.0) && (tw.enterFrac < trace.fraction) {
            if tw.enterFrac < 0.0 {
                tw.enterFrac = 0.0;
            }
            if !infoOnly {
                trace.fraction = tw.enterFrac;
                trace.plane = *(tw.clipplane);
                trace.surfaceFlags = (*cmg.shaders.add((*tw.leadside).shaderNum as usize)).surfaceFlags;
                trace.contents = brush.contents;
            }
        }
    }
}

/*
================
CM_TraceThroughTerrain

Terrain collision support (limited for Xbox compatibility)
================
*/
#[cfg(not(feature = "xbox"))]
pub unsafe fn CM_TraceThroughTerrain(
    tw: *mut traceWork_t,
    trace: *mut trace_t,
    brush: *mut cbrush_t,
) {
    let tw = &mut *tw;
    let trace = &mut *trace;

    // Stub: terrain support not implemented in this minimal port
    // Would require CCMLandScape and related terrain structures
}

/*
================
CM_TestInLeaf
================
*/
pub unsafe fn CM_TestInLeaf(tw: *mut traceWork_t, leaf: *mut cLeaf_t, local: *mut clipMap_t) {
    let tw = &mut *tw;
    let leaf = &*leaf;
    let local = &*local;

    // test box position against all brushes in the leaf
    for k in 0..leaf.numLeafBrushes {
        let brushnum = *local.leafbrushes.add((leaf.firstLeafBrush + k) as usize);
        let b = &mut *local.brushes.add(brushnum as usize);

        if b.checkcount == (local.checkcount as u16) {
            continue; // already checked this brush in another leaf
        }
        b.checkcount = local.checkcount as u16;

        if (b.contents & tw.contents) == 0 {
            continue;
        }

        CM_TestBoxInBrush(tw, b);
        if tw.trace.allsolid != 0 {
            return;
        }
    }

    // test against all patches
    #[cfg(not(feature = "xbox"))]
    {
        if cm_noCurves.is_null() || (*cm_noCurves).integer == 0 {
            for k in 0..leaf.numLeafSurfaces {
                let patch = *local.surfaces.add(*local.leafsurfaces.add((leaf.firstLeafSurface + k) as usize) as usize);
                if patch.is_null() {
                    continue;
                }

                let patch = &mut *patch;
                if patch.checkcount == local.checkcount {
                    continue; // already checked this patch in another leaf
                }
                patch.checkcount = local.checkcount;

                if (patch.contents & tw.contents) == 0 {
                    continue;
                }

                if CM_PositionTestInPatchCollide(tw, patch.pc) != 0 {
                    tw.trace.startsolid = qtrue;
                    tw.trace.allsolid = qtrue;
                    tw.trace.fraction = 0.0;
                    tw.trace.contents = patch.contents;
                    return;
                }
            }
        }
    }
}

/*
==================
CM_PositionTest
==================
*/
pub unsafe fn CM_PositionTest(tw: *mut traceWork_t) {
    let tw = &mut *tw;
    let mut leafs: [c_int; MAX_POSITION_LEAFS] = [0; MAX_POSITION_LEAFS];
    let mut ll: leafList_t = core::mem::zeroed();

    // identify the leafs we are touching
    VectorAdd(&tw.start, &tw.size[0], &mut ll.bounds[0]);
    VectorAdd(&tw.start, &tw.size[1], &mut ll.bounds[1]);

    for i in 0..3 {
        ll.bounds[0][i] -= 1.0;
        ll.bounds[1][i] += 1.0;
    }

    ll.count = 0;
    ll.maxcount = MAX_POSITION_LEAFS as c_int;
    ll.list = leafs.as_mut_ptr();
    ll.storeLeafs = Some(CM_StoreLeafs);
    ll.lastLeaf = 0;
    ll.overflowed = qfalse;

    cmg.checkcount += 1;

    CM_BoxLeafnums_r(&mut ll, 0);

    cmg.checkcount += 1;

    // test the contents of the leafs
    for i in 0..ll.count {
        CM_TestInLeaf(tw, &mut *cmg.leafs.add(*ll.list.add(i as usize) as usize), &mut cmg);
        if tw.trace.allsolid != 0 {
            break;
        }
    }
}

// ============================================================================
// Box tracing functions
// ============================================================================

/*
================
CM_TraceThroughPatch
================
*/
pub unsafe fn CM_TraceThroughPatch(tw: *mut traceWork_t, patch: *mut cPatch_t) {
    let tw = &mut *tw;
    let patch = &*patch;

    c_patch_traces += 1;

    let oldFrac = tw.trace.fraction;

    CM_TraceThroughPatchCollide(tw, patch.pc);

    if tw.trace.fraction < oldFrac {
        tw.trace.surfaceFlags = patch.surfaceFlags;
        tw.trace.contents = patch.contents;
    }
}

/*
================
CM_TraceThroughBrush (main tracing version - no trace_t parameter)
================
*/
pub unsafe fn CM_TraceThroughBrush(tw: *mut traceWork_t, brush: *mut cbrush_t) {
    let tw = &mut *tw;
    let brush = &*brush;

    let mut enterFrac: f32 = -1.0;
    let mut leaveFrac: f32 = 1.0;
    let mut clipplane: *mut cplane_t = core::ptr::null_mut();
    let mut leadside: *mut cbrushside_t = core::ptr::null_mut();

    if brush.numsides == 0 {
        return;
    }

    // I'm not sure if test is strictly correct.  Are all
    // bboxes axis aligned?  Do I care?  It seems to work
    // good enough...
    if tw.bounds[0][0] > brush.bounds[1][0]
        || tw.bounds[0][1] > brush.bounds[1][1]
        || tw.bounds[0][2] > brush.bounds[1][2]
        || tw.bounds[1][0] < brush.bounds[0][0]
        || tw.bounds[1][1] < brush.bounds[0][1]
        || tw.bounds[1][2] < brush.bounds[0][2]
    {
        return;
    }

    c_brush_traces += 1;

    let mut getout: bool = false;
    let mut startout: bool = false;

    // compare the trace against all planes of the brush
    // find the latest time the trace crosses a plane towards the interior
    // and the earliest time the trace crosses a plane towards the exterior
    for i in 0..brush.numsides as usize {
        let side = brush.sides.add(i);
        let plane = (*side).plane;

        // adjust the plane distance apropriately for mins/maxs
        let dist =
            (*plane).dist - DotProduct(&tw.offsets[(*plane).signbits as usize], &(*plane).normal);

        let d1 = DotProduct(&tw.start, &(*plane).normal) - dist;
        let d2 = DotProduct(&tw.end, &(*plane).normal) - dist;

        if d2 > 0.0 {
            getout = true; // endpoint is not in solid
        }
        if d1 > 0.0 {
            startout = true;
        }

        // if completely in front of face, no intersection with the entire brush
        if d1 > 0.0 && ((d2 >= SURFACE_CLIP_EPSILON) || (d2 >= d1)) {
            return;
        }

        // if it doesn't cross the plane, the plane isn't relevent
        if d1 <= 0.0 && d2 <= 0.0 {
            continue;
        }

        // crosses face
        if d1 > d2 {
            // enter
            let mut f = (d1 - SURFACE_CLIP_EPSILON) / (d1 - d2);
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
            let mut f = (d1 + SURFACE_CLIP_EPSILON) / (d1 - d2);
            if f > 1.0 {
                f = 1.0;
            }
            if f < leaveFrac {
                leaveFrac = f;
            }
        }
    }

    // all planes have been checked, and the trace was not
    // completely outside the brush
    if !startout {
        // original point was inside brush
        tw.trace.startsolid = qtrue;
        tw.trace.contents |= brush.contents; // note, we always want to know the contents of something we're inside of
        if !getout {
            // endpoint was inside brush
            tw.trace.allsolid = qtrue;
            tw.trace.fraction = 0.0;
        }
        return;
    }

    if enterFrac < leaveFrac {
        if enterFrac > -1.0 && enterFrac < tw.trace.fraction {
            if enterFrac < 0.0 {
                enterFrac = 0.0;
            }
            tw.trace.fraction = enterFrac;
            tw.trace.plane = *clipplane;
            tw.trace.surfaceFlags = (*cmg.shaders.add((*leadside).shaderNum as usize)).surfaceFlags;
            tw.trace.contents = brush.contents;
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
    patch: *mut c_void,
    checkcount: c_int,
) {
    // Stub: would need CCMPatch and related patch collision structures
}

/*
================
CM_GenericBoxCollide
================
*/
pub unsafe fn CM_GenericBoxCollide(abounds: *const vec3pair_t, bbounds: *const vec3pair_t) -> bool {
    let abounds = &*abounds;
    let bbounds = &*bbounds;

    // Check for completely no intersection
    for i in 0..3 {
        if abounds[1][i] < bbounds[0][i] {
            return false;
        }
        if abounds[0][i] > bbounds[1][i] {
            return false;
        }
    }
    true
}

/*
================
CM_TraceToLeaf
================
*/
pub unsafe fn CM_TraceToLeaf(tw: *mut traceWork_t, leaf: *mut cLeaf_t, local: *mut clipMap_t) {
    let tw = &mut *tw;
    let leaf = &*leaf;
    let local = &*local;

    // trace line against all brushes in the leaf
    for k in 0..leaf.numLeafBrushes {
        let brushnum = *local.leafbrushes.add((leaf.firstLeafBrush + k) as usize);

        let b = &mut *local.brushes.add(brushnum as usize);
        if b.checkcount == (local.checkcount as u16) {
            continue; // already checked this brush in another leaf
        }
        b.checkcount = local.checkcount as u16;

        if (b.contents & tw.contents) == 0 {
            continue;
        }

        CM_TraceThroughBrush(tw, b);
        if tw.trace.fraction == 0.0 {
            return;
        }
    }

    // trace line against all patches in the leaf
    if cm_noCurves.is_null() || (*cm_noCurves).integer == 0 {
        for k in 0..leaf.numLeafSurfaces {
            let patch = *local.surfaces.add(*local.leafsurfaces.add((leaf.firstLeafSurface + k) as usize) as usize);
            if patch.is_null() {
                continue;
            }

            let patch = &mut *patch;
            if patch.checkcount == local.checkcount {
                continue; // already checked this patch in another leaf
            }
            patch.checkcount = local.checkcount;

            if (patch.contents & tw.contents) == 0 {
                continue;
            }

            CM_TraceThroughPatch(tw, patch);
            if tw.trace.fraction == 0.0 {
                return;
            }
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
    let tw = &mut *tw;
    let local = &*local;
    let p1_ref = &*p1;
    let p2_ref = &*p2;

    if tw.trace.fraction <= p1f {
        return; // already hit something nearer
    }

    // if < 0, we are in a leaf node
    if num < 0 {
        CM_TraceToLeaf(tw, &mut *local.leafs.add((-1 - num) as usize), &mut *local);
        return;
    }

    // find the point distances to the seperating plane
    // and the offset for the size of the box
    let node = &*local.nodes.add(num as usize);
    let plane = node.plane;

    // adjust the plane distance apropriately for mins/maxs
    let (t1, t2, offset) = if (*plane).r#type < 3 {
        let t1 = p1_ref[(*plane).r#type as usize] - (*plane).dist;
        let t2 = p2_ref[(*plane).r#type as usize] - (*plane).dist;
        let offset = tw.extents[(*plane).r#type as usize];
        (t1, t2, offset)
    } else {
        let t1 = DotProduct(p1, &(*plane).normal) - (*plane).dist;
        let t2 = DotProduct(p2, &(*plane).normal) - (*plane).dist;
        let offset = if tw.isPoint != 0 {
            0.0
        } else {
            // an axial brush right behind a slanted bsp plane
            // will poke through when expanded, so adjust by sqrt(3)
            let mut off = (tw.extents[0] * (*plane).normal[0]).abs()
                + (tw.extents[1] * (*plane).normal[1]).abs()
                + (tw.extents[2] * (*plane).normal[2]).abs();

            off *= 2.0;

            tw.maxOffset
        };
        (t1, t2, offset)
    };

    // see which sides we need to consider
    if t1 >= offset + 1.0 && t2 >= offset + 1.0 {
        CM_TraceThroughTree(tw, local, node.children[0], p1f, p2f, p1, p2);
        return;
    }
    if t1 < -offset - 1.0 && t2 < -offset - 1.0 {
        CM_TraceThroughTree(tw, local, node.children[1], p1f, p2f, p1, p2);
        return;
    }

    // put the crosspoint SURFACE_CLIP_EPSILON pixels on the near side
    let (side, frac, frac2) = if t1 < t2 {
        let idist = 1.0 / (t1 - t2);
        (1, (t1 - offset + SURFACE_CLIP_EPSILON) * idist, (t1 + offset + SURFACE_CLIP_EPSILON) * idist)
    } else if t1 > t2 {
        let idist = 1.0 / (t1 - t2);
        (0, (t1 + offset + SURFACE_CLIP_EPSILON) * idist, (t1 - offset - SURFACE_CLIP_EPSILON) * idist)
    } else {
        (0, 1.0, 0.0)
    };

    // move up to the node
    let mut frac = frac;
    if frac < 0.0 {
        frac = 0.0;
    }
    if frac > 1.0 {
        frac = 1.0;
    }

    let midf = p1f + (p2f - p1f) * frac;

    let mut mid: vec3_t = [
        p1_ref[0] + frac * (p2_ref[0] - p1_ref[0]),
        p1_ref[1] + frac * (p2_ref[1] - p1_ref[1]),
        p1_ref[2] + frac * (p2_ref[2] - p1_ref[2]),
    ];

    CM_TraceThroughTree(tw, local, node.children[side], p1f, midf, p1, &mut mid);

    // go past the node
    let mut frac2 = frac2;
    if frac2 < 0.0 {
        frac2 = 0.0;
    }
    if frac2 > 1.0 {
        frac2 = 1.0;
    }

    let midf = p1f + (p2f - p1f) * frac2;

    mid = [
        p1_ref[0] + frac2 * (p2_ref[0] - p1_ref[0]),
        p1_ref[1] + frac2 * (p2_ref[1] - p1_ref[1]),
        p1_ref[2] + frac2 * (p2_ref[2] - p1_ref[2]),
    ];

    CM_TraceThroughTree(tw, local, node.children[side ^ 1], midf, p2f, &mut mid, p2);
}

pub unsafe fn CM_CalcExtents(
    start: *const vec3_t,
    end: *const vec3_t,
    tw: *const traceWork_t,
    bounds: *mut vec3pair_t,
) {
    let start = &*start;
    let end = &*end;
    let tw = &*tw;
    let bounds = &mut *bounds;

    for i in 0..3 {
        if start[i] < end[i] {
            bounds[0][i] = start[i] + tw.size[0][i];
            bounds[1][i] = end[i] + tw.size[1][i];
        } else {
            bounds[0][i] = end[i] + tw.size[0][i];
            bounds[1][i] = start[i] + tw.size[1][i];
        }
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
    let results = &mut *results;
    let start = &*start;
    let end = &*end;

    let mut mins = mins;
    let mut maxs = maxs;

    let mut local: *mut clipMap_t = core::ptr::null_mut();
    let cmod = CM_ClipHandleToModel(model, &mut local);
    let local = &mut *local;

    local.checkcount += 1; // for multi-check avoidance

    c_traces += 1; // for statistics, may be zeroed

    // fill in a default trace
    let mut tw: traceWork_t = core::mem::zeroed();
    tw.trace.fraction = 1.0; // assume it goes the entire distance until shown otherwise

    if local.numNodes == 0 {
        *results = tw.trace;
        return; // map not loaded, shouldn't happen
    }

    // allow NULL to be passed in for 0,0,0
    if mins.is_null() {
        mins = &[0.0, 0.0, 0.0];
    }
    if maxs.is_null() {
        maxs = &[0.0, 0.0, 0.0];
    }

    // set basic parms
    tw.contents = brushmask;

    // adjust so that mins and maxs are always symetric, which
    // avoids some complications with plane expanding of rotated bmodels
    let mut offset: vec3_t = [0.0; 3];
    for i in 0..3 {
        offset[i] = (mins[i] + maxs[i]) * 0.5;
        tw.size[0][i] = mins[i] - offset[i];
        tw.size[1][i] = maxs[i] - offset[i];
        tw.start[i] = start[i] + offset[i];
        tw.end[i] = end[i] + offset[i];
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

    // calculate bounds
    for i in 0..3 {
        if tw.start[i] < tw.end[i] {
            tw.bounds[0][i] = tw.start[i] + tw.size[0][i];
            tw.bounds[1][i] = tw.end[i] + tw.size[1][i];
        } else {
            tw.bounds[0][i] = tw.end[i] + tw.size[0][i];
            tw.bounds[1][i] = tw.start[i] + tw.size[1][i];
        }
    }

    // check for position test special case
    if start[0] == end[0] && start[1] == end[1] && start[2] == end[2] {
        if model != 0 {
            CM_TestInLeaf(&mut tw, &mut (*cmod).leaf, local);
        } else {
            CM_PositionTest(&mut tw);
        }
    } else {
        // check for point special case
        if tw.size[0][0] == 0.0 && tw.size[0][1] == 0.0 && tw.size[0][2] == 0.0 {
            tw.isPoint = qtrue;
            VectorClear(&mut tw.extents);
        } else {
            tw.isPoint = qfalse;
            tw.extents[0] = tw.size[1][0];
            tw.extents[1] = tw.size[1][1];
            tw.extents[2] = tw.size[1][2];
        }

        // general sweeping through world
        if model != 0 {
            CM_TraceToLeaf(&mut tw, &mut (*cmod).leaf, local);
        } else {
            CM_TraceThroughTree(&mut tw, local, 0, 0.0, 1.0, &mut tw.start, &mut tw.end);
        }
    }

    // generate endpos from the original, unmodified start/end
    if tw.trace.fraction == 1.0 {
        VectorCopy(end, &mut tw.trace.endpos);
    } else {
        for i in 0..3 {
            tw.trace.endpos[i] = start[i] + tw.trace.fraction * (end[i] - start[i]);
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
    let results = &mut *results;
    let start = &*start;
    let end = &*end;
    let origin = &*origin;
    let angles = &*angles;

    let mut mins = mins;
    let mut maxs = maxs;

    let mut trace: trace_t = core::mem::zeroed();
    let mut start_l: vec3_t = [0.0; 3];
    let mut end_l: vec3_t = [0.0; 3];
    let mut a: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut temp: vec3_t = [0.0; 3];
    let mut offset: vec3_t = [0.0; 3];
    let mut symetricSize: [vec3_t; 2] = [[0.0; 3]; 2];

    if mins.is_null() {
        mins = &[0.0, 0.0, 0.0];
    }
    if maxs.is_null() {
        maxs = &[0.0, 0.0, 0.0];
    }

    // adjust so that mins and maxs are always symetric, which
    // avoids some complications with plane expanding of rotated bmodels
    for i in 0..3 {
        offset[i] = (mins[i] + maxs[i]) * 0.5;
        symetricSize[0][i] = mins[i] - offset[i];
        symetricSize[1][i] = maxs[i] - offset[i];
        start_l[i] = start[i] + offset[i];
        end_l[i] = end[i] + offset[i];
    }

    // subtract origin offset
    VectorSubtract(&start_l, origin, &mut start_l);
    VectorSubtract(&end_l, origin, &mut end_l);

    // rotate start and end into the models frame of reference
    let rotated = if model != BOX_MODEL_HANDLE && (angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0) {
        true
    } else {
        false
    };

    if rotated {
        AngleVectors(angles, &mut forward, &mut right, &mut up);

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
    CM_BoxTrace(&mut trace, &start_l, &end_l, &symetricSize[0], &symetricSize[1], model, brushmask);

    if rotated && trace.fraction != 1.0 {
        // FIXME: figure out how to do this with existing angles
        VectorNegate(angles, &mut a);
        AngleVectors(&a, &mut forward, &mut right, &mut up);

        VectorCopy(&trace.plane.normal, &mut temp);
        trace.plane.normal[0] = DotProduct(&temp, &forward);
        trace.plane.normal[1] = -DotProduct(&temp, &right);
        trace.plane.normal[2] = DotProduct(&temp, &up);
    }

    trace.endpos[0] = start[0] + trace.fraction * (end[0] - start[0]);
    trace.endpos[1] = start[1] + trace.fraction * (end[1] - start[1]);
    trace.endpos[2] = start[2] + trace.fraction * (end[2] - start[2]);

    *results = trace;
}

/*
=================
CM_CullBox

Returns true if culled out
=================
*/
pub unsafe fn CM_CullBox(frustum: *const cplane_t, transformed: *const [vec3_t; 8]) -> bool {
    let frustum_slice = core::slice::from_raw_parts(frustum, 4);
    let transformed = &*transformed;

    // check against frustum planes
    for i in 0..4 {
        let mut j = 0;
        while j < 8 {
            if DotProduct(&transformed[j], &frustum_slice[i].normal) > frustum_slice[i].dist {
                // a point is in front
                break;
            }
            j += 1;
        }

        if j == 8 {
            // all points were behind one of the planes
            return true;
        }
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
    let bounds = &*bounds;
    let mut transformed: [vec3_t; 8] = [[0.0; 3]; 8];

    for i in 0..8 {
        transformed[i][0] = bounds[(i & 1) as usize][0];
        transformed[i][1] = bounds[((i >> 1) & 1) as usize][1];
        transformed[i][2] = bounds[((i >> 2) & 1) as usize][2];
    }

    // rwwFIXMEFIXME: Was not ! before. But that seems the way it should be and it works that way. Why?
    !CM_CullBox(frustum, &transformed)
}
