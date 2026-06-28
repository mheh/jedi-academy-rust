#![allow(non_snake_case)]

use core::ffi::c_char;

// Forward declarations for types referenced in this header
pub struct CRMInstance;
pub struct CGPGroup;
pub struct CRMInstanceFile;
pub struct CRandomTerrain;

// MAX_QPATH - maximum file path length from common Quake headers
const MAX_QPATH: usize = 64;

#[repr(C)]
pub struct CRMBSPInstance {
    mBsp: [c_char; MAX_QPATH],
    mAngleVariance: f32,
    mBaseAngle: f32,
    mAngleDiff: f32,

    mHoleRadius: f32,
}

impl CRMBSPInstance {
    pub fn new(instance: *mut CGPGroup, instFile: &mut CRMInstanceFile) -> Self {
        todo!()
    }

    pub fn GetPreviewColor(&self) -> i32 {
        (255 << 24) + 255
    }

    pub fn GetHoleRadius(&self) -> f32 {
        self.mHoleRadius
    }

    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: i32) -> bool {
        todo!()
    }

    pub fn GetModelName(&self) -> *const c_char {
        self.mBsp.as_ptr()
    }

    pub fn GetAngleDiff(&self) -> f32 {
        self.mAngleDiff
    }

    pub fn GetAngularType(&self) -> bool {
        self.mAngleDiff != 0.0f
    }
}
