#![allow(non_snake_case, non_camel_case_types, dead_code, unused_variables)]

use core::ffi::{c_char, c_float, c_int, c_ushort, c_uint};
use core::ptr::null_mut;

use crate::codemp::game::q_shared_h::{byte, CVAR_TEMP};
use crate::ffi::types::{fileHandle_t, qboolean, QFALSE};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct msg_t {
    pub allowoverflow: qboolean, // if false, do a Com_Error
    pub overflowed: qboolean,    // set to true if the buffer size failed (with allowoverflow set)
    pub oob: qboolean,           // set to true if the buffer size failed (with allowoverflow set)
    pub data: *mut byte,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int, // for bitwise reads and writes
}

pub type netadrtype_t = c_int;

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
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub next: *mut cvar_t,
    pub hashNext: *mut cvar_t,
}

unsafe extern "C" {
    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
}

static CL_SHOWNET_NAME: [c_char; 11] = [
    b'c' as c_char,
    b'l' as c_char,
    b'_' as c_char,
    b's' as c_char,
    b'h' as c_char,
    b'o' as c_char,
    b'w' as c_char,
    b'n' as c_char,
    b'e' as c_char,
    b't' as c_char,
    0,
];
static CL_SHOWNET_DEFAULT: [c_char; 2] = [b'0' as c_char, 0];

#[no_mangle]
pub static mut cl_cdkey: [c_char; 17] = [
    b'1' as c_char,
    b'2' as c_char,
    b'3' as c_char,
    b'4' as c_char,
    b'5' as c_char,
    b'6' as c_char,
    b'7' as c_char,
    b'8' as c_char,
    b'9' as c_char,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];

#[no_mangle]
pub static mut cl_shownet: *mut cvar_t = null_mut();

#[no_mangle]
pub extern "C" fn CL_Shutdown() {}

#[no_mangle]
pub extern "C" fn CL_Init() {
    unsafe {
        cl_shownet = Cvar_Get(CL_SHOWNET_NAME.as_ptr(), CL_SHOWNET_DEFAULT.as_ptr(), CVAR_TEMP);
    }
}

#[no_mangle]
pub extern "C" fn CL_MouseEvent(dx: c_int, dy: c_int, time: c_int) {}

#[no_mangle]
pub extern "C" fn Key_WriteBindings(f: fileHandle_t) {}

#[no_mangle]
pub extern "C" fn CL_Frame(msec: c_int) {}

#[no_mangle]
pub extern "C" fn CL_PacketEvent(from: netadr_t, msg: *mut msg_t) {}

#[no_mangle]
pub extern "C" fn CL_CharEvent(key: c_int) {}

#[no_mangle]
pub extern "C" fn CL_Disconnect(showMainMenu: qboolean) {}

#[no_mangle]
pub extern "C" fn CL_MapLoading() {}

#[no_mangle]
pub extern "C" fn CL_GameCommand() -> qboolean {
    QFALSE
}

#[no_mangle]
pub extern "C" fn CL_KeyEvent(key: c_int, down: qboolean, time: c_uint) {}

#[no_mangle]
pub extern "C" fn UI_GameCommand() -> qboolean {
    QFALSE
}

#[no_mangle]
pub extern "C" fn CL_ForwardCommandToServer(string: *const c_char) {}

#[no_mangle]
pub extern "C" fn CL_ConsolePrint(txt: *const c_char, silent: qboolean) {}

#[no_mangle]
pub extern "C" fn CL_JoystickEvent(axis: c_int, value: c_int, time: c_int) {}

#[no_mangle]
pub extern "C" fn CL_InitKeyCommands() {}

#[no_mangle]
pub extern "C" fn CL_CDDialog(msg: *const c_char) {}

#[no_mangle]
pub extern "C" fn CL_FlushMemory() {}

#[no_mangle]
pub extern "C" fn CL_StartHunkUsers() {}
