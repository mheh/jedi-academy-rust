#![allow(non_snake_case)]

use core::ffi::{c_float, c_int};
use std::mem;

//#define DotProduct(a,b)			((a)[0]*(b)[0]+(a)[1]*(b)[1]+(a)[2]*(b)[2])
//#define VectorSubtract(a,b,c)	((c)[0]=(a)[0]-(b)[0],(c)[1]=(a)[1]-(b)[1],(c)[2]=(a)[2]-(b)[2])
//#define VectorAdd(a,b,c)		((c)[0]=(a)[0]+(b)[0],(c)[1]=(a)[1]+(b)[1],(c)[2]=(a)[2]+(b)[2])
//#define VectorCopy(a,b)			((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
//#define VectorCopy(a,b)			((b).x=(a).x,(b).y=(a).y,(b).z=(a).z])

//#define	VectorScale(v, s, o)	((o)[0]=(v)[0]*(s),(o)[1]=(v)[1]*(s),(o)[2]=(v)[2]*(s))

// __VectorMA: multiply-add for vectors
macro_rules! __VectorMA {
    ($v:expr, $s:expr, $b:expr, $o:expr) => {{
        $o[0] = $v[0] + $b[0] * $s;
        $o[1] = $v[1] + $b[1] * $s;
        $o[2] = $v[2] + $b[2] * $s;
    }};
}

//#define CrossProduct(a,b,c)		((c)[0]=(a)[1]*(b)[2]-(a)[2]*(b)[1],(c)[1]=(a)[2]*(b)[0]-(a)[0]*(b)[2],(c)[2]=(a)[0]*(b)[1]-(a)[1]*(b)[0])

#[inline]
pub fn DotProduct4(x: &[f32; 4], y: &[f32; 4]) -> f32 {
    x[0] * y[0] + x[1] * y[1] + x[2] * y[2] + x[3] * y[3]
}

#[inline]
pub fn VectorSubtract4(a: &[f32; 4], b: &[f32; 4], c: &mut [f32; 4]) {
    c[0] = a[0] - b[0];
    c[1] = a[1] - b[1];
    c[2] = a[2] - b[2];
    c[3] = a[3] - b[3];
}

#[inline]
pub fn VectorAdd4(a: &[f32; 4], b: &[f32; 4], c: &mut [f32; 4]) {
    c[0] = a[0] + b[0];
    c[1] = a[1] + b[1];
    c[2] = a[2] + b[2];
    c[3] = a[3] + b[3];
}

#[inline]
pub fn VectorCopy4(a: &[f32; 4], b: &mut [f32; 4]) {
    b[0] = a[0];
    b[1] = a[1];
    b[2] = a[2];
    b[3] = a[3];
}

#[inline]
pub fn VectorScale4(v: &[f32; 4], s: f32, o: &mut [f32; 4]) {
    o[0] = v[0] * s;
    o[1] = v[1] * s;
    o[2] = v[2] * s;
    o[3] = v[3] * s;
}

#[inline]
pub fn VectorMA4(v: &[f32; 4], s: f32, b: &[f32; 4], o: &mut [f32; 4]) {
    o[0] = v[0] + b[0] * s;
    o[1] = v[1] + b[1] * s;
    o[2] = v[2] + b[2] * s;
    o[3] = v[3] + b[3] * s;
}

//#define VectorClear(a)			((a)[0]=(a)[1]=(a)[2]=0)

#[inline]
pub fn VectorNegate(a: &[f32; 3], b: &mut [f32; 3]) {
    b[0] = -a[0];
    b[1] = -a[1];
    b[2] = -a[2];
}

#[inline]
pub fn Vector4Copy(a: &[f32; 4], b: &mut [f32; 4]) {
    b[0] = a[0];
    b[1] = a[1];
    b[2] = a[2];
    b[3] = a[3];
}

#[inline]
pub fn SnapVector(v: &mut [f32; 3]) {
    v[0] = v[0] as c_int as f32;
    v[1] = v[1] as c_int as f32;
    v[2] = v[2] as c_int as f32;
}

//#include "util_heap.h"

pub const EQUAL_EPSILON: f32 = 0.001;

extern "C" {
    pub fn Q_fabs(f: c_float) -> c_float;
}

// if this is defined, vec3 will take four elements, which may allow
// easier SIMD optimizations
//#define	FAT_VEC3
//#ifdef __ppc__
//#pragma align(16)
//#endif

// class angles_t;
#[cfg(target_arch = "powerpc")]
// Vanilla PPC code, but since PPC has a reciprocal square root estimate instruction,
// runs *much* faster than calling sqrt(). We'll use two Newton-Raphson
// refinement steps to get bunch more precision in the 1/sqrt() value for very little cost.
// We'll then multiply 1/sqrt times the original value to get the sqrt.
// This is about 12.4 times faster than sqrt() and according to my testing (not exhaustive)
// it returns fairly accurate results (error below 1.0e-5 up to 100000.0 in 0.1 increments).
#[inline]
pub fn idSqrt(x: f32) -> f32 {
    const HALF: f32 = 0.5;
    const ONE: f32 = 1.0;

    // This'll NaN if it hits frsqrte. Handle both +0.0 and -0.0
    if x.abs() == 0.0 {
        return x;
    }

    let b = x;

    // Call PPC reciprocal square root estimate
    let y0: f32;
    #[cfg(all(target_arch = "powerpc", target_env = "gnu"))]
    {
        unsafe {
            asm!("frsqrte {}, {}", out(freg) y0, in(freg) b);
        }
    }
    #[cfg(not(all(target_arch = "powerpc", target_env = "gnu")))]
    {
        // Fallback for non-GCC PPC or non-PPC platforms
        y0 = unsafe { __frsqrte(b) };
    }

    /* First refinement step */
    let y1 = y0 + HALF * y0 * (ONE - b * y0 * y0);

    /* Second refinement step -- copy the output of the last step to the input of this step */
    let y0 = y1;
    let y1 = y0 + HALF * y0 * (ONE - b * y0 * y0);

    /* Get sqrt(x) from x * 1/sqrt(x) */
    x * y1
}

#[cfg(not(target_arch = "powerpc"))]
#[inline]
pub fn idSqrt(x: f64) -> f64 {
    x.sqrt()
}

extern "C" {
    #[cfg(target_arch = "powerpc")]
    pub fn __frsqrte(x: f32) -> f32;
}

//class idVec3_t  : public idHeap<idVec3_t> {

#[repr(C)]
pub struct idVec3_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // #ifdef FAT_VEC3
    // pub dist: f32,
    // #endif
}

impl idVec3_t {
    #[inline]
    pub fn new() -> Self {
        // #ifndef FAT_VEC3
        idVec3_t { x: 0.0, y: 0.0, z: 0.0 }
        // #else
        // idVec3_t { x: 0.0, y: 0.0, z: 0.0, dist: 0.0 }
        // #endif
    }

    #[inline]
    pub fn from(x: f32, y: f32, z: f32) -> Self {
        idVec3_t { x, y, z }
        // #ifdef FAT_VEC3
        // idVec3_t { x, y, z, dist: 0.0 }
        // #endif
    }

    // operator float *()
    #[inline]
    pub fn as_ptr(&self) -> *const f32 {
        &self.x
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        &mut self.x
    }

    // operator[](const int index) const
    #[inline]
    pub fn index(&self, index: c_int) -> f32 {
        unsafe { *(&self.x as *const f32).add(index as usize) }
    }

    // operator[](const int index)
    #[inline]
    pub fn index_mut(&mut self, index: c_int) -> &mut f32 {
        unsafe { &mut *(&mut self.x as *mut f32).add(index as usize) }
    }

    // void set( const float x, const float y, const float z )
    #[inline]
    pub fn set(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    // idVec3_t operator-() const
    #[inline]
    pub fn neg(&self) -> idVec3_t {
        idVec3_t { x: -self.x, y: -self.y, z: -self.z }
    }

    // idVec3_t &operator=( const idVec3_t &a )
    #[inline]
    pub fn assign(&mut self, a: &idVec3_t) -> &mut Self {
        self.x = a.x;
        self.y = a.y;
        self.z = a.z;
        self
    }

    // float operator*( const idVec3_t &a ) const
    // Dot product
    #[inline]
    pub fn dot(&self, a: &idVec3_t) -> f32 {
        self.x * a.x + self.y * a.y + self.z * a.z
    }

    // idVec3_t operator*( const float a ) const
    #[inline]
    pub fn mul(&self, a: f32) -> idVec3_t {
        idVec3_t { x: self.x * a, y: self.y * a, z: self.z * a }
    }

    // idVec3_t operator+( const idVec3_t &a ) const
    #[inline]
    pub fn add(&self, a: &idVec3_t) -> idVec3_t {
        idVec3_t { x: self.x + a.x, y: self.y + a.y, z: self.z + a.z }
    }

    // idVec3_t operator-( const idVec3_t &a ) const
    #[inline]
    pub fn sub(&self, a: &idVec3_t) -> idVec3_t {
        idVec3_t { x: self.x - a.x, y: self.y - a.y, z: self.z - a.z }
    }

    // idVec3_t &operator+=( const idVec3_t &a )
    #[inline]
    pub fn add_assign(&mut self, a: &idVec3_t) -> &mut Self {
        self.x += a.x;
        self.y += a.y;
        self.z += a.z;
        self
    }

    // idVec3_t &operator-=( const idVec3_t &a )
    #[inline]
    pub fn sub_assign(&mut self, a: &idVec3_t) -> &mut Self {
        self.x -= a.x;
        self.y -= a.y;
        self.z -= a.z;
        self
    }

    // idVec3_t &operator*=( const float a )
    #[inline]
    pub fn mul_assign(&mut self, a: f32) -> &mut Self {
        self.x *= a;
        self.y *= a;
        self.z *= a;
        self
    }

    // int operator==( const idVec3_t &a ) const
    #[inline]
    pub fn eq(&self, a: &idVec3_t) -> bool {
        if (self.x - a.x).abs() > EQUAL_EPSILON {
            return false;
        }

        if (self.y - a.y).abs() > EQUAL_EPSILON {
            return false;
        }

        if (self.z - a.z).abs() > EQUAL_EPSILON {
            return false;
        }

        true
    }

    // int operator!=( const idVec3_t &a ) const
    #[inline]
    pub fn ne(&self, a: &idVec3_t) -> bool {
        if (self.x - a.x).abs() > EQUAL_EPSILON {
            return true;
        }

        if (self.y - a.y).abs() > EQUAL_EPSILON {
            return true;
        }

        if (self.z - a.z).abs() > EQUAL_EPSILON {
            return true;
        }

        false
    }

    // idVec3_t Cross( const idVec3_t &a ) const
    #[inline]
    pub fn cross(&self, a: &idVec3_t) -> idVec3_t {
        idVec3_t {
            x: self.y * a.z - self.z * a.y,
            y: self.z * a.x - self.x * a.z,
            z: self.x * a.y - self.y * a.x,
        }
    }

    // idVec3_t &Cross( const idVec3_t &a, const idVec3_t &b )
    #[inline]
    pub fn cross_assign(&mut self, a: &idVec3_t, b: &idVec3_t) -> &mut Self {
        self.x = a.y * b.z - a.z * b.y;
        self.y = a.z * b.x - a.x * b.z;
        self.z = a.x * b.y - a.y * b.x;
        self
    }

    // float Length( void ) const
    #[inline]
    pub fn Length(&self) -> f32 {
        let length = self.x * self.x + self.y * self.y + self.z * self.z;
        #[cfg(target_arch = "powerpc")]
        {
            idSqrt(length) as f32
        }
        #[cfg(not(target_arch = "powerpc"))]
        {
            idSqrt(length as f64) as f32
        }
    }

    // float Normalize( void )
    #[inline]
    pub fn Normalize(&mut self) -> f32 {
        let length = self.Length();
        if length != 0.0 {
            let ilength = 1.0 / length;
            self.x *= ilength;
            self.y *= ilength;
            self.z *= ilength;
        }
        length
    }

    // void Zero( void )
    #[inline]
    pub fn Zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
    }

    // void Snap( void )
    #[inline]
    pub fn Snap(&mut self) {
        self.x = self.x as c_int as f32;
        self.y = self.y as c_int as f32;
        self.z = self.z as c_int as f32;
    }

    // SnapTowards
    // Round a vector to integers for more efficient network
    // transmission, but make sure that it rounds towards a given point
    // rather than blindly truncating.  This prevents it from truncating
    // into a wall.
    #[inline]
    pub fn SnapTowards(&mut self, to: &idVec3_t) {
        if to.x <= self.x {
            self.x = self.x as c_int as f32;
        } else {
            self.x = (self.x as c_int + 1) as f32;
        }

        if to.y <= self.y {
            self.y = self.y as c_int as f32;
        } else {
            self.y = (self.y as c_int + 1) as f32;
        }

        if to.z <= self.z {
            self.z = self.z as c_int as f32;
        } else {
            self.z = (self.z as c_int + 1) as f32;
        }
    }

    // float toYaw( void );
    // float toPitch( void );
    // angles_t toAngles( void );
    // char *string( void );
}

// friend idVec3_t operator*( float a, idVec3_t b )
#[inline]
pub fn mul_scalar_vec3(a: f32, b: &idVec3_t) -> idVec3_t {
    idVec3_t { x: b.x * a, y: b.y * a, z: b.z * a }
}

// friend idVec3_t LerpVector( const idVec3_t &w1, const idVec3_t &w2, const float t );

extern "C" {
    pub static mut vec_zero: idVec3_t;
}

//===============================================================

#[repr(C)]
pub struct Bounds {
    pub b: [idVec3_t; 2],
}

impl Bounds {
    // Bounds();
    #[inline]
    pub fn new() -> Self {
        Bounds {
            b: [idVec3_t { x: 0.0, y: 0.0, z: 0.0 }, idVec3_t { x: 0.0, y: 0.0, z: 0.0 }],
        }
    }

    // Bounds( const idVec3_t &mins, const idVec3_t &maxs )
    #[inline]
    pub fn from_minmax(mins: &idVec3_t, maxs: &idVec3_t) -> Self {
        Bounds {
            b: [idVec3_t { x: mins.x, y: mins.y, z: mins.z }, idVec3_t { x: maxs.x, y: maxs.y, z: maxs.z }],
        }
    }

    // bool IsCleared()
    #[inline]
    pub fn IsCleared(&self) -> bool {
        self.b[0].x > self.b[1].x
    }

    // bool ContainsPoint( const idVec3_t &p )
    #[inline]
    pub fn ContainsPoint(&self, p: &idVec3_t) -> bool {
        if p.x < self.b[0].x || p.y < self.b[0].y || p.z < self.b[0].z
            || p.x > self.b[1].x || p.y > self.b[1].y || p.z > self.b[1].z
        {
            return false;
        }
        true
    }

    // bool IntersectsBounds( const Bounds &b2 )
    // touching is NOT intersecting
    #[inline]
    pub fn IntersectsBounds(&self, b2: &Bounds) -> bool {
        if b2.b[1].x < self.b[0].x || b2.b[1].y < self.b[0].y || b2.b[1].z < self.b[0].z
            || b2.b[0].x > self.b[1].x || b2.b[0].y > self.b[1].y || b2.b[0].z > self.b[1].z
        {
            return false;
        }
        true
    }

    // idVec3_t Center()
    #[inline]
    pub fn Center(&self) -> idVec3_t {
        idVec3_t {
            x: (self.b[1].x + self.b[0].x) * 0.5,
            y: (self.b[1].y + self.b[0].y) * 0.5,
            z: (self.b[1].z + self.b[0].z) * 0.5,
        }
    }

    // void Clear()
    #[inline]
    pub fn Clear(&mut self) {
        self.b[0].x = 99999.0;
        self.b[0].y = 99999.0;
        self.b[0].z = 99999.0;
        self.b[1].x = -99999.0;
        self.b[1].y = -99999.0;
        self.b[1].z = -99999.0;
    }

    // void Zero()
    #[inline]
    pub fn Zero(&mut self) {
        self.b[0].x = 0.0;
        self.b[0].y = 0.0;
        self.b[0].z = 0.0;
        self.b[1].x = 0.0;
        self.b[1].y = 0.0;
        self.b[1].z = 0.0;
    }

    // void AddPoint( const idVec3_t &v )
    #[inline]
    pub fn AddPoint(&mut self, v: &idVec3_t) {
        if v.x < self.b[0].x {
            self.b[0].x = v.x;
        }
        if v.x > self.b[1].x {
            self.b[1].x = v.x;
        }
        if v.y < self.b[0].y {
            self.b[0].y = v.y;
        }
        if v.y > self.b[1].y {
            self.b[1].y = v.y;
        }
        if v.z < self.b[0].z {
            self.b[0].z = v.z;
        }
        if v.z > self.b[1].z {
            self.b[1].z = v.z;
        }
    }

    // void AddBounds( const Bounds &bb )
    #[inline]
    pub fn AddBounds(&mut self, bb: &Bounds) {
        if bb.b[0].x < self.b[0].x {
            self.b[0].x = bb.b[0].x;
        }
        if bb.b[0].y < self.b[0].y {
            self.b[0].y = bb.b[0].y;
        }
        if bb.b[0].z < self.b[0].z {
            self.b[0].z = bb.b[0].z;
        }

        if bb.b[1].x > self.b[1].x {
            self.b[1].x = bb.b[1].x;
        }
        if bb.b[1].y > self.b[1].y {
            self.b[1].y = bb.b[1].y;
        }
        if bb.b[1].z > self.b[1].z {
            self.b[1].z = bb.b[1].z;
        }
    }

    // float Radius( )
    #[inline]
    pub fn Radius(&self) -> f32 {
        let mut total = 0.0;
        for i in 0..3 {
            let mut a = self.b[0].index(i as c_int).abs();
            let aa = self.b[1].index(i as c_int).abs();
            if aa > a {
                a = aa;
            }
            total += a * a;
        }

        #[cfg(target_arch = "powerpc")]
        {
            idSqrt(total) as f32
        }
        #[cfg(not(target_arch = "powerpc"))]
        {
            idSqrt(total as f64) as f32
        }
    }
}

extern "C" {
    pub static mut boundsZero: Bounds;
}

//===============================================================

#[repr(C)]
pub struct idVec2_t {
    pub x: f32,
    pub y: f32,
}

impl idVec2_t {
    // operator float *()
    #[inline]
    pub fn as_ptr(&self) -> *const f32 {
        &self.x
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        &mut self.x
    }

    // float operator[]( int index ) const
    #[inline]
    pub fn index(&self, index: c_int) -> f32 {
        unsafe { *(&self.x as *const f32).add(index as usize) }
    }

    // float &operator[]( int index )
    #[inline]
    pub fn index_mut(&mut self, index: c_int) -> &mut f32 {
        unsafe { &mut *(&mut self.x as *mut f32).add(index as usize) }
    }
}

#[repr(C)]
pub struct vec4_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // #ifndef FAT_VEC3
    pub dist: f32,
    // #endif
}

impl vec4_t {
    #[inline]
    pub fn new() -> Self {
        vec4_t { x: 0.0, y: 0.0, z: 0.0, dist: 0.0 }
    }

    #[inline]
    pub fn from(x: f32, y: f32, z: f32, dist: f32) -> Self {
        vec4_t { x, y, z, dist }
    }

    // float operator[]( int index ) const
    #[inline]
    pub fn index(&self, index: c_int) -> f32 {
        unsafe { *(&self.x as *const f32).add(index as usize) }
    }

    // float &operator[]( int index )
    #[inline]
    pub fn index_mut(&mut self, index: c_int) -> &mut f32 {
        unsafe { &mut *(&mut self.x as *mut f32).add(index as usize) }
    }
}

#[repr(C)]
pub struct idVec5_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub s: f32,
    pub t: f32,
}

impl idVec5_t {
    // float operator[]( int index ) const
    #[inline]
    pub fn index(&self, index: c_int) -> f32 {
        unsafe { *(&self.x as *const f32).add(index as usize) }
    }

    // float &operator[]( int index )
    #[inline]
    pub fn index_mut(&mut self, index: c_int) -> &mut f32 {
        unsafe { &mut *(&mut self.x as *mut f32).add(index as usize) }
    }
}
