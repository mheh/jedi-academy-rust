//! Mechanical port of `codemp/RMG/RM_Area.cpp`.
//!
//! Implements the CRMArea and CRMAreaManager classes for managing areas in random missions.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;
use std::ptr::null_mut;

// Type definitions (expected from common headers)
pub type vec_t = f32;
pub type vec3_t = [f32; 3];
pub type vec3pair_t = [[f32; 3]; 2];

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================

/// Stub for unported `class CRMManager` (RM_Manager.h).
pub struct CRMManager {
    _opaque: [u8; 0],
}

impl CRMManager {
    /// GetLandScape() method stub
    pub fn GetLandScape(&self) -> *mut CCMLandScape {
        null_mut()
    }
}

/// Stub for unported `class CCMLandScape` (cm_landscape.h).
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

impl CCMLandScape {
    /// GetBounds() method stub - returns pointer to bounds array
    pub fn GetBounds(&self) -> *const vec3pair_t {
        std::ptr::null()
    }

    /// irand() method stub - random integer
    pub fn irand(&self, _min: c_int, _max: c_int) -> c_int {
        0
    }
}

// ============================================================================
// extern "C" functions from oracle (q_math.c)
// ============================================================================

extern "C" {
    /// `void _VectorCopy( const vec3_t in, vec3_t out )`.
    fn _VectorCopy(in_: *const f32, out: *mut f32);

    /// `void _VectorSubtract( const vec3_t a, const vec3_t b, vec3_t out )`.
    fn _VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);

    /// `vec_t VectorNormalize( vec3_t v )` — normalizes in place, returns old length.
    fn VectorNormalize(v: *mut f32) -> f32;

    /// `void _VectorMA( const vec3_t veca, float scale, const vec3_t vecb, vec3_t vecc )`.
    fn _VectorMA(veca: *const f32, scale: f32, vecb: *const f32, vecc: *mut f32);

    /// `vec_t _DotProduct( const vec3_t v1, const vec3_t v2 )`.
    fn _DotProduct(v1: *const f32, v2: *const f32) -> f32;

    /// `vec_t Distance( const vec3_t p1, const vec3_t p2 )`.
    fn Distance(p1: *const f32, p2: *const f32) -> f32;

    /// `void CrossProduct( const vec3_t v1, const vec3_t v2, vec3_t cross )`.
    fn CrossProduct(v1: *const f32, v2: *const f32, cross: *mut f32);

    /// `vec_t VectorLength( const vec3_t v )`.
    fn VectorLength(v: *const f32) -> f32;

    /// C standard math function: `float atan2(float y, float x)`.
    fn atan2(y: f32, x: f32) -> f32;
}

// Convenience macro-like wrapper functions for vector operations
#[inline]
fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    unsafe {
        _VectorCopy(src.as_ptr(), dst.as_mut_ptr());
    }
}

#[inline]
fn VectorSubtract(a: &vec3_t, b: &vec3_t, out: &mut vec3_t) {
    unsafe {
        _VectorSubtract(a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
    }
}

#[inline]
fn VectorNormalize_wrapper(v: &mut vec3_t) -> f32 {
    unsafe { VectorNormalize(v.as_mut_ptr()) }
}

#[inline]
fn VectorMA(veca: &vec3_t, scale: f32, vecb: &vec3_t, vecc: &mut vec3_t) {
    unsafe {
        _VectorMA(veca.as_ptr(), scale, vecb.as_ptr(), vecc.as_mut_ptr());
    }
}

#[inline]
fn DotProduct(v1: &vec3_t, v2: &vec3_t) -> f32 {
    unsafe { _DotProduct(v1.as_ptr(), v2.as_ptr()) }
}

#[inline]
fn Distance_wrapper(p1: &vec3_t, p2: &vec3_t) -> f32 {
    unsafe { Distance(p1.as_ptr(), p2.as_ptr()) }
}

#[inline]
fn CrossProduct_wrapper(v1: &vec3_t, v2: &vec3_t, cross: &mut vec3_t) {
    unsafe {
        CrossProduct(v1.as_ptr(), v2.as_ptr(), cross.as_mut_ptr());
    }
}

#[inline]
fn VectorLength_wrapper(v: &vec3_t) -> f32 {
    unsafe { VectorLength(v.as_ptr()) }
}

// Inline maximum function - matches C macro
#[inline]
fn maximum(x: f32, y: f32) -> f32 {
    if x > y { x } else { y }
}

// ============================================================================
// Global: TheRandomMissionManager
// ============================================================================

extern "C" {
    pub static mut TheRandomMissionManager: *mut CRMManager;
}

// ============================================================================
// CRMArea class
// ============================================================================

/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Area.cpp
 *
 ************************************************************************************************/

/// Represents an area in the random mission generation system.
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
    /************************************************************************************************
     * CRMArea::CRMArea
     *	constructor
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new(
        spacingRadius: f32,
        paddingSize: f32,
        confineRadius: f32,
        confineOrigin: vec3_t,
        lookAtOrigin: vec3_t,
        flatten: bool,
        symmetric: c_int,
    ) -> Self {
        CRMArea {
            mMoveCount: 0,
            mAngle: 0.0,
            mCollision: true,
            mConfineRadius: confineRadius,
            mPaddingSize: paddingSize,
            mSpacingRadius: spacingRadius,
            mFlatten: flatten,
            mLookAt: true,
            mLockOrigin: false,
            mSymmetric: symmetric,
            mRadius: spacingRadius,
            mOrigin: [0.0; 3],
            mConfineOrigin: confineOrigin,
            mLookAtOrigin: lookAtOrigin,
        }
    }

    /************************************************************************************************
     * CRMArea::LookAt
     *	Angle the area towards the given point
     *
     * inputs:
     *  lookat - the origin to look at
     *
     * return:
     *	the angle in radians that was calculated
     *
     ************************************************************************************************/
    pub fn LookAt(&mut self, lookat: vec3_t) -> f32 {
        if self.mLookAt {
            // this area orients itself towards a point
            let mut a: vec3_t = [0.0; 3];

            VectorCopy(&lookat, &mut self.mLookAtOrigin);
            VectorSubtract(&lookat, &self.mOrigin, &mut a);

            self.mAngle = unsafe { atan2(a[1], a[0]) };
        }

        self.mAngle
    }

    /************************************************************************************************
     * CRMArea::Mirror
     *	Mirrors the area to the other side of the map.  This includes mirroring the confine origin
     *  and lookat origin
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn Mirror(&mut self) {
        self.mOrigin[0] = -self.mOrigin[0];
        self.mOrigin[1] = -self.mOrigin[1];

        self.mConfineOrigin[0] = -self.mConfineOrigin[0];
        self.mConfineOrigin[1] = -self.mConfineOrigin[1];

        self.mLookAtOrigin[0] = -self.mLookAtOrigin[0];
        self.mLookAtOrigin[1] = -self.mLookAtOrigin[1];
    }

    // Getter methods
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

    pub fn GetOrigin(&self) -> *mut vec_t {
        // Note: In the original C++, GetOrigin() returns a mutable pointer
        // (not marked const), so we match that behavior here.
        self.mOrigin.as_ptr() as *mut vec_t
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

    // Setter methods
    pub fn SetOrigin(&mut self, origin: vec3_t) {
        VectorCopy(&origin, &mut self.mOrigin);
    }

    pub fn SetAngle(&mut self, angle: f32) {
        self.mAngle = angle;
    }

    pub fn SetSymmetric(&mut self, sym: c_int) {
        self.mSymmetric = sym;
    }

    pub fn SetRadius(&mut self, r: f32) {
        self.mRadius = r;
    }

    pub fn EnableCollision(&mut self, e: bool) {
        self.mCollision = e;
    }

    pub fn EnableLookAt(&mut self, la: bool) {
        self.mLookAt = la;
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

    pub fn IsCollisionEnabled(&self) -> bool {
        self.mCollision
    }

    pub fn IsFlattened(&self) -> bool {
        self.mFlatten
    }
}

// ============================================================================
// CRMAreaManager class
// ============================================================================

pub type rmAreaVector_t = Vec<*mut CRMArea>;

/// Manages a collection of areas for random mission generation.
pub struct CRMAreaManager {
    mAreas: rmAreaVector_t,
    mMins: vec3_t,
    mMaxs: vec3_t,
    mWidth: f32,
    mHeight: f32,
}

impl CRMAreaManager {
    /************************************************************************************************
     * CRMAreaManager::CRMAreaManager
     *	constructor
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new(mins: vec3_t, maxs: vec3_t) -> Self {
        let mut mMins = [0.0; 3];
        let mut mMaxs = [0.0; 3];
        VectorCopy(&mins, &mut mMins);
        VectorCopy(&maxs, &mut mMaxs);

        CRMAreaManager {
            mAreas: Vec::new(),
            mMins,
            mMaxs,
            mWidth: mMaxs[0] - mMins[0],
            mHeight: mMaxs[1] - mMins[1],
        }
    }

    /************************************************************************************************
     * CRMAreaManager::~CRMAreaManager
     *	Removes all managed areas
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn destroy(&mut self) {
        let mut i: usize = self.mAreas.len();
        while i > 0 {
            i -= 1;
            let area = self.mAreas[i];
            unsafe {
                let _ = Box::from_raw(area);
            }
        }
        self.mAreas.clear();
    }

    /************************************************************************************************
     * CRMAreaManager::MoveArea
     *	Moves an area within the area manager thus shifting any other areas as needed
     *
     * inputs:
     *  area   - area to be moved
     *  origin - new origin to attempt to move to
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn MoveArea(&mut self, movedArea: *mut CRMArea, mut origin: vec3_t) {
        let mut index: c_int;
        let size: c_int;

        // Increment the addcount (this is for infinite protection)
        unsafe {
            (*movedArea).AddMoveCount();
        }

        // Infinite recursion prevention
        if unsafe { (*movedArea).GetMoveCount() } > 250 {
            //		assert ( 0 );
            unsafe {
                (*movedArea).EnableCollision(false);
            }
            return;
        }

        // First set the area's origin, This may cause it to be in collision with
        // another area but that will get fixed later
        unsafe {
            (*movedArea).SetOrigin(origin);
        }

        // when symmetric we want to ensure that no instances end up on the "other" side of the imaginary diaganol that cuts the map in two
        // mSymmetric tells us which side of the map is legal
        if unsafe { (*movedArea).GetSymmetric() } != 0 {
            let landscape = unsafe { (*TheRandomMissionManager).GetLandScape() };
            if !landscape.is_null() {
                let bounds = unsafe { (*landscape).GetBounds() };
                if !bounds.is_null() {
                    let bounds_ref = unsafe { &*bounds };

                    let mut point: vec3_t = [0.0; 3];
                    let mut dir: vec3_t = [0.0; 3];
                    let mut tang: vec3_t = [0.0; 3];
                    let mut len: f32;

                    // Read the current origin safely
                    let current_origin = unsafe {
                        let ptr = (*movedArea).GetOrigin() as *mut [f32; 3];
                        if !ptr.is_null() {
                            std::ptr::read(ptr)
                        } else {
                            [0.0; 3]
                        }
                    };

                    VectorSubtract(&current_origin, &bounds_ref[0], &mut point);
                    VectorSubtract(&bounds_ref[1], &bounds_ref[0], &mut dir);
                    VectorNormalize_wrapper(&mut dir);

                    dir[2] = 0.0;
                    point[2] = 0.0;
                    VectorMA(&bounds_ref[0], DotProduct(&point, &dir), &dir, &mut tang);
                    VectorSubtract(&current_origin, &tang, &mut dir);

                    dir[2] = 0.0;
                    len = VectorNormalize_wrapper(&mut dir);

                    if len < unsafe { (*movedArea).GetRadius() } {
                        if unsafe { (*movedArea).GetLockOrigin() } {
                            unsafe {
                                (*movedArea).EnableCollision(false);
                            }
                            return;
                        }

                        let rand_val = unsafe {
                            (*landscape).irand(10, (*movedArea).GetSpacingRadius() as c_int)
                        };
                        VectorMA(
                            &point,
                            (unsafe { (*movedArea).GetSpacingRadius() } - len)
                                + rand_val as f32,
                            &dir,
                            &mut point,
                        );
                        origin[0] = point[0] + bounds_ref[0][0];
                        origin[1] = point[1] + bounds_ref[0][1];
                        unsafe {
                            (*movedArea).SetOrigin(origin);
                        }
                    }

                    let sym = unsafe { (*movedArea).GetSymmetric() };
                    match sym {
                        1 => {
                            // SYMMETRY_TOPLEFT
                            if origin[1] > origin[0] {
                                unsafe {
                                    (*movedArea).Mirror();
                                }
                            }
                        }

                        2 => {
                            // SYMMETRY_BOTTOMRIGHT
                            if origin[1] < origin[0] {
                                unsafe {
                                    (*movedArea).Mirror();
                                }
                            }
                        }

                        _ => {
                            // unknown symmetry type
                            // assert ( 0 );
                        }
                    }
                }
            }
        }

        // Confine to area unless we are being pushed back by the same guy who pushed us last time (infinite loop)
        if unsafe { (*movedArea).GetConfineRadius() } != 0.0 {
            if unsafe { (*movedArea).GetMoveCount() } < 25 {
                let mut cdiff: vec3_t = [0.0; 3];
                let mut cdist: f32;

                let current_origin = unsafe {
                    let ptr = (*movedArea).GetOrigin() as *mut [f32; 3];
                    if !ptr.is_null() {
                        std::ptr::read(ptr)
                    } else {
                        [0.0; 3]
                    }
                };

                let confine_origin = unsafe {
                    let ptr = (*movedArea).GetConfineOrigin() as *const [f32; 3];
                    if !ptr.is_null() {
                        std::ptr::read(ptr)
                    } else {
                        [0.0; 3]
                    }
                };

                VectorSubtract(&current_origin, &confine_origin, &mut cdiff);
                cdiff[2] = 0.0;
                cdist = VectorLength_wrapper(&cdiff);

                if cdist + unsafe { (*movedArea).GetSpacingRadius() } > unsafe { (*movedArea).GetConfineRadius() } {
                    cdist = unsafe { (*movedArea).GetConfineRadius() - (*movedArea).GetSpacingRadius() };
                    VectorNormalize_wrapper(&mut cdiff);

                    let mut new_origin = [0.0; 3];
                    VectorMA(&confine_origin, cdist, &cdiff, &mut new_origin);
                    unsafe {
                        (*movedArea).SetOrigin(new_origin);
                    }
                }
            }
        }

        // See if it fell off the world in the x direction
        unsafe {
            let current_origin_ptr = (*movedArea).GetOrigin() as *mut [f32; 3];
            let current_origin = std::ptr::read(current_origin_ptr);
            if current_origin[0] + (*movedArea).GetSpacingRadius() > self.mMaxs[0] {
                let landscape = (*TheRandomMissionManager).GetLandScape();
                let rand_val = if !landscape.is_null() {
                    (*landscape).irand(10, 200)
                } else {
                    100
                };
                let mut new_origin = current_origin;
                new_origin[0] =
                    self.mMaxs[0] - (*movedArea).GetSpacingRadius() - rand_val as f32;
                (*movedArea).SetOrigin(new_origin);
            } else if current_origin[0] - (*movedArea).GetSpacingRadius() < self.mMins[0] {
                let landscape = (*TheRandomMissionManager).GetLandScape();
                let rand_val = if !landscape.is_null() {
                    (*landscape).irand(10, 200)
                } else {
                    100
                };
                let mut new_origin = current_origin;
                new_origin[0] =
                    self.mMins[0] + (*movedArea).GetSpacingRadius() + rand_val as f32;
                (*movedArea).SetOrigin(new_origin);
            }
        }

        // See if it fell off the world in the y direction
        unsafe {
            let current_origin_ptr = (*movedArea).GetOrigin() as *mut [f32; 3];
            let current_origin = std::ptr::read(current_origin_ptr);
            if current_origin[1] + (*movedArea).GetSpacingRadius() > self.mMaxs[1] {
                let landscape = (*TheRandomMissionManager).GetLandScape();
                let rand_val = if !landscape.is_null() {
                    (*landscape).irand(10, 200)
                } else {
                    100
                };
                let mut new_origin = current_origin;
                new_origin[1] =
                    self.mMaxs[1] - (*movedArea).GetSpacingRadius() - rand_val as f32;
                (*movedArea).SetOrigin(new_origin);
            } else if current_origin[1] - (*movedArea).GetSpacingRadius() < self.mMins[1] {
                let landscape = (*TheRandomMissionManager).GetLandScape();
                let rand_val = if !landscape.is_null() {
                    (*landscape).irand(10, 200)
                } else {
                    100
                };
                let mut new_origin = current_origin;
                new_origin[1] =
                    self.mMins[1] + (*movedArea).GetSpacingRadius() + rand_val as f32;
                (*movedArea).SetOrigin(new_origin);
            }
        }

        // Look at what we need to look at
        unsafe {
            let lookat_ptr = (*movedArea).GetLookAtOrigin() as *const [f32; 3];
            let lookat_origin = if !lookat_ptr.is_null() {
                std::ptr::read(lookat_ptr)
            } else {
                [0.0; 3]
            };
            (*movedArea).LookAt(lookat_origin);
        }

        // Dont collide against things that have no collision
        //	if ( !movedArea->IsCollisionEnabled ( ) )
        //	{
        //		return;
        //	}

        // See if its colliding
        index = 0;
        size = self.mAreas.len() as c_int;
        while index < size {
            let area = self.mAreas[index as usize];
            let mut diff: vec3_t = [0.0; 3];
            let mut newOrigin: vec3_t = [0.0; 3];
            let mut dist: f32;
            let mut targetdist: f32;

            // Skip the one that was moved in the first place
            if area == movedArea {
                index += 1;
                continue;
            }

            if unsafe { (*area).GetLockOrigin() && (*movedArea).GetLockOrigin() } {
                index += 1;
                continue;
            }

            // Dont collide against things that have no collision
            if !unsafe { (*area).IsCollisionEnabled() } {
                index += 1;
                continue;
            }

            // Grab the distance between the two
            // only want the horizontal distance -- dmv
            //dist		= Distance ( movedArea->GetOrigin ( ), area->GetOrigin ( ));
            let mut maOrigin: vec3_t = unsafe {
                let ptr = (*movedArea).GetOrigin() as *mut [f32; 3];
                if !ptr.is_null() {
                    std::ptr::read(ptr)
                } else {
                    [0.0; 3]
                }
            };
            let mut aOrigin: vec3_t = unsafe {
                let ptr = (*area).GetOrigin() as *mut [f32; 3];
                if !ptr.is_null() {
                    std::ptr::read(ptr)
                } else {
                    [0.0; 3]
                }
            };
            maOrigin[2] = 0.0;
            aOrigin[2] = 0.0;
            dist = Distance_wrapper(&maOrigin, &aOrigin);
            targetdist = unsafe {
                (*movedArea).GetSpacingRadius()
                    + (*area).GetSpacingRadius()
                    + maximum((*movedArea).GetPaddingSize(), (*area).GetPaddingSize())
            };

            if dist == 0.0 {
                let landscape = unsafe { (*TheRandomMissionManager).GetLandScape() };
                let rand1 = if !landscape.is_null() {
                    unsafe { (*landscape).irand(0, 99) }
                } else {
                    50
                };
                let rand2 = if !landscape.is_null() {
                    unsafe { (*landscape).irand(0, 99) }
                } else {
                    50
                };

                unsafe {
                    let mut area_origin = std::ptr::read((*area).GetOrigin() as *mut [f32; 3]);
                    area_origin[0] += 50.0 * (rand1 as f32) / 100.0;
                    area_origin[1] += 50.0 * (rand2 as f32) / 100.0;
                    (*area).SetOrigin(area_origin);
                }

                aOrigin = unsafe {
                    let ptr = (*area).GetOrigin() as *mut [f32; 3];
                    if !ptr.is_null() {
                        std::ptr::read(ptr)
                    } else {
                        [0.0; 3]
                    }
                };
                aOrigin[2] = 0.0;

                dist = Distance_wrapper(&maOrigin, &aOrigin);
            }

            // Are they are enough apart?
            if dist >= targetdist {
                index += 1;
                continue;
            }

            // Dont move a step if locked
            if unsafe { (*area).GetLockOrigin() } {
                unsafe {
                    let area_origin = std::ptr::read((*area).GetOrigin() as *mut [f32; 3]);
                    self.MoveArea(area, area_origin);
                }
                index += 1;
                continue;
            }

            // we got a collision, move the guy we hit
            let area_origin = unsafe {
                std::ptr::read((*area).GetOrigin() as *mut [f32; 3])
            };
            let moved_origin = unsafe {
                std::ptr::read((*movedArea).GetOrigin() as *mut [f32; 3])
            };
            VectorSubtract(&area_origin, &moved_origin, &mut diff);
            diff[2] = 0.0;
            VectorNormalize_wrapper(&mut diff);

            // Push by the difference in the distance and no-collide radius
            VectorMA(&area_origin, targetdist - dist + 1.0, &diff, &mut newOrigin);

            // Move the area now
            self.MoveArea(area, newOrigin);

            index += 1;
        }
    }

    /************************************************************************************************
     * CRMAreaManager::CreateArea
     *	Creates an area and adds it to the list of managed areas
     *
     * inputs:
     *  none
     *
     * return:
     *	a pointer to the newly added area class
     *
     ************************************************************************************************/
    pub fn CreateArea(
        &mut self,
        origin: vec3_t,
        spacingRadius: f32,
        spacingLine: c_int,
        paddingSize: f32,
        confineRadius: f32,
        confineOrigin: vec3_t,
        lookAtOrigin: vec3_t,
        flatten: bool,
        collide: bool,
        lockorigin: bool,
        symmetric: c_int,
    ) -> *mut CRMArea {
        let area = Box::new(CRMArea::new(
            spacingRadius,
            paddingSize,
            confineRadius,
            confineOrigin,
            lookAtOrigin,
            flatten,
            symmetric,
        ));
        let area_ptr = Box::into_raw(area);

        if lockorigin || spacingLine != 0 {
            unsafe {
                (*area_ptr).LockOrigin();
            }
        }

        if origin[0] != lookAtOrigin[0] || origin[1] != lookAtOrigin[1] {
            unsafe {
                (*area_ptr).EnableLookAt(true);
            }
        }

        // First add the area to the list
        self.mAreas.push(area_ptr);

        unsafe {
            (*area_ptr).EnableCollision(collide);
        }

        // Set the real radius which is used for center line detection
        if spacingLine != 0 {
            unsafe {
                (*area_ptr).SetRadius(spacingRadius + ((spacingLine - 1) as f32) * spacingRadius);
            }
        }

        // Now move the area around
        self.MoveArea(area_ptr, origin);

        if origin[0] != lookAtOrigin[0] || origin[1] != lookAtOrigin[1] {
            let mut i: c_int;
            let mut linedir: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];
            let up: vec3_t = [0.0, 0.0, 1.0];

            VectorSubtract(&lookAtOrigin, &origin, &mut dir);
            VectorNormalize_wrapper(&mut dir);
            dir[2] = 0.0;
            CrossProduct_wrapper(&dir, &up, &mut linedir);

            i = 0;
            while i < spacingLine - 1 {
                let mut linearea: *mut CRMArea;
                let mut lineorigin: vec3_t = [0.0; 3];

                linearea = Box::into_raw(Box::new(CRMArea::new(
                    spacingRadius,
                    paddingSize,
                    0.0,
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    false,
                    symmetric,
                )));
                unsafe {
                    (*linearea).LockOrigin();
                    (*linearea).EnableCollision(collide);
                }

                VectorMA(&origin, spacingRadius + (spacingRadius * 2.0 * i as f32), &linedir, &mut lineorigin);
                self.mAreas.push(linearea);
                self.MoveArea(linearea, lineorigin);

                linearea = Box::into_raw(Box::new(CRMArea::new(
                    spacingRadius,
                    paddingSize,
                    0.0,
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    false,
                    symmetric,
                )));
                unsafe {
                    (*linearea).LockOrigin();
                    (*linearea).EnableCollision(collide);
                }

                VectorMA(&origin, -spacingRadius - (spacingRadius * 2.0 * i as f32), &linedir, &mut lineorigin);
                self.mAreas.push(linearea);
                self.MoveArea(linearea, lineorigin);

                i += 1;
            }
        }

        // Return it for convienience
        area_ptr
    }

    /************************************************************************************************
     * CRMAreaManager::EnumArea
     *	Allows for enumeration through the area list. If an invalid index is given then NULL will
     *  be returned;
     *
     * inputs:
     *  index - current enumeration index
     *
     * return:
     *	requested area class pointer or NULL if the index was invalid
     *
     ************************************************************************************************/
    pub fn EnumArea(&self, index: c_int) -> *mut CRMArea {
        // This isnt an assertion case because there is no size method for
        // the area manager so the areas are enumerated until NULL is returned.
        if index < 0 || index >= self.mAreas.len() as c_int {
            return null_mut();
        }

        self.mAreas[index as usize]
    }
}

impl Drop for CRMAreaManager {
    fn drop(&mut self) {
        self.destroy();
    }
}
