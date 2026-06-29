//! Mechanical port of `codemp/qcommon/exe_headers.cpp`.

// This file creates the PCH for the rest of the project to use

// In Rust, the PCH concept is replaced by module exports.
// This module re-exports the header-only declarations from exe_headers_h.
pub use super::exe_headers_h;
