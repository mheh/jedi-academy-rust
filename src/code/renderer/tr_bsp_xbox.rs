// tr_map.c

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"
// #include "tr_local.h"
// #include "../qcommon/cm_local.h"

/*

Loads and prepares a map file for scene rendering.

A single entry point:

void RE_LoadWorldMap( const char *name );

*/

use core::ffi::{c_int, c_char, c_void};

// Module-level stubs and externs - dependencies not yet ported
// These would come from tr_local.h, qcommon/cm_local.h, etc.

// Placeholder types for external dependencies
#[repr(C)]
pub struct world_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct shader_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct msurface_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct srfGridMesh_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct srfTriangles_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct srfFlare_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct drawVert_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct mapVert_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dface_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dpatch_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dtrisurf_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dflare_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dmodel_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dnode_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dleaf_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct mnode_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct mnode_s {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct mleaf_s {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dfog_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dbrush_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dbrushside_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct mgrid_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct shaderStage_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct dshader_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct fogParms_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct fog_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct cplane_t {
    // Fields will be populated when full port is complete
}

#[repr(C)]
pub struct Lump {
    // Fields will be populated when full port is complete
}

pub type qhandle_t = c_int;
pub type qboolean = c_int;
pub type vec3_t = [f32; 3];
pub type byte = u8;
pub type surfaceType_t = c_int;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub const MAX_GREYSCALE_CHANNEL_DIFF: c_int = 15;
pub const LIGHTMAP_SIZE: c_int = 128;
pub const MAX_QPATH: usize = 256;
pub const MAXLIGHTMAPS: usize = 4;
pub const MAX_FACE_POINTS: c_int = 128;
pub const MAX_PATCH_SIZE: c_int = 32;
pub const MAX_GRID_SIZE: c_int = 129;
pub const VERTEX_LM: usize = 8;
pub const DRAWVERT_ST_SCALE: f32 = 1024.0;
pub const POINTS_ST_SCALE: f32 = 1.0;
pub const GRID_DRAWVERT_ST_SCALE: f32 = 1024.0;
pub const POINTS_LIGHT_SCALE: f32 = 1.0;
pub const DRAWVERT_LIGHTMAP_SCALE: f32 = 1.0;
pub const TAG_TEMP_WORKSPACE: c_int = 6;
pub const MIN_WORLD_COORD: f32 = -65536.0;
pub const MAX_WORLD_COORD: f32 = 65536.0;
pub const CONTENTS_NODE: c_int = -1;
pub const SF_FACE: c_int = 1;
pub const SF_SKIP: c_int = 4;
pub const SF_TRIANGLES: c_int = 5;
pub const SF_FLARE: c_int = 6;
pub const GL_DDS_RGB16_EXT: c_int = 0;
pub const GL_CLAMP: c_int = 0;
pub const CGEN_EXACT_VERTEX: c_int = 5;
pub const CGEN_VERTEX: c_int = 6;
pub const CGEN_ONE_MINUS_VERTEX: c_int = 7;
pub const AGEN_VERTEX: c_int = 3;
pub const AGEN_ONE_MINUS_VERTEX: c_int = 4;
pub const LIGHTMAP_NONE: c_int = -1;
pub const SS_PORTAL: c_int = 3;
pub const SURF_NODRAW: c_int = 0x80;
pub const ERR_DROP: c_int = 1;
pub const PRINT_DEVELOPER: c_int = 0;
pub const PRINT_ALL: c_int = 1;
pub const PRINT_WARNING: c_int = 2;
pub const MAX_TOKEN_CHARS: usize = 1024;

// Globals
pub static mut s_worldData: world_t = unsafe { core::mem::zeroed() };
pub static mut fileBase: *mut byte = core::ptr::null_mut();
pub static mut c_subdivisions: c_int = 0;
pub static mut c_gridVerts: c_int = 0;

pub static mut skyboxportal: c_int = 0;

// External C functions - these would be declared from other modules
extern "C" {
    pub fn R_SyncRenderThread();
    pub fn Com_Error(error_code: c_int, format: *const c_char, ...);
    pub fn Com_Printf(format: *const c_char, ...);
    pub fn VID_Printf(print_type: c_int, format: *const c_char, ...);
    pub fn Hunk_Alloc(size: usize, zero: qboolean) -> *mut c_void;
    pub fn Z_Malloc(size: usize, tag: c_int, zero: qboolean, alignment: c_int) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn RE_RegisterShader(name: *const c_char) -> qhandle_t;
    pub fn R_CreateImage(
        name: *const c_char,
        pic: *mut byte,
        width: c_int,
        height: c_int,
        format: c_int,
        mipmap: qboolean,
        filter: c_int,
        wrap: c_int,
    ) -> qhandle_t;
    pub fn R_FindShader(
        name: *const c_char,
        lightmap_num: *const c_int,
        light_map_styles: *const byte,
        force_load: qboolean,
    ) -> *mut shader_t;
    pub fn COM_ParseExt(data_p: *mut *const c_char, allow_newlines: qboolean) -> *const c_char;
    pub fn COM_StripExtension(in_: *const c_char, out: *mut c_char);
    pub fn COM_SkipPath(pathname: *const c_char) -> *const c_char;
    pub fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_CastShort2FloatScale(dest: *mut f32, src: *const i16, scale: f32);
    pub fn CrossProduct(v1: &vec3_t, v2: &vec3_t, cross: &mut vec3_t);
    pub fn DotProduct(v1: &vec3_t, v2: &vec3_t) -> f32;
    pub fn VectorNormalizeFast(v: &mut vec3_t) -> f32;
    pub fn VectorNormalize(v: &mut vec3_t) -> f32;
    pub fn VectorAdd(veca: &vec3_t, vecb: &vec3_t, out: &mut vec3_t);
    pub fn VectorSubtract(veca: &vec3_t, vecb: &vec3_t, out: &mut vec3_t);
    pub fn VectorScale(v: &vec3_t, scale: f32, out: &mut vec3_t);
    pub fn VectorSet(v: &mut vec3_t, x: f32, y: f32, z: f32);
    pub fn VectorLength(v: &vec3_t) -> f32;
    pub fn ClearBounds(mins: &mut vec3_t, maxs: &mut vec3_t);
    pub fn AddPointToBounds(v: &vec3_t, mins: &mut vec3_t, maxs: &mut vec3_t);
    pub fn SetPlaneSignbits(plane: *mut cplane_t);
    pub fn PlaneTypeForNormal(normal: &vec3_t) -> c_int;
    pub fn R_SubdividePatchToGrid(
        width: c_int,
        height: c_int,
        points: *mut drawVert_t,
        ctrl: *mut drawVert_t,
        error_table: *mut f32,
    ) -> *mut srfGridMesh_t;
    pub fn R_AllocModel() -> *mut core::ffi::c_void;
    pub fn RE_InsertModelIntoHash(name: *const c_char, model: *mut core::ffi::c_void);
    pub fn Cvar_SetValue(var_name: *const c_char, value: f32);
    pub fn R_LoadLevelLightParms();
    pub fn R_GetLightParmsForLevel();
    pub fn R_RMGInit();

    pub static mut tr: core::ffi::c_void; // placeholder for tr global
    pub static mut cmg: core::ffi::c_void; // placeholder for cmg global
    pub static mut r_singleShader: *mut core::ffi::c_void; // placeholder for cvar
}

// Helper macros/constants
fn max(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

fn min(a: c_int, b: c_int) -> c_int {
    if a < b { a } else { b }
}

fn NEXT_SURFPOINT(flags: c_int) -> usize {
    (VERTEX_LM + ((flags & 0x7F) as usize) * 2) as usize
}

fn VERTEX_COLOR(flags: c_int) -> usize {
    (VERTEX_LM + ((flags & 0x7F) as usize) * 2) as usize
}

fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> c_int {
    let ir = (r as c_int) & 0xFF;
    let ig = (g as c_int) & 0xFF;
    let ib = (b as c_int) & 0xFF;
    let ia = (a as c_int) & 0xFF;
    (ia << 24) | (ib << 16) | (ig << 8) | ir
}

// We use a special hack to prevent slight differences in channels
// from exploding into big differences, as it causes lighting problems
// later on. This is the maximum channel separation for which we
// enable the hack.

unsafe fn R_ColorShiftLightingBytes16(in_: &[byte; 4], out: &mut [byte; 2]) {
    // What's the largest separation between the red, green, and blue
    // channels?
    let chan_diff = max(in_[0] as c_int, max(in_[1] as c_int, in_[2] as c_int))
        - min(in_[0] as c_int, min(in_[1] as c_int, in_[2] as c_int));

    if chan_diff <= MAX_GREYSCALE_CHANNEL_DIFF {
        // Ensure that all color channels compress to the same value
        let channel_avg = ((in_[0] as c_int) + (in_[1] as c_int) + (in_[2] as c_int) + 1) / 3;
        out[0] = (channel_avg & 0xF0) as byte;
        out[0] |= ((channel_avg & 0xF0) >> 4) as byte;
        out[1] = (channel_avg & 0xF0) as byte;
        out[1] |= ((in_[3] as c_int & 0xF0) >> 4) as byte;

        if channel_avg % 16 >= 8 {
            out[0] |= 0x10;
            out[0] |= 0x01;
            out[1] |= 0x10;
        }
        if (in_[4] as c_int) % 16 >= 8 {
            out[1] |= 0x01;
        }
        return;
    }

    // Normal case for vertex colors that are not "near" greyscale
    out[0] = (in_[0] as c_int & 0xF0) as byte;
    out[0] |= ((in_[1] as c_int & 0xF0) >> 4) as byte;
    out[1] = (in_[2] as c_int & 0xF0) as byte;
    out[1] |= ((in_[3] as c_int & 0xF0) >> 4) as byte;

    if (in_[0] as c_int) % 16 >= 8 {
        out[0] |= 0x10;
    }
    if (in_[1] as c_int) % 16 >= 8 {
        out[0] |= 0x1;
    }
    if (in_[2] as c_int) % 16 >= 8 {
        out[1] |= 0x10;
    }
    if (in_[3] as c_int) % 16 >= 8 {
        out[1] |= 0x1;
    }
}

unsafe fn HSVtoRGB(h: f32, s: f32, v: f32, rgb: &mut [f32; 3]) {
    let mut i: c_int;
    let mut f: f32;
    let mut p: f32;
    let mut q: f32;
    let mut t: f32;

    let mut h_mut = h * 5.0;

    i = h_mut.floor() as c_int;
    f = h_mut - (i as f32);

    p = v * (1.0 - s);
    q = v * (1.0 - s * f);
    t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => {
            rgb[0] = v;
            rgb[1] = t;
            rgb[2] = p;
        }
        1 => {
            rgb[0] = q;
            rgb[1] = v;
            rgb[2] = p;
        }
        2 => {
            rgb[0] = p;
            rgb[1] = v;
            rgb[2] = t;
        }
        3 => {
            rgb[0] = p;
            rgb[1] = q;
            rgb[2] = v;
        }
        4 => {
            rgb[0] = t;
            rgb[1] = p;
            rgb[2] = v;
        }
        5 => {
            rgb[0] = v;
            rgb[1] = p;
            rgb[2] = q;
        }
        _ => {}
    }
}

/*
===============
R_ColorShiftLightingBytes

===============
*/
unsafe fn R_ColorShiftLightingBytes_4byte(in_: &[byte; 4], out: &mut [byte; 4]) {
    let mut shift: c_int = 0;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;

    // should NOT do it if overbrightBits is 0
    // (tr.overbrightBits check would go here - accessing tr global)
    // shift = 1 - tr.overbrightBits;

    if shift == 0 {
        out[0] = in_[0];
        out[1] = in_[1];
        out[2] = in_[2];
        out[3] = in_[3];
        return;
    }

    // shift the data based on overbright range
    r = (in_[0] as c_int) << shift;
    g = (in_[1] as c_int) << shift;
    b = (in_[2] as c_int) << shift;

    // normalize by color instead of saturating to white
    if ((r | g | b) > 255) {
        let mut max_val: c_int;

        max_val = if r > g { r } else { g };
        max_val = if max_val > b { max_val } else { b };
        r = r * 255 / max_val;
        g = g * 255 / max_val;
        b = b * 255 / max_val;
    }

    out[0] = r as byte;
    out[1] = g as byte;
    out[2] = b as byte;
    out[3] = in_[3];
}

/*
===============
R_ColorShiftLightingBytes

===============
*/
unsafe fn R_ColorShiftLightingBytes_3byte(in_: &mut [byte; 3]) {
    let mut shift: c_int = 0;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;

    // should NOT do it if overbrightBits is 0
    // shift = 1 - tr.overbrightBits;

    if shift == 0 {
        return; // no need if not overbright
    }
    // shift the data based on overbright range
    r = (in_[0] as c_int) << shift;
    g = (in_[1] as c_int) << shift;
    b = (in_[2] as c_int) << shift;

    // normalize by color instead of saturating to white
    if ((r | g | b) > 255) {
        let mut max_val: c_int;

        max_val = if r > g { r } else { g };
        max_val = if max_val > b { max_val } else { b };
        r = r * 255 / max_val;
        g = g * 255 / max_val;
        b = b * 255 / max_val;
    }

    in_[0] = r as byte;
    in_[1] = g as byte;
    in_[2] = b as byte;
}

/*
===============
R_LoadLightmaps

===============
*/
unsafe fn R_LoadLightmaps(data: *mut c_void, len: c_int, ps_map_name: *const c_char) {
    let mut buf: *mut byte;
    let mut buf_p: *mut byte;
    let mut i: c_int;

    if len == 0 {
        return;
    }
    buf = (data as *mut byte).offset(core::mem::size_of::<c_int>() as isize);

    // we are about to upload textures
    R_SyncRenderThread();

    // create all the lightmaps
    let size = *(data as *mut c_int) as usize;
    // tr.numLightmaps = len as usize / size;

    let image = Z_Malloc(size, TAG_TEMP_WORKSPACE, qfalse, 32) as *mut byte;

    let mut s_map_name: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    COM_StripExtension(ps_map_name, s_map_name.as_mut_ptr()); // will already by MAX_QPATH legal, so no length check

    let num_lightmaps = len as usize / size;
    for i in 0..num_lightmaps as c_int {
        buf_p = buf.offset((i as usize * size) as isize);
        memcpy(image as *mut c_void, buf_p as *const c_void, size);

        let mut lmap_name: [c_char; MAX_QPATH + 32] = [0; MAX_QPATH + 32];
        Com_sprintf(
            lmap_name.as_mut_ptr(),
            MAX_QPATH + 32,
            "*%s/lightmap%d\0".as_ptr() as *const c_char,
            s_map_name.as_ptr(),
            i,
        );
        // tr.lightmaps[i] = R_CreateImage(
        //     lmap_name.as_ptr(),
        //     image,
        //     LIGHTMAP_SIZE,
        //     LIGHTMAP_SIZE,
        //     GL_DDS_RGB16_EXT,
        //     qfalse,
        //     0,
        //     GL_CLAMP,
        // );
    }

    Z_Free(image as *mut c_void);
}

/*
=================
RE_SetWorldVisData

This is called by the clipmodel subsystem so we can share the 1.8 megs of
space in big maps...
=================
*/
pub unsafe fn RE_SetWorldVisData(vis: *mut byte) {
    // tr.externalVisData = vis;
}

/*
=================
R_LoadVisibility
=================
*/
unsafe fn R_LoadVisibility() {
    let mut len: c_int;

    len = (((&s_worldData).num_clusters + 63) & !63) as c_int;
    let novis = Hunk_Alloc(len as usize, qfalse) as *mut u8;
    memset(novis as *mut c_void, 0xff, len as usize);

    // s_worldData.numClusters = cmg.numClusters;
    // s_worldData.clusterBytes = cmg.clusterBytes;

    // CM_Load should have given us the vis data to share, so
    // we don't need to allocate another copy
    // if ( tr.externalVisData ) {
    //     s_worldData.vis = tr.externalVisData;
    // } else {
    //     assert(0);
    // }
}

//===============================================================================

pub unsafe fn R_GetShaderByNum(shader_num: c_int, world_data: &world_t) -> qhandle_t {
    let mut shader: qhandle_t;

    if (shader_num < 0) || (shader_num >= 0) {
        // (shaderNum >= worldData.numShaders) check would go here
        Com_Printf("Warning: Bad index for R_GetShaderByNum - %i\0".as_ptr() as *const c_char, shader_num);
        return 0;
    }
    // shader = RE_RegisterShader(worldData.shaders[ shaderNum ].shader);
    // return shader;
    return 0; // placeholder
}

/*
===============
ShaderForShaderNum
===============
*/
unsafe fn ShaderForShaderNum(
    shader_num: c_int,
    lightmap_num: *const c_int,
    lightmap_styles: *const byte,
) -> *mut shader_t {
    let mut shader: *mut shader_t;
    // let mut dsh: *mut dshader_t;

    // shaderNum = shaderNum;
    if shader_num < 0 || shader_num >= 0 {
        // (shaderNum >= s_worldData.numShaders)
        Com_Error(
            ERR_DROP,
            "ShaderForShaderNum: bad num %i\0".as_ptr() as *const c_char,
            shader_num,
        );
    }
    // dsh = &s_worldData.shaders[ shaderNum ];

    shader = R_FindShader(
        core::ptr::null(),
        lightmap_num,
        lightmap_styles,
        qtrue,
    );

    // if the shader had errors, just use default shader
    // if ( shader->defaultShader ) {
    //     return tr.defaultShader;
    // }

    return shader;
}

unsafe fn NeedVertexColors(shader: *mut shader_t) -> bool {
    let mut i: c_int;
    // let stage: *mut shaderStage_t;

    for i in 0..0 {
        // for(i=0; i<shader->numUnfoggedPasses; i++)
        // stage = &shader->stages[i];
        // match(stage->rgbGen)
        // case CGEN_EXACT_VERTEX:
        // case CGEN_VERTEX:
        // case CGEN_ONE_MINUS_VERTEX:
        //     return true;
        // }
        // match(stage->alphaGen)
        // case AGEN_VERTEX:
        // case AGEN_ONE_MINUS_VERTEX:
        //     return true;
        // }
    }

    return false;
}

unsafe fn NumLightMaps(shader: *mut shader_t) -> c_int {
    let mut count: c_int = 0;
    let mut i: c_int;

    for i in 0..MAXLIGHTMAPS as c_int {
        // if(shader->lightmapIndex[i] >= 0) {
        //     count++;
        // } else {
        //     return count;
        // }
    }

    return count;
}

unsafe fn SurfaceFaceSize(
    num_verts: c_int,
    num_light_maps: c_int,
    need_vertex_colors: bool,
    num_indexes: c_int,
) -> c_int {
    let mut sface_size: c_int;

    sface_size = 0; // placeholder calculation
    // (int)&((srfSurfaceFace_t *)0)->srfPoints + 4 + (numVerts * sizeof(unsigned short) * ...)

    // Add in tangent size
    sface_size += (core::mem::size_of::<vec3_t>() as c_int) * num_verts;

    // Indices stored in 8 bits now.
    sface_size += num_indexes;

    return sface_size;
}

unsafe fn BuildDrawVertTangents(verts: *mut drawVert_t, indexes: *mut c_int, num_indexes: c_int, num_vertexes: c_int) {
    let mut i: c_int = 0;

    for i in 0..num_vertexes {
        // verts[i].tangent[0] = 0.0f;
        // verts[i].tangent[1] = 0.0f;
        // verts[i].tangent[2] = 0.0f;
    }

    for i in (0..num_indexes).step_by(3) {
        let mut vec1: vec3_t = [0.0; 3];
        let mut vec2: vec3_t = [0.0; 3];
        let mut du: vec3_t = [0.0; 3];
        let mut dv: vec3_t = [0.0; 3];
        let mut cp: vec3_t = [0.0; 3];
        let mut st0: [f32; 2] = [0.0; 2];
        let mut st1: [f32; 2] = [0.0; 2];
        let mut st2: [f32; 2] = [0.0; 2];

        // Q_CastShort2FloatScale(&st0[0], &verts[indexes[i]].dvst[0], 1.f / DRAWVERT_ST_SCALE);
        // ... tangent calculation code would go here
    }

    for i in 0..num_vertexes {
        // VectorNormalizeFast(verts[i].tangent);
    }
}

unsafe fn BuildMapVertTangents(
    verts: *mut mapVert_t,
    tangents: *mut vec3_t,
    indexes: *mut i16,
    num_indexes: c_int,
    num_vertexes: c_int,
) {
    let mut i: c_int = 0;

    for i in 0..num_vertexes {
        (*tangents.offset(i as isize))[0] = 0.0;
        (*tangents.offset(i as isize))[1] = 0.0;
        (*tangents.offset(i as isize))[2] = 0.0;
    }

    for i in (0..num_indexes).step_by(3) {
        let mut vec1: vec3_t = [0.0; 3];
        let mut vec2: vec3_t = [0.0; 3];
        let mut du: vec3_t = [0.0; 3];
        let mut dv: vec3_t = [0.0; 3];
        let mut cp: vec3_t = [0.0; 3];

        // Tangent calculation would go here
        // (accessing verts and indexes arrays)
    }

    for i in 0..num_vertexes {
        // VectorNormalizeFast(tangents[i]);
    }
}

/*
===============
ParseFace
===============
*/
unsafe fn ParseFace(
    ds: *mut dface_t,
    verts: *mut mapVert_t,
    surf: *mut msurface_t,
    indexes: *mut i16,
    p_face_data_buffer: &mut *mut byte,
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    // let cv: *mut srfSurfaceFace_t;
    let mut num_points: c_int;
    let mut num_indexes: c_int;
    let mut lightmap_num: [c_int; MAXLIGHTMAPS] = [0; MAXLIGHTMAPS];
    // let sface_size: c_int;
    // let ofs_indexes: c_int;
    let mut tangents: [vec3_t; 1000] = [[0.0; 3]; 1000];

    for i in 0..MAXLIGHTMAPS as c_int {
        // lightmapNum[i] = (int)ds->lightmapNum[i] - 4;
    }

    // get fog volume
    // surf->fogIndex = ds->fogNum + 1;

    // get shader value
    // surf->shader = ShaderForShaderNum( ds->shaderNum, lightmapNum, ds->lightmapStyles );
    // if ( r_singleShader->integer && !surf->shader->sky ) {
    //     surf->shader = tr.defaultShader;
    // }

    // let need_vertex_colors = NeedVertexColors(surf->shader);
    // let num_light_maps = NumLightMaps(surf->shader);
    // assert(numLightMaps <= 0x7F);

    // numPoints = ds->verts & 0xFFF;
    // if (numPoints > MAX_FACE_POINTS) {
    //     VID_Printf( PRINT_DEVELOPER, "MAX_FACE_POINTS exceeded: %i\n", numPoints);
    // }

    // numIndexes = ds->indexes & 0xFFF;

    // create the srfSurfaceFace_t
    // sfaceSize = SurfaceFaceSize(numPoints, numLightMaps, needVertexColors, numIndexes);
    // ofsIndexes = sfaceSize - numIndexes;

    // cv = (srfSurfaceFace_t *) pFaceDataBuffer;
    // pFaceDataBuffer += sfaceSize;

    // ... rest of parsing code would go here
}

/*
===============
ParseMesh
===============
*/
unsafe fn ParseMesh(
    ds: *mut dpatch_t,
    verts: *mut mapVert_t,
    surf: *mut msurface_t,
    points: *mut drawVert_t,
    ctrl: *mut drawVert_t,
    error_table: *mut f32,
) {
    // let grid: *mut srfGridMesh_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut width: c_int;
    let mut height: c_int;
    let mut num_points: c_int;
    let mut lightmap_num: [c_int; MAXLIGHTMAPS] = [0; MAXLIGHTMAPS];
    let mut bounds: [vec3_t; 2] = [[0.0; 3]; 2];
    let mut tmp_vec: vec3_t = [0.0; 3];

    for i in 0..MAXLIGHTMAPS as c_int {
        // lightmapNum[i] = (int)ds->lightmapNum[i] - 4;
    }

    // get fog volume
    // surf->fogIndex = ds->fogNum + 1;

    // get shader value
    // surf->shader = ShaderForShaderNum( ds->shaderNum, lightmapNum, ds->lightmapStyles );
    // if ( r_singleShader->integer && !surf->shader->sky ) {
    //     surf->shader = tr.defaultShader;
    // }

    // we may have a nodraw surface, because they might still need to
    // be around for movement clipping
    // if ( s_worldData.shaders[ ds->shaderNum ].surfaceFlags & SURF_NODRAW ) {
    //     surf->data = &skipData;
    //     return;
    // }

    // width = ds->patchWidth;
    // height = ds->patchHeight;

    // verts += ds->verts >> 12;
    // numPoints = width * height;
    // for ( i = 0 ; i < numPoints ; i++ ) {
    //     ... vertex copy code
    // }

    // pre-tesseleate
    // grid = R_SubdividePatchToGrid( width, height, points, ctrl, errorTable );
    // surf->data = (surfaceType_t *)grid;

    // ... rest of mesh loading code
}

/*
===============
ParseTriSurf
===============
*/
unsafe fn ParseTriSurf(
    ds: *mut dtrisurf_t,
    verts: *mut mapVert_t,
    surf: *mut msurface_t,
    indexes: *mut i16,
) {
    // let tri: *mut srfTriangles_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut num_verts: c_int;
    let mut num_indexes: c_int;

    // get fog volume
    // surf->fogIndex = ds->fogNum + 1;

    // get shader
    // surf->shader = ShaderForShaderNum( ds->shaderNum, lightmapsVertex, ds->lightmapStyles );
    // if ( r_singleShader->integer && !surf->shader->sky ) {
    //     surf->shader = tr.defaultShader;
    // }

    // numVerts = ds->verts & 0xFFF;
    // numIndexes = ds->indexes & 0xFFF;

    // tri = (srfTriangles_t *) Hunk_Alloc( sizeof( *tri ) + numVerts * sizeof( tri->verts[0] )
    //     + numIndexes * sizeof( tri->indexes[0] ), qtrue );
    // tri->surfaceType = SF_TRIANGLES;
    // tri->numVerts = numVerts;
    // tri->numIndexes = numIndexes;
    // tri->verts = (drawVert_t *)(tri + 1);
    // tri->indexes = (int *)(tri->verts + tri->numVerts );

    // surf->data = (surfaceType_t *)tri;

    // ... rest of trisurface parsing code
}

/*
===============
ParseFlare
===============
*/
unsafe fn ParseFlare(df: *mut dflare_t, surf: *mut msurface_t) {
    // let flare: *mut srfFlare_t;
    let mut i: c_int;

    // surf->fogIndex = df->fogNum + 1;

    // get shader
    // surf->shader = ShaderForShaderNum( df->shaderNum, lightmapsVertex, stylesDefault );

    // flare = (srfFlare_t *) Hunk_Alloc( sizeof( *flare ), qtrue );
    // flare->surfaceType = SF_FLARE;

    // for ( i = 0 ; i < 3 ; i++ ) {
    //     flare->origin[i] = df->origin[i];
    //     flare->color[i] = df->color[i];
    //     flare->normal[i] = df->normal[i];
    // }

    // surf->data = (surfaceType_t *)flare;
}

unsafe fn R_LoadFlares(surfaces: *mut c_void, surface_len: c_int) {
    let mut count: c_int;
    let mut i: c_int;
    let mut in_: *mut dflare_t = core::ptr::null_mut();
    // let out: *mut msurface_t;

    count = surface_len / (core::mem::size_of::<dflare_t>() as c_int);

    for i in 0..count {
        // in_ = (dflare_t *)surfaces + i;
        // out = s_worldData.surfaces + in_->code;
        // ParseFlare( in_, out );
    }
}

/*
===============
R_LoadSurfaces
===============
*/
pub unsafe fn R_LoadSurfaces(count: c_int) {
    // s_worldData.surfaces = (struct msurface_s *)
    //     Hunk_Alloc ( count * sizeof(msurface_s), qtrue );
    // s_worldData.numsurfaces = count;
}

/*
===============
R_LoadPatches
===============
*/
pub unsafe fn R_LoadPatches(verts: *mut c_void, vert_len: c_int, surfaces: *mut c_void, surface_len: c_int) {
    // let in_: *mut dpatch_t = core::ptr::null_mut();
    // let out: *mut msurface_t;
    // let dv: *mut mapVert_t;
    let mut count: c_int;
    let mut i: c_int;

    if surface_len == 0 {
        return;
    }

    count = surface_len / (core::mem::size_of::<dpatch_t>() as c_int);

    // dv = (mapVert_t *)(verts);
    // if (vertlen % sizeof(*dv))
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);

    let points = Z_Malloc(
        (MAX_PATCH_SIZE as usize) * (MAX_PATCH_SIZE as usize) * core::mem::size_of::<drawVert_t>(),
        TAG_TEMP_WORKSPACE,
        qfalse,
        1,
    ) as *mut drawVert_t;

    let ctrl = Z_Malloc(
        (MAX_GRID_SIZE as usize) * (MAX_GRID_SIZE as usize) * core::mem::size_of::<drawVert_t>(),
        TAG_TEMP_WORKSPACE,
        qfalse,
        1,
    ) as *mut drawVert_t;

    let error_table = Z_Malloc(
        (2 * MAX_GRID_SIZE as usize) * core::mem::size_of::<f32>(),
        TAG_TEMP_WORKSPACE,
        qfalse,
        1,
    ) as *mut f32;

    for i in 0..count {
        // in_ = (dpatch_t *)surfaces + i;
        // out = s_worldData.surfaces + in_->code;
        // ParseMesh ( in_, dv, out, points, ctrl, errorTable );
    }

    Z_Free(error_table as *mut c_void);
    Z_Free(ctrl as *mut c_void);
    Z_Free(points as *mut c_void);

    VID_Printf(PRINT_ALL, "...loaded %i meshes\n\0".as_ptr() as *const c_char, count);
}

/*
===============
R_LoadTriSurfs
===============
*/
pub unsafe fn R_LoadTriSurfs(
    index_data: *mut c_void,
    index_len: c_int,
    verts: *mut c_void,
    vert_len: c_int,
    surfaces: *mut c_void,
    surface_len: c_int,
) {
    // let in_: *mut dtrisurf_t = core::ptr::null_mut();
    // let out: *mut msurface_t;
    // let dv: *mut mapVert_t;
    let mut indexes: *mut i16;
    let mut count: c_int;
    let mut i: c_int;

    if surface_len == 0 {
        return;
    }

    count = surface_len / (core::mem::size_of::<dtrisurf_t>() as c_int);

    // dv = (mapVert_t *)(verts);
    // if (vertlen % sizeof(*dv))
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);

    indexes = index_data as *mut i16;
    // if ( indexlen % sizeof(*indexes))
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);

    for i in 0..count {
        // in_ = (dtrisurf_t *)surfaces + i;
        // out = s_worldData.surfaces + in_->code;
        // ParseTriSurf( in_, dv, out, indexes );
    }

    VID_Printf(PRINT_ALL, "...loaded %i trisurfs\n\0".as_ptr() as *const c_char, count);
}

/*
===============
R_LoadFaces
===============
*/
pub unsafe fn R_LoadFaces(
    index_data: *mut c_void,
    index_len: c_int,
    verts: *mut c_void,
    vert_len: c_int,
    surfaces: *mut c_void,
    surface_len: c_int,
) {
    // let in_: *mut dface_t = core::ptr::null_mut();
    // let out: *mut msurface_t;
    // let dv: *mut mapVert_t;
    let mut indexes: *mut i16;
    let mut count: c_int;
    let mut i: c_int;

    if surface_len == 0 {
        return;
    }

    count = surface_len / (core::mem::size_of::<dface_t>() as c_int);

    // dv = (mapVert_t *)(verts);
    // if (vertlen % sizeof(*dv))
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);

    indexes = index_data as *mut i16;
    // if ( indexlen % sizeof(*indexes))
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);

    // new bit, the face code on our biggest map requires over 15,000 mallocs, which was no problem on the hunk,
    // bit hits the zone pretty bad (even the tagFree takes about 9 seconds for that many memblocks),
    // so special-case pre-alloc enough space for this data (the patches etc can stay as they are)...

    let n_times = count / 100;
    let mut n_to_go = n_times;
    let mut i_face_data_size_required: c_int = 0;

    for i in 0..count {
        // in_ = (dface_t *)surfaces + i;
        // let mut lightmap_num: [c_int; MAXLIGHTMAPS] = [0; MAXLIGHTMAPS];
        // for(int j=0; j<4; j++) {
        //     lightmapNum[j] = (int)in->lightmapNum[j] - 4;
        // }
        // shader_t *shader = ShaderForShaderNum( in->shaderNum, lightmapNum, in->lightmapStyles );
        // bool needVertexColors = NeedVertexColors(shader);
        // int numLightMaps = NumLightMaps(shader);
        // int sfaceSize = SurfaceFaceSize(in->verts & 0xFFF, numLightMaps, needVertexColors, in->indexes & 0xFFF);
        // iFaceDataSizeRequired += sfaceSize;
        // assert(sfaceSize < 100 * 1024);
        if n_to_go <= 0 {
            n_to_go = n_times;
        } else {
            n_to_go -= 1;
        }
    }
    // in -= count;	// back it up, ready for loop-proper

    // since this ptr is to hunk data, I can pass it in and have it advanced without worrying about losing
    // the original alloc ptr...
    let mut org_face_data: *mut byte;
    let mut p_face_data_buffer: *mut byte =
        Hunk_Alloc(i_face_data_size_required as usize, qtrue) as *mut byte;
    org_face_data = p_face_data_buffer;

    // now do regular loop...
    for i in 0..count {
        // in_ = (dface_t *)surfaces + i;
        // out = s_worldData.surfaces + in_->code;
        // ParseFace( in_, dv, out, indexes, pFaceDataBuffer );
        if n_to_go <= 0 {
            n_to_go = n_times;
        } else {
            n_to_go -= 1;
        }
    }

    VID_Printf(PRINT_ALL, "...loaded %d faces\n\0".as_ptr() as *const c_char, count);
}

/*
=================
R_LoadSubmodels
=================
*/
unsafe fn R_LoadSubmodels(data: *mut c_void, len: c_int) {
    let mut in_: *mut dmodel_t;
    // let out: *mut bmodel_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;

    in_ = data as *mut dmodel_t;
    // if (len % sizeof(*in_))
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);
    count = len / (core::mem::size_of::<dmodel_t>() as c_int);

    // s_worldData.bmodels = out = (bmodel_t *) Hunk_Alloc( count * sizeof(*out), qtrue );

    // for ( i=0 ; i<count ; i++, in++, out++ ) {
    //     ... model loading code
    // }
}

//==================================================================

/*
=================
R_SetParent
=================
*/
unsafe fn R_SetParent(mut node: *mut mnode_t, parent: *mut mnode_t) {
    // node->parent = parent;
    // if (node->contents != -1)
    //     return;
    // R_SetParent (node->children[0], node);
    // R_SetParent (node->children[1], node);
}

/*
=================
R_LoadNodesAndLeafs
=================
*/
unsafe fn R_LoadNodesAndLeafs(nodes: *mut c_void, nodelen: c_int, leafs: *mut c_void, leaflen: c_int) {
    let mut i: c_int;
    let mut j: c_int;
    let mut p: c_int;
    let mut in_: *mut dnode_t;
    let mut in_leaf: *mut dleaf_t;
    // let out_node: *mut mnode_t;
    // let out_leaf: *mut mleaf_s;
    let mut num_nodes: c_int;
    let mut num_leafs: c_int;

    in_ = nodes as *mut dnode_t;
    // if (nodelen % sizeof(dnode_t) ||
    //     leaflen % sizeof(dleaf_t) ) {
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);
    // }
    num_nodes = nodelen / (core::mem::size_of::<dnode_t>() as c_int);
    num_leafs = leaflen / (core::mem::size_of::<dleaf_t>() as c_int);

    // out_node = (struct mnode_s *) Hunk_Alloc ( (numNodes) * sizeof(*out_node), qtrue );
    // out_leaf = (struct mleaf_s *) Hunk_Alloc ( (numLeafs) * sizeof(*out_leaf), qtrue );

    // s_worldData.nodes = out_node;
    // s_worldData.leafs = out_leaf;
    // s_worldData.numnodes = numNodes;
    // s_worldData.numleafs = numLeafs;

    // load nodes
    // for ( i=0 ; i<numNodes; i++, in++, out_node++)
    // { ... node loading code ... }

    // load leafs
    in_leaf = leafs as *mut dleaf_t;
    // for ( i=0 ; i<numLeafs ; i++, inLeaf++, outLeaf++)
    // { ... leaf loading code ... }

    // chain decendants
    // R_SetParent (s_worldData.nodes, NULL);
}

//=============================================================================

/*
=================
R_LoadShaders
=================
*/
pub unsafe fn R_LoadShaders() {
    // s_worldData.shaders = cm.shaders;
    // s_worldData.numShaders = cm.numShaders;
}

/*
=================
R_LoadMarksurfaces
=================
*/
unsafe fn R_LoadMarksurfaces(data: *mut c_void, len: c_int) {
    let mut i: c_int;
    let mut count: c_int;
    let mut in_: *mut c_int;
    // let out: *mut *mut msurface_t;

    in_ = data as *mut c_int;
    // if (len % sizeof(*in_))
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);
    count = len / (core::mem::size_of::<c_int>() as c_int);
    // out = (struct msurface_s **) Hunk_Alloc ( count*sizeof(*out), qtrue );

    // s_worldData.marksurfaces = out;
    // s_worldData.nummarksurfaces = count;

    for i in 0..count {
        // if(in[i] > s_worldData.numsurfaces)
        //     assert(0);
        // out[i] = s_worldData.surfaces + in[i];
        // if (out[i]->shader && out[i]->shader->sort == SS_PORTAL)
        // {
        //     s_worldData.portalPresent = qtrue;
        // }
    }
}

/*
=================
R_LoadPlanes
=================
*/
unsafe fn R_LoadPlanes() {
    // New method - share with server.
    // s_worldData.planes = cmg.planes;
    // s_worldData.numplanes = cmg.numPlanes;
}

/*
=================
R_LoadFogs

=================
*/
unsafe fn R_LoadFogs(
    fog_data: *mut c_void,
    fog_len: c_int,
    brush_data: *mut c_void,
    brush_len: c_int,
    side_data: *mut c_void,
    side_len: c_int,
) {
    let mut i: c_int;
    // let out: *mut fog_t;
    let mut fogs: *mut dfog_t;
    // let brushes: *mut dbrush_t;
    // let brush: *mut dbrush_t;
    // let sides: *mut dbrushside_t;
    let mut count: c_int;
    // let brushes_count: c_int;
    // let sides_count: c_int;
    // let side_num: c_int;
    // let plane_num: c_int;
    // let shader: *mut shader_t;
    let mut d: f32;
    let mut first_side: c_int = 0;
    let mut lightmaps: [c_int; MAXLIGHTMAPS] = [LIGHTMAP_NONE; MAXLIGHTMAPS];

    fogs = fog_data as *mut dfog_t;
    // if (foglen % sizeof(*fogs)) {
    //     Com_Error (ERR_DROP, "LoadMap: funny lump size in %s",s_worldData.name);
    // }
    count = fog_len / (core::mem::size_of::<dfog_t>() as c_int);

    // create fog structres for them
    // NOTE: we allocate memory for an extra one so that the LA goggles can turn on their own fog
    // s_worldData.numfogs = count + 1;
    // s_worldData.fogs = (fog_t *)Hunk_Alloc (( s_worldData.numfogs + 1)*sizeof(*out), qtrue );
    // s_worldData.globalFog = -1;
    // out = s_worldData.fogs + 1;

    if count == 0 {
        return;
    }

    // ... rest of fog loading code
}

/*
================
R_LoadLightGrid

================
*/
pub unsafe fn R_LoadLightGrid(data: *mut c_void, len: c_int) {
    let mut maxs: vec3_t = [0.0; 3];
    // let w: *mut world_t;
    let mut i: c_int;
    // let w_mins: *mut f32;
    // let w_maxs: *mut f32;

    // w = &s_worldData;

    // w->lightGridInverseSize[0] = 1.0 / w->lightGridSize[0];
    // w->lightGridInverseSize[1] = 1.0 / w->lightGridSize[1];
    // w->lightGridInverseSize[2] = 1.0 / w->lightGridSize[2];

    // w_mins = w->bmodels[0].bounds[0];
    // w_maxs = w->bmodels[0].bounds[1];

    // for ( i = 0 ; i < 3 ; i++ ) {
    //     w->lightGridOrigin[i] = w->lightGridSize[i] * ceil( wMins[i] / w->lightGridSize[i] );
    //     maxs[i] = w->lightGridSize[i] * floor( wMaxs[i] / w->lightGridSize[i] );
    //     w->lightGridBounds[i] = (maxs[i] - w->lightGridOrigin[i])/w->lightGridSize[i] + 1;
    // }

    // w->lightGridData = (mgrid_t *)Hunk_Alloc( len, qfalse );
    // memcpy( w->lightGridData, data, len as usize );
}

/*
================
R_LoadLightGridArray

================
*/
pub unsafe fn R_LoadLightGridArray(data: *mut c_void, len: c_int) {
    // let w: *mut world_t;

    // w = &s_worldData;

    // w->numGridArrayElements = w->lightGridBounds[0] * w->lightGridBounds[1] * w->lightGridBounds[2];

    // if ( len != w->numGridArrayElements * sizeof(*w->lightGridArray) ) {
    //     if (len>0)//don't warn if not even lit
    //         VID_Printf( PRINT_WARNING, "WARNING: light grid array mismatch\n" );
    //     w->lightGridData = NULL;
    //     return;
    // }

    // w->lightGridArray = (unsigned short *)Hunk_Alloc( len, qfalse );
    // memcpy( w->lightGridArray, data, len as usize );
}

/*
================
R_LoadEntities
================
*/
pub unsafe fn R_LoadEntities(data: *mut c_void, len: c_int) {
    let mut p: *const c_char;
    let mut token: *const c_char;
    let mut keyname: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
    let mut value: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
    // let w: *mut world_t;
    let mut ambient: f32 = 1.0;

    // w = &s_worldData;
    // w->lightGridSize[0] = 64;
    // w->lightGridSize[1] = 64;
    // w->lightGridSize[2] = 128;

    // VectorSet(tr.sunAmbient, 1, 1, 1);
    // tr.distanceCull = 12000;//DEFAULT_DISTANCE_CULL;

    p = data as *const c_char;

    token = COM_ParseExt(&mut p, qtrue);
    if token == core::ptr::null() || (*token as c_char) == 0 || (*token as c_char) != 123 {
        // '{'
        return;
    }

    // only parse the world spawn
    loop {
        // parse key
        token = COM_ParseExt(&mut p, qtrue);

        if token == core::ptr::null() || (*token as c_char) == 0 || (*token as c_char) == 125 {
            // '}'
            break;
        }
        Q_strncpyz(keyname.as_mut_ptr(), token, MAX_TOKEN_CHARS);

        // parse value
        token = COM_ParseExt(&mut p, qtrue);

        if token == core::ptr::null() || (*token as c_char) == 0 || (*token as c_char) == 125 {
            // '}'
            break;
        }
        Q_strncpyz(value.as_mut_ptr(), token, MAX_TOKEN_CHARS);

        if Q_stricmp(keyname.as_ptr(), "distanceCull\0".as_ptr() as *const c_char) == 0 {
            // sscanf(value, "%f", &tr.distanceCull );
            continue;
        }
        // check for linear fog -rww
        if Q_stricmp(keyname.as_ptr(), "linFogStart\0".as_ptr() as *const c_char) == 0 {
            // sscanf(value, "%f", &tr.rangedFog );
            // tr.rangedFog = -tr.rangedFog;
            continue;
        }
        // check for a different grid size
        if Q_stricmp(keyname.as_ptr(), "gridsize\0".as_ptr() as *const c_char) == 0 {
            // sscanf(value, "%f %f %f", &w->lightGridSize[0], &w->lightGridSize[1], &w->lightGridSize[2] );
            continue;
        }
        // find the optional world ambient for arioche
        if Q_stricmp(keyname.as_ptr(), "_color\0".as_ptr() as *const c_char) == 0 {
            // sscanf(value, "%f %f %f", &tr.sunAmbient[0], &tr.sunAmbient[1], &tr.sunAmbient[2] );
            continue;
        }
        if Q_stricmp(keyname.as_ptr(), "ambient\0".as_ptr() as *const c_char) == 0 {
            // sscanf(value, "%f", &ambient);
            continue;
        }
    }
    // both default to 1 so no harm if not present.
    // VectorScale( tr.sunAmbient, ambient, tr.sunAmbient);
}

/*
=================
RE_LoadWorldMap

Called directly from cgame
=================
*/
pub unsafe fn RE_LoadWorldMap_Actual(name: *const c_char, world_data: &mut world_t, index: c_int) {
    let mut strip_name: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    // let mut output_lumps: [Lump; 3] = [Lump::default(); 3];

    // This is no longer correct. The new code supports sub-models, apparently BSPs in
    // several chunks. If any map tries to use them, the following COM_Error will go
    // off. We haven't hit it yet, but if (when) we do, check out tr_bsp.cpp for changes.
    // if ( tr.worldMapLoaded ) {
    //     Com_Error( ERR_DROP, "ERROR: attempted to redundantly load world map\n" );
    // }

    // set default sun direction to be used if it isn't
    // overridden by a shader
    skyboxportal = 0;

    // tr.sunDirection[0] = 0.45f;
    // tr.sunDirection[1] = 0.3f;
    // tr.sunDirection[2] = 0.9f;

    // VectorNormalize( tr.sunDirection );

    // Cvar_SetValue( "r_sundir_x", tr.sunDirection[0] );
    // Cvar_SetValue( "r_sundir_y", tr.sunDirection[1] );
    // Cvar_SetValue( "r_sundir_z", tr.sunDirection[2] );

    // tr.worldMapLoaded = qtrue;

    // clear tr.world so if the level fails to load, the next
    // try will not look at the partially loaded version
    // tr.world = NULL;

    // Preserve data which was already set in cm_load
    // let surface_ptr = s_worldData.surfaces;
    // let num_surfaces = s_worldData.numsurfaces;
    // memset( &s_worldData, 0, core::mem::size_of_val(&s_worldData) );
    // s_worldData.surfaces = surface_ptr;
    // s_worldData.numsurfaces = num_surfaces;
    // s_worldData.numShaders = cmg.numShaders;

    // Q_strncpyz( s_worldData.name, name, core::mem::size_of_val(&s_worldData.name) );

    // Q_strncpyz( s_worldData.baseName, COM_SkipPath( s_worldData.name ), core::mem::size_of_val(&s_worldData.name) );
    // COM_StripExtension( s_worldData.baseName, s_worldData.baseName );

    COM_StripExtension(name, strip_name.as_mut_ptr());

    c_gridVerts = 0;

    // load into heap
    R_LoadPlanes();

    // outputLumps[0].load(stripName, "fogs");
    // outputLumps[1].load(stripName, "brushes");
    // outputLumps[2].load(stripName, "brushsides");
    // R_LoadFogs( outputLumps[0].data, outputLumps[0].len,
    //     outputLumps[1].data, outputLumps[1].len,
    //     outputLumps[2].data, outputLumps[2].len );
    // outputLumps[2].clear();
    // outputLumps[1].clear();

    // outputLumps[0].load(stripName, "leafsurfaces");
    // R_LoadMarksurfaces (outputLumps[0].data, outputLumps[0].len);

    // outputLumps[0].load(stripName, "nodes");
    // outputLumps[1].load(stripName, "leafs");
    // R_LoadNodesAndLeafs (outputLumps[0].data, outputLumps[0].len,
    //     outputLumps[1].data, outputLumps[1].len);
    // outputLumps[1].clear();

    // outputLumps[0].load(stripName, "models");
    // R_LoadSubmodels (outputLumps[0].data, outputLumps[0].len);

    R_LoadVisibility();

    // outputLumps[0].load(stripName, "entities");
    // R_LoadEntities( outputLumps[0].data, outputLumps[0].len );
    // outputLumps[0].load(stripName, "lightgrid");
    // R_LoadLightGrid( outputLumps[0].data, outputLumps[0].len );
    // outputLumps[0].load(stripName, "lightarray");
    // R_LoadLightGridArray( outputLumps[0].data, outputLumps[0].len );

    // only set tr.world now that we know the entire level has loaded properly
    // tr.world = &s_worldData;

    // Load the light parms for this level
    // R_LoadLevelLightParms();
    // R_GetLightParmsForLevel();
}

// new wrapper used for convenience to tell z_malloc()-fail recovery code whether it's safe to dump the cached-bsp or not.
//
extern "C" {
    pub static mut gbUsingCachedMapDataRightNow: qboolean;
}

pub unsafe fn RE_LoadWorldMap(name: *const c_char) {
    gbUsingCachedMapDataRightNow = qtrue; // !!!!!!!!!!!!

    RE_LoadWorldMap_Actual(name, &mut s_worldData, 0);

    gbUsingCachedMapDataRightNow = qfalse; // !!!!!!!!!!!!
}
