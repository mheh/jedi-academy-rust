//! `cm_randomterrain.h` — random terrain path declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, vec2_t, vec4_t, vec_t};
use crate::codemp::qcommon::cm_terrainmap_h::CCMLandScape;
use core::ffi::{c_char, c_int, c_uint};

// class CPathInfo;

pub const SPLINE_MERGE_SIZE: c_int = 3;
pub const CIRCLE_STAMP_SIZE: usize = 128;

#[repr(C)]
pub struct CPathInfo {
    pub mPoints: *mut vec4_t,
    pub mWork: *mut vec4_t,
    pub mWeights: *mut vec_t,
    pub mNumPoints: c_int,
    pub mMinWidth: f32,
    pub mMaxWidth: f32,
    pub mInc: f32,
    pub mDepth: f32,
    pub mBreadth: f32,
    pub mDeviation: f32,
    pub mCircleStamp: [[byte; CIRCLE_STAMP_SIZE]; CIRCLE_STAMP_SIZE],
}

// C++ member declarations, in class order:
// private: CreateCircle, Stamp
// public: CPathInfo, ~CPathInfo, GetNumPoints, GetPoint, GetWidth, GetInfo, DrawPath

pub const MAX_RANDOM_PATHS: usize = 30;

// Path Creation Flags
pub const PATH_CREATION_CONNECT_FRONT: c_uint = 0x00000001;

#[repr(C)]
pub struct CRandomTerrain {
    pub mLandScape: *mut CCMLandScape,
    pub mWidth: c_int,
    pub mHeight: c_int,
    pub mArea: c_int,
    pub mBorder: c_int,
    pub mGrid: *mut byte,
    pub mPaths: [*mut CPathInfo; MAX_RANDOM_PATHS],
}

// C++ member declarations, in class order:
// public: CRandomTerrain, ~CRandomTerrain, GetLandScape, GetBounds, rand_seed,
// get_rand_seed, flrand, irand, Init, Shutdown, CreatePath, GetPathInfo,
// ParseGenerate, Smooth, Generate, ClearPaths

unsafe extern "C" {
    pub fn RMG_CreateSeed(TextSeed: *mut c_char) -> c_uint;
}

// Referenced by the C++ member declarations above:
// CPathInfo::Stamp(int x, int y, int size, int depth, unsigned char *Data,
//                  int DataWidth, int DataHeight)
// CPathInfo::CPathInfo(CCMLandScape *landscape, int numPoints, float bx,
//                      float by, float ex, float ey, float minWidth,
//                      float maxWidth, float depth, float deviation,
//                      float breadth, CPathInfo *Connected,
//                      unsigned CreationFlags)
// void CPathInfo::GetInfo(float PercentInto, vec4_t Coord, vec4_t Vector)
// void CPathInfo::DrawPath(unsigned char *Data, int DataWidth, int DataHeight)
//
// void CRandomTerrain::Init(class CCMLandScape *landscape, byte *data,
//                           int width, int height)
// bool CRandomTerrain::CreatePath(int PathID, int ConnectedID,
//                                 unsigned CreationFlags, int numPoints,
//                                 float bx, float by, float ex, float ey,
//                                 float minWidth, float maxWidth, float depth,
//                                 float deviation, float breadth)
// bool CRandomTerrain::GetPathInfo(int PathNum, float PercentInto,
//                                  vec2_t Coord, vec2_t Vector)
// void CRandomTerrain::ParseGenerate(const char *GenerateFile)
#[allow(unused)]
type _cm_randomterrain_keep_vec2_t = vec2_t;
