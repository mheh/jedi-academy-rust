// q_math.c -- stateless support routines that are included in each code module

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::mem;

// Type aliases matching C definitions
pub type vec3_t = [f32; 3];
pub type vec4_t = [f32; 4];
pub type vec_t = f32;
pub type qboolean = c_int; // Typically qboolean is int in C

// Constants (from q_shared.h conceptually)
pub const NUMVERTEXNORMALS: usize = 162;
pub const PLANE_X: c_int = 0;
pub const PLANE_Y: c_int = 1;
pub const PLANE_Z: c_int = 2;
pub const PLANE_NON_AXIAL: c_int = 3;
pub const PITCH: usize = 0;
pub const YAW: usize = 1;
pub const ROLL: usize = 2;
pub const WORLD_SIZE: f32 = 65536.0;
pub const CT_MAX: usize = 48;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// Color table constants
pub const CT_NONE: usize = 0;
pub const CT_BLACK: usize = 1;
pub const CT_RED: usize = 2;
pub const CT_GREEN: usize = 3;
pub const CT_BLUE: usize = 4;
pub const CT_YELLOW: usize = 5;
pub const CT_MAGENTA: usize = 6;
pub const CT_CYAN: usize = 7;
pub const CT_WHITE: usize = 8;
pub const CT_LTGREY: usize = 9;
pub const CT_MDGREY: usize = 10;
pub const CT_DKGREY: usize = 11;
pub const CT_DKGREY2: usize = 12;
pub const CT_VLTORANGE: usize = 13;
pub const CT_LTORANGE: usize = 14;
pub const CT_DKORANGE: usize = 15;
pub const CT_VDKORANGE: usize = 16;
pub const CT_VLTBLUE1: usize = 17;
pub const CT_LTBLUE1: usize = 18;
pub const CT_DKBLUE1: usize = 19;
pub const CT_VDKBLUE1: usize = 20;
pub const CT_VLTBLUE2: usize = 21;
pub const CT_LTBLUE2: usize = 22;
pub const CT_DKBLUE2: usize = 23;
pub const CT_VDKBLUE2: usize = 24;
pub const CT_VLTBROWN1: usize = 25;
pub const CT_LTBROWN1: usize = 26;
pub const CT_DKBROWN1: usize = 27;
pub const CT_VDKBROWN1: usize = 28;
pub const CT_VLTGOLD1: usize = 29;
pub const CT_LTGOLD1: usize = 30;
pub const CT_DKGOLD1: usize = 31;
pub const CT_VDKGOLD1: usize = 32;
pub const CT_VLTPURPLE1: usize = 33;
pub const CT_LTPURPLE1: usize = 34;
pub const CT_DKPURPLE1: usize = 35;
pub const CT_VDKPURPLE1: usize = 36;
pub const CT_VLTPURPLE2: usize = 37;
pub const CT_LTPURPLE2: usize = 38;
pub const CT_DKPURPLE2: usize = 39;
pub const CT_VDKPURPLE2: usize = 40;
pub const CT_VLTPURPLE3: usize = 41;
pub const CT_LTPURPLE3: usize = 42;
pub const CT_DKPURPLE3: usize = 43;
pub const CT_VDKPURPLE3: usize = 44;
pub const CT_VLTRED1: usize = 45;
pub const CT_LTRED1: usize = 46;
pub const CT_DKRED1: usize = 47;
pub const CT_VDKRED1: usize = 48;
pub const CT_VDKRED: usize = 49;
pub const CT_DKRED: usize = 50;
pub const CT_VLTAQUA: usize = 51;
pub const CT_LTAQUA: usize = 52;
pub const CT_DKAQUA: usize = 53;
pub const CT_VDKAQUA: usize = 54;
pub const CT_LTPINK: usize = 55;
pub const CT_DKPINK: usize = 56;
pub const CT_LTCYAN: usize = 57;
pub const CT_DKCYAN: usize = 58;
pub const CT_LTBLUE3: usize = 59;
pub const CT_LTBLUE3_2: usize = 60;
pub const CT_DKBLUE3: usize = 61;
pub const CT_HUD_GREEN: usize = 62;
pub const CT_HUD_RED: usize = 63;
pub const CT_ICON_BLUE: usize = 64;
pub const CT_NO_AMMO_RED: usize = 65;
pub const CT_HUD_ORANGE: usize = 66;
pub const CT_TITLE: usize = 67;

pub const M_PI: f32 = 3.14159265358979323846;

// Placeholder for cplane_t struct (needs to be defined elsewhere)
// This is a stub to allow the function signatures to compile
#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub _type: c_int, // type is a reserved keyword
    pub signbits: c_int,
    pub pad: [u8; 4],
}

pub const vec3_origin: vec3_t = [0.0, 0.0, 0.0];

pub const axisDefault: [vec3_t; 3] = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

pub static mut colorTable: [vec4_t; CT_MAX] = [
    [0.0, 0.0, 0.0, 0.0],                                  // CT_NONE
    [0.0, 0.0, 0.0, 1.0],                                  // CT_BLACK
    [1.0, 0.0, 0.0, 1.0],                                  // CT_RED
    [0.0, 1.0, 0.0, 1.0],                                  // CT_GREEN
    [0.0, 0.0, 1.0, 1.0],                                  // CT_BLUE
    [1.0, 1.0, 0.0, 1.0],                                  // CT_YELLOW
    [1.0, 0.0, 1.0, 1.0],                                  // CT_MAGENTA
    [0.0, 1.0, 1.0, 1.0],                                  // CT_CYAN
    [1.0, 1.0, 1.0, 1.0],                                  // CT_WHITE
    [0.75, 0.75, 0.75, 1.0],                               // CT_LTGREY
    [0.50, 0.50, 0.50, 1.0],                               // CT_MDGREY
    [0.25, 0.25, 0.25, 1.0],                               // CT_DKGREY
    [0.15, 0.15, 0.15, 1.0],                               // CT_DKGREY2
    [0.992, 0.652, 0.0, 1.0],                              // CT_VLTORANGE -- needs values
    [0.810, 0.530, 0.0, 1.0],                              // CT_LTORANGE
    [0.610, 0.330, 0.0, 1.0],                              // CT_DKORANGE
    [0.402, 0.265, 0.0, 1.0],                              // CT_VDKORANGE
    [0.503, 0.375, 0.996, 1.0],                            // CT_VLTBLUE1
    [0.367, 0.261, 0.722, 1.0],                            // CT_LTBLUE1
    [0.199, 0.0, 0.398, 1.0],                              // CT_DKBLUE1
    [0.160, 0.117, 0.324, 1.0],                            // CT_VDKBLUE1
    [0.300, 0.628, 0.816, 1.0],                            // CT_VLTBLUE2 -- needs values
    [0.300, 0.628, 0.816, 1.0],                            // CT_LTBLUE2
    [0.191, 0.289, 0.457, 1.0],                            // CT_DKBLUE2
    [0.125, 0.250, 0.324, 1.0],                            // CT_VDKBLUE2
    [0.796, 0.398, 0.199, 1.0],                            // CT_VLTBROWN1 -- needs values
    [0.796, 0.398, 0.199, 1.0],                            // CT_LTBROWN1
    [0.558, 0.207, 0.027, 1.0],                            // CT_DKBROWN1
    [0.328, 0.125, 0.035, 1.0],                            // CT_VDKBROWN1
    [0.996, 0.796, 0.398, 1.0],                            // CT_VLTGOLD1 -- needs values
    [0.996, 0.796, 0.398, 1.0],                            // CT_LTGOLD1
    [0.605, 0.441, 0.113, 1.0],                            // CT_DKGOLD1
    [0.386, 0.308, 0.148, 1.0],                            // CT_VDKGOLD1
    [0.648, 0.562, 0.784, 1.0],                            // CT_VLTPURPLE1 -- needs values
    [0.648, 0.562, 0.784, 1.0],                            // CT_LTPURPLE1
    [0.437, 0.335, 0.597, 1.0],                            // CT_DKPURPLE1
    [0.308, 0.269, 0.375, 1.0],                            // CT_VDKPURPLE1
    [0.816, 0.531, 0.710, 1.0],                            // CT_VLTPURPLE2 -- needs values
    [0.816, 0.531, 0.710, 1.0],                            // CT_LTPURPLE2
    [0.566, 0.269, 0.457, 1.0],                            // CT_DKPURPLE2
    [0.343, 0.226, 0.316, 1.0],                            // CT_VDKPURPLE2
    [0.929, 0.597, 0.929, 1.0],                            // CT_VLTPURPLE3
    [0.570, 0.371, 0.570, 1.0],                            // CT_LTPURPLE3
    [0.355, 0.199, 0.355, 1.0],                            // CT_DKPURPLE3
    [0.285, 0.136, 0.230, 1.0],                            // CT_VDKPURPLE3
    [0.953, 0.378, 0.250, 1.0],                            // CT_VLTRED1
    [0.953, 0.378, 0.250, 1.0],                            // CT_LTRED1
    [0.593, 0.121, 0.109, 1.0],                            // CT_DKRED1
    [0.429, 0.171, 0.113, 1.0],                            // CT_VDKRED1
];

pub static mut g_color_table: [vec4_t; 8] = [
    [0.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [1.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0],
    [0.0, 1.0, 1.0, 1.0],
    [1.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
];

// truncation from const double to float
pub static bytedirs: [vec3_t; NUMVERTEXNORMALS] = [
    [-0.525731, 0.000000, 0.850651],
    [-0.442863, 0.238856, 0.864188],
    [-0.295242, 0.000000, 0.955423],
    [-0.309017, 0.500000, 0.809017],
    [-0.162460, 0.262866, 0.951056],
    [0.000000, 0.000000, 1.000000],
    [0.000000, 0.850651, 0.525731],
    [-0.147621, 0.716567, 0.681718],
    [0.147621, 0.716567, 0.681718],
    [0.000000, 0.525731, 0.850651],
    [0.309017, 0.500000, 0.809017],
    [0.525731, 0.000000, 0.850651],
    [0.295242, 0.000000, 0.955423],
    [0.442863, 0.238856, 0.864188],
    [0.162460, 0.262866, 0.951056],
    [-0.681718, 0.147621, 0.716567],
    [-0.809017, 0.309017, 0.500000],
    [-0.587785, 0.425325, 0.688191],
    [-0.850651, 0.525731, 0.000000],
    [-0.864188, 0.442863, 0.238856],
    [-0.716567, 0.681718, 0.147621],
    [-0.688191, 0.587785, 0.425325],
    [-0.500000, 0.809017, 0.309017],
    [-0.238856, 0.864188, 0.442863],
    [-0.425325, 0.688191, 0.587785],
    [-0.716567, 0.681718, -0.147621],
    [-0.500000, 0.809017, -0.309017],
    [-0.525731, 0.850651, 0.000000],
    [0.000000, 0.850651, -0.525731],
    [-0.238856, 0.864188, -0.442863],
    [0.000000, 0.955423, -0.295242],
    [-0.262866, 0.951056, -0.162460],
    [0.000000, 1.000000, 0.000000],
    [0.000000, 0.955423, 0.295242],
    [-0.262866, 0.951056, 0.162460],
    [0.238856, 0.864188, 0.442863],
    [0.262866, 0.951056, 0.162460],
    [0.500000, 0.809017, 0.309017],
    [0.238856, 0.864188, -0.442863],
    [0.262866, 0.951056, -0.162460],
    [0.500000, 0.809017, -0.309017],
    [0.850651, 0.525731, 0.000000],
    [0.716567, 0.681718, 0.147621],
    [0.716567, 0.681718, -0.147621],
    [0.525731, 0.850651, 0.000000],
    [0.425325, 0.688191, 0.587785],
    [0.864188, 0.442863, 0.238856],
    [0.688191, 0.587785, 0.425325],
    [0.809017, 0.309017, 0.500000],
    [0.681718, 0.147621, 0.716567],
    [0.587785, 0.425325, 0.688191],
    [0.955423, 0.295242, 0.000000],
    [1.000000, 0.000000, 0.000000],
    [0.951056, 0.162460, 0.262866],
    [0.850651, -0.525731, 0.000000],
    [0.955423, -0.295242, 0.000000],
    [0.864188, -0.442863, 0.238856],
    [0.951056, -0.162460, 0.262866],
    [0.809017, -0.309017, 0.500000],
    [0.681718, -0.147621, 0.716567],
    [0.850651, 0.000000, 0.525731],
    [0.864188, 0.442863, -0.238856],
    [0.809017, 0.309017, -0.500000],
    [0.951056, 0.162460, -0.262866],
    [0.525731, 0.000000, -0.850651],
    [0.681718, 0.147621, -0.716567],
    [0.681718, -0.147621, -0.716567],
    [0.850651, 0.000000, -0.525731],
    [0.809017, -0.309017, -0.500000],
    [0.864188, -0.442863, -0.238856],
    [0.951056, -0.162460, -0.262866],
    [0.147621, 0.716567, -0.681718],
    [0.309017, 0.500000, -0.809017],
    [0.425325, 0.688191, -0.587785],
    [0.442863, 0.238856, -0.864188],
    [0.587785, 0.425325, -0.688191],
    [0.688191, 0.587785, -0.425325],
    [-0.147621, 0.716567, -0.681718],
    [-0.309017, 0.500000, -0.809017],
    [0.000000, 0.525731, -0.850651],
    [-0.525731, 0.000000, -0.850651],
    [-0.442863, 0.238856, -0.864188],
    [-0.295242, 0.000000, -0.955423],
    [-0.162460, 0.262866, -0.951056],
    [0.000000, 0.000000, -1.000000],
    [0.295242, 0.000000, -0.955423],
    [0.162460, 0.262866, -0.951056],
    [-0.442863, -0.238856, -0.864188],
    [-0.309017, -0.500000, -0.809017],
    [-0.162460, -0.262866, -0.951056],
    [0.000000, -0.850651, -0.525731],
    [-0.147621, -0.716567, -0.681718],
    [0.147621, -0.716567, -0.681718],
    [0.000000, -0.525731, -0.850651],
    [0.309017, -0.500000, -0.809017],
    [0.442863, -0.238856, -0.864188],
    [0.162460, -0.262866, -0.951056],
    [0.238856, -0.864188, -0.442863],
    [0.500000, -0.809017, -0.309017],
    [0.425325, -0.688191, -0.587785],
    [0.716567, -0.681718, -0.147621],
    [0.688191, -0.587785, -0.425325],
    [0.587785, -0.425325, -0.688191],
    [0.000000, -0.955423, -0.295242],
    [0.000000, -1.000000, 0.000000],
    [0.262866, -0.951056, -0.162460],
    [0.000000, -0.850651, 0.525731],
    [0.000000, -0.955423, 0.295242],
    [0.238856, -0.864188, 0.442863],
    [0.262866, -0.951056, 0.162460],
    [0.500000, -0.809017, 0.309017],
    [0.716567, -0.681718, 0.147621],
    [0.525731, -0.850651, 0.000000],
    [-0.238856, -0.864188, -0.442863],
    [-0.500000, -0.809017, -0.309017],
    [-0.262866, -0.951056, -0.162460],
    [-0.850651, -0.525731, 0.000000],
    [-0.716567, -0.681718, -0.147621],
    [-0.716567, -0.681718, 0.147621],
    [-0.525731, -0.850651, 0.000000],
    [-0.500000, -0.809017, 0.309017],
    [-0.238856, -0.864188, 0.442863],
    [-0.262866, -0.951056, 0.162460],
    [-0.864188, -0.442863, 0.238856],
    [-0.809017, -0.309017, 0.500000],
    [-0.688191, -0.587785, 0.425325],
    [-0.681718, -0.147621, 0.716567],
    [-0.442863, -0.238856, 0.864188],
    [-0.587785, -0.425325, 0.688191],
    [-0.309017, -0.500000, 0.809017],
    [-0.147621, -0.716567, 0.681718],
    [-0.425325, -0.688191, 0.587785],
    [-0.162460, -0.262866, 0.951056],
    [0.442863, -0.238856, 0.864188],
    [0.162460, -0.262866, 0.951056],
    [0.309017, -0.500000, 0.809017],
    [0.147621, -0.716567, 0.681718],
    [0.000000, -0.525731, 0.850651],
    [0.425325, -0.688191, 0.587785],
    [0.587785, -0.425325, 0.688191],
    [0.688191, -0.587785, 0.425325],
    [-0.955423, 0.295242, 0.000000],
    [-0.951056, 0.162460, 0.262866],
    [-1.000000, 0.000000, 0.000000],
    [-0.850651, 0.000000, 0.525731],
    [-0.955423, -0.295242, 0.000000],
    [-0.951056, -0.162460, 0.262866],
    [-0.864188, 0.442863, -0.238856],
    [-0.951056, 0.162460, -0.262866],
    [-0.809017, 0.309017, -0.500000],
    [-0.864188, -0.442863, -0.238856],
    [-0.951056, -0.162460, -0.262866],
    [-0.809017, -0.309017, -0.500000],
    [-0.681718, 0.147621, -0.716567],
    [-0.681718, -0.147621, -0.716567],
    [-0.850651, 0.000000, -0.525731],
    [-0.688191, 0.587785, -0.425325],
    [-0.587785, 0.425325, -0.688191],
    [-0.425325, 0.688191, -0.587785],
    [-0.425325, -0.688191, -0.587785],
    [-0.587785, -0.425325, -0.688191],
    [-0.688191, -0.587785, -0.425325],
];

// Extern functions that must be provided by the engine
extern "C" {
    pub fn random() -> f32;
    pub fn log(x: f32) -> f32;
    pub fn sqrt(x: f32) -> f32;
    pub fn atan2(y: f32, x: f32) -> f32;
    pub fn cos(x: f32) -> f32;
    pub fn sin(x: f32) -> f32;
}

// Stub declarations for vector/math macros that need to be defined elsewhere
// These are placeholders that assume they'll be provided by the engine
pub fn DotProduct(a: &vec3_t, b: &vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

pub fn VectorSubtract(a: &vec3_t, b: &vec3_t, out: &mut vec3_t) {
    out[0] = a[0] - b[0];
    out[1] = a[1] - b[1];
    out[2] = a[2] - b[2];
}

pub fn CrossProduct(a: &vec3_t, b: &vec3_t, out: &mut vec3_t) {
    out[0] = a[1] * b[2] - a[2] * b[1];
    out[1] = a[2] * b[0] - a[0] * b[2];
    out[2] = a[0] * b[1] - a[1] * b[0];
}

pub fn VectorNormalize(v: &mut vec3_t) -> f32 {
    let mut length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length != 0.0 {
        let ilength = 1.0 / length;
        v[0] *= ilength;
        v[1] *= ilength;
        v[2] *= ilength;
    }
    length
}

pub fn VectorNormalize2(src: &vec3_t, dst: &mut vec3_t) -> f32 {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
    VectorNormalize(dst)
}

pub fn VectorMA(veca: &vec3_t, scale: f32, vecb: &vec3_t, vecc: &mut vec3_t) {
    vecc[0] = veca[0] + scale * vecb[0];
    vecc[1] = veca[1] + scale * vecb[1];
    vecc[2] = veca[2] + scale * vecb[2];
}

pub fn VectorLength(v: &vec3_t) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub fn VectorLengthSquared(v: &vec3_t) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

pub fn VectorClear(v: &mut vec3_t) {
    v[0] = 0.0;
    v[1] = 0.0;
    v[2] = 0.0;
}

pub fn Distance(a: &vec3_t, b: &vec3_t) -> f32 {
    let mut v: vec3_t = [0.0; 3];
    VectorSubtract(b, a, &mut v);
    VectorLength(&v)
}

pub fn DEG2RAD(degrees: f32) -> f32 {
    degrees * (M_PI * 2.0 / 360.0)
}

// ==============================================================

/*
erandom

This function produces a random number with a exponential
distribution and the specified mean value.
*/
pub fn erandom(mean: f32) -> f32 {
    let mut r: f32;

    loop {
        r = unsafe { random() };
        if r != 0.0 {
            break;
        }
    }

    -mean * unsafe { log(r) }
}

pub fn ClampChar(i: c_int) -> i8 {
    if i < -128 {
        -128
    } else if i > 127 {
        127
    } else {
        i as i8
    }
}

pub fn ClampShort(i: c_int) -> i16 {
    if i < -32768i32 {
        -32768i16
    } else if i > 32767 {
        32767
    } else {
        i as i16
    }
}

// this isn't a real cheap function to call!
pub fn DirToByte(dir: &vec3_t) -> c_int {
    let mut i: c_int;
    let mut best: c_int;
    let mut d: f32;
    let mut bestd: f32;

    if dir[0] == 0.0 && dir[1] == 0.0 && dir[2] == 0.0 {
        return 0;
    }

    bestd = 0.0;
    best = 0;
    for i in 0..NUMVERTEXNORMALS {
        d = DotProduct(dir, &bytedirs[i]);
        if d > bestd {
            bestd = d;
            best = i as c_int;
        }
    }

    best
}

pub fn ByteToDir(b: c_int, dir: &mut vec3_t) {
    if b < 0 || b >= NUMVERTEXNORMALS as c_int {
        VectorCopy(&vec3_origin, dir);
        return;
    }
    VectorCopy(&bytedirs[b as usize], dir);
}

pub fn ColorBytes3(r: f32, g: f32, b: f32) -> c_int {
    let mut i: c_int = 0;
    // Simulate byte pointer casting and assignment
    let bytes = i.to_le_bytes();
    let mut bytes_mut = bytes;
    bytes_mut[0] = (r * 255.0) as u8;
    bytes_mut[1] = (g * 255.0) as u8;
    bytes_mut[2] = (b * 255.0) as u8;
    i = c_int::from_le_bytes(bytes_mut);
    i
}

pub fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> c_int {
    let mut i: c_int = 0;
    // Simulate byte pointer casting and assignment
    let bytes = i.to_le_bytes();
    let mut bytes_mut = bytes;
    bytes_mut[0] = (r * 255.0) as u8;
    bytes_mut[1] = (g * 255.0) as u8;
    bytes_mut[2] = (b * 255.0) as u8;
    bytes_mut[3] = (a * 255.0) as u8;
    i = c_int::from_le_bytes(bytes_mut);
    i
}

pub fn NormalizeColor(input: &vec3_t, out: &mut vec3_t) -> f32 {
    let mut max: f32;

    max = input[0];
    if input[1] > max {
        max = input[1];
    }
    if input[2] > max {
        max = input[2];
    }

    if max == 0.0 {
        VectorClear(out);
    } else {
        out[0] = input[0] / max;
        out[1] = input[1] / max;
        out[2] = input[2] / max;
    }
    max
}

pub fn VectorAdvance(veca: &vec3_t, scale: f32, vecb: &vec3_t, vecc: &mut vec3_t) {
    vecc[0] = veca[0] + (scale * (vecb[0] - veca[0]));
    vecc[1] = veca[1] + (scale * (vecb[1] - veca[1]));
    vecc[2] = veca[2] + (scale * (vecb[2] - veca[2]));
}

//============================================================================

/*
=====================
PlaneFromPoints

Returns false if the triangle is degenrate.
The normal will point out of the clock for clockwise ordered points
=====================
*/
pub fn PlaneFromPoints(plane: &mut vec4_t, a: &vec3_t, b: &vec3_t, c: &vec3_t) -> qboolean {
    let mut d1: vec3_t = [0.0; 3];
    let mut d2: vec3_t = [0.0; 3];

    VectorSubtract(b, a, &mut d1);
    VectorSubtract(c, a, &mut d2);
    CrossProduct(&d2, &d1, &mut [plane[0], plane[1], plane[2]]);

    let mut plane_normal: vec3_t = [plane[0], plane[1], plane[2]];
    if VectorNormalize(&mut plane_normal) == 0.0 {
        return qfalse;
    }
    plane[0] = plane_normal[0];
    plane[1] = plane_normal[1];
    plane[2] = plane_normal[2];

    plane[3] = DotProduct(a, &plane_normal);
    qtrue
}

/*
===============
RotatePointAroundVector

This is not implemented very well...
===============
*/
pub fn RotatePointAroundVector(
    dst: &mut vec3_t,
    dir: &vec3_t,
    point: &vec3_t,
    degrees: f32,
) {
    let mut m: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut im: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut zrot: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut tmpmat: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut rot: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut vr: vec3_t = [0.0; 3];
    let mut vup: vec3_t = [0.0; 3];
    let mut vf: vec3_t = [dir[0], dir[1], dir[2]];
    let mut rad: f32;

    PerpendicularVector(&mut vr, dir);
    CrossProduct(&vr, &vf, &mut vup);

    m[0][0] = vr[0];
    m[1][0] = vr[1];
    m[2][0] = vr[2];

    m[0][1] = vup[0];
    m[1][1] = vup[1];
    m[2][1] = vup[2];

    m[0][2] = vf[0];
    m[1][2] = vf[1];
    m[2][2] = vf[2];

    // memcpy( im, m, sizeof( im ) );
    for i in 0..3 {
        for j in 0..3 {
            im[i][j] = m[i][j];
        }
    }

    im[0][1] = m[1][0];
    im[0][2] = m[2][0];
    im[1][0] = m[0][1];
    im[1][2] = m[2][1];
    im[2][0] = m[0][2];
    im[2][1] = m[1][2];

    // memset( zrot, 0, sizeof( zrot ) );
    for i in 0..3 {
        for j in 0..3 {
            zrot[i][j] = 0.0;
        }
    }
    zrot[0][0] = 1.0;
    zrot[1][1] = 1.0;
    zrot[2][2] = 1.0;

    rad = DEG2RAD(degrees);
    zrot[0][0] = unsafe { cos(rad) };
    zrot[0][1] = unsafe { sin(rad) };
    zrot[1][0] = -unsafe { sin(rad) };
    zrot[1][1] = unsafe { cos(rad) };

    MatrixMultiply(&m, &zrot, &mut tmpmat);
    MatrixMultiply(&tmpmat, &im, &mut rot);

    for i in 0..3 {
        dst[i] = rot[i][0] * point[0] + rot[i][1] * point[1] + rot[i][2] * point[2];
    }
}

/*
===============
RotateAroundDirection
===============
*/
pub fn RotateAroundDirection(axis: &mut [vec3_t; 3], yaw: f32) {
    // create an arbitrary axis[1]
    PerpendicularVector(&mut axis[1], &axis[0]);

    // rotate it around axis[0] by yaw
    if yaw != 0.0 {
        let mut temp: vec3_t = [0.0; 3];

        VectorCopy(&axis[1], &mut temp);
        RotatePointAroundVector(&mut axis[1], &axis[0], &temp, yaw);
    }

    // cross to get axis[2]
    CrossProduct(&axis[0], &axis[1], &mut axis[2]);
}

pub fn vectoangles(value1: &vec3_t, angles: &mut vec3_t) {
    let mut forward: f32;
    let mut yaw: f32;
    let mut pitch: f32;

    if value1[1] == 0.0 && value1[0] == 0.0 {
        yaw = 0.0;
        if value1[2] > 0.0 {
            pitch = 90.0;
        } else {
            pitch = 270.0;
        }
    } else {
        if value1[0] != 0.0 {
            yaw = unsafe { atan2(value1[1], value1[0]) } * 180.0 / M_PI;
        } else if value1[1] > 0.0 {
            yaw = 90.0;
        } else {
            yaw = 270.0;
        }
        if yaw < 0.0 {
            yaw += 360.0;
        }

        forward = unsafe { sqrt(value1[0] * value1[0] + value1[1] * value1[1]) };
        pitch = unsafe { atan2(value1[2], forward) } * 180.0 / M_PI;
        if pitch < 0.0 {
            pitch += 360.0;
        }
    }

    angles[PITCH] = -pitch;
    angles[YAW] = yaw;
    angles[ROLL] = 0.0;
}

pub fn ProjectPointOnPlane(dst: &mut vec3_t, p: &vec3_t, normal: &vec3_t) {
    let mut d: f32;
    let mut n: vec3_t = [0.0; 3];
    let mut inv_denom: f32;

    inv_denom = 1.0 / DotProduct(normal, normal);

    d = DotProduct(normal, p) * inv_denom;

    n[0] = normal[0] * inv_denom;
    n[1] = normal[1] * inv_denom;
    n[2] = normal[2] * inv_denom;

    dst[0] = p[0] - d * n[0];
    dst[1] = p[1] - d * n[1];
    dst[2] = p[2] - d * n[2];
}

/*
================
MakeNormalVectors

Given a normalized forward vector, create two
other perpendicular vectors
================
*/
pub fn MakeNormalVectors(forward: &vec3_t, right: &mut vec3_t, up: &mut vec3_t) {
    let mut d: f32;

    // this rotate and negate guarantees a vector
    // not colinear with the original
    right[1] = -forward[0];
    right[2] = forward[1];
    right[0] = forward[2];

    d = DotProduct(right, forward);
    VectorMA(right, -d, forward, right);
    VectorNormalize(right);
    CrossProduct(right, forward, up);
}

//============================================================================

/*
** float q_rsqrt( float number )
*/
pub fn Q_rsqrt(number: f32) -> f32 {
    let threehalfs: f32 = 1.5;

    let x2: f32 = number * 0.5;
    let mut y: f32 = number;
    // evil floating point bit level hacking
    let mut i: i32 = unsafe { *((&y as *const f32) as *const i32) };
    i = 0x5f3759df - (i >> 1); // what the fuck?
    y = unsafe { *((&i as *const i32) as *const f32) };
    y = y * (threehalfs - (x2 * y * y)); // 1st iteration
                                         //	y  = y * ( threehalfs - ( x2 * y * y ) );   // 2nd iteration, this can be removed

    y
}

pub fn Q_fabs(f: f32) -> f32 {
    let mut tmp: i32 = unsafe { *((&f as *const f32) as *const i32) };
    tmp &= 0x7FFFFFFF;
    unsafe { *((&tmp as *const i32) as *const f32) }
}

//============================================================

/*
=================
SetPlaneSignbits
=================
*/
pub fn SetPlaneSignbits(out: &mut cplane_t) {
    let mut bits: c_int = 0;

    // for fast box on planeside test
    for j in 0..3 {
        if out.normal[j] < 0.0 {
            bits |= 1 << j;
        }
    }
    out.signbits = bits;
}

/*
==================
BoxOnPlaneSide

Returns 1, 2, or 1 + 2

// this is the slow, general version
int BoxOnPlaneSide2 (vec3_t emins, vec3_t emaxs, struct cplane_s *p)
{
	int		i;
	float	dist1, dist2;
	int		sides;
	vec3_t	corners[2];

	for (i=0 ; i<3 ; i++)
	{
		if (p->normal[i] < 0)
		{
			corners[0][i] = emins[i];
			corners[1][i] = emaxs[i];
		}
		else
		{
			corners[1][i] = emins[i];
			corners[0][i] = emaxs[i];
		}
	}
	dist1 = DotProduct (p->normal, corners[0]) - p->dist;
	dist2 = DotProduct (p->normal, corners[1]) - p->dist;
	sides = 0;
	if (dist1 >= 0)
		sides = 1;
	if (dist2 < 0)
		sides |= 2;

	return sides;
}

==================
*/

pub fn BoxOnPlaneSide(emins: &vec3_t, emaxs: &vec3_t, p: &cplane_t) -> c_int {
    let mut dist1: f32;
    let mut dist2: f32;
    let mut sides: c_int;

    // fast axial cases
    if p._type < 3 {
        if p.dist <= emins[p._type as usize] {
            return 1;
        }
        if p.dist >= emaxs[p._type as usize] {
            return 2;
        }
        return 3;
    }

    // general case
    match p.signbits {
        0 => {
            dist1 = p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2];
            dist2 = p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2];
        }
        1 => {
            dist1 = p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2];
            dist2 = p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2];
        }
        2 => {
            dist1 = p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2];
            dist2 = p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2];
        }
        3 => {
            dist1 = p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2];
            dist2 = p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2];
        }
        4 => {
            dist1 = p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2];
            dist2 = p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2];
        }
        5 => {
            dist1 = p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2];
            dist2 = p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2];
        }
        6 => {
            dist1 = p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2];
            dist2 = p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2];
        }
        7 => {
            dist1 = p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2];
            dist2 = p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2];
        }
        _ => {
            dist1 = 0.0; // shut up compiler
            dist2 = 0.0;
        }
    }

    sides = 0;
    if dist1 >= p.dist {
        sides = 1;
    }
    if dist2 < p.dist {
        sides |= 2;
    }

    sides
}

/*
=================
RadiusFromBounds
=================
*/
pub fn RadiusFromBounds(mins: &vec3_t, maxs: &vec3_t) -> f32 {
    let mut corner: vec3_t = [0.0; 3];
    let mut a: f32;
    let mut b: f32;

    for i in 0..3 {
        a = Q_fabs(mins[i]);
        b = Q_fabs(maxs[i]);
        corner[i] = if a > b { a } else { b };
    }

    VectorLength(&corner)
}

pub fn ClearBounds(mins: &mut vec3_t, maxs: &mut vec3_t) {
    mins[0] = WORLD_SIZE; //99999;	// I used WORLD_SIZE instead of MAX_WORLD_COORD...
    mins[1] = WORLD_SIZE;
    mins[2] = WORLD_SIZE;
    maxs[0] = -WORLD_SIZE; //-99999;	// ... so it would definately be beyond furthese legal.
    maxs[1] = -WORLD_SIZE;
    maxs[2] = -WORLD_SIZE;
}

pub fn DistanceHorizontal(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];

    VectorSubtract(p2, p1, &mut v);
    unsafe { sqrt(v[0] * v[0] + v[1] * v[1]) } //Leave off the z component
}

pub fn DistanceHorizontalSquared(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];

    VectorSubtract(p2, p1, &mut v);
    v[0] * v[0] + v[1] * v[1] //Leave off the z component
}

pub fn Q_log2(mut val: c_int) -> c_int {
    let mut answer: c_int = 0;

    while (val >> 1) != 0 {
        answer += 1;
        val >>= 1;
    }
    answer
}

/*
=================
PlaneTypeForNormal
=================
*/
pub fn PlaneTypeForNormal(normal: &vec3_t) -> c_int {
    if normal[0] == 1.0 {
        return PLANE_X;
    }
    if normal[1] == 1.0 {
        return PLANE_Y;
    }
    if normal[2] == 1.0 {
        return PLANE_Z;
    }

    PLANE_NON_AXIAL
}

/*
================
MatrixMultiply
================
*/
pub fn MatrixMultiply(in1: &[[f32; 3]; 3], in2: &[[f32; 3]; 3], out: &mut [[f32; 3]; 3]) {
    out[0][0] = in1[0][0] * in2[0][0] + in1[0][1] * in2[1][0] + in1[0][2] * in2[2][0];
    out[0][1] = in1[0][0] * in2[0][1] + in1[0][1] * in2[1][1] + in1[0][2] * in2[2][1];
    out[0][2] = in1[0][0] * in2[0][2] + in1[0][1] * in2[1][2] + in1[0][2] * in2[2][2];
    out[1][0] = in1[1][0] * in2[0][0] + in1[1][1] * in2[1][0] + in1[1][2] * in2[2][0];
    out[1][1] = in1[1][0] * in2[0][1] + in1[1][1] * in2[1][1] + in1[1][2] * in2[2][1];
    out[1][2] = in1[1][0] * in2[0][2] + in1[1][1] * in2[1][2] + in1[1][2] * in2[2][2];
    out[2][0] = in1[2][0] * in2[0][0] + in1[2][1] * in2[1][0] + in1[2][2] * in2[2][0];
    out[2][1] = in1[2][0] * in2[0][1] + in1[2][1] * in2[1][1] + in1[2][2] * in2[2][1];
    out[2][2] = in1[2][0] * in2[0][2] + in1[2][1] * in2[1][2] + in1[2][2] * in2[2][2];
}

pub fn AngleVectors(angles: &vec3_t, forward: Option<&mut vec3_t>, right: Option<&mut vec3_t>, up: Option<&mut vec3_t>) {
    let mut angle: f32;
    static mut sr: f32 = 0.0;
    static mut sp: f32 = 0.0;
    static mut sy: f32 = 0.0;
    static mut cr: f32 = 0.0;
    static mut cp: f32 = 0.0;
    static mut cy: f32 = 0.0;
    // static to help MS compiler fp bugs

    angle = angles[YAW] * (M_PI * 2.0 / 360.0);
    unsafe {
        sy = sin(angle);
        cy = cos(angle);
    }
    angle = angles[PITCH] * (M_PI * 2.0 / 360.0);
    unsafe {
        sp = sin(angle);
        cp = cos(angle);
    }

    if let Some(fwd) = forward {
        fwd[0] = unsafe { cp * cy };
        fwd[1] = unsafe { cp * sy };
        fwd[2] = unsafe { -sp };
    }
    if right.is_some() || up.is_some() {
        angle = angles[ROLL] * (M_PI * 2.0 / 360.0);
        unsafe {
            sr = sin(angle);
            cr = cos(angle);
        }
        if let Some(r) = right {
            unsafe {
                r[0] = -sr * sp * cy + cr * sy;
                r[1] = -sr * sp * sy + -cr * cy;
                r[2] = -sr * cp;
            }
        }
        if let Some(u) = up {
            unsafe {
                u[0] = cr * sp * cy + sr * sy;
                u[1] = cr * sp * sy + -sr * cy;
                u[2] = cr * cp;
            }
        }
    }
}

/*
** assumes "src" is normalized
*/
pub fn PerpendicularVector(dst: &mut vec3_t, src: &vec3_t) {
    let mut pos: c_int = 0;
    let mut minelem: f32 = 1.0;
    let mut tempvec: vec3_t = [0.0; 3];

    /*
    ** find the smallest magnitude axially aligned vector
    ** bias towards using z instead of x or y
    */
    for i in (0..=2).rev() {
        if Q_fabs(src[i]) < minelem {
            pos = i as c_int;
            minelem = Q_fabs(src[i]);
        }
    }
    tempvec[0] = 0.0;
    tempvec[1] = 0.0;
    tempvec[2] = 0.0;
    tempvec[pos as usize] = 1.0;

    /*
    ** project the point onto the plane defined by src
    */
    ProjectPointOnPlane(dst, &tempvec, src);

    /*
    ** normalize the result
    */
    VectorNormalize(dst);
}

/*
-------------------------
DotProductNormalize
-------------------------
*/

pub fn DotProductNormalize(inVec1: &vec3_t, inVec2: &vec3_t) -> f32 {
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    VectorNormalize2(inVec1, &mut v1);
    VectorNormalize2(inVec2, &mut v2);

    DotProduct(&v1, &v2)
}

/*
-------------------------
G_FindClosestPointOnLineSegment
-------------------------
*/

pub fn G_FindClosestPointOnLineSegment(
    start: &vec3_t,
    end: &vec3_t,
    from: &vec3_t,
    result: &mut vec3_t,
) -> qboolean {
    let mut vecStart2From: vec3_t = [0.0; 3];
    let mut vecStart2End: vec3_t = [0.0; 3];
    let mut vecEnd2Start: vec3_t = [0.0; 3];
    let mut vecEnd2From: vec3_t = [0.0; 3];
    let mut distEnd2From: f32;
    let mut distEnd2Result: f32;
    let mut theta: f32;
    let mut cos_theta: f32;

    //Find the perpendicular vector to vec from start to end
    VectorSubtract(from, start, &mut vecStart2From);
    VectorSubtract(end, start, &mut vecStart2End);

    let mut dot = DotProductNormalize(&vecStart2From, &vecStart2End);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the start point
        VectorCopy(start, result);
        return qfalse;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        if VectorLengthSquared(&vecStart2From) < VectorLengthSquared(&vecStart2End) {
            VectorCopy(from, result);
        } else {
            VectorCopy(end, result);
        }
        return qfalse;
    }

    //Try other end
    VectorSubtract(from, end, &mut vecEnd2From);
    VectorSubtract(start, end, &mut vecEnd2Start);

    dot = DotProductNormalize(&vecEnd2From, &vecEnd2Start);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the start point
        VectorCopy(end, result);
        return qfalse;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        if VectorLengthSquared(&vecEnd2From) < VectorLengthSquared(&vecEnd2Start) {
            VectorCopy(from, result);
        } else {
            VectorCopy(end, result);
        }
        return qfalse;
    }

    //		      /|
    //		  c  / |
    //		    /  |a
    //	theta  /)__|
    //		      b
    //cos(theta) = b / c
    //solve for b
    //b = cos(theta) * c

    //angle between vecs end2from and end2start, should be between 0 and 90
    theta = 90.0 * (1.0 - dot); //theta

    //Get length of side from End2Result using sine of theta
    distEnd2From = VectorLength(&vecEnd2From); //c
    cos_theta = unsafe { cos(DEG2RAD(theta)) }; //cos(theta)
    distEnd2Result = cos_theta * distEnd2From; //b

    //Extrapolate to find result
    VectorNormalize(&mut vecEnd2Start);
    VectorMA(end, distEnd2Result, &vecEnd2Start, result);

    //perpendicular intersection is between the 2 endpoints
    qtrue
}

pub fn G_PointDistFromLineSegment(start: &vec3_t, end: &vec3_t, from: &vec3_t) -> f32 {
    let mut vecStart2From: vec3_t = [0.0; 3];
    let mut vecStart2End: vec3_t = [0.0; 3];
    let mut vecEnd2Start: vec3_t = [0.0; 3];
    let mut vecEnd2From: vec3_t = [0.0; 3];
    let mut intersection: vec3_t = [0.0; 3];
    let mut distEnd2From: f32;
    let mut distStart2From: f32;
    let mut distEnd2Result: f32;
    let mut theta: f32;
    let mut cos_theta: f32;

    //Find the perpendicular vector to vec from start to end
    VectorSubtract(from, start, &mut vecStart2From);
    VectorSubtract(end, start, &mut vecStart2End);
    VectorSubtract(from, end, &mut vecEnd2From);
    VectorSubtract(start, end, &mut vecEnd2Start);

    let mut dot = DotProductNormalize(&vecStart2From, &vecStart2End);

    distStart2From = Distance(start, from);
    distEnd2From = Distance(end, from);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the start point
        return distStart2From;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        return if distStart2From < distEnd2From {
            distStart2From
        } else {
            distEnd2From
        };
    }

    //Try other end

    dot = DotProductNormalize(&vecEnd2From, &vecEnd2Start);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the end point
        return distEnd2From;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        return if distStart2From < distEnd2From {
            distStart2From
        } else {
            distEnd2From
        };
    }

    //		      /|
    //		  c  / |
    //		    /  |a
    //	theta  /)__|
    //		      b
    //cos(theta) = b / c
    //solve for b
    //b = cos(theta) * c

    //angle between vecs end2from and end2start, should be between 0 and 90
    theta = 90.0 * (1.0 - dot); //theta

    //Get length of side from End2Result using sine of theta
    cos_theta = unsafe { cos(DEG2RAD(theta)) }; //cos(theta)
    distEnd2Result = cos_theta * distEnd2From; //b

    //Extrapolate to find result
    VectorNormalize(&mut vecEnd2Start);
    VectorMA(end, distEnd2Result, &vecEnd2Start, &mut intersection);

    //perpendicular intersection is between the 2 endpoints, return dist to it from from
    Distance(&intersection, from)
}
