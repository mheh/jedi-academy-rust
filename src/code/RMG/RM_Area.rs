/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Area.cpp
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_int;
use std::ptr;

mod types {
    use core::ffi::c_int;

    pub type vec_t = f32;
    pub type vec3_t = [f32; 3];

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum symmetry_t {
        SYMMETRY_NONE = 0,
        SYMMETRY_TOPLEFT = 1,
        SYMMETRY_BOTTOMRIGHT = 2,
    }
}

use types::*;

// External C functions from the engine
extern "C" {
    fn atan2(y: f32, x: f32) -> f32;
    fn sqrt(x: f32) -> f32;
}

// Vector operation helpers
#[inline]
fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

#[inline]
fn VectorSubtract(a: &vec3_t, b: &vec3_t, result: &mut vec3_t) {
    result[0] = a[0] - b[0];
    result[1] = a[1] - b[1];
    result[2] = a[2] - b[2];
}

#[inline]
fn VectorLength(v: &vec3_t) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn VectorNormalize(v: &mut vec3_t) -> f32 {
    let len = VectorLength(v);
    if len != 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
    len
}

#[inline]
fn DotProduct(a: &vec3_t, b: &vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn VectorMA(v1: &vec3_t, scale: f32, v2: &vec3_t, result: &mut vec3_t) {
    result[0] = v1[0] + scale * v2[0];
    result[1] = v1[1] + scale * v2[1];
    result[2] = v1[2] + scale * v2[2];
}

#[inline]
fn VectorClear(v: &mut vec3_t) {
    v[0] = 0.0;
    v[1] = 0.0;
    v[2] = 0.0;
}

#[inline]
fn CrossProduct(a: &vec3_t, b: &vec3_t, result: &mut vec3_t) {
    result[0] = a[1] * b[2] - a[2] * b[1];
    result[1] = a[2] * b[0] - a[0] * b[2];
    result[2] = a[0] * b[1] - a[1] * b[0];
}

#[inline]
fn Distance(a: &vec3_t, b: &vec3_t) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn maximum(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

// External landscape functions (stubs for external dependencies)
extern "C" {
    type Landscape;

    fn GetRandomMissionLandscape() -> *mut Landscape;
    fn GetLandscapeBounds(landscape: *mut Landscape) -> *const [vec3_t; 2];
    fn GetLandscapeRandomInt(landscape: *mut Landscape, min: c_int, max: c_int) -> c_int;
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
        confineOrigin: &vec3_t,
        lookAtOrigin: &vec3_t,
        flatten: bool,
        symmetric: c_int,
    ) -> Self {
        let mut area = CRMArea {
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
            mOrigin: [0.0, 0.0, 0.0],
            mConfineOrigin: [0.0, 0.0, 0.0],
            mLookAtOrigin: [0.0, 0.0, 0.0],
        };

        VectorCopy(confineOrigin, &mut area.mConfineOrigin);
        VectorCopy(lookAtOrigin, &mut area.mLookAtOrigin);

        area
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
    pub fn LookAt(&mut self, lookat: &vec3_t) {
        if self.mLookAt {
            // this area orients itself towards a point
            let mut a: vec3_t = [0.0, 0.0, 0.0];

            VectorCopy(lookat, &mut self.mLookAtOrigin);
            VectorSubtract(lookat, &self.mOrigin, &mut a);

            self.mAngle = unsafe { atan2(a[1], a[0]) };
        }
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

    pub fn AddMoveCount(&mut self) {
        self.mMoveCount += 1;
    }

    pub fn GetMoveCount(&self) -> c_int {
        self.mMoveCount
    }

    pub fn GetSymmetric(&self) -> c_int {
        self.mSymmetric
    }

    pub fn SetOrigin(&mut self, origin: &vec3_t) {
        VectorCopy(origin, &mut self.mOrigin);
    }

    pub fn GetOrigin(&self) -> &vec3_t {
        &self.mOrigin
    }

    pub fn GetOriginMut(&mut self) -> &mut vec3_t {
        &mut self.mOrigin
    }

    pub fn GetRadius(&self) -> f32 {
        self.mRadius
    }

    pub fn SetRadius(&mut self, radius: f32) {
        self.mRadius = radius;
    }

    pub fn GetLockOrigin(&self) -> bool {
        self.mLockOrigin
    }

    pub fn LockOrigin(&mut self) {
        self.mLockOrigin = true;
    }

    pub fn GetSpacingRadius(&self) -> f32 {
        self.mSpacingRadius
    }

    pub fn GetPaddingSize(&self) -> f32 {
        self.mPaddingSize
    }

    pub fn GetConfineOrigin(&self) -> &vec3_t {
        &self.mConfineOrigin
    }

    pub fn GetConfineRadius(&self) -> f32 {
        self.mConfineRadius
    }

    pub fn GetLookAtOrigin(&self) -> &vec3_t {
        &self.mLookAtOrigin
    }

    pub fn EnableLookAt(&mut self, enable: bool) {
        self.mLookAt = enable;
    }

    pub fn EnableCollision(&mut self, enable: bool) {
        self.mCollision = enable;
    }

    pub fn IsCollisionEnabled(&self) -> bool {
        self.mCollision
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
    pub fn new(mins: &vec3_t, maxs: &vec3_t) -> Self {
        let mut manager = CRMAreaManager {
            mAreas: Vec::new(),
            mMins: [0.0, 0.0, 0.0],
            mMaxs: [0.0, 0.0, 0.0],
            mWidth: 0.0,
            mHeight: 0.0,
        };

        VectorCopy(mins, &mut manager.mMins);
        VectorCopy(maxs, &mut manager.mMaxs);

        manager.mWidth = manager.mMaxs[0] - manager.mMins[0];
        manager.mHeight = manager.mMaxs[1] - manager.mMins[1];

        manager
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
    pub fn clear(&mut self) {
        let size = self.mAreas.len();
        for i in (0..size).rev() {
            // Box is automatically dropped when going out of scope
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
    pub fn MoveArea(&mut self, moved_index: usize, origin: &vec3_t) {
        if moved_index >= self.mAreas.len() {
            return;
        }

        // Increment the addcount (this is for infinite protection)
        self.mAreas[moved_index].AddMoveCount();

        // Infinite recursion prevention
        if self.mAreas[moved_index].GetMoveCount() > 250 {
            //		assert ( 0 );
            self.mAreas[moved_index].EnableCollision(false);
            return;
        }

        // First set the area's origin, This may cause it to be in collision with
        // another area but that will get fixed later
        self.mAreas[moved_index].SetOrigin(origin);

        // when symmetric we want to ensure that no instances end up on the "other" side of the imaginary diaganol that cuts the map in two
        // mSymmetric tells us which side of the map is legal
        if self.mAreas[moved_index].GetSymmetric() != 0 {
            // EXTERNAL: TheRandomMissionManager->GetLandScape()->GetBounds()
            // For now, we use stub access - this would be implemented with proper external bindings
            // let bounds = unsafe { GetLandscapeBounds(GetRandomMissionLandscape()) };
            // Stubbed for now
            let bounds: [vec3_t; 2] = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];

            let mut point: vec3_t = [0.0, 0.0, 0.0];
            let mut dir: vec3_t = [0.0, 0.0, 0.0];
            let mut tang: vec3_t = [0.0, 0.0, 0.0];
            let mut push: bool;
            let mut len: f32;

            VectorSubtract(self.mAreas[moved_index].GetOrigin(), &bounds[0], &mut point);
            VectorSubtract(&bounds[1], &bounds[0], &mut dir);
            VectorNormalize(&mut dir);

            dir[2] = 0.0;
            point[2] = 0.0;
            VectorMA(&bounds[0], DotProduct(&point, &dir), &dir, &mut tang);
            VectorSubtract(self.mAreas[moved_index].GetOrigin(), &tang, &mut dir);

            dir[2] = 0.0;
            push = false;
            len = VectorNormalize(&mut dir);

            if len < self.mAreas[moved_index].GetRadius() {
                if self.mAreas[moved_index].GetLockOrigin() {
                    self.mAreas[moved_index].EnableCollision(false);
                    return;
                }

                // EXTERNAL: irand from landscape
                let _rand_val = 0; // Stubbed
                let mut new_origin = *origin;
                VectorMA(&point, (self.mAreas[moved_index].GetSpacingRadius() - len) + _rand_val as f32, &dir, &mut new_origin);
                new_origin[0] = new_origin[0] + bounds[0][0];
                new_origin[1] = new_origin[1] + bounds[0][1];
                self.mAreas[moved_index].SetOrigin(&new_origin);
            }

            match self.mAreas[moved_index].GetSymmetric() {
                1 => {
                    // SYMMETRY_TOPLEFT
                    if self.mAreas[moved_index].GetOrigin()[1] > self.mAreas[moved_index].GetOrigin()[0] {
                        self.mAreas[moved_index].Mirror();
                    }
                }
                2 => {
                    // SYMMETRY_BOTTOMRIGHT
                    if self.mAreas[moved_index].GetOrigin()[1] < self.mAreas[moved_index].GetOrigin()[0] {
                        self.mAreas[moved_index].Mirror();
                    }
                }
                _ => {
                    // unknown symmetry type
                    // assert ( 0 );
                }
            }
        }

        // Confine to area unless we are being pushed back by the same guy who pushed us last time (infinite loop)
        if self.mAreas[moved_index].GetConfineRadius() != 0.0 {
            if self.mAreas[moved_index].GetMoveCount() < 25 {
                let mut cdiff: vec3_t = [0.0, 0.0, 0.0];
                let mut cdist: f32;

                VectorSubtract(self.mAreas[moved_index].GetOrigin(), self.mAreas[moved_index].GetConfineOrigin(), &mut cdiff);
                cdiff[2] = 0.0;
                cdist = VectorLength(&cdiff);

                if cdist + self.mAreas[moved_index].GetSpacingRadius() > self.mAreas[moved_index].GetConfineRadius() {
                    cdist = self.mAreas[moved_index].GetConfineRadius() - self.mAreas[moved_index].GetSpacingRadius();
                    VectorNormalize(&mut cdiff);

                    VectorMA(self.mAreas[moved_index].GetConfineOrigin(), cdist, &cdiff, self.mAreas[moved_index].GetOriginMut());
                }
            }
        }

        // See if it fell off the world in the x direction
        let origin_x = self.mAreas[moved_index].GetOrigin()[0];
        if origin_x + self.mAreas[moved_index].GetSpacingRadius() > self.mMaxs[0] {
            self.mAreas[moved_index].GetOriginMut()[0] = self.mMaxs[0] - self.mAreas[moved_index].GetSpacingRadius() - 100.0;
        } else if origin_x - self.mAreas[moved_index].GetSpacingRadius() < self.mMins[0] {
            self.mAreas[moved_index].GetOriginMut()[0] = self.mMins[0] + self.mAreas[moved_index].GetSpacingRadius() + 100.0;
        }

        // See if it fell off the world in the y direction
        let origin_y = self.mAreas[moved_index].GetOrigin()[1];
        if origin_y + self.mAreas[moved_index].GetSpacingRadius() > self.mMaxs[1] {
            self.mAreas[moved_index].GetOriginMut()[1] = self.mMaxs[1] - self.mAreas[moved_index].GetSpacingRadius() - 100.0;
        } else if origin_y - self.mAreas[moved_index].GetSpacingRadius() < self.mMins[1] {
            self.mAreas[moved_index].GetOriginMut()[1] = self.mMins[1] + self.mAreas[moved_index].GetSpacingRadius() + 100.0;
        }

        // Look at what we need to look at
        let lookat_origin = *self.mAreas[moved_index].GetLookAtOrigin();
        self.mAreas[moved_index].LookAt(&lookat_origin);

        // Dont collide against things that have no collision
        //	if ( !movedArea->IsCollisionEnabled ( ) )
        //	{
        //		return;
        //	}

        // See if its colliding
        let size = self.mAreas.len();
        let mut collision_checks: Vec<(usize, vec3_t)> = Vec::new();

        for index in 0..size {
            if index == moved_index {
                continue;
            }

            let mut diff: vec3_t = [0.0, 0.0, 0.0];
            let mut newOrigin: vec3_t = [0.0, 0.0, 0.0];
            let mut dist: f32;
            let mut targetdist: f32;

            if self.mAreas[index].GetLockOrigin() && self.mAreas[moved_index].GetLockOrigin() {
                continue;
            }

            // Dont collide against things that have no collision
            if !self.mAreas[index].IsCollisionEnabled() {
                continue;
            }

            // Grab the distance between the two
            // only want the horizontal distance -- dmv
            //dist		= Distance ( movedArea->GetOrigin ( ), area->GetOrigin ( ));
            let mut maOrigin: vec3_t = [0.0, 0.0, 0.0];
            let mut aOrigin: vec3_t = [0.0, 0.0, 0.0];
            VectorCopy(self.mAreas[moved_index].GetOrigin(), &mut maOrigin);
            VectorCopy(self.mAreas[index].GetOrigin(), &mut aOrigin);
            maOrigin[2] = 0.0;
            aOrigin[2] = 0.0;
            dist = Distance(&maOrigin, &aOrigin);
            targetdist = self.mAreas[moved_index].GetSpacingRadius() + self.mAreas[index].GetSpacingRadius()
                + maximum(self.mAreas[moved_index].GetPaddingSize(), self.mAreas[index].GetPaddingSize());

            if dist == 0.0 {
                // EXTERNAL: irand - stubbed
                self.mAreas[index].GetOriginMut()[0] += 50.0 * 0.5; // (50 * (float)(irand(0,99))/100.0f);
                self.mAreas[index].GetOriginMut()[1] += 50.0 * 0.5; // (50 * (float)(irand(0,99))/100.0f);

                VectorCopy(self.mAreas[index].GetOrigin(), &mut aOrigin);
                aOrigin[2] = 0.0;

                dist = Distance(&maOrigin, &aOrigin);
            }

            // Are they are enough apart?
            if dist >= targetdist {
                continue;
            }

            // Dont move a step if locked
            if self.mAreas[index].GetLockOrigin() {
                collision_checks.push((index, *self.mAreas[index].GetOrigin()));
                continue;
            }

            // we got a collision, move the guy we hit
            VectorSubtract(self.mAreas[index].GetOrigin(), self.mAreas[moved_index].GetOrigin(), &mut diff);
            diff[2] = 0.0;
            VectorNormalize(&mut diff);

            // Push by the difference in the distance and no-collide radius
            VectorMA(self.mAreas[index].GetOrigin(), targetdist - dist + 1.0, &diff, &mut newOrigin);

            // Move the area now
            collision_checks.push((index, newOrigin));
        }

        // Process collision checks after iteration to avoid borrow checker issues
        for (index, new_origin) in collision_checks {
            self.MoveArea(index, &new_origin);
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
        origin: &vec3_t,
        spacingRadius: f32,
        spacingLine: c_int,
        paddingSize: f32,
        confineRadius: f32,
        confineOrigin: &vec3_t,
        lookAtOrigin: &vec3_t,
        flatten: bool,
        collide: bool,
        lockorigin: bool,
        symmetric: c_int,
    ) -> *mut CRMArea {
        let mut area = Box::new(CRMArea::new(
            spacingRadius,
            paddingSize,
            confineRadius,
            confineOrigin,
            lookAtOrigin,
            flatten,
            symmetric,
        ));

        if lockorigin || spacingLine != 0 {
            area.LockOrigin();
        }

        if origin[0] != lookAtOrigin[0] || origin[1] != lookAtOrigin[1] {
            area.EnableLookAt(true);
        }

        // First add the area to the list
        self.mAreas.push(area);

        let area_index = self.mAreas.len() - 1;

        self.mAreas[area_index].EnableCollision(collide);

        // Set the real radius which is used for center line detection
        if spacingLine != 0 {
            self.mAreas[area_index].SetRadius(spacingRadius + (spacingLine - 1) as f32 * spacingRadius);
        }

        // Now move the area around
        self.MoveArea(area_index, origin);

        if origin[0] != lookAtOrigin[0] || origin[1] != lookAtOrigin[1] {
            let mut linedir: vec3_t = [0.0, 0.0, 0.0];
            let mut dir: vec3_t = [0.0, 0.0, 0.0];
            let up: vec3_t = [0.0, 0.0, 1.0];
            let mut zerodvec: vec3_t = [0.0, 0.0, 0.0];

            VectorClear(&mut zerodvec);

            VectorSubtract(lookAtOrigin, origin, &mut dir);
            VectorNormalize(&mut dir);
            dir[2] = 0.0;
            CrossProduct(&dir, &up, &mut linedir);

            for i in 0..(spacingLine - 1) {
                let linearea = Box::new(CRMArea::new(
                    spacingRadius,
                    paddingSize,
                    0.0,
                    &zerodvec,
                    &zerodvec,
                    false,
                    symmetric,
                ));
                self.mAreas.push(linearea);
                let linearea_index = self.mAreas.len() - 1;
                self.mAreas[linearea_index].LockOrigin();
                self.mAreas[linearea_index].EnableCollision(collide);

                let mut lineorigin: vec3_t = [0.0, 0.0, 0.0];
                VectorMA(origin, spacingRadius + (spacingRadius * 2.0 * i as f32), &linedir, &mut lineorigin);
                self.MoveArea(linearea_index, &lineorigin);

                let linearea2 = Box::new(CRMArea::new(
                    spacingRadius,
                    paddingSize,
                    0.0,
                    &zerodvec,
                    &zerodvec,
                    false,
                    symmetric,
                ));
                self.mAreas.push(linearea2);
                let linearea2_index = self.mAreas.len() - 1;
                self.mAreas[linearea2_index].LockOrigin();
                self.mAreas[linearea2_index].EnableCollision(collide);

                VectorMA(origin, -spacingRadius - (spacingRadius * 2.0 * i as f32), &linedir, &mut lineorigin);
                self.MoveArea(linearea2_index, &lineorigin);
            }
        }

        // Return it for convenience
        &mut *self.mAreas[area_index]
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
    pub fn EnumArea(&mut self, index: c_int) -> *mut CRMArea {
        // This isnt an assertion case because there is no size method for
        // the area manager so the areas are enumerated until NULL is returned.
        if index < 0 || (index as usize) >= self.mAreas.len() {
            ptr::null_mut()
        } else {
            &mut *self.mAreas[index as usize]
        }
    }
}
