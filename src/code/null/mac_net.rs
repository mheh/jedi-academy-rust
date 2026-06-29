#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// LOCAL stubs for unported types from game/q_shared.h and qcommon/qcommon.h
#[repr(C)]
pub struct netadr_t {
    pub type_: c_int,
    // Additional fields omitted from stub
}

#[repr(C)]
pub struct msg_t {
    // Stub structure
}

pub type qboolean = c_int;

// Stub for NA_LOOPBACK constant (from q_shared.h)
const NA_LOOPBACK: c_int = 0;

extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

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
    if unsafe { strcmp(s, b"localhost\0".as_ptr() as *const c_char) == 0 } {
        unsafe {
            memset(a as *mut c_void, 0, core::mem::size_of::<netadr_t>());
            (*a).type_ = NA_LOOPBACK;
        }
        return 1;
    }

    0
}

/*
==================
Sys_SendPacket
==================
*/
pub fn Sys_SendPacket(length: c_int, data: *mut c_void, to: netadr_t) {
}

/*
==================
Sys_GetPacket

Never called by the game logic, just the system event queing
==================
*/
pub fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean {
    0
}
