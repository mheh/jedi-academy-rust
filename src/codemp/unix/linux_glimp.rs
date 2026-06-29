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

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    // X11/X11 display functions
    fn XOpenDisplay(name: *const c_char) -> *mut c_void;
    fn XCloseDisplay(display: *mut c_void) -> c_int;
    fn DefaultScreen(display: *mut c_void) -> c_int;
    fn RootWindow(display: *mut c_void, screen: c_int) -> *mut c_void;
    fn XCreatePixmap(display: *mut c_void, drawable: *mut c_void, width: c_int, height: c_int, depth: c_int) -> *mut c_void;
    fn XFreePixmap(display: *mut c_void, pixmap: *mut c_void) -> c_int;
    fn XCreateGC(display: *mut c_void, drawable: *mut c_void, valuemask: u64, values: *mut c_void) -> *mut c_void;
    fn XFreeGC(display: *mut c_void, gc: *mut c_void) -> c_int;
    fn XFillRectangle(display: *mut c_void, drawable: *mut c_void, gc: *mut c_void, x: c_int, y: c_int, width: c_int, height: c_int) -> c_int;
    fn XCreatePixmapCursor(display: *mut c_void, source: *mut c_void, mask: *mut c_void, foreground: *mut c_void, background: *mut c_void, x: c_int, y: c_int) -> *mut c_void;
    fn XWarpPointer(display: *mut c_void, src_w: *mut c_void, dest_w: *mut c_void, src_x: c_int, src_y: c_int, src_width: c_int, src_height: c_int, dest_x: c_int, dest_y: c_int) -> c_int;
    fn XSync(display: *mut c_void, discard: i32) -> c_int;
    fn XDefineCursor(display: *mut c_void, window: *mut c_void, cursor: *mut c_void) -> c_int;
    fn XUndefineCursor(display: *mut c_void, window: *mut c_void) -> c_int;
    fn XGrabPointer(display: *mut c_void, grab_window: *mut c_void, owner_events: i32, event_mask: u64, pointer_mode: c_int, keyboard_mode: c_int, confine_to: *mut c_void, cursor: *mut c_void, time: u32) -> c_int;
    fn XUngrabPointer(display: *mut c_void, time: u32) -> c_int;
    fn XGetPointerControl(display: *mut c_void, accel_num: *mut c_int, accel_denom: *mut c_int, threshold: *mut c_int) -> c_int;
    fn XChangePointerControl(display: *mut c_void, do_accel: i32, do_threshold: i32, accel_num: c_int, accel_denom: c_int, threshold: c_int) -> c_int;
    fn XGrabKeyboard(display: *mut c_void, grab_window: *mut c_void, owner_events: i32, pointer_mode: c_int, keyboard_mode: c_int, time: u32) -> c_int;
    fn XUngrabKeyboard(display: *mut c_void, time: u32) -> c_int;
    fn XFlush(display: *mut c_void) -> c_int;
    fn XEventsQueued(display: *mut c_void, mode: c_int) -> c_int;
    fn XPending(display: *mut c_void) -> c_int;
    fn XPeekEvent(display: *mut c_void, event: *mut c_void) -> c_int;
    fn XNextEvent(display: *mut c_void, event: *mut c_void) -> c_int;
    fn XLookupString(event: *mut c_void, buffer: *mut c_char, nbytes: c_int, keysym: *mut u32, status: *mut c_void) -> c_int;
    fn XCreateWindow(display: *mut c_void, parent: *mut c_void, x: c_int, y: c_int, width: c_int, height: c_int, border_width: c_int, depth: c_int, class: c_int, visual: *mut c_void, valuemask: u64, attributes: *mut c_void) -> *mut c_void;
    fn XDestroyWindow(display: *mut c_void, window: *mut c_void) -> c_int;
    fn XMapWindow(display: *mut c_void, window: *mut c_void) -> c_int;
    fn XMoveWindow(display: *mut c_void, window: *mut c_void, x: c_int, y: c_int) -> c_int;
    fn XStoreName(display: *mut c_void, window: *mut c_void, window_name: *const c_char) -> c_int;
    fn XCreateColormap(display: *mut c_void, window: *mut c_void, visual: *mut c_void, alloc: c_int) -> *mut c_void;
    fn ConnectionNumber(display: *mut c_void) -> c_int;

    // GLX functions
    fn qglXChooseVisual(display: *mut c_void, screen: c_int, attriblist: *mut c_int) -> *mut c_void;
    fn qglXCreateContext(display: *mut c_void, vis: *mut c_void, shareList: *mut c_void, direct: i32) -> *mut c_void;
    fn qglXDestroyContext(display: *mut c_void, ctx: *mut c_void) -> c_int;
    fn qglXMakeCurrent(display: *mut c_void, drawable: *mut c_void, ctx: *mut c_void) -> i32;
    fn qglXSwapBuffers(display: *mut c_void, drawable: *mut c_void) -> c_int;
    fn qglGetString(name: u32) -> *const u8;
    fn qglGetIntegerv(pname: u32, params: *mut c_int) -> c_void;

    // XF86VidMode extension
    fn XF86VidModeQueryVersion(display: *mut c_void, majorVersion: *mut c_int, minorVersion: *mut c_int) -> i32;
    fn XF86VidModeGetAllModeLines(display: *mut c_void, screen: c_int, modecount: *mut c_int, modesinfo: *mut *mut XF86VidModeModeInfo) -> i32;
    fn XF86VidModeSwitchToMode(display: *mut c_void, screen: c_int, modeline: *mut XF86VidModeModeInfo) -> i32;
    fn XF86VidModeSetViewPort(display: *mut c_void, screen: c_int, x: c_int, y: c_int) -> i32;

    // XF86DGA extension
    fn XF86DGAQueryVersion(display: *mut c_void, majorVersion: *mut c_int, minorVersion: *mut c_int) -> i32;
    fn XF86DGADirectVideo(display: *mut c_void, screen: c_int, flags: c_int) -> i32;

    // System functions
    fn Sys_Milliseconds() -> c_int;
    fn Sys_QueEvent(time: c_int, type_: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void) -> c_void;
    fn Sys_Exit(code: c_int) -> c_void;

    // Standard library functions
    fn strlen(s: *const c_char) -> usize;
    fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    fn printf(format: *const c_char, ...) -> c_int;
    fn putenv(string: *const c_char) -> c_int;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn signal(sig: c_int, handler: unsafe extern "C" fn(c_int) -> c_void) -> unsafe extern "C" fn(c_int) -> c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlwr(str: *mut c_char) -> *mut c_char;
    fn assert(expr: c_int) -> c_void;

    // Game/renderer functions
    fn Q_stristr(s: *const c_char, find: *const c_char) -> *const c_char;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize) -> c_void;
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: u32) -> *mut cvar_t;
    fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // GLimp/renderer internals
    fn GLimp_Shutdown() -> c_void;
    fn GLimp_LogComment(comment: *const c_char) -> c_void;
    fn GLimp_SetGamma(red: *const u8, green: *const u8, blue: *const u8) -> c_void;
    fn QGL_Init(name: *const c_char) -> c_int;
    fn QGL_Shutdown() -> c_void;
    fn QGL_EnableLogging(enable: c_int) -> c_void;
    fn R_GetModeInfo(width: *mut c_int, height: *mut c_int, windowAspect: *mut f32, mode: c_int) -> c_int;

    // Global renderer state
    static glConfig: glconfig_t;
    static glState: glstate_t;
    static glw_state: glwstate_t;
    static ri: refimport_t;
    static r_glDriver: *mut cvar_t;
    static r_mode: *mut cvar_t;
    static r_fullscreen: *mut cvar_t;
    static r_colorbits: *mut cvar_t;
    static r_depthbits: *mut cvar_t;
    static r_stencilbits: *mut cvar_t;
    static r_drawBuffer: *mut cvar_t;
    static r_logFile: *mut cvar_t;
    static r_allowExtensions: *mut cvar_t;
    static r_ext_compressed_textures: *mut cvar_t;
    static r_ext_texture_env_add: *mut cvar_t;
    static r_ext_multitexture: *mut cvar_t;
    static r_ext_compiled_vertex_array: *mut cvar_t;
    static qglMultiTexCoord2fARB: *mut c_void;
    static qglActiveTextureARB: *mut c_void;
    static qglClientActiveTextureARB: *mut c_void;
    static qglLockArraysEXT: *mut c_void;
    static qglUnlockArraysEXT: *mut c_void;
    static cls: cls_t;
}

#[repr(C)]
pub struct glconfig_t {
    // (fields would be defined here, but we're treating this as an external type)
    // For now, we just declare it exists but don't define the layout
    pub vidWidth: c_int,
    pub vidHeight: c_int,
    pub windowAspect: f32,
    pub colorBits: c_int,
    pub depthBits: c_int,
    pub stencilBits: c_int,
    pub deviceSupportsGamma: c_int,
    pub driverType: c_int,
    pub hardwareType: c_int,
    pub textureCompression: c_int,
    pub textureEnvAddAvailable: c_int,
    pub maxActiveTextures: c_int,
    pub vendor_string: [c_char; 1024],
    pub renderer_string: [c_char; 1024],
    pub version_string: [c_char; 1024],
    pub extensions_string: [c_char; 16384],
}

#[repr(C)]
pub struct glstate_t {
    // (fields would be defined here)
}

#[repr(C)]
pub struct glwstate_t {
    pub log_fp: *mut c_void,
    pub OpenGLLib: *mut c_void,
}

#[repr(C)]
pub struct refimport_t {
    // (fields would be defined here)
}

#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub value: f32,
    pub integer: c_int,
    pub modified: c_int,
}

#[repr(C)]
pub struct cls_t {
    pub keyCatchers: c_int,
}

#[repr(C)]
pub struct XF86VidModeModeInfo {
    pub dotclock: u32,
    pub hdisplay: u16,
    pub hsyncstart: u16,
    pub hsyncend: u16,
    pub htotal: u16,
    pub hskew: u16,
    pub vdisplay: u16,
    pub vsyncstart: u16,
    pub vsyncend: u16,
    pub vtotal: u16,
    pub flags: u32,
    pub privsize: c_int,
    pub private: *mut c_int,
}

const WINDOW_CLASS_NAME: &[u8] = b"Quake 3: Arena\0";

#[repr(C)]
enum rserr_t {
    RSERR_OK = 0,
    RSERR_INVALID_FULLSCREEN = 1,
    RSERR_INVALID_MODE = 2,
    RSERR_UNKNOWN = 3,
}

static mut dpy: *mut c_void = core::ptr::null_mut();
static mut scrnum: c_int = 0;
static mut win: *mut c_void = core::ptr::null_mut();
static mut ctx: *mut c_void = core::ptr::null_mut();

// bk001206 - not needed anymore
// static qboolean autorepeaton = qtrue;

const KEY_MASK: u64 = (1 << 0) | (1 << 1); // KeyPressMask | KeyReleaseMask
const MOUSE_MASK: u64 = (1 << 2) | (1 << 3) | (1 << 6) | (1 << 13); // ButtonPressMask | ButtonReleaseMask | PointerMotionMask | ButtonMotionMask
const X_MASK: u64 = KEY_MASK | MOUSE_MASK | (1 << 15) | (1 << 14); // KEY_MASK | MOUSE_MASK | VisibilityChangeMask | StructureNotifyMask

static mut mouse_avail: c_int = 0;
static mut mouse_active: c_int = 0;
static mut mwx: c_int = 0;
static mut mwy: c_int = 0;
static mut mx: c_int = 0;
static mut my: c_int = 0;

// Time mouse was reset, we ignore the first 50ms of the mouse to allow settling of events
static mut mouseResetTime: c_int = 0;
const MOUSE_RESET_DELAY: c_int = 50;

static mut in_mouse: *mut cvar_t = core::ptr::null_mut();
static mut in_dgamouse: *mut cvar_t = core::ptr::null_mut();

// bk001130 - from cvs1.17 (mkv), but not static
pub static mut in_joystick: *mut cvar_t = core::ptr::null_mut();
pub static mut in_joystickDebug: *mut cvar_t = core::ptr::null_mut();
pub static mut joy_threshold: *mut cvar_t = core::ptr::null_mut();

static mut r_allowSoftwareGL: *mut cvar_t = core::ptr::null_mut(); // don't abort out if the pixelformat claims software
static mut r_previousglDriver: *mut cvar_t = core::ptr::null_mut();

static mut dgamouse: c_int = 0;
static mut vidmode_ext: c_int = 0;

static mut win_x: c_int = 0;
static mut win_y: c_int = 0;

static mut vidmodes: *mut *mut XF86VidModeModeInfo = core::ptr::null_mut();
//static int default_dotclock_vidmode; // bk001204 - unused
static mut num_vidmodes: c_int = 0;
static mut vidmode_active: c_int = 0;

static mut mouse_accel_numerator: c_int = 0;
static mut mouse_accel_denominator: c_int = 0;
static mut mouse_threshold: c_int = 0;

/*
* Find the first occurrence of find in s.
*/
// bk001130 - from cvs1.17 (mkv), const
// bk001130 - made first argument const
unsafe fn Q_stristr_impl(mut s: *const c_char, find: *const c_char) -> *const c_char {
    let mut c: c_char;
    let mut sc: c_char;
    let mut len: usize;

    c = *find as c_char;
    find = find.add(1);

    if c as u8 != 0 {
        if c as u8 >= b'a' as u8 && c as u8 <= b'z' as u8 {
            c = ((c as u8 - (b'a' as u8 - b'A' as u8)) as c_char);
        }
        len = strlen(find);
        loop {
            loop {
                sc = *s as c_char;
                s = s.add(1);
                if sc as u8 == 0 {
                    return core::ptr::null();
                }
                if sc as u8 >= b'a' as u8 && sc as u8 <= b'z' as u8 {
                    sc = ((sc as u8 - (b'a' as u8 - b'A' as u8)) as c_char);
                }
            }
            if sc == c {
                break;
            }
            if Q_stricmpn(s, find, len) == 0 {
                break;
            }
        }
        s = s.offset(-1);
    }
    s
}

/*****************************************************************************/
/* KEYBOARD                                                                  */
/*****************************************************************************/

// bk001204 - unused
// static unsigned int	keyshift[256];		// key to map to if shift held down in console
// static qboolean shift_down=qfalse;

const K_KP_PGUP: c_int = 0;
const K_PGUP: c_int = 0;
const K_KP_PGDN: c_int = 0;
const K_PGDN: c_int = 0;
const K_KP_HOME: c_int = 0;
const K_HOME: c_int = 0;
const K_KP_END: c_int = 0;
const K_END: c_int = 0;
const K_KP_LEFTARROW: c_int = 0;
const K_LEFTARROW: c_int = 0;
const K_KP_RIGHTARROW: c_int = 0;
const K_RIGHTARROW: c_int = 0;
const K_KP_DOWNARROW: c_int = 0;
const K_DOWNARROW: c_int = 0;
const K_KP_UPARROW: c_int = 0;
const K_UPARROW: c_int = 0;
const K_ESCAPE: c_int = 0;
const K_KP_ENTER: c_int = 0;
const K_ENTER: c_int = 0;
const K_TAB: c_int = 0;
const K_F1: c_int = 0;
const K_F2: c_int = 0;
const K_F3: c_int = 0;
const K_F4: c_int = 0;
const K_F5: c_int = 0;
const K_F6: c_int = 0;
const K_F7: c_int = 0;
const K_F8: c_int = 0;
const K_F9: c_int = 0;
const K_F10: c_int = 0;
const K_F11: c_int = 0;
const K_F12: c_int = 0;
const K_BACKSPACE: c_int = 0;
const K_KP_DEL: c_int = 0;
const K_DEL: c_int = 0;
const K_PAUSE: c_int = 0;
const K_SHIFT: c_int = 0;
const K_CTRL: c_int = 0;
const K_ALT: c_int = 0;
const K_KP_5: c_int = 0;
const K_INS: c_int = 0;
const K_KP_INS: c_int = 0;
const K_KP_PLUS: c_int = 0;
const K_KP_MINUS: c_int = 0;
const K_KP_SLASH: c_int = 0;
const K_MWHEELUP: c_int = 0;
const K_MWHEELDOWN: c_int = 0;
const K_MOUSE1: c_int = 0;

const SE_KEY: c_int = 1;
const SE_CHAR: c_int = 2;
const SE_MOUSE: c_int = 3;

const KEYCATCH_NONE: c_int = 0;
const KEYCATCH_CONSOLE: c_int = 1;

const PRINT_ALL: c_int = 0;
const CVAR_ARCHIVE: u32 = 1;
const CVAR_LATCH: u32 = 2;
const CVAR_ROM: u32 = 4;
const CVAR_TEMP: u32 = 8;

const OPENGL_DRIVER_NAME: &[u8] = b"libGL.so.1\0";
const _3DFX_DRIVER_NAME: &[u8] = b"3Dfx\0";

const GLX_RGBA: c_int = 4;
const GLX_RED_SIZE: c_int = 5;
const GLX_GREEN_SIZE: c_int = 6;
const GLX_BLUE_SIZE: c_int = 7;
const GLX_DOUBLEBUFFER: c_int = 5;
const GLX_DEPTH_SIZE: c_int = 12;
const GLX_STENCIL_SIZE: c_int = 13;

const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;
const GL_EXTENSIONS: u32 = 0x1F03;
const GL_MAX_ACTIVE_TEXTURES_ARB: u32 = 0x84E8;
const GL_FRONT: &[u8] = b"GL_FRONT\0";

const TC_NONE: c_int = 0;
const TC_S3TC: c_int = 1;

const GLDRV_ICD: c_int = 1;
const GLHW_GENERIC: c_int = 0;
const GLHW_3DFX_2D3D: c_int = 1;
const GLHW_RAGEPRO: c_int = 2;
const GLHW_PERMEDIA2: c_int = 3;
const GLHW_RIVA128: c_int = 4;

const ERR_FATAL: c_int = 0;

unsafe fn XLateKey(ev: *mut c_void, key: *mut c_int) -> *mut c_char {
    static mut buf: [c_char; 64] = [0; 64];
    let mut keysym: u32 = 0;
    // static qboolean setup = qfalse; // bk001204 - unused
    // int i; // bk001204 - unused

    *key = 0;

    XLookupString(ev, buf.as_mut_ptr(), 64, addr_of_mut!(keysym), core::ptr::null_mut());

    match keysym {
        0xff9a | 0xff8f => { *key = K_KP_PGUP; },
        0xff55 => { *key = K_PGUP; },

        0xff9b | 0xff83 => { *key = K_KP_PGDN; },
        0xff56 => { *key = K_PGDN; },

        0xff95 => { *key = K_KP_HOME; },
        0xff80 => { *key = K_KP_HOME; },
        0xff50 => { *key = K_HOME; },

        0xff9c | 0xff8f => { *key = K_KP_END; },
        0xff57 => { *key = K_END; },

        0xff96 => { *key = K_KP_LEFTARROW; },
        0xff81 => { *key = K_KP_LEFTARROW; },
        0xff51 => { *key = K_LEFTARROW; },

        0xff98 => { *key = K_KP_RIGHTARROW; },
        0xff83 => { *key = K_KP_RIGHTARROW; },
        0xff53 => { *key = K_RIGHTARROW; },

        0xff99 | 0xff84 => { *key = K_KP_DOWNARROW; },
        0xff54 => { *key = K_DOWNARROW; },

        0xff97 | 0xff86 => { *key = K_KP_UPARROW; },
        0xff52 => { *key = K_UPARROW; },

        0xff1b => { *key = K_ESCAPE; },

        0xff8d => { *key = K_KP_ENTER; },
        0xff0d => { *key = K_ENTER; },

        0xff09 => { *key = K_TAB; },

        0xffbe => { *key = K_F1; },
        0xffbf => { *key = K_F2; },
        0xffc0 => { *key = K_F3; },
        0xffc1 => { *key = K_F4; },
        0xffc2 => { *key = K_F5; },
        0xffc3 => { *key = K_F6; },
        0xffc4 => { *key = K_F7; },
        0xffc5 => { *key = K_F8; },
        0xffc6 => { *key = K_F9; },
        0xffc7 => { *key = K_F10; },
        0xffc8 => { *key = K_F11; },
        0xffc9 => { *key = K_F12; },

        // bk001206 - from Ryan's Fakk2
        //case XK_BackSpace: *key = 8; break; // ctrl-h
        0xff08 => { *key = K_BACKSPACE; }, // ctrl-h

        0xff9f | 0xff9e => { *key = K_KP_DEL; },
        0xffff => { *key = K_DEL; },

        0xff13 => { *key = K_PAUSE; },

        0xffe1 | 0xffe2 => { *key = K_SHIFT; },

        0xff1d | 0xffe3 | 0xffe4 => { *key = K_CTRL; },
        0xffe9 | 0xffe7 | 0xffea | 0xffea => { *key = K_ALT; },

        0xff8b => { *key = K_KP_5; },

        0xff63 => { *key = K_INS; },
        0xff9e | 0xff9c => { *key = K_KP_INS; },

        0xffaa => { *key = b'*' as c_int; },
        0xff8e => { *key = K_KP_PLUS; },
        0xff8b => { *key = K_KP_MINUS; },
        0xff8a => { *key = K_KP_SLASH; },

        // bk001130 - from cvs1.17 (mkv)
        0x0021 => { *key = b'1' as c_int; },
        0x0040 => { *key = b'2' as c_int; },
        0x0023 => { *key = b'3' as c_int; },
        0x0024 => { *key = b'4' as c_int; },
        0x0025 => { *key = b'5' as c_int; },
        0x005e => { *key = b'6' as c_int; },
        0x0026 => { *key = b'7' as c_int; },
        0x002a => { *key = b'8' as c_int; },
        0x0028 => { *key = b'9' as c_int; },
        0x0029 => { *key = b'0' as c_int; },

        _ => {
            *key = *buf.as_ptr() as c_int;
            if *key >= b'A' as c_int && *key <= b'Z' as c_int {
                *key = *key - b'A' as c_int + b'a' as c_int;
            }
        }
    }

    buf.as_mut_ptr()
}

// ========================================================================
// makes a null cursor
// ========================================================================

unsafe fn CreateNullCursor(display: *mut c_void, root: *mut c_void) -> *mut c_void {
    let mut cursormask: *mut c_void;
    let mut xgc: c_void;
    let mut gc: *mut c_void;
    let mut dummycolour: c_void;
    let mut cursor: *mut c_void;

    cursormask = XCreatePixmap(display, root, 1, 1, 1/*depth*/);
    // xgc.function = GXclear;
    gc = XCreateGC(display, cursormask, 0, addr_of_mut!(xgc) as *mut c_void);
    XFillRectangle(display, cursormask, gc, 0, 0, 1, 1);
    // dummycolour.pixel = 0;
    // dummycolour.red = 0;
    // dummycolour.flags = 04;
    cursor = XCreatePixmapCursor(display, cursormask, cursormask,
          addr_of_mut!(dummycolour) as *mut c_void, addr_of_mut!(dummycolour) as *mut c_void, 0, 0);
    XFreePixmap(display, cursormask);
    XFreeGC(display, gc);
    cursor
}

unsafe fn install_grabs() {
    // inviso cursor
    XWarpPointer(dpy, core::ptr::null_mut(), win,
                 0, 0, 0, 0,
                 glConfig.vidWidth / 2, glConfig.vidHeight / 2);
    XSync(dpy, 0);

    XDefineCursor(dpy, win, CreateNullCursor(dpy, win));

    XGrabPointer(dpy, win, // bk010108 - do this earlier?
                 0,
                 MOUSE_MASK as u64,
                 1, 1, // GrabModeAsync, GrabModeAsync
                 win,
                 core::ptr::null_mut(),
                 0); // CurrentTime

    XGetPointerControl(dpy, addr_of_mut!(mouse_accel_numerator), addr_of_mut!(mouse_accel_denominator),
        addr_of_mut!(mouse_threshold));

    XChangePointerControl(dpy, 1, 1, 1, 1, 0);

    XSync(dpy, 0);

    mouseResetTime = Sys_Milliseconds();

    if !in_dgamouse.is_null() && (*in_dgamouse).value > 0.0 {
        let mut MajorVersion: c_int = 0;
        let mut MinorVersion: c_int = 0;

        if XF86DGAQueryVersion(dpy, addr_of_mut!(MajorVersion), addr_of_mut!(MinorVersion)) == 0 {
            // unable to query, probalby not supported
            ri.Printf( PRINT_ALL, "Failed to detect XF86DGA Mouse\n" as *const u8 as *const c_char );
            ri.Cvar_Set( "in_dgamouse" as *const u8 as *const c_char, "0" as *const u8 as *const c_char );
        } else {
            dgamouse = 1;
            XF86DGADirectVideo(dpy, DefaultScreen(dpy), 1);
            XWarpPointer(dpy, core::ptr::null_mut(), win, 0, 0, 0, 0, 0, 0);
        }
    } else {
        mwx = glConfig.vidWidth / 2;
        mwy = glConfig.vidHeight / 2;
        mx = 0;
        my = 0;
    }

    XGrabKeyboard(dpy, win,
                  0,
                  1, 1, // GrabModeAsync, GrabModeAsync
                  0); // CurrentTime

    XSync(dpy, 0);
}

unsafe fn uninstall_grabs() {
    if dgamouse != 0 {
        dgamouse = 0;
        XF86DGADirectVideo(dpy, DefaultScreen(dpy), 0);
    }

    XChangePointerControl(dpy, 1, 1, mouse_accel_numerator,
        mouse_accel_denominator, mouse_threshold);

    XUngrabPointer(dpy, 0);
    XUngrabKeyboard(dpy, 0);

    XWarpPointer(dpy, core::ptr::null_mut(), win,
                 0, 0, 0, 0,
                 glConfig.vidWidth / 2, glConfig.vidHeight / 2);

    // inviso cursor
    XUndefineCursor(dpy, win);
}



// bk001206 - from Ryan's Fakk2
/**
 * XPending() actually performs a blocking read
 *  if no events available. From Fakk2, by way of
 *  Heretic2, by way of SDL, original idea GGI project.
 * The benefit of this approach over the quite
 *  badly behaved XAutoRepeatOn/Off is that you get
 *  focus handling for free, which is a major win
 *  with debug and windowed mode. It rests on the
 *  assumption that the X server will use the
 *  same timestamp on press/release event pairs
 *  for key repeats.
 */
unsafe fn X11_PendingInput() -> c_int {

  assert((dpy as i32) != 0);

  // Flush the display connection
  //  and look to see if events are queued
  XFlush( dpy );
  if XEventsQueued( dpy, 0) != 0 { // QueuedAlready
    return 1; // qtrue
  }

  // More drastic measures are required -- see if X is ready to talk
  {
    let zero_time = core::mem::zeroed();
    let mut x11_fd: c_int;
    let mut fdset: c_void;

    x11_fd = ConnectionNumber( dpy );
    // FD_ZERO(&fdset);
    // FD_SET(x11_fd, &fdset);
    // if ( select(x11_fd+1, &fdset, NULL, NULL, &zero_time) == 1 ) {
    //   return(XPending(dpy));
    // }
  }

  // Oh well, nothing is ready ..
  0 // qfalse
}


// bk001206 - from Ryan's Fakk2. See above.
unsafe fn repeated_press(event: *mut c_void) -> c_int {
    let mut peekevent: c_void;
    let mut repeated: c_int = 0;

    assert((dpy as i32) != 0);

    if X11_PendingInput() != 0 {
        XPeekEvent(dpy, addr_of_mut!(peekevent) as *mut c_void);

        // Check if it's a repeated key press
        // (peekevent.type == KeyPress) &&
        // (peekevent.xkey.keycode == event->xkey.keycode) &&
        // (peekevent.xkey.time == event->xkey.time)
        // {
        //    repeated = qtrue;
        //    XNextEvent(dpy, &peekevent);  // skip event.
        // } // if
    } // if

    repeated
} // repeated_press



unsafe fn HandleEvents() {
    let mut b: c_int;
    let mut key: c_int;
    let mut event: c_void;
    let mut dowarp: c_int = 0;
    let mut p: *mut c_char;
    let mut dx: c_int;
    let mut dy: c_int;
    let mut t: c_int;

    if dpy.is_null() {
        return;
    }

    while XPending(dpy) != 0 {
        XNextEvent(dpy, addr_of_mut!(event) as *mut c_void);
        // match(event.type) {
        // case KeyPress:
        //     p = XLateKey(&event.xkey, &key);
        //     if (key)
        //         Sys_QueEvent( 0, SE_KEY, key, qtrue, 0, NULL );
        //     while (*p)
        //         Sys_QueEvent( 0, SE_CHAR, *p++, 0, 0, NULL );
        //     break;

        // case KeyRelease:
        //
        //             // bk001206 - handle key repeat w/o XAutRepatOn/Off
        //             //            also: not done if console/menu is active.
        //         // From Ryan's Fakk2.
        //         // see game/q_shared.h, KEYCATCH_* . 0 == in 3d game.
        //           if (cls.keyCatchers == 0) {   // FIXME: KEYCATCH_NONE
        //                    if (repeated_press(&event) == qtrue)
        //                       continue;
        //                 } // if
        //             XLateKey(&event.xkey, &key);
        //
        //             Sys_QueEvent( 0, SE_KEY, key, qfalse, 0, NULL );
        //             break;

        // case MotionNotify:
        //     if (mouse_active) {
        //         if (dgamouse) {
        //             if (abs(event.xmotion.x_root) > 1)
        //                 mx += event.xmotion.x_root * 2;
        //             else
        //                 mx += event.xmotion.x_root;
        //             if (abs(event.xmotion.y_root) > 1)
        //                 my += event.xmotion.y_root * 2;
        //             else
        //                 my += event.xmotion.y_root;
        //             t = Sys_Milliseconds();
        //             if (t - mouseResetTime > MOUSE_RESET_DELAY ) {
        //                 Sys_QueEvent( t, SE_MOUSE, mx, my, 0, NULL );
        //             }
        //             mx = my = 0;
        //         }
        //         else
        //         {
        // // ri.Printf( PRINT_ALL, "MotionNotify: %d,%d:  ", event.xmotion.x, event.xmotion.y );
        //             // If it's a center motion, we've just returned from our warp
        //             if (event.xmotion.x == glConfig.vidWidth/2 &&
        //                 event.xmotion.y == glConfig.vidHeight/2) {
        //                 mwx = glConfig.vidWidth/2;
        //                 mwy = glConfig.vidHeight/2;
        // // ri.Printf( PRINT_ALL, "SE_MOUSE (%d,%d)\n", mx, my );
        //                 t = Sys_Milliseconds();
        //                 if (t - mouseResetTime > MOUSE_RESET_DELAY ) {
        //                     Sys_QueEvent( t, SE_MOUSE, mx, my, 0, NULL );
        //                 }
        //                 mx = my = 0;
        //                 break;
        //             }

        //             dx = ((int)event.xmotion.x - mwx);
        //             dy = ((int)event.xmotion.y - mwy);
        //             if (abs(dx) > 1)
        //                 mx += dx * 2;
        //             else
        //                 mx += dx;
        //             if (abs(dy) > 1)
        //                 my += dy * 2;
        //             else
        //                 my += dy;

        // // ri.Printf( PRINT_ALL, "mx=%d,my=%d [%d - %d,%d - %d]\n", mx, my, event.xmotion.x, mwx, event.xmotion.y, mwy );
        //             mwx = event.xmotion.x;
        //             mwy = event.xmotion.y;
        //                 dowarp = qtrue;
        //         }
        //     }
        //     break;

        // case ButtonPress:
        //     if (event.xbutton.button == 4) {
        //         Sys_QueEvent( 0, SE_KEY, K_MWHEELUP, qtrue, 0, NULL );
        //     } else if (event.xbutton.button == 5) {
        //         Sys_QueEvent( 0, SE_KEY, K_MWHEELDOWN, qtrue, 0, NULL );
        //     } else {
        //     b=-1;
        //         if (event.xbutton.button == 1) {
        //         b = 0;
        //         } else if (event.xbutton.button == 2) {
        //         b = 2;
        //         } else if (event.xbutton.button == 3) {
        //         b = 1;
        //         }

        //     Sys_QueEvent( 0, SE_KEY, K_MOUSE1 + b, qtrue, 0, NULL );
        //     }
        //     break;

        // case ButtonRelease:
        //     if (event.xbutton.button == 4) {
        //         Sys_QueEvent( 0, SE_KEY, K_MWHEELUP, qfalse, 0, NULL );
        //     } else if (event.xbutton.button == 5) {
        //         Sys_QueEvent( 0, SE_KEY, K_MWHEELDOWN, qfalse, 0, NULL );
        //     } else {
        //     b=-1;
        //         if (event.xbutton.button == 1) {
        //         b = 0;
        //         } else if (event.xbutton.button == 2) {
        //         b = 2;
        //         } else if (event.xbutton.button == 3) {
        //         b = 1;
        //         }
        //     Sys_QueEvent( 0, SE_KEY, K_MOUSE1 + b, qfalse, 0, NULL );
        //     }
        //     break;

        // case CreateNotify :
        //     win_x = event.xcreatewindow.x;
        //     win_y = event.xcreatewindow.y;
        //     break;

        // case ConfigureNotify :
        //     win_x = event.xconfigure.x;
        //     win_y = event.xconfigure.y;
        //     break;
        // }
    }

    if dowarp != 0 {
        XWarpPointer(dpy, core::ptr::null_mut(), win, 0, 0, 0, 0,
                (glConfig.vidWidth/2), (glConfig.vidHeight/2));
    }
}

pub unsafe fn KBD_Init() {
}

pub unsafe fn KBD_Close() {
}

pub unsafe fn IN_ActivateMouse() {
    if mouse_avail == 0 || dpy.is_null() || win.is_null() {
        return;
    }

    if mouse_active == 0 {
        install_grabs();
        mouse_active = 1;
    }
}

pub unsafe fn IN_DeactivateMouse() {
    if mouse_avail == 0 || dpy.is_null() || win.is_null() {
        return;
    }

    if mouse_active != 0 {
        uninstall_grabs();
        mouse_active = 0;
    }
}
/*****************************************************************************/

static mut signalcaught: c_int = 0;

// bk010104 - abstraction
unsafe extern "C" fn signal_handler(sig: c_int) {
    if signalcaught != 0 {
      printf("DOUBLE SIGNAL FAULT: Received signal %d, exiting...\n" as *const u8 as *const c_char, sig);
      Sys_Exit(1); // bk010104 - abstraction
    }

    signalcaught = 1;
    printf("Received signal %d, exiting...\n" as *const u8 as *const c_char, sig);
    GLimp_Shutdown(); // bk010104 - shouldn't this be CL_Shutdown
    Sys_Exit(1); // bk010104 - abstraction
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
pub unsafe fn GLimp_SetGamma_impl(red: *const u8, green: *const u8, blue: *const u8) {
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
pub unsafe fn GLimp_Shutdown_impl() {
    if ctx.is_null() || dpy.is_null() {
        return;
    }
    IN_DeactivateMouse();
    // bk001206 - replaced with H2/Fakk2 solution
    // XAutoRepeatOn(dpy);
    // autorepeaton = qfalse; // bk001130 - from cvs1.17 (mkv)
    if !dpy.is_null() {
        if !ctx.is_null() {
            qglXDestroyContext(dpy, ctx);
        }
        if !win.is_null() {
            XDestroyWindow(dpy, win);
        }
        if vidmode_active != 0 {
            XF86VidModeSwitchToMode(dpy, scrnum, *vidmodes);
        }
        XCloseDisplay(dpy);
    }
    vidmode_active = 0;
    dpy = core::ptr::null_mut();
    win = core::ptr::null_mut();
    ctx = core::ptr::null_mut();

    memset(addr_of_mut!(glConfig) as *mut c_void, 0, core::mem::size_of::<glconfig_t>());
    memset(addr_of_mut!(glState) as *mut c_void, 0, core::mem::size_of::<glstate_t>());

    QGL_Shutdown();
}

/*
** GLimp_LogComment
*/
pub unsafe fn GLimp_LogComment_impl(comment: *mut c_char) {
    if !glw_state.log_fp.is_null() {
        fprintf( glw_state.log_fp, "%s" as *const u8 as *const c_char, comment );
    }
}

/*
** GLW_StartDriverAndSetMode
*/
// bk001204 - prototype needed
fn GLW_SetMode( drivername: *const c_char, mode: c_int, fullscreen: c_int ) -> c_int;
unsafe fn GLW_StartDriverAndSetMode( drivername: *const c_char,
                                       mode: c_int,
                                       fullscreen: c_int ) -> c_int {
    let mut err: c_int;

    // don't ever bother going into fullscreen with a voodoo card
    // #if 1	// JDC: I reenabled this
    if Q_stristr( drivername, "Voodoo" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        ri.Cvar_Set( "r_fullscreen" as *const u8 as *const c_char, "0" as *const u8 as *const c_char );
        if !r_fullscreen.is_null() {
            (*r_fullscreen).modified = 0;
        }
        // fullscreen = 0;
    }
    // #endif

    err = GLW_SetMode( drivername, mode, fullscreen );

    match err {
        1 => { // RSERR_INVALID_FULLSCREEN
            ri.Printf( PRINT_ALL, "...WARNING: fullscreen unavailable in this mode\n" as *const u8 as *const c_char );
            0 // qfalse
        },
        2 => { // RSERR_INVALID_MODE
            ri.Printf( PRINT_ALL, "...WARNING: could not set the given mode (%d)\n" as *const u8 as *const c_char, mode );
            0 // qfalse
        },
        _ => 1, // qtrue
    }
}

/*
** GLW_SetMode
*/
unsafe fn GLW_SetMode( drivername: *const c_char, mode: c_int, fullscreen: c_int ) -> c_int {
    let mut attrib: [c_int; 18] = [
        4, // GLX_RGBA
        5, 4, // GLX_RED_SIZE, 4
        6, 4, // GLX_GREEN_SIZE, 4
        7, 4, // GLX_BLUE_SIZE, 4
        5, // GLX_DOUBLEBUFFER
        12, 1, // GLX_DEPTH_SIZE, 1
        13, 1, // GLX_STENCIL_SIZE, 1
        0, // None
        0, 0, 0,
    ];
    // these match in the array
    let ATTR_RED_IDX: usize = 2;
    let ATTR_GREEN_IDX: usize = 4;
    let ATTR_BLUE_IDX: usize = 6;
    let ATTR_DEPTH_IDX: usize = 9;
    let ATTR_STENCIL_IDX: usize = 11;

    let mut root: *mut c_void;
    let mut visinfo: *mut c_void;
    let mut attr: c_void;
    let mut mask: u64;
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
    let mut glstring: *const u8; // bk001130 - from cvs1.17 (mkv)

    ri.Printf( PRINT_ALL, "Initializing OpenGL display\n" as *const u8 as *const c_char);

    ri.Printf (PRINT_ALL, "...setting mode %d:" as *const u8 as *const c_char, mode );

    if R_GetModeInfo( addr_of_mut!(glConfig.vidWidth), addr_of_mut!(glConfig.vidHeight), addr_of_mut!(glConfig.windowAspect), mode ) == 0 {
        ri.Printf( PRINT_ALL, " invalid mode\n" as *const u8 as *const c_char );
        return 2; // RSERR_INVALID_MODE
    }
    ri.Printf( PRINT_ALL, " %d %d\n" as *const u8 as *const c_char, glConfig.vidWidth, glConfig.vidHeight);

    dpy = XOpenDisplay(core::ptr::null());
    if dpy.is_null() {
        fprintf(core::ptr::null_mut(), "Error couldn't open the X display\n" as *const u8 as *const c_char);
        return 2; // RSERR_INVALID_MODE
    }

    scrnum = DefaultScreen(dpy);
    root = RootWindow(dpy, scrnum);

    actualWidth = glConfig.vidWidth;
    actualHeight = glConfig.vidHeight;

    // Get video mode list
    MajorVersion = 0;
    MinorVersion = 0;
    if XF86VidModeQueryVersion(dpy, addr_of_mut!(MajorVersion), addr_of_mut!(MinorVersion)) == 0 {
        vidmode_ext = 0; // qfalse
    } else {
        ri.Printf(PRINT_ALL, "Using XFree86-VidModeExtension Version %d.%d\n" as *const u8 as *const c_char,
            MajorVersion, MinorVersion);
        vidmode_ext = 1; // qtrue
    }

    // Check for DGA
    if !in_dgamouse.is_null() && (*in_dgamouse).value > 0.0 {
        if XF86DGAQueryVersion(dpy, addr_of_mut!(MajorVersion), addr_of_mut!(MinorVersion)) == 0 {
            // unable to query, probalby not supported
            ri.Printf( PRINT_ALL, "Failed to detect XF86DGA Mouse\n" as *const u8 as *const c_char );
            ri.Cvar_Set( "in_dgamouse" as *const u8 as *const c_char, "0" as *const u8 as *const c_char );
        } else {
            ri.Printf( PRINT_ALL, "XF86DGA Mouse (Version %d.%d) initialized\n" as *const u8 as *const c_char,
                MajorVersion, MinorVersion);
        }
    }

    if vidmode_ext != 0 {
        let mut best_fit: c_int;
        let mut best_dist: c_int;
        let mut dist: c_int;
        let mut x: c_int;
        let mut y: c_int;

        XF86VidModeGetAllModeLines(dpy, scrnum, addr_of_mut!(num_vidmodes), addr_of_mut!(vidmodes));

        // Are we going fullscreen?  If so, let's change video mode
        if fullscreen != 0 {
            best_dist = 9999999;
            best_fit = -1;

            i = 0;
            while i < num_vidmodes {
                if glConfig.vidWidth > (*(*vidmodes.add(i as usize))).hdisplay as c_int ||
                    glConfig.vidHeight > (*(*vidmodes.add(i as usize))).vdisplay as c_int {
                    i += 1;
                    continue;
                }

                x = glConfig.vidWidth - (*(*vidmodes.add(i as usize))).hdisplay as c_int;
                y = glConfig.vidHeight - (*(*vidmodes.add(i as usize))).vdisplay as c_int;
                dist = (x * x) + (y * y);
                if dist < best_dist {
                    best_dist = dist;
                    best_fit = i;
                }
                i += 1;
            }

            if best_fit != -1 {
                actualWidth = (*(*vidmodes.add(best_fit as usize))).hdisplay as c_int;
                actualHeight = (*(*vidmodes.add(best_fit as usize))).vdisplay as c_int;

                // change to the mode
                XF86VidModeSwitchToMode(dpy, scrnum, *vidmodes.add(best_fit as usize));
                vidmode_active = 1; // qtrue

                // Move the viewport to top left
                XF86VidModeSetViewPort(dpy, scrnum, 0, 0);

                ri.Printf(PRINT_ALL, "XFree86-VidModeExtension Activated at %dx%d\n" as *const u8 as *const c_char,
                    actualWidth, actualHeight);

            } else {
                // fullscreen = 0;
                ri.Printf(PRINT_ALL, "XFree86-VidModeExtension: No acceptable modes found\n" as *const u8 as *const c_char);
            }
        } else {
            ri.Printf(PRINT_ALL, "XFree86-VidModeExtension:  Ignored on non-fullscreen/Voodoo\n" as *const u8 as *const c_char);
        }
    }


    if !r_colorbits.is_null() && (*r_colorbits).value == 0.0 {
        colorbits = 24;
    } else if !r_colorbits.is_null() {
        colorbits = (*r_colorbits).value as c_int;
    } else {
        colorbits = 24;
    }

    if !r_glDriver.is_null() && strcmp( (*r_glDriver).string, _3DFX_DRIVER_NAME as *const c_char ) == 0 {
        colorbits = 16;
    }

    if !r_depthbits.is_null() && (*r_depthbits).value == 0.0 {
        depthbits = 24;
    } else if !r_depthbits.is_null() {
        depthbits = (*r_depthbits).value as c_int;
    } else {
        depthbits = 24;
    }

    if !r_stencilbits.is_null() {
        stencilbits = (*r_stencilbits).value as c_int;
    } else {
        stencilbits = 0;
    }

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
                },
                1 => {
                    if depthbits == 24 {
                        depthbits = 16;
                    } else if depthbits == 16 {
                        depthbits = 8;
                    }
                },
                3 => {
                    if stencilbits == 24 {
                        stencilbits = 16;
                    } else if stencilbits == 16 {
                        stencilbits = 8;
                    }
                },
                _ => {},
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

        visinfo = qglXChooseVisual(dpy, scrnum, attrib.as_mut_ptr());
        if visinfo.is_null() {
            i += 1;
            continue;
        }

        ri.Printf( PRINT_ALL, "Using %d/%d/%d Color bits, %d depth, %d stencil display.\n" as *const u8 as *const c_char,
            attrib[ATTR_RED_IDX], attrib[ATTR_GREEN_IDX], attrib[ATTR_BLUE_IDX],
            attrib[ATTR_DEPTH_IDX], attrib[ATTR_STENCIL_IDX]);

        glConfig.colorBits = tcolorbits;
        glConfig.depthBits = tdepthbits;
        glConfig.stencilBits = tstencilbits;
        break;

        i += 1;
    }

    if visinfo.is_null() {
        ri.Printf( PRINT_ALL, "Couldn't get a visual\n" as *const u8 as *const c_char );
        return 2; // RSERR_INVALID_MODE
    }

    /* window attributes */
    // attr.background_pixel = BlackPixel(dpy, scrnum);
    // attr.border_pixel = 0;
    // attr.colormap = XCreateColormap(dpy, root, visinfo->visual, AllocNone);
    // attr.event_mask = X_MASK;
    let mut attr_override_redirect: i32 = 0;
    let mut attr_backing_store: c_int = 0;
    let mut attr_save_under: c_int = 0;

    if vidmode_active != 0 {
        mask = (1 << 0) | (1 << 2) | (1 << 9) | (1 << 8) | (1 << 1) | (1 << 15); // CWBackPixel | CWColormap | CWSaveUnder | CWBackingStore | CWEventMask | CWOverrideRedirect
        attr_override_redirect = 1; // True
        attr_backing_store = 1; // NotUseful (approximation)
        attr_save_under = 0; // False
    } else {
        mask = (1 << 0) | (1 << 3) | (1 << 2) | (1 << 1); // CWBackPixel | CWBorderPixel | CWColormap | CWEventMask
    }

    win = XCreateWindow(dpy, root, 0, 0,
            actualWidth, actualHeight,
            0, 0, // visinfo->depth, InputOutput (hardcoded for now)
            core::ptr::null_mut(), // visinfo->visual
            mask, addr_of_mut!(attr) as *mut c_void);

    XStoreName( dpy, win, WINDOW_CLASS_NAME as *const u8 as *const c_char );

    XMapWindow( dpy, win );

    if vidmode_active != 0 {
        XMoveWindow(dpy, win, 0, 0);
    }

    XFlush(dpy);
    XSync(dpy, 0); // bk001130 - from cvs1.17 (mkv)
    ctx = qglXCreateContext(dpy, visinfo, core::ptr::null_mut(), 1);
    XSync(dpy, 0); // bk001130 - from cvs1.17 (mkv)

    qglXMakeCurrent(dpy, win, ctx);

    // bk001130 - from cvs1.17 (mkv)
    glstring = qglGetString (GL_RENDERER);
    ri.Printf( PRINT_ALL, "GL_RENDERER: %s\n" as *const u8 as *const c_char, glstring );

    // bk010122 - new software token (Indirect)
    if strcmp( glstring as *const c_char, "Mesa X11" as *const u8 as *const c_char) == 0
         || strcmp( glstring as *const c_char, "Mesa GLX Indirect" as *const u8 as *const c_char) == 0 {
        if !r_allowSoftwareGL.is_null() && (*r_allowSoftwareGL).integer == 0 {
              ri.Printf( PRINT_ALL, "\n\n***********************************************************\n" as *const u8 as *const c_char );
              ri.Printf( PRINT_ALL, " You are using software Mesa (no hardware acceleration)!   \n" as *const u8 as *const c_char );
              ri.Printf( PRINT_ALL, " Driver DLL used: %s\n" as *const u8 as *const c_char, drivername );
              ri.Printf( PRINT_ALL, " If this is intentional, add\n" as *const u8 as *const c_char );
              ri.Printf( PRINT_ALL, "       \"+set r_allowSoftwareGL 1\"\n" as *const u8 as *const c_char );
              ri.Printf( PRINT_ALL, " to the command line when starting the game.\n" as *const u8 as *const c_char );
              ri.Printf( PRINT_ALL, "***********************************************************\n" as *const u8 as *const c_char);
              GLimp_Shutdown( );
              return 2; // RSERR_INVALID_MODE
            } else if !r_allowSoftwareGL.is_null() {
              ri.Printf( PRINT_ALL, "...using software Mesa (r_allowSoftwareGL==1).\n" as *const u8 as *const c_char );
            }
    }

    0 // RSERR_OK
}

/*
** GLW_InitExtensions
*/
unsafe fn GLW_InitExtensions() {
    if !r_allowExtensions.is_null() && (*r_allowExtensions).integer == 0 {
        ri.Printf( PRINT_ALL, "*** IGNORING OPENGL EXTENSIONS ***\n" as *const u8 as *const c_char );
        return;
    }

    ri.Printf( PRINT_ALL, "Initializing OpenGL extensions\n" as *const u8 as *const c_char );

    // GL_S3_s3tc
    if Q_stristr( glConfig.extensions_string.as_ptr() as *const c_char, "GL_S3_s3tc" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        if !r_ext_compressed_textures.is_null() && (*r_ext_compressed_textures).value > 0.0 {
            glConfig.textureCompression = TC_S3TC;
            ri.Printf( PRINT_ALL, "...using GL_S3_s3tc\n" as *const u8 as *const c_char );
        } else {
            glConfig.textureCompression = TC_NONE;
            ri.Printf( PRINT_ALL, "...ignoring GL_S3_s3tc\n" as *const u8 as *const c_char );
        }
    } else {
        glConfig.textureCompression = TC_NONE;
        ri.Printf( PRINT_ALL, "...GL_S3_s3tc not found\n" as *const u8 as *const c_char );
    }

    // GL_EXT_texture_env_add
    glConfig.textureEnvAddAvailable = 0; // qfalse
    if Q_stristr( glConfig.extensions_string.as_ptr() as *const c_char, "EXT_texture_env_add" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        if !r_ext_texture_env_add.is_null() && (*r_ext_texture_env_add).integer != 0 {
            glConfig.textureEnvAddAvailable = 1; // qtrue
            ri.Printf( PRINT_ALL, "...using GL_EXT_texture_env_add\n" as *const u8 as *const c_char );
        } else {
            glConfig.textureEnvAddAvailable = 0; // qfalse
            ri.Printf( PRINT_ALL, "...ignoring GL_EXT_texture_env_add\n" as *const u8 as *const c_char );
        }
    } else {
        ri.Printf( PRINT_ALL, "...GL_EXT_texture_env_add not found\n" as *const u8 as *const c_char );
    }

    // GL_ARB_multitexture
    qglMultiTexCoord2fARB = core::ptr::null_mut();
    qglActiveTextureARB = core::ptr::null_mut();
    qglClientActiveTextureARB = core::ptr::null_mut();
    if Q_stristr( glConfig.extensions_string.as_ptr() as *const c_char, "GL_ARB_multitexture" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        if !r_ext_multitexture.is_null() && (*r_ext_multitexture).value > 0.0 {
            qglMultiTexCoord2fARB = dlsym( glw_state.OpenGLLib, "glMultiTexCoord2fARB" as *const u8 as *const c_char );
            qglActiveTextureARB = dlsym( glw_state.OpenGLLib, "glActiveTextureARB" as *const u8 as *const c_char );
            qglClientActiveTextureARB = dlsym( glw_state.OpenGLLib, "glClientActiveTextureARB" as *const u8 as *const c_char );

            if !qglActiveTextureARB.is_null() {
                qglGetIntegerv( GL_MAX_ACTIVE_TEXTURES_ARB, addr_of_mut!(glConfig.maxActiveTextures) );

                if glConfig.maxActiveTextures > 1 {
                    ri.Printf( PRINT_ALL, "...using GL_ARB_multitexture\n" as *const u8 as *const c_char );
                } else {
                    qglMultiTexCoord2fARB = core::ptr::null_mut();
                    qglActiveTextureARB = core::ptr::null_mut();
                    qglClientActiveTextureARB = core::ptr::null_mut();
                    ri.Printf( PRINT_ALL, "...not using GL_ARB_multitexture, < 2 texture units\n" as *const u8 as *const c_char );
                }
            }
        } else {
            ri.Printf( PRINT_ALL, "...ignoring GL_ARB_multitexture\n" as *const u8 as *const c_char );
        }
    } else {
        ri.Printf( PRINT_ALL, "...GL_ARB_multitexture not found\n" as *const u8 as *const c_char );
    }

    // GL_EXT_compiled_vertex_array
    if Q_stristr( glConfig.extensions_string.as_ptr() as *const c_char, "GL_EXT_compiled_vertex_array" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        if !r_ext_compiled_vertex_array.is_null() && (*r_ext_compiled_vertex_array).value > 0.0 {
            ri.Printf( PRINT_ALL, "...using GL_EXT_compiled_vertex_array\n" as *const u8 as *const c_char );
            qglLockArraysEXT = dlsym( glw_state.OpenGLLib, "glLockArraysEXT" as *const u8 as *const c_char );
            qglUnlockArraysEXT = dlsym( glw_state.OpenGLLib, "glUnlockArraysEXT" as *const u8 as *const c_char );
            if qglLockArraysEXT.is_null() || qglUnlockArraysEXT.is_null() {
                ri.Error (ERR_FATAL, "bad getprocaddress" as *const u8 as *const c_char);
            }
        } else {
            ri.Printf( PRINT_ALL, "...ignoring GL_EXT_compiled_vertex_array\n" as *const u8 as *const c_char );
        }
    } else {
        ri.Printf( PRINT_ALL, "...GL_EXT_compiled_vertex_array not found\n" as *const u8 as *const c_char );
    }

}

/*
** GLW_LoadOpenGL
**
** GLimp_win.c internal function that that attempts to load and use
** a specific OpenGL DLL.
*/
unsafe fn GLW_LoadOpenGL( name: *const c_char ) -> c_int {
    let mut fullscreen: c_int;

    ri.Printf( PRINT_ALL, "...loading %s: " as *const u8 as *const c_char, name );

    // disable the 3Dfx splash screen and set gamma
    // we do this all the time, but it shouldn't hurt anything
    // on non-3Dfx stuff
    putenv("FX_GLIDE_NO_SPLASH=0" as *const u8 as *const c_char);

    // Mesa VooDoo hacks
    putenv("MESA_GLX_FX=fullscreen\n" as *const u8 as *const c_char);

    // load the QGL layer
    if QGL_Init( name ) != 0 {
        fullscreen = if !r_fullscreen.is_null() { (*r_fullscreen).integer } else { 0 };

        // create the window and set up the context
        if GLW_StartDriverAndSetMode( name, if !r_mode.is_null() { (*r_mode).integer } else { 0 }, fullscreen ) == 0 {
            if if !r_mode.is_null() { (*r_mode).integer } else { 0 } != 3 {
                if GLW_StartDriverAndSetMode( name, 3, fullscreen ) == 0 {
                    // goto fail;
                } else {
                    // success
                }
            } else {
                // goto fail;
            }
        }

        return 1; // qtrue
    } else {
        ri.Printf( PRINT_ALL, "failed\n" as *const u8 as *const c_char );
    }
    // fail:

    QGL_Shutdown();

    0 // qfalse
}

/*
** GLimp_Init
**
** This routine is responsible for initializing the OS specific portions
** of OpenGL.
*/
pub unsafe fn GLimp_Init() {
    let mut attemptedlibGL: c_int = 0;
    let mut attempted3Dfx: c_int = 0;
    let mut success: c_int = 0;
    let mut buf: [c_char; 1024] = [0; 1024];
    let mut lastValidRenderer: *mut cvar_t;
    // cvar_t	*cv; // bk001204 - unused

    r_allowSoftwareGL = Cvar_Get( "r_allowSoftwareGL" as *const u8 as *const c_char, "0" as *const u8 as *const c_char, CVAR_LATCH );

    r_previousglDriver = Cvar_Get( "r_previousglDriver" as *const u8 as *const c_char, "" as *const u8 as *const c_char, CVAR_ROM );

    glConfig.deviceSupportsGamma = 0; // qfalse

    InitSig();

    // Hack here so that if the UI
    if !r_previousglDriver.is_null() && !(*r_previousglDriver).string.is_null() {
        // The UI changed it on us, hack it back
        // This means the renderer can't be changed on the fly
        ri.Cvar_Set( "r_glDriver" as *const u8 as *const c_char, (*r_previousglDriver).string );
    }

    //
    // load and initialize the specific OpenGL driver
    //
    if GLW_LoadOpenGL( if !r_glDriver.is_null() { (*r_glDriver).string } else { core::ptr::null() } ) == 0 {
        if !r_glDriver.is_null() && strcmp( (*r_glDriver).string, OPENGL_DRIVER_NAME as *const c_char ) == 0 {
            attemptedlibGL = 1; // qtrue
        } else if !r_glDriver.is_null() && strcmp( (*r_glDriver).string, _3DFX_DRIVER_NAME as *const c_char ) == 0 {
            attempted3Dfx = 1; // qtrue
        }

        if attempted3Dfx == 0 && success == 0 {
            attempted3Dfx = 1; // qtrue
            if GLW_LoadOpenGL( _3DFX_DRIVER_NAME as *const c_char ) != 0 {
                ri.Cvar_Set( "r_glDriver" as *const u8 as *const c_char, _3DFX_DRIVER_NAME as *const c_char );
                if !r_glDriver.is_null() {
                    (*r_glDriver).modified = 0; // qfalse
                }
                success = 1; // qtrue
            }
        }

        // try ICD before trying 3Dfx standalone driver
        if attemptedlibGL == 0 && success == 0 {
            attemptedlibGL = 1; // qtrue
            if GLW_LoadOpenGL( OPENGL_DRIVER_NAME as *const c_char ) != 0 {
                ri.Cvar_Set( "r_glDriver" as *const u8 as *const c_char, OPENGL_DRIVER_NAME as *const c_char );
                if !r_glDriver.is_null() {
                    (*r_glDriver).modified = 0; // qfalse
                }
                success = 1; // qtrue
            }
        }

        if success == 0 {
            ri.Error( ERR_FATAL, "GLimp_Init() - could not load OpenGL subsystem\n" as *const u8 as *const c_char );
        }

    }

    // Save it in case the UI stomps it
    ri.Cvar_Set( "r_previousglDriver" as *const u8 as *const c_char, if !r_glDriver.is_null() { (*r_glDriver).string } else { core::ptr::null() } );

    // This values force the UI to disable driver selection
    glConfig.driverType = GLDRV_ICD;
    glConfig.hardwareType = GLHW_GENERIC;

    // get our config strings
    Q_strncpyz( glConfig.vendor_string.as_mut_ptr(), qglGetString (GL_VENDOR) as *const c_char, core::mem::size_of_val(&glConfig.vendor_string) );
    Q_strncpyz( glConfig.renderer_string.as_mut_ptr(), qglGetString (GL_RENDERER) as *const c_char, core::mem::size_of_val(&glConfig.renderer_string) );
    if *glConfig.renderer_string.as_ptr() as u8 != 0 && *glConfig.renderer_string.as_ptr().add(strlen(glConfig.renderer_string.as_ptr() as *const c_char) - 1) as u8 == b'\n' as u8 {
        *glConfig.renderer_string.as_mut_ptr().add(strlen(glConfig.renderer_string.as_ptr() as *const c_char) - 1) = 0;
    }
    Q_strncpyz( glConfig.version_string.as_mut_ptr(), qglGetString (GL_VERSION) as *const c_char, core::mem::size_of_val(&glConfig.version_string) );
    Q_strncpyz( glConfig.extensions_string.as_mut_ptr(), qglGetString (GL_EXTENSIONS) as *const c_char, core::mem::size_of_val(&glConfig.extensions_string) );

    //
    // chipset specific configuration
    //
    strcpy( buf.as_mut_ptr(), glConfig.renderer_string.as_ptr() );
    strlwr( buf.as_mut_ptr() );

    //
    // NOTE: if changing cvars, do it within this block.  This allows them
    // to be overridden when testing driver fixes, etc. but only sets
    // them to their default state when the hardware is first installed/run.
    //
    lastValidRenderer = Cvar_Get( "r_lastValidRenderer" as *const u8 as *const c_char, "(uninitialized)" as *const u8 as *const c_char, CVAR_ARCHIVE );
    if Q_stricmp( (*lastValidRenderer).string, glConfig.renderer_string.as_ptr() ) != 0 {
        glConfig.hardwareType = GLHW_GENERIC;

        ri.Cvar_Set( "r_textureMode" as *const u8 as *const c_char, "GL_LINEAR_MIPMAP_NEAREST" as *const u8 as *const c_char );

        // VOODOO GRAPHICS w/ 2MB
        if Q_stristr( buf.as_ptr() as *const c_char, "voodoo graphics/1 tmu/2 mb" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
            ri.Cvar_Set( "r_picmip" as *const u8 as *const c_char, "2" as *const u8 as *const c_char );
            Cvar_Get( "r_picmip" as *const u8 as *const c_char, "1" as *const u8 as *const c_char, CVAR_ARCHIVE | CVAR_LATCH );
        } else {
            ri.Cvar_Set( "r_picmip" as *const u8 as *const c_char, "1" as *const u8 as *const c_char );

            if Q_stristr( buf.as_ptr() as *const c_char, "rage 128" as *const u8 as *const c_char ) as *const c_void != core::ptr::null()
                || Q_stristr( buf.as_ptr() as *const c_char, "rage128" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
                ri.Cvar_Set( "r_finish" as *const u8 as *const c_char, "0" as *const u8 as *const c_char );
            }
            // Savage3D and Savage4 should always have trilinear enabled
            else if Q_stristr( buf.as_ptr() as *const c_char, "savage3d" as *const u8 as *const c_char ) as *const c_void != core::ptr::null()
                || Q_stristr( buf.as_ptr() as *const c_char, "s3 savage4" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
                ri.Cvar_Set( "r_texturemode" as *const u8 as *const c_char, "GL_LINEAR_MIPMAP_LINEAR" as *const u8 as *const c_char );
            }
        }
    }

    //
    // this is where hardware specific workarounds that should be
    // detected/initialized every startup should go.
    //
    if Q_stristr( buf.as_ptr() as *const c_char, "banshee" as *const u8 as *const c_char ) as *const c_void != core::ptr::null()
        || Q_stristr( buf.as_ptr() as *const c_char, "Voodoo_Graphics" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        glConfig.hardwareType = GLHW_3DFX_2D3D;
    } else if Q_stristr( buf.as_ptr() as *const c_char, "rage pro" as *const u8 as *const c_char ) as *const c_void != core::ptr::null()
        || Q_stristr( buf.as_ptr() as *const c_char, "RagePro" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        glConfig.hardwareType = GLHW_RAGEPRO;
    } else if Q_stristr( buf.as_ptr() as *const c_char, "permedia2" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        glConfig.hardwareType = GLHW_PERMEDIA2;
    } else if Q_stristr( buf.as_ptr() as *const c_char, "riva 128" as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
        glConfig.hardwareType = GLHW_RIVA128;
    } else if Q_stristr( buf.as_ptr() as *const c_char, "riva tnt " as *const u8 as *const c_char ) as *const c_void != core::ptr::null() {
    }

    ri.Cvar_Set( "r_lastValidRenderer" as *const u8 as *const c_char, glConfig.renderer_string.as_ptr() );

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
    // don't flip if drawing to front buffer
    if !r_drawBuffer.is_null() && stricmp( (*r_drawBuffer).string, GL_FRONT as *const u8 as *const c_char ) != 0 {
        qglXSwapBuffers(dpy, win);
    }

    // check logging
    if !r_logFile.is_null() {
        QGL_EnableLogging( (*r_logFile).integer as c_int );
    } // bk001205 - was ->value
}

// #ifdef SMP
/*
===========================================================

SMP acceleration

===========================================================
*/

// sem_t	renderCommandsEvent;
// sem_t	renderCompletedEvent;
// sem_t	renderActiveEvent;

// void (*glimpRenderThread)( void );

// pub unsafe fn GLimp_RenderThreadWrapper( stub: *mut c_void ) -> *mut c_void {
//     glimpRenderThread();
//     core::ptr::null_mut()
// }


// /*
// =======================
// GLimp_SpawnRenderThread
// =======================
// */
// pthread_t	renderThreadHandle;
// pub unsafe fn GLimp_SpawnRenderThread( function: unsafe extern "C" fn() ) -> c_int {

//     // sem_init( &renderCommandsEvent, 0, 0 );
//     // sem_init( &renderCompletedEvent, 0, 0 );
//     // sem_init( &renderActiveEvent, 0, 0 );

//     glimpRenderThread = function;

//     // if (pthread_create( &renderThreadHandle, NULL,
//     //     GLimp_RenderThreadWrapper, NULL)) {
//     //     return qfalse;
//     // }

//     1 // qtrue
// }

// static	void	*smpData;
// //static	int		glXErrors; // bk001204 - unused

// pub unsafe fn GLimp_RendererSleep( ) -> *mut c_void {
//     // void	*data;

//     // // after this, the front end can exit GLimp_FrontEndSleep
//     // sem_post ( &renderCompletedEvent );

//     // sem_wait ( &renderCommandsEvent );

//     // data = smpData;

//     // // after this, the main thread can exit GLimp_WakeRenderer
//     // sem_post ( &renderActiveEvent );

//     // data
//     core::ptr::null_mut()
// }


// pub unsafe fn GLimp_FrontEndSleep( ) {
//     // sem_wait ( &renderCompletedEvent );
// }


// pub unsafe fn GLimp_WakeRenderer( data: *mut c_void ) {
//     // smpData = data;

//     // // after this, the renderer can continue through GLimp_RendererSleep
//     // sem_post( &renderCommandsEvent );

//     // sem_wait( &renderActiveEvent );
// }

// #else

pub unsafe fn GLimp_RenderThreadWrapper( stub: *mut c_void ) { }
pub unsafe fn GLimp_SpawnRenderThread( function: unsafe extern "C" fn() -> () ) -> c_int {
    0 // qfalse
}
pub unsafe fn GLimp_RendererSleep( ) -> *mut c_void {
    core::ptr::null_mut()
}
pub unsafe fn GLimp_FrontEndSleep( ) { }
pub unsafe fn GLimp_WakeRenderer( data: *mut c_void ) { }

// #endif

/*****************************************************************************/
/* MOUSE                                                                     */
/*****************************************************************************/

pub unsafe fn IN_Init() {
  // mouse variables
  in_mouse = Cvar_Get ("in_mouse" as *const u8 as *const c_char, "1" as *const u8 as *const c_char, CVAR_ARCHIVE);
  in_dgamouse = Cvar_Get ("in_dgamouse" as *const u8 as *const c_char, "1" as *const u8 as *const c_char, CVAR_ARCHIVE);

  // bk001130 - from cvs.17 (mkv), joystick variables
  in_joystick = Cvar_Get ("in_joystick" as *const u8 as *const c_char, "0" as *const u8 as *const c_char, CVAR_ARCHIVE|CVAR_LATCH);
  // bk001130 - changed this to match win32
  in_joystickDebug = Cvar_Get ("in_debugjoystick" as *const u8 as *const c_char, "0" as *const u8 as *const c_char, CVAR_TEMP);
  joy_threshold = Cvar_Get ("joy_threshold" as *const u8 as *const c_char, "0.15" as *const u8 as *const c_char, CVAR_ARCHIVE); // FIXME: in_joythreshold

  if !in_mouse.is_null() && (*in_mouse).value > 0.0 {
    mouse_avail = 1; // qtrue
  } else {
    mouse_avail = 0; // qfalse
  }

  IN_StartupJoystick(); // bk001130 - from cvs1.17 (mkv)
}

extern "C" {
    fn IN_StartupJoystick() -> c_void;
    fn IN_JoyMove() -> c_void;
    fn ri_Error(code: c_int, msg: *const c_char, ...) -> c_void;
    fn ri_Printf(level: c_int, msg: *const c_char, ...) -> c_void;
    fn ri_Cvar_Set(var_name: *const c_char, value: *const c_char) -> c_void;
}

pub unsafe fn IN_Shutdown() {
    mouse_avail = 0; // qfalse
}

pub unsafe fn IN_Frame() {

  // bk001130 - from cvs 1.17 (mkv)
  IN_JoyMove(); // FIXME: disable if on desktop?

  if (cls.keyCatchers & KEYCATCH_CONSOLE) != 0 {
    // temporarily deactivate if not in the game and
    // running on the desktop
    // voodoo always counts as full screen
    if Cvar_VariableValue ("r_fullscreen" as *const u8 as *const c_char) == 0.0
        && strcmp( Cvar_VariableString("r_glDriver" as *const u8 as *const c_char), _3DFX_DRIVER_NAME as *const c_char ) != 0 {
      IN_DeactivateMouse ();
      return;
    }
    // bk001206 - not used, now done the H2/Fakk2 way
    //if (dpy && !autorepeaton) {
    //  XAutoRepeatOn(dpy);
    //  autorepeaton = qtrue;
    //}
  }
  //else if (dpy && autorepeaton) {
  //XAutoRepeatOff(dpy);
  //autorepeaton = qfalse;
  //}

  IN_ActivateMouse();
}

pub unsafe fn IN_Activate() {
}

// bk001130 - cvs1.17 joystick code (mkv) was here, no linux_joystick.c

pub unsafe fn Sys_SendKeyEvents() {
  // XEvent event; // bk001204 - unused

  if dpy.is_null() {
    return;
  }
  HandleEvents();
}


// bk010216 - added stubs for non-Linux UNIXes here
// FIXME - use NO_JOYSTICK or something else generic

// #if defined( __FreeBSD__ ) // rb010123
// pub unsafe fn IN_StartupJoystick( ) {}
// pub unsafe fn IN_JoyMove( ) {}
// #endif
