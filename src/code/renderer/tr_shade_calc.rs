// tr_shade_calc.c

// leave this as first line for PCH reasons...
//
// include "../server/exe_headers.h"

// include "tr_local.h"

use core::ffi::{c_int, c_char, c_void};

// Macro: WAVEVALUE( table, base, amplitude, phase, freq )  ((base) + table[ myftol( ( ( (phase) + backEnd.refdef.floatTime * (freq) ) * FUNCTABLE_SIZE ) ) & FUNCTABLE_MASK ] * (amplitude))
// This needs to be expanded inline in functions that use it

// Type aliases for C types
type genFunc_t = c_int;
type qboolean = c_int;
type byte = u8;
type glIndex_t = c_int;
type colorGen_t = c_int;
type vec3_t = [f32; 3];
type vec4_t = [f32; 4];

// C struct definitions
#[repr(C)]
pub struct waveForm_t {
    pub func: genFunc_t,
    pub base: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub frequency: f32,
}

#[repr(C)]
pub struct texModInfo_t {
    pub matrix: [[f32; 2]; 2],
    pub translate: [f32; 2],
}

#[repr(C)]
pub struct deformStage_t {
    pub deformation: c_int,
    pub deformationWave: waveForm_t,
    pub deformationSpread: f32,
    pub bulgeSpeed: f32,
    pub bulgeWidth: f32,
    pub bulgeHeight: f32,
    pub moveVector: vec3_t,
}

#[repr(C)]
pub struct fog_t {
    pub hasSurface: qboolean,
    pub surface: vec4_t,
    pub tcScale: f32,
}

// Stub types for external references
#[repr(C)]
pub struct tr_t {
    // Incomplete stub
}

#[repr(C)]
pub struct backEndState_t {
    // Incomplete stub
}

#[repr(C)]
pub struct trRefEntity_t {
    // Incomplete stub
}

#[repr(C)]
pub struct refEntity_t {
    // Incomplete stub
}

#[repr(C)]
pub struct tesselation_t {
    // Incomplete stub
}

// Constants
const GF_SIN: c_int = 0;
const GF_TRIANGLE: c_int = 1;
const GF_SQUARE: c_int = 2;
const GF_SAWTOOTH: c_int = 3;
const GF_INVERSE_SAWTOOTH: c_int = 4;
const GF_NOISE: c_int = 5;
const GF_RAND: c_int = 6;
const GF_NONE: c_int = 7;

const ERR_DROP: c_int = 0;
const PRINT_WARNING: c_int = 1;
const DEFORM_NONE: c_int = 0;
const DEFORM_NORMALS: c_int = 1;
const DEFORM_WAVE: c_int = 2;
const DEFORM_BULGE: c_int = 3;
const DEFORM_MOVE: c_int = 4;
const DEFORM_PROJECTION_SHADOW: c_int = 5;
const DEFORM_AUTOSPRITE: c_int = 6;
const DEFORM_AUTOSPRITE2: c_int = 7;
const DEFORM_TEXT0: c_int = 8;
const DEFORM_TEXT1: c_int = 9;
const DEFORM_TEXT2: c_int = 10;
const DEFORM_TEXT3: c_int = 11;
const DEFORM_TEXT4: c_int = 12;
const DEFORM_TEXT5: c_int = 13;
const DEFORM_TEXT6: c_int = 14;
const DEFORM_TEXT7: c_int = 15;

const FUNCTABLE_SIZE: c_int = 1024;
const FUNCTABLE_MASK: c_int = 1023;
const SHADER_MAX_VERTEXES: usize = 1024;
const WORLD_SIZE: f32 = 65536.0;
const NUM_TEX_COORDS: usize = 1;
const M_PI: f32 = 3.14159265358979323846;

const CGEN_LIGHTING_DIFFUSE_ENTITY: c_int = 0;
const RF_FIRST_PERSON: c_int = 1;
const RF_DISINTEGRATE1: c_int = 2;
const RF_DISINTEGRATE2: c_int = 4;

// Extern C declarations
extern "C" {
    pub static mut tr: tr_t;
    pub static mut backEnd: backEndState_t;
    pub static mut tess: tesselation_t;
    pub static vec3_origin: vec3_t;

    pub fn TableForFunc(func: genFunc_t) -> *mut f32;
    pub fn RB_CalcTransformTexCoords(tmi: *const texModInfo_t, st: *mut f32);
    pub fn RB_AddQuadStampExt(origin: *const vec3_t, width: *const vec3_t, height: *const vec3_t,
                              color: *const u8, fcol: f32, frow: f32, fcol_end: f32, frow_end: f32);
    pub fn RB_AddQuadStamp(mid: *const vec3_t, left: *const vec3_t, up: *const vec3_t, colors: *const u8);
    pub fn RB_ProjectionShadowDeform();
    pub fn RB_CalcFogTexCoords(st: *mut f32);
    pub fn R_FogFactor(x: f32, y: f32) -> f32;
    pub fn GetNoiseTime(t: c_int) -> f32;
    pub fn R_NoiseGet4f(x: f32, y: f32, z: f32, t: f32) -> f32;
    pub fn VectorScale(v: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    pub fn VectorAdd(v1: *const [f32; 3], v2: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorSubtract(v1: *const [f32; 3], v2: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorClear(v: *mut [f32; 3]);
    pub fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    pub fn VectorNormalizeFast(v: *mut [f32; 3]);
    pub fn VectorLength(v: *const [f32; 3]) -> f32;
    pub fn VectorLengthSquared(v: *const [f32; 3]) -> f32;
    pub fn DotProduct(v1: *const [f32; 3], v2: *const [f32; 3]) -> f32;
    pub fn CrossProduct(v1: *const [f32; 3], v2: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorMA(v: *const [f32; 3], scale: f32, dir: *const [f32; 3], out: *mut [f32; 3]);
    pub fn Q_rsqrt(x: f32) -> f32;
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn VID_Printf(level: c_int, fmt: *const c_char, ...);
    pub fn strlen(s: *const c_char) -> usize;
}

// Helper function - inline assembly version for x86, fallback for others
#[cfg(all(target_arch = "x86", not(all(target_os = "linux", target_arch = "x86"))))]
#[inline(never)]
unsafe fn myftol(f: f32) -> c_int {
    let tmp: c_int;
    core::arch::asm! {
        "fld {}",
        "fistp {}",
        "mov eax, {}",
        in(x87) f,
        out(x87) _,
        lateout("eax") tmp,
    }
    tmp
}

#[cfg(not(all(target_arch = "x86", not(all(target_os = "linux", target_arch = "x86")))))]
#[inline]
unsafe fn myftol(f: f32) -> c_int {
    f as c_int
}

// extern float GetNoiseTime( int t ); //from tr_noise, returns 0 to 2
/*
** EvalWaveForm
**
** Evaluates a given waveForm_t, referencing backEnd.refdef.time directly
*/
unsafe fn EvalWaveForm(wf: *const waveForm_t) -> f32 {
    let table: *mut f32;

    if (*wf).func == GF_NOISE {
        return ((*wf).base + R_NoiseGet4f(0.0, 0.0, 0.0, (backEnd.refdef.floatTime + (*wf).phase) * (*wf).frequency) * (*wf).amplitude);
    } else if (*wf).func == GF_RAND {
        if GetNoiseTime(backEnd.refdef.time as c_int + (*wf).phase as c_int) <= (*wf).frequency {
            return ((*wf).base + (*wf).amplitude);
        } else {
            return (*wf).base;
        }
    }
    table = TableForFunc((*wf).func);
    return ((*wf).base) + *table.add((myftol(((((*wf).phase + backEnd.refdef.floatTime * (*wf).frequency) * FUNCTABLE_SIZE as f32) as c_int)) as usize) & FUNCTABLE_MASK as usize)) as f32 * ((*wf).amplitude);
}

unsafe fn EvalWaveFormClamped(wf: *const waveForm_t) -> f32 {
    let mut glow: f32 = EvalWaveForm(wf);

    if glow < 0.0 {
        return 0.0;
    }

    if glow > 1.0 {
        return 1.0;
    }

    return glow;
}

/*
** RB_CalcStretchTexCoords
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcStretchTexCoords(wf: *const waveForm_t, st: *mut f32) {
    let mut p: f32;
    let mut tmi: texModInfo_t;

    p = 1.0f32 / EvalWaveForm(wf);

    tmi.matrix[0][0] = p;
    tmi.matrix[1][0] = 0.0;
    tmi.translate[0] = 0.5f32 - 0.5f32 * p;

    tmi.matrix[0][1] = 0.0;
    tmi.matrix[1][1] = p;
    tmi.translate[1] = 0.5f32 - 0.5f32 * p;

    RB_CalcTransformTexCoords(&tmi, st);
}

/*
====================================================================

DEFORMATIONS

====================================================================
*/

/*
========================
RB_CalcDeformVertexes

========================
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcDeformVertexes(ds: *mut deformStage_t) {
    let mut i: c_int;
    let mut offset: vec3_t = [0.0; 3];
    let mut scale: f32;
    let mut xyz: *mut f32 = tess.xyz as *mut f32;
    let mut normal: *mut f32 = tess.normal as *mut f32;
    let mut table: *mut f32;

    if (*ds).deformationWave.frequency == 0.0 {
        scale = EvalWaveForm(core::ptr::addr_of!((*ds).deformationWave));

        i = 0;
        while i < tess.numVertexes {
            VectorScale(normal as *const _, scale, &mut offset);

            *xyz.offset(0) += offset[0];
            *xyz.offset(1) += offset[1];
            *xyz.offset(2) += offset[2];

            i += 1;
            xyz = xyz.offset(4);
            normal = normal.offset(4);
        }
    } else {
        table = TableForFunc((*ds).deformationWave.func);

        i = 0;
        while i < tess.numVertexes {
            let off: f32 = (*xyz.offset(0) + *xyz.offset(1) + *xyz.offset(2)) * (*ds).deformationSpread;

            scale = *table.add((myftol(((((*ds).deformationWave.phase + off + backEnd.refdef.floatTime * (*ds).deformationWave.frequency) * FUNCTABLE_SIZE as f32) as c_int)) as usize) & FUNCTABLE_MASK as usize)) as f32 * (*ds).deformationWave.amplitude;

            VectorScale(normal as *const _, scale, &mut offset);

            *xyz.offset(0) += offset[0];
            *xyz.offset(1) += offset[1];
            *xyz.offset(2) += offset[2];

            i += 1;
            xyz = xyz.offset(4);
            normal = normal.offset(4);
        }
    }
}

/*
=========================
RB_CalcDeformNormals

Wiggle the normals for wavy environment mapping
=========================
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcDeformNormals(ds: *mut deformStage_t) {
    let mut i: c_int;
    let mut scale: f32;
    let mut xyz: *mut f32 = tess.xyz as *mut f32;
    let mut normal: *mut f32 = tess.normal as *mut f32;

    i = 0;
    while i < tess.numVertexes {
        scale = 0.98f32;
        scale = R_NoiseGet4f(*xyz.offset(0) * scale, *xyz.offset(1) * scale, *xyz.offset(2) * scale,
            backEnd.refdef.floatTime * (*ds).deformationWave.frequency);
        *normal.offset(0) += (*ds).deformationWave.amplitude * scale;

        scale = 0.98f32;
        scale = R_NoiseGet4f(100.0 + *xyz.offset(0) * scale, *xyz.offset(1) * scale, *xyz.offset(2) * scale,
            backEnd.refdef.floatTime * (*ds).deformationWave.frequency);
        *normal.offset(1) += (*ds).deformationWave.amplitude * scale;

        scale = 0.98f32;
        scale = R_NoiseGet4f(200.0 + *xyz.offset(0) * scale, *xyz.offset(1) * scale, *xyz.offset(2) * scale,
            backEnd.refdef.floatTime * (*ds).deformationWave.frequency);
        *normal.offset(2) += (*ds).deformationWave.amplitude * scale;

        VectorNormalizeFast(normal as *mut _);

        i += 1;
        xyz = xyz.offset(4);
        normal = normal.offset(4);
    }
}

/*
========================
RB_CalcBulgeVertexes

========================
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcBulgeVertexes(ds: *mut deformStage_t) {
    let mut i: c_int;
    let mut xyz: *mut f32 = tess.xyz as *mut f32;
    let mut normal: *mut f32 = tess.normal as *mut f32;
    let mut scale: f32;

    if (*ds).bulgeSpeed == 0.0f32 && (*ds).bulgeWidth == 0.0f32 {
        // We don't have a speed and width, so just use height to expand uniformly
        i = 0;
        while i < tess.numVertexes {
            *xyz.offset(0) += *normal.offset(0) * (*ds).bulgeHeight;
            *xyz.offset(1) += *normal.offset(1) * (*ds).bulgeHeight;
            *xyz.offset(2) += *normal.offset(2) * (*ds).bulgeHeight;

            i += 1;
            xyz = xyz.offset(4);
            normal = normal.offset(4);
        }
    } else {
        // I guess do some extra dumb stuff..the fact that it uses ST seems bad though because skin pages may be set up in certain ways that can cause
        //	very noticeable seams on sufaces ( like on the huge ion_cannon ).
        let st: *const f32 = tess.texCoords[0] as *const f32;
        let mut now: f32;
        let mut off: c_int;

        now = backEnd.refdef.time * (*ds).bulgeSpeed * 0.001f32;

        i = 0;
        while i < tess.numVertexes {
            off = (((FUNCTABLE_SIZE as f32 / (M_PI * 2.0)) * (*st.offset(0) * (*ds).bulgeWidth + now)) as c_int);

            scale = *tr.sinTable.add((off & FUNCTABLE_MASK) as usize) * (*ds).bulgeHeight;

            *xyz.offset(0) += *normal.offset(0) * scale;
            *xyz.offset(1) += *normal.offset(1) * scale;
            *xyz.offset(2) += *normal.offset(2) * scale;

            i += 1;
            xyz = xyz.offset(4);
            st = st.offset(2 * NUM_TEX_COORDS as isize);
            normal = normal.offset(4);
        }
    }
}


/*
======================
RB_CalcMoveVertexes

A deformation that can move an entire surface along a wave path
======================
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcMoveVertexes(ds: *mut deformStage_t) {
    let mut i: c_int;
    let mut xyz: *mut f32;
    let mut table: *mut f32;
    let mut scale: f32;
    let mut offset: vec3_t = [0.0; 3];

    table = TableForFunc((*ds).deformationWave.func);

    scale = *table.add((myftol(((((*ds).deformationWave.phase + backEnd.refdef.floatTime * (*ds).deformationWave.frequency) * FUNCTABLE_SIZE as f32) as c_int)) as usize) & FUNCTABLE_MASK as usize)) as f32 * (*ds).deformationWave.amplitude;

    VectorScale(&(*ds).moveVector, scale, &mut offset);

    xyz = tess.xyz as *mut f32;
    i = 0;
    while i < tess.numVertexes {
        VectorAdd(xyz as *const _, &offset, xyz as *mut _);

        i += 1;
        xyz = xyz.offset(4);
    }
}


/*
=============
DeformText

Change a polygon into a bunch of text polygons
=============
*/
#[no_mangle]
pub unsafe extern "C" fn DeformText(text: *const c_char) {
    let mut i: c_int;
    let mut origin: vec3_t = [0.0; 3];
    let mut width: vec3_t = [0.0; 3];
    let mut height: vec3_t = [0.0; 3];
    let mut len: c_int;
    let mut ch: c_int;
    let mut color: [u8; 4] = [0; 4];
    let mut bottom: f32;
    let mut top: f32;
    let mut mid: vec3_t = [0.0; 3];

    height[0] = 0.0;
    height[1] = 0.0;
    height[2] = -1.0;
    CrossProduct(&tess.normal[0], &height, &mut width);

    // find the midpoint of the box
    VectorClear(&mut mid);
    bottom = WORLD_SIZE;
    top = -WORLD_SIZE;
    i = 0;
    while i < 4 {
        VectorAdd(&tess.xyz[i as usize], &mid, &mut mid);
        if tess.xyz[i as usize][2] < bottom {
            bottom = tess.xyz[i as usize][2];
        }
        if tess.xyz[i as usize][2] > top {
            top = tess.xyz[i as usize][2];
        }
        i += 1;
    }
    VectorScale(&mid, 0.25f32, &mut origin);

    // determine the individual character size
    height[0] = 0.0;
    height[1] = 0.0;
    height[2] = (top - bottom) * 0.5f32;

    VectorScale(&width, height[2] * -0.75f32, &mut width);

    // determine the starting position
    len = strlen(text) as c_int;
    VectorMA(&origin, (len - 1) as f32, &width, &mut origin);

    // clear the shader indexes
    tess.numIndexes = 0;
    tess.numVertexes = 0;

    color[0] = 255;
    color[1] = 255;
    color[2] = 255;
    color[3] = 255;

    // draw each character
    i = 0;
    while i < len {
        ch = *text.offset(i as isize) as c_int;
        ch &= 255;

        if ch != ' ' as c_int {
            let mut row: c_int;
            let mut col: c_int;
            let mut frow: f32;
            let mut fcol: f32;
            let mut size: f32;

            row = ch >> 4;
            col = ch & 15;

            frow = (row as f32) * 0.0625f32;
            fcol = (col as f32) * 0.0625f32;
            size = 0.0625f32;

            RB_AddQuadStampExt(&origin, &width, &height, &color[0], fcol, frow, fcol + size, frow + size);
        }
        VectorMA(&origin, -2.0, &width, &mut origin);

        i += 1;
    }
}

/*
==================
GlobalVectorToLocal
==================
*/
unsafe fn GlobalVectorToLocal(in_: *const vec3_t, out: *mut vec3_t) {
    (*out)[0] = DotProduct(in_, &backEnd.ori.axis[0]);
    (*out)[1] = DotProduct(in_, &backEnd.ori.axis[1]);
    (*out)[2] = DotProduct(in_, &backEnd.ori.axis[2]);
}

/*
=====================
AutospriteDeform

Assuming all the triangles for this shader are independant
quads, rebuild them as forward facing sprites
=====================
*/
unsafe fn AutospriteDeform() {
    let mut i: c_int;
    let mut oldVerts: c_int;
    let mut xyz: *mut f32;
    let mut mid: vec3_t = [0.0; 3];
    let mut delta: vec3_t = [0.0; 3];
    let mut radius: f32;
    let mut left: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut leftDir: vec3_t = [0.0; 3];
    let mut upDir: vec3_t = [0.0; 3];

    if (tess.numVertexes & 3) != 0 {
        Com_Error(ERR_DROP, b"Autosprite shader %s had odd vertex count\0".as_ptr() as *const c_char);
    }
    if tess.numIndexes != ((tess.numVertexes >> 2) * 6) {
        Com_Error(ERR_DROP, b"Autosprite shader %s had odd index count\0".as_ptr() as *const c_char);
    }

    oldVerts = tess.numVertexes;
    tess.numVertexes = 0;
    tess.numIndexes = 0;

    if backEnd.currentEntity != core::ptr::addr_of_mut!(tr.worldEntity) {
        GlobalVectorToLocal(&backEnd.viewParms.or.axis[1], &mut leftDir);
        GlobalVectorToLocal(&backEnd.viewParms.or.axis[2], &mut upDir);
    } else {
        VectorCopy(&backEnd.viewParms.or.axis[1], &mut leftDir);
        VectorCopy(&backEnd.viewParms.or.axis[2], &mut upDir);
    }

    i = 0;
    while i < oldVerts {
        // find the midpoint
        xyz = core::ptr::addr_of_mut!(tess.xyz[i as usize][0]);

        mid[0] = 0.25f32 * (*xyz.offset(0) + *xyz.offset(4) + *xyz.offset(8) + *xyz.offset(12));
        mid[1] = 0.25f32 * (*xyz.offset(1) + *xyz.offset(5) + *xyz.offset(9) + *xyz.offset(13));
        mid[2] = 0.25f32 * (*xyz.offset(2) + *xyz.offset(6) + *xyz.offset(10) + *xyz.offset(14));

        VectorSubtract(xyz as *const _, &mid, &mut delta);
        radius = VectorLength(&delta) * 0.707f32;

        VectorScale(&leftDir, radius, &mut left);
        VectorScale(&upDir, radius, &mut up);

        if backEnd.viewParms.isMirror != 0 {
            VectorSubtract(&vec3_origin, &left, &mut left);
        }

        RB_AddQuadStamp(&mid, &left, &up, &tess.vertexColors[i as usize][0]);

        i += 4;
    }
}


/*
=====================
Autosprite2Deform

Autosprite2 will pivot a rectangular quad along the center of its long axis
=====================
*/
static edgeVerts: [[glIndex_t; 2]; 6] = [
    [0, 1],
    [0, 2],
    [0, 3],
    [1, 2],
    [1, 3],
    [2, 3]
];

unsafe fn Autosprite2Deform() {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut indexes: c_int;
    let mut xyz: *mut f32;
    let mut forward: vec3_t = [0.0; 3];

    if (tess.numVertexes & 3) != 0 {
        VID_Printf(PRINT_WARNING, b"Autosprite shader %s had odd vertex count\0".as_ptr() as *const c_char);
    }
    if tess.numIndexes != ((tess.numVertexes >> 2) * 6) {
        VID_Printf(PRINT_WARNING, b"Autosprite shader %s had odd index count\0".as_ptr() as *const c_char);
    }

    if backEnd.currentEntity != core::ptr::addr_of_mut!(tr.worldEntity) {
        GlobalVectorToLocal(&backEnd.viewParms.or.axis[0], &mut forward);
    } else {
        VectorCopy(&backEnd.viewParms.or.axis[0], &mut forward);
    }

    // this is a lot of work for two triangles...
    // we could precalculate a lot of it is an issue, but it would mess up
    // the shader abstraction
    i = 0;
    indexes = 0;
    while i < tess.numVertexes {
        let mut lengths: [f32; 2] = [0.0; 2];
        let mut nums: [c_int; 2] = [0; 2];
        let mut mid: [vec3_t; 2] = [[0.0; 3]; 2];
        let mut major: vec3_t = [0.0; 3];
        let mut minor: vec3_t = [0.0; 3];
        let mut v1: *mut f32;
        let mut v2: *mut f32;

        // find the midpoint
        xyz = core::ptr::addr_of_mut!(tess.xyz[i as usize][0]);

        // identify the two shortest edges
        nums[0] = 0;
        nums[1] = 0;
        lengths[0] = WORLD_SIZE;
        lengths[1] = WORLD_SIZE;

        j = 0;
        while j < 6 {
            let mut l: f32;
            let mut temp: vec3_t = [0.0; 3];

            v1 = xyz.offset(4 * edgeVerts[j as usize][0] as isize);
            v2 = xyz.offset(4 * edgeVerts[j as usize][1] as isize);

            VectorSubtract(v1 as *const _, v2 as *const _, &mut temp);

            l = DotProduct(&temp, &temp);
            if l < lengths[0] {
                nums[1] = nums[0];
                lengths[1] = lengths[0];
                nums[0] = j;
                lengths[0] = l;
            } else if l < lengths[1] {
                nums[1] = j;
                lengths[1] = l;
            }

            j += 1;
        }

        j = 0;
        while j < 2 {
            v1 = xyz.offset(4 * edgeVerts[nums[j as usize] as usize][0] as isize);
            v2 = xyz.offset(4 * edgeVerts[nums[j as usize] as usize][1] as isize);

            mid[j as usize][0] = 0.5f32 * (*v1.offset(0) + *v2.offset(0));
            mid[j as usize][1] = 0.5f32 * (*v1.offset(1) + *v2.offset(1));
            mid[j as usize][2] = 0.5f32 * (*v1.offset(2) + *v2.offset(2));

            j += 1;
        }

        // find the vector of the major axis
        VectorSubtract(&mid[1], &mid[0], &mut major);

        // cross this with the view direction to get minor axis
        CrossProduct(&major, &forward, &mut minor);
        VectorNormalize(&mut minor);

        // re-project the points
        j = 0;
        while j < 2 {
            let mut l: f32;

            v1 = xyz.offset(4 * edgeVerts[nums[j as usize] as usize][0] as isize);
            v2 = xyz.offset(4 * edgeVerts[nums[j as usize] as usize][1] as isize);

            l = 0.5 * (lengths[j as usize]).sqrt();

            // we need to see which direction this edge
            // is used to determine direction of projection
            k = 0;
            while k < 5 {
                if *tess.indexes.offset((indexes + k) as isize) == (i + edgeVerts[nums[j as usize] as usize][0])
                    && *tess.indexes.offset((indexes + k + 1) as isize) == (i + edgeVerts[nums[j as usize] as usize][1]) {
                    break;
                }
                k += 1;
            }

            if k == 5 {
                VectorMA(&mid[j as usize], l, &minor, v1 as *mut _);
                VectorMA(&mid[j as usize], -l, &minor, v2 as *mut _);
            } else {
                VectorMA(&mid[j as usize], -l, &minor, v1 as *mut _);
                VectorMA(&mid[j as usize], l, &minor, v2 as *mut _);
            }

            j += 1;
        }

        i += 4;
        indexes += 6;
    }
}


/*
=====================
RB_DeformTessGeometry

=====================
*/
// #pragma warning( disable : 4710 )	//vectorLength not inlined in AutospriteDeform which is auto-inlined in here
#[no_mangle]
pub unsafe extern "C" fn RB_DeformTessGeometry() {
    let mut i: c_int;
    let mut ds: *mut deformStage_t;

    i = 0;
    while i < tess.shader.numDeforms {
        ds = *tess.shader.deforms.offset(i as isize);

        match (*ds).deformation {
            DEFORM_NONE => {},
            DEFORM_NORMALS => {
                RB_CalcDeformNormals(ds);
            }
            DEFORM_WAVE => {
                RB_CalcDeformVertexes(ds);
            }
            DEFORM_BULGE => {
                RB_CalcBulgeVertexes(ds);
            }
            DEFORM_MOVE => {
                RB_CalcMoveVertexes(ds);
            }
            DEFORM_PROJECTION_SHADOW => {
                RB_ProjectionShadowDeform();
            }
            DEFORM_AUTOSPRITE => {
                AutospriteDeform();
            }
            DEFORM_AUTOSPRITE2 => {
                Autosprite2Deform();
            }
            DEFORM_TEXT0 | DEFORM_TEXT1 | DEFORM_TEXT2 | DEFORM_TEXT3 |
            DEFORM_TEXT4 | DEFORM_TEXT5 | DEFORM_TEXT6 | DEFORM_TEXT7 => {
//			DeformText( backEnd.refdef.text[ds->deformation - DEFORM_TEXT0] );
                DeformText(b"Raven Software\0".as_ptr() as *const c_char);
            }
            _ => {}
        }

        i += 1;
    }
}
// #pragma warning( default: 4710 )


/*
====================================================================

COLORS

====================================================================
*/


/*
** RB_CalcColorFromEntity
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcColorFromEntity_xbox(dstColors: *mut u32) {
    let mut i: c_int;
    let mut pColors: *mut u32 = dstColors;

    if backEnd.currentEntity.is_null() {
        return;
    }

    i = 0;
    while i < tess.numVertexes {
        *pColors = ((*backEnd.currentEntity).e.shaderRGBA[0] as u32) |
                   (((*backEnd.currentEntity).e.shaderRGBA[1] as u32) << 8) |
                   (((*backEnd.currentEntity).e.shaderRGBA[2] as u32) << 16) |
                   (((*backEnd.currentEntity).e.shaderRGBA[3] as u32) << 24);
        i += 1;
        pColors = pColors.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcColorFromEntity(dstColors: *mut u8) {
    let mut i: c_int;
    let mut pColors: *mut c_int = dstColors as *mut c_int;
    let mut c: c_int;

    if backEnd.currentEntity.is_null() {
        return;
    }

    c = *(backEnd.currentEntity as *const _ as *const c_int);

    i = 0;
    while i < tess.numVertexes {
        *pColors = c;
        i += 1;
        pColors = pColors.offset(1);
    }
}

/*
** RB_CalcColorFromOneMinusEntity
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcColorFromOneMinusEntity_xbox(dstColors: *mut u32) {
    let mut i: c_int;
    let mut pColors: *mut u32 = dstColors;
    let mut invModulate: [u8; 3] = [0; 3];

    if backEnd.currentEntity.is_null() {
        return;
    }

    invModulate[0] = (255u8).wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[0]);
    invModulate[1] = (255u8).wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[1]);
    invModulate[2] = (255u8).wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[2]);
    // this trashes alpha, but the AGEN block fixes it

    i = 0;
    while i < tess.numVertexes {
        *pColors = (invModulate[0] as u32) |
                   ((invModulate[1] as u32) << 8) |
                   ((invModulate[2] as u32) << 16) |
                   (((255u8.wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[3])) as u32) << 24);
        i += 1;
        pColors = pColors.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcColorFromOneMinusEntity(dstColors: *mut u8) {
    let mut i: c_int;
    let mut pColors: *mut c_int = dstColors as *mut c_int;
    let mut invModulate: [u8; 3] = [0; 3];

    if backEnd.currentEntity.is_null() {
        return;
    }

    invModulate[0] = (255u8).wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[0]);
    invModulate[1] = (255u8).wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[1]);
    invModulate[2] = (255u8).wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[2]);
    // this trashes alpha, but the AGEN block fixes it

    i = 0;
    while i < tess.numVertexes {
        *pColors = *(invModulate.as_ptr() as *const c_int);
        i += 1;
        pColors = pColors.offset(1);
    }
}

/*
** RB_CalcAlphaFromEntity
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcAlphaFromEntity_xbox(dstColors: *mut u32) {
    let mut i: c_int;

    if backEnd.currentEntity.is_null() {
        return;
    }

    i = 0;
    while i < tess.numVertexes {
        let dw = dstColors.offset(i as isize);
        let rgb: u32 = ((*dw) & 0x00ffffff);
        *dw = rgb | (((*backEnd.currentEntity).e.shaderRGBA[3] as u32 & 0xff) << 24);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcAlphaFromEntity(dstColors: *mut u8) {
    let mut i: c_int;

    if backEnd.currentEntity.is_null() {
        return;
    }

    let mut dc = dstColors.offset(3);

    i = 0;
    while i < tess.numVertexes {
        *dc = (*backEnd.currentEntity).e.shaderRGBA[3];
        i += 1;
        dc = dc.offset(4);
    }
}

/*
** RB_CalcAlphaFromOneMinusEntity
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcAlphaFromOneMinusEntity_xbox(dstColors: *mut u32) {
    let mut i: c_int;

    if backEnd.currentEntity.is_null() {
        return;
    }

    i = 0;
    while i < tess.numVertexes {
        let dw = dstColors.offset(i as isize);
        let rgb: u32 = ((*dw) & 0x00ffffff);
        *dw = rgb | (((255u8.wrapping_sub((*backEnd.currentEntity).e.shaderRGBA[3]) as u32) & 0xff) << 24);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcAlphaFromOneMinusEntity(dstColors: *mut u8) {
    let mut i: c_int;

    if backEnd.currentEntity.is_null() {
        return;
    }

    let mut dc = dstColors.offset(3);

    i = 0;
    while i < tess.numVertexes {
        *dc = 0xff - (*backEnd.currentEntity).e.shaderRGBA[3];
        i += 1;
        dc = dc.offset(4);
    }
}

/*
** RB_CalcWaveColor
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcWaveColor_xbox(wf: *const waveForm_t, dstColors: *mut u32) {
    let mut i: c_int;
    let mut v: c_int;
    let mut glow: f32;
    let mut colors: *mut u32 = dstColors;
    let mut color: [u8; 4] = [0; 4];

    if (*wf).func == GF_NOISE {
        glow = (*wf).base + R_NoiseGet4f(0.0, 0.0, 0.0, (backEnd.refdef.floatTime + (*wf).phase) * (*wf).frequency) * (*wf).amplitude;
    } else {
        glow = EvalWaveForm(wf) * tr.identityLight;
    }

    if glow < 0.0 {
        glow = 0.0;
    } else if glow > 1.0 {
        glow = 1.0;
    }

    v = myftol(255.0 * glow);
    color[0] = v as u8;
    color[1] = v as u8;
    color[2] = v as u8;
    color[3] = 255;

    i = 0;
    while i < tess.numVertexes {
        *colors = (color[0] as u32) | ((color[1] as u32) << 8) | ((color[2] as u32) << 16) | ((color[3] as u32) << 24);
        i += 1;
        colors = colors.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcWaveColor(wf: *const waveForm_t, dstColors: *mut u8) {
    let mut i: c_int;
    let mut v: c_int;
    let mut glow: f32;
    let mut colors: *mut c_int = dstColors as *mut c_int;
    let mut color: [u8; 4] = [0; 4];

    if (*wf).func == GF_NOISE {
        glow = (*wf).base + R_NoiseGet4f(0.0, 0.0, 0.0, (backEnd.refdef.floatTime + (*wf).phase) * (*wf).frequency) * (*wf).amplitude;
    } else {
        glow = EvalWaveForm(wf) * tr.identityLight;
    }

    if glow < 0.0 {
        glow = 0.0;
    } else if glow > 1.0 {
        glow = 1.0;
    }

    v = myftol(255.0 * glow);
    color[0] = v as u8;
    color[1] = v as u8;
    color[2] = v as u8;
    color[3] = 255;
    v = *(color.as_ptr() as *const c_int);

    i = 0;
    while i < tess.numVertexes {
        *colors = v;
        i += 1;
        colors = colors.offset(1);
    }
}

/*
** RB_CalcWaveAlpha
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcWaveAlpha_xbox(wf: *const waveForm_t, dstColors: *mut u32) {
    let mut i: c_int;
    let mut v: c_int;
    let glow: f32;

    glow = EvalWaveFormClamped(wf);

    v = (255.0 * glow) as c_int;

    i = 0;
    while i < tess.numVertexes {
        let dw = dstColors.offset(i as isize);
        let rgb: u32 = ((*dw) & 0x00ffffff);
        *dw = rgb | ((v as u32 & 0xff) << 24);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcWaveAlpha(wf: *const waveForm_t, dstColors: *mut u8) {
    let mut i: c_int;
    let mut v: c_int;
    let glow: f32;

    glow = EvalWaveFormClamped(wf);

    v = (255.0 * glow) as c_int;

    i = 0;
    while i < tess.numVertexes {
        let dc = dstColors.offset((i * 4 + 3) as isize);
        *dc = v as u8;
        i += 1;
    }
}

/*
** RB_CalcModulateColorsByFog
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcModulateColorsByFog_xbox(_colors: *mut u32) {

}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcModulateColorsByFog(colors: *mut u8) {
    let mut i: c_int;
    let mut texCoords: [[f32; 2]; SHADER_MAX_VERTEXES] = [[0.0; 2]; SHADER_MAX_VERTEXES];

    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    RB_CalcFogTexCoords(&mut texCoords[0][0]);

    i = 0;
    while i < tess.numVertexes {
        let f: f32 = 1.0 - R_FogFactor(texCoords[i as usize][0], texCoords[i as usize][1]);
        let col = colors.offset((i * 4) as isize);
        *col.offset(0) = ((*col.offset(0) as f32) * f) as u8;
        *col.offset(1) = ((*col.offset(1) as f32) * f) as u8;
        *col.offset(2) = ((*col.offset(2) as f32) * f) as u8;
        i += 1;
    }
}

/*
** RB_CalcModulateAlphasByFog
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcModulateAlphasByFog_xbox(_colors: *mut u32) {

}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcModulateAlphasByFog(colors: *mut u8) {
    let mut i: c_int;
    let mut texCoords: [[f32; 2]; SHADER_MAX_VERTEXES] = [[0.0; 2]; SHADER_MAX_VERTEXES];

    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    RB_CalcFogTexCoords(&mut texCoords[0][0]);

    i = 0;
    while i < tess.numVertexes {
        let col = colors.offset((i * 4 + 3) as isize);
        let f: f32 = 1.0 - R_FogFactor(texCoords[i as usize][0], texCoords[i as usize][1]);
        *col = ((*col as f32) * f) as u8;
        i += 1;
    }
}

/*
** RB_CalcModulateRGBAsByFog
*/
#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcModulateRGBAsByFog_xbox(_colors: *mut u32) {

}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcModulateRGBAsByFog(colors: *mut u8) {
    let mut i: c_int;
    let mut texCoords: [[f32; 2]; SHADER_MAX_VERTEXES] = [[0.0; 2]; SHADER_MAX_VERTEXES];

    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    RB_CalcFogTexCoords(&mut texCoords[0][0]);

    i = 0;
    while i < tess.numVertexes {
        let col = colors.offset((i * 4) as isize);
        let f: f32 = 1.0 - R_FogFactor(texCoords[i as usize][0], texCoords[i as usize][1]);
        *col.offset(0) = ((*col.offset(0) as f32) * f) as u8;
        *col.offset(1) = ((*col.offset(1) as f32) * f) as u8;
        *col.offset(2) = ((*col.offset(2) as f32) * f) as u8;
        *col.offset(3) = ((*col.offset(3) as f32) * f) as u8;
        i += 1;
    }
}


/*
====================================================================

TEX COORDS

====================================================================
*/

/*
========================
RB_CalcFogTexCoords

To do the clipped fog plane really correctly, we should use
projected textures, but I don't trust the drivers and it
doesn't fit our shader data.
========================
*/

// Note: RB_CalcFogTexCoords is declared extern and implemented elsewhere

/*
** RB_CalcEnvironmentTexCoords
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcEnvironmentTexCoords(st: *mut f32) {
    let mut i: c_int;
    let mut v: *mut f32;
    let mut normal: *mut f32;
    let mut viewer: vec3_t = [0.0; 3];
    let mut d: f32;

    v = tess.xyz[0].as_ptr() as *mut f32;
    normal = tess.normal[0].as_ptr() as *mut f32;

    if !backEnd.currentEntity.is_null() && ((*backEnd.currentEntity).e.renderfx & RF_FIRST_PERSON) != 0 {
        // this is a view model so we must use world lights instead of vieworg
        i = 0;
        let mut st_ptr = st;
        while i < tess.numVertexes {
            d = DotProduct(normal as *const _, &(*backEnd.currentEntity).lightDir);
            *st_ptr.offset(0) = *normal.offset(0) * d - (*backEnd.currentEntity).lightDir[0];
            *st_ptr.offset(1) = *normal.offset(1) * d - (*backEnd.currentEntity).lightDir[1];

            i += 1;
            v = v.offset(4);
            normal = normal.offset(4);
            st_ptr = st_ptr.offset(2);
        }
    } else {
        // the normal way
        i = 0;
        let mut st_ptr = st;
        while i < tess.numVertexes {
            VectorSubtract(&backEnd.ori.viewOrigin, v as *const _, &mut viewer);
            VectorNormalizeFast(&mut viewer);

            d = DotProduct(normal as *const _, &viewer);
            *st_ptr.offset(0) = *normal.offset(0) * d - 0.5 * viewer[0];
            *st_ptr.offset(1) = *normal.offset(1) * d - 0.5 * viewer[1];

            i += 1;
            v = v.offset(4);
            normal = normal.offset(4);
            st_ptr = st_ptr.offset(2);
        }
    }
}

/*
** RB_CalcTurbulentTexCoords
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcTurbulentTexCoords(wf: *const waveForm_t, st: *mut f32) {
    let mut i: c_int;
    let now: f32;

    now = ((*wf).phase + backEnd.refdef.floatTime * (*wf).frequency);

    i = 0;
    while i < tess.numVertexes {
        let s: f32 = *st.offset(0);
        let t: f32 = *st.offset(1);

        *st.offset(0) = s + *tr.sinTable.add((((((tess.xyz[i as usize][0] + tess.xyz[i as usize][2]) * 1.0 / 128.0 * 0.125 + now) * FUNCTABLE_SIZE as f32) as c_int) & FUNCTABLE_MASK) as usize) * (*wf).amplitude;
        *st.offset(1) = t + *tr.sinTable.add((((tess.xyz[i as usize][1] * 1.0 / 128.0 * 0.125 + now) * FUNCTABLE_SIZE as f32) as c_int) & FUNCTABLE_MASK) as usize) * (*wf).amplitude;

        i += 1;
        st = st.offset(2);
    }
}

/*
** RB_CalcScaleTexCoords
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcScaleTexCoords(scale: *const [f32; 2], st: *mut f32) {
    let mut i: c_int;

    i = 0;
    while i < tess.numVertexes {
        *st.offset(0) *= (*scale)[0];
        *st.offset(1) *= (*scale)[1];

        i += 1;
        st = st.offset(2);
    }
}

/*
** RB_CalcScrollTexCoords
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcScrollTexCoords(scrollSpeed: *const [f32; 2], st: *mut f32) {
    let mut i: c_int;
    let timeScale: f32 = backEnd.refdef.floatTime;
    let mut adjustedScrollS: f32;
    let mut adjustedScrollT: f32;

    adjustedScrollS = (*scrollSpeed)[0] * timeScale;
    adjustedScrollT = (*scrollSpeed)[1] * timeScale;

    // clamp so coordinates don't continuously get larger, causing problems
    // with hardware limits
    adjustedScrollS = adjustedScrollS - adjustedScrollS.floor();
    adjustedScrollT = adjustedScrollT - adjustedScrollT.floor();

    i = 0;
    while i < tess.numVertexes {
        *st.offset(0) += adjustedScrollS;
        *st.offset(1) += adjustedScrollT;

        i += 1;
        st = st.offset(2);
    }
}

/*
** RB_CalcTransformTexCoords
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcTransformTexCoords(tmi: *const texModInfo_t, st: *mut f32) {
    let mut i: c_int;

    i = 0;
    while i < tess.numVertexes {
        let s: f32 = *st.offset(0);
        let t: f32 = *st.offset(1);

        *st.offset(0) = s * (*tmi).matrix[0][0] + t * (*tmi).matrix[1][0] + (*tmi).translate[0];
        *st.offset(1) = s * (*tmi).matrix[0][1] + t * (*tmi).matrix[1][1] + (*tmi).translate[1];

        i += 1;
        st = st.offset(2);
    }
}


#[no_mangle]
pub unsafe extern "C" fn RB_CalcRotateTexCoords(degsPerSecond: f32, st: *mut f32) {
    let timeScale: f32 = backEnd.refdef.floatTime;
    let mut degs: f32;
    let mut index: c_int;
    let mut sinValue: f32;
    let mut cosValue: f32;
    let mut tmi: texModInfo_t;

    degs = -degsPerSecond * timeScale;
    index = (degs * (FUNCTABLE_SIZE as f32 / 360.0f32)) as c_int;

    sinValue = *tr.sinTable.add((index & FUNCTABLE_MASK) as usize);
    cosValue = *tr.sinTable.add((((index + FUNCTABLE_SIZE / 4) & FUNCTABLE_MASK)) as usize);

    tmi.matrix[0][0] = cosValue;
    tmi.matrix[1][0] = -sinValue;
    tmi.translate[0] = 0.5 - 0.5 * cosValue + 0.5 * sinValue;

    tmi.matrix[0][1] = sinValue;
    tmi.matrix[1][1] = cosValue;
    tmi.translate[1] = 0.5 - 0.5 * sinValue - 0.5 * cosValue;

    RB_CalcTransformTexCoords(&tmi, st);
}

/*
** RB_CalcSpecularAlpha
**
** Calculates specular coefficient and places it in the alpha channel
*/
static mut lightOrigin: vec3_t = [-960.0, 1980.0, 96.0];		// FIXME: track dynamically

#[cfg(any())]  // Placeholder for _XBOX conditional
#[no_mangle]
pub unsafe extern "C" fn RB_CalcSpecularAlpha_xbox(alphas: *mut u32) {
    let mut i: c_int;
    let mut v: *mut f32;
    let mut normal: *mut f32;
    let mut viewer: vec3_t = [0.0; 3];
    let mut reflected: vec3_t = [0.0; 3];
    let mut l: f32;
    let mut d: f32;
    let mut a: c_int;
    let mut lightDir: vec3_t = [0.0; 3];
    let numVertexes: c_int;

    v = tess.xyz[0].as_ptr() as *mut f32;
    normal = tess.normal[0].as_ptr() as *mut f32;

    numVertexes = tess.numVertexes;
    i = 0;
    while i < numVertexes {
        let _ilength: f32;

        if !backEnd.currentEntity.is_null() && (!(*backEnd.currentEntity).e.hModel.is_null() || !(*backEnd.currentEntity).e.ghoul2.is_null()) {
            VectorCopy(&(*backEnd.currentEntity).lightDir, &mut lightDir);
        } else {
            VectorSubtract(&lightOrigin, v as *const _, &mut lightDir);
            VectorNormalizeFast(&mut lightDir);
        }
        // calculate the specular color
        d = 2.0 * DotProduct(normal as *const _, &lightDir);

        // we don't optimize for the d < 0 case since this tends to
        // cause visual artifacts such as faceted "snapping"
        reflected[0] = *normal.offset(0) * d - lightDir[0];
        reflected[1] = *normal.offset(1) * d - lightDir[1];
        reflected[2] = *normal.offset(2) * d - lightDir[2];

        VectorSubtract(&backEnd.ori.viewOrigin, v as *const _, &mut viewer);
        let ilength: f32 = Q_rsqrt(DotProduct(&viewer, &viewer));
        l = DotProduct(&reflected, &viewer);
        l *= ilength;

        if l < 0.0 {
            a = 0;
        } else {
            l = l * l;
            l = l * l;
            a = (l * 255.0) as c_int;
            if a > 255 {
                a = 255;
            }
        }
        let rgb: u32 = ((*alphas) & 0x00ffffff);

        *alphas = rgb | ((a as u32) & 0xff) << 24;

        i += 1;
        v = v.offset(4);
        normal = normal.offset(4);
        alphas = alphas.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RB_CalcSpecularAlpha(alphas: *mut u8) {
    let mut i: c_int;
    let mut v: *mut f32;
    let mut normal: *mut f32;
    let mut viewer: vec3_t = [0.0; 3];
    let mut reflected: vec3_t = [0.0; 3];
    let mut l: f32;
    let mut d: f32;
    let mut b: c_int;
    let mut lightDir: vec3_t = [0.0; 3];
    let numVertexes: c_int;

    v = tess.xyz[0].as_ptr() as *mut f32;
    normal = tess.normal[0].as_ptr() as *mut f32;

    alphas = alphas.offset(3);

    numVertexes = tess.numVertexes;
    i = 0;
    while i < numVertexes {
        let _ilength: f32;

        if !backEnd.currentEntity.is_null() && (!(*backEnd.currentEntity).e.hModel.is_null() || !(*backEnd.currentEntity).e.ghoul2.is_null()) {
            VectorCopy(&(*backEnd.currentEntity).lightDir, &mut lightDir);
        } else {
            VectorSubtract(&lightOrigin, v as *const _, &mut lightDir);
            VectorNormalizeFast(&mut lightDir);
        }
        // calculate the specular color
        d = 2.0 * DotProduct(normal as *const _, &lightDir);

        // we don't optimize for the d < 0 case since this tends to
        // cause visual artifacts such as faceted "snapping"
        reflected[0] = *normal.offset(0) * d - lightDir[0];
        reflected[1] = *normal.offset(1) * d - lightDir[1];
        reflected[2] = *normal.offset(2) * d - lightDir[2];

        VectorSubtract(&backEnd.ori.viewOrigin, v as *const _, &mut viewer);
        let ilength: f32 = Q_rsqrt(DotProduct(&viewer, &viewer));
        l = DotProduct(&reflected, &viewer);
        l *= ilength;

        if l < 0.0 {
            b = 0;
        } else {
            l = l * l;
            l = l * l;
            b = (l * 255.0) as c_int;
            if b > 255 {
                b = 255;
            }
        }

        *alphas = b as u8;

        i += 1;
        v = v.offset(4);
        normal = normal.offset(4);
        alphas = alphas.offset(4);
    }
}

/*
** RB_CalcDiffuseColor
**
** The basic vertex lighting calc
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcDiffuseColor(colors: *mut u8) {
    let mut i: c_int;
    let mut j: c_int;
    let mut v: *mut f32;
    let mut normal: *mut f32;
    let mut incoming: f32;
    let ent: *mut trRefEntity_t;
    let ambientLightInt: c_int;
    let mut ambientLight: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];
    let mut directedLight: vec3_t = [0.0; 3];
    let numVertexes: c_int;

    ent = backEnd.currentEntity;
    ambientLightInt = (*ent).ambientLightInt;
    VectorCopy(&(*ent).ambientLight, &mut ambientLight);
    VectorCopy(&(*ent).directedLight, &mut directedLight);
    VectorCopy(&(*ent).lightDir, &mut lightDir);

    v = tess.xyz[0].as_ptr() as *mut f32;
    normal = tess.normal[0].as_ptr() as *mut f32;

    numVertexes = tess.numVertexes;

    i = 0;
    while i < numVertexes {
        incoming = DotProduct(normal as *const _, &lightDir);
        if incoming <= 0.0 {
            *(colors.offset((i * 4) as isize) as *mut c_int) = ambientLightInt;
        } else {
            j = myftol(ambientLight[0] + incoming * directedLight[0]);
            if j > 255 {
                j = 255;
            }
            *colors.offset((i * 4 + 0) as isize) = j as u8;

            j = myftol(ambientLight[1] + incoming * directedLight[1]);
            if j > 255 {
                j = 255;
            }
            *colors.offset((i * 4 + 1) as isize) = j as u8;

            j = myftol(ambientLight[2] + incoming * directedLight[2]);
            if j > 255 {
                j = 255;
            }
            *colors.offset((i * 4 + 2) as isize) = j as u8;

            *colors.offset((i * 4 + 3) as isize) = 255;
        }

        i += 1;
        v = v.offset(4);
        normal = normal.offset(4);
    }
}

/*
** RB_CalcDiffuseEntityColor
**
** The basic vertex lighting calc * Entity Color
*/
#[no_mangle]
pub unsafe extern "C" fn RB_CalcDiffuseEntityColor(colors: *mut u8) {
    let mut i: c_int;
    let mut v: *mut f32;
    let mut normal: *mut f32;
    let mut incoming: f32;
    let ent: *mut trRefEntity_t;
    let mut ambientLight: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];
    let mut directedLight: vec3_t = [0.0; 3];
    let numVertexes: c_int;
    let mut j: f32;
    let mut r: f32;
    let mut g: f32;
    let mut b: f32;

    if backEnd.currentEntity.is_null() {
        // error, use the normal lighting
        RB_CalcDiffuseColor(colors);
        return;
    }

    ent = backEnd.currentEntity;
    VectorCopy(&(*ent).ambientLight, &mut ambientLight);
    VectorCopy(&(*ent).directedLight, &mut directedLight);
    VectorCopy(&(*ent).lightDir, &mut lightDir);

    r = (*backEnd.currentEntity).e.shaderRGBA[0] as f32 / 255.0;
    g = (*backEnd.currentEntity).e.shaderRGBA[1] as f32 / 255.0;
    b = (*backEnd.currentEntity).e.shaderRGBA[2] as f32 / 255.0;

    v = tess.xyz[0].as_ptr() as *mut f32;
    normal = tess.normal[0].as_ptr() as *mut f32;

    numVertexes = tess.numVertexes;

    i = 0;
    while i < numVertexes {
        incoming = DotProduct(normal as *const _, &lightDir);
        if incoming <= 0.0 {
            // Use ambient light only
            *colors.offset((i * 4 + 0) as isize) = myftol(r * ambientLight[0]) as u8;
            *colors.offset((i * 4 + 1) as isize) = myftol(g * ambientLight[1]) as u8;
            *colors.offset((i * 4 + 2) as isize) = myftol(b * ambientLight[2]) as u8;
            *colors.offset((i * 4 + 3) as isize) = (*backEnd.currentEntity).e.shaderRGBA[3];
        } else {
            j = (ambientLight[0] + incoming * directedLight[0]);
            if j > 255.0 {
                j = 255.0;
            }
            *colors.offset((i * 4 + 0) as isize) = myftol(j * r) as u8;

            j = (ambientLight[1] + incoming * directedLight[1]);
            if j > 255.0 {
                j = 255.0;
            }
            *colors.offset((i * 4 + 1) as isize) = myftol(j * g) as u8;

            j = (ambientLight[2] + incoming * directedLight[2]);
            if j > 255.0 {
                j = 255.0;
            }
            *colors.offset((i * 4 + 2) as isize) = myftol(j * b) as u8;

            *colors.offset((i * 4 + 3) as isize) = (*backEnd.currentEntity).e.shaderRGBA[3];
        }

        i += 1;
        v = v.offset(4);
        normal = normal.offset(4);
    }
}

//---------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn RB_CalcDisintegrateColors(colors: *mut u8, rgbGen: colorGen_t) {
    let mut i: c_int;
    let mut numVertexes: c_int;
    let mut dis: f32;
    let mut threshold: f32;
    let mut v: *mut f32;
    let mut temp: vec3_t = [0.0; 3];
    let ent: *mut refEntity_t;

    ent = core::ptr::addr_of_mut!((*backEnd.currentEntity).e);
    v = tess.xyz[0].as_ptr() as *mut f32;

    // calculate the burn threshold at the given time, anything that passes the threshold will get burnt
    threshold = (backEnd.refdef.time - (*ent).endTime) * 0.045f32;

    numVertexes = tess.numVertexes;

    if ((*ent).renderfx & RF_DISINTEGRATE1) != 0 {
        // this handles the blacken and fading out of the regular player model
        i = 0;
        while i < numVertexes {
            VectorSubtract(&(*backEnd.currentEntity).e.oldorigin, v as *const _, &mut temp);

            dis = VectorLengthSquared(&temp);

            if dis < threshold * threshold {
                // completely disintegrated
                *colors.offset((i * 4 + 3) as isize) = 0x00;
            } else if dis < threshold * threshold + 60.0 {
                // blacken before fading out
                *colors.offset((i * 4 + 0) as isize) = 0x0;
                *colors.offset((i * 4 + 1) as isize) = 0x0;
                *colors.offset((i * 4 + 2) as isize) = 0x0;
                *colors.offset((i * 4 + 3) as isize) = 0xff;
            } else if dis < threshold * threshold + 150.0 {
                // darken more
                if rgbGen == CGEN_LIGHTING_DIFFUSE_ENTITY {
                    *colors.offset((i * 4 + 0) as isize) = ((*backEnd.currentEntity).e.shaderRGBA[0] as f32 * 0x6f as f32 / 255.0f32) as u8;
                    *colors.offset((i * 4 + 1) as isize) = ((*backEnd.currentEntity).e.shaderRGBA[1] as f32 * 0x6f as f32 / 255.0f32) as u8;
                    *colors.offset((i * 4 + 2) as isize) = ((*backEnd.currentEntity).e.shaderRGBA[2] as f32 * 0x6f as f32 / 255.0f32) as u8;
                } else {
                    *colors.offset((i * 4 + 0) as isize) = 0x6f;
                    *colors.offset((i * 4 + 1) as isize) = 0x6f;
                    *colors.offset((i * 4 + 2) as isize) = 0x6f;
                }
                *colors.offset((i * 4 + 3) as isize) = 0xff;
            } else if dis < threshold * threshold + 180.0 {
                // darken at edge of burn
                if rgbGen == CGEN_LIGHTING_DIFFUSE_ENTITY {
                    *colors.offset((i * 4 + 0) as isize) = ((*backEnd.currentEntity).e.shaderRGBA[0] as f32 * 0xaf as f32 / 255.0f32) as u8;
                    *colors.offset((i * 4 + 1) as isize) = ((*backEnd.currentEntity).e.shaderRGBA[1] as f32 * 0xaf as f32 / 255.0f32) as u8;
                    *colors.offset((i * 4 + 2) as isize) = ((*backEnd.currentEntity).e.shaderRGBA[2] as f32 * 0xaf as f32 / 255.0f32) as u8;
                } else {
                    *colors.offset((i * 4 + 0) as isize) = 0xaf;
                    *colors.offset((i * 4 + 1) as isize) = 0xaf;
                    *colors.offset((i * 4 + 2) as isize) = 0xaf;
                }
                *colors.offset((i * 4 + 3) as isize) = 0xff;
            } else {
                // not burning at all yet
                if rgbGen == CGEN_LIGHTING_DIFFUSE_ENTITY {
                    *colors.offset((i * 4 + 0) as isize) = (*backEnd.currentEntity).e.shaderRGBA[0];
                    *colors.offset((i * 4 + 1) as isize) = (*backEnd.currentEntity).e.shaderRGBA[1];
                    *colors.offset((i * 4 + 2) as isize) = (*backEnd.currentEntity).e.shaderRGBA[2];
                } else {
                    *colors.offset((i * 4 + 0) as isize) = 0xff;
                    *colors.offset((i * 4 + 1) as isize) = 0xff;
                    *colors.offset((i * 4 + 2) as isize) = 0xff;
                }
                *colors.offset((i * 4 + 3) as isize) = 0xff;
            }

            i += 1;
            v = v.offset(4);
        }
    } else if ((*ent).renderfx & RF_DISINTEGRATE2) != 0 {
        // this handles the glowing, burning bit that scales away from the model
        i = 0;
        while i < numVertexes {
            VectorSubtract(&(*backEnd.currentEntity).e.oldorigin, v as *const _, &mut temp);

            dis = VectorLengthSquared(&temp);

            if dis < threshold * threshold {
                // done burning
                *colors.offset((i * 4 + 0) as isize) = 0x00;
                *colors.offset((i * 4 + 1) as isize) = 0x00;
                *colors.offset((i * 4 + 2) as isize) = 0x00;
                *colors.offset((i * 4 + 3) as isize) = 0x00;
            } else {
                // still full burn
                *colors.offset((i * 4 + 0) as isize) = 0xff;
                *colors.offset((i * 4 + 1) as isize) = 0xff;
                *colors.offset((i * 4 + 2) as isize) = 0xff;
                *colors.offset((i * 4 + 3) as isize) = 0xff;
            }

            i += 1;
            v = v.offset(4);
        }
    }
}

//---------------------------------------------------------
#[no_mangle]
pub unsafe extern "C" fn RB_CalcDisintegrateVertDeform() {
    let mut xyz: *mut f32 = tess.xyz as *mut f32;
    let mut normal: *mut f32 = tess.normal as *mut f32;
    let mut scale: f32;
    let mut temp: vec3_t = [0.0; 3];

    if ((*backEnd.currentEntity).e.renderfx & RF_DISINTEGRATE2) != 0 {
        let threshold: f32 = (backEnd.refdef.time - (*backEnd.currentEntity).e.endTime) * 0.045f32;

        let mut i: c_int = 0;
        while i < tess.numVertexes {
            VectorSubtract(&(*backEnd.currentEntity).e.oldorigin, xyz as *const _, &mut temp);

            scale = VectorLengthSquared(&temp);

            if scale < threshold * threshold {
                *xyz.offset(0) += *normal.offset(0) * 2.0f32;
                *xyz.offset(1) += *normal.offset(1) * 2.0f32;
                *xyz.offset(2) += *normal.offset(2) * 0.5f32;
            } else if scale < threshold * threshold + 50.0 {
                *xyz.offset(0) += *normal.offset(0) * 1.0f32;
                *xyz.offset(1) += *normal.offset(1) * 1.0f32;
//				xyz[2] += normal[2] * 1;
            }

            i += 1;
            xyz = xyz.offset(4);
            normal = normal.offset(4);
        }
    }
}
