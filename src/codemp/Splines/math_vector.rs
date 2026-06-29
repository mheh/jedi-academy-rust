#![allow(non_snake_case)]

use super::math_quaternion::idVec3_t;
use crate::codemp::game::q_shared_h::M_PI;
use core::ops::{Index, IndexMut};

const LERP_DELTA: f32 = 1e-6;

// Local stub for Bounds type, needed for structural coherence of this file.
// Bounds is defined in math_vector.h and will be properly implemented when that header is ported.
#[repr(C)]
pub struct Bounds {
	pub b: [idVec3_t; 2],
}

impl Index<i32> for Bounds {
	type Output = idVec3_t;

	fn index(&self, index: i32) -> &idVec3_t {
		&self.b[index as usize]
	}
}

impl IndexMut<i32> for Bounds {
	fn index_mut(&mut self, index: i32) -> &mut idVec3_t {
		&mut self.b[index as usize]
	}
}

// idVec3_t vec_zero( 0.0f, 0.0f, 0.0f );
pub static mut vec_zero: idVec3_t = idVec3_t {
	x: 0.0f32,
	y: 0.0f32,
	z: 0.0f32,
};

// Bounds boundsZero;
pub static mut boundsZero: Bounds = Bounds {
	b: [
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

// float idVec3_t::toYaw( void )
impl idVec3_t {
	pub fn toYaw(&self) -> f32 {
		let mut yaw: f32;

		if (self.y == 0.0f32) && (self.x == 0.0f32) {
			yaw = 0.0f32;
		} else {
			yaw = self.y.atan2(self.x) * 180.0f32 / (M_PI as f32);
			if yaw < 0.0f32 {
				yaw += 360.0f32;
			}
		}

		yaw
	}
}

// float idVec3_t::toPitch( void )
impl idVec3_t {
	pub fn toPitch(&self) -> f32 {
		let mut forward: f32;
		let mut pitch: f32;

		if (self.x == 0.0f32) && (self.y == 0.0f32) {
			if self.z > 0.0f32 {
				pitch = 90.0f32;
			} else {
				pitch = 270.0f32;
			}
		} else {
			forward = (self.x * self.x + self.y * self.y).sqrt();
			pitch = self.z.atan2(forward) * 180.0f32 / (M_PI as f32);
			if pitch < 0.0f32 {
				pitch += 360.0f32;
			}
		}

		pitch
	}
}

/*
angles_t idVec3_t::toAngles( void ) {
	float forward;
	float yaw;
	float pitch;

	if ( ( x == 0 ) && ( y == 0 ) ) {
		yaw = 0;
		if ( z > 0 ) {
			pitch = 90;
		} else {
			pitch = 270;
		}
	} else {
		yaw = atan2( y, x ) * 180 / M_PI;
		if ( yaw < 0 ) {
			yaw += 360;
		}

		forward = ( float )idSqrt( x * x + y * y );
		pitch = atan2( z, forward ) * 180 / M_PI;
		if ( pitch < 0 ) {
			pitch += 360;
		}
	}

	return angles_t( -pitch, yaw, 0 );
}
*/

// idVec3_t LerpVector( idVec3_t &w1, idVec3_t &w2, const float t )
pub fn LerpVector(w1: &idVec3_t, w2: &idVec3_t, t: f32) -> idVec3_t {
	let mut omega: f32;
	let mut cosom: f32;
	let mut sinom: f32;
	let mut scale0: f32;
	let mut scale1: f32;

	cosom = w1 * w2;
	if (1.0f32 - cosom) > LERP_DELTA {
		omega = cosom.acos();
		sinom = omega.sin();
		scale0 = ((1.0f32 - t) * omega).sin() / sinom;
		scale1 = (t * omega).sin() / sinom;
	} else {
		scale0 = 1.0f32 - t;
		scale1 = t;
	}

	idVec3_t {
		x: w1.x * scale0 + w2.x * scale1,
		y: w1.y * scale0 + w2.y * scale1,
		z: w1.z * scale0 + w2.z * scale1,
	}
}

/*
=============
idVec3_t::string

This is just a convenience function
for printing vectors
=============
*/
// char *idVec3_t::string( void )
impl idVec3_t {
	pub fn string(&self) -> *const u8 {
		// Static buffer array to store formatted strings.
		// use an array so that multiple toString's won't collide
		static mut INDEX: i32 = 0;
		static mut STR: [[u8; 36]; 8] = [[0u8; 36]; 8];

		let s_idx: usize;
		unsafe {
			s_idx = INDEX as usize;
			INDEX = (INDEX + 1) & 7;

			// Format the vector as "%.2f %.2f %.2f"
			// sprintf( s, "%.2f %.2f %.2f", x, y, z );
			let formatted = format!("{:.2} {:.2} {:.2}", self.x, self.y, self.z);
			let bytes = formatted.as_bytes();

			for (i, &byte) in bytes.iter().enumerate() {
				if i < 36 {
					STR[s_idx][i] = byte;
				} else {
					break;
				}
			}
			// Null-terminate or let it overflow gracefully like C
			if bytes.len() < 36 {
				STR[s_idx][bytes.len()] = 0;
			}

			STR[s_idx].as_ptr()
		}
	}
}
