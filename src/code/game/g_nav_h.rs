#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint};

pub type vec3_t = [f32; 3];

pub const WAYPOINT_NONE: c_int = 0;

pub const MAX_RADIUS_CHECK: c_int = 1024;
pub const YAW_ITERATIONS: c_int = 16;

pub static mut navCalculatePaths: bool = false;

pub static mut NAVDEBUG_showNodes: bool = false;
pub static mut NAVDEBUG_showRadius: bool = false;
pub static mut NAVDEBUG_showEdges: bool = false;
pub static mut NAVDEBUG_showTestPath: bool = false;
pub static mut NAVDEBUG_showEnemyPath: bool = false;
pub static mut NAVDEBUG_showCombatPoints: bool = false;
pub static mut NAVDEBUG_showNavGoals: bool = false;
pub static mut NAVDEBUG_showCollision: bool = false;
pub static mut NAVDEBUG_showGrid: bool = false;
pub static mut NAVDEBUG_showNearest: bool = false;
pub static mut NAVDEBUG_showPointLines: bool = false;

pub static mut NAVDEBUG_curGoal: c_int = 0;

extern "C" {
    pub fn CG_DrawNode(origin: *const vec3_t, r#type: c_int);
    pub fn CG_DrawEdge(start: *const vec3_t, end: *const vec3_t, r#type: c_int);
    pub fn CG_DrawRadius(origin: *const vec3_t, radius: c_uint, r#type: c_int);
    pub fn CG_DrawCombatPoint(origin: *const vec3_t, r#type: c_int);
}
