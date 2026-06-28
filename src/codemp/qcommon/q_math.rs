// Copyright (C) 1999-2000 Id Software, Inc.
//
// q_math.c -- stateless support routines that are included in each code module

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// Type aliases matching the C definitions
pub type vec_t = f32;
pub type vec3_t = [f32; 3];
pub type vec4_t = [f32; 4];
pub type byte = u8;
pub type qboolean = c_int;

pub const PITCH: usize = 0;
pub const YAW: usize = 1;
pub const ROLL: usize = 2;

pub const M_PI: f32 = 3.14159265358979323846f32;

pub const NUMVERTEXNORMALS: usize = 162;

pub static mut vec3_origin: vec3_t = [0.0, 0.0, 0.0];
pub static mut axisDefault: [vec3_t; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

pub static mut colorBlack: vec4_t = [0.0, 0.0, 0.0, 1.0];
pub static mut colorRed: vec4_t = [1.0, 0.0, 0.0, 1.0];
pub static mut colorGreen: vec4_t = [0.0, 1.0, 0.0, 1.0];
pub static mut colorBlue: vec4_t = [0.0, 0.0, 1.0, 1.0];
pub static mut colorYellow: vec4_t = [1.0, 1.0, 0.0, 1.0];
pub static mut colorMagenta: vec4_t = [1.0, 0.0, 1.0, 1.0];
pub static mut colorCyan: vec4_t = [0.0, 1.0, 1.0, 1.0];
pub static mut colorWhite: vec4_t = [1.0, 1.0, 1.0, 1.0];
pub static mut colorLtGrey: vec4_t = [0.75, 0.75, 0.75, 1.0];
pub static mut colorMdGrey: vec4_t = [0.5, 0.5, 0.5, 1.0];
pub static mut colorDkGrey: vec4_t = [0.25, 0.25, 0.25, 1.0];

pub static mut colorLtBlue: vec4_t = [0.367f32, 0.261f32, 0.722f32, 1.0];
pub static mut colorDkBlue: vec4_t = [0.199f32, 0.0f32, 0.398f32, 1.0];

pub static mut g_color_table: [[f32; 4]; 8] = [
    [0.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [1.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0],
    [0.0, 1.0, 1.0, 1.0],
    [1.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
];

pub static mut bytedirs: [[f32; 3]; NUMVERTEXNORMALS] = [
    [-0.525731f32, 0.000000f32, 0.850651f32], [-0.442863f32, 0.238856f32, 0.864188f32],
    [-0.295242f32, 0.000000f32, 0.955423f32], [-0.309017f32, 0.500000f32, 0.809017f32],
    [-0.162460f32, 0.262866f32, 0.951056f32], [0.000000f32, 0.000000f32, 1.000000f32],
    [0.000000f32, 0.850651f32, 0.525731f32], [-0.147621f32, 0.716567f32, 0.681718f32],
    [0.147621f32, 0.716567f32, 0.681718f32], [0.000000f32, 0.525731f32, 0.850651f32],
    [0.309017f32, 0.500000f32, 0.809017f32], [0.525731f32, 0.000000f32, 0.850651f32],
    [0.295242f32, 0.000000f32, 0.955423f32], [0.442863f32, 0.238856f32, 0.864188f32],
    [0.162460f32, 0.262866f32, 0.951056f32], [-0.681718f32, 0.147621f32, 0.716567f32],
    [-0.809017f32, 0.309017f32, 0.500000f32], [-0.587785f32, 0.425325f32, 0.688191f32],
    [-0.850651f32, 0.525731f32, 0.000000f32], [-0.864188f32, 0.442863f32, 0.238856f32],
    [-0.716567f32, 0.681718f32, 0.147621f32], [-0.688191f32, 0.587785f32, 0.425325f32],
    [-0.500000f32, 0.809017f32, 0.309017f32], [-0.238856f32, 0.864188f32, 0.442863f32],
    [-0.425325f32, 0.688191f32, 0.587785f32], [-0.716567f32, 0.681718f32, -0.147621f32],
    [-0.500000f32, 0.809017f32, -0.309017f32], [-0.525731f32, 0.850651f32, 0.000000f32],
    [0.000000f32, 0.850651f32, -0.525731f32], [-0.238856f32, 0.864188f32, -0.442863f32],
    [0.000000f32, 0.955423f32, -0.295242f32], [-0.262866f32, 0.951056f32, -0.162460f32],
    [0.000000f32, 1.000000f32, 0.000000f32], [0.000000f32, 0.955423f32, 0.295242f32],
    [-0.262866f32, 0.951056f32, 0.162460f32], [0.238856f32, 0.864188f32, 0.442863f32],
    [0.262866f32, 0.951056f32, 0.162460f32], [0.500000f32, 0.809017f32, 0.309017f32],
    [0.238856f32, 0.864188f32, -0.442863f32], [0.262866f32, 0.951056f32, -0.162460f32],
    [0.500000f32, 0.809017f32, -0.309017f32], [0.850651f32, 0.525731f32, 0.000000f32],
    [0.716567f32, 0.681718f32, 0.147621f32], [0.716567f32, 0.681718f32, -0.147621f32],
    [0.525731f32, 0.850651f32, 0.000000f32], [0.425325f32, 0.688191f32, 0.587785f32],
    [0.864188f32, 0.442863f32, 0.238856f32], [0.688191f32, 0.587785f32, 0.425325f32],
    [0.809017f32, 0.309017f32, 0.500000f32], [0.681718f32, 0.147621f32, 0.716567f32],
    [0.587785f32, 0.425325f32, 0.688191f32], [0.955423f32, 0.295242f32, 0.000000f32],
    [1.000000f32, 0.000000f32, 0.000000f32], [0.951056f32, 0.162460f32, 0.262866f32],
    [0.850651f32, -0.525731f32, 0.000000f32], [0.955423f32, -0.295242f32, 0.000000f32],
    [0.864188f32, -0.442863f32, 0.238856f32], [0.951056f32, -0.162460f32, 0.262866f32],
    [0.809017f32, -0.309017f32, 0.500000f32], [0.681718f32, -0.147621f32, 0.716567f32],
    [0.850651f32, 0.000000f32, 0.525731f32], [0.864188f32, 0.442863f32, -0.238856f32],
    [0.809017f32, 0.309017f32, -0.500000f32], [0.951056f32, 0.162460f32, -0.262866f32],
    [0.525731f32, 0.000000f32, -0.850651f32], [0.681718f32, 0.147621f32, -0.716567f32],
    [0.681718f32, -0.147621f32, -0.716567f32], [0.850651f32, 0.000000f32, -0.525731f32],
    [0.809017f32, -0.309017f32, -0.500000f32], [0.864188f32, -0.442863f32, -0.238856f32],
    [0.951056f32, -0.162460f32, -0.262866f32], [0.147621f32, 0.716567f32, -0.681718f32],
    [0.309017f32, 0.500000f32, -0.809017f32], [0.425325f32, 0.688191f32, -0.587785f32],
    [0.442863f32, 0.238856f32, -0.864188f32], [0.587785f32, 0.425325f32, -0.688191f32],
    [0.688191f32, 0.587785f32, -0.425325f32], [-0.147621f32, 0.716567f32, -0.681718f32],
    [-0.309017f32, 0.500000f32, -0.809017f32], [0.000000f32, 0.525731f32, -0.850651f32],
    [-0.525731f32, 0.000000f32, -0.850651f32], [-0.442863f32, 0.238856f32, -0.864188f32],
    [-0.295242f32, 0.000000f32, -0.955423f32], [-0.162460f32, 0.262866f32, -0.951056f32],
    [0.000000f32, 0.000000f32, -1.000000f32], [0.295242f32, 0.000000f32, -0.955423f32],
    [0.162460f32, 0.262866f32, -0.951056f32], [-0.442863f32, -0.238856f32, -0.864188f32],
    [-0.309017f32, -0.500000f32, -0.809017f32], [-0.162460f32, -0.262866f32, -0.951056f32],
    [0.000000f32, -0.850651f32, -0.525731f32], [-0.147621f32, -0.716567f32, -0.681718f32],
    [0.147621f32, -0.716567f32, -0.681718f32], [0.000000f32, -0.525731f32, -0.850651f32],
    [0.309017f32, -0.500000f32, -0.809017f32], [0.442863f32, -0.238856f32, -0.864188f32],
    [0.162460f32, -0.262866f32, -0.951056f32], [0.238856f32, -0.864188f32, -0.442863f32],
    [0.500000f32, -0.809017f32, -0.309017f32], [0.425325f32, -0.688191f32, -0.587785f32],
    [0.716567f32, -0.681718f32, -0.147621f32], [0.688191f32, -0.587785f32, -0.425325f32],
    [0.587785f32, -0.425325f32, -0.688191f32], [0.000000f32, -0.955423f32, -0.295242f32],
    [0.000000f32, -1.000000f32, 0.000000f32], [0.262866f32, -0.951056f32, -0.162460f32],
    [0.000000f32, -0.850651f32, 0.525731f32], [0.000000f32, -0.955423f32, 0.295242f32],
    [0.238856f32, -0.864188f32, 0.442863f32], [0.262866f32, -0.951056f32, 0.162460f32],
    [0.500000f32, -0.809017f32, 0.309017f32], [0.716567f32, -0.681718f32, 0.147621f32],
    [0.525731f32, -0.850651f32, 0.000000f32], [-0.238856f32, -0.864188f32, -0.442863f32],
    [-0.500000f32, -0.809017f32, -0.309017f32], [-0.262866f32, -0.951056f32, -0.162460f32],
    [-0.850651f32, -0.525731f32, 0.000000f32], [-0.716567f32, -0.681718f32, -0.147621f32],
    [-0.716567f32, -0.681718f32, 0.147621f32], [-0.525731f32, -0.850651f32, 0.000000f32],
    [-0.500000f32, -0.809017f32, 0.309017f32], [-0.238856f32, -0.864188f32, 0.442863f32],
    [-0.262866f32, -0.951056f32, 0.162460f32], [-0.864188f32, -0.442863f32, 0.238856f32],
    [-0.809017f32, -0.309017f32, 0.500000f32], [-0.688191f32, -0.587785f32, 0.425325f32],
    [-0.681718f32, -0.147621f32, 0.716567f32], [-0.442863f32, -0.238856f32, 0.864188f32],
    [-0.587785f32, -0.425325f32, 0.688191f32], [-0.309017f32, -0.500000f32, 0.809017f32],
    [-0.147621f32, -0.716567f32, 0.681718f32], [-0.425325f32, -0.688191f32, 0.587785f32],
    [-0.162460f32, -0.262866f32, 0.951056f32], [0.442863f32, -0.238856f32, 0.864188f32],
    [0.162460f32, -0.262866f32, 0.951056f32], [0.309017f32, -0.500000f32, 0.809017f32],
    [0.147621f32, -0.716567f32, 0.681718f32], [0.000000f32, -0.525731f32, 0.850651f32],
    [0.425325f32, -0.688191f32, 0.587785f32], [0.587785f32, -0.425325f32, 0.688191f32],
    [0.688191f32, -0.587785f32, 0.425325f32], [-0.955423f32, 0.295242f32, 0.000000f32],
    [-0.951056f32, 0.162460f32, 0.262866f32], [-1.000000f32, 0.000000f32, 0.000000f32],
    [-0.850651f32, 0.000000f32, 0.525731f32], [-0.955423f32, -0.295242f32, 0.000000f32],
    [-0.951056f32, -0.162460f32, 0.262866f32], [-0.864188f32, 0.442863f32, -0.238856f32],
    [-0.951056f32, 0.162460f32, -0.262866f32], [-0.809017f32, 0.309017f32, -0.500000f32],
    [-0.864188f32, -0.442863f32, -0.238856f32], [-0.951056f32, -0.162460f32, -0.262866f32],
    [-0.809017f32, -0.309017f32, -0.500000f32], [-0.681718f32, 0.147621f32, -0.716567f32],
    [-0.681718f32, -0.147621f32, -0.716567f32], [-0.850651f32, 0.000000f32, -0.525731f32],
    [-0.688191f32, 0.587785f32, -0.425325f32], [-0.587785f32, 0.425325f32, -0.688191f32],
    [-0.425325f32, 0.688191f32, -0.587785f32], [-0.425325f32, -0.688191f32, -0.587785f32],
    [-0.587785f32, -0.425325f32, -0.688191f32], [-0.688191f32, -0.587785f32, -0.425325f32],
];

//==============================================================

pub fn Q_rand(seed: &mut i32) -> i32 {
    *seed = (69069i32).wrapping_mul(*seed).wrapping_add(1);
    *seed
}

pub fn Q_random(seed: &mut i32) -> f32 {
    ((Q_rand(seed) & 0xffff) as f32) / (0x10000 as f32)
}

pub fn Q_crandom(seed: &mut i32) -> f32 {
    2.0 * (Q_random(seed) - 0.5)
}

#[cfg(__LCC__)]
pub fn VectorCompare(v1: &vec3_t, v2: &vec3_t) -> c_int {
    if v1[0] != v2[0] || v1[1] != v2[1] || v1[2] != v2[2] {
        0
    } else {
        1
    }
}

#[cfg(__LCC__)]
pub fn VectorLength(v: &vec3_t) -> vec_t {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[cfg(__LCC__)]
pub fn VectorLengthSquared(v: &vec3_t) -> vec_t {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[cfg(__LCC__)]
pub fn Distance(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(p2, p1, &mut v);
    VectorLength(&v)
}

#[cfg(__LCC__)]
pub fn DistanceSquared(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(p2, p1, &mut v);
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

// fast vector normalize routine that does not check to make sure
// that length != 0, nor does it return length, uses rsqrt approximation
#[cfg(__LCC__)]
pub fn VectorNormalizeFast(v: &mut vec3_t) {
    let ilength: vec_t;

    ilength = Q_rsqrt(_DotProduct(v, v));

    v[0] *= ilength;
    v[1] *= ilength;
    v[2] *= ilength;
}

#[cfg(__LCC__)]
pub fn VectorInverse(v: &mut vec3_t) {
    v[0] = -v[0];
    v[1] = -v[1];
    v[2] = -v[2];
}

//i wrote this function in a console test app and it appeared faster
//in debug and release than the standard crossproduct asm generated
//by the compiler. however, when inlining the crossproduct function
//the compiler performs further optimizations and generally ends up
//being faster than this asm version. but feel free to try this one
//and see if you're heavily crossproducting in an area and looking
//for a way to optimize. -rww
#[cfg(__LCC__)]
pub fn CrossProduct(v1: &vec3_t, v2: &vec3_t, cross: &mut vec3_t) {
    cross[0] = v1[1] * v2[2] - v1[2] * v2[1];
    cross[1] = v1[2] * v2[0] - v1[0] * v2[2];
    cross[2] = v1[0] * v2[1] - v1[1] * v2[0];
}

//=======================================================

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
    if i < -32768 {
        -32768
    } else if i > 0x7fff {
        0x7fff
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
    i = 0;
    while i < NUMVERTEXNORMALS as c_int {
        d = _DotProduct(dir, &bytedirs[i as usize]);
        if d > bestd {
            bestd = d;
            best = i;
        }
        i += 1;
    }

    best
}

pub fn ByteToDir(b: c_int, dir: &mut vec3_t) {
    if b < 0 || b >= NUMVERTEXNORMALS as c_int {
        _VectorCopy(&unsafe { vec3_origin }, dir);
        return;
    }
    _VectorCopy(&bytedirs[b as usize], dir);
}

pub fn ColorBytes3(r: f32, g: f32, b: f32) -> u32 {
    let mut i: u32 = 0;

    // SAFETY: Casting to bytes array for C-like behavior
    unsafe {
        let bytes = &mut i as *mut u32 as *mut [u8; 4];
        (*bytes)[0] = (r * 255.0) as u8;
        (*bytes)[1] = (g * 255.0) as u8;
        (*bytes)[2] = (b * 255.0) as u8;
    }

    i
}

pub fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let mut i: u32 = 0;

    // SAFETY: Casting to bytes array for C-like behavior
    unsafe {
        let bytes = &mut i as *mut u32 as *mut [u8; 4];
        (*bytes)[0] = (r * 255.0) as u8;
        (*bytes)[1] = (g * 255.0) as u8;
        (*bytes)[2] = (b * 255.0) as u8;
        (*bytes)[3] = (a * 255.0) as u8;
    }

    i
}

pub fn NormalizeColor(in_vec: &vec3_t, out_vec: &mut vec3_t) -> f32 {
    let mut max: f32;

    max = in_vec[0];
    if in_vec[1] > max {
        max = in_vec[1];
    }
    if in_vec[2] > max {
        max = in_vec[2];
    }

    if max == 0.0 {
        _VectorClear(out_vec);
    } else {
        out_vec[0] = in_vec[0] / max;
        out_vec[1] = in_vec[1] / max;
        out_vec[2] = in_vec[2] / max;
    }
    max
}

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

    _VectorSubtract(b, a, &mut d1);
    _VectorSubtract(c, a, &mut d2);
    CrossProduct(&d2, &d1, &mut [plane[0], plane[1], plane[2]]);
    if VectorNormalize(&mut [plane[0], plane[1], plane[2]]) == 0.0 {
        return 0;
    }

    plane[3] = _DotProduct(a, &[plane[0], plane[1], plane[2]]);
    1
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
    let mut i: usize;
    let mut vr: vec3_t = [0.0; 3];
    let mut vup: vec3_t = [0.0; 3];
    let mut vf: vec3_t = [0.0; 3];
    let rad: f32;

    vf[0] = dir[0];
    vf[1] = dir[1];
    vf[2] = dir[2];

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

    im = m;

    im[0][1] = m[1][0];
    im[0][2] = m[2][0];
    im[1][0] = m[0][1];
    im[1][2] = m[2][1];
    im[2][0] = m[0][2];
    im[2][1] = m[1][2];

    zrot[0][0] = 1.0f32;
    zrot[1][1] = 1.0f32;
    zrot[2][2] = 1.0f32;

    rad = DEG2RAD(degrees);
    zrot[0][0] = rad.cos();
    zrot[0][1] = rad.sin();
    zrot[1][0] = -rad.sin();
    zrot[1][1] = rad.cos();

    MatrixMultiply(&m, &zrot, &mut tmpmat);
    MatrixMultiply(&tmpmat, &im, &mut rot);

    i = 0;
    while i < 3 {
        dst[i] = rot[i][0] * point[0] + rot[i][1] * point[1] + rot[i][2] * point[2];
        i += 1;
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

        _VectorCopy(&axis[1], &mut temp);
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
            yaw = (value1[1].atan2(value1[0]) * 180.0 / M_PI);
        } else if value1[1] > 0.0 {
            yaw = 90.0;
        } else {
            yaw = 270.0;
        }
        if yaw < 0.0 {
            yaw += 360.0;
        }

        forward = (value1[0] * value1[0] + value1[1] * value1[1]).sqrt();
        pitch = (value1[2].atan2(forward) * 180.0 / M_PI);
        if pitch < 0.0 {
            pitch += 360.0;
        }
    }

    angles[PITCH] = -pitch;
    angles[YAW] = yaw;
    angles[ROLL] = 0.0;
}

/*
=================
AnglesToAxis
=================
*/
pub fn AnglesToAxis(angles: &vec3_t, axis: &mut [vec3_t; 3]) {
    let mut right: vec3_t = [0.0; 3];

    // angle vectors returns "right" instead of "y axis"
    AngleVectors(angles, &mut axis[0], &mut right, &mut axis[2]);
    _VectorSubtract(&unsafe { vec3_origin }, &right, &mut axis[1]);
}

pub fn AxisClear(axis: &mut [vec3_t; 3]) {
    axis[0][0] = 1.0;
    axis[0][1] = 0.0;
    axis[0][2] = 0.0;
    axis[1][0] = 0.0;
    axis[1][1] = 1.0;
    axis[1][2] = 0.0;
    axis[2][0] = 0.0;
    axis[2][1] = 0.0;
    axis[2][2] = 1.0;
}

pub fn AxisCopy(in_axis: &[vec3_t; 3], out_axis: &mut [vec3_t; 3]) {
    _VectorCopy(&in_axis[0], &mut out_axis[0]);
    _VectorCopy(&in_axis[1], &mut out_axis[1]);
    _VectorCopy(&in_axis[2], &mut out_axis[2]);
}

pub fn ProjectPointOnPlane(dst: &mut vec3_t, p: &vec3_t, normal: &vec3_t) {
    let d: f32;
    let mut n: vec3_t = [0.0; 3];
    let inv_denom: f32;

    inv_denom = _DotProduct(normal, normal);
    // bk010122 - zero vectors get here
    let inv_denom = 1.0f32 / inv_denom;

    d = _DotProduct(normal, p) * inv_denom;

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
    let d: f32;

    // this rotate and negate guarantees a vector
    // not colinear with the original
    right[1] = -forward[0];
    right[2] = forward[1];
    right[0] = forward[2];

    d = _DotProduct(right, forward);
    _VectorMA(right, -d, forward, right);
    VectorNormalize(right);
    CrossProduct(right, forward, up);
}

pub fn VectorRotate(in_vec: &vec3_t, matrix: &[vec3_t; 3], out_vec: &mut vec3_t) {
    out_vec[0] = _DotProduct(in_vec, &matrix[0]);
    out_vec[1] = _DotProduct(in_vec, &matrix[1]);
    out_vec[2] = _DotProduct(in_vec, &matrix[2]);
}

//============================================================================

/*
** float q_rsqrt( float number )
*/
pub fn Q_rsqrt(number: f32) -> f32 {
    let mut i: i32;
    let x2: f32;
    let mut y: f32;
    const THREEHALFS: f32 = 1.5f32;

    let x2 = number * 0.5f32;
    y = number;
    // SAFETY: evil floating point bit level hacking
    unsafe {
        i = *(std::ptr::addr_of!(y) as *const i32);
    }
    i = 0x5f3759df - (i >> 1); // what the fuck?
    // SAFETY: evil floating point bit level hacking
    unsafe {
        y = *(std::ptr::addr_of!(i) as *const f32);
    }
    y = y * (THREEHALFS - (x2 * y * y)); // 1st iteration
    //	y  = y * ( threehalfs - ( x2 * y * y ) );   // 2nd iteration, this can be removed

    y
}

pub fn Q_fabs(f: f32) -> f32 {
    let mut tmp: i32;
    // SAFETY: bit level hacking
    unsafe {
        tmp = *(std::ptr::addr_of!(f) as *const i32);
    }
    tmp &= 0x7FFFFFFF;
    // SAFETY: bit level hacking
    unsafe {
        *(std::ptr::addr_of!(tmp) as *const f32)
    }
}

//============================================================

/*
===============
LerpAngle

===============
*/
pub fn LerpAngle(from: f32, to: f32, frac: f32) -> f32 {
    let mut a: f32;
    let mut to_mut: f32 = to;

    if to_mut - from > 180.0 {
        to_mut -= 360.0;
    }
    if to_mut - from < -180.0 {
        to_mut += 360.0;
    }
    a = from + frac * (to_mut - from);

    a
}

/*
=================
AngleSubtract

Always returns a value from -180 to 180
=================
*/
pub fn AngleSubtract(a1: f32, a2: f32) -> f32 {
    let mut a: f32;

    a = a1 - a2;
    a = a.rem_euclid(360.0); //chop it down quickly, then level it out
    while a > 180.0 {
        a -= 360.0;
    }
    while a < -180.0 {
        a += 360.0;
    }
    a
}

pub fn AnglesSubtract(v1: &vec3_t, v2: &vec3_t, v3: &mut vec3_t) {
    v3[0] = AngleSubtract(v1[0], v2[0]);
    v3[1] = AngleSubtract(v1[1], v2[1]);
    v3[2] = AngleSubtract(v1[2], v2[2]);
}

pub fn AngleMod(a_in: f32) -> f32 {
    (360.0 / 65536.0) * (((a_in * (65536.0 / 360.0)) as i32) & 65535) as f32
}

/*
=================
AngleNormalize360

returns angle normalized to the range [0 <= angle < 360]
=================
*/
pub fn AngleNormalize360(angle: f32) -> f32 {
    (360.0 / 65536.0) * (((angle * (65536.0 / 360.0)) as i32) & 65535) as f32
}

/*
=================
AngleNormalize180

returns angle normalized to the range [-180 < angle <= 180]
=================
*/
pub fn AngleNormalize180(angle_in: f32) -> f32 {
    let mut angle: f32 = AngleNormalize360(angle_in);
    if angle > 180.0 {
        angle -= 360.0;
    }
    angle
}

/*
=================
AngleDelta

returns the normalized delta from angle1 to angle2
=================
*/
pub fn AngleDelta(angle1: f32, angle2: f32) -> f32 {
    AngleNormalize180(angle1 - angle2)
}

//============================================================

#[repr(C)]
pub struct cplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
    pub pad: [u8; 2],
}

/*
=================
SetPlaneSignbits
=================
*/
pub fn SetPlaneSignbits(out: &mut cplane_t) {
    let mut bits: c_int;
    let mut j: c_int;

    // for fast box on planeside test
    bits = 0;
    j = 0;
    while j < 3 {
        if out.normal[j as usize] < 0.0 {
            bits |= 1 << j;
        }
        j += 1;
    }
    out.signbits = bits as u8;
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
    let dist1: f32;
    let dist2: f32;
    let sides: c_int;

    // fast axial cases
    if (p.type_ as c_int) < 3 {
        if p.dist <= emins[p.type_ as usize] {
            return 1;
        }
        if p.dist >= emaxs[p.type_ as usize] {
            return 2;
        }
        return 3;
    }

    // general case
    let (dist1, dist2) = match p.signbits as c_int {
        0 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
        ),
        1 => (
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
        ),
        2 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
        ),
        3 => (
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
        ),
        4 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
        ),
        5 => (
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emins[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emaxs[2],
        ),
        6 => (
            p.normal[0] * emaxs[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
            p.normal[0] * emins[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
        ),
        7 => (
            p.normal[0] * emins[0] + p.normal[1] * emins[1] + p.normal[2] * emins[2],
            p.normal[0] * emaxs[0] + p.normal[1] * emaxs[1] + p.normal[2] * emaxs[2],
        ),
        _ => (0.0, 0.0), // shut up compiler
    };

    let mut sides: c_int = 0;
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
    let mut i: c_int;
    let mut corner: vec3_t = [0.0; 3];
    let a: f32;
    let b: f32;

    i = 0;
    while i < 3 {
        a = mins[i as usize].abs();
        b = maxs[i as usize].abs();
        corner[i as usize] = if a > b { a } else { b };
        i += 1;
    }

    VectorLength(&corner)
}

pub fn ClearBounds(mins: &mut vec3_t, maxs: &mut vec3_t) {
    mins[0] = 99999.0;
    mins[1] = 99999.0;
    mins[2] = 99999.0;
    maxs[0] = -99999.0;
    maxs[1] = -99999.0;
    maxs[2] = -99999.0;
}

pub fn DistanceHorizontal(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];

    _VectorSubtract(p2, p1, &mut v);
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

pub fn DistanceHorizontalSquared(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];

    _VectorSubtract(p2, p1, &mut v);
    v[0] * v[0] + v[1] * v[1]
}

pub fn AddPointToBounds(v: &vec3_t, mins: &mut vec3_t, maxs: &mut vec3_t) {
    if v[0] < mins[0] {
        mins[0] = v[0];
    }
    if v[0] > maxs[0] {
        maxs[0] = v[0];
    }

    if v[1] < mins[1] {
        mins[1] = v[1];
    }
    if v[1] > maxs[1] {
        maxs[1] = v[1];
    }

    if v[2] < mins[2] {
        mins[2] = v[2];
    }
    if v[2] > maxs[2] {
        maxs[2] = v[2];
    }
}

pub fn VectorNormalize(v: &mut vec3_t) -> vec_t {
    let length: f32;
    let ilength: f32;

    length = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    let length = length.sqrt();

    if length != 0.0 {
        let ilength = 1.0 / length;
        v[0] *= ilength;
        v[1] *= ilength;
        v[2] *= ilength;
    }

    length
}

pub fn VectorNormalize2(v: &vec3_t, out: &mut vec3_t) -> vec_t {
    let length: f32;
    let ilength: f32;

    let length = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    let length = length.sqrt();

    if length != 0.0 {
        // bk0101022 - FPE related
        let ilength = 1.0 / length;
        out[0] = v[0] * ilength;
        out[1] = v[1] * ilength;
        out[2] = v[2] * ilength;
    } else {
        // bk0101022 - FPE related
        _VectorClear(out);
    }

    length
}

pub fn _VectorMA(veca: &vec3_t, scale: f32, vecb: &vec3_t, vecc: &mut vec3_t) {
    vecc[0] = veca[0] + scale * vecb[0];
    vecc[1] = veca[1] + scale * vecb[1];
    vecc[2] = veca[2] + scale * vecb[2];
}

pub fn _DotProduct(v1: &vec3_t, v2: &vec3_t) -> vec_t {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

pub fn _VectorSubtract(veca: &vec3_t, vecb: &vec3_t, out: &mut vec3_t) {
    out[0] = veca[0] - vecb[0];
    out[1] = veca[1] - vecb[1];
    out[2] = veca[2] - vecb[2];
}

pub fn _VectorAdd(veca: &vec3_t, vecb: &vec3_t, out: &mut vec3_t) {
    out[0] = veca[0] + vecb[0];
    out[1] = veca[1] + vecb[1];
    out[2] = veca[2] + vecb[2];
}

pub fn _VectorCopy(in_vec: &vec3_t, out: &mut vec3_t) {
    out[0] = in_vec[0];
    out[1] = in_vec[1];
    out[2] = in_vec[2];
}

pub fn _VectorClear(v: &mut vec3_t) {
    v[0] = 0.0;
    v[1] = 0.0;
    v[2] = 0.0;
}

pub fn _VectorScale(in_vec: &vec3_t, scale: vec_t, out: &mut vec3_t) {
    out[0] = in_vec[0] * scale;
    out[1] = in_vec[1] * scale;
    out[2] = in_vec[2] * scale;
}

pub fn Vector4Scale(in_vec: &vec4_t, scale: vec_t, out: &mut vec4_t) {
    out[0] = in_vec[0] * scale;
    out[1] = in_vec[1] * scale;
    out[2] = in_vec[2] * scale;
    out[3] = in_vec[3] * scale;
}

pub fn Q_log2(mut val: c_int) -> c_int {
    let mut answer: c_int;

    answer = 0;
    while {
        val >>= 1;
        val != 0
    } {
        answer += 1;
    }
    answer
}

/*
=================
PlaneTypeForNormal
=================
*/
/*
int	PlaneTypeForNormal (vec3_t normal) {
    if ( normal[0] == 1.0 )
        return PLANE_X;
    if ( normal[1] == 1.0 )
        return PLANE_Y;
    if ( normal[2] == 1.0 )
        return PLANE_Z;

    return PLANE_NON_AXIAL;
}
*/

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

pub fn AngleVectors(angles: &vec3_t, forward: &mut vec3_t, right: &mut vec3_t, up: &mut vec3_t) {
    let angle: f32;
    let sr: f32;
    let sp: f32;
    let sy: f32;
    let cr: f32;
    let cp: f32;
    let cy: f32;
    // static to help MS compiler fp bugs

    let angle = angles[YAW] * (M_PI * 2.0 / 360.0);
    let sy = angle.sin();
    let cy = angle.cos();
    let angle = angles[PITCH] * (M_PI * 2.0 / 360.0);
    let sp = angle.sin();
    let cp = angle.cos();
    let angle = angles[ROLL] * (M_PI * 2.0 / 360.0);
    let sr = angle.sin();
    let cr = angle.cos();

    forward[0] = cp * cy;
    forward[1] = cp * sy;
    forward[2] = -sp;

    right[0] = (-1.0 * sr * sp * cy + -1.0 * cr * -sy);
    right[1] = (-1.0 * sr * sp * sy + -1.0 * cr * cy);
    right[2] = -1.0 * sr * cp;

    up[0] = (cr * sp * cy + -sr * -sy);
    up[1] = (cr * sp * sy + -sr * cy);
    up[2] = cr * cp;
}

/*
** assumes "src" is normalized
*/
pub fn PerpendicularVector(dst: &mut vec3_t, src: &vec3_t) {
    let pos: usize;
    let mut i: c_int;
    let mut minelem: f32 = 1.0f32;
    let mut tempvec: vec3_t = [0.0; 3];

    /*
    ** find the smallest magnitude axially aligned vector
    */
    let mut pos: usize = 0;
    i = 0;
    while i < 3 {
        if src[i as usize].abs() < minelem {
            pos = i as usize;
            minelem = src[i as usize].abs();
        }
        i += 1;
    }
    tempvec[0] = 0.0f32;
    tempvec[1] = 0.0f32;
    tempvec[2] = 0.0f32;
    tempvec[pos] = 1.0f32;

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
** NormalToLatLong
**
** We use two byte encoded normals in some space critical applications.
** Lat = 0 at (1,0,0) to 360 (-1,0,0), encoded in 8-bit sine table format
** Lng = 0 at (0,0,1) to 180 (0,0,-1), encoded in 8-bit sine table format
**
*/
//rwwRMG - added
pub fn NormalToLatLong(normal: &vec3_t, bytes: &mut [u8; 2]) {
    // check for singularities
    if normal[0] == 0.0 && normal[1] == 0.0 {
        if normal[2] > 0.0f32 {
            bytes[0] = 0;
            bytes[1] = 0; // lat = 0, long = 0
        } else {
            bytes[0] = 128;
            bytes[1] = 0; // lat = 0, long = 128
        }
    } else {
        let a: c_int;
        let b: c_int;

        let a = ((RAD2DEG((normal[1].atan2(normal[0])) as vec_t) * (255.0f32 / 360.0f32)) as c_int);
        let a = a & 0xff;

        let b = ((RAD2DEG((normal[2].acos()) as vec_t) * (255.0f32 / 360.0f32)) as c_int);
        let b = b & 0xff;

        bytes[0] = b as u8; // longitude
        bytes[1] = a as u8; // lattitude
    }
}

// This is the VC libc version of rand() without multiple seeds per thread or 12 levels
// of subroutine calls.
// Both calls have been designed to minimise the inherent number of float <--> int
// conversions and the additional math required to get the desired value.
// eg the typical tint = (rand() * 255) / 32768
// becomes tint = irand(0, 255)

static mut holdrand: u32 = 0x89abcdef;

pub fn Rand_Init(seed: c_int) {
    unsafe {
        holdrand = seed as u32;
    }
}

// Returns a float min <= x < max (exclusive; will get max - 0.00001; but never max)

pub fn flrand(min: f32, max: f32) -> f32 {
    let result: f32;

    unsafe {
        holdrand = (holdrand.wrapping_mul(214013u32)).wrapping_add(2531011u32);
        let result = (holdrand >> 17) as f32; // 0 - 32767 range
        let result = ((result * (max - min)) / 32768.0f32) + min;

        result
    }
}

pub fn Q_flrand(min: f32, max: f32) -> f32 {
    flrand(min, max)
}

// Returns an integer min <= x <= max (ie inclusive)

pub fn irand(min: c_int, max_in: c_int) -> c_int {
    let result: c_int;
    let mut max: c_int = max_in;

    assert!((max - min) < 32768);

    max += 1;
    unsafe {
        holdrand = (holdrand.wrapping_mul(214013u32)).wrapping_add(2531011u32);
        let result = (holdrand >> 17) as c_int;
        let result = (((result as u32 * (max - min) as u32) >> 15) as c_int) + min;
        result
    }
}

pub fn Q_irand(value1: c_int, value2: c_int) -> c_int {
    irand(value1, value2)
}

pub fn powf_impl(x: f32, y: c_int) -> f32 {
    let r: f32 = x;
    let mut y_iter: c_int = y;
    y_iter -= 1;
    let mut r: f32 = r;
    while y_iter > 0 {
        r = r * r;
        y_iter -= 1;
    }
    r
}

// #ifdef Q3_VM
//rwwRMG - needed for HandleEntityAdjustment
pub fn fmod_impl(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return 0.0;
    }

    let result: i32 = (x / y) as i32;

    x - (result as f64 * y)
}
// #endif // Q3_VM

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

    _DotProduct(&v1, &v2)
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
    let distEnd2From: f32;
    let distEnd2Result: f32;
    let mut theta: f32;
    let cos_theta: f32;
    let mut dot: f32;

    //Find the perpendicular vector to vec from start to end
    _VectorSubtract(from, start, &mut vecStart2From);
    _VectorSubtract(end, start, &mut vecStart2End);

    dot = DotProductNormalize(&vecStart2From, &vecStart2End);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the start point
        _VectorCopy(start, result);
        return 0;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        if VectorLengthSquared(&vecStart2From) < VectorLengthSquared(&vecStart2End) {
            _VectorCopy(from, result);
        } else {
            _VectorCopy(end, result);
        }
        return 0;
    }

    //Try other end
    _VectorSubtract(from, end, &mut vecEnd2From);
    _VectorSubtract(start, end, &mut vecEnd2Start);

    dot = DotProductNormalize(&vecEnd2From, &vecEnd2Start);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the start point
        _VectorCopy(end, result);
        return 0;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        if VectorLengthSquared(&vecEnd2From) < VectorLengthSquared(&vecEnd2Start) {
            _VectorCopy(from, result);
        } else {
            _VectorCopy(end, result);
        }
        return 0;
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
    let distEnd2From = VectorLength(&vecEnd2From); //c
    let cos_theta = DEG2RAD(theta).cos(); //cos(theta)
    let distEnd2Result = cos_theta * distEnd2From; //b

    //Extrapolate to find result
    VectorNormalize(&mut vecEnd2Start);
    _VectorMA(end, distEnd2Result, &vecEnd2Start, result);

    //perpendicular intersection is between the 2 endpoints
    1
}

pub fn G_PointDistFromLineSegment(
    start: &vec3_t,
    end: &vec3_t,
    from: &vec3_t,
) -> f32 {
    let mut vecStart2From: vec3_t = [0.0; 3];
    let mut vecStart2End: vec3_t = [0.0; 3];
    let mut vecEnd2Start: vec3_t = [0.0; 3];
    let mut vecEnd2From: vec3_t = [0.0; 3];
    let mut intersection: vec3_t = [0.0; 3];
    let distEnd2From: f32;
    let distStart2From: f32;
    let distEnd2Result: f32;
    let mut theta: f32;
    let cos_theta: f32;
    let mut dot: f32;

    //Find the perpendicular vector to vec from start to end
    _VectorSubtract(from, start, &mut vecStart2From);
    _VectorSubtract(end, start, &mut vecStart2End);
    _VectorSubtract(from, end, &mut vecEnd2From);
    _VectorSubtract(start, end, &mut vecEnd2Start);

    dot = DotProductNormalize(&vecStart2From, &vecStart2End);

    let distStart2From = Distance(start, from);
    let distEnd2From = Distance(end, from);

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
    let cos_theta = DEG2RAD(theta).cos(); //cos(theta)
    let distEnd2Result = cos_theta * distEnd2From; //b

    //Extrapolate to find result
    VectorNormalize(&mut vecEnd2Start);
    _VectorMA(end, distEnd2Result, &vecEnd2Start, &mut intersection);

    //perpendicular intersection is between the 2 endpoints, return dist to it from from
    Distance(&intersection, from)
}

// Helper functions for macro replacement
#[inline]
pub fn DEG2RAD(x: f32) -> f32 {
    x * M_PI / 180.0
}

#[inline]
pub fn RAD2DEG(x: f32) -> f32 {
    x * 180.0 / M_PI
}

// Stubs for functions that require external definitions
pub fn VectorLength(v: &vec3_t) -> vec_t {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub fn VectorLengthSquared(v: &vec3_t) -> vec_t {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

pub fn Distance(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    _VectorSubtract(p2, p1, &mut v);
    VectorLength(&v)
}
