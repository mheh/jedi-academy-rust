#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_ushort, c_void};

use crate::codemp::game::q_shared_h::{byte, qboolean, QFALSE, QTRUE};

pub type netadrtype_t = c_int;

pub const NA_BOT: netadrtype_t = 0;
pub const NA_BAD: netadrtype_t = 1;
pub const NA_LOOPBACK: netadrtype_t = 2;
pub const NA_BROADCAST: netadrtype_t = 3;
pub const NA_IP: netadrtype_t = 4;
pub const NA_IPX: netadrtype_t = 5;
pub const NA_BROADCAST_IPX: netadrtype_t = 6;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct netadr_t {
    pub r#type: netadrtype_t,
    pub ip: [byte; 4],
    pub ipx: [byte; 10],
    pub port: c_ushort,
}

const _: () = assert!(core::mem::size_of::<netadr_t>() == 20);
const _: () = assert!(core::mem::align_of::<netadr_t>() == 4);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct msg_t {
    pub allowoverflow: qboolean,
    pub overflowed: qboolean,
    pub oob: qboolean,
    pub data: *mut byte,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

unsafe fn c_strcmp(mut s1: *const c_char, mut s2: *const c_char) -> c_int {
    loop {
        let c1 = *s1 as u8 as c_int;
        let c2 = *s2 as u8 as c_int;
        if c1 != c2 {
            return c1 - c2;
        }
        if c1 == 0 {
            return 0;
        }
        s1 = s1.add(1);
        s2 = s2.add(1);
    }
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
#[no_mangle]
pub unsafe extern "C" fn NET_StringToAdr(s: *mut c_char, a: *mut netadr_t) -> qboolean {
    if c_strcmp(s, c"localhost".as_ptr()) == 0 {
        core::ptr::write_bytes(a, 0, 1);
        (*a).r#type = NA_LOOPBACK;
        return QTRUE;
    }

    QFALSE
}

/*
==================
Sys_SendPacket
==================
*/
#[no_mangle]
pub unsafe extern "C" fn Sys_SendPacket(_length: c_int, _data: *mut c_void, _to: netadr_t) {}

/*
==================
Sys_GetPacket

Never called by the game logic, just the system event queing
==================
*/
#[no_mangle]
pub unsafe extern "C" fn Sys_GetPacket(
    _net_from: *mut netadr_t,
    _net_message: *mut msg_t,
) -> qboolean {
    QFALSE
}
