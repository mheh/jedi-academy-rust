/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Area.h
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_int;

// Type aliases for C compatibility
pub type vec_t = f32;
pub type vec3_t = [f32; 3];

// Forward declaration and helper for VectorCopy (external function)
// In the original code, VectorCopy is a macro that copies one vec3_t to another
fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

#[repr(C)]
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
            mOrigin: [0.0, 0.0, 0.0],
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
        // Stub: Mirror implementation from C++ not included in header
    }

    pub fn SetOrigin(&mut self, origin: vec3_t) {
        VectorCopy(&origin, &mut self.mOrigin);
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
        // Stub: LookAt implementation from C++ not included in header
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
        &self.mOrigin[0] as *const vec_t
    }

    pub fn GetConfineOrigin(&self) -> *const vec_t {
        &self.mConfineOrigin[0] as *const vec_t
    }

    pub fn GetLookAtOrigin(&self) -> *const vec_t {
        &self.mLookAtOrigin[0] as *const vec_t
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

pub type rmAreaVector_t = Vec<Box<CRMArea>>;

#[repr(C)]
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
        // Stub: CreateArea implementation from C++ not included in header
        core::ptr::null_mut()
    }

    pub fn MoveArea(&mut self, area: *mut CRMArea, origin: vec3_t) {
        // Stub: MoveArea implementation from C++ not included in header
    }

    pub fn EnumArea(&self, index: c_int) -> *mut CRMArea {
        // Stub: EnumArea implementation from C++ not included in header
        core::ptr::null_mut()
    }

    //	void		CreateMap		( void );
}
