// RM_Terrain.h — faithful port of oracle/codemp/RMG/RM_Terrain.h

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::qcommon::cm_landscape_h::*; // CCMLandScape, HEIGHT_RESOLUTION, thandle_t
use crate::codemp::game::q_shared_h::*; // MAX_QPATH, byte, qhandle_t

use core::ffi::{c_char, c_int};

unsafe extern "C" {
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn strlen(s: *const c_char) -> usize;
}

pub const MAX_RANDOM_MODELS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CRandomModel {
    pub mModelName: [c_char; MAX_QPATH],
    pub mFrequency: f32,
    pub mMinScale: f32,
    pub mMaxScale: f32,
}

impl CRandomModel {
    pub unsafe fn GetModel(&self) -> bool {
        strlen(self.mModelName.as_ptr()) != 0
    }
    pub unsafe fn GetModelName(&self) -> *const c_char {
        self.mModelName.as_ptr()
    }
    pub unsafe fn SetModel(&mut self, name: *const c_char) {
        Com_sprintf(
            self.mModelName.as_mut_ptr(),
            MAX_QPATH as c_int,
            b"%s.md3\0".as_ptr() as *const c_char,
            name,
        );
    }
    pub fn GetFrequency(&self) -> f32 {
        self.mFrequency
    }
    pub fn SetFrequency(&mut self, freq: f32) {
        self.mFrequency = freq;
    }
    pub fn GetMinScale(&self) -> f32 {
        self.mMinScale
    }
    pub fn SetMinScale(&mut self, minscale: f32) {
        self.mMinScale = minscale;
    }
    pub fn GetMaxScale(&self) -> f32 {
        self.mMaxScale
    }
    pub fn SetMaxScale(&mut self, maxscale: f32) {
        self.mMaxScale = maxscale;
    }
}

#[repr(C)]
pub struct CCGHeightDetails {
    pub mNumModels: c_int,
    pub mTotalFrequency: c_int,
    pub mModels: [CRandomModel; MAX_RANDOM_MODELS],
}

impl CCGHeightDetails {
    pub fn GetNumModels(&self) -> c_int {
        self.mNumModels
    }
    pub fn GetAverageFrequency(&self) -> c_int {
        self.mTotalFrequency / self.mNumModels
    }
}

// CCGPatch — unused in .cpp but declared in .h
#[repr(C)]
pub struct CCGPatch {
    // private: owner, localowner, common — all opaque
}

#[repr(C)]
pub struct CRMLandScape {
    pub common: *mut CCMLandScape,
    pub mDensityMap: *mut byte,
    pub mModelCount: c_int,
    pub mHeightDetails: [CCGHeightDetails; HEIGHT_RESOLUTION],
}

impl CRMLandScape {
    pub unsafe fn SetCommon(&mut self, landscape: *mut CCMLandScape) {
        self.common = landscape;
    }
    pub unsafe fn GetCommon(&self) -> *const CCMLandScape {
        self.common
    }
    pub unsafe fn GetCommonId(&self) -> thandle_t {
        (*self.common).GetTerrainId()
    }
    pub unsafe fn GetTerxels(&self) -> c_int {
        (*self.common).GetTerxels()
    }
    pub unsafe fn GetRealWidth(&self) -> c_int {
        (*self.common).GetRealWidth()
    }
    pub unsafe fn GetPatchScalarSize(&self) -> f32 {
        (*self.common).GetPatchScalarSize()
    }
    pub unsafe fn GetHeightDetail(&self, height: c_int) -> *const CCGHeightDetails {
        self.mHeightDetails.as_ptr().add(height as usize)
    }
    pub fn ClearModelCount(&mut self) {
        self.mModelCount = 0;
    }
    pub fn GetModelCount(&self) -> c_int {
        self.mModelCount
    }
}

// Free function prototypes (defined in RM_Terrain.cpp / RM_Terrain.rs)
unsafe extern "C" {
    pub fn RM_CreateRandomModels(terrainId: c_int, terrainInfo: *const c_char);
    pub fn RM_InitTerrain();
    pub fn RM_ShutdownTerrain();
}
