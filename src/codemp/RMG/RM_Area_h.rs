/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Area.h
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_int;

// Type definitions (expected from common headers)
pub type vec_t = f32;
pub type vec3_t = [f32; 3];

// External C function
extern "C" {
    fn VectorCopy(src: *const vec_t, dst: *mut vec_t);
}

pub struct CRMArea {
    mPaddingSize: f32,
    mSpacingRadius: f32,
    mConfineRadius: f32,
    mRadius: f32,
    mAngle: f32,
    mMoveCount: c_int,
    mOrigin: vec3_t,
    mConfineOrigin: vec3_t,
    mLookAtOrigin: vec3_t,
    mCollision: bool,
    mFlatten: bool,
    mLookAt: bool,
    mLockOrigin: bool,
    mSymmetric: c_int,
}

impl CRMArea {
    pub fn new(
        spacing: f32,
        padding: f32,
        confine: f32,
        confineOrigin: vec3_t,
        lookAtOrigin: vec3_t,
        flatten: bool,
        symmetric: c_int,
    ) -> Self {
        CRMArea {
            mPaddingSize: padding,
            mSpacingRadius: spacing,
            mConfineRadius: confine,
            mRadius: 0.0,
            mAngle: 0.0,
            mMoveCount: 0,
            mOrigin: [0.0; 3],
            mConfineOrigin: confineOrigin,
            mLookAtOrigin: lookAtOrigin,
            mCollision: false,
            mFlatten: flatten,
            mLookAt: false,
            mLockOrigin: false,
            mSymmetric: symmetric,
        }
    }

    pub fn Mirror(&mut self) {
        // implementation from RM_Area.cpp
    }

    pub fn SetOrigin(&mut self, origin: vec3_t) {
        unsafe {
            VectorCopy(origin.as_ptr(), self.mOrigin.as_mut_ptr());
        }
    }

    pub fn SetAngle(&mut self, angle: f32) {
        self.mAngle = angle;
    }

    pub fn SetSymmetric(&mut self, sym: c_int) {
        self.mSymmetric = sym;
    }

    pub fn EnableCollision(&mut self, e: bool) {
        self.mCollision = e;
    }

    pub fn EnableLookAt(&mut self, la: bool) {
        self.mLookAt = la;
    }

    pub fn LookAt(&mut self, lookat: vec3_t) -> f32 {
        // implementation from RM_Area.cpp
        0.0
    }

    pub fn LockOrigin(&mut self) {
        self.mLockOrigin = true;
    }

    pub fn AddMoveCount(&mut self) {
        self.mMoveCount += 1;
    }

    pub fn ClearMoveCount(&mut self) {
        self.mMoveCount = 0;
    }

    pub fn GetPaddingSize(&self) -> f32 {
        self.mPaddingSize
    }

    pub fn GetSpacingRadius(&self) -> f32 {
        self.mSpacingRadius
    }

    pub fn GetRadius(&self) -> f32 {
        self.mRadius
    }

    pub fn GetConfineRadius(&self) -> f32 {
        self.mConfineRadius
    }

    pub fn GetAngle(&self) -> f32 {
        self.mAngle
    }

    pub fn GetMoveCount(&self) -> c_int {
        self.mMoveCount
    }

    pub fn GetOrigin(&self) -> *const vec_t {
        self.mOrigin.as_ptr()
    }

    pub fn GetConfineOrigin(&self) -> *const vec_t {
        self.mConfineOrigin.as_ptr()
    }

    pub fn GetLookAtOrigin(&self) -> *const vec_t {
        self.mLookAtOrigin.as_ptr()
    }

    pub fn GetLookAt(&self) -> bool {
        self.mLookAt
    }

    pub fn GetLockOrigin(&self) -> bool {
        self.mLockOrigin
    }

    pub fn GetSymmetric(&self) -> c_int {
        self.mSymmetric
    }

    pub fn SetRadius(&mut self, r: f32) {
        self.mRadius = r;
    }

    pub fn IsCollisionEnabled(&self) -> bool {
        self.mCollision
    }

    pub fn IsFlattened(&self) -> bool {
        self.mFlatten
    }
}

pub type rmAreaVector_t = Vec<*mut CRMArea>;

pub struct CRMAreaManager {
    mAreas: rmAreaVector_t,
    mMins: vec3_t,
    mMaxs: vec3_t,
    mWidth: f32,
    mHeight: f32,
}

impl CRMAreaManager {
    pub fn new(mins: vec3_t, maxs: vec3_t) -> Self {
        CRMAreaManager {
            mAreas: Vec::new(),
            mMins: mins,
            mMaxs: maxs,
            mWidth: 0.0,
            mHeight: 0.0,
        }
    }

    pub fn CreateArea(
        &mut self,
        origin: vec3_t,
        spacing: f32,
        spacingline: c_int,
        padding: f32,
        confine: f32,
        confineOrigin: vec3_t,
        lookAtOrigin: vec3_t,
        flatten: bool,
        collide: bool,
        lockorigin: bool,
        symmetric: c_int,
    ) -> *mut CRMArea {
        // implementation from RM_Area.cpp
        std::ptr::null_mut()
    }

    pub fn MoveArea(&mut self, area: *mut CRMArea, origin: vec3_t) {
        // implementation from RM_Area.cpp
    }

    pub fn EnumArea(&self, index: c_int) -> *mut CRMArea {
        // implementation from RM_Area.cpp
        std::ptr::null_mut()
    }

    // void		CreateMap		( void );
}

impl Drop for CRMAreaManager {
    fn drop(&mut self) {
        // Destructor cleanup
    }
}
