// tr_shade_calc.c

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Type aliases and external dependencies (from tr_local.h)
type genFunc_t = c_int;
type qboolean = c_int;
type byte = u8;

const GF_SIN: genFunc_t = 0;
const GF_TRIANGLE: genFunc_t = 1;
const GF_SQUARE: genFunc_t = 2;
const GF_SAWTOOTH: genFunc_t = 3;
const GF_INVERSE_SAWTOOTH: genFunc_t = 4;
const GF_NOISE: genFunc_t = 5;
const GF_RAND: genFunc_t = 6;
const GF_NONE: genFunc_t = 7;

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

const FUNCTABLE_SIZE: usize = 1024;
const FUNCTABLE_MASK: usize = FUNCTABLE_SIZE - 1;

const NUM_TEX_COORDS: usize = 1;
const SHADER_MAX_VERTEXES: usize = 4000;

const S_COLOR_YELLOW: &str = "^3";
const ERR_DROP: c_int = 0;
const RF_DISINTEGRATE1: c_int = 1;
const RF_DISINTEGRATE2: c_int = 2;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// Vector types (matching C)
type vec3_t = [f32; 3];
type vec4_t = [f32; 4];

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
    pub bulgeWidth: f32,
    pub bulgeHeight: f32,
    pub bulgeSpeed: f32,
    pub moveVector: vec3_t,
}

#[repr(C)]
pub struct fog_t {
    pub origin: vec3_t,
    pub radius: f32,
    pub color: [u8; 3],
    pub colorInt: u32,
    pub tcScale: f32,
    pub depthForOpaque: f32,
    pub eyeT: f32,
    pub hasSurface: qboolean,
    pub surface: vec4_t,
}

#[repr(C)]
pub struct refEntity_t {
    pub hModel: *mut c_void,
    pub ghoul2: *mut c_void,
    pub lightDir: vec3_t,
    pub oldorigin: vec3_t,
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: qboolean,
    pub endTime: c_int,
    pub renderfx: c_int,
    pub shaderRGBA: [u8; 4],
}

#[repr(C)]
pub struct trRefEntity_t {
    pub e: refEntity_t,
    pub lightDir: vec3_t,
    pub ambientLight: vec3_t,
    pub directedLight: vec3_t,
    pub ambientLightInt: c_int,
}

#[repr(C)]
pub struct shader_t {
    pub name: [c_char; 256],
    pub numDeforms: c_int,
    pub deforms: [*mut deformStage_t; 16],
}

#[repr(C)]
pub struct orientationr_t {
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
    pub viewOrigin: vec3_t,
    pub modelMatrix: [f32; 16],
}

#[repr(C)]
pub struct refdef_t {
    pub time: c_int,
    pub floatTime: f32,
    pub text: [[c_char; 256]; 8],
}

#[repr(C)]
pub struct viewParms_t {
    pub ori: orientationr_t,
    pub isMirror: qboolean,
}

#[repr(C)]
pub struct backEndState_t {
    pub currentEntity: *mut trRefEntity_t,
    pub refdef: refdef_t,
    pub viewParms: viewParms_t,
    pub ori: orientationr_t,
}

#[repr(C)]
pub struct trGlobal_t {
    pub sinTable: [f32; FUNCTABLE_SIZE],
    pub triangleTable: [f32; FUNCTABLE_SIZE],
    pub squareTable: [f32; FUNCTABLE_SIZE],
    pub sawToothTable: [f32; FUNCTABLE_SIZE],
    pub inverseSawToothTable: [f32; FUNCTABLE_SIZE],
    pub identityLight: f32,
    pub world: *mut world_t,
    pub worldEntity: trRefEntity_t,
}

#[repr(C)]
pub struct world_t {
    pub fogs: *mut fog_t,
}

// Global state structures (matching C)
pub struct tessellation_t {
    pub xyz: *mut f32,
    pub normal: *mut f32,
    pub texCoords: [*mut f32; 1],
    pub vertexColors: *mut u8,
    pub indexes: *mut u32,
    pub numVertexes: c_int,
    pub numIndexes: c_int,
    pub fogNum: c_int,
    pub shader: *mut shader_t,
    pub shaderTime: f32,
}

// Extern declarations for globals
extern "C" {
    pub static mut tr: trGlobal_t;
    pub static mut tess: tessellation_t;
    pub static mut backEnd: backEndState_t;
}

// External function declarations
extern "C" {
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn R_NoiseGet4f(x: f32, y: f32, z: f32, t: f32) -> f32;
    pub fn GetNoiseTime(t: c_int) -> f32;
    pub fn RB_CalcTransformTexCoords(tmi: *const texModInfo_t, st: *mut f32);
    pub fn RB_AddQuadStampExt(
        origin: *const vec3_t,
        width: *const vec3_t,
        height: *const vec3_t,
        color: *const [u8; 4],
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
    );
    pub fn RB_AddQuadStamp(
        origin: *const vec3_t,
        left: *const vec3_t,
        up: *const vec3_t,
        color: *const [u8; 4],
    );
    pub fn RB_ProjectionShadowDeform();
    pub fn RB_CalcFogTexCoords(st: *mut f32);
    pub fn VectorNormalizeFast(v: *mut vec3_t) -> f32;
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn R_FogFactor(s: f32, t: f32) -> f32;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn Q_rsqrt(x: f32) -> f32;
}

// Helper macros/inline functions
#[inline]
fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t) {
    unsafe {
        (*out)[0] = (*v)[0] * scale;
        (*out)[1] = (*v)[1] * scale;
        (*out)[2] = (*v)[2] * scale;
    }
}

#[inline]
fn VectorAdd(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t) {
    unsafe {
        (*out)[0] = (*a)[0] + (*b)[0];
        (*out)[1] = (*a)[1] + (*b)[1];
        (*out)[2] = (*a)[2] + (*b)[2];
    }
}

#[inline]
fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t) {
    unsafe {
        (*out)[0] = (*a)[0] - (*b)[0];
        (*out)[1] = (*a)[1] - (*b)[1];
        (*out)[2] = (*a)[2] - (*b)[2];
    }
}

#[inline]
fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t) {
    unsafe {
        (*dst)[0] = (*src)[0];
        (*dst)[1] = (*src)[1];
        (*dst)[2] = (*src)[2];
    }
}

#[inline]
fn VectorClear(v: *mut vec3_t) {
    unsafe {
        (*v)[0] = 0.0;
        (*v)[1] = 0.0;
        (*v)[2] = 0.0;
    }
}

#[inline]
fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32 {
    unsafe { (*a)[0] * (*b)[0] + (*a)[1] * (*b)[1] + (*a)[2] * (*b)[2] }
}

#[inline]
fn CrossProduct(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t) {
    unsafe {
        (*out)[0] = (*a)[1] * (*b)[2] - (*a)[2] * (*b)[1];
        (*out)[1] = (*a)[2] * (*b)[0] - (*a)[0] * (*b)[2];
        (*out)[2] = (*a)[0] * (*b)[1] - (*a)[1] * (*b)[0];
    }
}

#[inline]
fn VectorLength(v: *const vec3_t) -> f32 {
    unsafe {
        ((*v)[0] * (*v)[0] + (*v)[1] * (*v)[1] + (*v)[2] * (*v)[2]).sqrt()
    }
}

#[inline]
fn VectorLengthSquared(v: *const vec3_t) -> f32 {
    unsafe { (*v)[0] * (*v)[0] + (*v)[1] * (*v)[1] + (*v)[2] * (*v)[2] }
}

#[inline]
fn VectorMA(a: *const vec3_t, scale: f32, b: *const vec3_t, out: *mut vec3_t) {
    unsafe {
        (*out)[0] = (*a)[0] + scale * (*b)[0];
        (*out)[1] = (*a)[1] + scale * (*b)[1];
        (*out)[2] = (*a)[2] + scale * (*b)[2];
    }
}

// Fast conversion to integer (handles platform-specific inline asm)
// Note: The original C code uses inline assembly on x86. This is a fallback
// for non-x86 or when inline asm is not available.
#[inline]
fn myftol(f: f32) -> c_int {
    f as c_int
}

// Vector constant
static vec3_origin: vec3_t = [0.0; 3];

#[inline]
unsafe fn WAVEVALUE(table: *const f32, base: f32, amplitude: f32, phase: f32, freq: f32) -> f32 {
    base + (*table.add((myftol((phase + tess.shaderTime * freq) * FUNCTABLE_SIZE as f32) as usize) & FUNCTABLE_MASK)) * amplitude
}

unsafe fn TableForFunc(func: genFunc_t) -> *mut f32 {
    match func {
        GF_SIN => tr.sinTable.as_mut_ptr(),
        GF_TRIANGLE => tr.triangleTable.as_mut_ptr(),
        GF_SQUARE => tr.squareTable.as_mut_ptr(),
        GF_SAWTOOTH => tr.sawToothTable.as_mut_ptr(),
        GF_INVERSE_SAWTOOTH => tr.inverseSawToothTable.as_mut_ptr(),
        GF_NONE | _ => {
            Com_Error(
                ERR_DROP,
                b"TableForFunc called with invalid function '%d' in shader '%s'\n\0".as_ptr() as *const c_char,
                func,
                (*(*tess.shader).name.as_ptr()) as *const c_char,
            );
            core::ptr::null_mut()
        }
    }
}

/*
** EvalWaveForm
**
** Evaluates a given waveForm_t, referencing backEnd.refdef.time directly
*/
unsafe fn EvalWaveForm(wf: *const waveForm_t) -> f32 {
    if (*wf).func == GF_NOISE {
        return (*wf).base + R_NoiseGet4f(0.0, 0.0, 0.0, (backEnd.refdef.floatTime + (*wf).phase) * (*wf).frequency) * (*wf).amplitude;
    } else if (*wf).func == GF_RAND {
        if GetNoiseTime(backEnd.refdef.time + (*wf).phase as c_int) <= (*wf).frequency {
            return (*wf).base + (*wf).amplitude;
        } else {
            return (*wf).base;
        }
    }
    let table = TableForFunc((*wf).func);

    WAVEVALUE(table, (*wf).base, (*wf).amplitude, (*wf).phase, (*wf).frequency)
}

unsafe fn EvalWaveFormClamped(wf: *const waveForm_t) -> f32 {
    let glow = EvalWaveForm(wf);

    if glow < 0.0 {
        return 0.0;
    }

    if glow > 1.0 {
        return 1.0;
    }

    glow
}

/*
** RB_CalcStretchTexCoords
*/
pub unsafe fn RB_CalcStretchTexCoords(wf: *const waveForm_t, st: *mut f32) {
    let p = 1.0 / EvalWaveForm(wf);

    let mut tmi: texModInfo_t = core::mem::zeroed();
    tmi.matrix[0][0] = p;
    tmi.matrix[1][0] = 0.0;
    tmi.translate[0] = 0.5 - 0.5 * p;

    tmi.matrix[0][1] = 0.0;
    tmi.matrix[1][1] = p;
    tmi.translate[1] = 0.5 - 0.5 * p;

    RB_CalcTransformTexCoords(addr_of!(tmi), st);
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
pub unsafe fn RB_CalcDeformVertexes(ds: *mut deformStage_t) {
    let mut offset: vec3_t = [0.0; 3];
    let xyz = tess.xyz as *mut f32;
    let normal = tess.normal as *mut f32;

    if (*ds).deformationWave.frequency == 0.0 {
        let scale = EvalWaveForm(addr_of!((*ds).deformationWave));

        for i in 0..tess.numVertexes {
            let mut xyz_ptr = xyz.add((i as usize) * 4);
            let mut normal_ptr = normal.add((i as usize) * 4);

            VectorScale(normal_ptr as *const vec3_t, scale, addr_of_mut!(offset));

            *xyz_ptr = *xyz_ptr + offset[0];
            *xyz_ptr.add(1) = *xyz_ptr.add(1) + offset[1];
            *xyz_ptr.add(2) = *xyz_ptr.add(2) + offset[2];
        }
    } else {
        let table = TableForFunc((*ds).deformationWave.func);

        for i in 0..tess.numVertexes {
            let mut xyz_ptr = xyz.add((i as usize) * 4);
            let mut normal_ptr = normal.add((i as usize) * 4);

            let off = (*xyz_ptr + *xyz_ptr.add(1) + *xyz_ptr.add(2)) * (*ds).deformationSpread;

            let scale = WAVEVALUE(table, (*ds).deformationWave.base,
                (*ds).deformationWave.amplitude,
                (*ds).deformationWave.phase + off,
                (*ds).deformationWave.frequency);

            VectorScale(normal_ptr as *const vec3_t, scale, addr_of_mut!(offset));

            *xyz_ptr = *xyz_ptr + offset[0];
            *xyz_ptr.add(1) = *xyz_ptr.add(1) + offset[1];
            *xyz_ptr.add(2) = *xyz_ptr.add(2) + offset[2];
        }
    }
}

/*
=========================
RB_CalcDeformNormals

Wiggle the normals for wavy environment mapping
=========================
*/
pub unsafe fn RB_CalcDeformNormals(ds: *mut deformStage_t) {
    let xyz = tess.xyz as *mut f32;
    let normal = tess.normal as *mut f32;

    for i in 0..tess.numVertexes {
        let xyz_ptr = xyz.add((i as usize) * 4);
        let normal_ptr = normal.add((i as usize) * 4);

        let mut scale = 0.98;
        scale = R_NoiseGet4f(*xyz_ptr * scale, *xyz_ptr.add(1) * scale, *xyz_ptr.add(2) * scale,
            tess.shaderTime * (*ds).deformationWave.frequency);
        *normal_ptr = *normal_ptr + (*ds).deformationWave.amplitude * scale;

        scale = 0.98;
        scale = R_NoiseGet4f(100.0 + *xyz_ptr * scale, *xyz_ptr.add(1) * scale, *xyz_ptr.add(2) * scale,
            tess.shaderTime * (*ds).deformationWave.frequency);
        *normal_ptr.add(1) = *normal_ptr.add(1) + (*ds).deformationWave.amplitude * scale;

        scale = 0.98;
        scale = R_NoiseGet4f(200.0 + *xyz_ptr * scale, *xyz_ptr.add(1) * scale, *xyz_ptr.add(2) * scale,
            tess.shaderTime * (*ds).deformationWave.frequency);
        *normal_ptr.add(2) = *normal_ptr.add(2) + (*ds).deformationWave.amplitude * scale;

        VectorNormalizeFast(normal_ptr as *mut vec3_t);
    }
}

/*
========================
RB_CalcBulgeVertexes

========================
*/
pub unsafe fn RB_CalcBulgeVertexes(ds: *mut deformStage_t) {
    //Old bulge code:
    /*
    int i;
    const float *st = ( const float * ) tess.texCoords[0];
    float		*xyz = ( float * ) tess.xyz;
    float		*normal = ( float * ) tess.normal;
    float		now;

    now = backEnd.refdef.time * ds->bulgeSpeed * 0.001f;

    for ( i = 0; i < tess.numVertexes; i++, xyz += 4, st += 2 * NUM_TEX_COORDS, normal += 4 ) {
        int		off;
        float scale;

        off = (float)( FUNCTABLE_SIZE / (M_PI*2) ) * ( st[0] * ds->bulgeWidth + now );

        scale = tr.sinTable[ off & FUNCTABLE_MASK ] * ds->bulgeHeight;

        xyz[0] += normal[0] * scale;
        xyz[1] += normal[1] * scale;
        xyz[2] += normal[2] * scale;
    }
    */

    let xyz = tess.xyz as *mut f32;
    let normal = tess.normal as *mut f32;

    if (*ds).bulgeSpeed == 0.0 && (*ds).bulgeWidth == 0.0 {
        // We don't have a speed and width, so just use height to expand uniformly
        for i in 0..tess.numVertexes {
            let xyz_ptr = xyz.add((i as usize) * 4);
            let normal_ptr = normal.add((i as usize) * 4);

            *xyz_ptr = *xyz_ptr + *normal_ptr * (*ds).bulgeHeight;
            *xyz_ptr.add(1) = *xyz_ptr.add(1) + *normal_ptr.add(1) * (*ds).bulgeHeight;
            *xyz_ptr.add(2) = *xyz_ptr.add(2) + *normal_ptr.add(2) * (*ds).bulgeHeight;
        }
    } else {
        // I guess do some extra dumb stuff..the fact that it uses ST seems bad though because skin pages may be set up in certain ways that can cause
        //	very noticeable seams on sufaces ( like on the huge ion_cannon ).
        let st = tess.texCoords[0] as *mut f32;
        let now = backEnd.refdef.time as f32 * (*ds).bulgeSpeed * 0.001;

        for i in 0..tess.numVertexes {
            let xyz_ptr = xyz.add((i as usize) * 4);
            let st_ptr = st.add((i as usize) * 2 * NUM_TEX_COORDS);
            let normal_ptr = normal.add((i as usize) * 4);

            let off = ((FUNCTABLE_SIZE as f32 / (std::f32::consts::PI * 2.0)) * (*st_ptr * (*ds).bulgeWidth + now)) as usize;

            let scale = tr.sinTable[off & FUNCTABLE_MASK] * (*ds).bulgeHeight;

            *xyz_ptr = *xyz_ptr + *normal_ptr * scale;
            *xyz_ptr.add(1) = *xyz_ptr.add(1) + *normal_ptr.add(1) * scale;
            *xyz_ptr.add(2) = *xyz_ptr.add(2) + *normal_ptr.add(2) * scale;
        }
    }
}


/*
======================
RB_CalcMoveVertexes

A deformation that can move an entire surface along a wave path
======================
*/
pub unsafe fn RB_CalcMoveVertexes(ds: *mut deformStage_t) {
    let table = TableForFunc((*ds).deformationWave.func);

    let scale = WAVEVALUE(table, (*ds).deformationWave.base,
        (*ds).deformationWave.amplitude,
        (*ds).deformationWave.phase,
        (*ds).deformationWave.frequency);

    let mut offset: vec3_t = [0.0; 3];
    VectorScale(addr_of!((*ds).moveVector), scale, addr_of_mut!(offset));

    let xyz = tess.xyz as *mut f32;
    for i in 0..tess.numVertexes {
        let xyz_ptr = xyz.add((i as usize) * 4);
        VectorAdd(xyz_ptr as *const vec3_t, addr_of!(offset), xyz_ptr as *mut vec3_t);
    }
}


/*
=============
DeformText

Change a polygon into a bunch of text polygons
=============
*/
pub unsafe fn DeformText(text: *const c_char) {
    let mut origin: vec3_t = [0.0; 3];
    let mut width: vec3_t = [0.0; 3];
    let mut height: vec3_t = [0.0; 3];
    let mut mid: vec3_t = [0.0; 3];
    let mut color: [u8; 4] = [0; 4];

    height[0] = 0.0;
    height[1] = 0.0;
    height[2] = -1.0;
    CrossProduct(addr_of!((*tess.normal)[0]), addr_of!(height), addr_of_mut!(width));

    // find the midpoint of the box
    VectorClear(addr_of_mut!(mid));
    let mut bottom = 999999.0;
    let mut top = -999999.0;
    for i in 0..4 {
        VectorAdd(addr_of!((*tess.xyz)[i]), addr_of!(mid), addr_of_mut!(mid));
        if (*tess.xyz)[i][2] < bottom {
            bottom = (*tess.xyz)[i][2];
        }
        if (*tess.xyz)[i][2] > top {
            top = (*tess.xyz)[i][2];
        }
    }
    VectorScale(addr_of!(mid), 0.25, addr_of_mut!(origin));

    // determine the individual character size
    height[0] = 0.0;
    height[1] = 0.0;
    height[2] = (top - bottom) * 0.5;

    VectorScale(addr_of!(width), height[2] * -0.75, addr_of_mut!(width));

    // determine the starting position
    let len = strlen(text);
    VectorMA(addr_of!(origin), ((len - 1) as f32), addr_of!(width), addr_of_mut!(origin));

    // clear the shader indexes
    tess.numIndexes = 0;
    tess.numVertexes = 0;

    color[0] = 255;
    color[1] = 255;
    color[2] = 255;
    color[3] = 255;

    // draw each character
    for i in 0..len {
        let ch = (*text.add(i)) as u8 as u32;
        let ch = ch & 255;

        if ch != b' ' as u32 {
            let row = (ch >> 4) as f32;
            let col = (ch & 15) as f32;

            let frow = row * 0.0625;
            let fcol = col * 0.0625;
            let size = 0.0625;

            RB_AddQuadStampExt(addr_of!(origin), addr_of!(width), addr_of!(height), addr_of!(color), fcol, frow, fcol + size, frow + size);
        }
        VectorMA(addr_of!(origin), -2.0, addr_of!(width), addr_of_mut!(origin));
    }
}

/*
==================
GlobalVectorToLocal
==================
*/
unsafe fn GlobalVectorToLocal(in_vec: *const vec3_t, out: *mut vec3_t) {
    (*out)[0] = DotProduct(in_vec, addr_of!(backEnd.ori.axis[0]));
    (*out)[1] = DotProduct(in_vec, addr_of!(backEnd.ori.axis[1]));
    (*out)[2] = DotProduct(in_vec, addr_of!(backEnd.ori.axis[2]));
}

/*
=====================
AutospriteDeform

Assuming all the triangles for this shader are independant
quads, rebuild them as forward facing sprites
=====================
*/
unsafe fn AutospriteDeform() {
    let mut mid: vec3_t = [0.0; 3];
    let mut delta: vec3_t = [0.0; 3];
    let mut left: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut leftDir: vec3_t = [0.0; 3];
    let mut upDir: vec3_t = [0.0; 3];

    if (tess.numVertexes & 3) != 0 {
        Com_Printf(b"%sAutosprite shader %s had odd vertex count\0".as_ptr() as *const c_char, S_COLOR_YELLOW.as_ptr() as *const c_char, (*(*tess.shader)).name.as_ptr() as *const c_char);
    }
    if tess.numIndexes != ((tess.numVertexes >> 2) * 6) {
        Com_Printf(b"%sAutosprite shader %s had odd index count\0".as_ptr() as *const c_char, S_COLOR_YELLOW.as_ptr() as *const c_char, (*(*tess.shader)).name.as_ptr() as *const c_char);
    }

    let oldVerts = tess.numVertexes;
    tess.numVertexes = 0;
    tess.numIndexes = 0;

    if backEnd.currentEntity != addr_of_mut!(tr.worldEntity) {
        GlobalVectorToLocal(addr_of!(backEnd.viewParms.ori.axis[1]), addr_of_mut!(leftDir));
        GlobalVectorToLocal(addr_of!(backEnd.viewParms.ori.axis[2]), addr_of_mut!(upDir));
    } else {
        VectorCopy(addr_of!(backEnd.viewParms.ori.axis[1]), addr_of_mut!(leftDir));
        VectorCopy(addr_of!(backEnd.viewParms.ori.axis[2]), addr_of_mut!(upDir));
    }

    let mut i = 0;
    while i < oldVerts {
        // find the midpoint
        let xyz = addr_of!((*tess.xyz)[i]);

        mid[0] = 0.25 * ((*tess.xyz)[i][0] + (*tess.xyz)[i + 1][0] + (*tess.xyz)[i + 2][0] + (*tess.xyz)[i + 3][0]);
        mid[1] = 0.25 * ((*tess.xyz)[i][1] + (*tess.xyz)[i + 1][1] + (*tess.xyz)[i + 2][1] + (*tess.xyz)[i + 3][1]);
        mid[2] = 0.25 * ((*tess.xyz)[i][2] + (*tess.xyz)[i + 1][2] + (*tess.xyz)[i + 2][2] + (*tess.xyz)[i + 3][2]);

        VectorSubtract(xyz, addr_of!(mid), addr_of_mut!(delta));
        let radius = VectorLength(addr_of!(delta)) * 0.707; // / sqrt(2)

        VectorScale(addr_of!(leftDir), radius, addr_of_mut!(left));
        VectorScale(addr_of!(upDir), radius, addr_of_mut!(up));

        if backEnd.viewParms.isMirror != 0 {
            VectorSubtract(addr_of!(vec3_origin), addr_of!(left), addr_of_mut!(left));
        }

        // compensate for scale in the axes if necessary
        if (*backEnd.currentEntity).e.nonNormalizedAxes != 0 {
            let mut axisLength = VectorLength(addr_of!((*backEnd.currentEntity).e.axis[0]));
            if axisLength == 0.0 {
                axisLength = 0.0;
            } else {
                axisLength = 1.0 / axisLength;
            }
            VectorScale(addr_of!(left), axisLength, addr_of_mut!(left));
            VectorScale(addr_of!(up), axisLength, addr_of_mut!(up));
        }

        RB_AddQuadStamp(addr_of!(mid), addr_of!(left), addr_of!(up), addr_of!((*tess.vertexColors)[i]));

        i += 4;
    }
}


/*
=====================
Autosprite2Deform

Autosprite2 will pivot a rectangular quad along the center of its long axis
=====================
*/
static edgeVerts: [[c_int; 2]; 6] = [
    [0, 1],
    [0, 2],
    [0, 3],
    [1, 2],
    [1, 3],
    [2, 3],
];

unsafe fn Autosprite2Deform() {
    let mut forward: vec3_t = [0.0; 3];

    if (tess.numVertexes & 3) != 0 {
        Com_Printf(b"%sAutosprite2 shader %s had odd vertex count\0".as_ptr() as *const c_char, S_COLOR_YELLOW.as_ptr() as *const c_char, (*(*tess.shader)).name.as_ptr() as *const c_char);
    }
    if tess.numIndexes != ((tess.numVertexes >> 2) * 6) {
        Com_Printf(b"%sAutosprite2 shader %s had odd index count\0".as_ptr() as *const c_char, S_COLOR_YELLOW.as_ptr() as *const c_char, (*(*tess.shader)).name.as_ptr() as *const c_char);
    }

    if backEnd.currentEntity != addr_of_mut!(tr.worldEntity) {
        GlobalVectorToLocal(addr_of!(backEnd.viewParms.ori.axis[0]), addr_of_mut!(forward));
    } else {
        VectorCopy(addr_of!(backEnd.viewParms.ori.axis[0]), addr_of_mut!(forward));
    }

    // this is a lot of work for two triangles...
    // we could precalculate a lot of it is an issue, but it would mess up
    // the shader abstraction
    let mut i = 0;
    let mut indexes = 0;
    while i < tess.numVertexes {
        let mut lengths: [f32; 2] = [0.0; 2];
        let mut nums: [usize; 2] = [0; 2];
        let mut mid: [vec3_t; 2] = [[0.0; 3]; 2];
        let mut major: vec3_t = [0.0; 3];
        let mut minor: vec3_t = [0.0; 3];

        // find the midpoint
        let xyz = addr_of!((*tess.xyz)[i]);

        // identify the two shortest edges
        nums[0] = 0;
        nums[1] = 0;
        lengths[0] = 999999.0;
        lengths[1] = 999999.0;

        for j in 0..6 {
            let mut temp: vec3_t = [0.0; 3];

            let v1 = addr_of!((*tess.xyz)[(i + 4 * edgeVerts[j][0] as i32) as usize]);
            let v2 = addr_of!((*tess.xyz)[(i + 4 * edgeVerts[j][1] as i32) as usize]);

            VectorSubtract(v1, v2, addr_of_mut!(temp));

            let l = DotProduct(addr_of!(temp), addr_of!(temp));
            if l < lengths[0] {
                nums[1] = nums[0];
                lengths[1] = lengths[0];
                nums[0] = j;
                lengths[0] = l;
            } else if l < lengths[1] {
                nums[1] = j;
                lengths[1] = l;
            }
        }

        for j in 0..2 {
            let v1 = addr_of!((*tess.xyz)[(i + 4 * edgeVerts[nums[j]][0] as i32) as usize]);
            let v2 = addr_of!((*tess.xyz)[(i + 4 * edgeVerts[nums[j]][1] as i32) as usize]);

            mid[j][0] = 0.5 * ((*v1)[0] + (*v2)[0]);
            mid[j][1] = 0.5 * ((*v1)[1] + (*v2)[1]);
            mid[j][2] = 0.5 * ((*v1)[2] + (*v2)[2]);
        }

        // find the vector of the major axis
        VectorSubtract(addr_of!(mid[1]), addr_of!(mid[0]), addr_of_mut!(major));

        // cross this with the view direction to get minor axis
        CrossProduct(addr_of!(major), addr_of!(forward), addr_of_mut!(minor));
        VectorNormalize(addr_of_mut!(minor));

        // re-project the points
        for j in 0..2 {
            let l = 0.5 * lengths[j].sqrt();

            let v1 = addr_of_mut!((*tess.xyz)[(i + 4 * edgeVerts[nums[j]][0] as i32) as usize]);
            let v2 = addr_of_mut!((*tess.xyz)[(i + 4 * edgeVerts[nums[j]][1] as i32) as usize]);

            // we need to see which direction this edge
            // is used to determine direction of projection
            let mut k = 0;
            while k < 5 {
                if (*tess.indexes)[(indexes + k) as usize] == (i + 4 * edgeVerts[nums[j]][0] as i32) as u32
                    && (*tess.indexes)[(indexes + k + 1) as usize] == (i + 4 * edgeVerts[nums[j]][1] as i32) as u32 {
                    break;
                }
                k += 1;
            }

            if k == 5 {
                VectorMA(addr_of!(mid[j]), l, addr_of!(minor), v1);
                VectorMA(addr_of!(mid[j]), -l, addr_of!(minor), v2);
            } else {
                VectorMA(addr_of!(mid[j]), -l, addr_of!(minor), v1);
                VectorMA(addr_of!(mid[j]), l, addr_of!(minor), v2);
            }
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
pub unsafe fn RB_DeformTessGeometry() {
    for i in 0..(*(*tess.shader)).numDeforms {
        let ds = (*(*tess.shader)).deforms[i as usize];

        match (*ds).deformation {
            DEFORM_NONE => {},
            DEFORM_NORMALS => {
                RB_CalcDeformNormals(ds);
            },
            DEFORM_WAVE => {
                RB_CalcDeformVertexes(ds);
            },
            DEFORM_BULGE => {
                RB_CalcBulgeVertexes(ds);
            },
            DEFORM_MOVE => {
                RB_CalcMoveVertexes(ds);
            },
            DEFORM_PROJECTION_SHADOW => {
                RB_ProjectionShadowDeform();
            },
            DEFORM_AUTOSPRITE => {
                AutospriteDeform();
            },
            DEFORM_AUTOSPRITE2 => {
                Autosprite2Deform();
            },
            DEFORM_TEXT0..=DEFORM_TEXT7 => {
                DeformText((*backEnd.refdef).text[((*ds).deformation - DEFORM_TEXT0) as usize].as_ptr());
            },
            _ => {},
        }
    }
}

/*
====================================================================

COLORS

====================================================================
*/


/*
** RB_CalcColorFromEntity
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcColorFromEntity(dstColors: *mut u32) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    for i in 0..tess.numVertexes {
        let pColors = dstColors.add(i as usize);
        *pColors = D3DCOLOR_RGBA(
            (*backEnd.currentEntity).e.shaderRGBA[0] as u32,
            (*backEnd.currentEntity).e.shaderRGBA[1] as u32,
            (*backEnd.currentEntity).e.shaderRGBA[2] as u32,
            (*backEnd.currentEntity).e.shaderRGBA[3] as u32,
        );
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcColorFromEntity(dstColors: *mut u8) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    let pColors = dstColors as *mut c_int;
    let c = *((*backEnd.currentEntity).e.shaderRGBA.as_ptr() as *const c_int);

    for i in 0..tess.numVertexes {
        *pColors.add(i as usize) = c;
    }
}

/*
** RB_CalcColorFromOneMinusEntity
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcColorFromOneMinusEntity(dstColors: *mut u32) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    let mut invModulate: [u8; 4] = [0; 4];
    invModulate[0] = 255 - (*backEnd.currentEntity).e.shaderRGBA[0];
    invModulate[1] = 255 - (*backEnd.currentEntity).e.shaderRGBA[1];
    invModulate[2] = 255 - (*backEnd.currentEntity).e.shaderRGBA[2];
    invModulate[3] = 255 - (*backEnd.currentEntity).e.shaderRGBA[3];	// this trashes alpha, but the AGEN block fixes it

    for i in 0..tess.numVertexes {
        let pColors = dstColors.add(i as usize);
        *pColors = D3DCOLOR_RGBA(
            invModulate[0] as u32,
            invModulate[1] as u32,
            invModulate[2] as u32,
            invModulate[3] as u32,
        );
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcColorFromOneMinusEntity(dstColors: *mut u8) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    let mut invModulate: [u8; 4] = [0; 4];
    invModulate[0] = 255 - (*backEnd.currentEntity).e.shaderRGBA[0];
    invModulate[1] = 255 - (*backEnd.currentEntity).e.shaderRGBA[1];
    invModulate[2] = 255 - (*backEnd.currentEntity).e.shaderRGBA[2];
    invModulate[3] = 255 - (*backEnd.currentEntity).e.shaderRGBA[3];	// this trashes alpha, but the AGEN block fixes it

    let pColors = dstColors as *mut c_int;
    for i in 0..tess.numVertexes {
        *pColors.add(i as usize) = *(invModulate.as_ptr() as *const c_int);
    }
}

/*
** RB_CalcAlphaFromEntity
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcAlphaFromEntity(dstColors: *mut u32) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    for i in 0..tess.numVertexes {
        let pColors = dstColors.add(i as usize);
        let rgb = *pColors & 0x00ffffff;
        *pColors = rgb | ((((*backEnd.currentEntity).e.shaderRGBA[3] & 0xff) as u32) << 24);
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcAlphaFromEntity(dstColors: *mut u8) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    let mut dst = dstColors.add(3);

    for i in 0..tess.numVertexes {
        *dst = (*backEnd.currentEntity).e.shaderRGBA[3];
        dst = dst.add(4);
    }
}

/*
** RB_CalcAlphaFromOneMinusEntity
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcAlphaFromOneMinusEntity(dstColors: *mut u32) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    for i in 0..tess.numVertexes {
        let pColors = dstColors.add(i as usize);
        let rgb = *pColors & 0x00ffffff;
        *pColors = rgb | (((255 - (*backEnd.currentEntity).e.shaderRGBA[3]) & 0xff) as u32) << 24;
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcAlphaFromOneMinusEntity(dstColors: *mut u8) {
    if backEnd.currentEntity.is_null() {
        return;
    }

    let mut dst = dstColors.add(3);

    for i in 0..tess.numVertexes {
        *dst = 0xff - (*backEnd.currentEntity).e.shaderRGBA[3];
        dst = dst.add(4);
    }
}

/*
** RB_CalcWaveColor
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcWaveColor(wf: *const waveForm_t, dstColors: *mut u32) {
    let mut color: [u8; 4] = [0; 4];

    let glow = if (*wf).func == GF_NOISE {
        (*wf).base + R_NoiseGet4f(0.0, 0.0, 0.0, (backEnd.refdef.floatTime + (*wf).phase) * (*wf).frequency) * (*wf).amplitude
    } else {
        EvalWaveForm(wf) * tr.identityLight
    };

    let glow = if glow < 0.0 {
        0.0
    } else if glow > 1.0 {
        1.0
    } else {
        glow
    };

    let v = myftol(255.0 * glow);
    color[0] = v as u8;
    color[1] = v as u8;
    color[2] = v as u8;
    color[3] = 255;

    for i in 0..tess.numVertexes {
        let colors = dstColors.add(i as usize);
        *colors = D3DCOLOR_RGBA(color[0] as u32, color[1] as u32, color[2] as u32, color[3] as u32);
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcWaveColor(wf: *const waveForm_t, dstColors: *mut u8) {
    let mut color: [u8; 4] = [0; 4];

    let glow = if (*wf).func == GF_NOISE {
        (*wf).base + R_NoiseGet4f(0.0, 0.0, 0.0, (tess.shaderTime + (*wf).phase) * (*wf).frequency) * (*wf).amplitude
    } else {
        EvalWaveForm(wf) * tr.identityLight
    };

    let glow = if glow < 0.0 {
        0.0
    } else if glow > 1.0 {
        1.0
    } else {
        glow
    };

    let v = myftol(255.0 * glow);
    color[0] = v as u8;
    color[1] = v as u8;
    color[2] = v as u8;
    color[3] = 255;
    let v = *(color.as_ptr() as *const c_int);

    let colors = dstColors as *mut c_int;
    for i in 0..tess.numVertexes {
        *colors.add(i as usize) = v;
    }
}

/*
** RB_CalcWaveAlpha
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcWaveAlpha(wf: *const waveForm_t, dstColors: *mut u32) {
    let glow = EvalWaveFormClamped(wf);

    let v = (255.0 * glow) as u32;

    for i in 0..tess.numVertexes {
        let pColors = dstColors.add(i as usize);
        let rgb = *pColors & 0x00ffffff;
        *pColors = rgb | ((v & 0xff) << 24);
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcWaveAlpha(wf: *const waveForm_t, dstColors: *mut u8) {
    let glow = EvalWaveFormClamped(wf);

    let v = (255.0 * glow) as u8;

    for i in 0..tess.numVertexes {
        let dst = dstColors.add((i as usize) * 4 + 3);
        *dst = v;
    }
}

/*
** RB_CalcModulateColorsByFog
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcModulateColorsByFog(_colors: *mut u32) {
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcModulateColorsByFog(colors: *mut u8) {
    let mut texCoords: [[f32; 2]; SHADER_MAX_VERTEXES] = [[0.0; 2]; SHADER_MAX_VERTEXES];

    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    RB_CalcFogTexCoords(texCoords[0].as_mut_ptr());

    for i in 0..tess.numVertexes {
        let f = 1.0 - R_FogFactor(texCoords[i as usize][0], texCoords[i as usize][1]);
        let color_ptr = colors.add((i as usize) * 4);
        *color_ptr = ((*color_ptr as f32) * f) as u8;
        *color_ptr.add(1) = ((*color_ptr.add(1) as f32) * f) as u8;
        *color_ptr.add(2) = ((*color_ptr.add(2) as f32) * f) as u8;
    }
}

/*
** RB_CalcModulateAlphasByFog
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcModulateAlphasByFog(_colors: *mut u32) {
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcModulateAlphasByFog(colors: *mut u8) {
    let mut texCoords: [[f32; 2]; SHADER_MAX_VERTEXES] = [[0.0; 2]; SHADER_MAX_VERTEXES];

    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    RB_CalcFogTexCoords(texCoords[0].as_mut_ptr());

    for i in 0..tess.numVertexes {
        let f = 1.0 - R_FogFactor(texCoords[i as usize][0], texCoords[i as usize][1]);
        let color_ptr = colors.add((i as usize) * 4 + 3);
        *color_ptr = ((*color_ptr as f32) * f) as u8;
    }
}

/*
** RB_CalcModulateRGBAsByFog
*/
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcModulateRGBAsByFog(_colors: *mut u32) {
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcModulateRGBAsByFog(colors: *mut u8) {
    let mut texCoords: [[f32; 2]; SHADER_MAX_VERTEXES] = [[0.0; 2]; SHADER_MAX_VERTEXES];

    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    RB_CalcFogTexCoords(texCoords[0].as_mut_ptr());

    for i in 0..tess.numVertexes {
        let f = 1.0 - R_FogFactor(texCoords[i as usize][0], texCoords[i as usize][1]);
        let color_ptr = colors.add((i as usize) * 4);
        *color_ptr = ((*color_ptr as f32) * f) as u8;
        *color_ptr.add(1) = ((*color_ptr.add(1) as f32) * f) as u8;
        *color_ptr.add(2) = ((*color_ptr.add(2) as f32) * f) as u8;
        *color_ptr.add(3) = ((*color_ptr.add(3) as f32) * f) as u8;
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
pub unsafe fn RB_CalcFogTexCoords(st: *mut f32) {
    let mut localVec: vec3_t = [0.0; 3];
    let mut fogDistanceVector: vec4_t = [0.0; 4];
    let mut fogDepthVector: vec4_t = [0.0; 4];

    let fog = (*tr.world).fogs.add(tess.fogNum as usize);

    // all fogging distance is based on world Z units
    VectorSubtract(addr_of!(backEnd.ori.origin), addr_of!(backEnd.viewParms.ori.origin), addr_of_mut!(localVec));
    #[cfg(target_os = "xbox")]
    {
        fogDistanceVector[0] = backEnd.ori.modelMatrix[2];
        fogDistanceVector[1] = backEnd.ori.modelMatrix[6];
        fogDistanceVector[2] = backEnd.ori.modelMatrix[10];
    }
    #[cfg(not(target_os = "xbox"))]
    {
        fogDistanceVector[0] = -backEnd.ori.modelMatrix[2];
        fogDistanceVector[1] = -backEnd.ori.modelMatrix[6];
        fogDistanceVector[2] = -backEnd.ori.modelMatrix[10];
    }
    fogDistanceVector[3] = DotProduct(addr_of!(localVec), addr_of!(backEnd.viewParms.ori.axis[0]));

    // scale the fog vectors based on the fog's thickness
    fogDistanceVector[0] *= (*fog).tcScale;
    fogDistanceVector[1] *= (*fog).tcScale;
    fogDistanceVector[2] *= (*fog).tcScale;
    fogDistanceVector[3] *= (*fog).tcScale;

    // rotate the gradient vector for this orientation
    let eyeT;
    if (*fog).hasSurface != 0 {
        fogDepthVector[0] = (*fog).surface[0] * backEnd.ori.axis[0][0] +
            (*fog).surface[1] * backEnd.ori.axis[0][1] + (*fog).surface[2] * backEnd.ori.axis[0][2];
        fogDepthVector[1] = (*fog).surface[0] * backEnd.ori.axis[1][0] +
            (*fog).surface[1] * backEnd.ori.axis[1][1] + (*fog).surface[2] * backEnd.ori.axis[1][2];
        fogDepthVector[2] = (*fog).surface[0] * backEnd.ori.axis[2][0] +
            (*fog).surface[1] * backEnd.ori.axis[2][1] + (*fog).surface[2] * backEnd.ori.axis[2][2];
        fogDepthVector[3] = -(*fog).surface[3] + DotProduct(addr_of!(backEnd.ori.origin), addr_of!((*fog).surface));

        eyeT = DotProduct(addr_of!(backEnd.ori.viewOrigin), addr_of!(fogDepthVector)) + fogDepthVector[3];
    } else {
        eyeT = 1.0;	// non-surface fog always has eye inside

        fogDepthVector[0] = 0.0;
        fogDepthVector[1] = 0.0;
        fogDepthVector[2] = 0.0;
        fogDepthVector[3] = 1.0;
    }

    // see if the viewpoint is outside
    // this is needed for clipping distance even for constant fog

    let eyeOutside = if eyeT < 0.0 { qtrue } else { qfalse };

    fogDistanceVector[3] += 1.0 / 512.0;

    // calculate density for each point
    for i in 0..tess.numVertexes {
        let v = addr_of!((*tess.xyz)[i as usize]);

        // calculate the length in fog
        let s = DotProduct(v, addr_of!(fogDistanceVector)) + fogDistanceVector[3];
        let mut t = DotProduct(v, addr_of!(fogDepthVector)) + fogDepthVector[3];

        // partially clipped fogs use the T axis
        if eyeOutside != 0 {
            if t < 1.0 {
                t = 1.0 / 32.0;	// point is outside, so no fogging
            } else {
                t = 1.0 / 32.0 + 30.0 / 32.0 * t / (t - eyeT);	// cut the distance at the fog plane
            }
        } else {
            if t < 0.0 {
                t = 1.0 / 32.0;	// point is outside, so no fogging
            } else {
                t = 31.0 / 32.0;
            }
        }

        *st = s;
        *st.add(1) = t;
        st = st.add(2);
    }
}



/*
** RB_CalcEnvironmentTexCoords
*/
pub unsafe fn RB_CalcEnvironmentTexCoords(st: *mut f32) {
    let mut viewer: vec3_t = [0.0; 3];
    let mut reflected: vec3_t = [0.0; 3];

    let v = addr_of!((*tess.xyz)[0]);
    let normal = addr_of!((*tess.normal)[0]);

    for i in 0..tess.numVertexes {
        let v_ptr = addr_of!((*tess.xyz)[i as usize]);
        let normal_ptr = addr_of!((*tess.normal)[i as usize]);
        let st_ptr = st.add((i as usize) * 2);

        VectorSubtract(addr_of!(backEnd.ori.viewOrigin), v_ptr, addr_of_mut!(viewer));
        VectorNormalizeFast(addr_of_mut!(viewer));

        let d = DotProduct(normal_ptr, addr_of!(viewer));

        reflected[0] = normal_ptr[0] * 2.0 * d - viewer[0];
        reflected[1] = normal_ptr[1] * 2.0 * d - viewer[1];
        reflected[2] = normal_ptr[2] * 2.0 * d - viewer[2];

        *st_ptr = 0.5 + reflected[1] * 0.5;
        *st_ptr.add(1) = 0.5 - reflected[2] * 0.5;
    }
}

/*
** RB_CalcTurbulentTexCoords
*/
pub unsafe fn RB_CalcTurbulentTexCoords(wf: *const waveForm_t, st: *mut f32) {
    let now = (*wf).phase + tess.shaderTime * (*wf).frequency;

    for i in 0..tess.numVertexes {
        let st_ptr = st.add((i as usize) * 2);
        let s = *st_ptr;
        let t = *st_ptr.add(1);

        *st_ptr = s + tr.sinTable[((((*tess.xyz)[i as usize][0] + (*tess.xyz)[i as usize][2]) * 1.0 / 128.0 * 0.125 + now) * FUNCTABLE_SIZE as f32) as usize & FUNCTABLE_MASK] * (*wf).amplitude;
        *st_ptr.add(1) = t + tr.sinTable[(((*tess.xyz)[i as usize][1] * 1.0 / 128.0 * 0.125 + now) * FUNCTABLE_SIZE as f32) as usize & FUNCTABLE_MASK] * (*wf).amplitude;
    }
}

/*
** RB_CalcScaleTexCoords
*/
pub unsafe fn RB_CalcScaleTexCoords(scale: *const [f32; 2], st: *mut f32) {
    for i in 0..tess.numVertexes {
        let st_ptr = st.add((i as usize) * 2);
        *st_ptr *= (*scale)[0];
        *st_ptr.add(1) *= (*scale)[1];
    }
}

/*
** RB_CalcScrollTexCoords
*/
pub unsafe fn RB_CalcScrollTexCoords(scrollSpeed: *const [f32; 2], st: *mut f32) {
    let timeScale = tess.shaderTime;
    let mut adjustedScrollS = (*scrollSpeed)[0] * timeScale;
    let mut adjustedScrollT = (*scrollSpeed)[1] * timeScale;

    // clamp so coordinates don't continuously get larger, causing problems
    // with hardware limits
    adjustedScrollS = adjustedScrollS - adjustedScrollS.floor();
    adjustedScrollT = adjustedScrollT - adjustedScrollT.floor();

    for i in 0..tess.numVertexes {
        let st_ptr = st.add((i as usize) * 2);
        *st_ptr += adjustedScrollS;
        *st_ptr.add(1) += adjustedScrollT;
    }
}

/*
** RB_CalcTransformTexCoords
*/
pub unsafe fn RB_CalcTransformTexCoords(tmi: *const texModInfo_t, st: *mut f32) {
    for i in 0..tess.numVertexes {
        let st_ptr = st.add((i as usize) * 2);
        let s = *st_ptr;
        let t = *st_ptr.add(1);

        *st_ptr = s * (*tmi).matrix[0][0] + t * (*tmi).matrix[1][0] + (*tmi).translate[0];
        *st_ptr.add(1) = s * (*tmi).matrix[0][1] + t * (*tmi).matrix[1][1] + (*tmi).translate[1];
    }
}

/*
** RB_CalcRotateTexCoords
*/
pub unsafe fn RB_CalcRotateTexCoords(degsPerSecond: f32, st: *mut f32) {
    let timeScale = tess.shaderTime;
    let degs = -degsPerSecond * timeScale;
    let index = (degs * (FUNCTABLE_SIZE as f32 / 360.0)) as usize;

    let sinValue = tr.sinTable[index & FUNCTABLE_MASK];
    let cosValue = tr.sinTable[(index + FUNCTABLE_SIZE / 4) & FUNCTABLE_MASK];

    let mut tmi: texModInfo_t = core::mem::zeroed();
    tmi.matrix[0][0] = cosValue;
    tmi.matrix[1][0] = -sinValue;
    tmi.translate[0] = 0.5 - 0.5 * cosValue + 0.5 * sinValue;

    tmi.matrix[0][1] = sinValue;
    tmi.matrix[1][1] = cosValue;
    tmi.translate[1] = 0.5 - 0.5 * sinValue - 0.5 * cosValue;

    RB_CalcTransformTexCoords(addr_of!(tmi), st);
}

/*
** RB_CalcSpecularAlpha
**
** Calculates specular coefficient and places it in the alpha channel
*/
static mut lightOrigin: vec3_t = [-960.0, 1980.0, 96.0];		// FIXME: track dynamically

#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcSpecularAlpha(alphas: *mut u32) {
    let mut viewer: vec3_t = [0.0; 3];
    let mut reflected: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];

    let v = addr_of!((*tess.xyz)[0]);
    let normal = addr_of!((*tess.normal)[0]);

    let numVertexes = tess.numVertexes;
    for i in 0..numVertexes {
        let v_ptr = addr_of!((*tess.xyz)[i as usize]);
        let normal_ptr = addr_of!((*tess.normal)[i as usize]);
        let alphas_ptr = alphas.add(i as usize);

        if !backEnd.currentEntity.is_null() &&
            (!(*backEnd.currentEntity).e.hModel.is_null() || !(*backEnd.currentEntity).e.ghoul2.is_null()) {	//this is a model so we can use world lights instead fake light
            VectorCopy(addr_of!((*backEnd.currentEntity).lightDir), addr_of_mut!(lightDir));
        } else {
            VectorSubtract(addr_of!(lightOrigin), v_ptr, addr_of_mut!(lightDir));
            VectorNormalizeFast(addr_of_mut!(lightDir));
        }
        // calculate the specular color
        let d = 2.0 * DotProduct(normal_ptr, addr_of!(lightDir));

        // we don't optimize for the d < 0 case since this tends to
        // cause visual artifacts such as faceted "snapping"
        reflected[0] = normal_ptr[0] * d - lightDir[0];
        reflected[1] = normal_ptr[1] * d - lightDir[1];
        reflected[2] = normal_ptr[2] * d - lightDir[2];

        VectorSubtract(addr_of!(backEnd.ori.viewOrigin), v_ptr, addr_of_mut!(viewer));
        let ilength = Q_rsqrt(DotProduct(addr_of!(viewer), addr_of!(viewer)));
        let mut l = DotProduct(addr_of!(reflected), addr_of!(viewer));
        l *= ilength;

        let a: u32 = if l < 0.0 {
            0
        } else {
            let l = l * l;
            let l = l * l;
            let a = (l * 255.0) as u32;
            if a > 255 { 255 } else { a }
        };
        let rgb = *alphas_ptr & 0x00ffffff;

        *alphas_ptr = rgb | (a & 0xff) << 24;
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcSpecularAlpha(alphas: *mut u8) {
    let mut viewer: vec3_t = [0.0; 3];
    let mut reflected: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];

    let v = addr_of!((*tess.xyz)[0]);
    let normal = addr_of!((*tess.normal)[0]);

    let mut alphas_ptr = alphas.add(3);

    let numVertexes = tess.numVertexes;
    for i in 0..numVertexes {
        let v_ptr = addr_of!((*tess.xyz)[i as usize]);
        let normal_ptr = addr_of!((*tess.normal)[i as usize]);

        if !backEnd.currentEntity.is_null() &&
            (!(*backEnd.currentEntity).e.hModel.is_null() || !(*backEnd.currentEntity).e.ghoul2.is_null()) {	//this is a model so we can use world lights instead fake light
            VectorCopy(addr_of!((*backEnd.currentEntity).lightDir), addr_of_mut!(lightDir));
        } else {
            VectorSubtract(addr_of!(lightOrigin), v_ptr, addr_of_mut!(lightDir));
            VectorNormalizeFast(addr_of_mut!(lightDir));
        }
        // calculate the specular color
        let d = 2.0 * DotProduct(normal_ptr, addr_of!(lightDir));

        // we don't optimize for the d < 0 case since this tends to
        // cause visual artifacts such as faceted "snapping"
        reflected[0] = normal_ptr[0] * d - lightDir[0];
        reflected[1] = normal_ptr[1] * d - lightDir[1];
        reflected[2] = normal_ptr[2] * d - lightDir[2];

        VectorSubtract(addr_of!(backEnd.ori.viewOrigin), v_ptr, addr_of_mut!(viewer));
        let ilength = Q_rsqrt(DotProduct(addr_of!(viewer), addr_of!(viewer)));
        let mut l = DotProduct(addr_of!(reflected), addr_of!(viewer));
        l *= ilength;

        let b: u8 = if l < 0.0 {
            0
        } else {
            let l = l * l;
            let l = l * l;
            let b = (l * 255.0) as c_int;
            if b > 255 { 255 } else { b } as u8
        };

        *alphas_ptr = b;
        alphas_ptr = alphas_ptr.add(4);
    }
}

/*
** RB_CalcDiffuseColor
**
** The basic vertex lighting calc
*/
pub unsafe fn RB_CalcDiffuseColor(colors: *mut u8) {
    let ent = backEnd.currentEntity;
    let ambientLightInt = (*ent).ambientLightInt;
    let mut ambientLight: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];
    let mut directedLight: vec3_t = [0.0; 3];
    VectorCopy(addr_of!((*ent).ambientLight), addr_of_mut!(ambientLight));
    VectorCopy(addr_of!((*ent).directedLight), addr_of_mut!(directedLight));
    VectorCopy(addr_of!((*ent).lightDir), addr_of_mut!(lightDir));

    let v = addr_of!((*tess.xyz)[0]);
    let normal = addr_of!((*tess.normal)[0]);

    let numVertexes = tess.numVertexes;
    for i in 0..numVertexes {
        let v_ptr = addr_of!((*tess.xyz)[i as usize]);
        let normal_ptr = addr_of!((*tess.normal)[i as usize]);

        let incoming = DotProduct(normal_ptr, addr_of!(lightDir));
        if incoming <= 0.0 {
            *(colors.add((i as usize) * 4) as *mut c_int) = ambientLightInt;
            continue;
        }
        let mut j = myftol(ambientLight[0] + incoming * directedLight[0]);
        if j > 255 {
            j = 255;
        }
        *colors.add((i as usize) * 4) = j as u8;

        j = myftol(ambientLight[1] + incoming * directedLight[1]);
        if j > 255 {
            j = 255;
        }
        *colors.add((i as usize) * 4 + 1) = j as u8;

        j = myftol(ambientLight[2] + incoming * directedLight[2]);
        if j > 255 {
            j = 255;
        }
        *colors.add((i as usize) * 4 + 2) = j as u8;

        *colors.add((i as usize) * 4 + 3) = 255;
    }
}

/*
** RB_CalcDiffuseColorEntity
**
** The basic vertex lighting calc * Entity Color
*/
pub unsafe fn RB_CalcDiffuseEntityColor(colors: *mut u8) {
    if backEnd.currentEntity.is_null() {
        //error, use the normal lighting
        RB_CalcDiffuseColor(colors);
        return;
    }

    let ent = backEnd.currentEntity;
    let mut ambientLight: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];
    let mut directedLight: vec3_t = [0.0; 3];
    VectorCopy(addr_of!((*ent).ambientLight), addr_of_mut!(ambientLight));
    VectorCopy(addr_of!((*ent).directedLight), addr_of_mut!(directedLight));
    VectorCopy(addr_of!((*ent).lightDir), addr_of_mut!(lightDir));

    let r = (*backEnd.currentEntity).e.shaderRGBA[0] as f32 / 255.0;
    let g = (*backEnd.currentEntity).e.shaderRGBA[1] as f32 / 255.0;
    let b = (*backEnd.currentEntity).e.shaderRGBA[2] as f32 / 255.0;

    let mut ambientLightInt: [u8; 4] = [0; 4];
    ambientLightInt[0] = myftol(r * (*ent).ambientLight[0]) as u8;
    ambientLightInt[1] = myftol(g * (*ent).ambientLight[1]) as u8;
    ambientLightInt[2] = myftol(b * (*ent).ambientLight[2]) as u8;
    ambientLightInt[3] = (*backEnd.currentEntity).e.shaderRGBA[3];

    let v = addr_of!((*tess.xyz)[0]);
    let normal = addr_of!((*tess.normal)[0]);

    let numVertexes = tess.numVertexes;

    for i in 0..numVertexes {
        let v_ptr = addr_of!((*tess.xyz)[i as usize]);
        let normal_ptr = addr_of!((*tess.normal)[i as usize]);

        let incoming = DotProduct(normal_ptr, addr_of!(lightDir));
        if incoming <= 0.0 {
            *(colors.add((i as usize) * 4) as *mut c_int) = *(ambientLightInt.as_ptr() as *const c_int);
            continue;
        }
        let mut j = ambientLight[0] + incoming * directedLight[0];
        if j > 255.0 {
            j = 255.0;
        }
        *colors.add((i as usize) * 4) = myftol(j * r) as u8;

        j = ambientLight[1] + incoming * directedLight[1];
        if j > 255.0 {
            j = 255.0;
        }
        *colors.add((i as usize) * 4 + 1) = myftol(j * g) as u8;

        j = ambientLight[2] + incoming * directedLight[2];
        if j > 255.0 {
            j = 255.0;
        }
        *colors.add((i as usize) * 4 + 2) = myftol(j * b) as u8;

        *colors.add((i as usize) * 4 + 3) = (*backEnd.currentEntity).e.shaderRGBA[3];
    }
}

//---------------------------------------------------------
#[cfg(target_os = "xbox")]
pub unsafe fn RB_CalcDisintegrateColors(colors: *mut u32) {
    let ent = addr_of_mut!((*backEnd.currentEntity).e);
    let v = addr_of!((*tess.xyz)[0]);

    // calculate the burn threshold at the given time, anything that passes the threshold will get burnt
    let threshold = (backEnd.refdef.time as f32 - (*ent).endTime as f32) * 0.045; // endTime is really the start time, maybe I should just use a completely meaningless substitute?

    let numVertexes = tess.numVertexes;

    if (*ent).renderfx & RF_DISINTEGRATE1 != 0 {
        // this handles the blacken and fading out of the regular player model
        for i in 0..numVertexes {
            let v_ptr = addr_of!((*tess.xyz)[i as usize]);
            let mut temp: vec3_t = [0.0; 3];

            let rgb = colors[i as usize] & 0x00ffffff;

            VectorSubtract(addr_of!((*backEnd.currentEntity).e.oldorigin), v_ptr, addr_of_mut!(temp));

            let dis = VectorLengthSquared(addr_of!(temp));

            if dis < threshold * threshold {
                // completely disintegrated
                colors[i as usize] = rgb | (0x00 << 24);
            } else if dis < threshold * threshold + 60.0 {
                // blacken before fading out
                colors[i as usize] = D3DCOLOR_RGBA(0x00, 0x00, 0x00, 0xff);
            } else if dis < threshold * threshold + 150.0 {
                // darken more
                colors[i as usize] = D3DCOLOR_RGBA(0x6f, 0x6f, 0x6f, 0xff);
            } else if dis < threshold * threshold + 180.0 {
                // darken at edge of burn
                colors[i as usize] = D3DCOLOR_RGBA(0xaf, 0xaf, 0xaf, 0xff);
            } else {
                // not burning at all yet
                colors[i as usize] = D3DCOLOR_RGBA(0xff, 0xff, 0xff, 0xff);
            }
        }
    } else if (*ent).renderfx & RF_DISINTEGRATE2 != 0 {
        // this handles the glowing, burning bit that scales away from the model
        for i in 0..numVertexes {
            let v_ptr = addr_of!((*tess.xyz)[i as usize]);
            let mut temp: vec3_t = [0.0; 3];

            VectorSubtract(addr_of!((*backEnd.currentEntity).e.oldorigin), v_ptr, addr_of_mut!(temp));

            let dis = VectorLengthSquared(addr_of!(temp));

            if dis < threshold * threshold {
                // done burning
                colors[i as usize] = D3DCOLOR_RGBA(0x00, 0x00, 0x00, 0x00);
            } else {
                // still full burn
                colors[i as usize] = D3DCOLOR_RGBA(0xff, 0xff, 0xff, 0xff);
            }
        }
    }
}

#[cfg(not(target_os = "xbox"))]
pub unsafe fn RB_CalcDisintegrateColors(colors: *mut u8) {
    let ent = addr_of_mut!((*backEnd.currentEntity).e);
    let v = addr_of!((*tess.xyz)[0]);

    // calculate the burn threshold at the given time, anything that passes the threshold will get burnt
    let threshold = (backEnd.refdef.time as f32 - (*ent).endTime as f32) * 0.045; // endTime is really the start time, maybe I should just use a completely meaningless substitute?

    let numVertexes = tess.numVertexes;

    if (*ent).renderfx & RF_DISINTEGRATE1 != 0 {
        // this handles the blacken and fading out of the regular player model
        for i in 0..numVertexes {
            let v_ptr = addr_of!((*tess.xyz)[i as usize]);
            let mut temp: vec3_t = [0.0; 3];

            VectorSubtract(addr_of!((*backEnd.currentEntity).e.oldorigin), v_ptr, addr_of_mut!(temp));

            let dis = VectorLengthSquared(addr_of!(temp));

            if dis < threshold * threshold {
                // completely disintegrated
                *colors.add((i as usize) * 4 + 3) = 0x00;
            } else if dis < threshold * threshold + 60.0 {
                // blacken before fading out
                *colors.add((i as usize) * 4 + 0) = 0x0;
                *colors.add((i as usize) * 4 + 1) = 0x0;
                *colors.add((i as usize) * 4 + 2) = 0x0;
                *colors.add((i as usize) * 4 + 3) = 0xff;
            } else if dis < threshold * threshold + 150.0 {
                // darken more
                *colors.add((i as usize) * 4 + 0) = 0x6f;
                *colors.add((i as usize) * 4 + 1) = 0x6f;
                *colors.add((i as usize) * 4 + 2) = 0x6f;
                *colors.add((i as usize) * 4 + 3) = 0xff;
            } else if dis < threshold * threshold + 180.0 {
                // darken at edge of burn
                *colors.add((i as usize) * 4 + 0) = 0xaf;
                *colors.add((i as usize) * 4 + 1) = 0xaf;
                *colors.add((i as usize) * 4 + 2) = 0xaf;
                *colors.add((i as usize) * 4 + 3) = 0xff;
            } else {
                // not burning at all yet
                *colors.add((i as usize) * 4 + 0) = 0xff;
                *colors.add((i as usize) * 4 + 1) = 0xff;
                *colors.add((i as usize) * 4 + 2) = 0xff;
                *colors.add((i as usize) * 4 + 3) = 0xff;
            }
        }
    } else if (*ent).renderfx & RF_DISINTEGRATE2 != 0 {
        // this handles the glowing, burning bit that scales away from the model
        for i in 0..numVertexes {
            let v_ptr = addr_of!((*tess.xyz)[i as usize]);
            let mut temp: vec3_t = [0.0; 3];

            VectorSubtract(addr_of!((*backEnd.currentEntity).e.oldorigin), v_ptr, addr_of_mut!(temp));

            let dis = VectorLengthSquared(addr_of!(temp));

            if dis < threshold * threshold {
                // done burning
                *colors.add((i as usize) * 4 + 0) = 0x00;
                *colors.add((i as usize) * 4 + 1) = 0x00;
                *colors.add((i as usize) * 4 + 2) = 0x00;
                *colors.add((i as usize) * 4 + 3) = 0x00;
            } else {
                // still full burn
                *colors.add((i as usize) * 4 + 0) = 0xff;
                *colors.add((i as usize) * 4 + 1) = 0xff;
                *colors.add((i as usize) * 4 + 2) = 0xff;
                *colors.add((i as usize) * 4 + 3) = 0xff;
            }
        }
    }
}

//---------------------------------------------------------
pub unsafe fn RB_CalcDisintegrateVertDeform() {
    let xyz = tess.xyz as *mut f32;
    let normal = tess.normal as *mut f32;

    if (*backEnd.currentEntity).e.renderfx & RF_DISINTEGRATE2 != 0 {
        let threshold = (backEnd.refdef.time as f32 - (*backEnd.currentEntity).e.endTime as f32) * 0.045;

        for i in 0..tess.numVertexes {
            let xyz_ptr = xyz.add((i as usize) * 4);
            let normal_ptr = normal.add((i as usize) * 4);

            let mut temp: vec3_t = [0.0; 3];
            VectorSubtract(addr_of!((*backEnd.currentEntity).e.oldorigin), xyz_ptr as *const vec3_t, addr_of_mut!(temp));

            let scale = VectorLengthSquared(addr_of!(temp));

            if scale < threshold * threshold {
                *xyz_ptr = *xyz_ptr + *normal_ptr * 2.0;
                *xyz_ptr.add(1) = *xyz_ptr.add(1) + *normal_ptr.add(1) * 2.0;
                *xyz_ptr.add(2) = *xyz_ptr.add(2) + *normal_ptr.add(2) * 0.5;
            } else if scale < threshold * threshold + 50.0 {
                *xyz_ptr = *xyz_ptr + *normal_ptr * 1.0;
                *xyz_ptr.add(1) = *xyz_ptr.add(1) + *normal_ptr.add(1) * 1.0;
                //				xyz[2] += normal[2] * 1;
            }
        }
    }
}

// Helper stub for D3DCOLOR_RGBA (Xbox specific)
#[cfg(target_os = "xbox")]
#[inline]
fn D3DCOLOR_RGBA(r: u32, g: u32, b: u32, a: u32) -> u32 {
    ((a & 0xff) << 24) | ((r & 0xff) << 16) | ((g & 0xff) << 8) | (b & 0xff)
}
