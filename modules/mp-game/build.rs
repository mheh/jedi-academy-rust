//! jampgame build script.
//!
//! The real logic lives once in the workspace-root `build.rs` (MSVC legacy_stdio
//! link + optional `oracle` parity-C compilation); this shim includes it so the
//! source stays single. NOTE: under `--features oracle`, the root script's
//! `oracle_c` path is relative to this crate's manifest dir — that parity build
//! needs the path made workspace-relative (tracked as a follow-up); the default
//! (no-oracle) build is CWD-independent and works as-is.
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../build.rs"));
