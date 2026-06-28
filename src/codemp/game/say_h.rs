//! `say.h` — saying ids.

#![allow(non_upper_case_globals)]

use core::ffi::c_int;

pub type saying_t = c_int;

// Acknowledge command
pub const SAY_ACKCOMM1: saying_t = 0;
pub const SAY_ACKCOMM2: saying_t = 1;
pub const SAY_ACKCOMM3: saying_t = 2;
pub const SAY_ACKCOMM4: saying_t = 3;
// Refuse command
pub const SAY_REFCOMM1: saying_t = 4;
pub const SAY_REFCOMM2: saying_t = 5;
pub const SAY_REFCOMM3: saying_t = 6;
pub const SAY_REFCOMM4: saying_t = 7;
// Bad command
pub const SAY_BADCOMM1: saying_t = 8;
pub const SAY_BADCOMM2: saying_t = 9;
pub const SAY_BADCOMM3: saying_t = 10;
pub const SAY_BADCOMM4: saying_t = 11;
// Unfinished hail
pub const SAY_BADHAIL1: saying_t = 12;
pub const SAY_BADHAIL2: saying_t = 13;
pub const SAY_BADHAIL3: saying_t = 14;
pub const SAY_BADHAIL4: saying_t = 15;
// # #eol
pub const NUM_SAYINGS: saying_t = 16;
