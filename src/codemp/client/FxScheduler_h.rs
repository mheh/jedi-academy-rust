//! Mechanical port of `codemp/client/FxScheduler.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(improper_ctypes)]
#![allow(clippy::too_many_arguments)]

use core::ffi::{c_char, c_float, c_int, c_short};
use core::marker::PhantomData;

use crate::codemp::game::q_math::{flrand, irand};
use crate::codemp::game::q_shared::Q_stricmp;
use crate::codemp::game::q_shared_h::{qhandle_t, trace_t, vec3_t, vec4_t, MAX_QPATH};
use crate::codemp::qcommon::GenericParser2_h::{CGPGroup, CGPValue};
use crate::codemp::qcommon::stringed_interface_h::std_string;

use super::FxUtil_h::EMatImpactEffect;
use super::FxSystem_h::CGhoul2Info_v;

// ---------------------------------------------------------------------------
// Header constants.
// ---------------------------------------------------------------------------

pub const FX_FILE_PATH: &[u8; 8] = b"effects\0";

pub const FX_MAX_TRACE_DIST: c_int = 16384;
pub const FX_MAX_EFFECTS: usize = 256;
pub const FX_MAX_2DEFFECTS: usize = 64;
pub const FX_MAX_EFFECT_COMPONENTS: usize = 24;
pub const FX_MAX_PRIM_NAME: usize = 32;
pub const MAX_LOOPED_FX: usize = 32;

pub const FX_ORG_ON_SPHERE: c_int = 0x00001;
pub const FX_AXIS_FROM_SPHERE: c_int = 0x00002;
pub const FX_ORG_ON_CYLINDER: c_int = 0x00004;

pub const FX_ORG2_FROM_TRACE: c_int = 0x00010;
pub const FX_TRACE_IMPACT_FX: c_int = 0x00020;
pub const FX_ORG2_IS_OFFSET: c_int = 0x00040;

pub const FX_CHEAP_ORG_CALC: c_int = 0x00100;
pub const FX_CHEAP_ORG2_CALC: c_int = 0x00200;
pub const FX_VEL_IS_ABSOLUTE: c_int = 0x00400;
pub const FX_ACCEL_IS_ABSOLUTE: c_int = 0x00800;

pub const FX_RAND_ROT_AROUND_FWD: c_int = 0x01000;
pub const FX_EVEN_DISTRIBUTION: c_int = 0x02000;
pub const FX_RGB_COMPONENT_INTERP: c_int = 0x04000;

pub const FX_AFFECTED_BY_WIND: c_int = 0x10000;

// ---------------------------------------------------------------------------
// STL stand-ins.
// ---------------------------------------------------------------------------

// Porting deviation: the C++ header stores `std::map<std::string, int>` by value.
// The Rust port keeps the type visible as a lightweight wrapper so the owning
// structs can keep the original member order.
#[repr(C)]
pub struct std_map<K, V> {
    _opaque: [usize; 0],
    _K: PhantomData<K>,
    _V: PhantomData<V>,
}

impl<K, V> std_map<K, V> {
    pub const fn new() -> Self {
        Self {
            _opaque: [],
            _K: PhantomData,
            _V: PhantomData,
        }
    }
}

// Porting deviation: `CMediaHandles`/`TScheduledEffect` need usable container
// behavior for the inline methods in this header, so the Rust stand-ins are
// Vec-backed wrappers rather than opaque layout-only shells.
#[repr(C)]
pub struct std_vector<T> {
    _items: Vec<T>,
    _T: PhantomData<T>,
}

impl<T> std_vector<T> {
    pub fn new() -> Self {
        Self {
            _items: Vec::new(),
            _T: PhantomData,
        }
    }

    pub fn push_back(&mut self, item: T) {
        self._items.push(item);
    }

    pub fn size(&self) -> usize {
        self._items.len()
    }

    pub fn get(&self, index: usize) -> T
    where
        T: Copy,
    {
        self._items[index]
    }
}

#[repr(C)]
pub struct std_list<T> {
    _items: Vec<T>,
    _T: PhantomData<T>,
}

impl<T> std_list<T> {
    pub fn new() -> Self {
        Self {
            _items: Vec::new(),
            _T: PhantomData,
        }
    }

    pub fn push_back(&mut self, item: T) {
        self._items.push(item);
    }

    pub fn size(&self) -> usize {
        self._items.len()
    }
}

// ---------------------------------------------------------------------------
// CMediaHandles.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct CMediaHandles {
    mMediaList: std_vector<c_int>,
}

impl CMediaHandles {
    pub fn AddHandle(&mut self, item: c_int) {
        self.mMediaList.push_back(item);
    }

    pub fn GetHandle(&mut self) -> c_int {
        if self.mMediaList.size() == 0 {
            0
        } else {
            self.mMediaList.get(irand(0, self.mMediaList.size() as c_int - 1) as usize)
        }
    }

    pub unsafe fn operator_assign(&mut self, _that: *const CMediaHandles) {
        todo!("CMediaHandles::operator= is implemented in FxScheduler.cpp")
    }
}

// ---------------------------------------------------------------------------
// CFxRange.
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CFxRange {
    mMin: c_float,
    mMax: c_float,
}

impl CFxRange {
    pub fn CFxRange() -> Self {
        Self {
            mMin: 0.0,
            mMax: 0.0,
        }
    }

    pub fn SetRange(&mut self, min: c_float, max: c_float) {
        self.mMin = min;
        self.mMax = max;
    }

    pub fn GetMax(&self) -> c_float {
        self.mMax
    }

    pub fn GetMin(&self) -> c_float {
        self.mMin
    }

    pub fn GetVal_fraction(&self, fraction: c_float) -> c_float {
        if self.mMin != self.mMax {
            self.mMin + fraction * (self.mMax - self.mMin)
        } else {
            self.mMin
        }
    }

    pub fn GetVal(&self) -> c_float {
        if self.mMin != self.mMax {
            flrand(self.mMin, self.mMax)
        } else {
            self.mMin
        }
    }

    pub fn GetRoundedVal(&self) -> c_int {
        if self.mMin == self.mMax {
            self.mMin as c_int
        } else {
            (flrand(self.mMin, self.mMax) + 0.5) as c_int
        }
    }

    pub fn operator_eq(&self, rhs: &CFxRange) -> bool {
        self.mMin == rhs.mMin && self.mMax == rhs.mMax
    }
}

// ---------------------------------------------------------------------------
// Supported primitive types.
// ---------------------------------------------------------------------------

pub type EPrimType = c_int;

pub const None: EPrimType = 0;
pub const Particle: EPrimType = 1;
pub const Line: EPrimType = 2;
pub const Tail: EPrimType = 3;
pub const Cylinder: EPrimType = 4;
pub const Emitter: EPrimType = 5;
pub const Sound: EPrimType = 6;
pub const Decal: EPrimType = 7;
pub const OrientedParticle: EPrimType = 8;
pub const Electricity: EPrimType = 9;
pub const FxRunner: EPrimType = 10;
pub const Light: EPrimType = 11;
pub const CameraShake: EPrimType = 12;
pub const ScreenFlash: EPrimType = 13;

// ---------------------------------------------------------------------------
// CPrimitiveTemplate.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct CPrimitiveTemplate {
    pub mCopy: bool,
    pub mRefCount: c_int,
    pub mName: [c_char; FX_MAX_PRIM_NAME],

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

    pub mFlags: c_int,
    pub mSpawnFlags: c_int,

    pub mMatImpactFX: EMatImpactEffect,

    pub mMin: vec3_t,
    pub mMax: vec3_t,

    pub mOrigin1X: CFxRange,
    pub mOrigin1Y: CFxRange,
    pub mOrigin1Z: CFxRange,

    pub mOrigin2X: CFxRange,
    pub mOrigin2Y: CFxRange,
    pub mOrigin2Z: CFxRange,

    pub mRadius: CFxRange,
    pub mHeight: CFxRange,
    pub mWindModifier: CFxRange,

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

    pub mSoundRadius: c_int,
    pub mSoundVolume: c_int,
}

impl CPrimitiveTemplate {
    pub unsafe fn ParseVector(
        &mut self,
        _val: *const c_char,
        _min: *mut vec3_t,
        _max: *mut vec3_t,
    ) -> bool {
        todo!("CPrimitiveTemplate::ParseVector is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseFloat(
        &mut self,
        _val: *const c_char,
        _min: *mut c_float,
        _max: *mut c_float,
    ) -> bool {
        todo!("CPrimitiveTemplate::ParseFloat is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseGroupFlags(&mut self, _val: *const c_char, _flags: *mut c_int) -> bool {
        todo!("CPrimitiveTemplate::ParseGroupFlags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseMin(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseMin is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseMax(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseMax is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseDelay(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseDelay is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseCount(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseCount is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseLife(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseLife is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseElasticity(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseElasticity is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseFlags(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseFlags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSpawnFlags(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSpawnFlags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseOrigin1(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseOrigin1 is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseOrigin2(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseOrigin2 is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRadius(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseRadius is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseHeight(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseHeight is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseWindModifier(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseWindModifier is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRotation(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseRotation is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRotationDelta(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseRotationDelta is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAngle(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseAngle is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAngleDelta(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseAngleDelta is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseVelocity(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseVelocity is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAcceleration(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseAcceleration is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseGravity(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseGravity is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseDensity(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseDensity is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseVariance(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseVariance is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRGB(&mut self, _grp: *mut CGPGroup) -> bool {
        todo!("CPrimitiveTemplate::ParseRGB is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAlpha(&mut self, _grp: *mut CGPGroup) -> bool {
        todo!("CPrimitiveTemplate::ParseAlpha is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSize(&mut self, _grp: *mut CGPGroup) -> bool {
        todo!("CPrimitiveTemplate::ParseSize is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSize2(&mut self, _grp: *mut CGPGroup) -> bool {
        todo!("CPrimitiveTemplate::ParseSize2 is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseLength(&mut self, _grp: *mut CGPGroup) -> bool {
        todo!("CPrimitiveTemplate::ParseLength is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseModels(&mut self, _grp: *mut CGPValue) -> bool {
        todo!("CPrimitiveTemplate::ParseModels is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseShaders(&mut self, _grp: *mut CGPValue) -> bool {
        todo!("CPrimitiveTemplate::ParseShaders is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSounds(&mut self, _grp: *mut CGPValue) -> bool {
        todo!("CPrimitiveTemplate::ParseSounds is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseImpactFxStrings(&mut self, _grp: *mut CGPValue) -> bool {
        todo!("CPrimitiveTemplate::ParseImpactFxStrings is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseDeathFxStrings(&mut self, _grp: *mut CGPValue) -> bool {
        todo!("CPrimitiveTemplate::ParseDeathFxStrings is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseEmitterFxStrings(&mut self, _grp: *mut CGPValue) -> bool {
        todo!("CPrimitiveTemplate::ParseEmitterFxStrings is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParsePlayFxStrings(&mut self, _grp: *mut CGPValue) -> bool {
        todo!("CPrimitiveTemplate::ParsePlayFxStrings is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRGBStart(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseRGBStart is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRGBEnd(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseRGBEnd is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRGBParm(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseRGBParm is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseRGBFlags(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseRGBFlags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAlphaStart(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseAlphaStart is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAlphaEnd(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseAlphaEnd is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAlphaParm(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseAlphaParm is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseAlphaFlags(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseAlphaFlags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSizeStart(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSizeStart is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSizeEnd(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSizeEnd is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSizeParm(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSizeParm is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSizeFlags(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSizeFlags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSize2Start(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSize2Start is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSize2End(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSize2End is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSize2Parm(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSize2Parm is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseSize2Flags(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseSize2Flags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseLengthStart(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseLengthStart is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseLengthEnd(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseLengthEnd is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseLengthParm(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseLengthParm is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseLengthFlags(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseLengthFlags is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParseMaterialImpact(&mut self, _val: *const c_char) -> bool {
        todo!("CPrimitiveTemplate::ParseMaterialImpact is implemented in FxScheduler.cpp")
    }

    pub unsafe fn CPrimitiveTemplate() -> Self {
        todo!("CPrimitiveTemplate::CPrimitiveTemplate is implemented in FxScheduler.cpp")
    }

    pub unsafe fn ParsePrimitive(&mut self, _grp: *mut CGPGroup) -> bool {
        todo!("CPrimitiveTemplate::ParsePrimitive is implemented in FxScheduler.cpp")
    }

    pub unsafe fn operator_assign(&mut self, _that: *const CPrimitiveTemplate) {
        todo!("CPrimitiveTemplate::operator= is implemented in FxScheduler.cpp")
    }
}

// ---------------------------------------------------------------------------
// SEffectTemplate.
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SEffectTemplate {
    pub mInUse: bool,
    pub mCopy: bool,
    pub mEffectName: [c_char; MAX_QPATH],
    pub mPrimitiveCount: c_int,
    pub mRepeatDelay: c_int,
    pub mPrimitives: [*mut CPrimitiveTemplate; FX_MAX_EFFECT_COMPONENTS],
}

impl SEffectTemplate {
    pub unsafe fn operator_eq(&self, name: *const c_char) -> bool {
        Q_stricmp(self.mEffectName.as_ptr(), name) == 0
    }

    pub unsafe fn operator_assign(&mut self, _that: *const SEffectTemplate) {
        todo!("SEffectTemplate::operator= is implemented in FxScheduler.cpp")
    }
}

// ---------------------------------------------------------------------------
// CFxScheduler private nested types.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct SScheduledEffect {
    mpTemplate: *mut CPrimitiveTemplate,
    mStartTime: c_int,
    mModelNum: c_char,
    mBoltNum: c_char,
    mEntNum: c_short,
    mPortalEffect: bool,
    mIsRelative: bool,
    iGhoul2: c_int,
    mOrigin: vec3_t,
    mAxis: [vec3_t; 3],
}

impl SScheduledEffect {
    pub fn operator_le(&self, time: c_int) -> bool {
        self.mStartTime <= time
    }
}

#[repr(C)]
pub struct SLoopedEffect {
    mId: c_int,
    mBoltInfo: c_int,
    mGhoul2: CGhoul2Info_v,
    mNextTime: c_int,
    mLoopStopTime: c_int,
    mPortalEffect: bool,
    mIsRelative: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CScheduled2DEffect {
    pub mScreenX: c_float,
    pub mScreenY: c_float,
    pub mWidth: c_float,
    pub mHeight: c_float,
    pub mColor: vec4_t,
    pub mShaderHandle: qhandle_t,
}

impl CScheduled2DEffect {
    pub fn CScheduled2DEffect() -> Self {
        Self {
            mScreenX: 0.0,
            mScreenY: 0.0,
            mWidth: 0.0,
            mHeight: 0.0,
            mColor: [1.0, 1.0, 1.0, 1.0],
            mShaderHandle: 0,
        }
    }
}

pub type TEffectID = std_map<std_string, c_int>;
pub type TScheduledEffect = std_list<*mut SScheduledEffect>;

// ---------------------------------------------------------------------------
// CFxScheduler.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct CFxScheduler {
    mLoopedEffectArray: [SLoopedEffect; MAX_LOOPED_FX],

    mEffectTemplates: [SEffectTemplate; FX_MAX_EFFECTS],
    mEffectIDs: TEffectID,

    m2DEffects: [CScheduled2DEffect; FX_MAX_2DEFFECTS],
    mNextFree2DEffect: c_int,

    mFxSchedule: TScheduledEffect,
}

impl CFxScheduler {
    fn ScheduleLoopedEffect(
        &mut self,
        _id: c_int,
        _boltInfo: c_int,
        _iGhoul2: c_int,
        _isPortal: bool,
        _iLoopTime: c_int,
        _isRelative: bool,
    ) -> c_int {
        todo!("CFxScheduler::ScheduleLoopedEffect is implemented in FxScheduler.cpp")
    }

    fn AddLoopedEffects(&mut self) {
        todo!("CFxScheduler::AddLoopedEffects is implemented in FxScheduler.cpp")
    }

    unsafe fn GetNewEffectTemplate(&mut self, _id: *mut c_int, _file: *const c_char) -> *mut SEffectTemplate {
        todo!("CFxScheduler::GetNewEffectTemplate is implemented in FxScheduler.cpp")
    }

    unsafe fn AddPrimitiveToEffect(&mut self, _fx: *mut SEffectTemplate, _prim: *mut CPrimitiveTemplate) {
        todo!("CFxScheduler::AddPrimitiveToEffect is implemented in FxScheduler.cpp")
    }

    unsafe fn ParseEffect(&mut self, _file: *const c_char, _base: *mut CGPGroup) -> c_int {
        todo!("CFxScheduler::ParseEffect is implemented in FxScheduler.cpp")
    }

    unsafe fn CreateEffect_primitive(
        &mut self,
        _fx: *mut CPrimitiveTemplate,
        _origin: vec3_t,
        _axis: &mut [vec3_t; 3],
        _lateTime: c_int,
        _fxParm: c_int,
        _iGhoul2: c_int,
        _entNum: c_int,
        _modelNum: c_int,
        _boltNum: c_int,
    ) {
        todo!("CFxScheduler::CreateEffect(CPrimitiveTemplate*, ...) is implemented in FxScheduler.cpp")
    }

    unsafe fn CreateEffect_scheduled(&mut self, _fx: *mut CPrimitiveTemplate, _schedFx: *mut SScheduledEffect) {
        todo!("CFxScheduler::CreateEffect(SScheduledEffect*) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn CFxScheduler() -> Self {
        todo!("CFxScheduler::CFxScheduler is implemented in FxScheduler.cpp")
    }

    pub unsafe fn RegisterEffect(&mut self, _file: *const c_char, _bHasCorrectPath: bool) -> c_int {
        todo!("CFxScheduler::RegisterEffect is implemented in FxScheduler.cpp")
    }

    pub unsafe fn PlayEffect_id_org_fwd(
        &mut self,
        _id: c_int,
        _org: vec3_t,
        _fwd: vec3_t,
        _vol: c_int,
        _rad: c_int,
        _isPortal: bool,
    ) {
        todo!("CFxScheduler::PlayEffect(int, vec3_t, vec3_t, ...) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn PlayEffect_id_origin_axis(
        &mut self,
        _id: c_int,
        _origin: vec3_t,
        _axis: &mut [vec3_t; 3],
        _boltInfo: c_int,
        _iGhoul2: c_int,
        _fxParm: c_int,
        _vol: c_int,
        _rad: c_int,
        _isPortal: bool,
        _iLoopTime: c_int,
        _isRelative: bool,
    ) {
        todo!("CFxScheduler::PlayEffect(int, vec3_t, vec3_t[3], ...) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn PlayEffect_file_org(
        &mut self,
        _file: *const c_char,
        _org: vec3_t,
        _vol: c_int,
        _rad: c_int,
    ) {
        todo!("CFxScheduler::PlayEffect(const char*, vec3_t, ...) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn PlayEffect_file_org_fwd(
        &mut self,
        _file: *const c_char,
        _org: vec3_t,
        _fwd: vec3_t,
        _vol: c_int,
        _rad: c_int,
    ) {
        todo!("CFxScheduler::PlayEffect(const char*, vec3_t, vec3_t, ...) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn PlayEffect_file_origin_axis(
        &mut self,
        _file: *const c_char,
        _origin: vec3_t,
        _axis: &mut [vec3_t; 3],
        _boltInfo: c_int,
        _iGhoul2: c_int,
        _fxParm: c_int,
        _vol: c_int,
        _rad: c_int,
        _iLoopTime: c_int,
        _isRelative: bool,
    ) {
        todo!("CFxScheduler::PlayEffect(const char*, vec3_t, vec3_t[3], ...) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn StopEffect(&mut self, _file: *const c_char, _boltInfo: c_int, _isPortal: bool) {
        todo!("CFxScheduler::StopEffect is implemented in FxScheduler.cpp")
    }

    pub unsafe fn AddScheduledEffects(&mut self, _portal: bool) {
        todo!("CFxScheduler::AddScheduledEffects is implemented in FxScheduler.cpp")
    }

    pub unsafe fn Add2DEffect(
        &mut self,
        _x: c_float,
        _y: c_float,
        _w: c_float,
        _h: c_float,
        _color: vec4_t,
        _shaderHandle: qhandle_t,
    ) -> bool {
        todo!("CFxScheduler::Add2DEffect is implemented in FxScheduler.cpp")
    }

    pub unsafe fn Draw2DEffects(&mut self, _screenXScale: c_float, _screenYScale: c_float) {
        todo!("CFxScheduler::Draw2DEffects is implemented in FxScheduler.cpp")
    }

    pub fn NumScheduledFx(&mut self) -> c_int {
        self.mFxSchedule.size() as c_int
    }

    pub unsafe fn Clean(&mut self, _bRemoveTemplates: bool, _idToPreserve: c_int) {
        todo!("CFxScheduler::Clean is implemented in FxScheduler.cpp")
    }

    pub unsafe fn GetEffectCopy_fxHandle(
        &mut self,
        _fxHandle: c_int,
        _newHandle: *mut c_int,
    ) -> *mut SEffectTemplate {
        todo!("CFxScheduler::GetEffectCopy(int, int*) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn GetEffectCopy_file(
        &mut self,
        _file: *const c_char,
        _newHandle: *mut c_int,
    ) -> *mut SEffectTemplate {
        todo!("CFxScheduler::GetEffectCopy(const char*, int*) is implemented in FxScheduler.cpp")
    }

    pub unsafe fn GetPrimitiveCopy(
        &mut self,
        _effectCopy: *mut SEffectTemplate,
        _componentName: *const c_char,
    ) -> *mut CPrimitiveTemplate {
        todo!("CFxScheduler::GetPrimitiveCopy is implemented in FxScheduler.cpp")
    }

    pub unsafe fn MaterialImpact(&mut self, _tr: *mut trace_t, _effect: *mut CEffect) {
        todo!("CFxScheduler::MaterialImpact is implemented in FxScheduler.cpp")
    }
}

#[repr(C)]
pub struct CEffect {
    _unused: [u8; 0],
}

unsafe extern "C" {
    pub static mut theFxScheduler: CFxScheduler;
}
