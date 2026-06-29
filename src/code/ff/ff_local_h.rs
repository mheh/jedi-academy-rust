#![allow(non_snake_case)]

pub const FF_ACCESSOR: () = ();
pub const FF_API_VERSION: i32 = 1;

// Better sound synchronization
// This is default value for cvar ff_delay. User may tweak this.
pub const FF_DELAY: &str = "40";
// Default: all channels output to primary device
pub const FF_CHANNEL: &str = "0,0;1,0;2,0;3,0;4,0;5,0";

// Optional system features
pub const FF_PRINT: () = ();
// #ifdef FF_PRINT
pub const FF_CONSOLECOMMAND: () = ();
// #endif
// (end) Optional system features

// Includes from original C code:
// #include "..\game\q_shared.h"	// includes ff_public.h
// #include "..\qcommon\qcommon.h"

pub const FF_MAX_PATH: usize = crate::code::game::q_shared::MAX_QPATH;
