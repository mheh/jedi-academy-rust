//! Mechanical port of `code/null/null_net.c`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// Type definitions (from qcommon.h, q_shared.h)
// ============================================================================

/// `byte` — unsigned char.
pub type byte = u8;

/// Network address type.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum netadrtype_t {
    NA_BAD,      // an address lookup failed
    NA_LOOPBACK,
}

/// Network address structure.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct netadr_t {
    pub r#type: netadrtype_t,
    pub port: u16,
}

/// Message structure.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct msg_t {
    pub allowoverflow: c_int,
    pub overflowed: c_int,
    pub data: *mut byte,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

/// `qboolean` — engine boolean, passed as a C `int`.
pub type qboolean = c_int;
pub const QTRUE: c_int = 1;

// ============================================================================
// C library functions
// ============================================================================

extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// ============================================================================
// Functions
// ============================================================================

/*
=============
NET_StringToAdr

localhost
idnewt
idnewt:28000
192.246.40.70
192.246.40.70:28000
=============
*/
pub fn NET_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean {
    unsafe {
        if strcmp(s, b"localhost\0".as_ptr() as *const c_char) == 0 {
            memset(a as *mut c_void, 0, core::mem::size_of::<netadr_t>());
            (*a).r#type = netadrtype_t::NA_LOOPBACK;
            return QTRUE;
        }
    }
    0
}

/*
==================
Sys_SendPacket
==================
*/
pub fn Sys_SendPacket(_length: c_int, _data: *mut c_void, _to: netadr_t) {
}

/*
==================
Sys_GetPacket

Never called by the game logic, just the system event queing
==================
*/
pub fn Sys_GetPacket(_net_from: *mut netadr_t, _net_message: *mut msg_t) -> qboolean {
    0
}
