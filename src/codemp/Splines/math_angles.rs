#![allow(non_snake_case)]

use super::math_quaternion::{angles_t, idVec3_t, mat3_t, quat_t};
use crate::codemp::game::q_shared_h::M_PI;

pub static mut ang_zero: angles_t = angles_t {
	pitch: 0.0f32,
	yaw: 0.0f32,
	roll: 0.0f32,
};

pub fn toAngles_mat3_dst(src: &mut mat3_t, dst: &mut angles_t) {
	let mut theta: f64;
	let mut cp: f64;
	let mut sp: f64;

	sp = src[0][2] as f64;

	// cap off our sin value so that we don't get any NANs
	if sp > 1.0 {
		sp = 1.0;
	} else if sp < -1.0 {
		sp = -1.0;
	}

	theta = -sp.asin();
	cp = theta.cos();

	if cp > 8192.0 * f32::EPSILON as f64 {
		dst.pitch = (theta * 180.0 / M_PI) as f32;
		dst.yaw = ((src[0][1] as f64).atan2(src[0][0] as f64) * 180.0 / M_PI) as f32;
		dst.roll = ((src[1][2] as f64).atan2(src[2][2] as f64) * 180.0 / M_PI) as f32;
	} else {
		dst.pitch = (theta * 180.0 / M_PI) as f32;
		dst.yaw = (-(src[1][0] as f64).atan2(src[1][1] as f64) * 180.0 / M_PI) as f32;
		dst.roll = 0.0;
	}
}

pub fn toAngles_quat_dst(src: &mut quat_t, dst: &mut angles_t) {
	let mut temp = mat3_t {
		mat: [
			idVec3_t {
				x: 0.0f32,
				y: 0.0f32,
				z: 0.0f32,
			},
			idVec3_t {
				x: 0.0f32,
				y: 0.0f32,
				z: 0.0f32,
			},
			idVec3_t {
				x: 0.0f32,
				y: 0.0f32,
				z: 0.0f32,
			},
		],
	};

	unsafe {
		toMatrix(src, &mut temp);
	}
	toAngles_mat3_dst(&mut temp, dst);
}

pub fn toAngles_src_dst(src: &mut idVec3_t, dst: &mut angles_t) {
	dst.pitch = src.x;
	dst.yaw = src.y;
	dst.roll = src.z;
}

extern "C" {
	fn toMatrix(src: &mut quat_t, dst: &mut mat3_t);
}

impl angles_t {
	// void angles_t::toVectors( idVec3_t *forward, idVec3_t *right, idVec3_t *up )
	pub fn toVectors(&self, forward: *mut idVec3_t, right: *mut idVec3_t, up: *mut idVec3_t) {
		let mut angle: f32;
		let mut sr: f32;
		let mut sp: f32;
		let mut sy: f32;
		let mut cr: f32;
		let mut cp: f32;
		let mut cy: f32;
		// static to help MS compiler fp bugs

		angle = self.yaw * (M_PI as f32 * 2.0f32 / 360.0f32);
		sy = angle.sin();
		cy = angle.cos();

		angle = self.pitch * (M_PI as f32 * 2.0f32 / 360.0f32);
		sp = angle.sin();
		cp = angle.cos();

		angle = self.roll * (M_PI as f32 * 2.0f32 / 360.0f32);
		sr = angle.sin();
		cr = angle.cos();

		if !forward.is_null() {
			unsafe {
				(*forward).x = cp * cy;
				(*forward).y = cp * sy;
				(*forward).z = -sp;
			}
		}

		if !right.is_null() {
			unsafe {
				(*right).x = -sr * sp * cy + cr * sy;
				(*right).y = -sr * sp * sy + -cr * cy;
				(*right).z = -sr * cp;
			}
		}

		if !up.is_null() {
			unsafe {
				(*up).x = cr * sp * cy + -sr * -sy;
				(*up).y = cr * sp * sy + -sr * cy;
				(*up).z = cr * cp;
			}
		}
	}

	// idVec3_t angles_t::toForward( void )
	pub fn toForward(&self) -> idVec3_t {
		let mut angle: f32;
		let mut sp: f32;
		let mut sy: f32;
		let mut cp: f32;
		let mut cy: f32;
		// static to help MS compiler fp bugs

		angle = self.yaw * (M_PI as f32 * 2.0f32 / 360.0f32);
		sy = angle.sin();
		cy = angle.cos();

		angle = self.pitch * (M_PI as f32 * 2.0f32 / 360.0f32);
		sp = angle.sin();
		cp = angle.cos();

		idVec3_t {
			x: cp * cy,
			y: cp * sy,
			z: -sp,
		}
	}

	/*
	=================
	Normalize360

	returns angles normalized to the range [0 <= angle < 360]
	=================
	*/
	pub fn Normalize360(&mut self) -> &mut angles_t {
		self.pitch = (360.0 / 65536.0) * (((self.pitch * (65536.0 / 360.0)) as i32) & 65535) as f32;
		self.yaw = (360.0 / 65536.0) * (((self.yaw * (65536.0 / 360.0)) as i32) & 65535) as f32;
		self.roll = (360.0 / 65536.0) * (((self.roll * (65536.0 / 360.0)) as i32) & 65535) as f32;

		self
	}

	/*
	=================
	Normalize180

	returns angles normalized to the range [-180 < angle <= 180]
	=================
	*/
	pub fn Normalize180(&mut self) -> &mut angles_t {
		self.Normalize360();

		if self.pitch > 180.0 {
			self.pitch -= 360.0;
		}

		if self.yaw > 180.0 {
			self.yaw -= 360.0;
		}

		if self.roll > 180.0 {
			self.roll -= 360.0;
		}
		self
	}
}
