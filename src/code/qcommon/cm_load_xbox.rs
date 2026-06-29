// cmodel.c -- model loading

use core::ffi::{c_int, c_char, c_void};

// Includes translated to module references:
// #include "cm_local.h" -> local cm_local module
// #include "cm_patch.h" -> local cm_patch module
// #include "../renderer/tr_local.h" -> external renderer module
// #include "../RMG/RM_Headers.h" -> external RMG module
// #include "sparc.h" -> external sparc module
// #include "../zlib/zlib.h" -> external zlib module

// Local SPARC template instance - using a simple placeholder since we need extern C integration
#[repr(C)]
pub struct SPARC_byte {
    // Placeholder for SPARC<byte> template - actual fields depend on sparc.h port
    _opaque: *mut c_void,
}

static mut visData: SPARC_byte = SPARC_byte { _opaque: core::ptr::null_mut() };

extern "C" {
    // From Z_Malloc family
    fn Z_Malloc(size: c_int, tag: c_int, clear: bool) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Z_TagFree(tag: c_int);

    // From rendering system
    fn RE_SetWorldVisData(visdata: *mut c_void);
    fn R_LoadSurfaces(count: c_int);
    fn R_LoadPatches(verts: *mut c_void, vertlen: c_int, surfaces: *mut c_void, surfacelen: c_int);
    fn R_LoadTriSurfs(indexdata: *mut c_void, indexlen: c_int, verts: *mut c_void, vertlen: c_int, surfaces: *mut c_void, surfacelen: c_int);
    fn R_LoadFaces(indexdata: *mut c_void, indexlen: c_int, verts: *mut c_void, vertlen: c_int, surfaces: *mut c_void, surfacelen: c_int);
    fn R_LoadFlares(surfaces: *mut c_void, surfacelen: c_int);
    fn R_LoadShaders();
    fn R_LoadLightmaps(data: *mut c_void, len: c_int, psMapName: *const c_char);

    // From common system
    fn Com_Error(level: c_int, format: *const c_char, ...);
    fn Com_DPrintf(format: *const c_char, ...);
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut c_void;

    // From q_shared / string utilities
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, len: usize) -> *mut c_void;
    fn memset(ptr: *mut c_void, value: c_int, len: usize) -> *mut c_void;
    fn va(format: *const c_char, ...) -> *const c_char;

    // From zlib
    fn crc32(crc: u32, buf: *const u8, len: usize) -> u32;

    // From common utilities
    fn COM_StripExtension(in_: *const c_char, out: *mut c_char);
    fn Hunk_ClearHigh();

    // Terrain system (conditional)
    static mut TheRandomMissionManager: *mut c_void;

    // Visibility tracking
    static mut vidRestartReloadMap: bool;
    static mut fileBase: *mut u8;

    // From cm_local
    fn CM_ClearLevelPatches();
    fn PlaneTypeForNormal(normal: *const f32) -> c_int;
    fn CM_GridAlloc();
    fn CM_PatchCollideFromGridTempAlloc();
    fn CM_PreparePatchCollide(num: c_int);
    fn CM_TempPatchPlanesAlloc();
    fn CM_GeneratePatchCollide(width: c_int, height: c_int, points: *const [f32; 3], facetbuf: *mut c_void, gridbuf: *mut c_void) -> *mut c_void;
    fn CM_PatchCollideFromGridTempDealloc();
    fn CM_GridDealloc();
    fn CM_TempPatchPlanesDealloc();
    fn CM_InitBoxHull();
    fn CM_FloodAreaConnections();
    fn CM_CleanLeafCache();

    // Port-level stub: SaveGame interface
    fn SG_Append(chid: c_int, data: *const c_void, length: c_int) -> bool;
    fn SG_Read(chid: c_int, pvAddress: *mut c_void, iLength: c_int, ppvAddressPtr: *mut *mut c_void) -> c_int;
}

extern "C" {
    // World data from renderer
    pub static mut s_worldData: c_void;
}

// Forward declarations
extern "C" {
    fn CM_LoadShaderText(forceReload: bool);
}

#[cfg(BSPC)]
extern "C" {
    fn SetPlaneSignbits(out: *mut cplane_t);
}

// to allow boxes to be treated as brush models, we allocate
// some extra indexes along with those needed by the map
const BOX_BRUSHES: c_int = 1;
const BOX_SIDES: c_int = 6;
const BOX_LEAFS: c_int = 2;
const BOX_PLANES: c_int = 12;

const LL_MACRO: &str = "x=LittleLong(x)";

// Type definitions from cm_local.h - these should be imported/declared
#[repr(C)]
pub struct clipMap_t {
    // Placeholder - actual structure defined in cm_local
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cmodel_t {
    // Placeholder - actual structure defined in cm_local
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cplane_t {
    // Placeholder - actual structure defined in cm_local
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cbrush_t {
    // Placeholder - actual structure defined in cm_local
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dshader_t {
    // Placeholder - from BSP format
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CCMShader {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dmodel_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dnode_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cNode_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dbrush_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cbrushside_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dbrushside_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cLeaf_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dleaf_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cArea_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dplane_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct mapVert_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct dpatch_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cPatch_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct facetLoad_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct Lump {
    // Placeholder - data and len fields
    _opaque: [u8; 0],
}

// Global data
pub static mut cmg: clipMap_t = unsafe { core::mem::zeroed() };
pub static mut c_pointcontents: c_int = 0;
pub static mut c_traces: c_int = 0;
pub static mut c_brush_traces: c_int = 0;
pub static mut c_patch_traces: c_int = 0;

pub static mut cmod_base: *mut u8 = core::ptr::null_mut();

#[cfg(not(BSPC))]
pub static mut cm_noAreas: *mut c_void = core::ptr::null_mut();
#[cfg(not(BSPC))]
pub static mut cm_noCurves: *mut c_void = core::ptr::null_mut();
#[cfg(not(BSPC))]
pub static mut cm_playerCurveClip: *mut c_void = core::ptr::null_mut();

pub static mut box_model: cmodel_t = unsafe { core::mem::zeroed() };
pub static mut box_planes: *mut cplane_t = core::ptr::null_mut();
pub static mut box_brush: *mut cbrush_t = core::ptr::null_mut();

pub static mut CM_OrOfAllContentsFlagsInMap: c_int = 0;

pub static mut SubBSP: [clipMap_t; 4] = unsafe { [core::mem::zeroed(); 4] }; // MAX_SUB_BSP placeholder
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
#[no_mangle]
pub extern "C" fn CMod_LoadShaders(data: *mut c_void, len: c_int) {
    let mut in_: *mut dshader_t;
    let mut i: c_int;
    let mut count: c_int;
    let mut out: *mut CCMShader;

    in_ = data as *mut dshader_t;
    if len as usize % core::mem::size_of::<dshader_t>() != 0 {
        unsafe { Com_Error(0, b"CMod_LoadShaders: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<dshader_t>()) as c_int;

    if count < 1 {
        unsafe { Com_Error(0, b"Map with no shaders\0" as *const u8 as *const c_char) };
    }
    unsafe {
        let cmg_ptr = &mut cmg as *mut clipMap_t;
        // cmg.shaders = (CCMShader *) Z_Malloc( count * sizeof( *cmg.shaders ), TAG_BSP, qfalse);
        // cmg.numShaders = count;
        // ... field assignments would go here
    }

    // s_worldData.shaders = (dshader_t *) Z_Malloc ( count*sizeof(dshader_t), TAG_BSP, qfalse );
    // s_worldData.numShaders = count;

    out = unsafe { &mut cmg as *mut clipMap_t } as *mut CCMShader;
    i = 0;
    while i < count {
        unsafe {
            // Q_strncpyz(out->shader, in->shader, MAX_QPATH);
            // out->contentFlags = in->contentFlags;
            // out->surfaceFlags = in->surfaceFlags;
            // Q_strncpyz(s_worldData.shaders[i].shader, in->shader, MAX_QPATH);
            // s_worldData.shaders[i].contentFlags = in->contentFlags;
            // s_worldData.shaders[i].surfaceFlags = in->surfaceFlags;
        }
        i += 1;
        in_ = unsafe { in_.add(1) };
        out = unsafe { out.add(1) };
    }
}

/*
=================
CMod_LoadSubmodels
=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadSubmodels(data: *mut c_void, len: c_int) {
    let mut in_: *mut dmodel_t;
    let mut out: *mut cmodel_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;
    let mut indexes: *mut c_int;

    in_ = data as *mut dmodel_t;
    if len as usize % core::mem::size_of::<dmodel_t>() != 0 {
        unsafe { Com_Error(0, b"CMod_LoadSubmodels: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<dmodel_t>()) as c_int;

    if count < 1 {
        unsafe { Com_Error(0, b"Map with no models\0" as *const u8 as *const c_char) };
    }

    // if ( count > MAX_SUBMODELS ) {
    //     Com_Error( ERR_DROP, "MAX_SUBMODELS (%d) exceeded by %d", MAX_SUBMODELS, count-MAX_SUBMODELS );
    // }

    // cmg.cmodels = (struct cmodel_s *) Z_Malloc( count * sizeof( *cmg.cmodels ), TAG_BSP, qtrue );
    // cmg.numSubModels = count;

    i = 0;
    while i < count {
        // out = &cmg.cmodels[i];

        j = 0;
        while j < 3 {
            // spread the mins / maxs by a pixel
            // out->mins[j] = in->mins[j] - 1;
            // out->maxs[j] = in->maxs[j] + 1;
            j += 1;
        }

        if i == 0 {
            // world model doesn't need other info
            // (skip rest of loop)
        } else {
            // make a "leaf" just to hold the model's brushes and surfaces
            // out->leaf.numLeafBrushes = in->numBrushes;
            // indexes = (int *) Z_Malloc( out->leaf.numLeafBrushes * 4, TAG_BSP, qfalse);
            // out->leaf.firstLeafBrush = indexes - cmg.leafbrushes;
            j = 0;
            // while ( j < out->leaf.numLeafBrushes ) {
            //     indexes[j] = in->firstBrush + j;
            //     j += 1;
            // }

            // out->leaf.numLeafSurfaces = in->numSurfaces;
            // indexes = (int *) Z_Malloc( out->leaf.numLeafSurfaces * 4, TAG_BSP, qfalse);
            // out->leaf.firstLeafSurface = indexes - cmg.leafsurfaces;
            j = 0;
            // while ( j < out->leaf.numLeafSurfaces ) {
            //     indexes[j] = in->firstSurface + j;
            //     j += 1;
            // }
        }
        i += 1;
        in_ = unsafe { in_.add(1) };
    }
}

/*
=================
CMod_LoadNodes

=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadNodes(data: *mut c_void, len: c_int) {
    let mut in_: *mut dnode_t;
    let mut out: *mut cNode_t;
    let mut i: c_int;
    let mut count: c_int;

    in_ = data as *mut dnode_t;
    if len as usize % core::mem::size_of::<dnode_t>() != 0 {
        unsafe { Com_Error(0, b"MOD_LoadBmodel: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<dnode_t>()) as c_int;

    if count < 1 {
        unsafe { Com_Error(0, b"Map has no nodes\0" as *const u8 as *const c_char) };
    }
    // cmg.nodes = (cNode_t *) Z_Malloc( count * sizeof( *cmg.nodes ), TAG_BSP, qfalse);
    // cmg.numNodes = count;

    // out = cmg.nodes;

    i = 0;
    while i < count {
        // out->children[0] = in->children[0];
        // out->children[1] = in->children[1];
        i += 1;
        out = unsafe { out.add(1) };
        in_ = unsafe { in_.add(1) };
    }
}

/*
=================
CM_BoundBrush

=================
*/
#[no_mangle]
pub extern "C" fn CM_BoundBrush(b: *mut cbrush_t) {
    // b->bounds[0][0] = -cmg.planes[b->sides[0].planeNum.GetValue()].dist;
    // b->bounds[1][0] = cmg.planes[b->sides[1].planeNum.GetValue()].dist;
    // b->bounds[0][1] = -cmg.planes[b->sides[2].planeNum.GetValue()].dist;
    // b->bounds[1][1] = cmg.planes[b->sides[3].planeNum.GetValue()].dist;
    // b->bounds[0][2] = -cmg.planes[b->sides[4].planeNum.GetValue()].dist;
    // b->bounds[1][2] = cmg.planes[b->sides[5].planeNum.GetValue()].dist;
}

/*
=================
CMod_LoadBrushes

=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadBrushes(data: *mut c_void, len: c_int) {
    let mut in_: *mut dbrush_t;
    let mut out: *mut cbrush_t;
    let mut i: c_int;
    let mut count: c_int;

    in_ = data as *mut dbrush_t;
    if len as usize % core::mem::size_of::<dbrush_t>() != 0 {
        unsafe { Com_Error(0, b"MOD_LoadBmodel: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<dbrush_t>()) as c_int;

    // cmg.brushes = (cbrush_t *) Z_Malloc( ( BOX_BRUSHES + count ) * sizeof( *cmg.brushes ), TAG_BSP, qfalse);
    // cmg.numBrushes = count;

    // out = cmg.brushes;

    i = 0;
    while i < count {
        // out->sides = cmg.brushsides + in->firstSide;
        // out->numsides = in->numSides;
        // out->shaderNum = in->shaderNum;
        // if ( out->shaderNum < 0 || out->shaderNum >= cmg.numShaders ) {
        //     Com_Error( ERR_DROP, "CMod_LoadBrushes: bad shaderNum: %i", out->shaderNum );
        // }
        // out->contents = cmg.shaders[out->shaderNum].contentFlags;
        // TEMP HACK: for water that cuts vis but is not solid!!!
        // if ( cmg.shaders[out->shaderNum].surfaceFlags & SURF_SLICK )
        // {
        //     out->contents &= ~CONTENTS_SOLID;
        // }
        // CM_OrOfAllContentsFlagsInMap |= out->contents;
        // CM_BoundBrush( out );
        i += 1;
        out = unsafe { out.add(1) };
        in_ = unsafe { in_.add(1) };
    }
}

/*
=================
CMod_LoadLeafs
=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadLeafs(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut out: *mut cLeaf_t;
    let mut in_: *mut dleaf_t;
    let mut count: c_int;

    in_ = data as *mut dleaf_t;
    if len as usize % core::mem::size_of::<dleaf_t>() != 0 {
        unsafe { Com_Error(0, b"MOD_LoadBmodel: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<dleaf_t>()) as c_int;

    if count < 1 {
        unsafe { Com_Error(0, b"Map with no leafs\0" as *const u8 as *const c_char) };
    }

    // cmg.leafs = (cLeaf_t *) Z_Malloc( ( BOX_LEAFS + count ) * sizeof( *cmg.leafs ), TAG_BSP, qfalse);
    // cmg.numLeafs = count;
    // out = cmg.leafs;

    i = 0;
    while i < count {
        // out->cluster = in->cluster;
        // out->area = in->area;
        // out->firstLeafBrush = in->firstLeafBrush;
        // out->numLeafBrushes = in->numLeafBrushes;
        // out->firstLeafSurface = in->firstLeafSurface;
        // out->numLeafSurfaces = in->numLeafSurfaces;
        // if (out->cluster >= cmg.numClusters)
        //     cmg.numClusters = out->cluster + 1;
        // if (out->area >= cmg.numAreas)
        //     cmg.numAreas = out->area + 1;
        i += 1;
        in_ = unsafe { in_.add(1) };
        out = unsafe { out.add(1) };
    }

    // cmg.areas = (cArea_t *) Z_Malloc( cmg.numAreas * sizeof( *cmg.areas ), TAG_BSP, qtrue );

    unsafe {
        if !vidRestartReloadMap {
            // cmg.areaPortals = (int *) Z_Malloc( cmg.numAreas * cmg.numAreas * sizeof( *cmg.areaPortals ), TAG_BSP, qtrue );
        }
    }
}

/*
=================
CMod_LoadPlanes
=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadPlanes(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut out: *mut cplane_t;
    let mut in_: *mut dplane_t;
    let mut count: c_int;
    let mut bits: c_int;

    in_ = data as *mut dplane_t;
    if len as usize % core::mem::size_of::<dplane_t>() != 0 {
        unsafe { Com_Error(0, b"MOD_LoadBmodel: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<dplane_t>()) as c_int;

    if count < 1 {
        unsafe { Com_Error(0, b"Map with no planes\0" as *const u8 as *const c_char) };
    }
    // cmg.planes = (struct cplane_s *) Z_Malloc( ( BOX_PLANES + count ) * sizeof( *cmg.planes ), TAG_BSP, qfalse);
    // cmg.numPlanes = count;

    // out = cmg.planes;

    i = 0;
    while i < count {
        bits = 0;
        j = 0;
        while j < 3 {
            // out->normal[j] = in->normal[j];
            // if (out->normal[j] < 0)
            //     bits |= 1<<j;
            j += 1;
        }

        // out->dist = in->dist;
        // out->type = PlaneTypeForNormal( out->normal );
        // out->signbits = bits;
        i += 1;
        in_ = unsafe { in_.add(1) };
        out = unsafe { out.add(1) };
    }
}

/*
=================
CMod_LoadLeafBrushes
=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadLeafBrushes(data: *mut c_void, len: c_int) {
    let mut out: *mut c_int;
    let mut in_: *mut c_int;
    let mut count: c_int;

    in_ = data as *mut c_int;
    if len as usize % core::mem::size_of::<c_int>() != 0 {
        unsafe { Com_Error(0, b"MOD_LoadBmodel: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<c_int>()) as c_int;

    // cmg.leafbrushes = (int *) Z_Malloc( ( BOX_BRUSHES + count ) * sizeof( *cmg.leafbrushes ), TAG_BSP, qfalse);
    // cmg.numLeafBrushes = count;

    // out = cmg.leafbrushes;

    unsafe {
        memcpy(out as *mut c_void, in_ as *const c_void, len as usize);
    }
}

/*
=================
CMod_LoadBrushSides
=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadBrushSides(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut out: *mut cbrushside_t;
    let mut in_: *mut dbrushside_t;
    let mut count: c_int;

    in_ = data as *mut dbrushside_t;
    if len as usize % core::mem::size_of::<dbrushside_t>() != 0 {
        unsafe { Com_Error(0, b"MOD_LoadBmodel: funny lump size\0" as *const u8 as *const c_char) };
    }
    count = (len as usize / core::mem::size_of::<dbrushside_t>()) as c_int;

    // cmg.brushsides = (cbrushside_t *) Z_Malloc( ( BOX_SIDES + count ) * sizeof( *cmg.brushsides ), TAG_BSP, qfalse);
    // cmg.numBrushSides = count;

    // out = cmg.brushsides;

    i = 0;
    while i < count {
        // out->planeNum = in->planeNum;
        // assert(in->planeNum == out->planeNum.GetValue());
        // out->shaderNum = in->shaderNum;
        // if ( out->shaderNum < 0 || out->shaderNum >= cmg.numShaders ) {
        //     Com_Error( ERR_DROP, "CMod_LoadBrushSides: bad shaderNum: %i", out->shaderNum );
        // }
        i += 1;
        in_ = unsafe { in_.add(1) };
        out = unsafe { out.add(1) };
    }
}

/*
=================
CMod_LoadEntityString
=================
*/
#[no_mangle]
pub extern "C" fn CMod_LoadEntityString(data: *mut c_void, len: c_int) {
    // cmg.entityString = (char *) Z_Malloc( len, TAG_BSP, qfalse);
    // cmg.numEntityChars = len;
    unsafe {
        memcpy(core::ptr::null_mut() as *mut c_void, data, len as usize);
    }
}

/*
=================
CMod_LoadVisibility
=================
*/
const VIS_HEADER: usize = 8;

#[no_mangle]
pub extern "C" fn CMod_LoadVisibility(data: *mut c_void, len: c_int) {
    let buf: *mut c_char;

    if len == 0 {
        // cmg.visibility = NULL;
        return;
    }
    buf = data as *mut c_char;

    // visData.SetAllocator(SparcAllocator, SparcDeallocator);

    // cmg.vised = qtrue;
    // cmg.numClusters = ((int *)buf)[0];
    // cmg.clusterBytes = ((int *)buf)[1];
    // visData.Load(buf + VIS_HEADER, len - VIS_HEADER);
    // cmg.visibility = &visData;
    unsafe {
        RE_SetWorldVisData(unsafe { core::ptr::addr_of_mut!(visData) as *mut c_void });
    }
}

//==================================================================

/*
=================
CMod_LoadPatches
=================
*/
const MAX_PATCH_VERTS: c_int = 1024;

#[no_mangle]
pub extern "C" fn CMod_LoadPatches(verts: *mut c_void, vertlen: c_int, surfaces: *mut c_void, surfacelen: c_int, numsurfs: c_int) {
    let mut dv: *mut mapVert_t;
    let mut dv_p: *mut mapVert_t;
    let mut in_: *mut dpatch_t;
    let mut count: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut c: c_int;
    let mut patch: *mut cPatch_t;
    let mut points: [[f32; 3]; 1024];
    let mut width: c_int;
    let mut height: c_int;
    let mut shaderNum: c_int;

    count = surfacelen as usize / core::mem::size_of::<dpatch_t>() as c_int;

    // cmg.numSurfaces = numsurfs;
    // cmg.surfaces = (cPatch_t **) Z_Malloc( cmg.numSurfaces * sizeof( cmg.surfaces[0] ), TAG_BSP, qtrue );

    dv = verts as *mut mapVert_t;
    if vertlen as usize % core::mem::size_of::<mapVert_t>() != 0 {
        unsafe { Com_Error(0, b"MOD_LoadBmodel: funny lump size\0" as *const u8 as *const c_char) };
    }

    let patchScratch: *mut u8 = unsafe { Z_Malloc((core::mem::size_of::<cPatch_t>() * count as usize) as c_int, 0, true) as *mut u8 };

    unsafe {
        CM_GridAlloc();
        CM_PatchCollideFromGridTempAlloc();
        CM_PreparePatchCollide(count);
        CM_TempPatchPlanesAlloc();
    }

    let facetbuf: *mut facetLoad_t = unsafe {
        Z_Malloc(
            (1024 * core::mem::size_of::<facetLoad_t>()) as c_int,
            0,
            false,
        ) as *mut facetLoad_t
    };

    let gridbuf: *mut c_int = unsafe {
        Z_Malloc(
            (256 * 256 * 2 * core::mem::size_of::<c_int>()) as c_int,
            0,
            false,
        ) as *mut c_int
    };

    i = 0;
    while i < count {
        in_ = unsafe { (surfaces as *mut dpatch_t).add(i as usize) };

        // cmg.surfaces[ in->code ] = patch = (cPatch_t *) patchScratch;
        patch = patchScratch as *mut cPatch_t;
        // patchScratch += sizeof( *patch );

        // load the full drawverts onto the stack
        // width = in->patchWidth;
        // height = in->patchHeight;
        c = width * height;
        if c > MAX_PATCH_VERTS {
            unsafe { Com_Error(0, b"ParseMesh: MAX_PATCH_VERTS\0" as *const u8 as *const c_char) };
        }

        // dv_p = dv + (in->verts >> 12);
        j = 0;
        while j < c {
            // points[j][0] = dv_p->xyz[0];
            // points[j][1] = dv_p->xyz[1];
            // points[j][2] = dv_p->xyz[2];
            j += 1;
            dv_p = unsafe { dv_p.add(1) };
        }

        // shaderNum = in->shaderNum;
        // patch->contents = cmg.shaders[shaderNum].contentFlags;
        // CM_OrOfAllContentsFlagsInMap |= patch->contents;
        // patch->surfaceFlags = cmg.shaders[shaderNum].surfaceFlags;

        // create the internal facet structure
        // patch->pc = CM_GeneratePatchCollide( width, height, points, facetbuf, gridbuf );
        i += 1;
    }

    unsafe {
        CM_PatchCollideFromGridTempDealloc();
        CM_GridDealloc();
        CM_TempPatchPlanesDealloc();

        Z_Free(gridbuf as *mut c_void);
        Z_Free(facetbuf as *mut c_void);
    }
}

//==================================================================

#[cfg(BSPC)]
#[no_mangle]
pub extern "C" fn CM_FreeMap() {
    unsafe {
        memset(core::ptr::addr_of_mut!(cmg) as *mut c_void, 0, core::mem::size_of::<clipMap_t>());
        Hunk_ClearHigh();
        CM_ClearLevelPatches();
    }
}

/*
==================
CM_LoadMap

Loads in the map and all submodels
==================
*/
pub static mut gpvCachedMapDiskImage: *mut c_void = core::ptr::null_mut();
pub static mut gsCachedMapDiskImage: [c_char; 260] = [0; 260]; // MAX_QPATH
pub static mut gbUsingCachedMapDataRightNow: bool = false; // if true, signifies that you can't delete this at the moment!! (used during z_malloc()-fail recovery attempt)

// called in response to a "devmapbsp blah" or "devmapall blah" command, do NOT use inside CM_Load unless you pass in qtrue
//
// new bool return used to see if anything was freed, used during z_malloc failure re-try
//
#[no_mangle]
pub extern "C" fn CM_DeleteCachedMap(bGuaranteedOkToDelete: bool) -> bool {
    let mut bActuallyFreedSomething: bool = false;

    unsafe {
        if bGuaranteedOkToDelete || !gbUsingCachedMapDataRightNow {
            // dump cached disk image...
            //
            if !gpvCachedMapDiskImage.is_null() {
                Z_Free(gpvCachedMapDiskImage);
                gpvCachedMapDiskImage = core::ptr::null_mut();

                bActuallyFreedSomething = true;
            }
            gsCachedMapDiskImage[0] = 0;

            // force map loader to ignore cached internal BSP structures for next level CM_LoadMap() call...
            //
            // cmg.name[0] = '\0';
        }
    }

    return bActuallyFreedSomething;
}

#[no_mangle]
pub extern "C" fn CM_Free() {
    unsafe {
        CM_ClearLevelPatches();
        // visData.Release();
        Z_TagFree(0); // TAG_BSP
    }
}

#[no_mangle]
pub extern "C" fn CM_LoadMap_Actual(name: *const c_char, clientload: bool, checksum: *mut c_int) {
    // let mut buf: *const c_int = core::ptr::null();
    // let mut surfBuf: *const c_int = core::ptr::null();
    let mut last_checksum: u32 = 0;
    let mut lmName: [c_char; 260] = [0; 260]; // MAX_QPATH
    let mut stripName: [c_char; 260] = [0; 260]; // MAX_QPATH
    // let mut outputLump: Lump;

    unsafe {
        if name.is_null() || *name == 0 {
            Com_Error(0, b"CM_LoadMap: NULL name\0" as *const u8 as *const c_char);
        }

        #[cfg(not(BSPC))]
        {
            cm_noAreas = Cvar_Get(b"cm_noAreas\0" as *const u8 as *const c_char, b"0\0" as *const u8 as *const c_char, 0);
            cm_noCurves = Cvar_Get(b"cm_noCurves\0" as *const u8 as *const c_char, b"0\0" as *const u8 as *const c_char, 0);
            cm_playerCurveClip = Cvar_Get(b"cm_playerCurveClip\0" as *const u8 as *const c_char, b"1\0" as *const u8 as *const c_char, 0);
        }

        Com_DPrintf(b"CM_LoadMap( %s, %i )\n\0" as *const u8 as *const c_char, name, clientload as c_int);

        // if ( !strcmp( cmg.name, name ) && clientload ) {
        //     *checksum = last_checksum;
        //     return;
        // }

        // free old stuff
        let mut ap: *mut c_int = core::ptr::null_mut();
        if vidRestartReloadMap {
            // ap = cmg.areaPortals;
        }
        memset(core::ptr::addr_of_mut!(cmg) as *mut c_void, 0, core::mem::size_of::<clipMap_t>());
        if vidRestartReloadMap {
            // cmg.areaPortals = ap;
        }

        if *name == 0 {
            // cmg.numLeafs = 1;
            // cmg.numClusters = 1;
            // cmg.numAreas = 1;
            // cmg.cmodels = (struct cmodel_s *) Z_Malloc( sizeof( *cmg.cmodels ), TAG_BSP, qtrue );
            *checksum = 0;
            return;
        }

        last_checksum = crc32(0, name as *const u8, strlen(name));
        COM_StripExtension(name, stripName.as_mut_ptr());

        // load into heap
        // outputLump.load(stripName, "shaders");
        // CMod_LoadShaders( outputLump.data, outputLump.len );
        // R_LoadShaders();

        strcpy(lmName.as_mut_ptr(), name);
        // outputLump.load(stripName, "lightmaps");
        // R_LoadLightmaps( outputLump.data, outputLump.len, lmName);

        {
            fileBase = core::ptr::null_mut();
            // outputLump.clear();

            // Lump misc;
            // misc.load(stripName, "misc");

            // int num_surfs = *(int*)misc.data;
            // misc.clear();

            // R_LoadSurfaces(num_surfs);

            // Lump verts;
            // verts.load(stripName, "verts");

            // Lump patches;
            // patches.load(stripName, "patches");

            // CMod_LoadPatches(verts.data, verts.len,
            //     patches.data, patches.len,
            //     num_surfs );
            // R_LoadPatches(verts.data, verts.len,
            //     patches.data, patches.len);

            // patches.clear();

            // Lump indexes;
            // indexes.load(stripName, "indexes");

            // Lump trisurfs;
            // trisurfs.load(stripName, "trisurfs");

            // R_LoadTriSurfs(indexes.data, indexes.len,
            //     verts.data, verts.len,
            //     trisurfs.data, trisurfs.len);

            // trisurfs.clear();

            // Lump faces;
            // faces.load(stripName, "faces");

            // R_LoadFaces(indexes.data, indexes.len,
            //     verts.data, verts.len,
            //     faces.data, faces.len);

            // Lump flares;
            // flares.load(stripName, "flares");

            // R_LoadFlares(flares.data, flares.len);
        }

        // outputLump.load(stripName, "leafs");
        // CMod_LoadLeafs (outputLump.data, outputLump.len);

        // outputLump.load(stripName, "leafbrushes");
        // CMod_LoadLeafBrushes (outputLump.data, outputLump.len);

        // cmg.leafsurfaces = NULL;
        // outputLump.load(stripName, "planes");
        // CMod_LoadPlanes (outputLump.data, outputLump.len);

        // outputLump.load(stripName, "brushsides");
        // CMod_LoadBrushSides (outputLump.data, outputLump.len);
        // outputLump.load(stripName, "brushes");
        // CMod_LoadBrushes (outputLump.data, outputLump.len);

        // outputLump.load(stripName, "models");
        // CMod_LoadSubmodels (outputLump.data, outputLump.len);

        // outputLump.load(stripName, "nodes");
        // CMod_LoadNodes (outputLump.data, outputLump.len);

        // outputLump.load(stripName, "entities");
        // CMod_LoadEntityString (outputLump.data, outputLump.len);

        // outputLump.load(stripName, "visibility");
        // CMod_LoadVisibility( outputLump.data, outputLump.len);

        // TotalSubModels += cmg.numSubModels;

        CM_InitBoxHull();

        *checksum = last_checksum as c_int;

        // do this whether or not the map was cached from last load...
        //
        CM_FloodAreaConnections();

        // allow this to be cached if it is loaded by the server
        if !clientload {
            // Q_strncpyz( cmg.name, name, sizeof( cmg.name ) );
        }
        CM_CleanLeafCache();
    }
}

// need a wrapper function around this because of multiple returns, need to ensure bool is correct...
//
#[no_mangle]
pub extern "C" fn CM_LoadMap(name: *const c_char, clientload: bool, checksum: *mut c_int) {
    CM_LoadMap_Actual(name, clientload, checksum);
}

#[no_mangle]
pub extern "C" fn CM_SameMap(server: *mut c_char) -> bool {
    unsafe {
        if (cmg as *const clipMap_t as usize) == 0 || server.is_null() || *server == 0 {
            return false;
        }

        // if (Q_stricmp(cmg.name, va("maps/%s.bsp", server)))
        // {
        //     return false;
        // }

        return true;
    }
}

#[cfg(not(target_os = "windows"))]
#[no_mangle]
pub extern "C" fn CM_HasTerrain() -> bool {
    // if (cmg.landScape)
    //     return true;
    false
}

/*
==================
CM_ClearMap
==================
*/
#[no_mangle]
pub extern "C" fn CM_ClearMap() {
    let mut i: c_int;

    unsafe {
        CM_OrOfAllContentsFlagsInMap = 2; // CONTENTS_BODY

        #[cfg(not(BSPC))]
        {
            // CM_ShutdownShaderProperties();
            // MAT_Shutdown();
        }

        #[cfg(not(target_os = "windows"))]
        {
            if !TheRandomMissionManager.is_null() {
                // delete TheRandomMissionManager;
                TheRandomMissionManager = core::ptr::null_mut();
            }

            // if (cmg.landScape)
            // {
            //     delete cmg.landScape;
            //     cmg.landScape = 0;
            // }
        }

        memset(core::ptr::addr_of_mut!(cmg) as *mut c_void, 0, core::mem::size_of::<clipMap_t>());
        CM_ClearLevelPatches();

        i = 0;
        while i < NumSubBSP {
            memset(core::ptr::addr_of_mut!(SubBSP[i as usize]) as *mut c_void, 0, core::mem::size_of::<clipMap_t>());
            i += 1;
        }
        NumSubBSP = 0;
        TotalSubModels = 0;
    }
}

#[no_mangle]
pub extern "C" fn CM_TotalMapContents() -> c_int {
    unsafe { CM_OrOfAllContentsFlagsInMap }
}

/*
==================
CM_ClipHandleToModel
==================
*/
#[no_mangle]
pub extern "C" fn CM_ClipHandleToModel(handle: c_int, clipMap: *mut *mut clipMap_t) -> *mut cmodel_t {
    let mut i: c_int;
    let mut count: c_int;

    if handle < 0 {
        unsafe { Com_Error(0, b"CM_ClipHandleToModel: bad handle %i\0" as *const u8 as *const c_char, handle) };
    }

    unsafe {
        if handle < 0 {
            // cmg.numSubModels placeholder - would come from actual structure
            if !clipMap.is_null() {
                *clipMap = core::ptr::addr_of_mut!(cmg);
            }
            // return &cmg.cmodels[handle];
        }
        if handle == 0 {
            // BOX_MODEL_HANDLE
            if !clipMap.is_null() {
                *clipMap = core::ptr::addr_of_mut!(cmg);
            }
            return core::ptr::addr_of_mut!(box_model);
        }

        count = 0; // cmg.numSubModels
        i = 0;
        while i < NumSubBSP {
            // if (handle < count + SubBSP[i].numSubModels)
            // {
            //     if (clipMap)
            //     {
            //         *clipMap = &SubBSP[i];
            //     }
            //     return &SubBSP[i].cmodels[handle - count];
            // }
            // count += SubBSP[i].numSubModels;
            i += 1;
        }

        // if ( handle < MAX_SUBMODELS )
        // {
        //     Com_Error( ERR_DROP, "CM_ClipHandleToModel: bad handle %i < %i < %i",
        //         cmg.numSubModels, handle, MAX_SUBMODELS );
        // }
        Com_Error(0, b"CM_ClipHandleToModel: bad handle %i\0" as *const u8 as *const c_char, handle);
    }

    core::ptr::null_mut()
}

/*
==================
CM_InlineModel
==================
*/
#[no_mangle]
pub extern "C" fn CM_InlineModel(index: c_int) -> c_int {
    unsafe {
        if index < 0 || index >= TotalSubModels {
            Com_Error(0, b"CM_InlineModel: bad number (may need to re-BSP map?)\0" as *const u8 as *const c_char);
        }
    }
    index
}

#[no_mangle]
pub extern "C" fn CM_NumClusters() -> c_int {
    0 // cmg.numClusters
}

#[no_mangle]
pub extern "C" fn CM_NumInlineModels() -> c_int {
    0 // cmg.numSubModels
}

#[no_mangle]
pub extern "C" fn CM_EntityString() -> *mut c_char {
    core::ptr::null_mut() // cmg.entityString
}

#[no_mangle]
pub extern "C" fn CM_SubBSPEntityString(index: c_int) -> *mut c_char {
    unsafe {
        // SubBSP[index as usize].entityString
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn CM_LeafCluster(leafnum: c_int) -> c_int {
    unsafe {
        if leafnum < 0 {
            // || leafnum >= cmg.numLeafs
            Com_Error(0, b"CM_LeafCluster: bad number\0" as *const u8 as *const c_char);
        }
        // return cmg.leafs[leafnum].cluster;
        0
    }
}

#[no_mangle]
pub extern "C" fn CM_LeafArea(leafnum: c_int) -> c_int {
    unsafe {
        if leafnum < 0 {
            // || leafnum >= cmg.numLeafs
            Com_Error(0, b"CM_LeafArea: bad number\0" as *const u8 as *const c_char);
        }
        // return cmg.leafs[leafnum].area;
        0
    }
}

//=======================================================================

/*
===================
CM_InitBoxHull

Set up the planes and nodes so that the six floats of a bounding box
can just be stored out and get a proper clipping hull structure.
===================
*/
#[no_mangle]
pub extern "C" fn CM_InitBoxHull_impl() {
    let mut i: c_int;
    let mut side: c_int;
    let mut p: *mut cplane_t;
    let mut s: *mut cbrushside_t;

    unsafe {
        box_planes = core::ptr::addr_of_mut!(cmg) as *mut cplane_t; // &cmg.planes[cmg.numPlanes];

        box_brush = core::ptr::addr_of_mut!(cmg) as *mut cbrush_t; // &cmg.brushes[cmg.numBrushes];
        // box_brush->numsides = 6;
        // box_brush->sides = cmg.brushsides + cmg.numBrushSides;
        // box_brush->contents = CONTENTS_BODY;

        // box_model.leaf.numLeafBrushes = 1;
        // box_model.leaf.firstLeafBrush = cmg.numLeafBrushes;
        // cmg.leafbrushes[cmg.numLeafBrushes] = cmg.numBrushes;

        i = 0;
        while i < 6 {
            side = i & 1;

            // brush sides
            s = core::ptr::addr_of_mut!(cmg) as *mut cbrushside_t; // &cmg.brushsides[cmg.numBrushSides+i];
            // s->planeNum = cmg.numPlanes+i*2+side;
            // s->shaderNum = cmg.numShaders;

            // planes
            p = unsafe { box_planes.add((i * 2) as usize) };
            // p->type = i>>1;
            // p->signbits = 0;
            // VectorClear (p->normal);
            // p->normal[i>>1] = 1;

            p = unsafe { box_planes.add((i * 2 + 1) as usize) };
            // p->type = 3 + (i>>1);
            // p->signbits = 0;
            // VectorClear (p->normal);
            // p->normal[i>>1] = -1;

            // SetPlaneSignbits( p );
            i += 1;
        }
    }
}

/*
===================
CM_HeadnodeForBox

To keep everything totally uniform, bounding boxes are turned into small
BSP trees instead of being compared directly.
===================
*/
#[no_mangle]
pub extern "C" fn CM_TempBoxModel(mins: *const [f32; 3], maxs: *const [f32; 3]) -> c_int {
    unsafe {
        // box_planes[0].dist = maxs[0];
        // box_planes[1].dist = -maxs[0];
        // box_planes[2].dist = mins[0];
        // box_planes[3].dist = -mins[0];
        // box_planes[4].dist = maxs[1];
        // box_planes[5].dist = -maxs[1];
        // box_planes[6].dist = mins[1];
        // box_planes[7].dist = -mins[1];
        // box_planes[8].dist = maxs[2];
        // box_planes[9].dist = -maxs[2];
        // box_planes[10].dist = mins[2];
        // box_planes[11].dist = -mins[2];

        // VectorCopy( mins, box_brush->bounds[0] );
        // VectorCopy( maxs, box_brush->bounds[1] );

        //FIXME: this is the "correct" way, but not the way JK2 was designed around... fix for further projects
        // box_brush->contents = contents;

        0 // BOX_MODEL_HANDLE
    }
}

/*
===================
CM_ModelBounds
===================
*/
#[no_mangle]
pub extern "C" fn CM_ModelBounds(cmg_ref: *mut clipMap_t, model: c_int, mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
    let mut cmod: *mut cmodel_t;

    cmod = CM_ClipHandleToModel(model, core::ptr::null_mut());
    // VectorCopy( cmod->mins, mins );
    // VectorCopy( cmod->maxs, maxs );
}

/*
===================
CM_RegisterTerrain

Allows physics to examine the terrain data.
===================
*/
#[cfg(not(BSPC))]
#[cfg(not(target_os = "windows"))]
#[no_mangle]
pub extern "C" fn CM_RegisterTerrain(_config: *const c_char, _server: bool) -> *mut c_void {
    // Terrain removed on Xbox
    core::ptr::null_mut()
}

/*
===================
CM_ShutdownTerrain
===================
*/
#[cfg(not(BSPC))]
#[cfg(not(target_os = "windows"))]
#[no_mangle]
pub extern "C" fn CM_ShutdownTerrain(_terrainId: c_int) {
    // Terrain removed on Xbox
}

#[no_mangle]
pub extern "C" fn CM_LoadSubBSP(name: *const c_char, clientload: bool) -> c_int {
    let mut i: c_int;
    let mut checksum: c_int = 0;
    let mut count: c_int;

    unsafe {
        count = 0; // cmg.numSubModels
        i = 0;
        while i < NumSubBSP {
            if stricmp(name, core::ptr::null()) == 0 {
                // SubBSP[i as usize].name
                return count;
            }
            // count += SubBSP[i as usize].numSubModels;
            i += 1;
        }

        // if (NumSubBSP == MAX_SUB_BSP)
        // {
        //     Com_Error (ERR_DROP, "CM_LoadSubBSP: too many unique sub BSPs");
        // }

        #[cfg(target_os = "windows")]
        {
            // assert(0); // MATT! - testing now - fix this later!
        }
        #[cfg(not(target_os = "windows"))]
        {
            // CM_LoadMap_Actual( name, clientload, &checksum, SubBSP[NumSubBSP as usize] );
        }
        // NumSubBSP++;

        return count;
    }
}

#[no_mangle]
pub extern "C" fn CM_FindSubBSP(modelIndex: c_int) -> c_int {
    let mut i: c_int;
    let mut count: c_int;

    unsafe {
        count = 0; // cmg.numSubModels
        if modelIndex < count {
            // belongs to the main bsp
            return -1;
        }

        i = 0;
        while i < NumSubBSP {
            // count += SubBSP[i as usize].numSubModels;
            if modelIndex < count {
                return i;
            }
            i += 1;
        }
        return -1;
    }
}

#[no_mangle]
pub extern "C" fn CM_GetWorldBounds(mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
    unsafe {
        // VectorCopy ( cmg.cmodels[0].mins, mins );
        // VectorCopy ( cmg.cmodels[0].maxs, maxs );
    }
}

#[no_mangle]
pub extern "C" fn CM_ModelContents_Actual(model: c_int, cm: *mut clipMap_t) -> c_int {
    let mut cmod: *mut cmodel_t;
    let mut contents: c_int = 0;
    let mut i: c_int;

    unsafe {
        let cm_ptr = if cm.is_null() {
            core::ptr::addr_of_mut!(cmg)
        } else {
            cm
        };

        cmod = CM_ClipHandleToModel(model, core::ptr::null_mut());

        //MCG ADDED - return the contents, too
        // if( cmod->leaf.numLeafBrushes )		// check for brush
        // {
        //     int brushNum;
        //     for ( i = cmod->leaf.firstLeafBrush; i < cmod->leaf.firstLeafBrush+cmod->leaf.numLeafBrushes; i++ )
        //     {
        //         brushNum = cm->leafbrushes[i];
        //         contents |= cm->brushes[brushNum].contents;
        //     }
        // }
        // if( cmod->leaf.numLeafSurfaces )	// if not brush, check for patch
        // {
        //     int surfaceNum;
        //     for ( i = cmod->leaf.firstLeafSurface; i < cmod->leaf.firstLeafSurface+cmod->leaf.numLeafSurfaces; i++ )
        //     {
        //         surfaceNum = cm->leafsurfaces[i];
        //         if ( cm->surfaces[surfaceNum] != NULL )
        //         {//HERNH?  How could we have a null surf within our cmod->leaf.numLeafSurfaces?
        //             contents |= cm->surfaces[surfaceNum]->contents;
        //         }
        //     }
        // }
    }
    contents
}

#[no_mangle]
pub extern "C" fn CM_ModelContents(model: c_int, subBSPIndex: c_int) -> c_int {
    unsafe {
        if subBSPIndex < 0 {
            return CM_ModelContents_Actual(model, core::ptr::null_mut());
        }

        return CM_ModelContents_Actual(model, core::ptr::addr_of_mut!(SubBSP[subBSPIndex as usize]));
    }
}

//support for save/load games
/*
===================
CM_WritePortalState

Writes the portal state to a savegame file
===================
*/
// having to proto this stuff again here is crap, but wtf?...
//

#[no_mangle]
pub extern "C" fn CM_WritePortalState() {
    unsafe {
        SG_Append(
            b'PRTS' as c_int,
            core::ptr::addr_of!(cmg) as *const c_void,
            // cmg.numAreas * cmg.numAreas * sizeof( *cmg.areaPortals )
            0,
        );
    }
}

/*
===================
CM_ReadPortalState

Reads the portal state from a savegame file
and recalculates the area connections
===================
*/
#[no_mangle]
pub extern "C" fn CM_ReadPortalState() {
    unsafe {
        SG_Read(
            b'PRTS' as c_int,
            core::ptr::addr_of_mut!(cmg) as *mut c_void,
            // cmg.numAreas * cmg.numAreas * sizeof( *cmg.areaPortals )
            0,
            core::ptr::null_mut(),
        );
        CM_FloodAreaConnections();
    }
}
