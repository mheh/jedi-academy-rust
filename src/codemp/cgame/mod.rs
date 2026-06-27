//! Ported `codemp/cgame/` sources. This is the JKA *client game* tree; in the
//! MP-only server port only the pieces the server links are pulled across (e.g.
//! the `animTable` string table, the data companion to `game/anims.rs`). The
//! directory mirrors the upstream `refs/raven-jediacademy/codemp/cgame/` layout so each
//! Rust file maps 1:1 to its C origin.

pub mod animtable;
