//! Stubbed mirror of Raven's original `code/` source tree.
//!
//! These modules are intentionally empty until each original source directory is
//! ported. Keeping the top-level layout in place gives future work a stable home
//! while the current crate continues to ship the multiplayer `jampgame` module.

#[path = "0_compiled_first/mod.rs"]
pub mod compiled_first;

pub mod RMG;
pub mod Ragl;
pub mod Ratl;
pub mod Ravl;
pub mod Rufl;
pub mod SHDebug;
pub mod cgame;
pub mod client;
pub mod ff;
pub mod game;
pub mod ghoul2;
pub mod goblib;
pub mod icarus;

#[path = "jpeg-6/mod.rs"]
pub mod jpeg_6;

pub mod mac;
pub mod mp3code;
pub mod null;
pub mod png;
pub mod qcommon;
pub mod renderer;
pub mod server;
pub mod smartheap;
pub mod ui;
pub mod unix;
pub mod win32;
pub mod x_exe;
pub mod x_game;
pub mod zlib32;
