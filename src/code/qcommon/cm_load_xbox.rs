// cmodel.c -- model loading

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types,
         dead_code, unused_variables, unused_mut, unused_imports,
         unused_assignments, unused_unsafe)]

use crate::code::qcommon::cm_local_h::*;
use crate::code::qcommon::cm_patch_h::*;
use crate::code::renderer::tr_local_h::*;
use crate::code::RMG::RM_Headers_h::*;
use crate::code::qcommon::sparc_h::*;
use crate::code::zlib::zlib_h::*;
use core::ffi::{c_int, c_uint, c_char, c_void, c_ulong};
use core::mem::size_of;

// porting note: SPARC<byte> is a C++ template instance; imported via sparc_h glob and used
// as SPARC<byte> (trusting sparc_h exposes a generic SPARC<T>).
// C++ default-constructs the static; mirrored as zeroed static for blind-port compatibility.
static mut visData: SPARC<byte> = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "C" fn SparcAllocator(size: c_uint) -> *mut c_void {
    Z_Malloc(size as c_int, TAG_BSP, false)
}

#[no_mangle]
pub unsafe extern "C" fn SparcDeallocator(ptr: *mut c_void) {
    Z_Free(ptr);
}

// extern world_t s_worldData;   -- comes via tr_local_h glob import

extern "C" {
    fn CM_LoadShaderText(forceReload: bool);
}

#[cfg(feature = "bspc")]
pub unsafe fn SetPlaneSignbits(out: *mut cplane_t) {
    let mut bits: c_int;
    let mut j: c_int;

    // for fast box on planeside test
    bits = 0;
    j = 0;
    while j < 3 {
        if (*out).normal[j as usize] < 0.0 {
            bits |= 1 << j;
        }
        j += 1;
    }
    (*out).signbits = bits;
}

// to allow boxes to be treated as brush models, we allocate
// some extra indexes along with those needed by the map
const BOX_BRUSHES: c_int = 1;
const BOX_SIDES:   c_int = 6;
const BOX_LEAFS:   c_int = 2;
const BOX_PLANES:  c_int = 12;

// #define LL(x) x=LittleLong(x)
macro_rules! LL {
    ($x:expr) => { $x = LittleLong($x) };
}

pub static mut cmg: clipMap_t = unsafe { core::mem::zeroed() };
pub static mut c_pointcontents: c_int = 0;
pub static mut c_traces:        c_int = 0;
pub static mut c_brush_traces:  c_int = 0;
pub static mut c_patch_traces:  c_int = 0;

pub static mut cmod_base: *mut byte = core::ptr::null_mut();

#[cfg(not(feature = "bspc"))]
pub static mut cm_noAreas:         *mut cvar_t = core::ptr::null_mut();
#[cfg(not(feature = "bspc"))]
pub static mut cm_noCurves:        *mut cvar_t = core::ptr::null_mut();
#[cfg(not(feature = "bspc"))]
pub static mut cm_playerCurveClip: *mut cvar_t = core::ptr::null_mut();

pub static mut box_model:  cmodel_t      = unsafe { core::mem::zeroed() };
pub static mut box_planes: *mut cplane_t = core::ptr::null_mut();
pub static mut box_brush:  *mut cbrush_t = core::ptr::null_mut();

pub static mut CM_OrOfAllContentsFlagsInMap: c_int = 0;

// void CM_InitBoxHull (void);       -- defined below
// void CM_FloodAreaConnections (void); -- comes via cm_local_h glob

pub static mut SubBSP:         [clipMap_t; MAX_SUB_BSP as usize] = unsafe { core::mem::zeroed() };
pub static mut NumSubBSP:      c_int = 0;
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
pub unsafe extern "C" fn CMod_LoadShaders(data: *mut c_void, len: c_int) {
    let mut in_: *mut dshader_t;
    let mut i: c_int;
    let mut count: c_int;
    let mut out: *mut CCMShader;

    in_ = data as *mut dshader_t;
    if len as usize % size_of::<dshader_t>() != 0 {
        Com_Error(ERR_DROP, b"CMod_LoadShaders: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<dshader_t>()) as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no shaders\0".as_ptr() as *const c_char);
    }
    cmg.shaders = Z_Malloc(
        (count as usize * size_of::<CCMShader>()) as c_int, TAG_BSP, qfalse,
    ) as *mut CCMShader;
    cmg.numShaders = count;

    s_worldData.shaders = Z_Malloc(
        (count as usize * size_of::<dshader_t>()) as c_int, TAG_BSP, qfalse,
    ) as *mut dshader_t;
    s_worldData.numShaders = count;

    out = cmg.shaders;
    i = 0;
    while i < count {
        Q_strncpyz((*out).shader.as_mut_ptr(), (*in_).shader.as_ptr(), MAX_QPATH);
        (*out).contentFlags = (*in_).contentFlags;
        (*out).surfaceFlags = (*in_).surfaceFlags;

        Q_strncpyz((*s_worldData.shaders.add(i as usize)).shader.as_mut_ptr(), (*in_).shader.as_ptr(), MAX_QPATH);
        (*s_worldData.shaders.add(i as usize)).contentFlags = (*in_).contentFlags;
        (*s_worldData.shaders.add(i as usize)).surfaceFlags = (*in_).surfaceFlags;

        i += 1;
        in_ = in_.add(1);
        out = out.add(1);
    }
}


/*
=================
CMod_LoadSubmodels
=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadSubmodels(data: *mut c_void, len: c_int) {
    let mut in_: *mut dmodel_t;
    let mut out: *mut cmodel_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;
    let mut indexes: *mut c_int;

    in_ = data as *mut dmodel_t;
    if len as usize % size_of::<dmodel_t>() != 0 {
        Com_Error(ERR_DROP, b"CMod_LoadSubmodels: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<dmodel_t>()) as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no models\0".as_ptr() as *const c_char);
    }

    if count > MAX_SUBMODELS {
        Com_Error(ERR_DROP,
            b"MAX_SUBMODELS (%d) exceeded by %d\0".as_ptr() as *const c_char,
            MAX_SUBMODELS, count - MAX_SUBMODELS);
    }

    cmg.cmodels = Z_Malloc(
        (count as usize * size_of::<cmodel_t>()) as c_int, TAG_BSP, qtrue,
    ) as *mut cmodel_t;
    cmg.numSubModels = count;

    i = 0;
    while i < count {
        // out is set inside the loop body (shadows the C for-loop increment)
        out = &mut *cmg.cmodels.add(i as usize);
        in_ = (data as *mut dmodel_t).add(i as usize);

        j = 0;
        while j < 3 {
            // spread the mins / maxs by a pixel
            (*out).mins[j as usize] = (*in_).mins[j as usize] - 1.0;
            (*out).maxs[j as usize] = (*in_).maxs[j as usize] + 1.0;
            j += 1;
        }

        if i == 0 {
            i += 1;
            continue; // world model doesn't need other info
        }

        // make a "leaf" just to hold the model's brushes and surfaces
        (*out).leaf.numLeafBrushes = (*in_).numBrushes;
        indexes = Z_Malloc((*out).leaf.numLeafBrushes * 4, TAG_BSP, qfalse) as *mut c_int;
        (*out).leaf.firstLeafBrush = indexes.offset_from(cmg.leafbrushes) as c_int;
        j = 0;
        while j < (*out).leaf.numLeafBrushes {
            *indexes.add(j as usize) = (*in_).firstBrush + j;
            j += 1;
        }

        (*out).leaf.numLeafSurfaces = (*in_).numSurfaces;
        indexes = Z_Malloc((*out).leaf.numLeafSurfaces * 4, TAG_BSP, qfalse) as *mut c_int;
        (*out).leaf.firstLeafSurface = indexes.offset_from(cmg.leafsurfaces) as c_int;
        j = 0;
        while j < (*out).leaf.numLeafSurfaces {
            *indexes.add(j as usize) = (*in_).firstSurface + j;
            j += 1;
        }

        i += 1;
    }
}

/*
=================
CMod_LoadNodes

=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadNodes(data: *mut c_void, len: c_int) {
    let mut in_: *mut dnode_t;
    let mut out: *mut cNode_t;
    let mut i: c_int;
    let mut count: c_int;

    in_ = data as *mut dnode_t;
    if len as usize % size_of::<dnode_t>() != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<dnode_t>()) as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map has no nodes\0".as_ptr() as *const c_char);
    }
    cmg.nodes = Z_Malloc(
        (count as usize * size_of::<cNode_t>()) as c_int, TAG_BSP, qfalse,
    ) as *mut cNode_t;
    cmg.numNodes = count;

    out = cmg.nodes;

    i = 0;
    while i < count {
        (*out).children[0] = (*in_).children[0];
        (*out).children[1] = (*in_).children[1];
        i += 1;
        out = out.add(1);
        in_ = in_.add(1);
    }
}

/*
=================
CM_BoundBrush

=================
*/
#[no_mangle]
pub unsafe extern "C" fn CM_BoundBrush(b: *mut cbrush_t) {
    (*b).bounds[0][0] = -cmg.planes[(*(*b).sides.add(0)).planeNum.GetValue() as usize].dist;
    (*b).bounds[1][0] =  cmg.planes[(*(*b).sides.add(1)).planeNum.GetValue() as usize].dist;

    (*b).bounds[0][1] = -cmg.planes[(*(*b).sides.add(2)).planeNum.GetValue() as usize].dist;
    (*b).bounds[1][1] =  cmg.planes[(*(*b).sides.add(3)).planeNum.GetValue() as usize].dist;

    (*b).bounds[0][2] = -cmg.planes[(*(*b).sides.add(4)).planeNum.GetValue() as usize].dist;
    (*b).bounds[1][2] =  cmg.planes[(*(*b).sides.add(5)).planeNum.GetValue() as usize].dist;
}


/*
=================
CMod_LoadBrushes

=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadBrushes(data: *mut c_void, len: c_int) {
    let mut in_: *mut dbrush_t;
    let mut out: *mut cbrush_t;
    let mut i: c_int;
    let mut count: c_int;

    in_ = data as *mut dbrush_t;
    if len as usize % size_of::<dbrush_t>() != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<dbrush_t>()) as c_int;

    cmg.brushes = Z_Malloc(
        ((BOX_BRUSHES + count) as usize * size_of::<cbrush_t>()) as c_int, TAG_BSP, qfalse,
    ) as *mut cbrush_t;
    cmg.numBrushes = count;

    out = cmg.brushes;

    i = 0;
    while i < count {
        (*out).sides = cmg.brushsides.add((*in_).firstSide as usize);
        (*out).numsides = (*in_).numSides;

        (*out).shaderNum = (*in_).shaderNum;
        if (*out).shaderNum < 0 || (*out).shaderNum >= cmg.numShaders {
            Com_Error(ERR_DROP,
                b"CMod_LoadBrushes: bad shaderNum: %i\0".as_ptr() as *const c_char,
                (*out).shaderNum);
        }
        (*out).contents = (*cmg.shaders.add((*out).shaderNum as usize)).contentFlags;
        //TEMP HACK: for water that cuts vis but is not solid!!!
        if (*cmg.shaders.add((*out).shaderNum as usize)).surfaceFlags & SURF_SLICK != 0 {
            (*out).contents &= !CONTENTS_SOLID;
        }

        CM_OrOfAllContentsFlagsInMap |= (*out).contents;

        CM_BoundBrush(out);

        i += 1;
        out = out.add(1);
        in_ = in_.add(1);
    }
}

/*
=================
CMod_LoadLeafs
=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadLeafs(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut out: *mut cLeaf_t;
    let mut in_: *mut dleaf_t;
    let mut count: c_int;

    in_ = data as *mut dleaf_t;
    if len as usize % size_of::<dleaf_t>() != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<dleaf_t>()) as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no leafs\0".as_ptr() as *const c_char);
    }

    cmg.leafs = Z_Malloc(
        ((BOX_LEAFS + count) as usize * size_of::<cLeaf_t>()) as c_int, TAG_BSP, qfalse,
    ) as *mut cLeaf_t;
    cmg.numLeafs = count;
    out = cmg.leafs;

    i = 0;
    while i < count {
        (*out).cluster             = (*in_).cluster;
        (*out).area                = (*in_).area;
        (*out).firstLeafBrush      = (*in_).firstLeafBrush;
        (*out).numLeafBrushes      = (*in_).numLeafBrushes;
        (*out).firstLeafSurface    = (*in_).firstLeafSurface;
        (*out).numLeafSurfaces     = (*in_).numLeafSurfaces;

        if (*out).cluster >= cmg.numClusters {
            cmg.numClusters = (*out).cluster + 1;
        }
        if (*out).area >= cmg.numAreas {
            cmg.numAreas = (*out).area + 1;
        }

        i += 1;
        in_ = in_.add(1);
        out = out.add(1);
    }

    cmg.areas = Z_Malloc(
        (cmg.numAreas as usize * size_of::<cArea_t>()) as c_int, TAG_BSP, qtrue,
    ) as *mut cArea_t;

    // extern qboolean vidRestartReloadMap;  -- comes via glob import
    if vidRestartReloadMap == qfalse {
        cmg.areaPortals = Z_Malloc(
            (cmg.numAreas * cmg.numAreas) as usize as c_int * size_of::<c_int>() as c_int,
            TAG_BSP, qtrue,
        ) as *mut c_int;
    }
}

/*
=================
CMod_LoadPlanes
=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadPlanes(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut out: *mut cplane_t;
    let mut in_: *mut dplane_t;
    let mut count: c_int;
    let mut bits: c_int;

    in_ = data as *mut dplane_t;
    if len as usize % size_of::<dplane_t>() != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<dplane_t>()) as c_int;

    if count < 1 {
        Com_Error(ERR_DROP, b"Map with no planes\0".as_ptr() as *const c_char);
    }
    cmg.planes = Z_Malloc(
        ((BOX_PLANES + count) as usize * size_of::<cplane_t>()) as c_int, TAG_BSP, qfalse,
    ) as *mut cplane_t;
    cmg.numPlanes = count;

    out = cmg.planes;

    i = 0;
    while i < count {
        bits = 0;
        j = 0;
        while j < 3 {
            (*out).normal[j as usize] = (*in_).normal[j as usize];
            if (*out).normal[j as usize] < 0.0 {
                bits |= 1 << j;
            }
            j += 1;
        }

        (*out).dist     = (*in_).dist;
        (*out).type_    = PlaneTypeForNormal((*out).normal.as_ptr());
        (*out).signbits = bits;

        i += 1;
        in_ = in_.add(1);
        out = out.add(1);
    }
}

/*
=================
CMod_LoadLeafBrushes
=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadLeafBrushes(data: *mut c_void, len: c_int) {
    let mut out: *mut c_int;
    let mut in_: *mut c_int;
    let mut count: c_int;

    in_ = data as *mut c_int;
    if len as usize % size_of::<c_int>() != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<c_int>()) as c_int;

    cmg.leafbrushes = Z_Malloc(
        ((BOX_BRUSHES + count) as usize * size_of::<c_int>()) as c_int, TAG_BSP, qfalse,
    ) as *mut c_int;
    cmg.numLeafBrushes = count;

    out = cmg.leafbrushes;

    memcpy(out as *mut c_void, in_ as *const c_void, len as usize);
}

/*
=================
CMod_LoadBrushSides
=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadBrushSides(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut out: *mut cbrushside_t;
    let mut in_: *mut dbrushside_t;
    let mut count: c_int;

    in_ = data as *mut dbrushside_t;
    if len as usize % size_of::<dbrushside_t>() != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }
    count = (len as usize / size_of::<dbrushside_t>()) as c_int;

    cmg.brushsides = Z_Malloc(
        ((BOX_SIDES + count) as usize * size_of::<cbrushside_t>()) as c_int, TAG_BSP, qfalse,
    ) as *mut cbrushside_t;
    cmg.numBrushSides = count;

    out = cmg.brushsides;

    i = 0;
    while i < count {
        (*out).planeNum = (*in_).planeNum;
        // porting note: planeNum on cbrushside_t is a C++ wrapper type; the assertion
        // checks that the assigned value round-trips through GetValue().
        debug_assert_eq!((*in_).planeNum, (*out).planeNum.GetValue());

        (*out).shaderNum = (*in_).shaderNum;
        if (*out).shaderNum < 0 || (*out).shaderNum >= cmg.numShaders {
            Com_Error(ERR_DROP,
                b"CMod_LoadBrushSides: bad shaderNum: %i\0".as_ptr() as *const c_char,
                (*out).shaderNum);
        }

        i += 1;
        in_ = in_.add(1);
        out = out.add(1);
    }
}


/*
=================
CMod_LoadEntityString
=================
*/
#[no_mangle]
pub unsafe extern "C" fn CMod_LoadEntityString(data: *mut c_void, len: c_int) {
    cmg.entityString = Z_Malloc(len, TAG_BSP, qfalse) as *mut c_char;
    cmg.numEntityChars = len;
    memcpy(cmg.entityString as *mut c_void, data, len as usize);
}

/*
=================
CMod_LoadVisibility
=================
*/
const VIS_HEADER: usize = 8;

#[no_mangle]
pub unsafe extern "C" fn CMod_LoadVisibility(data: *mut c_void, len: c_int) {
    let buf: *mut c_char;

    if len == 0 {
        cmg.visibility = core::ptr::null_mut();
        return;
    }
    buf = data as *mut c_char;

    visData.SetAllocator(SparcAllocator, SparcDeallocator);

    cmg.vised = qtrue;
    cmg.numClusters  = *(buf as *const c_int).add(0);
    cmg.clusterBytes = *(buf as *const c_int).add(1);
    visData.Load(buf.add(VIS_HEADER), len - VIS_HEADER as c_int);
    cmg.visibility = core::ptr::addr_of_mut!(visData);
    RE_SetWorldVisData(core::ptr::addr_of_mut!(visData) as *mut c_void);
}

//==================================================================


/*
=================
CMod_LoadPatches
=================
*/
const MAX_PATCH_VERTS: c_int = 1024;

#[no_mangle]
pub unsafe extern "C" fn CMod_LoadPatches(
    verts: *mut c_void, vertlen: c_int,
    surfaces: *mut c_void, surfacelen: c_int,
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
    let mut points: [vec3_t; MAX_PATCH_VERTS as usize] = core::mem::zeroed();
    let mut width: c_int;
    let mut height: c_int;
    let mut shaderNum: c_int;

    count = (surfacelen as usize / size_of::<dpatch_t>()) as c_int;

    cmg.numSurfaces = numsurfs;
    cmg.surfaces = Z_Malloc(
        (cmg.numSurfaces as usize * size_of::<*mut cPatch_t>()) as c_int, TAG_BSP, qtrue,
    ) as *mut *mut cPatch_t;

    dv = verts as *mut mapVert_t;
    if vertlen as usize % size_of::<mapVert_t>() != 0 {
        Com_Error(ERR_DROP, b"MOD_LoadBmodel: funny lump size\0".as_ptr() as *const c_char);
    }

    let mut patchScratch: *mut u8 = Z_Malloc(
        (size_of::<cPatch_t>() * count as usize) as c_int, TAG_BSP, qtrue,
    ) as *mut u8;

    // extern void CM_GridAlloc();  -- comes via cm_patch_h glob
    // extern void CM_PatchCollideFromGridTempAlloc();
    // extern void CM_PreparePatchCollide(int num);
    // extern void CM_TempPatchPlanesAlloc();
    CM_GridAlloc();
    CM_PatchCollideFromGridTempAlloc();
    CM_PreparePatchCollide(count);
    CM_TempPatchPlanesAlloc();

    let facetbuf: *mut facetLoad_t = Z_Malloc(
        (MAX_PATCH_PLANES as usize * size_of::<facetLoad_t>()) as c_int,
        TAG_TEMP_WORKSPACE, qfalse,
    ) as *mut facetLoad_t;

    let gridbuf: *mut c_int = Z_Malloc(
        (CM_MAX_GRID_SIZE * CM_MAX_GRID_SIZE * 2 * size_of::<c_int>() as c_int) as c_int,
        TAG_TEMP_WORKSPACE, qfalse,
    ) as *mut c_int;

    i = 0;
    while i < count {
        in_ = (surfaces as *mut dpatch_t).add(i as usize);

        *cmg.surfaces.add((*in_).code as usize) = patchScratch as *mut cPatch_t;
        patch = patchScratch as *mut cPatch_t;
        patchScratch = patchScratch.add(size_of::<cPatch_t>());

        // load the full drawverts onto the stack
        width  = (*in_).patchWidth;
        height = (*in_).patchHeight;
        c = width * height;
        if c > MAX_PATCH_VERTS {
            Com_Error(ERR_DROP, b"ParseMesh: MAX_PATCH_VERTS\0".as_ptr() as *const c_char);
        }

        dv_p = dv.add(((*in_).verts >> 12) as usize);
        j = 0;
        while j < c {
            points[j as usize][0] = (*dv_p).xyz[0];
            points[j as usize][1] = (*dv_p).xyz[1];
            points[j as usize][2] = (*dv_p).xyz[2];
            j += 1;
            dv_p = dv_p.add(1);
        }

        shaderNum = (*in_).shaderNum;
        (*patch).contents = (*cmg.shaders.add(shaderNum as usize)).contentFlags;
        CM_OrOfAllContentsFlagsInMap |= (*patch).contents;

        (*patch).surfaceFlags = (*cmg.shaders.add(shaderNum as usize)).surfaceFlags;

        // create the internal facet structure
        (*patch).pc = CM_GeneratePatchCollide(width, height, points.as_ptr(), facetbuf, gridbuf);

        i += 1;
    }

    // extern void CM_GridDealloc();
    // extern void CM_PatchCollideFromGridTempDealloc();
    // extern void CM_TempPatchPlanesDealloc();
    CM_PatchCollideFromGridTempDealloc();
    CM_GridDealloc();
    CM_TempPatchPlanesDealloc();

    Z_Free(gridbuf as *mut c_void);
    Z_Free(facetbuf as *mut c_void);
}

//==================================================================

#[cfg(feature = "bspc")]
/*
==================
CM_FreeMap

Free any loaded map and all submodels
==================
*/
#[no_mangle]
pub unsafe extern "C" fn CM_FreeMap() {
    memset(core::ptr::addr_of_mut!(cmg) as *mut c_void, 0, size_of::<clipMap_t>());
    Hunk_ClearHigh();
    CM_ClearLevelPatches();
}

/*
==================
CM_LoadMap

Loads in the map and all submodels
==================
*/
pub static mut gpvCachedMapDiskImage: *mut c_void = core::ptr::null_mut();
pub static mut gsCachedMapDiskImage: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
pub static mut gbUsingCachedMapDataRightNow: qboolean = qfalse; // if true, signifies that you can't delete this at the moment!! (used during z_malloc()-fail recovery attempt)

// called in response to a "devmapbsp blah" or "devmapall blah" command, do NOT use inside CM_Load unless you pass in qtrue
//
// new bool return used to see if anything was freed, used during z_malloc failure re-try
//
#[no_mangle]
pub unsafe extern "C" fn CM_DeleteCachedMap(bGuaranteedOkToDelete: qboolean) -> qboolean {
    let mut bActuallyFreedSomething: qboolean = qfalse;

    if bGuaranteedOkToDelete != qfalse || gbUsingCachedMapDataRightNow == qfalse {
        // dump cached disk image...
        //
        if !gpvCachedMapDiskImage.is_null() {
            Z_Free(gpvCachedMapDiskImage);
                   gpvCachedMapDiskImage = core::ptr::null_mut();

            bActuallyFreedSomething = qtrue;
        }
        gsCachedMapDiskImage[0] = b'\0' as c_char;

        // force map loader to ignore cached internal BSP structures for next level CM_LoadMap() call...
        //
        cmg.name[0] = b'\0' as c_char;
    }

    bActuallyFreedSomething
}

#[no_mangle]
pub unsafe extern "C" fn CM_Free() {
    CM_ClearLevelPatches();
    visData.Release();
    Z_TagFree(TAG_BSP);
}

// R_LoadSurfaces, R_LoadPatches, R_LoadTriSurfs, R_LoadFaces, R_LoadFlares,
// R_LoadShaders, R_LoadLightmaps, fileBase -- come via tr_local_h glob import.
// vidRestartReloadMap -- comes via cm_local_h / tr_local_h glob import.

unsafe fn CM_LoadMap_Actual(name: *const c_char, clientload: qboolean, checksum: *mut c_int) {
    let mut buf:      *const c_int = core::ptr::null();
    let mut surfBuf:  *const c_int = core::ptr::null();
    static mut last_checksum: c_uint = 0;
    let mut lmName:    [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut stripName: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut outputLump: Lump = core::mem::zeroed();

    if name.is_null() || *name == 0 {
        Com_Error(ERR_DROP, b"CM_LoadMap: NULL name\0".as_ptr() as *const c_char);
    }

    #[cfg(not(feature = "bspc"))]
    {
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
    }
    Com_DPrintf(b"CM_LoadMap( %s, %i )\n\0".as_ptr() as *const c_char, name, clientload);

    if strcmp(cmg.name.as_ptr(), name) == 0 && clientload != qfalse {
        *checksum = last_checksum as c_int;
        return;
    }

    // free old stuff
    // extern qboolean vidRestartReloadMap;  -- comes via glob import
    let ap: *mut c_int;
    if vidRestartReloadMap != qfalse {
        ap = cmg.areaPortals;
    } else {
        ap = core::ptr::null_mut();
    }
    memset(core::ptr::addr_of_mut!(cmg) as *mut c_void, 0, size_of::<clipMap_t>());
    if vidRestartReloadMap != qfalse {
        cmg.areaPortals = ap;
    }

    if *name == 0 {
        cmg.numLeafs    = 1;
        cmg.numClusters = 1;
        cmg.numAreas    = 1;
        cmg.cmodels = Z_Malloc(
            size_of::<cmodel_t>() as c_int, TAG_BSP, qtrue,
        ) as *mut cmodel_t;
        *checksum = 0;
        return;
    }

    last_checksum = crc32(0, name as *const Bytef, strlen(name));
    COM_StripExtension(name, stripName.as_mut_ptr());

    // load into heap
    outputLump.load(stripName.as_ptr(), b"shaders\0".as_ptr() as *const c_char);
    CMod_LoadShaders(outputLump.data, outputLump.len);
    R_LoadShaders();

    strcpy(lmName.as_mut_ptr(), name);
    outputLump.load(stripName.as_ptr(), b"lightmaps\0".as_ptr() as *const c_char);
    R_LoadLightmaps(outputLump.data, outputLump.len, lmName.as_ptr());

    {
        fileBase = core::ptr::null_mut();
        outputLump.clear();

        let mut misc: Lump = core::mem::zeroed();
        misc.load(stripName.as_ptr(), b"misc\0".as_ptr() as *const c_char);

        let num_surfs: c_int = *(misc.data as *const c_int);
        misc.clear();

        R_LoadSurfaces(num_surfs);

        let mut verts: Lump = core::mem::zeroed();
        verts.load(stripName.as_ptr(), b"verts\0".as_ptr() as *const c_char);

        let mut patches: Lump = core::mem::zeroed();
        patches.load(stripName.as_ptr(), b"patches\0".as_ptr() as *const c_char);

        CMod_LoadPatches(verts.data, verts.len,
            patches.data, patches.len,
            num_surfs);
        R_LoadPatches(verts.data, verts.len,
            patches.data, patches.len);

        patches.clear();

        let mut indexes: Lump = core::mem::zeroed();
        indexes.load(stripName.as_ptr(), b"indexes\0".as_ptr() as *const c_char);

        let mut trisurfs: Lump = core::mem::zeroed();
        trisurfs.load(stripName.as_ptr(), b"trisurfs\0".as_ptr() as *const c_char);

        R_LoadTriSurfs(indexes.data, indexes.len,
            verts.data, verts.len,
            trisurfs.data, trisurfs.len);

        trisurfs.clear();

        let mut faces: Lump = core::mem::zeroed();
        faces.load(stripName.as_ptr(), b"faces\0".as_ptr() as *const c_char);

        R_LoadFaces(indexes.data, indexes.len,
            verts.data, verts.len,
            faces.data, faces.len);

        let mut flares: Lump = core::mem::zeroed();
        flares.load(stripName.as_ptr(), b"flares\0".as_ptr() as *const c_char);

        R_LoadFlares(flares.data, flares.len);
    }

    outputLump.load(stripName.as_ptr(), b"leafs\0".as_ptr() as *const c_char);
    CMod_LoadLeafs(outputLump.data, outputLump.len);

    outputLump.load(stripName.as_ptr(), b"leafbrushes\0".as_ptr() as *const c_char);
    CMod_LoadLeafBrushes(outputLump.data, outputLump.len);

    cmg.leafsurfaces = core::ptr::null_mut();
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

    TotalSubModels += cmg.numSubModels;

    CM_InitBoxHull();

    *checksum = last_checksum as c_int;

    // do this whether or not the map was cached from last load...
    //
    CM_FloodAreaConnections();

    // allow this to be cached if it is loaded by the server
    if clientload == qfalse {
        Q_strncpyz(
            cmg.name.as_mut_ptr(),
            name,
            size_of::<[c_char; MAX_QPATH as usize]>() as c_int,
        );
    }
    CM_CleanLeafCache();
}

// need a wrapper function around this because of multiple returns, need to ensure bool is correct...
//
#[no_mangle]
pub unsafe extern "C" fn CM_LoadMap(name: *const c_char, clientload: qboolean, checksum: *mut c_int) {
    CM_LoadMap_Actual(name, clientload, checksum);
}

#[no_mangle]
pub unsafe extern "C" fn CM_SameMap(server: *mut c_char) -> qboolean {
    if cmg.name[0] == 0 || server.is_null() || *server == 0 {
        return qfalse;
    }

    if Q_stricmp(cmg.name.as_ptr(), va(b"maps/%s.bsp\0".as_ptr() as *const c_char, server)) != 0 {
        return qfalse;
    }

    qtrue
}

#[cfg(not(feature = "xbox"))]
#[no_mangle]
pub unsafe extern "C" fn CM_HasTerrain() -> qboolean {
    if !cmg.landScape.is_null() {
        return qtrue;
    }
    qfalse
}

/*
==================
CM_ClearMap
==================
*/
#[no_mangle]
pub unsafe extern "C" fn CM_ClearMap() {
    let mut i: c_int;

    CM_OrOfAllContentsFlagsInMap = CONTENTS_BODY;

    #[cfg(not(feature = "bspc"))]
    {
        // CM_ShutdownShaderProperties();
        // MAT_Shutdown();
    }

    #[cfg(not(feature = "xbox"))]
    {
        if !TheRandomMissionManager.is_null() {
            // delete TheRandomMissionManager;
            // porting note: C++ delete on opaque C++ object; trust engine teardown for memory.
            TheRandomMissionManager = core::ptr::null_mut() as *mut _;
        }

        if !cmg.landScape.is_null() {
            // delete cmg.landScape;
            cmg.landScape = core::ptr::null_mut() as *mut _;
        }
    }

    memset(core::ptr::addr_of_mut!(cmg) as *mut c_void, 0, size_of::<clipMap_t>());
    CM_ClearLevelPatches();

    i = 0;
    while i < NumSubBSP {
        memset(
            core::ptr::addr_of_mut!(SubBSP[i as usize]) as *mut c_void,
            0,
            size_of::<clipMap_t>(),
        );
        i += 1;
    }
    NumSubBSP     = 0;
    TotalSubModels = 0;
}

#[no_mangle]
pub unsafe extern "C" fn CM_TotalMapContents() -> c_int {
    CM_OrOfAllContentsFlagsInMap
}

/*
==================
CM_ClipHandleToModel
==================
*/
#[no_mangle]
pub unsafe extern "C" fn CM_ClipHandleToModel(
    handle: clipHandle_t,
    clipMap: *mut *mut clipMap_t,
) -> *mut cmodel_t {
    let mut i: c_int;
    let mut count: c_int;

    if handle < 0 {
        Com_Error(ERR_DROP,
            b"CM_ClipHandleToModel: bad handle %i\0".as_ptr() as *const c_char,
            handle);
    }
    if handle < cmg.numSubModels {
        if !clipMap.is_null() {
            *clipMap = core::ptr::addr_of_mut!(cmg);
        }
        return &mut *cmg.cmodels.add(handle as usize);
    }
    if handle == BOX_MODEL_HANDLE {
        if !clipMap.is_null() {
            *clipMap = core::ptr::addr_of_mut!(cmg);
        }
        return core::ptr::addr_of_mut!(box_model);
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
        Com_Error(ERR_DROP,
            b"CM_ClipHandleToModel: bad handle %i < %i < %i\0".as_ptr() as *const c_char,
            cmg.numSubModels, handle, MAX_SUBMODELS);
    }
    Com_Error(ERR_DROP,
        b"CM_ClipHandleToModel: bad handle %i\0".as_ptr() as *const c_char,
        handle + MAX_SUBMODELS);

    core::ptr::null_mut()
}

/*
==================
CM_InlineModel
==================
*/
#[no_mangle]
pub unsafe extern "C" fn CM_InlineModel(index: c_int) -> clipHandle_t {
    if index < 0 || index >= TotalSubModels {
        Com_Error(ERR_DROP,
            b"CM_InlineModel: bad number (may need to re-BSP map?)\0".as_ptr() as *const c_char);
    }
    index
}

#[no_mangle]
pub unsafe extern "C" fn CM_NumClusters() -> c_int {
    cmg.numClusters
}

#[no_mangle]
pub unsafe extern "C" fn CM_NumInlineModels() -> c_int {
    cmg.numSubModels
}

#[no_mangle]
pub unsafe extern "C" fn CM_EntityString() -> *mut c_char {
    cmg.entityString
}

#[no_mangle]
pub unsafe extern "C" fn CM_SubBSPEntityString(index: c_int) -> *mut c_char {
    SubBSP[index as usize].entityString
}

#[no_mangle]
pub unsafe extern "C" fn CM_LeafCluster(leafnum: c_int) -> c_int {
    if leafnum < 0 || leafnum >= cmg.numLeafs {
        Com_Error(ERR_DROP, b"CM_LeafCluster: bad number\0".as_ptr() as *const c_char);
    }
    (*cmg.leafs.add(leafnum as usize)).cluster
}

#[no_mangle]
pub unsafe extern "C" fn CM_LeafArea(leafnum: c_int) -> c_int {
    if leafnum < 0 || leafnum >= cmg.numLeafs {
        Com_Error(ERR_DROP, b"CM_LeafArea: bad number\0".as_ptr() as *const c_char);
    }
    (*cmg.leafs.add(leafnum as usize)).area
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
pub unsafe extern "C" fn CM_InitBoxHull() {
    let mut i: c_int;
    let mut side: c_int;
    let mut p: *mut cplane_t;
    let mut s: *mut cbrushside_t;

    box_planes = cmg.planes.add(cmg.numPlanes as usize);

    box_brush = cmg.brushes.add(cmg.numBrushes as usize);
    (*box_brush).numsides = 6;
    (*box_brush).sides = cmg.brushsides.add(cmg.numBrushSides as usize);
    (*box_brush).contents = CONTENTS_BODY;

    (*core::ptr::addr_of_mut!(box_model)).leaf.numLeafBrushes = 1;
    // box_model.leaf.firstLeafBrush = cmg.numBrushes;
    (*core::ptr::addr_of_mut!(box_model)).leaf.firstLeafBrush = cmg.numLeafBrushes;
    *cmg.leafbrushes.add(cmg.numLeafBrushes as usize) = cmg.numBrushes;

    i = 0;
    while i < 6 {
        side = i & 1;

        // brush sides
        s = cmg.brushsides.add((cmg.numBrushSides + i) as usize);
        (*s).planeNum = cmg.numPlanes + i * 2 + side;
        (*s).shaderNum = cmg.numShaders;

        // planes
        p = box_planes.add((i * 2) as usize);
        (*p).type_    = i >> 1;
        (*p).signbits = 0;
        VectorClear((*p).normal.as_mut_ptr());
        (*p).normal[(i >> 1) as usize] = 1.0;

        p = box_planes.add((i * 2 + 1) as usize);
        (*p).type_    = 3 + (i >> 1);
        (*p).signbits = 0;
        VectorClear((*p).normal.as_mut_ptr());
        (*p).normal[(i >> 1) as usize] = -1.0;

        SetPlaneSignbits(p);

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
#[no_mangle]
pub unsafe extern "C" fn CM_TempBoxModel(mins: *const vec3_t, maxs: *const vec3_t) -> clipHandle_t {
    //, const int contents ) {
    (*box_planes.add(0)).dist  =  (*maxs)[0];
    (*box_planes.add(1)).dist  = -(*maxs)[0];
    (*box_planes.add(2)).dist  =  (*mins)[0];
    (*box_planes.add(3)).dist  = -(*mins)[0];
    (*box_planes.add(4)).dist  =  (*maxs)[1];
    (*box_planes.add(5)).dist  = -(*maxs)[1];
    (*box_planes.add(6)).dist  =  (*mins)[1];
    (*box_planes.add(7)).dist  = -(*mins)[1];
    (*box_planes.add(8)).dist  =  (*maxs)[2];
    (*box_planes.add(9)).dist  = -(*maxs)[2];
    (*box_planes.add(10)).dist =  (*mins)[2];
    (*box_planes.add(11)).dist = -(*mins)[2];

    VectorCopy((*mins).as_ptr(), (*box_brush).bounds[0].as_mut_ptr());
    VectorCopy((*maxs).as_ptr(), (*box_brush).bounds[1].as_mut_ptr());

    //FIXME: this is the "correct" way, but not the way JK2 was designed around... fix for further projects
    // box_brush->contents = contents;

    BOX_MODEL_HANDLE
}


/*
===================
CM_ModelBounds
===================
*/
#[no_mangle]
pub unsafe extern "C" fn CM_ModelBounds(
    cmg: *mut clipMap_t,
    model: clipHandle_t,
    mins: *mut vec3_t,
    maxs: *mut vec3_t,
) {
    let cmod: *mut cmodel_t;

    cmod = CM_ClipHandleToModel(model, core::ptr::null_mut());
    VectorCopy((*cmod).mins.as_ptr(), (*mins).as_mut_ptr());
    VectorCopy((*cmod).maxs.as_ptr(), (*maxs).as_mut_ptr());
}

/*
===================
CM_RegisterTerrain

Allows physics to examine the terrain data.
===================
*/
#[cfg(not(feature = "bspc"))]
/* #if 0	// Removing terrain on Xbox
CCMLandScape *CM_RegisterTerrain(const char *config, bool server)
{
	thandle_t		terrainId;
	CCMLandScape	*ls;

	terrainId = atol(Info_ValueForKey(config, "terrainId"));
	if(terrainId && cmg.landScape)
	{
		// Already spawned so just return
		ls = cmg.landScape;
		ls->IncreaseRefCount();
		return(ls);
	}
	// Doesn't exist so create and link in
	//cmg.numTerrains++;
	ls = CM_InitTerrain(config, 1, server);

	// Increment for the next instance
	if (cmg.landScape)
	{
		Com_Error(ERR_DROP, "You can't have more than one terrain brush.");
	}
	cmg.landScape = ls;
	return(ls);
}

/*
===================
CM_ShutdownTerrain
===================
*/

void CM_ShutdownTerrain( thandle_t terrainId)
{
	CCMLandScape	*landscape;

	landscape = cmg.landScape;
	if (landscape)
	{
		landscape->DecreaseRefCount();
		if(landscape->GetRefCount() <= 0)
		{
			delete landscape;
			cmg.landScape = NULL;
		}
	}
}
#endif	// No terrain on Xbox */
const _CM_TERRAIN_REMOVED_ON_XBOX: () = ();

#[no_mangle]
pub unsafe extern "C" fn CM_LoadSubBSP(name: *const c_char, clientload: qboolean) -> c_int {
    let mut i: c_int;
    let mut checksum: c_int = 0;
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
        Com_Error(ERR_DROP,
            b"CM_LoadSubBSP: too many unique sub BSPs\0".as_ptr() as *const c_char);
    }

    #[cfg(feature = "xbox")]
    {
        debug_assert!(false, "MATT! - testing now - fix this later!");
        // assert(0); // MATT! - testing now - fix this later!
    }
    #[cfg(not(feature = "xbox"))]
    {
        // porting note: non-Xbox path calls a 4-arg CM_LoadMap_Actual (taking SubBSP ref)
        // defined in the non-Xbox TU; trusting linker resolves it.
        CM_LoadMap_Actual(name, clientload, &mut checksum, &mut SubBSP[NumSubBSP as usize]);
    }
    NumSubBSP += 1;

    count
}

#[no_mangle]
pub unsafe extern "C" fn CM_FindSubBSP(modelIndex: c_int) -> c_int {
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

#[no_mangle]
pub unsafe extern "C" fn CM_GetWorldBounds(mins: *mut vec3_t, maxs: *mut vec3_t) {
    VectorCopy((*cmg.cmodels.add(0)).mins.as_ptr(), (*mins).as_mut_ptr());
    VectorCopy((*cmg.cmodels.add(0)).maxs.as_ptr(), (*maxs).as_mut_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn CM_ModelContents_Actual(
    model: clipHandle_t,
    cm: *mut clipMap_t,
) -> c_int {
    let cmod: *mut cmodel_t;
    let mut contents: c_int = 0;
    let mut i: c_int;

    let mut cm_ptr: *mut clipMap_t = if cm.is_null() {
        core::ptr::addr_of_mut!(cmg)
    } else {
        cm
    };

    cmod = CM_ClipHandleToModel(model, &mut cm_ptr);

    //MCG ADDED - return the contents, too
    if (*cmod).leaf.numLeafBrushes != 0 {
        // check for brush
        let mut brushNum: c_int;
        i = (*cmod).leaf.firstLeafBrush;
        while i < (*cmod).leaf.firstLeafBrush + (*cmod).leaf.numLeafBrushes {
            brushNum = *(*cm_ptr).leafbrushes.add(i as usize);
            contents |= (*(*cm_ptr).brushes.add(brushNum as usize)).contents;
            i += 1;
        }
    }
    if (*cmod).leaf.numLeafSurfaces != 0 {
        // if not brush, check for patch
        let mut surfaceNum: c_int;
        i = (*cmod).leaf.firstLeafSurface;
        while i < (*cmod).leaf.firstLeafSurface + (*cmod).leaf.numLeafSurfaces {
            surfaceNum = *(*cm_ptr).leafsurfaces.add(i as usize);
            if !(*(*cm_ptr).surfaces.add(surfaceNum as usize)).is_null() {
                //HERNH?  How could we have a null surf within our cmod->leaf.numLeafSurfaces?
                contents |= (**(*cm_ptr).surfaces.add(surfaceNum as usize)).contents;
            }
            i += 1;
        }
    }
    contents
}

#[no_mangle]
pub unsafe extern "C" fn CM_ModelContents(model: clipHandle_t, subBSPIndex: c_int) -> c_int {
    if subBSPIndex < 0 {
        return CM_ModelContents_Actual(model, core::ptr::null_mut());
    }

    CM_ModelContents_Actual(model, &mut SubBSP[subBSPIndex as usize])
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
extern "C" {
    fn SG_Append(chid: c_ulong, data: *const c_void, length: c_int) -> qboolean;
    fn SG_Read(
        chid: c_ulong,
        pvAddress: *mut c_void,
        iLength: c_int,
        ppvAddressPtr: *mut *mut c_void,
    ) -> c_int;
}

#[no_mangle]
pub unsafe extern "C" fn CM_WritePortalState() {
    SG_Append(
        0x50525453_u32 as c_ulong, // 'PRTS'
        cmg.areaPortals as *const c_void,
        cmg.numAreas * cmg.numAreas * size_of::<c_int>() as c_int,
    );
}

/*
===================
CM_ReadPortalState

Reads the portal state from a savegame file
and recalculates the area connections
===================
*/
#[no_mangle]
pub unsafe extern "C" fn CM_ReadPortalState() {
    SG_Read(
        0x50525453_u32 as c_ulong, // 'PRTS'
        cmg.areaPortals as *mut c_void,
        cmg.numAreas * cmg.numAreas * size_of::<c_int>() as c_int,
        core::ptr::null_mut(),
    );
    CM_FloodAreaConnections();
}
