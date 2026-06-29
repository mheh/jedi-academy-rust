// Anything above this #include will be ignored by the compiler

/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Area.cpp
 *
 ************************************************************************************************/

// #ifdef _WIN32
// #pragma optimize("p", on)
// #endif
// -- no Rust equivalent for MSVC optimize pragma; porting deviation noted.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_assignments)]

use core::ffi::c_int;

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::RMG::RM_Headers_h::*;

// CRMArea and CRMAreaManager are the paired classes declared in RM_Area.h.
// In Rust, implementing the constructor, LookAt, and Mirror methods (which are
// defined in this .cpp file) requires access to private struct fields.  Rust
// does not permit cross-module private-field access, so we define both structs
// locally — as real structs faithful to RM_Area.h — rather than importing them
// from crate::codemp::RMG::RM_Area_h.  Porting deviation noted.

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
        spacingRadius: f32,
        paddingSize: f32,
        confineRadius: f32,
        confineOrigin: vec3_t,
        lookAtOrigin: vec3_t,
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
            mOrigin: [0.0; 3],
            mConfineOrigin: [0.0; 3],
            mLookAtOrigin: [0.0; 3],
        };

        VectorCopy(&confineOrigin, &mut area.mConfineOrigin);
        VectorCopy(&lookAtOrigin, &mut area.mLookAtOrigin);

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
    pub fn LookAt(&mut self, lookat: vec3_t) -> f32 {
        if self.mLookAt {
            // this area orients itself towards a point
            let mut a: vec3_t = [0.0; 3];

            VectorCopy(&lookat, &mut self.mLookAtOrigin);
            VectorSubtract(&lookat, &self.mOrigin, &mut a);

            self.mAngle = (a[1] as f64).atan2(a[0] as f64) as f32;
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

    // --- inline methods from RM_Area.h ---

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
    // Returns a mutable raw pointer to the start of mOrigin (vec_t* in C++, non-const).
    // Callers may read or write through this pointer; &self is used for pragmatic Rust
    // reasons (returning *mut from &self is sound for non-aliasing field pointers).
    pub fn GetOrigin(&self) -> *mut vec_t {
        self.mOrigin.as_ptr() as *mut vec_t
    }
    pub fn GetConfineOrigin(&self) -> *mut vec_t {
        self.mConfineOrigin.as_ptr() as *mut vec_t
    }
    pub fn GetLookAtOrigin(&self) -> *mut vec_t {
        self.mLookAtOrigin.as_ptr() as *mut vec_t
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
        let mut mgr = CRMAreaManager {
            mAreas: Vec::new(),
            mMins: [0.0; 3],
            mMaxs: [0.0; 3],
            mWidth: 0.0,
            mHeight: 0.0,
        };

        VectorCopy(&mins, &mut mgr.mMins);
        VectorCopy(&maxs, &mut mgr.mMaxs);

        mgr.mWidth = mgr.mMaxs[0] - mgr.mMins[0];
        mgr.mHeight = mgr.mMaxs[1] - mgr.mMins[1];

        mgr
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
    pub unsafe fn MoveArea(&mut self, movedArea: *mut CRMArea, mut origin: vec3_t) {
        let mut index: c_int = 0;
        let mut size: c_int = 0;

        // Increment the addcount (this is for infinite protection)
        (*movedArea).AddMoveCount();

        // Infinite recursion prevention
        if (*movedArea).GetMoveCount() > 250 {
            //		assert ( 0 );
            (*movedArea).EnableCollision(false);
            return;
        }

        // First set the area's origin, This may cause it to be in collision with
        // another area but that will get fixed later
        (*movedArea).SetOrigin(origin);

        // when symmetric we want to ensure that no instances end up on the "other" side of the imaginary diaganol that cuts the map in two
        // mSymmetric tells us which side of the map is legal
        if (*movedArea).GetSymmetric() != 0 {
            let landscape = (*TheRandomMissionManager).GetLandScape();
            let bounds = (*landscape).GetBounds();

            let mut point: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];
            let mut tang: vec3_t = [0.0; 3];
            let _push: bool;
            let len: f32;

            VectorSubtract(
                &*((*movedArea).GetOrigin() as *const vec3_t),
                &(*bounds)[0],
                &mut point,
            );
            VectorSubtract(&(*bounds)[1], &(*bounds)[0], &mut dir);
            VectorNormalize(&mut dir);

            dir[2] = 0.0;
            point[2] = 0.0;
            VectorMA(&(*bounds)[0], DotProduct(&point, &dir), &dir, &mut tang);
            VectorSubtract(
                &*((*movedArea).GetOrigin() as *const vec3_t),
                &tang,
                &mut dir,
            );

            dir[2] = 0.0;
            _push = false;
            len = VectorNormalize(&mut dir);

            if len < (*movedArea).GetRadius() {
                if (*movedArea).GetLockOrigin() {
                    (*movedArea).EnableCollision(false);
                    return;
                }

                VectorMA(
                    &point,
                    ((*movedArea).GetSpacingRadius() - len)
                        + (*(*TheRandomMissionManager).GetLandScape())
                            .irand(10, (*movedArea).GetSpacingRadius() as c_int)
                            as f32,
                    &dir,
                    &mut point,
                );
                origin[0] = point[0] + (*bounds)[0][0];
                origin[1] = point[1] + (*bounds)[0][1];
                (*movedArea).SetOrigin(origin);
            }

            match (*movedArea).GetSymmetric() {
                x if x == SYMMETRY_TOPLEFT as c_int => {
                    if origin[1] > origin[0] {
                        (*movedArea).Mirror();
                    }
                }

                x if x == SYMMETRY_BOTTOMRIGHT as c_int => {
                    if origin[1] < origin[0] {
                        (*movedArea).Mirror();
                    }
                }

                _ => {
                    // unknown symmetry type
                    unreachable!();
                }
            }
        }

        // Confine to area unless we are being pushed back by the same guy who pushed us last time (infinite loop)
        if (*movedArea).GetConfineRadius() != 0.0 {
            if (*movedArea).GetMoveCount() < 25 {
                let mut cdiff: vec3_t = [0.0; 3];
                let mut cdist: f32;

                VectorSubtract(
                    &*((*movedArea).GetOrigin() as *const vec3_t),
                    &*((*movedArea).GetConfineOrigin() as *const vec3_t),
                    &mut cdiff,
                );
                cdiff[2] = 0.0;
                cdist = VectorLength(&cdiff);

                if cdist + (*movedArea).GetSpacingRadius() > (*movedArea).GetConfineRadius() {
                    cdist = (*movedArea).GetConfineRadius() - (*movedArea).GetSpacingRadius();
                    VectorNormalize(&mut cdiff);

                    VectorMA(
                        &*((*movedArea).GetConfineOrigin() as *const vec3_t),
                        cdist,
                        &cdiff,
                        &mut *((*movedArea).GetOrigin() as *mut vec3_t),
                    );
                }
            } else {
                index = 0;
            }
        }

        // See if it fell off the world in the x direction
        if *(*movedArea).GetOrigin().add(0) + (*movedArea).GetSpacingRadius() > self.mMaxs[0] {
            *(*movedArea).GetOrigin().add(0) = self.mMaxs[0]
                - (*movedArea).GetSpacingRadius()
                - (*(*TheRandomMissionManager).GetLandScape()).irand(10, 200) as f32;
        } else if *(*movedArea).GetOrigin().add(0) - (*movedArea).GetSpacingRadius()
            < self.mMins[0]
        {
            *(*movedArea).GetOrigin().add(0) = self.mMins[0]
                + (*movedArea).GetSpacingRadius()
                + (*(*TheRandomMissionManager).GetLandScape()).irand(10, 200) as f32;
        }

        // See if it fell off the world in the y direction
        if *(*movedArea).GetOrigin().add(1) + (*movedArea).GetSpacingRadius() > self.mMaxs[1] {
            *(*movedArea).GetOrigin().add(1) = self.mMaxs[1]
                - (*movedArea).GetSpacingRadius()
                - (*(*TheRandomMissionManager).GetLandScape()).irand(10, 200) as f32;
        } else if *(*movedArea).GetOrigin().add(1) - (*movedArea).GetSpacingRadius()
            < self.mMins[1]
        {
            *(*movedArea).GetOrigin().add(1) = self.mMins[1]
                + (*movedArea).GetSpacingRadius()
                + (*(*TheRandomMissionManager).GetLandScape()).irand(10, 200) as f32;
        }

        // Look at what we need to look at
        let lookat = *((*movedArea).GetLookAtOrigin() as *const vec3_t);
        (*movedArea).LookAt(lookat);

        // Dont collide against things that have no collision
        //	if ( !movedArea->IsCollisionEnabled ( ) )
        //	{
        //		return;
        //	}

        // See if its colliding
        index = 0;
        size = self.mAreas.len() as c_int;
        while index < size {
            let area: *mut CRMArea = self.mAreas[index as usize];
            let mut diff: vec3_t = [0.0; 3];
            let mut newOrigin: vec3_t = [0.0; 3];
            let dist: f32;
            let targetdist: f32;

            // Skip the one that was moved in the first place
            if area == movedArea {
                index += 1;
                continue;
            }

            if (*area).GetLockOrigin() && (*movedArea).GetLockOrigin() {
                index += 1;
                continue;
            }

            // Dont collide against things that have no collision
            if !(*area).IsCollisionEnabled() {
                index += 1;
                continue;
            }

            // Grab the distance between the two
            // only want the horizontal distance -- dmv
            //dist		= Distance ( movedArea->GetOrigin ( ), area->GetOrigin ( ));
            let mut maOrigin: vec3_t = [0.0; 3];
            let mut aOrigin: vec3_t = [0.0; 3];
            VectorCopy(
                &*((*movedArea).GetOrigin() as *const vec3_t),
                &mut maOrigin,
            );
            VectorCopy(&*((*area).GetOrigin() as *const vec3_t), &mut aOrigin);
            maOrigin[2] = 0.0;
            aOrigin[2] = 0.0;
            dist = Distance(&maOrigin, &aOrigin);
            targetdist = (*movedArea).GetSpacingRadius()
                + (*area).GetSpacingRadius()
                + maximum((*movedArea).GetPaddingSize(), (*area).GetPaddingSize());

            let mut dist_mut = dist;

            if dist_mut == 0.0 {
                *(*area).GetOrigin().add(0) +=
                    50.0 * ((*(*TheRandomMissionManager).GetLandScape()).irand(0, 99) as f32)
                        / 100.0;
                *(*area).GetOrigin().add(1) +=
                    50.0 * ((*(*TheRandomMissionManager).GetLandScape()).irand(0, 99) as f32)
                        / 100.0;

                VectorCopy(&*((*area).GetOrigin() as *const vec3_t), &mut aOrigin);
                aOrigin[2] = 0.0;

                dist_mut = Distance(&maOrigin, &aOrigin);
            }

            // Are they are enough apart?
            if dist_mut >= targetdist {
                index += 1;
                continue;
            }

            // Dont move a step if locked
            if (*area).GetLockOrigin() {
                let a_origin = *((*area).GetOrigin() as *const vec3_t);
                self.MoveArea(area, a_origin);
                index += 1;
                continue;
            }

            // we got a collision, move the guy we hit
            VectorSubtract(
                &*((*area).GetOrigin() as *const vec3_t),
                &*((*movedArea).GetOrigin() as *const vec3_t),
                &mut diff,
            );
            diff[2] = 0.0;
            VectorNormalize(&mut diff);

            // Push by the difference in the distance and no-collide radius
            VectorMA(
                &*((*area).GetOrigin() as *const vec3_t),
                targetdist - dist_mut + 1.0,
                &diff,
                &mut newOrigin,
            );

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
    pub unsafe fn CreateArea(
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
        let area: *mut CRMArea = Box::into_raw(Box::new(CRMArea::new(
            spacingRadius,
            paddingSize,
            confineRadius,
            confineOrigin,
            lookAtOrigin,
            flatten,
            symmetric,
        )));

        if lockorigin || spacingLine != 0 {
            (*area).LockOrigin();
        }

        if origin[0] != lookAtOrigin[0] || origin[1] != lookAtOrigin[1] {
            (*area).EnableLookAt(true);
        }

        // First add the area to the list
        self.mAreas.push(area);

        (*area).EnableCollision(collide);

        // Set the real radius which is used for center line detection
        if spacingLine != 0 {
            (*area).SetRadius(spacingRadius + (spacingLine - 1) as f32 * spacingRadius);
        }

        // Now move the area around
        self.MoveArea(area, origin);

        if origin[0] != lookAtOrigin[0] || origin[1] != lookAtOrigin[1] {
            let mut i: c_int;
            let mut linedir: vec3_t = [0.0; 3];
            let mut dir: vec3_t = [0.0; 3];
            let up: vec3_t = [0.0, 0.0, 1.0];

            VectorSubtract(&lookAtOrigin, &origin, &mut dir);
            VectorNormalize(&mut dir);
            dir[2] = 0.0;
            CrossProduct(&dir, &up, &mut linedir);

            i = 0;
            while i < spacingLine - 1 {
                let linearea: *mut CRMArea;
                let mut lineorigin: vec3_t = [0.0; 3];

                linearea = Box::into_raw(Box::new(CRMArea::new(
                    spacingRadius,
                    paddingSize,
                    0.0,
                    vec3_origin,
                    vec3_origin,
                    false,
                    symmetric,
                )));
                (*linearea).LockOrigin();
                (*linearea).EnableCollision(collide);

                VectorMA(
                    &origin,
                    spacingRadius + (spacingRadius * 2.0 * i as f32),
                    &linedir,
                    &mut lineorigin,
                );
                self.mAreas.push(linearea);
                self.MoveArea(linearea, lineorigin);

                let linearea: *mut CRMArea = Box::into_raw(Box::new(CRMArea::new(
                    spacingRadius,
                    paddingSize,
                    0.0,
                    vec3_origin,
                    vec3_origin,
                    false,
                    symmetric,
                )));
                (*linearea).LockOrigin();
                (*linearea).EnableCollision(collide);

                VectorMA(
                    &origin,
                    -spacingRadius - (spacingRadius * 2.0 * i as f32),
                    &linedir,
                    &mut lineorigin,
                );
                self.mAreas.push(linearea);
                self.MoveArea(linearea, lineorigin);

                i += 1;
            }
        }

        // Return it for convienience
        area
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
            return core::ptr::null_mut();
        }

        self.mAreas[index as usize]
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
impl Drop for CRMAreaManager {
    fn drop(&mut self) {
        let mut i: i32 = self.mAreas.len() as i32 - 1;
        while i >= 0 {
            unsafe {
                drop(Box::from_raw(self.mAreas[i as usize]));
            }
            i -= 1;
        }

        self.mAreas.clear();
    }
}

// #ifdef _WIN32
// #pragma optimize("p", off)
// #endif
