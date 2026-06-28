//! q_shared.h -- included first by ALL program modules.
//! these are the definitions that have no dependance on
//! central system services, and can be used by any part
//! of the program without any state issues.
//!
//! A user mod should never modify this file

use core::ffi::{c_char, c_int, c_void};

// incursion of DOOM code into the Q3A codebase
//#define	Q3_VERSION		"DOOM 0.01"

// alignment macros for SIMD
// Note: In Rust, these are no-ops; alignment is handled at compile-time
pub const ALIGN_ON: &str = "";
pub const ALIGN_OFF: &str = "";

//======================= WIN32 DEFINES =================================

// Conditional platform detection for CPU strings
#[cfg(target_arch = "x86")]
#[cfg(not(target_os = "sunos"))]
pub const ID386: c_int = 1;

#[cfg(not(all(target_arch = "x86", not(target_os = "sunos"))))]
pub const ID386: c_int = 0;

// for windows fastcall option
pub const QDECL: &str = "";

#[cfg(target_os = "windows")]
pub const MAC_STATIC: &str = "";

#[cfg(target_os = "windows")]
pub mod platform {
    // buildstring will be incorporated into the version string
    #[cfg(debug_assertions)]
    #[cfg(target_arch = "x86")]
    pub const CPUSTRING: &str = "win-x86-debug";

    #[cfg(debug_assertions)]
    #[cfg(target_arch = "x86_64")]
    pub const CPUSTRING: &str = "win-x86_64-debug";

    #[cfg(not(debug_assertions))]
    #[cfg(target_arch = "x86")]
    pub const CPUSTRING: &str = "win-x86";

    #[cfg(not(debug_assertions))]
    #[cfg(target_arch = "x86_64")]
    pub const CPUSTRING: &str = "win-x86_64";

    pub const PATH_SEP: char = '\\';
}

//======================= MAC OS X SERVER DEFINES =====================

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub mod platform {
    pub const MAC_STATIC: &str = "";
    pub const CPUSTRING: &str = "MacOSXS-aarch64";
    pub const PATH_SEP: char = '/';
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub mod platform {
    pub const MAC_STATIC: &str = "";
    pub const CPUSTRING: &str = "MacOSXS-x86_64";
    pub const PATH_SEP: char = '/';
}

#[cfg(all(target_os = "macos", target_arch = "x86"))]
pub mod platform {
    pub const MAC_STATIC: &str = "";
    pub const CPUSTRING: &str = "MacOSXS-i386";
    pub const PATH_SEP: char = '/';
}

#[cfg(target_os = "macos")]
pub const GAME_HARD_LINKED: bool = true;

#[cfg(target_os = "macos")]
pub const CGAME_HARD_LINKED: bool = true;

#[cfg(target_os = "macos")]
pub const UI_HARD_LINKED: bool = true;

//======================= LINUX DEFINES =================================

#[cfg(target_os = "linux")]
pub mod platform {
    #[cfg(target_arch = "x86")]
    pub const CPUSTRING: &str = "linux-i386";

    #[cfg(target_arch = "x86_64")]
    pub const CPUSTRING: &str = "linux-x86_64";

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub const CPUSTRING: &str = "linux-other";

    pub const PATH_SEP: char = '/';
}

#[cfg(target_os = "linux")]
pub const MAC_STATIC: &str = "";

#[cfg(all(target_os = "linux", feature = "Q3_STATIC"))]
pub const GAME_HARD_LINKED: bool = true;

#[cfg(all(target_os = "linux", feature = "Q3_STATIC"))]
pub const CGAME_HARD_LINKED: bool = true;

#[cfg(all(target_os = "linux", feature = "Q3_STATIC"))]
pub const UI_HARD_LINKED: bool = true;

#[cfg(all(target_os = "linux", feature = "Q3_STATIC"))]
pub const BOTLIB_HARD_LINKED: bool = true;

//=============================================================

pub type qboolean = c_int;

/// for signed/unsigned mismatch
pub const QFALSE: qboolean = 0;
pub const QTRUE: qboolean = 1;

pub type byte = u8;

pub const EQUAL_EPSILON: f32 = 0.001;

pub type qhandle_t = c_int;
pub type sfxHandle_t = c_int;
pub type fileHandle_t = c_int;
pub type clipHandle_t = c_int;

#[repr(C)]
pub enum jointHandle_t {
    INVALID_JOINT = -1,
}

// Standard constants
pub const MAX_QINT: c_int = 0x7fffffff;
pub const MIN_QINT: c_int = -0x7fffffff - 1;

/// angle indexes
pub const PITCH: usize = 0;    // up / down
pub const YAW: usize = 1;      // left / right
pub const ROLL: usize = 2;     // fall over

// the game guarantees that no string from the network will ever
// exceed MAX_STRING_CHARS
pub const MAX_STRING_CHARS: usize = 1024;  // max length of a string passed to Cmd_TokenizeString
pub const MAX_STRING_TOKENS: usize = 256;  // max tokens resulting from Cmd_TokenizeString
pub const MAX_TOKEN_CHARS: usize = 1024;   // max length of an individual token

pub const MAX_INFO_STRING: usize = 1024;
pub const MAX_INFO_KEY: usize = 1024;
pub const MAX_INFO_VALUE: usize = 1024;

pub const MAX_QPATH: usize = 64;      // max length of a quake game pathname
pub const MAX_OSPATH: usize = 128;    // max length of a filesystem pathname

pub const MAX_NAME_LENGTH: usize = 32; // max length of a client name

// paramters for command buffer stuffing
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum cbufExec_t {
    EXEC_NOW = 0,    // don't return until completed, a VM should NEVER use this,
                     // because some commands might cause the VM to be unloaded...
    EXEC_INSERT = 1, // insert at current position, but don't run yet
    EXEC_APPEND = 2, // add to end of the command buffer (normal case)
}

//
// these aren't needed by any of the VMs.  put in another header?
//
pub const MAX_MAP_AREA_BYTES: usize = 32; // bit vector of area visibility

// parameters to the main Error routine
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum errorParm_t {
    ERR_NONE = 0,
    ERR_FATAL = 1,       // exit the entire game with a popup window
    ERR_DROP = 2,        // print to console and disconnect from game
    ERR_DISCONNECT = 3,  // don't kill server
    ERR_NEED_CD = 4,     // pop up the need-cd dialog
}

// font rendering values used by ui and cgame

pub const PROP_GAP_WIDTH: usize = 3;
pub const PROP_SPACE_WIDTH: usize = 8;
pub const PROP_HEIGHT: usize = 27;
pub const PROP_SMALL_SIZE_SCALE: f32 = 0.75;

pub const BLINK_DIVISOR: usize = 200;
pub const PULSE_DIVISOR: usize = 75;

pub const UI_LEFT: u32 = 0x00000000;   // default
pub const UI_CENTER: u32 = 0x00000001;
pub const UI_RIGHT: u32 = 0x00000002;
pub const UI_FORMATMASK: u32 = 0x00000007;
pub const UI_SMALLFONT: u32 = 0x00000010;
pub const UI_BIGFONT: u32 = 0x00000020;   // default
pub const UI_GIANTFONT: u32 = 0x00000040;
pub const UI_DROPSHADOW: u32 = 0x00000800;
pub const UI_BLINK: u32 = 0x00001000;
pub const UI_INVERSE: u32 = 0x00002000;
pub const UI_PULSE: u32 = 0x00004000;

/*
==============================================================

MATHLIB

==============================================================
*/

// These are C++ only in the original, but we provide the constants for compatibility
pub const SIDE_FRONT: c_int = 0;
pub const SIDE_BACK: c_int = 1;
pub const SIDE_ON: c_int = 2;
pub const SIDE_CROSS: c_int = 3;

pub const Q_PI: f64 = 3.14159265358979323846;
pub const M_PI: f64 = 3.14159265358979323846; // matches value in gcc v2 math.h

// NUMVERTEXNORMALS is defined as 162 in the original
pub const NUMVERTEXNORMALS: usize = 162;

// External declaration for bytedirs array (defined in math vector implementation)
// extern const idVec3_t bytedirs[NUMVERTEXNORMALS];

// all drawing is done to a 640*480 virtual screen size
// and will be automatically scaled to the real resolution
pub const SCREEN_WIDTH: usize = 640;
pub const SCREEN_HEIGHT: usize = 480;

pub const TINYCHAR_WIDTH: usize = 8;       // SMALLCHAR_WIDTH
pub const TINYCHAR_HEIGHT: usize = 8;      // SMALLCHAR_HEIGHT/2

pub const SMALLCHAR_WIDTH: usize = 8;
pub const SMALLCHAR_HEIGHT: usize = 16;

pub const BIGCHAR_WIDTH: usize = 16;
pub const BIGCHAR_HEIGHT: usize = 16;

pub const GIANTCHAR_WIDTH: usize = 32;
pub const GIANTCHAR_HEIGHT: usize = 48;

// extern declarations for color vectors
// extern vec4_t colorBlack;
// extern vec4_t colorRed;
// extern vec4_t colorGreen;
// extern vec4_t colorBlue;
// extern vec4_t colorYellow;
// extern vec4_t colorMagenta;
// extern vec4_t colorCyan;
// extern vec4_t colorWhite;
// extern vec4_t colorLtGrey;
// extern vec4_t colorMdGrey;
// extern vec4_t colorDkGrey;

pub const Q_COLOR_ESCAPE: char = '^';

/// Macro replacement: Q_IsColorString(p) checks if p is a color escape sequence
#[inline]
pub fn Q_IsColorString(p: *const c_char) -> bool {
    unsafe {
        !p.is_null()
            && *p == Q_COLOR_ESCAPE as c_char
            && !(*p.offset(1)).is_null()
            && *p.offset(1) != Q_COLOR_ESCAPE as c_char
    }
}

pub const COLOR_BLACK: c_char = b'0' as c_char;
pub const COLOR_RED: c_char = b'1' as c_char;
pub const COLOR_GREEN: c_char = b'2' as c_char;
pub const COLOR_YELLOW: c_char = b'3' as c_char;
pub const COLOR_BLUE: c_char = b'4' as c_char;
pub const COLOR_CYAN: c_char = b'5' as c_char;
pub const COLOR_MAGENTA: c_char = b'6' as c_char;
pub const COLOR_WHITE: c_char = b'7' as c_char;

/// ColorIndex macro: extract color index from character
#[inline]
pub fn ColorIndex(c: c_char) -> u32 {
    ((c as u32) - ('0' as u32)) & 7
}

pub const S_COLOR_BLACK: &[u8] = b"^0";
pub const S_COLOR_RED: &[u8] = b"^1";
pub const S_COLOR_GREEN: &[u8] = b"^2";
pub const S_COLOR_YELLOW: &[u8] = b"^3";
pub const S_COLOR_BLUE: &[u8] = b"^4";
pub const S_COLOR_CYAN: &[u8] = b"^5";
pub const S_COLOR_MAGENTA: &[u8] = b"^6";
pub const S_COLOR_WHITE: &[u8] = b"^7";

// extern vec4_t g_color_table[8];

/// MAKERGB macro: set RGB values in a 3-element array
#[inline]
pub fn MAKERGB(v: &mut [f32; 3], r: f32, g: f32, b: f32) {
    v[0] = r;
    v[1] = g;
    v[2] = b;
}

/// MAKERGBA macro: set RGBA values in a 4-element array
#[inline]
pub fn MAKERGBA(v: &mut [f32; 4], r: f32, g: f32, b: f32, a: f32) {
    v[0] = r;
    v[1] = g;
    v[2] = b;
    v[3] = a;
}

/// DEG2RAD macro: convert degrees to radians
#[inline]
pub fn DEG2RAD(a: f32) -> f32 {
    (a * M_PI as f32) / 180.0
}

/// RAD2DEG macro: convert radians to degrees
#[inline]
pub fn RAD2DEG(a: f32) -> f32 {
    (a * 180.0) / M_PI as f32
}

// struct cplane_s forward declaration
// (defined elsewhere in the codebase)

// extern declarations for origin vectors and default matrix
// extern idVec3_t vec3_origin;
// extern vec4_t vec4_origin;
// extern mat3_t axisDefault;

pub const NANMASK: u32 = 255 << 23;

/// IS_NAN macro: check if a float is NaN
#[inline]
pub fn IS_NAN(x: f32) -> bool {
    (x.to_bits() & NANMASK) == NANMASK
}

// Function declarations (C ABI)
extern "C" {
    pub fn Q_fabs(f: f32) -> f32;
    pub fn Q_rsqrt(f: f32) -> f32; // reciprocal square root

    pub fn ClampChar(i: c_int) -> i8;
    pub fn ClampShort(i: c_int) -> i16;

    // this isn't a real cheap function to call!
    // pub fn DirToByte(dir: &idVec3_t) -> c_int;
    // pub fn ByteToDir(b: c_int, dir: &mut vec3_p);

    pub fn Q_log2(val: c_int) -> c_int;

    pub fn Q_rand(seed: *mut c_int) -> c_int;
    pub fn Q_random(seed: *mut c_int) -> f32;
    pub fn Q_crandom(seed: *mut c_int) -> f32;

    pub fn Q_rint(i: f32) -> f32;

    // pub fn vectoangles(value1: &idVec3_t, angles: &mut angles_p);
    // pub fn AnglesToAxis(angles: &angles_c, axis: &mut mat3_p);

    // pub fn AxisCopy(i: &mat3_c, out: &mut mat3_p);
    // pub fn AxisRotated(i: &mat3_c) -> qboolean;

    // pub fn SignbitsForNormal(normal: &idVec3_t) -> c_int;
    // pub fn BoxOnPlaneSide(b: &Bounds, p: *mut cplane_s) -> c_int;

    pub fn AngleMod(a: f32) -> f32;
    pub fn LerpAngle(from: f32, to: f32, frac: f32) -> f32;
    pub fn AngleSubtract(a1: f32, a2: f32) -> f32;
    // pub fn AnglesSubtract(v1: &angles_c, v2: &angles_c, v3: &mut angles_p);

    pub fn AngleNormalize360(angle: f32) -> f32;
    pub fn AngleNormalize180(angle: f32) -> f32;
    pub fn AngleDelta(angle1: f32, angle2: f32) -> f32;

    // pub fn PlaneFromPoints(plane: &mut vec4_t, a: &idVec3_t, b: &idVec3_t, c: &idVec3_t) -> qboolean;
    // pub fn ProjectPointOnPlane(dst: &mut idVec3_t, p: &idVec3_t, normal: &idVec3_t);
    // pub fn RotatePointAroundVector(dst: &mut idVec3_t, dir: &idVec3_t, point: &idVec3_t, degrees: f32);
    // pub fn RotateAroundDirection(axis: &mut mat3_p, yaw: f32);
    // pub fn MakeNormalVectors(forward: &idVec3_t, right: &mut idVec3_t, up: &mut idVec3_t);

    // pub fn PlaneTypeForNormal(normal: &idVec3_t) -> c_int;

    // pub fn MatrixMultiply(in1: &mat3_c, in2: &mat3_c, out: &mut mat3_p);
    // pub fn MatrixInverseMultiply(in1: &mat3_c, in2: &mat3_c, out: &mut mat3_p);
    // pub fn MatrixTransformVector(i: &idVec3_t, matrix: &mat3_c, out: &mut idVec3_t);
    // pub fn MatrixProjectVector(i: &idVec3_t, matrix: &mat3_c, out: &mut idVec3_t);
    // pub fn AngleVectors(angles: &angles_c, forward: &mut idVec3_t, right: &mut idVec3_t, up: &mut idVec3_t);
    // pub fn PerpendicularVector(dst: &mut idVec3_t, src: &idVec3_t);

    // pub fn TriangleArea(a: &idVec3_t, b: &idVec3_t, c: &idVec3_t) -> f32;
}

//=============================================

extern "C" {
    pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32;

    pub fn Com_HashString(fname: *const c_char) -> c_int;

    pub fn Com_SkipPath(pathname: *mut c_char) -> *mut c_char;

    // it is ok for out == in
    pub fn Com_StripExtension(i: *const c_char, out: *mut c_char);

    // "extension" should include the dot: ".map"
    pub fn Com_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char);

    pub fn Com_ParseInfos(
        buf: *const c_char,
        max: c_int,
        infos: *mut [c_char; MAX_INFO_STRING],
    ) -> c_int;
}

pub const FILE_HASH_SIZE: usize = 1024;

/*
=====================================================================================

SCRIPT PARSING

=====================================================================================
*/

// this just controls the comment printing, it doesn't actually load a file
extern "C" {
    pub fn Com_BeginParseSession(filename: *const c_char);
    pub fn Com_EndParseSession();

    pub fn Com_GetCurrentParseLine() -> c_int;

    // Will never return NULL, just empty strings.
    // An empty string will only be returned at end of file.
    // ParseOnLine will return empty if there isn't another token on this line

    // this funny typedef just means a moving pointer into a const char * buffer
    pub fn Com_Parse(data_p: *mut *const c_char) -> *const c_char;
    pub fn Com_ParseOnLine(data_p: *mut *const c_char) -> *const c_char;
    pub fn Com_ParseRestOfLine(data_p: *mut *const c_char) -> *const c_char;

    pub fn Com_UngetToken();

    pub fn Com_MatchToken(buf_p: *mut *const c_char, match_: *const c_char, warning: qboolean);

    pub fn Com_ScriptError(msg: *const c_char, ...);
    pub fn Com_ScriptWarning(msg: *const c_char, ...);

    pub fn Com_SkipBracedSection(program: *mut *const c_char);
    pub fn Com_SkipRestOfLine(data: *mut *const c_char);

    pub fn Com_ParseFloat(buf_p: *mut *const c_char) -> f32;
    pub fn Com_ParseInt(buf_p: *mut *const c_char) -> c_int;

    pub fn Com_Parse1DMatrix(buf_p: *mut *const c_char, x: c_int, m: *mut f32);
    pub fn Com_Parse2DMatrix(buf_p: *mut *const c_char, y: c_int, x: c_int, m: *mut f32);
    pub fn Com_Parse3DMatrix(buf_p: *mut *const c_char, z: c_int, y: c_int, x: c_int, m: *mut f32);
}

//=====================================================================================

extern "C" {
    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
}

// mode parm for FS_FOpenFile
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum fsMode_t {
    FS_READ = 0,
    FS_WRITE = 1,
    FS_APPEND = 2,
    FS_APPEND_SYNC = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum fsOrigin_t {
    FS_SEEK_CUR = 0,
    FS_SEEK_END = 1,
    FS_SEEK_SET = 2,
}

//=============================================

extern "C" {
    pub fn Q_isprint(c: c_int) -> c_int;
    pub fn Q_islower(c: c_int) -> c_int;
    pub fn Q_isupper(c: c_int) -> c_int;
    pub fn Q_isalpha(c: c_int) -> c_int;

    // portable case insensitive compare
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    pub fn Q_strlwr(s1: *mut c_char) -> *mut c_char;
    pub fn Q_strupr(s1: *mut c_char) -> *mut c_char;
    pub fn Q_strrchr(string: *const c_char, c: c_int) -> *mut c_char;

    // buffer size safe library replacements
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn Q_strcat(dest: *mut c_char, size: c_int, src: *const c_char);

    // strlen that discounts Quake color sequences
    pub fn Q_PrintStrlen(string: *const c_char) -> c_int;
    // removes color sequences from string
    pub fn Q_CleanStr(string: *mut c_char) -> *mut c_char;

    pub fn Com_Filter(filter: *const c_char, name: *const c_char, casesensitive: c_int) -> c_int;
    pub fn Com_StringContains(
        str1: *const c_char,
        str2: *const c_char,
        casesensitive: c_int,
    ) -> *const c_char;
}

//=============================================

extern "C" {
    pub fn BigShort(l: i16) -> i16;
    pub fn LittleShort(l: i16) -> i16;
    pub fn BigLong(l: c_int) -> c_int;
    pub fn LittleLong(l: c_int) -> c_int;
    pub fn BigFloat(l: f32) -> f32;
    pub fn LittleFloat(l: f32) -> f32;

    pub fn Swap_Init();
    pub fn va(format: *mut c_char, ...) -> *mut c_char;

    pub fn Com_Error(level: c_int, error: *const c_char, ...);
    pub fn Com_Printf(msg: *const c_char, ...);
    pub fn Com_DPrintf(msg: *const c_char, ...);
}

//=============================================

#[repr(C)]
pub struct growList_t {
    pub frameMemory: qboolean,
    pub currentElements: c_int,
    pub maxElements: c_int, // will reallocate and move when exceeded
    pub elements: *mut *mut c_void,
}

// you don't need to init the growlist if you don't mind it growing and moving
// the list as it expands
extern "C" {
    pub fn Com_InitGrowList(list: *mut growList_t, maxElements: c_int);
    pub fn Com_AddToGrowList(list: *mut growList_t, data: *mut c_void) -> c_int;
    pub fn Com_GrowListElement(list: *const growList_t, index: c_int) -> *mut c_void;
    pub fn Com_IndexForGrowListElement(list: *const growList_t, element: *const c_void) -> c_int;
}

//
// key / value info strings
//
extern "C" {
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char;
    pub fn Info_RemoveKey(s: *mut c_char, key: *const c_char);
    pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char);
    pub fn Info_Validate(s: *const c_char) -> qboolean;
    pub fn Info_NextPair(
        s: *mut *const c_char,
        key: *mut [c_char; MAX_INFO_KEY],
        value: *mut [c_char; MAX_INFO_VALUE],
    );
}

// get cvar defs, collision defs, etc
//#include "../shared/interface.h"

// get key code numbers for events
//#include "../shared/keycodes.h"

// get the polygon winding functions
//#include "../shared/windings.h"

// get the flags class
//#include "../shared/idflags.h"
