//! Mechanical port of `codemp/qcommon/cm_local.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use crate::codemp::game::q_shared_h::{
    byte, cplane_t, qboolean, trace_t, vec3_t, vec3pair_t, MAX_QPATH,
};
use crate::codemp::qcommon::cm_landscape_h::{CCMLandScape, thandle_t};
use crate::codemp::qcommon::cm_patch_h::patchCollide_s;
use crate::codemp::qcommon::cm_public_h::clipHandle_t;
use crate::codemp::qcommon::files_h::cvar_t;
use core::ffi::{c_char, c_int, c_short, c_uchar, c_ushort};

pub const MAX_SUBMODELS: c_int = 512;
pub const BOX_MODEL_HANDLE: c_int = MAX_SUBMODELS - 1;
pub const CAPSULE_MODEL_HANDLE: c_int = MAX_SUBMODELS - 2;

#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct NotSoShort {
    pub bytes: [u8; 3],
}

#[cfg(feature = "xbox")]
pub struct SPARC<T>(core::marker::PhantomData<T>);

#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct cNode_t {
    pub planeNum: c_int,
    pub children: [c_short; 2], // negative numbers are leafs
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct cNode_t {
    pub plane: *mut cplane_t,
    pub children: [c_int; 2], // negative numbers are leafs
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

const _: () = assert!(core::mem::size_of::<cLeaf_t>() == 20);
const _: () = assert!(core::mem::align_of::<cLeaf_t>() == 4);

#[repr(C)]
pub struct cmodel_s {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub leaf: cLeaf_t, // submodels don't reference the main tree
    pub firstNode: c_int, // only for cmodel[0] (for the main and bsp instances)
}

pub type cmodel_t = cmodel_s;

const _: () = assert!(core::mem::size_of::<cmodel_t>() == 48);
const _: () = assert!(core::mem::align_of::<cmodel_t>() == 4);

#[cfg(feature = "xbox")]
#[repr(C, packed)]
pub struct cbrushside_s {
    pub planeNum: NotSoShort,
    pub shaderNum: c_uchar,
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct cbrushside_s {
    pub plane: *mut cplane_t,
    pub shaderNum: c_int,
}

pub type cbrushside_t = cbrushside_s;

#[cfg(feature = "xbox")]
const _: () = assert!(core::mem::size_of::<NotSoShort>() == 3);
#[cfg(feature = "xbox")]
const _: () = assert!(core::mem::align_of::<NotSoShort>() == 1);
#[cfg(feature = "xbox")]
const _: () = assert!(core::mem::size_of::<cbrushside_t>() == 4);
#[cfg(feature = "xbox")]
const _: () = assert!(core::mem::align_of::<cbrushside_t>() == 1);

#[repr(C)]
pub struct cbrush_s {
    pub shaderNum: c_int, // the shader that determined the contents
    pub contents: c_int,
    pub bounds: [vec3_t; 2],
    pub sides: *mut cbrushside_t,
    pub numsides: c_ushort,
    pub checkcount: c_ushort, // to avoid repeated testings
}

pub type cbrush_t = cbrush_s;

#[repr(C)]
pub struct CCMShader {
    pub shader: [c_char; MAX_QPATH],
    pub mNext: *mut CCMShader,
    pub surfaceFlags: c_int,
    pub contentFlags: c_int,
}

impl CCMShader {
    #[inline]
    pub fn GetName(&self) -> *const c_char {
        self.shader.as_ptr()
    }

    #[inline]
    pub fn GetNext(&self) -> *mut CCMShader {
        self.mNext
    }

    #[inline]
    pub fn SetNext(&mut self, next: *mut CCMShader) {
        self.mNext = next;
    }

    #[inline]
    pub fn Destroy(&mut self) {}
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

const _: () = assert!(core::mem::size_of::<cArea_t>() == 8);
const _: () = assert!(core::mem::align_of::<cArea_t>() == 4);

#[cfg(feature = "xbox")]
#[repr(C)]
pub struct clipMap_t {
    pub name: [c_char; MAX_QPATH],

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
    pub haswater: qboolean,
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct clipMap_t {
    pub name: [c_char; MAX_QPATH],

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
    pub areas: *mut cArea_t,
    pub areaPortals: *mut c_int, // [ numAreas*numAreas ] reference counts

    pub numSurfaces: c_int,
    pub surfaces: *mut *mut cPatch_t, // non-patches will be NULL

    pub floodvalid: c_int,
    pub checkcount: c_int, // incremented on each trace

    // rwwRMG - added:
    pub landScape: *mut CCMLandScape,
}

// keep 1/8 unit away to keep the position valid before network snapping
// and to avoid various numeric issues
pub const SURFACE_CLIP_EPSILON: f32 = 0.125;

#[repr(C)]
pub struct sphere_t {
    pub r#use: qboolean,
    pub radius: f32,
    pub halfheight: f32,
    pub offset: vec3_t,
}

const _: () = assert!(core::mem::size_of::<sphere_t>() == 24);
const _: () = assert!(core::mem::align_of::<sphere_t>() == 4);

#[repr(C)]
pub struct traceWork_s {
    pub start: vec3_t,
    pub end: vec3_t,
    pub size: [vec3_t; 2], // size of the box being swept through the model
    pub offsets: [vec3_t; 8], // [signbits][x] = either size[0][x] or size[1][x]
    pub maxOffset: f32, // longest corner length from origin
    pub extents: vec3_t, // greatest of abs(size[0]) and abs(size[1])
    pub modelOrigin: vec3_t, // origin of the model tracing through
    pub contents: c_int, // ored contents of the model tracing through
    pub isPoint: qboolean, // optimized case
    // pub trace: trace_t, // returned from trace call
    pub sphere: sphere_t, // sphere for oriendted capsule collision

    // rwwRMG - added:
    pub bounds: vec3pair_t, // enclosing box of start and end surrounding by size
    pub localBounds: vec3pair_t, // enclosing box of start and end surrounding by size for a segment

    pub baseEnterFrac: f32, // global enter fraction (before processing subsections of the brush)
    pub baseLeaveFrac: f32, // global leave fraction (before processing subsections of the brush)
    pub enterFrac: f32, // fraction where the ray enters the brush
    pub leaveFrac: f32, // fraction where the ray leaves the brush
    pub leadside: *mut cbrushside_t,
    pub clipplane: *mut cplane_t,
    pub startout: bool,
    pub getout: bool,
}

pub type traceWork_t = traceWork_s;

#[repr(C)]
pub struct leafList_s {
    pub count: c_int,
    pub maxcount: c_int,
    pub overflowed: qboolean,
    pub list: *mut c_int,
    pub bounds: [vec3_t; 2],
    pub lastLeaf: c_int, // for overflows where each leaf can't be stored individually
    pub storeLeafs: Option<unsafe extern "C" fn(*mut leafList_s, c_int)>,
}

pub type leafList_t = leafList_s;

unsafe extern "C" {
    pub static mut cmg: clipMap_t; // rwwRMG - changed from cm
    pub static mut c_pointcontents: c_int;
    pub static mut c_traces: c_int;
    pub static mut c_brush_traces: c_int;
    pub static mut c_patch_traces: c_int;
    pub static mut cm_noAreas: *mut cvar_t;
    pub static mut cm_noCurves: *mut cvar_t;
    pub static mut cm_playerCurveClip: *mut cvar_t;

    // cm_test.c
    pub fn CM_BoxBrushes(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        boxList: *mut *mut cbrush_t,
        listsize: c_int,
    ) -> c_int;
    // rwwRMG - changed to boxList to not conflict with list type

    pub fn CM_CullWorldBox(frustum: *const cplane_t, bounds: *const vec3pair_t) -> bool;
    // rwwRMG - added

    pub fn CM_StoreLeafs(ll: *mut leafList_t, nodenum: c_int);
    pub fn CM_StoreBrushes(ll: *mut leafList_t, nodenum: c_int);

    pub fn CM_BoxLeafnums_r(ll: *mut leafList_t, nodenum: c_int);

    pub fn CM_ClipHandleToModel(
        handle: clipHandle_t,
        clipMap: *mut *mut clipMap_t, // C++ default: 0
    ) -> *mut cmodel_t;

    // cm_patch.c
    pub fn CM_GeneratePatchCollide(
        width: c_int,
        height: c_int,
        points: *mut vec3_t,
    ) -> *mut patchCollide_s;
    pub fn CM_TraceThroughPatchCollide(
        tw: *mut traceWork_t,
        trace: *mut trace_t,
        pc: *const patchCollide_s,
    );
    pub fn CM_PositionTestInPatchCollide(tw: *mut traceWork_t, pc: *const patchCollide_s) -> qboolean;
    pub fn CM_ClearLevelPatches();

    // rwwRMG - added
    pub fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *mut CCMLandScape;
    pub fn CM_ShutdownTerrain(terrainId: thandle_t);

    // cm_shader.cpp
    pub fn CM_SetupShaderProperties();
    pub fn CM_ShutdownShaderProperties();
    // Porting deviation: Rust cannot overload `CM_GetShaderInfo`, so the two C++
    // overloads are split into explicit declarations.
    pub fn CM_GetShaderInfo_name(name: *const c_char) -> *mut CCMShader;
    pub fn CM_GetShaderInfo_shaderNum(shaderNum: c_int) -> *mut CCMShader;
    pub fn CM_GetModelFormalName(
        model: *const c_char,
        skin: *const c_char,
        name: *mut c_char,
        size: c_int,
    );

    // cm_load.cpp
    pub fn CM_GetWorldBounds(mins: *mut vec3_t, maxs: *mut vec3_t);
}
