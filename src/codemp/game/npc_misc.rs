//! Port of `NPC_misc.c` — the NPC debug-print helpers.
//!
//! The file is just the two variadic debug printers `Debug_Printf` and
//! `Debug_NPCPrintf`. Both are `cv->value < debugLevel` early-out wrappers that
//! pick a color from the debug level, format the message, and emit it through
//! `Com_Printf`.
//!
//! Following the established varargs deviation (see `crate/DEVIATIONS.md`
//! "Deferred: varargs"), the C `char *fmt, ...` tail is taken as a
//! pre-formatted `core::fmt::Arguments` (the caller builds it with
//! `format_args!`), and `printNPC->targetname`'s `%s` uses the `Sz` C-string
//! adapter.
//
// NPC_misc.cpp
//

#![allow(non_snake_case)] // C function names (`Debug_Printf`) kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{level, Com_Printf};
use crate::codemp::game::q_shared::Sz;
use crate::codemp::game::q_shared_h::{
    Q_COLOR_ESCAPE, COLOR_GREEN, COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};
use crate::ffi::types::vmCvar_t;

// b_local.h debug levels.
pub const DEBUG_LEVEL_DETAIL: c_int = 4;
pub const DEBUG_LEVEL_INFO: c_int = 3;
pub const DEBUG_LEVEL_WARNING: c_int = 2;
pub const DEBUG_LEVEL_ERROR: c_int = 1;

// q_shared.h `S_COLOR_*` string constants.
const S_COLOR_RED: &str = "^1";
const S_COLOR_GREEN: &str = "^2";
const S_COLOR_YELLOW: &str = "^3";
const S_COLOR_WHITE: &str = "^7";

/*
Debug_Printf
*/
/// # Safety
/// `cv` must point to a valid [`vmCvar_t`]; `level` must be initialised.
pub unsafe fn Debug_Printf(cv: *const vmCvar_t, debugLevel: c_int, args: core::fmt::Arguments) {
    let color: &str;

    if ((*cv).value as c_int) < debugLevel {
        return;
    }

    if debugLevel == DEBUG_LEVEL_DETAIL {
        color = S_COLOR_WHITE;
    } else if debugLevel == DEBUG_LEVEL_INFO {
        color = S_COLOR_GREEN;
    } else if debugLevel == DEBUG_LEVEL_WARNING {
        color = S_COLOR_YELLOW;
    } else if debugLevel == DEBUG_LEVEL_ERROR {
        color = S_COLOR_RED;
    } else {
        color = S_COLOR_RED;
    }

    // va_start / vsprintf(msg, fmt, argptr) / va_end — `args` is the formatted tail.
    let msg = format!("{}", args);

    Com_Printf(&format!("{}{:5}:{}", color, (*addr_of!(level)).time, msg));
}

/*
Debug_NPCPrintf
*/
/// # Safety
/// `printNPC` must point to a valid [`gentity_t`]; `cv` must point to a valid
/// [`vmCvar_t`]; `level` must be initialised.
pub unsafe fn Debug_NPCPrintf(
    printNPC: *const gentity_t,
    cv: *const vmCvar_t,
    debugLevel: c_int,
    args: core::fmt::Arguments,
) {
    let color: c_int;

    if ((*cv).value as c_int) < debugLevel {
        return;
    }

    //	if ( debugNPCName.string[0] && Q_stricmp( debugNPCName.string, printNPC->targetname) != 0 )
    //	{
    //		return;
    //	}

    if debugLevel == DEBUG_LEVEL_DETAIL {
        color = COLOR_WHITE as c_int;
    } else if debugLevel == DEBUG_LEVEL_INFO {
        color = COLOR_GREEN as c_int;
    } else if debugLevel == DEBUG_LEVEL_WARNING {
        color = COLOR_YELLOW as c_int;
    } else if debugLevel == DEBUG_LEVEL_ERROR {
        color = COLOR_RED as c_int;
    } else {
        color = COLOR_RED as c_int;
    }

    // va_start / vsprintf(msg, fmt, argptr) / va_end — `args` is the formatted tail.
    let msg = format!("{}", args);

    Com_Printf(&format!(
        "{}{}{:5} ({}) {}",
        Q_COLOR_ESCAPE as u8 as char,
        color as u8 as char,
        (*addr_of!(level)).time,
        Sz((*printNPC).targetname),
        msg
    ));
}
