#![allow(non_snake_case)]

use core::ops::{Index, IndexMut};

// Local stubs for unported header types, needed for structural coherence of this file.
// These types are defined in the corresponding .h headers and will be properly
// implemented when those headers are ported.

#[repr(C)]
pub struct quat_t {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
}

impl Index<i32> for quat_t {
	type Output = f32;

	fn index(&self, index: i32) -> &f32 {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			3 => &self.w,
			_ => &self.w,
		}
	}
}

impl IndexMut<i32> for quat_t {
	fn index_mut(&mut self, index: i32) -> &mut f32 {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			2 => &mut self.z,
			3 => &mut self.w,
			_ => &mut self.w,
		}
	}
}

#[repr(C)]
pub struct idVec3_t {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Index<i32> for idVec3_t {
	type Output = f32;

	fn index(&self, index: i32) -> &f32 {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			_ => &self.z,
		}
	}
}

impl IndexMut<i32> for idVec3_t {
	fn index_mut(&mut self, index: i32) -> &mut f32 {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			2 => &mut self.z,
			_ => &mut self.z,
		}
	}
}

#[repr(C)]
pub struct angles_t {
	pub pitch: f32,
	pub yaw: f32,
	pub roll: f32,
}

#[repr(C)]
pub struct mat3_t {
	pub mat: [idVec3_t; 3],
}

impl Index<i32> for mat3_t {
	type Output = idVec3_t;

	fn index(&self, index: i32) -> &idVec3_t {
		&self.mat[index as usize]
	}
}

impl IndexMut<i32> for mat3_t {
	fn index_mut(&mut self, index: i32) -> &mut idVec3_t {
		&mut self.mat[index as usize]
	}
}

// External function declarations
extern "C" {
	fn toMatrix(src: &angles_t, dst: &mut mat3_t);
}

// void toQuat( idVec3_t &src, quat_t &dst )
pub fn toQuat(src: &idVec3_t, dst: &mut quat_t) {
	dst.x = src.x;
	dst.y = src.y;
	dst.z = src.z;
	dst.w = 0.0f32;
}

// void toQuat( angles_t &src, quat_t &dst )
pub fn toQuat_angles(src: &angles_t, dst: &mut quat_t) {
	let mut temp = mat3_t {
		mat: [
			idVec3_t {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			idVec3_t {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			idVec3_t {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
		],
	};

	unsafe {
		toMatrix(src, &mut temp);
	}
	toQuat_mat3(&temp, dst);
}

// void toQuat( mat3_t &src, quat_t &dst )
pub fn toQuat_mat3(src: &mat3_t, dst: &mut quat_t) {
	let mut trace: f32;
	let mut s: f32;
	let mut i: i32;
	let mut j: i32;
	let mut k: i32;

	static NEXT: [i32; 3] = [1, 2, 0];

	trace = src[0][0] + src[1][1] + src[2][2];
	if trace > 0.0f32 {
		s = (trace + 1.0f32).sqrt();
		dst.w = s * 0.5f32;
		s = 0.5f32 / s;

		dst.x = (src[2][1] - src[1][2]) * s;
		dst.y = (src[0][2] - src[2][0]) * s;
		dst.z = (src[1][0] - src[0][1]) * s;
	} else {
		i = 0;
		if src[1][1] > src[0][0] {
			i = 1;
		}
		if src[2][2] > src[i][i] {
			i = 2;
		}

		j = NEXT[i as usize];
		k = NEXT[j as usize];

		s = ((src[i][i] - (src[j][j] + src[k][k])) + 1.0f32).sqrt();
		dst[i] = s * 0.5f32;

		s = 0.5f32 / s;

		dst.w = (src[k][j] - src[j][k]) * s;
		dst[j] = (src[j][i] + src[i][j]) * s;
		dst[k] = (src[k][i] + src[i][k]) * s;
	}
}
