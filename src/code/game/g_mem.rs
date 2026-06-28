//
// g_mem.rs
//
// leave this line at the top for all g_xxxx files...
// (C header: g_headers.h)
// (C header: g_local.h)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_void, c_char};

// ============================================================================
// Type Definitions
// ============================================================================

/// Forward reference to cvar_t from q_shared.h (used as pointer).
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
}

// ============================================================================
// Constants
// ============================================================================

// TAG_G_ALLOC - memory tag for game allocations
// Value from oracle/code/game/g_mem.cpp context
const TAG_G_ALLOC: c_int = 0; // PORTING: placeholder - actual value from tag enum

// qfalse from q_shared.h
const QFALSE: c_int = 0;

// ============================================================================
// External Functions and Globals
// ============================================================================

extern "C" {
    /// gi.Printf - Printf function from game import
    pub fn gi_Printf(fmt: *const c_char, ...) -> c_int;

    /// gi.Malloc - Memory allocation function from game import
    pub fn gi_Malloc(size: c_int, tag: c_int, clear: c_int) -> *mut c_void;

    /// gi.cvar - Get/create cvar function from game import
    pub fn gi_cvar(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
}

// ============================================================================
// Global Variables
// ============================================================================

/*#define POOLSIZE	(2 * 1024 * 1024)

static char		memoryPool[POOLSIZE];
*/
static mut allocPoint: c_int = 0;
static mut g_debugalloc: *mut cvar_t = core::ptr::null_mut();

// ============================================================================
// Functions
// ============================================================================

pub unsafe fn G_Alloc(size: c_int) -> *mut c_void {
    if (*g_debugalloc).integer != 0 {
        gi_Printf(b"G_Alloc of %i bytes\n\0".as_ptr() as *const c_char, size);
    }

    allocPoint += size;

    gi_Malloc(size, TAG_G_ALLOC, QFALSE)
}

pub unsafe fn G_InitMemory() {
    allocPoint = 0;
    g_debugalloc = gi_cvar(
        b"g_debugalloc\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0,
    );
}

pub unsafe fn Svcmd_GameMem_f() {
    gi_Printf(
        b"Game memory status: %i allocated\n\0".as_ptr() as *const c_char,
        allocPoint,
    );
}
