//! Mechanical port of `code/cgame/FxScheduler.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;
use std::collections::{BTreeMap, LinkedList};

use crate::codemp::game::q_shared_h::{vec3_t, MAX_QPATH, WORLD_SIZE};
use crate::codemp::qcommon::GenericParser2_h::{CGPGroup, CGPValue};
use crate::code::qcommon::sstring_h::sstring;

// Typedef sstring_t to sstring with MAX_QPATH size
pub type fxString_t = sstring<MAX_QPATH>;

pub const FX_FILE_PATH: &str = "effects";

pub const FX_MAX_TRACE_DIST: c_int = WORLD_SIZE;
pub const FX_MAX_EFFECTS: usize = 150;		// how many effects the system can store
pub const FX_MAX_EFFECT_COMPONENTS: usize = 24;		// how many primitives an effect can hold, this should be plenty
pub const FX_MAX_PRIM_NAME: usize = 32;

//-----------------------------------------------
// These are spawn flags for primitiveTemplates
//-----------------------------------------------

pub const FX_ORG_ON_SPHERE: i32 = 0x00001;	// Pretty dang expensive, calculates a point on a sphere/ellipsoid
pub const FX_AXIS_FROM_SPHERE: i32 = 0x00002;	// Can be used in conjunction with org_on_sphere to cause particles to move out
										//	from the center of the sphere
pub const FX_ORG_ON_CYLINDER: i32 = 0x00004;	// calculate point on cylinder/disk

pub const FX_ORG2_FROM_TRACE: i32 = 0x00010;
pub const FX_TRACE_IMPACT_FX: i32 = 0x00020;	// if trace impacts, we should play one of the specified impact fx files
pub const FX_ORG2_IS_OFFSET: i32 = 0x00040;	// template specified org2 should be the offset from a trace endpos or
										//	passed in org2. You might use this to lend a random flair to the endpos.
										//	Note: this is done pre-trace, so you may have to specify large numbers for this

pub const FX_CHEAP_ORG_CALC: i32 = 0x00100;	// Origin is calculated relative to passed in axis unless this is on.
pub const FX_CHEAP_ORG2_CALC: i32 = 0x00200;	// Origin2 is calculated relative to passed in axis unless this is on.
pub const FX_VEL_IS_ABSOLUTE: i32 = 0x00400;	// Velocity isn't relative to passed in axis with this flag on.
pub const FX_ACCEL_IS_ABSOLUTE: i32 = 0x00800;	// Acceleration isn't relative to passed in axis with this flag on.

pub const FX_RAND_ROT_AROUND_FWD: i32 = 0x01000;	// Randomly rotates up and right around forward vector
pub const FX_EVEN_DISTRIBUTION: i32 = 0x02000;	// When you have a delay, it normally picks a random time to play.  When
										// this flag is on, it generates an even time distribution
pub const FX_RGB_COMPONENT_INTERP: i32 = 0x04000;	// Picks a color on the line defined by RGB min & max, default is to pick color in cube defined by min & max

pub const FX_AFFECTED_BY_WIND: i32 = 0x10000; // we take into account our wind vector when we spawn in

pub const FX_SND_LESS_ATTENUATION: i32 = 0x20000;	// attenuate sounds less

//-----------------------------------------------------------------
//
// CMediaHandles
//
// Primitive templates might want to use a list of sounds, shaders
//	or models to get a bit more variation in their effects.
//
//-----------------------------------------------------------------
pub struct CMediaHandles {
    mMediaList: Vec<i32>,
}

impl CMediaHandles {
    pub fn AddHandle(&mut self, item: i32) {
        self.mMediaList.push(item);
    }

    pub fn GetHandle(&self) -> i32 {
        if self.mMediaList.len() == 0 {
            0
        } else {
            // Note: irand function would need to be called to get random index
            // For now, this is the mechanical translation of the original
            self.mMediaList[0] // placeholder - actual implementation needs irand
        }
    }
}

impl Clone for CMediaHandles {
    fn clone(&self) -> Self {
        CMediaHandles {
            mMediaList: self.mMediaList.clone(),
        }
    }
}

//-----------------------------------------------------------------
//
// CFxRange
//
// Primitive templates typically use this class to define each of
//	its members.  This is done to make it easier to create effects
//	with a desired range of characteristics.
//
//-----------------------------------------------------------------
#[derive(Clone)]
pub struct CFxRange {
    mMin: f32,
    mMax: f32,
}

impl CFxRange {
    pub fn new() -> Self {
        CFxRange { mMin: 0.0, mMax: 0.0 }
    }

    #[inline]
    pub fn SetRange(&mut self, min: f32, max: f32) {
        self.mMin = min;
        self.mMax = max;
    }

    #[inline]
    pub fn SetMin(&mut self, min: f32) {
        self.mMin = min;
    }

    #[inline]
    pub fn SetMax(&mut self, max: f32) {
        self.mMax = max;
    }

    #[inline]
    pub fn GetMax(&self) -> f32 {
        self.mMax
    }

    #[inline]
    pub fn GetMin(&self) -> f32 {
        self.mMin
    }

    #[inline]
    pub fn GetVal(&self, percent: f32) -> f32 {
        if self.mMin == self.mMax {
            self.mMin
        } else {
            self.mMin + (self.mMax - self.mMin) * percent
        }
    }

    #[inline]
    pub fn GetValRand(&self) -> f32 {
        // Note: flrand function would need to be available
        // For now, this is the mechanical translation
        if self.mMin == self.mMax {
            self.mMin
        } else {
            self.mMin // placeholder - actual implementation needs flrand
        }
    }

    #[inline]
    pub fn GetRoundedVal(&self) -> c_int {
        // Note: flrand function would need to be available
        if self.mMin == self.mMax {
            self.mMin as c_int
        } else {
            (self.mMin + 0.5f32) as c_int // placeholder - actual implementation needs flrand
        }
    }

    #[inline]
    pub fn ForceRange(&mut self, min: f32, max: f32) {
        if self.mMin < min { self.mMin = min; }
        if self.mMin > max { self.mMin = max; }
        if self.mMax < min { self.mMax = min; }
        if self.mMax > max { self.mMax = max; }
    }

    #[inline]
    pub fn Sort(&mut self) {
        if self.mMin > self.mMax {
            let temp = self.mMin;
            self.mMin = self.mMax;
            self.mMax = temp;
        }
    }
}

impl PartialEq for CFxRange {
    fn eq(&self, rhs: &CFxRange) -> bool {
        (self.mMin == rhs.mMin) && (self.mMax == rhs.mMax)
    }
}

//----------------------------
// Supported primitive types
//----------------------------

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum EPrimType {
    None = 0,
    Particle,		// sprite
    Line,
    Tail,			// comet-like tail thing
    Cylinder,
    Emitter,		// emits effects as it moves, can also attach a chunk
    Sound,
    Decal,			// projected onto architecture
    OrientedParticle,
    Electricity,
    FxRunner,
    Light,
    CameraShake,
    ScreenFlash,
}

//-----------------------------------------------------------------
//
// CPrimitiveTemplate
//
// The primitive template is used to spawn 1 or more fx primitives
//	with the range of characteristics defined by the template.
//
// As such, I just made this one huge shared class knowing that
//	there won't be many of them in memory at once, 	and we won't
//	be dynamically creating and deleting them mid-game.  Also,
//	note that not every primitive type will use all of these fields.
//
//-----------------------------------------------------------------
pub struct CPrimitiveTemplate {
    // These kinds of things should not even be allowed to be accessed publicly
    pub mCopy: bool,
    pub mRefCount: c_int,		// For a copy of a primitive...when we figure out how many items we want to spawn,
                                //	we'll store that here and then decrement us for each we actually spawn.  When we
                                //	hit zero, we are no longer used and so we can just free ourselves

    pub mName: [u8; FX_MAX_PRIM_NAME],

    pub mType: EPrimType,

    pub mSpawnDelay: CFxRange,
    pub mSpawnCount: CFxRange,
    pub mLife: CFxRange,
    pub mCullRange: c_int,

    pub mMediaHandles: CMediaHandles,
    pub mImpactFxHandles: CMediaHandles,
    pub mDeathFxHandles: CMediaHandles,
    pub mEmitterFxHandles: CMediaHandles,
    pub mPlayFxHandles: CMediaHandles,

    pub mFlags: c_int,			// These need to get passed on to the primitive
    pub mSpawnFlags: c_int,	// These are only used to control spawning, but never get passed to prims.

    pub mMin: vec3_t,
    pub mMax: vec3_t,

    pub mOrigin1X: CFxRange,
    pub mOrigin1Y: CFxRange,
    pub mOrigin1Z: CFxRange,

    pub mOrigin2X: CFxRange,
    pub mOrigin2Y: CFxRange,
    pub mOrigin2Z: CFxRange,

    pub mRadius: CFxRange,		// spawn on sphere/ellipse/disk stuff.
    pub mHeight: CFxRange,

    pub mRotation: CFxRange,
    pub mRotationDelta: CFxRange,

    pub mAngle1: CFxRange,
    pub mAngle2: CFxRange,
    pub mAngle3: CFxRange,

    pub mAngle1Delta: CFxRange,
    pub mAngle2Delta: CFxRange,
    pub mAngle3Delta: CFxRange,

    pub mVelX: CFxRange,
    pub mVelY: CFxRange,
    pub mVelZ: CFxRange,

    pub mAccelX: CFxRange,
    pub mAccelY: CFxRange,
    pub mAccelZ: CFxRange,

    pub mGravity: CFxRange,

    pub mDensity: CFxRange,
    pub mVariance: CFxRange,

    pub mRedStart: CFxRange,
    pub mGreenStart: CFxRange,
    pub mBlueStart: CFxRange,

    pub mRedEnd: CFxRange,
    pub mGreenEnd: CFxRange,
    pub mBlueEnd: CFxRange,

    pub mRGBParm: CFxRange,

    pub mAlphaStart: CFxRange,
    pub mAlphaEnd: CFxRange,
    pub mAlphaParm: CFxRange,

    pub mSizeStart: CFxRange,
    pub mSizeEnd: CFxRange,
    pub mSizeParm: CFxRange,

    pub mSize2Start: CFxRange,
    pub mSize2End: CFxRange,
    pub mSize2Parm: CFxRange,

    pub mLengthStart: CFxRange,
    pub mLengthEnd: CFxRange,
    pub mLengthParm: CFxRange,

    pub mTexCoordS: CFxRange,
    pub mTexCoordT: CFxRange,

    pub mElasticity: CFxRange,
}

impl CPrimitiveTemplate {
    // Lower level parsing utilities
    pub fn ParseVector(&mut self, val: &str, min: &mut vec3_t, max: &mut vec3_t) -> bool {
        // Implementation stub
        false
    }

    pub fn ParseFloat(&mut self, val: &str, min: &mut f32, max: &mut f32) -> bool {
        // Implementation stub
        false
    }

    pub fn ParseGroupFlags(&mut self, val: &str, flags: &mut c_int) -> bool {
        // Implementation stub
        false
    }

    // Base key processing
    // Note that these all have their own parse functions in case it becomes important to do certain kinds
    //	of validation specific to that type.
    pub fn ParseMin(&mut self, val: &str) -> bool { false }
    pub fn ParseMax(&mut self, val: &str) -> bool { false }
    pub fn ParseDelay(&mut self, val: &str) -> bool { false }
    pub fn ParseCount(&mut self, val: &str) -> bool { false }
    pub fn ParseLife(&mut self, val: &str) -> bool { false }
    pub fn ParseElasticity(&mut self, val: &str) -> bool { false }
    pub fn ParseFlags(&mut self, val: &str) -> bool { false }
    pub fn ParseSpawnFlags(&mut self, val: &str) -> bool { false }

    pub fn ParseOrigin1(&mut self, val: &str) -> bool { false }
    pub fn ParseOrigin2(&mut self, val: &str) -> bool { false }
    pub fn ParseRadius(&mut self, val: &str) -> bool { false }
    pub fn ParseHeight(&mut self, val: &str) -> bool { false }
    pub fn ParseRotation(&mut self, val: &str) -> bool { false }
    pub fn ParseRotationDelta(&mut self, val: &str) -> bool { false }
    pub fn ParseAngle(&mut self, val: &str) -> bool { false }
    pub fn ParseAngleDelta(&mut self, val: &str) -> bool { false }
    pub fn ParseVelocity(&mut self, val: &str) -> bool { false }
    pub fn ParseAcceleration(&mut self, val: &str) -> bool { false }
    pub fn ParseGravity(&mut self, val: &str) -> bool { false }
    pub fn ParseDensity(&mut self, val: &str) -> bool { false }
    pub fn ParseVariance(&mut self, val: &str) -> bool { false }

    // Group type processing
    pub fn ParseRGB(&mut self, grp: *const CGPGroup) -> bool { false }
    pub fn ParseAlpha(&mut self, grp: *const CGPGroup) -> bool { false }
    pub fn ParseSize(&mut self, grp: *const CGPGroup) -> bool { false }
    pub fn ParseSize2(&mut self, grp: *const CGPGroup) -> bool { false }
    pub fn ParseLength(&mut self, grp: *const CGPGroup) -> bool { false }

    pub fn ParseModels(&mut self, grp: *const CGPValue) -> bool { false }
    pub fn ParseShaders(&mut self, grp: *const CGPValue) -> bool { false }
    pub fn ParseSounds(&mut self, grp: *const CGPValue) -> bool { false }
    #[cfg(feature = "_IMMERSION")]
    pub fn ParseForces(&mut self, grp: *const CGPValue) -> bool { false }

    pub fn ParseImpactFxStrings(&mut self, grp: *const CGPValue) -> bool { false }
    pub fn ParseDeathFxStrings(&mut self, grp: *const CGPValue) -> bool { false }
    pub fn ParseEmitterFxStrings(&mut self, grp: *const CGPValue) -> bool { false }
    pub fn ParsePlayFxStrings(&mut self, grp: *const CGPValue) -> bool { false }

    // Group keys
    pub fn ParseRGBStart(&mut self, val: &str) -> bool { false }
    pub fn ParseRGBEnd(&mut self, val: &str) -> bool { false }
    pub fn ParseRGBParm(&mut self, val: &str) -> bool { false }
    pub fn ParseRGBFlags(&mut self, val: &str) -> bool { false }

    pub fn ParseAlphaStart(&mut self, val: &str) -> bool { false }
    pub fn ParseAlphaEnd(&mut self, val: &str) -> bool { false }
    pub fn ParseAlphaParm(&mut self, val: &str) -> bool { false }
    pub fn ParseAlphaFlags(&mut self, val: &str) -> bool { false }

    pub fn ParseSizeStart(&mut self, val: &str) -> bool { false }
    pub fn ParseSizeEnd(&mut self, val: &str) -> bool { false }
    pub fn ParseSizeParm(&mut self, val: &str) -> bool { false }
    pub fn ParseSizeFlags(&mut self, val: &str) -> bool { false }

    pub fn ParseSize2Start(&mut self, val: &str) -> bool { false }
    pub fn ParseSize2End(&mut self, val: &str) -> bool { false }
    pub fn ParseSize2Parm(&mut self, val: &str) -> bool { false }
    pub fn ParseSize2Flags(&mut self, val: &str) -> bool { false }

    pub fn ParseLengthStart(&mut self, val: &str) -> bool { false }
    pub fn ParseLengthEnd(&mut self, val: &str) -> bool { false }
    pub fn ParseLengthParm(&mut self, val: &str) -> bool { false }
    pub fn ParseLengthFlags(&mut self, val: &str) -> bool { false }

    pub fn new() -> Self {
        CPrimitiveTemplate {
            mCopy: false,
            mRefCount: 0,
            mName: [0; FX_MAX_PRIM_NAME],
            mType: EPrimType::None,
            mSpawnDelay: CFxRange::new(),
            mSpawnCount: CFxRange::new(),
            mLife: CFxRange::new(),
            mCullRange: 0,
            mMediaHandles: CMediaHandles { mMediaList: Vec::new() },
            mImpactFxHandles: CMediaHandles { mMediaList: Vec::new() },
            mDeathFxHandles: CMediaHandles { mMediaList: Vec::new() },
            mEmitterFxHandles: CMediaHandles { mMediaList: Vec::new() },
            mPlayFxHandles: CMediaHandles { mMediaList: Vec::new() },
            mFlags: 0,
            mSpawnFlags: 0,
            mMin: [0.0; 3],
            mMax: [0.0; 3],
            mOrigin1X: CFxRange::new(),
            mOrigin1Y: CFxRange::new(),
            mOrigin1Z: CFxRange::new(),
            mOrigin2X: CFxRange::new(),
            mOrigin2Y: CFxRange::new(),
            mOrigin2Z: CFxRange::new(),
            mRadius: CFxRange::new(),
            mHeight: CFxRange::new(),
            mRotation: CFxRange::new(),
            mRotationDelta: CFxRange::new(),
            mAngle1: CFxRange::new(),
            mAngle2: CFxRange::new(),
            mAngle3: CFxRange::new(),
            mAngle1Delta: CFxRange::new(),
            mAngle2Delta: CFxRange::new(),
            mAngle3Delta: CFxRange::new(),
            mVelX: CFxRange::new(),
            mVelY: CFxRange::new(),
            mVelZ: CFxRange::new(),
            mAccelX: CFxRange::new(),
            mAccelY: CFxRange::new(),
            mAccelZ: CFxRange::new(),
            mGravity: CFxRange::new(),
            mDensity: CFxRange::new(),
            mVariance: CFxRange::new(),
            mRedStart: CFxRange::new(),
            mGreenStart: CFxRange::new(),
            mBlueStart: CFxRange::new(),
            mRedEnd: CFxRange::new(),
            mGreenEnd: CFxRange::new(),
            mBlueEnd: CFxRange::new(),
            mRGBParm: CFxRange::new(),
            mAlphaStart: CFxRange::new(),
            mAlphaEnd: CFxRange::new(),
            mAlphaParm: CFxRange::new(),
            mSizeStart: CFxRange::new(),
            mSizeEnd: CFxRange::new(),
            mSizeParm: CFxRange::new(),
            mSize2Start: CFxRange::new(),
            mSize2End: CFxRange::new(),
            mSize2Parm: CFxRange::new(),
            mLengthStart: CFxRange::new(),
            mLengthEnd: CFxRange::new(),
            mLengthParm: CFxRange::new(),
            mTexCoordS: CFxRange::new(),
            mTexCoordT: CFxRange::new(),
            mElasticity: CFxRange::new(),
        }
    }

    pub fn ParsePrimitive(&mut self, grp: *const CGPGroup) -> bool {
        // Implementation stub
        false
    }
}

// forward declaration
#[repr(C)]
pub struct SEffectTemplate {
    pub mInUse: bool,
    pub mCopy: bool,
    pub mEffectName: [u8; MAX_QPATH],					// is this extraneous??
    pub mPrimitiveCount: c_int,
    pub mRepeatDelay: c_int,
    pub mPrimitives: [*mut CPrimitiveTemplate; FX_MAX_EFFECT_COMPONENTS],
}

impl SEffectTemplate {
    pub fn operator_eq_str(&self, name: &str) -> bool {
        // Compare effect name with given string (case-insensitive in original with stricmp)
        false // Implementation stub
    }
}

//-----------------------------------------------------------------
//
// CFxScheduler
//
// The scheduler not only handles requests to play an effect, it
//	tracks the request throughout its life if necessary, creating
//	any of the delayed components as needed.
//
//-----------------------------------------------------------------
// needs to be in global space now (loadsave stuff)

pub const MAX_LOOPED_FX: usize = 32;

// We hold a looped effect here
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SLoopedEffect {
    pub mId: c_int,			// effect id
    pub mBoltInfo: c_int,		// used to determine which bolt on the ghoul2 model we should be attaching this effect to
    pub mNextTime: c_int,		//time to render again
    pub mLoopStopTime: c_int,	//time to die
    pub mPortalEffect: bool,	// rww - render this before skyportals, and not in the normal world view.
    pub mIsRelative: bool,	// bolt this puppy on keep it updated
}

pub struct CFxScheduler {
    // We hold a scheduled effect here
    mLoopedEffectArray: [SLoopedEffect; MAX_LOOPED_FX],

    // this makes looking up the index based on the string name much easier
    mEffectIDs: BTreeMap<fxString_t, c_int>,

    // Effects
    mEffectTemplates: [SEffectTemplate; FX_MAX_EFFECTS],

    // List of scheduled effects that will need to be created at the correct time.
    mFxSchedule: LinkedList<SScheduledEffect>,
}

// Private nested structure for scheduled effects
struct SScheduledEffect {
    mpTemplate: *mut CPrimitiveTemplate,	// primitive template
    mStartTime: c_int,
    mModelNum: u8,		// uset to determine which ghoul2 model we want to bolt this effect to
    mBoltNum: u8,		// used to determine which bolt on the ghoul2 model we should be attaching this effect to
    mEntNum: i16,		// used to determine which entity this ghoul model is attached to.
    mClientID: i16,		// FIXME: redundant. this is used for muzzle bolts, merge into normal bolting
    mPortalEffect: bool,	// rww - render this before skyportals, and not in the normal world view.
    mIsRelative: bool,	// bolt this puppy on keep it updated
    mOrigin: vec3_t,
    mAxis: [vec3_t; 3],
}

impl PartialOrd<c_int> for SScheduledEffect {
    fn partial_cmp(&self, other: &c_int) -> Option<std::cmp::Ordering> {
        if self.mStartTime <= *other {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

/* Looped Effects get stored and reschedule at mRepeatRate */

impl CFxScheduler {
    pub fn new() -> Self {
        CFxScheduler {
            mLoopedEffectArray: [SLoopedEffect {
                mId: 0,
                mBoltInfo: 0,
                mNextTime: 0,
                mLoopStopTime: 0,
                mPortalEffect: false,
                mIsRelative: false,
            }; MAX_LOOPED_FX],
            mEffectIDs: BTreeMap::new(),
            mEffectTemplates: [SEffectTemplate {
                mInUse: false,
                mCopy: false,
                mEffectName: [0; MAX_QPATH],
                mPrimitiveCount: 0,
                mRepeatDelay: 0,
                mPrimitives: [std::ptr::null_mut(); FX_MAX_EFFECT_COMPONENTS],
            }; FX_MAX_EFFECTS],
            mFxSchedule: LinkedList::new(),
        }
    }

    // Private function prototypes
    fn ScheduleLoopedEffect(&mut self, id: c_int, boltInfo: c_int, isPortal: bool, iLoopTime: c_int, isRelative: bool) -> c_int {
        // Implementation stub
        0
    }

    fn AddLoopedEffects(&mut self) {
        // Implementation stub
    }

    fn GetNewEffectTemplate(&mut self, id: &mut c_int, file: &str) -> *mut SEffectTemplate {
        // Implementation stub
        std::ptr::null_mut()
    }

    fn AddPrimitiveToEffect(&mut self, fx: *mut SEffectTemplate, prim: *mut CPrimitiveTemplate) {
        // Implementation stub
    }

    fn ParseEffect(&mut self, file: &str, base: *const CGPGroup) -> c_int {
        // Implementation stub
        0
    }

    fn CreateEffectFull(&mut self, fx: *mut CPrimitiveTemplate, origin: vec3_t, axis: &[vec3_t; 3], lateTime: c_int, clientID: c_int, modelNum: c_int, boltNum: c_int) {
        // Implementation stub
    }

    fn CreateEffectClientOnly(&mut self, fx: *mut CPrimitiveTemplate, clientID: c_int, lateTime: c_int) {
        // Implementation stub
    }

    pub fn LoadSave_Read(&mut self) {
        // Implementation stub
    }

    pub fn LoadSave_Write(&mut self) {
        // Implementation stub
    }

    pub fn FX_CopeWithAnyLoadedSaveGames(&mut self) {
        // Implementation stub
    }

    pub fn RegisterEffect(&mut self, file: &str, bHasCorrectPath: bool) -> c_int {
        // handles pre-caching
        // Implementation stub
        0
    }

    // Nasty overloaded madness
    pub fn PlayEffect_Org(&mut self, id: c_int, org: vec3_t, isPortal: bool) {
        // uses a default up axis
        // Implementation stub
    }

    pub fn PlayEffect_OrgFwd(&mut self, id: c_int, org: vec3_t, fwd: vec3_t, isPortal: bool) {
        // builds arbitrary perp. right vector, does a cross product to define up
        // Implementation stub
    }

    pub fn PlayEffect_Full(&mut self, id: c_int, origin: vec3_t, axis: &[vec3_t; 3], boltInfo: c_int, entNum: c_int, isPortal: bool, iLoopTime: c_int, isRelative: bool) {
        // Implementation stub
    }

    pub fn PlayEffect_FileOrg(&mut self, file: &str, org: vec3_t, isPortal: bool) {
        // uses a default up axis
        // Implementation stub
    }

    pub fn PlayEffect_FileOrgFwd(&mut self, file: &str, org: vec3_t, fwd: vec3_t, isPortal: bool) {
        // builds arbitrary perp. right vector, does a cross product to define up
        // Implementation stub
    }

    pub fn PlayEffect_FileFull(&mut self, file: &str, origin: vec3_t, axis: &[vec3_t; 3], boltInfo: c_int, entNum: c_int, isPortal: bool, iLoopTime: c_int, isRelative: bool) {
        // Implementation stub
    }

    //for muzzle
    pub fn PlayEffect_FileClientID(&mut self, file: &str, clientID: c_int, isPortal: bool) {
        // Implementation stub
    }

    #[cfg(feature = "_IMMERSION")]	// for ff-system
    pub fn PlayEffect_ClientIDOrgFwd(&mut self, id: c_int, clientNum: c_int, org: vec3_t, fwd: vec3_t, isPortal: bool) {
        // Implementation stub
    }

    #[cfg(feature = "_IMMERSION")]
    pub fn PlayEffect_FileClientIDOrgFwd(&mut self, file: &str, clientNum: c_int, origin: vec3_t, forward: vec3_t, isPortal: bool) {
        // Implementation stub
    }

    pub fn StopEffect(&mut self, file: &str, boltInfo: c_int, isPortal: bool) {
        //find a scheduled Looping effect with these parms and kill it
        // Implementation stub
    }

    pub fn AddScheduledEffects(&mut self, portal: bool) {
        // call once per CGame frame [rww ammendment - twice now actually, but first only renders portal effects]
        // Implementation stub
    }

    pub fn NumScheduledFx(&self) -> usize {
        self.mFxSchedule.len()
    }

    pub fn Clean(&mut self, bRemoveTemplates: bool, idToPreserve: c_int) {
        // clean out the system
        // Implementation stub
    }

    // FX Override functions
    pub fn GetEffectCopy(&mut self, fxHandle: c_int, newHandle: &mut c_int) -> *mut SEffectTemplate {
        // Implementation stub
        std::ptr::null_mut()
    }

    pub fn GetEffectCopyFromFile(&mut self, file: &str, newHandle: &mut c_int) -> *mut SEffectTemplate {
        // Implementation stub
        std::ptr::null_mut()
    }

    pub fn GetPrimitiveCopy(&mut self, effectCopy: *mut SEffectTemplate, componentName: &str) -> *mut CPrimitiveTemplate {
        // Implementation stub
        std::ptr::null_mut()
    }
}

//-------------------
// The one and only
//-------------------
// Porting note: In C++ this is declared as `extern CFxScheduler theFxScheduler;`
// meaning the definition is in FxScheduler.cpp. For Rust, we declare it as a mutable static.
// Initialization happens in FxScheduler.cpp port when system is initialized.
pub static mut theFxScheduler: Option<Box<CFxScheduler>> = None;
