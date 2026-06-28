// win_local.h: Win32-specific Quake3 header file

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_uint};

// Forward declarations for types defined elsewhere
// sysEventType_t defined in sys/sys.h or similar
// usercmd_t defined in game/bg_public.h or similar
// qboolean defined in qcommon/q_shared.h or similar

// Local stubs for Windows API types
#[cfg(not(target_os = "xbox"))]
pub type HWND = *mut c_void;
#[cfg(not(target_os = "xbox"))]
pub type HINSTANCE = *mut c_void;
#[cfg(not(target_os = "xbox"))]
pub type LONG = c_int;
#[cfg(not(target_os = "xbox"))]
pub type UINT = c_int;
#[cfg(not(target_os = "xbox"))]
pub type WPARAM = usize;
#[cfg(not(target_os = "xbox"))]
pub type LPARAM = isize;

#[cfg(not(target_os = "xbox"))]
#[repr(C)]
pub struct OSVERSIONINFO {
    pub dwOSVersionInfoSize: c_int,
    pub dwMajorVersion: c_int,
    pub dwMinorVersion: c_int,
    pub dwBuildNumber: c_int,
    pub dwPlatformId: c_int,
    pub szCSDVersion: [c_char; 128],
}

extern "C" {
    pub fn IN_MouseEvent(mstate: c_int);
}

extern "C" {
    pub fn Sys_QueEvent(
        time: c_int,
        type_: c_int, // sysEventType_t
        value: c_int,
        value2: c_int,
        ptrLength: c_int,
        ptr: *mut c_void,
    );
}

extern "C" {
    pub fn Sys_CreateConsole();
    pub fn Sys_DestroyConsole();
}

extern "C" {
    pub fn Sys_ConsoleInput() -> *mut c_char;
}

// Input subsystem

extern "C" {
    pub fn IN_Init();
    pub fn IN_Shutdown();
    pub fn IN_JoystickCommands();
}

extern "C" {
    pub fn IN_Move(cmd: *mut c_void); // usercmd_t *cmd
                                       // add additional non keyboard / non mouse movement on top of the keyboard move cmd
}

extern "C" {
    pub fn IN_DeactivateWin32Mouse();
}

extern "C" {
    pub fn IN_Activate(active: c_int); // qboolean active
    pub fn IN_Frame();
}

// window procedure
#[cfg(not(target_os = "xbox"))]
extern "C" {
    pub fn MainWndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LONG;
}

extern "C" {
    pub fn Conbuf_AppendText(msg: *const c_char);
}

extern "C" {
    pub fn SNDDMA_Activate(bAppActive: c_int); // qboolean bAppActive
}

#[cfg(not(target_os = "xbox"))]
#[repr(C)]
pub struct WinVars_t {
    pub hWnd: HWND,
    pub hInstance: HINSTANCE,
    pub activeApp: c_int, // qboolean
    pub isMinimized: c_int, // qboolean
    pub osversion: OSVERSIONINFO,

    // when we get a windows message, we store the time off so keyboard processing
    // can know the exact time of an event
    pub sysMsgTime: c_uint,
}

#[cfg(not(target_os = "xbox"))]
extern "C" {
    pub static mut g_wv: WinVars_t;
}

pub const MAX_QUED_EVENTS: c_int = 256;
pub const MASK_QUED_EVENTS: c_int = MAX_QUED_EVENTS - 1;
