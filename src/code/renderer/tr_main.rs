// tr_main.c -- main control flow for each frame

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_void};
use core::mem::size_of;

// tr_local.h declarations (partial stubs as needed for structural coherence)
// In a full port, these would be in a separate tr_local_h module

// Vector and matrix types (typically f32 arrays)
type vec3_t = [f32; 3];
type vec4_t = [f32; 4];

#[repr(C)]
pub struct trGlobals_t {
    // Stub - full definition in tr_local.h port
    pub or: orientationr_t,
    pub viewParms: viewParms_t,
    pub viewCount: c_int,
    pub frameCount: c_int,
    pub frameSceneNum: c_int,
    pub refdef: refdef_t,
    pub currentEntityNum: c_int,
    pub currentEntity: *mut trRefEntity_t,
    pub currentModel: *mut model_t,
    pub defaultShader: *mut shader_t,
    pub shiftedEntityNum: c_int,
    pub world: *mut world_t,
    pub distanceCull: f32,
    pub sortedShaders: [*mut shader_t; 4096], // MAX_SHADERS
}

#[repr(C)]
pub struct orientationr_t {
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
    pub viewOrigin: vec3_t,
    pub modelMatrix: [f32; 16],
}

#[repr(C)]
pub struct viewParms_t {
    pub or: orientationr_t,
    pub world: orientationr_t,
    pub fovX: f32,
    pub fovY: f32,
    pub viewportWidth: c_int,
    pub viewportHeight: c_int,
    pub projectionMatrix: [f32; 16],
    pub frustum: [cplane_t; 5],
    pub visBounds: [vec3_t; 2],
    pub zFar: f32,
    pub isPortal: c_int,
    pub isMirror: c_int,
    pub pvsOrigin: vec3_t,
    pub portalPlane: cplane_t,
    pub frameSceneNum: c_int,
    pub frameCount: c_int,
}

#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub r#type: c_int,
    pub signbits: c_int,
    pub pad: [u8; 4],
}

#[repr(C)]
pub struct refdef_t {
    pub time: c_int,
    pub rdflags: c_int,
    pub vieworg: vec3_t,
    pub viewaxis: [vec3_t; 3],
    pub fov_x: f32,
    pub fov_y: f32,
    pub drawSurfs: *mut drawSurf_t,
    pub numDrawSurfs: c_int,
    pub entities: *mut trRefEntity_t,
    pub num_entities: c_int,
    pub dlights: *mut dlight_t,
    pub numDlights: c_int,
    pub fogIndex: c_int,
}

#[repr(C)]
pub struct drawSurf_t {
    pub sort: u32,
    pub surface: *mut surfaceType_t,
}

#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,
    pub axisLength: f32,
    pub needDlights: c_int,
}

#[repr(C)]
pub struct refEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
    pub hModel: c_int,
    pub lightingOrigin: vec3_t,
    pub shadowPlane: f32,
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: c_int,
    pub frame: c_int,
    pub oldframe: c_int,
    pub backlerp: f32,
    pub skinNum: c_int,
    pub customShader: c_int,
    pub ghoul2: *mut c_void,
    pub radius: f32,
    pub oldorigin: vec3_t,
}

#[repr(C)]
pub struct shader_t {
    pub name: [c_int; 64],
    pub sortedIndex: c_int,
    pub sort: c_int,
    pub surfaceFlags: c_int,
    pub portalRange: f32,
}

#[repr(C)]
pub struct surfaceType_t {
    pub dummy: c_int,
}

#[repr(C)]
pub struct srfTriangles_t {
    pub verts: *mut drawVert_t,
    pub indexes: *mut c_int,
}

#[repr(C)]
pub struct drawVert_t {
    pub xyz: vec3_t,
}

#[repr(C)]
pub struct srfGridMesh_t {
    pub verts: *mut drawVert_t,
}

#[repr(C)]
pub struct srfPoly_t {
    pub verts: *mut drawVert_t,
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    pub plane: cplane_t,
}

#[repr(C)]
pub struct orientation_t {
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
}

#[repr(C)]
pub struct model_t {
    pub r#type: c_int,
}

#[repr(C)]
pub struct world_t {
    pub numfogs: c_int,
    pub fogs: *mut fog_t,
}

#[repr(C)]
pub struct fog_t {
    pub bounds: [vec3_t; 2],
    pub parms: fogParms_t,
}

#[repr(C)]
pub struct fogParms_t {
    pub color: [f32; 3],
}

#[repr(C)]
pub struct dlight_t {
    pub dummy: c_int,
}

// Opaque types used but not fully defined here
pub struct color4ub_t {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
pub struct tess_t {
    pub numVertexes: c_int,
    pub xyz: [[f32; 3]; 4000],
    pub normal: [[f32; 3]; 4000],
    pub indexes: [c_int; 6000],
    pub numIndexes: c_int,
    pub shader: *mut shader_t,
}

// Constants
const CULL_IN: c_int = 0;
const CULL_CLIP: c_int = 1;
const CULL_OUT: c_int = 2;

const SF_ENTITY: c_int = 0;
const SF_FACE: c_int = 1;
const SF_TRIANGLES: c_int = 2;
const SF_POLY: c_int = 3;
const SF_GRID: c_int = 4;

const RT_MODEL: c_int = 0;
const RT_PORTALSURFACE: c_int = 1;
const RT_SPRITE: c_int = 2;
const RT_ORIENTED_QUAD: c_int = 3;
const RT_BEAM: c_int = 4;
const RT_CYLINDER: c_int = 5;
const RT_LATHE: c_int = 6;
const RT_CLOUDS: c_int = 7;
const RT_LINE: c_int = 8;
const RT_ELECTRICITY: c_int = 9;
const RT_SABER_GLOW: c_int = 10;

const MOD_MESH: c_int = 0;
const MOD_BRUSH: c_int = 1;
const MOD_MDXM: c_int = 2;
const MOD_BAD: c_int = 3;

const TR_WORLDENT: c_int = 0;

const PLANE_NON_AXIAL: c_int = 4;

const M_PI: f32 = 3.14159265358979323846;

const RF_ALPHA_FADE: c_int = 0x01;
const RF_FIRST_PERSON: c_int = 0x02;
const RF_THIRD_PERSON: c_int = 0x04;
const RF_SHADOW_ONLY: c_int = 0x08;

const RDF_NOWORLDMODEL: c_int = 0x01;
const RDF_doLAGoggles: c_int = 0x02;
const RDF_ForceSightOn: c_int = 0x04;

const SURF_FORCESIGHT: c_int = 0x01;

const SS_BAD: c_int = 0;
const SS_PORTAL: c_int = 1;

const QSORT_SHADERNUM_SHIFT: c_int = 10;
const QSORT_FOGNUM_SHIFT: c_int = 5;
const QSORT_ENTITYNUM_SHIFT: c_int = 15;

const DRAWSURF_MASK: c_int = 0xffff;
const MAX_SHADERS: c_int = 4096;
const MAX_ENTITIES: c_int = 2048;
const MAX_DRAWSURFS: c_int = 65536;

const MAX_LIGHT_STYLES: c_int = 64;

const CONTENTS_FOG: c_int = 0x10000;

const GLS_DEPTHMASK_TRUE: c_int = 0x01;
const GLS_SRCBLEND_ONE: c_int = 0x02;
const GLS_DSTBLEND_ONE: c_int = 0x04;
const GLS_POLYMODE_LINE: c_int = 0x08;

const CT_FRONT_SIDED: c_int = 0;

const GL_POLYGON: c_int = 9;

const ERR_DROP: c_int = 1;

const PRINT_DEVELOPER: c_int = 1;
const PRINT_ALL: c_int = 0;

const S_COLOR_RED: &str = "^1";

const CUTOFF: usize = 8;

pub static mut tr: trGlobals_t = trGlobals_t {
    or: orientationr_t {
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
        viewOrigin: [0.0; 3],
        modelMatrix: [0.0; 16],
    },
    viewParms: viewParms_t {
        or: orientationr_t {
            origin: [0.0; 3],
            axis: [[0.0; 3]; 3],
            viewOrigin: [0.0; 3],
            modelMatrix: [0.0; 16],
        },
        world: orientationr_t {
            origin: [0.0; 3],
            axis: [[0.0; 3]; 3],
            viewOrigin: [0.0; 3],
            modelMatrix: [0.0; 16],
        },
        fovX: 0.0,
        fovY: 0.0,
        viewportWidth: 0,
        viewportHeight: 0,
        projectionMatrix: [0.0; 16],
        frustum: [cplane_t {
            normal: [0.0; 3],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0; 4],
        }; 5],
        visBounds: [[0.0; 3]; 2],
        zFar: 0.0,
        isPortal: 0,
        isMirror: 0,
        pvsOrigin: [0.0; 3],
        portalPlane: cplane_t {
            normal: [0.0; 3],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0; 4],
        },
        frameSceneNum: 0,
        frameCount: 0,
    },
    viewCount: 0,
    frameCount: 0,
    frameSceneNum: 0,
    refdef: refdef_t {
        time: 0,
        rdflags: 0,
        vieworg: [0.0; 3],
        viewaxis: [[0.0; 3]; 3],
        fov_x: 0.0,
        fov_y: 0.0,
        drawSurfs: core::ptr::null_mut(),
        numDrawSurfs: 0,
        entities: core::ptr::null_mut(),
        num_entities: 0,
        dlights: core::ptr::null_mut(),
        numDlights: 0,
        fogIndex: 0,
    },
    currentEntityNum: 0,
    currentEntity: core::ptr::null_mut(),
    currentModel: core::ptr::null_mut(),
    defaultShader: core::ptr::null_mut(),
    shiftedEntityNum: 0,
    world: core::ptr::null_mut(),
    distanceCull: 0.0,
    sortedShaders: [core::ptr::null_mut(); 4096],
};

static mut s_flipMatrix: [f32; 16] = [
    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    0.0, 0.0, -1.0, 0.0,
    -1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0
];

// entities that will have procedurally generated surfaces will just
// point at this for their sorting surface
pub static mut entitySurface: c_int = SF_ENTITY;

pub static mut preTransEntMatrix: [f32; 16] = [0.0; 16];

pub static mut recursivePortalCount: c_int = 0;

pub static mut tess: tess_t = tess_t {
    numVertexes: 0,
    xyz: [[0.0; 3]; 4000],
    normal: [[0.0; 3]; 4000],
    indexes: [0; 6000],
    numIndexes: 0,
    shader: core::ptr::null_mut(),
};

// External function declarations
extern "C" {
    pub fn r_nocull() -> *mut c_void;
    pub fn r_noportals() -> *mut c_void;
    pub fn r_fastsky() -> *mut c_void;
    pub fn r_portalOnly() -> *mut c_void;
    pub fn r_znear() -> *mut c_void;
    pub fn r_drawentities() -> *mut c_void;
    pub fn r_debugSurface() -> *mut c_void;
    pub fn r_debugStyle() -> *mut c_void;

    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorClear(v: *mut vec3_t);
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t);
    pub fn VectorAdd(a: *const vec3_t, b: *const vec3_t, c: *mut vec3_t);
    pub fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
    pub fn VectorMA(v: *const vec3_t, scale: f32, dir: *const vec3_t, out: *mut vec3_t);
    pub fn VectorLength(v: *const vec3_t) -> f32;
    pub fn VectorLengthSquared(v: *const vec3_t) -> f32;
    pub fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, cross: *mut vec3_t);
    pub fn PerpendicularVector(dst: *mut vec3_t, src: *const vec3_t);
    pub fn AxisCopy(in_: *const [vec3_t; 3], out: *mut [vec3_t; 3]);
    pub fn RotatePointAroundVector(dst: *mut vec3_t, axis: *const vec3_t, point: *const vec3_t, degrees: f32);
    pub fn DistanceSquared(a: *const vec3_t, b: *const vec3_t) -> f32;

    pub fn Com_Clamp(min: f32, ideal: f32, max: f32) -> f32;
    pub fn Com_Error(code: c_int, msg: *const u8, ...);
    pub fn Com_Printf(msg: *const u8, ...);

    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    pub fn sqrtf(x: f32) -> f32;
    pub fn sinf(x: f32) -> f32;
    pub fn cosf(x: f32) -> f32;
    pub fn tanf(x: f32) -> f32;

    pub fn PlaneFromPoints(plane: *mut vec4_t, a: *const vec3_t, b: *const vec3_t, c: *const vec3_t);
    pub fn SetPlaneSignbits(plane: *mut cplane_t);

    pub fn R_GetShaderByHandle(hShader: c_int) -> *mut shader_t;
    pub fn R_GetModelByHandle(hModel: c_int) -> *mut model_t;
    pub fn RB_BeginSurface(shader: *mut shader_t, fogNum: c_int);
    pub fn R_AddMD3Surfaces(ent: *mut trRefEntity_t);
    pub fn R_AddBrushModelSurfaces(ent: *mut trRefEntity_t);
    pub fn R_AddGhoulSurfaces(ent: *mut trRefEntity_t);
    pub fn R_AddWorldSurfaces();
    pub fn R_AddPolygonSurfaces();
    pub fn R_AddTerrainSurfaces();
    pub fn R_AddDrawSurfCmd(drawSurfs: *mut drawSurf_t, numDrawSurfs: c_int);
    pub fn R_RenderView(parms: *mut viewParms_t);
    pub fn R_FogParmsMatch(fog1: c_int, fog2: c_int) -> c_int;
    pub fn GL_State(stateBits: c_int);
    pub fn GL_Bind(image: *mut c_void);
    pub fn GL_Cull(cullType: c_int);
    pub fn CM_DrawDebugSurface(drawPolygon: extern "C" fn(c_int, c_int, *mut f32));
    pub fn VID_Printf(print_level: c_int, msg: *const u8, ...);
    pub fn SV_PointContents(point: *const vec3_t, passEntityNum: c_int) -> c_int;
    pub fn G2API_HaveWeGhoul2Models(ghoul2: *const c_void) -> c_int;
    pub fn assert(cond: c_int);
}

// Utility functions for accessing static mut cvars
unsafe fn get_r_nocull() -> *mut cvarValue_t {
    r_nocull() as *mut cvarValue_t
}

unsafe fn get_r_noportals() -> *mut cvarValue_t {
    r_noportals() as *mut cvarValue_t
}

unsafe fn get_r_fastsky() -> *mut cvarValue_t {
    r_fastsky() as *mut cvarValue_t
}

unsafe fn get_r_portalOnly() -> *mut cvarValue_t {
    r_portalOnly() as *mut cvarValue_t
}

unsafe fn get_r_znear() -> *mut cvarValue_t {
    r_znear() as *mut cvarValue_t
}

unsafe fn get_r_drawentities() -> *mut cvarValue_t {
    r_drawentities() as *mut cvarValue_t
}

unsafe fn get_r_debugSurface() -> *mut cvarValue_t {
    r_debugSurface() as *mut cvarValue_t
}

unsafe fn get_r_debugStyle() -> *mut cvarValue_t {
    r_debugStyle() as *mut cvarValue_t
}

#[repr(C)]
pub struct cvarValue_t {
    pub integer: c_int,
    pub value: f32,
}

const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

/*
=================
R_CullLocalBox

Returns CULL_IN, CULL_CLIP, or CULL_OUT
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_CullLocalBox(bounds: *const [vec3_t; 2]) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut transformed: [vec3_t; 8] = [[0.0; 3]; 8];
    let mut dists: [f32; 8] = [0.0; 8];
    let mut v: vec3_t = [0.0; 3];
    let mut frust: *mut cplane_t;
    let mut anyBack: c_int;
    let mut front: c_int;
    let mut back: c_int;

    if (*get_r_nocull()).integer == 1 {
        return CULL_CLIP;
    }

    // transform into world space
    i = 0;
    while i < 8 {
        v[0] = (*bounds)[i as usize & 1][0];
        v[1] = (*bounds)[((i >> 1) as usize) & 1][1];
        v[2] = (*bounds)[((i >> 2) as usize) & 1][2];

        VectorCopy(&tr.or.origin as *const vec3_t, &mut transformed[i as usize]);
        VectorMA(&transformed[i as usize], v[0], &tr.or.axis[0], &mut transformed[i as usize]);
        VectorMA(&transformed[i as usize], v[1], &tr.or.axis[1], &mut transformed[i as usize]);
        VectorMA(&transformed[i as usize], v[2], &tr.or.axis[2], &mut transformed[i as usize]);

        i += 1;
    }

    // check against frustum planes
    anyBack = 0;
    i = 0;
    while i < 5 {
        frust = &mut tr.viewParms.frustum[i as usize];

        front = 0;
        back = 0;
        j = 0;
        while j < 8 {
            dists[j as usize] = DotProduct(&transformed[j as usize], &(*frust).normal);
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

/*
** R_CullLocalPointAndRadius
*/
#[no_mangle]
pub unsafe extern "C" fn R_CullLocalPointAndRadius(pt: *const vec3_t, radius: f32) -> c_int {
    let mut transformed: vec3_t = [0.0; 3];

    R_LocalPointToWorld(pt, &mut transformed);

    return R_CullPointAndRadius(&transformed, radius);
}

/*
** R_CullPointAndRadius
*/
#[no_mangle]
pub unsafe extern "C" fn R_CullPointAndRadius(pt: *const vec3_t, radius: f32) -> c_int {
    let mut i: c_int;
    let mut dist: f32;
    let mut frust: *mut cplane_t;
    let mut mightBeClipped: c_int = 0; // qfalse

    if (*get_r_nocull()).integer == 1 {
        return CULL_CLIP;
    }

    // check against frustum planes
    i = 0;
    while i < 5 {
        frust = &mut tr.viewParms.frustum[i as usize];

        dist = DotProduct(pt, &(*frust).normal) - (*frust).dist;
        if dist < -radius {
            return CULL_OUT;
        } else if dist <= radius {
            mightBeClipped = 1; // qtrue
        }

        i += 1;
    }

    if mightBeClipped != 0 {
        return CULL_CLIP;
    }

    return CULL_IN;		// completely inside frustum
}


/*
=================
R_LocalNormalToWorld

=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_LocalNormalToWorld(local: *const vec3_t, world: *mut vec3_t) {
    (*world)[0] = (*local)[0] * tr.or.axis[0][0] + (*local)[1] * tr.or.axis[1][0] + (*local)[2] * tr.or.axis[2][0];
    (*world)[1] = (*local)[0] * tr.or.axis[0][1] + (*local)[1] * tr.or.axis[1][1] + (*local)[2] * tr.or.axis[2][1];
    (*world)[2] = (*local)[0] * tr.or.axis[0][2] + (*local)[1] * tr.or.axis[1][2] + (*local)[2] * tr.or.axis[2][2];
}

/*
=================
R_LocalPointToWorld

=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_LocalPointToWorld(local: *const vec3_t, world: *mut vec3_t) {
    (*world)[0] = (*local)[0] * tr.or.axis[0][0] + (*local)[1] * tr.or.axis[1][0] + (*local)[2] * tr.or.axis[2][0] + tr.or.origin[0];
    (*world)[1] = (*local)[0] * tr.or.axis[0][1] + (*local)[1] * tr.or.axis[1][1] + (*local)[2] * tr.or.axis[2][1] + tr.or.origin[1];
    (*world)[2] = (*local)[0] * tr.or.axis[0][2] + (*local)[1] * tr.or.axis[1][2] + (*local)[2] * tr.or.axis[2][2] + tr.or.origin[2];
}

#[no_mangle]
pub unsafe extern "C" fn R_InvertMatrix(sourcemat: *mut f32, destmat: *mut f32) {
    let mut i: c_int;
    let mut j: c_int;
    let mut temp: c_int = 0;

    i = 0;
    while i < 3 {
        j = 0;
        while j < 3 {
            *destmat.offset((j * 4 + i) as isize) = *sourcemat.offset(temp as isize);
            temp += 1;
            j += 1;
        }
        i += 1;
    }
    i = 0;
    while i < 3 {
        temp = i * 4;
        *destmat.offset((temp + 3) as isize) = 0.0;		// destmat[destmat[i][3]=0;
        j = 0;
        while j < 3 {
            *destmat.offset((temp + 3) as isize) -= *destmat.offset((temp + j) as isize) * *sourcemat.offset((j * 4 + 3) as isize);		// dest->matrix[i][3]-=dest->matrix[i][j]*src->matrix[j][3];
            j += 1;
        }
        i += 1;
    }
}

/*
=================
R_WorldNormalToEntity

=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_WorldNormalToEntity(worldvec: *const vec3_t, entvec: *mut vec3_t) {
    (*entvec)[0] = -(*worldvec)[0] * preTransEntMatrix[0] - (*worldvec)[1] * preTransEntMatrix[4] + (*worldvec)[2] * preTransEntMatrix[8];
    (*entvec)[1] = -(*worldvec)[0] * preTransEntMatrix[1] - (*worldvec)[1] * preTransEntMatrix[5] + (*worldvec)[2] * preTransEntMatrix[9];
    (*entvec)[2] = -(*worldvec)[0] * preTransEntMatrix[2] - (*worldvec)[1] * preTransEntMatrix[6] + (*worldvec)[2] * preTransEntMatrix[10];
}

/*
=================
R_WorldPointToEntity

=================
*/
/*#[no_mangle]
pub unsafe extern "C" fn R_WorldPointToEntity(worldvec: *mut vec3_t, entvec: *mut vec3_t) {
    (*entvec)[0] = (*worldvec)[0] * preTransEntMatrix[0] + (*worldvec)[1] * preTransEntMatrix[4] + (*worldvec)[2] * preTransEntMatrix[8] + preTransEntMatrix[12];
    (*entvec)[1] = (*worldvec)[0] * preTransEntMatrix[1] + (*worldvec)[1] * preTransEntMatrix[5] + (*worldvec)[2] * preTransEntMatrix[9] + preTransEntMatrix[13];
    (*entvec)[2] = (*worldvec)[0] * preTransEntMatrix[2] + (*worldvec)[1] * preTransEntMatrix[6] + (*worldvec)[2] * preTransEntMatrix[10] + preTransEntMatrix[14];
}
*/

/*
=================
R_WorldToLocal

=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_WorldToLocal(world: *mut vec3_t, local: *mut vec3_t) {
    (*local)[0] = DotProduct(world, &tr.or.axis[0]);
    (*local)[1] = DotProduct(world, &tr.or.axis[1]);
    (*local)[2] = DotProduct(world, &tr.or.axis[2]);
}

/*
==========================
R_TransformModelToClip

==========================
*/
#[no_mangle]
pub unsafe extern "C" fn R_TransformModelToClip(src: *const vec3_t, modelMatrix: *const f32, projectionMatrix: *const f32,
                            eye: *mut vec4_t, dst: *mut vec4_t) {
    let mut i: c_int;

    i = 0;
    while i < 4 {
        (*eye)[i as usize] =
            (*src)[0] * *modelMatrix.offset(i as isize + 0 * 4) +
            (*src)[1] * *modelMatrix.offset(i as isize + 1 * 4) +
            (*src)[2] * *modelMatrix.offset(i as isize + 2 * 4) +
            1.0 * *modelMatrix.offset(i as isize + 3 * 4);
        i += 1;
    }

    i = 0;
    while i < 4 {
        (*dst)[i as usize] =
            (*eye)[0] * *projectionMatrix.offset(i as isize + 0 * 4) +
            (*eye)[1] * *projectionMatrix.offset(i as isize + 1 * 4) +
            (*eye)[2] * *projectionMatrix.offset(i as isize + 2 * 4) +
            (*eye)[3] * *projectionMatrix.offset(i as isize + 3 * 4);
        i += 1;
    }
}

/*
==========================
R_TransformClipToWindow

==========================
*/
#[no_mangle]
pub unsafe extern "C" fn R_TransformClipToWindow(clip: *const vec4_t, view: *const viewParms_t, normalized: *mut vec4_t, window: *mut vec4_t) {
    (*normalized)[0] = (*clip)[0] / (*clip)[3];
    (*normalized)[1] = (*clip)[1] / (*clip)[3];
    (*normalized)[2] = ((*clip)[2] + (*clip)[3]) / (2.0 * (*clip)[3]);

    (*window)[0] = 0.5 * (1.0 + (*normalized)[0]) * (*view).viewportWidth as f32;
    (*window)[1] = 0.5 * (1.0 + (*normalized)[1]) * (*view).viewportHeight as f32;
    (*window)[2] = (*normalized)[2];

    (*window)[0] = ((*window)[0] + 0.5) as i32 as f32;
    (*window)[1] = ((*window)[1] + 0.5) as i32 as f32;
}


/*
==========================
myGlMultMatrix

==========================
*/
#[no_mangle]
pub unsafe extern "C" fn myGlMultMatrix(a: *const f32, b: *const f32, out: *mut f32) {
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

/*
=================
R_RotateForEntity

Generates an orientation for an entity and viewParms
Does NOT produce any GL calls
Called by both the front end and the back end
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_RotateForEntity(ent: *const trRefEntity_t, viewParms: *const viewParms_t,
                           or: *mut orientationr_t) {
    //	float	glMatrix[16];
    let mut delta: vec3_t = [0.0; 3];
    let mut axisLength: f32;

    if (*ent).e.reType != RT_MODEL {
        *or = (*viewParms).world;
        return;
    }

    VectorCopy(&(*ent).e.origin, &mut (*or).origin);

    VectorCopy(&(*ent).e.axis[0], &mut (*or).axis[0]);
    VectorCopy(&(*ent).e.axis[1], &mut (*or).axis[1]);
    VectorCopy(&(*ent).e.axis[2], &mut (*or).axis[2]);

    preTransEntMatrix[0] = (*or).axis[0][0];
    preTransEntMatrix[4] = (*or).axis[1][0];
    preTransEntMatrix[8] = (*or).axis[2][0];
    preTransEntMatrix[12] = (*or).origin[0];

    preTransEntMatrix[1] = (*or).axis[0][1];
    preTransEntMatrix[5] = (*or).axis[1][1];
    preTransEntMatrix[9] = (*or).axis[2][1];
    preTransEntMatrix[13] = (*or).origin[1];

    preTransEntMatrix[2] = (*or).axis[0][2];
    preTransEntMatrix[6] = (*or).axis[1][2];
    preTransEntMatrix[10] = (*or).axis[2][2];
    preTransEntMatrix[14] = (*or).origin[2];

    preTransEntMatrix[3] = 0.0;
    preTransEntMatrix[7] = 0.0;
    preTransEntMatrix[11] = 0.0;
    preTransEntMatrix[15] = 1.0;

    myGlMultMatrix(&preTransEntMatrix[0], &(*viewParms).world.modelMatrix[0], &mut (*or).modelMatrix[0]);

    // calculate the viewer origin in the model's space
    // needed for fog, specular, and environment mapping
    VectorSubtract(&(*viewParms).or.origin, &(*or).origin, &mut delta);

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

    (*or).viewOrigin[0] = DotProduct(&delta, &(*or).axis[0]) * axisLength;
    (*or).viewOrigin[1] = DotProduct(&delta, &(*or).axis[1]) * axisLength;
    (*or).viewOrigin[2] = DotProduct(&delta, &(*or).axis[2]) * axisLength;
}

/*
=================
R_RotateForViewer

Sets up the modelview matrix for a given viewParm
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_RotateForViewer() {
    let mut viewerMatrix: [f32; 16] = [0.0; 16];
    let mut origin: vec3_t = [0.0; 3];

    memset(&mut tr.or as *mut orientationr_t as *mut c_void, 0, size_of::<orientationr_t>());
    tr.or.axis[0][0] = 1.0;
    tr.or.axis[1][1] = 1.0;
    tr.or.axis[2][2] = 1.0;
    VectorCopy(&tr.viewParms.or.origin, &mut tr.or.viewOrigin);

    // transform by the camera placement
    VectorCopy(&tr.viewParms.or.origin, &mut origin);

    viewerMatrix[0] = tr.viewParms.or.axis[0][0];
    viewerMatrix[4] = tr.viewParms.or.axis[0][1];
    viewerMatrix[8] = tr.viewParms.or.axis[0][2];
    viewerMatrix[12] = -origin[0] * viewerMatrix[0] + -origin[1] * viewerMatrix[4] + -origin[2] * viewerMatrix[8];

    viewerMatrix[1] = tr.viewParms.or.axis[1][0];
    viewerMatrix[5] = tr.viewParms.or.axis[1][1];
    viewerMatrix[9] = tr.viewParms.or.axis[1][2];
    viewerMatrix[13] = -origin[0] * viewerMatrix[1] + -origin[1] * viewerMatrix[5] + -origin[2] * viewerMatrix[9];

    viewerMatrix[2] = tr.viewParms.or.axis[2][0];
    viewerMatrix[6] = tr.viewParms.or.axis[2][1];
    viewerMatrix[10] = tr.viewParms.or.axis[2][2];
    viewerMatrix[14] = -origin[0] * viewerMatrix[2] + -origin[1] * viewerMatrix[6] + -origin[2] * viewerMatrix[10];

    viewerMatrix[3] = 0.0;
    viewerMatrix[7] = 0.0;
    viewerMatrix[11] = 0.0;
    viewerMatrix[15] = 1.0;

    // convert from our coordinate system (looking down X)
    // to OpenGL's coordinate system (looking down -Z)
    myGlMultMatrix(&viewerMatrix[0], &s_flipMatrix[0], &mut tr.or.modelMatrix[0]);

    tr.viewParms.world = tr.or;
}

/*
** SetFarClip
*/
unsafe fn SetFarClip() {
    let mut farthestCornerDistance: f32 = 0.0;
    let mut i: c_int;

    // if not rendering the world (icons, menus, etc)
    // set a 2k far clip plane
    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        tr.viewParms.zFar = 2048.0;
        return;
    }

    //
    // set far clipping planes dynamically
    //
    i = 0;
    while i < 8 {
        let mut v: vec3_t = [0.0; 3];
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

        distance = DistanceSquared(&tr.viewParms.or.origin, &v);

        if distance > farthestCornerDistance {
            farthestCornerDistance = distance;
        }

        i += 1;
    }
    // Bring in the zFar to the distanceCull distance
    // The sky renders at zFar so need to move it out a little
    // ...and make sure there is a minimum zfar to prevent problems
    tr.viewParms.zFar = Com_Clamp(2048.0, tr.distanceCull * 1.732, sqrtf(farthestCornerDistance));
}


/*
===============
R_SetupProjection
===============
*/
#[no_mangle]
pub unsafe extern "C" fn R_SetupProjection() {
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
    zNear = (*get_r_znear()).value;
    zFar = tr.viewParms.zFar;

    ymax = zNear * tanf(tr.refdef.fov_y * M_PI / 360.0);
    ymin = -ymax;

    xmax = zNear * tanf(tr.refdef.fov_x * M_PI / 360.0);
    xmin = -xmax;

    width = xmax - xmin;
    height = ymax - ymin;
    depth = zFar - zNear;

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

/*
=================
R_SetupFrustum

Setup that culling frustum planes for the current view
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_SetupFrustum() {
    let mut i: c_int;
    let mut xs: f32;
    let mut xc: f32;
    let mut ang: f32;

    ang = tr.viewParms.fovX / 180.0 * M_PI * 0.5;
    xs = sinf(ang);
    xc = cosf(ang);

    VectorScale(&tr.viewParms.or.axis[0], xs, &mut tr.viewParms.frustum[0].normal);
    VectorMA(&tr.viewParms.frustum[0].normal, xc, &tr.viewParms.or.axis[1], &mut tr.viewParms.frustum[0].normal);

    VectorScale(&tr.viewParms.or.axis[0], xs, &mut tr.viewParms.frustum[1].normal);
    VectorMA(&tr.viewParms.frustum[1].normal, -xc, &tr.viewParms.or.axis[1], &mut tr.viewParms.frustum[1].normal);

    ang = tr.viewParms.fovY / 180.0 * M_PI * 0.5;
    xs = sinf(ang);
    xc = cosf(ang);

    VectorScale(&tr.viewParms.or.axis[0], xs, &mut tr.viewParms.frustum[2].normal);
    VectorMA(&tr.viewParms.frustum[2].normal, xc, &tr.viewParms.or.axis[2], &mut tr.viewParms.frustum[2].normal);

    VectorScale(&tr.viewParms.or.axis[0], xs, &mut tr.viewParms.frustum[3].normal);
    VectorMA(&tr.viewParms.frustum[3].normal, -xc, &tr.viewParms.or.axis[2], &mut tr.viewParms.frustum[3].normal);


    // this is the far plane
    VectorScale(&tr.viewParms.or.axis[0], -1.0, &mut tr.viewParms.frustum[4].normal);

    i = 0;
    while i < 5 {
        tr.viewParms.frustum[i as usize].r#type = PLANE_NON_AXIAL;
        tr.viewParms.frustum[i as usize].dist = DotProduct(&tr.viewParms.or.origin, &tr.viewParms.frustum[i as usize].normal);
        if i == 4 {
            // far plane does not go through the view point, it goes alot farther..
            tr.viewParms.frustum[i as usize].dist -= tr.distanceCull * 1.02; // a little slack so we don't cull stuff
        }
        SetPlaneSignbits(&mut tr.viewParms.frustum[i as usize]);
        i += 1;
    }
}


/*
=================
R_MirrorPoint
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_MirrorPoint(in_: *mut vec3_t, surface: *mut orientation_t, camera: *mut orientation_t, out: *mut vec3_t) {
    let mut i: c_int;
    let mut local: vec3_t = [0.0; 3];
    let mut transformed: vec3_t = [0.0; 3];
    let mut d: f32;

    VectorSubtract(in_, &(*surface).origin, &mut local);

    VectorClear(&mut transformed);
    i = 0;
    while i < 3 {
        d = DotProduct(&local, &(*surface).axis[i as usize]);
        VectorMA(&transformed, d, &(*camera).axis[i as usize], &mut transformed);
        i += 1;
    }

    VectorAdd(&transformed, &(*camera).origin, out);
}

#[no_mangle]
pub unsafe extern "C" fn R_MirrorVector(in_: *mut vec3_t, surface: *mut orientation_t, camera: *mut orientation_t, out: *mut vec3_t) {
    let mut i: c_int;
    let mut d: f32;

    VectorClear(out);
    i = 0;
    while i < 3 {
        d = DotProduct(in_, &(*surface).axis[i as usize]);
        VectorMA(out, d, &(*camera).axis[i as usize], out);
        i += 1;
    }
}


/*
=============
R_PlaneForSurface
=============
*/
#[no_mangle]
pub unsafe extern "C" fn R_PlaneForSurface(surfType: *mut surfaceType_t, plane: *mut cplane_t) {
    let mut tri: *mut srfTriangles_t;
    let mut grid: *mut srfGridMesh_t;
    let mut poly: *mut srfPoly_t;
    let mut v1: *mut drawVert_t;
    let mut v2: *mut drawVert_t;
    let mut v3: *mut drawVert_t;
    let mut plane4: vec4_t = [0.0; 4];

    if surfType.is_null() {
        memset(plane as *mut c_void, 0, size_of::<cplane_t>());
        (*plane).normal[0] = 1.0;
        return;
    }
    let surf_type = *(surfType as *const c_int);
    match surf_type {
    SF_FACE => {
        *plane = (*(surfType as *mut srfSurfaceFace_t)).plane;
        return;
    },
    SF_TRIANGLES => {
        tri = surfType as *mut srfTriangles_t;
        v1 = (*tri).verts.offset(*((*tri).indexes) as isize);
        v2 = (*tri).verts.offset(*((*tri).indexes.offset(1)) as isize);
        v3 = (*tri).verts.offset(*((*tri).indexes.offset(2)) as isize);
        PlaneFromPoints(&mut plane4, &(*v1).xyz, &(*v2).xyz, &(*v3).xyz);
        VectorCopy(&plane4 as *const vec4_t as *const vec3_t, &mut (*plane).normal);
        (*plane).dist = plane4[3];
        return;
    },
    SF_POLY => {
        poly = surfType as *mut srfPoly_t;
        PlaneFromPoints(&mut plane4, &(*(*poly).verts).xyz, &(*(*poly).verts.offset(1)).xyz, &(*(*poly).verts.offset(2)).xyz);
        VectorCopy(&plane4 as *const vec4_t as *const vec3_t, &mut (*plane).normal);
        (*plane).dist = plane4[3];
        return;
    },
    SF_GRID => {
        grid = surfType as *mut srfGridMesh_t;
        v1 = &(*(*grid).verts);
        v2 = (*grid).verts.offset(1);
        v3 = (*grid).verts.offset(2);
        PlaneFromPoints(&mut plane4, &(*v3).xyz, &(*v2).xyz, &(*v1).xyz);
        VectorCopy(&plane4 as *const vec4_t as *const vec3_t, &mut (*plane).normal);
        (*plane).dist = plane4[3];
        return;
    },
    _ => {
        memset(plane as *mut c_void, 0, size_of::<cplane_t>());
        (*plane).normal[0] = 1.0;
        return;
    },
    }
}

/*
=================
R_GetPortalOrientation

entityNum is the entity that the portal surface is a part of, which may
be moving and rotating.

Returns qtrue if it should be mirrored
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_GetPortalOrientations(drawSurf: *mut drawSurf_t, entityNum: c_int,
                             surface: *mut orientation_t, camera: *mut orientation_t,
                             pvsOrigin: *mut vec3_t, mirror: *mut c_int) -> c_int {
    let mut i: c_int;
    let mut originalPlane: cplane_t;
    let mut plane: cplane_t;
    let mut e: *mut trRefEntity_t;
    let mut d: f32;
    let mut transformed: vec3_t = [0.0; 3];

    // create plane axis for the portal we are seeing
    R_PlaneForSurface((*drawSurf).surface, &mut originalPlane);

    // rotate the plane if necessary
    if entityNum != TR_WORLDENT {
        tr.currentEntityNum = entityNum;
        tr.currentEntity = &mut tr.refdef.entities[entityNum as usize];

        // get the orientation of the entity
        R_RotateForEntity(tr.currentEntity, &tr.viewParms, &mut tr.or);

        // rotate the plane, but keep the non-rotated version for matching
        // against the portalSurface entities
        R_LocalNormalToWorld(&originalPlane.normal, &mut plane.normal);
        plane.dist = originalPlane.dist + DotProduct(&plane.normal, &tr.or.origin);

        // translate the original plane
        originalPlane.dist = originalPlane.dist + DotProduct(&originalPlane.normal, &tr.or.origin);
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
        e = &mut (*tr.refdef.entities.offset(i as isize));
        if (*e).e.reType != RT_PORTALSURFACE {
            i += 1;
            continue;
        }

        d = DotProduct(&(*e).e.origin, &originalPlane.normal) - originalPlane.dist;
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

            *mirror = 1; // qtrue
            return 1; // qtrue
        }

        // project the origin onto the surface plane to get
        // an origin point we can rotate around
        d = DotProduct(&(*e).e.origin, &plane.normal) - plane.dist;
        VectorMA(&(*e).e.origin, -d, &(*surface).axis[0], &mut (*surface).origin);

        // now get the camera origin and orientation
        VectorCopy(&(*e).e.oldorigin, &mut (*camera).origin);
        AxisCopy(&(*e).e.axis as *const [vec3_t; 3], &mut (*camera).axis);
        VectorSubtract(&vec3_origin, &(*camera).axis[0], &mut (*camera).axis[0]);
        VectorSubtract(&vec3_origin, &(*camera).axis[1], &mut (*camera).axis[1]);

        // optionally rotate
        if (*e).e.frame != 0 {
            // continuous rotate
            d = (tr.refdef.time as f32 / 1000.0) * (*e).e.frame as f32;
            VectorCopy(&(*camera).axis[1], &mut transformed);
            RotatePointAroundVector(&mut (*camera).axis[1], &(*camera).axis[0], &transformed, d);
            CrossProduct(&(*camera).axis[0], &(*camera).axis[1], &mut (*camera).axis[2]);
        } else if (*e).e.skinNum != 0 {
            // bobbing rotate
            //d = 4 * sin( tr.refdef.time * 0.003 );
            d = (*e).e.skinNum as f32;
            VectorCopy(&(*camera).axis[1], &mut transformed);
            RotatePointAroundVector(&mut (*camera).axis[1], &(*camera).axis[0], &transformed, d);
            CrossProduct(&(*camera).axis[0], &(*camera).axis[1], &mut (*camera).axis[2]);
        }
        *mirror = 0; // qfalse
        return 1; // qtrue
        i += 1;
    }

    // if we didn't locate a portal entity, don't render anything.
    // We don't want to just treat it as a mirror, because without a
    // portal entity the server won't have communicated a proper entity set
    // in the snapshot

    // unfortunately, with local movement prediction it is easily possible
    // to see a surface before the server has communicated the matching
    // portal surface entity, so we don't want to print anything here...

    //VID_Printf( PRINT_ALL, "Portal surface without a portal entity\n" );

    return 0; // qfalse
}

unsafe fn IsMirror(drawSurf: *const drawSurf_t, entityNum: c_int) -> c_int {
    let mut i: c_int;
    let mut originalPlane: cplane_t;
    let mut plane: cplane_t;
    let mut e: *mut trRefEntity_t;
    let mut d: f32;

    // create plane axis for the portal we are seeing
    R_PlaneForSurface((*drawSurf).surface, &mut originalPlane);

    // rotate the plane if necessary
    if entityNum != TR_WORLDENT {
        tr.currentEntityNum = entityNum;
        tr.currentEntity = &mut tr.refdef.entities[entityNum as usize];

        // get the orientation of the entity
        R_RotateForEntity(tr.currentEntity, &tr.viewParms, &mut tr.or);

        // rotate the plane, but keep the non-rotated version for matching
        // against the portalSurface entities
        R_LocalNormalToWorld(&originalPlane.normal, &mut plane.normal);
        plane.dist = originalPlane.dist + DotProduct(&plane.normal, &tr.or.origin);

        // translate the original plane
        originalPlane.dist = originalPlane.dist + DotProduct(&originalPlane.normal, &tr.or.origin);
    } else {
        plane = originalPlane;
    }

    // locate the portal entity closest to this plane.
    // origin will be the origin of the portal, origin2 will be
    // the origin of the camera
    i = 0;
    while i < tr.refdef.num_entities {
        e = &mut (*tr.refdef.entities.offset(i as isize));
        if (*e).e.reType != RT_PORTALSURFACE {
            i += 1;
            continue;
        }

        d = DotProduct(&(*e).e.origin, &originalPlane.normal) - originalPlane.dist;
        if d > 64.0 || d < -64.0 {
            i += 1;
            continue;
        }

        // if the entity is just a mirror, don't use as a camera point
        if (*e).e.oldorigin[0] == (*e).e.origin[0] &&
            (*e).e.oldorigin[1] == (*e).e.origin[1] &&
            (*e).e.oldorigin[2] == (*e).e.origin[2] {
            return 1; // qtrue
        }

        return 0; // qfalse
        i += 1;
    }
    return 0; // qfalse
}

/*
** SurfIsOffscreen
**
** Determines if a surface is completely offscreen.
*/
unsafe fn SurfIsOffscreen(drawSurf: *const drawSurf_t, _clipDest: *mut [vec4_t; 128]) -> c_int {
    let mut shortest: f32 = 1000000000.0;
    let mut entityNum: c_int;
    let mut numTriangles: c_int;
    let mut shader: *mut shader_t;
    let mut fogNum: c_int;
    let mut dlighted: c_int;
    let mut clip: vec4_t = [0.0; 4];
    let mut eye: vec4_t = [0.0; 4];
    let mut i: c_int;
    let mut pointOr: u32 = 0;
    let mut pointAnd: u32 = !0u32;

    R_RotateForViewer();

    R_DecomposeSort((*drawSurf).sort, &mut entityNum, &mut shader, &mut fogNum, &mut dlighted);
    RB_BeginSurface(shader, fogNum);
    let rb_surfaceTable_func: extern "C" fn(*mut surfaceType_t) = core::mem::transmute(0usize); // stub
    rb_surfaceTable_func((*drawSurf).surface);

    assert((tess.numVertexes < 128) as c_int);

    i = 0;
    while i < tess.numVertexes {
        let mut j: c_int;
        let mut pointFlags: u32 = 0;

        R_TransformModelToClip(&tess.xyz[i as usize], &tr.or.modelMatrix[0], &tr.viewParms.projectionMatrix[0], &mut eye, &mut clip);

        j = 0;
        while j < 3 {
            if clip[j as usize] >= clip[3] {
                pointFlags |= 1u32 << ((j * 2) as u32);
            } else if clip[j as usize] <= -clip[3] {
                pointFlags |= 1u32 << ((j * 2 + 1) as u32);
            }
            j += 1;
        }
        pointAnd &= pointFlags;
        pointOr |= pointFlags;

        i += 1;
    }

    // trivially reject
    if pointAnd != 0 {
        return 1; // qtrue
    }

    // determine if this surface is backfaced and also determine the distance
    // to the nearest vertex so we can cull based on portal range.  Culling
    // based on vertex distance isn't 100% correct (we should be checking for
    // range to the surface), but it's good enough for the types of portals
    // we have in the game right now.
    numTriangles = tess.numIndexes / 3;

    i = 0;
    while i < tess.numIndexes {
        let mut normal: vec3_t = [0.0; 3];
        let mut dot: f32;
        let mut len: f32;

        VectorSubtract(&tess.xyz[tess.indexes[i as usize] as usize], &tr.viewParms.or.origin, &mut normal);

        len = VectorLengthSquared(&normal);			// lose the sqrt
        if len < shortest {
            shortest = len;
        }

        if (dot = DotProduct(&normal, &tess.normal[tess.indexes[i as usize] as usize])) >= 0.0 {
            numTriangles -= 1;
        }

        i += 3;
    }
    if numTriangles == 0 {
        return 1; // qtrue
    }

    // mirrors can early out at this point, since we don't do a fade over distance
    // with them (although we could)
    if IsMirror(drawSurf, entityNum) != 0 {
        return 0; // qfalse
    }

    if shortest > ((*shader).portalRange * (*shader).portalRange) {
        return 1; // qtrue
    }

    return 0; // qfalse
}

/*
========================
R_MirrorViewBySurface

Returns qtrue if another view has been rendered
========================
*/
#[no_mangle]
pub unsafe extern "C" fn R_MirrorViewBySurface(drawSurf: *mut drawSurf_t, entityNum: c_int) -> c_int {
    let mut clipDest: [vec4_t; 128] = [[0.0; 4]; 128];
    let mut newParms: viewParms_t;
    let mut oldParms: viewParms_t;
    let mut surface: orientation_t;
    let mut camera: orientation_t;

    // don't recursively mirror
    if (tr.viewParms.isPortal) != 0 {
        VID_Printf(PRINT_DEVELOPER, b"WARNING: recursive mirror/portal found\n" as *const u8);
        return 0; // qfalse
    }

    if (*get_r_noportals()).integer != 0 || (*get_r_fastsky()).integer != 0 {
        return 0; // qfalse
    }

    // trivially reject portal/mirror
    if SurfIsOffscreen(drawSurf, &mut clipDest) != 0 {
        return 0; // qfalse
    }

    // save old viewParms so we can return to it after the mirror view
    oldParms = tr.viewParms;

    newParms = tr.viewParms;
    newParms.isPortal = 1; // qtrue
    let mut mirror: c_int = 0;
    if R_GetPortalOrientations(drawSurf, entityNum, &mut surface, &mut camera,
        &mut newParms.pvsOrigin, &mut mirror) == 0 {
        return 0; // qfalse		// bad portal, no portalentity
    }
    newParms.isMirror = mirror;

    R_MirrorPoint(&mut oldParms.or.origin, &mut surface, &mut camera, &mut newParms.or.origin);

    VectorSubtract(&vec3_origin, &camera.axis[0], &mut newParms.portalPlane.normal);
    newParms.portalPlane.dist = DotProduct(&camera.origin, &newParms.portalPlane.normal);

    R_MirrorVector(&mut oldParms.or.axis[0], &mut surface, &mut camera, &mut newParms.or.axis[0]);
    R_MirrorVector(&mut oldParms.or.axis[1], &mut surface, &mut camera, &mut newParms.or.axis[1]);
    R_MirrorVector(&mut oldParms.or.axis[2], &mut surface, &mut camera, &mut newParms.or.axis[2]);

    // OPTIMIZE: restrict the viewport on the mirrored view

    // render the mirror view
    R_RenderView(&mut newParms);

    tr.viewParms = oldParms;

    return 1; // qtrue
}

/*
=================
R_SpriteFogNum

See if a sprite is inside a fog volume
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_SpriteFogNum(ent: *mut trRefEntity_t) -> c_int {
    let mut i: c_int;
    let mut fog: *mut fog_t;

    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return 0;
    }

    if (tr.refdef.rdflags & RDF_doLAGoggles) != 0 {
        return (*tr.world).numfogs;
    }

    let mut partialFog: c_int = 0;
    i = 1;
    while i < (*tr.world).numfogs {
        fog = &mut (*tr.world).fogs.offset(i as isize);
        if (*ent).e.origin[0] - (*ent).e.radius >= (*fog).bounds[0][0]
            && (*ent).e.origin[0] + (*ent).e.radius <= (*fog).bounds[1][0]
            && (*ent).e.origin[1] - (*ent).e.radius >= (*fog).bounds[0][1]
            && (*ent).e.origin[1] + (*ent).e.radius <= (*fog).bounds[1][1]
            && (*ent).e.origin[2] - (*ent).e.radius >= (*fog).bounds[0][2]
            && (*ent).e.origin[2] + (*ent).e.radius <= (*fog).bounds[1][2] {
            //totally inside it
            return i;
        }
        if ((*ent).e.origin[0] - (*ent).e.radius >= (*fog).bounds[0][0] && (*ent).e.origin[1] - (*ent).e.radius >= (*fog).bounds[0][1] && (*ent).e.origin[2] - (*ent).e.radius >= (*fog).bounds[0][2] &&
            (*ent).e.origin[0] - (*ent).e.radius <= (*fog).bounds[1][0] && (*ent).e.origin[1] - (*ent).e.radius <= (*fog).bounds[1][1] && (*ent).e.origin[2] - (*ent).e.radius <= (*fog).bounds[1][2]) ||
            ((*ent).e.origin[0] + (*ent).e.radius >= (*fog).bounds[0][0] && (*ent).e.origin[1] + (*ent).e.radius >= (*fog).bounds[0][1] && (*ent).e.origin[2] + (*ent).e.radius >= (*fog).bounds[0][2] &&
            (*ent).e.origin[0] + (*ent).e.radius <= (*fog).bounds[1][0] && (*ent).e.origin[1] + (*ent).e.radius <= (*fog).bounds[1][1] && (*ent).e.origin[2] + (*ent).e.radius <= (*fog).bounds[1][2]) {
            //partially inside it
            if tr.refdef.fogIndex == i || R_FogParmsMatch(tr.refdef.fogIndex, i) != 0 {
                //take new one only if it's the same one that the viewpoint is in
                return i;
            } else if partialFog == 0 {
                //first partialFog
                partialFog = i;
            }
        }

        i += 1;
    }

    return partialFog;
}

/*
==========================================================================================

DRAWSURF SORTING

==========================================================================================
*/

/*
=================
qsort replacement

=================
*/
// SWAP_DRAW_SURF macro translated to inline code
macro_rules! SWAP_DRAW_SURF {
    ($a:expr, $b:expr, $temp:expr) => {
        $temp = unsafe { *(($a as *mut i32).offset(0)) };
        unsafe { *(($a as *mut i32).offset(0)) = *(($b as *mut i32).offset(0)) };
        unsafe { *(($b as *mut i32).offset(0)) = $temp };
        $temp = unsafe { *(($a as *mut i32).offset(1)) };
        unsafe { *(($a as *mut i32).offset(1)) = *(($b as *mut i32).offset(1)) };
        unsafe { *(($b as *mut i32).offset(1)) = $temp };
    };
}

/* this parameter defines the cutoff between using quick sort and
   insertion sort for arrays; arrays with lengths shorter or equal to the
   below value use insertion sort */

unsafe fn shortsort(lo: *mut drawSurf_t, hi: *mut drawSurf_t) {
    let mut p: *mut drawSurf_t;
    let mut max: *mut drawSurf_t;
    let mut temp: c_int;

    while hi > lo {
        max = lo;
        p = lo.offset(1);
        while p <= hi {
            if (*p).sort > (*max).sort {
                max = p;
            }
            p = p.offset(1);
        }
        let temp_mut = &mut temp;
        SWAP_DRAW_SURF!(max, hi, *temp_mut);
        hi = hi.offset(-1);
    }
}


/* sort the array between lo and hi (inclusive)
FIXME: this was lifted and modified from the microsoft lib source...
 */

#[no_mangle]
pub unsafe extern "C" fn qsortFast(
    base: *mut c_void,
    num: u32,
    width: u32
    ) {
    let mut lo: *mut u8 = base as *mut u8;
    let mut hi: *mut u8;              /* ends of sub-array currently sorting */
    let mut mid: *mut u8;                  /* points to middle of subarray */
    let mut loguy: *mut u8;
    let mut higuy: *mut u8;        /* traveling pointers for partition step */
    let mut size: u32;              /* size of the sub-array */
    let mut lostk: [*mut u8; 30] = [core::ptr::null_mut(); 30];
    let mut histk: [*mut u8; 30] = [core::ptr::null_mut(); 30];
    let mut stkptr: c_int;                 /* stack for saving sub-array to be processed */
    let mut temp: c_int;

    if size_of::<drawSurf_t>() != 8 {
        Com_Error(ERR_DROP, b"change SWAP_DRAW_SURF macro\0" as *const u8);
    }

    /* Note: the number of stack entries required is no more than
       1 + log2(size), so 30 is sufficient for any array */

    if num < 2 || width == 0 {
        return;                 /* nothing to do */
    }

    stkptr = 0;                 /* initialize stack */

    hi = base as *mut u8;
    hi = hi.offset((width as isize) * ((num - 1) as isize));        /* initialize limits */

    /* this entry point is for pseudo-recursion calling: setting
       lo and hi and jumping to here is like recursion, but stkptr is
       prserved, locals aren't, so we preserve stuff on the stack */
    {
        let mut lo_mut = lo;
        let mut hi_mut = hi;
        let mut stkptr_mut = stkptr;

        loop {
            size = ((hi_mut as usize - lo_mut as usize) / (width as usize)) as u32 + 1;        /* number of el's to sort */

            /* below a certain size, it is faster to use a O(n^2) sorting method */
            if size <= CUTOFF as u32 {
                shortsort(lo_mut as *mut drawSurf_t, hi_mut as *mut drawSurf_t);
            } else {
                /* First we pick a partititioning element.  The efficiency of the
                   algorithm demands that we find one that is approximately the
                   median of the values, but also that we select one fast.  Using
                   the first one produces bad performace if the array is already
                   sorted, so we use the middle one, which would require a very
                   wierdly arranged array for worst case performance.  Testing shows
                   that a median-of-three algorithm does not, in general, increase
                   performance. */

                mid = lo_mut.offset((((size / 2) * width) as isize));      /* find middle element */
                SWAP_DRAW_SURF!(mid, lo_mut, temp);               /* swap it to beginning of array */

                /* We now wish to partition the array into three pieces, one
                   consisiting of elements <= partition element, one of elements
                   equal to the parition element, and one of element >= to it.  This
                   is done below; comments indicate conditions established at every
                   step. */

                loguy = lo_mut;
                higuy = hi_mut.offset(width as isize);

                /* Note that higuy decreases and loguy increases on every iteration,
                   so loop must terminate. */
                loop {
                    /* lo <= loguy < hi, lo < higuy <= hi + 1,
                       A[i] <= A[lo] for lo <= i <= loguy,
                       A[i] >= A[lo] for higuy <= i <= hi */

                    loop {
                        loguy = loguy.offset(width as isize);
                        if !(loguy <= hi_mut &&
                            (*(loguy as *mut drawSurf_t)).sort <= (*(lo_mut as *mut drawSurf_t)).sort) {
                            break;
                        }
                    }

                    /* lo < loguy <= hi+1, A[i] <= A[lo] for lo <= i < loguy,
                       either loguy > hi or A[loguy] > A[lo] */

                    loop {
                        higuy = higuy.offset(-(width as isize));
                        if !(higuy > lo_mut &&
                            (*(higuy as *mut drawSurf_t)).sort >= (*(lo_mut as *mut drawSurf_t)).sort) {
                            break;
                        }
                    }

                    /* lo-1 <= higuy <= hi, A[i] >= A[lo] for higuy < i <= hi,
                       either higuy <= lo or A[higuy] < A[lo] */

                    if higuy < loguy {
                        break;
                    }

                    /* if loguy > hi or higuy <= lo, then we would have exited, so
                       A[loguy] > A[lo], A[higuy] < A[lo],
                       loguy < hi, highy > lo */

                    SWAP_DRAW_SURF!(loguy, higuy, temp);

                    /* A[loguy] < A[lo], A[higuy] > A[lo]; so condition at top
                       of loop is re-established */
                }

                /*     A[i] >= A[lo] for higuy < i <= hi,
                       A[i] <= A[lo] for lo <= i < loguy,
                       higuy < loguy, lo <= higuy <= hi
                   implying:
                       A[i] >= A[lo] for loguy <= i <= hi,
                       A[i] <= A[lo] for lo <= i <= higuy,
                       A[i] = A[lo] for higuy < i < loguy */

                SWAP_DRAW_SURF!(lo_mut, higuy, temp);     /* put partition element in place */

                /* OK, now we have the following:
                      A[i] >= A[higuy] for loguy <= i <= hi,
                      A[i] <= A[higuy] for lo <= i < higuy
                      A[i] = A[lo] for higuy <= i < loguy    */

                /* We've finished the partition, now we want to sort the subarrays
                   [lo, higuy-1] and [loguy, hi].
                   We do the smaller one first to minimize stack usage.
                   We only sort arrays of length 2 or more.*/

                if (higuy as usize - width as usize) as isize - lo_mut as isize >= hi_mut as isize - loguy as isize {
                    if lo_mut.offset(width as isize) < higuy {
                        lostk[stkptr_mut as usize] = lo_mut;
                        histk[stkptr_mut as usize] = higuy.offset(-(width as isize));
                        stkptr_mut += 1;
                    }                           /* save big recursion for later */

                    if loguy < hi_mut {
                        lo_mut = loguy;
                        continue;           /* do small recursion */
                    }
                } else {
                    if loguy < hi_mut {
                        lostk[stkptr_mut as usize] = loguy;
                        histk[stkptr_mut as usize] = hi_mut;
                        stkptr_mut += 1;               /* save big recursion for later */
                    }

                    if lo_mut.offset(width as isize) < higuy {
                        hi_mut = higuy.offset(-(width as isize));
                        continue;           /* do small recursion */
                    }
                }
            }

            /* We have sorted the array, except for any pending sorts on the stack.
               Check if there are any, and do them. */

            stkptr_mut -= 1;
            if stkptr_mut >= 0 {
                lo_mut = lostk[stkptr_mut as usize];
                hi_mut = histk[stkptr_mut as usize];
                continue;           /* pop subarray from stack */
            } else {
                break;                 /* all subarrays done */
            }
        }
    }
}


//==========================================================================================

/*
=================
R_AddDrawSurf
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_AddDrawSurf(surface: *const surfaceType_t, shader: *const shader_t, fogIndex: c_int, dlightMap: c_int) {
    let mut index: c_int;

    // instead of checking for overflow, we just mask the index
    // so it wraps around
    index = tr.refdef.numDrawSurfs & DRAWSURF_MASK;

    if (tr.refdef.rdflags & RDF_doLAGoggles) != 0 {
        let fogIndex_mut = fogIndex as c_int;
        // This is a mutable variable, will reassign
    }

    if (((*shader).surfaceFlags & SURF_FORCESIGHT) != 0) && ((tr.refdef.rdflags & RDF_ForceSightOn) == 0) {
        //if shader is only seen with ForceSight and we don't have ForceSight on, then don't draw
        return;
    }

    // the sort data is packed into a single 32 bit value so it can be
    // compared quickly during the qsorting process
    tr.refdef.drawSurfs[index as usize].sort = (((*shader).sortedIndex as u32) << (QSORT_SHADERNUM_SHIFT as u32))
        | (tr.shiftedEntityNum as u32) | ((fogIndex as u32) << (QSORT_FOGNUM_SHIFT as u32)) | (dlightMap as u32);
    tr.refdef.drawSurfs[index as usize].surface = surface as *mut surfaceType_t;
    tr.refdef.numDrawSurfs += 1;
}

/*
=================
R_DecomposeSort
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_DecomposeSort(sort: u32, entityNum: *mut c_int, shader: *mut *mut shader_t,
                        fogNum: *mut c_int, dlightMap: *mut c_int) {
    *fogNum = ((sort >> (QSORT_FOGNUM_SHIFT as u32)) & 31) as c_int;
    *shader = tr.sortedShaders[((sort >> (QSORT_SHADERNUM_SHIFT as u32)) & ((MAX_SHADERS - 1) as u32)) as usize];
    *entityNum = ((sort >> (QSORT_ENTITYNUM_SHIFT as u32)) & ((MAX_ENTITIES - 1) as u32)) as c_int;
    *dlightMap = (sort & 3) as c_int;
}

/*
=================
R_SortDrawSurfs
=================
*/
#[no_mangle]
pub unsafe extern "C" fn R_SortDrawSurfs(drawSurfs: *mut drawSurf_t, numDrawSurfs: c_int) {
    let mut shader: *mut shader_t;
    let mut fogNum: c_int;
    let mut entityNum: c_int;
    let mut dlighted: c_int;

    // it is possible for some views to not have any surfaces
    if numDrawSurfs < 1 {
        // we still need to add it for hyperspace cases
        R_AddDrawSurfCmd(drawSurfs, numDrawSurfs);
        return;
    }

    // if we overflowed MAX_DRAWSURFS, the drawsurfs
    // wrapped around in the buffer and we will be missing
    // the first surfaces, not the last ones
    let mut numDrawSurfs_mut = numDrawSurfs;
    if numDrawSurfs_mut > MAX_DRAWSURFS {
        numDrawSurfs_mut = MAX_DRAWSURFS;
    }

    // sort the drawsurfs by sort type, then orientation, then shader
    qsortFast(drawSurfs as *mut c_void, numDrawSurfs_mut as u32, size_of::<drawSurf_t>() as u32);

    // check for any pass through drawing, which
    // may cause another view to be rendered first
    let mut i: c_int = 0;
    while i < numDrawSurfs_mut {
        R_DecomposeSort((*drawSurfs.offset(i as isize)).sort, &mut entityNum, &mut shader, &mut fogNum, &mut dlighted);

        if (*shader).sort > SS_PORTAL {
            break;
        }

        // no shader should ever have this sort type
        if (*shader).sort == SS_BAD {
            Com_Error(ERR_DROP, b"Shader '%s'with sort == SS_BAD\0" as *const u8, (*shader).name.as_ptr() as *const u8);
        }

        // if the mirror was completely clipped away, we may need to check another surface
        if R_MirrorViewBySurface(&mut (*drawSurfs.offset(i as isize)), entityNum) != 0 {
            // this is a debug option to see exactly what is being mirrored
            if (*get_r_portalOnly()).integer != 0 {
                return;
            }
            break;		// only one mirror view at a time
        }

        i += 1;
    }

    R_AddDrawSurfCmd(drawSurfs, numDrawSurfs_mut);
}

/*
=============
R_AddEntitySurfaces
=============
*/
#[no_mangle]
pub unsafe extern "C" fn R_AddEntitySurfaces() {
    let mut ent: *mut trRefEntity_t;
    let mut shader: *mut shader_t;

    if (*get_r_drawentities()).integer == 0 {
        return;
    }

    tr.currentEntityNum = 0;
    while tr.currentEntityNum < tr.refdef.num_entities {
        ent = &mut tr.refdef.entities[tr.currentEntityNum as usize];
        tr.currentEntity = ent;

        (*ent).needDlights = 0; // qfalse

        // preshift the value we are going to OR into the drawsurf sort
        tr.shiftedEntityNum = tr.currentEntityNum << QSORT_ENTITYNUM_SHIFT;

        if (((*ent).e.renderfx & RF_ALPHA_FADE)) != 0 {
            // we need to make sure this is not sorted before the world..in fact we
            // want this to be sorted quite late...like how about last.
            // I don't want to use the highest bit, since no doubt someone fumbled
            // handling that as an unsigned quantity somewhere
            tr.shiftedEntityNum |= 0x80000000i32;
        }
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
        },
        RT_SPRITE | RT_ORIENTED_QUAD | RT_BEAM | RT_CYLINDER | RT_LATHE |
        RT_CLOUDS | RT_LINE | RT_ELECTRICITY | RT_SABER_GLOW => {
            // self blood sprites, talk balloons, etc should not be drawn in the primary
            // view.  We can't just do this check for all entities, because md3
            // entities may still want to cast shadows from them
            if (((*ent).e.renderfx & RF_THIRD_PERSON) != 0) && !(tr.viewParms.isPortal != 0) {
                tr.currentEntityNum += 1;
                continue;
            }
            shader = R_GetShaderByHandle((*ent).e.customShader);
            R_AddDrawSurf(&entitySurface as *const c_int as *const surfaceType_t, shader, R_SpriteFogNum(ent), 0);
        },

        RT_MODEL => {
            // we must set up parts of tr.or for model culling
            R_RotateForEntity(ent, &tr.viewParms, &mut tr.or);

            tr.currentModel = R_GetModelByHandle((*ent).e.hModel);
            if tr.currentModel.is_null() {
                R_AddDrawSurf(&entitySurface as *const c_int as *const surfaceType_t, tr.defaultShader, 0, 0);
            } else {
                match (*tr.currentModel).r#type {
                MOD_MESH => {
                    R_AddMD3Surfaces(ent);
                },
                MOD_BRUSH => {
                    R_AddBrushModelSurfaces(ent);
                },
                /*
Ghoul2 Insert Start
*/

                MOD_MDXM => {
                    R_AddGhoulSurfaces(ent);
                },
                MOD_BAD => {		// null model axis
                    if (((*ent).e.renderfx & RF_THIRD_PERSON) != 0) && !(tr.viewParms.isPortal != 0) {
                        if (((*ent).e.renderfx & RF_SHADOW_ONLY) == 0) {
                            // empty
                        }
                    }

                    if !(*ent).e.ghoul2.is_null() && G2API_HaveWeGhoul2Models(*((*ent).e.ghoul2 as *const c_void)) != 0 {
                        R_AddGhoulSurfaces(ent);
                    } else {
                        R_AddDrawSurf(&entitySurface as *const c_int as *const surfaceType_t, tr.defaultShader, 0, 0);
                    }
                },
                /*
Ghoul2 Insert End
*/

                _ => {
                    Com_Error(ERR_DROP, b"R_AddEntitySurfaces: Bad modeltype\0" as *const u8);
                },
                }
            }
        },
        _ => {
            Com_Error(ERR_DROP, b"R_AddEntitySurfaces: Bad reType\0" as *const u8);
        },
        }

        tr.currentEntityNum += 1;
    }
}


/*
====================
R_GenerateDrawSurfs
====================
*/
#[no_mangle]
pub unsafe extern "C" fn R_GenerateDrawSurfs() {
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

/*
================
R_DebugPolygon
================
*/
#[no_mangle]
pub unsafe extern "C" fn R_DebugPolygon(color: c_int, numPoints: c_int, points: *mut f32) {
    let mut i: c_int;

    GL_State(GLS_DEPTHMASK_TRUE | GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE);

    // draw solid shade

    let qglColor3f: extern "C" fn(f32, f32, f32) = core::mem::transmute(0usize); // stub
    let qglBegin: extern "C" fn(c_int) = core::mem::transmute(0usize); // stub
    let qglVertex3fv: extern "C" fn(*mut f32) = core::mem::transmute(0usize); // stub
    let qglEnd: extern "C" fn() = core::mem::transmute(0usize); // stub
    let qglDepthRange: extern "C" fn(f32, f32) = core::mem::transmute(0usize); // stub

    qglColor3f(
        ((color & 1) as f32),
        (((color >> 1) & 1) as f32),
        (((color >> 2) & 1) as f32)
    );
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

/*
====================
R_DebugGraphics

Visualization aid for movement clipping debugging
====================
*/
#[no_mangle]
pub unsafe extern "C" fn R_DebugGraphics() {
    if (*get_r_debugSurface()).integer == 0 {
        return;
    }

    // the render thread can't make callbacks to the main thread
    //R_SyncRenderThread();

    GL_Bind(core::ptr::null_mut()); // tr.whiteImage stub
    GL_Cull(CT_FRONT_SIDED);
    CM_DrawDebugSurface(R_DebugPolygon);
}

#[no_mangle]
pub unsafe extern "C" fn R_FogParmsMatch(fog1: c_int, fog2: c_int) -> c_int {
    let mut i: c_int = 0;
    while i < 2 {
        if (*tr.world).fogs[fog1 as usize].parms.color[i as usize] != (*tr.world).fogs[fog2 as usize].parms.color[i as usize] {
            return 0; // qfalse
        }
        i += 1;
    }
    return 1; // qtrue
}

#[no_mangle]
pub unsafe extern "C" fn R_SetViewFogIndex() {
    if (*tr.world).numfogs > 1 {
        //more than just the LA goggles
        let mut fog: *mut fog_t;
        let contents: c_int = SV_PointContents(&tr.refdef.vieworg, 0);
        if (contents & CONTENTS_FOG) != 0 {
            //only take a tr.refdef.fogIndex if the tr.refdef.vieworg is actually *in* that fog brush (assumption: checks pointcontents for any CONTENTS_FOG, not that particular brush...)
            tr.refdef.fogIndex = 1;
            while tr.refdef.fogIndex < (*tr.world).numfogs {
                fog = &mut (*tr.world).fogs[tr.refdef.fogIndex as usize];
                if tr.refdef.vieworg[0] >= (*fog).bounds[0][0]
                    && tr.refdef.vieworg[1] >= (*fog).bounds[0][1]
                    && tr.refdef.vieworg[2] >= (*fog).bounds[0][2]
                    && tr.refdef.vieworg[0] <= (*fog).bounds[1][0]
                    && tr.refdef.vieworg[1] <= (*fog).bounds[1][1]
                    && tr.refdef.vieworg[2] <= (*fog).bounds[1][2] {
                    break;
                }
                tr.refdef.fogIndex += 1;
            }
            if tr.refdef.fogIndex == (*tr.world).numfogs {
                tr.refdef.fogIndex = 0;
            }
        } else {
            tr.refdef.fogIndex = 0;
        }
    } else {
        tr.refdef.fogIndex = 0;
    }
}

extern "C" {
    pub fn RE_SetLightStyle(style: c_int, colors: c_int);
}

/*
================
R_RenderView

A view may be either the actual camera view,
or a mirror / remote location
================
*/
#[no_mangle]
pub unsafe extern "C" fn R_RenderView(parms: *mut viewParms_t) {
    let mut firstDrawSurf: c_int;

    if (*parms).viewportWidth <= 0 || (*parms).viewportHeight <= 0 {
        return;
    }

    if (*get_r_debugStyle()).integer >= 0 {
        let mut i: c_int;
        let whitecolor: color4ub_t = color4ub_t { r: 0xff, g: 0xff, b: 0xff, a: 0xff };
        let blackcolor: color4ub_t = color4ub_t { r: 0x00, g: 0x00, b: 0x00, a: 0xff };

        i = 0;
        while i < MAX_LIGHT_STYLES {
            RE_SetLightStyle(i, *((&blackcolor as *const color4ub_t) as *const c_int));
            i += 1;
        }
        RE_SetLightStyle((*get_r_debugStyle()).integer, *((&whitecolor as *const color4ub_t) as *const c_int));
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

    if (tr.refdef.rdflags & RDF_NOWORLDMODEL) == 0 {
        // Trying to do this with no world is not good.
        R_SetViewFogIndex();
    }

    R_GenerateDrawSurfs();

    R_SortDrawSurfs(
        tr.refdef.drawSurfs.offset(firstDrawSurf as isize),
        tr.refdef.numDrawSurfs - firstDrawSurf
    );

    // draw main system development information (surface outlines, etc)
    R_DebugGraphics();
}
