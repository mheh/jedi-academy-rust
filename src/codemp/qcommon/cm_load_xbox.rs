// cmodel.c -- model loading
use core::ffi::{c_int, c_char, c_void};
use std::ffi::CStr;
use std::mem;

// rwwRMG - include
// rwwRMG - include

// Static SPARC<byte> visData equivalent - defined as Option to handle null state
pub static mut visData: Option<*mut c_void> = None;

pub extern "C" fn SparcAllocator(size: c_int) -> *mut c_void {
    unsafe { Z_Malloc(size as usize, 5 /* TAG_BSP */, false) }
}

pub extern "C" fn SparcDeallocator(ptr: *mut c_void) {
    unsafe { Z_Free(ptr) }
}

// Forward declarations
extern "C" {
    pub fn Z_Malloc(size: usize, tag: c_int, clear: bool) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Z_TagFree(tag: c_int);
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Memset(dst: *mut c_void, c: c_int, count: usize) -> *mut c_void;
    pub fn Hunk_Alloc(size: usize, tag: c_int) -> *mut c_void;
    pub fn Cvar_Get(varname: *const c_char, varvalue: *const c_char, flags: c_int) -> *mut c_void;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn COM_StripExtension(in_: *const c_char, out: *mut c_char);
    pub fn PlaneTypeForNormal(normal: *const [f32; 3]) -> c_int;
    pub fn CM_LoadShaderText(forceReload: bool);
    pub fn RE_SetWorldVisData(vis: *mut c_void);
    pub fn CM_GridAlloc();
    pub fn CM_PatchCollideFromGridTempAlloc();
    pub fn CM_PreparePatchCollide(num: c_int);
    pub fn CM_TempPatchPlanesAlloc();
    pub fn CM_GridDealloc();
    pub fn CM_PatchCollideFromGridTempDealloc();
    pub fn CM_TempPatchPlanesDealloc();
    pub fn CM_GeneratePatchCollide(
        width: c_int,
        height: c_int,
        points: *mut [f32; 3],
        facetbuf: *mut c_void,
        gridbuf: *mut c_int,
    ) -> *mut c_void;
    pub fn CM_ClearLevelPatches();
    pub fn CM_ClearMap();
    pub fn CM_InitBoxHull();
    pub fn CM_FloodAreaConnections();
    pub fn CM_SetupShaderProperties();
    pub fn CM_ShutdownShaderProperties();
    pub fn CM_LeafCluster(leafnum: c_int) -> c_int;
    pub fn CM_LeafArea(leafnum: c_int) -> c_int;
    pub fn CM_ClipHandleToModel(handle: c_int) -> *mut c_void;
    pub fn CM_InitTerrain(config: *const c_char, a: c_int, server: bool) -> *mut c_void;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn memcpy(dest: *mut c_void, src: *const c_void, len: usize) -> *mut c_void;
    pub fn crc32(crc: u32, buf: *const u8, len: usize) -> u32;

    #[link_name = "TheRandomMissionManager"]
    pub static mut TheRandomMissionManager: *mut c_void;
}

// Macro: to allow boxes to be treated as brush models, we allocate
// some extra indexes along with those needed by the map
const BOX_BRUSHES: c_int = 1;
const BOX_SIDES: c_int = 6;
const BOX_LEAFS: c_int = 2;
const BOX_PLANES: c_int = 12;

const BOX_MODEL_HANDLE: c_int = -1;
const CAPSULE_MODEL_HANDLE: c_int = -2;

const MAX_QPATH: usize = 256;
const MAX_SUBMODELS: c_int = 256;
const MAX_SUB_BSP: c_int = 16;

const MAX_PATCH_VERTS: c_int = 1024;
const CM_MAX_GRID_SIZE: c_int = 9;

const TAG_BSP: c_int = 5;
const TAG_TEMP_WORKSPACE: c_int = 6;

const VIS_HEADER: c_int = 8;

const ERR_DROP: c_int = 1;

const CVAR_CHEAT: c_int = 16;
const CVAR_ARCHIVE: c_int = 1;

const CONTENTS_BODY: c_int = 0x04;

// to allow boxes to be treated as brush models, we allocate
// some extra indexes along with those needed by the map
#[allow(non_snake_case)]
#[repr(C)]
pub struct clipMap_t {
    pub name: [c_char; MAX_QPATH],
    pub numShaders: c_int,
    pub shaders: *mut c_void, // CCMShader*

    pub numPlanes: c_int,
    pub planes: *mut c_void, // cplane_t*

    pub numNodes: c_int,
    pub nodes: *mut c_void, // cNode_t*

    pub numLeafs: c_int,
    pub leafs: *mut c_void, // cLeaf_t*

    pub numLeafBrushes: c_int,
    pub leafbrushes: *mut c_int,

    pub numLeafSurfaces: c_int,
    pub leafsurfaces: *mut c_int,

    pub numSubModels: c_int,
    pub cmodels: *mut c_void, // cmodel_t*

    pub numBrushes: c_int,
    pub brushes: *mut c_void, // cbrush_t*

    pub numBrushSides: c_int,
    pub brushsides: *mut c_void, // cbrushside_t*

    pub numSurfaces: c_int,
    pub surfaces: *mut *mut c_void, // cPatch_t**

    pub numClusters: c_int,
    pub clusterBytes: c_int,
    pub vised: bool,
    pub visibility: *mut c_void, // SPARC<byte>*

    pub numAreas: c_int,
    pub areas: *mut c_void, // cArea_t*
    pub areaPortals: *mut c_int,

    pub numEntityChars: c_int,
    pub entityString: *mut c_char,

    pub landScape: *mut c_void, // CCMLandScape*
}

#[repr(C)]
pub struct cmodel_t {
    pub bounds: [[f32; 3]; 2],
    pub firstNode: c_int,
    pub leaf: c_void, // cLeaf_t leaf
}

#[repr(C)]
pub struct cplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
    pub pad: [u8; 2],
}

#[repr(C)]
pub struct cbrush_t {
    pub bounds: [[f32; 3]; 2],
    pub numsides: c_int,
    pub sides: *mut c_void, // cbrushside_t*
    pub shaderNum: c_int,
    pub contents: c_int,
}

#[repr(C)]
pub struct cbrushside_t {
    pub planeNum: c_int,
    pub shaderNum: c_int,
}

#[repr(C)]
pub struct dshader_t {
    pub shader: [c_char; MAX_QPATH],
    pub contentFlags: c_int,
    pub surfaceFlags: c_int,
}

#[repr(C)]
pub struct CCMShader {
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
pub struct cNode_t {
    pub planeNum: c_int,
    pub children: [c_int; 2],
}

#[repr(C)]
pub struct dleaf_t {
    pub cluster: c_int,
    pub area: c_int,
    pub firstLeafBrush: c_int,
    pub numLeafBrushes: c_int,
    pub firstLeafSurface: c_int,
    pub numLeafSurfaces: c_int,
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
pub struct dplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
}

#[repr(C)]
pub struct dbrush_t {
    pub firstSide: c_int,
    pub numSides: c_int,
    pub shaderNum: c_int,
}

#[repr(C)]
pub struct dbrushside_t {
    pub planeNum: c_int,
    pub shaderNum: c_int,
}

#[repr(C)]
pub struct dpatch_t {
    pub shaderNum: c_int,
    pub patchWidth: c_int,
    pub patchHeight: c_int,
    pub verts: c_int,
    pub code: c_int,
}

#[repr(C)]
pub struct mapVert_t {
    pub xyz: [f32; 3],
    pub st: [f32; 2],
    pub normal: [f32; 3],
    pub color: [u8; 4],
}

#[repr(C)]
pub struct cPatch_t {
    pub contents: c_int,
    pub surfaceFlags: c_int,
    pub pc: *mut c_void,
}

#[repr(C)]
pub struct cArea_t {
    pub numareaportals: c_int,
    pub firstareaportal: c_int,
    pub floodnum: c_int,
    pub floodvalid: c_int,
}

#[repr(C)]
pub struct Lump {
    pub data: *mut c_void,
    pub len: c_int,
}

impl Lump {
    pub fn load(&mut self, stripName: *const c_char, suffix: *const c_char) {
        // Stub - implementation handled by external code
    }

    pub fn clear(&mut self) {
        self.data = std::ptr::null_mut();
        self.len = 0;
    }
}

#[repr(C)]
pub struct facetLoad_t {
    // Stub structure
    data: [u8; 1],
}

pub static mut cmg: clipMap_t = clipMap_t {
    name: [0; MAX_QPATH],
    numShaders: 0,
    shaders: std::ptr::null_mut(),
    numPlanes: 0,
    planes: std::ptr::null_mut(),
    numNodes: 0,
    nodes: std::ptr::null_mut(),
    numLeafs: 0,
    leafs: std::ptr::null_mut(),
    numLeafBrushes: 0,
    leafbrushes: std::ptr::null_mut(),
    numLeafSurfaces: 0,
    leafsurfaces: std::ptr::null_mut(),
    numSubModels: 0,
    cmodels: std::ptr::null_mut(),
    numBrushes: 0,
    brushes: std::ptr::null_mut(),
    numBrushSides: 0,
    brushsides: std::ptr::null_mut(),
    numSurfaces: 0,
    surfaces: std::ptr::null_mut(),
    numClusters: 0,
    clusterBytes: 0,
    vised: false,
    visibility: std::ptr::null_mut(),
    numAreas: 0,
    areas: std::ptr::null_mut(),
    areaPortals: std::ptr::null_mut(),
    numEntityChars: 0,
    entityString: std::ptr::null_mut(),
    landScape: std::ptr::null_mut(),
};

pub static mut c_pointcontents: c_int = 0;
pub static mut c_traces: c_int = 0;
pub static mut c_brush_traces: c_int = 0;
pub static mut c_patch_traces: c_int = 0;

pub static mut cmod_base: *mut u8 = std::ptr::null_mut();

pub static mut cm_noAreas: *mut c_void = std::ptr::null_mut();
pub static mut cm_noCurves: *mut c_void = std::ptr::null_mut();
pub static mut cm_playerCurveClip: *mut c_void = std::ptr::null_mut();

pub static mut box_model: cmodel_t = cmodel_t {
    bounds: [[0.0; 3]; 2],
    firstNode: 0,
    leaf: (),
};
pub static mut box_planes: *mut cplane_t = std::ptr::null_mut();
pub static mut box_brush: *mut cbrush_t = std::ptr::null_mut();

pub static mut SubBSP: [clipMap_t; MAX_SUB_BSP as usize] = [clipMap_t {
    name: [0; MAX_QPATH],
    numShaders: 0,
    shaders: std::ptr::null_mut(),
    numPlanes: 0,
    planes: std::ptr::null_mut(),
    numNodes: 0,
    nodes: std::ptr::null_mut(),
    numLeafs: 0,
    leafs: std::ptr::null_mut(),
    numLeafBrushes: 0,
    leafbrushes: std::ptr::null_mut(),
    numLeafSurfaces: 0,
    leafsurfaces: std::ptr::null_mut(),
    numSubModels: 0,
    cmodels: std::ptr::null_mut(),
    numBrushes: 0,
    brushes: std::ptr::null_mut(),
    numBrushSides: 0,
    brushsides: std::ptr::null_mut(),
    numSurfaces: 0,
    surfaces: std::ptr::null_mut(),
    numClusters: 0,
    clusterBytes: 0,
    vised: false,
    visibility: std::ptr::null_mut(),
    numAreas: 0,
    areas: std::ptr::null_mut(),
    areaPortals: std::ptr::null_mut(),
    numEntityChars: 0,
    entityString: std::ptr::null_mut(),
    landScape: std::ptr::null_mut(),
}; MAX_SUB_BSP as usize];

pub static mut NumSubBSP: c_int = 0;
pub static mut TotalSubModels: c_int = 0;

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
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadShaders(data: *mut c_void, len: c_int) {
    let mut in_: *mut dshader_t = data as *mut dshader_t;
    let mut i: c_int;
    let mut count: c_int;
    let mut out_: *mut CCMShader;

    in_ = data as *mut dshader_t;
    if len % (mem::size_of::<dshader_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"CMod_LoadShaders: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<dshader_t>() as c_int);

    if count < 1 {
        Com_Error(
            ERR_DROP,
            b"Map with no shaders\0".as_ptr() as *const c_char,
        );
    }
    cmg.shaders = Hunk_Alloc(((1 + count) * mem::size_of::<CCMShader>() as c_int) as usize, 0) as *mut CCMShader as *mut c_void;
    cmg.numShaders = count;

    out_ = cmg.shaders as *mut CCMShader;
    i = 0;
    while i < count {
        Q_strncpyz(
            (*out_).shader.as_mut_ptr(),
            (*in_).shader.as_ptr(),
            MAX_QPATH as c_int,
        );
        (*out_).contentFlags = (*in_).contentFlags;
        (*out_).surfaceFlags = (*in_).surfaceFlags;

        in_ = in_.offset(1);
        out_ = out_.offset(1);
        i += 1;
    }
}


/*
=================
CMod_LoadSubmodels
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadSubmodels(data: *mut c_void, len: c_int) {
    let mut in_: *mut dmodel_t;
    let mut out_: *mut cmodel_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;
    let mut indexes: *mut c_int;

    in_ = data as *mut dmodel_t;
    if len % (mem::size_of::<dmodel_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"CMod_LoadSubmodels: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<dmodel_t>() as c_int);

    if count < 1 {
        Com_Error(
            ERR_DROP,
            b"Map with no models\0".as_ptr() as *const c_char,
        );
    }

    if count > MAX_SUBMODELS {
        Com_Error(
            ERR_DROP,
            b"MAX_SUBMODELS (%d) exceeded by %d\0".as_ptr() as *const c_char,
            MAX_SUBMODELS,
            count - MAX_SUBMODELS,
        );
    }

    cmg.cmodels = Hunk_Alloc((count * mem::size_of::<cmodel_t>() as c_int) as usize, 0) as *mut c_void;
    cmg.numSubModels = count;

    i = 0;
    loop {
        if !(i < count) {
            break;
        }
        out_ = (cmg.cmodels as *mut cmodel_t).offset(i as isize);

        j = 0;
        loop {
            if !(j < 3) {
                break;
            }
            // spread the mins / maxs by a pixel
            (*out_).bounds[0][j as usize] = (*in_).mins[j as usize] - 1.0;
            (*out_).bounds[1][j as usize] = (*in_).maxs[j as usize] + 1.0;
            j += 1;
        }

        if i == 0 {
            (*out_).firstNode = 0;
            in_ = in_.offset(1);
            i += 1;
            continue; // world model doesn't need other info
        }

        (*out_).firstNode = -1;

        // make a "leaf" just to hold the model's brushes and surfaces
        // Note: This simplified structure doesn't fully represent the leaf member
        // The actual implementation would need proper leaf_t handling

        indexes = Hunk_Alloc(((*in_).numBrushes * 4) as usize, 0) as *mut c_int;
        // out->leaf.firstLeafBrush = indexes - cmg.leafbrushes;
        j = 0;
        loop {
            if !(j < (*in_).numBrushes) {
                break;
            }
            *indexes.offset(j as isize) = (*in_).firstBrush + j;
            j += 1;
        }

        indexes = Hunk_Alloc(((*in_).numSurfaces * 4) as usize, 0) as *mut c_int;
        // out->leaf.firstLeafSurface = indexes - cmg.leafsurfaces;
        j = 0;
        loop {
            if !(j < (*in_).numSurfaces) {
                break;
            }
            *indexes.offset(j as isize) = (*in_).firstSurface + j;
            j += 1;
        }

        in_ = in_.offset(1);
        i += 1;
    }
}

/*
=================
CMod_LoadNodes

=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadNodes(data: *mut c_void, len: c_int) {
    let mut in_: *mut dnode_t;
    let mut out_: *mut cNode_t;
    let mut i: c_int;
    let mut count: c_int;

    in_ = data as *mut dnode_t;
    if len % (mem::size_of::<dnode_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<dnode_t>() as c_int);

    if count < 1 {
        Com_Error(
            ERR_DROP,
            b"Map has no nodes\0".as_ptr() as *const c_char,
        );
    }
    cmg.nodes = Hunk_Alloc((count * mem::size_of::<cNode_t>() as c_int) as usize, 0) as *mut c_void;
    cmg.numNodes = count;

    out_ = cmg.nodes as *mut cNode_t;

    i = 0;
    loop {
        if !(i < count) {
            break;
        }
        (*out_).planeNum = (*in_).planeNum;
        (*out_).children[0] = (*in_).children[0];
        (*out_).children[1] = (*in_).children[1];

        out_ = out_.offset(1);
        in_ = in_.offset(1);
        i += 1;
    }
}

/*
=================
CM_BoundBrush

=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_BoundBrush(b: *mut cbrush_t) {
    let sides = (*b).sides as *const cbrushside_t;

    (*b).bounds[0][0] = -(*(cmg.planes as *mut cplane_t)
        .offset((*sides.offset(0)).planeNum as isize))
        .dist;
    (*b).bounds[1][0] = (*(cmg.planes as *mut cplane_t)
        .offset((*sides.offset(1)).planeNum as isize))
        .dist;

    (*b).bounds[0][1] = -(*(cmg.planes as *mut cplane_t)
        .offset((*sides.offset(2)).planeNum as isize))
        .dist;
    (*b).bounds[1][1] = (*(cmg.planes as *mut cplane_t)
        .offset((*sides.offset(3)).planeNum as isize))
        .dist;

    (*b).bounds[0][2] = -(*(cmg.planes as *mut cplane_t)
        .offset((*sides.offset(4)).planeNum as isize))
        .dist;
    (*b).bounds[1][2] = (*(cmg.planes as *mut cplane_t)
        .offset((*sides.offset(5)).planeNum as isize))
        .dist;
}


/*
=================
CMod_LoadBrushes

=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadBrushes(data: *mut c_void, len: c_int) {
    let mut in_: *mut dbrush_t;
    let mut out_: *mut cbrush_t;
    let mut i: c_int;
    let mut count: c_int;

    in_ = data as *mut dbrush_t;
    if len % (mem::size_of::<dbrush_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<dbrush_t>() as c_int);

    cmg.brushes = Hunk_Alloc(
        ((BOX_BRUSHES + count) * mem::size_of::<cbrush_t>() as c_int) as usize,
        0,
    ) as *mut c_void;
    cmg.numBrushes = count;

    out_ = cmg.brushes as *mut cbrush_t;

    i = 0;
    loop {
        if !(i < count) {
            break;
        }
        (*out_).sides =
            (cmg.brushsides as *mut cbrushside_t).offset((*in_).firstSide as isize) as *mut c_void;
        (*out_).numsides = (*in_).numSides;

        (*out_).shaderNum = (*in_).shaderNum;
        if (*out_).shaderNum < 0 || (*out_).shaderNum >= cmg.numShaders {
            Com_Error(
                ERR_DROP,
                b"CMod_LoadBrushes: bad shaderNum: %i\0".as_ptr() as *const c_char,
                (*out_).shaderNum,
            );
        }
        (*out_).contents =
            (*(cmg.shaders as *mut CCMShader).offset((*out_).shaderNum as isize)).contentFlags;

        CM_BoundBrush(out_);

        out_ = out_.offset(1);
        in_ = in_.offset(1);
        i += 1;
    }
}

/*
=================
CMod_LoadLeafs
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadLeafs(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut out_: *mut cLeaf_t;
    let mut in_: *mut dleaf_t;
    let mut count: c_int;

    in_ = data as *mut dleaf_t;
    if len % (mem::size_of::<dleaf_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<dleaf_t>() as c_int);

    if count < 1 {
        Com_Error(
            ERR_DROP,
            b"Map with no leafs\0".as_ptr() as *const c_char,
        );
    }

    cmg.leafs = Hunk_Alloc(
        ((BOX_LEAFS + count) * mem::size_of::<cLeaf_t>() as c_int) as usize,
        0,
    ) as *mut c_void;
    cmg.numLeafs = count;
    out_ = cmg.leafs as *mut cLeaf_t;

    i = 0;
    loop {
        if !(i < count) {
            break;
        }
        (*out_).cluster = (*in_).cluster;
        (*out_).area = (*in_).area;
        (*out_).firstLeafBrush = (*in_).firstLeafBrush;
        (*out_).numLeafBrushes = (*in_).numLeafBrushes;
        (*out_).firstLeafSurface = (*in_).firstLeafSurface;
        (*out_).numLeafSurfaces = (*in_).numLeafSurfaces;

        if (*out_).cluster >= cmg.numClusters {
            cmg.numClusters = (*out_).cluster + 1;
        }
        if (*out_).area >= cmg.numAreas {
            cmg.numAreas = (*out_).area + 1;
        }

        in_ = in_.offset(1);
        out_ = out_.offset(1);
        i += 1;
    }

    cmg.areas = Hunk_Alloc((cmg.numAreas * mem::size_of::<cArea_t>() as c_int) as usize, 0) as *mut c_void;
    cmg.areaPortals = Hunk_Alloc(
        ((cmg.numAreas * cmg.numAreas) * mem::size_of::<c_int>() as c_int) as usize,
        0,
    ) as *mut c_int;
}

/*
=================
CMod_LoadPlanes
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadPlanes(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut out_: *mut cplane_t;
    let mut in_: *mut dplane_t;
    let mut count: c_int;
    let mut bits: c_int;

    in_ = data as *mut dplane_t;
    if len % (mem::size_of::<dplane_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<dplane_t>() as c_int);

    if count < 1 {
        Com_Error(
            ERR_DROP,
            b"Map with no planes\0".as_ptr() as *const c_char,
        );
    }
    cmg.planes = Hunk_Alloc(
        ((BOX_PLANES + count) * mem::size_of::<cplane_t>() as c_int) as usize,
        0,
    ) as *mut c_void;
    cmg.numPlanes = count;

    out_ = cmg.planes as *mut cplane_t;

    i = 0;
    loop {
        if !(i < count) {
            break;
        }
        bits = 0;
        j = 0;
        loop {
            if !(j < 3) {
                break;
            }
            (*out_).normal[j as usize] = (*in_).normal[j as usize];
            if (*out_).normal[j as usize] < 0.0 {
                bits |= 1 << j;
            }
            j += 1;
        }

        (*out_).dist = (*in_).dist;
        (*out_).type_ = PlaneTypeForNormal((*out_).normal.as_ptr() as *const [f32; 3]) as u8;
        (*out_).signbits = bits as u8;

        in_ = in_.offset(1);
        out_ = out_.offset(1);
        i += 1;
    }
}

/*
=================
CMod_LoadLeafBrushes
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadLeafBrushes(data: *mut c_void, len: c_int) {
    let mut out_: *mut c_int;
    let mut in_: *mut c_int;
    let mut count: c_int;

    in_ = data as *mut c_int;
    if len % (mem::size_of::<c_int>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<c_int>() as c_int);

    cmg.leafbrushes = Hunk_Alloc(
        ((count + BOX_BRUSHES) * mem::size_of::<c_int>() as c_int) as usize,
        0,
    ) as *mut c_int;
    cmg.numLeafBrushes = count;

    out_ = cmg.leafbrushes;

    memcpy(
        out_ as *mut c_void,
        in_ as *const c_void,
        len as usize,
    );
}


#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadLeafSurfaces(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut out_: *mut c_int;
    let mut in_: *mut c_int;
    let mut count: c_int;

    in_ = data as *mut c_int;
    if len % (mem::size_of::<c_int>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<c_int>() as c_int);

    cmg.leafsurfaces =
        Hunk_Alloc((count * mem::size_of::<c_int>() as c_int) as usize, 0) as *mut c_int;
    cmg.numLeafSurfaces = count;

    out_ = cmg.leafsurfaces;

    memcpy(
        out_ as *mut c_void,
        in_ as *const c_void,
        len as usize,
    );
}

/*
=================
CMod_LoadBrushSides
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadBrushSides(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut out_: *mut cbrushside_t;
    let mut in_: *mut dbrushside_t;
    let mut count: c_int;

    in_ = data as *mut dbrushside_t;
    if len % (mem::size_of::<dbrushside_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }
    count = len / (mem::size_of::<dbrushside_t>() as c_int);

    cmg.brushsides = Hunk_Alloc(
        ((BOX_SIDES + count) * mem::size_of::<cbrushside_t>() as c_int) as usize,
        0,
    ) as *mut c_void;
    cmg.numBrushSides = count;

    out_ = cmg.brushsides as *mut cbrushside_t;

    i = 0;
    loop {
        if !(i < count) {
            break;
        }
        (*out_).planeNum = (*in_).planeNum;
        // assert(in->planeNum == out->planeNum.GetValue());

        (*out_).shaderNum = (*in_).shaderNum;
        if (*out_).shaderNum < 0 || (*out_).shaderNum >= cmg.numShaders {
            Com_Error(
                ERR_DROP,
                b"CMod_LoadBrushSides: bad shaderNum: %i\0".as_ptr() as *const c_char,
                (*out_).shaderNum,
            );
        }

        in_ = in_.offset(1);
        out_ = out_.offset(1);
        i += 1;
    }
}


/*
=================
CMod_LoadEntityString
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadEntityString(data: *mut c_void, len: c_int) {
    cmg.entityString = Hunk_Alloc(len as usize, 0) as *mut c_char;
    cmg.numEntityChars = len;
    memcpy(cmg.entityString as *mut c_void, data, len as usize);
}

/*
=================
CMod_LoadVisibility
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadVisibility(data: *mut c_void, len: c_int) {
    let buf: *mut c_char;

    if len == 0 {
        cmg.visibility = std::ptr::null_mut();
        return;
    }
    buf = data as *mut c_char;

    // visData.SetAllocator(SparcAllocator, SparcDeallocator);

    cmg.vised = true;
    cmg.numClusters = *(buf as *mut c_int);
    cmg.clusterBytes = *((buf as *mut c_int).offset(1));
    // visData.Load(buf + VIS_HEADER, len - VIS_HEADER);
    cmg.visibility = buf.offset(VIS_HEADER as isize) as *mut c_void;
    // RE_SetWorldVisData(&visData);
}

//==================================================================


/*
=================
CMod_LoadPatches
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CMod_LoadPatches(
    verts: *mut c_void,
    vertlen: c_int,
    surfaces: *mut c_void,
    surfacelen: c_int,
    numsurfs: c_int,
) {
    let mut dv: *mut mapVert_t;
    let mut dv_p: *mut mapVert_t;
    let mut in_: *mut dpatch_t;
    let mut count: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut c: c_int;
    let mut patch: *mut cPatch_t;
    let mut points: [[f32; 3]; MAX_PATCH_VERTS as usize] = [[0.0; 3]; MAX_PATCH_VERTS as usize];
    let mut width: c_int;
    let mut height: c_int;
    let mut shaderNum: c_int;

    count = surfacelen / (mem::size_of::<dpatch_t>() as c_int);

    cmg.numSurfaces = numsurfs;
    cmg.surfaces = Hunk_Alloc((cmg.numSurfaces * mem::size_of::<*mut cPatch_t>() as c_int) as usize, 0) as *mut *mut c_void;

    dv = verts as *mut mapVert_t;
    if vertlen % (mem::size_of::<mapVert_t>() as c_int) != 0 {
        Com_Error(
            ERR_DROP,
            b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char,
        );
    }

    let patchScratch: *mut u8 = Z_Malloc(
        (mem::size_of::<cPatch_t>() as c_int * count) as usize,
        TAG_BSP,
        true,
    ) as *mut u8;

    CM_GridAlloc();
    CM_PatchCollideFromGridTempAlloc();
    CM_PreparePatchCollide(count);
    CM_TempPatchPlanesAlloc();

    let facetbuf: *mut facetLoad_t = Z_Malloc(
        (16 * mem::size_of::<facetLoad_t>() as c_int) as usize, // MAX_PATCH_PLANES = 16 approximately
        TAG_TEMP_WORKSPACE,
        false,
    ) as *mut facetLoad_t;

    let gridbuf: *mut c_int = Z_Malloc(
        ((CM_MAX_GRID_SIZE * CM_MAX_GRID_SIZE * 2) * mem::size_of::<c_int>() as c_int) as usize,
        TAG_TEMP_WORKSPACE,
        false,
    ) as *mut c_int;

    i = 0;
    let mut patchScratch_iter = patchScratch;
    loop {
        if !(i < count) {
            break;
        }
        in_ = (surfaces as *mut dpatch_t).offset(i as isize);

        patch = patchScratch_iter as *mut cPatch_t;
        *(cmg.surfaces as *mut *mut cPatch_t).offset((*in_).code as isize) = patch;
        patchScratch_iter = patchScratch_iter.offset(mem::size_of::<cPatch_t>() as isize);

        // load the full drawverts onto the stack
        width = (*in_).patchWidth;
        height = (*in_).patchHeight;
        c = width * height;
        if c > MAX_PATCH_VERTS {
            Com_Error(
                ERR_DROP,
                b"ParseMesh: MAX_PATCH_VERTS\0".as_ptr() as *const c_char,
            );
        }

        dv_p = dv.offset(((*in_).verts >> 12) as isize);
        j = 0;
        loop {
            if !(j < c) {
                break;
            }
            points[j as usize][0] = (*dv_p).xyz[0];
            points[j as usize][1] = (*dv_p).xyz[1];
            points[j as usize][2] = (*dv_p).xyz[2];

            dv_p = dv_p.offset(1);
            j += 1;
        }

        shaderNum = (*in_).shaderNum;
        (*patch).contents = (*(cmg.shaders as *mut CCMShader).offset(shaderNum as isize)).contentFlags;
        (*patch).surfaceFlags = (*(cmg.shaders as *mut CCMShader).offset(shaderNum as isize)).surfaceFlags;

        // create the internal facet structure
        (*patch).pc = CM_GeneratePatchCollide(
            width,
            height,
            points.as_mut_ptr(),
            facetbuf as *mut c_void,
            gridbuf,
        );

        i += 1;
    }

    CM_PatchCollideFromGridTempDealloc();
    CM_GridDealloc();
    CM_TempPatchPlanesDealloc();

    Z_Free(gridbuf as *mut c_void);
    Z_Free(facetbuf as *mut c_void);
}


/*
==================
CM_LoadMap

Loads in the map and all submodels
==================
*/
pub static mut gpvCachedMapDiskImage: *mut c_void = std::ptr::null_mut();
pub static mut gsCachedMapDiskImage: [c_char; MAX_QPATH] = [0; MAX_QPATH];
pub static mut gbUsingCachedMapDataRightNow: bool = false;
// if true, signifies that you can't delete this at the moment!! (used during z_malloc()-fail recovery attempt)

// called in response to a "devmapbsp blah" or "devmapall blah" command, do NOT use inside CM_Load unless you pass in true
//
// new bool return used to see if anything was freed, used during z_malloc failure re-try
//
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_DeleteCachedMap(bGuaranteedOkToDelete: bool) -> bool {
    let mut bActuallyFreedSomething: bool = false;

    if bGuaranteedOkToDelete || !gbUsingCachedMapDataRightNow {
        // dump cached disk image...
        //
        if !gpvCachedMapDiskImage.is_null() {
            Z_Free(gpvCachedMapDiskImage);
            gpvCachedMapDiskImage = std::ptr::null_mut();

            bActuallyFreedSomething = true;
        }
        gsCachedMapDiskImage[0] = 0;

        // force map loader to ignore cached internal BSP structures for next level CM_LoadMap() call...
        //
        cmg.name[0] = 0;
    }

    bActuallyFreedSomething
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_Free() {
    CM_ClearLevelPatches();
    // visData.Release();
    Z_TagFree(TAG_BSP);
}


#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_LoadMap_Actual(
    name: *const c_char,
    clientload: bool,
    checksum: *mut c_int,
) {
    let mut buf: *const c_int = std::ptr::null();
    let mut surfBuf: *const c_int = std::ptr::null();
    static mut last_checksum: u32 = 0;
    let mut lmName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut stripName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut outputLump: Lump = Lump {
        data: std::ptr::null_mut(),
        len: 0,
    };

    if name.is_null() || *name == 0 {
        Com_Error(
            ERR_DROP,
            b"CM_LoadMap: NULL name\0".as_ptr() as *const c_char,
        );
    }

    cm_noAreas = Cvar_Get(
        b"cm_noAreas\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        CVAR_CHEAT,
    );
    cm_noCurves = Cvar_Get(
        b"cm_noCurves\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        CVAR_CHEAT,
    );
    cm_playerCurveClip = Cvar_Get(
        b"cm_playerCurveClip\0".as_ptr() as *const c_char,
        b"1\0".as_ptr() as *const c_char,
        CVAR_ARCHIVE | CVAR_CHEAT,
    );

    Com_DPrintf(
        b"CM_LoadMap( %s, %i )\n\0".as_ptr() as *const c_char,
        name,
        if clientload { 1 } else { 0 },
    );

    if strcmp(cmg.name.as_ptr(), name) == 0 && clientload {
        *checksum = last_checksum as c_int;
        return;
    }

    CM_ClearMap();
    CM_ClearLevelPatches();

    // free old stuff
    Com_Memset(
        &mut cmg as *mut clipMap_t as *mut c_void,
        0,
        mem::size_of::<clipMap_t>(),
    );

    if *name == 0 {
        cmg.numLeafs = 1;
        cmg.numClusters = 1;
        cmg.numAreas = 1;
        cmg.cmodels = Z_Malloc(mem::size_of::<cmodel_t>(), TAG_BSP, true) as *mut c_void;
        *checksum = 0;
        return;
    }

    last_checksum = crc32(0, name as *const u8, strlen(name));
    COM_StripExtension(name, stripName.as_mut_ptr());

    // load into heap
    outputLump.load(stripName.as_ptr(), b"shaders\0".as_ptr() as *const c_char);
    CMod_LoadShaders(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"leafs\0".as_ptr() as *const c_char);
    CMod_LoadLeafs(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"leafbrushes\0".as_ptr() as *const c_char);
    CMod_LoadLeafBrushes(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"leafsurfaces\0".as_ptr() as *const c_char);
    CMod_LoadLeafSurfaces(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"planes\0".as_ptr() as *const c_char);
    CMod_LoadPlanes(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"brushsides\0".as_ptr() as *const c_char);
    CMod_LoadBrushSides(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"brushes\0".as_ptr() as *const c_char);
    CMod_LoadBrushes(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"models\0".as_ptr() as *const c_char);
    CMod_LoadSubmodels(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"nodes\0".as_ptr() as *const c_char);
    CMod_LoadNodes(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"entities\0".as_ptr() as *const c_char);
    CMod_LoadEntityString(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"visibility\0".as_ptr() as *const c_char);
    CMod_LoadVisibility(outputLump.data, outputLump.len);

    let mut misc: Lump = Lump {
        data: std::ptr::null_mut(),
        len: 0,
    };
    misc.load(stripName.as_ptr(), b"misc\0".as_ptr() as *const c_char);

    let num_surfs: c_int = *(misc.data as *mut c_int);
    misc.clear();

    let mut verts: Lump = Lump {
        data: std::ptr::null_mut(),
        len: 0,
    };
    verts.load(stripName.as_ptr(), b"verts\0".as_ptr() as *const c_char);

    let mut patches: Lump = Lump {
        data: std::ptr::null_mut(),
        len: 0,
    };
    patches.load(stripName.as_ptr(), b"patches\0".as_ptr() as *const c_char);
    CMod_LoadPatches(verts.data, verts.len, patches.data, patches.len, num_surfs);
    patches.clear();

    TotalSubModels += cmg.numSubModels;

    CM_LoadShaderText(false);
    CM_InitBoxHull();
    CM_SetupShaderProperties();

    *checksum = last_checksum as c_int;

    // do this whether or not the map was cached from last load...
    //
    CM_FloodAreaConnections();

    // allow this to be cached if it is loaded by the server
    if !clientload {
        Q_strncpyz(
            cmg.name.as_mut_ptr(),
            name,
            mem::size_of::<[c_char; MAX_QPATH]>() as c_int,
        );
    }
}

// need a wrapper function around this because of multiple returns, need to ensure bool is correct...
//
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_LoadMap(name: *const c_char, clientload: bool, checksum: *mut c_int) {
    CM_LoadMap_Actual(name, clientload, checksum);
}


/*
==================
CM_ClearMap
==================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_ClearMapLocal() {
    let mut i: c_int;

    CM_ShutdownShaderProperties();
    // MAT_Shutdown();

    if !TheRandomMissionManager.is_null() {
        // delete TheRandomMissionManager;
        TheRandomMissionManager = std::ptr::null_mut();
    }

    if !cmg.landScape.is_null() {
        // delete cmg.landScape;
        cmg.landScape = std::ptr::null_mut();
    }

    Com_Memset(
        &mut cmg as *mut clipMap_t as *mut c_void,
        0,
        mem::size_of::<clipMap_t>(),
    );
    CM_ClearLevelPatches();

    i = 0;
    loop {
        if !(i < NumSubBSP) {
            break;
        }
        memset(
            &mut SubBSP[i as usize] as *mut clipMap_t as *mut c_void,
            0,
            mem::size_of::<clipMap_t>(),
        );
        i += 1;
    }
    NumSubBSP = 0;
    TotalSubModels = 0;
}

/*
==================
CM_ClipHandleToModel
==================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_ClipHandleToModel_Local(
    handle: c_int,
    clipMap: *mut *mut clipMap_t,
) -> *mut cmodel_t {
    let mut i: c_int;
    let mut count: c_int;

    if handle < 0 {
        Com_Error(
            ERR_DROP,
            b"CM_ClipHandleToModel: bad handle %i\0".as_ptr() as *const c_char,
            handle,
        );
    }
    if handle < cmg.numSubModels {
        if !clipMap.is_null() {
            *clipMap = &mut cmg;
        }
        return (cmg.cmodels as *mut cmodel_t).offset(handle as isize);
    }
    if handle == BOX_MODEL_HANDLE {
        if !clipMap.is_null() {
            *clipMap = &mut cmg;
        }
        return &mut box_model;
    }

    count = cmg.numSubModels;
    i = 0;
    loop {
        if !(i < NumSubBSP) {
            break;
        }
        if handle < count + SubBSP[i as usize].numSubModels {
            if !clipMap.is_null() {
                *clipMap = &mut SubBSP[i as usize];
            }
            return (SubBSP[i as usize].cmodels as *mut cmodel_t)
                .offset((handle - count) as isize);
        }
        count += SubBSP[i as usize].numSubModels;
        i += 1;
    }

    if handle < MAX_SUBMODELS {
        Com_Error(
            ERR_DROP,
            b"CM_ClipHandleToModel: bad handle %i < %i < %i\0".as_ptr() as *const c_char,
            cmg.numSubModels,
            handle,
            MAX_SUBMODELS,
        );
    }
    Com_Error(
        ERR_DROP,
        b"CM_ClipHandleToModel: bad handle %i\0".as_ptr() as *const c_char,
        handle + MAX_SUBMODELS,
    );

    std::ptr::null_mut()
}
/*
==================
CM_InlineModel
==================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_InlineModel(index: c_int) -> c_int {
    if index < 0 || index >= TotalSubModels {
        Com_Error(
            ERR_DROP,
            b"CM_InlineModel: bad number (may need to re-BSP map?)\0".as_ptr() as *const c_char,
        );
    }
    index
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_NumClusters() -> c_int {
    cmg.numClusters
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_NumInlineModels() -> c_int {
    cmg.numSubModels
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_EntityString() -> *mut c_char {
    cmg.entityString
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_SubBSPEntityString(index: c_int) -> *mut c_char {
    SubBSP[index as usize].entityString
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_LeafCluster_Local(leafnum: c_int) -> c_int {
    if leafnum < 0 || leafnum >= cmg.numLeafs {
        Com_Error(
            ERR_DROP,
            b"CM_LeafCluster: bad number\0".as_ptr() as *const c_char,
        );
    }
    (*(cmg.leafs as *mut cLeaf_t).offset(leafnum as isize)).cluster
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_LeafArea_Local(leafnum: c_int) -> c_int {
    if leafnum < 0 || leafnum >= cmg.numLeafs {
        Com_Error(
            ERR_DROP,
            b"CM_LeafArea: bad number\0".as_ptr() as *const c_char,
        );
    }
    (*(cmg.leafs as *mut cLeaf_t).offset(leafnum as isize)).area
}

//=======================================================================


/*
===================
CM_InitBoxHull

Set up the planes and nodes so that the six floats of a bounding box
can just be stored out and get a proper clipping hull structure.
===================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_InitBoxHull_Local() {
    let mut i: c_int;
    let mut side: c_int;
    let mut p: *mut cplane_t;
    let mut s: *mut cbrushside_t;

    box_planes = (cmg.planes as *mut cplane_t).offset(cmg.numPlanes as isize);

    box_brush = (cmg.brushes as *mut cbrush_t).offset(cmg.numBrushes as isize);
    (*box_brush).numsides = 6;
    (*box_brush).sides = (cmg.brushsides as *mut cbrushside_t).offset(cmg.numBrushSides as isize) as *mut c_void;
    (*box_brush).contents = CONTENTS_BODY;

    box_model.firstNode = -1;
    // box_model.leaf.numLeafBrushes = 1;
    // box_model.leaf.firstLeafBrush = cmg.numBrushes;
    // box_model.leaf.firstLeafBrush = cmg.numLeafBrushes;
    // cmg.leafbrushes[cmg.numLeafBrushes] = cmg.numBrushes;

    i = 0;
    loop {
        if !(i < 6) {
            break;
        }
        side = i & 1;

        // brush sides
        s = (cmg.brushsides as *mut cbrushside_t).offset((cmg.numBrushSides + i) as isize);
        (*s).planeNum = cmg.numPlanes + i * 2 + side;
        (*s).shaderNum = cmg.numShaders;

        // planes
        p = &mut *box_planes.offset((i * 2) as isize);
        (*p).type_ = (i >> 1) as u8;
        (*p).signbits = 0;
        (*p).normal[0] = 0.0;
        (*p).normal[1] = 0.0;
        (*p).normal[2] = 0.0;
        (*p).normal[(i >> 1) as usize] = 1.0;

        p = &mut *box_planes.offset((i * 2 + 1) as isize);
        (*p).type_ = (3 + (i >> 1)) as u8;
        (*p).signbits = 0;
        (*p).normal[0] = 0.0;
        (*p).normal[1] = 0.0;
        (*p).normal[2] = 0.0;
        (*p).normal[(i >> 1) as usize] = -1.0;

        // SetPlaneSignbits( p );
        i += 1;
    }
}



/*
===================
CM_HeadnodeForBox

To keep everything totally uniform, bounding boxes are turned into small
BSP trees instead of being compared directly.
===================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_TempBoxModel(
    mins: *const [f32; 3],
    maxs: *const [f32; 3],
    capsule: c_int,
) -> c_int {
    box_model.bounds[0][0] = (*maxs)[0];
    box_model.bounds[0][1] = (*maxs)[1];
    box_model.bounds[0][2] = (*maxs)[2];
    box_model.bounds[1][0] = (*mins)[0];
    box_model.bounds[1][1] = (*mins)[1];
    box_model.bounds[1][2] = (*mins)[2];

    if capsule != 0 {
        return CAPSULE_MODEL_HANDLE;
    }

    (*box_planes.offset(0)).dist = (*maxs)[0];
    (*box_planes.offset(1)).dist = -(*maxs)[0];
    (*box_planes.offset(2)).dist = (*mins)[0];
    (*box_planes.offset(3)).dist = -(*mins)[0];
    (*box_planes.offset(4)).dist = (*maxs)[1];
    (*box_planes.offset(5)).dist = -(*maxs)[1];
    (*box_planes.offset(6)).dist = (*mins)[1];
    (*box_planes.offset(7)).dist = -(*mins)[1];
    (*box_planes.offset(8)).dist = (*maxs)[2];
    (*box_planes.offset(9)).dist = -(*maxs)[2];
    (*box_planes.offset(10)).dist = (*mins)[2];
    (*box_planes.offset(11)).dist = -(*mins)[2];

    (*box_brush).bounds[0][0] = (*mins)[0];
    (*box_brush).bounds[0][1] = (*mins)[1];
    (*box_brush).bounds[0][2] = (*mins)[2];
    (*box_brush).bounds[1][0] = (*maxs)[0];
    (*box_brush).bounds[1][1] = (*maxs)[1];
    (*box_brush).bounds[1][2] = (*maxs)[2];

    BOX_MODEL_HANDLE
}



/*
===================
CM_ModelBounds
===================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_ModelBounds(
    model: c_int,
    mins: *mut [f32; 3],
    maxs: *mut [f32; 3],
) {
    let mut cmod: *mut cmodel_t;

    cmod = CM_ClipHandleToModel_Local(model, std::ptr::null_mut());
    (*mins)[0] = (*cmod).bounds[0][0];
    (*mins)[1] = (*cmod).bounds[0][1];
    (*mins)[2] = (*cmod).bounds[0][2];
    (*maxs)[0] = (*cmod).bounds[1][0];
    (*maxs)[1] = (*cmod).bounds[1][1];
    (*maxs)[2] = (*cmod).bounds[1][2];
}

/*
===================
CM_RegisterTerrain

Allows physics to examine the terrain data.
===================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_RegisterTerrain(
    config: *const c_char,
    server: bool,
) -> *mut c_void {
    let mut ls: *mut c_void;

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
            b"You cannot have more than one terrain brush.\n\0".as_ptr() as *const c_char,
        );
    }
    cmg.landScape = ls;
    ls
}

/*
===================
CM_ShutdownTerrain
===================
*/

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_ShutdownTerrain(terrainId: c_int) {
    let mut landscape: *mut c_void;

    landscape = cmg.landScape;
    if !landscape.is_null() {
        // landscape->DecreaseRefCount();
        // if(landscape->GetRefCount() <= 0)
        {
            // delete landscape;
            cmg.landScape = std::ptr::null_mut();
        }
    }
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_LoadSubBSP(name: *const c_char, clientload: bool) -> c_int {
    let mut i: c_int;
    let mut checksum: c_int;
    let mut count: c_int;

    count = cmg.numSubModels;
    i = 0;
    loop {
        if !(i < NumSubBSP) {
            break;
        }
        if stricmp(name, SubBSP[i as usize].name.as_ptr()) == 0 {
            return count;
        }
        count += SubBSP[i as usize].numSubModels;
        i += 1;
    }

    if NumSubBSP == MAX_SUB_BSP {
        Com_Error(
            ERR_DROP,
            b"CM_LoadSubBSP: too many unique sub BSPs\0".as_ptr() as *const c_char,
        );
    }

    // #ifdef _XBOX
    //     assert(0); // MATT! - testing now - fix this later!
    // #else
    CM_LoadMap_Actual(
        name,
        clientload,
        &mut checksum,
    );
    // #endif
    NumSubBSP += 1;

    count
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_FindSubBSP(modelIndex: c_int) -> c_int {
    let mut i: c_int;
    let mut count: c_int;

    count = cmg.numSubModels;
    if modelIndex < count {
        // belongs to the main bsp
        return -1;
    }

    i = 0;
    loop {
        if !(i < NumSubBSP) {
            break;
        }
        count += SubBSP[i as usize].numSubModels;
        if modelIndex < count {
            return i;
        }
        i += 1;
    }
    -1
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_GetWorldBounds(mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
    (*mins)[0] = (*(cmg.cmodels as *mut cmodel_t)).bounds[0][0];
    (*mins)[1] = (*(cmg.cmodels as *mut cmodel_t)).bounds[0][1];
    (*mins)[2] = (*(cmg.cmodels as *mut cmodel_t)).bounds[0][2];
    (*maxs)[0] = (*(cmg.cmodels as *mut cmodel_t)).bounds[1][0];
    (*maxs)[1] = (*(cmg.cmodels as *mut cmodel_t)).bounds[1][1];
    (*maxs)[2] = (*(cmg.cmodels as *mut cmodel_t)).bounds[1][2];
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_ModelContents_Actual(model: c_int, cm: *mut clipMap_t) -> c_int {
    let mut cmod: *mut cmodel_t;
    let mut contents: c_int = 0;
    let mut i: c_int;

    let mut cm_ptr = cm;
    if cm.is_null() {
        cm_ptr = &mut cmg;
    }

    cmod = CM_ClipHandleToModel_Local(model, std::ptr::null_mut());

    //MCG ADDED - return the contents, too
    // Note: Simplified structure, actual leaf member needs proper handling
    // if( cmod->leaf.numLeafBrushes )		// check for brush
    // {
    //     let mut brushNum: c_int;
    //     for ( i = cmod->leaf.firstLeafBrush; i < cmod->leaf.firstLeafBrush+cmod->leaf.numLeafBrushes; i++ )
    //     {
    //         brushNum = (*cm_ptr).leafbrushes[i];
    //         contents |= (*((*cm_ptr).brushes as *mut cbrush_t).offset(brushNum as isize)).contents;
    //     }
    // }
    // if( cmod->leaf.numLeafSurfaces )	// if not brush, check for patch
    // {
    //     let mut surfaceNum: c_int;
    //     for ( i = cmod->leaf.firstLeafSurface; i < cmod->leaf.firstLeafSurface+cmod->leaf.numLeafSurfaces; i++ )
    //     {
    //         surfaceNum = (*cm_ptr).leafsurfaces[i];
    //         if ( (*((*cm_ptr).surfaces as *mut *mut cPatch_t).offset(surfaceNum as isize)) != std::ptr::null_mut() )
    //         {//HERNH?  How could we have a null surf within our cmod->leaf.numLeafSurfaces?
    //             contents |= (**(*((*cm_ptr).surfaces as *mut *mut cPatch_t).offset(surfaceNum as isize))).contents;
    //         }
    //     }
    // }
    contents
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn CM_ModelContents(model: c_int, subBSPIndex: c_int) -> c_int {
    if subBSPIndex < 0 {
        return CM_ModelContents_Actual(model, std::ptr::null_mut());
    }

    CM_ModelContents_Actual(model, &mut SubBSP[subBSPIndex as usize])
}

// Stub for memset (should use Com_Memset in actual implementation)
#[allow(non_snake_case)]
unsafe fn memset(s: *mut c_void, c: c_int, len: usize) -> *mut c_void {
    Com_Memset(s, c, len)
}
