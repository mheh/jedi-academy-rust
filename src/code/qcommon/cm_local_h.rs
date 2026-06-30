// Ported from oracle/code/qcommon/cm_local.h

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::ffi::{c_char, c_int};

use crate::code::game::q_shared_h::*;
use crate::code::qcommon::qcommon_h::*;
use crate::code::qcommon::cm_polylib_h::*;
use crate::code::qcommon::cm_landscape_h::*;
// patchCollide_s is used here but not defined in this header;
// imported per triage caution — defined in cm_patch_h
use crate::code::qcommon::cm_patch_h::*;

// #include "sparc.h" — xbox only
#[cfg(feature = "xbox")]
use crate::code::qcommon::sparc_h::*;

pub const BOX_MODEL_HANDLE: c_int = (MAX_SUBMODELS - 1) as c_int;

// -----------------------------------------------------------------------
// cNode_t
// -----------------------------------------------------------------------

#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct cNode_t {
    pub children: [i16; 2],     // negative numbers are leafs
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct cNode_t {
    pub plane: *mut cplane_t,
    pub children: [c_int; 2],   // negative numbers are leafs
}

// -----------------------------------------------------------------------
// cLeaf_t
// -----------------------------------------------------------------------

#[repr(C)]
pub struct cLeaf_t {
    pub cluster: c_int,
    pub area: c_int,

    pub firstLeafBrush: c_int,
    pub numLeafBrushes: c_int,

    pub firstLeafSurface: c_int,
    pub numLeafSurfaces: c_int,
}

// -----------------------------------------------------------------------
// cmodel_t (cmodel_s)
// -----------------------------------------------------------------------

#[repr(C)]
pub struct cmodel_t {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub leaf: cLeaf_t,  // submodels don't reference the main tree
}

// -----------------------------------------------------------------------
// cbrushside_t (cbrushside_s)
// -----------------------------------------------------------------------

#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct cbrushside_t {
    pub planeNum: NotSoShort,
    pub shaderNum: u8,
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct cbrushside_t {
    pub plane: *mut cplane_t,
    pub shaderNum: c_int,
}

// -----------------------------------------------------------------------
// cbrush_t (cbrush_s)
// -----------------------------------------------------------------------

#[repr(C)]
pub struct cbrush_t {
    pub shaderNum: c_int,           // the shader that determined the contents
    pub contents: c_int,
    pub bounds: [vec3_t; 2],
    pub sides: *mut cbrushside_t,
    pub numsides: u16,
    pub checkcount: u16,            // to avoid repeated testings
}

// -----------------------------------------------------------------------
// CCMShader
// -----------------------------------------------------------------------

#[repr(C)]
pub struct CCMShader {
    pub shader: [c_char; MAX_QPATH as usize],
    pub mNext: *mut CCMShader,
    pub surfaceFlags: c_int,
    pub contentFlags: c_int,
}

impl CCMShader {
    pub fn GetName(&self) -> *const c_char {
        &self.shader[0] as *const c_char
    }
    pub fn GetNext(&self) -> *mut CCMShader {
        self.mNext
    }
    pub fn SetNext(&mut self, next: *mut CCMShader) {
        self.mNext = next;
    }
    pub fn Destroy(&mut self) {}
}

// -----------------------------------------------------------------------
// cPatch_t
// -----------------------------------------------------------------------

#[repr(C)]
pub struct cPatch_t {
    pub checkcount: c_int,          // to avoid repeated testings
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub pc: *mut patchCollide_s,
}

// -----------------------------------------------------------------------
// cArea_t
// -----------------------------------------------------------------------

#[repr(C)]
pub struct cArea_t {
    pub floodnum: c_int,
    pub floodvalid: c_int,
}

// -----------------------------------------------------------------------
// clipMap_t — xbox variant
// SPARC<byte>: C++ template SPARC<byte> from sparc.h; imported via sparc_h glob
// -----------------------------------------------------------------------

#[cfg(feature = "xbox")]
#[repr(C)]
pub struct clipMap_t {
    pub name: [c_char; MAX_QPATH as usize],

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
    pub visibility: *mut SPARC<byte>,
    pub vised: qboolean,            // if false, visibility is just a single cluster of ffs

    pub numEntityChars: c_int,
    pub entityString: *mut c_char,

    pub numAreas: c_int,
    pub areas: *mut cArea_t,
    pub areaPortals: *mut c_int,    // [ numAreas*numAreas ] reference counts

    pub numSurfaces: c_int,
    pub surfaces: *mut *mut cPatch_t, // non-patches will be NULL

    pub floodvalid: c_int,
    pub checkcount: c_int,          // incremented on each trace

//	CCMLandScape	*landScape;		// Removing terrain from Xbox
}

// -----------------------------------------------------------------------
// clipMap_t — non-xbox variant
// -----------------------------------------------------------------------

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct clipMap_t {
    pub name: [c_char; MAX_QPATH as usize],

    pub numShaders: c_int,
    //dshader_t	*shaders;
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
    pub vised: qboolean,            // if false, visibility is just a single cluster of ffs

    pub numEntityChars: c_int,
    pub entityString: *mut c_char,

    pub numAreas: c_int,
    pub areas: *mut cArea_t,
    pub areaPortals: *mut c_int,    // [ numAreas*numAreas ] reference counts

    pub numSurfaces: c_int,
    pub surfaces: *mut *mut cPatch_t, // non-patches will be NULL

    pub floodvalid: c_int,
    pub checkcount: c_int,          // incremented on each trace

    pub landScape: *mut CCMLandScape,
}


// keep 1/8 unit away to keep the position valid before network snapping
// and to avoid various numeric issues
pub const SURFACE_CLIP_EPSILON: f32 = 0.125;

extern "C" {
    pub static mut cmg: clipMap_t;
    pub static mut c_pointcontents: c_int;
    pub static mut c_traces: c_int;
    pub static mut c_brush_traces: c_int;
    pub static mut c_patch_traces: c_int;
    pub static mut cm_noAreas: *mut cvar_t;
    pub static mut cm_noCurves: *mut cvar_t;
    pub static mut cm_playerCurveClip: *mut cvar_t;

    pub static mut SubBSP: [clipMap_t; MAX_SUB_BSP as usize];
    pub static mut NumSubBSP: c_int;
}

// cm_test.c

// Used for oriented capsule collision detection
#[repr(C)]
pub struct sphere_t {
    // C field named `use`; renamed to use_ — `use` is a Rust reserved keyword
    pub use_: qboolean,
    pub radius: f32,
    pub halfheight: f32,
    pub offset: vec3_t,
}

#[repr(C)]
pub struct traceWork_t {
    pub start: vec3_t,
    pub end: vec3_t,
    pub size: [vec3_t; 2],          // size of the box being swept through the model
    pub offsets: [vec3_t; 8],       // [signbits][x] = either size[0][x] or size[1][x]
    pub maxOffset: f32,              // longest corner length from origin
    pub extents: vec3_t,             // greatest of abs(size[0]) and abs(size[1])

    pub bounds: [vec3_t; 2],         // enclosing box of start and end surrounding by size
    pub localBounds: vec3pair_t,     // enclosing box of start and end surrounding by size for a segment

    pub modelOrigin: vec3_t,         // origin of the model tracing through
    pub contents: c_int,             // ored contents of the model tracing through
    pub isPoint: qboolean,           // optimized case
    pub sphere: sphere_t,            // sphere for oriendted capsule collision

    pub baseEnterFrac: f32,          // global enter fraction (before processing subsections of the brush)
    pub baseLeaveFrac: f32,          // global leave fraction (before processing subsections of the brush)
    pub enterFrac: f32,              // fraction where the ray enters the brush
    pub leaveFrac: f32,              // fraction where the ray leaves the brush
    pub leadside: *mut cbrushside_t,
    pub clipplane: *mut cplane_t,
    pub startout: bool,
    pub getout: bool,

    pub trace: trace_t,              // returned from trace call
    // make sure nothing goes under here for Ghoul2 collision purposes
}

#[repr(C)]
pub struct leafList_t {
    pub count: c_int,
    pub maxcount: c_int,
    pub overflowed: qboolean,
    pub list: *mut c_int,
    pub bounds: [vec3_t; 2],
    pub lastLeaf: c_int,    // for overflows where each leaf can't be stored individually
    pub storeLeafs: Option<unsafe extern "C" fn(*mut leafList_t, c_int)>,
}


extern "C" {
    pub fn CM_BoxBrushes(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        boxlist: *mut *mut cbrush_t,
        listsize: c_int,
    ) -> c_int;

    pub fn CM_StoreLeafs(ll: *mut leafList_t, nodenum: c_int);
    pub fn CM_StoreBrushes(ll: *mut leafList_t, nodenum: c_int);

    pub fn CM_BoxLeafnums_r(ll: *mut leafList_t, nodenum: c_int);

    pub fn CM_ClipHandleToModel(
        handle: clipHandle_t,
        clipMap: *mut *mut clipMap_t,
    ) -> *mut cmodel_t;
    pub fn CM_CleanLeafCache();

    // cm_load.c
    // C++ ref `clipMap_t &cm` translated as *mut clipMap_t; vec3_t params decay to pointer
    pub fn CM_ModelBounds(
        cm: *mut clipMap_t,
        model: clipHandle_t,
        mins: *mut vec3_t,
        maxs: *mut vec3_t,
    );

    // cm_patch.c
    pub fn CM_GeneratePatchCollide(
        width: c_int,
        height: c_int,
        points: *mut vec3_t,
    ) -> *mut patchCollide_s;
    pub fn CM_TraceThroughPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s);
    pub fn CM_PositionTestInPatchCollide(
        tw: *mut traceWork_t,
        pc: *const patchCollide_s,
    ) -> qboolean;
    pub fn CM_ClearLevelPatches();

    // cm_shader.cpp
    pub fn CM_SetupShaderProperties();
    pub fn CM_ShutdownShaderProperties();
    pub fn CM_GetShaderInfo(name: *const c_char) -> *mut CCMShader;
    // porting deviation: C++ overload CM_GetShaderInfo(int shaderNum) renamed CM_GetShaderInfo_int
    // to avoid duplicate extern "C" symbol; C++ ABI mangles these differently
    pub fn CM_GetShaderInfo_int(shaderNum: c_int) -> *mut CCMShader;
    pub fn CM_GetModelFormalName(
        model: *const c_char,
        skin: *const c_char,
        name: *mut c_char,
        size: c_int,
    );

    //cm_trace.cpp
    // vec3pair_t bounds: array typedef decays to pointer in the C parameter list
    pub fn CM_CalcExtents(
        start: *const vec3_t,
        end: *const vec3_t,
        tw: *const traceWork_t,
        bounds: *mut vec3pair_t,
    );
    // C++ ref `trace_t &trace` translated as *mut trace_t; traceWork_s tag -> traceWork_t
    pub fn CM_HandlePatchCollision(
        tw: *mut traceWork_t,
        trace: *mut trace_t,
        tStart: *const vec3_t,
        tEnd: *const vec3_t,
        patch: *mut CCMPatch,
        checkcount: c_int,
    );
    pub fn CM_GenericBoxCollide(
        abounds: *const vec3pair_t,
        bbounds: *const vec3pair_t,
    ) -> bool;

    //RM_Terrain.cpp
    pub fn Round(value: f32) -> c_int;
}

//random utils for cm_terrain (and others?)
// #define VectorInc(v) ((v)[0] += 1.0f,(v)[1] += 1.0f,(v)[2] +=1.0f)
#[inline]
pub unsafe fn VectorInc(v: *mut vec3_t) {
    (*v)[0] += 1.0_f32;
    (*v)[1] += 1.0_f32;
    (*v)[2] += 1.0_f32;
}

// #define VectorDec(v) ((v)[0] -= 1.0f,(v)[1] -= 1.0f,(v)[2] -=1.0f)
#[inline]
pub unsafe fn VectorDec(v: *mut vec3_t) {
    (*v)[0] -= 1.0_f32;
    (*v)[1] -= 1.0_f32;
    (*v)[2] -= 1.0_f32;
}

// #define VectorInverseScaleVector(a,b,c) ((c)[0]=(a)[0]/(b)[0],(c)[1]=(a)[1]/(b)[1],(c)[2]=(a)[2]/(b)[2])
#[inline]
pub unsafe fn VectorInverseScaleVector(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t) {
    (*c)[0] = (*a)[0] / (*b)[0];
    (*c)[1] = (*a)[1] / (*b)[1];
    (*c)[2] = (*a)[2] / (*b)[2];
}

// #define VectorScaleVectorAdd(c,a,b,o) ((o)[0]=(c)[0]+((a)[0]*(b)[0]),(o)[1]=(c)[1]+((a)[1]*(b)[1]),(o)[2]=(c)[2]+((a)[2]*(b)[2]))
#[inline]
pub unsafe fn VectorScaleVectorAdd(
    c: *const vec3_t,
    a: *const vec3_t,
    b: *const vec3_t,
    o: *mut vec3_t,
) {
    (*o)[0] = (*c)[0] + ((*a)[0] * (*b)[0]);
    (*o)[1] = (*c)[1] + ((*a)[1] * (*b)[1]);
    (*o)[2] = (*c)[2] + ((*a)[2] * (*b)[2]);
}

// #define minimum(x,y) ((x)<(y)?(x):(y))
#[inline]
pub fn minimum<T: PartialOrd>(x: T, y: T) -> T {
    if x < y { x } else { y }
}

// #define maximum(x,y) ((x)>(y)?(x):(y))
#[inline]
pub fn maximum<T: PartialOrd>(x: T, y: T) -> T {
    if x > y { x } else { y }
}
