//Anything above this include will be ignored by the compiler

use core::ffi::{c_int, c_char};
use core::ptr;

// Extern declarations for external dependencies
extern "C" {
    // From client.h and other headers
    static mut cl: TCGVectorData; // Simplified reference, actual type may be more complex
    static mut cgvm: *mut core::ffi::c_void;

    static mut theFxHelper: CFxHelper;

    fn COM_StripExtension(in_: *const c_char, out: *mut c_char);
    fn strlwr(s: *mut c_char) -> *mut c_char;
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn memset(s: *mut core::ffi::c_void, c: c_int, n: usize) -> *mut core::ffi::c_void;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn VectorSet(v: *mut f32, x: f32, y: f32, z: f32);
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorCopy4(src: *const f32, dst: *mut f32);
    fn VectorClear(v: *mut f32);
    fn VectorScale(v: *const f32, scale: f32, out: *mut f32);
    fn VectorMA(v1: *mut f32, scale: f32, v2: *const f32, out: *mut f32);
    fn VectorAdd(v1: *const f32, v2: *const f32, out: *mut f32);
    fn VectorNormalize2(v: *const f32, out: *mut f32) -> f32;
    fn VectorNormalize(v: *mut f32) -> f32;
    fn CrossProduct(v1: *const f32, v2: *const f32, out: *mut f32);
    fn DistanceSquared(p1: *const f32, p2: *const f32) -> f32;
    fn AxisCopy(from: *const [f32; 3], to: *mut [f32; 3]);
    fn MakeNormalVectors(forward: *const f32, right: *mut f32, up: *mut f32);
    fn RotatePointAroundVector(dst: *mut f32, dir: *const f32, point: *const f32, degrees: f32);
    fn vectoangles(vec: *const f32, angles: *mut f32);
    fn fabsf(x: f32) -> f32;
    fn flrand(min: f32, max: f32) -> f32;
    fn abs(x: c_int) -> c_int;
    fn Round(x: f32) -> c_int;
    fn VM_Call(vm: *mut core::ffi::c_void, op: c_int, ...) -> c_int;

    fn FX_Add(portal: c_int);
    fn FX_AddParticle(
        org: *const f32, vel: *const f32, accel: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        rotation: f32, rotationDelta: f32,
        min: *const f32, max: *const f32, elasticity: f32,
        deathFx: c_int, impactFx: c_int,
        life: c_int, media: c_int, flags: c_int, matImpact: c_int, fxParm: c_int,
        iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int
    );
    fn FX_AddLine(
        org1: *const f32, org2: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        life: c_int, media: c_int, flags: c_int, matImpact: c_int, fxParm: c_int,
        iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int
    );
    fn FX_AddTail(
        org: *const f32, vel: *const f32, accel: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        lengthStart: f32, lengthEnd: f32, lengthParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        min: *const f32, max: *const f32, elasticity: f32,
        deathFx: c_int, impactFx: c_int,
        life: c_int, media: c_int, flags: c_int, matImpact: c_int, fxParm: c_int,
        iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int
    );
    fn FX_AddElectricity(
        org1: *const f32, org2: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        elasticity: f32, life: c_int, media: c_int, flags: c_int,
        matImpact: c_int, fxParm: c_int,
        iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int
    );
    fn FX_AddCylinder(
        org: *const f32, axis0: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        size2Start: f32, size2End: f32, size2Parm: f32,
        lengthStart: f32, lengthEnd: f32, lengthParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        life: c_int, media: c_int, flags: c_int, matImpact: c_int, fxParm: c_int,
        iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int,
        fromTrace: c_int
    );
    fn FX_AddEmitter(
        org: *const f32, vel: *const f32, accel: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        ang: *const f32, angDelta: *const f32,
        min: *const f32, max: *const f32, elasticity: f32,
        deathFx: c_int, impactFx: c_int, emitterFx: c_int,
        density: f32, variance: f32,
        life: c_int, model: c_int, flags: c_int, matImpact: c_int, fxParm: c_int
    );
    fn FX_AddOrientedParticle(
        org: *const f32, axis0: *const f32, vel: *const f32, accel: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        rotation: f32, rotationDelta: f32,
        min: *const f32, max: *const f32, elasticity: f32,
        deathFx: c_int, impactFx: c_int,
        life: c_int, media: c_int, flags: c_int, matImpact: c_int, fxParm: c_int,
        iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int
    );
    fn FX_AddLight(
        org: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        life: c_int, flags: c_int, matImpact: c_int, fxParm: c_int,
        iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int
    );
    fn FX_AddFlash(
        org: *const f32,
        sizeStart: f32, sizeEnd: f32, sizeParm: f32,
        alphaStart: f32, alphaEnd: f32, alphaParm: f32,
        sRGB: *const f32, eRGB: *const f32, rgbParm: f32,
        life: c_int, media: c_int, flags: c_int, matImpact: c_int, fxParm: c_int
    );
}

// Type stubs for dependencies
#[repr(C)]
pub struct TCGVectorData {
    pub mEntityNum: c_int,
    pub mPoint: [f32; 3],
    pub mSharedMemory: [u8; 0],
}

#[repr(C)]
pub struct CFxHelper {
    pub mTime: c_int,
    pub refdef: *mut core::ffi::c_void,
}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;
pub type qhandle_t = c_int;
pub type fileHandle_t = c_int;

const qfalse: qboolean = 0;
const qtrue: qboolean = 1;

const MAX_QPATH: usize = 256;
const MAX_LOOPED_FX: usize = 32;
const FX_MAX_EFFECTS: usize = 4096;
const FX_MAX_EFFECT_COMPONENTS: usize = 8;
const FX_MAX_2DEFFECTS: usize = 8;
const FX_MAX_TRACE_DIST: f32 = 8192.0;
const ENTITYNUM_NONE: c_int = 1023;

const ENTITY_SHIFT: c_int = 0;
const ENTITY_AND: c_int = 0x3FF;
const MODEL_SHIFT: c_int = 10;
const MODEL_AND: c_int = 0x3F;
const BOLT_SHIFT: c_int = 16;
const BOLT_AND: c_int = 0xFFFF;

const CG_GET_LERP_ORIGIN: c_int = 13;

// Spawn flags
const FX_AFFECTED_BY_WIND: c_int = 0x00000001;
const FX_CHEAP_ORG_CALC: c_int = 0x00000002;
const FX_CHEAP_ORG2_CALC: c_int = 0x00000004;
const FX_ORG_ON_SPHERE: c_int = 0x00000008;
const FX_ORG_ON_CYLINDER: c_int = 0x00000010;
const FX_AXIS_FROM_SPHERE: c_int = 0x00000020;
const FX_EVEN_DISTRIBUTION: c_int = 0x00000040;
const FX_RAND_ROT_AROUND_FWD: c_int = 0x00000080;
const FX_VEL_IS_ABSOLUTE: c_int = 0x00000200;
const FX_ACCEL_IS_ABSOLUTE: c_int = 0x00000400;
const FX_ORG2_FROM_TRACE: c_int = 0x00000800;
const FX_ORG2_IS_OFFSET: c_int = 0x00001000;
const FX_TRACE_IMPACT_FX: c_int = 0x00002000;
const FX_GHOUL2_DECALS: c_int = 0x00040000;
const FX_RGB_COMPONENT_INTERP: c_int = 0x00400000;

const FX_RELATIVE: c_int = 0x02000000;

// Flags for materials
const MATERIAL_MASK: c_int = 0xFF000000;

const FS_READ: c_int = 0;

const DEG2RAD: f32 = std::f32::consts::PI / 180.0;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum EPrimType {
    Particle = 0,
    Line = 1,
    Tail = 2,
    Sound = 3,
    Cylinder = 4,
    Electricity = 5,
    Emitter = 6,
    Decal = 7,
    OrientedParticle = 8,
    FxRunner = 9,
    Light = 10,
    CameraShake = 11,
    ScreenFlash = 12,
    None = 13,
}

// Stub types for external dependencies that we can't fully define here
#[repr(C)]
pub struct CMediaHandles {
    // Simplified stub - actual implementation in Oracle
}

#[repr(C)]
pub struct CPrimitiveTemplate {
    pub mType: EPrimType,
    pub mName: [c_char; 128],
    pub mSpawnFlags: c_int,
    pub mFlags: c_int,
    pub mCopy: bool,
    pub mRefCount: c_int,
    pub mLife: CRangedValue,
    pub mCullRange: f32,
    pub mSoundRadius: c_int,
    pub mSoundVolume: c_int,
    pub mSpawnCount: CRangedValue,
    pub mSpawnDelay: CRangedValue,
    pub mOrigin1X: CRangedValue,
    pub mOrigin1Y: CRangedValue,
    pub mOrigin1Z: CRangedValue,
    pub mOrigin2X: CRangedValue,
    pub mOrigin2Y: CRangedValue,
    pub mOrigin2Z: CRangedValue,
    pub mRadius: CRangedValue,
    pub mHeight: CRangedValue,
    pub mVelX: CRangedValue,
    pub mVelY: CRangedValue,
    pub mVelZ: CRangedValue,
    pub mAccelX: CRangedValue,
    pub mAccelY: CRangedValue,
    pub mAccelZ: CRangedValue,
    pub mGravity: CRangedValue,
    pub mSizeStart: CRangedValue,
    pub mSizeEnd: CRangedValue,
    pub mSizeParm: CRangedValue,
    pub mSize2Start: CRangedValue,
    pub mSize2End: CRangedValue,
    pub mSize2Parm: CRangedValue,
    pub mLengthStart: CRangedValue,
    pub mLengthEnd: CRangedValue,
    pub mLengthParm: CRangedValue,
    pub mAlphaStart: CRangedValue,
    pub mAlphaEnd: CRangedValue,
    pub mAlphaParm: CRangedValue,
    pub mRedStart: CRangedValue,
    pub mGreenStart: CRangedValue,
    pub mBlueStart: CRangedValue,
    pub mRedEnd: CRangedValue,
    pub mGreenEnd: CRangedValue,
    pub mBlueEnd: CRangedValue,
    pub mRGBParm: CRangedValue,
    pub mRotation: CRangedValue,
    pub mRotationDelta: CRangedValue,
    pub mAngle1: CRangedValue,
    pub mAngle2: CRangedValue,
    pub mAngle3: CRangedValue,
    pub mAngle1Delta: CRangedValue,
    pub mAngle2Delta: CRangedValue,
    pub mAngle3Delta: CRangedValue,
    pub mElasticity: CRangedValue,
    pub mDensity: CRangedValue,
    pub mVariance: CRangedValue,
    pub mWindModifier: CRangedValue,
    pub mMin: [f32; 3],
    pub mMax: [f32; 3],
    pub mMediaHandles: CMediaHandles,
    pub mDeathFxHandles: CMediaHandles,
    pub mImpactFxHandles: CMediaHandles,
    pub mEmitterFxHandles: CMediaHandles,
    pub mPlayFxHandles: CMediaHandles,
    pub mMatImpactFX: c_int,
}

#[repr(C)]
pub struct CRangedValue {
    // Simplified stub
}

impl CRangedValue {
    pub fn GetVal(&self) -> f32 {
        0.0
    }
    pub fn GetVal_percent(&self, _percent: f32) -> f32 {
        0.0
    }
    pub fn GetMin(&self) -> f32 {
        0.0
    }
    pub fn GetMax(&self) -> f32 {
        0.0
    }
}

#[repr(C)]
pub struct SEffectTemplate {
    pub mEffectName: [c_char; MAX_QPATH],
    pub mPrimitiveCount: c_int,
    pub mPrimitives: [*mut CPrimitiveTemplate; FX_MAX_EFFECT_COMPONENTS],
    pub mInUse: bool,
    pub mCopy: bool,
    pub mRepeatDelay: c_int,
}

impl SEffectTemplate {
    pub fn assign(&mut self, that: &SEffectTemplate) {
        self.mCopy = true;

        unsafe {
            strcpy(self.mEffectName.as_mut_ptr(), that.mEffectName.as_ptr());
        }

        self.mPrimitiveCount = that.mPrimitiveCount;

        for i in 0..(that.mPrimitiveCount as usize) {
            unsafe {
                let new_prim = Box::into_raw(Box::new(*(*that.mPrimitives[i])));
                self.mPrimitives[i] = new_prim;
                (*self.mPrimitives[i]).mCopy = true;
            }
        }
    }
}

#[repr(C)]
pub struct SLoopedEffect {
    pub mId: c_int,
    pub mBoltInfo: c_int,
    pub mGhoul2: CGhoul2Index,
    pub mPortalEffect: bool,
    pub mIsRelative: bool,
    pub mNextTime: c_int,
    pub mLoopStopTime: c_int,
}

#[repr(C)]
pub struct CGhoul2Index {
    pub mItem: c_int,
}

#[repr(C)]
pub struct S2DEffect {
    pub mScreenX: f32,
    pub mScreenY: f32,
    pub mWidth: f32,
    pub mHeight: f32,
    pub mColor: [f32; 4],
    pub mShaderHandle: qhandle_t,
}

#[repr(C)]
pub struct SScheduledEffect {
    pub mStartTime: c_int,
    pub mpTemplate: *mut CPrimitiveTemplate,
    pub mIsRelative: bool,
    pub mPortalEffect: bool,
    pub mBoltNum: c_int,
    pub mEntNum: c_int,
    pub mModelNum: c_int,
    pub iGhoul2: c_int,
    pub mOrigin: [f32; 3],
    pub mAxis: [[f32; 3]; 3],
}

impl PartialOrd for SScheduledEffect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.mStartTime.partial_cmp(&other.mStartTime)
    }
}

impl PartialEq for SScheduledEffect {
    fn eq(&self, other: &Self) -> bool {
        self.mStartTime == other.mStartTime
    }
}

impl Ord for SScheduledEffect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.mStartTime.cmp(&other.mStartTime)
    }
}

impl Eq for SScheduledEffect {}

#[repr(C)]
pub struct trace_t {
    pub endpos: [f32; 3],
    pub plane: CPlane,
    pub surfaceFlags: c_int,
}

#[repr(C)]
pub struct CPlane {
    pub normal: [f32; 3],
    pub d: f32,
}

#[repr(C)]
pub struct CGhoul2Info_v {
    pub handle: c_int,
}

impl CGhoul2Info_v {
    pub fn new(handle: c_int) -> Self {
        CGhoul2Info_v { handle }
    }

    pub fn kill(&mut self) {
        // Remove model ref without deleting it
    }
}

#[repr(C)]
pub struct CGPValue {
}

#[repr(C)]
pub struct CGPGroup {
}

impl CGPGroup {
    pub fn GetName(&self) -> *const c_char {
        ptr::null()
    }

    pub fn GetPairs(&self) -> *mut CGPValue {
        ptr::null_mut()
    }

    pub fn GetSubGroups(&self) -> *mut CGPGroup {
        ptr::null_mut()
    }

    pub fn GetNext(&self) -> *mut CGPGroup {
        ptr::null_mut()
    }
}

#[repr(C)]
pub struct CGenericParser2 {
}

impl CGenericParser2 {
    pub fn new() -> Self {
        CGenericParser2 {}
    }

    pub fn Parse(&mut self, _buffer: *mut *mut c_char) {
    }

    pub fn GetBaseParseGroup(&self) -> *mut CGPGroup {
        ptr::null_mut()
    }
}

pub type TScheduledEffect = std::collections::LinkedList<*mut SScheduledEffect>;
pub type TEffectID = std::collections::HashMap<String, c_int>;

#[repr(C)]
pub struct CFxScheduler {
    pub mNextFree2DEffect: c_int,
    pub mEffectTemplates: [SEffectTemplate; FX_MAX_EFFECTS],
    pub mLoopedEffectArray: [SLoopedEffect; MAX_LOOPED_FX],
    pub mFxSchedule: TScheduledEffect,
    pub mEffectIDs: TEffectID,
    pub m2DEffects: [S2DEffect; FX_MAX_2DEFFECTS],
}

impl CFxScheduler {
    pub fn new() -> Self {
        let mut scheduler = CFxScheduler {
            mNextFree2DEffect: 0,
            mEffectTemplates: unsafe { std::mem::zeroed() },
            mLoopedEffectArray: unsafe { std::mem::zeroed() },
            mFxSchedule: TScheduledEffect::new(),
            mEffectIDs: TEffectID::new(),
            m2DEffects: unsafe { std::mem::zeroed() },
        };

        scheduler.mNextFree2DEffect = 0;
        unsafe {
            memset(
                scheduler.mEffectTemplates.as_mut_ptr() as *mut core::ffi::c_void,
                0,
                std::mem::size_of_val(&scheduler.mEffectTemplates),
            );
            memset(
                scheduler.mLoopedEffectArray.as_mut_ptr() as *mut core::ffi::c_void,
                0,
                std::mem::size_of_val(&scheduler.mLoopedEffectArray),
            );
        }

        scheduler
    }

    //-----------------------------------------------------------
    pub fn ScheduleLoopedEffect(
        &mut self,
        id: c_int,
        boltInfo: c_int,
        iGhoul2: c_int,
        isPortal: bool,
        iLoopTime: c_int,
        isRelative: bool,
    ) -> c_int {
        let mut i: c_int = 0;

        assert!(id != 0);
        assert!(boltInfo != -1);

        // see if it's already playing so we can just update it
        for idx in 0..MAX_LOOPED_FX {
            i = idx as c_int;
            if unsafe { &self.mLoopedEffectArray[idx] }.mId == id
                && unsafe { &self.mLoopedEffectArray[idx] }.mBoltInfo == boltInfo
                && unsafe { &self.mLoopedEffectArray[idx] }.mPortalEffect == isPortal
            {
                #[cfg(debug_assertions)]
                {
                    unsafe {
                        theFxHelper.Print(
                            "CFxScheduler::ScheduleLoopedEffect- updating %s\n",
                            self.mEffectTemplates[id as usize].mEffectName.as_ptr(),
                        );
                    }
                }
                break;
            }
        }

        if i as usize == MAX_LOOPED_FX {
            // didn't find it existing, so find a free spot
            for idx in 0..MAX_LOOPED_FX {
                i = idx as c_int;
                if unsafe { &self.mLoopedEffectArray[idx] }.mId == 0 {
                    break;
                }
            }
        }

        if i as usize == MAX_LOOPED_FX {
            // bad
            assert!(i as usize != MAX_LOOPED_FX);
            unsafe {
                theFxHelper.Print(
                    "CFxScheduler::AddLoopedEffect- No Free Slots available for %d\n",
                    self.mEffectTemplates[id as usize].mEffectName.as_ptr(),
                );
            }
            return -1;
        }

        unsafe {
            self.mLoopedEffectArray[i as usize].mId = id;
            self.mLoopedEffectArray[i as usize].mBoltInfo = boltInfo;
            self.mLoopedEffectArray[i as usize].mGhoul2.mItem = iGhoul2;
            self.mLoopedEffectArray[i as usize].mPortalEffect = isPortal;
            self.mLoopedEffectArray[i as usize].mIsRelative = isRelative;
            self.mLoopedEffectArray[i as usize].mNextTime =
                theFxHelper.mTime + self.mEffectTemplates[id as usize].mRepeatDelay;
            self.mLoopedEffectArray[i as usize].mLoopStopTime = if iLoopTime == 1 {
                0
            } else {
                theFxHelper.mTime + iLoopTime
            };
        }

        i
    }

    pub fn StopEffect(&mut self, file: *const c_char, boltInfo: c_int, isPortal: bool) {
        let mut sfile: [c_char; MAX_QPATH] = unsafe { std::mem::zeroed() };

        // Get an extenstion stripped version of the file
        unsafe {
            COM_StripExtension(file, sfile.as_mut_ptr());
            let id = *self.mEffectIDs.get(
                std::ffi::CStr::from_ptr(sfile.as_ptr())
                    .to_str()
                    .unwrap_or("")
            ).unwrap_or(&0);

            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                if id == 0 {
                    theFxHelper.Print(
                        "CFxScheduler::StopEffect- unregistered/non-existent effect: %s\n",
                        sfile.as_ptr(),
                    );
                    return;
                }
            }

            for i in 0..MAX_LOOPED_FX {
                if self.mLoopedEffectArray[i].mId == id
                    && self.mLoopedEffectArray[i].mBoltInfo == boltInfo
                    && self.mLoopedEffectArray[i].mPortalEffect == isPortal
                {
                    memset(
                        &mut self.mLoopedEffectArray[i] as *mut _ as *mut core::ffi::c_void,
                        0,
                        std::mem::size_of::<SLoopedEffect>(),
                    );
                    return;
                }
            }

            #[cfg(debug_assertions)]
            {
                theFxHelper.Print(
                    "CFxScheduler::StopEffect- (%s) is not looping!\n",
                    file,
                );
            }
        }
    }

    pub fn AddLoopedEffects(&mut self) {
        unsafe {
            for i in 0..MAX_LOOPED_FX {
                if self.mLoopedEffectArray[i].mId != 0
                    && self.mLoopedEffectArray[i].mNextTime < theFxHelper.mTime
                {
                    let entNum = ((self.mLoopedEffectArray[i].mBoltInfo >> ENTITY_SHIFT)
                        & ENTITY_AND) as usize;
                    // Find out where the entity currently is
                    let data = &mut cl as *mut TCGVectorData;
                    (*data).mEntityNum = entNum as c_int;
                    VM_Call(cgvm, CG_GET_LERP_ORIGIN);

                    let id = self.mLoopedEffectArray[i].mId as usize;
                    self.PlayEffect(
                        id as c_int,
                        (*data).mPoint.as_ptr(),
                        std::ptr::null_mut(),
                        0,
                        self.mLoopedEffectArray[i].mGhoul2.mItem,
                        -1,
                        -1,
                        self.mLoopedEffectArray[i].mPortalEffect,
                        false,
                        self.mLoopedEffectArray[i].mIsRelative,
                    ); // very important to send FALSE to not recursively add me!

                    self.mLoopedEffectArray[i].mNextTime = theFxHelper.mTime
                        + self.mEffectTemplates[id].mRepeatDelay;
                    if self.mLoopedEffectArray[i].mLoopStopTime != 0
                        && self.mLoopedEffectArray[i].mLoopStopTime < theFxHelper.mTime
                    {
                        // time's up
                        // kill this entry
                        memset(
                            &mut self.mLoopedEffectArray[i] as *mut _ as *mut core::ffi::c_void,
                            0,
                            std::mem::size_of::<SLoopedEffect>(),
                        );
                    }
                }
            }
        }
    }

    //------------------------------------------------------
    // Clean
    //	Free up any memory we've allocated so we aren't leaking memory
    //
    // Input:
    //	Whether to clean everything or just stop the playing (active) effects
    //
    // Return:
    //	None
    //
    //------------------------------------------------------
    pub fn Clean(&mut self, bRemoveTemplates: bool, idToPreserve: c_int) {
        // Ditch any scheduled effects
        let mut itr = self.mFxSchedule.front();

        while let Some(_) = itr {
            if let Some(node) = self.mFxSchedule.pop_front() {
                unsafe {
                    Box::from_raw(node);
                }
            }
            itr = self.mFxSchedule.front();
        }

        if bRemoveTemplates {
            // Ditch any effect templates
            for i in 1..FX_MAX_EFFECTS {
                if i == idToPreserve as usize {
                    continue;
                }

                if unsafe { &self.mEffectTemplates[i] }.mInUse {
                    // Ditch the primitives
                    for j in 0..(unsafe { &self.mEffectTemplates[i] }.mPrimitiveCount as usize) {
                        unsafe {
                            if !self.mEffectTemplates[i].mPrimitives[j].is_null() {
                                Box::from_raw(self.mEffectTemplates[i].mPrimitives[j]);
                            }
                        }
                    }
                }

                unsafe { self.mEffectTemplates[i].mInUse = false; }
            }

            if idToPreserve == 0 {
                self.mEffectIDs.clear();
            } else {
                // Clear the effect names, but first get the name of the effect to preserve,
                // and restore it after clearing.
                let mut str: String = String::new();

                for (name, id) in self.mEffectIDs.iter() {
                    if *id == idToPreserve {
                        str = name.clone();
                        break;
                    }
                }

                self.mEffectIDs.clear();

                self.mEffectIDs.insert(str, idToPreserve);
            }
        }
    }

    //------------------------------------------------------
    // RegisterEffect
    //	Attempt to open the specified effect file, if
    //	file read succeeds, parse the file.
    //
    // Input:
    //	path or filename to open
    //
    // Return:
    //	int handle to the effect
    //------------------------------------------------------
    pub fn RegisterEffect(&mut self, file: *const c_char, bHasCorrectPath: bool) -> c_int {
        // Dealing with file names:
        // File names can come from two places - the editor, in which case we should use the given
        // path as is, and the effect file, in which case we should add the correct path and extension.
        // In either case we create a stripped file name to use for naming effects.
        //

        let mut sfile: [c_char; MAX_QPATH] = unsafe { std::mem::zeroed() };

        unsafe {
            COM_StripExtension(file, sfile.as_mut_ptr());
            strlwr(sfile.as_mut_ptr());

            Com_DPrintf("Registering effect : %s\n", sfile.as_ptr());

            // see if the specified file is already registered.  If it is, just return the id of that file
            let sfile_str = std::ffi::CStr::from_ptr(sfile.as_ptr())
                .to_str()
                .unwrap_or("");
            if let Some(id) = self.mEffectIDs.get(sfile_str) {
                return *id;
            }
        }

        // Simplified stub implementation for file handling
        0
    }

    //------------------------------------------------------
    // ParseEffect
    //	Starts at ground zero, using each group header to
    //	determine which kind of effect we are working with.
    //	Then we call the appropriate function to parse the
    //	specified effect group.
    //
    // Input:
    //	base group, essentially the whole files contents
    //
    // Return:
    //	int handle of the effect
    //------------------------------------------------------
    pub fn ParseEffect(&mut self, file: *const c_char, _base: *mut CGPGroup) -> c_int {
        let mut handle: c_int = 0;

        let effect = self.GetNewEffectTemplate(&mut handle, file);

        if handle == 0 || effect.is_null() {
            // failure
            return 0;
        }

        handle
    }

    //------------------------------------------------------
    // AddPrimitiveToEffect
    //	Takes a primitive and attaches it to the effect.
    //
    // Input:
    //	Effect template that we tack the primitive on to
    //	Primitive to add to the effect template
    //
    // Return:
    //	None
    //------------------------------------------------------
    pub fn AddPrimitiveToEffect(&mut self, fx: *mut SEffectTemplate, prim: *mut CPrimitiveTemplate) {
        unsafe {
            let ct = (*fx).mPrimitiveCount as usize;

            if ct >= FX_MAX_EFFECT_COMPONENTS {
                theFxHelper.Print("FxScheduler:  Error--too many primitives in an effect\n");
            } else {
                (*fx).mPrimitives[ct] = prim;
                (*fx).mPrimitiveCount += 1;
            }
        }
    }

    //------------------------------------------------------
    // GetNewEffectTemplate
    //	Finds an unused effect template and returns it to the
    //	caller.
    //
    // Input:
    //	pointer to an id that will be filled in,
    //	file name-- should be NULL when requesting a copy
    //
    // Return:
    //	the id of the added effect template
    //------------------------------------------------------
    pub fn GetNewEffectTemplate(
        &mut self,
        id: *mut c_int,
        file: *const c_char,
    ) -> *mut SEffectTemplate {
        // wanted zero to be a bogus effect ID, so we just skip it.
        for i in 1..FX_MAX_EFFECTS {
            unsafe {
                let effect = &mut self.mEffectTemplates[i];

                if !effect.mInUse {
                    *id = i as c_int;
                    memset(effect as *mut _ as *mut core::ffi::c_void, 0, std::mem::size_of::<SEffectTemplate>());

                    // If we are a copy, we really won't have a name that we care about saving for later
                    if !file.is_null() {
                        let file_str = std::ffi::CStr::from_ptr(file)
                            .to_str()
                            .unwrap_or("")
                            .to_string();
                        self.mEffectIDs.insert(file_str, i as c_int);
                        strcpy(effect.mEffectName.as_mut_ptr(), file);
                    }

                    effect.mInUse = true;
                    effect.mRepeatDelay = 300;
                    return effect;
                }
            }
        }

        unsafe {
            theFxHelper.Print("FxScheduler:  Error--reached max effects\n");
            *id = 0;
        }
        ptr::null_mut()
    }

    //------------------------------------------------------
    // GetEffectCopy
    //	Returns a copy of the desired effect so that it can
    //	easily be modified run-time.
    //
    // Input:
    //	file-- the name of the effect file that you want a copy of
    //	newHandle-- will actually be the returned handle to the new effect
    //				you have to hold onto this if you intend to call it again
    //
    // Return:
    //	the pointer to the copy
    //------------------------------------------------------
    pub fn GetEffectCopy_byName(
        &mut self,
        file: *const c_char,
        newHandle: *mut c_int,
    ) -> *mut SEffectTemplate {
        unsafe {
            let file_str = std::ffi::CStr::from_ptr(file)
                .to_str()
                .unwrap_or("")
                .to_string();
            if let Some(&fxHandle) = self.mEffectIDs.get(&file_str) {
                self.GetEffectCopy(fxHandle, newHandle)
            } else {
                *newHandle = 0;
                ptr::null_mut()
            }
        }
    }

    //------------------------------------------------------
    // GetEffectCopy
    //	Returns a copy of the desired effect so that it can
    //	easily be modified run-time.
    //
    // Input:
    //	fxHandle-- the handle to the effect that you want a copy of
    //	newHandle-- will actually be the returned handle to the new effect
    //				you have to hold onto this if you intend to call it again
    //
    // Return:
    //	the pointer to the copy
    //------------------------------------------------------
    pub fn GetEffectCopy(
        &mut self,
        fxHandle: c_int,
        newHandle: *mut c_int,
    ) -> *mut SEffectTemplate {
        if fxHandle < 1 || fxHandle >= FX_MAX_EFFECTS as c_int {
            // Didn't even request a valid effect to copy!!!
            unsafe {
                theFxHelper.Print(
                    "FxScheduler: Bad effect file copy request: id = %d\n",
                    fxHandle,
                );
                *newHandle = 0;
            }
            return ptr::null_mut();
        }

        unsafe {
            if !self.mEffectTemplates[fxHandle as usize].mInUse {
                // Didn't even request a valid effect to copy!!!
                theFxHelper.Print(
                    "FxScheduler: Bad effect file copy request: id %d not inuse\n",
                    fxHandle,
                );

                *newHandle = 0;
                return ptr::null_mut();
            }
        }

        // Copies shouldn't have names, otherwise they could trash our stl map used for getting ID from name
        let copy = self.GetNewEffectTemplate(newHandle, ptr::null());

        if !copy.is_null() && unsafe { *newHandle } != 0 {
            unsafe {
                // do the effect copy and mark us as what we are
                (*copy).assign(&self.mEffectTemplates[fxHandle as usize]);
                (*copy).mCopy = true;
            }

            // the user had better hold onto this handle if they ever hope to call this effect.
            return copy;
        }

        // No space left to return an effect
        unsafe {
            *newHandle = 0;
        }
        ptr::null_mut()
    }

    //------------------------------------------------------
    // GetPrimitiveCopy
    //	Helper function that returns a copy of the desired primitive
    //
    // Input:
    //	fxHandle - the pointer to the effect copy you want to override
    //	componentName - name of the component to find
    //
    // Return:
    //	the pointer to the desired primitive
    //------------------------------------------------------
    pub fn GetPrimitiveCopy(
        &self,
        effectCopy: *mut SEffectTemplate,
        componentName: *const c_char,
    ) -> *mut CPrimitiveTemplate {
        if effectCopy.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            if !(*effectCopy).mInUse {
                return ptr::null_mut();
            }

            for i in 0..((*effectCopy).mPrimitiveCount as usize) {
                if libc_stricmp((*effectCopy).mPrimitives[i] as *const c_char, componentName) == 0 {
                    // we found a match, so return it
                    return (*effectCopy).mPrimitives[i];
                }
            }
        }

        // bah, no good.
        ptr::null_mut()
    }

    pub fn MaterialImpact(&self, _tr: *mut trace_t, _effect: *mut core::ffi::c_void) {
        /*	EMatImpactEffect matImpactEffect = effect->GetMatImpactFX();
        int impactParm = effect->GetMatImpactParm();

        if (matImpactEffect == MATIMPACTFX_NONE)
        {
            return;
        }
        else if (matImpactEffect == MATIMPACTFX_SHELLSOUND)
        {
            // only want to play this for the first impact
            effect->SetMatImpactFX(MATIMPACTFX_NONE);

            int	material = tr->surfaceFlags & MATERIAL_MASK;
            const char *ammoName = CWeaponSystem::GetAmmoName(impactParm);

            if(ammoName && materials[material].HasShellSound(ammoName))
            {
                theFxHelper.PlaySound( tr->endpos, ENTITYNUM_NONE, CHAN_AUTO, materials[material].GetShellSoundHandle(ammoName) );
            }
        }*/
    }

    //------------------------------------------------------
    // PlayEffect
    //	Handles scheduling an effect so all the components
    //	happen at the specified time.  Takes a fwd vector
    //	and builds a right and up vector
    //
    // Input:
    //	Effect file id, the origin, and a fwd vector
    //
    // Return:
    //	none
    //------------------------------------------------------
    pub fn PlayEffect_with_forward(
        &mut self,
        id: c_int,
        origin: *const f32,
        forward: *const f32,
        vol: c_int,
        rad: c_int,
        isPortal: bool,
    ) {
        let mut axis: [[f32; 3]; 3] = unsafe { std::mem::zeroed() };

        // Take the forward vector and create two arbitrary but perpendicular vectors
        unsafe {
            VectorCopy(forward, axis[0].as_mut_ptr());
            MakeNormalVectors(forward, axis[1].as_mut_ptr(), axis[2].as_mut_ptr());

            self.PlayEffect(
                id,
                origin,
                axis.as_ptr() as *const [f32; 3],
                -1,
                0,
                -1,
                vol,
                rad,
                isPortal,
                0,
                false,
            );
        }
    }

    //------------------------------------------------------
    // PlayEffect
    //	Handles scheduling an effect so all the components
    //	happen at the specified time.  Uses the specified axis
    //
    // Input:
    //	Effect file name, the origin, and axis.
    //	Optional boltInfo (defaults to -1)
    //  and iGhoul2 used by boltInfo
    //
    // Return:
    //	none
    //------------------------------------------------------
    pub fn PlayEffect_from_file(
        &mut self,
        file: *const c_char,
        origin: *const f32,
        axis: *const [f32; 3],
        boltInfo: c_int,
        iGhoul2: c_int,
        fxParm: c_int,
        vol: c_int,
        rad: c_int,
        iLoopTime: c_int,
        isRelative: bool,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = unsafe { std::mem::zeroed() };

        // Get an extenstion stripped version of the file
        unsafe {
            COM_StripExtension(file, sfile.as_mut_ptr());

            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                let sfile_str = std::ffi::CStr::from_ptr(sfile.as_ptr())
                    .to_str()
                    .unwrap_or("");
                if *self.mEffectIDs.get(sfile_str).unwrap_or(&0) == 0 {
                    theFxHelper.Print(
                        "CFxScheduler::PlayEffect unregistered/non-existent effect: %s\n",
                        sfile.as_ptr(),
                    );
                    return;
                }
            }

            let id = *self
                .mEffectIDs
                .get(std::ffi::CStr::from_ptr(sfile.as_ptr()).to_str().unwrap_or(""))
                .unwrap_or(&0);
            self.PlayEffect(
                id, origin, axis, boltInfo, iGhoul2, fxParm, vol, rad, false, iLoopTime, isRelative,
            );
        }
    }

    pub fn PlayEffect_with_file_and_forward(
        &mut self,
        file: *const c_char,
        origin: *const f32,
        forward: *const f32,
        vol: c_int,
        rad: c_int,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = unsafe { std::mem::zeroed() };

        // Get an extenstion stripped version of the file
        unsafe {
            COM_StripExtension(file, sfile.as_mut_ptr());

            let id = *self
                .mEffectIDs
                .get(std::ffi::CStr::from_ptr(sfile.as_ptr()).to_str().unwrap_or(""))
                .unwrap_or(&0);
            self.PlayEffect_with_forward(id, origin, forward, vol, rad, false);
        }
    }

    //------------------------------------------------------
    // PlayEffect
    //	Handles scheduling an effect so all the components
    //	happen at the specified time.  Uses the specified axis
    //
    // Input:
    //	Effect id, the origin, and axis.
    //	Optional boltInfo (defaults to -1)
    //  Optional entity number to be used by a cheap entity origin bolt (defaults to -1)
    //
    // Return:
    //	none
    //------------------------------------------------------
    pub fn PlayEffect(
        &mut self,
        id: c_int,
        origin: *const f32,
        axis: *const [f32; 3],
        boltInfo: c_int,
        iGhoul2: c_int,
        fxParm: c_int,
        vol: c_int,
        rad: c_int,
        isPortal: bool,
        iLoopTime: c_int,
        isRelative: bool,
    ) {
        unsafe {
            if id < 1 || id >= FX_MAX_EFFECTS as c_int || !self.mEffectTemplates[id as usize].mInUse {
                // Now you've done it!
                ReportPlayEffectError(id);
                return;
            }

            #[cfg(feature = "_SOF2DEV_")]
            {
                if fx_freeze.integer != 0 {
                    return;
                }
            }

            let mut modelNum: c_int = 0;
            let mut boltNum: c_int = -1;
            let mut entityNum: c_int = -1;

            let mut forceScheduling = false;

            if boltInfo > 0 {
                // extract the wraith ID from the bolt info
                modelNum = (boltInfo >> MODEL_SHIFT) & MODEL_AND;
                boltNum = (boltInfo >> BOLT_SHIFT) & BOLT_AND;
                entityNum = (boltInfo >> ENTITY_SHIFT) & ENTITY_AND;

                // We always force ghoul bolted objects to be scheduled so that they don't play right away.
                forceScheduling = true;

                if iLoopTime != 0 {
                    // 0 = not looping, 1 for infinite, else duration
                    // store off the id to reschedule every frame
                    self.ScheduleLoopedEffect(id, boltInfo, iGhoul2, isPortal, iLoopTime, isRelative);
                }
            }

            // Get the effect.
            let fx = &self.mEffectTemplates[id as usize];

            #[cfg(not(feature = "FINAL_BUILD"))]
            {
                if fx_debug.integer == 2 {
                    Com_Printf("> %s\n", fx.mEffectName.as_ptr());
                }
            }

            // Loop through the primitives and schedule each bit
            for i in 0..(fx.mPrimitiveCount as usize) {
                totalPrimitives += 1;
                let prim = fx.mPrimitives[i];

                if prim.is_null() {
                    continue;
                }

                let prim_ref = &mut *prim;

                prim_ref.mSoundRadius = rad;
                prim_ref.mSoundVolume = vol;

                if prim_ref.mCullRange != 0.0 {
                    if DistanceSquared(origin, (theFxHelper.refdef as *const f32)) > prim_ref.mCullRange {
                        // is too far away
                        continue;
                    }
                }

                // Scale the particles based on the countscale factor.  Never, ever scale the particles upwards, however.
                let mut fxscale = 1.0; // fx_countScale->value, simplified
                if fxscale > 1.0 {
                    fxscale = 1.0;
                }

                // Only use scalability if there is a range
                // Temp fix until I have time to reweight all the scalability files
                let count = if fabsf(prim_ref.mSpawnCount.GetMax() - prim_ref.mSpawnCount.GetMin()) > 1.0 {
                    Round(prim_ref.mSpawnCount.GetVal() * fxscale)
                } else {
                    Round(prim_ref.mSpawnCount.GetVal())
                };

                // Make sure we have at least one particle after scaling
                let count = if prim_ref.mSpawnCount.GetMin() >= 1.0 && count < 1 {
                    1
                } else {
                    count
                };

                if prim_ref.mCopy {
                    // If we are a copy, we need to store a "how many references count" so that we
                    // can keep the primitive template around for the correct amount of time.
                    prim_ref.mRefCount = count;
                }

                let mut factor = 0.0;
                if prim_ref.mSpawnFlags & FX_EVEN_DISTRIBUTION != 0 {
                    factor = abs(prim_ref.mSpawnDelay.GetMax() as c_int - prim_ref.mSpawnDelay.GetMin() as c_int) as f32 / (count as f32);
                }

                // Schedule the random number of bits
                for t in 0..count {
                    totalEffects += 1;
                    let delay = if prim_ref.mSpawnFlags & FX_EVEN_DISTRIBUTION != 0 {
                        (t as f32 * factor) as c_int
                    } else {
                        prim_ref.mSpawnDelay.GetVal() as c_int
                    };

                    // if the delay is so small, we may as well just create this bit right now
                    if delay < 1 && !forceScheduling && !isPortal {
                        if boltInfo == -1 && entityNum != -1 {
                            // Find out where the entity currently is
                            let data = &mut cl as *mut TCGVectorData;
                            (*data).mEntityNum = entityNum;
                            VM_Call(cgvm, CG_GET_LERP_ORIGIN);
                            self.CreateEffect(prim, (*data).mPoint.as_ptr(), axis, -delay, fxParm, 0, -1, 0, -1);
                        } else {
                            self.CreateEffect(prim, origin, axis, -delay, fxParm, 0, -1, 0, -1);
                        }
                    } else {
                        // We have to create a new scheduled effect so that we can create it at a later point
                        // you should avoid this because it's much more expensive
                        let sfx = Box::into_raw(Box::new(SScheduledEffect {
                            mStartTime: theFxHelper.mTime + delay,
                            mpTemplate: prim,
                            mIsRelative: isRelative,
                            mPortalEffect: isPortal,
                            mBoltNum: -1,
                            mEntNum: ENTITYNUM_NONE,
                            mModelNum: 0,
                            iGhoul2: 0,
                            mOrigin: [0.0; 3],
                            mAxis: [[0.0; 3]; 3],
                        }));

                        if boltInfo == -1 {
                            (*sfx).iGhoul2 = 0;
                            if entityNum == -1 {
                                // we aren't bolting, so make sure the spawn system knows this by putting -1's in these fields
                                (*sfx).mBoltNum = -1;
                                (*sfx).mEntNum = ENTITYNUM_NONE;
                                (*sfx).mModelNum = 0;

                                if !origin.is_null() {
                                    VectorCopy(origin, (*sfx).mOrigin.as_mut_ptr());
                                } else {
                                    VectorClear((*sfx).mOrigin.as_mut_ptr());
                                }

                                AxisCopy(axis, (*sfx).mAxis.as_mut_ptr());
                            } else {
                                // we are doing bolting onto the origin of the entity, so use a cheaper method
                                (*sfx).mBoltNum = -1;
                                (*sfx).mEntNum = entityNum;
                                (*sfx).mModelNum = 0;

                                AxisCopy(axis, (*sfx).mAxis.as_mut_ptr());
                            }
                        } else {
                            // we are bolting, so store the extra info
                            (*sfx).mBoltNum = boltNum;
                            (*sfx).mEntNum = entityNum;
                            (*sfx).mModelNum = modelNum;
                            (*sfx).iGhoul2 = iGhoul2;

                            // Also, the ghoul bolt may not be around yet, so delay the creation one frame
                            (*sfx).mStartTime += 1;
                        }

                        self.mFxSchedule.push_front(sfx);
                    }
                }
            }

            // We track effect templates and primitive templates separately.
            if fx.mCopy {
                // We don't use dynamic memory allocation, so just mark us as dead
                self.mEffectTemplates[id as usize].mInUse = false;
            }
        }
    }

    //------------------------------------------------------
    // AddScheduledEffects
    //	Handles determining if a scheduled effect should
    //	be created or not.  If it should it handles converting
    //	the template effect into a real one.
    //
    // Input:
    //	none
    //
    // Return:
    //	none
    //------------------------------------------------------
    pub fn AddScheduledEffects(&mut self, portal: bool) {
        unsafe {
            if portal {
                gEffectsInPortal = true;
            } else {
                self.AddLoopedEffects();
            }

            let mut itr = self.mFxSchedule.front();

            while let Some(_) = itr {
                if let Some(node_ptr) = self.mFxSchedule.pop_front() {
                    let schedEffect = &*node_ptr;

                    if portal == schedEffect.mPortalEffect {
                        // only render portal fx on the skyportal pass and vice versa
                        if schedEffect.mStartTime <= theFxHelper.mTime {
                            if schedEffect.mBoltNum == -1 {
                                // ok, are we spawning a bolt on effect or a normal one?
                                if schedEffect.mEntNum != ENTITYNUM_NONE {
                                    // Find out where the entity currently is
                                    let data = &mut cl as *mut TCGVectorData;
                                    (*data).mEntityNum = schedEffect.mEntNum;
                                    VM_Call(cgvm, CG_GET_LERP_ORIGIN);
                                    self.CreateEffect(
                                        schedEffect.mpTemplate,
                                        (*data).mPoint.as_ptr(),
                                        schedEffect.mAxis.as_ptr() as *const [f32; 3],
                                        theFxHelper.mTime - schedEffect.mStartTime,
                                        -1,
                                        0,
                                        -1,
                                        0,
                                        -1,
                                    );
                                } else {
                                    self.CreateEffect(
                                        schedEffect.mpTemplate,
                                        schedEffect.mOrigin.as_ptr(),
                                        schedEffect.mAxis.as_ptr() as *const [f32; 3],
                                        theFxHelper.mTime - schedEffect.mStartTime,
                                        -1,
                                        0,
                                        -1,
                                        0,
                                        -1,
                                    );
                                }
                            } else {
                                // bolted on effect
                                // do we need to go and re-get the bolt matrix again? Since it takes time lets try to do it only once
                                static mut oldModelNum: c_int = -1;
                                static mut oldEntNum: c_int = -1;
                                static mut oldBoltIndex: c_int = -1;
                                static mut doesBoltExist: qboolean = qfalse;
                                static mut origin: [f32; 3] = [0.0; 3];
                                static mut axis: [[f32; 3]; 3] = [[0.0; 3]; 3];

                                if (schedEffect.mModelNum != oldModelNum)
                                    || (schedEffect.mEntNum != oldEntNum)
                                    || (schedEffect.mBoltNum != oldBoltIndex)
                                {
                                    oldModelNum = schedEffect.mModelNum;
                                    oldEntNum = schedEffect.mEntNum;
                                    oldBoltIndex = schedEffect.mBoltNum;
                                    let mut Ghoul2 = CGhoul2Info_v::new(schedEffect.iGhoul2);
                                    doesBoltExist = theFxHelper.GetOriginAxisFromBolt(
                                        &Ghoul2,
                                        schedEffect.mEntNum,
                                        schedEffect.mModelNum,
                                        schedEffect.mBoltNum,
                                        origin.as_mut_ptr(),
                                        axis.as_mut_ptr(),
                                    ) as qboolean;
                                    Ghoul2.kill(); // remove the model ref without actually deleting it
                                }

                                // only do this if we found the bolt
                                if doesBoltExist != 0 {
                                    if schedEffect.mIsRelative {
                                        self.CreateEffect(
                                            schedEffect.mpTemplate,
                                            origin.as_ptr(),
                                            axis.as_ptr() as *const [f32; 3],
                                            0,
                                            -1,
                                            schedEffect.iGhoul2,
                                            schedEffect.mEntNum,
                                            schedEffect.mModelNum,
                                            schedEffect.mBoltNum,
                                        );
                                    } else {
                                        self.CreateEffect(
                                            schedEffect.mpTemplate,
                                            origin.as_ptr(),
                                            axis.as_ptr() as *const [f32; 3],
                                            theFxHelper.mTime - schedEffect.mStartTime,
                                            -1,
                                            0,
                                            -1,
                                            0,
                                            -1,
                                        );
                                    }
                                }
                            }
                        }
                    }

                    Box::from_raw(node_ptr);
                }

                itr = self.mFxSchedule.front();
            }

            // Add all active effects into the scene
            FX_Add(if portal { 1 } else { 0 });

            gEffectsInPortal = false;
        }
    }

    pub fn Add2DEffect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: *const f32,
        shaderHandle: qhandle_t,
    ) -> bool {
        // need some sort of scale here because the effect was created using world units, not pixels
        let fxScale2D = 10.0;

        if self.mNextFree2DEffect < FX_MAX_2DEFFECTS as c_int {
            unsafe {
                self.m2DEffects[self.mNextFree2DEffect as usize].mScreenX = x;
                self.m2DEffects[self.mNextFree2DEffect as usize].mScreenY = y;
                self.m2DEffects[self.mNextFree2DEffect as usize].mWidth = w * fxScale2D;
                self.m2DEffects[self.mNextFree2DEffect as usize].mHeight = h * fxScale2D;
                VectorCopy4(color, self.m2DEffects[self.mNextFree2DEffect as usize].mColor.as_mut_ptr());
                self.m2DEffects[self.mNextFree2DEffect as usize].mShaderHandle = shaderHandle;
            }

            self.mNextFree2DEffect += 1;
            return true;
        }
        false
    }

    pub fn Draw2DEffects(&mut self, screenXScale: f32, screenYScale: f32) {
        for i in 0..(self.mNextFree2DEffect as usize) {
            let mut x = unsafe { self.m2DEffects[i].mScreenX };
            let mut y = unsafe { self.m2DEffects[i].mScreenY };
            let mut w = unsafe { self.m2DEffects[i].mWidth };
            let mut h = unsafe { self.m2DEffects[i].mHeight };

            x *= screenXScale;
            w *= screenXScale;
            y *= screenYScale;
            h *= screenYScale;

            unsafe {
                // allow 2d effect coloring?
                // re.DrawStretchPic(x - (w*0.5f), y - (h*0.5f), w, h, 0, 0, 1, 1, /*m2DEffects[i].mColor,*/ m2DEffects[i].mShaderHandle);
            }
        }
        // now that all 2D effects have been drawn we can consider the entire array to be free
        self.mNextFree2DEffect = 0;
    }

    //------------------------------------------------------
    // CreateEffect
    //	Creates the specified fx taking into account the
    //	multitude of different ways it could be spawned.
    //
    // Input:
    //	template used to build the effect, desired effect origin,
    //	desired orientation and how late the effect is so that
    //	it can be moved to the correct spot
    //
    // Return:
    //	none
    //------------------------------------------------------
    pub fn CreateEffect(
        &mut self,
        fx: *mut CPrimitiveTemplate,
        origin: *const f32,
        axis: *const [f32; 3],
        lateTime: c_int,
        fxParm: c_int,
        iGhoul2: c_int,
        entNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) {
        unsafe {
            let mut org: [f32; 3] = [0.0; 3];
            let mut org2: [f32; 3] = [0.0; 3];
            let mut temp: [f32; 3] = [0.0; 3];
            let mut vel: [f32; 3] = [0.0; 3];
            let mut accel: [f32; 3] = [0.0; 3];
            let mut sRGB: [f32; 3] = [0.0; 3];
            let mut eRGB: [f32; 3] = [0.0; 3];
            let mut ang: [f32; 3] = [0.0; 3];
            let mut angDelta: [f32; 3] = [0.0; 3];
            let mut ax: [[f32; 3]; 3] = [[0.0; 3]; 3];
            let mut tr: trace_t = std::mem::zeroed();
            let mut emitterModel: c_int = 0;

            // We may modify the axis, so make a work copy
            AxisCopy(axis, ax.as_mut_ptr());

            let mut flags = (*fx).mFlags;
            if iGhoul2 > 0 && modelNum >= 0 && boltNum >= 0 {
                // since you passed in these values, mark as relative to use them if it is supported
                match (*fx).mType {
                    EPrimType::Particle
                    | EPrimType::Line
                    | EPrimType::Tail
                    | EPrimType::Electricity
                    | EPrimType::Cylinder
                    | EPrimType::Emitter
                    | EPrimType::OrientedParticle
                    | EPrimType::Light => {
                        flags |= FX_RELATIVE;
                    }
                    EPrimType::Decal
                    | EPrimType::FxRunner
                    | EPrimType::ScreenFlash => {
                        // not supported yet
                    }
                    EPrimType::Sound | EPrimType::CameraShake => {
                        // does not work bolted
                    }
                    _ => {}
                }
            }

            if (*fx).mSpawnFlags & FX_RAND_ROT_AROUND_FWD != 0 {
                RotatePointAroundVector(
                    ax[1].as_mut_ptr(),
                    ax[0].as_ptr(),
                    (*axis)[1].as_ptr(),
                    flrand(0.0, 360.0),
                );
                CrossProduct(ax[0].as_ptr(), ax[1].as_ptr(), ax[2].as_mut_ptr());
            }

            // Origin calculations
            //-------------------------------------
            if (*fx).mSpawnFlags & FX_CHEAP_ORG_CALC != 0 || flags & FX_RELATIVE != 0 {
                // let's take the easy way out
                VectorSet(
                    org.as_mut_ptr(),
                    (*fx).mOrigin1X.GetVal(),
                    (*fx).mOrigin1Y.GetVal(),
                    (*fx).mOrigin1Z.GetVal(),
                );
            } else {
                // time for some extra work
                VectorScale(ax[0].as_ptr(), (*fx).mOrigin1X.GetVal(), org.as_mut_ptr());
                VectorMA(
                    org.as_mut_ptr(),
                    (*fx).mOrigin1Y.GetVal(),
                    ax[1].as_ptr(),
                    org.as_mut_ptr(),
                );
                VectorMA(
                    org.as_mut_ptr(),
                    (*fx).mOrigin1Z.GetVal(),
                    ax[2].as_ptr(),
                    org.as_mut_ptr(),
                );
            }

            // We always add our calculated offset to the passed in origin, unless relative!
            if flags & FX_RELATIVE == 0 {
                VectorAdd(org.as_ptr(), origin, org.as_mut_ptr());
            }

            // Now, we may need to calc a point on a sphere/ellipsoid/cylinder/disk and add that to it
            //----------------------------------------------------------------
            if (*fx).mSpawnFlags & FX_ORG_ON_SPHERE != 0 {
                let x = DEG2RAD * flrand(0.0, 360.0);
                let y = DEG2RAD * flrand(0.0, 180.0);

                let width = (*fx).mRadius.GetVal();
                let height = (*fx).mHeight.GetVal();

                // calculate point on ellipse
                VectorSet(
                    temp.as_mut_ptr(),
                    (x).sin() * width * (y).sin(),
                    (x).cos() * width * (y).sin(),
                    (y).cos() * height,
                ); // sinx * siny, cosx * siny, cosy
                VectorAdd(org.as_ptr(), temp.as_ptr(), org.as_mut_ptr());

                if (*fx).mSpawnFlags & FX_AXIS_FROM_SPHERE != 0 {
                    // well, we will now override the axis at the users request
                    VectorNormalize2(temp.as_ptr(), ax[0].as_mut_ptr());
                    MakeNormalVectors(ax[0].as_ptr(), ax[1].as_mut_ptr(), ax[2].as_mut_ptr());
                }
            } else if (*fx).mSpawnFlags & FX_ORG_ON_CYLINDER != 0 {
                let mut pt: [f32; 3] = [0.0; 3];

                // set up our point, then rotate around the current direction to.  Make unrotated cylinder centered around 0,0,0
                VectorScale(ax[1].as_ptr(), (*fx).mRadius.GetVal(), pt.as_mut_ptr());
                VectorMA(
                    pt.as_mut_ptr(),
                    flrand(-1.0, 1.0) * 0.5 * (*fx).mHeight.GetVal(),
                    ax[0].as_ptr(),
                    pt.as_mut_ptr(),
                );
                RotatePointAroundVector(
                    temp.as_mut_ptr(),
                    ax[0].as_ptr(),
                    pt.as_ptr(),
                    flrand(0.0, 360.0),
                );

                VectorAdd(org.as_ptr(), temp.as_ptr(), org.as_mut_ptr());

                if (*fx).mSpawnFlags & FX_AXIS_FROM_SPHERE != 0 {
                    let mut up = [0.0, 0.0, 1.0];

                    // well, we will now override the axis at the users request
                    VectorNormalize2(temp.as_ptr(), ax[0].as_mut_ptr());

                    if ax[0][2] == 1.0 {
                        // readjust up
                        VectorSet(up.as_mut_ptr(), 0.0, 1.0, 0.0);
                    }

                    CrossProduct(up.as_ptr(), ax[0].as_ptr(), ax[1].as_mut_ptr());
                    CrossProduct(ax[0].as_ptr(), ax[1].as_ptr(), ax[2].as_mut_ptr());
                }
            }

            if (*fx).mType as c_int == EPrimType::OrientedParticle as c_int {
                // bolted oriented particles use origin2 as an angular rotation offset...
                if flags & FX_RELATIVE != 0 {
                    VectorSet(
                        ax[0].as_mut_ptr(),
                        (*fx).mOrigin2X.GetVal(),
                        (*fx).mOrigin2Y.GetVal(),
                        (*fx).mOrigin2Z.GetVal(),
                    );
                }
            }

            // There are only a few types that really use velocity and acceleration, so do extra work for those types
            //--------------------------------------------------------------------------------------------------------
            if (*fx).mType as c_int == EPrimType::Particle as c_int
                || (*fx).mType as c_int == EPrimType::OrientedParticle as c_int
                || (*fx).mType as c_int == EPrimType::Tail as c_int
                || (*fx).mType as c_int == EPrimType::Emitter as c_int
            {
                // Velocity calculations
                //-------------------------------------
                if (*fx).mSpawnFlags & FX_VEL_IS_ABSOLUTE != 0 || flags & FX_RELATIVE != 0 {
                    VectorSet(
                        vel.as_mut_ptr(),
                        (*fx).mVelX.GetVal(),
                        (*fx).mVelY.GetVal(),
                        (*fx).mVelZ.GetVal(),
                    );
                } else {
                    // bah, do some extra work to coerce it
                    VectorScale(ax[0].as_ptr(), (*fx).mVelX.GetVal(), vel.as_mut_ptr());
                    VectorMA(vel.as_mut_ptr(), (*fx).mVelY.GetVal(), ax[1].as_ptr(), vel.as_mut_ptr());
                    VectorMA(vel.as_mut_ptr(), (*fx).mVelZ.GetVal(), ax[2].as_ptr(), vel.as_mut_ptr());
                }

                //-------------------------------------
                if (*fx).mSpawnFlags & FX_AFFECTED_BY_WIND != 0 {
                    // wind is affecting us, so modify our initial velocity.  ideally, we would update throughout our lives, but this is easier
                    // CL_GetWindVector( wind );
                    // VectorMA( vel, fx->mWindModifier.GetVal() * 0.01f, wind, vel );
                }

                // Acceleration calculations
                //-------------------------------------
                if (*fx).mSpawnFlags & FX_ACCEL_IS_ABSOLUTE != 0 || flags & FX_RELATIVE != 0 {
                    VectorSet(
                        accel.as_mut_ptr(),
                        (*fx).mAccelX.GetVal(),
                        (*fx).mAccelY.GetVal(),
                        (*fx).mAccelZ.GetVal(),
                    );
                } else {
                    VectorScale(ax[0].as_ptr(), (*fx).mAccelX.GetVal(), accel.as_mut_ptr());
                    VectorMA(accel.as_mut_ptr(), (*fx).mAccelY.GetVal(), ax[1].as_ptr(), accel.as_mut_ptr());
                    VectorMA(accel.as_mut_ptr(), (*fx).mAccelZ.GetVal(), ax[2].as_ptr(), accel.as_mut_ptr());
                }

                // Gravity is completely decoupled from acceleration since it is __always__ absolute
                // NOTE: I only effect Z ( up/down in the Quake world )
                accel[2] += (*fx).mGravity.GetVal();

                // There may be a lag between when the effect should be created and when it actually gets created.
                // Since we know what the discrepancy is, we can attempt to compensate...
                if lateTime > 0 {
                    // Calc the time differences
                    let ftime = lateTime as f32 * 0.001;
                    let time2 = ftime * ftime * 0.5;

                    VectorMA(vel.as_mut_ptr(), ftime, accel.as_ptr(), vel.as_mut_ptr());

                    // Predict the new position
                    for idx in 0..3 {
                        org[idx] = org[idx] + ftime * vel[idx] + time2 * vel[idx];
                    }
                }
            } // end moving types

            // Line type primitives work with an origin2, so do the extra work for them
            //--------------------------------------------------------------------------
            if (*fx).mType as c_int == EPrimType::Line as c_int
                || (*fx).mType as c_int == EPrimType::Electricity as c_int
            {
                // We may have to do a trace to find our endpoint
                if (*fx).mSpawnFlags & FX_ORG2_FROM_TRACE != 0 {
                    VectorMA(org.as_ptr(), FX_MAX_TRACE_DIST, ax[0].as_ptr(), temp.as_mut_ptr());

                    if (*fx).mSpawnFlags & FX_ORG2_IS_OFFSET != 0 {
                        // add a random flair to the endpoint...note: org2 will have to be pretty large to affect this much
                        // we also do this pre-trace as opposed to post trace since we may have to render an impact effect
                        // and we will want the normal at the exact endpos...
                        if (*fx).mSpawnFlags & FX_CHEAP_ORG2_CALC != 0 || flags & FX_RELATIVE != 0 {
                            VectorSet(
                                org2.as_mut_ptr(),
                                (*fx).mOrigin2X.GetVal(),
                                (*fx).mOrigin2Y.GetVal(),
                                (*fx).mOrigin2Z.GetVal(),
                            );
                            VectorAdd(org2.as_ptr(), temp.as_ptr(), temp.as_mut_ptr());
                        } else {
                            // I can only imagine a few cases where you might want to do this...
                            VectorMA(temp.as_mut_ptr(), (*fx).mOrigin2X.GetVal(), ax[0].as_ptr(), temp.as_mut_ptr());
                            VectorMA(temp.as_mut_ptr(), (*fx).mOrigin2Y.GetVal(), ax[1].as_ptr(), temp.as_mut_ptr());
                            VectorMA(temp.as_mut_ptr(), (*fx).mOrigin2Z.GetVal(), ax[2].as_ptr(), temp.as_mut_ptr());
                        }
                    }

                    theFxHelper.Trace(
                        &mut tr,
                        org.as_ptr(),
                        ptr::null(),
                        ptr::null(),
                        temp.as_ptr(),
                        -1,
                        MASK_SOLID,
                    );

                    VectorCopy(tr.endpos.as_ptr(), org2.as_mut_ptr());

                    if (*fx).mSpawnFlags & FX_TRACE_IMPACT_FX != 0 {
                        self.PlayEffect(
                            (*fx).mImpactFxHandles.GetHandle(),
                            org2.as_ptr(),
                            ax.as_ptr() as *const [f32; 3],
                            -1,
                            0,
                            -1,
                            -1,
                            -1,
                            false,
                            0,
                            false,
                        );
                    }
                } else {
                    if (*fx).mSpawnFlags & FX_CHEAP_ORG2_CALC != 0 || flags & FX_RELATIVE != 0 {
                        VectorSet(
                            org2.as_mut_ptr(),
                            (*fx).mOrigin2X.GetVal(),
                            (*fx).mOrigin2Y.GetVal(),
                            (*fx).mOrigin2Z.GetVal(),
                        );
                    } else {
                        VectorScale(ax[0].as_ptr(), (*fx).mOrigin2X.GetVal(), org2.as_mut_ptr());
                        VectorMA(org2.as_mut_ptr(), (*fx).mOrigin2Y.GetVal(), ax[1].as_ptr(), org2.as_mut_ptr());
                        VectorMA(org2.as_mut_ptr(), (*fx).mOrigin2Z.GetVal(), ax[2].as_ptr(), org2.as_mut_ptr());
                    }
                    if flags & FX_RELATIVE == 0 {
                        VectorAdd(org2.as_ptr(), origin, org2.as_mut_ptr());
                    }
                }
            } // end special org2 types

            // handle RGB color, but only for types that will use it
            //---------------------------------------------------------------------------
            if (*fx).mType as c_int != EPrimType::Sound as c_int
                && (*fx).mType as c_int != EPrimType::FxRunner as c_int
                && (*fx).mType as c_int != EPrimType::CameraShake as c_int
            {
                GetRGB_Colors(fx, sRGB.as_mut_ptr(), eRGB.as_mut_ptr());
            }

            // Now create the appropriate effect entity
            //------------------------
            match (*fx).mType {
                //---------
                EPrimType::Particle => {
                    //---------
                    FX_AddParticle(
                        org.as_ptr(),
                        vel.as_ptr(),
                        accel.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mRotation.GetVal(),
                        (*fx).mRotationDelta.GetVal(),
                        (*fx).mMin.as_ptr(),
                        (*fx).mMax.as_ptr(),
                        (*fx).mElasticity.GetVal(),
                        (*fx).mDeathFxHandles.GetHandle(),
                        (*fx).mImpactFxHandles.GetHandle(),
                        (*fx).mLife.GetVal() as c_int,
                        (*fx).mMediaHandles.GetHandle(),
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                        iGhoul2,
                        entNum,
                        modelNum,
                        boltNum,
                    );
                }

                //---------
                EPrimType::Line => {
                    //---------
                    FX_AddLine(
                        org.as_ptr(),
                        org2.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mLife.GetVal() as c_int,
                        (*fx).mMediaHandles.GetHandle(),
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                        iGhoul2,
                        entNum,
                        modelNum,
                        boltNum,
                    );
                }

                //---------
                EPrimType::Tail => {
                    //---------
                    FX_AddTail(
                        org.as_ptr(),
                        vel.as_ptr(),
                        accel.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mLengthStart.GetVal(),
                        (*fx).mLengthEnd.GetVal(),
                        (*fx).mLengthParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mMin.as_ptr(),
                        (*fx).mMax.as_ptr(),
                        (*fx).mElasticity.GetVal(),
                        (*fx).mDeathFxHandles.GetHandle(),
                        (*fx).mImpactFxHandles.GetHandle(),
                        (*fx).mLife.GetVal() as c_int,
                        (*fx).mMediaHandles.GetHandle(),
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                        iGhoul2,
                        entNum,
                        modelNum,
                        boltNum,
                    );
                }

                //----------------
                EPrimType::Electricity => {
                    //----------------
                    FX_AddElectricity(
                        org.as_ptr(),
                        org2.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mElasticity.GetVal(),
                        (*fx).mLife.GetVal() as c_int,
                        (*fx).mMediaHandles.GetHandle(),
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                        iGhoul2,
                        entNum,
                        modelNum,
                        boltNum,
                    );
                }

                //---------
                EPrimType::Cylinder => {
                    //---------
                    FX_AddCylinder(
                        org.as_ptr(),
                        ax[0].as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mSize2Start.GetVal(),
                        (*fx).mSize2End.GetVal(),
                        (*fx).mSize2Parm.GetVal(),
                        (*fx).mLengthStart.GetVal(),
                        (*fx).mLengthEnd.GetVal(),
                        (*fx).mLengthParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mLife.GetVal() as c_int,
                        (*fx).mMediaHandles.GetHandle(),
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                        iGhoul2,
                        entNum,
                        modelNum,
                        boltNum,
                        if ((*fx).mSpawnFlags & FX_ORG2_FROM_TRACE) != 0 { 1 } else { 0 },
                    );
                }

                //---------
                EPrimType::Emitter => {
                    //---------
                    // for chunk angles, you don't really need much control over the end result...you just want variation..
                    VectorSet(
                        ang.as_mut_ptr(),
                        (*fx).mAngle1.GetVal(),
                        (*fx).mAngle2.GetVal(),
                        (*fx).mAngle3.GetVal(),
                    );

                    vectoangles(ax[0].as_ptr(), temp.as_mut_ptr());
                    VectorAdd(ang.as_ptr(), temp.as_ptr(), ang.as_mut_ptr());

                    VectorSet(
                        angDelta.as_mut_ptr(),
                        (*fx).mAngle1Delta.GetVal(),
                        (*fx).mAngle2Delta.GetVal(),
                        (*fx).mAngle3Delta.GetVal(),
                    );

                    emitterModel = (*fx).mMediaHandles.GetHandle();

                    FX_AddEmitter(
                        org.as_ptr(),
                        vel.as_ptr(),
                        accel.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        ang.as_ptr(),
                        angDelta.as_ptr(),
                        (*fx).mMin.as_ptr(),
                        (*fx).mMax.as_ptr(),
                        (*fx).mElasticity.GetVal(),
                        (*fx).mDeathFxHandles.GetHandle(),
                        (*fx).mImpactFxHandles.GetHandle(),
                        (*fx).mEmitterFxHandles.GetHandle(),
                        (*fx).mDensity.GetVal(),
                        (*fx).mVariance.GetVal(),
                        (*fx).mLife.GetVal() as c_int,
                        emitterModel,
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                    );
                }

                //---------
                EPrimType::Decal => {
                    //---------
                    theFxHelper.AddDecalToScene(
                        (*fx).mMediaHandles.GetHandle(),
                        org.as_ptr(),
                        ax[0].as_ptr(),
                        (*fx).mRotation.GetVal(),
                        sRGB[0],
                        sRGB[1],
                        sRGB[2],
                        (*fx).mAlphaStart.GetVal(),
                        qtrue,
                        (*fx).mSizeStart.GetVal(),
                        qfalse,
                    );

                    if ((*fx).mFlags & FX_GHOUL2_DECALS) != 0 {
                        theFxHelper.AddGhoul2Decal(
                            (*fx).mMediaHandles.GetHandle(),
                            org.as_ptr(),
                            ax[0].as_ptr(),
                            (*fx).mSizeStart.GetVal(),
                        );
                    }
                }

                //-------------------
                EPrimType::OrientedParticle => {
                    //-------------------
                    FX_AddOrientedParticle(
                        org.as_ptr(),
                        ax[0].as_ptr(),
                        vel.as_ptr(),
                        accel.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mRotation.GetVal(),
                        (*fx).mRotationDelta.GetVal(),
                        (*fx).mMin.as_ptr(),
                        (*fx).mMax.as_ptr(),
                        (*fx).mElasticity.GetVal(),
                        (*fx).mDeathFxHandles.GetHandle(),
                        (*fx).mImpactFxHandles.GetHandle(),
                        (*fx).mLife.GetVal() as c_int,
                        (*fx).mMediaHandles.GetHandle(),
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                        iGhoul2,
                        entNum,
                        modelNum,
                        boltNum,
                    );
                }

                //---------
                EPrimType::Sound => {
                    //---------
                    if gEffectsInPortal != 0 {
                        // could orient this anyway for panning, but eh. It's going to appear to the player in the sky the same place no matter what, so just make it a local sound.
                        theFxHelper.PlayLocalSound((*fx).mMediaHandles.GetHandle(), 0);
                    } else {
                        theFxHelper.PlaySound(
                            org.as_ptr(),
                            ENTITYNUM_NONE,
                            0,
                            (*fx).mMediaHandles.GetHandle(),
                            (*fx).mSoundVolume,
                            (*fx).mSoundRadius,
                        );
                    }
                }

                //---------
                EPrimType::FxRunner => {
                    //---------
                    self.PlayEffect(
                        (*fx).mPlayFxHandles.GetHandle(),
                        org.as_ptr(),
                        ax.as_ptr() as *const [f32; 3],
                        -1,
                        0,
                        -1,
                        -1,
                        -1,
                        false,
                        0,
                        false,
                    );
                }

                //---------
                EPrimType::Light => {
                    //---------
                    FX_AddLight(
                        org.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mLife.GetVal() as c_int,
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                        iGhoul2,
                        entNum,
                        modelNum,
                        boltNum,
                    );
                }

                //---------
                EPrimType::CameraShake => {
                    //---------
                    // It calculates how intense the shake should be based on how close you are to the origin you pass in here
                    // elasticity is actually the intensity...radius is the distance in which the shake will have some effect
                    // life is how long the effect lasts.
                    theFxHelper.CameraShake(
                        org.as_ptr(),
                        (*fx).mElasticity.GetVal(),
                        (*fx).mRadius.GetVal(),
                        (*fx).mLife.GetVal() as c_int,
                    );
                }

                //--------------
                EPrimType::ScreenFlash => {
                    //--------------
                    FX_AddFlash(
                        org.as_ptr(),
                        (*fx).mSizeStart.GetVal(),
                        (*fx).mSizeEnd.GetVal(),
                        (*fx).mSizeParm.GetVal(),
                        (*fx).mAlphaStart.GetVal(),
                        (*fx).mAlphaEnd.GetVal(),
                        (*fx).mAlphaParm.GetVal(),
                        sRGB.as_ptr(),
                        eRGB.as_ptr(),
                        (*fx).mRGBParm.GetVal(),
                        (*fx).mLife.GetVal() as c_int,
                        (*fx).mMediaHandles.GetHandle(),
                        flags,
                        (*fx).mMatImpactFX,
                        fxParm,
                    );
                }

                _ => {
                    assert!(false);
                }
            }

            // Track when we need to clean ourselves up if we are a copy
            if (*fx).mCopy {
                (*fx).mRefCount -= 1;

                if (*fx).mRefCount <= 0 {
                    Box::from_raw(fx);
                }
            }
        }
    }

    //------------------------------------------------------
    // CreateEffect
    //	Creates the fx_runner
    //
    // Input:
    //	template used to build the effect, and the scheduled effect we are based off of
    //
    // Return:
    //	none
    //------------------------------------------------------
    pub fn CreateEffect_from_scheduled(
        &mut self,
        fx: *mut CPrimitiveTemplate,
        scheduledFx: *mut SScheduledEffect,
    ) {
        unsafe {
            let mut boltInfo: c_int = 0;

            // annoying bit....we have to pack the values back into an int before calling playEffect since there isn't the ideal overload we can already use.
            boltInfo = ((*scheduledFx).mModelNum & MODEL_AND) << MODEL_SHIFT;
            boltInfo |= ((*scheduledFx).mBoltNum & BOLT_AND) << BOLT_SHIFT;
            boltInfo |= ((*scheduledFx).mEntNum & ENTITY_AND) << ENTITY_SHIFT;

            self.PlayEffect(
                (*fx).mPlayFxHandles.GetHandle(),
                (*scheduledFx).mOrigin.as_ptr(),
                (*scheduledFx).mAxis.as_ptr() as *const [f32; 3],
                boltInfo,
                0,
                -1,
                -1,
                -1,
                false,
                0,
                false,
            );
        }
    }
}

fn ReportPlayEffectError(id: c_int) {
    unsafe {
        theFxHelper.Print("CFxScheduler::PlayEffect called with invalid effect ID: %i\n", id);
    }
}

fn GetRGB_Colors(fx: *mut CPrimitiveTemplate, outStartRGB: *mut f32, outEndRGB: *mut f32) {
    unsafe {
        let percent = if (*fx).mSpawnFlags & FX_RGB_COMPONENT_INTERP != 0 {
            flrand(0.0, 1.0)
        } else {
            0.0
        };

        if (*fx).mSpawnFlags & FX_RGB_COMPONENT_INTERP != 0 {
            VectorSet(
                outStartRGB,
                (*fx).mRedStart.GetVal_percent(percent),
                (*fx).mGreenStart.GetVal_percent(percent),
                (*fx).mBlueStart.GetVal_percent(percent),
            );
            VectorSet(
                outEndRGB,
                (*fx).mRedEnd.GetVal_percent(percent),
                (*fx).mGreenEnd.GetVal_percent(percent),
                (*fx).mBlueEnd.GetVal_percent(percent),
            );
        } else {
            VectorSet(
                outStartRGB,
                (*fx).mRedStart.GetVal(),
                (*fx).mGreenStart.GetVal(),
                (*fx).mBlueStart.GetVal(),
            );
            VectorSet(
                outEndRGB,
                (*fx).mRedEnd.GetVal(),
                (*fx).mGreenEnd.GetVal(),
                (*fx).mBlueEnd.GetVal(),
            );
        }
    }
}

pub static mut theFxScheduler: CFxScheduler = CFxScheduler {
    mNextFree2DEffect: 0,
    mEffectTemplates: unsafe { std::mem::zeroed() },
    mLoopedEffectArray: unsafe { std::mem::zeroed() },
    mFxSchedule: std::collections::LinkedList::new(),
    mEffectIDs: std::collections::HashMap::new(),
    m2DEffects: unsafe { std::mem::zeroed() },
};

pub static mut totalPrimitives: c_int = 0;
pub static mut totalEffects: c_int = 0;
pub static mut gEffectsInPortal: c_int = 0;

fn libc_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut i = 0;
        loop {
            let c1 = (*s1.add(i) as c_char).to_ascii_lowercase() as c_int;
            let c2 = (*s2.add(i) as c_char).to_ascii_lowercase() as c_int;
            if c1 != c2 {
                return c1 - c2;
            }
            if c1 == 0 {
                return 0;
            }
            i += 1;
        }
    }
}

// Stub for missing extern functions we need to satisfy compiler
#[allow(non_snake_case)]
impl CFxHelper {
    pub fn Print(&self, _fmt: *const c_char, _args: ...) {
        // Stub - actual implementation from engine
    }

    pub fn OpenFile(&self, _filename: *const c_char, _fh: *mut fileHandle_t, _mode: c_int) -> c_int {
        // Stub
        0
    }

    pub fn CloseFile(&self, _fh: fileHandle_t) {
        // Stub
    }

    pub fn ReadFile(&self, _buffer: *mut c_char, _len: c_int, _fh: fileHandle_t) -> c_int {
        // Stub
        0
    }

    pub fn Trace(
        &self,
        _tr: *mut trace_t,
        _start: *const f32,
        _mins: *const f32,
        _maxs: *const f32,
        _end: *const f32,
        _passent: c_int,
        _contentmask: c_int,
    ) {
        // Stub
    }

    pub fn AddDecalToScene(
        &self,
        _handle: c_int,
        _org: *const f32,
        _normal: *const f32,
        _rotation: f32,
        _r: f32,
        _g: f32,
        _b: f32,
        _a: f32,
        _permanent: c_int,
        _size: f32,
        _doLighting: c_int,
    ) {
        // Stub
    }

    pub fn AddGhoul2Decal(
        &self,
        _handle: c_int,
        _org: *const f32,
        _normal: *const f32,
        _size: f32,
    ) {
        // Stub
    }

    pub fn PlayLocalSound(&self, _handle: c_int, _channel: c_int) {
        // Stub
    }

    pub fn PlaySound(
        &self,
        _org: *const f32,
        _entity: c_int,
        _channel: c_int,
        _handle: c_int,
        _volume: c_int,
        _radius: c_int,
    ) {
        // Stub
    }

    pub fn CameraShake(&self, _org: *const f32, _intensity: f32, _radius: f32, _duration: c_int) {
        // Stub
    }

    pub fn GetOriginAxisFromBolt(
        &self,
        _ghoul2: &CGhoul2Info_v,
        _entNum: c_int,
        _modelNum: c_int,
        _boltNum: c_int,
        _origin: *mut f32,
        _axis: *mut [f32; 3],
    ) -> bool {
        // Stub
        false
    }
}

impl CMediaHandles {
    pub fn GetHandle(&self) -> c_int {
        0 // Stub
    }
}

impl CPrimitiveTemplate {
    pub fn ParsePrimitive(&mut self, _group: *mut CGPGroup) {
        // Stub
    }
}

impl CGPValue {
    pub fn GetName(&self) -> *const c_char {
        ptr::null()
    }

    pub fn GetTopValue(&self) -> *const c_char {
        ptr::null()
    }
}

const MASK_SOLID: c_int = 1;

// Helper constants that were defined in the C++ code
const CHAN_AUTO: c_int = 0;
