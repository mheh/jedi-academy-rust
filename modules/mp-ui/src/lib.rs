//! mp-ui — GENERATED module manifest.
//! Mounts the unchanged `src/` pool via #[path]; do not edit by hand.
//! Regenerate with scripts/genmod (see handoffs).

#![allow(non_snake_case)]

#[macro_use]
#[path = "../../../src/macros.rs"]
mod macros;

pub mod codemp;
pub mod ffi;
#[cfg(feature = "oracle")]
#[path = "../../../src/oracle.rs"]
pub mod oracle;
pub mod trap;
