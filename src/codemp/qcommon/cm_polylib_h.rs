//! `cm_polylib.h` — collision-model winding helpers.

#![allow(non_snake_case)]

use crate::codemp::game::q_shared_h::{vec3_t, vec_t};
use core::ffi::c_int;

// this is only used for visualization tools in cm_ debug functions
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct winding_t {
    pub numpoints: c_int,
    pub p: [vec3_t; 4], // variable sized
}

pub const MAX_POINTS_ON_WINDING: c_int = 64;

pub const SIDE_FRONT: c_int = 0;
pub const SIDE_BACK: c_int = 1;
pub const SIDE_ON: c_int = 2;
pub const SIDE_CROSS: c_int = 3;

pub const CLIP_EPSILON: vec_t = 0.1;

pub const MAX_MAP_BOUNDS: c_int = 65535;

// you can define on_epsilon in the makefile as tighter
pub const ON_EPSILON: vec_t = 0.1;

unsafe extern "C" {
    pub fn AllocWinding(points: c_int) -> *mut winding_t;
    pub fn WindingArea(w: *mut winding_t) -> vec_t;
    pub fn WindingCenter(w: *mut winding_t, center: *mut vec3_t);
    pub fn ClipWindingEpsilon(
        input: *mut winding_t,
        normal: *mut vec3_t,
        dist: vec_t,
        epsilon: vec_t,
        front: *mut *mut winding_t,
        back: *mut *mut winding_t,
    );
    pub fn ChopWinding(input: *mut winding_t, normal: *mut vec3_t, dist: vec_t) -> *mut winding_t;
    pub fn CopyWinding(w: *mut winding_t) -> *mut winding_t;
    pub fn ReverseWinding(w: *mut winding_t) -> *mut winding_t;
    pub fn BaseWindingForPlane(normal: *mut vec3_t, dist: vec_t) -> *mut winding_t;
    pub fn CheckWinding(w: *mut winding_t);
    pub fn WindingPlane(w: *mut winding_t, normal: *mut vec3_t, dist: *mut vec_t);
    pub fn RemoveColinearPoints(w: *mut winding_t);
    pub fn WindingOnPlaneSide(w: *mut winding_t, normal: *mut vec3_t, dist: vec_t) -> c_int;
    pub fn FreeWinding(w: *mut winding_t);
    pub fn WindingBounds(w: *mut winding_t, mins: *mut vec3_t, maxs: *mut vec3_t);

    pub fn AddWindingToConvexHull(w: *mut winding_t, hull: *mut *mut winding_t, normal: *mut vec3_t);

    // frees the original if clipped
    pub fn ChopWindingInPlace(w: *mut *mut winding_t, normal: *mut vec3_t, dist: vec_t, epsilon: vec_t);

    pub fn pw(w: *mut winding_t);
}
