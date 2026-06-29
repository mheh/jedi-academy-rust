// tr_map.c

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"
// #include "tr_local.h"

/*

Loads and prepares a map file for scene rendering.

A single entry point:

void RE_LoadWorldMap( const char *name );

*/

use core::ffi::{c_int, c_char, c_void};

// LOCAL STUBS - these would come from engine/tr_local.h and other headers
// For now, we declare minimal stubs needed for structural coherence

// Extern functions and globals from the engine
extern "C" {
    // From tr_local.h and related headers
    fn Hunk_Alloc(size: usize, zero: i32) -> *mut c_void;
    fn COM_StripExtension(in_: *const c_char, out: *mut c_char);
    fn COM_ParseExt(p: *mut *const c_char, allow_line_breaks: i32) -> *const c_char;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn FS_ReadFile(name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buf: *mut c_void);
    fn VID_Printf(level: c_int, fmt: *const c_char, ...);
    fn Z_Malloc(size: usize, tag: c_int, zero: i32) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
    fn floor(x: f32) -> f32;
    fn ceil(x: f32) -> f32;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // tr global state
    static mut tr: TR;

    // shader functions
    fn R_FindShader(
        name: *const c_char,
        lightmapNum: *const c_int,
        styles: *const u8,
        forceLoad: i32,
    ) -> *mut shader_t;
    fn RE_RegisterShader(name: *const c_char) -> u32;
    fn R_CreateImage(
        name: *const c_char,
        pic: *const u8,
        width: c_int,
        height: c_int,
        format: c_int,
        mipmap: i32,
        picmip: i32,
        compress: c_int,
        wrapClampMode: c_int,
    ) -> *mut image_t;
    fn R_AllocModel() -> *mut model_t;
    fn R_SubdividePatchToGrid(width: c_int, height: c_int, points: *const drawVert_t) -> *mut srfGridMesh_t;
    fn RE_InsertModelIntoHash(name: *const c_char, mod_: *mut model_t);

    // math functions
    fn VectorNormalize(v: *mut f32) -> f32;
    fn VectorAdd(a: *const f32, b: *const f32, out: *mut f32);
    fn VectorScale(in_: *const f32, scale: f32, out: *mut f32);
    fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
    fn VectorLength(v: *const f32) -> f32;
    fn VectorSet(v: *mut f32, x: f32, y: f32, z: f32);
    fn DotProduct(a: *const f32, b: *const f32) -> f32;
    fn ClearBounds(mins: *mut f32, maxs: *mut f32);
    fn AddPointToBounds(v: *const f32, mins: *mut f32, maxs: *mut f32);
    fn PlaneTypeForNormal(normal: *const f32) -> c_int;
    fn SetPlaneSignbits(plane: *mut cplane_t);
    fn LittleLong(l: c_int) -> c_int;
    fn LittleFloat(f: f32) -> f32;
    fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> u32;

    // globals from other modules
    static mut skyboxportal: c_int;
    static gbUsingCachedMapDataRightNow: i32;
    static mut gpvCachedMapDiskImage: *mut c_void;
    static gsCachedMapDiskImage: [c_char; 260];

    // va function
    fn va(fmt: *const c_char, ...) -> *const c_char;

    // vertex light maps
    static lightmapsVertex: [c_int; 4];
    static lightmapsFullBright: [c_int; 4];
    static stylesDefault: [u8; 4];
}

// Type definitions from engine headers
#[repr(C)]
pub struct world_t {
    // Placeholder - actual fields from tr_local.h
}

#[repr(C)]
pub struct TR {
    // Placeholder - actual fields
}

#[repr(C)]
pub struct shader_t {
    // Placeholder
}

#[repr(C)]
pub struct image_t {
    // Placeholder
}

#[repr(C)]
pub struct model_t {
    // Placeholder
}

#[repr(C)]
pub struct msurface_t {
    // Placeholder
}

#[repr(C)]
pub struct drawVert_t {
    // Placeholder
}

#[repr(C)]
pub struct srfGridMesh_t {
    // Placeholder
}

#[repr(C)]
pub struct srfSurfaceFace_t {
    // Placeholder
}

#[repr(C)]
pub struct srfTriangles_t {
    // Placeholder
}

#[repr(C)]
pub struct srfFlare_t {
    // Placeholder
}

#[repr(C)]
pub struct cplane_t {
    // Placeholder
}

#[repr(C)]
pub struct dshader_t {
    // Placeholder
}

#[repr(C)]
pub struct dmodel_t {
    // Placeholder
}

#[repr(C)]
pub struct dnode_t {
    // Placeholder
}

#[repr(C)]
pub struct dleaf_t {
    // Placeholder
}

#[repr(C)]
pub struct mnode_t {
    // Placeholder
}

#[repr(C)]
pub struct dsurface_t {
    // Placeholder
}

#[repr(C)]
pub struct mapVert_t {
    // Placeholder
}

#[repr(C)]
pub struct lump_t {
    // Placeholder
}

#[repr(C)]
pub struct dplane_t {
    // Placeholder
}

#[repr(C)]
pub struct dheader_t {
    // Placeholder
}

#[repr(C)]
pub struct dfog_t {
    // Placeholder
}

#[repr(C)]
pub struct dbrush_t {
    // Placeholder
}

#[repr(C)]
pub struct dbrushside_t {
    // Placeholder
}

#[repr(C)]
pub struct fog_t {
    // Placeholder
}

#[repr(C)]
pub struct mgrid_t {
    // Placeholder
}

#[repr(C)]
pub struct bmodel_t {
    // Placeholder
}

#[repr(C)]
pub struct surfaceType_t {
    // Placeholder
}

// Constants
const LIGHTMAP_SIZE: usize = 128;
const MAX_PATCH_SIZE: usize = 32;
const MAX_QPATH: usize = 64;
const MAX_TOKEN_CHARS: usize = 1024;
const MAXLIGHTMAPS: usize = 4;
const SHADER_MAX_VERTEXES: c_int = 1000;
const SHADER_MAX_INDEXES: c_int = 6000;
const TAG_HUNKMISCMODELS: c_int = 22;
const BSP_VERSION: c_int = 46;
const LUMP_SHADERS: usize = 0;
const LUMP_LIGHTMAPS: usize = 1;
const LUMP_PLANES: usize = 2;
const LUMP_ENTITIES: usize = 3;
const LUMP_NODES: usize = 4;
const LUMP_LEAFS: usize = 5;
const LUMP_LEAFSURFACES: usize = 6;
const LUMP_MODELS: usize = 7;
const LUMP_BRUSHES: usize = 8;
const LUMP_BRUSHSIDES: usize = 9;
const LUMP_VISIBILITY: usize = 10;
const LUMP_LIGHTGRID: usize = 11;
const LUMP_SURFACES: usize = 12;
const LUMP_DRAWVERTS: usize = 13;
const LUMP_DRAWINDEXES: usize = 14;
const LUMP_FOGS: usize = 15;
const LUMP_LIGHTARRAY: usize = 16;
const LIGHTMAP_BY_VERTEX: c_int = -1;
const LIGHTMAP_NONE: c_int = -2;
const CONTENTS_NODE: c_int = -3;
const PRINT_ALL: c_int = 1;
const PRINT_WARNING: c_int = 2;
const ERR_DROP: c_int = 2;
const GL_RGBA: c_int = 6408;
const GL_CLAMP: c_int = 0x2900;
const MST_PATCH: c_int = 2;
const MST_TRIANGLE_SOUP: c_int = 3;
const MST_PLANAR: c_int = 1;
const MST_FLARE: c_int = 4;
const SF_FACE: c_int = 1;
const SF_GRID: c_int = 2;
const SF_TRIANGLES: c_int = 3;
const SF_FLARE: c_int = 4;
const SF_SKIP: c_int = 5;
const SURF_NODRAW: u32 = 0x80;
const MIN_WORLD_COORD: f32 = -65536.0;
const MAX_WORLD_COORD: f32 = 65536.0;
const VERTEX_LM: usize = 5;
const VERTEX_COLOR: usize = 13;

// Static globals
static mut s_worldData: world_t = unsafe { core::mem::zeroed() };
static mut fileBase: *mut u8 = core::ptr::null_mut();

pub static mut c_subdivisions: c_int = 0;
pub static mut c_gridVerts: c_int = 0;

extern "C" {
    fn R_RMGInit();
}

//===============================================================================

unsafe fn HSVtoRGB(h: f32, s: f32, v: f32, rgb: *mut f32) {
    let mut i: c_int;
    let mut f: f32;
    let mut p: f32;
    let mut q: f32;
    let mut t: f32;

    let mut h_mut = h * 5.0;

    i = floor(h_mut) as c_int;
    f = h_mut - i as f32;

    p = v * (1.0 - s);
    q = v * (1.0 - s * f);
    t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => {
            *rgb.offset(0) = v;
            *rgb.offset(1) = t;
            *rgb.offset(2) = p;
        }
        1 => {
            *rgb.offset(0) = q;
            *rgb.offset(1) = v;
            *rgb.offset(2) = p;
        }
        2 => {
            *rgb.offset(0) = p;
            *rgb.offset(1) = v;
            *rgb.offset(2) = t;
        }
        3 => {
            *rgb.offset(0) = p;
            *rgb.offset(1) = q;
            *rgb.offset(2) = v;
        }
        4 => {
            *rgb.offset(0) = t;
            *rgb.offset(1) = p;
            *rgb.offset(2) = v;
        }
        5 => {
            *rgb.offset(0) = v;
            *rgb.offset(1) = p;
            *rgb.offset(2) = q;
        }
        _ => {}
    }
}

/*
===============
R_ColorShiftLightingBytes

===============
*/
unsafe fn R_ColorShiftLightingBytes_4(in_: *const u8, out: *mut u8) {
    let mut shift: c_int = 0;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;

    // should NOT do it if overbrightBits is 0
    if (*core::ptr::addr_of!(tr)).overbrightBits != 0 {
        shift = 1 - (*core::ptr::addr_of!(tr)).overbrightBits;
    }

    if shift == 0 {
        *out.offset(0) = *in_.offset(0);
        *out.offset(1) = *in_.offset(1);
        *out.offset(2) = *in_.offset(2);
        *out.offset(3) = *in_.offset(3);
        return;
    }

    // shift the data based on overbright range
    r = (*in_.offset(0) as c_int) << shift;
    g = (*in_.offset(1) as c_int) << shift;
    b = (*in_.offset(2) as c_int) << shift;

    // normalize by color instead of saturating to white
    if (r | g | b) > 255 {
        let mut max: c_int;

        max = if r > g { r } else { g };
        max = if max > b { max } else { b };
        r = r * 255 / max;
        g = g * 255 / max;
        b = b * 255 / max;
    }

    *out.offset(0) = r as u8;
    *out.offset(1) = g as u8;
    *out.offset(2) = b as u8;
    *out.offset(3) = *in_.offset(3);
}

/*
===============
R_ColorShiftLightingBytes

===============
*/
unsafe fn R_ColorShiftLightingBytes_3(in_: *mut u8) {
    let mut shift: c_int = 0;
    let mut r: c_int;
    let mut g: c_int;
    let mut b: c_int;

    // should NOT do it if overbrightBits is 0
    if (*core::ptr::addr_of!(tr)).overbrightBits != 0 {
        shift = 1 - (*core::ptr::addr_of!(tr)).overbrightBits;
    }

    if shift == 0 {
        return; //no need if not overbright
    }
    // shift the data based on overbright range
    r = (*in_.offset(0) as c_int) << shift;
    g = (*in_.offset(1) as c_int) << shift;
    b = (*in_.offset(2) as c_int) << shift;

    // normalize by color instead of saturating to white
    if (r | g | b) > 255 {
        let mut max: c_int;

        max = if r > g { r } else { g };
        max = if max > b { max } else { b };
        r = r * 255 / max;
        g = g * 255 / max;
        b = b * 255 / max;
    }

    *in_.offset(0) = r as u8;
    *in_.offset(1) = g as u8;
    *in_.offset(2) = b as u8;
}


/*
===============
R_LoadLightmaps

===============
*/
unsafe fn R_LoadLightmaps(
    l: *mut lump_t,
    psMapName: *const c_char,
    worldData: &mut world_t,
) {
    let mut buf: *mut u8;
    let mut buf_p: *mut u8;
    let mut len: c_int;
    let mut image: [u8; LIGHTMAP_SIZE * LIGHTMAP_SIZE * 4] = [0; LIGHTMAP_SIZE * LIGHTMAP_SIZE * 4];
    let mut i: c_int;
    let mut j: c_int;
    let mut maxIntensity: f32 = 0.0;
    let mut sumIntensity: f64 = 0.0;
    let mut count: c_int;

    if (worldData as *mut _ as *const _) == (&s_worldData as *const _) {
        (*core::ptr::addr_of_mut!(tr)).numLightmaps = 0;
    }

    len = (*l).filelen as c_int;
    if len == 0 {
        return;
    }
    buf = fileBase.add((*l).fileofs as usize);

    // we are about to upload textures
    //R_SyncRenderThread();

    // create all the lightmaps
    (*worldData).startLightMapIndex = (*core::ptr::addr_of!(tr)).numLightmaps;
    count = len / (LIGHTMAP_SIZE as c_int * LIGHTMAP_SIZE as c_int * 3);
    (*core::ptr::addr_of_mut!(tr)).numLightmaps += count;

    // if we are in r_vertexLight mode, we don't need the lightmaps at all
    if (*core::ptr::addr_of!(tr)).r_vertexLight != 0 {
        return;
    }

    let mut sMapName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    COM_StripExtension(psMapName, sMapName.as_mut_ptr()); // will already by MAX_QPATH legal, so no length check

    i = 0;
    while i < count {
        // expand the 24 bit on-disk to 32 bit
        buf_p = buf.add((i as usize) * LIGHTMAP_SIZE * LIGHTMAP_SIZE * 3);

        if (*core::ptr::addr_of!(tr)).r_lightmap == 2 {
            // color code by intensity as development tool	(FIXME: check range)
            j = 0;
            while j < (LIGHTMAP_SIZE * LIGHTMAP_SIZE) as c_int {
                let r: f32 = *buf_p.offset((j as usize * 3 + 0) as isize) as f32;
                let g: f32 = *buf_p.offset((j as usize * 3 + 1) as isize) as f32;
                let b: f32 = *buf_p.offset((j as usize * 3 + 2) as isize) as f32;
                let mut intensity: f32;
                let mut out: [f32; 3] = [0.0; 3];

                intensity = 0.33 * r + 0.685 * g + 0.063 * b;

                if intensity > 255.0 {
                    intensity = 1.0;
                } else {
                    intensity /= 255.0;
                }

                if intensity > maxIntensity {
                    maxIntensity = intensity;
                }

                HSVtoRGB(intensity, 1.00, 0.50, out.as_mut_ptr());

                image[(j as usize) * 4 + 0] = (out[0] * 255.0) as u8;
                image[(j as usize) * 4 + 1] = (out[1] * 255.0) as u8;
                image[(j as usize) * 4 + 2] = (out[2] * 255.0) as u8;
                image[(j as usize) * 4 + 3] = 255;

                sumIntensity += intensity as f64;
                j += 1;
            }
        } else {
            j = 0;
            while j < (LIGHTMAP_SIZE * LIGHTMAP_SIZE) as c_int {
                R_ColorShiftLightingBytes_4(
                    buf_p.add((j as usize) * 3),
                    image.as_mut_ptr().add((j as usize) * 4),
                );
                image[(j as usize) * 4 + 3] = 255;
                j += 1;
            }
        }

        let lightmap_name = va(
            "\\x24%s/lightmap%d\0".as_ptr() as *const c_char,
            sMapName.as_ptr(),
            (*worldData).startLightMapIndex + i,
        );
        (*core::ptr::addr_of_mut!(tr)).lightmaps[((*worldData).startLightMapIndex + i) as usize] =
            R_CreateImage(
                lightmap_name,
                image.as_ptr(),
                LIGHTMAP_SIZE as c_int,
                LIGHTMAP_SIZE as c_int,
                GL_RGBA,
                0,
                0,
                (*core::ptr::addr_of!(tr)).r_ext_compressed_lightmaps,
                GL_CLAMP,
            );

        i += 1;
    }

    if (*core::ptr::addr_of!(tr)).r_lightmap == 2 {
        VID_Printf(
            PRINT_ALL,
            "Brightest lightmap value: %d\n\0".as_ptr() as *const c_char,
            (maxIntensity * 255.0) as c_int,
        );
    }
}



/*
=================
RE_SetWorldVisData

This is called by the clipmodel subsystem so we can share the 1.8 megs of
space in big maps...
=================
*/
pub unsafe fn RE_SetWorldVisData(vis: *const u8) {
    (*core::ptr::addr_of_mut!(tr)).externalVisData = vis;
}


/*
=================
R_LoadVisibility
=================
*/
unsafe fn R_LoadVisibility(l: *mut lump_t, worldData: &mut world_t) {
    let mut len: c_int;
    let mut buf: *mut u8;

    len = (((*worldData).numClusters + 63) & !63) as c_int;
    (*worldData).novis = Hunk_Alloc(len as usize, 0) as *mut u8;
    memset((*worldData).novis as *mut c_void, 0xff, len as usize);

    len = (*l).filelen as c_int;
    if len == 0 {
        return;
    }
    buf = fileBase.add((*l).fileofs as usize);

    (*worldData).numClusters = LittleLong(*(buf as *const c_int));
    (*worldData).clusterBytes = LittleLong(*((buf as *const c_int).offset(1)));

    // CM_Load should have given us the vis data to share, so
    // we don't need to allocate another copy
    if (*core::ptr::addr_of!(tr)).externalVisData as *const _ as usize != 0 {
        (*worldData).vis = (*core::ptr::addr_of!(tr)).externalVisData;
    } else {
        let mut dest: *mut u8;

        dest = Hunk_Alloc((len - 8) as usize, 0) as *mut u8;
        memcpy(
            dest as *mut c_void,
            buf.add(8) as *const c_void,
            (len - 8) as usize,
        );
        (*worldData).vis = dest;
    }
}

//===============================================================================

pub unsafe fn R_GetShaderByNum(shaderNum: c_int, worldData: &mut world_t) -> u32 {
    let mut shader: u32;

    if shaderNum < 0 || shaderNum >= (*worldData).numShaders {
        Com_Printf(
            "Warning: Bad index for R_GetShaderByNum - %i\0".as_ptr() as *const c_char,
            shaderNum,
        );
        return 0;
    }
    shader = RE_RegisterShader(
        ((*worldData).shaders as *const dshader_t)
            .add(shaderNum as usize) as *const c_char,
    );
    return shader;
}

/*
===============
ShaderForShaderNum
===============
*/
unsafe fn ShaderForShaderNum(
    shaderNum: c_int,
    lightmapNum: *const c_int,
    lightmapStyles: *const u8,
    vertexStyles: *const u8,
    worldData: &mut world_t,
) -> *mut shader_t {
    let mut shader: *mut shader_t;
    let mut dsh: *mut dshader_t;
    let mut styles: *const u8;

    styles = lightmapStyles;

    let mut shaderNum_mut = LittleLong(shaderNum);
    if shaderNum_mut < 0 || shaderNum_mut >= (*worldData).numShaders {
        Com_Error(
            ERR_DROP,
            "ShaderForShaderNum: bad num %i\0".as_ptr() as *const c_char,
            shaderNum_mut,
        );
    }
    dsh = ((*worldData).shaders as *mut dshader_t).add(shaderNum_mut as usize);

    if *lightmapNum == LIGHTMAP_BY_VERTEX {
        styles = vertexStyles;
    }

    if (*core::ptr::addr_of!(tr)).r_vertexLight != 0 {
        // lightmapNum = lightmapsVertex;
        styles = vertexStyles;
    }

    /*	if ( r_fullbright->integer )
    {
        lightmapNum = lightmapsFullBright;
        styles = vertexStyles;
    }
    */
    shader = R_FindShader(
        (*dsh).shader.as_ptr(),
        lightmapNum,
        styles,
        1,
    );

    // if the shader had errors, just use default shader
    if (*shader).defaultShader != 0 {
        return (*core::ptr::addr_of!(tr)).defaultShader;
    }

    return shader;
}

/*
===============
ParseFace
===============
*/
unsafe fn ParseFace(
    ds: *mut dsurface_t,
    verts: *mut mapVert_t,
    surf: *mut msurface_t,
    indexes: *mut c_int,
    pFaceDataBuffer: &mut *mut u8,
    worldData: &mut world_t,
    index: c_int,
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut cv: *mut srfSurfaceFace_t;
    let mut numPoints: c_int;
    let mut numIndexes: c_int;
    let mut lightmapNum: [c_int; MAXLIGHTMAPS] = [0; MAXLIGHTMAPS];
    let mut sfaceSize: c_int;
    let mut ofsIndexes: c_int;

    i = 0;
    while i < MAXLIGHTMAPS as c_int {
        lightmapNum[i as usize] = LittleLong(*((*ds).lightmapNum.as_ptr().add(i as usize)));
        if lightmapNum[i as usize] >= 0 {
            lightmapNum[i as usize] += (*worldData).startLightMapIndex;
        }
        i += 1;
    }

    // get fog volume
    (*surf).fogIndex = LittleLong((*ds).fogNum) + 1;
    if index != 0 && (*surf).fogIndex == 0 && (*core::ptr::addr_of!(tr)).world as usize != 0
        && (*(*core::ptr::addr_of!(tr)).world).globalFog != -1
    {
        (*surf).fogIndex = (*worldData).globalFog;
    }

    // get shader value
    (*surf).shader = ShaderForShaderNum(
        (*ds).shaderNum,
        lightmapNum.as_ptr(),
        (*ds).lightmapStyles.as_ptr(),
        (*ds).vertexStyles.as_ptr(),
        worldData,
    );
    if (*core::ptr::addr_of!(tr)).r_singleShader != 0 && (*(*surf).shader).sky == 0 {
        (*surf).shader = (*core::ptr::addr_of!(tr)).defaultShader;
    }

    numPoints = LittleLong((*ds).numVerts);
    numIndexes = LittleLong((*ds).numIndexes);

    // create the srfSurfaceFace_t
    sfaceSize = core::mem::size_of::<srfSurfaceFace_t>() as c_int; // approximate: &((srfSurfaceFace_t *)0)->points[numPoints]
    ofsIndexes = sfaceSize;
    sfaceSize += core::mem::size_of::<c_int>() as c_int * numIndexes;

    cv = *pFaceDataBuffer as *mut srfSurfaceFace_t;
    *pFaceDataBuffer = pFaceDataBuffer.add(sfaceSize as usize);

    (*cv).surfaceType = SF_FACE;
    (*cv).numPoints = numPoints;
    (*cv).numIndices = numIndexes;
    (*cv).ofsIndices = ofsIndexes;

    let verts_offset = LittleLong((*ds).firstVert);
    let verts_ptr = verts.add(verts_offset as usize);

    i = 0;
    while i < numPoints {
        j = 0;
        while j < 3 {
            // (*cv).points[i][j] = LittleFloat((*verts_ptr.add(i as usize)).xyz[j as usize]);
            j += 1;
        }
        j = 0;
        while j < 2 {
            // (*cv).points[i][3 + j] = LittleFloat((*verts_ptr.add(i as usize)).st[j as usize]);
            k = 0;
            while k < MAXLIGHTMAPS as c_int {
                // (*cv).points[i][VERTEX_LM + j + (k * 2)] = LittleFloat((*verts_ptr.add(i as usize)).lightmap[k as usize][j as usize]);
                k += 1;
            }
            j += 1;
        }
        k = 0;
        while k < MAXLIGHTMAPS as c_int {
            R_ColorShiftLightingBytes_4(
                (*verts_ptr.add(i as usize)).color[k as usize].as_ptr(),
                (*cv).points.as_mut_ptr().add(i as usize) as *mut u8,
            );
            k += 1;
        }
        i += 1;
    }

    let indexes_offset = LittleLong((*ds).firstIndex);
    let indexes_ptr = indexes.add(indexes_offset as usize);

    i = 0;
    while i < numIndexes {
        *(((*cv).ofsIndices as *mut u8).add((*cv).ofsIndices as usize) as *mut c_int).add(i as usize) =
            LittleLong(*indexes_ptr.add(i as usize));
        i += 1;
    }

    // take the plane information from the lightmap vector
    i = 0;
    while i < 3 {
        // (*cv).plane.normal[i as usize] = LittleFloat((*ds).lightmapVecs[2][i as usize]);
        i += 1;
    }
    // (*cv).plane.dist = DotProduct((*cv).points[0].as_ptr(), (*cv).plane.normal.as_ptr());
    SetPlaneSignbits(&mut (*cv).plane);
    // (*cv).plane.type = PlaneTypeForNormal((*cv).plane.normal.as_ptr());

    (*surf).data = cv as *mut surfaceType_t;
}


/*
===============
ParseMesh
===============
*/
unsafe fn ParseMesh(
    ds: *mut dsurface_t,
    verts: *mut mapVert_t,
    surf: *mut msurface_t,
    worldData: &mut world_t,
    index: c_int,
) {
    let mut grid: *mut srfGridMesh_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut width: c_int;
    let mut height: c_int;
    let mut numPoints: c_int;
    let mut points: [drawVert_t; MAX_PATCH_SIZE * MAX_PATCH_SIZE] = unsafe { core::mem::zeroed() };
    let mut lightmapNum: [c_int; MAXLIGHTMAPS] = [0; MAXLIGHTMAPS];
    let mut bounds: [[f32; 3]; 2] = [[0.0; 3]; 2];
    let mut tmpVec: [f32; 3] = [0.0; 3];
    static skipData: surfaceType_t = unsafe { core::mem::zeroed() };

    i = 0;
    while i < MAXLIGHTMAPS as c_int {
        lightmapNum[i as usize] = LittleLong(*((*ds).lightmapNum.as_ptr().add(i as usize)));
        if lightmapNum[i as usize] >= 0 {
            lightmapNum[i as usize] += (*worldData).startLightMapIndex;
        }
        i += 1;
    }

    // get fog volume
    (*surf).fogIndex = LittleLong((*ds).fogNum) + 1;
    if index != 0 && (*surf).fogIndex == 0 && (*core::ptr::addr_of!(tr)).world as usize != 0
        && (*(*core::ptr::addr_of!(tr)).world).globalFog != -1
    {
        (*surf).fogIndex = (*worldData).globalFog;
    }

    // get shader value
    (*surf).shader = ShaderForShaderNum(
        (*ds).shaderNum,
        lightmapNum.as_ptr(),
        (*ds).lightmapStyles.as_ptr(),
        (*ds).vertexStyles.as_ptr(),
        worldData,
    );
    if (*core::ptr::addr_of!(tr)).r_singleShader != 0 && (*(*surf).shader).sky == 0 {
        (*surf).shader = (*core::ptr::addr_of!(tr)).defaultShader;
    }

    // we may have a nodraw surface, because they might still need to
    // be around for movement clipping
    if (*((*worldData).shaders as *const dshader_t)
        .add(LittleLong((*ds).shaderNum) as usize))
    .surfaceFlags
        & SURF_NODRAW
        != 0
    {
        (*surf).data = &skipData as *const _ as *mut _;
        return;
    }

    width = LittleLong((*ds).patchWidth);
    height = LittleLong((*ds).patchHeight);

    let verts_offset = LittleLong((*ds).firstVert);
    let verts_ptr = verts.add(verts_offset as usize);

    numPoints = width * height;
    i = 0;
    while i < numPoints {
        j = 0;
        while j < 3 {
            // points[i as usize].xyz[j as usize] = LittleFloat((*verts_ptr.add(i as usize)).xyz[j as usize]);
            // points[i as usize].normal[j as usize] = LittleFloat((*verts_ptr.add(i as usize)).normal[j as usize]);
            j += 1;
        }
        j = 0;
        while j < 2 {
            // points[i as usize].st[j as usize] = LittleFloat((*verts_ptr.add(i as usize)).st[j as usize]);
            k = 0;
            while k < MAXLIGHTMAPS as c_int {
                // points[i as usize].lightmap[k as usize][j as usize] =
                //     LittleFloat((*verts_ptr.add(i as usize)).lightmap[k as usize][j as usize]);
                k += 1;
            }
            j += 1;
        }
        k = 0;
        while k < MAXLIGHTMAPS as c_int {
            R_ColorShiftLightingBytes_4(
                (*verts_ptr.add(i as usize)).color[k as usize].as_ptr(),
                points[i as usize].color[k as usize].as_mut_ptr(),
            );
            k += 1;
        }
        i += 1;
    }

    // pre-tesseleate
    grid = R_SubdividePatchToGrid(width, height, points.as_ptr());
    (*surf).data = grid as *mut surfaceType_t;

    // copy the level of detail origin, which is the center
    // of the group of all curves that must subdivide the same
    // to avoid cracking
    i = 0;
    while i < 3 {
        bounds[0][i as usize] = LittleFloat((*ds).lightmapVecs[0][i as usize]);
        bounds[1][i as usize] = LittleFloat((*ds).lightmapVecs[1][i as usize]);
        i += 1;
    }
    VectorAdd(bounds[0].as_ptr(), bounds[1].as_ptr(), bounds[1].as_mut_ptr());
    VectorScale(bounds[1].as_ptr(), 0.5, (*grid).lodOrigin.as_mut_ptr());
    VectorSubtract(
        bounds[0].as_ptr(),
        (*grid).lodOrigin.as_ptr(),
        tmpVec.as_mut_ptr(),
    );
    (*grid).lodRadius = VectorLength(tmpVec.as_ptr());
}

/*
===============
ParseTriSurf
===============
*/
unsafe fn ParseTriSurf(
    ds: *mut dsurface_t,
    verts: *mut mapVert_t,
    surf: *mut msurface_t,
    indexes: *mut c_int,
    worldData: &mut world_t,
    index: c_int,
) {
    let mut tri: *mut srfTriangles_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut numVerts: c_int;
    let mut numIndexes: c_int;

    // get fog volume
    (*surf).fogIndex = LittleLong((*ds).fogNum) + 1;
    if index != 0 && (*surf).fogIndex == 0 && (*core::ptr::addr_of!(tr)).world as usize != 0
        && (*(*core::ptr::addr_of!(tr)).world).globalFog != -1
    {
        (*surf).fogIndex = (*worldData).globalFog;
    }

    // get shader
    (*surf).shader = ShaderForShaderNum(
        (*ds).shaderNum,
        lightmapsVertex.as_ptr(),
        (*ds).lightmapStyles.as_ptr(),
        (*ds).vertexStyles.as_ptr(),
        worldData,
    );
    if (*core::ptr::addr_of!(tr)).r_singleShader != 0 && (*(*surf).shader).sky == 0 {
        (*surf).shader = (*core::ptr::addr_of!(tr)).defaultShader;
    }

    numVerts = LittleLong((*ds).numVerts);
    numIndexes = LittleLong((*ds).numIndexes);

    if numVerts >= SHADER_MAX_VERTEXES {
        Com_Error(
            ERR_DROP,
            "ParseTriSurf: verts > MAX (%d > %d) on misc_model %s\0".as_ptr() as *const c_char,
            numVerts,
            SHADER_MAX_VERTEXES,
            (*(*surf).shader).name.as_ptr(),
        );
    }
    if numIndexes >= SHADER_MAX_INDEXES {
        Com_Error(
            ERR_DROP,
            "ParseTriSurf: indices > MAX (%d > %d) on misc_model %s\0".as_ptr() as *const c_char,
            numIndexes,
            SHADER_MAX_INDEXES,
            (*(*surf).shader).name.as_ptr(),
        );
    }

    tri = Z_Malloc(
        core::mem::size_of::<srfTriangles_t>()
            + (numVerts as usize) * core::mem::size_of::<drawVert_t>()
            + (numIndexes as usize) * core::mem::size_of::<c_int>(),
        TAG_HUNKMISCMODELS,
        0,
    ) as *mut srfTriangles_t;
    (*tri).dlightBits = 0; //JIC
    (*tri).surfaceType = SF_TRIANGLES;
    (*tri).numVerts = numVerts;
    (*tri).numIndexes = numIndexes;
    (*tri).verts = (tri as *mut u8).add(core::mem::size_of::<srfTriangles_t>()) as *mut drawVert_t;
    (*tri).indexes = ((*tri).verts as *mut u8).add((numVerts as usize) * core::mem::size_of::<drawVert_t>())
        as *mut c_int;

    (*surf).data = tri as *mut surfaceType_t;

    // copy vertexes
    let verts_offset = LittleLong((*ds).firstVert);
    let verts_ptr = verts.add(verts_offset as usize);

    ClearBounds((*tri).bounds[0].as_mut_ptr(), (*tri).bounds[1].as_mut_ptr());
    i = 0;
    while i < numVerts {
        j = 0;
        while j < 3 {
            // (*tri).verts[i as usize].xyz[j as usize] = LittleFloat((*verts_ptr.add(i as usize)).xyz[j as usize]);
            // (*tri).verts[i as usize].normal[j as usize] = LittleFloat((*verts_ptr.add(i as usize)).normal[j as usize]);
            j += 1;
        }
        AddPointToBounds(
            (*(*tri).verts.add(i as usize)).xyz.as_ptr(),
            (*tri).bounds[0].as_mut_ptr(),
            (*tri).bounds[1].as_mut_ptr(),
        );
        j = 0;
        while j < 2 {
            // (*tri).verts[i as usize].st[j as usize] = LittleFloat((*verts_ptr.add(i as usize)).st[j as usize]);
            k = 0;
            while k < MAXLIGHTMAPS as c_int {
                // (*tri).verts[i as usize].lightmap[k as usize][j as usize] =
                //     LittleFloat((*verts_ptr.add(i as usize)).lightmap[k as usize][j as usize]);
                k += 1;
            }
            j += 1;
        }
        k = 0;
        while k < MAXLIGHTMAPS as c_int {
            R_ColorShiftLightingBytes_4(
                (*verts_ptr.add(i as usize)).color[k as usize].as_ptr(),
                (*(*tri).verts.add(i as usize)).color[k as usize].as_mut_ptr(),
            );
            k += 1;
        }
        i += 1;
    }

    // copy indexes
    let indexes_offset = LittleLong((*ds).firstIndex);
    let indexes_ptr = indexes.add(indexes_offset as usize);

    i = 0;
    while i < numIndexes {
        *(*tri).indexes.add(i as usize) = LittleLong(*indexes_ptr.add(i as usize));
        if *(*tri).indexes.add(i as usize) < 0
            || *(*tri).indexes.add(i as usize) >= numVerts
        {
            Com_Error(
                ERR_DROP,
                "Bad index in triangle surface\0".as_ptr() as *const c_char,
            );
        }
        i += 1;
    }
}

/*
===============
ParseFlare
===============
*/
unsafe fn ParseFlare(
    ds: *mut dsurface_t,
    verts: *mut mapVert_t,
    surf: *mut msurface_t,
    indexes: *mut c_int,
    worldData: &mut world_t,
    index: c_int,
) {
    let mut flare: *mut srfFlare_t;
    let mut i: c_int;
    let lightmaps: [c_int; MAXLIGHTMAPS] = [LIGHTMAP_BY_VERTEX, 0, 0, 0];

    // get fog volume
    (*surf).fogIndex = LittleLong((*ds).fogNum) + 1;
    if index != 0 && (*surf).fogIndex == 0 && (*core::ptr::addr_of!(tr)).world as usize != 0
        && (*(*core::ptr::addr_of!(tr)).world).globalFog != -1
    {
        (*surf).fogIndex = (*worldData).globalFog;
    }

    // get shader
    (*surf).shader = ShaderForShaderNum(
        (*ds).shaderNum,
        lightmaps.as_ptr(),
        (*ds).lightmapStyles.as_ptr(),
        (*ds).vertexStyles.as_ptr(),
        worldData,
    );
    if (*core::ptr::addr_of!(tr)).r_singleShader != 0 && (*(*surf).shader).sky == 0 {
        (*surf).shader = (*core::ptr::addr_of!(tr)).defaultShader;
    }

    flare = Hunk_Alloc(core::mem::size_of::<srfFlare_t>(), 1) as *mut srfFlare_t;
    (*flare).surfaceType = SF_FLARE;

    (*surf).data = flare as *mut surfaceType_t;

    i = 0;
    while i < 3 {
        // (*flare).origin[i as usize] = LittleFloat((*ds).lightmapOrigin[i as usize]);
        // (*flare).color[i as usize] = LittleFloat((*ds).lightmapVecs[0][i as usize]);
        // (*flare).normal[i as usize] = LittleFloat((*ds).lightmapVecs[2][i as usize]);
        i += 1;
    }
}

/*
===============
R_LoadSurfaces
===============
*/
unsafe fn R_LoadSurfaces(
    surfs: *mut lump_t,
    verts: *mut lump_t,
    indexLump: *mut lump_t,
    worldData: &mut world_t,
    index: c_int,
) {
    let mut in_: *mut dsurface_t;
    let mut out: *mut msurface_t;
    let mut dv: *mut mapVert_t;
    let mut indexes: *mut c_int;
    let mut count: c_int;
    let mut numFaces: c_int = 0;
    let mut numMeshes: c_int = 0;
    let mut numTriSurfs: c_int = 0;
    let mut numFlares: c_int = 0;
    let mut i: c_int;

    in_ = (fileBase.add((*surfs).fileofs as usize)) as *mut dsurface_t;
    if (*surfs).filelen % core::mem::size_of::<dsurface_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    count = ((*surfs).filelen / core::mem::size_of::<dsurface_t>()) as c_int;

    dv = (fileBase.add((*verts).fileofs as usize)) as *mut mapVert_t;
    if (*verts).filelen % core::mem::size_of::<mapVert_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }

    indexes = (fileBase.add((*indexLump).fileofs as usize)) as *mut c_int;
    if (*indexLump).filelen % core::mem::size_of::<c_int>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }

    out = Hunk_Alloc((count as usize) * core::mem::size_of::<msurface_t>(), 1) as *mut msurface_t;

    (*worldData).surfaces = out;
    (*worldData).numsurfaces = count;

    // new bit, the face code on our biggest map requires over 15,000 mallocs, which was no problem on the hunk,
    //	bit hits the zone pretty bad (even the tagFree takes about 9 seconds for that many memblocks),
    //	so special-case pre-alloc enough space for this data (the patches etc can stay as they are)...
    //
    let mut iFaceDataSizeRequired: c_int = 0;
    i = 0;
    while i < count {
        match LittleLong((*in_.add(i as usize)).surfaceType) {
            MST_PLANAR => {
                let sfaceSize: c_int = (core::mem::size_of::<srfSurfaceFace_t>()) as c_int;
                let sfaceSize = sfaceSize + core::mem::size_of::<c_int>() as c_int * LittleLong((*in_.add(i as usize)).numIndexes);
                iFaceDataSizeRequired += sfaceSize;
            }
            _ => {}
        }
        i += 1;
    }

    // since this ptr is to hunk data, I can pass it in and have it advanced without worrying about losing
    //	the original alloc ptr...
    //
    let mut pFaceDataBuffer: *mut u8 = Hunk_Alloc(iFaceDataSizeRequired as usize, 1) as *mut u8;

    // now do regular loop...
    //
    i = 0;
    while i < count {
        match LittleLong((*in_).surfaceType) {
            MST_PATCH => {
                ParseMesh(in_, dv, out, worldData, index);
                numMeshes += 1;
            }
            MST_TRIANGLE_SOUP => {
                ParseTriSurf(in_, dv, out, indexes, worldData, index);
                numTriSurfs += 1;
            }
            MST_PLANAR => {
                ParseFace(in_, dv, out, indexes, &mut pFaceDataBuffer, worldData, index);
                numFaces += 1;
            }
            MST_FLARE => {
                ParseFlare(in_, dv, out, indexes, worldData, index);
                numFlares += 1;
            }
            _ => {
                Com_Error(
                    ERR_DROP,
                    "Bad surfaceType\0".as_ptr() as *const c_char,
                );
            }
        }
        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }

    VID_Printf(
        PRINT_ALL,
        "...loaded %d faces, %i meshes, %i trisurfs, %i flares\n\0".as_ptr() as *const c_char,
        numFaces,
        numMeshes,
        numTriSurfs,
        numFlares,
    );
}



/*
=================
R_LoadSubmodels
=================
*/
unsafe fn R_LoadSubmodels(l: *mut lump_t, worldData: &mut world_t, index: c_int) {
    let mut in_: *mut dmodel_t;
    let mut out: *mut bmodel_t;
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;

    in_ = (fileBase.add((*l).fileofs as usize)) as *mut dmodel_t;
    if (*l).filelen % core::mem::size_of::<dmodel_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    count = ((*l).filelen / core::mem::size_of::<dmodel_t>()) as c_int;

    out = Hunk_Alloc((count as usize) * core::mem::size_of::<bmodel_t>(), 1) as *mut bmodel_t;
    (*worldData).bmodels = out;

    i = 0;
    while i < count {
        let mut model: *mut model_t;

        model = R_AllocModel();

        assert!(model as usize != 0); // this should never happen

        // (*model).type = MOD_BRUSH;
        (*model).bmodel = out;
        if index != 0 {
            Com_sprintf(
                (*model).name.as_mut_ptr(),
                core::mem::size_of_val(&(*model).name),
                "*%d-%d\0".as_ptr() as *const c_char,
                index,
                i,
            );
            // (*model).bspInstance = true;
        } else {
            Com_sprintf(
                (*model).name.as_mut_ptr(),
                core::mem::size_of_val(&(*model).name),
                "*%d\0".as_ptr() as *const c_char,
                i,
            );
        }

        j = 0;
        while j < 3 {
            (*out).bounds[0][j as usize] = LittleFloat((*in_).mins[j as usize]);
            (*out).bounds[1][j as usize] = LittleFloat((*in_).maxs[j as usize]);
            j += 1;
        }
        /*
        Ghoul2 Insert Start
        */

        RE_InsertModelIntoHash((*model).name.as_ptr(), model);
        /*
        Ghoul2 Insert End
        */

        (*out).firstSurface = (*worldData).surfaces.add(LittleLong((*in_).firstSurface) as usize);
        (*out).numSurfaces = LittleLong((*in_).numSurfaces);

        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}



//==================================================================

/*
=================
R_SetParent
=================
*/
unsafe fn R_SetParent(node: *mut mnode_t, parent: *mut mnode_t) {
    (*node).parent = parent;
    if (*node).contents != -1 {
        return;
    }
    R_SetParent((*node).children[0], node);
    R_SetParent((*node).children[1], node);
}

/*
=================
R_LoadNodesAndLeafs
=================
*/
unsafe fn R_LoadNodesAndLeafs(
    nodeLump: *mut lump_t,
    leafLump: *mut lump_t,
    worldData: &mut world_t,
) {
    let mut i: c_int;
    let mut j: c_int;
    let mut p: c_int;
    let mut in_: *mut dnode_t;
    let mut inLeaf: *mut dleaf_t;
    let mut out: *mut mnode_t;
    let mut numNodes: c_int;
    let mut numLeafs: c_int;

    in_ = (fileBase.add((*nodeLump).fileofs as usize)) as *mut dnode_t;
    if (*nodeLump).filelen % core::mem::size_of::<dnode_t>() != 0
        || (*leafLump).filelen % core::mem::size_of::<dleaf_t>() != 0
    {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    numNodes = ((*nodeLump).filelen / core::mem::size_of::<dnode_t>()) as c_int;
    numLeafs = ((*leafLump).filelen / core::mem::size_of::<dleaf_t>()) as c_int;

    out = Hunk_Alloc(
        ((numNodes + numLeafs) as usize) * core::mem::size_of::<mnode_t>(),
        1,
    ) as *mut mnode_t;

    (*worldData).nodes = out;
    (*worldData).numnodes = numNodes + numLeafs;
    (*worldData).numDecisionNodes = numNodes;

    // load nodes
    i = 0;
    while i < numNodes {
        j = 0;
        while j < 3 {
            (*out).mins[j as usize] = LittleLong((*in_).mins[j as usize]);
            (*out).maxs[j as usize] = LittleLong((*in_).maxs[j as usize]);
            j += 1;
        }

        p = LittleLong((*in_).planeNum);
        (*out).plane = (*worldData).planes.add(p as usize);

        (*out).contents = CONTENTS_NODE; // differentiate from leafs

        j = 0;
        while j < 2 {
            p = LittleLong((*in_).children[j as usize]);
            if p >= 0 {
                (*out).children[j as usize] = (*worldData).nodes.add(p as usize);
            } else {
                (*out).children[j as usize] =
                    (*worldData).nodes.add((numNodes + (-1 - p)) as usize);
            }
            j += 1;
        }

        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }

    // load leafs
    inLeaf = (fileBase.add((*leafLump).fileofs as usize)) as *mut dleaf_t;
    i = 0;
    while i < numLeafs {
        j = 0;
        while j < 3 {
            (*out).mins[j as usize] = LittleLong((*inLeaf).mins[j as usize]);
            (*out).maxs[j as usize] = LittleLong((*inLeaf).maxs[j as usize]);
            j += 1;
        }

        (*out).cluster = LittleLong((*inLeaf).cluster);
        (*out).area = LittleLong((*inLeaf).area);

        if (*out).cluster >= (*worldData).numClusters {
            (*worldData).numClusters = (*out).cluster + 1;
        }

        (*out).firstmarksurface = (*worldData).marksurfaces.add(LittleLong((*inLeaf).firstLeafSurface) as usize);
        (*out).nummarksurfaces = LittleLong((*inLeaf).numLeafSurfaces);

        inLeaf = inLeaf.add(1);
        out = out.add(1);
        i += 1;
    }

    // chain decendants
    R_SetParent((*worldData).nodes, core::ptr::null_mut());
}

//=============================================================================

/*
=================
R_LoadShaders
=================
*/
unsafe fn R_LoadShaders(l: *mut lump_t, worldData: &mut world_t) {
    let mut i: c_int;
    let mut count: c_int;
    let mut in_: *mut dshader_t;
    let mut out: *mut dshader_t;

    in_ = (fileBase.add((*l).fileofs as usize)) as *mut dshader_t;
    if (*l).filelen % core::mem::size_of::<dshader_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    count = ((*l).filelen / core::mem::size_of::<dshader_t>()) as c_int;
    out = Hunk_Alloc((count as usize) * core::mem::size_of::<dshader_t>(), 0) as *mut dshader_t;

    (*worldData).shaders = out;
    (*worldData).numShaders = count;

    memcpy(
        out as *mut c_void,
        in_ as *const c_void,
        (count as usize) * core::mem::size_of::<dshader_t>(),
    );

    i = 0;
    while i < count {
        (*out.add(i as usize)).surfaceFlags = LittleLong((*out.add(i as usize)).surfaceFlags);
        (*out.add(i as usize)).contentFlags = LittleLong((*out.add(i as usize)).contentFlags);
        i += 1;
    }
}


/*
=================
R_LoadMarksurfaces
=================
*/
unsafe fn R_LoadMarksurfaces(l: *mut lump_t, worldData: &mut world_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut count: c_int;
    let mut in_: *mut c_int;
    let mut out: *mut *mut msurface_t;

    in_ = (fileBase.add((*l).fileofs as usize)) as *mut c_int;
    if (*l).filelen % core::mem::size_of::<c_int>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    count = ((*l).filelen / core::mem::size_of::<c_int>()) as c_int;
    out = Hunk_Alloc((count as usize) * core::mem::size_of::<*mut msurface_t>(), 1) as *mut *mut msurface_t;

    (*worldData).marksurfaces = out;
    (*worldData).nummarksurfaces = count;

    i = 0;
    while i < count {
        j = LittleLong(*in_.add(i as usize));
        *out.add(i as usize) = (*worldData).surfaces.add(j as usize);
        i += 1;
    }
}


/*
=================
R_LoadPlanes
=================
*/
unsafe fn R_LoadPlanes(l: *mut lump_t, worldData: &mut world_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut out: *mut cplane_t;
    let mut in_: *mut dplane_t;
    let mut count: c_int;
    let mut bits: c_int;

    in_ = (fileBase.add((*l).fileofs as usize)) as *mut dplane_t;
    if (*l).filelen % core::mem::size_of::<dplane_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    count = ((*l).filelen / core::mem::size_of::<dplane_t>()) as c_int;
    out = Hunk_Alloc((count as usize) * 2 * core::mem::size_of::<cplane_t>(), 1) as *mut cplane_t;

    (*worldData).planes = out;
    (*worldData).numplanes = count;

    i = 0;
    while i < count {
        bits = 0;
        j = 0;
        while j < 3 {
            // (*out).normal[j as usize] = LittleFloat((*in_).normal[j as usize]);
            // if (*out).normal[j as usize] < 0.0 {
            //     bits |= 1 << j;
            // }
            j += 1;
        }

        // (*out).dist = LittleFloat((*in_).dist);
        // (*out).type = PlaneTypeForNormal((*out).normal.as_ptr());
        // (*out).signbits = bits;

        in_ = in_.add(1);
        out = out.add(1);
        i += 1;
    }
}

/*
=================
R_LoadFogs

=================
*/
unsafe fn R_LoadFogs(
    l: *mut lump_t,
    brushesLump: *mut lump_t,
    sidesLump: *mut lump_t,
    worldData: &mut world_t,
    index: c_int,
) {
    let mut i: c_int;
    let mut out: *mut fog_t;
    let mut fogs: *mut dfog_t;
    let mut brushes: *mut dbrush_t;
    let mut brush: *mut dbrush_t;
    let mut sides: *mut dbrushside_t;
    let mut count: c_int;
    let mut brushesCount: c_int;
    let mut sidesCount: c_int;
    let mut sideNum: c_int;
    let mut planeNum: c_int;
    let mut shader: *mut shader_t;
    let mut d: f32;
    let mut firstSide: c_int = 0;
    let lightmaps: [c_int; MAXLIGHTMAPS] = [LIGHTMAP_NONE, 0, 0, 0];

    fogs = (fileBase.add((*l).fileofs as usize)) as *mut dfog_t;
    if (*l).filelen % core::mem::size_of::<dfog_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    count = ((*l).filelen / core::mem::size_of::<dfog_t>()) as c_int;

    // create fog strucutres for them
    (*worldData).numfogs = count + 1;
    (*worldData).fogs = Hunk_Alloc(
        (((*worldData).numfogs + 1) as usize) * core::mem::size_of::<fog_t>(),
        1,
    ) as *mut fog_t;
    (*worldData).globalFog = -1;
    out = (*worldData).fogs.add(1);

    // Copy the global fog from the main world into the bsp instance
    if index != 0 {
        if (*core::ptr::addr_of!(tr)).world as usize != 0
            && ((*(*core::ptr::addr_of!(tr)).world).globalFog != -1)
        {
            // Use the nightvision fog slot
            *(*worldData).fogs.add((*worldData).numfogs as usize) =
                (*(*core::ptr::addr_of!(tr)).world)
                    .fogs[(*(*core::ptr::addr_of!(tr)).world).globalFog as usize];
            (*worldData).globalFog = (*worldData).numfogs;
            (*worldData).numfogs += 1;
        }
    }

    if count == 0 {
        return;
    }

    brushes = (fileBase.add((*brushesLump).fileofs as usize)) as *mut dbrush_t;
    if (*brushesLump).filelen % core::mem::size_of::<dbrush_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    brushesCount = ((*brushesLump).filelen / core::mem::size_of::<dbrush_t>()) as c_int;

    sides = (fileBase.add((*sidesLump).fileofs as usize)) as *mut dbrushside_t;
    if (*sidesLump).filelen % core::mem::size_of::<dbrushside_t>() != 0 {
        Com_Error(
            ERR_DROP,
            "LoadMap: funny lump size in %s\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
        );
    }
    sidesCount = ((*sidesLump).filelen / core::mem::size_of::<dbrushside_t>()) as c_int;

    i = 0;
    while i < count {
        (*out).originalBrushNumber = LittleLong((*fogs).brushNum);
        if (*out).originalBrushNumber == -1 {
            if index != 0 {
                Com_Error(
                    ERR_DROP,
                    "LoadMap: global fog not allowed in bsp instances - %s\0".as_ptr() as *const c_char,
                    (*core::ptr::addr_of!(tr)).worldDir.as_ptr(),
                );
            }
            VectorSet(
                (*out).bounds[0].as_mut_ptr(),
                MIN_WORLD_COORD,
                MIN_WORLD_COORD,
                MIN_WORLD_COORD,
            );
            VectorSet(
                (*out).bounds[1].as_mut_ptr(),
                MAX_WORLD_COORD,
                MAX_WORLD_COORD,
                MAX_WORLD_COORD,
            );
            (*worldData).globalFog = i + 1;
        } else {
            if ((*out).originalBrushNumber as u32) >= brushesCount as u32 {
                Com_Error(
                    ERR_DROP,
                    "fog brushNumber out of range\0".as_ptr() as *const c_char,
                );
            }
            brush = brushes.add((*out).originalBrushNumber as usize);

            firstSide = LittleLong((*brush).firstSide);

            if (firstSide as u32) > (sidesCount - 6) as u32 {
                Com_Error(
                    ERR_DROP,
                    "fog brush sideNumber out of range\0".as_ptr() as *const c_char,
                );
            }

            // brushes are always sorted with the axial sides first
            sideNum = firstSide + 0;
            planeNum = LittleLong((*sides.add(sideNum as usize)).planeNum);
            (*out).bounds[0][0] = -((*(*worldData).planes.add(planeNum as usize)).dist);

            sideNum = firstSide + 1;
            planeNum = LittleLong((*sides.add(sideNum as usize)).planeNum);
            (*out).bounds[1][0] = (*(*worldData).planes.add(planeNum as usize)).dist;

            sideNum = firstSide + 2;
            planeNum = LittleLong((*sides.add(sideNum as usize)).planeNum);
            (*out).bounds[0][1] = -((*(*worldData).planes.add(planeNum as usize)).dist);

            sideNum = firstSide + 3;
            planeNum = LittleLong((*sides.add(sideNum as usize)).planeNum);
            (*out).bounds[1][1] = (*(*worldData).planes.add(planeNum as usize)).dist;

            sideNum = firstSide + 4;
            planeNum = LittleLong((*sides.add(sideNum as usize)).planeNum);
            (*out).bounds[0][2] = -((*(*worldData).planes.add(planeNum as usize)).dist);

            sideNum = firstSide + 5;
            planeNum = LittleLong((*sides.add(sideNum as usize)).planeNum);
            (*out).bounds[1][2] = (*(*worldData).planes.add(planeNum as usize)).dist;
        }

        // get information from the shader for fog parameters
        shader = R_FindShader(
            (*fogs).shader.as_ptr(),
            lightmaps.as_ptr(),
            stylesDefault.as_ptr(),
            1,
        );

        assert!((*shader).fogParms as usize != 0);
        if (*shader).fogParms as usize == 0 {
            //bad shader!!
            (*out).parms.color[0] = 1.0;
            (*out).parms.color[1] = 0.0;
            (*out).parms.color[2] = 0.0;
            (*out).parms.color[3] = 0.0;
            (*out).parms.depthForOpaque = 250.0;
        } else {
            (*out).parms = *((*shader).fogParms);
        }
        (*out).colorInt = ColorBytes4(
            (*out).parms.color[0],
            (*out).parms.color[1],
            (*out).parms.color[2],
            1.0,
        );

        d = if (*out).parms.depthForOpaque < 1.0 {
            1.0
        } else {
            (*out).parms.depthForOpaque
        };
        (*out).tcScale = 1.0 / (d * 8.0);

        // set the gradient vector
        sideNum = LittleLong((*fogs).visibleSide);

        if sideNum == -1 {
            (*out).hasSurface = 0;
        } else {
            (*out).hasSurface = 1;
            planeNum = LittleLong((*sides.add((firstSide + sideNum) as usize)).planeNum);
            VectorSubtract(
                core::ptr::null(),
                (*(*worldData).planes.add(planeNum as usize)).normal.as_ptr(),
                (*out).surface.as_mut_ptr(),
            );
            (*out).surface[3] = -((*(*worldData).planes.add(planeNum as usize)).dist);
        }

        out = out.add(1);
        fogs = fogs.add(1);
        i += 1;
    }

    if index == 0 {
        // Initialise the last fog so we can use it with the LA Goggles
        // NOTE: We are might appear to be off the end of the array, but we allocated an extra memory slot above but [purposely] didn't
        //	increment the total world numFogs to match our array size
        VectorSet(
            (*out).bounds[0].as_mut_ptr(),
            MIN_WORLD_COORD,
            MIN_WORLD_COORD,
            MIN_WORLD_COORD,
        );
        VectorSet(
            (*out).bounds[1].as_mut_ptr(),
            MAX_WORLD_COORD,
            MAX_WORLD_COORD,
            MAX_WORLD_COORD,
        );
        (*out).originalBrushNumber = -1;
        (*out).parms.color[0] = 0.0;
        (*out).parms.color[1] = 0.0;
        (*out).parms.color[2] = 0.0;
        (*out).parms.color[3] = 0.0;
        (*out).parms.depthForOpaque = 0.0;
        (*out).colorInt = 0x00000000;
        (*out).tcScale = 0.0;
        (*out).hasSurface = 0;
    }
}


/*
================
R_LoadLightGrid

================
*/
pub unsafe fn R_LoadLightGrid(l: *mut lump_t, worldData: &mut world_t) {
    let mut i: c_int;
    let mut j: c_int;
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut w: *mut world_t;
    let mut wMins: *mut f32;
    let mut wMaxs: *mut f32;

    w = worldData as *mut _;

    (*w).lightGridInverseSize[0] = 1.0 / (*w).lightGridSize[0];
    (*w).lightGridInverseSize[1] = 1.0 / (*w).lightGridSize[1];
    (*w).lightGridInverseSize[2] = 1.0 / (*w).lightGridSize[2];

    wMins = (*w).bmodels[0].bounds[0].as_mut_ptr();
    wMaxs = (*w).bmodels[0].bounds[1].as_mut_ptr();

    i = 0;
    while i < 3 {
        (*w).lightGridOrigin[i as usize] =
            (*w).lightGridSize[i as usize] * ceil(*wMins.add(i as usize) / (*w).lightGridSize[i as usize]);
        maxs[i as usize] =
            (*w).lightGridSize[i as usize] * floor(*wMaxs.add(i as usize) / (*w).lightGridSize[i as usize]);
        (*w).lightGridBounds[i as usize] =
            ((maxs[i as usize] - (*w).lightGridOrigin[i as usize]) / (*w).lightGridSize[i as usize] + 1.0)
                as c_int;
        i += 1;
    }

    let numGridDataElements: c_int =
        ((*l).filelen / core::mem::size_of::<mgrid_t>()) as c_int;

    (*w).lightGridData =
        Hunk_Alloc((*l).filelen as usize, 0) as *mut mgrid_t;
    memcpy(
        (*w).lightGridData as *mut c_void,
        (fileBase.add((*l).fileofs as usize)) as *const c_void,
        (*l).filelen as usize,
    );

    // deal with overbright bits
    i = 0;
    while i < numGridDataElements {
        j = 0;
        while j < MAXLIGHTMAPS as c_int {
            R_ColorShiftLightingBytes_3((*(*w).lightGridData.add(i as usize)).ambientLight[j as usize].as_mut_ptr());
            R_ColorShiftLightingBytes_3((*(*w).lightGridData.add(i as usize)).directLight[j as usize].as_mut_ptr());
            j += 1;
        }
        i += 1;
    }
}

/*
================
R_LoadLightGridArray

================
*/
pub unsafe fn R_LoadLightGridArray(l: *mut lump_t, worldData: &mut world_t) {
    let mut w: *mut world_t;

    w = worldData as *mut _;

    (*w).numGridArrayElements =
        (*w).lightGridBounds[0] * (*w).lightGridBounds[1] * (*w).lightGridBounds[2];

    if (*l).filelen != ((*w).numGridArrayElements as usize * core::mem::size_of::<u16>()) as i32 {
        if (*l).filelen > 0 {
            //don't warn if not even lit
            VID_Printf(
                PRINT_WARNING,
                "WARNING: light grid array mismatch\n\0".as_ptr() as *const c_char,
            );
        }
        (*w).lightGridData = core::ptr::null_mut();
        return;
    }

    (*w).lightGridArray = Hunk_Alloc((*l).filelen as usize, 0) as *mut u16;
    memcpy(
        (*w).lightGridArray as *mut c_void,
        (fileBase.add((*l).fileofs as usize)) as *const c_void,
        (*l).filelen as usize,
    );
}


/*
================
R_LoadEntities
================
*/
pub unsafe fn R_LoadEntities(l: *mut lump_t, worldData: &mut world_t) {
    let mut p: *const c_char;
    let mut token: *const c_char;
    let mut keyname: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
    let mut value: [c_char; MAX_TOKEN_CHARS] = [0; MAX_TOKEN_CHARS];
    let mut w: *mut world_t;
    let mut ambient: f32 = 1.0;

    w = worldData as *mut _;
    (*w).lightGridSize[0] = 64.0;
    (*w).lightGridSize[1] = 64.0;
    (*w).lightGridSize[2] = 128.0;

    VectorSet(
        (*core::ptr::addr_of_mut!(tr)).sunAmbient.as_mut_ptr(),
        1.0,
        1.0,
        1.0,
    );
    (*core::ptr::addr_of_mut!(tr)).distanceCull = 12000.0; //DEFAULT_DISTANCE_CULL;

    p = (fileBase.add((*l).fileofs as usize)) as *const c_char;

    let mut p_mut = p;
    token = COM_ParseExt(&mut p_mut, 1);
    if *token == 0 as c_char || *token != (b'{' as c_char) {
        return;
    }

    // only parse the world spawn
    loop {
        // parse key
        token = COM_ParseExt(&mut p_mut, 1);

        if *token == 0 as c_char || *token == (b'}' as c_char) {
            break;
        }
        Q_strncpyz(keyname.as_mut_ptr(), token, core::mem::size_of_val(&keyname));

        // parse value
        token = COM_ParseExt(&mut p_mut, 1);

        if *token == 0 as c_char || *token == (b'}' as c_char) {
            break;
        }
        Q_strncpyz(value.as_mut_ptr(), token, core::mem::size_of_val(&value));

        // check for remapping of shaders for vertex lighting
        /*		s = "vertexremapshader";
        if (!Q_strncmp(keyname, s, strlen(s)) ) {
            s = strchr(value, ';');
            if (!s) {
                VID_Printf( S_COLOR_YELLOW "WARNING: no semi colon in vertexshaderremap '%s'\n", value );
                break;
            }
            *s++ = 0;
            if (r_vertexLight->integer) {
                R_RemapShader(value, s, "0");
            }
            continue;
        }
        // check for remapping of shaders
        s = "remapshader";
        if (!Q_strncmp(keyname, s, strlen(s)) ) {
            s = strchr(value, ';');
            if (!s) {
                VID_Printf( S_COLOR_YELLOW "WARNING: no semi colon in shaderremap '%s'\n", value );
                break;
            }
            *s++ = 0;
            R_RemapShader(value, s, "0");
            continue;
        }
        */ if Q_stricmp(
            keyname.as_ptr(),
            "distanceCull\0".as_ptr() as *const c_char,
        ) == 0
        {
            sscanf(value.as_ptr(), "%f\0".as_ptr() as *const c_char, &mut (*core::ptr::addr_of_mut!(tr)).distanceCull);
            continue;
        }
        //check for linear fog -rww
        if Q_stricmp(
            keyname.as_ptr(),
            "linFogStart\0".as_ptr() as *const c_char,
        ) == 0
        {
            sscanf(
                value.as_ptr(),
                "%f\0".as_ptr() as *const c_char,
                &mut (*core::ptr::addr_of_mut!(tr)).rangedFog,
            );
            (*core::ptr::addr_of_mut!(tr)).rangedFog = -(*core::ptr::addr_of_mut!(tr)).rangedFog;
            continue;
        }
        // check for a different grid size
        if Q_stricmp(keyname.as_ptr(), "gridsize\0".as_ptr() as *const c_char) == 0 {
            sscanf(
                value.as_ptr(),
                "%f %f %f\0".as_ptr() as *const c_char,
                &mut (*w).lightGridSize[0],
                &mut (*w).lightGridSize[1],
                &mut (*w).lightGridSize[2],
            );
            continue;
        }
        // find the optional world ambient for arioche
        if Q_stricmp(keyname.as_ptr(), "_color\0".as_ptr() as *const c_char) == 0 {
            sscanf(
                value.as_ptr(),
                "%f %f %f\0".as_ptr() as *const c_char,
                &mut (*core::ptr::addr_of_mut!(tr)).sunAmbient[0],
                &mut (*core::ptr::addr_of_mut!(tr)).sunAmbient[1],
                &mut (*core::ptr::addr_of_mut!(tr)).sunAmbient[2],
            );
            continue;
        }
        if Q_stricmp(keyname.as_ptr(), "ambient\0".as_ptr() as *const c_char) == 0 {
            sscanf(value.as_ptr(), "%f\0".as_ptr() as *const c_char, &mut ambient);
            continue;
        }
    }
    //both default to 1 so no harm if not present.
    VectorScale(
        (*core::ptr::addr_of!(tr)).sunAmbient.as_ptr(),
        ambient,
        (*core::ptr::addr_of_mut!(tr)).sunAmbient.as_mut_ptr(),
    );
}


/*
=================
RE_LoadWorldMap

Called directly from cgame
=================
*/
pub unsafe fn RE_LoadWorldMap_Actual(name: *const c_char, worldData: &mut world_t, index: c_int) {
    let mut i: c_int;
    let mut header: *mut dheader_t;
    let mut buffer: *mut u8 = core::ptr::null_mut();
    let mut loadedSubBSP: i32 = 0;

    if (*core::ptr::addr_of!(tr)).worldMapLoaded != 0 && index == 0 {
        Com_Error(
            ERR_DROP,
            "ERROR: attempted to redundantly load world map\n\0".as_ptr() as *const c_char,
        );
    }

    // set default sun direction to be used if it isn't
    // overridden by a shader
    if index == 0 {
        skyboxportal = 0;

        (*core::ptr::addr_of_mut!(tr)).sunDirection[0] = 0.45;
        (*core::ptr::addr_of_mut!(tr)).sunDirection[1] = 0.3;
        (*core::ptr::addr_of_mut!(tr)).sunDirection[2] = 0.9;

        VectorNormalize((*core::ptr::addr_of_mut!(tr)).sunDirection.as_mut_ptr());

        (*core::ptr::addr_of_mut!(tr)).worldMapLoaded = 1;

        // clear tr.world so if the level fails to load, the next
        // try will not look at the partially loaded version
        (*core::ptr::addr_of_mut!(tr)).world = core::ptr::null_mut();
    }

    // check for cached disk file from the server first...
    //
    if gpvCachedMapDiskImage as usize != 0 {
        if strcmp(name, gsCachedMapDiskImage.as_ptr()) == 0 {
            // we should always get here, if inside the first IF...
            //
            buffer = gpvCachedMapDiskImage as *mut u8;
        } else {
            // this should never happen (ie renderer loading a different map than the server), but just in case...
            //
            //		assert(0);
            //		Z_Free(gpvCachedMapDiskImage);
            //			   gpvCachedMapDiskImage = NULL;
            //rww - this is a valid possibility now because of sub-bsp loading.\
            //it's alright, just keep the current cache
            loadedSubBSP = 1;
        }
    }

    if buffer as usize == 0 {
        // still needs loading...
        //
        let mut buffer_ptr: *mut c_void = core::ptr::null_mut();
        FS_ReadFile(name, &mut buffer_ptr);
        buffer = buffer_ptr as *mut u8;
        if buffer as usize == 0 {
            Com_Error(
                ERR_DROP,
                "RE_LoadWorldMap: %s not found\0".as_ptr() as *const c_char,
                name,
            );
        }
    }

    memset(
        worldData as *mut c_void,
        0,
        core::mem::size_of_val(worldData),
    );

    Q_strncpyz(
        (*core::ptr::addr_of_mut!(tr)).worldDir.as_mut_ptr(),
        name,
        core::mem::size_of_val(&(*core::ptr::addr_of_mut!(tr)).worldDir),
    );
    COM_StripExtension(
        (*core::ptr::addr_of_mut!(tr)).worldDir.as_mut_ptr(),
        (*core::ptr::addr_of_mut!(tr)).worldDir.as_mut_ptr(),
    );

    c_gridVerts = 0;

    header = buffer as *mut dheader_t;
    fileBase = buffer;

    (*header).version = LittleLong((*header).version);

    if (*header).version != BSP_VERSION {
        Com_Error(
            ERR_DROP,
            "RE_LoadWorldMap: %s has wrong version number (%i should be %i)\0".as_ptr() as *const c_char,
            name,
            (*header).version,
            BSP_VERSION,
        );
    }

    // swap all the lumps
    i = 0;
    while i < (core::mem::size_of::<dheader_t>() / 4) as c_int {
        *((header as *mut c_int).add(i as usize)) =
            LittleLong(*((header as *const c_int).add(i as usize)));
        i += 1;
    }

    // load into heap
    R_LoadShaders(&mut (*header).lumps[LUMP_SHADERS], worldData);
    R_LoadLightmaps(&mut (*header).lumps[LUMP_LIGHTMAPS], name, worldData);
    R_LoadPlanes(&mut (*header).lumps[LUMP_PLANES], worldData);
    R_LoadFogs(
        &mut (*header).lumps[LUMP_FOGS],
        &mut (*header).lumps[LUMP_BRUSHES],
        &mut (*header).lumps[LUMP_BRUSHSIDES],
        worldData,
        index,
    );
    R_LoadSurfaces(
        &mut (*header).lumps[LUMP_SURFACES],
        &mut (*header).lumps[LUMP_DRAWVERTS],
        &mut (*header).lumps[LUMP_DRAWINDEXES],
        worldData,
        index,
    );
    R_LoadMarksurfaces(&mut (*header).lumps[LUMP_LEAFSURFACES], worldData);
    R_LoadNodesAndLeafs(
        &mut (*header).lumps[LUMP_NODES],
        &mut (*header).lumps[LUMP_LEAFS],
        worldData,
    );
    R_LoadSubmodels(&mut (*header).lumps[LUMP_MODELS], worldData, index);
    R_LoadVisibility(&mut (*header).lumps[LUMP_VISIBILITY], worldData);

    if index == 0 {
        R_LoadEntities(&mut (*header).lumps[LUMP_ENTITIES], worldData);
        R_LoadLightGrid(&mut (*header).lumps[LUMP_LIGHTGRID], worldData);
        R_LoadLightGridArray(&mut (*header).lumps[LUMP_LIGHTARRAY], worldData);

        // only set tr.world now that we know the entire level has loaded properly
        (*core::ptr::addr_of_mut!(tr)).world = worldData as *mut _;
    }


    if gpvCachedMapDiskImage as usize != 0 && loadedSubBSP == 0 {
        // For the moment, I'm going to keep this disk image around in case we need it to respawn.
        //  No problem for memory, since it'll only be a NZ ptr if we're not low on physical memory
        //	( ie we've got > 96MB).
        //
        //  So don't do this...
        //
        //		Z_Free( gpvCachedMapDiskImage );
        //				gpvCachedMapDiskImage = NULL;
    } else {
        FS_FreeFile(buffer as *mut c_void);
    }
}


// new wrapper used for convenience to tell z_malloc()-fail recovery code whether it's safe to dump the cached-bsp or not.
//
pub unsafe fn RE_LoadWorldMap(name: *const c_char) {
    gbUsingCachedMapDataRightNow = 1; // !!!!!!!!!!!!

    RE_LoadWorldMap_Actual(name, &mut s_worldData, 0);

    gbUsingCachedMapDataRightNow = 0; // !!!!!!!!!!!!
}
