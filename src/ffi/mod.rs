//! The C ABI boundary between the JKA engine and this game module.
//!
//! Numbering is transcribed verbatim from the **original Raven JKA**
//! `refs/raven-jediacademy/codemp/game/g_public.h`. Do not renumber: these integers are
//! the wire protocol the engine speaks.

pub mod exports;
pub mod game_export;
pub mod game_import;
pub mod syscalls;
pub mod types;

pub use game_export::GameExport;
pub use game_import::GameImport;
