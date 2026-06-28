// cmodel.c -- model loading

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char, c_void};
use crate::code::game::q_shared_h::{
    byte, vec3_t, vec3pair_t, qboolean, clipHandle_t, vec_t, MAX_QPATH,
};
use crate::code::qcommon::qcommon_h::{cvar_t, LittleLong, LittleFloat};
use crate::code::qcommon::qfiles_h::{
    lump_t, dshader_t, dmodel_t, dnode_t, dleaf_t, dbrush_t, dplane_t, dbrushside_t,
    dsurface_t, mapVert_t, dheader_t,
};

// Forward declarations for types that may not be fully defined here
#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: vec_t,
    pub type_: byte,
    pub signbits: byte,
    pub pad: [byte; 2],
}

#[repr(C)]
pub struct CCMShader {
    pub shader: [c_char; MAX_QPATH as usize],
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
pub struct cmodel_s {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub leaf: cLeaf_t,
}
pub type cmodel_t = cmodel_s;

#[repr(C)]
pub struct cbrushside_t {
    pub plane: *mut cplane_t,
    pub shaderNum: c_int,
}

#[repr(C)]
pub struct cbrush_t {
    pub shaderNum: c_int,
    pub contents: c_int,
    pub bounds: [vec3_t; 2],
    pub sides: *mut cbrushside_t,
    pub numsides: u16,
    pub checkcount: u16,
}

#[repr(C)]
pub struct cNode_t {
    pub plane: *mut cplane_t,
    pub children: [c_int; 2],
}

#[repr(C)]
pub struct cArea_t {
    pub floodnum: c_int,
    pub floodvalid: c_int,
}

#[repr(C)]
pub struct cPatch_t {
    pub checkcount: c_int,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub pc: *mut patchCollide_s,
}

#[repr(C)]
pub struct patchCollide_s;

pub const BOX_BRUSHES: c_int = 1;
pub const BOX_SIDES: c_int = 6;
pub const BOX_LEAFS: c_int = 2;
pub const BOX_PLANES: c_int = 12;
pub const BOX_MODEL_HANDLE: c_int = 512 - 1;  // MAX_SUBMODELS - 1
pub const MAX_SUB_BSP: c_int = 8;
pub const MAX_SUBMODELS: c_int = 512;

pub static mut cmg: clipMap_t = unsafe {
    core::mem::MaybeUninit::uninit().assume_init()
};
pub static mut c_pointcontents: c_int = 0;
pub static mut c_traces: c_int = 0;
pub static mut c_brush_traces: c_int = 0;
pub static mut c_patch_traces: c_int = 0;

pub static mut cmod_base: *mut byte = core::ptr::null_mut();

#[cfg(not(feature = "bspc"))]
pub static mut cm_noAreas: *mut cvar_t = core::ptr::null_mut();
#[cfg(not(feature = "bspc"))]
pub static mut cm_noCurves: *mut cvar_t = core::ptr::null_mut();
#[cfg(not(feature = "bspc"))]
pub static mut cm_playerCurveClip: *mut cvar_t = core::ptr::null_mut();

pub static mut box_model: cmodel_t = unsafe {
    core::mem::MaybeUninit::uninit().assume_init()
};
pub static mut box_planes: *mut cplane_t = core::ptr::null_mut();
pub static mut box_brush: *mut cbrush_t = core::ptr::null_mut();

pub static mut CM_OrOfAllContentsFlagsInMap: c_int = 0;

pub static mut SubBSP: [clipMap_t; MAX_SUB_BSP as usize] = unsafe {
    core::mem::MaybeUninit::uninit().assume_init()
};
pub static mut NumSubBSP: c_int = 0;
pub static mut TotalSubModels: c_int = 0;

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
    pub visibility: *mut byte,
    pub vised: qboolean,
    pub numEntityChars: c_int,
    pub entityString: *mut c_char,
    pub numAreas: c_int,
    pub areas: *mut cArea_t,
    pub areaPortals: *mut c_int,
    pub numSurfaces: c_int,
    pub surfaces: *mut *mut cPatch_t,
    pub floodvalid: c_int,
    pub checkcount: c_int,
    pub landScape: *mut c_void,
}

extern "C" {
    pub fn CM_InitBoxHull();
    pub fn CM_FloodAreaConnections(cm: *mut clipMap_t);
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Z_Malloc(size: c_int, tag: c_int, clear: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Z_TagFree(tag: c_int);
    pub fn FS_FOpenFileRead(filename: *const c_char, handle: *mut c_int, unique_file: qboolean) -> c_int;
    pub fn FS_Read(buffer: *mut c_void, len: c_int, f: c_int);
    pub fn FS_FCloseFile(f: c_int);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn VectorCopy(in_: *const vec3_t, out: *mut vec3_t);
    pub fn VectorClear(v: *mut vec3_t);
    pub fn Com_BlockChecksum(data: *const c_void, length: c_int) -> u32;
    pub fn CM_GeneratePatchCollide(width: c_int, height: c_int, points: *mut vec3_t) -> *mut patchCollide_s;
    pub fn CM_ClearLevelPatches();
    pub fn CM_LoadShaderText(forceReload: qboolean);
    pub fn CM_SetupShaderProperties();
    pub fn CM_ShutdownShaderProperties();
    pub fn CM_CleanLeafCache();
    pub fn Sys_LowPhysicalMemory() -> qboolean;
    pub fn PlaneTypeForNormal(normal: *const vec3_t) -> c_int;
    pub fn va(fmt: *const c_char, ...) -> *mut c_char;
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    pub fn CM_InitTerrain(config: *const c_char, checksum: c_int, server: bool) -> *mut c_void;
    pub fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn atol(str: *const c_char) -> i64;
    pub fn SetPlaneSignbits(p: *mut cplane_t);
}

const TAG_BSP: c_int = 1;
const TAG_BSP_DISKIMAGE: c_int = 2;
const CVAR_CHEAT: c_int = 1;
const CVAR_ARCHIVE: c_int = 2;
const ERR_DROP: c_int = 1;
const VIS_HEADER: c_int = 8;
const MAX_PATCH_VERTS: c_int = 1024;
const MST_PATCH: c_int = 2;

static mut gpvCachedMapDiskImage: *mut c_void = core::ptr::null_mut();
static mut gsCachedMapDiskImage: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
static mut gbUsingCachedMapDataRightNow: qboolean = 0;

/*
===============================================================================

					MAP LOADING

===============================================================================
*/

/*
=================
CMod_LoadShaders
=================
*/
pub unsafe fn CMod_LoadShaders(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut in_: *mut dshader_t;
    let mut i: c_int;
    let mut count: c_int;
    let mut out: *mut CCMShader;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut dshader_t;
    if (*l).filelen % core::mem::size_of::<dshader_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"CMod_LoadShaders: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<dshader_t>() as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no shaders\0".as_ptr() as *const c_char);
    }
    cm.shaders = Z_Malloc((1 + count) * core::mem::size_of::<CCMShader>() as c_int, TAG_BSP, 1) as *mut CCMShader;
    cm.numShaders = count;

    out = cm.shaders;
    i = 0;
    while i < count {
        Q_strncpyz((*out).shader.as_mut_ptr(), (*in_).shader.as_ptr(), MAX_QPATH);
        (*out).contentFlags = LittleLong((*in_).contentFlags);
        (*out).surfaceFlags = LittleLong((*in_).surfaceFlags);
        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
CMod_LoadSubmodels
=================
*/
pub unsafe fn CMod_LoadSubmodels(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut in_: *mut dmodel_t;
    let mut out: *mut cmodel_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;
    let mut indexes: *mut c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut dmodel_t;
    if (*l).filelen % core::mem::size_of::<dmodel_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"CMod_LoadSubmodels: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<dmodel_t>() as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no models\0".as_ptr() as *const c_char);
    }

    // FIXME: note that MAX_SUBMODELS - 1 is used for BOX_MODEL_HANDLE, if that slot gets used, that would be bad, no?
    if count > MAX_SUBMODELS {
        Com_Error(ERR_DROP, b"MAX_SUBMODELS (%d) exceeded by %d\0".as_ptr() as *const c_char, MAX_SUBMODELS, count - MAX_SUBMODELS);
    }

    cm.cmodels = Z_Malloc(count * core::mem::size_of::<cmodel_t>() as c_int, TAG_BSP, 1) as *mut cmodel_t;
    cm.numSubModels = count;

    i = 0;
    while i < count {
        out = &mut (*cm.cmodels.add(i as usize));

        j = 0;
        while j < 3 {
            (*out).mins[j as usize] = LittleFloat((*in_).mins[j as usize]) - 1.0;
            (*out).maxs[j as usize] = LittleFloat((*in_).maxs[j as usize]) + 1.0;
            j += 1;
        }

        // rww - I changed this to do the &cm == &cmg check. sof2 does not have to do this,
        // but I think they have a different tracing system that allows it to catch subbsp
        // stuff without extra leaf data. The reason we have to do this for subbsp instances
        // is that they often are compiled in a sort of "prefab" form, so the first model isn't
        // necessarily the world model.
        if i == 0 && (cm as *const _ == &cmg as *const _) {
            in_ = in_.add(1);
            i += 1;
            continue;	// world model doesn't need other info
        }

        // make a "leaf" just to hold the model's brushes and surfaces
        (*out).leaf.numLeafBrushes = LittleLong((*in_).numBrushes);
        indexes = Z_Malloc((*out).leaf.numLeafBrushes * 4, TAG_BSP, 0) as *mut c_int;
        (*out).leaf.firstLeafBrush = indexes as isize - cm.leafbrushes as isize;
        j = 0;
        while j < (*out).leaf.numLeafBrushes {
            *indexes.add(j as usize) = LittleLong((*in_).firstBrush) + j;
            j += 1;
        }

        (*out).leaf.numLeafSurfaces = LittleLong((*in_).numSurfaces);
        indexes = Z_Malloc((*out).leaf.numLeafSurfaces * 4, TAG_BSP, 0) as *mut c_int;
        (*out).leaf.firstLeafSurface = indexes as isize - cm.leafsurfaces as isize;
        j = 0;
        while j < (*out).leaf.numLeafSurfaces {
            *indexes.add(j as usize) = LittleLong((*in_).firstSurface) + j;
            j += 1;
        }

        in_ = in_.add(1);
        i += 1;
    }
}

/*
=================
CMod_LoadNodes
=================
*/
pub unsafe fn CMod_LoadNodes(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut in_: *mut dnode_t;
    let mut child: c_int;
    let mut out: *mut cNode_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut dnode_t;
    if (*l).filelen % core::mem::size_of::<dnode_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<dnode_t>() as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map has no nodes\0".as_ptr() as *const c_char);
    }
    cm.nodes = Z_Malloc(count * core::mem::size_of::<cNode_t>() as c_int, TAG_BSP, 0) as *mut cNode_t;
    cm.numNodes = count;

    out = cm.nodes;

    i = 0;
    while i < count {
        (*out).plane = cm.planes.add(LittleLong((*in_).planeNum) as usize);
        j = 0;
        while j < 2 {
            child = LittleLong((*in_).children[j as usize]);
            (*out).children[j as usize] = child;
            j += 1;
        }
        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
CM_BoundBrush
=================
*/
pub unsafe fn CM_BoundBrush(b: *mut cbrush_t) {
    (*b).bounds[0][0] = -(*(*b).sides).plane.as_ref().unwrap().dist;
    (*b).bounds[1][0] = (*(*b).sides.add(1)).plane.as_ref().unwrap().dist;

    (*b).bounds[0][1] = -(*(*b).sides.add(2)).plane.as_ref().unwrap().dist;
    (*b).bounds[1][1] = (*(*b).sides.add(3)).plane.as_ref().unwrap().dist;

    (*b).bounds[0][2] = -(*(*b).sides.add(4)).plane.as_ref().unwrap().dist;
    (*b).bounds[1][2] = (*(*b).sides.add(5)).plane.as_ref().unwrap().dist;
}

/*
=================
CMod_LoadBrushes
=================
*/
pub unsafe fn CMod_LoadBrushes(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut in_: *mut dbrush_t;
    let mut out: *mut cbrush_t;
    let mut i: c_int;
    let mut count: c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut dbrush_t;
    if (*l).filelen % core::mem::size_of::<dbrush_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<dbrush_t>() as c_int;

    cm.brushes = Z_Malloc((BOX_BRUSHES + count) * core::mem::size_of::<cbrush_t>() as c_int, TAG_BSP, 0) as *mut cbrush_t;
    cm.numBrushes = count;

    out = cm.brushes;

    i = 0;
    while i < count {
        (*out).sides = cm.brushsides.add(LittleLong((*in_).firstSide) as usize);
        (*out).numsides = LittleLong((*in_).numSides) as u16;

        (*out).shaderNum = LittleLong((*in_).shaderNum);
        if (*out).shaderNum < 0 || (*out).shaderNum >= cm.numShaders {
            Com_Error(ERR_DROP, b"CMod_LoadBrushes: bad shaderNum: %i\0".as_ptr() as *const c_char, (*out).shaderNum);
        }
        (*out).contents = (*cm.shaders.add((*out).shaderNum as usize)).contentFlags;
        CM_OrOfAllContentsFlagsInMap |= (*out).contents;
        (*out).checkcount = 0;

        CM_BoundBrush(out);

        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
CMod_LoadLeafs
=================
*/
pub unsafe fn CMod_LoadLeafs(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut i: c_int;
    let mut out: *mut cLeaf_t;
    let mut in_: *mut dleaf_t;
    let mut count: c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut dleaf_t;
    if (*l).filelen % core::mem::size_of::<dleaf_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<dleaf_t>() as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no leafs\0".as_ptr() as *const c_char);
    }

    cm.leafs = Z_Malloc((BOX_LEAFS + count) * core::mem::size_of::<cLeaf_t>() as c_int, TAG_BSP, 0) as *mut cLeaf_t;
    cm.numLeafs = count;
    out = cm.leafs;

    i = 0;
    while i < count {
        (*out).cluster = LittleLong((*in_).cluster);
        (*out).area = LittleLong((*in_).area);
        (*out).firstLeafBrush = LittleLong((*in_).firstLeafBrush);
        (*out).numLeafBrushes = LittleLong((*in_).numLeafBrushes);
        (*out).firstLeafSurface = LittleLong((*in_).firstLeafSurface);
        (*out).numLeafSurfaces = LittleLong((*in_).numLeafSurfaces);

        if (*out).cluster >= cm.numClusters {
            cm.numClusters = (*out).cluster + 1;
        }
        if (*out).area >= cm.numAreas {
            cm.numAreas = (*out).area + 1;
        }

        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }

    cm.areas = Z_Malloc(cm.numAreas * core::mem::size_of::<cArea_t>() as c_int, TAG_BSP, 1) as *mut cArea_t;
    cm.areaPortals = Z_Malloc(cm.numAreas * cm.numAreas * core::mem::size_of::<c_int>() as c_int, TAG_BSP, 1) as *mut c_int;
}

/*
=================
CMod_LoadPlanes
=================
*/
pub unsafe fn CMod_LoadPlanes(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut out: *mut cplane_t;
    let mut in_: *mut dplane_t;
    let mut count: c_int;
    let mut bits: c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut dplane_t;
    if (*l).filelen % core::mem::size_of::<dplane_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<dplane_t>() as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no planes\0".as_ptr() as *const c_char);
    }
    cm.planes = Z_Malloc((BOX_PLANES + count) * core::mem::size_of::<cplane_t>() as c_int, TAG_BSP, 0) as *mut cplane_t;
    cm.numPlanes = count;

    out = cm.planes;

    i = 0;
    while i < count {
        bits = 0;
        j = 0;
        while j < 3 {
            (*out).normal[j as usize] = LittleFloat((*in_).normal[j as usize]);
            if (*out).normal[j as usize] < 0.0 {
                bits |= 1 << j;
            }
            j += 1;
        }

        (*out).dist = LittleFloat((*in_).dist);
        (*out).type_ = PlaneTypeForNormal((*out).normal.as_ptr()) as byte;
        (*out).signbits = bits as byte;

        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
CMod_LoadLeafBrushes
=================
*/
pub unsafe fn CMod_LoadLeafBrushes(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut i: c_int;
    let mut out: *mut c_int;
    let mut in_: *mut c_int;
    let mut count: c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut c_int;
    if (*l).filelen % core::mem::size_of::<c_int>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<c_int>() as c_int;

    cm.leafbrushes = Z_Malloc((BOX_BRUSHES + count) * core::mem::size_of::<c_int>() as c_int, TAG_BSP, 0) as *mut c_int;
    cm.numLeafBrushes = count;

    out = cm.leafbrushes;

    i = 0;
    while i < count {
        *out = LittleLong(*in_);
        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
CMod_LoadLeafSurfaces
=================
*/
pub unsafe fn CMod_LoadLeafSurfaces(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut i: c_int;
    let mut out: *mut c_int;
    let mut in_: *mut c_int;
    let mut count: c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut c_int;
    if (*l).filelen % core::mem::size_of::<c_int>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<c_int>() as c_int;

    cm.leafsurfaces = Z_Malloc(count * core::mem::size_of::<c_int>() as c_int, TAG_BSP, 0) as *mut c_int;
    cm.numLeafSurfaces = count;

    out = cm.leafsurfaces;

    i = 0;
    while i < count {
        *out = LittleLong(*in_);
        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
CMod_LoadBrushSides
=================
*/
pub unsafe fn CMod_LoadBrushSides(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut i: c_int;
    let mut out: *mut cbrushside_t;
    let mut in_: *mut dbrushside_t;
    let mut count: c_int;
    let mut num: c_int;

    in_ = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut dbrushside_t;
    if (*l).filelen % core::mem::size_of::<dbrushside_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (*l).filelen / core::mem::size_of::<dbrushside_t>() as c_int;

    cm.brushsides = Z_Malloc((BOX_SIDES + count) * core::mem::size_of::<cbrushside_t>() as c_int, TAG_BSP, 0) as *mut cbrushside_t;
    cm.numBrushSides = count;

    out = cm.brushsides;

    i = 0;
    while i < count {
        num = LittleLong((*in_).planeNum);
        (*out).plane = &mut *cm.planes.add(num as usize);
        (*out).shaderNum = LittleLong((*in_).shaderNum);
        if (*out).shaderNum < 0 || (*out).shaderNum >= cm.numShaders {
            Com_Error(ERR_DROP, b"CMod_LoadBrushSides: bad shaderNum: %i\0".as_ptr() as *const c_char, (*out).shaderNum);
        }
        // out->surfaceFlags = cm.shaders[out->shaderNum].surfaceFlags;

        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
CMod_LoadEntityString
=================
*/
pub unsafe fn CMod_LoadEntityString(l: *mut lump_t, cm: &mut clipMap_t) {
    cm.entityString = Z_Malloc((*l).filelen, TAG_BSP, 0) as *mut c_char;
    cm.numEntityChars = (*l).filelen;
    core::ptr::copy_nonoverlapping(
        (cmod_base as *mut u8).add((*l).fileofs as usize) as *const c_void,
        cm.entityString as *mut c_void,
        (*l).filelen as usize,
    );
}

/*
=================
CMod_LoadVisibility
=================
*/
pub unsafe fn CMod_LoadVisibility(l: *mut lump_t, cm: &mut clipMap_t) {
    let mut len: c_int;
    let mut buf: *mut byte;

    len = (*l).filelen;
    if len == 0 {
        cm.clusterBytes = (cm.numClusters + 31) & !31;
        cm.visibility = Z_Malloc(cm.clusterBytes, TAG_BSP, 0) as *mut byte;
        core::ptr::write_bytes(cm.visibility, 255, cm.clusterBytes as usize);
        return;
    }
    buf = (cmod_base as *mut u8).add((*l).fileofs as usize) as *mut byte;

    cm.vised = 1;
    cm.visibility = Z_Malloc(len, TAG_BSP, 1) as *mut byte;
    cm.numClusters = LittleLong(*(buf as *mut c_int));
    cm.clusterBytes = LittleLong(*((buf as *mut c_int).add(1)));
    core::ptr::copy_nonoverlapping(
        buf.add(VIS_HEADER as usize) as *const c_void,
        cm.visibility as *mut c_void,
        (len - VIS_HEADER) as usize,
    );
}

//==================================================================

/*
=================
CMod_LoadPatches
=================
*/
pub unsafe fn CMod_LoadPatches(surfs: *mut lump_t, verts: *mut lump_t, cm: &mut clipMap_t) {
    let mut dv: *mut mapVert_t;
    let mut dv_p: *mut mapVert_t;
    let mut in_: *mut dsurface_t;
    let mut count: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut c: c_int;
    let mut patch: *mut cPatch_t;
    let mut points: [vec3_t; MAX_PATCH_VERTS as usize] = [[0.0; 3]; MAX_PATCH_VERTS as usize];
    let mut width: c_int;
    let mut height: c_int;
    let mut shaderNum: c_int;

    in_ = (cmod_base as *mut u8).add((*surfs).fileofs as usize) as *mut dsurface_t;
    if (*surfs).filelen % core::mem::size_of::<dsurface_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    cm.numSurfaces = count = (*surfs).filelen / core::mem::size_of::<dsurface_t>() as c_int;
    cm.surfaces = Z_Malloc(cm.numSurfaces * core::mem::size_of::<*mut cPatch_t>() as c_int, TAG_BSP, 1) as *mut *mut cPatch_t;

    dv = (cmod_base as *mut u8).add((*verts).fileofs as usize) as *mut mapVert_t;
    if (*verts).filelen % core::mem::size_of::<mapVert_t>() as c_int != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }

    // scan through all the surfaces, but only load patches,
    // not planar faces
    i = 0;
    while i < count {
        if LittleLong((*in_).surfaceType) != MST_PATCH {
            in_ = in_.add(1);
            i += 1;
            continue;		// ignore other surfaces
        }
        // FIXME: check for non-colliding patches

        *cm.surfaces.add(i as usize) = patch = Z_Malloc(core::mem::size_of::<cPatch_t>() as c_int, TAG_BSP, 1) as *mut cPatch_t;

        // load the full drawverts onto the stack
        width = LittleLong((*in_).patchWidth);
        height = LittleLong((*in_).patchHeight);
        c = width * height;
        if c > MAX_PATCH_VERTS {
            Com_Error(ERR_DROP, b"ParseMesh: MAX_PATCH_VERTS\0".as_ptr() as *const c_char);
        }

        dv_p = dv.add(LittleLong((*in_).firstVert) as usize);
        j = 0;
        while j < c {
            points[j as usize][0] = LittleFloat((*dv_p).xyz[0]);
            points[j as usize][1] = LittleFloat((*dv_p).xyz[1]);
            points[j as usize][2] = LittleFloat((*dv_p).xyz[2]);
            dv_p = dv_p.add(1);
            j += 1;
        }

        shaderNum = LittleLong((*in_).shaderNum);
        (*patch).contents = (*cm.shaders.add(shaderNum as usize)).contentFlags;
        CM_OrOfAllContentsFlagsInMap |= (*patch).contents;

        (*patch).surfaceFlags = (*cm.shaders.add(shaderNum as usize)).surfaceFlags;

        // create the internal facet structure
        (*patch).pc = CM_GeneratePatchCollide(width, height, points.as_mut_ptr());

        in_ = in_.add(1);
        i += 1;
    }
}

//==================================================================

#[cfg(feature = "bspc")]
pub unsafe fn CM_FreeMap() {
    core::ptr::write_bytes(&mut cmg as *mut _, 0, 1);
    Hunk_ClearHigh();
    CM_ClearLevelPatches();
}

pub unsafe fn CM_LumpChecksum(lump: *mut lump_t) -> u32 {
    LittleLong(Com_BlockChecksum(
        (cmod_base as *mut u8).add((*lump).fileofs as usize) as *const c_void,
        (*lump).filelen,
    ) as c_int) as u32
}

pub unsafe fn CM_Checksum(header: *mut dheader_t) -> u32 {
    let mut checksums: [u32; 16] = [0; 16];
    checksums[0] = CM_LumpChecksum(&mut (*header).lumps[1]);
    checksums[1] = CM_LumpChecksum(&mut (*header).lumps[4]);
    checksums[2] = CM_LumpChecksum(&mut (*header).lumps[6]);
    checksums[3] = CM_LumpChecksum(&mut (*header).lumps[5]);
    checksums[4] = CM_LumpChecksum(&mut (*header).lumps[2]);
    checksums[5] = CM_LumpChecksum(&mut (*header).lumps[9]);
    checksums[6] = CM_LumpChecksum(&mut (*header).lumps[8]);
    checksums[7] = CM_LumpChecksum(&mut (*header).lumps[7]);
    checksums[8] = CM_LumpChecksum(&mut (*header).lumps[3]);
    checksums[9] = CM_LumpChecksum(&mut (*header).lumps[13]);
    checksums[10] = CM_LumpChecksum(&mut (*header).lumps[10]);

    LittleLong(Com_BlockChecksum(checksums.as_ptr() as *const c_void, 11 * 4) as c_int) as u32
}

/*
==================
CM_LoadMap

Loads in the map and all submodels
==================
*/
static mut last_checksum: u32 = 0;

pub unsafe fn CM_DeleteCachedMap(bGuaranteedOkToDelete: qboolean) -> qboolean {
    let mut bActuallyFreedSomething: qboolean = 0;

    if (bGuaranteedOkToDelete != 0) || (gbUsingCachedMapDataRightNow == 0) {
        // dump cached disk image...
        if !gpvCachedMapDiskImage.is_null() {
            Z_Free(gpvCachedMapDiskImage);
            gpvCachedMapDiskImage = core::ptr::null_mut();

            bActuallyFreedSomething = 1;
        }
        gsCachedMapDiskImage[0] = 0;

        // force map loader to ignore cached internal BSP structures for next level CM_LoadMap() call...
        cmg.name[0] = 0;
    }

    return bActuallyFreedSomething;
}

pub unsafe fn CM_LoadMap_Actual(name: *const c_char, clientload: qboolean, checksum: *mut c_int, cm: *mut clipMap_t) {
    let mut buf: *mut c_int;
    let mut i: c_int;
    let mut header: dheader_t = core::mem::MaybeUninit::uninit().assume_init();
    let mut subBSPData: *mut c_void = core::ptr::null_mut();

    if name.is_null() || *name == 0 {
        Com_Error(ERR_DROP, b"CM_LoadMap: NULL name\0".as_ptr() as *const c_char);
    }

    #[cfg(not(feature = "bspc"))]
    {
        cm_noAreas = Cvar_Get(b"cm_noAreas\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
        cm_noCurves = Cvar_Get(b"cm_noCurves\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_CHEAT);
        cm_playerCurveClip = Cvar_Get(b"cm_playerCurveClip\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE | CVAR_CHEAT);
    }
    Com_DPrintf(b"CM_LoadMap( %s, %i )\n\0".as_ptr() as *const c_char, name, clientload);

    if (core::ffi::CStr::from_ptr((*cm).name.as_ptr()).to_bytes() == core::ffi::CStr::from_ptr(name).to_bytes()) && (clientload != 0) {
        *checksum = last_checksum as c_int;
        return;
    }

    if cm as *const _ == &cmg as *const _ {
        // if there was a cached disk image but the name was empty (ie ERR_DROP happened) or just doesn't match
        // the current name, then ditch it...
        if (!gpvCachedMapDiskImage.is_null()) &&
            ((gsCachedMapDiskImage[0] == 0) || (core::ffi::CStr::from_ptr(gsCachedMapDiskImage.as_ptr()).to_bytes() != core::ffi::CStr::from_ptr(name).to_bytes()))
        {
            Z_Free(gpvCachedMapDiskImage);
            gpvCachedMapDiskImage = core::ptr::null_mut();
            gsCachedMapDiskImage[0] = 0;

            CM_ClearMap();
        }
    }

    // if there's a valid map name, and it's the same as last time (respawn?), and it's the server-load,
    // then keep the data from last time...
    if (*name != 0) && (core::ffi::CStr::from_ptr((*cm).name.as_ptr()).to_bytes() == core::ffi::CStr::from_ptr(name).to_bytes()) &&
        (clientload == 0) && (cm as *const _ == &cmg as *const _)
    {
        // clear some stuff that needs zeroing...
        (*cm).floodvalid = 0;
        // NO... don't reset this because the brush checkcounts are cached,
        // so when you load up, brush checkcounts equal the cm.checkcount
        // and the trace will be skipped (because everything loads and
        // traces in the same exact order ever time you load the map)
        (*cm).checkcount += 1;
        core::ptr::write_bytes((*cm).areas, 0, ((*cm).numAreas * core::mem::size_of::<cArea_t>() as c_int) as usize);
        core::ptr::write_bytes((*cm).areaPortals, 0, ((*cm).numAreas * (*cm).numAreas * core::mem::size_of::<c_int>() as c_int) as usize);
    } else {
        // ... else load map from scratch...
        if cm as *const _ == &cmg as *const _ {
            assert_eq!(clientload, 0);	// logic check. I'm assuming that a client load doesn't get this far?

            // free old stuff
            core::ptr::write_bytes(cm, 0, 1);
            CM_ClearLevelPatches();
            Z_TagFree(TAG_BSP);

            if *name == 0 {
                (*cm).numLeafs = 1;
                (*cm).numClusters = 1;
                (*cm).numAreas = 1;
                (*cm).cmodels = Z_Malloc(core::mem::size_of::<cmodel_t>() as c_int, TAG_BSP, 1) as *mut cmodel_t;
                *checksum = 0;
                return;
            }
        }

        // load the file into a buffer that we either discard as usual at the bottom, or if we've got enough memory
        // then keep it long enough to save the renderer re-loading it, then discard it after that.
        let mut h: c_int = 0;
        let iBSPLen: c_int = FS_FOpenFileRead(name, &mut h, 0);
        if h == 0 {
            Com_Error(ERR_DROP, b"Couldn't load %s\0".as_ptr() as *const c_char, name);
            return;
        }
        // rww - only do this when not loading a sub-bsp!
        if cm as *const _ == &cmg as *const _ {
            if !gpvCachedMapDiskImage.is_null() && gsCachedMapDiskImage[0] != 0 {
                // didn't get cleared elsewhere so free it before we allocate the pointer again
                // Maps with terrain will allow this to happen because they want everything to be cleared out (going between terrain and no-terrain is messy)
                Z_Free(gpvCachedMapDiskImage);
            }
            gsCachedMapDiskImage[0] = 0;		// flag that map isn't valid, until name is filled in
            gpvCachedMapDiskImage = Z_Malloc(iBSPLen, TAG_BSP_DISKIMAGE, 0);
            FS_Read(gpvCachedMapDiskImage, iBSPLen, h);
            FS_FCloseFile(h);

            buf = gpvCachedMapDiskImage as *mut c_int;	// so the rest of the code works as normal
        } else {
            // otherwise, read straight in..
            subBSPData = Z_Malloc(iBSPLen, TAG_BSP_DISKIMAGE, 0);
            FS_Read(subBSPData, iBSPLen, h);
            FS_FCloseFile(h);

            buf = subBSPData as *mut c_int;
        }

        // carry on as before...

        last_checksum = LittleLong(Com_BlockChecksum(buf as *const c_void, iBSPLen)) as u32;

        header = *(buf as *mut dheader_t);
        i = 0;
        while i < (core::mem::size_of::<dheader_t>() as c_int) / 4 {
            *((((&mut header) as *mut dheader_t) as *mut c_int).add(i as usize)) = LittleLong(*(((buf as *mut c_int).add(i as usize))));
            i += 1;
        }

        if header.version != 1 {
            Z_Free(gpvCachedMapDiskImage);
            gpvCachedMapDiskImage = core::ptr::null_mut();

            Com_Error(ERR_DROP, b"CM_LoadMap: %s has wrong version number (%i should be %i)\0".as_ptr() as *const c_char, name, header.version, 1);
        }

        cmod_base = buf as *mut byte;

        // load into heap
        CMod_LoadShaders(&mut header.lumps[1], &mut *cm);
        CMod_LoadLeafs(&mut header.lumps[4], &mut *cm);
        CMod_LoadLeafBrushes(&mut header.lumps[6], &mut *cm);
        CMod_LoadLeafSurfaces(&mut header.lumps[5], &mut *cm);
        CMod_LoadPlanes(&mut header.lumps[2], &mut *cm);
        CMod_LoadBrushSides(&mut header.lumps[9], &mut *cm);
        CMod_LoadBrushes(&mut header.lumps[8], &mut *cm);
        CMod_LoadSubmodels(&mut header.lumps[7], &mut *cm);
        CMod_LoadNodes(&mut header.lumps[3], &mut *cm);
        CMod_LoadEntityString(&mut header.lumps[0], &mut *cm);
        CMod_LoadVisibility(&mut header.lumps[16], &mut *cm);
        CMod_LoadPatches(&mut header.lumps[13], &mut header.lumps[10], &mut *cm);

        TotalSubModels += (*cm).numSubModels;

        // we are NOT freeing the file, because it is cached for the ref
        // actually we do because the new hunk sys won't allow it
        // actually we DON'T now <g>, if we've got enough ram to keep it for the renderer's disk-load...
        if (Sys_LowPhysicalMemory() != 0) {
            Z_Free(gpvCachedMapDiskImage);
            gpvCachedMapDiskImage = core::ptr::null_mut();
        } else {
            // ... do nothing, and let the renderer free it after it's finished playing with it...
        }

        if !subBSPData.is_null() {
            Z_Free(subBSPData);
        }

        if cm as *const _ == &cmg as *const _ {
            #[cfg(not(feature = "bspc"))]
            {
                CM_LoadShaderText(0);
            }
            CM_InitBoxHull();
            #[cfg(not(feature = "bspc"))]
            {
                CM_SetupShaderProperties();
            }

            Q_strncpyz(gsCachedMapDiskImage.as_mut_ptr(), name, core::mem::size_of::<[c_char; MAX_QPATH as usize]>() as c_int);	// so the renderer can check it
        }
    }

    *checksum = last_checksum as c_int;

    // do this whether or not the map was cached from last load...
    CM_FloodAreaConnections(cm);

    // allow this to be cached if it is loaded by the server
    if clientload == 0 {
        Q_strncpyz((*cm).name.as_mut_ptr(), name, core::mem::size_of::<[c_char; MAX_QPATH as usize]>() as c_int);
    }
    CM_CleanLeafCache();
}

// need a wrapper function around this because of multiple returns, need to ensure bool is correct...
pub unsafe fn CM_LoadMap(name: *const c_char, clientload: qboolean, checksum: *mut c_int, subBSP: qboolean) {
    if subBSP != 0 {
        CM_LoadSubBSP(va(b"maps/%s.bsp\0".as_ptr() as *const c_char, (name as *const c_char).add(1)), 0);
        // CM_LoadMap_Actual( name, clientload, checksum, cmg );
    } else {
        gbUsingCachedMapDataRightNow = 1;	// !!!!!!!!!!!!!!!!!!

        CM_LoadMap_Actual(name, clientload, checksum, &mut cmg);

        gbUsingCachedMapDataRightNow = 0;	// !!!!!!!!!!!!!!!!!!
    }
}

pub unsafe fn CM_SameMap(server: *mut c_char) -> qboolean {
    if (cmg.name[0] == 0) || server.is_null() || (*server == 0) {
        return 0;
    }

    if stricmp(cmg.name.as_ptr(), va(b"maps/%s.bsp\0".as_ptr() as *const c_char, server)) != 0 {
        return 0;
    }

    return 1;
}

pub unsafe fn CM_HasTerrain() -> qboolean {
    if !cmg.landScape.is_null() {
        return 1;
    }

    return 0;
}

/*
==================
CM_ClearMap
==================
*/
pub unsafe fn CM_ClearMap() {
    let mut i: c_int;

    CM_OrOfAllContentsFlagsInMap = 0x2;  // CONTENTS_BODY

    #[cfg(not(feature = "bspc"))]
    {
        CM_ShutdownShaderProperties();
    }

    // if (TheRandomMissionManager)
    // {
    //     delete TheRandomMissionManager;
    //     TheRandomMissionManager = 0;
    // }

    if !cmg.landScape.is_null() {
        // delete cmg.landScape;
        cmg.landScape = core::ptr::null_mut();
    }

    core::ptr::write_bytes(&mut cmg as *mut _, 0, 1);
    CM_ClearLevelPatches();

    i = 0;
    while i < NumSubBSP {
        core::ptr::write_bytes(&mut SubBSP[i as usize] as *mut _, 0, 1);
        i += 1;
    }
    NumSubBSP = 0;
    TotalSubModels = 0;
}

pub unsafe fn CM_TotalMapContents() -> c_int {
    return CM_OrOfAllContentsFlagsInMap;
}

/*
==================
CM_ClipHandleToModel
==================
*/
pub unsafe fn CM_ClipHandleToModel(handle: clipHandle_t, clipMap: *mut *mut clipMap_t) -> *mut cmodel_t {
    let mut i: c_int;
    let mut count: c_int;

    if handle < 0 {
        Com_Error(ERR_DROP, b"CM_ClipHandleToModel: bad handle %i\0".as_ptr() as *const c_char, handle);
    }
    if handle < cmg.numSubModels {
        if !clipMap.is_null() {
            *clipMap = &mut cmg;
        }
        return &mut *cmg.cmodels.add(handle as usize);
    }
    if handle == BOX_MODEL_HANDLE {
        if !clipMap.is_null() {
            *clipMap = &mut cmg;
        }
        return &mut box_model;
    }

    count = cmg.numSubModels;
    i = 0;
    while i < NumSubBSP {
        if handle < count + SubBSP[i as usize].numSubModels {
            if !clipMap.is_null() {
                *clipMap = &mut SubBSP[i as usize];
            }
            return &mut *SubBSP[i as usize].cmodels.add((handle - count) as usize);
        }
        count += SubBSP[i as usize].numSubModels;
        i += 1;
    }

    if handle < MAX_SUBMODELS {
        Com_Error(ERR_DROP, b"CM_ClipHandleToModel: bad handle %i < %i < %i\0".as_ptr() as *const c_char,
            cmg.numSubModels, handle, MAX_SUBMODELS);
    }
    Com_Error(ERR_DROP, b"CM_ClipHandleToModel: bad handle %i\0".as_ptr() as *const c_char, handle + MAX_SUBMODELS);

    return core::ptr::null_mut();
}

/*
==================
CM_InlineModel
==================
*/
pub unsafe fn CM_InlineModel(index: c_int) -> clipHandle_t {
    if index < 0 || index >= TotalSubModels {
        Com_Error(ERR_DROP, b"CM_InlineModel: bad number (may need to re-BSP map?)\0".as_ptr() as *const c_char);
    }
    return index;
}

pub unsafe fn CM_NumClusters() -> c_int {
    return cmg.numClusters;
}

pub unsafe fn CM_NumInlineModels() -> c_int {
    return cmg.numSubModels;
}

pub unsafe fn CM_EntityString() -> *mut c_char {
    return cmg.entityString;
}

pub unsafe fn CM_SubBSPEntityString(index: c_int) -> *mut c_char {
    return SubBSP[index as usize].entityString;
}

pub unsafe fn CM_LeafCluster(leafnum: c_int) -> c_int {
    if leafnum < 0 || leafnum >= cmg.numLeafs {
        Com_Error(ERR_DROP, b"CM_LeafCluster: bad number\0".as_ptr() as *const c_char);
    }
    return (*cmg.leafs.add(leafnum as usize)).cluster;
}

pub unsafe fn CM_LeafArea(leafnum: c_int) -> c_int {
    if leafnum < 0 || leafnum >= cmg.numLeafs {
        Com_Error(ERR_DROP, b"CM_LeafArea: bad number\0".as_ptr() as *const c_char);
    }
    return (*cmg.leafs.add(leafnum as usize)).area;
}

//=======================================================================

/*
===================
CM_InitBoxHull

Set up the planes and nodes so that the six floats of a bounding box
can just be stored out and get a proper clipping hull structure.
===================
*/
// Note: CM_InitBoxHull is defined as extern, implemented elsewhere

/*
===================
CM_HeadnodeForBox

To keep everything totally uniform, bounding boxes are turned into small
BSP trees instead of being compared directly.
===================
*/
pub unsafe fn CM_TempBoxModel(mins: *const vec3_t, maxs: *const vec3_t) -> clipHandle_t {
    (*box_planes.add(0)).dist = (*maxs)[0];
    (*box_planes.add(1)).dist = -(*maxs)[0];
    (*box_planes.add(2)).dist = (*mins)[0];
    (*box_planes.add(3)).dist = -(*mins)[0];
    (*box_planes.add(4)).dist = (*maxs)[1];
    (*box_planes.add(5)).dist = -(*maxs)[1];
    (*box_planes.add(6)).dist = (*mins)[1];
    (*box_planes.add(7)).dist = -(*mins)[1];
    (*box_planes.add(8)).dist = (*maxs)[2];
    (*box_planes.add(9)).dist = -(*maxs)[2];
    (*box_planes.add(10)).dist = (*mins)[2];
    (*box_planes.add(11)).dist = -(*mins)[2];

    VectorCopy(mins, (*box_brush).bounds[0].as_mut_ptr());
    VectorCopy(maxs, (*box_brush).bounds[1].as_mut_ptr());

    // FIXME: this is the "correct" way, but not the way JK2 was designed around... fix for further projects
    // box_brush->contents = contents;

    return BOX_MODEL_HANDLE;
}

/*
===================
CM_ModelBounds
===================
*/
pub unsafe fn CM_ModelBounds(cm: &mut clipMap_t, model: clipHandle_t, mins: *mut vec3_t, maxs: *mut vec3_t) {
    let mut cmod: *mut cmodel_t;

    cmod = CM_ClipHandleToModel(model, core::ptr::null_mut());
    VectorCopy((*cmod).mins.as_ptr(), mins);
    VectorCopy((*cmod).maxs.as_ptr(), maxs);
}

/*
===================
CM_RegisterTerrain

Allows physics to examine the terrain data.
===================
*/
#[cfg(not(feature = "bspc"))]
pub unsafe fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *mut c_void {
    let mut terrainId: c_int;
    let mut ls: *mut c_void;

    terrainId = atol(Info_ValueForKey(config, b"terrainId\0".as_ptr() as *const c_char)) as c_int;
    if (terrainId != 0) && !cmg.landScape.is_null() {
        // Already spawned so just return
        ls = cmg.landScape;
        // ls->IncreaseRefCount();
        return ls;
    }
    // Doesn't exist so create and link in
    // cmg.numTerrains++;
    ls = CM_InitTerrain(config, 1, server);

    // Increment for the next instance
    if !cmg.landScape.is_null() {
        Com_Error(ERR_DROP, b"You can't have more than one terrain brush.\0".as_ptr() as *const c_char);
    }
    cmg.landScape = ls;
    return ls;
}

/*
===================
CM_ShutdownTerrain
===================
*/
#[cfg(not(feature = "bspc"))]
pub unsafe fn CM_ShutdownTerrain(terrainId: c_int) {
    let mut landscape: *mut c_void;

    landscape = cmg.landScape;
    if !landscape.is_null() {
        // landscape->DecreaseRefCount();
        // if(landscape->GetRefCount() <= 0)
        // {
        //     delete landscape;
        cmg.landScape = core::ptr::null_mut();
        // }
    }
}

pub unsafe fn CM_LoadSubBSP(name: *const c_char, clientload: qboolean) -> c_int {
    let mut i: c_int;
    let mut checksum: c_int;
    let mut count: c_int;

    count = cmg.numSubModels;
    i = 0;
    while i < NumSubBSP {
        if stricmp(name, SubBSP[i as usize].name.as_ptr()) == 0 {
            return count;
        }
        count += SubBSP[i as usize].numSubModels;
        i += 1;
    }

    if NumSubBSP == MAX_SUB_BSP {
        Com_Error(ERR_DROP, b"CM_LoadSubBSP: too many unique sub BSPs\0".as_ptr() as *const c_char);
    }

    CM_LoadMap_Actual(name, clientload, &mut checksum, &mut SubBSP[NumSubBSP as usize]);
    NumSubBSP += 1;

    return count;
}

pub unsafe fn CM_FindSubBSP(modelIndex: c_int) -> c_int {
    let mut i: c_int;
    let mut count: c_int;

    count = cmg.numSubModels;
    if modelIndex < count {
        return -1;	// belongs to the main bsp
    }

    i = 0;
    while i < NumSubBSP {
        count += SubBSP[i as usize].numSubModels;
        if modelIndex < count {
            return i;
        }
        i += 1;
    }
    return -1;
}

pub unsafe fn CM_GetWorldBounds(mins: *mut vec3_t, maxs: *mut vec3_t) {
    VectorCopy((*cmg.cmodels).mins.as_ptr(), mins);
    VectorCopy((*cmg.cmodels).maxs.as_ptr(), maxs);
}

pub unsafe fn CM_ModelContents_Actual(model: clipHandle_t, cm: *mut clipMap_t) -> c_int {
    let mut cmod: *mut cmodel_t;
    let mut contents: c_int = 0;
    let mut i: c_int;

    let cm_ptr = if cm.is_null() {
        &mut cmg
    } else {
        &mut *cm
    };

    cmod = CM_ClipHandleToModel(model, core::ptr::null_mut());

    // MCG ADDED - return the contents, too
    if (*cmod).leaf.numLeafBrushes != 0 {		// check for brush
        let mut brushNum: c_int;
        i = (*cmod).leaf.firstLeafBrush;
        while i < (*cmod).leaf.firstLeafBrush + (*cmod).leaf.numLeafBrushes {
            brushNum = *cm_ptr.leafbrushes.add(i as usize);
            contents |= (*cm_ptr.brushes.add(brushNum as usize)).contents;
            i += 1;
        }
    }
    if (*cmod).leaf.numLeafSurfaces != 0 {	// if not brush, check for patch
        let mut surfaceNum: c_int;
        i = (*cmod).leaf.firstLeafSurface;
        while i < (*cmod).leaf.firstLeafSurface + (*cmod).leaf.numLeafSurfaces {
            surfaceNum = *cm_ptr.leafsurfaces.add(i as usize);
            if !(*cm_ptr.surfaces.add(surfaceNum as usize)).is_null() {
                // HERNH?  How could we have a null surf within our cmod->leaf.numLeafSurfaces?
                contents |= (**cm_ptr.surfaces.add(surfaceNum as usize)).contents;
            }
            i += 1;
        }
    }
    return contents;
}

pub unsafe fn CM_ModelContents(model: clipHandle_t, subBSPIndex: c_int) -> c_int {
    if subBSPIndex < 0 {
        return CM_ModelContents_Actual(model, core::ptr::null_mut());
    }

    return CM_ModelContents_Actual(model, &mut SubBSP[subBSPIndex as usize]);
}

//support for save/load games
/*
===================
CM_WritePortalState

Writes the portal state to a savegame file
===================
*/
extern "C" {
    pub fn SG_Append(chid: u32, data: *const c_void, length: c_int) -> qboolean;
    pub fn SG_Read(chid: u32, pvAddress: *mut c_void, iLength: c_int, ppvAddressPtr: *mut *mut c_void) -> c_int;
}

pub unsafe fn CM_WritePortalState() {
    SG_Append(
        ((b'P' as u32) << 24) | ((b'R' as u32) << 16) | ((b'T' as u32) << 8) | (b'S' as u32),
        cmg.areaPortals as *const c_void,
        cmg.numAreas * cmg.numAreas * core::mem::size_of::<c_int>() as c_int,
    );
}

/*
===================
CM_ReadPortalState

Reads the portal state from a savegame file
and recalculates the area connections
===================
*/
pub unsafe fn CM_ReadPortalState() {
    SG_Read(
        ((b'P' as u32) << 24) | ((b'R' as u32) << 16) | ((b'T' as u32) << 8) | (b'S' as u32),
        cmg.areaPortals as *mut c_void,
        cmg.numAreas * cmg.numAreas * core::mem::size_of::<c_int>() as c_int,
        core::ptr::null_mut(),
    );
    CM_FloodAreaConnections(&mut cmg);
}

// Local stubs for missing dependencies
#[cfg(feature = "bspc")]
extern "C" {
    pub fn Hunk_ClearHigh();
}
