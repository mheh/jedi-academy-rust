//! `syn.h` — synonym context constants.

#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_uint};

pub const CONTEXT_ALL: c_uint = 0xFFFFFFFF;
pub const CONTEXT_NORMAL: c_uint = 1;
pub const CONTEXT_NEARBYITEM: c_uint = 2;
pub const CONTEXT_CTFREDTEAM: c_uint = 4;
pub const CONTEXT_CTFBLUETEAM: c_uint = 8;
pub const CONTEXT_REPLY: c_uint = 16;
pub const CONTEXT_HARVESTERREDTEAM: c_uint = 128;
pub const CONTEXT_HARVESTERBLUETEAM: c_uint = 256;

pub const CONTEXT_NAMES: c_int = 1024;
