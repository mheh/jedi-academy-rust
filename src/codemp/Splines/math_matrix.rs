#![allow(non_snake_case)]

use super::math_quaternion::{angles_t, idVec3_t, mat3_t, quat_t};

// Local stubs for vector operations, needed for structural coherence of this file.
// These are part of idVec3_t and will be fully implemented when math_vector.h is ported.

impl idVec3_t {
	// set method to initialize a vector - from math_vector.h
	pub fn set(&mut self, x: f32, y: f32, z: f32) {
		self.x = x;
		self.y = y;
		self.z = z;
	}
}

// Implement vector dot product (operator*) for idVec3_t
// Returns: a.x * b.x + a.y * b.y + a.z * b.z
impl core::ops::Mul<idVec3_t> for idVec3_t {
	type Output = f32;

	fn mul(self, other: idVec3_t) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}
}

impl core::ops::Mul<&idVec3_t> for &idVec3_t {
	type Output = f32;

	fn mul(self, other: &idVec3_t) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}
}

// Implement scalar multiplication for idVec3_t
impl core::ops::Mul<f32> for idVec3_t {
	type Output = idVec3_t;

	fn mul(self, scalar: f32) -> idVec3_t {
		idVec3_t {
			x: self.x * scalar,
			y: self.y * scalar,
			z: self.z * scalar,
		}
	}
}

impl core::ops::Mul<f32> for &idVec3_t {
	type Output = idVec3_t;

	fn mul(self, scalar: f32) -> idVec3_t {
		idVec3_t {
			x: self.x * scalar,
			y: self.y * scalar,
			z: self.z * scalar,
		}
	}
}

// Implement vector addition for idVec3_t
impl core::ops::Add for idVec3_t {
	type Output = idVec3_t;

	fn add(self, other: idVec3_t) -> idVec3_t {
		idVec3_t {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
}

impl core::ops::Add<idVec3_t> for &idVec3_t {
	type Output = idVec3_t;

	fn add(self, other: idVec3_t) -> idVec3_t {
		idVec3_t {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
}

// For Clone to work with mat3_t
impl Clone for idVec3_t {
	fn clone(&self) -> Self {
		idVec3_t {
			x: self.x,
			y: self.y,
			z: self.z,
		}
	}
}

impl Clone for mat3_t {
	fn clone(&self) -> Self {
		mat3_t {
			mat: [self.mat[0].clone(), self.mat[1].clone(), self.mat[2].clone()],
		}
	}
}

// Import M_PI from q_shared_h
use crate::codemp::game::q_shared_h::M_PI;

// mat3_t mat3_default( idVec3_t( 1, 0, 0 ), idVec3_t( 0, 1, 0 ), idVec3_t( 0, 0, 1 ) );
pub static mut mat3_default: mat3_t = mat3_t {
	mat: [
		idVec3_t { x: 1.0f32, y: 0.0f32, z: 0.0f32 },
		idVec3_t { x: 0.0f32, y: 1.0f32, z: 0.0f32 },
		idVec3_t { x: 0.0f32, y: 0.0f32, z: 1.0f32 },
	],
};

// void toMatrix( quat_t const &src, mat3_t &dst )
pub fn toMatrix(src: &quat_t, dst: &mut mat3_t) {
	let mut wx: f32;
	let mut wy: f32;
	let mut wz: f32;
	let mut xx: f32;
	let mut yy: f32;
	let mut yz: f32;
	let mut xy: f32;
	let mut xz: f32;
	let mut zz: f32;
	let mut x2: f32;
	let mut y2: f32;
	let mut z2: f32;

	x2 = src.x + src.x;
	y2 = src.y + src.y;
	z2 = src.z + src.z;

	xx = src.x * x2;
	xy = src.x * y2;
	xz = src.x * z2;

	yy = src.y * y2;
	yz = src.y * z2;
	zz = src.z * z2;

	wx = src.w * x2;
	wy = src.w * y2;
	wz = src.w * z2;

	dst[0][0] = 1.0f32 - (yy + zz);
	dst[0][1] = xy - wz;
	dst[0][2] = xz + wy;

	dst[1][0] = xy + wz;
	dst[1][1] = 1.0f32 - (xx + zz);
	dst[1][2] = yz - wx;

	dst[2][0] = xz - wy;
	dst[2][1] = yz + wx;
	dst[2][2] = 1.0f32 - (xx + yy);
}

// void toMatrix( angles_t const &src, mat3_t &dst )
pub fn toMatrix_angles(src: &angles_t, dst: &mut mat3_t) {
	let mut angle: f32;
	static mut sr: f32 = 0.0f32;
	static mut sp: f32 = 0.0f32;
	static mut sy: f32 = 0.0f32;
	static mut cr: f32 = 0.0f32;
	static mut cp: f32 = 0.0f32;
	static mut cy: f32 = 0.0f32;
	// static to help MS compiler fp bugs

	angle = src.yaw * (M_PI * 2.0f32 / 360.0f32);
	unsafe {
		sy = angle.sin();
		cy = angle.cos();
	}

	angle = src.pitch * (M_PI * 2.0f32 / 360.0f32);
	unsafe {
		sp = angle.sin();
		cp = angle.cos();
	}

	angle = src.roll * (M_PI * 2.0f32 / 360.0f32);
	unsafe {
		sr = angle.sin();
		cr = angle.cos();
	}

	unsafe {
		// dst[ 0 ].set( cp * cy, cp * sy, -sp );
		dst[0].set(cp * cy, cp * sy, -sp);
		dst[1].set(sr * sp * cy + cr * -sy, sr * sp * sy + cr * cy, sr * cp);
		dst[2].set(cr * sp * cy + -sr * -sy, cr * sp * sy + -sr * cy, cr * cp);
	}
}

// void toMatrix( idVec3_t const &src, mat3_t &dst )
pub fn toMatrix_vec(src: &idVec3_t, dst: &mut mat3_t) {
	let sup = angles_t {
		pitch: src.x,
		yaw: src.y,
		roll: src.z,
	};
	toMatrix_angles(&sup, dst);
}

// void mat3_t::ProjectVector( const idVec3_t &src, idVec3_t &dst ) const
pub fn ProjectVector(this: &mat3_t, src: &idVec3_t, dst: &mut idVec3_t) {
	dst.x = src * this[0];
	dst.y = src * this[1];
	dst.z = src * this[2];
}

// void mat3_t::UnprojectVector( const idVec3_t &src, idVec3_t &dst ) const
pub fn UnprojectVector(this: &mat3_t, src: &idVec3_t, dst: &mut idVec3_t) {
	*dst = this[0] * src.x + this[1] * src.y + this[2] * src.z;
}

// void mat3_t::Transpose( mat3_t &matrix )
pub fn Transpose_into(this: &mat3_t, matrix: &mut mat3_t) {
	for i in 0..3 {
		for j in 0..3 {
			let i_idx = i as i32;
			let j_idx = j as i32;
			matrix[i_idx][j_idx] = this[j_idx][i_idx];
		}
	}
}

// void mat3_t::Transpose( void )
pub fn Transpose(this: &mut mat3_t) {
	let mut temp: f32;

	for i in 0..3 {
		for j in (i + 1)..3 {
			let i_idx = i as i32;
			let j_idx = j as i32;
			temp = this[i_idx][j_idx];
			this[i_idx][j_idx] = this[j_idx][i_idx];
			this[j_idx][i_idx] = temp;
		}
	}
}

// mat3_t mat3_t::Inverse( void ) const
pub fn Inverse(this: &mat3_t) -> mat3_t {
	let mut inv = this.clone();

	Transpose(&mut inv);

	inv
}

// void mat3_t::Clear( void )
pub fn Clear(this: &mut mat3_t) {
	this[0].set(1.0f32, 0.0f32, 0.0f32);
	this[1].set(0.0f32, 1.0f32, 0.0f32);
	this[2].set(0.0f32, 0.0f32, 1.0f32);
}
