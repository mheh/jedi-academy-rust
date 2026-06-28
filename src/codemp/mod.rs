//! Ported `codemp/` game-module sources, laid out to mirror the upstream
//! `refs/raven-jediacademy/codemp/` tree so each Rust file maps 1:1 to its C origin.

pub mod RMG;
pub mod Ratl;
pub mod Ravl;
pub mod Splines;
pub mod botlib;
pub mod cgame;
pub mod client;
pub mod encryption;
pub mod ff;
pub mod game;
pub mod ghoul2;
pub mod goblib;
pub mod icarus;

#[path = "jpeg-6/mod.rs"]
pub mod jpeg_6;

pub mod mp3code;
pub mod namespace_begin_h;
pub mod namespace_end_h;
pub mod null;
pub mod png;
pub mod qcommon;
pub mod renderer;
pub mod server;
pub mod smartheap;
pub mod strings;
pub mod ui;
pub mod unix;
pub mod win32;
pub mod x_botlib;
pub mod x_exe;
pub mod x_jk2cgame;
pub mod x_jk2game;
pub mod x_ui;
pub mod zlib32;
