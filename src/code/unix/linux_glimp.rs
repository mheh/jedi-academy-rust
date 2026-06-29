/*
** GLW_IMP.C
**
** This file contains ALL Linux specific stuff having to do with the
** OpenGL refresh.  When a port is being made the following functions
** must be implemented by the port:
**
** GLimp_EndFrame
** GLimp_Init
** GLimp_Shutdown
** GLimp_SwitchFullscreen
**
*/

use core::ffi::{c_int, c_char, c_void, c_ulong};
use std::ffi::CStr;
use std::ptr::{self, null_mut};

// X11 opaque types (forward declarations)
#[allow(non_camel_case_types)]
pub type Display = c_void;
#[allow(non_camel_case_types)]
pub type Window = c_ulong;
#[allow(non_camel_case_types)]
pub type Pixmap = c_ulong;
#[allow(non_camel_case_types)]
pub type Cursor = c_ulong;
#[allow(non_camel_case_types)]
pub type GC = *mut c_void;
#[allow(non_camel_case_types)]
pub type GLXContext = *mut c_void;
#[allow(non_camel_case_types)]
pub type XVisualInfo = c_void;

// qboolean equivalent
pub type qboolean = c_int;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// rserr_t enum
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum rserr_t {
    RSERR_OK = 0,
    RSERR_INVALID_FULLSCREEN = 1,
    RSERR_INVALID_MODE = 2,
    RSERR_UNKNOWN = 3,
}

// Key codes
const K_KP_PGUP: c_int = 0x1001;
const K_PGUP: c_int = 0x1003;
const K_KP_PGDN: c_int = 0x1002;
const K_PGDN: c_int = 0x1004;
const K_KP_HOME: c_int = 0x1007;
const K_HOME: c_int = 0x1008;
const K_KP_END: c_int = 0x1009;
const K_END: c_int = 0x100a;
const K_KP_LEFTARROW: c_int = 0x100b;
const K_LEFTARROW: c_int = 0x100c;
const K_KP_RIGHTARROW: c_int = 0x100d;
const K_RIGHTARROW: c_int = 0x100e;
const K_KP_DOWNARROW: c_int = 0x100f;
const K_DOWNARROW: c_int = 0x1010;
const K_KP_UPARROW: c_int = 0x1011;
const K_UPARROW: c_int = 0x1012;
const K_ESCAPE: c_int = 0x1b;
const K_KP_ENTER: c_int = 0x100d;
const K_ENTER: c_int = 0x0d;
const K_TAB: c_int = 0x09;
const K_F1: c_int = 0x70;
const K_F2: c_int = 0x71;
const K_F3: c_int = 0x72;
const K_F4: c_int = 0x73;
const K_F5: c_int = 0x74;
const K_F6: c_int = 0x75;
const K_F7: c_int = 0x76;
const K_F8: c_int = 0x77;
const K_F9: c_int = 0x78;
const K_F10: c_int = 0x79;
const K_F11: c_int = 0x7a;
const K_F12: c_int = 0x7b;
const K_KP_DEL: c_int = 0x100f;
const K_DEL: c_int = 0x100b;
const K_PAUSE: c_int = 0x1013;
const K_SHIFT: c_int = 0x1014;
const K_CTRL: c_int = 0x1015;
const K_ALT: c_int = 0x1016;
const K_KP_5: c_int = 0x100c;
const K_INS: c_int = 0x1009;
const K_KP_INS: c_int = 0x100a;
const K_KP_PLUS: c_int = 0x100b;
const K_KP_MINUS: c_int = 0x100c;
const K_KP_SLASH: c_int = 0x100d;
const K_MOUSE1: c_int = 0x1100;

// X11 event masks
const KEY_MASK: c_ulong = 1 | 2; // KeyPressMask | KeyReleaseMask
const MOUSE_MASK: c_ulong = 4 | 8 | 64 | 32; // ButtonPressMask | ButtonReleaseMask | PointerMotionMask | ButtonMotionMask
const X_MASK: c_ulong = KEY_MASK | MOUSE_MASK | 8 | 16; // VisibilityChangeMask | StructureNotifyMask

// System event types
const SE_KEY: c_int = 0;
const SE_CHAR: c_int = 1;
const SE_MOUSE: c_int = 2;

// X11 KeySym constants
const XK_KP_Page_Up: c_int = 0xff9b;
const XK_KP_9: c_int = 0xff9c;
const XK_Page_Up: c_int = 0xff55;
const XK_KP_Page_Down: c_int = 0xff9a;
const XK_KP_3: c_int = 0xff9c;
const XK_Page_Down: c_int = 0xff56;
const XK_KP_Home: c_int = 0xff95;
const XK_KP_7: c_int = 0xff97;
const XK_Home: c_int = 0xff50;
const XK_KP_End: c_int = 0xff9c;
const XK_KP_1: c_int = 0xff9c;
const XK_End: c_int = 0xff57;
const XK_KP_Left: c_int = 0xff96;
const XK_KP_4: c_int = 0xff96;
const XK_Left: c_int = 0xff51;
const XK_KP_Right: c_int = 0xff98;
const XK_KP_6: c_int = 0xff98;
const XK_Right: c_int = 0xff53;
const XK_KP_Down: c_int = 0xff99;
const XK_KP_2: c_int = 0xff99;
const XK_Down: c_int = 0xff54;
const XK_KP_Up: c_int = 0xff97;
const XK_KP_8: c_int = 0xff97;
const XK_Up: c_int = 0xff52;
const XK_Escape: c_int = 0xff1b;
const XK_KP_Enter: c_int = 0xff8d;
const XK_Return: c_int = 0xff0d;
const XK_Tab: c_int = 0xff09;
const XK_F1: c_int = 0xffbe;
const XK_F2: c_int = 0xffbf;
const XK_F3: c_int = 0xffc0;
const XK_F4: c_int = 0xffc1;
const XK_F5: c_int = 0xffc2;
const XK_F6: c_int = 0xffc3;
const XK_F7: c_int = 0xffc4;
const XK_F8: c_int = 0xffc5;
const XK_F9: c_int = 0xffc6;
const XK_F10: c_int = 0xffc7;
const XK_F11: c_int = 0xffc8;
const XK_F12: c_int = 0xffc9;
const XK_BackSpace: c_int = 0xff08;
const XK_KP_Delete: c_int = 0xff9f;
const XK_KP_Decimal: c_int = 0xff9f;
const XK_Delete: c_int = 0xffff;
const XK_Pause: c_int = 0xff13;
const XK_Shift_L: c_int = 0xffe1;
const XK_Shift_R: c_int = 0xffe2;
const XK_Execute: c_int = 0xff62;
const XK_Control_L: c_int = 0xffe3;
const XK_Control_R: c_int = 0xffe4;
const XK_Alt_L: c_int = 0xffe9;
const XK_Meta_L: c_int = 0xffe7;
const XK_Alt_R: c_int = 0xffea;
const XK_Meta_R: c_int = 0xffe8;
const XK_KP_Begin: c_int = 0xff9d;
const XK_Insert: c_int = 0xff63;
const XK_KP_Insert: c_int = 0xff9e;
const XK_KP_0: c_int = 0xff9e;
const XK_KP_Multiply: c_int = 0xffaa;
const XK_KP_Add: c_int = 0xffab;
const XK_KP_Subtract: c_int = 0xffad;
const XK_KP_Divide: c_int = 0xffaf;

// GLX constants
const GLX_RGBA: c_int = 4;
const GLX_RED_SIZE: c_int = 5;
const GLX_GREEN_SIZE: c_int = 6;
const GLX_BLUE_SIZE: c_int = 7;
const GLX_DOUBLEBUFFER: c_int = 5;
const GLX_DEPTH_SIZE: c_int = 12;
const GLX_STENCIL_SIZE: c_int = 13;

// GL constants
const GL_VENDOR: c_int = 0x1f00;
const GL_RENDERER: c_int = 0x1f01;
const GL_VERSION: c_int = 0x1f02;
const GL_EXTENSIONS: c_int = 0x1f03;
const GL_NO_ERROR: c_int = 0;

// TextureCompression enum values
const TC_NONE: c_int = 0;
const TC_S3TC: c_int = 1;
const TC_S3TC_DXT: c_int = 2;

// Cvar constants
const CVAR_ARCHIVE: c_int = 1;

// Print levels
const PRINT_ALL: c_int = 0;
const PRINT_DEVELOPER: c_int = 1;

// Error levels
const ERR_FATAL: c_int = 1;

// Grab modes
const GrabModeSync: c_int = 0;
const GrabModeAsync: c_int = 1;

// Window attributes
const CWBackPixel: c_ulong = 1;
const CWBorderPixel: c_ulong = 2;
const CWColormap: c_ulong = 4;
const CWSaveUnder: c_ulong = 0x400;
const CWBackingStore: c_ulong = 0x800;
const CWEventMask: c_ulong = 0x4000;
const CWOverrideRedirect: c_ulong = 0x8000;

// Other constants
const True: c_int = 1;
const False: c_int = 0;
const NotUseful: c_int = 0;
const None: c_ulong = 0;
const CurrentTime: c_ulong = 0;

// Connection flags
const CVAR_ARCHIVE: c_int = 1;

// Structures
#[derive(Clone, Copy)]
#[repr(C)]
pub struct XGCValues {
    function: c_int,
    plane_mask: c_ulong,
    foreground: c_ulong,
    background: c_ulong,
    line_width: c_int,
    line_style: c_int,
    cap_style: c_int,
    join_style: c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XColor {
    pixel: c_ulong,
    red: c_int,
    green: c_int,
    blue: c_int,
    flags: c_char,
    pad: c_char,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XSetWindowAttributes {
    background_pixmap: Pixmap,
    background_pixel: c_ulong,
    border_pixmap: Pixmap,
    border_pixel: c_ulong,
    bit_gravity: c_int,
    win_gravity: c_int,
    backing_store: c_int,
    backing_planes: c_ulong,
    backing_pixel: c_ulong,
    save_under: c_int,
    event_mask: c_ulong,
    do_not_propagate_mask: c_ulong,
    colormap: c_ulong,
    cursor: Cursor,
    override_redirect: c_int,
}

// XKeyEvent structure (part of XEvent)
#[derive(Clone, Copy)]
#[repr(C)]
pub struct XKeyEvent {
    event_type: c_int,
    display: *mut Display,
    window: Window,
    root: Window,
    subwindow: Window,
    time: c_ulong,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_int,
    keycode: c_int,
    same_screen: c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XMotionEvent {
    event_type: c_int,
    display: *mut Display,
    window: Window,
    root: Window,
    subwindow: Window,
    time: c_ulong,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_int,
    is_hint: c_char,
    same_screen: c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XButtonEvent {
    event_type: c_int,
    display: *mut Display,
    window: Window,
    root: Window,
    subwindow: Window,
    time: c_ulong,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_int,
    button: c_int,
    same_screen: c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XCreateWindowEvent {
    event_type: c_int,
    display: *mut Display,
    parent: Window,
    window: Window,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    border_width: c_int,
    override_redirect: c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XConfigureEvent {
    event_type: c_int,
    display: *mut Display,
    event: Window,
    window: Window,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    border_width: c_int,
    above: Window,
    override_redirect: c_int,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union XEvent {
    xtype: c_int,
    xkey: XKeyEvent,
    xmotion: XMotionEvent,
    xbutton: XButtonEvent,
    xcreatewindow: XCreateWindowEvent,
    xconfigure: XConfigureEvent,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XF86VidModeModeInfo {
    dotclock: c_int,
    hdisplay: c_int,
    hsyncstart: c_int,
    hsyncend: c_int,
    htotal: c_int,
    vdisplay: c_int,
    vsyncstart: c_int,
    vsyncend: c_int,
    vtotal: c_int,
    flags: c_int,
    privsize: c_int,
    priv: *mut c_void,
}

// glConfig structure (external, declared here as opaque)
#[allow(non_camel_case_types)]
pub struct glconfig_t {
    // This is a placeholder - actual structure would be in tr_local.h
}

// glState structure (external, declared here as opaque)
#[allow(non_camel_case_types)]
pub struct glstate_t {
    // This is a placeholder - actual structure would be defined elsewhere
}

// glwstate_t structure (from unix_glw.h)
#[allow(non_camel_case_types)]
pub struct glwstate_t {
    log_fp: *mut c_void,
    OpenGLLib: *mut c_void,
}

// cvar_t structure (external)
#[allow(non_camel_case_types)]
pub struct cvar_t {
    name: *const c_char,
    string: *const c_char,
    value: f32,
    integer: c_int,
    modified: qboolean,
}

// External globals and structures
extern "C" {
    pub static mut glw_state: glwstate_t;
    pub static mut glConfig: glconfig_t;
    pub static mut glState: glstate_t;
    pub static mut r_colorbits: *mut cvar_t;
    pub static mut r_depthbits: *mut cvar_t;
    pub static mut r_stencilbits: *mut cvar_t;
    pub static mut r_glDriver: *mut cvar_t;
    pub static mut r_fullscreen: *mut cvar_t;
    pub static mut r_fakeFullscreen: *mut cvar_t;
    pub static mut r_mode: *mut cvar_t;
    pub static mut r_ext_compressed_textures: *mut cvar_t;
    pub static mut r_ext_multitexture: *mut cvar_t;
    pub static mut r_ext_texture_filter_anisotropic: *mut cvar_t;
    pub static mut r_ext_compiled_vertex_array: *mut cvar_t;
    pub static mut r_drawBuffer: *mut cvar_t;
    pub static mut r_logFile: *mut cvar_t;
    pub static mut r_texturebits: *mut cvar_t;
    pub static mut cls: libc::c_void; // Placeholder
}

// X11 functions
extern "C" {
    pub fn XOpenDisplay(display_name: *const c_char) -> *mut Display;
    pub fn XCloseDisplay(display: *mut Display) -> c_int;
    pub fn XLookupString(event: *mut XKeyEvent, buffer: *mut c_char, buffer_size: c_int, keysym: *mut c_int, status: *mut c_void) -> c_int;
    pub fn XCreatePixmap(display: *mut Display, drawable: Window, width: c_int, height: c_int, depth: c_int) -> Pixmap;
    pub fn XCreateGC(display: *mut Display, drawable: Window, valuemask: c_ulong, values: *mut XGCValues) -> GC;
    pub fn XFillRectangle(display: *mut Display, drawable: Window, gc: GC, x: c_int, y: c_int, width: c_int, height: c_int) -> c_int;
    pub fn XCreatePixmapCursor(display: *mut Display, source: Pixmap, mask: Pixmap, foreground: *mut XColor, background: *mut XColor, x: c_int, y: c_int) -> Cursor;
    pub fn XFreePixmap(display: *mut Display, pixmap: Pixmap) -> c_int;
    pub fn XFreeGC(display: *mut Display, gc: GC) -> c_int;
    pub fn XDefineCursor(display: *mut Display, window: Window, cursor: Cursor) -> c_int;
    pub fn XGrabPointer(display: *mut Display, grab_window: Window, owner_events: c_int, event_mask: c_ulong, pointer_mode: c_int, keyboard_mode: c_int, confine_to: Window, cursor: Cursor, time: c_ulong) -> c_int;
    pub fn XGetPointerControl(display: *mut Display, accel_num: *mut c_int, accel_denom: *mut c_int, threshold: *mut c_int) -> c_int;
    pub fn XChangePointerControl(display: *mut Display, do_accel: c_int, do_threshold: c_int, accel_num: c_int, accel_denom: c_int, threshold: c_int) -> c_int;
    pub fn XWarpPointer(display: *mut Display, src_window: Window, dest_window: Window, src_x: c_int, src_y: c_int, src_width: c_int, src_height: c_int, dest_x: c_int, dest_y: c_int) -> c_int;
    pub fn XGrabKeyboard(display: *mut Display, grab_window: Window, owner_events: c_int, pointer_mode: c_int, keyboard_mode: c_int, time: c_ulong) -> c_int;
    pub fn XUngrabPointer(display: *mut Display, time: c_ulong) -> c_int;
    pub fn XUngrabKeyboard(display: *mut Display, time: c_ulong) -> c_int;
    pub fn XUndefineCursor(display: *mut Display, window: Window) -> c_int;
    pub fn XAutoRepeatOn(display: *mut Display) -> c_int;
    pub fn XAutoRepeatOff(display: *mut Display) -> c_int;
    pub fn XSync(display: *mut Display, discard: c_int) -> c_int;
    pub fn XPending(display: *mut Display) -> c_int;
    pub fn XNextEvent(display: *mut Display, event: *mut XEvent) -> c_int;
    pub fn XCreateWindow(display: *mut Display, parent: Window, x: c_int, y: c_int, width: c_int, height: c_int, border_width: c_int, depth: c_int, class: c_int, visual: *mut c_void, valuemask: c_ulong, attributes: *mut XSetWindowAttributes) -> Window;
    pub fn XDestroyWindow(display: *mut Display, window: Window) -> c_int;
    pub fn XMapWindow(display: *mut Display, window: Window) -> c_int;
    pub fn XMoveWindow(display: *mut Display, window: Window, x: c_int, y: c_int) -> c_int;
    pub fn XCreateColormap(display: *mut Display, window: Window, visual: *mut c_void, alloc: c_int) -> c_ulong;
    pub fn XFlush(display: *mut Display) -> c_int;
    pub fn XF86VidModeQueryVersion(display: *mut Display, major: *mut c_int, minor: *mut c_int) -> c_int;
    pub fn XF86VidModeGetAllModeLines(display: *mut Display, screen: c_int, modecount: *mut c_int, modelines: *mut *mut *mut XF86VidModeModeInfo) -> c_int;
    pub fn XF86VidModeSwitchToMode(display: *mut Display, screen: c_int, modeline: *mut XF86VidModeModeInfo) -> c_int;
    pub fn XF86VidModeSetViewPort(display: *mut Display, screen: c_int, x: c_int, y: c_int) -> c_int;
    pub fn XF86DGAQueryVersion(display: *mut Display, major: *mut c_int, minor: *mut c_int) -> c_int;
    pub fn XF86DGADirectVideo(display: *mut Display, screen: c_int, flags: c_int) -> c_int;
}

// GLX functions
extern "C" {
    pub fn qglXChooseVisual(display: *mut Display, screen: c_int, attrib_list: *mut c_int) -> *mut XVisualInfo;
    pub fn qglXCreateContext(display: *mut Display, visual: *mut XVisualInfo, shareList: *mut GLXContext, direct: c_int) -> *mut GLXContext;
    pub fn qglXMakeCurrent(display: *mut Display, drawable: Window, ctx: *mut GLXContext) -> c_int;
    pub fn qglXDestroyContext(display: *mut Display, ctx: *mut GLXContext) -> c_int;
    pub fn qglXSwapBuffers(display: *mut Display, drawable: Window) -> c_int;
    pub fn qglGetString(name: c_int) -> *const c_char;
    pub fn qglGetError() -> c_int;
    pub fn qglFinish() -> c_void;
}

// Renderer interface functions (ri namespace)
extern "C" {
    pub fn ri_Printf(print_level: c_int, fmt: *const c_char, ...);
    pub fn ri_Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn ri_Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn ri_Error(error_level: c_int, fmt: *const c_char, ...);
}

// QGL functions
extern "C" {
    pub fn QGL_Init(name: *const c_char) -> qboolean;
    pub fn QGL_Shutdown();
    pub fn QGL_EnableLogging(log: c_int);
}

// System functions
extern "C" {
    pub fn R_GetModeInfo(width: *mut c_int, height: *mut c_int, aspect: *mut f32, mode: c_int) -> qboolean;
    pub fn Sys_QueEvent(time: c_int, event_type: c_int, value: c_int, value2: c_int, ptrlen: c_int, ptr: *mut c_void);
    pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    pub fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
}

// libc functions
extern "C" {
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub fn signal(sig: c_int, handler: unsafe extern "C" fn(c_int)) -> unsafe extern "C" fn(c_int);
    pub fn _exit(status: c_int) -> !;
    pub fn printf(fmt: *const c_char, ...) -> c_int;
    pub fn fprintf(stream: *mut c_void, fmt: *const c_char, ...) -> c_int;
    pub fn putenv(string: *const c_char) -> c_int;
}

// semaphore functions
extern "C" {
    pub fn sem_init(sem: *mut c_void, pshared: c_int, value: c_int) -> c_int;
    pub fn sem_post(sem: *mut c_void) -> c_int;
    pub fn sem_wait(sem: *mut c_void) -> c_int;
}

// pthread functions
extern "C" {
    pub fn pthread_create(thread: *mut c_void, attr: *mut c_void, start_routine: unsafe extern "C" fn(*mut c_void), arg: *mut c_void) -> c_int;
}

// Helper macros/functions (local stubs or implementations)
fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe { strcmp(s1, s2) }
}

fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize) {
    unsafe {
        let mut i = 0;
        while i < size - 1 {
            let byte = *(src.add(i)) as u8;
            if byte == 0 {
                break;
            }
            *dest.add(i) = byte as c_char;
            i += 1;
        }
        *dest.add(i) = 0;
    }
}

fn strlwr(s: *mut c_char) {
    unsafe {
        let mut i = 0;
        loop {
            let byte = *s.add(i) as u8;
            if byte == 0 {
                break;
            }
            if byte >= b'A' && byte <= b'Z' {
                *s.add(i) = (byte - b'A' + b'a') as c_char;
            }
            i += 1;
        }
    }
}

fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut i = 0;
        loop {
            let b1 = *s1.add(i) as u8;
            let b2 = *s2.add(i) as u8;

            let c1 = if b1 >= b'A' && b1 <= b'Z' { b1 - b'A' + b'a' } else { b1 };
            let c2 = if b2 >= b'A' && b2 <= b'Z' { b2 - b'A' + b'a' } else { b2 };

            if c1 != c2 {
                return (c1 as c_int) - (c2 as c_int);
            }

            if c1 == 0 {
                break;
            }

            i += 1;
        }
        return 0;
    }
}

fn DefaultScreen(display: *mut Display) -> c_int {
    // This is typically a macro that accesses display->default_screen
    // For now, return 0 as a reasonable default
    0
}

fn RootWindow(display: *mut Display, screen: c_int) -> Window {
    // This would typically access the display's root window
    // Placeholder implementation
    1
}

fn BlackPixel(display: *mut Display, screen: c_int) -> c_ulong {
    0
}

fn AllocNone() -> c_int {
    0
}

fn InputOutput() -> c_int {
    1
}

fn GXclear() -> c_int {
    0
}

// Global variables
static mut dpy: *mut Display = ptr::null_mut();
static mut scrnum: c_int = 0;
static mut win: Window = 0;
static mut ctx: *mut GLXContext = ptr::null_mut();

static mut autorepeaton: qboolean = qtrue;

static mut mouse_avail: qboolean = qfalse;
static mut mouse_active: qboolean = qfalse;
static mut mx: c_int = 0;
static mut my: c_int = 0;

static mut in_mouse: *mut cvar_t = ptr::null_mut();
static mut in_dgamouse: *mut cvar_t = ptr::null_mut();

static mut r_fakeFullscreen: *mut cvar_t = ptr::null_mut();

pub static mut dgamouse: qboolean = qfalse;
pub static mut vidmode_ext: qboolean = qfalse;

static mut win_x: c_int = 0;
static mut win_y: c_int = 0;

static mut vidmodes: *mut *mut XF86VidModeModeInfo = ptr::null_mut();
static mut default_dotclock_vidmode: c_int = 0;
static mut num_vidmodes: c_int = 0;
static mut vidmode_active: qboolean = qfalse;

static mut mouse_accel_numerator: c_int = 0;
static mut mouse_accel_denominator: c_int = 0;
static mut mouse_threshold: c_int = 0;

/*****************************************************************************/
/* KEYBOARD                                                                  */
/*****************************************************************************/

static mut keyshift: [c_int; 256] = [0; 256]; // key to map to if shift held down in console
static mut shift_down: qboolean = qfalse;

unsafe fn XLateKey(ev: *mut XKeyEvent, key: *mut c_int) -> *mut c_char {
    static mut buf: [c_char; 64] = [0; 64];
    let mut keysym: c_int = 0;
    static mut setup: qboolean = qfalse;
    let mut i: c_int = 0;

    *key = 0;

    XLookupString(ev, buf.as_mut_ptr(), 64, &mut keysym, ptr::null_mut());

    // ri.Printf( PRINT_ALL, "keysym=%04X\n", (int)keysym);
    match keysym {
        XK_KP_Page_Up | XK_KP_9 => *key = K_KP_PGUP,
        XK_Page_Up => *key = K_PGUP,

        XK_KP_Page_Down | XK_KP_3 => *key = K_KP_PGDN,
        XK_Page_Down => *key = K_PGDN,

        XK_KP_Home => *key = K_KP_HOME,
        XK_KP_7 => *key = K_KP_HOME,
        XK_Home => *key = K_HOME,

        XK_KP_End | XK_KP_1 => *key = K_KP_END,
        XK_End => *key = K_END,

        XK_KP_Left => *key = K_KP_LEFTARROW,
        XK_KP_4 => *key = K_KP_LEFTARROW,
        XK_Left => *key = K_LEFTARROW,

        XK_KP_Right => *key = K_KP_RIGHTARROW,
        XK_KP_6 => *key = K_KP_RIGHTARROW,
        XK_Right => *key = K_RIGHTARROW,

        XK_KP_Down | XK_KP_2 => *key = K_KP_DOWNARROW,
        XK_Down => *key = K_DOWNARROW,

        XK_KP_Up | XK_KP_8 => *key = K_KP_UPARROW,
        XK_Up => *key = K_UPARROW,

        XK_Escape => *key = K_ESCAPE,

        XK_KP_Enter => *key = K_KP_ENTER,
        XK_Return => *key = K_ENTER,

        XK_Tab => *key = K_TAB,

        XK_F1 => *key = K_F1,
        XK_F2 => *key = K_F2,
        XK_F3 => *key = K_F3,
        XK_F4 => *key = K_F4,
        XK_F5 => *key = K_F5,
        XK_F6 => *key = K_F6,
        XK_F7 => *key = K_F7,
        XK_F8 => *key = K_F8,
        XK_F9 => *key = K_F9,
        XK_F10 => *key = K_F10,
        XK_F11 => *key = K_F11,
        XK_F12 => *key = K_F12,

        // case XK_BackSpace: *key = K_BACKSPACE; break;
        XK_BackSpace => *key = 8, // ctrl-h

        XK_KP_Delete | XK_KP_Decimal => *key = K_KP_DEL,
        XK_Delete => *key = K_DEL,

        XK_Pause => *key = K_PAUSE,

        XK_Shift_L | XK_Shift_R => *key = K_SHIFT,

        XK_Execute | XK_Control_L | XK_Control_R => *key = K_CTRL,

        XK_Alt_L | XK_Meta_L | XK_Alt_R | XK_Meta_R => *key = K_ALT,

        XK_KP_Begin => *key = K_KP_5,

        XK_Insert => *key = K_INS,
        XK_KP_Insert | XK_KP_0 => *key = K_KP_INS,

        XK_KP_Multiply => *key = '*' as c_int,
        XK_KP_Add => *key = K_KP_PLUS,
        XK_KP_Subtract => *key = K_KP_MINUS,
        XK_KP_Divide => *key = K_KP_SLASH,

        _ => {
            *key = *(buf.as_ptr() as *const c_char as *const u8) as c_int;
            if *key >= 'A' as c_int && *key <= 'Z' as c_int {
                *key = *key - 'A' as c_int + 'a' as c_int;
            }
        }
    }

    buf.as_mut_ptr()
}

// ========================================================================
// makes a null cursor
// ========================================================================

unsafe fn CreateNullCursor(display: *mut Display, root: Window) -> Cursor {
    let mut cursormask: Pixmap;
    let mut xgc: XGCValues;
    let mut gc: GC;
    let mut dummycolour: XColor;
    let mut cursor: Cursor;

    cursormask = XCreatePixmap(display, root, 1, 1, 1);
    xgc.function = GXclear();
    gc = XCreateGC(display, cursormask, 0x1, &mut xgc); // GCFunction
    XFillRectangle(display, cursormask, gc, 0, 0, 1, 1);
    dummycolour.pixel = 0;
    dummycolour.red = 0;
    dummycolour.flags = 04 as c_char;
    cursor = XCreatePixmapCursor(display, cursormask, cursormask,
          &mut dummycolour, &mut dummycolour, 0, 0);
    XFreePixmap(display, cursormask);
    XFreeGC(display, gc);
    cursor
}

unsafe fn install_grabs() {
    // inviso cursor
    XDefineCursor(dpy, win, CreateNullCursor(dpy, win));

    XGrabPointer(dpy, win,
                 False,
                 MOUSE_MASK,
                 GrabModeAsync, GrabModeAsync,
                 win,
                 None,
                 CurrentTime);

    XGetPointerControl(dpy, &mut mouse_accel_numerator, &mut mouse_accel_denominator,
        &mut mouse_threshold);

    XChangePointerControl(dpy, qtrue, qtrue, 2, 1, 0);

    if (*in_dgamouse).value > 0.0 {
        let mut MajorVersion: c_int = 0;
        let mut MinorVersion: c_int = 0;

        if XF86DGAQueryVersion(dpy, &mut MajorVersion, &mut MinorVersion) == 0 {
            // unable to query, probalby not supported
            // ri.Printf( PRINT_ALL, "Failed to detect XF86DGA Mouse\n" );
            // ri.Cvar_Set( "in_dgamouse", "0" );
        } else {
            dgamouse = qtrue;
            XF86DGADirectVideo(dpy, DefaultScreen(dpy), 1); // XF86DGADirectMouse
            XWarpPointer(dpy, None, win, 0, 0, 0, 0, 0, 0);
        }
    } else {
        XWarpPointer(dpy, None, win,
                     0, 0, 0, 0,
                     0, 0); // glConfig.vidWidth / 2, glConfig.vidHeight / 2
    }

    XGrabKeyboard(dpy, win,
                  False,
                  GrabModeAsync, GrabModeAsync,
                  CurrentTime);

    // XSync(dpy, True);
}

unsafe fn uninstall_grabs() {
    if dgamouse != qfalse {
        dgamouse = qfalse;
        XF86DGADirectVideo(dpy, DefaultScreen(dpy), 0);
    }

    XChangePointerControl(dpy, qtrue, qtrue, mouse_accel_numerator,
        mouse_accel_denominator, mouse_threshold);

    XUngrabPointer(dpy, CurrentTime);
    XUngrabKeyboard(dpy, CurrentTime);

    // inviso cursor
    XUndefineCursor(dpy, win);

    // XAutoRepeatOn(dpy);
    // XSync(dpy, True);
}

unsafe fn HandleEvents() {
    let mut b: c_int;
    let mut key: c_int = 0;
    let mut event: XEvent;
    let mut dowarp: qboolean = qfalse;
    let mut mwx: c_int = 0;
    let mut mwy: c_int = 0;
    let mut p: *mut c_char;

    if dpy.is_null() {
        return;
    }

    while XPending(dpy) != 0 {
        XNextEvent(dpy, &mut event);
        match event.xtype {
            2 => { // KeyPress
                p = XLateKey(&mut event.xkey, &mut key);
                if key != 0 {
                    Sys_QueEvent(0, SE_KEY, key, qtrue, 0, ptr::null_mut());
                }
                while *p != 0 {
                    Sys_QueEvent(0, SE_CHAR, *p as c_int, 0, 0, ptr::null_mut());
                    p = p.add(1);
                }
            }
            3 => { // KeyRelease
                XLateKey(&mut event.xkey, &mut key);
                Sys_QueEvent(0, SE_KEY, key, qfalse, 0, ptr::null_mut());
            }

            // #if 0
            // case KeyPress:
            // case KeyRelease:
            //     key = XLateKey(&event.xkey);
            //     Sys_QueEvent( 0, SE_KEY, key, event.type == KeyPress, 0, NULL );
            //     if (key == K_SHIFT)
            //         shift_down = (event.type == KeyPress);
            //     if (key < 128 && (event.type == KeyPress)) {
            //         if (shift_down)
            //             key = keyshift[key];
            //         Sys_QueEvent( 0, SE_CHAR, key, 0, 0, NULL );
            //     }
            // #endif

            6 => { // MotionNotify
                if mouse_active != qfalse {
                    if dgamouse != qfalse {
                        if event.xmotion.x_root.abs() > 1 {
                            mx += event.xmotion.x_root * 2;
                        } else {
                            mx += event.xmotion.x_root;
                        }
                        if event.xmotion.y_root.abs() > 1 {
                            my += event.xmotion.y_root * 2;
                        } else {
                            my += event.xmotion.y_root;
                        }
                        // ri.Printf(PRINT_ALL, "mouse (%d,%d) (root=%d,%d)\n", event.xmotion.x + win_x, event.xmotion.y + win_y, event.xmotion.x_root, event.xmotion.y_root);
                    } else {
                        // ri.Printf(PRINT_ALL, "mouse x=%d,y=%d\n", (int)event.xmotion.x - mwx, (int)event.xmotion.y - mwy);
                        mx += event.xmotion.x - mwx;
                        my += event.xmotion.y - mwy;
                        mwx = event.xmotion.x;
                        mwy = event.xmotion.y;

                        if mx != 0 || my != 0 {
                            dowarp = qtrue;
                        }
                    }
                }
            }

            4 => { // ButtonPress
                b = -1;
                if event.xbutton.button == 1 {
                    b = 0;
                } else if event.xbutton.button == 2 {
                    b = 2;
                } else if event.xbutton.button == 3 {
                    b = 1;
                }
                Sys_QueEvent(0, SE_KEY, K_MOUSE1 + b, qtrue, 0, ptr::null_mut());
            }

            5 => { // ButtonRelease
                b = -1;
                if event.xbutton.button == 1 {
                    b = 0;
                } else if event.xbutton.button == 2 {
                    b = 2;
                } else if event.xbutton.button == 3 {
                    b = 1;
                }
                Sys_QueEvent(0, SE_KEY, K_MOUSE1 + b, qfalse, 0, ptr::null_mut());
            }

            10 => { // CreateNotify
                win_x = event.xcreatewindow.x;
                win_y = event.xcreatewindow.y;
            }

            12 => { // ConfigureNotify
                win_x = event.xconfigure.x;
                win_y = event.xconfigure.y;
            }

            _ => {}
        }
    }

    if dowarp != qfalse {
        /* move the mouse to the window center again */
        XWarpPointer(dpy, None, win, 0, 0, 0, 0,
                0, 0); // (glConfig.vidWidth/2),(glConfig.vidHeight/2)
    }
}

pub unsafe fn KBD_Init() {
}

pub unsafe fn KBD_Close() {
}

pub unsafe fn IN_ActivateMouse() {
    if mouse_avail == qfalse || dpy.is_null() || win == 0 {
        return;
    }

    if mouse_active == qfalse {
        mx = 0;
        my = 0; // don't spazz
        install_grabs();
        mouse_active = qtrue;
    }
}

pub unsafe fn IN_DeactivateMouse() {
    if mouse_avail == qfalse || dpy.is_null() || win == 0 {
        return;
    }

    if mouse_active != qfalse {
        uninstall_grabs();
        mouse_active = qfalse;
    }
}

/*****************************************************************************/

static mut signalcaught: qboolean = qfalse;

unsafe extern "C" fn signal_handler(sig: c_int) {
    if signalcaught != qfalse {
        printf(b"DOUBLE SIGNAL FAULT: Received signal %d, exiting...\n\0".as_ptr() as *const c_char, sig);
        _exit(1);
    }

    signalcaught = qtrue;
    printf(b"Received signal %d, exiting...\n\0".as_ptr() as *const c_char, sig);
    GLimp_Shutdown();
    _exit(1);
}

unsafe fn InitSig() {
    signal(1, signal_handler); // SIGHUP
    signal(3, signal_handler); // SIGQUIT
    signal(4, signal_handler); // SIGILL
    signal(5, signal_handler); // SIGTRAP
    signal(6, signal_handler); // SIGIOT
    signal(7, signal_handler); // SIGBUS
    signal(8, signal_handler); // SIGFPE
    signal(11, signal_handler); // SIGSEGV
    signal(15, signal_handler); // SIGTERM
}

/*
** GLimp_SetGamma
**
** This routine should only be called if glConfig.deviceSupportsGamma is TRUE
*/
pub unsafe fn GLimp_SetGamma(_red: *mut c_char, _green: *mut c_char, _blue: *mut c_char) {
}

/*
** GLimp_Shutdown
**
** This routine does all OS specific shutdown procedures for the OpenGL
** subsystem.  Under OpenGL this means NULLing out the current DC and
** HGLRC, deleting the rendering context, and releasing the DC acquired
** for the window.  The state structure is also nulled out.
**
*/
pub unsafe fn GLimp_Shutdown() {
    if ctx.is_null() || dpy.is_null() {
        return;
    }
    IN_DeactivateMouse();
    XAutoRepeatOn(dpy);
    if !dpy.is_null() {
        if !ctx.is_null() {
            qglXDestroyContext(dpy, ctx);
        }
        if win != 0 {
            XDestroyWindow(dpy, win);
        }
        if vidmode_active != qfalse {
            XF86VidModeSwitchToMode(dpy, scrnum, *vidmodes);
        }
        XCloseDisplay(dpy);
    }
    vidmode_active = qfalse;
    dpy = ptr::null_mut();
    win = 0;
    ctx = ptr::null_mut();

    memset(ptr::addr_of_mut!(glConfig) as *mut c_void, 0, std::mem::size_of_val(&glConfig));
    memset(ptr::addr_of_mut!(glState) as *mut c_void, 0, std::mem::size_of_val(&glState));

    QGL_Shutdown();
}

/*
** GLimp_LogComment
*/
pub unsafe fn GLimp_LogComment(comment: *mut c_char) {
    if !glw_state.log_fp.is_null() {
        fprintf(glw_state.log_fp, b"%s\0".as_ptr() as *const c_char, comment);
    }
}

/*
** GLW_StartDriverAndSetMode
*/
unsafe fn GLW_StartDriverAndSetMode(drivername: *const c_char,
                                   mode: c_int,
                                   fullscreen: qboolean) -> qboolean {
    let mut err: rserr_t;

    // don't ever bother going into fullscreen with a voodoo card
    // #if 1	// JDC: I reenabled this
    if !strstr(drivername, b"Voodoo\0".as_ptr() as *const c_char).is_null() {
        // ri.Cvar_Set( "r_fullscreen", "0" );
        // r_fullscreen->modified = qfalse;
        // fullscreen = qfalse;
    }
    // #endif

    err = GLW_SetMode(drivername, mode, fullscreen);

    match err {
        rserr_t::RSERR_INVALID_FULLSCREEN => {
            // ri.Printf( PRINT_ALL, "...WARNING: fullscreen unavailable in this mode\n" );
            return qfalse;
        }
        rserr_t::RSERR_INVALID_MODE => {
            // ri.Printf( PRINT_ALL, "...WARNING: could not set the given mode (%d)\n", mode );
            return qfalse;
        }
        _ => {}
    }
    return qtrue;
}

/*
** GLW_SetMode
*/
unsafe fn GLW_SetMode(drivername: *const c_char, mode: c_int, fullscreen: qboolean) -> rserr_t {
    let mut attrib: [c_int; 15] = [
        GLX_RGBA,                  // 0
        GLX_RED_SIZE, 4,            // 1, 2
        GLX_GREEN_SIZE, 4,          // 3, 4
        GLX_BLUE_SIZE, 4,           // 5, 6
        GLX_DOUBLEBUFFER,           // 7
        GLX_DEPTH_SIZE, 1,          // 8, 9
        GLX_STENCIL_SIZE, 1,        // 10, 11
        0,                          // None
        0,
        0,
    ];
    // these match in the array
    const ATTR_RED_IDX: usize = 2;
    const ATTR_GREEN_IDX: usize = 4;
    const ATTR_BLUE_IDX: usize = 6;
    const ATTR_DEPTH_IDX: usize = 9;
    const ATTR_STENCIL_IDX: usize = 11;

    let mut root: Window;
    let mut visinfo: *mut XVisualInfo;
    let mut attr: XSetWindowAttributes;
    let mut mask: c_ulong;
    let mut colorbits: c_int;
    let mut depthbits: c_int;
    let mut stencilbits: c_int;
    let mut tcolorbits: c_int;
    let mut tdepthbits: c_int;
    let mut tstencilbits: c_int;
    let mut MajorVersion: c_int;
    let mut MinorVersion: c_int;
    let mut actualWidth: c_int;
    let mut actualHeight: c_int;
    let mut i: c_int;

    r_fakeFullscreen = ri_Cvar_Get(b"r_fakeFullscreen\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

    // ri.Printf( PRINT_ALL, "Initializing OpenGL display\n");
    // ri.Printf (PRINT_ALL, "...setting mode %d:", mode );

    // if ( !R_GetModeInfo( &glConfig.vidWidth, &glConfig.vidHeight, &glConfig.windowAspect, mode ) )
    // {
    //     ri.Printf( PRINT_ALL, " invalid mode\n" );
    //     return RSERR_INVALID_MODE;
    // }
    // ri.Printf( PRINT_ALL, " %d %d\n", glConfig.vidWidth, glConfig.vidHeight);

    if (dpy = XOpenDisplay(ptr::null())) == ptr::null_mut() {
        fprintf(libc::stderr, b"Error couldn't open the X display\n\0".as_ptr() as *const c_char);
        return rserr_t::RSERR_INVALID_MODE;
    }

    scrnum = DefaultScreen(dpy);
    root = RootWindow(dpy, scrnum);

    actualWidth = 640; // placeholder
    actualHeight = 480; // placeholder

    // Get video mode list
    MajorVersion = 0;
    MinorVersion = 0;
    if XF86VidModeQueryVersion(dpy, &mut MajorVersion, &mut MinorVersion) == 0 {
        vidmode_ext = qfalse;
    } else {
        // ri.Printf(PRINT_ALL, "Using XFree86-VidModeExtension Version %d.%d\n",
        //     MajorVersion, MinorVersion);
        vidmode_ext = qtrue;
    }

    if vidmode_ext != qfalse {
        let mut best_fit: c_int;
        let mut best_dist: c_int;
        let mut dist: c_int;
        let mut x: c_int;
        let mut y: c_int;

        XF86VidModeGetAllModeLines(dpy, scrnum, &mut num_vidmodes, &mut vidmodes);

        // Are we going fullscreen?  If so, let's change video mode
        if fullscreen != qfalse && (*r_fakeFullscreen).integer == 0 {
            best_dist = 9999999;
            best_fit = -1;

            i = 0;
            while i < num_vidmodes {
                // if (glConfig.vidWidth > vidmodes[i]->hdisplay ||
                //     glConfig.vidHeight > vidmodes[i]->vdisplay)
                //     continue;

                // x = glConfig.vidWidth - vidmodes[i]->hdisplay;
                // y = glConfig.vidHeight - vidmodes[i]->vdisplay;
                x = 640 - (*(*vidmodes.add(i as usize))).hdisplay;
                y = 480 - (*(*vidmodes.add(i as usize))).vdisplay;
                dist = (x * x) + (y * y);
                if dist < best_dist {
                    best_dist = dist;
                    best_fit = i;
                }
                i += 1;
            }

            if best_fit != -1 {
                actualWidth = (*(*vidmodes.add(best_fit as usize))).hdisplay;
                actualHeight = (*(*vidmodes.add(best_fit as usize))).vdisplay;

                // change to the mode
                XF86VidModeSwitchToMode(dpy, scrnum, *vidmodes.add(best_fit as usize));
                vidmode_active = qtrue;

                // Move the viewport to top left
                XF86VidModeSetViewPort(dpy, scrnum, 0, 0);
            }
        }
    }


    if (*r_colorbits).value <= 0.0 {
        colorbits = 24;
    } else {
        colorbits = (*r_colorbits).value as c_int;
    }

    // if ( !Q_stricmp( r_glDriver->string, _3DFX_DRIVER_NAME ) )
    //     colorbits = 16;

    if (*r_depthbits).value <= 0.0 {
        depthbits = 24;
    } else {
        depthbits = (*r_depthbits).value as c_int;
    }
    stencilbits = (*r_stencilbits).value as c_int;

    i = 0;
    while i < 16 {
        // 0 - default
        // 1 - minus colorbits
        // 2 - minus depthbits
        // 3 - minus stencil
        if (i % 4) == 0 && i != 0 {
            // one pass, reduce
            match i / 4 {
                2 => {
                    if colorbits == 24 {
                        colorbits = 16;
                    }
                }
                1 => {
                    if depthbits == 24 {
                        depthbits = 16;
                    } else if depthbits == 16 {
                        depthbits = 8;
                    }
                }
                3 => {
                    if stencilbits == 24 {
                        stencilbits = 16;
                    } else if stencilbits == 16 {
                        stencilbits = 8;
                    }
                }
                _ => {}
            }
        }

        tcolorbits = colorbits;
        tdepthbits = depthbits;
        tstencilbits = stencilbits;

        if (i % 4) == 3 { // reduce colorbits
            if tcolorbits == 24 {
                tcolorbits = 16;
            }
        }

        if (i % 4) == 2 { // reduce depthbits
            if tdepthbits == 24 {
                tdepthbits = 16;
            } else if tdepthbits == 16 {
                tdepthbits = 8;
            }
        }

        if (i % 4) == 1 { // reduce stencilbits
            if tstencilbits == 24 {
                tstencilbits = 16;
            } else if tstencilbits == 16 {
                tstencilbits = 8;
            } else {
                tstencilbits = 0;
            }
        }

        if tcolorbits == 24 {
            attrib[ATTR_RED_IDX] = 8;
            attrib[ATTR_GREEN_IDX] = 8;
            attrib[ATTR_BLUE_IDX] = 8;
        } else {
            // must be 16 bit
            attrib[ATTR_RED_IDX] = 4;
            attrib[ATTR_GREEN_IDX] = 4;
            attrib[ATTR_BLUE_IDX] = 4;
        }

        attrib[ATTR_DEPTH_IDX] = tdepthbits; // default to 24 depth
        attrib[ATTR_STENCIL_IDX] = tstencilbits;

        // #if 0
        // ri.Printf( PRINT_DEVELOPER, "Attempting %d/%d/%d Color bits, %d depth, %d stencil display...",
        //     attrib[ATTR_RED_IDX], attrib[ATTR_GREEN_IDX], attrib[ATTR_BLUE_IDX],
        //     attrib[ATTR_DEPTH_IDX], attrib[ATTR_STENCIL_IDX]);
        // #endif

        visinfo = qglXChooseVisual(dpy, scrnum, attrib.as_mut_ptr());
        if visinfo.is_null() {
            // #if 0
            // ri.Printf( PRINT_DEVELOPER, "failed\n");
            // #endif
            i += 1;
            continue;
        }

        // #if 0
        // ri.Printf( PRINT_DEVELOPER, "Successful\n");
        // #endif

        // ri.Printf( PRINT_ALL, "Using %d/%d/%d Color bits, %d depth, %d stencil display.\n",
        //     attrib[ATTR_RED_IDX], attrib[ATTR_GREEN_IDX], attrib[ATTR_BLUE_IDX],
        //     attrib[ATTR_DEPTH_IDX], attrib[ATTR_STENCIL_IDX]);

        // glConfig.colorBits = tcolorbits;
        // glConfig.depthBits = tdepthbits;
        // glConfig.stencilBits = tstencilbits;
        break;
    }

    if visinfo.is_null() {
        // ri.Printf( PRINT_ALL, "Couldn't get a visual\n" );
        return rserr_t::RSERR_INVALID_MODE;
    }

    /* window attributes */
    attr.background_pixel = BlackPixel(dpy, scrnum);
    attr.border_pixel = 0;
    attr.colormap = XCreateColormap(dpy, root, visinfo, AllocNone());
    attr.event_mask = X_MASK;
    if vidmode_active != qfalse {
        mask = CWBackPixel | CWColormap | CWSaveUnder | CWBackingStore |
            CWEventMask | CWOverrideRedirect;
        attr.override_redirect = True;
        attr.backing_store = NotUseful;
        attr.save_under = False;
    } else {
        mask = CWBackPixel | CWBorderPixel | CWColormap | CWEventMask;
    }

    win = XCreateWindow(dpy, root, 0, 0,
            actualWidth, actualHeight,
            0, (*visinfo).as_ref().unwrap_or(&XVisualInfo{} as &_).as_ref().unwrap_or(&0).as_ref().unwrap_or(&0).as_ref().unwrap_or(&0) as *const _ as c_int, InputOutput(),
            visinfo as *mut c_void, mask, &mut attr);
    XMapWindow(dpy, win);

    if vidmode_active != qfalse {
        XMoveWindow(dpy, win, 0, 0);
    }

    // Check for DGA
    if (*in_dgamouse).value > 0.0 {
        if XF86DGAQueryVersion(dpy, &mut MajorVersion, &mut MinorVersion) == 0 {
            // unable to query, probalby not supported
            // ri.Printf( PRINT_ALL, "Failed to detect XF86DGA Mouse\n" );
            // ri.Cvar_Set( "in_dgamouse", "0" );
        } else {
            // ri.Printf( PRINT_ALL, "XF86DGA Mouse (Version %d.%d) initialized\n",
            //     MajorVersion, MinorVersion);
        }
    }

    XFlush(dpy);

    ctx = qglXCreateContext(dpy, visinfo, ptr::null_mut(), True);

    qglXMakeCurrent(dpy, win, ctx);

    return rserr_t::RSERR_OK;
}

/*
** GLW_InitExtensions
*/
unsafe fn GLW_InitExtensions() {
    let extensions_string: *const c_char = qglGetString(GL_EXTENSIONS);

    // Use modern texture compression extensions
    if !strstr(extensions_string, b"ARB_texture_compression\0".as_ptr() as *const c_char).is_null()
        && !strstr(extensions_string, b"EXT_texture_compression_s3tc\0".as_ptr() as *const c_char).is_null() {
        if (*r_ext_compressed_textures).value > 0.0 {
            // glConfig.textureCompression = TC_S3TC_DXT;
            // ri.Printf( PRINT_ALL, "...using GL_EXT_texture_compression_s3tc\n" );
        } else {
            // glConfig.textureCompression = TC_NONE;
            // ri.Printf( PRINT_ALL, "...ignoring GL_EXT_texture_compression_s3tc\n" );
        }
    }
    // Or check for old ones
    else if !strstr(extensions_string, b"GL_S3_s3tc\0".as_ptr() as *const c_char).is_null() {
        if (*r_ext_compressed_textures).value > 0.0 {
            // glConfig.textureCompression = TC_S3TC;
            // ri.Printf( PRINT_ALL, "...using GL_S3_s3tc\n" );
        } else {
            // glConfig.textureCompression = TC_NONE;
            // ri.Printf( PRINT_ALL, "...ignoring GL_S3_s3tc\n" );
        }
    } else {
        // glConfig.textureCompression = TC_NONE;
        // ri.Printf( PRINT_ALL, "...no texture compression found\n" );
    }

    // #if 0
    //     // WGL_EXT_swap_control
    //     if ( strstr( glConfig.extensions_string, "WGL_EXT_swap_control" ) )
    //     {
    //         qwglSwapIntervalEXT = ( BOOL (WINAPI *)(int)) qwglGetProcAddress( "wglSwapIntervalEXT" );
    //         ri.Printf( PRINT_ALL, "...using WGL_EXT_swap_control\n" );
    //     }
    //     else
    //     {
    //         ri.Printf( PRINT_ALL, "...WGL_EXT_swap_control not found\n" );
    //     }
    // #endif

    // GL_ARB_multitexture
    // qglMultiTexCoord2fARB = NULL;
    // qglActiveTextureARB = NULL;
    // qglClientActiveTextureARB = NULL;
    if !strstr(extensions_string, b"GL_ARB_multitexture\0".as_ptr() as *const c_char).is_null() {
        if (*r_ext_multitexture).value > 0.0 {
            // qglMultiTexCoord2fARB = ( PFNGLMULTITEXCOORD2FARBPROC ) dlsym( glw_state.OpenGLLib, "glMultiTexCoord2fARB" );
            // qglActiveTextureARB = ( PFNGLACTIVETEXTUREARBPROC ) dlsym( glw_state.OpenGLLib, "glActiveTextureARB" );
            // qglClientActiveTextureARB = ( PFNGLCLIENTACTIVETEXTUREARBPROC ) dlsym( glw_state.OpenGLLib, "glClientActiveTextureARB" );

            // if ( qglActiveTextureARB )
            // {
            //     ri.Printf( PRINT_ALL, "...using GL_ARB_multitexture\n" );
            // }
            // else
            // {
            //     ri.Printf( PRINT_ALL, "...blind search for ARB_multitexture failed\n" );
            // }
        } else {
            // ri.Printf( PRINT_ALL, "...ignoring GL_ARB_multitexture\n" );
        }
    } else {
        // ri.Printf( PRINT_ALL, "...GL_ARB_multitexture not found\n" );
    }

    // GL_EXT_texture_filter_anisotropic
    // glConfig.textureFilterAnisotropicAvailable = qfalse;
    if !strstr(extensions_string, b"EXT_texture_filter_anisotropic\0".as_ptr() as *const c_char).is_null() {
        // glConfig.textureFilterAnisotropicAvailable = qtrue;
        // ri.Printf( PRINT_ALL, "...GL_EXT_texture_filter_anisotropic available\n" );

        if (*r_ext_texture_filter_anisotropic).integer != 0 {
            // ri.Printf( PRINT_ALL, "...using GL_EXT_texture_filter_anisotropic\n" );
        } else {
            // ri.Printf( PRINT_ALL, "...ignoring GL_EXT_texture_filter_anisotropic\n" );
        }
        // ri.Cvar_Set( "r_ext_texture_filter_anisotropic_avail", "1" );
    } else {
        // ri.Printf( PRINT_ALL, "...GL_EXT_texture_filter_anisotropic not found\n" );
        // ri.Cvar_Set( "r_ext_texture_filter_anisotropic_avail", "0" );
    }

    // GL_EXT_compiled_vertex_array
    if !strstr(extensions_string, b"GL_EXT_compiled_vertex_array\0".as_ptr() as *const c_char).is_null() {
        if (*r_ext_compiled_vertex_array).value > 0.0 {
            // ri.Printf( PRINT_ALL, "...using GL_EXT_compiled_vertex_array\n" );
            // qglLockArraysEXT = ( void ( APIENTRY * )( int, int ) ) dlsym( glw_state.OpenGLLib, "glLockArraysEXT" );
            // qglUnlockArraysEXT = ( void ( APIENTRY * )( void ) ) dlsym( glw_state.OpenGLLib, "glUnlockArraysEXT" );
            // if (!qglLockArraysEXT || !qglUnlockArraysEXT) {
            //     ri.Error (ERR_FATAL, "bad getprocaddress");
            // }
        } else {
            // ri.Printf( PRINT_ALL, "...ignoring GL_EXT_compiled_vertex_array\n" );
        }
    } else {
        // ri.Printf( PRINT_ALL, "...GL_EXT_compiled_vertex_array not found\n" );
    }
}

/*
** GLW_LoadOpenGL
**
** GLimp_win.c internal function that that attempts to load and use
** a specific OpenGL DLL.
*/
unsafe fn GLW_LoadOpenGL(name: *const c_char) -> qboolean {
    let mut fullscreen: qboolean;

    // ri.Printf( PRINT_ALL, "...loading %s: ", name );

    // disable the 3Dfx splash screen and set gamma
    // we do this all the time, but it shouldn't hurt anything
    // on non-3Dfx stuff
    putenv(b"FX_GLIDE_NO_SPLASH=0\0".as_ptr() as *const c_char);

    // Mesa VooDoo hacks
    putenv(b"MESA_GLX_FX=fullscreen\n\0".as_ptr() as *const c_char);

    // load the QGL layer
    if QGL_Init(name) != qfalse {
        fullscreen = (*r_fullscreen).integer;

        // create the window and set up the context
        if GLW_StartDriverAndSetMode(name, (*r_mode).integer, fullscreen) == qfalse {
            if (*r_mode).integer != 3 {
                if GLW_StartDriverAndSetMode(name, 3, fullscreen) == qfalse {
                    goto_fail: {}
                }
            } else {
                goto_fail: {}
            }
        }

        return qtrue;
    } else {
        // ri.Printf( PRINT_ALL, "failed\n" );
    }

    goto_fail: {}

    QGL_Shutdown();

    return qfalse;
}

/*
** GLimp_Init
**
** This routine is responsible for initializing the OS specific portions
** of OpenGL.
*/
pub unsafe fn GLimp_Init() {
    let mut attemptedlibGL: qboolean = qfalse;
    let mut attempted3Dfx: qboolean = qfalse;
    let mut success: qboolean = qfalse;
    let mut buf: [c_char; 1024] = [0; 1024];
    let mut lastValidRenderer: *mut cvar_t = ri_Cvar_Get(b"r_lastValidRenderer\0".as_ptr() as *const c_char, b"(uninitialized)\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
    let mut cv: *mut cvar_t = ptr::null_mut();

    // glConfig.deviceSupportsGamma = qfalse;

    InitSig();

    //
    // load and initialize the specific OpenGL driver
    //
    if GLW_LoadOpenGL((*r_glDriver).string) == qfalse {
        // if ( !Q_stricmp( r_glDriver->string, OPENGL_DRIVER_NAME ) )
        // {
        //     attemptedlibGL = qtrue;
        // }
        // else if ( !Q_stricmp( r_glDriver->string, _3DFX_DRIVER_NAME ) )
        // {
        //     attempted3Dfx = qtrue;
        // }

        if attempted3Dfx == qfalse && success == qfalse {
            attempted3Dfx = qtrue;
            if GLW_LoadOpenGL(b"3dfx\0".as_ptr() as *const c_char) != qfalse {
                // ri.Cvar_Set( "r_glDriver", _3DFX_DRIVER_NAME );
                // r_glDriver->modified = qfalse;
                success = qtrue;
            }
        }

        // try ICD before trying 3Dfx standalone driver
        if attemptedlibGL == qfalse && success == qfalse {
            attemptedlibGL = qtrue;
            if GLW_LoadOpenGL(b"libGL\0".as_ptr() as *const c_char) != qfalse {
                // ri.Cvar_Set( "r_glDriver", OPENGL_DRIVER_NAME );
                // r_glDriver->modified = qfalse;
                success = qtrue;
            }
        }

        if success == qfalse {
            // ri.Error( ERR_FATAL, "GLimp_Init() - could not load OpenGL subsystem\n" );
        }
    }

    // get our config strings
    Q_strncpyz(buf.as_mut_ptr(), qglGetString(GL_VENDOR), 1024);
    // Q_strncpyz( glConfig.vendor_string, qglGetString (GL_VENDOR), sizeof( glConfig.vendor_string ) );
    // Q_strncpyz( glConfig.renderer_string, qglGetString (GL_RENDERER), sizeof( glConfig.renderer_string ) );
    // if (*glConfig.renderer_string && glConfig.renderer_string[strlen(glConfig.renderer_string) - 1] == '\n')
    //     glConfig.renderer_string[strlen(glConfig.renderer_string) - 1] = 0;
    // Q_strncpyz( glConfig.version_string, qglGetString (GL_VERSION), sizeof( glConfig.version_string ) );
    // Q_strncpyz( glConfig.extensions_string, qglGetString (GL_EXTENSIONS), sizeof( glConfig.extensions_string ) );

    //
    // chipset specific configuration
    //
    strcpy(buf.as_mut_ptr(), qglGetString(GL_RENDERER));
    strlwr(buf.as_mut_ptr());

    // if ( Q_stricmp( lastValidRenderer->string, glConfig.renderer_string ) )
    // {
    //     ri.Cvar_Set( "r_picmip", "1" );
    //     ri.Cvar_Set( "r_twopartfog", "0" );
    //     ri.Cvar_Set( "r_textureMode", "GL_LINEAR_MIPMAP_NEAREST" );

    //     //
    //     // voodoo issues
    //     //
    //     if ( strstr( buf, "voodoo" ) && !strstr( buf, "banshee" ) )
    //     {
    //         ri.Cvar_Set( "r_fakeFullscreen", "1");
    //     }

    //     //
    //     // Riva128 issues
    //     //
    //     if ( strstr( buf, "riva 128" ) )
    //     {
    //         ri.Cvar_Set( "r_twopartfog", "1" );
    //     }

    //     //
    //     // Rage Pro issues
    //     //
    //     if ( strstr( buf, "rage pro" ) )
    //     {
    //         ri.Cvar_Set( "r_mode", "2" );
    //         ri.Cvar_Set( "r_twopartfog", "1" );
    //     }

    //     //
    //     // Permedia2 issues
    //     //
    //     if ( strstr( buf, "permedia2" ) )
    //     {
    //         ri.Cvar_Set( "r_vertexLight", "1" );
    //     }

    //     //
    //     // Riva TNT issues
    //     //
    //     if ( strstr( buf, "riva tnt " ) )
    //     {
    //         if ( r_texturebits->integer == 32 ||
    //              ( ( r_texturebits->integer == 0 ) && glConfig.colorBits > 16 ) )
    //         {
    //             ri.Cvar_Set( "r_picmip", "1" );
    //         }
    //     }

    //     ri.Cvar_Set( "r_lastValidRenderer", glConfig.renderer_string );
    // }

    // initialize extensions
    GLW_InitExtensions();

    InitSig();

    return;
}


/*
** GLimp_EndFrame
**
** Responsible for doing a swapbuffers and possibly for other stuff
** as yet to be determined.  Probably better not to make this a GLimp
** function and instead do a call to GLimp_SwapBuffers.
*/
pub unsafe fn GLimp_EndFrame() {
    // #if 0
    //     int	err;

    //     if ( !glState.finishCalled )
    //         qglFinish();

    //     // check for errors
    //     if ( !gl_ignore_errors->value ) {
    //         if ( ( err = qglGetError() ) != GL_NO_ERROR )
    //         {
    //             ri.Error( ERR_FATAL, "GLimp_EndFrame() - glGetError() failed (0x%x)!\n", err );
    //         }
    //     }
    // #endif

    // don't flip if drawing to front buffer
    if stricmp((*r_drawBuffer).string, b"GL_FRONT\0".as_ptr() as *const c_char) != 0 {
        qglXSwapBuffers(dpy, win);
    }

    // check logging
    QGL_EnableLogging((*r_logFile).value as c_int);

    // #if 0
    //     GLimp_LogComment( "*** RE_EndFrame ***\n" );

    //     // decrement log
    //     if ( gl_log->value )
    //     {
    //         ri.Cvar_Set( "gl_log", va("%i",gl_log->value - 1 ) );
    //     }
    // #endif
}

/*
===========================================================

SMP acceleration

===========================================================
*/

static mut renderCommandsEvent: c_void = 0 as c_void;
static mut renderCompletedEvent: c_void = 0 as c_void;
static mut renderActiveEvent: c_void = 0 as c_void;

static mut glimpRenderThread: Option<unsafe extern "C" fn()> = None;

unsafe extern "C" fn GLimp_RenderThreadWrapper(_stub: *mut c_void) {
    if let Some(func) = glimpRenderThread {
        func();
    }

    // #if 0
    //     // unbind the context before we die
    //     qglXMakeCurrent(dpy, None, NULL);
    // #endif
}


/*
=======================
GLimp_SpawnRenderThread
=======================
*/
static mut renderThreadHandle: c_void = 0 as c_void;

pub unsafe fn GLimp_SpawnRenderThread(function: Option<unsafe extern "C" fn()>) -> qboolean {
    sem_init(&mut renderCommandsEvent as *mut _ as *mut c_void, 0, 0);
    sem_init(&mut renderCompletedEvent as *mut _ as *mut c_void, 0, 0);
    sem_init(&mut renderActiveEvent as *mut _ as *mut c_void, 0, 0);

    glimpRenderThread = function;

    if pthread_create(&mut renderThreadHandle as *mut _ as *mut c_void, ptr::null_mut(),
        GLimp_RenderThreadWrapper, ptr::null_mut()) != 0 {
        return qfalse;
    }

    return qtrue;
}

static mut smpData: *mut c_void = ptr::null_mut();
static mut glXErrors: c_int = 0;

pub unsafe fn GLimp_RendererSleep() -> *mut c_void {
    let mut data: *mut c_void;

    // #if 0
    //     if ( !qglXMakeCurrent(dpy, None, NULL) ) {
    //         glXErrors++;
    //     }
    // #endif

    // ResetEvent( renderActiveEvent );

    // after this, the front end can exit GLimp_FrontEndSleep
    sem_post(&mut renderCompletedEvent as *mut _ as *mut c_void);

    sem_wait(&mut renderCommandsEvent as *mut _ as *mut c_void);

    // #if 0
    //     if ( !qglXMakeCurrent(dpy, win, ctx) ) {
    //         glXErrors++;
    //     }
    // #endif

    // ResetEvent( renderCompletedEvent );
    // ResetEvent( renderCommandsEvent );

    data = smpData;

    // after this, the main thread can exit GLimp_WakeRenderer
    sem_post(&mut renderActiveEvent as *mut _ as *mut c_void);

    return data;
}


pub unsafe fn GLimp_FrontEndSleep() {
    sem_wait(&mut renderCompletedEvent as *mut _ as *mut c_void);

    // #if 0
    //     if ( !qglXMakeCurrent(dpy, win, ctx) ) {
    //         glXErrors++;
    //     }
    // #endif
}


pub unsafe fn GLimp_WakeRenderer(data: *mut c_void) {
    smpData = data;

    // #if 0
    //     if ( !qglXMakeCurrent(dpy, None, NULL) ) {
    //         glXErrors++;
    //     }
    // #endif

    // after this, the renderer can continue through GLimp_RendererSleep
    sem_post(&mut renderCommandsEvent as *mut _ as *mut c_void);

    sem_wait(&mut renderActiveEvent as *mut _ as *mut c_void);
}

/*===========================================================*/

/*****************************************************************************/
/* MOUSE                                                                     */
/*****************************************************************************/

pub unsafe fn IN_Init() {
    // mouse variables
    in_mouse = Cvar_Get(b"in_mouse\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);
    in_dgamouse = Cvar_Get(b"in_dgamouse\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, CVAR_ARCHIVE);

    if (*in_mouse).value > 0.0 {
        mouse_avail = qtrue;
    } else {
        mouse_avail = qfalse;
    }
}

pub unsafe fn IN_Shutdown() {
    mouse_avail = qfalse;
}

pub unsafe fn IN_MouseMove() {
    if mouse_avail == qfalse || dpy.is_null() || win == 0 {
        return;
    }

    // #if 0
    //     if (!dgamouse) {
    //         Window root, child;
    //         int root_x, root_y;
    //         int win_x, win_y;
    //         unsigned int mask_return;
    //         int mwx = glConfig.vidWidth/2;
    //         int mwy = glConfig.vidHeight/2;

    //         XQueryPointer(dpy, win, &root, &child,
    //             &root_x, &root_y, &win_x, &win_y, &mask_return);

    //         mx = win_x - mwx;
    //         my = win_y - mwy;

    //         XWarpPointer(dpy,None,win,0,0,0,0, mwx, mwy);
    //     }
    // #endif

    if mx != 0 || my != 0 {
        Sys_QueEvent(0, SE_MOUSE, mx, my, 0, ptr::null_mut());
    }
    mx = 0;
    my = 0;
}

pub unsafe fn IN_Frame() {
    // if ( cls.keyCatchers || cls.state != CA_ACTIVE ) {
    //     // temporarily deactivate if not in the game and
    //     // running on the desktop
    //     // voodoo always counts as full screen
    //     if (Cvar_VariableValue ("r_fullscreen") == 0
    //         && strcmp( Cvar_VariableString("r_glDriver"), _3DFX_DRIVER_NAME ) )	{
    //         IN_DeactivateMouse ();
    //         return;
    //     }
    //     if (dpy && !autorepeaton) {
    //         XAutoRepeatOn(dpy);
    //         autorepeaton = qtrue;
    //     }
    // } else if (dpy && autorepeaton) {
    //     XAutoRepeatOff(dpy);
    //     autorepeaton = qfalse;
    // }

    IN_ActivateMouse();

    // post events to the system que
    IN_MouseMove();
}

pub unsafe fn IN_Activate() {
}

pub unsafe fn Sys_SendKeyEvents() {
    let mut event: XEvent;

    if dpy.is_null() {
        return;
    }

    HandleEvents();
    // while (XCheckMaskEvent(dpy,KEY_MASK|MOUSE_MASK,&event))
    //     HandleEvent(&event);
}

// Helper C string functions
extern "C" {
    pub fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
}
