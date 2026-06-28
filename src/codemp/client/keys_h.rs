//! Mechanical port of `codemp/client/keys.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::codemp::ui::keycodes_h::MAX_KEYS;
use crate::ffi::types::{fileHandle_t, qboolean};

// Unported shared typedef dependency from the original headers.
pub type word = u16;

#[repr(C)]
pub struct qkey_t {
    pub down: qboolean,
    pub repeats: c_int, // if > 1, it is autorepeating
    pub binding: *mut c_char,
}

pub const MAX_EDIT_LINE: c_int = 256;
pub const COMMAND_HISTORY: c_int = 32;

#[repr(C)]
pub struct field_t {
    pub cursor: c_int,
    pub scroll: c_int,
    pub widthInChars: c_int,
    pub buffer: [c_char; MAX_EDIT_LINE as usize],
}

#[repr(C)]
pub struct keyGlobals_t {
    pub historyEditLines: [field_t; COMMAND_HISTORY as usize],

    pub nextHistoryLine: c_int, // the last line in the history buffer, not masked
    pub historyLine: c_int,     // the line being displayed from history buffer
    // will be <= nextHistoryLine
    pub g_consoleField: field_t,

    pub anykeydown: qboolean,
    pub key_overstrikeMode: qboolean,
    pub keyDownCount: c_int,

    pub keys: [qkey_t; MAX_KEYS as usize],
}

#[repr(C)]
pub struct keyname_t {
    pub upper: word,
    pub lower: word,
    pub name: *mut c_char,
    pub keynum: c_int,
    pub menukey: bool,
}

unsafe extern "C" {
    pub static mut kg: keyGlobals_t;
    pub static mut keynames: [keyname_t; MAX_KEYS as usize];

    pub fn Field_Clear(edit: *mut field_t);
    pub fn Field_KeyDownEvent(edit: *mut field_t, key: c_int);
    pub fn Field_CharEvent(edit: *mut field_t, ch: c_int);
    pub fn Field_Draw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, showCursor: qboolean);
    pub fn Field_BigDraw(
        edit: *mut field_t,
        x: c_int,
        y: c_int,
        width: c_int,
        showCursor: qboolean,
    );

    pub static mut chatField: field_t;
    pub static mut chat_team: qboolean;
    pub static mut chat_playerNum: c_int;

    pub fn Key_WriteBindings(f: fileHandle_t);
    pub fn Key_SetBinding(keynum: c_int, binding: *const c_char);
    pub fn Key_GetBinding(keynum: c_int) -> *mut c_char;
    pub fn Key_IsDown(keynum: c_int) -> qboolean;
    pub fn Key_GetOverstrikeMode() -> qboolean;
    pub fn Key_SetOverstrikeMode(state: qboolean);
    pub fn Key_ClearStates();
    pub fn Key_GetKey(binding: *const c_char) -> c_int;
}
