// bg_local.h -- local definitions for the bg (both games) files

#![allow(non_snake_case)]

use core::ffi::{c_int, c_float};

// Forward declarations for opaque types from included headers
#[repr(C)]
pub struct pmove_t {
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
	_opaque: [u8; 0],
}

// Type aliases
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

pub const TIMER_LAND: c_int = 130;
pub const TIMER_GESTURE: c_int = (34*66+50);

pub const OVERCLIP: f32 = 1.001;

// all of the locals will be zeroed before each
// pmove, just to make damn sure we don't have
// any differences when running on client or server
#[repr(C)]
#[derive(Clone, Copy)]
pub struct pml_t {
	pub forward: vec3_t,
	pub right: vec3_t,
	pub up: vec3_t,
	pub frametime: f32,

	pub msec: c_int,

	pub walking: qboolean,
	pub groundPlane: qboolean,
	pub groundTrace: trace_t,

	pub impactSpeed: f32,

	pub previous_origin: vec3_t,
	pub previous_velocity: vec3_t,
	pub previous_waterlevel: c_int,
}

extern "C" {
	pub static mut pm: *mut pmove_t;
	pub static mut pml: pml_t;

	// movement parameters
	pub static pm_stopspeed: c_float;
	pub static pm_duckScale: c_float;
	pub static pm_swimScale: c_float;
	pub static pm_wadeScale: c_float;

	pub static pm_accelerate: c_float;
	pub static pm_airaccelerate: c_float;
	pub static pm_wateraccelerate: c_float;
	pub static pm_flyaccelerate: c_float;

	pub static pm_friction: c_float;
	pub static pm_waterfriction: c_float;
	pub static pm_flightfriction: c_float;

	pub static mut c_pmove: c_int;

	pub fn PM_ClipVelocity(r#in: *mut vec3_t, normal: *mut vec3_t, out: *mut vec3_t, overbounce: f32);
	pub fn PM_AddTouchEnt(entityNum: c_int);
	pub fn PM_AddEvent(newEvent: c_int);

	pub fn PM_SlideMove(gravity: f32) -> qboolean;
	pub fn PM_StepSlideMove(gravity: f32);
}
