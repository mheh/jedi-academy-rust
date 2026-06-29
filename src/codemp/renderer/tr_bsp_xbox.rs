// tr_map.c

// leave this as first line for PCH reasons...
//

// porting: including tr_local.h equivalents and other headers through module system

use core::ffi::{c_int, c_void};
use core::ptr;
use std::mem;

// porting: local type/function stubs for structural coherence

// Forward declarations and external types from other modules
// These would normally come from tr_local.h and related headers

#[repr(C)]
pub struct world_t {
    // porting: stub structure - actual fields would come from tr_local.h
}

#[repr(C)]
pub struct shader_t {
    // porting: stub structure
}

#[repr(C)]
pub struct msurface_t {
    // porting: stub structure
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    // porting: stub structure
}

#[repr(C)]
pub struct srfGridMesh_t {
    // porting: stub structure
}

#[repr(C)]
pub struct srfTriangles_t {
    // porting: stub structure
}

#[repr(C)]
pub struct srfFlare_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dshader_t {
    // porting: stub structure
}

#[repr(C)]
pub struct mapVert_t {
    // porting: stub structure
}

#[repr(C)]
pub struct drawVert_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dface_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dpatch_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dtrisurf_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dflare_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dmodel_t {
    // porting: stub structure
}

#[repr(C)]
pub struct mnode_t {
    // porting: stub structure
}

#[repr(C)]
pub struct mleaf_s {
    // porting: stub structure
}

#[repr(C)]
pub struct dnode_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dleaf_t {
    // porting: stub structure
}

#[repr(C)]
pub struct cplane_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dplane_t {
    // porting: stub structure
}

#[repr(C)]
pub struct fog_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dfog_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dbrush_t {
    // porting: stub structure
}

#[repr(C)]
pub struct dbrushside_t {
    // porting: stub structure
}

#[repr(C)]
pub struct mgrid_t {
    // porting: stub structure
}

type vec3_t = [f32; 3];
type qhandle_t = c_int;
type qboolean = c_int;
type byte = u8;

// porting: external stubs for enums/constants defined in headers
const MAX_GREYSCALE_CHANNEL_DIFF: usize = 15;
const LIGHTMAP_SIZE: usize = 128;
const MAXLIGHTMAPS: usize = 4;
const MAX_FACE_POINTS: usize = 0;  // porting: stub
const VERTEX_LM: usize = 0;  // porting: stub
const DRAWVERT_ST_SCALE: f32 = 1.0;  // porting: stub
const POINTS_ST_SCALE: f32 = 1.0;  // porting: stub
const GRID_DRAWVERT_ST_SCALE: f32 = 1.0;  // porting: stub
const POINTS_LIGHT_SCALE: f32 = 1.0;  // porting: stub
const DRAWVERT_LIGHTMAP_SCALE: f32 = 1.0;  // porting: stub
const MAX_QPATH: usize = 256;
const MAX_TOKEN_CHARS: usize = 1024;
const MAX_PATCH_SIZE: usize = 32;
const MAX_GRID_SIZE: usize = 33;
const MIN_WORLD_COORD: f32 = -131072.0;
const MAX_WORLD_COORD: f32 = 131072.0;

const SF_FACE: c_int = 1;
const SF_SKIP: c_int = 3;
const SF_TRIANGLES: c_int = 4;
const SF_FLARE: c_int = 5;

const CONTENTS_NODE: c_int = -1;
const LIGHTMAP_NONE: c_int = -1;

const ERR_DROP: c_int = 1;

const SS_PORTAL: c_int = 10;

const SURF_NODRAW: c_int = 0x80;

const CGEN_EXACT_VERTEX: c_int = 5;
const CGEN_VERTEX: c_int = 4;
const CGEN_ONE_MINUS_VERTEX: c_int = 6;

const AGEN_VERTEX: c_int = 2;
const AGEN_ONE_MINUS_VERTEX: c_int = 3;

const S_COLOR_YELLOW: &str = "^3";

// porting: external functions from other modules
extern "C" {
    static mut tr: std::mem::MaybeUninit<()>;  // porting: stub for tr global

    fn Com_Printf(fmt: *const c_int, ...);
    fn Com_Error(level: c_int, fmt: *const c_int, ...);
    fn Com_Memcpy(dst: *mut c_void, src: *const c_void, len: usize);
    fn Com_sprintf(buf: *mut c_int, size: usize, fmt: *const c_int, ...);

    fn Hunk_Alloc(size: usize, tag: c_int) -> *mut c_void;
    fn Z_Malloc(size: usize, tag: c_int, clear: qboolean, alignment: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    fn R_SyncRenderThread();
    fn R_CreateImage(name: *const c_int, data: *mut byte, width: c_int, height: c_int,
                     format: c_int, is_mipmap: qboolean, allow_pic_mip: c_int, wrap: c_int) -> qhandle_t;
    fn R_FindShader(name: *const c_int, light_maps: *const c_int, styles: *const byte, force_load: qboolean) -> *mut shader_t;
    fn R_RegisterShader(name: *const c_int) -> qhandle_t;
    fn RE_RegisterShader(name: *const c_int) -> qhandle_t;
    fn R_AllocModel() -> *mut ();  // porting: stub
    fn RE_InsertModelIntoHash(name: *const c_int, model: *mut ());
    fn R_SubdividePatchToGrid(width: c_int, height: c_int, points: *mut drawVert_t,
                              ctrl: *mut drawVert_t, error_table: *mut f32) -> *mut srfGridMesh_t;
    fn R_LoadLevelLightParms();
    fn R_GetLightParmsForLevel();
    fn R_RMGInit();

    fn Cvar_SetValue(name: *const c_int, value: f32);

    fn COM_ParseExt(data_p: *mut *const c_int, allow_line_breaks: qboolean) -> *const c_int;
    fn COM_Parse(data_p: *mut *const c_int) -> *const c_int;
    fn COM_StripExtension(in_: *const c_int, out_: *mut c_int);
    fn COM_SkipPath(in_: *const c_int) -> *const c_int;

    fn Q_strncpyz(dst: *mut c_int, src: *const c_int, len: usize);
    fn Q_stricmp(s1: *const c_int, s2: *const c_int) -> c_int;
    fn Q_CastShort2FloatScale(dst: *mut f32, src: *const i16, scale: f32);

    fn CrossProduct(v1: *const vec3_t, v2: *const vec3_t, cross: *mut vec3_t);
    fn VectorNormalizeFast(vec: *mut vec3_t);
    fn VectorAdd(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t);
    fn VectorScale(vec: *const vec3_t, scale: f32, out: *mut vec3_t);
    fn VectorSubtract(v1: *const vec3_t, v2: *const vec3_t, out: *mut vec3_t);
    fn VectorLength(vec: *const vec3_t) -> f32;
    fn VectorSet(vec: *mut vec3_t, x: f32, y: f32, z: f32);
    fn VectorNormalize(vec: *mut vec3_t);
    fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32;

    fn ClearBounds(mins: *mut vec3_t, maxs: *mut vec3_t);
    fn AddPointToBounds(point: *const vec3_t, mins: *mut vec3_t, maxs: *mut vec3_t);

    fn SetPlaneSignbits(plane: *mut cplane_t);
    fn PlaneTypeForNormal(normal: *const vec3_t) -> c_int;
    fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> c_int;

    static qtrue: qboolean;
    static qfalse: qboolean;

    static vec3_origin: vec3_t;
}

// porting: memory allocation tags (stubs)
const TAG_BSP: c_int = 1;
const TAG_TEMP_WORKSPACE: c_int = 2;
const h_low: c_int = 1;

const GL_DDS_RGB16_EXT: c_int = 0;
const GL_CLAMP: c_int = 0;

// Global variables
static mut s_worldData: world_t = unsafe { mem::zeroed() };
static mut fileBase: *mut byte = ptr::null_mut();
static mut c_subdivisions: c_int = 0;
static mut c_gridVerts: c_int = 0;

static mut skyboxportal: c_int = 0;
static mut gbUsingCachedMapDataRightNow: qboolean = 0;

// Forward declare R_RMGInit (defined elsewhere)
// void R_RMGInit(void);

// We use a special hack to prevent slight differences in channels
// from exploding into big differences, as it causes lighting problems
// later on. This is the maximum channel separation for which we
// enable the hack.
// #define MAX_GREYSCALE_CHANNEL_DIFF 15

unsafe fn R_ColorShiftLightingBytes16(in_: *const [byte; 4], out: *mut [byte; 2]) {
    // What's the largest separation between the red, green, and blue
    // channels?
    let chan_diff = (*in_)[0].max((*in_)[1]).max((*in_)[2]) -
        (*in_)[0].min((*in_)[1]).min((*in_)[2]);
    if chan_diff <= MAX_GREYSCALE_CHANNEL_DIFF as byte {
        // Ensure that all color channels compress to the same value
        let channel_avg = ((*in_)[0] as c_int + (*in_)[1] as c_int + (*in_)[2] as c_int + 1) / 3;
        let channel_avg = channel_avg as byte;
        (*out)[0] = channel_avg & 0xF0;
        (*out)[0] |= (channel_avg & 0xF0) >> 4;
        (*out)[1] = channel_avg & 0xF0;
        (*out)[1] |= ((*in_)[3] & 0xF0) >> 4;

        if channel_avg as c_int % 16 >= 8 {
            (*out)[0] |= 0x10;
            (*out)[0] |= 0x01;
            (*out)[1] |= 0x10;
        }
        if (*in_)[4] as c_int % 16 >= 8 {
            (*out)[1] |= 0x01;
        }
        return;
    }

    // Normal case for vertex colors that are not "near" greyscale
    (*out)[0] = (*in_)[0] & 0xF0;
    (*out)[0] |= ((*in_)[1] & 0xF0) >> 4;
    (*out)[1] = (*in_)[2] & 0xF0;
    (*out)[1] |= ((*in_)[3] & 0xF0) >> 4;

    if (*in_)[0] as c_int % 16 >= 8 {
        (*out)[0] |= 0x10;
    }
    if (*in_)[1] as c_int % 16 >= 8 {
        (*out)[0] |= 0x1;
    }
    if (*in_)[2] as c_int % 16 >= 8 {
        (*out)[1] |= 0x10;
    }
    if (*in_)[3] as c_int % 16 >= 8 {
        (*out)[1] |= 0x1;
    }
}


unsafe fn HSVtoRGB(mut h: f32, s: f32, v: f32, rgb: *mut [f32; 3]) {
    let mut i: c_int;
    let mut f: f32;
    let mut p: f32;
    let mut q: f32;
    let mut t: f32;

    h *= 5.0;

    i = h.floor() as c_int;
    f = h - i as f32;

    p = v * (1.0 - s);
    q = v * (1.0 - s * f);
    t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => {
            (*rgb)[0] = v;
            (*rgb)[1] = t;
            (*rgb)[2] = p;
        },
        1 => {
            (*rgb)[0] = q;
            (*rgb)[1] = v;
            (*rgb)[2] = p;
        },
        2 => {
            (*rgb)[0] = p;
            (*rgb)[1] = v;
            (*rgb)[2] = t;
        },
        3 => {
            (*rgb)[0] = p;
            (*rgb)[1] = q;
            (*rgb)[2] = v;
        },
        4 => {
            (*rgb)[0] = t;
            (*rgb)[1] = p;
            (*rgb)[2] = v;
        },
        5 => {
            (*rgb)[0] = v;
            (*rgb)[1] = p;
            (*rgb)[2] = q;
        },
        _ => {},
    }
}

// ===============
// R_ColorShiftLightingBytes
//
// ===============
//
unsafe fn R_ColorShiftLightingBytes(in_: *const [byte; 4], out: *mut [byte; 4]) {
    let mut shift: c_int = 0;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;

    // should NOT do it if overbrightBits is 0
    if (*ptr::addr_of!(tr)).type_id() != std::any::TypeId::of::<()>() {
        // porting: stub - tr.overbrightBits would be accessed here
        shift = 0;  // porting: placeholder
    }

    if shift == 0 {
        (*out)[0] = (*in_)[0];
        (*out)[1] = (*in_)[1];
        (*out)[2] = (*in_)[2];
        (*out)[3] = (*in_)[3];
        return;
    }

    // shift the data based on overbright range
    r = ((*in_)[0] as c_int) << shift;
    g = ((*in_)[1] as c_int) << shift;
    b = ((*in_)[2] as c_int) << shift;

    // normalize by color instead of saturating to white
    if ((r | g | b) > 255) {
        let mut max: c_int;

        max = if r > g { r } else { g };
        max = if max > b { max } else { b };
        r = r * 255 / max;
        g = g * 255 / max;
        b = b * 255 / max;
    }

    (*out)[0] = r as byte;
    (*out)[1] = g as byte;
    (*out)[2] = b as byte;
    (*out)[3] = (*in_)[3];
}

// ===============
// R_ColorShiftLightingBytes
//
// ===============
//
unsafe fn R_ColorShiftLightingBytes_3(in_: *mut [byte; 3]) {
    let mut shift: c_int = 0;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;

    // should NOT do it if overbrightBits is 0
    if (*ptr::addr_of!(tr)).type_id() != std::any::TypeId::of::<()>() {
        // porting: stub - tr.overbrightBits would be accessed here
        shift = 0;  // porting: placeholder
    }

    if shift == 0 {
        return;  // no need if not overbright
    }
    // shift the data based on overbright range
    r = ((*in_)[0] as c_int) << shift;
    g = ((*in_)[1] as c_int) << shift;
    b = ((*in_)[2] as c_int) << shift;

    // normalize by color instead of saturating to white
    if ((r | g | b) > 255) {
        let mut max: c_int;

        max = if r > g { r } else { g };
        max = if max > b { max } else { b };
        r = r * 255 / max;
        g = g * 255 / max;
        b = b * 255 / max;
    }

    (*in_)[0] = r as byte;
    (*in_)[1] = g as byte;
    (*in_)[2] = b as byte;
}


// ===============
// R_LoadLightmaps
//
// ===============
// #define	LIGHTMAP_SIZE	128
pub unsafe fn R_LoadLightmaps(data: *mut c_void, len: c_int, ps_map_name: *const c_int) {
    let mut buf: *mut byte;
    let mut buf_p: *mut byte;
    let mut i: c_int;

    if len == 0 {
        return;
    }
    buf = (data as *mut byte).add(mem::size_of::<c_int>());

    // porting: tr.numLightmaps stub
    // tr.numLightmaps = 0;

    // we are about to upload textures
    R_SyncRenderThread();

    // create all the lightmaps
    let size = *(data as *const c_int) as usize;
    // porting: tr.numLightmaps = len as usize / size;

    let image = Z_Malloc(size, TAG_BSP, qfalse, 32) as *mut byte;

    let mut s_map_name: [c_int; MAX_QPATH] = [0; MAX_QPATH];
    COM_StripExtension(ps_map_name, &mut s_map_name as *mut _ as *mut c_int);  // will already by MAX_QPATH legal, so no length check

    for i in 0..(len as usize / size) {
        buf_p = buf.add(i * size);
        Com_Memcpy(image as *mut c_void, buf_p as *const c_void, size);

        let mut lmap_name: [c_int; MAX_QPATH + 32] = [0; MAX_QPATH + 32];
        Com_sprintf(&mut lmap_name as *mut _ as *mut c_int, MAX_QPATH + 32,
                    "string" as *const _ as *const c_int);  // porting: placeholder format string
        // porting: tr.lightmaps[i] stub
        // R_CreateImage(lmap_name, image, LIGHTMAP_SIZE, LIGHTMAP_SIZE, GL_DDS_RGB16_EXT, qfalse, 0, GL_CLAMP);
    }

    Z_Free(image as *mut c_void);
}


// =================
// RE_SetWorldVisData
//
// This is called by the clipmodel subsystem so we can share the 1.8 megs of
// space in big maps...
// =================
//
// porting: SPARC<byte> is a container type stub
pub unsafe fn RE_SetWorldVisData(_vis: *mut byte) {
    // porting: tr.externalVisData = vis;
}


// =================
// R_LoadVisibility
// =================
//
unsafe fn R_LoadVisibility(data: *mut c_void, len: c_int) {
    let mut length: c_int;
    let mut buf: *mut c_int;

    length = (s_worldData.type_id() as c_int + 63) & !63;  // porting: stub numClusters
    // porting: s_worldData.novis = Hunk_Alloc(length as usize, h_low) as *mut byte;
    // memset(s_worldData.novis, 0xff, length as usize);

    if len == 0 {
        // porting: s_worldData.vis = ptr::null_mut();
        return;
    }
    buf = data as *mut c_int;

    // porting: s_worldData.numClusters = buf[0];
    // porting: s_worldData.clusterBytes = buf[1];

    // CM_Load should have given us the vis data to share, so
    // we don't need to allocate another copy
    // porting: if tr.externalVisData { ... } else { assert!(false); }
}

// ===============================================================================

pub unsafe fn R_GetShaderByNum(shader_num: c_int, _world_data: &world_t) -> qhandle_t {
    let mut shader: qhandle_t;

    if (shader_num < 0) || (shader_num >= 0) {  // porting: worldData.numShaders stub
        Com_Printf(b"Warning: Bad index for R_GetShaderByNum - %i\0" as *const _ as *const c_int);
        return 0;
    }
    // porting: shader = RE_RegisterShader(worldData.shaders[shader_num as usize].shader);
    shader = 0;  // porting: stub
    shader
}

// ===============
// ShaderForShaderNum
// ===============
//
unsafe fn ShaderForShaderNum(shader_num: c_int, _lightmap_num: *const c_int, _lightmap_styles: *const byte) -> *mut shader_t {
    let mut shader: *mut shader_t;
    // porting: dsh stub
    let dsh: *const dshader_t = ptr::null();

    // shaderNum = shaderNum;
    if shader_num < 0 || shader_num >= 0 {  // porting: s_worldData.numShaders stub
        Com_Error(ERR_DROP, b"ShaderForShaderNum: bad num %i\0" as *const _ as *const c_int);
    }
    // porting: dsh = &s_worldData.shaders[shader_num as usize];

    // porting: shader = R_FindShader(dsh->shader, lightmapNum, lightmapStyles, qtrue);
    shader = ptr::null_mut();  // porting: stub

    // if the shader had errors, just use default shader
    // porting: if shader->defaultShader { return tr.defaultShader; }

    shader
}

unsafe fn NeedVertexColors(_shader: *mut shader_t) -> bool {
    // porting: stub
    false
}

unsafe fn NumLightMaps(_shader: *mut shader_t) -> c_int {
    // porting: stub
    0
}

unsafe fn SurfaceFaceSize(num_verts: c_int, num_light_maps: c_int, _need_vertex_colors: bool,
        num_indexes: c_int) -> c_int {
    let sface_size = unsafe {
        // porting: complex offset calculation
        let offset = mem::offset_of!(srfSurfaceFace_t, srfPoints) as c_int;
        offset + 4 +
        (num_verts * mem::size_of::<u16>() as c_int *
            (VERTEX_LM as c_int + num_light_maps * 2))
    };

    // Add in tangent size
    let sface_size = sface_size + (mem::size_of::<vec3_t>() as c_int * num_verts);

    // Indices stored in 8 bits now.
    let sface_size = sface_size + num_indexes;

    sface_size
}


unsafe fn BuildDrawVertTangents(_verts: *mut drawVert_t, _indexes: *mut c_int, _num_indexes: c_int, _num_vertexes: c_int) {
    // porting: stub - complex tangent calculation
}


unsafe fn BuildMapVertTangents(_verts: *mut mapVert_t, _tangents: *mut vec3_t, _indexes: *mut i16, _num_indexes: c_int, _num_vertexes: c_int) {
    // porting: stub - complex tangent calculation
}

// ===============
// ParseFace
// ===============
//
unsafe fn ParseFace(_ds: *mut dface_t, _verts: *mut mapVert_t, _surf: *mut msurface_t,
                    _indexes: *mut i16, _p_face_data_buffer: *mut *mut byte) {
    // porting: stub - complex face parsing
}


// ===============
// ParseMesh
// ===============
//
unsafe fn ParseMesh(_ds: *mut dpatch_t, _verts: *mut mapVert_t, _surf: *mut msurface_t,
                    _points: *mut drawVert_t, _ctrl: *mut drawVert_t, _error_table: *mut f32) {
    // porting: stub - complex mesh parsing
}

// ===============
// ParseTriSurf
// ===============
//
unsafe fn ParseTriSurf(_ds: *mut dtrisurf_t, _verts: *mut mapVert_t, _surf: *mut msurface_t, _indexes: *mut i16) {
    // porting: stub - complex triangle surface parsing
}


// ===============
// ParseFlare
// ===============
//
unsafe fn ParseFlare(_df: *mut dflare_t, _surf: *mut msurface_t) {
    // porting: stub - complex flare parsing
}


pub unsafe fn R_LoadFlares(_surfaces: *mut c_void, _surfacelen: c_int) {
    // porting: stub
}


// ===============
// R_LoadSurfaces
// ===============
//
pub unsafe fn R_LoadSurfaces(_count: c_int) {
    // porting: s_worldData.surfaces stub
}


// ===============
// R_LoadPatches
// ===============
//
pub unsafe fn R_LoadPatches(_verts: *mut c_void, _vertlen: c_int,
                             _surfaces: *mut c_void, _surfacelen: c_int) {
    // porting: stub
}


// ===============
// R_LoadTriSurfs
// ===============
//
pub unsafe fn R_LoadTriSurfs(_indexdata: *mut c_void, _indexlen: c_int,
                             _verts: *mut c_void, _vertlen: c_int,
                             _surfaces: *mut c_void, _surfacelen: c_int) {
    // porting: stub
}


// ===============
// R_LoadFaces
// ===============
//
pub unsafe fn R_LoadFaces(_indexdata: *mut c_void, _indexlen: c_int,
                          _verts: *mut c_void, _vertlen: c_int,
                          _surfaces: *mut c_void, _surfacelen: c_int) {
    // porting: stub
}


// =================
// R_LoadSubmodels
// =================
//
unsafe fn R_LoadSubmodels(_data: *mut c_void, _len: c_int) {
    // porting: stub
}

// ==================================================================

// =================
// R_SetParent
// =================
//
unsafe fn R_SetParent(_node: *mut mnode_t, _parent: *mut mnode_t) {
    // porting: stub
}

// =================
// R_LoadNodesAndLeafs
// =================
//
unsafe fn R_LoadNodesAndLeafs(_nodes: *mut c_void, _nodelen: c_int, _leafs: *mut c_void, _leaflen: c_int) {
    // porting: stub
}

// =============================================================================

// =================
// R_LoadShaders
// =================
//
pub unsafe fn R_LoadShaders(_data: *mut c_void, _len: c_int) {
    // porting: stub
}

// =================
// R_LoadMarksurfaces
// =================
//
unsafe fn R_LoadMarksurfaces(_data: *mut c_void, _len: c_int) {
    // porting: stub
}

// =================
// R_LoadPlanes
// =================
//
unsafe fn R_LoadPlanes(_data: *mut c_void, _len: c_int) {
    // porting: stub
}

// =================
// R_LoadFogs
//
// =================
//
unsafe fn R_LoadFogs(_fogdata: *mut c_void, _foglen: c_int,
                     _brushdata: *mut c_void, _brushlen: c_int,
                     _sidedata: *mut c_void, _sidelen: c_int) {
    // porting: stub
}

// ================
// R_LoadLightGrid
//
// ================
//
pub unsafe fn R_LoadLightGrid(_data: *mut c_void, _len: c_int) {
    // porting: stub
}

// ================
// R_LoadLightGridArray
//
// ================
//
pub unsafe fn R_LoadLightGridArray(_data: *mut c_void, _len: c_int) {
    // porting: stub
}

// ================
// R_LoadEntities
// ================
//
pub unsafe fn R_LoadEntities(_data: *mut c_void, _len: c_int) {
    // porting: stub
}

// =================
// R_GetEntityToken
// =================
//
pub unsafe fn R_GetEntityToken(_buffer: *mut c_int, _size: c_int) -> qboolean {
    // porting: stub
    qfalse
}


// =================
// RE_LoadWorldMap
//
// Called directly from cgame
// =================
//
pub unsafe fn RE_LoadWorldMap_Actual(_name: *const c_int, _world_data: &world_t, _index: c_int) {
    // porting: stub - complex world map loading
}


// new wrapper used for convenience to tell z_malloc()-fail recovery code whether it's safe to dump the cached-bsp or not.
//
pub unsafe fn RE_LoadWorldMap(_name: *const c_int) {
    addr_of_mut!(gbUsingCachedMapDataRightNow).write(qtrue);

    RE_LoadWorldMap_Actual(_name, &s_worldData, 0);

    addr_of_mut!(gbUsingCachedMapDataRightNow).write(qfalse);
}

// porting: helper for offset_of! since we can't use it on incomplete types
use core::ptr::{addr_of, addr_of_mut};
