//! Mechanical port of `codemp/qcommon/cm_load.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use crate::codemp::game::q_shared_h::{byte, qboolean, vec3_t, MAX_QPATH, qfalse, qtrue};
use crate::codemp::qcommon::cm_local_h::{
    cArea_t, cNode_t, cbrush_t, cbrushside_t, cLeaf_t, cmodel_t, cPatch_t, clipMap_t, CCMShader,
    MAX_SUBMODELS, BOX_MODEL_HANDLE, CAPSULE_MODEL_HANDLE,
};
use crate::codemp::qcommon::cm_public_h::clipHandle_t;
use crate::codemp::qcommon::cm_landscape_h::CCMLandScape;
use core::ffi::{c_char, c_int, c_void, c_uint};
use core::ptr::{addr_of_mut, null_mut};

// cmodel.c -- model loading
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #include "cm_local.h"
// #include "cm_landscape.h" //rwwRMG - include
// #include "../RMG/RM_Headers.h" //rwwRMG - include

// #ifdef BSPC
// #include "../bspc/l_qfiles.h"

#[cfg(feature = "bspc")]
mod bspc_only {
    use crate::codemp::game::q_shared_h::cplane_t;
    use core::ffi::c_int;

    pub fn SetPlaneSignbits(out: *mut cplane_t) {
        let mut bits: c_int = 0;
        let mut j: c_int;

        // for fast box on planeside test
        bits = 0;
        j = 0;
        while j < 3 {
            unsafe {
                if (*out).normal[j as usize] < 0.0 {
                    bits |= 1 << j;
                }
            }
            j += 1;
        }
        unsafe {
            (*out).signbits = bits as u32;
        }
    }
}

// #endif //BSPC

// to allow boxes to be treated as brush models, we allocate
// some extra indexes along with those needed by the map
const BOX_BRUSHES: c_int = 1;
const BOX_SIDES: c_int = 6;
const BOX_LEAFS: c_int = 2;
const BOX_PLANES: c_int = 12;

const LL: fn(c_int) -> c_int = LittleLong;

// Extern function declarations
extern "C" {
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;
    fn Com_Memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn Hunk_Alloc(size: c_int, tag: c_int) -> *mut c_void;
    fn Cvar_Get(varname: *const c_char, varvalue: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn LittleLong(l: c_int) -> c_int;
    fn LittleFloat(f: f32) -> f32;
    fn Com_BlockChecksum(buffer: *const c_void, length: c_int) -> c_uint;
    fn PlaneTypeForNormal(normal: *const f32) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn FS_FOpenFileRead(name: *const c_char, f: *mut fileHandle_t, unique_ref: qboolean) -> c_int;
    fn FS_Read(buffer: *mut c_void, len: c_int, h: fileHandle_t);
    fn FS_FCloseFile(f: fileHandle_t);
    fn Z_Malloc(size: c_int, tag: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Com_Memset_detail(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn VectorCopy(in_vec: *const vec3_t, out: *mut vec3_t);
    fn VectorClear(v: *mut vec3_t);
    fn LoadQuakeFile(file: *mut quakefile_t, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn CM_GeneratePatchCollide(width: c_int, height: c_int, points: *const vec3_t) -> *mut cPatch_t_pc;
    fn CM_InitTerrain(config: *const c_char, zero: c_int, server: bool) -> *mut CCMLandScape;
    fn CM_ClearLevelPatches();
    fn CM_LoadShaderText(b: qboolean);
    fn CM_SetupShaderProperties();
    fn CM_ShutdownShaderProperties();
    fn CM_FloodAreaConnections(cm: *mut clipMap_t);
    fn Sys_LowPhysicalMemory() -> qboolean;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    static mut com_dedicated: cvar_t;
}

// Stub types for things we don't need to fully define here
#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

#[repr(C)]
pub struct dshader_t {
    pub shader: [c_char; MAX_QPATH],
    pub contentFlags: c_int,
    pub surfaceFlags: c_int,
}

#[repr(C)]
pub struct dmodel_t {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub firstBrush: c_int,
    pub numBrushes: c_int,
    pub firstSurface: c_int,
    pub numSurfaces: c_int,
}

#[repr(C)]
pub struct dnode_t {
    pub planeNum: c_int,
    pub children: [c_int; 2],
}

#[repr(C)]
pub struct dbrush_t {
    pub firstSide: c_int,
    pub numSides: c_int,
    pub shaderNum: c_int,
}

#[repr(C)]
pub struct dleaf_t {
    pub cluster: c_int,
    pub area: c_int,
    pub mins: [c_int; 3],
    pub maxs: [c_int; 3],
    pub firstLeafBrush: c_int,
    pub numLeafBrushes: c_int,
    pub firstLeafSurface: c_int,
    pub numLeafSurfaces: c_int,
}

#[repr(C)]
pub struct dplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: c_int,
}

#[repr(C)]
pub struct dbrushside_t {
    pub planeNum: c_int,
    pub shaderNum: c_int,
}

#[repr(C)]
pub struct dsurface_t {
    pub surfaceType: c_int,
    pub shaderNum: c_int,
    pub firstVert: c_int,
    pub numVerts: c_int,
    pub firstIndex: c_int,
    pub numIndexes: c_int,
    pub lightmapNum: c_int,
    pub lightmapX: c_int,
    pub lightmapY: c_int,
    pub lightmapWidth: c_int,
    pub lightmapHeight: c_int,
    pub lightmapOrigin: [f32; 3],
    pub lightmapVecs: [[f32; 3]; 2],
    pub patchWidth: c_int,
    pub patchHeight: c_int,
}

#[repr(C)]
pub struct drawVert_t {
    pub xyz: [f32; 3],
    pub st: [f32; 2],
    pub lightmap: [f32; 2],
    pub normal: [f32; 3],
    pub color: [u8; 4],
}

#[repr(C)]
pub struct lump_t {
    pub fileofs: c_int,
    pub filelen: c_int,
}

#[repr(C)]
pub struct dheader_t {
    pub ident: c_int,
    pub version: c_int,
    pub lumps: [lump_t; 18],
}

#[derive(Copy, Clone)]
pub struct fileHandle_t {
    handle: c_int,
}

#[repr(C)]
pub struct quakefile_t;

// Helper struct for patch collide
pub type cPatch_t_pc = c_void;

const ERR_DROP: c_int = 0;
const CVAR_CHEAT: c_int = 1;
const CVAR_ARCHIVE: c_int = 2;
const TAG_BSP_DISKIMAGE: c_int = 3;
const MST_PATCH: c_int = 2;
const LUMP_SHADERS: usize = 0;
const LUMP_LEAFS: usize = 1;
const LUMP_LEAFBRUSHES: usize = 2;
const LUMP_LEAFSURFACES: usize = 3;
const LUMP_PLANES: usize = 4;
const LUMP_BRUSHSIDES: usize = 5;
const LUMP_BRUSHES: usize = 6;
const LUMP_MODELS: usize = 7;
const LUMP_NODES: usize = 8;
const LUMP_ENTITIES: usize = 9;
const LUMP_SURFACES: usize = 10;
const LUMP_DRAWVERTS: usize = 11;
const LUMP_VISIBILITY: usize = 12;
const BSP_VERSION: c_int = 46;
const MAX_SUB_BSP: usize = 16;
const VIS_HEADER: c_int = 8;
const MAX_PATCH_VERTS: c_int = 1024;
const h_high: c_int = 1;
const CONTENTS_BODY: c_int = 32;

// rwwRMG - changed from cm
pub static mut cmg: clipMap_t = unsafe { core::mem::zeroed() };
pub static mut c_pointcontents: c_int = 0;
pub static mut c_traces: c_int = 0;
pub static mut c_brush_traces: c_int = 0;
pub static mut c_patch_traces: c_int = 0;

pub static mut cmod_base: *mut byte = null_mut();

#[cfg(not(feature = "bspc"))]
pub static mut cm_noAreas: *mut cvar_t = null_mut();
#[cfg(not(feature = "bspc"))]
pub static mut cm_noCurves: *mut cvar_t = null_mut();
#[cfg(not(feature = "bspc"))]
pub static mut cm_playerCurveClip: *mut cvar_t = null_mut();

pub static mut box_model: cmodel_t = unsafe { core::mem::zeroed() };
pub static mut box_planes: *mut cplane_t = null_mut();
pub static mut box_brush: *mut cbrush_t = null_mut();

// rwwRMG - added:
pub static mut SubBSP: [clipMap_t; MAX_SUB_BSP] = unsafe { [core::mem::zeroed(); MAX_SUB_BSP] };
pub static mut NumSubBSP: c_int = 0;
pub static mut TotalSubModels: c_int = 0;

// ===============================================================================
//
// MAP LOADING
//
// ===============================================================================

//
// =================
// CMod_LoadShaders
// =================
//
pub fn CMod_LoadShaders(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut in_: *mut dshader_t =
            (cmod_base.add((*l).fileofs as usize)) as *mut dshader_t;
        let mut i: c_int;
        let mut count: c_int;
        let mut out: *mut CCMShader;

        if (*l).filelen % (core::mem::size_of::<dshader_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"CMod_LoadShaders: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<dshader_t>() as c_int);

        if count < 1 {
            Com_Error(ERR_DROP, c"Map with no shaders\0".as_ptr());
        }
        cm.shaders = Hunk_Alloc((1 + count) * (core::mem::size_of::<CCMShader>() as c_int), h_high)
            as *mut CCMShader;
        cm.numShaders = count;

        out = cm.shaders;
        i = 0;
        while i < count {
            Q_strncpyz(
                (*out).shader.as_mut_ptr(),
                (*in_).shader.as_ptr(),
                MAX_QPATH,
            );
            (*out).surfaceFlags = LittleLong((*in_).surfaceFlags);
            (*out).contentFlags = LittleLong((*in_).contentFlags);
            i += 1;
            in_ = in_.offset(1);
            out = out.offset(1);
        }
    }
}

//
// =================
// CMod_LoadSubmodels
// =================
//
pub fn CMod_LoadSubmodels(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut in_: *mut dmodel_t = (cmod_base.add((*l).fileofs as usize)) as *mut dmodel_t;
        let mut out: *mut cmodel_t;
        let mut i: c_int;
        let mut j: c_int;
        let mut count: c_int;
        let mut indexes: *mut c_int;

        if (*l).filelen % (core::mem::size_of::<dmodel_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"CMod_LoadSubmodels: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<dmodel_t>() as c_int);

        if count < 1 {
            Com_Error(ERR_DROP, c"Map with no models\0".as_ptr());
        }
        cm.cmodels =
            Hunk_Alloc(count * (core::mem::size_of::<cmodel_t>() as c_int), h_high)
                as *mut cmodel_t;
        cm.numSubModels = count;

        if count > MAX_SUBMODELS as c_int {
            Com_Error(ERR_DROP, c"MAX_SUBMODELS exceeded\0".as_ptr());
        }

        i = 0;
        while i < count {
            out = &mut *cm.cmodels.add(i as usize);

            j = 0;
            while j < 3 {
                // spread the mins / maxs by a pixel
                (*out).mins[j as usize] = LittleFloat((*in_).mins[j as usize]) - 1.0;
                (*out).maxs[j as usize] = LittleFloat((*in_).maxs[j as usize]) + 1.0;
                j += 1;
            }

            // rwwRMG - sof2 doesn't have to add this &cm == &cmg check.
            // Are they getting leaf data elsewhere? (the reason this needs to be done is
            // in sub bsp instances the first brush model isn't necessary a world model and might be
            // real architecture)
            if i == 0 && (cm as *const clipMap_t) == (addr_of_mut!(cmg) as *const clipMap_t) {
                (*out).firstNode = 0;
                in_ = in_.offset(1);
                i += 1;
                continue;
                // world model doesn't need other info
            }

            // make a "leaf" just to hold the model's brushes and surfaces
            (*out).firstNode = -1;

            // make a "leaf" just to hold the model's brushes and surfaces
            (*out).leaf.numLeafBrushes = LittleLong((*in_).numBrushes);
            indexes = Hunk_Alloc((*out).leaf.numLeafBrushes * 4, h_high) as *mut c_int;
            (*out).leaf.firstLeafBrush = indexes.offset_from(cm.leafbrushes) as c_int;
            j = 0;
            while j < (*out).leaf.numLeafBrushes {
                *indexes.offset(j as isize) = LittleLong((*in_).firstBrush) + j;
                j += 1;
            }

            (*out).leaf.numLeafSurfaces = LittleLong((*in_).numSurfaces);
            indexes = Hunk_Alloc((*out).leaf.numLeafSurfaces * 4, h_high) as *mut c_int;
            (*out).leaf.firstLeafSurface = indexes.offset_from(cm.leafsurfaces) as c_int;
            j = 0;
            while j < (*out).leaf.numLeafSurfaces {
                *indexes.offset(j as isize) = LittleLong((*in_).firstSurface) + j;
                j += 1;
            }

            in_ = in_.offset(1);
            i += 1;
        }
    }
}

//
// =================
// CMod_LoadNodes
//
// =================
//
pub fn CMod_LoadNodes(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut in_: *mut dnode_t = (cmod_base.add((*l).fileofs as usize)) as *mut dnode_t;
        let mut child: c_int;
        let mut out: *mut cNode_t;
        let mut i: c_int;
        let mut j: c_int;
        let mut count: c_int;

        if (*l).filelen % (core::mem::size_of::<dnode_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<dnode_t>() as c_int);

        if count < 1 {
            Com_Error(ERR_DROP, c"Map has no nodes\0".as_ptr());
        }
        cm.nodes = Hunk_Alloc(count * (core::mem::size_of::<cNode_t>() as c_int), h_high)
            as *mut cNode_t;
        cm.numNodes = count;

        out = cm.nodes;

        i = 0;
        while i < count {
            #[cfg(not(feature = "xbox"))]
            {
                (*out).plane = cm.planes.add(LittleLong((*in_).planeNum) as usize);
            }
            j = 0;
            while j < 2 {
                child = LittleLong((*in_).children[j as usize]);
                (*out).children[j as usize] = child;
                j += 1;
            }
            i += 1;
            out = out.offset(1);
            in_ = in_.offset(1);
        }
    }
}

//
// =================
// CM_BoundBrush
//
// =================
//
pub fn CM_BoundBrush(b: *mut cbrush_t) {
    unsafe {
        (*b).bounds[0][0] = -(*(*b).sides.offset(0)).plane.as_ref().unwrap().dist;
        (*b).bounds[1][0] = (*(*b).sides.offset(1)).plane.as_ref().unwrap().dist;

        (*b).bounds[0][1] = -(*(*b).sides.offset(2)).plane.as_ref().unwrap().dist;
        (*b).bounds[1][1] = (*(*b).sides.offset(3)).plane.as_ref().unwrap().dist;

        (*b).bounds[0][2] = -(*(*b).sides.offset(4)).plane.as_ref().unwrap().dist;
        (*b).bounds[1][2] = (*(*b).sides.offset(5)).plane.as_ref().unwrap().dist;
    }
}

//
// =================
// CMod_LoadBrushes
//
// =================
//
pub fn CMod_LoadBrushes(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut in_: *mut dbrush_t = (cmod_base.add((*l).fileofs as usize)) as *mut dbrush_t;
        let mut out: *mut cbrush_t;
        let mut i: c_int;
        let mut count: c_int;

        if (*l).filelen % (core::mem::size_of::<dbrush_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<dbrush_t>() as c_int);

        cm.brushes = Hunk_Alloc(
            (BOX_BRUSHES + count) * (core::mem::size_of::<cbrush_t>() as c_int),
            h_high,
        ) as *mut cbrush_t;
        cm.numBrushes = count;

        out = cm.brushes;

        i = 0;
        while i < count {
            (*out).sides = cm.brushsides.add(LittleLong((*in_).firstSide) as usize);
            (*out).numsides = LittleLong((*in_).numSides) as u16;

            (*out).shaderNum = LittleLong((*in_).shaderNum);
            if (*out).shaderNum < 0 || (*out).shaderNum >= cm.numShaders {
                Com_Error(
                    ERR_DROP,
                    c"CMod_LoadBrushes: bad shaderNum: %i\0".as_ptr(),
                    (*out).shaderNum,
                );
            }
            (*out).contents = cm.shaders.add((*out).shaderNum as usize).as_ref().unwrap().contentFlags;

            // Landscapes are set up afterwards in the entity spawning
            // out->landscape = NULL;	//the memory was cleared already by hunk_alloc
            // out->checkcount=0;

            CM_BoundBrush(out);

            i += 1;
            out = out.offset(1);
            in_ = in_.offset(1);
        }
    }
}

//
// =================
// CMod_LoadLeafs
// =================
//
pub fn CMod_LoadLeafs(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut i: c_int;
        let mut out: *mut cLeaf_t;
        let mut in_: *mut dleaf_t = (cmod_base.add((*l).fileofs as usize)) as *mut dleaf_t;
        let mut count: c_int;

        if (*l).filelen % (core::mem::size_of::<dleaf_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<dleaf_t>() as c_int);

        if count < 1 {
            Com_Error(ERR_DROP, c"Map with no leafs\0".as_ptr());
        }

        cm.leafs = Hunk_Alloc((BOX_LEAFS + count) * (core::mem::size_of::<cLeaf_t>() as c_int), h_high)
            as *mut cLeaf_t;
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

            i += 1;
            in_ = in_.offset(1);
            out = out.offset(1);
        }

        cm.areas = Hunk_Alloc(cm.numAreas * (core::mem::size_of::<cArea_t>() as c_int), h_high)
            as *mut cArea_t;
        cm.areaPortals = Hunk_Alloc(
            cm.numAreas * cm.numAreas * (core::mem::size_of::<c_int>() as c_int),
            h_high,
        ) as *mut c_int;
    }
}

//
// =================
// CMod_LoadPlanes
// =================
//
pub fn CMod_LoadPlanes(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut i: c_int;
        let mut j: c_int;
        let mut out: *mut cplane_t;
        let mut in_: *mut dplane_t = (cmod_base.add((*l).fileofs as usize)) as *mut dplane_t;
        let mut count: c_int;
        let mut bits: c_int;

        if (*l).filelen % (core::mem::size_of::<dplane_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<dplane_t>() as c_int);

        if count < 1 {
            Com_Error(ERR_DROP, c"Map with no planes\0".as_ptr());
        }
        cm.planes = Hunk_Alloc(
            (BOX_PLANES + count) * (core::mem::size_of::<cplane_t>() as c_int),
            h_high,
        ) as *mut cplane_t;
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
            (*out).type_ = PlaneTypeForNormal((*out).normal.as_ptr());
            (*out).signbits = bits as u32;

            i += 1;
            in_ = in_.offset(1);
            out = out.offset(1);
        }
    }
}

//
// =================
// CMod_LoadLeafBrushes
// =================
//
pub fn CMod_LoadLeafBrushes(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut i: c_int;
        let mut out: *mut c_int;
        let mut in_: *mut c_int = (cmod_base.add((*l).fileofs as usize)) as *mut c_int;
        let mut count: c_int;

        if (*l).filelen % (core::mem::size_of::<c_int>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<c_int>() as c_int);

        cm.leafbrushes =
            Hunk_Alloc((count + BOX_BRUSHES) * (core::mem::size_of::<c_int>() as c_int), h_high)
                as *mut c_int;
        cm.numLeafBrushes = count;

        out = cm.leafbrushes;

        i = 0;
        while i < count {
            *out = LittleLong(*in_);
            i += 1;
            in_ = in_.offset(1);
            out = out.offset(1);
        }
    }
}

//
// =================
// CMod_LoadLeafSurfaces
// =================
//
pub fn CMod_LoadLeafSurfaces(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut i: c_int;
        let mut out: *mut c_int;
        let mut in_: *mut c_int = (cmod_base.add((*l).fileofs as usize)) as *mut c_int;
        let mut count: c_int;

        if (*l).filelen % (core::mem::size_of::<c_int>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<c_int>() as c_int);

        cm.leafsurfaces = Hunk_Alloc(count * (core::mem::size_of::<c_int>() as c_int), h_high)
            as *mut c_int;
        cm.numLeafSurfaces = count;

        out = cm.leafsurfaces;

        i = 0;
        while i < count {
            *out = LittleLong(*in_);
            i += 1;
            in_ = in_.offset(1);
            out = out.offset(1);
        }
    }
}

//
// =================
// CMod_LoadBrushSides
// =================
//
pub fn CMod_LoadBrushSides(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut i: c_int;
        let mut out: *mut cbrushside_t;
        let mut in_: *mut dbrushside_t = (cmod_base.add((*l).fileofs as usize)) as *mut dbrushside_t;
        let mut count: c_int;
        let mut num: c_int;

        if (*l).filelen % (core::mem::size_of::<dbrushside_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        count = (*l).filelen / (core::mem::size_of::<dbrushside_t>() as c_int);

        cm.brushsides =
            Hunk_Alloc((BOX_SIDES + count) * (core::mem::size_of::<cbrushside_t>() as c_int), h_high)
                as *mut cbrushside_t;
        cm.numBrushSides = count;

        out = cm.brushsides;

        i = 0;
        while i < count {
            num = LittleLong((*in_).planeNum);
            #[cfg(not(feature = "xbox"))]
            {
                (*out).plane = &mut *cm.planes.add(num as usize);
            }
            (*out).shaderNum = LittleLong((*in_).shaderNum);
            if (*out).shaderNum < 0 || (*out).shaderNum >= cm.numShaders {
                Com_Error(
                    ERR_DROP,
                    c"CMod_LoadBrushSides: bad shaderNum: %i\0".as_ptr(),
                    (*out).shaderNum,
                );
            }
            i += 1;
            in_ = in_.offset(1);
            out = out.offset(1);
        }
    }
}

//
// =================
// CMod_LoadEntityString
// =================
//
pub fn CMod_LoadEntityString(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        cm.entityString = Hunk_Alloc((*l).filelen, h_high) as *mut c_char;
        cm.numEntityChars = (*l).filelen;
        Com_Memcpy(
            cm.entityString as *mut c_void,
            cmod_base.add((*l).fileofs as usize) as *const c_void,
            (*l).filelen as usize,
        );
    }
}

//
// =================
// CMod_LoadVisibility
// =================
//
pub fn CMod_LoadVisibility(l: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let len: c_int = (*l).filelen;
        let buf: *mut byte;

        if len == 0 {
            cm.clusterBytes = (cm.numClusters + 31) & !31;
            cm.visibility = Hunk_Alloc(cm.clusterBytes, h_high) as *mut byte;
            Com_Memset(cm.visibility as *mut c_void, 255, cm.clusterBytes as usize);
            return;
        }
        buf = cmod_base.add((*l).fileofs as usize);

        cm.vised = qtrue;
        cm.visibility = Hunk_Alloc(len, h_high) as *mut byte;
        cm.numClusters = LittleLong(*(buf.add(0) as *const c_int));
        cm.clusterBytes = LittleLong(*(buf.add(4) as *const c_int));
        Com_Memcpy(
            cm.visibility as *mut c_void,
            buf.add(VIS_HEADER as usize) as *const c_void,
            (len - VIS_HEADER) as usize,
        );
    }
}

// ==================================================================

//
// =================
// CMod_LoadPatches
// =================
//
pub fn CMod_LoadPatches(surfs: *const lump_t, verts: *const lump_t, cm: &mut clipMap_t) {
    unsafe {
        let mut dv: *mut drawVert_t;
        let mut dv_p: *mut drawVert_t;
        let mut in_: *mut dsurface_t = (cmod_base.add((*surfs).fileofs as usize)) as *mut dsurface_t;
        let mut count: c_int;
        let mut i: c_int;
        let mut j: c_int;
        let mut c: c_int;
        let mut patch: *mut cPatch_t;
        let mut points: [vec3_t; MAX_PATCH_VERTS as usize];
        let mut width: c_int;
        let mut height: c_int;
        let mut shaderNum: c_int;

        if (*surfs).filelen % (core::mem::size_of::<dsurface_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }
        cm.numSurfaces = (*surfs).filelen / (core::mem::size_of::<dsurface_t>() as c_int);
        count = cm.numSurfaces;
        cm.surfaces = Hunk_Alloc(
            cm.numSurfaces * (core::mem::size_of::<*mut cPatch_t>() as c_int),
            h_high,
        ) as *mut *mut cPatch_t;

        dv = (cmod_base.add((*verts).fileofs as usize)) as *mut drawVert_t;
        if (*verts).filelen % (core::mem::size_of::<drawVert_t>() as c_int) != 0 {
            Com_Error(ERR_DROP, c"MOD_LoadBmodel: funny lump size\0".as_ptr());
        }

        // scan through all the surfaces, but only load patches,
        // not planar faces
        i = 0;
        while i < count {
            if LittleLong((*in_).surfaceType) != MST_PATCH {
                i += 1;
                in_ = in_.offset(1);
                continue;
                // ignore other surfaces
            }
            // FIXME: check for non-colliding patches

            *cm.surfaces.add(i as usize) = patch =
                Hunk_Alloc(core::mem::size_of::<cPatch_t>() as c_int, h_high) as *mut cPatch_t;

            // load the full drawverts onto the stack
            width = LittleLong((*in_).patchWidth);
            height = LittleLong((*in_).patchHeight);
            c = width * height;
            if c > MAX_PATCH_VERTS {
                Com_Error(ERR_DROP, c"ParseMesh: MAX_PATCH_VERTS\0".as_ptr());
            }

            dv_p = dv.add(LittleLong((*in_).firstVert) as usize);
            j = 0;
            while j < c {
                points[j as usize][0] = LittleFloat((*dv_p).xyz[0]);
                points[j as usize][1] = LittleFloat((*dv_p).xyz[1]);
                points[j as usize][2] = LittleFloat((*dv_p).xyz[2]);
                j += 1;
                dv_p = dv_p.offset(1);
            }

            shaderNum = LittleLong((*in_).shaderNum);
            (*patch).contents = cm.shaders.add(shaderNum as usize).as_ref().unwrap().contentFlags;
            (*patch).surfaceFlags = cm.shaders.add(shaderNum as usize).as_ref().unwrap().surfaceFlags;

            // create the internal facet structure
            (*patch).pc = CM_GeneratePatchCollide(width, height, points.as_ptr()) as *mut _;

            i += 1;
            in_ = in_.offset(1);
        }
    }
}

// ==================================================================

pub fn CM_LumpChecksum(lump: *mut lump_t) -> c_uint {
    unsafe {
        LittleLong(Com_BlockChecksum(
            cmod_base.add((*lump).fileofs as usize) as *const c_void,
            (*lump).filelen,
        ) as c_int) as c_uint
    }
}

pub fn CM_Checksum(header: *mut dheader_t) -> c_uint {
    unsafe {
        let mut checksums: [c_uint; 16] = [0; 16];
        checksums[0] = CM_LumpChecksum(&mut (*header).lumps[LUMP_SHADERS]);
        checksums[1] = CM_LumpChecksum(&mut (*header).lumps[LUMP_LEAFS]);
        checksums[2] = CM_LumpChecksum(&mut (*header).lumps[LUMP_LEAFBRUSHES]);
        checksums[3] = CM_LumpChecksum(&mut (*header).lumps[LUMP_LEAFSURFACES]);
        checksums[4] = CM_LumpChecksum(&mut (*header).lumps[LUMP_PLANES]);
        checksums[5] = CM_LumpChecksum(&mut (*header).lumps[LUMP_BRUSHSIDES]);
        checksums[6] = CM_LumpChecksum(&mut (*header).lumps[LUMP_BRUSHES]);
        checksums[7] = CM_LumpChecksum(&mut (*header).lumps[LUMP_MODELS]);
        checksums[8] = CM_LumpChecksum(&mut (*header).lumps[LUMP_NODES]);
        checksums[9] = CM_LumpChecksum(&mut (*header).lumps[LUMP_SURFACES]);
        checksums[10] = CM_LumpChecksum(&mut (*header).lumps[LUMP_DRAWVERTS]);

        LittleLong(Com_BlockChecksum(
            checksums.as_ptr() as *const c_void,
            11 * 4,
        ) as c_int) as c_uint
    }
}

//
// ==================
// CM_LoadMap
//
// Loads in the map and all submodels
// ==================
//
pub static mut gpvCachedMapDiskImage: *mut c_void = null_mut();
pub static mut gsCachedMapDiskImage: [c_char; MAX_QPATH] = unsafe { [0; MAX_QPATH] };
pub static mut gbUsingCachedMapDataRightNow: qboolean = qfalse; // if true, signifies that you can't delete this at the moment!! (used during z_malloc()-fail recovery attempt)

// called in response to a "devmapbsp blah" or "devmapall blah" command, do NOT use inside CM_Load unless you pass in qtrue
//
// new bool return used to see if anything was freed, used during z_malloc failure re-try
//
pub fn CM_DeleteCachedMap(bGuaranteedOkToDelete: qboolean) -> qboolean {
    unsafe {
        let mut bActuallyFreedSomething: qboolean = qfalse;

        if bGuaranteedOkToDelete != qfalse || gbUsingCachedMapDataRightNow == qfalse {
            // dump cached disk image...
            //
            if !gpvCachedMapDiskImage.is_null() {
                Z_Free(gpvCachedMapDiskImage);
                gpvCachedMapDiskImage = null_mut();

                bActuallyFreedSomething = qtrue;
            }
            gsCachedMapDiskImage[0] = 0;

            // force map loader to ignore cached internal BSP structures for next level CM_LoadMap() call...
            //
            cmg.name[0] = 0;
        }

        bActuallyFreedSomething
    }
}

pub unsafe fn CM_LoadMap_Actual(
    name: *const c_char,
    clientload: qboolean,
    checksum: *mut c_int,
    cm: &mut clipMap_t,
) {
    //rwwRMG - function needs heavy modification
    let mut buf: *mut c_int;
    let mut i: c_int;
    let mut header: dheader_t;
    static mut last_checksum: c_uint = 0;
    let mut origName: [c_char; 260] = [0; 260]; // MAX_OSPATH
    let mut newBuff: *mut c_void = null_mut();

    if name.is_null() || *name == 0 {
        Com_Error(ERR_DROP, c"CM_LoadMap: NULL name\0".as_ptr());
    }

    #[cfg(not(feature = "bspc"))]
    {
        cm_noAreas =
            Cvar_Get(c"cm_noAreas\0".as_ptr(), c"0\0".as_ptr(), CVAR_CHEAT);
        cm_noCurves =
            Cvar_Get(c"cm_noCurves\0".as_ptr(), c"0\0".as_ptr(), CVAR_CHEAT);
        cm_playerCurveClip = Cvar_Get(
            c"cm_playerCurveClip\0".as_ptr(),
            c"1\0".as_ptr(),
            CVAR_ARCHIVE | CVAR_CHEAT,
        );
    }
    Com_DPrintf(c"CM_LoadMap( %s, %i )\n\0".as_ptr(), name, clientload);

    let mut i_tmp: c_int = 0;
    let mut name_tmp: *const c_char = name;
    let mut match_found: qboolean = qtrue;
    while i_tmp < MAX_QPATH && *cm.name.as_ptr().add(i_tmp as usize) != 0
        && *name_tmp != 0
    {
        if *cm.name.as_ptr().add(i_tmp as usize) != *name_tmp {
            match_found = qfalse;
            break;
        }
        i_tmp += 1;
        name_tmp = name_tmp.add(1);
    }
    if match_found
        && *cm.name.as_ptr().add(i_tmp as usize)
            == *name_tmp
        && clientload != qfalse
    {
        *checksum = last_checksum as c_int;
        return;
    }

    // strcpy(origName, name);
    let mut src = name;
    let mut dst = origName.as_mut_ptr();
    while *src != 0 && (dst.offset_from(origName.as_ptr()) as c_int) < 260 {
        *dst = *src;
        src = src.add(1);
        dst = dst.add(1);
    }
    *dst = 0;

    if (cm as *const clipMap_t) == (addr_of_mut!(cmg) as *const clipMap_t) {
        // free old stuff
        CM_ClearMap();
        CM_ClearLevelPatches();
    }

    // free old stuff
    Com_Memset(cm as *mut c_void, 0, core::mem::size_of::<clipMap_t>());

    if *name == 0 {
        cm.numLeafs = 1;
        cm.numClusters = 1;
        cm.numAreas = 1;
        cm.cmodels =
            Hunk_Alloc(core::mem::size_of::<cmodel_t>() as c_int, h_high) as *mut cmodel_t;
        *checksum = 0;
        return;
    }

    //
    // load the file
    //
    // rww - Doesn't this sort of defeat the purpose? We're clearing it even if the map is the same as the last one!
    // Not touching it though in case I'm just overlooking something.
    if !gpvCachedMapDiskImage.is_null()
        && (cm as *const clipMap_t) == (addr_of_mut!(cmg) as *const clipMap_t)
    {
        // MP code: this'll only be NZ if we got an ERR_DROP during last map load,
        // so it's really just a safety measure.
        Z_Free(gpvCachedMapDiskImage);
        gpvCachedMapDiskImage = null_mut();
    }

    #[cfg(not(feature = "bspc"))]
    {
        //
        // load the file into a buffer that we either discard as usual at the bottom, or if we've got enough memory
        // then keep it long enough to save the renderer re-loading it (if not dedicated server),
        // then discard it after that...
        //
        buf = null_mut();
        let mut h: fileHandle_t = fileHandle_t { handle: 0 };
        let iBSPLen: c_int = FS_FOpenFileRead(name, &mut h, qfalse);
        if !h.is_null() {
            newBuff = Z_Malloc(iBSPLen, TAG_BSP_DISKIMAGE);
            FS_Read(newBuff, iBSPLen, h);
            FS_FCloseFile(h);

            buf = newBuff as *mut c_int; // so the rest of the code works as normal
            if (cm as *const clipMap_t) == (addr_of_mut!(cmg) as *const clipMap_t) {
                gpvCachedMapDiskImage = newBuff;
                newBuff = null_mut();
            }

            // carry on as before...
            //
        }

        let buf_iBSPLen = if buf.is_null() { 0 } else { iBSPLen };
        let buf_for_checksum = if buf.is_null() {
            null_mut() as *const c_void
        } else {
            buf as *const c_void
        };

        if buf.is_null() {
            Com_Error(ERR_DROP, c"Couldn't load %s\0".as_ptr(), name);
        }

        last_checksum = LittleLong(Com_BlockChecksum(buf_for_checksum, buf_iBSPLen)) as c_uint;
        *checksum = last_checksum as c_int;
    }

    #[cfg(feature = "bspc")]
    {
        let iBSPLen: c_int = LoadQuakeFile(name as *mut quakefile_t, &mut (buf as *mut *mut c_void));

        if buf.is_null() {
            Com_Error(ERR_DROP, c"Couldn't load %s\0".as_ptr(), name);
        }

        last_checksum = LittleLong(Com_BlockChecksum(buf as *const c_void, iBSPLen)) as c_uint;
        *checksum = last_checksum as c_int;
    }

    header = *(buf as *mut dheader_t);
    i = 0;
    while i < ((core::mem::size_of::<dheader_t>() / 4) as c_int) {
        *(((&mut header) as *mut dheader_t) as *mut c_int).add(i as usize) =
            LittleLong(*((buf as *mut c_int).add(i as usize)));
        i += 1;
    }

    if header.version != BSP_VERSION {
        Z_Free(gpvCachedMapDiskImage);
        gpvCachedMapDiskImage = null_mut();

        Com_Error(
            ERR_DROP,
            c"CM_LoadMap: %s has wrong version number (%i should be %i)\0".as_ptr(),
            name,
            header.version,
            BSP_VERSION,
        );
    }

    cmod_base = buf as *mut byte;

    // load into heap
    CMod_LoadShaders(&header.lumps[LUMP_SHADERS], cm);
    CMod_LoadLeafs(&header.lumps[LUMP_LEAFS], cm);
    CMod_LoadLeafBrushes(&header.lumps[LUMP_LEAFBRUSHES], cm);
    CMod_LoadLeafSurfaces(&header.lumps[LUMP_LEAFSURFACES], cm);
    CMod_LoadPlanes(&header.lumps[LUMP_PLANES], cm);
    CMod_LoadBrushSides(&header.lumps[LUMP_BRUSHSIDES], cm);
    CMod_LoadBrushes(&header.lumps[LUMP_BRUSHES], cm);
    CMod_LoadSubmodels(&header.lumps[LUMP_MODELS], cm);
    CMod_LoadNodes(&header.lumps[LUMP_NODES], cm);
    CMod_LoadEntityString(&header.lumps[LUMP_ENTITIES], cm);
    CMod_LoadVisibility(&header.lumps[LUMP_VISIBILITY], cm);
    CMod_LoadPatches(&header.lumps[LUMP_SURFACES], &header.lumps[LUMP_DRAWVERTS], cm);

    TotalSubModels += cm.numSubModels;

    if (cm as *const clipMap_t) == (addr_of_mut!(cmg) as *const clipMap_t) {
        // Load in the shader text - return instantly if already loaded
        #[cfg(not(feature = "bspc"))]
        {
            CM_LoadShaderText(qfalse);
        }
        CM_InitBoxHull();
        #[cfg(not(feature = "bspc"))]
        {
            CM_SetupShaderProperties();
        }
    }

    #[cfg(not(feature = "bspc"))]
    {
        //
        // if we've got enough memory, and it's not a dedicated-server, then keep the loaded map binary around
        // for the renderer to chew on... (but not if this gets ported to a big-endian machine, because some of the
        // map data will have been Little-Long'd, but some hasn't).
        //
        if Sys_LowPhysicalMemory() != qfalse
            || (*com_dedicated).integer != 0
        //		|| we're on a big-endian machine
        {
            Z_Free(gpvCachedMapDiskImage);
            gpvCachedMapDiskImage = null_mut();
        } else {
            // ... do nothing, and let the renderer free it after it's finished playing with it...
            //
        }
    }

    #[cfg(feature = "bspc")]
    {
        FS_FreeFile(buf as *mut c_void);
    }

    CM_FloodAreaConnections(cm);

    // allow this to be cached if it is loaded by the server
    if clientload == qfalse {
        Q_strncpyz(
            cm.name.as_mut_ptr(),
            origName.as_ptr(),
            core::mem::size_of_val(&cm.name) as c_int,
        );
    }
}

// need a wrapper function around this because of multiple returns, need to ensure bool is correct...
//
pub fn CM_LoadMap(name: *const c_char, clientload: qboolean, checksum: *mut c_int) {
    unsafe {
        gbUsingCachedMapDataRightNow = qtrue; // !!!!!!!!!!!!!!!!!!

        CM_LoadMap_Actual(name, clientload, checksum, &mut cmg);

        gbUsingCachedMapDataRightNow = qfalse; // !!!!!!!!!!!!!!!!!!
    }
}

//
// ==================
// CM_ClearMap
// ==================
//
pub fn CM_ClearMap() {
    unsafe {
        let mut i: c_int;

        #[cfg(not(feature = "bspc"))]
        {
            CM_ShutdownShaderProperties();
            //	MAT_Shutdown();
        }

        if !cmg.landScape.is_null() {
            // Note: This is C++ code doing delete, but we're using raw pointers
            // In Rust, we'd need to properly deallocate this
            // delete TheRandomMissionManager;
            // TheRandomMissionManager = 0;
        }

        if !cmg.landScape.is_null() {
            // delete cmg.landScape;
            cmg.landScape = null_mut();
        }

        Com_Memset(&mut cmg as *mut clipMap_t as *mut c_void, 0, core::mem::size_of::<clipMap_t>());
        CM_ClearLevelPatches();

        i = 0;
        while i < NumSubBSP {
            core::ptr::write_bytes(
                SubBSP.as_mut_ptr().add(i as usize),
                0,
                core::mem::size_of::<clipMap_t>(),
            );
            i += 1;
        }
        NumSubBSP = 0;
        TotalSubModels = 0;
    }
}

//
// ==================
// CM_ClipHandleToModel
// ==================
//
pub fn CM_ClipHandleToModel(
    handle: clipHandle_t,
    clipMap: *mut *mut clipMap_t,
) -> *mut cmodel_t {
    unsafe {
        let mut i: c_int;
        let mut count: c_int;

        if handle < 0 {
            Com_Error(
                ERR_DROP,
                c"CM_ClipHandleToModel: bad handle %i\0".as_ptr(),
                handle,
            );
        }
        if handle < cmg.numSubModels {
            if !clipMap.is_null() {
                *clipMap = addr_of_mut!(cmg);
            }
            return &mut *cmg.cmodels.add(handle as usize);
        }
        if handle == BOX_MODEL_HANDLE {
            if !clipMap.is_null() {
                *clipMap = addr_of_mut!(cmg);
            }
            return addr_of_mut!(box_model);
        }

        count = cmg.numSubModels;
        i = 0;
        while i < NumSubBSP {
            if handle < count + SubBSP[i as usize].numSubModels {
                if !clipMap.is_null() {
                    *clipMap = &mut SubBSP[i as usize];
                }
                return &mut *SubBSP[i as usize]
                    .cmodels
                    .add((handle - count) as usize);
            }
            count += SubBSP[i as usize].numSubModels;
            i += 1;
        }

        if handle < MAX_SUBMODELS as c_int {
            Com_Error(
                ERR_DROP,
                c"CM_ClipHandleToModel: bad handle %i < %i < %i\0".as_ptr(),
                cmg.numSubModels,
                handle,
                MAX_SUBMODELS,
            );
        }
        Com_Error(
            ERR_DROP,
            c"CM_ClipHandleToModel: bad handle %i\0".as_ptr(),
            handle + MAX_SUBMODELS as c_int,
        );

        null_mut()
    }
}

//
// ==================
// CM_InlineModel
// ==================
//
pub fn CM_InlineModel(index: c_int) -> clipHandle_t {
    unsafe {
        if index < 0 || index >= TotalSubModels {
            Com_Error(
                ERR_DROP,
                c"CM_InlineModel: bad number: %d > %d\0".as_ptr(),
                index,
                TotalSubModels,
            );
        }
        index
    }
}

pub fn CM_NumClusters() -> c_int {
    unsafe { cmg.numClusters }
}

pub fn CM_NumInlineModels() -> c_int {
    unsafe { cmg.numSubModels }
}

pub fn CM_EntityString() -> *mut c_char {
    unsafe { cmg.entityString }
}

pub fn CM_SubBSPEntityString(index: c_int) -> *mut c_char {
    unsafe { SubBSP[index as usize].entityString }
}

pub fn CM_LeafCluster(leafnum: c_int) -> c_int {
    unsafe {
        if leafnum < 0 || leafnum >= cmg.numLeafs {
            Com_Error(ERR_DROP, c"CM_LeafCluster: bad number\0".as_ptr());
        }
        cmg.leafs.as_ref().unwrap().add(leafnum as usize).cluster
    }
}

pub fn CM_LeafArea(leafnum: c_int) -> c_int {
    unsafe {
        if leafnum < 0 || leafnum >= cmg.numLeafs {
            Com_Error(ERR_DROP, c"CM_LeafArea: bad number\0".as_ptr());
        }
        cmg.leafs.as_ref().unwrap().add(leafnum as usize).area
    }
}

// =======================================================================

//
// ===================
// CM_InitBoxHull
//
// Set up the planes and nodes so that the six floats of a bounding box
// can just be stored out and get a proper clipping hull structure.
// ===================
//
pub fn CM_InitBoxHull() {
    unsafe {
        let mut i: c_int;
        let mut side: c_int;
        let mut p: *mut cplane_t;
        let mut s: *mut cbrushside_t;

        box_planes = &mut *cmg.planes.add(cmg.numPlanes as usize);

        box_brush = &mut *cmg.brushes.add(cmg.numBrushes as usize);
        (*box_brush).numsides = 6;
        (*box_brush).sides = cmg.brushsides.add(cmg.numBrushSides as usize);
        (*box_brush).contents = CONTENTS_BODY;

        box_model.firstNode = -1;
        box_model.leaf.numLeafBrushes = 1;
        //	box_model.leaf.firstLeafBrush = cmg.numBrushes;
        box_model.leaf.firstLeafBrush = cmg.numLeafBrushes;
        *cmg.leafbrushes.add(cmg.numLeafBrushes as usize) = cmg.numBrushes;

        i = 0;
        while i < 6 {
            side = i & 1;

            // brush sides
            s = &mut *cmg.brushsides.add((cmg.numBrushSides + i) as usize);
            (*s).plane = &mut *cmg.planes.add((cmg.numPlanes + i * 2 + side) as usize);
            (*s).shaderNum = cmg.numShaders;

            // planes
            p = &mut *box_planes.add((i * 2) as usize);
            (*p).type_ = (i >> 1) as c_int;
            (*p).signbits = 0;
            VectorClear((*p).normal.as_mut_ptr());
            (*p).normal[(i >> 1) as usize] = 1.0;

            p = &mut *box_planes.add((i * 2 + 1) as usize);
            (*p).type_ = (3 + (i >> 1)) as c_int;
            (*p).signbits = 0;
            VectorClear((*p).normal.as_mut_ptr());
            (*p).normal[(i >> 1) as usize] = -1.0;

            #[cfg(feature = "bspc")]
            {
                bspc_only::SetPlaneSignbits(p);
            }

            i += 1;
        }
    }
}

//
// ===================
// CM_TempBoxModel
//
// To keep everything totally uniform, bounding boxes are turned into small
// BSP trees instead of being compared directly.
// Capsules are handled differently though.
// ===================
//
pub fn CM_TempBoxModel(mins: *const vec3_t, maxs: *const vec3_t, capsule: c_int) -> clipHandle_t {
    unsafe {
        VectorCopy(mins, box_model.mins.as_mut_ptr());
        VectorCopy(maxs, box_model.maxs.as_mut_ptr());

        if capsule != 0 {
            return CAPSULE_MODEL_HANDLE;
        }

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

        BOX_MODEL_HANDLE
    }
}

//
// ===================
// CM_ModelBounds
// ===================
//
pub fn CM_ModelBounds(model: clipHandle_t, mins: *mut vec3_t, maxs: *mut vec3_t) {
    unsafe {
        let cmod: *mut cmodel_t;

        cmod = CM_ClipHandleToModel(model, null_mut());
        VectorCopy((*cmod).mins.as_ptr(), mins);
        VectorCopy((*cmod).maxs.as_ptr(), maxs);
    }
}

//
// ===================
// CM_RegisterTerrain
//
// Allows physics to examine the terrain data.
// ===================
//
#[cfg(not(feature = "bspc"))]
pub fn CM_RegisterTerrain(config: *const c_char, server: bool) -> *mut CCMLandScape {
    unsafe {
        let mut ls: *mut CCMLandScape;

        if !cmg.landScape.is_null() {
            // Already spawned so just return
            ls = cmg.landScape;
            // ls->IncreaseRefCount();
            return ls;
        }
        // Doesn't exist so create and link in
        ls = CM_InitTerrain(config, 0, server);

        // Increment for the next instance
        if !cmg.landScape.is_null() {
            Com_Error(
                ERR_DROP,
                c"You cannot have more than one terrain brush.\n\0".as_ptr(),
            );
        }
        cmg.landScape = ls;
        ls
    }
}

//
// ===================
// CM_ShutdownTerrain
// ===================
//
#[cfg(not(feature = "bspc"))]
pub fn CM_ShutdownTerrain(_terrainId: u32) {
    unsafe {
        let landscape: *mut CCMLandScape;

        landscape = cmg.landScape;

        if !landscape.is_null() {
            // landscape->DecreaseRefCount();
            // if(landscape->GetRefCount() <= 0)
            // {
            // delete landscape;
            cmg.landScape = null_mut();
            // }
        }
    }
}

pub fn CM_LoadSubBSP(name: *const c_char, clientload: qboolean) -> c_int {
    unsafe {
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

        if NumSubBSP == MAX_SUB_BSP as c_int {
            Com_Error(ERR_DROP, c"CM_LoadSubBSP: too many unique sub BSPs\0".as_ptr());
        }

        CM_LoadMap_Actual(name, clientload, &mut checksum, &mut SubBSP[NumSubBSP as usize]);
        NumSubBSP += 1;

        count
    }
}

pub fn CM_FindSubBSP(modelIndex: c_int) -> c_int {
    unsafe {
        let mut i: c_int;
        let mut count: c_int;

        count = cmg.numSubModels;
        if modelIndex < count {
            // belongs to the main bsp
            return -1;
        }

        i = 0;
        while i < NumSubBSP {
            count += SubBSP[i as usize].numSubModels;
            if modelIndex < count {
                return i;
            }
            i += 1;
        }
        -1
    }
}

pub fn CM_GetWorldBounds(mins: *mut vec3_t, maxs: *mut vec3_t) {
    unsafe {
        VectorCopy((*cmg.cmodels).mins.as_ptr(), mins);
        VectorCopy((*cmg.cmodels).maxs.as_ptr(), maxs);
    }
}

pub fn CM_ModelContents_Actual(model: clipHandle_t, cm: *mut clipMap_t) -> c_int {
    unsafe {
        let mut cmod: *mut cmodel_t;
        let mut contents: c_int = 0;
        let mut i: c_int;
        let mut cm_local = if cm.is_null() {
            addr_of_mut!(cmg)
        } else {
            cm
        };

        cmod = CM_ClipHandleToModel(model, null_mut());

        // MCG ADDED - return the contents, too
        if (*cmod).leaf.numLeafBrushes != 0 {
            // check for brush
            let mut brushNum: c_int;
            i = (*cmod).leaf.firstLeafBrush;
            while i < (*cmod).leaf.firstLeafBrush + (*cmod).leaf.numLeafBrushes {
                brushNum = *(*cm_local).leafbrushes.add(i as usize);
                contents |= (*(*cm_local).brushes.add(brushNum as usize)).contents;
                i += 1;
            }
        }
        if (*cmod).leaf.numLeafSurfaces != 0 {
            // if not brush, check for patch
            let mut surfaceNum: c_int;
            i = (*cmod).leaf.firstLeafSurface;
            while i < (*cmod).leaf.firstLeafSurface + (*cmod).leaf.numLeafSurfaces {
                surfaceNum = *(*cm_local).leafsurfaces.add(i as usize);
                if !(*(*cm_local).surfaces.add(surfaceNum as usize)).is_null() {
                    // HERNH?  How could we have a null surf within our cmod->leaf.numLeafSurfaces?
                    contents |=
                        (*(**(*cm_local).surfaces.add(surfaceNum as usize))).contents;
                }
                i += 1;
            }
        }
        contents
    }
}

pub fn CM_ModelContents(model: clipHandle_t, subBSPIndex: c_int) -> c_int {
    unsafe {
        if subBSPIndex < 0 {
            return CM_ModelContents_Actual(model, null_mut());
        }

        CM_ModelContents_Actual(model, &mut SubBSP[subBSPIndex as usize])
    }
}

// Helper trait for fileHandle_t to check if it's null
impl fileHandle_t {
    pub fn is_null(&self) -> bool {
        self.handle == 0
    }
}

// Note: Some of the extern declarations are stubbed here because they're defined elsewhere
// and we need to avoid circular dependencies. The actual implementations should be in their
// respective modules.

use crate::codemp::game::q_shared_h::cplane_t;
