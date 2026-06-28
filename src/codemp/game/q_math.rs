//! Vector / scalar math — a faithful port of `q_math.c`, compared against OpenJK.
//!
//! Function names mirror the C originals and the original comments are carried over,
//! so the Rust can be diffed against the source. Every function is parity-tested
//! against the extracted C oracle: `cargo test --features oracle`.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)] // global tables keep their C names (vec3_origin, g_color_table, …)

use super::q_shared_h::{
    byte, cplane_t, qboolean, vec3_t, vec4_t, vec_t, DEG2RAD, M_PI, NUMVERTEXNORMALS, PITCH,
    QFALSE, QTRUE, RAD2DEG, ROLL, YAW,
};
use core::ffi::c_int;
use core::sync::atomic::{AtomicU32, Ordering};

pub static vec3_origin: vec3_t = [0.0, 0.0, 0.0];
pub static axisDefault: [vec3_t; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

pub static colorBlack: vec4_t = [0.0, 0.0, 0.0, 1.0];
pub static colorRed: vec4_t = [1.0, 0.0, 0.0, 1.0];
pub static colorGreen: vec4_t = [0.0, 1.0, 0.0, 1.0];
pub static colorBlue: vec4_t = [0.0, 0.0, 1.0, 1.0];
pub static colorYellow: vec4_t = [1.0, 1.0, 0.0, 1.0];
pub static colorMagenta: vec4_t = [1.0, 0.0, 1.0, 1.0];
pub static colorCyan: vec4_t = [0.0, 1.0, 1.0, 1.0];
pub static colorWhite: vec4_t = [1.0, 1.0, 1.0, 1.0];
pub static colorLtGrey: vec4_t = [0.75, 0.75, 0.75, 1.0];
pub static colorMdGrey: vec4_t = [0.5, 0.5, 0.5, 1.0];
pub static colorDkGrey: vec4_t = [0.25, 0.25, 0.25, 1.0];
pub static colorLtBlue: vec4_t = [0.367, 0.261, 0.722, 1.0];
pub static colorDkBlue: vec4_t = [0.199, 0.0, 0.398, 1.0];

pub static g_color_table: [vec4_t; 8] = [
    [0.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [1.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0],
    [0.0, 1.0, 1.0, 1.0],
    [1.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
];

/// `bytedirs` — the 162 quantized unit normals used by [`DirToByte`]/[`ByteToDir`].
/// Transcribed verbatim from q_math.c (the `f` float suffixes are implicit on `vec_t`).
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

//==============================================================

/// `int Q_rand( int *seed )` — linear congruential RNG step. The C relies on
/// signed-int wraparound (UB-ish but two's-complement in practice); `wrapping_*`
/// reproduces it exactly.
pub fn Q_rand(seed: &mut i32) -> i32 {
    *seed = 69069i32.wrapping_mul(*seed).wrapping_add(1);
    *seed
}

/// `float Q_random( int *seed )` — uniform random in [0, 1).
pub fn Q_random(seed: &mut i32) -> f32 {
    (Q_rand(seed) & 0xffff) as f32 / 0x10000 as f32
}

/// `float Q_crandom( int *seed )` — uniform random in (-1, 1). The C computes
/// `2.0 * (Q_random(seed) - 0.5)` in double then narrows to float — matched here.
pub fn Q_crandom(seed: &mut i32) -> f32 {
    (2.0_f64 * (Q_random(seed) as f64 - 0.5)) as f32
}

//==============================================================
// The q_shared.h vector macros, ported as functions. Oracle: the `_`-prefixed
// "just in case you don't want to use the macros" counterparts in q_math.c.
//
// 1:1-completeness ledger: q_math.c's six `_`-prefixed standalone functions
// (q_math.c:1235-1268, the "just in case you don't want to use the macros"
// out-of-line copies declared at q_shared.h:1335-1340). `ported_index.py` reports
// `_DotProduct`/`_VectorAdd`/`_VectorCopy`/`_VectorMA`/`_VectorScale`/
// `_VectorSubtract` as "missing" — FALSE POSITIVES: each is the out-of-line twin
// of the identically-bodied macro, and on the build ABI the macros expand inline
// (q_shared.h:1262-1269, the `#if 1` branch — NOT the `#else` `_`-prefixed
// branch :1280-1285), so the `_`-prefixed symbols are never the consumed form.
// They are ported live under their UNPREFIXED names just below. Every line here
// is a comment; this block adds ZERO live code.
//
//   _DotProduct (q_math.c:1242) -> live as DotProduct, q_math.rs:157
//   pub fn _DotProduct(v1: &vec3_t, v2: &vec3_t) -> vec_t {
//       v1[0]*v2[0] + v1[1]*v2[1] + v1[2]*v2[2]
//   }
//
//   _VectorSubtract (q_math.c:1246) -> live as VectorSubtract, q_math.rs:162
//   pub fn _VectorSubtract(veca: &vec3_t, vecb: &vec3_t, out: &mut vec3_t) {
//       out[0] = veca[0] - vecb[0];
//       out[1] = veca[1] - vecb[1];
//       out[2] = veca[2] - vecb[2];
//   }
//
//   _VectorAdd (q_math.c:1252) -> live as VectorAdd, q_math.rs:169
//   pub fn _VectorAdd(veca: &vec3_t, vecb: &vec3_t, out: &mut vec3_t) {
//       out[0] = veca[0] + vecb[0];
//       out[1] = veca[1] + vecb[1];
//       out[2] = veca[2] + vecb[2];
//   }
//
//   _VectorCopy (q_math.c:1258) -> live as VectorCopy, q_math.rs:190
//   pub fn _VectorCopy(in_: &vec3_t, out: &mut vec3_t) {
//       out[0] = in_[0];
//       out[1] = in_[1];
//       out[2] = in_[2];
//   }
//
//   _VectorScale (q_math.c:1264) -> live as VectorScale, q_math.rs:176
//   pub fn _VectorScale(in_: &vec3_t, scale: vec_t, out: &mut vec3_t) {
//       out[0] = in_[0] * scale;
//       out[1] = in_[1] * scale;
//       out[2] = in_[2] * scale;
//   }
//
//   _VectorMA (q_math.c:1235) -> live as VectorMA, q_math.rs:183
//   pub fn _VectorMA(veca: &vec3_t, scale: vec_t, vecb: &vec3_t, vecc: &mut vec3_t) {
//       vecc[0] = veca[0] + scale * vecb[0];
//       vecc[1] = veca[1] + scale * vecb[1];
//       vecc[2] = veca[2] + scale * vecb[2];
//   }
//==============================================================

/// `DotProduct(x,y)` — dot product `x·y`.
pub fn DotProduct(x: &vec3_t, y: &vec3_t) -> vec_t {
    x[0] * y[0] + x[1] * y[1] + x[2] * y[2]
}

/// `VectorSubtract(a,b,c)` — `c = a - b`.
pub fn VectorSubtract(a: &vec3_t, b: &vec3_t, c: &mut vec3_t) {
    c[0] = a[0] - b[0];
    c[1] = a[1] - b[1];
    c[2] = a[2] - b[2];
}

/// `VectorAdd(a,b,c)` — `c = a + b`.
pub fn VectorAdd(a: &vec3_t, b: &vec3_t, c: &mut vec3_t) {
    c[0] = a[0] + b[0];
    c[1] = a[1] + b[1];
    c[2] = a[2] + b[2];
}

/// `VectorScale(v,s,o)` — `o = v * s`.
pub fn VectorScale(v: &vec3_t, s: vec_t, o: &mut vec3_t) {
    o[0] = v[0] * s;
    o[1] = v[1] * s;
    o[2] = v[2] * s;
}

/// `VectorMA(v,s,b,o)` — multiply-add: `o = v + b*s`.
pub fn VectorMA(v: &vec3_t, s: vec_t, b: &vec3_t, o: &mut vec3_t) {
    o[0] = v[0] + b[0] * s;
    o[1] = v[1] + b[1] * s;
    o[2] = v[2] + b[2] * s;
}

/// `VectorCopy(a,b)` — `b = a`.
pub fn VectorCopy(a: &vec3_t, b: &mut vec3_t) {
    b[0] = a[0];
    b[1] = a[1];
    b[2] = a[2];
}

/// `CrossProduct(v1,v2,cross)` — the inline from q_shared.h.
pub fn CrossProduct(v1: &vec3_t, v2: &vec3_t, cross: &mut vec3_t) {
    cross[0] = v1[1] * v2[2] - v1[2] * v2[1];
    cross[1] = v1[2] * v2[0] - v1[0] * v2[2];
    cross[2] = v1[0] * v2[1] - v1[1] * v2[0];
}

/// `void VectorInverse( vec3_t v )` — negate in place.
pub fn VectorInverse(v: &mut vec3_t) {
    v[0] = -v[0];
    v[1] = -v[1];
    v[2] = -v[2];
}

/// `VectorClear(a)` — set to the zero vector.
pub fn VectorClear(a: &mut vec3_t) {
    a[0] = 0.0;
    a[1] = 0.0;
    a[2] = 0.0;
}

/// `VectorNegate(a,b)` — `b = -a`.
pub fn VectorNegate(a: &vec3_t, b: &mut vec3_t) {
    b[0] = -a[0];
    b[1] = -a[1];
    b[2] = -a[2];
}

/// `VectorSet(v,x,y,z)` — assign components.
pub fn VectorSet(v: &mut vec3_t, x: vec_t, y: vec_t, z: vec_t) {
    v[0] = x;
    v[1] = y;
    v[2] = z;
}

/// `vec_t VectorLength( const vec3_t v )` — Euclidean length.
pub fn VectorLength(v: &vec3_t) -> vec_t {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// `vec_t VectorLengthSquared( const vec3_t v )`.
pub fn VectorLengthSquared(v: &vec3_t) -> vec_t {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// `int VectorCompare( const vec3_t v1, const vec3_t v2 )` — exact equality (1/0).
pub fn VectorCompare(v1: &vec3_t, v2: &vec3_t) -> c_int {
    if v1[0] != v2[0] || v1[1] != v2[1] || v1[2] != v2[2] {
        return 0;
    }
    1
}

/// `vec_t Distance( const vec3_t p1, const vec3_t p2 )`.
pub fn Distance(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    VectorSubtract(p2, p1, &mut v);
    VectorLength(&v)
}

/// `vec_t DistanceSquared( const vec3_t p1, const vec3_t p2 )`.
pub fn DistanceSquared(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];
    VectorSubtract(p2, p1, &mut v);
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// fast vector normalize routine that does not check to make sure
/// that length != 0, nor does it return length, uses rsqrt approximation
pub fn VectorNormalizeFast(v: &mut vec3_t) {
    let ilength = Q_rsqrt(DotProduct(v, v));

    v[0] *= ilength;
    v[1] *= ilength;
    v[2] *= ilength;
}

/// `float LerpAngle (float from, float to, float frac)`.
pub fn LerpAngle(from: f32, mut to: f32, frac: f32) -> f32 {
    if to - from > 180.0 {
        to -= 360.0;
    }
    if to - from < -180.0 {
        to += 360.0;
    }
    from + frac * (to - from)
}

/// `float AngleSubtract( float a1, float a2 )` — always returns a value from -180 to 180.
pub fn AngleSubtract(a1: f32, a2: f32) -> f32 {
    let mut a = a1 - a2;
    a = (a as f64 % 360.0) as f32; // fmod(a,360) — chop it down quickly, then level it out
    while a > 180.0 {
        a -= 360.0;
    }
    while a < -180.0 {
        a += 360.0;
    }
    a
}

/// `void AnglesSubtract( vec3_t v1, vec3_t v2, vec3_t v3 )`.
pub fn AnglesSubtract(v1: &vec3_t, v2: &vec3_t, v3: &mut vec3_t) {
    v3[0] = AngleSubtract(v1[0], v2[0]);
    v3[1] = AngleSubtract(v1[1], v2[1]);
    v3[2] = AngleSubtract(v1[2], v2[2]);
}

/// `float AngleMod(float a)`. C computes `(360.0/65536) * ((int)(a*(65536/360.0)) & 65535)`
/// in double precision (note `(int)` truncates toward zero) then narrows to float.
pub fn AngleMod(a: f32) -> f32 {
    let masked = (a as f64 * (65536.0 / 360.0)) as i32 & 65535;
    ((360.0 / 65536.0) * masked as f64) as f32
}

/// `float AngleNormalize360 ( float angle )` — normalized to [0 <= angle < 360].
/// (Same expression as [`AngleMod`].)
pub fn AngleNormalize360(angle: f32) -> f32 {
    let masked = (angle as f64 * (65536.0 / 360.0)) as i32 & 65535;
    ((360.0 / 65536.0) * masked as f64) as f32
}

/// `float AngleNormalize180 ( float angle )` — normalized to [-180 < angle <= 180].
pub fn AngleNormalize180(angle: f32) -> f32 {
    let mut angle = AngleNormalize360(angle);
    if angle > 180.0 {
        angle -= 360.0;
    }
    angle
}

/// `float AngleDelta ( float angle1, float angle2 )` — normalized delta angle1→angle2.
pub fn AngleDelta(angle1: f32, angle2: f32) -> f32 {
    AngleNormalize180(angle1 - angle2)
}

/// `void vectoangles( const vec3_t value1, vec3_t angles )` — direction → euler angles.
/// `atan2`/`sqrt` are computed in double then narrowed to float, as in the C.
pub fn vectoangles(value1: &vec3_t, angles: &mut vec3_t) {
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
            yaw = ((value1[1] as f64).atan2(value1[0] as f64) * 180.0 / (M_PI as f64)) as f32;
        } else if value1[1] > 0.0 {
            yaw = 90.0;
        } else {
            yaw = 270.0;
        }
        if yaw < 0.0 {
            yaw += 360.0;
        }

        let forward = (value1[0] * value1[0] + value1[1] * value1[1]).sqrt();
        pitch = ((value1[2] as f64).atan2(forward as f64) * 180.0 / (M_PI as f64)) as f32;
        if pitch < 0.0 {
            pitch += 360.0;
        }
    }

    angles[PITCH] = -pitch;
    angles[YAW] = yaw;
    angles[ROLL] = 0.0;
}

/// `void AngleVectors( const vec3_t angles, vec3_t forward, vec3_t right, vec3_t up )` —
/// euler angles → orthonormal basis. Any output may be `None`. Trig is done in double
/// (matching C's `sin`/`cos`) then narrowed to float; the per-component expressions are
/// kept in the original operation order so results stay bit-exact.
pub fn AngleVectors(
    angles: &vec3_t,
    forward: Option<&mut vec3_t>,
    right: Option<&mut vec3_t>,
    up: Option<&mut vec3_t>,
) {
    let mut angle: f32;

    angle = angles[YAW] * (M_PI * 2.0 / 360.0);
    let sy = (angle as f64).sin() as f32;
    let cy = (angle as f64).cos() as f32;
    angle = angles[PITCH] * (M_PI * 2.0 / 360.0);
    let sp = (angle as f64).sin() as f32;
    let cp = (angle as f64).cos() as f32;
    angle = angles[ROLL] * (M_PI * 2.0 / 360.0);
    let sr = (angle as f64).sin() as f32;
    let cr = (angle as f64).cos() as f32;

    if let Some(forward) = forward {
        forward[0] = cp * cy;
        forward[1] = cp * sy;
        forward[2] = -sp;
    }
    if let Some(right) = right {
        right[0] = -1.0 * sr * sp * cy + -1.0 * cr * -sy;
        right[1] = -1.0 * sr * sp * sy + -1.0 * cr * cy;
        right[2] = -1.0 * sr * cp;
    }
    if let Some(up) = up {
        up[0] = cr * sp * cy + -sr * -sy;
        up[1] = cr * sp * sy + -sr * cy;
        up[2] = cr * cp;
    }
}

/// `void ProjectPointOnPlane( vec3_t dst, const vec3_t p, const vec3_t normal )`.
pub fn ProjectPointOnPlane(dst: &mut vec3_t, p: &vec3_t, normal: &vec3_t) {
    let mut inv_denom = DotProduct(normal, normal);
    debug_assert!(Q_fabs(inv_denom) != 0.0); // bk010122 - zero vectors get here
    inv_denom = 1.0 / inv_denom;

    let d = DotProduct(normal, p) * inv_denom;

    let mut n: vec3_t = [0.0; 3];
    n[0] = normal[0] * inv_denom;
    n[1] = normal[1] * inv_denom;
    n[2] = normal[2] * inv_denom;

    dst[0] = p[0] - d * n[0];
    dst[1] = p[1] - d * n[1];
    dst[2] = p[2] - d * n[2];
}

/// `void AnglesToAxis( const vec3_t angles, vec3_t axis[3] )`.
pub fn AnglesToAxis(angles: &vec3_t, axis: &mut [vec3_t; 3]) {
    let mut right: vec3_t = [0.0; 3];
    let [a0, a1, a2] = axis;
    // angle vectors returns "right" instead of "y axis"
    AngleVectors(angles, Some(a0), Some(&mut right), Some(a2));
    VectorSubtract(&vec3_origin, &right, a1);
}

/// `void AxisClear( vec3_t axis[3] )`.
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

/// `void AxisCopy( vec3_t in[3], vec3_t out[3] )`.
pub fn AxisCopy(in_: &[vec3_t; 3], out: &mut [vec3_t; 3]) {
    VectorCopy(&in_[0], &mut out[0]);
    VectorCopy(&in_[1], &mut out[1]);
    VectorCopy(&in_[2], &mut out[2]);
}

/// `void MakeNormalVectors( const vec3_t forward, vec3_t right, vec3_t up )`.
/// Given a normalized forward vector, create two other perpendicular vectors.
pub fn MakeNormalVectors(forward: &vec3_t, right: &mut vec3_t, up: &mut vec3_t) {
    // this rotate and negate guarantees a vector
    // not colinear with the original
    right[1] = -forward[0];
    right[2] = forward[1];
    right[0] = forward[2];

    let d = DotProduct(right, forward);
    // VectorMA (right, -d, forward, right) — inlined (right aliases input & output)
    right[0] = right[0] + forward[0] * -d;
    right[1] = right[1] + forward[1] * -d;
    right[2] = right[2] + forward[2] * -d;
    VectorNormalize(right);
    CrossProduct(right, forward, up);
}

/// `void VectorRotate( vec3_t in, vec3_t matrix[3], vec3_t out )`.
pub fn VectorRotate(in_: &vec3_t, matrix: &[vec3_t; 3], out: &mut vec3_t) {
    out[0] = DotProduct(in_, &matrix[0]);
    out[1] = DotProduct(in_, &matrix[1]);
    out[2] = DotProduct(in_, &matrix[2]);
}

/// `void PerpendicularVector( vec3_t dst, const vec3_t src )` — assumes `src` is normalized.
pub fn PerpendicularVector(dst: &mut vec3_t, src: &vec3_t) {
    let mut pos = 0usize;
    let mut minelem = 1.0f32;
    let mut tempvec: vec3_t = [0.0; 3];

    // find the smallest magnitude axially aligned vector
    for i in 0..3 {
        if (src[i] as f64).abs() < minelem as f64 {
            pos = i;
            minelem = (src[i] as f64).abs() as f32;
        }
    }
    tempvec[0] = 0.0;
    tempvec[1] = 0.0;
    tempvec[2] = 0.0;
    tempvec[pos] = 1.0;

    // project the point onto the plane defined by src
    ProjectPointOnPlane(dst, &tempvec, src);

    // normalize the result
    VectorNormalize(dst);
}

/// `float Q_rsqrt( float number )` — fast inverse square root.
///
/// The original used `long i; i = *(long*)&y; ...` — strict-aliasing UB, and on a
/// 64-bit build `long` is 64-bit so the bit-hack reads 8 bytes (garbage). OpenJK
/// fixed this with the `byteAlias_t` type-pun union; we mirror that fix here with
/// `f32::to_bits`/`from_bits` (the exact safe equivalent). See roadmap stage-2/01.
pub fn Q_rsqrt(number: f32) -> f32 {
    let threehalfs: f32 = 1.5;

    let x2 = number * 0.5;
    let mut y;
    let mut i = number.to_bits() as i32; // evil floating point bit level hacking
    i = 0x5f3759df - (i >> 1); // what the fuck?
    y = f32::from_bits(i as u32);
    y = y * (threehalfs - (x2 * y * y)); // 1st iteration
                                         // y  = y * ( threehalfs - ( x2 * y * y ) );   // 2nd iteration, this can be removed

    // Original: #ifndef Q3_VM / #ifdef __linux__  assert( !isnan(y) ); // bk010122 - FPE?
    // (OpenJK made this unconditional via assert(!Q_isnan(y)); debug_assert is the
    // Rust equivalent and does not affect the returned value.)
    debug_assert!(!y.is_nan());
    y
}

/// `float Q_fabs( float f )` — absolute value by clearing the sign bit.
///
/// Original used `int tmp = *(int*)&f; tmp &= 0x7FFFFFFF; ...` (strict-aliasing UB);
/// mirrors OpenJK's `byteAlias_t` fix via `to_bits`/`from_bits`.
pub fn Q_fabs(f: f32) -> f32 {
    let tmp = f.to_bits() & 0x7FFFFFFF;
    f32::from_bits(tmp)
}

/// `vec_t VectorNormalize( vec3_t v )` — normalize in place; returns the OLD length.
///
/// Pure float math, identical in original JKA and OpenJK. The C uses double `sqrt`
/// narrowed to float; that equals `f32::sqrt` for every input (sqrt double-rounding
/// is safe since f64's 53-bit mantissa ≥ 2·24+2), so the port stays bit-exact.
pub fn VectorNormalize(v: &mut vec3_t) -> vec_t {
    let mut length: f32;
    let ilength: f32;

    length = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    length = length.sqrt();

    if length != 0.0 {
        ilength = 1.0 / length;
        v[0] *= ilength;
        v[1] *= ilength;
        v[2] *= ilength;
    }

    length
}

/// `vec_t VectorNormalize2( const vec3_t v, vec3_t out )` — normalize `v` into `out`;
/// returns the length. Zero-length clears `out` (the original's `VectorClear`).
pub fn VectorNormalize2(v: &vec3_t, out: &mut vec3_t) -> vec_t {
    let mut length: f32;
    let ilength: f32;

    length = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    length = length.sqrt();

    if length != 0.0 {
        // bk0101022 - FPE related (original guarded by #ifndef Q3_VM, kept commented out):
        //	  assert( ((Q_fabs(v[0])!=0.0f) || (Q_fabs(v[1])!=0.0f) || (Q_fabs(v[2])!=0.0f)) );
        ilength = 1.0 / length;
        out[0] = v[0] * ilength;
        out[1] = v[1] * ilength;
        out[2] = v[2] * ilength;
    } else {
        // bk0101022 - FPE related (original guarded by #ifndef Q3_VM, kept commented out):
        //	  assert( ((Q_fabs(v[0])==0.0f) && (Q_fabs(v[1])==0.0f) && (Q_fabs(v[2])==0.0f)) );
        // VectorClear( out );
        out[0] = 0.0;
        out[1] = 0.0;
        out[2] = 0.0;
    }

    length
}

//==============================================================
// bounds + horizontal distance

/// `float RadiusFromBounds( const vec3_t mins, const vec3_t maxs )` — radius of the
/// sphere that encloses the AABB. `fabs` on a float promotes to double in C, but for
/// absolute value that is bit-exact with the f32 `.abs()`.
pub fn RadiusFromBounds(mins: &vec3_t, maxs: &vec3_t) -> f32 {
    let mut corner: vec3_t = [0.0; 3];

    for i in 0..3 {
        let a = mins[i].abs();
        let b = maxs[i].abs();
        corner[i] = if a > b { a } else { b };
    }

    VectorLength(&corner)
}

/// `void ClearBounds( vec3_t mins, vec3_t maxs )` — set an "inside out" AABB so the
/// first [`AddPointToBounds`] grows it from nothing.
pub fn ClearBounds(mins: &mut vec3_t, maxs: &mut vec3_t) {
    mins[0] = 99999.0;
    mins[1] = 99999.0;
    mins[2] = 99999.0;
    maxs[0] = -99999.0;
    maxs[1] = -99999.0;
    maxs[2] = -99999.0;
}

/// `vec_t DistanceHorizontal( const vec3_t p1, const vec3_t p2 )` — XY-plane distance.
pub fn DistanceHorizontal(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];

    VectorSubtract(p2, p1, &mut v);
    (v[0] * v[0] + v[1] * v[1]).sqrt() // Leave off the z component
}

/// `vec_t DistanceHorizontalSquared( const vec3_t p1, const vec3_t p2 )`.
pub fn DistanceHorizontalSquared(p1: &vec3_t, p2: &vec3_t) -> vec_t {
    let mut v: vec3_t = [0.0; 3];

    VectorSubtract(p2, p1, &mut v);
    v[0] * v[0] + v[1] * v[1] // Leave off the z component
}

/// `void AddPointToBounds( const vec3_t v, vec3_t mins, vec3_t maxs )` — grow the AABB
/// to include `v`.
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

//==============================================================
// color

/// `unsigned ColorBytes3 (float r, float g, float b)` — pack three [0,1] color
/// components into the low three bytes of a `u32`, matching the C's
/// `((byte*)&i)[k] = c*255` writes (`to_ne_bytes`/`from_ne_bytes` reproduces the exact
/// memory order on any endianness; the float→byte cast truncates toward zero as C does).
///
/// DEVIATION: the C declares `unsigned i;` and writes only 3 bytes, so its top byte is
/// indeterminate (uninitialized stack). We zero it for determinism; the parity test
/// ignores that byte.
pub fn ColorBytes3(r: f32, g: f32, b: f32) -> u32 {
    let mut i = [0u8; 4];
    i[0] = (r * 255.0) as u8;
    i[1] = (g * 255.0) as u8;
    i[2] = (b * 255.0) as u8;
    u32::from_ne_bytes(i)
}

/// `unsigned ColorBytes4 (float r, float g, float b, float a)` — pack four [0,1]
/// components into a `u32` (all four bytes written).
pub fn ColorBytes4(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let mut i = [0u8; 4];
    i[0] = (r * 255.0) as u8;
    i[1] = (g * 255.0) as u8;
    i[2] = (b * 255.0) as u8;
    i[3] = (a * 255.0) as u8;
    u32::from_ne_bytes(i)
}

/// `float NormalizeColor( const vec3_t in, vec3_t out )` — scale a color so its largest
/// component is 1; returns that max. A zero max clears `out`.
pub fn NormalizeColor(in_: &vec3_t, out: &mut vec3_t) -> f32 {
    let mut max = in_[0];
    if in_[1] > max {
        max = in_[1];
    }
    if in_[2] > max {
        max = in_[2];
    }

    if max == 0.0 {
        VectorClear(out);
    } else {
        out[0] = in_[0] / max;
        out[1] = in_[1] / max;
        out[2] = in_[2] / max;
    }
    max
}

//==============================================================
// clamps, Q_log2, Vector4Scale

/// `signed char ClampChar( int i )` — saturate an int to the signed-byte range.
pub fn ClampChar(i: c_int) -> i8 {
    if i < -128 {
        return -128;
    }
    if i > 127 {
        return 127;
    }
    i as i8
}

/// `signed short ClampShort( int i )` — saturate an int to the signed-short range.
pub fn ClampShort(i: c_int) -> i16 {
    if i < -32768 {
        return -32768;
    }
    if i > 0x7fff {
        return 0x7fff;
    }
    i as i16
}

/// `void Vector4Scale( const vec4_t in, vec_t scale, vec4_t out )`.
pub fn Vector4Scale(in_: &vec4_t, scale: vec_t, out: &mut vec4_t) {
    out[0] = in_[0] * scale;
    out[1] = in_[1] * scale;
    out[2] = in_[2] * scale;
    out[3] = in_[3] * scale;
}

/// `int Q_log2( int val )` — floor(log2(val)) via repeated right shift. (`val>>=1` is an
/// arithmetic shift on signed ints, so negative inputs loop forever, exactly as in C.)
pub fn Q_log2(mut val: c_int) -> c_int {
    let mut answer = 0;
    while {
        val >>= 1;
        val != 0
    } {
        answer += 1;
    }
    answer
}

//==============================================================
// quantized normals (bytedirs)

/// `int DirToByte( vec3_t dir )` — index of the closest precomputed unit normal.
/// this isn't a real cheap function to call!
///
/// The C's `if ( !dir ) return 0;` null-guard is subsumed by the non-null `&vec3_t`.
pub fn DirToByte(dir: &vec3_t) -> c_int {
    let mut bestd: f32 = 0.0;
    let mut best: c_int = 0;
    for i in 0..NUMVERTEXNORMALS {
        let d = DotProduct(dir, &bytedirs[i]);
        if d > bestd {
            bestd = d;
            best = i as c_int;
        }
    }

    best
}

/// `void ByteToDir( int b, vec3_t dir )` — inverse of [`DirToByte`]; out-of-range → origin.
pub fn ByteToDir(b: c_int, dir: &mut vec3_t) {
    if b < 0 || b >= NUMVERTEXNORMALS as c_int {
        VectorCopy(&vec3_origin, dir);
        return;
    }
    VectorCopy(&bytedirs[b as usize], dir);
}

//==============================================================
// matrices + rotation

/// `void MatrixMultiply(float in1[3][3], float in2[3][3], float out[3][3])` — 3×3 product.
pub fn MatrixMultiply(in1: &[[vec_t; 3]; 3], in2: &[[vec_t; 3]; 3], out: &mut [[vec_t; 3]; 3]) {
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

/// `qboolean PlaneFromPoints( vec4_t plane, const vec3_t a, const vec3_t b, const vec3_t c )`.
/// Returns false if the triangle is degenrate.
/// The normal will point out of the clock for clockwise ordered points.
///
/// In the C, `CrossProduct`/`VectorNormalize` operate on `plane` in place (its first 3
/// components); we use a temp normal and copy it back, which is identical — `VectorNormalize`
/// leaves a zero vector unchanged, so the degenerate path matches byte-for-byte (and, as in
/// C, `plane[3]` is left untouched when returning false).
pub fn PlaneFromPoints(plane: &mut vec4_t, a: &vec3_t, b: &vec3_t, c: &vec3_t) -> qboolean {
    let mut d1: vec3_t = [0.0; 3];
    let mut d2: vec3_t = [0.0; 3];

    VectorSubtract(b, a, &mut d1);
    VectorSubtract(c, a, &mut d2);
    let mut normal: vec3_t = [0.0; 3];
    CrossProduct(&d2, &d1, &mut normal);
    let length = VectorNormalize(&mut normal);
    plane[0] = normal[0];
    plane[1] = normal[1];
    plane[2] = normal[2];
    if length == 0.0 {
        return QFALSE;
    }

    plane[3] = DotProduct(a, &normal);
    QTRUE
}

/// `void RotatePointAroundVector( vec3_t dst, const vec3_t dir, const vec3_t point, float degrees )`.
/// This is not implemented very well...
pub fn RotatePointAroundVector(dst: &mut vec3_t, dir: &vec3_t, point: &vec3_t, degrees: f32) {
    let mut m = [[0.0f32; 3]; 3];
    let mut im;
    let mut zrot = [[0.0f32; 3]; 3];
    let mut tmpmat = [[0.0f32; 3]; 3];
    let mut rot = [[0.0f32; 3]; 3];
    let mut vr: vec3_t = [0.0; 3];
    let mut vup: vec3_t = [0.0; 3];
    let mut vf: vec3_t = [0.0; 3];

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

    im = m; // memcpy( im, m, sizeof( im ) )

    im[0][1] = m[1][0];
    im[0][2] = m[2][0];
    im[1][0] = m[0][1];
    im[1][2] = m[2][1];
    im[2][0] = m[0][2];
    im[2][1] = m[1][2];

    // memset( zrot, 0, sizeof( zrot ) ) — zrot already zero-initialized above
    zrot[0][0] = 1.0;
    zrot[1][1] = 1.0;
    zrot[2][2] = 1.0;

    let rad = DEG2RAD(degrees);
    zrot[0][0] = (rad as f64).cos() as f32;
    zrot[0][1] = (rad as f64).sin() as f32;
    zrot[1][0] = -((rad as f64).sin() as f32);
    zrot[1][1] = (rad as f64).cos() as f32;

    MatrixMultiply(&m, &zrot, &mut tmpmat);
    MatrixMultiply(&tmpmat, &im, &mut rot);

    for i in 0..3 {
        dst[i] = rot[i][0] * point[0] + rot[i][1] * point[1] + rot[i][2] * point[2];
    }
}

/// `void RotateAroundDirection( vec3_t axis[3], float yaw )`.
pub fn RotateAroundDirection(axis: &mut [vec3_t; 3], yaw: f32) {
    let [a0, a1, a2] = axis;

    // create an arbitrary axis[1]
    PerpendicularVector(a1, a0);

    // rotate it around axis[0] by yaw
    if yaw != 0.0 {
        let mut temp: vec3_t = [0.0; 3];

        VectorCopy(a1, &mut temp);
        RotatePointAroundVector(a1, a0, &temp, yaw);
    }

    // cross to get axis[2]
    CrossProduct(a0, a1, a2);
}

//==============================================================
// NormalToLatLong

/// `void NormalToLatLong( const vec3_t normal, byte bytes[2] )` — pack a unit normal
/// into two bytes (sine-table encoded lat/long).
///
/// We use two byte encoded normals in some space critical applications.
/// Lat = 0 at (1,0,0) to 360 (-1,0,0), encoded in 8-bit sine table format.
/// Lng = 0 at (0,0,1) to 180 (0,0,-1), encoded in 8-bit sine table format.
///
/// `atan2`/`acos` run in double (C libm) then narrow to `vec_t` before `RAD2DEG`,
/// matching the `(vec_t)atan2(...)` casts in the C.
pub fn NormalToLatLong(normal: &vec3_t, bytes: &mut [byte; 2]) {
    // check for singularities
    if normal[0] == 0.0 && normal[1] == 0.0 {
        if normal[2] > 0.0 {
            bytes[0] = 0;
            bytes[1] = 0; // lat = 0, long = 0
        } else {
            bytes[0] = 128;
            bytes[1] = 0; // lat = 0, long = 128
        }
    } else {
        let a = (RAD2DEG((normal[1] as f64).atan2(normal[0] as f64) as vec_t) * (255.0 / 360.0))
            as c_int
            & 0xff;

        let b = (RAD2DEG((normal[2] as f64).acos() as vec_t) * (255.0 / 360.0)) as c_int & 0xff;

        bytes[0] = b as byte; // longitude
        bytes[1] = a as byte; // lattitude
    }
}

//==============================================================
// Rand_Init / flrand / irand
//
// This is the VC libc version of rand() without multiple seeds per thread or 12 levels
// of subroutine calls. Both calls minimise float<->int conversions.

/// `holdrand` — RNG state shared by [`Rand_Init`]/[`flrand`]/[`irand`].
///
/// DEVIATION: the C declares `static unsigned long holdrand`. On an LP64 target
/// `unsigned long` is 64-bit, so `holdrand * 214013` would wrap at 64 bits and the
/// stream would differ from a 32-bit build. We mirror OpenJK's `uint32_t holdrand` fix
/// (a `u32` with wrapping arithmetic) so the sequence is deterministic and
/// 32/64-consistent. `AtomicU32` provides the global mutable state safely.
static holdrand: AtomicU32 = AtomicU32::new(0x89abcdef);

/// `void Rand_Init(int seed)`.
pub fn Rand_Init(seed: c_int) {
    holdrand.store(seed as u32, Ordering::Relaxed);
}

/// `float flrand(float min, float max)` — returns min <= x < max (exclusive; will get
/// max - 0.00001; but never max). The C's `assert((max-min) < 32768)` is a `debug_assert!`.
pub fn flrand(min: f32, max: f32) -> f32 {
    debug_assert!((max - min) < 32768.0);

    let h = holdrand
        .load(Ordering::Relaxed)
        .wrapping_mul(214013)
        .wrapping_add(2531011);
    holdrand.store(h, Ordering::Relaxed);
    let result = (h >> 17) as f32; // 0 - 32767 range
    ((result * (max - min)) / 32768.0) + min
}

/// `int irand(int min, int max)` — returns an integer min <= x <= max (ie inclusive).
pub fn irand(min: c_int, mut max: c_int) -> c_int {
    debug_assert!((max - min) < 32768);

    max = max.wrapping_add(1);
    let h = holdrand
        .load(Ordering::Relaxed)
        .wrapping_mul(214013)
        .wrapping_add(2531011);
    holdrand.store(h, Ordering::Relaxed);
    let result = (h >> 17) as c_int;
    (result.wrapping_mul(max - min) >> 15).wrapping_add(min)
}

/// `int Q_irand(int value1, int value2)` — PC moved this rww convenience wrapper out
/// of q_shared.c into q_math.c, where it simply forwards to [`irand`] (the MSVC-style
/// holdrand LCG). The Xbox build had a separate 15-bit `bg_lib::rand`-based body in
/// q_shared.c; PC's stream and `>> 17` shift differ. Faithful PC re-port.
pub fn Q_irand(value1: c_int, value2: c_int) -> c_int {
    irand(value1, value2)
}

/// `float Q_flrand(float min, float max)` — a thin rww convenience wrapper that simply
/// forwards to [`flrand`] (the MSVC-style holdrand LCG). Faithful 1:1 port.
pub fn Q_flrand(min: f32, max: f32) -> f32 {
    flrand(min, max)
}

//==============================================================
// powf

/// `float powf ( float x, int y )` — integer power.
///
/// ⚠ JKA BUG carried over faithfully: `r = r * r` squares each iteration, so for `y >= 3`
/// this returns `x^(2^(y-1))`, not `x^y` (e.g. `powf(x, 3)` yields `x^4`). OpenJK fixes
/// this (renamed `Q_powf`, body `r *= x`); that correction is a Stage 2 candidate and is
/// deliberately NOT applied here. The C also collides with libm `powf`; see roadmap stage-2.
pub fn powf(x: f32, mut y: c_int) -> f32 {
    let mut r = x;
    y -= 1;
    while y > 0 {
        r = r * r;
        y -= 1;
    }
    r
}

//==============================================================
// fmod (Q3_VM libc shim)

/// `double fmod( double x, double y )` — the `#ifdef Q3_VM` libc replacement.
///
/// `//rwwRMG - needed for HandleEntityAdjustment`
///
/// ⚠ NOT a real `fmod`: the C body is `int result = x / y; return x - (result * y);`, so the
/// quotient is **truncated to `int`** before the subtraction (faithful integer truncation,
/// mirrored here by `as i32` then back to `f64`). Guarded `#[cfg(feature = "vm")]` — the Rust
/// mirror of C's `#if defined(Q3_VM)` (see `bg_lib.rs` Group B).
#[cfg(feature = "vm")]
pub fn fmod(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return 0.0;
    }

    let result = (x / y) as i32;

    x - (result as f64 * y)
}

//==============================================================
// DotProductNormalize + line-segment helpers

/// `float DotProductNormalize( const vec3_t inVec1, const vec3_t inVec2 )` — dot of the
/// two vectors after normalizing each (i.e. the cosine of the angle between them).
pub fn DotProductNormalize(inVec1: &vec3_t, inVec2: &vec3_t) -> f32 {
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    VectorNormalize2(inVec1, &mut v1);
    VectorNormalize2(inVec2, &mut v2);

    DotProduct(&v1, &v2)
}

/// `qboolean G_FindClosestPointOnLineSegment( start, end, from, result )` — closest point
/// on segment start→end to `from`. Returns false when the foot of the perpendicular falls
/// outside the segment (result is then clamped to the nearer endpoint).
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

    //Find the perpendicular vector to vec from start to end
    VectorSubtract(from, start, &mut vecStart2From);
    VectorSubtract(end, start, &mut vecStart2End);

    let mut dot = DotProductNormalize(&vecStart2From, &vecStart2End);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the start point
        VectorCopy(start, result);
        return QFALSE;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        if VectorLengthSquared(&vecStart2From) < VectorLengthSquared(&vecStart2End) {
            VectorCopy(from, result);
        } else {
            VectorCopy(end, result);
        }
        return QFALSE;
    }

    //Try other end
    VectorSubtract(from, end, &mut vecEnd2From);
    VectorSubtract(start, end, &mut vecEnd2Start);

    dot = DotProductNormalize(&vecEnd2From, &vecEnd2Start);

    if dot <= 0.0 {
        //The perpendicular would be beyond or through the start point
        VectorCopy(end, result);
        return QFALSE;
    }

    if dot == 1.0 {
        //parallel, closer of 2 points will be the target
        if VectorLengthSquared(&vecEnd2From) < VectorLengthSquared(&vecEnd2Start) {
            VectorCopy(from, result);
        } else {
            VectorCopy(end, result);
        }
        return QFALSE;
    }

    //angle between vecs end2from and end2start, should be between 0 and 90
    let theta = 90.0 * (1.0 - dot); //theta

    //Get length of side from End2Result using sine of theta
    let distEnd2From = VectorLength(&vecEnd2From); //c
    let cos_theta = (DEG2RAD(theta) as f64).cos() as f32; //cos(theta)
    let distEnd2Result = cos_theta * distEnd2From; //b

    //Extrapolate to find result
    VectorNormalize(&mut vecEnd2Start);
    VectorMA(end, distEnd2Result, &vecEnd2Start, result);

    //perpendicular intersection is between the 2 endpoints
    QTRUE
}

/// `float G_PointDistFromLineSegment( start, end, from )` — distance from `from` to the
/// closest point on segment start→end.
pub fn G_PointDistFromLineSegment(start: &vec3_t, end: &vec3_t, from: &vec3_t) -> f32 {
    let mut vecStart2From: vec3_t = [0.0; 3];
    let mut vecStart2End: vec3_t = [0.0; 3];
    let mut vecEnd2Start: vec3_t = [0.0; 3];
    let mut vecEnd2From: vec3_t = [0.0; 3];
    let mut intersection: vec3_t = [0.0; 3];

    //Find the perpendicular vector to vec from start to end
    VectorSubtract(from, start, &mut vecStart2From);
    VectorSubtract(end, start, &mut vecStart2End);
    VectorSubtract(from, end, &mut vecEnd2From);
    VectorSubtract(start, end, &mut vecEnd2Start);

    let mut dot = DotProductNormalize(&vecStart2From, &vecStart2End);

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

    //angle between vecs end2from and end2start, should be between 0 and 90
    let theta = 90.0 * (1.0 - dot); //theta

    //Get length of side from End2Result using sine of theta
    let cos_theta = (DEG2RAD(theta) as f64).cos() as f32; //cos(theta)
    let distEnd2Result = cos_theta * distEnd2From; //b

    //Extrapolate to find result
    VectorNormalize(&mut vecEnd2Start);
    VectorMA(end, distEnd2Result, &vecEnd2Start, &mut intersection);

    //perpendicular intersection is between the 2 endpoints, return dist to it from from
    Distance(&intersection, from)
}

//==============================================================
// SetPlaneSignbits + BoxOnPlaneSide

/// `void SetPlaneSignbits (cplane_t *out)` — precompute the sign-bit lookup used by the
/// fast box on planeside test.
pub fn SetPlaneSignbits(out: &mut cplane_t) {
    // for fast box on planeside test
    let mut bits = 0;
    for j in 0..3 {
        if out.normal[j] < 0.0 {
            bits |= 1 << j;
        }
    }
    out.signbits = bits as byte;
}

/// `int BoxOnPlaneSide (vec3_t emins, vec3_t emaxs, struct cplane_s *p)` — which side(s)
/// of the plane the AABB straddles. Returns 1, 2, or 1 + 2. This is the portable C
/// version (the original `#if __LCC__ || C_ONLY || !id386` branch), not the x86 `__asm`.
pub fn BoxOnPlaneSide(emins: &vec3_t, emaxs: &vec3_t, p: &cplane_t) -> c_int {
    let dist1: f32;
    let dist2: f32;

    // fast axial cases
    if p.r#type < 3 {
        if p.dist <= emins[p.r#type as usize] {
            return 1;
        }
        if p.dist >= emaxs[p.r#type as usize] {
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
            dist1 = 0.0;
            dist2 = 0.0; // shut up compiler
        }
    }

    let mut sides = 0;
    if dist1 >= p.dist {
        sides = 1;
    }
    if dist2 < p.dist {
        sides |= 2;
    }

    sides
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;

    /// A spread of inputs across magnitudes for parity checks.
    fn scalar_samples() -> [f32; 12] {
        [
            1.0,
            2.0,
            3.0,
            4.0,
            0.5,
            0.25,
            100.0,
            0.01,
            1234.5,
            1.0e-3,
            1.0e6,
            core::f32::consts::PI,
        ]
    }

    #[test]
    fn Q_rsqrt_matches_oracle_bit_exact() {
        for n in scalar_samples() {
            let rust = Q_rsqrt(n);
            let c = unsafe { oracle::Q_rsqrt(n) };
            assert_eq!(
                rust.to_bits(),
                c.to_bits(),
                "Q_rsqrt({n}): rust={rust} ({:#x}) c={c} ({:#x})",
                rust.to_bits(),
                c.to_bits()
            );
        }
    }

    #[test]
    fn Q_fabs_matches_oracle_bit_exact() {
        for n in scalar_samples().iter().flat_map(|&n| [n, -n, 0.0, -0.0]) {
            let rust = Q_fabs(n);
            let c = unsafe { oracle::Q_fabs(n) };
            assert_eq!(rust.to_bits(), c.to_bits(), "Q_fabs({n})");
        }
    }

    /// The `#ifdef Q3_VM` `fmod` shim (vm-gated): bit-exact against `jka_fmod`, including the
    /// `y == 0.0 -> 0.0` guard and the intentional integer truncation of the quotient.
    #[cfg(feature = "vm")]
    #[test]
    fn fmod_matches_oracle_bit_exact() {
        let xs: [f64; 9] = [0.0, 1.0, -1.0, 7.5, -7.5, 360.0, 1234.5, 1.0e6, -0.25];
        let ys: [f64; 7] = [0.0, 1.0, -1.0, 2.0, 360.0, 0.5, 3.0];
        for &x in &xs {
            for &y in &ys {
                let rust = super::fmod(x, y);
                let c = unsafe { oracle::jka_fmod(x, y) };
                assert_eq!(rust.to_bits(), c.to_bits(), "fmod({x}, {y})");
            }
        }
    }

    fn vec3_samples() -> [[f32; 3]; 6] {
        [
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0],
            [-1.0, 0.5, 2.0],
            [100.0, 0.0, 0.0],
            [0.001, 0.002, 0.003],
            [3.0, 4.0, 0.0],
        ]
    }

    #[test]
    fn VectorNormalize_matches_oracle_bit_exact() {
        for v in vec3_samples() {
            let mut r = v;
            let rl = VectorNormalize(&mut r);
            let mut c = v;
            let cl = unsafe { oracle::VectorNormalize(c.as_mut_ptr()) };
            assert_eq!(rl.to_bits(), cl.to_bits(), "length for {v:?}");
            for k in 0..3 {
                assert_eq!(r[k].to_bits(), c[k].to_bits(), "component {k} for {v:?}");
            }
        }
    }

    #[test]
    fn vector_ops_match_oracle_bit_exact() {
        let pairs = [
            ([1.0f32, 2.0, 3.0], [4.0f32, 5.0, 6.0]),
            ([-1.5, 0.0, 2.25], [3.0, -4.0, 0.5]),
            ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            ([100.0, -100.0, 0.001], [0.0, 50.0, -0.5]),
        ];
        let s = 2.5f32;
        for (a, b) in pairs {
            assert_eq!(
                DotProduct(&a, &b).to_bits(),
                unsafe { oracle::_DotProduct(a.as_ptr(), b.as_ptr()) }.to_bits()
            );

            let mut ro = [0.0f32; 3];
            let mut co = [0.0f32; 3];

            VectorSubtract(&a, &b, &mut ro);
            unsafe { oracle::_VectorSubtract(a.as_ptr(), b.as_ptr(), co.as_mut_ptr()) };
            assert_eq!(
                ro.map(f32::to_bits),
                co.map(f32::to_bits),
                "Subtract {a:?} {b:?}"
            );

            VectorAdd(&a, &b, &mut ro);
            unsafe { oracle::_VectorAdd(a.as_ptr(), b.as_ptr(), co.as_mut_ptr()) };
            assert_eq!(ro.map(f32::to_bits), co.map(f32::to_bits), "Add");

            VectorScale(&a, s, &mut ro);
            unsafe { oracle::_VectorScale(a.as_ptr(), s, co.as_mut_ptr()) };
            assert_eq!(ro.map(f32::to_bits), co.map(f32::to_bits), "Scale");

            VectorMA(&a, s, &b, &mut ro);
            unsafe { oracle::_VectorMA(a.as_ptr(), s, b.as_ptr(), co.as_mut_ptr()) };
            assert_eq!(ro.map(f32::to_bits), co.map(f32::to_bits), "MA");

            VectorCopy(&a, &mut ro);
            unsafe { oracle::_VectorCopy(a.as_ptr(), co.as_mut_ptr()) };
            assert_eq!(ro.map(f32::to_bits), co.map(f32::to_bits), "Copy");

            CrossProduct(&a, &b, &mut ro);
            unsafe { oracle::CrossProduct(a.as_ptr(), b.as_ptr(), co.as_mut_ptr()) };
            assert_eq!(ro.map(f32::to_bits), co.map(f32::to_bits), "Cross");

            let mut rv = a;
            let mut cv = a;
            VectorInverse(&mut rv);
            unsafe { oracle::VectorInverse(cv.as_mut_ptr()) };
            assert_eq!(rv.map(f32::to_bits), cv.map(f32::to_bits), "Inverse");
        }
    }

    #[test]
    fn length_distance_match_oracle_bit_exact() {
        let pairs = [
            ([1.0f32, 2.0, 3.0], [4.0f32, 5.0, 6.0]),
            ([-1.5, 0.0, 2.25], [3.0, -4.0, 0.5]),
            ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            ([100.0, -100.0, 0.001], [0.0, 50.0, -0.5]),
        ];
        for (a, b) in pairs {
            assert_eq!(
                VectorLength(&a).to_bits(),
                unsafe { oracle::VectorLength(a.as_ptr()) }.to_bits(),
                "VectorLength {a:?}"
            );
            assert_eq!(
                VectorLengthSquared(&a).to_bits(),
                unsafe { oracle::VectorLengthSquared(a.as_ptr()) }.to_bits()
            );
            assert_eq!(VectorCompare(&a, &b), unsafe {
                oracle::VectorCompare(a.as_ptr(), b.as_ptr())
            });
            assert_eq!(VectorCompare(&a, &a), unsafe {
                oracle::VectorCompare(a.as_ptr(), a.as_ptr())
            });
            assert_eq!(
                Distance(&a, &b).to_bits(),
                unsafe { oracle::Distance(a.as_ptr(), b.as_ptr()) }.to_bits(),
                "Distance {a:?} {b:?}"
            );
            assert_eq!(
                DistanceSquared(&a, &b).to_bits(),
                unsafe { oracle::DistanceSquared(a.as_ptr(), b.as_ptr()) }.to_bits()
            );

            let mut rv = a;
            let mut cv = a;
            VectorNormalizeFast(&mut rv);
            unsafe { oracle::VectorNormalizeFast(cv.as_mut_ptr()) };
            assert_eq!(
                rv.map(f32::to_bits),
                cv.map(f32::to_bits),
                "NormalizeFast {a:?}"
            );
        }
    }

    #[test]
    fn angle_fns_match_oracle_bit_exact() {
        let angles = [
            0.0f32, 45.0, 90.0, 180.0, 270.0, 359.0, 360.0, 720.0, -45.0, -180.0, -360.0, 123.456,
            -777.7, 30.0, 1.5,
        ];
        for a in angles {
            assert_eq!(
                AngleMod(a).to_bits(),
                unsafe { oracle::AngleMod(a) }.to_bits(),
                "AngleMod({a})"
            );
            assert_eq!(
                AngleNormalize360(a).to_bits(),
                unsafe { oracle::AngleNormalize360(a) }.to_bits(),
                "AngleNormalize360({a})"
            );
            assert_eq!(
                AngleNormalize180(a).to_bits(),
                unsafe { oracle::AngleNormalize180(a) }.to_bits(),
                "AngleNormalize180({a})"
            );
        }
        for a1 in angles {
            for a2 in angles {
                assert_eq!(
                    AngleSubtract(a1, a2).to_bits(),
                    unsafe { oracle::AngleSubtract(a1, a2) }.to_bits(),
                    "AngleSubtract({a1},{a2})"
                );
                assert_eq!(
                    AngleDelta(a1, a2).to_bits(),
                    unsafe { oracle::AngleDelta(a1, a2) }.to_bits(),
                    "AngleDelta({a1},{a2})"
                );
                // LerpAngle with a few frac values
                for frac in [0.0f32, 0.25, 0.5, 1.0] {
                    assert_eq!(
                        LerpAngle(a1, a2, frac).to_bits(),
                        unsafe { oracle::LerpAngle(a1, a2, frac) }.to_bits(),
                        "LerpAngle({a1},{a2},{frac})"
                    );
                }
            }
        }
        // AnglesSubtract (component-wise)
        let v1 = [10.0f32, 200.0, -50.0];
        let v2 = [350.0f32, 30.0, 40.0];
        let mut r = [0.0f32; 3];
        let mut c = [0.0f32; 3];
        AnglesSubtract(&v1, &v2, &mut r);
        unsafe { oracle::AnglesSubtract(v1.as_ptr(), v2.as_ptr(), c.as_mut_ptr()) };
        assert_eq!(r.map(f32::to_bits), c.map(f32::to_bits));
    }

    #[test]
    fn angle_vectors_and_vectoangles_match_oracle_bit_exact() {
        let angle_sets = [
            [0.0f32, 0.0, 0.0],
            [0.0, 90.0, 0.0],
            [45.0, 45.0, 0.0],
            [30.0, 200.0, 10.0],
            [-15.0, 270.0, 33.0],
            [89.9, 359.9, -45.0],
            [12.34, 56.78, 90.0],
        ];
        for a in angle_sets {
            let (mut rf, mut rr, mut ru) = ([0.0f32; 3], [0.0f32; 3], [0.0f32; 3]);
            let (mut cf, mut cr, mut cu) = ([0.0f32; 3], [0.0f32; 3], [0.0f32; 3]);
            AngleVectors(&a, Some(&mut rf), Some(&mut rr), Some(&mut ru));
            unsafe {
                oracle::AngleVectors(
                    a.as_ptr(),
                    cf.as_mut_ptr(),
                    cr.as_mut_ptr(),
                    cu.as_mut_ptr(),
                )
            };
            assert_eq!(rf.map(f32::to_bits), cf.map(f32::to_bits), "forward {a:?}");
            assert_eq!(rr.map(f32::to_bits), cr.map(f32::to_bits), "right {a:?}");
            assert_eq!(ru.map(f32::to_bits), cu.map(f32::to_bits), "up {a:?}");
        }

        let dirs = [
            [1.0f32, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [1.0, 1.0, 1.0],
            [-3.0, 4.0, -5.0],
            [0.0, 0.0, 0.0],
            [10.0, -2.5, 0.3],
        ];
        for d in dirs {
            let mut ra = [0.0f32; 3];
            let mut ca = [0.0f32; 3];
            vectoangles(&d, &mut ra);
            unsafe { oracle::vectoangles(d.as_ptr(), ca.as_mut_ptr()) };
            assert_eq!(
                ra.map(f32::to_bits),
                ca.map(f32::to_bits),
                "vectoangles {d:?}"
            );
        }
    }

    #[test]
    fn geometry_fns_match_oracle_bit_exact() {
        // helpers to compare axis/matrix ([[f32;3];3]) bit-exact
        fn bits3(m: &[[f32; 3]; 3]) -> [[u32; 3]; 3] {
            m.map(|v| v.map(f32::to_bits))
        }

        let angle_sets = [
            [0.0f32, 0.0, 0.0],
            [45.0, 45.0, 0.0],
            [30.0, 200.0, 10.0],
            [-15.0, 270.0, 33.0],
            [12.34, 56.78, 90.0],
        ];
        for a in angle_sets {
            let mut r = [[0.0f32; 3]; 3];
            let mut c = [[0.0f32; 3]; 3];
            AnglesToAxis(&a, &mut r);
            unsafe { oracle::AnglesToAxis(a.as_ptr(), c.as_mut_ptr() as *mut f32) };
            assert_eq!(bits3(&r), bits3(&c), "AnglesToAxis {a:?}");
        }

        let vecs = [
            [1.0f32, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 2.0, 3.0],
            [-3.0, 4.0, -5.0],
            [0.3, -0.7, 0.65],
        ];
        for v in vecs {
            // ProjectPointOnPlane (use a non-zero normal `n`)
            let n = [0.0f32, 0.0, 1.0];
            let mut rd = [0.0f32; 3];
            let mut cd = [0.0f32; 3];
            ProjectPointOnPlane(&mut rd, &v, &n);
            unsafe { oracle::ProjectPointOnPlane(cd.as_mut_ptr(), v.as_ptr(), n.as_ptr()) };
            assert_eq!(
                rd.map(f32::to_bits),
                cd.map(f32::to_bits),
                "ProjectPointOnPlane {v:?}"
            );

            // MakeNormalVectors (forward must be non-zero)
            let (mut rr, mut ru) = ([0.0f32; 3], [0.0f32; 3]);
            let (mut cr, mut cu) = ([0.0f32; 3], [0.0f32; 3]);
            MakeNormalVectors(&v, &mut rr, &mut ru);
            unsafe { oracle::MakeNormalVectors(v.as_ptr(), cr.as_mut_ptr(), cu.as_mut_ptr()) };
            assert_eq!(
                rr.map(f32::to_bits),
                cr.map(f32::to_bits),
                "MakeNormalVectors right {v:?}"
            );
            assert_eq!(
                ru.map(f32::to_bits),
                cu.map(f32::to_bits),
                "MakeNormalVectors up {v:?}"
            );

            // PerpendicularVector (src non-zero)
            let mut rp = [0.0f32; 3];
            let mut cp = [0.0f32; 3];
            PerpendicularVector(&mut rp, &v);
            unsafe { oracle::PerpendicularVector(cp.as_mut_ptr(), v.as_ptr()) };
            assert_eq!(
                rp.map(f32::to_bits),
                cp.map(f32::to_bits),
                "PerpendicularVector {v:?}"
            );
        }

        // VectorRotate
        let in_ = [1.0f32, 2.0, 3.0];
        let matrix = [[0.0f32, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let mut ro = [0.0f32; 3];
        let mut co = [0.0f32; 3];
        VectorRotate(&in_, &matrix, &mut ro);
        unsafe {
            oracle::VectorRotate(in_.as_ptr(), matrix.as_ptr() as *const f32, co.as_mut_ptr())
        };
        assert_eq!(ro.map(f32::to_bits), co.map(f32::to_bits), "VectorRotate");

        // AxisClear / AxisCopy
        let mut rax = [[7.0f32; 3]; 3];
        let mut cax = [[7.0f32; 3]; 3];
        AxisClear(&mut rax);
        unsafe { oracle::AxisClear(cax.as_mut_ptr() as *mut f32) };
        assert_eq!(bits3(&rax), bits3(&cax), "AxisClear");

        let src = [[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let mut rout = [[0.0f32; 3]; 3];
        let mut cout = [[0.0f32; 3]; 3];
        AxisCopy(&src, &mut rout);
        unsafe { oracle::AxisCopy(src.as_ptr() as *const f32, cout.as_mut_ptr() as *mut f32) };
        assert_eq!(bits3(&rout), bits3(&cout), "AxisCopy");
    }

    #[test]
    fn vector_set_clear_negate() {
        let mut v = [9.0f32; 3];
        VectorClear(&mut v);
        assert_eq!(v, [0.0, 0.0, 0.0]);
        VectorSet(&mut v, 1.0, -2.0, 3.5);
        assert_eq!(v, [1.0, -2.0, 3.5]);
        let mut n = [0.0f32; 3];
        VectorNegate(&v, &mut n);
        assert_eq!(n, [-1.0, 2.0, -3.5]);
    }

    #[test]
    fn rng_matches_oracle_bit_exact() {
        // Same starting seeds; walk the sequence and compare each step + the seed.
        for start in [0i32, 1, 42, -1, 1337, i32::MAX, i32::MIN] {
            let mut rs = start;
            let mut cs = start;
            for _ in 0..32 {
                let r_rand = Q_rand(&mut rs);
                let c_rand = unsafe { oracle::Q_rand(&mut cs) };
                assert_eq!(r_rand, c_rand, "Q_rand seq from {start}");
                assert_eq!(rs, cs, "seed drift from {start}");
            }
            // Q_random / Q_crandom (advance both identically)
            let mut rs = start;
            let mut cs = start;
            for _ in 0..16 {
                let rr = Q_random(&mut rs);
                let cr = unsafe { oracle::Q_random(&mut cs) };
                assert_eq!(rr.to_bits(), cr.to_bits(), "Q_random from {start}");
            }
            let mut rs = start;
            let mut cs = start;
            for _ in 0..16 {
                let rc = Q_crandom(&mut rs);
                let cc = unsafe { oracle::Q_crandom(&mut cs) };
                assert_eq!(rc.to_bits(), cc.to_bits(), "Q_crandom from {start}");
            }
        }
    }

    #[test]
    fn bounds_and_horizontal_dist_match_oracle_bit_exact() {
        let pts = [
            [1.0f32, 2.0, 3.0],
            [-4.0, 5.0, -6.0],
            [0.0, 0.0, 0.0],
            [100.0, -50.0, 0.25],
            [-0.001, 0.002, -0.003],
            [12.5, 12.5, -99.0],
        ];
        for a in pts {
            for b in pts {
                assert_eq!(
                    RadiusFromBounds(&a, &b).to_bits(),
                    unsafe { oracle::RadiusFromBounds(a.as_ptr(), b.as_ptr()) }.to_bits(),
                    "RadiusFromBounds {a:?} {b:?}"
                );
                assert_eq!(
                    DistanceHorizontal(&a, &b).to_bits(),
                    unsafe { oracle::DistanceHorizontal(a.as_ptr(), b.as_ptr()) }.to_bits(),
                    "DistanceHorizontal {a:?} {b:?}"
                );
                assert_eq!(
                    DistanceHorizontalSquared(&a, &b).to_bits(),
                    unsafe { oracle::DistanceHorizontalSquared(a.as_ptr(), b.as_ptr()) }.to_bits(),
                    "DistanceHorizontalSquared {a:?} {b:?}"
                );
            }
        }

        // ClearBounds, then AddPointToBounds over a point cloud — compare to oracle.
        let (mut rmins, mut rmaxs) = ([0.0f32; 3], [0.0f32; 3]);
        let (mut cmins, mut cmaxs) = ([0.0f32; 3], [0.0f32; 3]);
        ClearBounds(&mut rmins, &mut rmaxs);
        unsafe { oracle::ClearBounds(cmins.as_mut_ptr(), cmaxs.as_mut_ptr()) };
        assert_eq!(
            rmins.map(f32::to_bits),
            cmins.map(f32::to_bits),
            "ClearBounds mins"
        );
        assert_eq!(
            rmaxs.map(f32::to_bits),
            cmaxs.map(f32::to_bits),
            "ClearBounds maxs"
        );
        for p in pts {
            AddPointToBounds(&p, &mut rmins, &mut rmaxs);
            unsafe { oracle::AddPointToBounds(p.as_ptr(), cmins.as_mut_ptr(), cmaxs.as_mut_ptr()) };
            assert_eq!(
                rmins.map(f32::to_bits),
                cmins.map(f32::to_bits),
                "AddPointToBounds mins {p:?}"
            );
            assert_eq!(
                rmaxs.map(f32::to_bits),
                cmaxs.map(f32::to_bits),
                "AddPointToBounds maxs {p:?}"
            );
        }
    }

    #[test]
    fn color_fns_match_oracle_bit_exact() {
        let colors = [
            [0.0f32, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.5, 0.25, 0.75],
            [0.367, 0.261, 0.722],
            [0.1, 0.9, 0.4],
            [1.0, 0.0, 0.5],
        ];
        for c in colors {
            // NormalizeColor
            let mut ro = [0.0f32; 3];
            let mut co = [0.0f32; 3];
            let rmax = NormalizeColor(&c, &mut ro);
            let cmax = unsafe { oracle::NormalizeColor(c.as_ptr(), co.as_mut_ptr()) };
            assert_eq!(rmax.to_bits(), cmax.to_bits(), "NormalizeColor max {c:?}");
            assert_eq!(
                ro.map(f32::to_bits),
                co.map(f32::to_bits),
                "NormalizeColor out {c:?}"
            );

            // ColorBytes4 — all four bytes defined, compare full word
            let alphas = [0.0f32, 0.5, 1.0];
            for a in alphas {
                let r = ColorBytes4(c[0], c[1], c[2], a);
                let cc = unsafe { oracle::ColorBytes4(c[0], c[1], c[2], a) };
                assert_eq!(r, cc, "ColorBytes4 {c:?} a={a}");
            }

            // ColorBytes3 — C's 4th byte is indeterminate; compare only the 3 written bytes
            let r3 = ColorBytes3(c[0], c[1], c[2]).to_ne_bytes();
            let c3 = unsafe { oracle::ColorBytes3(c[0], c[1], c[2]) }.to_ne_bytes();
            assert_eq!(r3[0..3], c3[0..3], "ColorBytes3 (low 3 bytes) {c:?}");
        }
    }

    #[test]
    fn clamps_log2_vector4scale_match_oracle_bit_exact() {
        for i in [
            i32::MIN,
            -100000,
            -32769,
            -32768,
            -129,
            -128,
            -1,
            0,
            1,
            127,
            128,
            32767,
            32768,
            100000,
            i32::MAX,
        ] {
            assert_eq!(
                ClampChar(i),
                unsafe { oracle::ClampChar(i) },
                "ClampChar({i})"
            );
            assert_eq!(
                ClampShort(i),
                unsafe { oracle::ClampShort(i) },
                "ClampShort({i})"
            );
        }
        // Q_log2 over non-negative inputs (negative loops forever in both — by design).
        for v in [
            0i32,
            1,
            2,
            3,
            7,
            8,
            15,
            16,
            1023,
            1024,
            0x4000_0000,
            i32::MAX,
        ] {
            assert_eq!(Q_log2(v), unsafe { oracle::Q_log2(v) }, "Q_log2({v})");
        }
        let v4s = [
            [1.0f32, 2.0, 3.0, 4.0],
            [-1.5, 0.0, 2.25, -7.0],
            [0.001, 100.0, -0.5, 12.5],
        ];
        for v in v4s {
            for s in [0.0f32, 1.0, -2.5, 0.333] {
                let mut ro = [0.0f32; 4];
                let mut co = [0.0f32; 4];
                Vector4Scale(&v, s, &mut ro);
                unsafe { oracle::Vector4Scale(v.as_ptr(), s, co.as_mut_ptr()) };
                assert_eq!(
                    ro.map(f32::to_bits),
                    co.map(f32::to_bits),
                    "Vector4Scale {v:?} s={s}"
                );
            }
        }
    }

    #[test]
    fn dir_byte_match_oracle_bit_exact() {
        // ByteToDir over the full index range + a few out-of-range values.
        for b in (-3..(NUMVERTEXNORMALS as c_int + 3)).chain([i32::MIN, i32::MAX]) {
            let mut rd = [9.0f32; 3];
            let mut cd = [9.0f32; 3];
            ByteToDir(b, &mut rd);
            unsafe { oracle::ByteToDir(b, cd.as_mut_ptr()) };
            assert_eq!(rd.map(f32::to_bits), cd.map(f32::to_bits), "ByteToDir({b})");
        }
        // DirToByte: every table entry must map to itself, plus arbitrary directions.
        let mut dirs: Vec<[f32; 3]> = bytedirs.to_vec();
        dirs.extend([
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.3, -0.7, 0.65],
            [-2.0, 5.0, -1.0],
            [0.577, 0.577, 0.577],
        ]);
        for d in dirs {
            assert_eq!(
                DirToByte(&d),
                unsafe { oracle::DirToByte(d.as_ptr()) },
                "DirToByte({d:?})"
            );
        }
    }

    #[test]
    fn matrix_and_rotation_match_oracle_bit_exact() {
        fn bits3(m: &[[f32; 3]; 3]) -> [[u32; 3]; 3] {
            m.map(|v| v.map(f32::to_bits))
        }

        let mats = [
            [[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            [[0.5, -1.5, 2.25], [-3.0, 0.0, 1.0], [10.0, -0.25, 0.1]],
        ];
        for a in mats {
            for b in mats {
                let mut r = [[0.0f32; 3]; 3];
                let mut c = [[0.0f32; 3]; 3];
                MatrixMultiply(&a, &b, &mut r);
                unsafe {
                    oracle::MatrixMultiply(
                        a.as_ptr() as *const f32,
                        b.as_ptr() as *const f32,
                        c.as_mut_ptr() as *mut f32,
                    )
                };
                assert_eq!(bits3(&r), bits3(&c), "MatrixMultiply");
            }
        }

        // PlaneFromPoints — include degenerate (collinear) triangles → qfalse.
        let tris = [
            ([0.0f32, 0.0, 0.0], [1.0f32, 0.0, 0.0], [0.0f32, 1.0, 0.0]),
            ([1.0, 2.0, 3.0], [4.0, 0.0, 1.0], [-2.0, 5.0, 0.0]),
            ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]), // collinear → degenerate
            ([5.0, 5.0, 5.0], [5.0, 5.0, 5.0], [5.0, 5.0, 5.0]), // all equal → degenerate
        ];
        for (a, b, c) in tris {
            let mut rplane = [0.0f32; 4];
            let mut cplane = [0.0f32; 4];
            let rret = PlaneFromPoints(&mut rplane, &a, &b, &c);
            let cret = unsafe {
                oracle::PlaneFromPoints(cplane.as_mut_ptr(), a.as_ptr(), b.as_ptr(), c.as_ptr())
            };
            assert_eq!(rret, cret, "PlaneFromPoints ret {a:?}");
            assert_eq!(
                rplane.map(f32::to_bits),
                cplane.map(f32::to_bits),
                "PlaneFromPoints plane {a:?}"
            );
        }

        // RotatePointAroundVector
        let dirs = [
            [1.0f32, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.577_350_3, 0.577_350_3, 0.577_350_3],
        ];
        let points = [[1.0f32, 0.0, 0.0], [3.0, -2.0, 1.0], [0.0, 5.0, 0.0]];
        for dir in dirs {
            for point in points {
                for deg in [0.0f32, 30.0, 90.0, 180.0, -45.0, 360.0, 12.34] {
                    let mut rd = [0.0f32; 3];
                    let mut cd = [0.0f32; 3];
                    RotatePointAroundVector(&mut rd, &dir, &point, deg);
                    unsafe {
                        oracle::RotatePointAroundVector(
                            cd.as_mut_ptr(),
                            dir.as_ptr(),
                            point.as_ptr(),
                            deg,
                        )
                    };
                    assert_eq!(
                        rd.map(f32::to_bits),
                        cd.map(f32::to_bits),
                        "RotatePointAroundVector dir={dir:?} pt={point:?} deg={deg}"
                    );
                }
            }
        }

        // RotateAroundDirection — seed axis[0], fill the rest.
        for a0 in dirs {
            for yaw in [0.0f32, 25.0, 90.0, -60.0, 200.0] {
                let mut raxis = [a0, [9.0; 3], [9.0; 3]];
                let mut caxis = [a0, [9.0; 3], [9.0; 3]];
                RotateAroundDirection(&mut raxis, yaw);
                unsafe { oracle::RotateAroundDirection(caxis.as_mut_ptr() as *mut f32, yaw) };
                assert_eq!(
                    bits3(&raxis),
                    bits3(&caxis),
                    "RotateAroundDirection a0={a0:?} yaw={yaw}"
                );
            }
        }
    }

    #[test]
    fn normal_to_lat_long_matches_oracle_bit_exact() {
        let normals = [
            [1.0f32, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],  // singularity (+z)
            [0.0, 0.0, -1.0], // singularity (-z)
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.577_350_3, 0.577_350_3, 0.577_350_3],
            [-0.5, 0.5, 0.707_106_8],
            [0.3, -0.7, 0.648_074],
            [0.0, 0.0, 0.0], // both x,y zero, z==0 → singularity else-branch (z>0 false)
        ];
        for n in normals {
            let mut rb = [0u8; 2];
            let mut cb = [0u8; 2];
            NormalToLatLong(&n, &mut rb);
            unsafe { oracle::NormalToLatLong(n.as_ptr(), cb.as_mut_ptr()) };
            assert_eq!(rb, cb, "NormalToLatLong {n:?}");
        }
    }

    #[test]
    fn flrand_irand_match_oracle_bit_exact() {
        // holdrand is global mutable state in both implementations; seed both with
        // Rand_Init and walk in lockstep (each call advances both streams identically).
        for seed in [0i32, 1, 42, -1, 1337, 0x1234_5678, i32::MIN, i32::MAX] {
            Rand_Init(seed);
            unsafe { oracle::Rand_Init(seed) };
            let franges = [
                (0.0f32, 1.0),
                (-1.0, 1.0),
                (0.0, 100.0),
                (-50.0, 50.0),
                (10.0, 20.0),
            ];
            for _ in 0..40 {
                for &(min, max) in &franges {
                    let r = flrand(min, max);
                    let c = unsafe { oracle::flrand(min, max) };
                    assert_eq!(r.to_bits(), c.to_bits(), "flrand({min},{max}) seed={seed}");
                }
            }

            Rand_Init(seed);
            unsafe { oracle::Rand_Init(seed) };
            let iranges = [(0i32, 1), (0, 255), (-10, 10), (1, 100), (-1000, 1000)];
            for _ in 0..40 {
                for &(min, max) in &iranges {
                    let r = irand(min, max);
                    let c = unsafe { oracle::irand(min, max) };
                    assert_eq!(r, c, "irand({min},{max}) seed={seed}");
                }
            }
        }
    }

    #[test]
    fn Q_irand_matches_oracle_bit_exact() {
        // PC moved Q_irand out of q_shared.c into q_math.c, where it is a thin wrapper
        // over irand (the holdrand MSVC LCG). Seed both sides via Rand_Init and walk in
        // lockstep, exactly as the irand test does.
        for seed in [0i32, 1, 42, -1, 1337, 0x1234_5678, i32::MIN, i32::MAX] {
            Rand_Init(seed);
            unsafe { oracle::Rand_Init(seed) };
            let iranges = [(0i32, 1), (0, 255), (-10, 10), (1, 100), (-1000, 1000)];
            for _ in 0..40 {
                for &(min, max) in &iranges {
                    let r = Q_irand(min, max);
                    let c = unsafe { oracle::Q_irand(min, max) };
                    assert_eq!(r, c, "Q_irand({min},{max}) seed={seed}");
                }
            }
        }
    }

    #[test]
    fn Q_flrand_matches_oracle_bit_exact() {
        // Q_flrand is a thin rww wrapper over flrand (the holdrand MSVC LCG). Seed both
        // sides via Rand_Init and walk in lockstep, exactly as the flrand test does.
        for seed in [0i32, 1, 42, -1, 1337, 0x1234_5678, i32::MIN, i32::MAX] {
            Rand_Init(seed);
            unsafe { oracle::Rand_Init(seed) };
            let franges = [
                (0.0f32, 1.0),
                (-1.0, 1.0),
                (0.0, 100.0),
                (-50.0, 50.0),
                (10.0, 20.0),
            ];
            for _ in 0..40 {
                for &(min, max) in &franges {
                    let r = Q_flrand(min, max);
                    let c = unsafe { oracle::Q_flrand(min, max) };
                    assert_eq!(
                        r.to_bits(),
                        c.to_bits(),
                        "Q_flrand({min},{max}) seed={seed}"
                    );
                }
            }
        }
    }

    #[test]
    fn powf_matches_oracle_bit_exact() {
        // Small exponents only — `r=r*r` blows up fast (overflows to inf, still bit-equal).
        for x in [0.0f32, 0.5, 1.0, 1.1, 2.0, -1.5, -0.5, 3.3] {
            for y in [-2i32, -1, 0, 1, 2, 3, 4, 5] {
                let r = powf(x, y);
                let c = unsafe { oracle::jka_powf(x, y) };
                assert_eq!(r.to_bits(), c.to_bits(), "powf({x},{y})");
            }
        }
    }

    #[test]
    fn dotproductnormalize_and_linesegment_match_oracle_bit_exact() {
        let vecs = [
            [1.0f32, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [-3.0, 4.0, -5.0],
            [0.0, 0.0, 0.0],
            [10.0, -2.5, 0.3],
        ];
        for a in vecs {
            for b in vecs {
                assert_eq!(
                    DotProductNormalize(&a, &b).to_bits(),
                    unsafe { oracle::DotProductNormalize(a.as_ptr(), b.as_ptr()) }.to_bits(),
                    "DotProductNormalize {a:?} {b:?}"
                );
            }
        }

        // Line-segment helpers: cover perpendicular-inside, beyond-ends, parallel cases.
        let segs = [
            ([0.0f32, 0.0, 0.0], [10.0f32, 0.0, 0.0]),
            ([1.0, 2.0, 3.0], [4.0, 8.0, -1.0]),
            ([-5.0, -5.0, -5.0], [5.0, 5.0, 5.0]),
        ];
        let froms = [
            [5.0f32, 5.0, 0.0],
            [-3.0, 1.0, 2.0],
            [20.0, 0.0, 0.0],  // beyond end
            [-20.0, 0.0, 0.0], // before start
            [2.0, 4.0, 6.0],   // on the (1,2,3)->(4,8,-1)... arbitrary
            [0.0, 0.0, 0.0],
        ];
        for (start, end) in segs {
            for from in froms {
                let mut rr = [0.0f32; 3];
                let mut cr = [0.0f32; 3];
                let rret = G_FindClosestPointOnLineSegment(&start, &end, &from, &mut rr);
                let cret = unsafe {
                    oracle::G_FindClosestPointOnLineSegment(
                        start.as_ptr(),
                        end.as_ptr(),
                        from.as_ptr(),
                        cr.as_mut_ptr(),
                    )
                };
                assert_eq!(
                    rret, cret,
                    "G_FindClosestPointOnLineSegment ret start={start:?} from={from:?}"
                );
                assert_eq!(
                    rr.map(f32::to_bits),
                    cr.map(f32::to_bits),
                    "G_FindClosestPointOnLineSegment result start={start:?} from={from:?}"
                );

                assert_eq!(
                    G_PointDistFromLineSegment(&start, &end, &from).to_bits(),
                    unsafe {
                        oracle::G_PointDistFromLineSegment(
                            start.as_ptr(),
                            end.as_ptr(),
                            from.as_ptr(),
                        )
                    }
                    .to_bits(),
                    "G_PointDistFromLineSegment start={start:?} from={from:?}"
                );
            }
        }
    }

    #[test]
    fn plane_sidetests_match_oracle_bit_exact() {
        // A spread of planes: 3 axial (type 0/1/2) + the 8 sign combinations of a
        // non-axial normal (type 3) to exercise every signbits case 0..7.
        let mut planes: Vec<cplane_t> = vec![
            cplane_t {
                normal: [1.0, 0.0, 0.0],
                dist: 5.0,
                r#type: 0,
                signbits: 0,
                pad: [0, 0],
            },
            cplane_t {
                normal: [0.0, 1.0, 0.0],
                dist: -3.0,
                r#type: 1,
                signbits: 0,
                pad: [0, 0],
            },
            cplane_t {
                normal: [0.0, 0.0, 1.0],
                dist: 0.0,
                r#type: 2,
                signbits: 0,
                pad: [0, 0],
            },
        ];
        for &sx in &[1.0f32, -1.0] {
            for &sy in &[1.0f32, -1.0] {
                for &sz in &[1.0f32, -1.0] {
                    let mut n = [sx * 0.5, sy * 0.5, sz * 0.707_106_8];
                    VectorNormalize(&mut n);
                    planes.push(cplane_t {
                        normal: n,
                        dist: 2.5,
                        r#type: 3,
                        signbits: 0,
                        pad: [0, 0],
                    });
                }
            }
        }

        let fboxes = [
            ([-1.0f32, -1.0, -1.0], [1.0f32, 1.0, 1.0]),
            ([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
            ([-100.0, -100.0, -100.0], [-50.0, -50.0, -50.0]),
            ([3.0, -2.0, 1.0], [8.0, 4.0, 6.0]),
        ];

        for mut pl in planes {
            // SetPlaneSignbits parity, then use the populated plane downstream.
            let mut cp = pl;
            SetPlaneSignbits(&mut pl);
            unsafe { oracle::SetPlaneSignbits(&mut cp as *mut cplane_t) };
            assert_eq!(
                pl.signbits, cp.signbits,
                "SetPlaneSignbits normal={:?}",
                pl.normal
            );

            for (mins, maxs) in fboxes {
                let r = BoxOnPlaneSide(&mins, &maxs, &pl);
                let c = unsafe {
                    oracle::BoxOnPlaneSide(mins.as_ptr(), maxs.as_ptr(), &pl as *const cplane_t)
                };
                assert_eq!(
                    r, c,
                    "BoxOnPlaneSide normal={:?} type={} box=({mins:?},{maxs:?})",
                    pl.normal, pl.r#type
                );
            }
        }
    }

    #[test]
    fn VectorNormalize2_matches_oracle_bit_exact() {
        for v in vec3_samples() {
            let mut ro = [0.0f32; 3];
            let rl = VectorNormalize2(&v, &mut ro);
            let mut co = [0.0f32; 3];
            let cl = unsafe { oracle::VectorNormalize2(v.as_ptr(), co.as_mut_ptr()) };
            assert_eq!(rl.to_bits(), cl.to_bits(), "length for {v:?}");
            for k in 0..3 {
                assert_eq!(ro[k].to_bits(), co[k].to_bits(), "component {k} for {v:?}");
            }
        }
    }
}
