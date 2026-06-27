//! Shared C types crossing the engine↔game boundary.
//!
//! These mirror `refs/raven-jediacademy/codemp/game/q_shared.h`. Note they use fixed-width
//! `int`/`float` fields (not `intptr_t`), so their layout is identical on 32- and
//! 64-bit builds.

#![allow(non_camel_case_types)] // keep C type names for 1:1 traceability

use core::ffi::{c_char, c_float, c_int};

/// `qboolean` — engine boolean, passed as a C `int`.
pub type qboolean = c_int;
pub const QFALSE: c_int = 0;
pub const QTRUE: c_int = 1;

/// `fileHandle_t`.
pub type fileHandle_t = c_int;

/// `cvarHandle_t`.
pub type cvarHandle_t = c_int;

/// `MAX_CVAR_VALUE_STRING` from q_shared.h.
pub const MAX_CVAR_VALUE_STRING: usize = 256;

/// `vmCvar_t` — the module-side mirror of an engine cvar. Register it with
/// [`crate::trap::Cvar_Register`] and refresh it with [`crate::trap::Cvar_Update`].
#[repr(C)]
#[derive(Clone, Copy)]
pub struct vmCvar_t {
    pub handle: cvarHandle_t,
    pub modificationCount: c_int,
    pub value: c_float,
    pub integer: c_int,
    pub string: [c_char; MAX_CVAR_VALUE_STRING],
}

impl vmCvar_t {
    /// A zero-initialised cvar handle, ready to pass to `Cvar_Register`.
    pub const fn zeroed() -> Self {
        vmCvar_t {
            handle: 0,
            modificationCount: 0,
            value: 0.0,
            integer: 0,
            string: [0; MAX_CVAR_VALUE_STRING],
        }
    }
}

impl Default for vmCvar_t {
    fn default() -> Self {
        Self::zeroed()
    }
}
