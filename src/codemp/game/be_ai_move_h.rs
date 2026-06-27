//! Bot movement-AI types from `be_ai_move.h`.
//!
//! `bot_initmove_t` and `bot_moveresult_t` cross the VM ABI by pointer in the
//! `trap_BotInitMoveState`/`trap_BotMoveToGoal` syscalls, so their layout is
//! load-bearing. Faithful 1:1 with the original-JKA
//! `refs/raven-jediacademy/codemp/game/be_ai_move.h`. (`bot_avoidspot_t` is defined in the
//! header but not referenced by any trap signature — it is carried here for fidelity.)

#![allow(non_camel_case_types)]

use super::q_shared_h::vec3_t;
use core::ffi::c_int;

/// `bot_initmove_t` (be_ai_move.h) — initializes the movement state. The ored
/// moveflags `MFL_ONGROUND`/`MFL_TELEPORTED`/`MFL_WATERJUMP` come from the playerstate.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct bot_initmove_t {
    /// origin of the bot
    pub origin: vec3_t,
    /// velocity of the bot
    pub velocity: vec3_t,
    /// view offset
    pub viewoffset: vec3_t,
    /// entity number of the bot
    pub entitynum: c_int,
    /// client number of the bot
    pub client: c_int,
    /// time the bot thinks
    pub thinktime: f32,
    /// presencetype of the bot
    pub presencetype: c_int,
    /// view angles of the bot
    pub viewangles: vec3_t,
    /// values ored to the movement flags
    pub or_moveflags: c_int,
}

/// `bot_moveresult_t` (be_ai_move.h) — result of a movement step. NOTE: the
/// `ideal_viewangles` are only valid if `MOVERESULT_MOVEMENTVIEW` is set.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct bot_moveresult_t {
    /// true if movement failed all together
    pub failure: c_int,
    /// failure or blocked type
    pub r#type: c_int,
    /// true if blocked by an entity
    pub blocked: c_int,
    /// entity blocking the bot
    pub blockentity: c_int,
    /// last executed travel type
    pub traveltype: c_int,
    /// result flags
    pub flags: c_int,
    /// weapon used for movement
    pub weapon: c_int,
    /// movement direction
    pub movedir: vec3_t,
    /// ideal viewangles for the movement
    pub ideal_viewangles: vec3_t,
}

/// `bot_avoidspot_t` (be_ai_move.h) — an avoid spot. Defined in the header but not
/// referenced by any trap signature; carried for fidelity.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct bot_avoidspot_t {
    pub origin: vec3_t,
    pub radius: f32,
    pub r#type: c_int,
}
