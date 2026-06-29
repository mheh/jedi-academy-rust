// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_void};

// ===== Extern type stubs (used for blind port) =====
// These types are declared as opaque stubs since they're defined in other modules
#[repr(C)]
pub struct vec3_t([f32; 3]);

#[repr(C)]
pub struct cplane_t {
    normal: vec3_t,
    dist: f32,
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    data: *mut c_void,  // First field is surfaceType_t (typically an enum/int)
    plane: cplane_t,
    // Additional fields omitted for blind port stub
}

#[repr(C)]
pub struct srfGridMesh_t {
    data: *mut c_void,
    localOrigin: vec3_t,
    meshRadius: f32,
    meshBounds: [[f32; 3]; 2],
    dlightBits: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct srfTriangles_t {
    data: *mut c_void,
    bounds: [[f32; 3]; 2],
    dlightBits: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct msurface_t {
    data: *mut c_void,
    shader: *mut c_void,
    fogIndex: c_int,
    viewCount: c_int,
    dlightBits: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct shader_t {
    cullType: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct dlight_t {
    origin: vec3_t,
    radius: f32,
    // Additional fields omitted
}

#[repr(C)]
pub struct refdef_t {
    num_dlights: c_int,
    dlights: *mut dlight_t,
    areamaskModified: c_int,
    areamask: [u8; 32],
    viewaxis: [[f32; 3]; 3],
    rdflags: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct viewParms_t {
    frustum: [cplane_t; 5],
    visBounds: [[f32; 3]; 2],
    pvsOrigin: vec3_t,
    // Additional fields omitted
}

#[repr(C)]
pub struct bmodel_t {
    bounds: [[f32; 3]; 2],
    numSurfaces: c_int,
    firstSurface: *mut msurface_t,
    // Additional fields omitted
}

#[repr(C)]
pub struct model_t {
    bmodel: *mut bmodel_t,
    bspInstance: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct trRefEntity_t {
    e: EntityState_t,
    dlightBits: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct EntityState_t {
    hModel: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct mnode_t {
    contents: c_int,
    visframe: c_int,
    mins: [f32; 3],
    maxs: [f32; 3],
    plane: *mut cplane_t,
    children: [*mut mnode_t; 2],
    parent: *mut mnode_t,
    cluster: c_int,
    area: c_int,
    firstmarksurface: *mut *mut msurface_t,
    nummarksurfaces: c_int,
    planeNum: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct mleaf_s {
    contents: c_int,
    visframe: c_int,
    cluster: c_int,
    area: c_int,
    parent: *mut mnode_t,
    // Additional fields omitted
}

#[repr(C)]
pub struct world_t {
    nodes: *mut mnode_t,
    leafs: *mut mleaf_s,
    marksurfaces: *mut *mut msurface_t,
    numnodes: c_int,
    numleafs: c_int,
    numClusters: c_int,
    clusterBytes: c_int,
    novis: *mut u8,
    vis: *mut u8,
    // Additional fields omitted
}

#[repr(C)]
pub struct perfCounter_t {
    c_sphere_cull_patch_out: c_int,
    c_sphere_cull_patch_clip: c_int,
    c_sphere_cull_patch_in: c_int,
    c_box_cull_patch_out: c_int,
    c_box_cull_patch_in: c_int,
    c_box_cull_patch_clip: c_int,
    c_dlightSurfacesCulled: c_int,
    c_dlightSurfaces: c_int,
    c_leafs: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct glConfig_t {
    // Additional fields omitted
}

#[repr(C)]
pub struct backEndState_t {
    viewParms: viewParms_t,
    // Additional fields omitted
}

#[repr(C)]
pub struct trGlobals_t {
    refdef: refdef_t,
    viewParms: viewParms_t,
    world: *mut world_t,
    pc: perfCounter_t,
    currentEntityNum: c_int,
    currentEntity: *mut trRefEntity_t,
    visCount: c_int,
    viewCluster: c_int,
    shiftedEntityNum: c_int,
    // Additional fields omitted
}

#[repr(C)]
pub struct cvar_t {
    integer: c_int,
    modified: c_int,
    // Additional fields omitted
}

// ===== Global stubs =====
extern "C" {
    pub static mut tr: trGlobals_t;
    pub static mut r_nocurves: *mut cvar_t;
    pub static mut r_nocull: *mut cvar_t;
    pub static mut r_facePlaneCull: *mut cvar_t;
    pub static mut r_lockpvs: *mut cvar_t;
    pub static mut r_novis: *mut cvar_t;
    pub static mut r_drawworld: *mut cvar_t;
    pub static mut r_showcluster: *mut cvar_t;
    pub static mut VVLightMan: LightManager;
}

#[repr(C)]
pub struct LightManager {
    num_dlights: c_int,
    // Additional fields omitted
}

// ===== Extern function stubs =====
extern "C" {
    pub fn R_CullLocalBox(bounds: *const [f32; 3]) -> c_int;
    pub fn R_CullLocalPointAndRadius(origin: vec3_t, radius: f32) -> c_int;
    pub fn R_CullPointAndRadius(origin: vec3_t, radius: f32) -> c_int;
    pub fn R_AddDrawSurf(data: *mut c_void, shader: *mut c_void, fogIndex: c_int, dlightBits: c_int);
    pub fn R_GetModelByHandle(handle: c_int) -> *mut model_t;
    pub fn R_SetupEntityLighting(refdef: *mut refdef_t, ent: *mut trRefEntity_t);
    pub fn R_DlightBmodel(bmodel: *mut bmodel_t, qboolean: c_int);
    pub fn BoxOnPlaneSide(mins: *const [f32; 3], maxs: *const [f32; 3], plane: *const cplane_t) -> c_int;
    pub fn Com_Error(code: c_int, msg: *const u8, ...);
    pub fn VID_Printf(level: c_int, msg: *const u8, ...);
    pub fn CM_ClusterPVS(cluster: c_int) -> *mut u8;
    pub fn ClearBounds(mins: *mut [f32; 3], maxs: *mut [f32; 3]);
}

// ===== Constants =====
const CULL_IN: c_int = 0;
const CULL_CLIP: c_int = 1;
const CULL_OUT: c_int = 2;

const SF_FACE: c_int = 0;
const SF_GRID: c_int = 1;
const SF_TRIANGLES: c_int = 2;

const CT_TWO_SIDED: c_int = 0;
const CT_FRONT_SIDED: c_int = 1;

const TR_WORLDENT: c_int = 0;
const QSORT_ENTITYNUM_SHIFT: c_int = 24;

const CONTENTS_SOLID: c_int = 1;

const RDF_NOWORLDMODEL: c_int = 4;

const ERR_DROP: c_int = 1;

const PRINT_ALL: c_int = 0;

const MAX_DLIGHTS: c_int = 32;

// ===== Inline helper functions =====
#[inline]
unsafe fn DotProduct(a: vec3_t, b: vec3_t) -> f32 {
    a.0[0] * b.0[0] + a.0[1] * b.0[1] + a.0[2] * b.0[2]
}

#[inline]
unsafe fn VectorCompare(a: vec3_t, b: vec3_t) -> bool {
    a.0[0] == b.0[0] && a.0[1] == b.0[1] && a.0[2] == b.0[2]
}

#[inline]
unsafe fn VectorSubtract(a: vec3_t, b: vec3_t, out: &mut vec3_t) {
    out.0[0] = a.0[0] - b.0[0];
    out.0[1] = a.0[1] - b.0[1];
    out.0[2] = a.0[2] - b.0[2];
}

#[inline]
unsafe fn CrossProduct(a: vec3_t, b: vec3_t, out: &mut vec3_t) {
    out.0[0] = a.0[1] * b.0[2] - a.0[2] * b.0[1];
    out.0[1] = a.0[2] * b.0[0] - a.0[0] * b.0[2];
    out.0[2] = a.0[0] * b.0[1] - a.0[1] * b.0[0];
}

#[inline]
unsafe fn VectorScale(v: &mut vec3_t, scale: f32, out: &mut vec3_t) {
    out.0[0] = v.0[0] * scale;
    out.0[1] = v.0[1] * scale;
    out.0[2] = v.0[2] * scale;
}

#[inline]
unsafe fn VectorCopy(src: vec3_t, dst: &mut vec3_t) {
    dst.0[0] = src.0[0];
    dst.0[1] = src.0[1];
    dst.0[2] = src.0[2];
}

static mut lookingForWorstLeaf: bool = false;

#[cfg(target_os = "xbox")]
unsafe fn GetCoordsForLeaf(leafNum: c_int, coords: &mut vec3_t) -> bool {
    let mut face: *mut srfSurfaceFace_t;
    let mut surf: *mut msurface_t;
    let mut i: c_int;

    for i in 0..(*(*tr.world).leafs.add(leafNum as usize)).nummarksurfaces {
        surf = *(*tr.world).marksurfaces.add(
            ((*(*tr.world).leafs.add(leafNum as usize)).firstMarkSurfNum + i) as usize,
        );

        if surf.is_null() || (*surf).data.is_null() || *((*surf).data as *mut c_int) != SF_FACE {
            continue;
        }

        face = (*surf).data as *mut srfSurfaceFace_t;
        // Q_CastShort2Float(&coords[0], (short*)(face->srfPoints + 0));
        // Q_CastShort2Float(&coords[1], (short*)(face->srfPoints + 1));
        // Q_CastShort2Float(&coords[2], (short*)(face->srfPoints + 2));
        return true;
    }

    false
}

/*
=================
R_CullTriSurf

Returns true if the grid is completely culled away.
Also sets the clipped hint bit in tess
=================
*/
unsafe fn R_CullTriSurf(cv: *mut srfTriangles_t) -> bool {
    let boxCull: c_int;

    boxCull = R_CullLocalBox(&(*cv).bounds[0]);

    if boxCull == CULL_OUT {
        return true;
    }
    false
}

/*
=================
R_CullGrid

Returns true if the grid is completely culled away.
Also sets the clipped hint bit in tess
=================
*/
unsafe fn R_CullGrid(cv: *mut srfGridMesh_t) -> bool {
    let mut boxCull: c_int;
    let sphereCull: c_int;

    if (*r_nocurves).integer != 0 {
        return true;
    }

    if tr.currentEntityNum != TR_WORLDENT {
        sphereCull = R_CullLocalPointAndRadius((*cv).localOrigin, (*cv).meshRadius);
    } else {
        sphereCull = R_CullPointAndRadius((*cv).localOrigin, (*cv).meshRadius);
    }
    boxCull = CULL_OUT;

    // check for trivial reject
    if sphereCull == CULL_OUT {
        tr.pc.c_sphere_cull_patch_out += 1;
        return true;
    }
    // check bounding box if necessary
    else if sphereCull == CULL_CLIP {
        tr.pc.c_sphere_cull_patch_clip += 1;

        boxCull = R_CullLocalBox(&(*cv).meshBounds[0]);

        if boxCull == CULL_OUT {
            tr.pc.c_box_cull_patch_out += 1;
            return true;
        } else if boxCull == CULL_IN {
            tr.pc.c_box_cull_patch_in += 1;
        } else {
            tr.pc.c_box_cull_patch_clip += 1;
        }
    } else {
        tr.pc.c_sphere_cull_patch_in += 1;
    }

    false
}

/*
================
R_CullSurface

Tries to back face cull surfaces before they are lighted or
added to the sorting list.

This will also allow mirrors on both sides of a model without recursion.
================
*/
unsafe fn R_CullSurface(surface: *mut c_void, shader: *mut shader_t) -> bool {
    let sface: *mut srfSurfaceFace_t;
    let d: f32;

    if (*r_nocull).integer == 1 {
        return false;
    }

    if *(surface as *mut c_int) == SF_GRID {
        return R_CullGrid(surface as *mut srfGridMesh_t);
    }

    if *(surface as *mut c_int) == SF_TRIANGLES {
        return R_CullTriSurf(surface as *mut srfTriangles_t);
    }

    if *(surface as *mut c_int) != SF_FACE {
        return false;
    }

    if (*shader).cullType == CT_TWO_SIDED {
        return false;
    }

    // face culling
    if (*r_facePlaneCull).integer == 0 {
        return false;
    }

    sface = surface as *mut srfSurfaceFace_t;
    d = DotProduct(tr.refdef.viewaxis[0], (*sface).plane.normal);

    // don't cull exactly on the plane, because there are levels of rounding
    // through the BSP, ICD, and hardware that may cause pixel gaps if an
    // epsilon isn't allowed here
    if (*shader).cullType == CT_FRONT_SIDED {
        if d < (*sface).plane.dist - 8.0 {
            return true;
        }
    } else {
        if d > (*sface).plane.dist + 8.0 {
            return true;
        }
    }

    false
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightFace(face: *mut srfSurfaceFace_t, mut dlightBits: c_int) -> c_int {
    let d: f32;
    let i: c_int;
    let dl: *mut dlight_t;

    for i in 0..tr.refdef.num_dlights {
        if (dlightBits & (1 << i)) == 0 {
            continue;
        }
        dl = (*tr.refdef.dlights).add(i as usize);
        d = DotProduct((*dl).origin, (*face).plane.normal) - (*face).plane.dist;
        if !VectorCompare((*face).plane.normal, vec3_t([0.0, 0.0, 0.0]))
            && (d < -(*dl).radius || d > (*dl).radius)
        {
            // dlight doesn't reach the plane
            dlightBits &= !(1 << i);
        }
    }

    if dlightBits == 0 {
        tr.pc.c_dlightSurfacesCulled += 1;
    }

    (*face).dlightBits = dlightBits;
    dlightBits
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightGrid(grid: *mut srfGridMesh_t, mut dlightBits: c_int) -> c_int {
    let i: c_int;
    let dl: *mut dlight_t;

    for i in 0..tr.refdef.num_dlights {
        if (dlightBits & (1 << i)) == 0 {
            continue;
        }
        dl = (*tr.refdef.dlights).add(i as usize);
        if (*dl).origin.0[0] - (*dl).radius > (*grid).meshBounds[1][0]
            || (*dl).origin.0[0] + (*dl).radius < (*grid).meshBounds[0][0]
            || (*dl).origin.0[1] - (*dl).radius > (*grid).meshBounds[1][1]
            || (*dl).origin.0[1] + (*dl).radius < (*grid).meshBounds[0][1]
            || (*dl).origin.0[2] - (*dl).radius > (*grid).meshBounds[1][2]
            || (*dl).origin.0[2] + (*dl).radius < (*grid).meshBounds[0][2]
        {
            // dlight doesn't reach the bounds
            dlightBits &= !(1 << i);
        }
    }

    if dlightBits == 0 {
        tr.pc.c_dlightSurfacesCulled += 1;
    }

    (*grid).dlightBits = dlightBits;
    dlightBits
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightTrisurf(surf: *mut srfTriangles_t, dlightBits: c_int) -> c_int {
    // FIXME: more dlight culling to trisurfs...
    (*surf).dlightBits = dlightBits;
    dlightBits
    // Note: the following #if 0 block is omitted as it's dead code in the original
}

/*
====================
R_DlightSurface

The given surface is going to be drawn, and it touches a leaf
that is touched by one or more dlights, so try to throw out
more dlights if possible.
====================
*/
#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightSurface(surf: *mut msurface_t, mut dlightBits: c_int) -> c_int {
    if *((*surf).data as *mut c_int) == SF_FACE {
        dlightBits = R_DlightFace((*surf).data as *mut srfSurfaceFace_t, dlightBits);
    } else if *((*surf).data as *mut c_int) == SF_GRID {
        dlightBits = R_DlightGrid((*surf).data as *mut srfGridMesh_t, dlightBits);
    } else if *((*surf).data as *mut c_int) == SF_TRIANGLES {
        dlightBits = R_DlightTrisurf((*surf).data as *mut srfTriangles_t, dlightBits);
    } else {
        dlightBits = 0;
    }

    if dlightBits != 0 {
        tr.pc.c_dlightSurfaces += 1;
    }

    dlightBits
}

/*
======================
R_AddWorldSurface
======================
*/
#[cfg(feature = "vv_lighting")]
pub unsafe fn R_AddWorldSurface(surf: *mut msurface_t, dlightBits: c_int, noViewCount: bool) {
    /*
    if ( surf->viewCount == tr.viewCount ) {
        return;		// already in this view
    }
    */

    // rww - changed this to be like sof2mp's so RMG will look right.
    // Will this affect anything that is non-rmg?

    if !noViewCount {
        if (*surf).viewCount == tr.viewCount {
            // already in this view, but lets make sure all the dlight bits are set
            if *((*surf).data as *mut c_int) == SF_FACE {
                let face = (*surf).data as *mut srfSurfaceFace_t;
                (*face).dlightBits |= dlightBits;
            } else if *((*surf).data as *mut c_int) == SF_GRID {
                let grid = (*surf).data as *mut srfGridMesh_t;
                (*grid).dlightBits |= dlightBits;
            } else if *((*surf).data as *mut c_int) == SF_TRIANGLES {
                let tri = (*surf).data as *mut srfTriangles_t;
                (*tri).dlightBits |= dlightBits;
            }
            return;
        }
        (*surf).viewCount = tr.viewCount;
        // FIXME: bmodel fog?
    }

    // surf->viewCount = tr.viewCount;
    // FIXME: bmodel fog?

    // try to cull before dlighting or adding
    if R_CullSurface((*surf).data, (*surf).shader as *mut shader_t) {
        return;
    }

    // check for dlighting
    let mut dlightBits = dlightBits;
    if dlightBits != 0 {
        dlightBits = VVLightMan.R_DlightSurface(surf, dlightBits);
        dlightBits = if dlightBits != 0 { 1 } else { 0 };
    }

    R_AddDrawSurf((*surf).data, (*surf).shader, (*surf).fogIndex, dlightBits);
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_AddWorldSurface(surf: *mut msurface_t, dlightBits: c_int, noViewCount: bool) {
    /*
    if ( surf->viewCount == tr.viewCount ) {
        return;		// already in this view
    }
    */

    // rww - changed this to be like sof2mp's so RMG will look right.
    // Will this affect anything that is non-rmg?

    if !noViewCount {
        if (*surf).viewCount == tr.viewCount {
            // already in this view, but lets make sure all the dlight bits are set
            if *((*surf).data as *mut c_int) == SF_FACE {
                let face = (*surf).data as *mut srfSurfaceFace_t;
                (*face).dlightBits |= dlightBits;
            } else if *((*surf).data as *mut c_int) == SF_GRID {
                let grid = (*surf).data as *mut srfGridMesh_t;
                (*grid).dlightBits |= dlightBits;
            } else if *((*surf).data as *mut c_int) == SF_TRIANGLES {
                let tri = (*surf).data as *mut srfTriangles_t;
                (*tri).dlightBits |= dlightBits;
            }
            return;
        }
        (*surf).viewCount = tr.viewCount;
        // FIXME: bmodel fog?
    }

    // surf->viewCount = tr.viewCount;
    // FIXME: bmodel fog?

    // try to cull before dlighting or adding
    if R_CullSurface((*surf).data, (*surf).shader as *mut shader_t) {
        return;
    }

    // check for dlighting
    let mut dlightBits = dlightBits;
    if dlightBits != 0 {
        dlightBits = R_DlightSurface(surf, dlightBits);
        dlightBits = if dlightBits != 0 { 1 } else { 0 };
    }

    R_AddDrawSurf((*surf).data, (*surf).shader, (*surf).fogIndex, dlightBits);
}

/*
=============================================================

    BRUSH MODELS

=============================================================
*/

/*
=================
R_AddBrushModelSurfaces
=================
*/
pub unsafe fn R_AddBrushModelSurfaces(ent: *mut trRefEntity_t) {
    let bmodel: *mut bmodel_t;
    let clip: c_int;
    let pModel: *mut model_t;
    let mut i: c_int;

    pModel = R_GetModelByHandle((*ent).e.hModel);

    bmodel = (*pModel).bmodel;

    clip = R_CullLocalBox(&(*bmodel).bounds[0]);
    if clip == CULL_OUT {
        return;
    }

    if (*pModel).bspInstance != 0 {
        #[cfg(feature = "vv_lighting")]
        {
            VVLightMan.R_SetupEntityLighting(&mut tr.refdef, ent);
        }
        #[cfg(not(feature = "vv_lighting"))]
        {
            R_SetupEntityLighting(&mut tr.refdef, ent);
        }
    }

    #[cfg(feature = "vv_lighting")]
    {
        VVLightMan.R_DlightBmodel(bmodel, 0);
    }
    #[cfg(not(feature = "vv_lighting"))]
    {
        R_DlightBmodel(bmodel, 0);
    }

    for i in 0..(*bmodel).numSurfaces {
        R_AddWorldSurface(
            (*bmodel).firstSurface.add(i as usize),
            (*(*tr.currentEntity)).dlightBits,
            true,
        );
    }
}

fn GetQuadArea(v1: vec3_t, v2: vec3_t, v3: vec3_t, v4: vec3_t) -> f32 {
    let mut vec1: vec3_t = vec3_t([0.0; 3]);
    let mut vec2: vec3_t = vec3_t([0.0; 3]);
    let mut dis1: vec3_t = vec3_t([0.0; 3]);
    let mut dis2: vec3_t = vec3_t([0.0; 3]);

    unsafe {
        // Get area of tri1
        VectorSubtract(v1, v2, &mut vec1);
        VectorSubtract(v1, v4, &mut vec2);
        CrossProduct(vec1, vec2, &mut dis1);
        VectorScale(&mut dis1, 0.25, &mut dis1);

        // Get area of tri2
        VectorSubtract(v3, v2, &mut vec1);
        VectorSubtract(v3, v4, &mut vec2);
        CrossProduct(vec1, vec2, &mut dis2);
        VectorScale(&mut dis2, 0.25, &mut dis2);

        // Return addition of disSqr of each tri area
        dis1.0[0] * dis1.0[0]
            + dis1.0[1] * dis1.0[1]
            + dis1.0[2] * dis1.0[2]
            + dis2.0[0] * dis2.0[0]
            + dis2.0[1] * dis2.0[1]
            + dis2.0[2] * dis2.0[2]
    }
}

#[cfg(target_os = "xbox")]
fn GetQuadAreaShort(
    v1: &[u16; 3],
    v2: &[u16; 3],
    v3: &[u16; 3],
    v4: &[u16; 3],
) -> f32 {
    let mut fv1: vec3_t = vec3_t([0.0; 3]);
    let mut fv2: vec3_t = vec3_t([0.0; 3]);
    let mut fv3: vec3_t = vec3_t([0.0; 3]);
    let mut fv4: vec3_t = vec3_t([0.0; 3]);

    unsafe {
        for i in 0..3 {
            // Q_CastShort2Float(&fv1[i], (short*)&v1[i]);
            // Q_CastShort2Float(&fv2[i], (short*)&v2[i]);
            // Q_CastShort2Float(&fv3[i], (short*)&v3[i]);
            // Q_CastShort2Float(&fv4[i], (short*)&v4[i]);
        }

        GetQuadArea(fv1, fv2, fv3, fv4)
    }
}

pub unsafe fn RE_GetBModelVerts(
    bmodelIndex: c_int,
    verts: *mut vec3_t,
    normal: vec3_t,
) {
    let mut surfs: *mut msurface_t;
    let mut face: *mut srfSurfaceFace_t;
    let bmodel: *mut bmodel_t;
    let pModel: *mut model_t;
    let mut i: c_int;
    // Not sure if we really need to track the best two candidates
    let mut maxDist: [c_int; 2] = [0, 0];
    let mut maxIndx: [c_int; 2] = [0, 0];
    let mut dist: c_int = 0;
    let dot1: f32;
    let dot2: f32;

    pModel = R_GetModelByHandle(bmodelIndex);
    bmodel = (*pModel).bmodel;

    // Loop through all surfaces on the brush and find the best two candidates
    for i in 0..(*bmodel).numSurfaces {
        surfs = (*bmodel).firstSurface.add(i as usize);
        face = (*surfs).data as *mut srfSurfaceFace_t;

        // It seems that the safest way to handle this is by finding the area of the faces
        #[cfg(target_os = "xbox")]
        {
            // let nextSurfPoint = NEXT_SURFPOINT(face->flags);
            // dist = GetQuadArea( face->srfPoints, face->srfPoints + nextSurfPoint,
            //         face->srfPoints + nextSurfPoint * 2, face->srfPoints +
            //                  nextSurfPoint * 3 );
        }
        #[cfg(not(target_os = "xbox"))]
        {
            // Note: face->points access would need to be adapted for Rust
            // dist = GetQuadArea( face->points[0], face->points[1], face->points[2], face->points[3] );
        }

        // Check against the highest max
        if dist > maxDist[0] {
            // Shuffle our current maxes down
            maxDist[1] = maxDist[0];
            maxIndx[1] = maxIndx[0];

            maxDist[0] = dist;
            maxIndx[0] = i;
        }
        // Check against the second highest max
        else if dist >= maxDist[1] {
            // just stomp the old
            maxDist[1] = dist;
            maxIndx[1] = i;
        }
    }

    // Hopefully we've found two best case candidates.  Now we should see which of these faces the viewer
    surfs = (*bmodel).firstSurface.add(maxIndx[0] as usize);
    face = (*surfs).data as *mut srfSurfaceFace_t;
    dot1 = DotProduct((*face).plane.normal, tr.refdef.viewaxis[0]);

    surfs = (*bmodel).firstSurface.add(maxIndx[1] as usize);
    face = (*surfs).data as *mut srfSurfaceFace_t;
    dot2 = DotProduct((*face).plane.normal, tr.refdef.viewaxis[0]);

    if dot2 < dot1 && dot2 < 0.0 {
        i = maxIndx[1]; // use the second face
    } else if dot1 < dot2 && dot1 < 0.0 {
        i = maxIndx[0]; // use the first face
    } else {
        // Possibly only have one face, so may as well use the first face, which also should be the best one
        // i = rand() & 1; // ugh, we don't know which to use.  I'd hope this would never happen
        i = maxIndx[0]; // use the first face
    }

    surfs = (*bmodel).firstSurface.add(i as usize);
    face = (*surfs).data as *mut srfSurfaceFace_t;

    #[cfg(target_os = "xbox")]
    {
        // let nextSurfPoint = NEXT_SURFPOINT(face->flags);
        // for ( int t = 0; t < 4; t++ )
        // {
        //     Q_CastShort2Float(&verts[t][0], (short*)(face->srfPoints + nextSurfPoint * t + 0));
        //     Q_CastShort2Float(&verts[t][1], (short*)(face->srfPoints + nextSurfPoint * t + 1));
        //     Q_CastShort2Float(&verts[t][2], (short*)(face->srfPoints + nextSurfPoint * t + 2));
        // }
    }
    #[cfg(not(target_os = "xbox"))]
    {
        // Note: face->points access would need to be adapted
        // for ( int t = 0; t < 4; t++ )
        // {
        //     VectorCopy(	face->points[t], verts[t] );
        // }
    }
}

/*
=============================================================

    WORLD MODEL

=============================================================
*/

/*
================
R_RecursiveWorldNode
================
*/
#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_RecursiveWorldNode(node: *mut mnode_t, mut planeBits: c_int, mut dlightBits: c_int) {
    loop {
        let mut newDlights: [c_int; 2];

        // if the node wasn't marked as potentially visible, exit
        if (*node).visframe != tr.visCount {
            return;
        }

        // if the bounding volume is outside the frustum, nothing
        // inside can be visible OPTIMIZE: don't do this all the way to leafs?

        if (*r_nocull).integer != 1 {
            let mut r: c_int;

            if (planeBits & 1) != 0 {
                r = BoxOnPlaneSide(&(*node).mins, &(*node).maxs, &tr.viewParms.frustum[0]);
                if r == 2 {
                    return; // culled
                }
                if r == 1 {
                    planeBits &= !1; // all descendants will also be in front
                }
            }

            if (planeBits & 2) != 0 {
                r = BoxOnPlaneSide(&(*node).mins, &(*node).maxs, &tr.viewParms.frustum[1]);
                if r == 2 {
                    return; // culled
                }
                if r == 1 {
                    planeBits &= !2; // all descendants will also be in front
                }
            }

            if (planeBits & 4) != 0 {
                r = BoxOnPlaneSide(&(*node).mins, &(*node).maxs, &tr.viewParms.frustum[2]);
                if r == 2 {
                    return; // culled
                }
                if r == 1 {
                    planeBits &= !4; // all descendants will also be in front
                }
            }

            if (planeBits & 8) != 0 {
                r = BoxOnPlaneSide(&(*node).mins, &(*node).maxs, &tr.viewParms.frustum[3]);
                if r == 2 {
                    return; // culled
                }
                if r == 1 {
                    planeBits &= !8; // all descendants will also be in front
                }
            }

            if (planeBits & 16) != 0 {
                r = BoxOnPlaneSide(&(*node).mins, &(*node).maxs, &tr.viewParms.frustum[4]);
                if r == 2 {
                    return; // culled
                }
                if r == 1 {
                    planeBits &= !16; // all descendants will also be in front
                }
            }
        }

        if (*node).contents != -1 {
            break;
        }

        // determine which dlights are needed
        if (*r_nocull).integer != 2 {
            newDlights[0] = 0;
            newDlights[1] = 0;
            if dlightBits != 0 {
                let mut i: c_int;
                for i in 0..tr.refdef.num_dlights {
                    let mut dl: *mut dlight_t;
                    let mut dist: f32;

                    if (dlightBits & (1 << i)) != 0 {
                        dl = (*tr.refdef.dlights).add(i as usize);
                        dist = DotProduct((*dl).origin, *(*(*node).plane).normal) - (*(*node).plane).dist;

                        if dist > -(*dl).radius {
                            newDlights[0] |= 1 << i;
                        }
                        if dist < (*dl).radius {
                            newDlights[1] |= 1 << i;
                        }
                    }
                }
            }
        } else {
            newDlights[0] = dlightBits;
            newDlights[1] = dlightBits;
        }
        // recurse down the children, front side first
        R_RecursiveWorldNode((*node).children[0], planeBits, newDlights[0]);

        // tail recurse
        node = (*node).children[1];
        dlightBits = newDlights[1];
    }

    // leaf node, so add mark surfaces
    let mut c: c_int;
    let mut surf: *mut msurface_t;
    let mut mark: *mut *mut msurface_t;

    tr.pc.c_leafs += 1;

    // add to z buffer bounds
    if (*node).mins[0] < tr.viewParms.visBounds[0][0] {
        tr.viewParms.visBounds[0][0] = (*node).mins[0];
    }
    if (*node).mins[1] < tr.viewParms.visBounds[0][1] {
        tr.viewParms.visBounds[0][1] = (*node).mins[1];
    }
    if (*node).mins[2] < tr.viewParms.visBounds[0][2] {
        tr.viewParms.visBounds[0][2] = (*node).mins[2];
    }

    if (*node).maxs[0] > tr.viewParms.visBounds[1][0] {
        tr.viewParms.visBounds[1][0] = (*node).maxs[0];
    }
    if (*node).maxs[1] > tr.viewParms.visBounds[1][1] {
        tr.viewParms.visBounds[1][1] = (*node).maxs[1];
    }
    if (*node).maxs[2] > tr.viewParms.visBounds[1][2] {
        tr.viewParms.visBounds[1][2] = (*node).maxs[2];
    }

    // add the individual surfaces
    mark = (*node).firstmarksurface;
    c = (*node).nummarksurfaces;
    while c > 0 {
        // the surface may have already been added if it
        // spans multiple leafs
        surf = *mark;
        R_AddWorldSurface(surf, dlightBits, false);
        mark = mark.add(1);
        c -= 1;
    }
}

/*
===============
R_PointInLeaf
===============
*/
unsafe fn R_PointInLeaf(p: vec3_t) -> *mut mnode_t {
    let mut node: *mut mnode_t;
    let d: f32;
    let plane: *mut cplane_t;

    if tr.world.is_null() {
        Com_Error(ERR_DROP, "R_PointInLeaf: bad model\0".as_ptr() as *const u8);
    }

    node = (*tr.world).nodes;
    loop {
        if (*node).contents != -1 {
            break;
        }
        #[cfg(target_os = "xbox")]
        {
            plane = (*(*tr.world).planes).add((*node).planeNum as usize);
        }
        #[cfg(not(target_os = "xbox"))]
        {
            plane = (*node).plane;
        }
        d = DotProduct(p, (*plane).normal) - (*plane).dist;
        if d > 0.0 {
            node = (*node).children[0];
        } else {
            node = (*node).children[1];
        }
    }

    node
}

/*
==============
R_ClusterPVS
==============
*/
unsafe fn R_ClusterPVS(cluster: c_int) -> *mut u8 {
    if tr.world.is_null()
        || (*tr.world).vis.is_null()
        || cluster < 0
        || cluster >= (*tr.world).numClusters
    {
        return (*tr.world).novis;
    }

    #[cfg(target_os = "xbox")]
    {
        // Note: This would require calling a decompress function on Xbox
        // return tr.world->vis->Decompress(cluster * tr.world->clusterBytes,
        //         tr.world->numClusters);
        (*tr.world).vis
    }
    #[cfg(not(target_os = "xbox"))]
    {
        (*tr.world)
            .vis
            .add((cluster * (*tr.world).clusterBytes) as usize)
    }
}

/*
=================
R_inPVS
=================
*/
#[cfg(target_os = "xbox")]
pub unsafe fn R_inPVS(p1: vec3_t, p2: vec3_t) -> bool {
    let mut leaf: *mut mleaf_s;
    let mut vis: *mut u8;

    leaf = R_PointInLeaf(p1) as *mut mleaf_s;
    vis = CM_ClusterPVS((*leaf).cluster);
    leaf = R_PointInLeaf(p2) as *mut mleaf_s;

    if vis.is_null() || ((*vis.add((*leaf).cluster as usize >> 3) & (1 << ((*leaf).cluster & 7))) == 0) {
        return false;
    }
    true
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn R_inPVS(p1: vec3_t, p2: vec3_t) -> bool {
    let mut leaf: *mut mnode_t;
    let vis: *mut u8;

    leaf = R_PointInLeaf(p1);
    vis = CM_ClusterPVS((*leaf).cluster);
    leaf = R_PointInLeaf(p2);

    if (*vis.add((*leaf).cluster as usize >> 3) & (1 << ((*leaf).cluster & 7))) == 0 {
        return false;
    }
    true
}

/*
===============
R_MarkLeaves

Mark the leaves and nodes that are in the PVS for the current
cluster
===============
*/
#[cfg(target_os = "xbox")]
pub unsafe fn R_MarkLeaves(leafOverride: *mut mleaf_s) {
    let vis: *const u8;
    let mut leaf: *mut mleaf_s;
    let mut parent: *mut mnode_t;
    let mut i: c_int;
    let cluster: c_int;

    // lockpvs lets designers walk around to determine the
    // extent of the current pvs
    if (*r_lockpvs).integer != 0 {
        return;
    }

    // current viewcluster
    if leafOverride.is_null() {
        leaf = R_PointInLeaf(tr.viewParms.pvsOrigin) as *mut mleaf_s;
    } else {
        leaf = leafOverride;
    }
    cluster = (*leaf).cluster;

    // Note: This is an assertion in the original code
    // assert(leaf->contents != -1);

    // if the cluster is the same and the area visibility matrix
    // hasn't changed, we don't need to mark everything again

    if tr.viewCluster == cluster && tr.refdef.areamaskModified == 0 {
        return;
    }

    tr.visCount += 1;
    tr.viewCluster = cluster;

    if (*r_novis).integer != 0 || tr.viewCluster == -1 {
        for i in 0..(*tr.world).numnodes {
            if (*(*tr.world).nodes.add(i as usize)).contents != CONTENTS_SOLID {
                (*(*tr.world).nodes.add(i as usize)).visframe = tr.visCount;
            }
        }
        return;
    }

    vis = R_ClusterPVS(tr.viewCluster);

    let mut leaf = (*tr.world).leafs;
    for i in 0..(*tr.world).numleafs {
        cluster = (*leaf).cluster;
        if cluster < 0 || cluster >= (*tr.world).numClusters {
            leaf = leaf.add(1);
            continue;
        }

        // check general pvs
        if (*vis.add(cluster as usize >> 3) & (1 << (cluster & 7))) == 0 {
            leaf = leaf.add(1);
            continue;
        }

        // check for door connection
        if !lookingForWorstLeaf
            && ((*tr.refdef.areamask.as_ptr().add((*leaf).area as usize >> 3))
                & (1 << ((*leaf).area & 7)))
                != 0
        {
            leaf = leaf.add(1);
            continue; // not visible
        }

        parent = leaf as *mut mnode_t;
        // Note: This is an assertion in the original code
        // assert(leaf->contents != -1);
        loop {
            if (*parent).visframe == tr.visCount {
                break;
            }
            (*parent).visframe = tr.visCount;
            parent = (*parent).parent;
            if parent.is_null() {
                break;
            }
        }

        leaf = leaf.add(1);
    }
}

#[cfg(not(target_os = "xbox"))]
unsafe fn R_MarkLeaves() {
    let vis: *const u8;
    let mut leaf: *mut mnode_t;
    let mut parent: *mut mnode_t;
    let mut i: c_int;
    let cluster: c_int;

    // lockpvs lets designers walk around to determine the
    // extent of the current pvs
    if (*r_lockpvs).integer != 0 {
        return;
    }

    // current viewcluster
    leaf = R_PointInLeaf(tr.viewParms.pvsOrigin);
    cluster = (*leaf).cluster;

    // if the cluster is the same and the area visibility matrix
    // hasn't changed, we don't need to mark everything again

    // if r_showcluster was just turned on, remark everything
    if tr.viewCluster == cluster && tr.refdef.areamaskModified == 0 && (*r_showcluster).modified == 0
    {
        return;
    }

    if (*r_showcluster).modified != 0 || (*r_showcluster).integer != 0 {
        (*r_showcluster).modified = 0;
        if (*r_showcluster).integer != 0 {
            VID_Printf(
                PRINT_ALL,
                "cluster:%i  area:%i\n\0".as_ptr() as *const u8,
                cluster,
                (*leaf).area,
            );
        }
    }

    tr.visCount += 1;
    tr.viewCluster = cluster;

    if (*r_novis).integer != 0 || tr.viewCluster == -1 {
        for i in 0..(*tr.world).numnodes {
            if (*(*tr.world).nodes.add(i as usize)).contents != CONTENTS_SOLID {
                (*(*tr.world).nodes.add(i as usize)).visframe = tr.visCount;
            }
        }
        return;
    }

    vis = R_ClusterPVS(tr.viewCluster);

    let mut leaf = (*tr.world).nodes;
    for i in 0..(*tr.world).numnodes {
        cluster = (*leaf).cluster;
        if cluster < 0 || cluster >= (*tr.world).numClusters {
            leaf = leaf.add(1);
            continue;
        }

        // check general pvs
        if (*vis.add(cluster as usize >> 3) & (1 << (cluster & 7))) == 0 {
            leaf = leaf.add(1);
            continue;
        }

        // check for door connection
        if ((*tr.refdef.areamask.as_ptr().add((*leaf).area as usize >> 3))
            & (1 << ((*leaf).area & 7)))
            != 0
        {
            leaf = leaf.add(1);
            continue; // not visible
        }

        parent = leaf;
        loop {
            if (*parent).visframe == tr.visCount {
                break;
            }
            (*parent).visframe = tr.visCount;
            parent = (*parent).parent;
            if parent.is_null() {
                break;
            }
        }

        leaf = leaf.add(1);
    }
}

/*
=============
R_AddWorldSurfaces
=============
*/
#[cfg(target_os = "xbox")]
pub unsafe fn R_AddWorldSurfaces() {
    if (*r_drawworld).integer == 0 {
        return;
    }

    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return;
    }

    tr.currentEntityNum = TR_WORLDENT;
    tr.shiftedEntityNum = tr.currentEntityNum << QSORT_ENTITYNUM_SHIFT;

    // clear out the visible min/max
    ClearBounds(&mut tr.viewParms.visBounds[0], &mut tr.viewParms.visBounds[1]);

    // perform frustum culling and add all the potentially visible surfaces
    if VVLightMan.num_dlights > MAX_DLIGHTS {
        VVLightMan.num_dlights = MAX_DLIGHTS;
    }

    // Note: R_RecursiveWorldNode call would need VVLightMan version
    // VVLightMan.R_RecursiveWorldNode( tr.world->nodes, 15, ( 1 << VVLightMan.num_dlights ) - 1 );
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn R_AddWorldSurfaces() {
    if (*r_drawworld).integer == 0 {
        return;
    }

    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return;
    }

    tr.currentEntityNum = TR_WORLDENT;
    tr.shiftedEntityNum = tr.currentEntityNum << QSORT_ENTITYNUM_SHIFT;

    // determine which leaves are in the PVS / areamask
    R_MarkLeaves();

    // clear out the visible min/max
    ClearBounds(&mut tr.viewParms.visBounds[0], &mut tr.viewParms.visBounds[1]);

    // perform frustum culling and add all the potentially visible surfaces
    if tr.refdef.num_dlights > 32 {
        tr.refdef.num_dlights = 32;
    }

    R_RecursiveWorldNode((*tr.world).nodes, 31, (1 << tr.refdef.num_dlights) - 1);
}
