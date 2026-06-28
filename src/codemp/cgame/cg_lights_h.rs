//! Mechanical port of `codemp/cgame/cg_lights.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, MAX_QPATH};
use core::ffi::c_int;

/// `color4ub_t` — RGBA color as an array of unsigned bytes.
pub type color4ub_t = [byte; 4];

/// `clightstyle_t` — a light style entry containing a length and color data.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct clightstyle_t {
    pub length: c_int,
    pub value: color4ub_t,
    pub map: [color4ub_t; MAX_QPATH],
}

unsafe extern "C" {
    pub fn CG_ClearLightStyles();
    pub fn CG_RunLightStyles();
    pub fn CG_SetLightstyle(i: c_int);
}
