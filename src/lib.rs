//! jedi-academy-rust — Rust port of the *Star Wars Jedi Knight: Jedi Academy*
//! multiplayer server game module (`jampgame`).
//!
//! This crate compiles to the shared library the JKA engine loads as its
//! server-side game VM. It targets the **original Raven JKA** engine↔game ABI
//! (the `GAME_*` / `G_*` enum numbering in `refs/raven-jediacademy/codemp/game/g_public.h`),
//! while using the wider `intptr_t` calling convention introduced by OpenJK so the
//! same source builds correctly for **both 32-bit and 64-bit** engines:
//!
//! * On a 32-bit target `isize == i32`, matching original Raven JKA exactly.
//! * On a 64-bit target `isize == i64`, matching OpenJK's `intptr_t` widening.
//!
//! See the project-root `CLAUDE.md` (one level above this `crate/` submodule) for the
//! full ABI/reference notes.
//!
//! ## Boundary
//! * [`ffi::exports`] — the two C symbols the engine calls: `dllEntry`, `vmMain`.
//! * [`ffi::syscalls`] — the single choke point for game→engine calls.
//! * [`trap`] — safe wrappers over those syscalls (the `trap_*` functions).
//! * [`codemp::game`] — the ported game-module sources (mirrors the upstream
//!   `codemp/game/` tree: `q_shared.rs`, `q_math.rs`, `g_main.rs`, …).

#![allow(non_snake_case)] // FFI exports & trap_* wrappers intentionally mirror C names

#[macro_use]
mod macros;

pub mod codemp;
pub mod ffi;
pub mod trap;

/// Test-only oracle: FFI bindings to the extracted original Raven C. Present only
/// under the `oracle` feature (off by default); see `build.rs` and `oracle/`.
#[cfg(feature = "oracle")]
pub mod oracle;

// Re-export the engine entry points at the crate root so they are part of the
// cdylib's exported symbol table.
pub use ffi::exports::{dllEntry, vmMain};
