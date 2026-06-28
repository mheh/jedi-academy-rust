//! `g_nav.h` — shared NPC navigation declarations.
//!
//! This file is shared by the exe nav code.
//! If you modify it without recompiling the exe with new code, there could be issues.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::q_shared_h::{qboolean, trace_t, vec3_t};
use core::ffi::{c_char, c_int};

pub const WAYPOINT_NONE: c_int = -1;

pub const MAX_STORED_WAYPOINTS: usize = 512; //???
pub const MAX_WAYPOINT_REACHED_DIST_SQUARED: c_int = 1024; //32 squared
pub const MAX_COLL_AVOID_DIST: c_int = 128;
pub const NAVGOAL_USE_RADIUS: c_int = 16384; //Used to force the waypoint_navgoals with a manually set radius to actually do a DistanceSquared check, not just bounds overlap

pub const MIN_STOP_DIST: c_int = 64;
pub const MIN_BLOCKED_SPEECH_TIME: c_int = 4000;
pub const MIN_DOOR_BLOCK_DIST: c_int = 16;
pub const MIN_DOOR_BLOCK_DIST_SQR: c_int = MIN_DOOR_BLOCK_DIST * MIN_DOOR_BLOCK_DIST;
pub const SHOVE_SPEED: c_int = 200;
pub const SHOVE_LIFT: c_int = 10;
pub const MAX_RADIUS_CHECK: c_int = 1024;
pub const YAW_ITERATIONS: c_int = 16;

// This is probably wrong - VVFIXME
// Some kind of wacky code sharing going on here, but we need these things
// in g_navnew.c -- which is now C++ code in the GAME on Xbox, so the
// original test fails.

//rww - Rest of defines here are also shared in exe, do not modify.
pub const __NEWCOLLECT: c_int = 1;

pub const _HARD_CONNECT: c_int = 1;

//Node flags
pub const NF_ANY: c_int = 0;
//pub const NF_CLEAR_LOS: c_int = 0x00000001;
pub const NF_CLEAR_PATH: c_int = 0x00000002;
pub const NF_RECALC: c_int = 0x00000004;

//Edge flags
pub const EFLAG_NONE: c_int = 0;
pub const EFLAG_BLOCKED: c_int = 0x00000001;
pub const EFLAG_FAILED: c_int = 0x00000002;

//Miscellaneous defines
pub const NODE_NONE: c_int = -1;
pub const NAV_HEADER_ID: c_int =
    ((b'J' as c_int) << 24) | ((b'N' as c_int) << 16) | ((b'V' as c_int) << 8) | (b'5' as c_int);
pub const NODE_HEADER_ID: c_int =
    ((b'N' as c_int) << 24) | ((b'O' as c_int) << 16) | ((b'D' as c_int) << 8) | (b'E' as c_int);

//this stuff is local and can be modified, don't even show it to the engine.
pub use crate::codemp::game::g_nav::{
    navCalculatePaths, NAVDEBUG_curGoal, NAVDEBUG_showCollision, NAVDEBUG_showCombatPoints,
    NAVDEBUG_showEdges, NAVDEBUG_showEnemyPath, NAVDEBUG_showNavGoals, NAVDEBUG_showNodes,
    NAVDEBUG_showRadius, NAVDEBUG_showTestPath,
};

unsafe extern "C" {
    pub fn NAV_Shutdown();
    pub fn NAV_CalculatePaths(filename: *const c_char, checksum: c_int);
    pub fn NAV_CalculateSquadPaths(filename: *const c_char, checksum: c_int);

    pub fn NAV_ShowDebugInfo();

    pub fn NAV_GetNearestNode(self_: *mut gentity_t, lastNode: c_int) -> c_int;
    pub fn NAV_TestBestNode(
        self_: *mut gentity_t,
        startID: c_int,
        endID: c_int,
        failEdge: qboolean,
    ) -> c_int;

    pub fn NPC_GetMoveDirection(out: *mut vec3_t, distance: *mut f32) -> qboolean;
    pub fn NPC_MoveToGoalExt(point: *mut vec3_t);
    pub fn NAV_FindPlayerWaypoint(clNum: c_int);
    pub fn NAV_CheckAhead(
        self_: *mut gentity_t,
        end: *mut vec3_t,
        trace: *mut trace_t,
        clipmask: c_int,
    ) -> qboolean;
}
