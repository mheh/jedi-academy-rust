//! sp-engine — GENERATED module manifest.
//! Mounts the unchanged `src/` pool via #[path]; do not edit by hand.
//! Regenerate with scripts/genmod (see handoffs).

#![allow(non_snake_case)]

#[macro_use]
#[path = "../../../src/macros.rs"]
mod macros;

pub mod code;
pub mod codemp;
pub mod ffi;
#[cfg(feature = "oracle")]
#[path = "../../../src/oracle.rs"]
pub mod oracle;
pub mod trap;

// WIP scaffold: this executable's C `main`/`WinMain` lives in one of the
// mounted platform modules; wiring it to Rust `fn main` is a Phase-2 task.
fn main() {
    unimplemented!("engine entry point not yet wired — see handoffs/workspace-scaffold.md");
}
