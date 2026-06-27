//! Bot goal-AI types from `be_ai_goal.h`.
//!
//! `bot_goal_t` crosses the VM ABI by pointer in the `trap_Bot*Goal*` syscalls, so its
//! layout is load-bearing. Faithful 1:1 with the original-JKA
//! `refs/raven-jediacademy/codemp/game/be_ai_goal.h`.

#![allow(non_camel_case_types)]

use super::q_shared_h::vec3_t;
use core::ffi::c_int;

pub const MAX_AVOIDGOALS: usize = 256;
pub const MAX_GOALSTACK: usize = 8;

pub const GFL_NONE: c_int = 0;
pub const GFL_ITEM: c_int = 1;
pub const GFL_ROAM: c_int = 2;
pub const GFL_DROPPED: c_int = 4;

/// `bot_goal_t` (be_ai_goal.h) — a bot goal: where it is, its area/entity, and flags.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct bot_goal_t {
    /// origin of the goal
    pub origin: vec3_t,
    /// area number of the goal
    pub areanum: c_int,
    /// mins of the goal
    pub mins: vec3_t,
    /// maxs of the goal
    pub maxs: vec3_t,
    /// number of the goal entity
    pub entitynum: c_int,
    /// goal number
    pub number: c_int,
    /// goal flags
    pub flags: c_int,
    /// item information
    pub iteminfo: c_int,
}
