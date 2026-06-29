// Translated from: oracle/code/qcommon/cm_local.h

use core::ffi::{c_char, c_int, c_void};

// Included from qcommon.h - forward declarations and types
use crate::code::game::q_shared_h::*;
use crate::code::qcommon::cm_polylib_h::*;
use crate::code::qcommon::cm_landscape_h::*;

#[cfg(target_os = "xbox")]
mod xbox_specific {
    use core::ffi::c_int;
    use crate::code::game::q_shared_h::*;

    #[repr(C, packed)]
    pub struct cNode_t {
        pub children: [i16; 2], // negative numbers are leafs
    }
}

#[cfg(not(target_os = "xbox"))]
#[repr(C)]
pub struct cNode_t {
    pub plane: *mut cplane_t,
    pub children: [c_int; 2], // negative numbers are leafs
}

#[cfg(target_os = "xbox")]
pub use xbox_specific::cNode_t;

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
pub struct cmodel_t {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub leaf: cLeaf_t, // submodels don't reference the main tree
}

#[cfg(target_os = "xbox")]
mod xbox_brushside {
    use crate::code::game::q_shared_h::*;

    #[repr(C, packed)]
    pub struct cbrushside_t {
        pub planeNum: NotSoShort,
        pub shaderNum: u8,
    }
}

#[cfg(not(target_os = "xbox"))]
#[repr(C)]
pub struct cbrushside_t {
    pub plane: *mut cplane_t,
    pub shaderNum: c_int,
}

#[cfg(target_os = "xbox")]
pub use xbox_brushside::cbrushside_t;

#[repr(C)]
pub struct cbrush_t {
    pub shaderNum: c_int, // the shader that determined the contents
    pub contents: c_int,
    pub bounds: [vec3_t; 2],
    pub sides: *mut cbrushside_t,
    pub numsides: u16,
    pub checkcount: u16, // to avoid repeated testings
}

// C++ class translated to Rust struct
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

    pub fn Destroy(&mut self) {
        // empty destructor
    }
}

#[repr(C)]
pub struct cPatch_t {
    pub checkcount: c_int, // to avoid repeated testings
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub pc: *mut patchCollide_s,
}

#[repr(C)]
pub struct cArea_t {
    pub floodnum: c_int,
    pub floodvalid: c_int,
}

#[cfg(target_os = "xbox")]
mod xbox_clipmap {
    use core::ffi::{c_char, c_int};
    use crate::code::game::q_shared_h::*;
    use super::*;

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
        pub visibility: *mut SPARC_byte,
        pub vised: qboolean, // if false, visibility is just a single cluster of ffs
        pub numEntityChars: c_int,
        pub entityString: *mut c_char,
        pub numAreas: c_int,
        pub areas: *mut cArea_t,
        pub areaPortals: *mut c_int, // [ numAreas*numAreas ] reference counts
        pub numSurfaces: c_int,
        pub surfaces: *mut *mut cPatch_t, // non-patches will be NULL
        pub floodvalid: c_int,
        pub checkcount: c_int, // incremented on each trace
        //	pub landScape: *mut CCMLandScape,		// Removing terrain from Xbox
    }
}

#[cfg(not(target_os = "xbox"))]
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
    pub visibility: *mut u8,
    pub vised: qboolean, // if false, visibility is just a single cluster of ffs
    pub numEntityChars: c_int,
    pub entityString: *mut c_char,
    pub numAreas: c_int,
    pub areas: *mut cArea_t,
    pub areaPortals: *mut c_int, // [ numAreas*numAreas ] reference counts
    pub numSurfaces: c_int,
    pub surfaces: *mut *mut cPatch_t, // non-patches will be NULL
    pub floodvalid: c_int,
    pub checkcount: c_int, // incremented on each trace
    pub landScape: *mut CCMLandScape,
}

#[cfg(target_os = "xbox")]
pub use xbox_clipmap::clipMap_t;

const BOX_MODEL_HANDLE: c_int = MAX_SUBMODELS - 1;

// keep 1/8 unit away to keep the position valid before network snapping
// and to avoid various numeric issues
const SURFACE_CLIP_EPSILON: f32 = 0.125;

extern "C" {
    pub static mut cmg: clipMap_t;
    pub static mut c_pointcontents: c_int;
    pub static mut c_traces: c_int;
    pub static mut c_brush_traces: c_int;
    pub static mut c_patch_traces: c_int;
    pub static mut cm_noAreas: *mut cvar_t;
    pub static mut cm_noCurves: *mut cvar_t;
    pub static mut cm_playerCurveClip: *mut cvar_t;
}

extern "C" {
    pub static mut SubBSP: [clipMap_t; MAX_SUB_BSP as usize];
    pub static mut NumSubBSP: c_int;
}

// cm_test.c

// Used for oriented capsule collision detection
#[repr(C)]
pub struct sphere_t {
    pub use_: qboolean,
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
    pub maxOffset: f32, // longest corner length from origin
    pub extents: vec3_t, // greatest of abs(size[0]) and abs(size[1])
    pub bounds: [vec3_t; 2], // enclosing box of start and end surrounding by size
    pub localBounds: vec3pair_t, // enclosing box of start and end surrounding by size for a segment
    pub modelOrigin: vec3_t, // origin of the model tracing through
    pub contents: c_int, // ored contents of the model tracing through
    pub isPoint: qboolean, // optimized case
    pub sphere: sphere_t, // sphere for oriendted capsule collision
    pub baseEnterFrac: f32, // global enter fraction (before processing subsections of the brush)
    pub baseLeaveFrac: f32, // global leave fraction (before processing subsections of the brush)
    pub enterFrac: f32, // fraction where the ray enters the brush
    pub leaveFrac: f32, // fraction where the ray leaves the brush
    pub leadside: *mut cbrushside_t,
    pub clipplane: *mut cplane_t,
    pub startout: bool,
    pub getout: bool,
    pub trace: trace_t, // returned from trace call
    // make sure nothing goes under here for Ghoul2 collision purposes
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
    pub fn CM_ClipHandleToModel(handle: clipHandle_t, clipMap: *mut *mut clipMap_t) -> *mut cmodel_t;
    pub fn CM_CleanLeafCache();
    pub fn CM_ModelBounds(cm: &mut clipMap_t, model: clipHandle_t, mins: *mut vec3_t, maxs: *mut vec3_t);
    pub fn CM_GeneratePatchCollide(
        width: c_int,
        height: c_int,
        points: *mut vec3_t,
    ) -> *mut patchCollide_s;
    pub fn CM_TraceThroughPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s);
    pub fn CM_PositionTestInPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s) -> qboolean;
    pub fn CM_ClearLevelPatches();
    pub fn CM_SetupShaderProperties();
    pub fn CM_ShutdownShaderProperties();
    pub fn CM_GetShaderInfo(name: *const c_char) -> *mut CCMShader;
    pub fn CM_GetShaderInfo_i(shaderNum: c_int) -> *mut CCMShader;
    pub fn CM_GetModelFormalName(model: *const c_char, skin: *const c_char, name: *mut c_char, size: c_int);
    pub fn CM_CalcExtents(
        start: *const vec3_t,
        end: *const vec3_t,
        tw: *const traceWork_t,
        bounds: *mut vec3pair_t,
    );
    pub fn CM_HandlePatchCollision(
        tw: *mut traceWork_s,
        trace: &mut trace_t,
        tStart: *const vec3_t,
        tEnd: *const vec3_t,
        patch: *mut CCMPatch,
        checkcount: c_int,
    );
    pub fn CM_GenericBoxCollide(abounds: *const vec3pair_t, bbounds: *const vec3pair_t) -> bool;
    pub fn Round(value: f32) -> c_int;
}

//random utils for cm_terrain (and others?)
// VectorInc(v) macro: ((v)[0] += 1.0f,(v)[1] += 1.0f,(v)[2] +=1.0f)
#[inline]
pub fn VectorInc(v: &mut vec3_t) {
    v[0] += 1.0f32;
    v[1] += 1.0f32;
    v[2] += 1.0f32;
}

// VectorDec(v) macro: ((v)[0] -= 1.0f,(v)[1] -= 1.0f,(v)[2] -=1.0f)
#[inline]
pub fn VectorDec(v: &mut vec3_t) {
    v[0] -= 1.0f32;
    v[1] -= 1.0f32;
    v[2] -= 1.0f32;
}

// VectorInverseScaleVector(a,b,c) macro: ((c)[0]=(a)[0]/(b)[0],(c)[1]=(a)[1]/(b)[1],(c)[2]=(a)[2]/(b)[2])
#[inline]
pub fn VectorInverseScaleVector(a: &vec3_t, b: &vec3_t, c: &mut vec3_t) {
    c[0] = a[0] / b[0];
    c[1] = a[1] / b[1];
    c[2] = a[2] / b[2];
}

// VectorScaleVectorAdd(c,a,b,o) macro: ((o)[0]=(c)[0]+((a)[0]*(b)[0]),(o)[1]=(c)[1]+((a)[1]*(b)[1]),(o)[2]=(c)[2]+((a)[2]*(b)[2]))
#[inline]
pub fn VectorScaleVectorAdd(c: &vec3_t, a: &vec3_t, b: &vec3_t, o: &mut vec3_t) {
    o[0] = c[0] + (a[0] * b[0]);
    o[1] = c[1] + (a[1] * b[1]);
    o[2] = c[2] + (a[2] * b[2]);
}

// minimum(x,y) macro: ((x)<(y)?(x):(y))
#[inline]
pub const fn minimum(x: i32, y: i32) -> i32 {
    if x < y { x } else { y }
}

// maximum(x,y) macro: ((x)>(y)?(x):(y))
#[inline]
pub const fn maximum(x: i32, y: i32) -> i32 {
    if x > y { x } else { y }
}

// Forward declarations for types that may be defined elsewhere
#[repr(C)]
pub struct patchCollide_s {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CCMLandScape {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CCMPatch {
    _private: [u8; 0],
}

#[cfg(target_os = "xbox")]
#[repr(C)]
pub struct SPARC_byte {
    _private: [u8; 0],
}
