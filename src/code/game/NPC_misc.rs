//
// NPC_misc.cpp
//

// leave this line at the top for all NPC_xxxx.cpp files...
// (C header: g_headers.h)
// (C header: b_local.h)
// (C header: q_shared.h)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char, c_void};
use libc;

// ============================================================================
// Types
// ============================================================================

/// Forward reference to cvar_t from q_shared.h (only used as pointer).
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

/// C structure for game entities.
/// Note: This is a partial definition - only includes fields up to targetname.
/// Full definition would be much larger but we only need targetname access here.
#[repr(C)]
pub struct gentity_t {
    // entityState_t s; (forward declaration - large, we skip it)
    // Using byte padding to skip to targetname field
    _entityState: [c_char; 512],  // entityState_t s (approximate size)
    pub client: *mut c_void,       // struct gclient_s *client
    pub inuse: c_int,              // qboolean
    pub linked: c_int,             // qboolean
    pub svFlags: c_int,
    pub bmodel: c_int,             // qboolean
    pub mins: [f32; 3],            // vec3_t
    pub maxs: [f32; 3],            // vec3_t
    pub contents: c_int,
    pub absmin: [f32; 3],          // vec3_t
    pub absmax: [f32; 3],          // vec3_t
    pub currentOrigin: [f32; 3],   // vec3_t
    pub currentAngles: [f32; 3],   // vec3_t
    pub owner: *mut gentity_t,
    // CGhoul2Info_v ghoul2; (forward declaration)
    _ghoul2: [c_char; 576],        // CGhoul2Info_v (approximate size)
    pub modelScale: [f32; 3],      // vec3_t
    // Large gap here in the actual structure
    // Skipping to the essential entity fields section
    pub classname: *mut c_char,
    pub spawnflags: c_int,
    pub flags: c_int,
    pub model: *mut c_char,
    pub model2: *mut c_char,
    pub freetime: c_int,
    pub eventTime: c_int,
    pub freeAfterEvent: c_int,     // qboolean
    pub physicsBounce: f32,
    pub clipmask: c_int,
    pub speed: f32,
    pub resultspeed: f32,
    pub lastMoveTime: c_int,
    pub movedir: [f32; 3],         // vec3_t
    pub lastOrigin: [f32; 3],      // vec3_t
    pub lastAngles: [f32; 3],      // vec3_t
    pub mass: f32,
    pub lastImpact: c_int,
    pub watertype: c_int,
    pub waterlevel: c_int,
    pub wupdate: i16,
    pub prev_waterlevel: i16,
    pub angle: f32,
    pub target: *mut c_char,
    pub target2: *mut c_char,
    pub target3: *mut c_char,
    pub target4: *mut c_char,
    pub targetJump: *mut c_char,
    pub targetname: *mut c_char,   // The field we need
}

/// Partial stub for level structure (only level.time is accessed)
#[repr(C)]
pub struct level_s {
    pub time: c_int,
    // ... other fields not needed for this file
}

/// Partial stub for game import structure (only gi.Printf is called)
#[repr(C)]
pub struct gameImport_t {
    _padding: [*mut c_void; 0],
}

// ============================================================================
// Constants
// ============================================================================

const DEBUG_LEVEL_DETAIL: c_int = 4;
const DEBUG_LEVEL_INFO: c_int = 3;
const DEBUG_LEVEL_WARNING: c_int = 2;
const DEBUG_LEVEL_ERROR: c_int = 1;

// Color escape codes (string form for S_COLOR_*)
const S_COLOR_WHITE: &[u8] = b"^7";
const S_COLOR_GREEN: &[u8] = b"^2";
const S_COLOR_YELLOW: &[u8] = b"^3";
const S_COLOR_RED: &[u8] = b"^1";

// Color codes (char form for COLOR_*)
const COLOR_WHITE: c_char = b'7' as c_char;
const COLOR_GREEN: c_char = b'2' as c_char;
const COLOR_YELLOW: c_char = b'3' as c_char;
const COLOR_RED: c_char = b'1' as c_char;

const Q_COLOR_ESCAPE: c_char = b'^' as c_char;

// ============================================================================
// External Functions and Globals
// ============================================================================

extern "C" {
    pub static mut debugNPCName: *mut cvar_t;

    pub static level: level_s;

    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Note: gi.Printf is typically called as a function pointer from the gameImport_t struct,
    // but here we declare it as a direct extern function for simplicity
    pub fn gi_Printf(fmt: *const c_char, ...) -> c_int;
}

// ============================================================================
// Functions
// ============================================================================

/*
Debug_Printf
*/
#[no_mangle]
pub unsafe extern "C" fn Debug_Printf(cv: *mut cvar_t, debugLevel: c_int, fmt: *const c_char, mut args: ...) {
    let color: *const c_char;
    let mut msg: [c_char; 1024] = [0; 1024];

    if (*cv).value < debugLevel as f32 {
        return;
    }

    if debugLevel == DEBUG_LEVEL_DETAIL {
        color = S_COLOR_WHITE.as_ptr() as *const c_char;
    } else if debugLevel == DEBUG_LEVEL_INFO {
        color = S_COLOR_GREEN.as_ptr() as *const c_char;
    } else if debugLevel == DEBUG_LEVEL_WARNING {
        color = S_COLOR_YELLOW.as_ptr() as *const c_char;
    } else if debugLevel == DEBUG_LEVEL_ERROR {
        color = S_COLOR_RED.as_ptr() as *const c_char;
    } else {
        color = S_COLOR_RED.as_ptr() as *const c_char;
    }

    libc::vsprintf(msg.as_mut_ptr(), fmt, args.as_va_list());

    gi_Printf(
        "%s%5i:%s\0".as_ptr() as *const c_char,
        color,
        level.time,
        msg.as_ptr(),
    );
}

/*
Debug_NPCPrintf
*/
#[no_mangle]
pub unsafe extern "C" fn Debug_NPCPrintf(
    printNPC: *mut gentity_t,
    cv: *mut cvar_t,
    debugLevel: c_int,
    fmt: *const c_char,
    mut args: ...,
) {
    let color: c_char;
    let mut msg: [c_char; 1024] = [0; 1024];

    if (*cv).value < debugLevel as f32 {
        return;
    }

    if !(*debugNPCName).string.is_null() && (*(*debugNPCName).string) != 0 && Q_stricmp((*debugNPCName).string, (*printNPC).targetname) != 0 {
        return;
    }

    if debugLevel == DEBUG_LEVEL_DETAIL {
        color = COLOR_WHITE;
    } else if debugLevel == DEBUG_LEVEL_INFO {
        color = COLOR_GREEN;
    } else if debugLevel == DEBUG_LEVEL_WARNING {
        color = COLOR_YELLOW;
    } else if debugLevel == DEBUG_LEVEL_ERROR {
        color = COLOR_RED;
    } else {
        color = COLOR_RED;
    }

    libc::vsprintf(msg.as_mut_ptr(), fmt, args.as_va_list());

    gi_Printf(
        "%c%c%5i (%s) %s\0".as_ptr() as *const c_char,
        Q_COLOR_ESCAPE,
        color,
        level.time,
        (*printNPC).targetname,
        msg.as_ptr(),
    );
}
