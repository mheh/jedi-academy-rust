//! Mechanical port of `code/client/keys.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};

// ============================================================================
// Type Aliases and Constants
// ============================================================================

/// qboolean - from q_shared.h
pub type qboolean = c_int;

/// word - from q_shared.h
pub type word = u16;

/// fileHandle_t - from q_shared.h
pub type fileHandle_t = c_int;

pub const MAX_EDIT_LINE: usize = 256;
pub const COMMAND_HISTORY: usize = 32;
pub const MAX_KEYS: usize = 320;

// ============================================================================
// Structs
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct qkey_t {
    pub down: qboolean,
    pub repeats: c_int,              // if > 1, it is autorepeating
    pub binding: *mut c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct field_t {
    pub cursor: c_int,
    pub scroll: c_int,
    pub widthInChars: c_int,
    pub buffer: [c_char; MAX_EDIT_LINE],
}

#[repr(C)]
pub struct keyGlobals_s {
    pub historyEditLines: [field_t; COMMAND_HISTORY],
    pub nextHistoryLine: c_int,      // the last line in the history buffer, not masked
    pub historyLine: c_int,          // the line being displayed from history buffer
                                     // will be <= nextHistoryLine
    pub g_consoleField: field_t,
    pub anykeydown: qboolean,
    pub key_overstrikeMode: qboolean,
    pub keyDownCount: c_int,
    pub keys: [qkey_t; MAX_KEYS],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct keyname_t {
    pub upper: word,
    pub lower: word,
    pub name: *mut c_char,
    pub keynum: c_int,
    pub menukey: bool,
}

// ============================================================================
// Extern Declarations
// ============================================================================

extern "C" {
    pub static mut kg: keyGlobals_s;
    pub static mut keynames: [keyname_t; MAX_KEYS];
    pub static mut chatField: field_t;

    pub fn Field_Clear(edit: *mut field_t);
    pub fn Field_KeyDownEvent(edit: *mut field_t, key: c_int);
    pub fn Field_Draw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, showCursor: qboolean);
    pub fn Field_BigDraw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, showCursor: qboolean);

    pub fn Key_WriteBindings(f: fileHandle_t);
    pub fn Key_SetBinding(keynum: c_int, binding: *const c_char);
    pub fn Key_GetBinding(keynum: c_int) -> *mut c_char;
    pub fn Key_IsDown(keynum: c_int) -> qboolean;
    pub fn Key_GetOverstrikeMode() -> qboolean;
    pub fn Key_SetOverstrikeMode(state: qboolean);
    pub fn Key_ClearStates();
}
