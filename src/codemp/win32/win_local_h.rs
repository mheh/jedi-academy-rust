// win_local.h: Win32-specific Quake3 header file

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};

// Original C preprocessor context (informational):
// #if defined (_MSC_VER) && (_MSC_VER >= 1200)
// #pragma warning(disable : 4201)
// #pragma warning( push )
// #endif
// //#include <windows.h>
// #include "../qcommon/platform.h"
// #if defined (_MSC_VER) && (_MSC_VER >= 1200)
// #pragma warning( pop )
// #endif

// Stub type definitions for Q3 types defined in platform.h (declared to be in scope)
pub type qboolean = c_int;
pub type sysEventType_t = c_int;

// Stub opaque pointer types (actual definitions expected from platform.h or other modules)
pub type netadr_t = c_void;
pub type msg_t = c_void;
pub type usercmd_t = c_void;

// Windows API types (opaque or mapped from core::ffi)
pub type HINSTANCE = *mut c_void;
pub type HWND = *mut c_void;
pub type LONG = c_int;
pub type UINT = c_uint;
pub type WPARAM = usize;
pub type LPARAM = isize;

// Stub for OSVERSIONINFO from Windows headers
#[repr(C)]
pub struct OSVERSIONINFO {
    pub dwOSVersionInfoSize: c_uint,
    pub dwMajorVersion: c_uint,
    pub dwMinorVersion: c_uint,
    pub dwBuildNumber: c_uint,
    pub dwPlatformId: c_uint,
    pub szCSDVersion: [c_char; 128],
}

extern "C" {
    pub fn IN_MouseEvent(mstate: c_int);

    pub fn Sys_QueEvent(
        time: c_int,
        type_: sysEventType_t,
        value: c_int,
        value2: c_int,
        ptrLength: c_int,
        ptr: *mut c_void,
    );

    pub fn Sys_CreateConsole();
    pub fn Sys_DestroyConsole();

    pub fn Sys_ConsoleInput() -> *mut c_char;

    pub fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean;

    // Input subsystem

    pub fn IN_Init();
    pub fn IN_Shutdown();
    pub fn IN_JoystickCommands();

    pub fn IN_Move(cmd: *mut usercmd_t);
    // add additional non keyboard / non mouse movement on top of the keyboard move cmd

    pub fn IN_DeactivateWin32Mouse();

    pub fn IN_Activate(active: qboolean);
    pub fn IN_Frame();

    // window procedure
    // #ifndef _XBOX
    pub fn MainWndProc(
        hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LONG;
    // #endif

    pub fn Conbuf_AppendText(msg: *const c_char);

    pub fn SNDDMA_Activate(bAppActive: qboolean);
    pub fn SNDDMA_InitDS() -> c_int;
}

// #ifndef _XBOX
#[repr(C)]
pub struct WinVars_t {
    pub reflib_library: HINSTANCE,    // Handle to refresh DLL
    pub reflib_active: qboolean,

    pub hWnd: HWND,
    pub hInstance: HINSTANCE,
    pub activeApp: qboolean,
    pub isMinimized: qboolean,
    pub osversion: OSVERSIONINFO,

    // when we get a windows message, we store the time off so keyboard processing
    // can know the exact time of an event
    pub sysMsgTime: c_uint,
}

extern "C" {
    pub static mut g_wv: WinVars_t;
}
// #endif

pub const MAX_QUED_EVENTS: c_int = 256;
pub const MASK_QUED_EVENTS: c_int = MAX_QUED_EVENTS - 1;
