// linux_local.h: Linux-specific Quake3 header file

use core::ffi::{c_char, c_int, c_void};

use crate::ffi::types::qboolean;
use crate::codemp::qcommon::net_chan::{netadr_t, msg_t};
use super::linux_joystick::sysEventType_t;

extern "C" {
    pub fn Sys_QueEvent(time: c_int, r#type: sysEventType_t, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void);
    pub fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean;
    pub fn Sys_SendKeyEvents();

    // Input subsystem

    pub fn IN_Init();
    pub fn IN_Frame();
    pub fn IN_Shutdown();


    pub fn IN_JoyMove();
    pub fn IN_StartupJoystick();

    // GL subsystem
    pub fn QGL_Init(dllname: *const c_char) -> qboolean;
    pub fn QGL_EnableLogging(enable: qboolean);
    pub fn QGL_Shutdown();




    // bk001130 - win32
    // pub fn IN_JoystickCommands();

    pub fn strlwr(s: *mut c_char) -> *mut c_char;
}
