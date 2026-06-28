// tr_main.c -- main control flow for each frame
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "tr_local.h"
// #include "../ghoul2/G2_local.h"
// Yeah, this might be kind of bad, but no linux version is planned so far :-) - AReis
// Gee- thanks guys - jdrews, the linux porter...
// #ifndef _XBOX
// #ifndef __linux__
// #include "../win32/glw_win.h"
// #endif
// #endif

use core::ffi::c_int;

// Local type stubs for now - these would be defined in tr_local.h, qcommon headers, etc.
// Placeholder structs to allow this file to compile in isolation
#[repr(C)]
pub struct trGlobals_t {
    // Placeholder - actual definition would be in tr_local.h
}

#[repr(C)]
pub struct viewParms_t {
    // Placeholder
}

#[repr(C)]
pub struct cplane_t {
    // Placeholder
}

#[repr(C)]
pub struct trRefEntity_t {
    // Placeholder
}

#[repr(C)]
pub struct orientationr_t {
    // Placeholder
}

#[repr(C)]
pub struct orientation_t {
    // Placeholder
}

#[repr(C)]
pub struct drawSurf_t {
    // Placeholder
}

#[repr(C)]
pub struct shader_t {
    // Placeholder
}

#[repr(C)]
pub struct surfaceType_t {
    // Placeholder
}

#[repr(C)]
pub struct srfTriangles_t {
    // Placeholder
}

#[repr(C)]
pub struct srfPoly_t {
    // Placeholder
}

#[repr(C)]
pub struct drawVert_t {
    // Placeholder
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    // Placeholder
}

#[repr(C)]
pub struct fog_t {
    // Placeholder
}

// Type aliases
pub type vec3_t = [f32; 3];
pub type vec4_t = [f32; 4];
pub type qboolean = c_int;

// Global variables
pub static mut tr: trGlobals_t = unsafe { core::mem::zeroed() };

static s_flipMatrix: [f32; 16] = [
    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    #[cfg(target_os = "xbox")]
    {
        0.0, 0.0, 1.0, 0.0,
        -1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    }
    #[cfg(not(target_os = "xbox"))]
    {
        0.0, 0.0, -1.0, 0.0,
        -1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    }
];

// External function declarations
extern "C" {
    fn R_AddTerrainSurfaces();
    fn R_LocalPointToWorld(local: *const vec3_t, world: *mut vec3_t);
    fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    fn VectorMA(dst: *mut vec3_t, scale: f32, src: *const vec3_t, out: *mut vec3_t);
    fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
    fn VectorScale(src: *const vec3_t, scale: f32, dst: *mut vec3_t);
    fn PerpendicularVector(dst: *mut vec3_t, src: *const vec3_t);
    fn CrossProduct(src1: *const vec3_t, src2: *const vec3_t, dst: *mut vec3_t);
    fn VectorSubtract(src1: *const vec3_t, src2: *const vec3_t, dst: *mut vec3_t);
    fn VectorAdd(src1: *const vec3_t, src2: *const vec3_t, dst: *mut vec3_t);
    fn VectorClear(dst: *mut vec3_t);
    fn VectorLength(v: *const vec3_t) -> f32;
    fn VectorLengthSquared(v: *const vec3_t) -> f32;
    fn DistanceSquared(src1: *const vec3_t, src2: *const vec3_t) -> f32;
    fn RotatePointAroundVector(dst: *mut vec3_t, axis: *const vec3_t, src: *const vec3_t, angle: f32);
    fn PlaneFromPoints(plane: *mut vec4_t, p1: *const vec3_t, p2: *const vec3_t, p3: *const vec3_t);
    fn AxisCopy(src: *const [[f32; 3]; 3], dst: *mut [[f32; 3]; 3]);
    fn SetPlaneSignbits(plane: *mut cplane_t);
    fn Com_Memset(dst: *mut core::ffi::c_void, value: c_int, count: usize);
    fn Com_Clamp(min: f32, max: f32, value: f32) -> f32;
    fn Com_Printf(format: *const c_int, ...);
    fn Com_DPrintf(format: *const c_int, ...);
    fn Com_Error(error_type: c_int, format: *const c_int, ...);
    fn R_DecomposeSort(sort: c_int, entity_num: *mut c_int, shader: *mut *mut shader_t, fog_num: *mut c_int, dlight_map: *mut c_int);
    fn RB_BeginSurface(shader: *mut shader_t, fog_num: c_int);
    fn R_TransformModelToClip(src: *const vec3_t, model_matrix: *const f32, projection_matrix: *const f32, eye: *mut vec4_t, dst: *mut vec4_t);
    fn R_GetShaderByHandle(handle: c_int) -> *mut shader_t;
    fn R_GetModelByHandle(handle: c_int) -> *mut core::ffi::c_void;
    fn R_AddMD3Surfaces(ent: *mut trRefEntity_t);
    fn R_AddBrushModelSurfaces(ent: *mut trRefEntity_t);
    fn R_AddGhoulSurfaces(ent: *mut trRefEntity_t);
    fn G2API_HaveWeGhoul2Models(ghoul2: *const core::ffi::c_void) -> bool;
    fn R_MarkLeaves(leaf: *mut core::ffi::c_void);
    fn R_AddWorldSurfaces();
    fn R_AddPolygonSurfaces();
    fn R_SetupProjection();
    fn R_AddEntitySurfaces();
    fn R_SyncRenderThread();
    fn GL_Bind(image: *mut core::ffi::c_void);
    fn GL_Cull(cull_type: c_int);
    fn GL_State(state: c_int);
    fn CM_DrawDebugSurface(callback: extern "C" fn(c_int, c_int, *mut f32));
    fn R_AddDrawSurfCmd(draw_surfs: *mut drawSurf_t, num_draw_surfs: c_int);
    fn R_RenderView(parms: *mut viewParms_t);
    fn R_AddDrawSurf(surface: *mut surfaceType_t, shader: *mut shader_t, fog_index: c_int, dlight_map: c_int);
    fn qglColor3f(r: f32, g: f32, b: f32);
    fn qglBegin(mode: c_int);
    fn qglVertex3fv(v: *const f32);
    fn qglEnd();
    fn qglDepthRange(z_near: f32, z_far: f32);

    // Conditional declaration for Xbox
    #[cfg(target_os = "xbox")]
    fn R_GenerateDrawSurfs(is_portal: bool);
    #[cfg(not(target_os = "xbox"))]
    fn R_GenerateDrawSurfs();
}

// Global variables (non-dedicated only)
#[cfg(not(feature = "dedicated"))]
pub static mut entitySurface: surfaceType_t = unsafe { core::mem::zeroed() };

static mut preTransEntMatrix: [f32; 16] = [0.0; 16];

// Constants
const CULL_IN: c_int = 0;
const CULL_CLIP: c_int = 1;
const CULL_OUT: c_int = 2;
const RT_MODEL: c_int = 0;
const RT_SPRITE: c_int = 1;
const RT_BEAM: c_int = 2;
const RT_ORIENTED_QUAD: c_int = 3;
const RT_ELECTRICITY: c_int = 4;
const RT_LINE: c_int = 5;
const RT_ORIENTEDLINE: c_int = 6;
const RT_CYLINDER: c_int = 7;
const RT_SABER_GLOW: c_int = 8;
const RT_PORTALSURFACE: c_int = 9;
const RT_ENT_CHAIN: c_int = 10;
const RF_FIRST_PERSON: c_int = 0x1;
const RF_THIRD_PERSON: c_int = 0x2;
const RF_SHADOW_ONLY: c_int = 0x4;
const RDF_NOWORLDMODEL: c_int = 0x1;
const RDF_AUTOMAP: c_int = 0x2;
const RDF_NOFOG: c_int = 0x4;
const SF_ENTITY: c_int = 0;
const SF_FACE: c_int = 1;
const SF_TRIANGLES: c_int = 2;
const SF_POLY: c_int = 3;
const MOD_MESH: c_int = 0;
const MOD_BRUSH: c_int = 1;
const MOD_MDXM: c_int = 2;
const MOD_BAD: c_int = 3;
const TR_WORLDENT: c_int = 0;
const SS_PORTAL: c_int = 0;
const SS_BAD: c_int = 1;
const PLANE_NON_AXIAL: c_int = 3;
const DRAWSURF_MASK: c_int = 0x3FFF;
const QSORT_SHADERNUM_SHIFT: c_int = 28;
const QSORT_FOGNUM_SHIFT: c_int = 5;
const QSORT_ENTITYNUM_SHIFT: c_int = 10;
const MAX_SHADERS: c_int = 256;
const MAX_ENTITIES: c_int = 2048;
const MAX_DRAWSURFS: c_int = 0x10000;
const CUTOFF: usize = 8;
const GLS_DEPTHMASK_TRUE: c_int = 0x1;
const GLS_SRCBLEND_ONE: c_int = 0x2;
const GLS_DSTBLEND_ONE: c_int = 0x4;
const GLS_POLYMODE_LINE: c_int = 0x8;
const GL_POLYGON: c_int = 0x9;
const CT_FRONT_SIDED: c_int = 0;
const ERR_DROP: c_int = 0;
const S_COLOR_RED: &[u8] = b"\x03";

// Stub cvar pointers (would be defined elsewhere)
extern "C" {
    pub static mut r_nocull: *mut c_int;
    pub static mut r_drawentities: *mut c_int;
    pub static mut r_znear: *mut c_int;
    pub static mut r_noportals: *mut c_int;
    pub static mut r_fastsky: *mut c_int;
    pub static mut r_portalOnly: *mut c_int;
    pub static mut r_debugSurface: *mut c_int;
}

extern "C" {
    pub static mut vec3_origin: vec3_t;
}

// Local helper structs for tess (tessellator) - placeholder
#[repr(C)]
pub struct tess_t {
    // Placeholder
}

extern "C" {
    pub static mut tess: tess_t;
    pub static mut rb_surfaceTable: [extern "C" fn(*mut surfaceType_t); 256];
}

#[cfg(not(feature = "dedicated"))]
pub unsafe fn R_CullLocalBox(bounds: &[[f32; 3]; 2]) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut transformed: [[f32; 3]; 8] = [[0.0; 3]; 8];
    let mut dists: [f32; 8] = [0.0; 8];
    let mut v: [f32; 3] = [0.0; 3];
    let mut frust: *mut cplane_t;
    let mut anyBack: c_int;
    let mut front: c_int;
    let mut back: c_int;

    if (*r_nocull).as_ref().map(|x| *x == 1).unwrap_or(false) {
        return CULL_CLIP;
    }

    // transform into world space
    i = 0;
    while i < 8 {
        v[0] = bounds[(i & 1) as usize][0];
        v[1] = bounds[((i >> 1) & 1) as usize][1];
        v[2] = bounds[((i >> 2) & 1) as usize][2];

        VectorCopy(&tr.ori.origin as *const _ as *const [f32; 3], &mut transformed[i as usize]);
        VectorMA(&mut transformed[i as usize], v[0], &tr.ori.axis[0], &mut transformed[i as usize]);
        VectorMA(&mut transformed[i as usize], v[1], &tr.ori.axis[1], &mut transformed[i as usize]);
        VectorMA(&mut transformed[i as usize], v[2], &tr.ori.axis[2], &mut transformed[i as usize]);
        i += 1;
    }

    // check against frustum planes
    anyBack = 0;
    i = 0;
    while i < 4 {
        frust = &mut tr.viewParms.frustum[i as usize];

        front = 0;
        back = 0;
        j = 0;
        while j < 8 {
            dists[j as usize] = DotProduct(&transformed[j as usize], &(*frust).normal as *const _ as *const [f32; 3]);
            if dists[j as usize] > (*frust).dist {
                front = 1;
                if back != 0 {
                    break;		// a point is in front
                }
            } else {
                back = 1;
            }
            j += 1;
        }
        if front == 0 {
            // all points were behind one of the planes
            return CULL_OUT;
        }
        anyBack |= back;
        i += 1;
    }

    if anyBack == 0 {
        return CULL_IN;		// completely inside frustum
    }

    return CULL_CLIP;		// partially clipped
}

pub unsafe fn R_CullLocalPointAndRadius(pt: &[f32; 3], radius: f32) -> c_int {
    let mut transformed: [f32; 3] = [0.0; 3];

    R_LocalPointToWorld(pt, &mut transformed);

    return R_CullPointAndRadius(&transformed, radius);
}

pub unsafe fn R_CullPointAndRadius(pt: &[f32; 3], radius: f32) -> c_int {
    let mut i: c_int;
    let mut dist: f32;
    let mut frust: *mut cplane_t;
    let mut mightBeClipped: qboolean = 0;

    if (*r_nocull).as_ref().map(|x| *x == 1).unwrap_or(false) {
        return CULL_CLIP;
    }

    // check against frustum planes
    i = 0;
    while i < 4 {
        frust = &mut tr.viewParms.frustum[i as usize];

        dist = DotProduct(pt, &(*frust).normal as *const _ as *const [f32; 3]) - (*frust).dist;
        if dist < -radius {
            return CULL_OUT;
        } else if dist <= radius {
            mightBeClipped = 1;
        }
        i += 1;
    }

    if mightBeClipped != 0 {
        return CULL_CLIP;
    }

    return CULL_IN;		// completely inside frustum
}

#[cfg(not(feature = "dedicated"))]
pub unsafe fn R_LocalNormalToWorld(local: &[f32; 3], world: &mut [f32; 3]) {
    world[0] = local[0] * tr.ori.axis[0][0] + local[1] * tr.ori.axis[1][0] + local[2] * tr.ori.axis[2][0];
    world[1] = local[0] * tr.ori.axis[0][1] + local[1] * tr.ori.axis[1][1] + local[2] * tr.ori.axis[2][1];
    world[2] = local[0] * tr.ori.axis[0][2] + local[1] * tr.ori.axis[1][2] + local[2] * tr.ori.axis[2][2];
}

pub unsafe fn R_LocalPointToWorld(local: &[f32; 3], world: &mut [f32; 3]) {
    world[0] = local[0] * tr.ori.axis[0][0] + local[1] * tr.ori.axis[1][0] + local[2] * tr.ori.axis[2][0] + tr.ori.origin[0];
    world[1] = local[0] * tr.ori.axis[0][1] + local[1] * tr.ori.axis[1][1] + local[2] * tr.ori.axis[2][1] + tr.ori.origin[1];
    world[2] = local[0] * tr.ori.axis[0][2] + local[1] * tr.ori.axis[1][2] + local[2] * tr.ori.axis[2][2] + tr.ori.origin[2];
}

#[cfg(not(feature = "dedicated"))]
pub unsafe fn R_WorldNormalToEntity(worldvec: &[f32; 3], entvec: &mut [f32; 3]) {
    entvec[0] = -worldvec[0] * preTransEntMatrix[0] - worldvec[1] * preTransEntMatrix[4] + worldvec[2] * preTransEntMatrix[8];
    entvec[1] = -worldvec[0] * preTransEntMatrix[1] - worldvec[1] * preTransEntMatrix[5] + worldvec[2] * preTransEntMatrix[9];
    entvec[2] = -worldvec[0] * preTransEntMatrix[2] - worldvec[1] * preTransEntMatrix[6] + worldvec[2] * preTransEntMatrix[10];
}

// R_WorldPointToEntity is commented out in original
/*
pub unsafe fn R_WorldPointToEntity (worldvec: &[f32; 3], entvec: &mut [f32; 3])
{
    entvec[0] = worldvec[0] * preTransEntMatrix[0] + worldvec[1] * preTransEntMatrix[4] + worldvec[2] * preTransEntMatrix[8]+preTransEntMatrix[12];
    entvec[1] = worldvec[0] * preTransEntMatrix[1] + worldvec[1] * preTransEntMatrix[5] + worldvec[2] * preTransEntMatrix[9]+preTransEntMatrix[13];
    entvec[2] = worldvec[0] * preTransEntMatrix[2] + worldvec[1] * preTransEntMatrix[6] + worldvec[2] * preTransEntMatrix[10]+preTransEntMatrix[14];
}
*/

pub unsafe fn R_WorldToLocal(world: &[f32; 3], local: &mut [f32; 3]) {
    local[0] = DotProduct(world, &tr.ori.axis[0] as *const _ as *const [f32; 3]);
    local[1] = DotProduct(world, &tr.ori.axis[1] as *const _ as *const [f32; 3]);
    local[2] = DotProduct(world, &tr.ori.axis[2] as *const _ as *const [f32; 3]);
}

pub unsafe fn R_TransformModelToClip(src: &[f32; 3], modelMatrix: *const f32, projectionMatrix: *const f32,
                                       eye: &mut [f32; 4], dst: &mut [f32; 4]) {
    let mut i: c_int;

    i = 0;
    while i < 4 {
        eye[i as usize] =
            src[0] * *modelMatrix.offset((i as isize + 0 * 4)) +
            src[1] * *modelMatrix.offset((i as isize + 1 * 4)) +
            src[2] * *modelMatrix.offset((i as isize + 2 * 4)) +
            1.0 * *modelMatrix.offset((i as isize + 3 * 4));
        i += 1;
    }

    i = 0;
    while i < 4 {
        dst[i as usize] =
            eye[0] * *projectionMatrix.offset((i as isize + 0 * 4)) +
            eye[1] * *projectionMatrix.offset((i as isize + 1 * 4)) +
            eye[2] * *projectionMatrix.offset((i as isize + 2 * 4)) +
            eye[3] * *projectionMatrix.offset((i as isize + 3 * 4));
        i += 1;
    }
}

pub unsafe fn R_TransformClipToWindow(clip: &[f32; 4], view: *const viewParms_t, normalized: &mut [f32; 4], window: &mut [f32; 4]) {
    normalized[0] = clip[0] / clip[3];
    normalized[1] = clip[1] / clip[3];
    normalized[2] = (clip[2] + clip[3]) / (2.0 * clip[3]);

    window[0] = 0.5 * (1.0 + normalized[0]) * (*view).viewportWidth;
    window[1] = 0.5 * (1.0 + normalized[1]) * (*view).viewportHeight;
    window[2] = normalized[2];

    window[0] = (window[0] + 0.5) as i32 as f32;
    window[1] = (window[1] + 0.5) as i32 as f32;
}

pub unsafe fn myGlMultMatrix(a: *const f32, b: *const f32, out: *mut f32) {
    let mut i: c_int;
    let mut j: c_int;

    i = 0;
    while i < 4 {
        j = 0;
        while j < 4 {
            *out.offset((i * 4 + j) as isize) =
                *a.offset((i * 4 + 0) as isize) * *b.offset((0 * 4 + j) as isize)
                + *a.offset((i * 4 + 1) as isize) * *b.offset((1 * 4 + j) as isize)
                + *a.offset((i * 4 + 2) as isize) * *b.offset((2 * 4 + j) as isize)
                + *a.offset((i * 4 + 3) as isize) * *b.offset((3 * 4 + j) as isize);
            j += 1;
        }
        i += 1;
    }
}

pub unsafe fn R_RotateForEntity(ent: *const trRefEntity_t, viewParms: *const viewParms_t,
                                 ori: *mut orientationr_t) {
    let mut delta: [f32; 3] = [0.0; 3];
    let mut axisLength: f32;

    if (*ent).e.reType != RT_MODEL {
        *ori = (*viewParms).world;
        return;
    }

    VectorCopy(&(*ent).e.origin, &mut (*ori).origin);

    VectorCopy(&(*ent).e.axis[0], &mut (*ori).axis[0]);
    VectorCopy(&(*ent).e.axis[1], &mut (*ori).axis[1]);
    VectorCopy(&(*ent).e.axis[2], &mut (*ori).axis[2]);

    preTransEntMatrix[0] = (*ori).axis[0][0];
    preTransEntMatrix[4] = (*ori).axis[1][0];
    preTransEntMatrix[8] = (*ori).axis[2][0];
    preTransEntMatrix[12] = (*ori).origin[0];

    preTransEntMatrix[1] = (*ori).axis[0][1];
    preTransEntMatrix[5] = (*ori).axis[1][1];
    preTransEntMatrix[9] = (*ori).axis[2][1];
    preTransEntMatrix[13] = (*ori).origin[1];

    preTransEntMatrix[2] = (*ori).axis[0][2];
    preTransEntMatrix[6] = (*ori).axis[1][2];
    preTransEntMatrix[10] = (*ori).axis[2][2];
    preTransEntMatrix[14] = (*ori).origin[2];

    preTransEntMatrix[3] = 0.0;
    preTransEntMatrix[7] = 0.0;
    preTransEntMatrix[11] = 0.0;
    preTransEntMatrix[15] = 1.0;

    myGlMultMatrix(&preTransEntMatrix[0], &(*viewParms).world.modelMatrix[0], &mut (*ori).modelMatrix[0]);

    // calculate the viewer origin in the model's space
    // needed for fog, specular, and environment mapping
    VectorSubtract(&(*viewParms).ori.origin, &(*ori).origin, &mut delta);

    // compensate for scale in the axes if necessary
    if (*ent).e.nonNormalizedAxes != 0 {
        axisLength = VectorLength(&(*ent).e.axis[0]);
        if axisLength == 0.0 {
            axisLength = 0.0;
        } else {
            axisLength = 1.0 / axisLength;
        }
    } else {
        axisLength = 1.0;
    }

    (*ori).viewOrigin[0] = DotProduct(&delta, &(*ori).axis[0] as *const _ as *const [f32; 3]) * axisLength;
    (*ori).viewOrigin[1] = DotProduct(&delta, &(*ori).axis[1] as *const _ as *const [f32; 3]) * axisLength;
    (*ori).viewOrigin[2] = DotProduct(&delta, &(*ori).axis[2] as *const _ as *const [f32; 3]) * axisLength;
}

pub unsafe fn R_RotateForViewer() {
    let mut viewerMatrix: [f32; 16] = [0.0; 16];
    let mut origin: [f32; 3] = [0.0; 3];

    Com_Memset(&mut tr.ori as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of_val(&tr.ori));
    tr.ori.axis[0][0] = 1.0;
    tr.ori.axis[1][1] = 1.0;
    tr.ori.axis[2][2] = 1.0;
    VectorCopy(&tr.viewParms.ori.origin, &mut tr.ori.viewOrigin);

    // transform by the camera placement
    VectorCopy(&tr.viewParms.ori.origin, &mut origin);

    viewerMatrix[0] = tr.viewParms.ori.axis[0][0];
    viewerMatrix[4] = tr.viewParms.ori.axis[0][1];
    viewerMatrix[8] = tr.viewParms.ori.axis[0][2];
    viewerMatrix[12] = -origin[0] * viewerMatrix[0] + -origin[1] * viewerMatrix[4] + -origin[2] * viewerMatrix[8];

    viewerMatrix[1] = tr.viewParms.ori.axis[1][0];
    viewerMatrix[5] = tr.viewParms.ori.axis[1][1];
    viewerMatrix[9] = tr.viewParms.ori.axis[1][2];
    viewerMatrix[13] = -origin[0] * viewerMatrix[1] + -origin[1] * viewerMatrix[5] + -origin[2] * viewerMatrix[9];

    viewerMatrix[2] = tr.viewParms.ori.axis[2][0];
    viewerMatrix[6] = tr.viewParms.ori.axis[2][1];
    viewerMatrix[10] = tr.viewParms.ori.axis[2][2];
    viewerMatrix[14] = -origin[0] * viewerMatrix[2] + -origin[1] * viewerMatrix[6] + -origin[2] * viewerMatrix[10];

    viewerMatrix[3] = 0.0;
    viewerMatrix[7] = 0.0;
    viewerMatrix[11] = 0.0;
    viewerMatrix[15] = 1.0;

    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    myGlMultMatrix(&viewerMatrix[0], &s_flipMatrix[0], &mut tr.ori.modelMatrix[0]);

    tr.viewParms.world = tr.ori;
}

unsafe fn SetFarClip() {
    let mut farthestCornerDistance: f32 = 0.0;
    let mut i: c_int;

    // if not rendering the world (icons, menus, etc)
    // set a 2k far clip plane
    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        if (tr.refdef.rdflags & RDF_AUTOMAP) != 0 {
            //override the zfar then
            tr.viewParms.zFar = 32768.0;
        } else {
            tr.viewParms.zFar = 2048.0;
        }
        return;
    }

    //
    // set far clipping planes dynamically
    //
    i = 0;
    while i < 8 {
        let mut v: [f32; 3] = [0.0; 3];
        let mut distance: f32;

        if (i & 1) != 0 {
            v[0] = tr.viewParms.visBounds[0][0];
        } else {
            v[0] = tr.viewParms.visBounds[1][0];
        }

        if (i & 2) != 0 {
            v[1] = tr.viewParms.visBounds[0][1];
        } else {
            v[1] = tr.viewParms.visBounds[1][1];
        }

        if (i & 4) != 0 {
            v[2] = tr.viewParms.visBounds[0][2];
        } else {
            v[2] = tr.viewParms.visBounds[1][2];
        }

        distance = DistanceSquared(&tr.viewParms.ori.origin, &v);

        if distance > farthestCornerDistance {
            farthestCornerDistance = distance;
        }
        i += 1;
    }
    // Bring in the zFar to the distanceCull distance
    // The sky renders at zFar so need to move it out a little
    // ...and make sure there is a minimum zfar to prevent problems
    tr.viewParms.zFar = Com_Clamp(2048.0, tr.distanceCull * 1.732, farthestCornerDistance.sqrt());

    /*
    if (r_shadows->integer == 2)
    { //volume caps need an "infinite" far clipping plane. So I'm using this semi-arbitrary massive number.
        tr.viewParms.zFar = 524288.0f;
    }
    */
}

pub unsafe fn R_SetupProjection() {
    let mut xmin: f32;
    let mut xmax: f32;
    let mut ymin: f32;
    let mut ymax: f32;
    let mut width: f32;
    let mut height: f32;
    let mut depth: f32;
    let mut zNear: f32;
    let mut zFar: f32;

    // dynamically compute far clip plane distance
    SetFarClip();

    //
    // set up projection matrix
    //
    zNear = (*r_znear).as_ref().map(|x| *x as f32).unwrap_or(0.0);
    zFar = tr.viewParms.zFar;

    ymax = zNear * (tr.refdef.fov_y * std::f32::consts::PI / 360.0).tan();
    ymin = -ymax;

    xmax = zNear * (tr.refdef.fov_x * std::f32::consts::PI / 360.0).tan();
    xmin = -xmax;

    width = xmax - xmin;
    height = ymax - ymin;
    depth = zFar - zNear;

    #[cfg(target_os = "xbox")]
    {
        tr.viewParms.projectionMatrix[0] = 2.0 * zNear / width;
        tr.viewParms.projectionMatrix[4] = 0.0;
        tr.viewParms.projectionMatrix[8] = (xmax + xmin) / width;	// normally 0
        tr.viewParms.projectionMatrix[12] = 0.0;

        tr.viewParms.projectionMatrix[1] = 0.0;
        tr.viewParms.projectionMatrix[5] = 2.0 * zNear / height;
        tr.viewParms.projectionMatrix[9] = (ymax + ymin) / height;	// normally 0
        tr.viewParms.projectionMatrix[13] = 0.0;

        tr.viewParms.projectionMatrix[2] = 0.0;
        tr.viewParms.projectionMatrix[6] = 0.0;
        tr.viewParms.projectionMatrix[10] = (zFar + zNear) / depth;
        tr.viewParms.projectionMatrix[14] = -2.0 * zFar * zNear / depth;

        tr.viewParms.projectionMatrix[3] = 0.0;
        tr.viewParms.projectionMatrix[7] = 0.0;
        tr.viewParms.projectionMatrix[11] = 1.0;
        tr.viewParms.projectionMatrix[15] = 0.0;
    }
    #[cfg(not(target_os = "xbox"))]
    {
        tr.viewParms.projectionMatrix[0] = 2.0 * zNear / width;
        tr.viewParms.projectionMatrix[4] = 0.0;
        tr.viewParms.projectionMatrix[8] = (xmax + xmin) / width;	// normally 0
        tr.viewParms.projectionMatrix[12] = 0.0;

        tr.viewParms.projectionMatrix[1] = 0.0;
        tr.viewParms.projectionMatrix[5] = 2.0 * zNear / height;
        tr.viewParms.projectionMatrix[9] = (ymax + ymin) / height;	// normally 0
        tr.viewParms.projectionMatrix[13] = 0.0;

        tr.viewParms.projectionMatrix[2] = 0.0;
        tr.viewParms.projectionMatrix[6] = 0.0;
        tr.viewParms.projectionMatrix[10] = -(zFar + zNear) / depth;
        tr.viewParms.projectionMatrix[14] = -2.0 * zFar * zNear / depth;

        tr.viewParms.projectionMatrix[3] = 0.0;
        tr.viewParms.projectionMatrix[7] = 0.0;
        tr.viewParms.projectionMatrix[11] = -1.0;
        tr.viewParms.projectionMatrix[15] = 0.0;
    }
}

pub unsafe fn R_SetupFrustum() {
    let mut i: c_int;
    let mut xs: f32;
    let mut xc: f32;
    let mut ang: f32;

    ang = tr.viewParms.fovX / 180.0 * std::f32::consts::PI * 0.5;
    xs = ang.sin();
    xc = ang.cos();

    VectorScale(&tr.viewParms.ori.axis[0], xs, &mut tr.viewParms.frustum[0].normal);
    VectorMA(&mut tr.viewParms.frustum[0].normal, xc, &tr.viewParms.ori.axis[1], &mut tr.viewParms.frustum[0].normal);

    VectorScale(&tr.viewParms.ori.axis[0], xs, &mut tr.viewParms.frustum[1].normal);
    VectorMA(&mut tr.viewParms.frustum[1].normal, -xc, &tr.viewParms.ori.axis[1], &mut tr.viewParms.frustum[1].normal);

    ang = tr.viewParms.fovY / 180.0 * std::f32::consts::PI * 0.5;
    xs = ang.sin();
    xc = ang.cos();

    VectorScale(&tr.viewParms.ori.axis[0], xs, &mut tr.viewParms.frustum[2].normal);
    VectorMA(&mut tr.viewParms.frustum[2].normal, xc, &tr.viewParms.ori.axis[2], &mut tr.viewParms.frustum[2].normal);

    VectorScale(&tr.viewParms.ori.axis[0], xs, &mut tr.viewParms.frustum[3].normal);
    VectorMA(&mut tr.viewParms.frustum[3].normal, -xc, &tr.viewParms.ori.axis[2], &mut tr.viewParms.frustum[3].normal);

    i = 0;
    while i < 4 {
        tr.viewParms.frustum[i as usize].type_ = PLANE_NON_AXIAL;
        tr.viewParms.frustum[i as usize].dist = DotProduct(&tr.viewParms.ori.origin, &tr.viewParms.frustum[i as usize].normal as *const _ as *const [f32; 3]);
        SetPlaneSignbits(&mut tr.viewParms.frustum[i as usize]);
        i += 1;
    }
}

pub unsafe fn R_MirrorPoint(in_: &[f32; 3], surface: *mut orientation_t, camera: *mut orientation_t, out: &mut [f32; 3]) {
    let mut i: c_int;
    let mut local: [f32; 3] = [0.0; 3];
    let mut transformed: [f32; 3] = [0.0; 3];
    let mut d: f32;

    VectorSubtract(in_, &(*surface).origin, &mut local);

    VectorClear(&mut transformed);
    i = 0;
    while i < 3 {
        d = DotProduct(&local, &(*surface).axis[i as usize] as *const _ as *const [f32; 3]);
        VectorMA(&mut transformed, d, &(*camera).axis[i as usize], &mut transformed);
        i += 1;
    }

    VectorAdd(&transformed, &(*camera).origin, out);
}

pub unsafe fn R_MirrorVector(in_: &[f32; 3], surface: *mut orientation_t, camera: *mut orientation_t, out: &mut [f32; 3]) {
    let mut i: c_int;
    let mut d: f32;

    VectorClear(out);
    i = 0;
    while i < 3 {
        d = DotProduct(in_, &(*surface).axis[i as usize] as *const _ as *const [f32; 3]);
        VectorMA(out, d, &(*camera).axis[i as usize], out);
        i += 1;
    }
}

pub unsafe fn R_PlaneForSurface(surfType: *mut surfaceType_t, plane: *mut cplane_t) {
    let mut tri: *mut srfTriangles_t;
    let mut poly: *mut srfPoly_t;
    let mut v1: *mut drawVert_t;
    let mut v2: *mut drawVert_t;
    let mut v3: *mut drawVert_t;
    let mut plane4: [f32; 4] = [0.0; 4];

    if surfType.is_null() {
        Com_Memset(plane as *mut core::ffi::c_void, 0, core::mem::size_of::<cplane_t>());
        (*plane).normal[0] = 1.0;
        return;
    }
    match *surfType as c_int {
    SF_FACE => {
        *plane = (*(surfType as *mut srfSurfaceFace_t)).plane;
        return;
    }
    SF_TRIANGLES => {
        tri = surfType as *mut srfTriangles_t;
        v1 = (*tri).verts.offset((*tri).indexes[0] as isize);
        v2 = (*tri).verts.offset((*tri).indexes[1] as isize);
        v3 = (*tri).verts.offset((*tri).indexes[2] as isize);
        PlaneFromPoints(&mut plane4[0], &(*v1).xyz, &(*v2).xyz, &(*v3).xyz);
        VectorCopy(&plane4 as *const _ as *const [f32; 3], &mut (*plane).normal);
        (*plane).dist = plane4[3];
        return;
    }
    SF_POLY => {
        poly = surfType as *mut srfPoly_t;
        PlaneFromPoints(&mut plane4[0], &(*poly).verts[0].xyz, &(*poly).verts[1].xyz, &(*poly).verts[2].xyz);
        VectorCopy(&plane4 as *const _ as *const [f32; 3], &mut (*plane).normal);
        (*plane).dist = plane4[3];
        return;
    }
    _ => {
        Com_Memset(plane as *mut core::ffi::c_void, 0, core::mem::size_of::<cplane_t>());
        (*plane).normal[0] = 1.0;
        return;
    }
    }
}

pub unsafe fn R_GetPortalOrientations(drawSurf: *mut drawSurf_t, entityNum: c_int,
                                    surface: *mut orientation_t, camera: *mut orientation_t,
                                    pvsOrigin: &mut [f32; 3], mirror: *mut qboolean) -> qboolean {
    let mut i: c_int;
    let mut originalPlane: cplane_t = core::mem::zeroed();
    let mut plane: cplane_t = core::mem::zeroed();
    let mut e: *mut trRefEntity_t;
    let mut d: f32;
    let mut transformed: [f32; 3] = [0.0; 3];

    // create plane axis for the portal we are seeing
    R_PlaneForSurface((*drawSurf).surface, &mut originalPlane);

    // rotate the plane if necessary
    if entityNum != TR_WORLDENT {
        tr.currentEntityNum = entityNum;
        tr.currentEntity = &mut tr.refdef.entities[entityNum as usize];

        // get the orientation of the entity
        R_RotateForEntity(tr.currentEntity, &tr.viewParms, &mut tr.ori);

        // rotate the plane, but keep the non-rotated version for matching
        // against the portalSurface entities
        R_LocalNormalToWorld(&originalPlane.normal, &mut plane.normal);
        plane.dist = originalPlane.dist + DotProduct(&plane.normal, &tr.ori.origin as *const _ as *const [f32; 3]);

        // translate the original plane
        originalPlane.dist = originalPlane.dist + DotProduct(&originalPlane.normal, &tr.ori.origin as *const _ as *const [f32; 3]);
    } else {
        plane = originalPlane;
    }

    VectorCopy(&plane.normal, &mut (*surface).axis[0]);
    PerpendicularVector(&mut (*surface).axis[1], &(*surface).axis[0]);
    CrossProduct(&(*surface).axis[0], &(*surface).axis[1], &mut (*surface).axis[2]);

    // locate the portal entity closest to this plane.
    // origin will be the origin of the portal, origin2 will be
    // the origin of the camera
    i = 0;
    while i < tr.refdef.num_entities {
        e = &mut tr.refdef.entities[i as usize];
        if (*e).e.reType != RT_PORTALSURFACE {
            i += 1;
            continue;
        }

        d = DotProduct(&(*e).e.origin, &originalPlane.normal as *const _ as *const [f32; 3]) - originalPlane.dist;
        if d > 64.0 || d < -64.0 {
            i += 1;
            continue;
        }

        // get the pvsOrigin from the entity
        VectorCopy(&(*e).e.oldorigin, pvsOrigin);

        // if the entity is just a mirror, don't use as a camera point
        if (*e).e.oldorigin[0] == (*e).e.origin[0] &&
            (*e).e.oldorigin[1] == (*e).e.origin[1] &&
            (*e).e.oldorigin[2] == (*e).e.origin[2] {
            VectorScale(&plane.normal, plane.dist, &mut (*surface).origin);
            VectorCopy(&(*surface).origin, &mut (*camera).origin);
            VectorSubtract(&vec3_origin, &(*surface).axis[0], &mut (*camera).axis[0]);
            VectorCopy(&(*surface).axis[1], &mut (*camera).axis[1]);
            VectorCopy(&(*surface).axis[2], &mut (*camera).axis[2]);

            *mirror = 1;
            return 1;
        }

        // project the origin onto the surface plane to get
        // an origin point we can rotate around
        d = DotProduct(&(*e).e.origin, &plane.normal as *const _ as *const [f32; 3]) - plane.dist;
        VectorMA(&(*e).e.origin, -d, &(*surface).axis[0], &mut (*surface).origin);

        // now get the camera origin and orientation
        VectorCopy(&(*e).e.oldorigin, &mut (*camera).origin);
        AxisCopy(&(*e).e.axis as *const _ as *const [[f32; 3]; 3], &mut (*camera).axis as *mut _ as *mut [[f32; 3]; 3]);
        VectorSubtract(&vec3_origin, &(*camera).axis[0], &mut (*camera).axis[0]);
        VectorSubtract(&vec3_origin, &(*camera).axis[1], &mut (*camera).axis[1]);

        // optionally rotate
        if (*e).e.oldframe != 0 {
            // if a speed is specified
            if (*e).e.frame != 0 {
                // continuous rotate
                d = (tr.refdef.time / 1000.0) * (*e).e.frame as f32;
                VectorCopy(&(*camera).axis[1], &mut transformed);
                RotatePointAroundVector(&mut (*camera).axis[1], &(*camera).axis[0], &transformed, d);
                CrossProduct(&(*camera).axis[0], &(*camera).axis[1], &mut (*camera).axis[2]);
            } else {
                // bobbing rotate, with skinNum being the rotation offset
                d = (tr.refdef.time * 0.003).sin();
                d = (*e).e.skinNum as f32 + d * 4.0;
                VectorCopy(&(*camera).axis[1], &mut transformed);
                RotatePointAroundVector(&mut (*camera).axis[1], &(*camera).axis[0], &transformed, d);
                CrossProduct(&(*camera).axis[0], &(*camera).axis[1], &mut (*camera).axis[2]);
            }
        } else if (*e).e.skinNum != 0 {
            d = (*e).e.skinNum as f32;
            VectorCopy(&(*camera).axis[1], &mut transformed);
            RotatePointAroundVector(&mut (*camera).axis[1], &(*camera).axis[0], &transformed, d);
            CrossProduct(&(*camera).axis[0], &(*camera).axis[1], &mut (*camera).axis[2]);
        }
        *mirror = 0;
        return 1;
        i += 1;
    }

    // if we didn't locate a portal entity, don't render anything.
    // We don't want to just treat it as a mirror, because without a
    // portal entity the server won't have communicated a proper entity set
    // in the snapshot

    // unfortunately, with local movement prediction it is easily possible
    // to see a surface before the server has communicated the matching
    // portal surface entity, so we don't want to print anything here...

    //Com_Printf ("Portal surface without a portal entity\n" );

    return 0;
}

unsafe fn IsMirror(drawSurf: *const drawSurf_t, entityNum: c_int) -> qboolean {
    let mut i: c_int;
    let mut originalPlane: cplane_t = core::mem::zeroed();
    let mut plane: cplane_t = core::mem::zeroed();
    let mut e: *mut trRefEntity_t;
    let mut d: f32;

    // create plane axis for the portal we are seeing
    R_PlaneForSurface((*drawSurf).surface, &mut originalPlane);

    // rotate the plane if necessary
    if entityNum != TR_WORLDENT {
        tr.currentEntityNum = entityNum;
        tr.currentEntity = &mut tr.refdef.entities[entityNum as usize];

        // get the orientation of the entity
        R_RotateForEntity(tr.currentEntity, &tr.viewParms, &mut tr.ori);

        // rotate the plane, but keep the non-rotated version for matching
        // against the portalSurface entities
        R_LocalNormalToWorld(&originalPlane.normal, &mut plane.normal);
        plane.dist = originalPlane.dist + DotProduct(&plane.normal, &tr.ori.origin as *const _ as *const [f32; 3]);

        // translate the original plane
        originalPlane.dist = originalPlane.dist + DotProduct(&originalPlane.normal, &tr.ori.origin as *const _ as *const [f32; 3]);
    } else {
        plane = originalPlane;
    }

    // locate the portal entity closest to this plane.
    // origin will be the origin of the portal, origin2 will be
    // the origin of the camera
    i = 0;
    while i < tr.refdef.num_entities {
        e = &mut tr.refdef.entities[i as usize];
        if (*e).e.reType != RT_PORTALSURFACE {
            i += 1;
            continue;
        }

        d = DotProduct(&(*e).e.origin, &originalPlane.normal as *const _ as *const [f32; 3]) - originalPlane.dist;
        if d > 64.0 || d < -64.0 {
            i += 1;
            continue;
        }

        // if the entity is just a mirror, don't use as a camera point
        if (*e).e.oldorigin[0] == (*e).e.origin[0] &&
            (*e).e.oldorigin[1] == (*e).e.origin[1] &&
            (*e).e.oldorigin[2] == (*e).e.origin[2] {
            return 1;
        }

        return 0;
        i += 1;
    }
    return 0;
}

#[cfg(not(feature = "dedicated"))]
unsafe fn SurfIsOffscreen(drawSurf: *const drawSurf_t, clipDest: &mut [[f32; 4]; 128]) -> qboolean {
    let mut shortest: f32 = 100000000.0;
    let mut entityNum: c_int;
    let mut numTriangles: c_int;
    let mut shader: *mut shader_t;
    let mut fogNum: c_int;
    let mut dlighted: c_int;
    let mut clip: [f32; 4] = [0.0; 4];
    let mut eye: [f32; 4] = [0.0; 4];
    let mut i: c_int;
    let mut pointOr: c_int = 0;
    let mut pointAnd: c_int = !0;

    R_RotateForViewer();

    R_DecomposeSort((*drawSurf).sort, &mut entityNum, &mut shader, &mut fogNum, &mut dlighted);
    RB_BeginSurface(shader, fogNum);
    (rb_surfaceTable[*(*drawSurf).surface as usize])((*drawSurf).surface);
    assert!(tess.numVertexes < 128);
    i = 0;
    while i < tess.numVertexes {
        let mut j: c_int;
        let mut pointFlags: c_int = 0;

        R_TransformModelToClip(&tess.xyz[i as usize], &tr.ori.modelMatrix[0], &tr.viewParms.projectionMatrix[0], &mut eye, &mut clip);

        j = 0;
        while j < 3 {
            if clip[j as usize] >= clip[3] {
                pointFlags |= (1 << (j * 2));
            } else if clip[j as usize] <= -clip[3] {
                pointFlags |= (1 << (j * 2 + 1));
            }
            j += 1;
        }
        pointAnd &= pointFlags;
        pointOr |= pointFlags;
        i += 1;
    }

    // trivially reject
    if pointAnd != 0 {
        return 1;
    }

    // determine if this surface is backfaced and also determine the distance
    // to the nearest vertex so we can cull based on portal range.  Culling
    // based on vertex distance isn't 100% correct (we should be checking for
    // range to the surface), but it's good enough for the types of portals
    // we have in the game right now.
    numTriangles = tess.numIndexes / 3;

    i = 0;
    while i < tess.numIndexes {
        let mut normal: [f32; 3] = [0.0; 3];
        let mut dot: f32;
        let mut len: f32;

        VectorSubtract(&tess.xyz[tess.indexes[i as usize] as usize], &tr.viewParms.ori.origin, &mut normal);

        len = VectorLengthSquared(&normal);			// lose the sqrt
        if len < shortest {
            shortest = len;
        }

        if (dot = DotProduct(&normal, &tess.normal[tess.indexes[i as usize] as usize] as *const _ as *const [f32; 3])) >= 0.0 {
            numTriangles -= 1;
        }
        i += 3;
    }
    if numTriangles == 0 {
        return 1;
    }

    // mirrors can early out at this point, since we don't do a fade over distance
    // with them (although we could)
    if IsMirror(drawSurf, entityNum) != 0 {
        return 0;
    }

    if shortest > (tess.shader.portalRange * tess.shader.portalRange) {
        return 1;
    }

    return 0;
}

pub unsafe fn R_MirrorViewBySurface(drawSurf: *mut drawSurf_t, entityNum: c_int) -> qboolean {
    let mut clipDest: [[f32; 4]; 128] = [[0.0; 4]; 128];
    let mut newParms: viewParms_t = core::mem::zeroed();
    let mut oldParms: viewParms_t = core::mem::zeroed();
    let mut surface: orientation_t = core::mem::zeroed();
    let mut camera: orientation_t = core::mem::zeroed();

    // don't recursively mirror
    if tr.viewParms.isPortal != 0 {
        Com_DPrintf(S_COLOR_RED as *const _ as *const c_int);
        return 0;
    }

    if (*r_noportals).as_ref().map(|x| *x != 0).unwrap_or(false) || (*r_fastsky).as_ref().map(|x| *x == 1).unwrap_or(false) {
        return 0;
    }

    // trivially reject portal/mirror
    if SurfIsOffscreen(drawSurf, &mut clipDest) != 0 {
        return 0;
    }

    // save old viewParms so we can return to it after the mirror view
    oldParms = tr.viewParms;

    newParms = tr.viewParms;
    newParms.isPortal = 1;
    if R_GetPortalOrientations(drawSurf, entityNum, &mut surface, &mut camera,
        &mut newParms.pvsOrigin, &mut newParms.isMirror) == 0 {
        return 0;		// bad portal, no portalentity
    }

    R_MirrorPoint(&oldParms.ori.origin, &mut surface as *mut _, &mut camera as *mut _, &mut newParms.ori.origin);

    VectorSubtract(&vec3_origin, &camera.axis[0], &mut newParms.portalPlane.normal);
    newParms.portalPlane.dist = DotProduct(&camera.origin, &newParms.portalPlane.normal as *const _ as *const [f32; 3]);

    R_MirrorVector(&oldParms.ori.axis[0], &mut surface as *mut _, &mut camera as *mut _, &mut newParms.ori.axis[0]);
    R_MirrorVector(&oldParms.ori.axis[1], &mut surface as *mut _, &mut camera as *mut _, &mut newParms.ori.axis[1]);
    R_MirrorVector(&oldParms.ori.axis[2], &mut surface as *mut _, &mut camera as *mut _, &mut newParms.ori.axis[2]);

    // OPTIMIZE: restrict the viewport on the mirrored view

    // render the mirror view
    R_RenderView(&mut newParms);

    tr.viewParms = oldParms;

    return 1;
}

pub unsafe fn R_SpriteFogNum(ent: *mut trRefEntity_t) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut fog: *mut fog_t;

    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return 0;
    }

    i = 1;
    while i < tr.world.numfogs {
        fog = &mut tr.world.fogs[i as usize];
        j = 0;
        while j < 3 {
            if (*ent).e.origin[j as usize] - (*ent).e.radius >= (*fog).bounds[1][j as usize] {
                break;
            }
            if (*ent).e.origin[j as usize] + (*ent).e.radius <= (*fog).bounds[0][j as usize] {
                break;
            }
            j += 1;
        }
        if j == 3 {
            return i;
        }
        i += 1;
    }

    return 0;
}

//==========================================================================================
//
// DRAWSURF SORTING
//
//==========================================================================================

// macro replacement using inline function
#[inline]
unsafe fn swap_draw_surf(a: *mut drawSurf_t, b: *mut drawSurf_t) {
    let temp_0 = (*(a as *mut [c_int; 2]))[0];
    (*(a as *mut [c_int; 2]))[0] = (*(b as *mut [c_int; 2]))[0];
    (*(b as *mut [c_int; 2]))[0] = temp_0;
    let temp_1 = (*(a as *mut [c_int; 2]))[1];
    (*(a as *mut [c_int; 2]))[1] = (*(b as *mut [c_int; 2]))[1];
    (*(b as *mut [c_int; 2]))[1] = temp_1;
}

// this parameter defines the cutoff between using quick sort and
// insertion sort for arrays; arrays with lengths shorter or equal to the
// below value use insertion sort

unsafe fn shortsort(lo: *mut drawSurf_t, hi: *mut drawSurf_t) {
    let mut p: *mut drawSurf_t;
    let mut max: *mut drawSurf_t;

    while hi > lo {
        max = lo;
        p = lo.offset(1);
        while p <= hi {
            if (*p).sort > (*max).sort {
                max = p;
            }
            p = p.offset(1);
        }
        swap_draw_surf(max, hi);
        hi = hi.offset(-1);
    }
}

// sort the array between lo and hi (inclusive)
// FIXME: this was lifted and modified from the microsoft lib source...

pub unsafe fn qsortFast(
    base: *mut core::ffi::c_void,
    num: c_int,
    width: c_int
    )
{
    let mut lo: *mut c_int;
    let mut hi: *mut c_int;		      // ends of sub-array currently sorting
    let mut mid: *mut c_int;                  // points to middle of subarray
    let mut loguy: *mut c_int;
    let mut higuy: *mut c_int;        // traveling pointers for partition step
    let mut size: c_int;              // size of the sub-array
    let mut lostk: [*mut c_int; 30] = [core::ptr::null_mut(); 30];
    let mut histk: [*mut c_int; 30] = [core::ptr::null_mut(); 30];
    let mut stkptr: c_int;                 // stack for saving sub-array to be processed
    let mut temp: c_int;

    if core::mem::size_of::<drawSurf_t>() != 8 {
        Com_Error(ERR_DROP, b"change SWAP_DRAW_SURF macro\0" as *const _ as *const c_int);
    }

    // Note: the number of stack entries required is no more than
    // 1 + log2(size), so 30 is sufficient for any array

    if num < 2 || width == 0 {
        return;                 // nothing to do
    }

    stkptr = 0;                 // initialize stack

    lo = base as *mut c_int;
    hi = (base as *mut c_int).offset((width as isize) * ((num-1) as isize) / 4);        // initialize limits

    // this entry point is for pseudo-recursion calling: setting
    // lo and hi and jumping to here is like recursion, but stkptr is
    // preserved, locals aren't, so we preserve stuff on the stack
    loop {
        size = ((hi as isize - lo as isize) / (width as isize) + 1) as c_int;        // number of el's to sort

        // below a certain size, it is faster to use a O(n^2) sorting method
        if (size as usize) <= CUTOFF {
            shortsort(lo as *mut drawSurf_t, hi as *mut drawSurf_t);
        } else {
            // First we pick a partititioning element.  The efficiency of the
            // algorithm demands that we find one that is approximately the
            // median of the values, but also that we select one fast.  Using
            // the first one produces bad performace if the array is already
            // sorted, so we use the middle one, which would require a very
            // wierdly arranged array for worst case performance.  Testing shows
            // that a median-of-three algorithm does not, in general, increase
            // performance.

            mid = lo.offset(((size / 2) as isize) * (width as isize / 4));      // find middle element
            swap_draw_surf(mid as *mut drawSurf_t, lo as *mut drawSurf_t);               // swap it to beginning of array

            // We now wish to partition the array into three pieces, one
            // consisiting of elements <= partition element, one of elements
            // equal to the parition element, and one of element >= to it.  This
            // is done below; comments indicate conditions established at every
            // step.

            loguy = lo;
            higuy = hi.offset((width as isize) / 4);

            // Note that higuy decreases and loguy increases on every iteration,
            // so loop must terminate.
            loop {
                // lo <= loguy < hi, lo < higuy <= hi + 1,
                // A[i] <= A[lo] for lo <= i <= loguy,
                // A[i] >= A[lo] for higuy <= i <= hi

                loop {
                    loguy = loguy.offset((width as isize) / 4);
                    if !(loguy <= hi &&
                        (*(loguy as *const drawSurf_t)).sort <= (*(lo as *const drawSurf_t)).sort) {
                        break;
                    }
                }

                // lo < loguy <= hi+1, A[i] <= A[lo] for lo <= i < loguy,
                // either loguy > hi or A[loguy] > A[lo]

                loop {
                    higuy = higuy.offset(-(width as isize) / 4);
                    if !(higuy > lo &&
                        (*(higuy as *const drawSurf_t)).sort >= (*(lo as *const drawSurf_t)).sort) {
                        break;
                    }
                }

                // lo-1 <= higuy <= hi, A[i] >= A[lo] for higuy < i <= hi,
                // either higuy <= lo or A[higuy] < A[lo]

                if higuy < loguy {
                    break;
                }

                // if loguy > hi or higuy <= lo, then we would have exited, so
                // A[loguy] > A[lo], A[higuy] < A[lo],
                // loguy < hi, highy > lo

                swap_draw_surf(loguy as *mut drawSurf_t, higuy as *mut drawSurf_t);

                // A[loguy] < A[lo], A[higuy] > A[lo]; so condition at top
                // of loop is re-established
            }

            //     A[i] >= A[lo] for higuy < i <= hi,
            //     A[i] <= A[lo] for lo <= i < loguy,
            //     higuy < loguy, lo <= higuy <= hi
            // implying:
            //     A[i] >= A[lo] for loguy <= i <= hi,
            //     A[i] <= A[lo] for lo <= i <= higuy,
            //     A[i] = A[lo] for higuy < i < loguy

            swap_draw_surf(lo as *mut drawSurf_t, higuy as *mut drawSurf_t);     // put partition element in place

            // OK, now we have the following:
            //       A[i] >= A[higuy] for loguy <= i <= hi,
            //       A[i] <= A[higuy] for lo <= i < higuy
            //       A[i] = A[lo] for higuy <= i < loguy

            // We've finished the partition, now we want to sort the subarrays
            // [lo, higuy-1] and [loguy, hi].
            // We do the smaller one first to minimize stack usage.
            // We only sort arrays of length 2 or more.

            if higuy as isize - (width as isize) / 4 - lo as isize >= hi as isize - loguy as isize {
                if lo.offset((width as isize) / 4) < higuy {
                    lostk[stkptr as usize] = lo;
                    histk[stkptr as usize] = higuy.offset(-(width as isize) / 4);
                    stkptr += 1;
                }                           // save big recursion for later

                if loguy < hi {
                    lo = loguy;
                    continue;           // do small recursion
                }
            } else {
                if loguy < hi {
                    lostk[stkptr as usize] = loguy;
                    histk[stkptr as usize] = hi;
                    stkptr += 1;               // save big recursion for later
                }

                if lo.offset((width as isize) / 4) < higuy {
                    hi = higuy.offset(-(width as isize) / 4);
                    continue;           // do small recursion
                }
            }
        }

        // We have sorted the array, except for any pending sorts on the stack.
        // Check if there are any, and do them.

        stkptr -= 1;
        if stkptr >= 0 {
            lo = lostk[stkptr as usize];
            hi = histk[stkptr as usize];
            continue;           // pop subarray from stack
        } else {
            break;                 // all subarrays done
        }
    }
}

pub unsafe fn R_AddDrawSurf(surface: *mut surfaceType_t, shader: *mut shader_t,
                           fogIndex: c_int, dlightMap: c_int) {
    let mut index: c_int;

    if (tr.refdef.rdflags & RDF_NOFOG) != 0 {
        // fogIndex = 0;  // assignment to itself, so just drop it
    }

    // instead of checking for overflow, we just mask the index
    // so it wraps around
    index = tr.refdef.numDrawSurfs & DRAWSURF_MASK;
    // the sort data is packed into a single 32 bit value so it can be
    // compared quickly during the qsorting process
    tr.refdef.drawSurfs[index as usize].sort = ((*shader).sortedIndex << QSORT_SHADERNUM_SHIFT)
        | tr.shiftedEntityNum | (fogIndex << QSORT_FOGNUM_SHIFT) | dlightMap;
    tr.refdef.drawSurfs[index as usize].surface = surface;
    tr.refdef.numDrawSurfs += 1;
}

pub unsafe fn R_DecomposeSort(sort: c_int, entityNum: *mut c_int, shader: *mut *mut shader_t,
                              fogNum: *mut c_int, dlightMap: *mut c_int) {
    *fogNum = (sort >> QSORT_FOGNUM_SHIFT) & 31;
    *shader = tr.sortedShaders[((sort >> QSORT_SHADERNUM_SHIFT) & (MAX_SHADERS-1)) as usize];
    *entityNum = (sort >> QSORT_ENTITYNUM_SHIFT) & (MAX_ENTITIES-1);
    *dlightMap = sort & 3;
}

pub unsafe fn R_SortDrawSurfs(drawSurfs: *mut drawSurf_t, numDrawSurfs: c_int) {
    let mut shader: *mut shader_t;
    let mut fogNum: c_int;
    let mut entityNum: c_int;
    let mut dlighted: c_int;
    let mut i: c_int;

    // it is possible for some views to not have any surfaces
    if numDrawSurfs < 1 {
        // we still need to add it for hyperspace cases
        R_AddDrawSurfCmd(drawSurfs, numDrawSurfs);
        return;
    }

    // if we overflowed MAX_DRAWSURFS, the drawsurfs
    // wrapped around in the buffer and we will be missing
    // the first surfaces, not the last ones
    if numDrawSurfs > MAX_DRAWSURFS {
        // numDrawSurfs = MAX_DRAWSURFS;
        #[cfg(all(debug_assertions, target_os = "xbox"))]
        {
            Com_Printf(b"\x031Draw surface overflow!  Tell Brian.\n\0" as *const _ as *const c_int);
        }
    }

    #[cfg(not(target_os = "xbox"))]
    {
        // sort the drawsurfs by sort type, then orientation, then shader
        qsortFast(drawSurfs as *mut core::ffi::c_void, numDrawSurfs, core::mem::size_of::<drawSurf_t>() as c_int);
    }

    // check for any pass through drawing, which
    // may cause another view to be rendered first
    i = 0;
    while i < numDrawSurfs {
        R_DecomposeSort((*drawSurfs.offset(i as isize)).sort, &mut entityNum, &mut shader, &mut fogNum, &mut dlighted);

        if (*shader).sort > SS_PORTAL {
            break;
        }

        // no shader should ever have this sort type
        if (*shader).sort == SS_BAD {
            Com_Error(ERR_DROP, b"Shader '%s'with sort == SS_BAD\0" as *const _ as *const c_int);
        }

        // if the mirror was completely clipped away, we may need to check another surface
        if R_MirrorViewBySurface(drawSurfs.offset(i as isize), entityNum) != 0 {
            // this is a debug option to see exactly what is being mirrored
            if (*r_portalOnly).as_ref().map(|x| *x != 0).unwrap_or(false) {
                return;
            }
            break;		// only one mirror view at a time
        }
        i += 1;
    }

    #[cfg(target_os = "xbox")]
    {
        qsortFast(drawSurfs as *mut core::ffi::c_void, numDrawSurfs, core::mem::size_of::<drawSurf_t>() as c_int);
    }

    R_AddDrawSurfCmd(drawSurfs, numDrawSurfs);
}

pub unsafe fn R_AddEntitySurfaces() {
    let mut ent: *mut trRefEntity_t;
    let mut shader: *mut shader_t;

    if (*r_drawentities).as_ref().map(|x| *x == 0).unwrap_or(true) {
        return;
    }

    tr.currentEntityNum = 0;
    while tr.currentEntityNum < tr.refdef.num_entities {
        ent = &mut tr.refdef.entities[tr.currentEntityNum as usize];
        tr.currentEntity = ent;

        assert!((*ent).e.renderfx >= 0);

        (*ent).needDlights = 0;

        // preshift the value we are going to OR into the drawsurf sort
        tr.shiftedEntityNum = tr.currentEntityNum << QSORT_ENTITYNUM_SHIFT;

        //
        // the weapon model must be handled special --
        // we don't want the hacked weapon position showing in
        // mirrors, because the true body position will already be drawn
        //
        if (((*ent).e.renderfx & RF_FIRST_PERSON) != 0) && (tr.viewParms.isPortal != 0) {
            tr.currentEntityNum += 1;
            continue;
        }

        // simple generated models, like sprites and beams, are not culled
        match (*ent).e.reType {
        RT_PORTALSURFACE => {
            // don't draw anything
        }
        RT_SPRITE | RT_BEAM | RT_ORIENTED_QUAD | RT_ELECTRICITY | RT_LINE | RT_ORIENTEDLINE | RT_CYLINDER | RT_SABER_GLOW => {
            // self blood sprites, talk balloons, etc should not be drawn in the primary
            // view.  We can't just do this check for all entities, because md3
            // entities may still want to cast shadows from them
            if (((*ent).e.renderfx & RF_THIRD_PERSON) != 0) && (tr.viewParms.isPortal == 0) {
                tr.currentEntityNum += 1;
                continue;
            }
            shader = R_GetShaderByHandle((*ent).e.customShader);
            R_AddDrawSurf(&mut entitySurface as *mut _, shader, R_SpriteFogNum(ent), 0);
        }

        RT_MODEL => {
            // we must set up parts of tr.ori for model culling
            R_RotateForEntity(ent, &tr.viewParms, &mut tr.ori);

            tr.currentModel = R_GetModelByHandle((*ent).e.hModel);
            if tr.currentModel.is_null() {
                R_AddDrawSurf(&mut entitySurface as *mut _, tr.defaultShader, 0, 0);
            } else {
                match (*tr.currentModel).type_ {
                MOD_MESH => {
                    R_AddMD3Surfaces(ent);
                }
                MOD_BRUSH => {
                    R_AddBrushModelSurfaces(ent);
                }
                // Ghoul2 Insert Start
                MOD_MDXM => {
                    //g2r
                    if !((*ent).e.ghoul2).is_null() {
                        R_AddGhoulSurfaces(ent);
                    }
                }
                MOD_BAD => {		// null model axis
                    if (((*ent).e.renderfx & RF_THIRD_PERSON) != 0) && (tr.viewParms.isPortal == 0) {
                        if ((*ent).e.renderfx & RF_SHADOW_ONLY) == 0 {
                            tr.currentEntityNum += 1;
                            continue;
                        }
                    }

                    if !((*ent).e.ghoul2).is_null() && G2API_HaveWeGhoul2Models((*ent).e.ghoul2) {
                        R_AddGhoulSurfaces(ent);
                    } else {
                        R_AddDrawSurf(&mut entitySurface as *mut _, tr.defaultShader, 0, 0);
                    }
                }
                // Ghoul2 Insert End
                _ => {
                    Com_Error(ERR_DROP, b"R_AddEntitySurfaces: Bad modeltype\0" as *const _ as *const c_int);
                }
                }
            }
        }

        RT_ENT_CHAIN => {
            shader = R_GetShaderByHandle((*ent).e.customShader);
            R_AddDrawSurf(&mut entitySurface as *mut _, shader, R_SpriteFogNum(ent), 0);
        }

        _ => {
            Com_Error(ERR_DROP, b"R_AddEntitySurfaces: Bad reType\0" as *const _ as *const c_int);
        }
        }

        tr.currentEntityNum += 1;
    }
}

#[cfg(target_os = "xbox")]
pub unsafe fn R_GenerateDrawSurfs(isPortal: bool) {
    // determine which leaves are in the PVS / areamask
    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) == 0 {
        R_MarkLeaves(core::ptr::null_mut());
    }

    R_AddWorldSurfaces();

    R_AddPolygonSurfaces();

    R_AddTerrainSurfaces();

    // set the projection matrix with the minimum zfar
    // now that we have the world bounded
    // this needs to be done before entities are
    // added, because they use the projection
    // matrix for lod calculation
    R_SetupProjection();

    R_AddEntitySurfaces();
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn R_GenerateDrawSurfs() {
    R_AddWorldSurfaces();

    R_AddPolygonSurfaces();

    R_AddTerrainSurfaces(); //rwwRMG - added

    // set the projection matrix with the minimum zfar
    // now that we have the world bounded
    // this needs to be done before entities are
    // added, because they use the projection
    // matrix for lod calculation
    R_SetupProjection();

    R_AddEntitySurfaces();
}

pub unsafe fn R_DebugPolygon(color: c_int, numPoints: c_int, points: *mut f32) {
    let mut i: c_int;

    GL_State(GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE);

    // draw solid shade

    qglColor3f((color & 1) as f32, ((color >> 1) & 1) as f32, ((color >> 2) & 1) as f32);
    qglBegin(GL_POLYGON);
    i = 0;
    while i < numPoints {
        qglVertex3fv(points.offset((i * 3) as isize));
        i += 1;
    }
    qglEnd();

    // draw wireframe outline
    GL_State(GLS_POLYMODE_LINE | GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE);
    qglDepthRange(0.0, 0.0);
    qglColor3f(1.0, 1.0, 1.0);
    qglBegin(GL_POLYGON);
    i = 0;
    while i < numPoints {
        qglVertex3fv(points.offset((i * 3) as isize));
        i += 1;
    }
    qglEnd();
    qglDepthRange(0.0, 1.0);
}

pub unsafe fn R_DebugGraphics() {
    if (*r_debugSurface).as_ref().map(|x| *x == 0).unwrap_or(true) {
        return;
    }

    // the render thread can't make callbacks to the main thread
    R_SyncRenderThread();

    GL_Bind(tr.whiteImage as *mut core::ffi::c_void);
    GL_Cull(CT_FRONT_SIDED);
    CM_DrawDebugSurface(R_DebugPolygon);
}

pub unsafe fn R_RenderView(parms: *mut viewParms_t) {
    let mut firstDrawSurf: c_int;

    if (*parms).viewportWidth <= 0 || (*parms).viewportHeight <= 0 {
        return;
    }

    tr.viewCount += 1;

    tr.viewParms = *parms;
    tr.viewParms.frameSceneNum = tr.frameSceneNum;
    tr.viewParms.frameCount = tr.frameCount;

    firstDrawSurf = tr.refdef.numDrawSurfs;

    tr.viewCount += 1;

    // set viewParms.world
    R_RotateForViewer();

    R_SetupFrustum();

    #[cfg(target_os = "xbox")]
    {
        R_GenerateDrawSurfs((*parms).isPortal != 0);
    }
    #[cfg(not(target_os = "xbox"))]
    {
        R_GenerateDrawSurfs();
    }

    R_SortDrawSurfs(tr.refdef.drawSurfs.offset(firstDrawSurf as isize), tr.refdef.numDrawSurfs - firstDrawSurf);

    // draw main system development information (surface outlines, etc)
    R_DebugGraphics();
}
