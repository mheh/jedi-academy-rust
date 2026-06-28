// leave this as first line for PCH reasons...
//

/*
** WIN_GLIMP.C
**
** This file contains ALL Win32 specific stuff having to do with the
** OpenGL refresh.  When a port is being made the following functions
** must be implemented by the port:
**
** GLimp_EndFrame
** GLimp_Init
** GLimp_LogComment
** GLimp_Shutdown
**
** Note that the GLW_xxx functions are Windows specific GL-subsystem
** related functions that are relevant ONLY to win_glimp.c
*/

use core::ffi::{c_int, c_char, c_void};

extern "C" {
    fn WG_CheckHardwareGamma();
    fn WG_RestoreGamma();
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
enum rserr_t {
    RSERR_OK = 0,
    RSERR_INVALID_FULLSCREEN = 1,
    RSERR_INVALID_MODE = 2,
    RSERR_UNKNOWN = 3,
}

const TRY_PFD_SUCCESS: c_int = 0;
const TRY_PFD_FAIL_SOFT: c_int = 1;
const TRY_PFD_FAIL_HARD: c_int = 2;

const WINDOW_CLASS_NAME: &[u8] = b"Jedi Knight\xae: Jedi Academy\0";

static mut s_classRegistered: bool = false;

// function declaration
extern "C" {
    fn QGL_EnableLogging(enable: bool);
    fn QGL_Init(dllname: *const c_char) -> bool;
    fn QGL_Shutdown();
}

// variable declarations
#[repr(C)]
pub struct glwstate_t {
    // TODO: Define glwstate_t fields based on the original C struct
}

extern "C" {
    pub static mut glw_state: glwstate_t;
}

// cvar pointers
extern "C" {
    static mut r_allowSoftwareGL: *mut cvar_t;
    static mut r_maskMinidriver: *mut cvar_t;
    static mut r_verbose: *mut cvar_t;
    static mut r_colorbits: *mut cvar_t;
    static mut r_depthbits: *mut cvar_t;
    static mut r_stencilbits: *mut cvar_t;
    static mut r_stereo: *mut cvar_t;
    static mut r_fullscreen: *mut cvar_t;
    static mut r_mode: *mut cvar_t;
    static mut r_displayRefresh: *mut cvar_t;
    static mut r_ext_compressed_textures: *mut cvar_t;
    static mut r_ext_preferred_tc_method: *mut cvar_t;
    static mut r_allowExtensions: *mut cvar_t;
    static mut r_ext_texture_env_add: *mut cvar_t;
    static mut r_ext_texture_filter_anisotropic: *mut cvar_t;
    static mut r_ext_multitexture: *mut cvar_t;
    static mut r_ext_compiled_vertex_array: *mut cvar_t;
    static mut r_ext_point_parameters: *mut cvar_t;
    static mut r_ext_nv_point_sprite: *mut cvar_t;
    static mut r_swapInterval: *mut cvar_t;
    static mut r_logFile: *mut cvar_t;
}

#[repr(C)]
pub struct cvar_t {
    // TODO: Define cvar_t fields
}

// Whether the current hardware supports dynamic glows/flares.
extern "C" {
    pub static mut g_bDynamicGlowSupported: bool;
}

// Hack variable for deciding which kind of texture rectangle thing to do (for some
// reason it acts different on radeon! It's against the spec!).
pub static mut g_bTextureRectangleHack: bool = false;

extern "C" {
    pub static mut glConfig: glconfig_t;
    pub static mut glState: glstate_t;
    pub static mut g_wv: g_wv_t;
}

#[repr(C)]
pub struct glconfig_t {
    // TODO: Define glconfig_t fields
}

#[repr(C)]
pub struct glstate_t {
    // TODO: Define glstate_t fields
}

#[repr(C)]
pub struct g_wv_t {
    // TODO: Define g_wv_t fields
}

extern "C" {
    fn VID_Printf(level: c_int, format: *const c_char, ...);
    fn Com_Error(level: c_int, format: *const c_char, ...);
    fn Com_Printf(format: *const c_char, ...);
    fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cmd_ExecuteString(text: *const c_char);
    fn R_GetModeInfo(width: *mut c_int, height: *mut c_int, mode: c_int) -> bool;
    fn Language_IsAsian() -> bool;
    fn SE_GetString(text: *const c_char) -> *const c_char;
    fn Sys_LowPhysicalMemory() -> bool;
}

const MAX_STRING_CHARS: usize = 4096;
const MAX_PFDS: usize = 256;

const PRINT_ALL: c_int = 0;
const PRINT_WARNING: c_int = 1;

const ERR_FATAL: c_int = 3;

// Platform-specific types and constants (Windows-specific)
type HDC = *mut c_void;
type HWND = *mut c_void;
type HINSTANCE = *mut c_void;
type HGLRC = *mut c_void;

// PIXELFORMATDESCRIPTOR structure
#[repr(C)]
pub struct PIXELFORMATDESCRIPTOR {
    pub nSize: u16,
    pub nVersion: u16,
    pub dwFlags: u32,
    pub iPixelType: u8,
    pub cColorBits: u8,
    pub cRedBits: u8,
    pub cRedShift: u8,
    pub cGreenBits: u8,
    pub cGreenShift: u8,
    pub cBlueBits: u8,
    pub cBlueShift: u8,
    pub cAlphaBits: u8,
    pub cAlphaShift: u8,
    pub cAccumBits: u8,
    pub cAccumRedBits: u8,
    pub cAccumGreenBits: u8,
    pub cAccumBlueBits: u8,
    pub cAccumAlphaBits: u8,
    pub cDepthBits: u8,
    pub cStencilBits: u8,
    pub cAuxBuffers: u8,
    pub iLayerType: u8,
    pub bReserved: u8,
    pub dwLayerMask: u32,
    pub dwVisibleMask: u32,
    pub dwDamageMask: u32,
}

#[repr(C)]
pub struct DEVMODE {
    pub dmDeviceName: [c_char; 32],
    pub dmSpecVersion: u16,
    pub dmDriverVersion: u16,
    pub dmSize: u16,
    pub dmDriverExtra: u16,
    pub dmFields: u32,
    pub dmOrientation: i16,
    pub dmPaperSize: i16,
    pub dmPaperLength: i16,
    pub dmPaperWidth: i16,
    pub dmScale: i16,
    pub dmCopies: i16,
    pub dmDefaultSource: i16,
    pub dmPrintQuality: i16,
    pub dmColor: i16,
    pub dmDuplex: i16,
    pub dmYResolution: i16,
    pub dmTTOption: i16,
    pub dmCollate: i16,
    pub dmFormName: [c_char; 32],
    pub dmLogPixels: u16,
    pub dmBitsPerPel: u32,
    pub dmPelsWidth: u32,
    pub dmPelsHeight: u32,
    pub dmDisplayFlags: u32,
    pub dmDisplayFrequency: u32,
    pub dmICMMethod: u32,
    pub dmICMIntent: u32,
    pub dmMediaType: u32,
    pub dmDitherType: u32,
    pub dmReserved1: u32,
    pub dmReserved2: u32,
}

#[repr(C)]
pub struct OSVERSIONINFO {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
    pub szCSDVersion: [c_char; 128],
}

#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct WNDCLASS {
    pub style: u32,
    pub lpfnWndProc: *mut c_void,
    pub cbClsExtra: c_int,
    pub cbWndExtra: c_int,
    pub hInstance: HINSTANCE,
    pub hIcon: *mut c_void,
    pub hCursor: *mut c_void,
    pub hbrBackground: *mut c_void,
    pub lpszMenuName: *const c_char,
    pub lpszClassName: *const c_char,
}

// Windows API functions
extern "C" {
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strlwr(s: *mut c_char) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn va(format: *const c_char, ...) -> *const c_char;

    // Windows-specific API functions
    fn GetDC(hwnd: HWND) -> HDC;
    fn ReleaseDC(hwnd: HWND, hdc: HDC) -> c_int;
    fn GetDesktopWindow() -> HWND;
    fn GetDeviceCaps(hdc: HDC, index: c_int) -> c_int;
    fn DescribePixelFormat(hdc: HDC, format: c_int, size: u32, pfd: *mut PIXELFORMATDESCRIPTOR) -> c_int;
    fn SetPixelFormat(hdc: HDC, format: c_int, pfd: *const PIXELFORMATDESCRIPTOR) -> c_int;
    fn RegisterClass(wndclass: *const WNDCLASS) -> c_int;
    fn CreateWindowEx(exStyle: u32, className: *const c_char, windowName: *const c_char, style: u32, x: c_int, y: c_int, width: c_int, height: c_int, parent: HWND, menu: *mut c_void, instance: HINSTANCE, param: *mut c_void) -> HWND;
    fn ShowWindow(hwnd: HWND, cmd: c_int) -> c_int;
    fn UpdateWindow(hwnd: HWND) -> c_int;
    fn DestroyWindow(hwnd: HWND) -> c_int;
    fn SetForegroundWindow(hwnd: HWND) -> HWND;
    fn SetFocus(hwnd: HWND) -> HWND;
    fn AdjustWindowRect(rect: *mut RECT, style: u32, menu: c_int) -> c_int;
    fn MessageBox(hwnd: HWND, text: *const c_char, caption: *const c_char, kind: u32) -> c_int;
    fn GetVersionEx(vinfo: *mut OSVERSIONINFO) -> c_int;
    fn ChangeDisplaySettings(devmode: *const DEVMODE, flags: u32) -> c_int;
    fn EnumDisplaySettings(device: *const c_char, mode: u32, devmode: *mut DEVMODE) -> c_int;
    fn LoadIcon(instance: HINSTANCE, name: *const c_char) -> *mut c_void;
    fn LoadCursor(instance: HINSTANCE, name: *const c_char) -> *mut c_void;
    fn fclose(stream: *mut c_void) -> c_int;
    fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    fn SwapBuffers(hdc: HDC) -> c_int;
}

// GL function pointers and initialization
extern "C" {
    fn qglGetString(name: u32) -> *const u8;
    fn qglGetIntegerv(pname: u32, params: *mut c_int);
    fn qglGetFloatv(pname: u32, params: *mut f32);
    fn qglCreateContext(hdc: HDC) -> HGLRC;
    fn qglMakeCurrent(hdc: HDC, hrc: HGLRC) -> c_int;
    fn qglDeleteContext(hrc: HGLRC) -> c_int;
    fn qglGetProcAddress(name: *const c_char) -> *mut c_void;
    fn qwglCreateContext(hdc: HDC) -> HGLRC;
    fn qwglMakeCurrent(hdc: HDC, hrc: HGLRC) -> c_int;
    fn qwglDeleteContext(hrc: HGLRC) -> c_int;
    fn qwglGetProcAddress(name: *const c_char) -> *mut c_void;
    pub static mut qwglSwapIntervalEXT: *mut c_void;
    pub static mut qglMultiTexCoord2fARB: *mut c_void;
    pub static mut qglActiveTextureARB: *mut c_void;
    pub static mut qglClientActiveTextureARB: *mut c_void;
    pub static mut qglLockArraysEXT: *mut c_void;
    pub static mut qglUnlockArraysEXT: *mut c_void;
    pub static mut qglPointParameterfEXT: *mut c_void;
    pub static mut qglPointParameterfvEXT: *mut c_void;
    pub static mut qglPointParameteriNV: *mut c_void;
    pub static mut qglPointParameterivNV: *mut c_void;
    pub static mut qglCombinerParameterfvNV: *mut c_void;
    pub static mut qglCombinerParameterivNV: *mut c_void;
    pub static mut qglCombinerParameterfNV: *mut c_void;
    pub static mut qglCombinerParameteriNV: *mut c_void;
    pub static mut qglCombinerInputNV: *mut c_void;
    pub static mut qglCombinerOutputNV: *mut c_void;
    pub static mut qglFinalCombinerInputNV: *mut c_void;
    pub static mut qglGetCombinerInputParameterfvNV: *mut c_void;
    pub static mut qglGetCombinerInputParameterivNV: *mut c_void;
    pub static mut qglGetCombinerOutputParameterfvNV: *mut c_void;
    pub static mut qglGetCombinerOutputParameterivNV: *mut c_void;
    pub static mut qglGetFinalCombinerInputParameterfvNV: *mut c_void;
    pub static mut qglGetFinalCombinerInputParameterivNV: *mut c_void;
    pub static mut qglProgramStringARB: *mut c_void;
    pub static mut qglBindProgramARB: *mut c_void;
    pub static mut qglDeleteProgramsARB: *mut c_void;
    pub static mut qglGenProgramsARB: *mut c_void;
    pub static mut qglProgramEnvParameter4dARB: *mut c_void;
    pub static mut qglProgramEnvParameter4dvARB: *mut c_void;
    pub static mut qglProgramEnvParameter4fARB: *mut c_void;
    pub static mut qglProgramEnvParameter4fvARB: *mut c_void;
    pub static mut qglProgramLocalParameter4dARB: *mut c_void;
    pub static mut qglProgramLocalParameter4dvARB: *mut c_void;
    pub static mut qglProgramLocalParameter4fARB: *mut c_void;
    pub static mut qglProgramLocalParameter4fvARB: *mut c_void;
    pub static mut qglGetProgramEnvParameterdvARB: *mut c_void;
    pub static mut qglGetProgramEnvParameterfvARB: *mut c_void;
    pub static mut qglGetProgramLocalParameterdvARB: *mut c_void;
    pub static mut qglGetProgramLocalParameterfvARB: *mut c_void;
    pub static mut qglGetProgramivARB: *mut c_void;
    pub static mut qglGetProgramStringARB: *mut c_void;
    pub static mut qglIsProgramARB: *mut c_void;
    pub static mut qwglGetPixelFormatAttribivARB: *mut c_void;
    pub static mut qwglGetPixelFormatAttribfvARB: *mut c_void;
    pub static mut qwglChoosePixelFormatARB: *mut c_void;
    pub static mut qwglCreatePbufferARB: *mut c_void;
    pub static mut qwglGetPbufferDCARB: *mut c_void;
    pub static mut qwglReleasePbufferDCARB: *mut c_void;
    pub static mut qwglDestroyPbufferARB: *mut c_void;
    pub static mut qwglQueryPbufferARB: *mut c_void;
    pub static mut qwglBindTexImageARB: *mut c_void;
    pub static mut qwglReleaseTexImageARB: *mut c_void;
    pub static mut qwglSetPbufferAttribARB: *mut c_void;
}

// Windows constants
const WS_OVERLAPPED: u32 = 0x00000000;
const WS_BORDER: u32 = 0x00800000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_POPUP: u32 = 0x80000000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const SW_SHOW: c_int = 5;
const SW_HIDE: c_int = 0;
const DM_PELSWIDTH: u32 = 0x00080000;
const DM_PELSHEIGHT: u32 = 0x00100000;
const DM_DISPLAYFREQUENCY: u32 = 0x00400000;
const DM_BITSPERPEL: u32 = 0x00040000;
const CDS_FULLSCREEN: u32 = 4;
const DISP_CHANGE_SUCCESSFUL: c_int = 0;
const DISP_CHANGE_RESTART: c_int = 1;
const DISP_CHANGE_BADPARAM: c_int = -2;
const DISP_CHANGE_BADFLAGS: c_int = -3;
const DISP_CHANGE_FAILED: c_int = -4;
const DISP_CHANGE_BADMODE: c_int = -5;
const DISP_CHANGE_NOTUPDATED: c_int = -6;
const PFD_DRAW_TO_WINDOW: u32 = 0x00000004;
const PFD_SUPPORT_OPENGL: u32 = 0x00000020;
const PFD_DOUBLEBUFFER: u32 = 0x00000001;
const PFD_TYPE_RGBA: u8 = 0;
const PFD_MAIN_PLANE: u8 = 0;
const PFD_STEREO: u32 = 0x00000001;
const PFD_GENERIC_FORMAT: u32 = 0x00000040;
const PFD_GENERIC_ACCELERATED: u32 = 0x00001000;
const PFD_TYPE_RGBA: u8 = 0;
const IDI_ICON1: u16 = 1;
const IDC_ARROW: u32 = 32512;
const VER_PLATFORM_WIN32_NT: u32 = 2;
const VER_PLATFORM_WIN32_WINDOWS: u32 = 1;
const MB_OKCANCEL: u32 = 1;
const MB_ICONEXCLAMATION: u32 = 0x00000030;
const IDOK: c_int = 1;
const ENUM_CURRENT_SETTINGS: u32 = 4294967295u32;
const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;
const GL_EXTENSIONS: u32 = 0x1F03;
const GL_MAX_TEXTURE_SIZE: u32 = 0x0D33;
const GL_MAX_ACTIVE_TEXTURES_ARB: u32 = 0x84E8;
const CVAR_ARCHIVE: c_int = 1;
const CVAR_LATCH: c_int = 4;

const WINDOW_STYLE: u32 = WS_OVERLAPPED | WS_BORDER | WS_CAPTION | WS_VISIBLE;

// TC (Texture Compression) constants
const TC_NONE: c_int = 0;
const TC_S3TC: c_int = 1;
const TC_S3TC_DXT: c_int = 2;

static mut GLW_ChoosePFD_impl: fn(HDC, *mut PIXELFORMATDESCRIPTOR) -> c_int;
static mut GLW_SetMode_impl: fn(c_int, c_int, bool) -> rserr_t;
static mut GLW_InitExtensions_impl: fn();

fn GLW_ChoosePFD(hDC: HDC, pPFD: *mut PIXELFORMATDESCRIPTOR) -> c_int {
    unsafe {
        let mut pfds: [PIXELFORMATDESCRIPTOR; MAX_PFDS + 1];
        let mut maxPFD: c_int = 0;
        let mut i: c_int;
        let mut bestMatch: c_int = 0;

        VID_Printf(PRINT_ALL, "...GLW_ChoosePFD( %d, %d, %d )\n" as *const c_char, (*pPFD).cColorBits as c_int, (*pPFD).cDepthBits as c_int, (*pPFD).cStencilBits as c_int);

        // count number of PFDs
        maxPFD = DescribePixelFormat(hDC, 1, core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32, &mut pfds[0]);
        if maxPFD > MAX_PFDS as c_int {
            VID_Printf(PRINT_WARNING, "...numPFDs > MAX_PFDS (%d > %d)\n" as *const c_char, maxPFD, MAX_PFDS as c_int);
            maxPFD = MAX_PFDS as c_int;
        }

        VID_Printf(PRINT_ALL, "...%d PFDs found\n" as *const c_char, maxPFD - 1);

        // grab information
        i = 1;
        while i <= maxPFD {
            DescribePixelFormat(hDC, i, core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32, &mut pfds[i as usize]);
            i += 1;
        }

        // look for a best match
        i = 1;
        while i <= maxPFD {
            // make sure this has hardware acceleration
            if (pfds[i as usize].dwFlags & PFD_GENERIC_FORMAT) != 0 {
                if (*r_allowSoftwareGL).is_null() || (*(*r_allowSoftwareGL)).integer == 0 {
                    if !(*r_verbose).is_null() && (*(*r_verbose)).integer != 0 {
                        VID_Printf(PRINT_ALL, "...PFD %d rejected, software acceleration\n" as *const c_char, i);
                    }
                    i += 1;
                    continue;
                }
            }

            // verify pixel type
            if pfds[i as usize].iPixelType != PFD_TYPE_RGBA {
                if !(*r_verbose).is_null() && (*(*r_verbose)).integer != 0 {
                    VID_Printf(PRINT_ALL, "...PFD %d rejected, not RGBA\n" as *const c_char, i);
                }
                i += 1;
                continue;
            }

            // verify proper flags
            if (((pfds[i as usize].dwFlags & (*pPFD).dwFlags) & (*pPFD).dwFlags) != (*pPFD).dwFlags) {
                if !(*r_verbose).is_null() && (*(*r_verbose)).integer != 0 {
                    VID_Printf(PRINT_ALL, "...PFD %d rejected, improper flags (%x instead of %x)\n" as *const c_char, i, pfds[i as usize].dwFlags, (*pPFD).dwFlags);
                }
                i += 1;
                continue;
            }

            // verify enough bits
            if pfds[i as usize].cDepthBits < 15 {
                i += 1;
                continue;
            }
            if (pfds[i as usize].cStencilBits < 4) && ((*pPFD).cStencilBits > 0) {
                i += 1;
                continue;
            }

            // selection criteria (in order of priority):
            //
            // PFD_STEREO
            // colorBits
            // depthBits
            // stencilBits
            if bestMatch != 0 {
                // check stereo
                if (pfds[i as usize].dwFlags & PFD_STEREO) != 0 && (pfds[bestMatch as usize].dwFlags & PFD_STEREO) == 0 && ((*pPFD).dwFlags & PFD_STEREO) != 0 {
                    bestMatch = i;
                    i += 1;
                    continue;
                }

                if (pfds[i as usize].dwFlags & PFD_STEREO) == 0 && (pfds[bestMatch as usize].dwFlags & PFD_STEREO) != 0 && ((*pPFD).dwFlags & PFD_STEREO) != 0 {
                    bestMatch = i;
                    i += 1;
                    continue;
                }

                // check color
                if pfds[bestMatch as usize].cColorBits != (*pPFD).cColorBits {
                    // prefer perfect match
                    if pfds[i as usize].cColorBits == (*pPFD).cColorBits {
                        bestMatch = i;
                        i += 1;
                        continue;
                    }
                    // otherwise if this PFD has more bits than our best, use it
                    else if pfds[i as usize].cColorBits > pfds[bestMatch as usize].cColorBits {
                        bestMatch = i;
                        i += 1;
                        continue;
                    }
                }

                // check depth
                if pfds[bestMatch as usize].cDepthBits != (*pPFD).cDepthBits {
                    // prefer perfect match
                    if pfds[i as usize].cDepthBits == (*pPFD).cDepthBits {
                        bestMatch = i;
                        i += 1;
                        continue;
                    }
                    // otherwise if this PFD has more bits than our best, use it
                    else if pfds[i as usize].cDepthBits > pfds[bestMatch as usize].cDepthBits {
                        bestMatch = i;
                        i += 1;
                        continue;
                    }
                }

                // check stencil
                if pfds[bestMatch as usize].cStencilBits != (*pPFD).cStencilBits {
                    // prefer perfect match
                    if pfds[i as usize].cStencilBits == (*pPFD).cStencilBits {
                        bestMatch = i;
                        i += 1;
                        continue;
                    }
                    // otherwise if this PFD has more bits than our best, use it
                    else if (pfds[i as usize].cStencilBits > pfds[bestMatch as usize].cStencilBits) && ((*pPFD).cStencilBits > 0) {
                        bestMatch = i;
                        i += 1;
                        continue;
                    }
                }
            } else {
                bestMatch = i;
            }
            i += 1;
        }

        if bestMatch == 0 {
            return 0;
        }

        if (pfds[bestMatch as usize].dwFlags & PFD_GENERIC_FORMAT) != 0 {
            if (*r_allowSoftwareGL).is_null() || (*(*r_allowSoftwareGL)).integer == 0 {
                VID_Printf(PRINT_ALL, "...no hardware acceleration found\n" as *const c_char);
                return 0;
            } else {
                VID_Printf(PRINT_ALL, "...using software emulation\n" as *const c_char);
            }
        } else if (pfds[bestMatch as usize].dwFlags & PFD_GENERIC_ACCELERATED) != 0 {
            VID_Printf(PRINT_ALL, "...MCD acceleration found\n" as *const c_char);
        } else {
            VID_Printf(PRINT_ALL, "...hardware acceleration found\n" as *const c_char);
        }

        *pPFD = pfds[bestMatch as usize];

        return bestMatch;
    }
}

// void GLW_CreatePFD
//
// Helper function zeros out then fills in a PFD
fn GLW_CreatePFD(pPFD: *mut PIXELFORMATDESCRIPTOR, colorbits: c_int, depthbits: c_int, stencilbits: c_int, stereo: bool) {
    unsafe {
        let mut src = PIXELFORMATDESCRIPTOR {
            nSize: core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA,
            cColorBits: 24,
            cRedBits: 0,
            cRedShift: 0,
            cGreenBits: 0,
            cGreenShift: 0,
            cBlueBits: 0,
            cBlueShift: 0,
            cAlphaBits: 0,
            cAlphaShift: 0,
            cAccumBits: 0,
            cAccumRedBits: 0,
            cAccumGreenBits: 0,
            cAccumBlueBits: 0,
            cAccumAlphaBits: 0,
            cDepthBits: 24,
            cStencilBits: 8,
            cAuxBuffers: 0,
            iLayerType: PFD_MAIN_PLANE,
            bReserved: 0,
            dwLayerMask: 0,
            dwVisibleMask: 0,
            dwDamageMask: 0,
        };

        src.cColorBits = colorbits as u8;
        src.cDepthBits = depthbits as u8;
        src.cStencilBits = stencilbits as u8;

        if stereo {
            VID_Printf(PRINT_ALL, "...attempting to use stereo\n" as *const c_char);
            src.dwFlags |= PFD_STEREO;
            glConfig.stereoEnabled = true;
        } else {
            glConfig.stereoEnabled = false;
        }

        *pPFD = src;
    }
}

// GLW_MakeContext
fn GLW_MakeContext(pPFD: *mut PIXELFORMATDESCRIPTOR) -> c_int {
    unsafe {
        let mut pixelformat: c_int;

        // don't putz around with pixelformat if it's already set (e.g. this is a soft
        // reset of the graphics system)
        if !glw_state.pixelFormatSet {
            // choose, set, and describe our desired pixel format.  If we're
            // using a minidriver then we need to bypass the GDI functions,
            // otherwise use the GDI functions.
            if (pixelformat = GLW_ChoosePFD(glw_state.hDC, pPFD)) == 0 {
                VID_Printf(PRINT_ALL, "...GLW_ChoosePFD failed\n" as *const c_char);
                return TRY_PFD_FAIL_SOFT;
            }
            VID_Printf(PRINT_ALL, "...PIXELFORMAT %d selected\n" as *const c_char, pixelformat);

            DescribePixelFormat(glw_state.hDC, pixelformat, core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32, pPFD);

            if SetPixelFormat(glw_state.hDC, pixelformat, pPFD) == 0 {
                VID_Printf(PRINT_ALL, "...SetPixelFormat failed\n" as *const c_char, glw_state.hDC);
                return TRY_PFD_FAIL_SOFT;
            }

            glw_state.pixelFormatSet = true;
        }

        // startup the OpenGL subsystem by creating a context and making it current
        if glw_state.hGLRC.is_null() {
            VID_Printf(PRINT_ALL, "...creating GL context: " as *const c_char);
            if (glw_state.hGLRC = qwglCreateContext(glw_state.hDC)).is_null() {
                VID_Printf(PRINT_ALL, "failed\n" as *const c_char);
                return TRY_PFD_FAIL_HARD;
            }
            VID_Printf(PRINT_ALL, "succeeded\n" as *const c_char);

            VID_Printf(PRINT_ALL, "...making context current: " as *const c_char);
            if qwglMakeCurrent(glw_state.hDC, glw_state.hGLRC) == 0 {
                qwglDeleteContext(glw_state.hGLRC);
                glw_state.hGLRC = core::ptr::null_mut();
                VID_Printf(PRINT_ALL, "failed\n" as *const c_char);
                return TRY_PFD_FAIL_HARD;
            }
            VID_Printf(PRINT_ALL, "succeeded\n" as *const c_char);
        }

        return TRY_PFD_SUCCESS;
    }
}

// GLW_InitDriver
//
// - get a DC if one doesn't exist
// - create an HGLRC if one doesn't exist
fn GLW_InitDriver(colorbits: c_int) -> bool {
    unsafe {
        let mut tpfd: c_int;
        let mut depthbits: c_int;
        let mut stencilbits: c_int;
        static mut pfd: PIXELFORMATDESCRIPTOR = PIXELFORMATDESCRIPTOR {
            nSize: 0,
            nVersion: 0,
            dwFlags: 0,
            iPixelType: 0,
            cColorBits: 0,
            cRedBits: 0,
            cRedShift: 0,
            cGreenBits: 0,
            cGreenShift: 0,
            cBlueBits: 0,
            cBlueShift: 0,
            cAlphaBits: 0,
            cAlphaShift: 0,
            cAccumBits: 0,
            cAccumRedBits: 0,
            cAccumGreenBits: 0,
            cAccumBlueBits: 0,
            cAccumAlphaBits: 0,
            cDepthBits: 0,
            cStencilBits: 0,
            cAuxBuffers: 0,
            iLayerType: 0,
            bReserved: 0,
            dwLayerMask: 0,
            dwVisibleMask: 0,
            dwDamageMask: 0,
        };

        VID_Printf(PRINT_ALL, "Initializing OpenGL driver\n" as *const c_char);

        // get a DC for our window if we don't already have one allocated
        if glw_state.hDC.is_null() {
            VID_Printf(PRINT_ALL, "...getting DC: " as *const c_char);

            if (glw_state.hDC = GetDC(g_wv.hWnd)).is_null() {
                VID_Printf(PRINT_ALL, "failed\n" as *const c_char);
                return false;
            }
            VID_Printf(PRINT_ALL, "succeeded\n" as *const c_char);
        }

        let mut colorbits = colorbits;
        if colorbits == 0 {
            colorbits = glw_state.desktopBitsPixel;
        }

        // implicitly assume Z-buffer depth == desktop color depth
        if (*r_depthbits).is_null() || (*(*r_depthbits)).integer == 0 {
            if colorbits > 16 {
                depthbits = 24;
            } else {
                depthbits = 16;
            }
        } else {
            depthbits = (*(*r_depthbits)).integer;
        }

        // do not allow stencil if Z-buffer depth likely won't contain it
        stencilbits = (*(*r_stencilbits)).integer;
        if depthbits < 24 {
            stencilbits = 0;
        }

        // make two attempts to set the PIXELFORMAT

        // first attempt: r_colorbits, depthbits, and r_stencilbits
        if !glw_state.pixelFormatSet {
            GLW_CreatePFD(&mut pfd, colorbits, depthbits, stencilbits, (*(*r_stereo)).integer != 0);
            if (tpfd = GLW_MakeContext(&mut pfd)) != TRY_PFD_SUCCESS {
                if tpfd == TRY_PFD_FAIL_HARD {
                    VID_Printf(PRINT_WARNING, "...failed hard\n" as *const c_char);
                    return false;
                }

                // punt if we've already tried the desktop bit depth and no stencil bits
                if ((*(*r_colorbits)).integer == glw_state.desktopBitsPixel) && (stencilbits == 0) {
                    ReleaseDC(g_wv.hWnd, glw_state.hDC);
                    glw_state.hDC = core::ptr::null_mut();

                    VID_Printf(PRINT_ALL, "...failed to find an appropriate PIXELFORMAT\n" as *const c_char);

                    return false;
                }

                // second attempt: desktop's color bits and no stencil
                let mut colorbits2 = colorbits;
                if colorbits2 > glw_state.desktopBitsPixel {
                    colorbits2 = glw_state.desktopBitsPixel;
                }
                GLW_CreatePFD(&mut pfd, colorbits2, depthbits, 0, (*(*r_stereo)).integer != 0);
                if GLW_MakeContext(&mut pfd) != TRY_PFD_SUCCESS {
                    if !glw_state.hDC.is_null() {
                        ReleaseDC(g_wv.hWnd, glw_state.hDC);
                        glw_state.hDC = core::ptr::null_mut();
                    }

                    VID_Printf(PRINT_ALL, "...failed to find an appropriate PIXELFORMAT\n" as *const c_char);

                    return false;
                }
            }

            // report if stereo is desired but unavailable
            if (pfd.dwFlags & PFD_STEREO) == 0 && (*(*r_stereo)).integer != 0 {
                VID_Printf(PRINT_ALL, "...failed to select stereo pixel format\n" as *const c_char);
                glConfig.stereoEnabled = false;
            }
        }

        // store PFD specifics
        glConfig.colorBits = pfd.cColorBits as c_int;
        glConfig.depthBits = pfd.cDepthBits as c_int;
        glConfig.stencilBits = pfd.cStencilBits as c_int;

        return true;
    }
}

// GLW_CreateWindow
//
// Responsible for creating the Win32 window and initializing the OpenGL driver.
fn GLW_CreateWindow(width: c_int, height: c_int, colorbits: c_int, cdsFullscreen: bool) -> bool {
    unsafe {
        let mut r: RECT;
        let mut vid_xpos: *mut cvar_t;
        let mut vid_ypos: *mut cvar_t;
        let mut stylebits: u32;
        let mut x: c_int;
        let mut y: c_int;
        let mut w: c_int;
        let mut h: c_int;
        let mut exstyle: u32;

        // register the window class if necessary
        if !s_classRegistered {
            let mut wc: WNDCLASS = core::mem::zeroed();

            wc.style = 0;
            wc.lpfnWndProc = glw_state.wndproc;
            wc.cbClsExtra = 0;
            wc.cbWndExtra = 0;
            wc.hInstance = g_wv.hInstance;
            wc.hIcon = LoadIcon(g_wv.hInstance, MAKEINTRESOURCE(IDI_ICON1));
            wc.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW as *const c_char);
            wc.hbrBackground = core::ptr::null_mut();
            wc.lpszMenuName = core::ptr::null();
            wc.lpszClassName = WINDOW_CLASS_NAME.as_ptr() as *const c_char;

            if RegisterClass(&wc) == 0 {
                Com_Error(ERR_FATAL, "GLW_CreateWindow: could not register window class" as *const c_char);
            }
            s_classRegistered = true;
            VID_Printf(PRINT_ALL, "...registered window class\n" as *const c_char);
        }

        // create the HWND if one does not already exist
        if g_wv.hWnd.is_null() {
            // compute width and height
            r.left = 0;
            r.top = 0;
            r.right = width;
            r.bottom = height;

            if cdsFullscreen {
                exstyle = WS_EX_TOPMOST;
                stylebits = WS_SYSMENU | WS_POPUP | WS_VISIBLE;
            } else {
                exstyle = 0;
                stylebits = WS_SYSMENU | WINDOW_STYLE | WS_MINIMIZEBOX;
                AdjustWindowRect(&mut r, stylebits, 0);
            }

            w = r.right - r.left;
            h = r.bottom - r.top;

            if cdsFullscreen {
                x = 0;
                y = 0;
            } else {
                vid_xpos = Cvar_Get("vid_xpos" as *const c_char, "" as *const c_char, 0);
                vid_ypos = Cvar_Get("vid_ypos" as *const c_char, "" as *const c_char, 0);
                x = (*vid_xpos).integer;
                y = (*vid_ypos).integer;

                // adjust window coordinates if necessary
                // so that the window is completely on screen
                if x < 0 {
                    x = 0;
                }
                if y < 0 {
                    y = 0;
                }

                if w < glw_state.desktopWidth && h < glw_state.desktopHeight {
                    if x + w > glw_state.desktopWidth {
                        x = glw_state.desktopWidth - w;
                    }
                    if y + h > glw_state.desktopHeight {
                        y = glw_state.desktopHeight - h;
                    }
                }
            }

            g_wv.hWnd = CreateWindowEx(
                exstyle,
                WINDOW_CLASS_NAME.as_ptr() as *const c_char,
                WINDOW_CLASS_NAME.as_ptr() as *const c_char,
                stylebits,
                x,
                y,
                w,
                h,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                g_wv.hInstance,
                core::ptr::null_mut(),
            );

            if g_wv.hWnd.is_null() {
                Com_Error(ERR_FATAL, "GLW_CreateWindow() - Couldn't create window" as *const c_char);
            }

            ShowWindow(g_wv.hWnd, SW_SHOW);
            UpdateWindow(g_wv.hWnd);
            VID_Printf(PRINT_ALL, "...created window@%d,%d (%dx%d)\n" as *const c_char, x, y, w, h);
        } else {
            VID_Printf(PRINT_ALL, "...window already present, CreateWindowEx skipped\n" as *const c_char);
        }

        if !GLW_InitDriver(colorbits) {
            ShowWindow(g_wv.hWnd, SW_HIDE);
            DestroyWindow(g_wv.hWnd);
            g_wv.hWnd = core::ptr::null_mut();

            return false;
        }

        SetForegroundWindow(g_wv.hWnd);
        SetFocus(g_wv.hWnd);

        return true;
    }
}

fn PrintCDSError(value: c_int) {
    match value {
        DISP_CHANGE_RESTART => {
            VID_Printf(PRINT_ALL, "restart required\n" as *const c_char);
        }
        DISP_CHANGE_BADPARAM => {
            VID_Printf(PRINT_ALL, "bad param\n" as *const c_char);
        }
        DISP_CHANGE_BADFLAGS => {
            VID_Printf(PRINT_ALL, "bad flags\n" as *const c_char);
        }
        DISP_CHANGE_FAILED => {
            VID_Printf(PRINT_ALL, "DISP_CHANGE_FAILED\n" as *const c_char);
        }
        DISP_CHANGE_BADMODE => {
            VID_Printf(PRINT_ALL, "bad mode\n" as *const c_char);
        }
        DISP_CHANGE_NOTUPDATED => {
            VID_Printf(PRINT_ALL, "not updated\n" as *const c_char);
        }
        _ => {
            VID_Printf(PRINT_ALL, "unknown error %d\n" as *const c_char, value);
        }
    }
}

// GLW_SetMode
fn GLW_SetMode(mode: c_int, colorbits: c_int, cdsFullscreen: bool) -> rserr_t {
    unsafe {
        let mut hDC: HDC;
        let win_fs: [&[u8]; 2] = [b"W\0", b"FS\0"];
        let mut cdsRet: c_int;
        let mut dm: DEVMODE = core::mem::zeroed();

        // print out informational messages
        VID_Printf(PRINT_ALL, "...setting mode %d:" as *const c_char, mode);
        if !R_GetModeInfo(&mut glConfig.vidWidth, &mut glConfig.vidHeight, mode) {
            VID_Printf(PRINT_ALL, " invalid mode\n" as *const c_char);
            return rserr_t::RSERR_INVALID_MODE;
        }
        VID_Printf(PRINT_ALL, " %d %d %s\n" as *const c_char, glConfig.vidWidth, glConfig.vidHeight, win_fs[if cdsFullscreen { 1 } else { 0 }].as_ptr() as *const c_char);

        // check our desktop attributes
        hDC = GetDC(GetDesktopWindow());
        glw_state.desktopBitsPixel = GetDeviceCaps(hDC, 12); // BITSPIXEL
        glw_state.desktopWidth = GetDeviceCaps(hDC, 8); // HORZRES
        glw_state.desktopHeight = GetDeviceCaps(hDC, 10); // VERTRES
        ReleaseDC(GetDesktopWindow(), hDC);

        // verify desktop bit depth
        if glw_state.desktopBitsPixel < 15 || glw_state.desktopBitsPixel == 24 {
            if !cdsFullscreen && (colorbits == 0 || colorbits >= 15) {
                let mut sErrorHead: [c_char; 1024] = [0; 1024];

                let error_head = if Language_IsAsian() {
                    "Low Desktop Color Depth" as *const c_char
                } else {
                    SE_GetString("CON_TEXT_LOW_DESKTOP_COLOUR_DEPTH" as *const c_char)
                };
                Q_strncpyz(sErrorHead.as_mut_ptr(), error_head, 1024);

                let psErrorBody = if Language_IsAsian() {
                    "It is highly unlikely that a correct windowed\n\
                     display can be initialized with the current\n\
                     desktop display depth.  Select 'OK' to try\n\
                     anyway.  Select 'Cancel' to try a fullscreen\n\
                     mode instead." as *const c_char
                } else {
                    SE_GetString("CON_TEXT_TRY_ANYWAY" as *const c_char)
                };

                if MessageBox(
                    core::ptr::null_mut(),
                    psErrorBody,
                    sErrorHead.as_ptr(),
                    MB_OKCANCEL | MB_ICONEXCLAMATION,
                ) != IDOK
                {
                    return rserr_t::RSERR_INVALID_MODE;
                }
            }
        }

        // do a CDS if needed
        if cdsFullscreen {
            dm.dmSize = core::mem::size_of::<DEVMODE>() as u16;

            dm.dmPelsWidth = glConfig.vidWidth as u32;
            dm.dmPelsHeight = glConfig.vidHeight as u32;
            dm.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT;

            if !(*r_displayRefresh).is_null() && (*(*r_displayRefresh)).integer != 0 {
                dm.dmDisplayFrequency = (*(*r_displayRefresh)).integer as u32;
                dm.dmFields |= DM_DISPLAYFREQUENCY;
            }

            // try to change color depth if possible
            if colorbits != 0 {
                if glw_state.allowdisplaydepthchange {
                    dm.dmBitsPerPel = colorbits as u32;
                    dm.dmFields |= DM_BITSPERPEL;
                    VID_Printf(PRINT_ALL, "...using colorsbits of %d\n" as *const c_char, colorbits);
                } else {
                    VID_Printf(PRINT_ALL, "WARNING:...changing depth not supported on Win95 < pre-OSR 2.x\n" as *const c_char);
                }
            } else {
                VID_Printf(PRINT_ALL, "...using desktop display depth of %d\n" as *const c_char, glw_state.desktopBitsPixel);
            }

            // if we're already in fullscreen then just create the window
            if glw_state.cdsFullscreen {
                VID_Printf(PRINT_ALL, "...already fullscreen, avoiding redundant CDS\n" as *const c_char);

                if !GLW_CreateWindow(glConfig.vidWidth, glConfig.vidHeight, colorbits, true) {
                    VID_Printf(PRINT_ALL, "...restoring display settings\n" as *const c_char);
                    ChangeDisplaySettings(core::ptr::null(), 0);
                    return rserr_t::RSERR_INVALID_MODE;
                }
            }
            // need to call CDS
            else {
                VID_Printf(PRINT_ALL, "...calling CDS: " as *const c_char);

                // try setting the exact mode requested, because some drivers don't report
                // the low res modes in EnumDisplaySettings, but still work
                if (cdsRet = ChangeDisplaySettings(&dm, CDS_FULLSCREEN)) == DISP_CHANGE_SUCCESSFUL {
                    VID_Printf(PRINT_ALL, "ok\n" as *const c_char);

                    if !GLW_CreateWindow(glConfig.vidWidth, glConfig.vidHeight, colorbits, true) {
                        VID_Printf(PRINT_ALL, "...restoring display settings\n" as *const c_char);
                        ChangeDisplaySettings(core::ptr::null(), 0);
                        return rserr_t::RSERR_INVALID_MODE;
                    }

                    glw_state.cdsFullscreen = true;
                } else {
                    // the exact mode failed, so scan EnumDisplaySettings for the next largest mode
                    let mut devmode: DEVMODE = core::mem::zeroed();
                    let mut modeNum: c_int;

                    VID_Printf(PRINT_ALL, "failed, " as *const c_char);

                    PrintCDSError(cdsRet);

                    VID_Printf(PRINT_ALL, "...trying next higher resolution:" as *const c_char);

                    // we could do a better matching job here...
                    modeNum = 0;
                    loop {
                        if EnumDisplaySettings(core::ptr::null(), modeNum as u32, &mut devmode) == 0 {
                            modeNum = -1;
                            break;
                        }
                        if devmode.dmPelsWidth >= glConfig.vidWidth as u32
                            && devmode.dmPelsHeight >= glConfig.vidHeight as u32
                            && devmode.dmBitsPerPel >= 15
                        {
                            break;
                        }
                        modeNum += 1;
                    }

                    if modeNum != -1 && (cdsRet = ChangeDisplaySettings(&devmode, CDS_FULLSCREEN)) == DISP_CHANGE_SUCCESSFUL {
                        VID_Printf(PRINT_ALL, " ok\n" as *const c_char);
                        if !GLW_CreateWindow(glConfig.vidWidth, glConfig.vidHeight, colorbits, true) {
                            VID_Printf(PRINT_ALL, "...restoring display settings\n" as *const c_char);
                            ChangeDisplaySettings(core::ptr::null(), 0);
                            return rserr_t::RSERR_INVALID_MODE;
                        }

                        glw_state.cdsFullscreen = true;
                    } else {
                        VID_Printf(PRINT_ALL, " failed, " as *const c_char);

                        PrintCDSError(cdsRet);

                        VID_Printf(PRINT_ALL, "...restoring display settings\n" as *const c_char);
                        ChangeDisplaySettings(core::ptr::null(), 0);

                        // jfm:  i took out the following code to allow fallback to mode 3, with this code it goes half windowed and just doesn't work.
                        // glw_state.cdsFullscreen = false;
                        // glConfig.isFullscreen = false;
                        // if (!GLW_CreateWindow(glConfig.vidWidth, glConfig.vidHeight, colorbits, false)) {
                        //     return rserr_t::RSERR_INVALID_MODE;
                        // }

                        return rserr_t::RSERR_INVALID_FULLSCREEN;
                    }
                }
            }
        } else {
            if glw_state.cdsFullscreen {
                ChangeDisplaySettings(core::ptr::null(), 0);
            }

            glw_state.cdsFullscreen = false;
            if !GLW_CreateWindow(glConfig.vidWidth, glConfig.vidHeight, colorbits, false) {
                return rserr_t::RSERR_INVALID_MODE;
            }
        }

        // success, now check display frequency, although this won't be valid on Voodoo(2)
        dm = core::mem::zeroed();
        dm.dmSize = core::mem::size_of::<DEVMODE>() as u16;
        if EnumDisplaySettings(core::ptr::null(), ENUM_CURRENT_SETTINGS, &mut dm) != 0 {
            glConfig.displayFrequency = dm.dmDisplayFrequency as c_int;
        }

        // NOTE: this is overridden later on standalone 3Dfx drivers
        glConfig.isFullscreen = cdsFullscreen;

        return rserr_t::RSERR_OK;
    }
}

// GLW_InitTextureCompression
fn GLW_InitTextureCompression() {
    unsafe {
        let mut newer_tc: bool;
        let mut old_tc: bool;

        // Check for available tc methods.
        newer_tc = (!strstr(glConfig.extensions_string, "ARB_texture_compression" as *const c_char).is_null()
            && !strstr(glConfig.extensions_string, "EXT_texture_compression_s3tc" as *const c_char).is_null());
        old_tc = !strstr(glConfig.extensions_string, "GL_S3_s3tc" as *const c_char).is_null();

        if old_tc {
            VID_Printf(PRINT_ALL, "...GL_S3_s3tc available\n" as *const c_char);
        }

        if newer_tc {
            VID_Printf(PRINT_ALL, "...GL_EXT_texture_compression_s3tc available\n" as *const c_char);
        }

        if (*r_ext_compressed_textures).is_null() || (*(*r_ext_compressed_textures)).value == 0.0 {
            // Compressed textures are off
            glConfig.textureCompression = TC_NONE;
            VID_Printf(PRINT_ALL, "...ignoring texture compression\n" as *const c_char);
        } else if !old_tc && !newer_tc {
            // Requesting texture compression, but no method found
            glConfig.textureCompression = TC_NONE;
            VID_Printf(PRINT_ALL, "...no supported texture compression method found\n" as *const c_char);
            VID_Printf(PRINT_ALL, ".....ignoring texture compression\n" as *const c_char);
        } else {
            // some form of supported texture compression is avaiable, so see if the user has a preference
            if (*(*r_ext_preferred_tc_method)).integer == TC_NONE {
                // No preference, so pick the best
                if newer_tc {
                    VID_Printf(PRINT_ALL, "...no tc preference specified\n" as *const c_char);
                    VID_Printf(PRINT_ALL, ".....using GL_EXT_texture_compression_s3tc\n" as *const c_char);
                    glConfig.textureCompression = TC_S3TC_DXT;
                } else {
                    VID_Printf(PRINT_ALL, "...no tc preference specified\n" as *const c_char);
                    VID_Printf(PRINT_ALL, ".....using GL_S3_s3tc\n" as *const c_char);
                    glConfig.textureCompression = TC_S3TC;
                }
            } else {
                // User has specified a preference, now see if this request can be honored
                if old_tc && newer_tc {
                    // both are avaiable, so we can use the desired tc method
                    if (*(*r_ext_preferred_tc_method)).integer == TC_S3TC {
                        VID_Printf(PRINT_ALL, "...using preferred tc method, GL_S3_s3tc\n" as *const c_char);
                        glConfig.textureCompression = TC_S3TC;
                    } else {
                        VID_Printf(PRINT_ALL, "...using preferred tc method, GL_EXT_texture_compression_s3tc\n" as *const c_char);
                        glConfig.textureCompression = TC_S3TC_DXT;
                    }
                } else {
                    // Both methods are not available, so this gets trickier
                    if (*(*r_ext_preferred_tc_method)).integer == TC_S3TC {
                        // Preferring to user older compression
                        if old_tc {
                            VID_Printf(PRINT_ALL, "...using GL_S3_s3tc\n" as *const c_char);
                            glConfig.textureCompression = TC_S3TC;
                        } else {
                            // Drat, preference can't be honored
                            VID_Printf(PRINT_ALL, "...preferred tc method, GL_S3_s3tc not available\n" as *const c_char);
                            VID_Printf(PRINT_ALL, ".....falling back to GL_EXT_texture_compression_s3tc\n" as *const c_char);
                            glConfig.textureCompression = TC_S3TC_DXT;
                        }
                    } else {
                        // Preferring to user newer compression
                        if newer_tc {
                            VID_Printf(PRINT_ALL, "...using GL_EXT_texture_compression_s3tc\n" as *const c_char);
                            glConfig.textureCompression = TC_S3TC_DXT;
                        } else {
                            // Drat, preference can't be honored
                            VID_Printf(PRINT_ALL, "...preferred tc method, GL_EXT_texture_compression_s3tc not available\n" as *const c_char);
                            VID_Printf(PRINT_ALL, ".....falling back to GL_S3_s3tc\n" as *const c_char);
                            glConfig.textureCompression = TC_S3TC;
                        }
                    }
                }
            }
        }
    }
}

// GLW_InitExtensions
fn GLW_InitExtensions() {
    unsafe {
        if (*r_allowExtensions).is_null() || (*(*r_allowExtensions)).integer == 0 {
            VID_Printf(PRINT_ALL, "*** IGNORING OPENGL EXTENSIONS ***\n" as *const c_char);
            g_bDynamicGlowSupported = false;
            Cvar_Set("r_DynamicGlow" as *const c_char, "0" as *const c_char);
            return;
        }

        VID_Printf(PRINT_ALL, "Initializing OpenGL extensions\n" as *const c_char);

        // Select our tc scheme
        GLW_InitTextureCompression();

        // GL_EXT_texture_env_add
        glConfig.textureEnvAddAvailable = false;
        if !strstr(glConfig.extensions_string, "EXT_texture_env_add" as *const c_char).is_null() {
            if !(*r_ext_texture_env_add).is_null() && (*(*r_ext_texture_env_add)).integer != 0 {
                glConfig.textureEnvAddAvailable = true;
                VID_Printf(PRINT_ALL, "...using GL_EXT_texture_env_add\n" as *const c_char);
            } else {
                glConfig.textureEnvAddAvailable = false;
                VID_Printf(PRINT_ALL, "...ignoring GL_EXT_texture_env_add\n" as *const c_char);
            }
        } else {
            VID_Printf(PRINT_ALL, "...GL_EXT_texture_env_add not found\n" as *const c_char);
        }

        // GL_EXT_texture_filter_anisotropic
        glConfig.maxTextureFilterAnisotropy = 0.0;
        if !strstr(glConfig.extensions_string, "EXT_texture_filter_anisotropic" as *const c_char).is_null() {
            const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;
            qglGetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut glConfig.maxTextureFilterAnisotropy);
            Com_Printf("...GL_EXT_texture_filter_anisotropic available\n" as *const c_char);

            if !(*r_ext_texture_filter_anisotropic).is_null() && (*(*r_ext_texture_filter_anisotropic)).integer > 1 {
                Com_Printf("...using GL_EXT_texture_filter_anisotropic\n" as *const c_char);
            } else {
                Com_Printf("...ignoring GL_EXT_texture_filter_anisotropic\n" as *const c_char);
            }
            Cvar_Set(
                "r_ext_texture_filter_anisotropic_avail" as *const c_char,
                va("%f" as *const c_char, glConfig.maxTextureFilterAnisotropy),
            );
            if !(*r_ext_texture_filter_anisotropic).is_null() && (*(*r_ext_texture_filter_anisotropic)).value > glConfig.maxTextureFilterAnisotropy {
                Cvar_Set(
                    "r_ext_texture_filter_anisotropic" as *const c_char,
                    va("%f" as *const c_char, glConfig.maxTextureFilterAnisotropy),
                );
            }
        } else {
            Com_Printf("...GL_EXT_texture_filter_anisotropic not found\n" as *const c_char);
            Cvar_Set("r_ext_texture_filter_anisotropic_avail" as *const c_char, "0" as *const c_char);
        }

        // GL_EXT_clamp_to_edge
        glConfig.clampToEdgeAvailable = false;
        if !strstr(glConfig.extensions_string, "GL_EXT_texture_edge_clamp" as *const c_char).is_null() {
            glConfig.clampToEdgeAvailable = true;
            VID_Printf(PRINT_ALL, "...Using GL_EXT_texture_edge_clamp\n" as *const c_char);
        }

        // WGL_EXT_swap_control
        qwglSwapIntervalEXT = qwglGetProcAddress("wglSwapIntervalEXT" as *const c_char);
        if !qwglSwapIntervalEXT.is_null() {
            VID_Printf(PRINT_ALL, "...using WGL_EXT_swap_control\n" as *const c_char);
            if !(*r_swapInterval).is_null() {
                (*(*r_swapInterval)).modified = true;
            }
        } else {
            VID_Printf(PRINT_ALL, "...WGL_EXT_swap_control not found\n" as *const c_char);
        }

        // GL_ARB_multitexture
        qglMultiTexCoord2fARB = core::ptr::null_mut();
        qglActiveTextureARB = core::ptr::null_mut();
        qglClientActiveTextureARB = core::ptr::null_mut();
        if !strstr(glConfig.extensions_string, "GL_ARB_multitexture" as *const c_char).is_null() {
            if !(*r_ext_multitexture).is_null() && (*(*r_ext_multitexture)).integer != 0 {
                qglMultiTexCoord2fARB = qwglGetProcAddress("glMultiTexCoord2fARB" as *const c_char);
                qglActiveTextureARB = qwglGetProcAddress("glActiveTextureARB" as *const c_char);
                qglClientActiveTextureARB = qwglGetProcAddress("glClientActiveTextureARB" as *const c_char);

                if !qglActiveTextureARB.is_null() {
                    qglGetIntegerv(GL_MAX_ACTIVE_TEXTURES_ARB, &mut glConfig.maxActiveTextures);

                    if glConfig.maxActiveTextures > 1 {
                        VID_Printf(PRINT_ALL, "...using GL_ARB_multitexture\n" as *const c_char);
                    } else {
                        qglMultiTexCoord2fARB = core::ptr::null_mut();
                        qglActiveTextureARB = core::ptr::null_mut();
                        qglClientActiveTextureARB = core::ptr::null_mut();
                        VID_Printf(PRINT_ALL, "...not using GL_ARB_multitexture, < 2 texture units\n" as *const c_char);
                    }
                }
            } else {
                VID_Printf(PRINT_ALL, "...ignoring GL_ARB_multitexture\n" as *const c_char);
            }
        } else {
            VID_Printf(PRINT_ALL, "...GL_ARB_multitexture not found\n" as *const c_char);
        }

        // GL_EXT_compiled_vertex_array
        qglLockArraysEXT = core::ptr::null_mut();
        qglUnlockArraysEXT = core::ptr::null_mut();
        if !strstr(glConfig.extensions_string, "GL_EXT_compiled_vertex_array" as *const c_char).is_null() {
            if !(*r_ext_compiled_vertex_array).is_null() && (*(*r_ext_compiled_vertex_array)).integer != 0 {
                VID_Printf(PRINT_ALL, "...using GL_EXT_compiled_vertex_array\n" as *const c_char);
                qglLockArraysEXT = qwglGetProcAddress("glLockArraysEXT" as *const c_char);
                qglUnlockArraysEXT = qwglGetProcAddress("glUnlockArraysEXT" as *const c_char);
                if qglLockArraysEXT.is_null() || qglUnlockArraysEXT.is_null() {
                    Com_Error(ERR_FATAL, "bad getprocaddress" as *const c_char);
                }
            } else {
                VID_Printf(PRINT_ALL, "...ignoring GL_EXT_compiled_vertex_array\n" as *const c_char);
            }
        } else {
            VID_Printf(PRINT_ALL, "...GL_EXT_compiled_vertex_array not found\n" as *const c_char);
        }

        // GL_EXT_point_parameters
        qglPointParameterfEXT = core::ptr::null_mut();
        qglPointParameterfvEXT = core::ptr::null_mut();
        if !strstr(glConfig.extensions_string, "GL_EXT_point_parameters" as *const c_char).is_null() {
            if !(*r_ext_point_parameters).is_null() && (*(*r_ext_point_parameters)).integer != 0 {
                qglPointParameterfEXT = qwglGetProcAddress("glPointParameterfEXT" as *const c_char);
                qglPointParameterfvEXT = qwglGetProcAddress("glPointParameterfvEXT" as *const c_char);
                if qglPointParameterfEXT.is_null() || qglPointParameterfvEXT.is_null() {
                    VID_Printf(ERR_FATAL, "Bad GetProcAddress for GL_EXT_point_parameters" as *const c_char);
                }
                VID_Printf(PRINT_ALL, "...using GL_EXT_point_parameters\n" as *const c_char);
            } else {
                VID_Printf(PRINT_ALL, "...ignoring GL_EXT_point_parameters\n" as *const c_char);
            }
        } else {
            VID_Printf(PRINT_ALL, "...GL_EXT_point_parameters not found\n" as *const c_char);
        }

        // GL_NV_point_sprite
        qglPointParameteriNV = core::ptr::null_mut();
        qglPointParameterivNV = core::ptr::null_mut();
        if !strstr(glConfig.extensions_string, "GL_NV_point_sprite" as *const c_char).is_null() {
            if !(*r_ext_nv_point_sprite).is_null() && (*(*r_ext_nv_point_sprite)).integer != 0 {
                qglPointParameteriNV = qwglGetProcAddress("glPointParameteriNV" as *const c_char);
                qglPointParameterivNV = qwglGetProcAddress("glPointParameterivNV" as *const c_char);
                if qglPointParameteriNV.is_null() || qglPointParameterivNV.is_null() {
                    VID_Printf(ERR_FATAL, "Bad GetProcAddress for GL_NV_point_sprite" as *const c_char);
                }
                VID_Printf(PRINT_ALL, "...using GL_NV_point_sprite\n" as *const c_char);
            } else {
                VID_Printf(PRINT_ALL, "...ignoring GL_NV_point_sprite\n" as *const c_char);
            }
        } else {
            VID_Printf(PRINT_ALL, "...GL_NV_point_sprite not found\n" as *const c_char);
        }

        let mut bNVRegisterCombiners: bool = false;
        // Register Combiners.
        if !strstr(glConfig.extensions_string, "GL_NV_register_combiners" as *const c_char).is_null() {
            // NOTE: This extension requires multitexture support (over 2 units).
            if glConfig.maxActiveTextures >= 2 {
                bNVRegisterCombiners = true;
                // Register Combiners function pointer address load.	- AReis
                // NOTE: VV guys will _definetly_ not be able to use regcoms. Pixel Shaders are just as good though :-)
                // NOTE: Also, this is an nVidia specific extension (of course), so fragment shaders would serve the same purpose
                // if we needed some kind of fragment/pixel manipulation support.
                qglCombinerParameterfvNV = qwglGetProcAddress("glCombinerParameterfvNV" as *const c_char);
                qglCombinerParameterivNV = qwglGetProcAddress("glCombinerParameterivNV" as *const c_char);
                qglCombinerParameterfNV = qwglGetProcAddress("glCombinerParameterfNV" as *const c_char);
                qglCombinerParameteriNV = qwglGetProcAddress("glCombinerParameteriNV" as *const c_char);
                qglCombinerInputNV = qwglGetProcAddress("glCombinerInputNV" as *const c_char);
                qglCombinerOutputNV = qwglGetProcAddress("glCombinerOutputNV" as *const c_char);
                qglFinalCombinerInputNV = qwglGetProcAddress("glFinalCombinerInputNV" as *const c_char);
                qglGetCombinerInputParameterfvNV = qwglGetProcAddress("glGetCombinerInputParameterfvNV" as *const c_char);
                qglGetCombinerInputParameterivNV = qwglGetProcAddress("glGetCombinerInputParameterivNV" as *const c_char);
                qglGetCombinerOutputParameterfvNV = qwglGetProcAddress("glGetCombinerOutputParameterfvNV" as *const c_char);
                qglGetCombinerOutputParameterivNV = qwglGetProcAddress("glGetCombinerOutputParameterivNV" as *const c_char);
                qglGetFinalCombinerInputParameterfvNV = qwglGetProcAddress("glGetFinalCombinerInputParameterfvNV" as *const c_char);
                qglGetFinalCombinerInputParameterivNV = qwglGetProcAddress("glGetFinalCombinerInputParameterivNV" as *const c_char);

                // Validate the functions we need.
                if qglCombinerParameterfvNV.is_null() || qglCombinerParameterivNV.is_null() || qglCombinerParameterfNV.is_null() || qglCombinerParameteriNV.is_null() || qglCombinerInputNV.is_null()
                    || qglCombinerOutputNV.is_null() || qglFinalCombinerInputNV.is_null() || qglGetCombinerInputParameterfvNV.is_null() || qglGetCombinerInputParameterivNV.is_null()
                    || qglGetCombinerOutputParameterfvNV.is_null() || qglGetCombinerOutputParameterivNV.is_null() || qglGetFinalCombinerInputParameterfvNV.is_null() || qglGetFinalCombinerInputParameterivNV.is_null()
                {
                    bNVRegisterCombiners = false;
                    qglCombinerParameterfvNV = core::ptr::null_mut();
                    qglCombinerParameteriNV = core::ptr::null_mut();
                    Com_Printf("...GL_NV_register_combiners failed\n" as *const c_char);
                }
            } else {
                bNVRegisterCombiners = false;
                Com_Printf("...ignoring GL_NV_register_combiners\n" as *const c_char);
            }
        } else {
            bNVRegisterCombiners = false;
            Com_Printf("...GL_NV_register_combiners not found\n" as *const c_char);
        }

        // NOTE: Vertex and Fragment Programs are very dependant on each other - this is actually a
        // good thing! So, just check to see which we support (one or the other) and load the shared
        // function pointers. ARB rocks!

        // Vertex Programs.
        let mut bARBVertexProgram: bool = false;
        if !strstr(glConfig.extensions_string, "GL_ARB_vertex_program" as *const c_char).is_null() {
            bARBVertexProgram = true;
        } else {
            bARBVertexProgram = false;
            Com_Printf("...GL_ARB_vertex_program not found\n" as *const c_char);
        }

        let mut bARBFragmentProgram: bool = false;
        // Fragment Programs.
        if !strstr(glConfig.extensions_string, "GL_ARB_fragment_program" as *const c_char).is_null() {
            bARBFragmentProgram = true;
        } else {
            bARBFragmentProgram = false;
            Com_Printf("...GL_ARB_fragment_program not found\n" as *const c_char);
        }

        // If we support one or the other, load the shared function pointers.
        if bARBVertexProgram || bARBFragmentProgram {
            qglProgramStringARB = qwglGetProcAddress("glProgramStringARB" as *const c_char);
            qglBindProgramARB = qwglGetProcAddress("glBindProgramARB" as *const c_char);
            qglDeleteProgramsARB = qwglGetProcAddress("glDeleteProgramsARB" as *const c_char);
            qglGenProgramsARB = qwglGetProcAddress("glGenProgramsARB" as *const c_char);
            qglProgramEnvParameter4dARB = qwglGetProcAddress("glProgramEnvParameter4dARB" as *const c_char);
            qglProgramEnvParameter4dvARB = qwglGetProcAddress("glProgramEnvParameter4dvARB" as *const c_char);
            qglProgramEnvParameter4fARB = qwglGetProcAddress("glProgramEnvParameter4fARB" as *const c_char);
            qglProgramEnvParameter4fvARB = qwglGetProcAddress("glProgramEnvParameter4fvARB" as *const c_char);
            qglProgramLocalParameter4dARB = qwglGetProcAddress("glProgramLocalParameter4dARB" as *const c_char);
            qglProgramLocalParameter4dvARB = qwglGetProcAddress("glProgramLocalParameter4dvARB" as *const c_char);
            qglProgramLocalParameter4fARB = qwglGetProcAddress("glProgramLocalParameter4fARB" as *const c_char);
            qglProgramLocalParameter4fvARB = qwglGetProcAddress("glProgramLocalParameter4fvARB" as *const c_char);
            qglGetProgramEnvParameterdvARB = qwglGetProcAddress("glGetProgramEnvParameterdvARB" as *const c_char);
            qglGetProgramEnvParameterfvARB = qwglGetProcAddress("glGetProgramEnvParameterfvARB" as *const c_char);
            qglGetProgramLocalParameterdvARB = qwglGetProcAddress("glGetProgramLocalParameterdvARB" as *const c_char);
            qglGetProgramLocalParameterfvARB = qwglGetProcAddress("glGetProgramLocalParameterfvARB" as *const c_char);
            qglGetProgramivARB = qwglGetProcAddress("glGetProgramivARB" as *const c_char);
            qglGetProgramStringARB = qwglGetProcAddress("glGetProgramStringARB" as *const c_char);
            qglIsProgramARB = qwglGetProcAddress("glIsProgramARB" as *const c_char);

            // Validate the functions we need.
            if qglProgramStringARB.is_null() || qglBindProgramARB.is_null() || qglDeleteProgramsARB.is_null() || qglGenProgramsARB.is_null()
                || qglProgramEnvParameter4dARB.is_null() || qglProgramEnvParameter4dvARB.is_null() || qglProgramEnvParameter4fARB.is_null()
                || qglProgramEnvParameter4fvARB.is_null() || qglProgramLocalParameter4dARB.is_null() || qglProgramLocalParameter4dvARB.is_null()
                || qglProgramLocalParameter4fARB.is_null() || qglProgramLocalParameter4fvARB.is_null() || qglGetProgramEnvParameterdvARB.is_null()
                || qglGetProgramEnvParameterfvARB.is_null() || qglGetProgramLocalParameterdvARB.is_null() || qglGetProgramLocalParameterfvARB.is_null()
                || qglGetProgramivARB.is_null() || qglGetProgramStringARB.is_null() || qglIsProgramARB.is_null()
            {
                bARBVertexProgram = false;
                bARBFragmentProgram = false;
                qglGenProgramsARB = core::ptr::null_mut();
                qglProgramEnvParameter4fARB = core::ptr::null_mut();
                Com_Printf("...ignoring GL_ARB_vertex_program\n" as *const c_char);
                Com_Printf("...ignoring GL_ARB_fragment_program\n" as *const c_char);
            }
        }

        // Figure out which texture rectangle extension to use.
        let mut bTexRectSupported: bool = false;
        if strnicmp(glConfig.vendor_string, "ATI Technologies" as *const c_char, 16) == 0
            && strnicmp(glConfig.version_string, "1.3.3" as *const c_char, 5) == 0
            && glConfig.version_string.add(5).read() < ('9' as c_char)
        {
            g_bTextureRectangleHack = true;
        }

        if !strstr(glConfig.extensions_string, "GL_NV_texture_rectangle" as *const c_char).is_null()
            || !strstr(glConfig.extensions_string, "GL_EXT_texture_rectangle" as *const c_char).is_null()
        {
            bTexRectSupported = true;
        }

        // OK, so not so good to put this here, but no one else uses it!!! -AReis
        type PFNWGLGETEXTENSIONSSTRINGARBPROC = *const c_char;
        let qwglGetExtensionsStringARB: PFNWGLGETEXTENSIONSSTRINGARBPROC = qwglGetProcAddress("wglGetExtensionsStringARB" as *const c_char) as PFNWGLGETEXTENSIONSSTRINGARBPROC;

        let mut wglExtensions: *const c_char = core::ptr::null();
        let mut bHasPixelFormat: bool = false;
        let mut bHasRenderTexture: bool = false;

        // Get the WGL extensions string.
        if !qwglGetExtensionsStringARB.is_null() {
            wglExtensions = qwglGetExtensionsStringARB;
        }

        // This externsion is used to get the wgl extension string.
        if !wglExtensions.is_null() {
            // Pixel Format.
            if !strstr(wglExtensions, "WGL_ARB_pixel_format" as *const c_char).is_null() {
                qwglGetPixelFormatAttribivARB = qwglGetProcAddress("wglGetPixelFormatAttribivARB" as *const c_char);
                qwglGetPixelFormatAttribfvARB = qwglGetProcAddress("wglGetPixelFormatAttribfvARB" as *const c_char);
                qwglChoosePixelFormatARB = qwglGetProcAddress("wglChoosePixelFormatARB" as *const c_char);

                // Validate the functions we need.
                if qwglGetPixelFormatAttribivARB.is_null() || qwglGetPixelFormatAttribfvARB.is_null() || qwglChoosePixelFormatARB.is_null() {
                    Com_Printf("...ignoring WGL_ARB_pixel_format\n" as *const c_char);
                } else {
                    bHasPixelFormat = true;
                }
            } else {
                Com_Printf("...ignoring WGL_ARB_pixel_format\n" as *const c_char);
            }

            // Offscreen pixel-buffer.
            // NOTE: VV guys can use the equivelant SetRenderTarget() with the correct texture surfaces.
            let mut bWGLARBPbuffer: bool = false;
            if !strstr(wglExtensions, "WGL_ARB_pbuffer" as *const c_char).is_null() && bHasPixelFormat {
                bWGLARBPbuffer = true;
                qwglCreatePbufferARB = qwglGetProcAddress("wglCreatePbufferARB" as *const c_char);
                qwglGetPbufferDCARB = qwglGetProcAddress("wglGetPbufferDCARB" as *const c_char);
                qwglReleasePbufferDCARB = qwglGetProcAddress("wglReleasePbufferDCARB" as *const c_char);
                qwglDestroyPbufferARB = qwglGetProcAddress("wglDestroyPbufferARB" as *const c_char);
                qwglQueryPbufferARB = qwglGetProcAddress("wglQueryPbufferARB" as *const c_char);

                // Validate the functions we need.
                if qwglCreatePbufferARB.is_null() || qwglGetPbufferDCARB.is_null() || qwglReleasePbufferDCARB.is_null() || qwglDestroyPbufferARB.is_null() || qwglQueryPbufferARB.is_null() {
                    bWGLARBPbuffer = false;
                    Com_Printf("...WGL_ARB_pbuffer failed\n" as *const c_char);
                }
            } else {
                bWGLARBPbuffer = false;
                Com_Printf("...WGL_ARB_pbuffer not found\n" as *const c_char);
            }

            // Render-Texture (requires pbuffer ext (and it's dependancies of course).
            if !strstr(wglExtensions, "WGL_ARB_render_texture" as *const c_char).is_null() && bWGLARBPbuffer {
                qwglBindTexImageARB = qwglGetProcAddress("wglBindTexImageARB" as *const c_char);
                qwglReleaseTexImageARB = qwglGetProcAddress("wglReleaseTexImageARB" as *const c_char);
                qwglSetPbufferAttribARB = qwglGetProcAddress("wglSetPbufferAttribARB" as *const c_char);

                // Validate the functions we need.
                if qwglCreatePbufferARB.is_null() || qwglGetPbufferDCARB.is_null() || qwglReleasePbufferDCARB.is_null() || qwglDestroyPbufferARB.is_null() || qwglQueryPbufferARB.is_null() {
                    Com_Printf("...ignoring WGL_ARB_render_texture\n" as *const c_char);
                } else {
                    bHasRenderTexture = true;
                }
            } else {
                Com_Printf("...ignoring WGL_ARB_render_texture\n" as *const c_char);
            }
        }

        // Find out how many general combiners they have.
        const GL_MAX_GENERAL_COMBINERS_NV: u32 = 0x854D;
        let mut iNumGeneralCombiners: c_int = 0;
        qglGetIntegerv(GL_MAX_GENERAL_COMBINERS_NV, &mut iNumGeneralCombiners);

        // Only allow dynamic glows/flares if they have the hardware
        if bTexRectSupported && bARBVertexProgram && bHasRenderTexture && !qglActiveTextureARB.is_null() && glConfig.maxActiveTextures >= 4
            && (((bNVRegisterCombiners && iNumGeneralCombiners >= 2) || bARBFragmentProgram))
        {
            g_bDynamicGlowSupported = true;
            // this would overwrite any achived setting gwg
            // Cvar_Set( "r_DynamicGlow", "1" );
        } else {
            g_bDynamicGlowSupported = false;
            Cvar_Set("r_DynamicGlow" as *const c_char, "0" as *const c_char);
        }
    }
}

// GLW_CheckOSVersion
fn GLW_CheckOSVersion() -> bool {
    unsafe {
        const OSR2_BUILD_NUMBER: u32 = 1111;

        let mut vinfo: OSVERSIONINFO = core::mem::zeroed();

        vinfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFO>() as u32;

        glw_state.allowdisplaydepthchange = false;

        if GetVersionEx(&mut vinfo) != 0 {
            if vinfo.dwMajorVersion > 4 {
                glw_state.allowdisplaydepthchange = true;
            } else if vinfo.dwMajorVersion == 4 {
                if vinfo.dwPlatformId == VER_PLATFORM_WIN32_NT {
                    glw_state.allowdisplaydepthchange = true;
                } else if vinfo.dwPlatformId == VER_PLATFORM_WIN32_WINDOWS {
                    if (vinfo.dwBuildNumber as u16) >= OSR2_BUILD_NUMBER as u16 {
                        glw_state.allowdisplaydepthchange = true;
                    }
                }
            }
        } else {
            VID_Printf(PRINT_ALL, "GLW_CheckOSVersion() - GetVersionEx failed\n" as *const c_char);
            return false;
        }

        return true;
    }
}

// GLW_LoadOpenGL
//
// GLimp_win.c internal function that attempts to load and use
// a specific OpenGL DLL.
fn GLW_LoadOpenGL() -> bool {
    unsafe {
        let mut buffer: [c_char; 1024] = [0; 1024];
        let mut cdsFullscreen: bool;

        strlwr(strcpy(buffer.as_mut_ptr(), "opengl32.dll" as *const c_char));

        // load the driver and bind our function pointers to it
        if QGL_Init(buffer.as_ptr()) {
            cdsFullscreen = (*(*r_fullscreen)).integer != 0;

            // create the window and set up the context
            if !GLW_StartDriverAndSetMode((*(*r_mode)).integer, (*(*r_colorbits)).integer, cdsFullscreen) {
                // if we're on a 24/32-bit desktop and we're going fullscreen
                // try it again but with a 16-bit desktop
                if (*(*r_colorbits)).integer != 16 || cdsFullscreen != true || (*(*r_mode)).integer != 3 {
                    if !GLW_StartDriverAndSetMode(3, 16, true) {
                        goto fail;
                    }
                }
            }
            return true;
        }
        fail: QGL_Shutdown();

        return false;
    }
}

fn GLW_StartDriverAndSetMode(mode: c_int, colorbits: c_int, cdsFullscreen: bool) -> bool {
    unsafe {
        let err: rserr_t = GLW_SetMode(mode, colorbits, cdsFullscreen);

        match err {
            rserr_t::RSERR_INVALID_FULLSCREEN => {
                VID_Printf(PRINT_ALL, "...WARNING: fullscreen unavailable in this mode\n" as *const c_char);
                return false;
            }
            rserr_t::RSERR_INVALID_MODE => {
                VID_Printf(PRINT_ALL, "...WARNING: could not set the given mode (%d)\n" as *const c_char, mode);
                return false;
            }
            _ => {}
        }
        return true;
    }
}

fn GLW_StartOpenGL() {
    unsafe {
        // load and initialize the specific OpenGL driver
        if !GLW_LoadOpenGL() {
            Com_Error(ERR_FATAL, "GLW_StartOpenGL() - could not load OpenGL subsystem\n" as *const c_char);
        }
    }
}

// GLimp_EndFrame
#[no_mangle]
pub extern "C" fn GLimp_EndFrame() {
    unsafe {
        // swapinterval stuff
        if !(*r_swapInterval).is_null() && (*(*r_swapInterval)).modified {
            (*(*r_swapInterval)).modified = false;

            if !glConfig.stereoEnabled {
                if !qwglSwapIntervalEXT.is_null() {
                    // NOTE: qwglSwapIntervalEXT is a function pointer and needs to be called properly
                    // This is a simplified version - actual implementation would need proper casting
                }
            }
        }

        // don't flip if drawing to front buffer
        SwapBuffers(glw_state.hDC);

        // check logging
        QGL_EnableLogging(if !(*r_logFile).is_null() { (*(*r_logFile)).integer != 0 } else { false });
    }
}

// GLimp_Init
//
// This is the platform specific OpenGL initialization function.  It
// is responsible for loading OpenGL, initializing it, setting
// extensions, creating a window of the appropriate size, doing
// fullscreen manipulations, etc.  Its overall responsibility is
// to make sure that a functional OpenGL subsystem is operating
// when it returns to the ref.
#[no_mangle]
pub extern "C" fn GLimp_Init() {
    unsafe {
        let mut buf: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
        let lastValidRenderer: *mut cvar_t = Cvar_Get("r_lastValidRenderer" as *const c_char, "(uninitialized)" as *const c_char, CVAR_ARCHIVE);
        let mut cv: *mut cvar_t;

        VID_Printf(PRINT_ALL, "Initializing OpenGL subsystem\n" as *const c_char);

        // check OS version to see if we can do fullscreen display changes
        if !GLW_CheckOSVersion() {
            Com_Error(ERR_FATAL, "GLimp_Init() - incorrect operating system\n" as *const c_char);
        }

        // save off hInstance and wndproc
        cv = Cvar_Get("win_hinstance" as *const c_char, "" as *const c_char, 0);
        sscanf((*cv).string, "%i" as *const c_char, &mut (g_wv.hInstance as *mut c_int));

        cv = Cvar_Get("win_wndproc" as *const c_char, "" as *const c_char, 0);
        sscanf((*cv).string, "%i" as *const c_char, &mut (glw_state.wndproc as *mut c_int));

        r_allowSoftwareGL = Cvar_Get("r_allowSoftwareGL" as *const c_char, "0" as *const c_char, CVAR_LATCH);

        // load appropriate DLL and initialize subsystem
        GLW_StartOpenGL();

        // get our config strings
        glConfig.vendor_string = qglGetString(GL_VENDOR) as *const c_char;
        glConfig.renderer_string = qglGetString(GL_RENDERER) as *const c_char;
        glConfig.version_string = qglGetString(GL_VERSION) as *const c_char;
        glConfig.extensions_string = qglGetString(GL_EXTENSIONS) as *const c_char;

        if glConfig.vendor_string.is_null() || glConfig.renderer_string.is_null() || glConfig.version_string.is_null() || glConfig.extensions_string.is_null() {
            Com_Error(ERR_FATAL, "GLimp_Init() - Invalid GL Driver\n" as *const c_char);
        }

        // OpenGL driver constants
        qglGetIntegerv(GL_MAX_TEXTURE_SIZE, &mut glConfig.maxTextureSize);
        // stubbed or broken drivers may have reported 0...
        if glConfig.maxTextureSize <= 0 {
            glConfig.maxTextureSize = 0;
        }

        // chipset specific configuration
        strcpy(buf.as_mut_ptr(), glConfig.renderer_string);
        strlwr(buf.as_mut_ptr());

        // NOTE: if changing cvars, do it within this block.  This allows them
        // to be overridden when testing driver fixes, etc. but only sets
        // them to their default state when the hardware is first installed/run.

        if Q_stricmp((*lastValidRenderer).string, glConfig.renderer_string) != 0 {
            if Sys_LowPhysicalMemory() {
                Cvar_Set("s_khz" as *const c_char, "11" as *const c_char);
                Cvar_Set("cg_VariantSoundCap" as *const c_char, "2" as *const c_char);
                Cvar_Set("s_allowDynamicMusic" as *const c_char, "0" as *const c_char);
            }
            // reset to defaults
            Cvar_Set("r_picmip" as *const c_char, "1" as *const c_char);

            // Savage3D and Savage4 should always have trilinear enabled
            if !strstr(buf.as_ptr(), "savage3d" as *const c_char).is_null()
                || !strstr(buf.as_ptr(), "s3 savage4" as *const c_char).is_null()
                || !strstr(buf.as_ptr(), "geforce" as *const c_char).is_null()
                || !strstr(buf.as_ptr(), "quadro" as *const c_char).is_null()
            {
                Cvar_Set("r_texturemode" as *const c_char, "GL_LINEAR_MIPMAP_LINEAR" as *const c_char);
            } else {
                Cvar_Set("r_textureMode" as *const c_char, "GL_LINEAR_MIPMAP_NEAREST" as *const c_char);
            }

            if !strstr(buf.as_ptr(), "kyro" as *const c_char).is_null() {
                Cvar_Set("r_ext_texture_filter_anisotropic" as *const c_char, "0" as *const c_char);
                Cvar_Set("r_ext_preferred_tc_method" as *const c_char, "1" as *const c_char);
            }

            if !strstr(buf.as_ptr(), "geforce2" as *const c_char).is_null() {
                Cvar_Set("cg_renderToTextureFX" as *const c_char, "0" as *const c_char);
            }

            if !strstr(buf.as_ptr(), "radeon 9000" as *const c_char).is_null() {
                Cvar_Set("cg_renderToTextureFX" as *const c_char, "0" as *const c_char);
            }

            GLW_InitExtensions();
            // this must be a really sucky card!
            if (glConfig.textureCompression == TC_NONE) || (glConfig.maxActiveTextures < 2) || (glConfig.maxTextureSize <= 512) {
                Cvar_Set("r_picmip" as *const c_char, "2" as *const c_char);
                Cvar_Set("r_colorbits" as *const c_char, "16" as *const c_char);
                Cvar_Set("r_texturebits" as *const c_char, "16" as *const c_char);
                Cvar_Set("r_mode" as *const c_char, "3" as *const c_char);
                Cmd_ExecuteString("exec low.cfg\n" as *const c_char);
            }
        }

        Cvar_Set("r_lastValidRenderer" as *const c_char, glConfig.renderer_string);
        GLW_InitExtensions();

        WG_CheckHardwareGamma();
    }
}

// GLimp_Shutdown
//
// This routine does all OS specific shutdown procedures for the OpenGL
// subsystem.
#[no_mangle]
pub extern "C" fn GLimp_Shutdown() {
    unsafe {
        let success: [&[u8]; 2] = [b"failed\0", b"success\0"];
        let mut retVal: c_int;

        // FIXME: Brian, we need better fallbacks from partially initialized failures
        if qwglMakeCurrent.is_null() {
            return;
        }

        VID_Printf(PRINT_ALL, "Shutting down OpenGL subsystem\n" as *const c_char);

        // restore gamma.  We do this first because 3Dfx's extension needs a valid OGL subsystem
        WG_RestoreGamma();

        // set current context to NULL
        if !qwglMakeCurrent.is_null() {
            retVal = if qwglMakeCurrent(core::ptr::null_mut(), core::ptr::null_mut()) != 0 { 1 } else { 0 };

            VID_Printf(PRINT_ALL, "...wglMakeCurrent( NULL, NULL ): %s\n" as *const c_char, success[retVal as usize].as_ptr() as *const c_char);
        }

        // delete HGLRC
        if !glw_state.hGLRC.is_null() {
            retVal = if qwglDeleteContext(glw_state.hGLRC) != 0 { 1 } else { 0 };
            VID_Printf(PRINT_ALL, "...deleting GL context: %s\n" as *const c_char, success[retVal as usize].as_ptr() as *const c_char);
            glw_state.hGLRC = core::ptr::null_mut();
        }

        // release DC
        if !glw_state.hDC.is_null() {
            retVal = if ReleaseDC(g_wv.hWnd, glw_state.hDC) != 0 { 1 } else { 0 };
            VID_Printf(PRINT_ALL, "...releasing DC: %s\n" as *const c_char, success[retVal as usize].as_ptr() as *const c_char);
            glw_state.hDC = core::ptr::null_mut();
        }

        // destroy window
        if !g_wv.hWnd.is_null() {
            VID_Printf(PRINT_ALL, "...destroying window\n" as *const c_char);
            ShowWindow(g_wv.hWnd, SW_HIDE);
            DestroyWindow(g_wv.hWnd);
            g_wv.hWnd = core::ptr::null_mut();
            glw_state.pixelFormatSet = false;
        }

        // close the r_logFile
        if !glw_state.log_fp.is_null() {
            fclose(glw_state.log_fp);
            glw_state.log_fp = core::ptr::null_mut();
        }

        // reset display settings
        if glw_state.cdsFullscreen {
            VID_Printf(PRINT_ALL, "...resetting display\n" as *const c_char);
            ChangeDisplaySettings(core::ptr::null(), 0);
            glw_state.cdsFullscreen = false;
        }

        // shutdown QGL subsystem
        QGL_Shutdown();

        memset(&mut glConfig as *mut _ as *mut c_void, 0, core::mem::size_of::<glconfig_t>());
        memset(&mut glState as *mut _ as *mut c_void, 0, core::mem::size_of::<glstate_t>());
    }
}

// GLimp_LogComment
#[no_mangle]
pub extern "C" fn GLimp_LogComment(comment: *mut c_char) {
    unsafe {
        if !glw_state.log_fp.is_null() {
            fprintf(glw_state.log_fp, "%s" as *const c_char, comment);
        }
    }
}

// Helper macro for MAKEINTRESOURCE
fn MAKEINTRESOURCE(id: u16) -> *const c_char {
    id as *const c_char
}
