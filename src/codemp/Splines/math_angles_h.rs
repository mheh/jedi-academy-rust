#![allow(non_snake_case)]

use core::ffi::c_int;

// Forward declarations for types defined in math_vector.h
// These would normally be in separate modules but are declared here as stubs
// to maintain structural coherence of this port.
pub struct mat3_t;
pub struct quat_t;
pub struct idVec3_t {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub type vec3_p = *mut idVec3_t;

#[repr(C)]
pub struct angles_t {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

extern "C" {
    pub static mut ang_zero: angles_t;
}

// Forward declarations for friend functions
pub fn toAngles_src_dst(src: &mut idVec3_t, dst: &mut angles_t);
pub fn toAngles_quat_dst(src: &mut quat_t, dst: &mut angles_t);
pub fn toAngles_mat3_dst(src: &mut mat3_t, dst: &mut angles_t);

impl angles_t {
    // angles_t();
    #[inline]
    pub fn new() -> angles_t {
        angles_t {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        }
    }

    // angles_t( float pitch, float yaw, float roll );
    #[inline]
    pub fn new_with_values(pitch: f32, yaw: f32, roll: f32) -> angles_t {
        angles_t { pitch, yaw, roll }
    }

    // angles_t( const idVec3_t &vec );
    #[inline]
    pub fn new_from_vec(vec: &idVec3_t) -> angles_t {
        angles_t {
            pitch: vec.x,
            yaw: vec.y,
            roll: vec.z,
        }
    }

    // float operator[]( int index ) const;
    #[inline]
    pub fn index(&self, index: c_int) -> f32 {
        debug_assert!((index >= 0) && (index < 3));
        unsafe {
            let ptr = &self.pitch as *const f32;
            *ptr.offset(index as isize)
        }
    }

    // float& operator[]( int index );
    #[inline]
    pub fn index_mut(&mut self, index: c_int) -> &mut f32 {
        debug_assert!((index >= 0) && (index < 3));
        unsafe {
            let ptr = &mut self.pitch as *mut f32;
            &mut *ptr.offset(index as isize)
        }
    }

    // operator vec3_p();
    #[inline]
    pub fn to_vec3_p(&mut self) -> vec3_p {
        unsafe {
            &mut self.pitch as *mut f32 as *mut idVec3_t
        }
    }

    // void set( float pitch, float yaw, float roll );
    #[inline]
    pub fn set(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.pitch = pitch;
        self.yaw = yaw;
        self.roll = roll;
    }

    // void operator=( angles_t const &a );
    #[inline]
    pub fn assign_angles(&mut self, a: &angles_t) {
        self.pitch = a.pitch;
        self.yaw = a.yaw;
        self.roll = a.roll;
    }

    // void operator=( idVec3_t const &a );
    #[inline]
    pub fn assign_vec3(&mut self, a: &idVec3_t) {
        self.pitch = a.x;
        self.yaw = a.y;
        self.roll = a.z;
    }

    // angles_t& operator+=( angles_t const &a );
    #[inline]
    pub fn add_assign(&mut self, a: &angles_t) -> &mut angles_t {
        self.pitch += a.pitch;
        self.yaw += a.yaw;
        self.roll += a.roll;
        self
    }

    // angles_t& operator+=( idVec3_t const &a );
    #[inline]
    pub fn add_assign_vec3(&mut self, a: &idVec3_t) -> &mut angles_t {
        self.pitch += a.x;
        self.yaw += a.y;
        self.roll += a.z;
        self
    }

    // angles_t& operator-=( angles_t &a );
    #[inline]
    pub fn sub_assign(&mut self, a: &angles_t) -> &mut angles_t {
        self.pitch -= a.pitch;
        self.yaw -= a.yaw;
        self.roll -= a.roll;
        self
    }

    // angles_t& operator*=( float a );
    #[inline]
    pub fn mul_assign(&mut self, a: f32) -> &mut angles_t {
        self.pitch *= a;
        self.yaw *= a;
        self.roll *= a;
        self
    }

    // angles_t& Zero( void );
    #[inline]
    pub fn Zero(&mut self) -> &mut angles_t {
        self.pitch = 0.0;
        self.yaw = 0.0;
        self.roll = 0.0;
        self
    }

    // angles_t& Normalize360( void );
    pub fn Normalize360(&mut self) -> &mut angles_t {
        // Implementation would be in a .cpp file
        self
    }

    // angles_t& Normalize180( void );
    pub fn Normalize180(&mut self) -> &mut angles_t {
        // Implementation would be in a .cpp file
        self
    }

    // void toVectors( idVec3_t *forward, idVec3_t *right = NULL, idVec3_t *up = NULL );
    pub fn toVectors(&self, forward: *mut idVec3_t, right: *mut idVec3_t, up: *mut idVec3_t) {
        // Implementation would be in a .cpp file
    }

    // idVec3_t toForward( void );
    pub fn toForward(&self) -> idVec3_t {
        // Implementation would be in a .cpp file
        idVec3_t {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

// friend angles_t operator+( const angles_t &a, const angles_t &b );
#[inline]
pub fn angles_add(a: &angles_t, b: &angles_t) -> angles_t {
    angles_t {
        pitch: a.pitch + b.pitch,
        yaw: a.yaw + b.yaw,
        roll: a.roll + b.roll,
    }
}

// friend angles_t operator-( angles_t &a, angles_t &b );
#[inline]
pub fn angles_sub(a: &angles_t, b: &angles_t) -> angles_t {
    angles_t {
        pitch: a.pitch - b.pitch,
        yaw: a.yaw - b.yaw,
        roll: a.roll - b.roll,
    }
}

// friend angles_t operator*( const angles_t &a, float b );
#[inline]
pub fn angles_mul(a: &angles_t, b: f32) -> angles_t {
    angles_t {
        pitch: a.pitch * b,
        yaw: a.yaw * b,
        roll: a.roll * b,
    }
}

// friend angles_t operator*( float a, const angles_t &b );
#[inline]
pub fn angles_mul_f_a(a: f32, b: &angles_t) -> angles_t {
    angles_t {
        pitch: a * b.pitch,
        yaw: a * b.yaw,
        roll: a * b.roll,
    }
}

// friend int operator==( angles_t &a, angles_t &b );
#[inline]
pub fn angles_eq(a: &angles_t, b: &angles_t) -> c_int {
    if (a.pitch == b.pitch) && (a.yaw == b.yaw) && (a.roll == b.roll) {
        1
    } else {
        0
    }
}

// friend int operator!=( angles_t &a, angles_t &b );
#[inline]
pub fn angles_ne(a: &angles_t, b: &angles_t) -> c_int {
    if (a.pitch != b.pitch) || (a.yaw != b.yaw) || (a.roll != b.roll) {
        1
    } else {
        0
    }
}
