//! `cm_terrainmap.h` — random terrain minimap declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, vec3_t};
use core::ffi::{c_char, c_int};

pub const TM_WIDTH: usize = 512;
pub const TM_HEIGHT: usize = 512;
pub const TM_BORDER: usize = 16;
pub const TM_REAL_WIDTH: usize = TM_WIDTH - TM_BORDER - TM_BORDER;
pub const TM_REAL_HEIGHT: usize = TM_HEIGHT - TM_BORDER - TM_BORDER;

// Opaque stub for the unported terrain landscape class.
#[repr(C)]
pub struct CCMLandScape {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CTerrainMap {
    pub mImage: [[[byte; 4]; TM_WIDTH]; TM_HEIGHT], // image to output
    pub mBufImage: [[[byte; 4]; TM_WIDTH]; TM_HEIGHT], // src data for image, color and bump

    pub mSymBld: *mut byte,
    pub mSymBldWidth: c_int,
    pub mSymBldHeight: c_int,

    pub mSymStart: *mut byte,
    pub mSymStartWidth: c_int,
    pub mSymStartHeight: c_int,

    pub mSymEnd: *mut byte,
    pub mSymEndWidth: c_int,
    pub mSymEndHeight: c_int,

    pub mSymObjective: *mut byte,
    pub mSymObjectiveWidth: c_int,
    pub mSymObjectiveHeight: c_int,

    pub mLandscape: *mut CCMLandScape,
}

// C++ member declarations, in class order:
// private: ApplyBackground, ApplyHeightmap
// public: CTerrainMap, ~CTerrainMap, ConvertPos, AddBuilding, AddStart, AddEnd,
// AddObjective, AddNPC, AddWallRect, AddNode, AddPlayer, Upload, SaveImageToDisk

pub const SIDE_NONE: c_int = 0;
pub const SIDE_BLUE: c_int = 1;
pub const SIDE_RED: c_int = 2;

unsafe extern "C" {
    pub fn CM_TM_Create(landscape: *mut CCMLandScape);
    pub fn CM_TM_Free();
    pub fn CM_TM_AddStart(x: c_int, y: c_int, side: c_int);
    pub fn CM_TM_AddEnd(x: c_int, y: c_int, side: c_int);
    pub fn CM_TM_AddObjective(x: c_int, y: c_int, side: c_int);
    pub fn CM_TM_AddNPC(x: c_int, y: c_int, friendly: bool);
    pub fn CM_TM_AddWallRect(x: c_int, y: c_int, side: c_int);
    pub fn CM_TM_AddNode(x: c_int, y: c_int);
    pub fn CM_TM_AddBuilding(x: c_int, y: c_int, side: c_int);
    pub fn CM_TM_Upload(player_origin: *mut vec3_t, player_angles: *mut vec3_t);
    pub fn CM_TM_SaveImageToDisk(
        terrainName: *const c_char,
        missionName: *const c_char,
        seed: *const c_char,
    );
    pub fn CM_TM_ConvertPosition(x: *mut c_int, y: *mut c_int, Width: c_int, Height: c_int);
}
