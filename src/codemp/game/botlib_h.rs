//! Bot AI library interface types from `botlib.h`.
//!
//! JKA's MP game module replaced the Quake3 bot library with its own waypoint system
//! (`ai_main.c`/`ai_wpnav.c`/`ai_util.c`), so almost all of `botlib.h` — the
//! `botlib_import_t`/`aas_export_t`/`ea_export_t`/`ai_export_t` function-pointer tables
//! and the `bot_entitystate_t`/`bsp_trace_t` structs — is **unreferenced** by the game
//! module and is therefore omitted. Only the two things the module actually uses are
//! ported here: the `ACTION_*` action flags and `bot_input_t` (the bot's per-frame input,
//! converted to a `usercmd_t` by `BotInputToUserCommand`). Faithful 1:1 with the PC
//! `botlib.h` (`refs/raven-jediacademy`).
//!
//! Xbox→PC: PC uncomments the 20 chat-state function pointers (`BotAllocChatState` …
//! `BotSetChatName`) inside the `ai_export_t` vtable. That vtable is one of the omitted
//! engine-side tables (the module reaches the engine's bot-chat services through the
//! `G_BOTLIB_AI_*_CHAT_STATE` trap numbers, already repinned to PC in ce440b49 / wired in
//! `ffi/game_import.rs`), so the uncomment is a **no-op for this port** — nothing in the
//! module dereferences the struct.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

use super::q_shared_h::vec3_t;

// print types (botlib.h:40-43), passed to the (disabled) `BotAI_Print` hook.
pub const PRT_MESSAGE: c_int = 1;
pub const PRT_WARNING: c_int = 2;
pub const PRT_ERROR: c_int = 3;
pub const PRT_FATAL: c_int = 4;

// action flags (botlib.h:65-82)
pub const ACTION_ATTACK: c_int = 0x0000001;
pub const ACTION_USE: c_int = 0x0000002;
pub const ACTION_RESPAWN: c_int = 0x0000008;
pub const ACTION_JUMP: c_int = 0x0000010;
pub const ACTION_MOVEUP: c_int = 0x0000020;
pub const ACTION_CROUCH: c_int = 0x0000080;
pub const ACTION_MOVEDOWN: c_int = 0x0000100;
pub const ACTION_MOVEFORWARD: c_int = 0x0000200;
pub const ACTION_MOVEBACK: c_int = 0x0000800;
pub const ACTION_MOVELEFT: c_int = 0x0001000;
pub const ACTION_MOVERIGHT: c_int = 0x0002000;
pub const ACTION_DELAYEDJUMP: c_int = 0x0008000;
pub const ACTION_TALK: c_int = 0x0010000;
pub const ACTION_GESTURE: c_int = 0x0020000;
pub const ACTION_WALK: c_int = 0x0080000;
pub const ACTION_FORCEPOWER: c_int = 0x0100000;
pub const ACTION_ALT_ATTACK: c_int = 0x0200000;
// The CTF/teamplay flags below are commented out in the C header (botlib.h:83-90) and are
// only referenced inside an `#if 0` block in `BotInputToUserCommand` (ai_main.c), so they
// are intentionally left undefined here:
//   ACTION_AFFIRMATIVE 0x0100000  ACTION_NEGATIVE 0x0200000  ACTION_GETFLAG  0x0800000
//   ACTION_GUARDBASE   0x1000000  ACTION_PATROL   0x2000000  ACTION_FOLLOWME 0x8000000

/// `bot_input_t` (botlib.h:92-101) — the bot input, converted to a `usercmd_t`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct bot_input_t {
    /// time since last output (in seconds)
    pub thinktime: f32,
    /// movement direction
    pub dir: vec3_t,
    /// speed in the range [0, 400]
    pub speed: f32,
    /// the view angles
    pub viewangles: vec3_t,
    /// one of the `ACTION_?` flags
    pub actionflags: c_int,
    /// weapon to use
    pub weapon: c_int,
}
