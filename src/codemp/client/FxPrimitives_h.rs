//! Mechanical port of `codemp/client/FxPrimitives.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_short};
use core::ops::{Deref, DerefMut};

use crate::codemp::game::q_math::{VectorClear, VectorCopy};
use crate::codemp::game::q_shared_h::{byte, qboolean, qhandle_t, vec2_t, vec3_t};

pub type EMatImpactEffect = c_int;

pub const MATIMPACTFX_NONE: EMatImpactEffect = 0;
pub const MATIMPACTFX_SHELLSOUND: EMatImpactEffect = 1;

pub const RT_SPRITE: c_int = 2;

pub const MAX_CPOLY_VERTS: usize = 5;

pub const MAX_EFFECTS: c_int = 1800;

// Generic group flags, used by parser, then get converted to the appropriate specific flags
pub const FX_PARM_MASK: u32 = 0x0000000C; // use this to mask off any transition types that use a parm
pub const FX_GENERIC_MASK: u32 = 0x0000000F;
pub const FX_LINEAR: u32 = 0x00000001;
pub const FX_RAND: u32 = 0x00000002;
pub const FX_NONLINEAR: u32 = 0x00000004;
pub const FX_WAVE: u32 = 0x00000008;
pub const FX_CLAMP: u32 = 0x0000000C;

// Group flags
pub const FX_ALPHA_SHIFT: u32 = 0;
pub const FX_ALPHA_PARM_MASK: u32 = 0x0000000C;
pub const FX_ALPHA_LINEAR: u32 = 0x00000001;
pub const FX_ALPHA_RAND: u32 = 0x00000002;
pub const FX_ALPHA_NONLINEAR: u32 = 0x00000004;
pub const FX_ALPHA_WAVE: u32 = 0x00000008;
pub const FX_ALPHA_CLAMP: u32 = 0x0000000C;

pub const FX_RGB_SHIFT: u32 = 4;
pub const FX_RGB_PARM_MASK: u32 = 0x000000C0;
pub const FX_RGB_LINEAR: u32 = 0x00000010;
pub const FX_RGB_RAND: u32 = 0x00000020;
pub const FX_RGB_NONLINEAR: u32 = 0x00000040;
pub const FX_RGB_WAVE: u32 = 0x00000080;
pub const FX_RGB_CLAMP: u32 = 0x000000C0;

pub const FX_SIZE_SHIFT: u32 = 8;
pub const FX_SIZE_PARM_MASK: u32 = 0x00000C00;
pub const FX_SIZE_LINEAR: u32 = 0x00000100;
pub const FX_SIZE_RAND: u32 = 0x00000200;
pub const FX_SIZE_NONLINEAR: u32 = 0x00000400;
pub const FX_SIZE_WAVE: u32 = 0x00000800;
pub const FX_SIZE_CLAMP: u32 = 0x00000C00;

pub const FX_LENGTH_SHIFT: u32 = 12;
pub const FX_LENGTH_PARM_MASK: u32 = 0x0000C000;
pub const FX_LENGTH_LINEAR: u32 = 0x00001000;
pub const FX_LENGTH_RAND: u32 = 0x00002000;
pub const FX_LENGTH_NONLINEAR: u32 = 0x00004000;
pub const FX_LENGTH_WAVE: u32 = 0x00008000;
pub const FX_LENGTH_CLAMP: u32 = 0x0000C000;

pub const FX_SIZE2_SHIFT: u32 = 16;
pub const FX_SIZE2_PARM_MASK: u32 = 0x000C0000;
pub const FX_SIZE2_LINEAR: u32 = 0x00010000;
pub const FX_SIZE2_RAND: u32 = 0x00020000;
pub const FX_SIZE2_NONLINEAR: u32 = 0x00040000;
pub const FX_SIZE2_WAVE: u32 = 0x00080000;
pub const FX_SIZE2_CLAMP: u32 = 0x000C0000;

// Shared flag--these flags, at first glance would appear to be shared, but are safe.  I'd rather not do this, but as you can see, features flags are currently all accounted for
pub const FX_PAPER_PHYSICS: u32 = 0x00010000; // emitters ONLY.  shared with FX_SIZE_2_LINEAR
pub const FX_LOCALIZED_FLASH: u32 = 0x00010000; // full screen flashes ONLY.  shared with FX_SIZE_2_LINEAR
pub const FX_PLAYER_VIEW: u32 = 0x00010000; // player view effects ONLY.  shared with FX_SIZE_2_LINEAR

// Feature flags
pub const FX_DEPTH_HACK: u32 = 0x00100000;
pub const FX_RELATIVE: u32 = 0x00200000;
pub const FX_SET_SHADER_TIME: u32 = 0x00400000;
pub const FX_EXPENSIVE_PHYSICS: u32 = 0x00800000;

//rww - g2-related flags (these can slow things down significantly, use sparingly)
//These should be used only with particles/decals as they steal flags used by cylinders.
pub const FX_GHOUL2_TRACE: u32 = 0x00020000; //use in conjunction with particles - actually do full ghoul2 traces for physics collision against entities with a ghoul2 instance
//shared FX_SIZE2_RAND (used only with cylinders)
pub const FX_GHOUL2_DECALS: u32 = 0x00040000; //use in conjunction with decals - can project decal as a ghoul2 gore skin object onto ghoul2 models
//shared FX_SIZE2_NONLINEAR (used only with cylinders)

pub const FX_ATTACHED_MODEL: u32 = 0x01000000;

pub const FX_APPLY_PHYSICS: u32 = 0x02000000;
pub const FX_USE_BBOX: u32 = 0x04000000; // can make physics more accurate at the expense of speed

pub const FX_USE_ALPHA: u32 = 0x08000000; // the FX system actually uses RGB to do fades, but this will override that
//	and cause it to fill in the alpha.

pub const FX_EMIT_FX: u32 = 0x10000000; // emitters technically don't have to emit stuff, but when they do
//	this flag needs to be set
pub const FX_DEATH_RUNS_FX: u32 = 0x20000000; // Normal death triggers effect, but not kill_on_impact
pub const FX_KILL_ON_IMPACT: u32 = 0x40000000; // works just like it says, but only when physics are on.
pub const FX_IMPACT_RUNS_FX: u32 = 0x80000000u32; // an effect can call another effect when it hits something.

// Lightning flags, duplicates of existing flags, but lightning doesn't use those flags in that context...and nothing will ever use these in this context..so we are safe.
pub const FX_TAPER: u32 = 0x01000000; // tapers as it moves towards its endpoint
pub const FX_BRANCH: u32 = 0x02000000; // enables lightning branching
pub const FX_GROW: u32 = 0x04000000; // lightning grows from start point to end point over the course of its life

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CGhoul2Info_v {
    pub mItem: c_int,
}

impl CGhoul2Info_v {
    pub fn kill(&mut self) {
        // this scary method zeros the infovector handle without actually freeing it
        // it is used for some places where a copy is made, but we don't want to go through the trouble
        // of making a deep copy
        self.mItem = 0;
    }
}

const _: () = assert!(core::mem::size_of::<CGhoul2Info_v>() == 4);
const _: () = assert!(core::mem::align_of::<CGhoul2Info_v>() == 4);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SFxHelper {
    pub mTime: c_int,
}

impl SFxHelper {
    pub unsafe fn GetTime(&self) -> c_int {
        self.mTime
    }
}

pub static mut theFxHelper: SFxHelper = SFxHelper { mTime: 0 };

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct miniRefEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
    pub hModel: qhandle_t,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: qboolean,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub customShader: qhandle_t,
    pub shaderRGBA: [byte; 4],
    pub shaderTexCoord: vec2_t,
    pub radius: c_float,
    pub rotation: c_float,
    pub shaderTime: c_float,
    pub frame: c_int,
}

const _: () = assert!(core::mem::size_of::<miniRefEntity_t>() == 108);
const _: () = assert!(core::mem::align_of::<miniRefEntity_t>() == 4);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TVert {
    pub origin: vec3_t,
    pub rgb: vec3_t,
    pub destrgb: vec3_t,
    pub curRGB: vec3_t,
    pub alpha: c_float,
    pub destAlpha: c_float,
    pub curAlpha: c_float,
    pub ST: [c_float; 2],
    pub destST: [c_float; 2],
    pub curST: [c_float; 2],
}

const _: () = assert!(core::mem::size_of::<TVert>() == 84);
const _: () = assert!(core::mem::align_of::<TVert>() == 4);

#[repr(C)]
#[derive(Default)]
pub struct CEffect {
    pub mNext: *mut CEffect,
    pub mOrigin1: vec3_t,
    pub mTimeStart: c_int,
    pub mTimeEnd: c_int,
    pub mFlags: u32,
    pub mMatImpactFX: EMatImpactEffect,
    pub mMatImpactParm: c_int,
    pub mMin: vec3_t,
    pub mMax: vec3_t,
    pub mImpactFxID: c_int,
    pub mDeathFxID: c_int,
    pub mRefEnt: miniRefEntity_t,
    pub mSoundRadius: c_int,
    pub mSoundVolume: c_int,
}

impl CEffect {
    pub unsafe fn CEffect() -> Self {
        todo!("CEffect::CEffect body is in FxPrimitives.cpp")
    }

    pub unsafe fn Die(&mut self) {}

    pub unsafe fn Update(&mut self) -> bool {
        true
    }

    pub unsafe fn Draw(&mut self) {}

    pub unsafe fn GetRefEnt(&mut self) -> *mut miniRefEntity_t {
        core::ptr::addr_of_mut!(self.mRefEnt)
    }

    pub unsafe fn SetNext(&mut self, Next: *mut CEffect) {
        self.mNext = Next;
    }

    pub unsafe fn GetNext(&mut self) -> *mut CEffect {
        self.mNext
    }

    pub unsafe fn GetOrigin(&self, dest: *mut vec3_t) {
        VectorCopy(&self.mOrigin1, &mut *dest);
    }

    pub unsafe fn SetSTScale(&mut self, s: c_float, t: c_float) {
        self.mRefEnt.shaderTexCoord[0] = s;
        self.mRefEnt.shaderTexCoord[1] = t;
    }

    pub unsafe fn SetSound(&mut self, vol: c_int, rad: c_int) {
        self.mSoundRadius = rad;
        self.mSoundVolume = vol;
    }

    pub unsafe fn SetMin(&mut self, min: *mut vec3_t) {
        if min.is_null() {
            VectorClear(&mut self.mMin);
        } else {
            VectorCopy(&*min, &mut self.mMin);
        }
    }

    pub unsafe fn SetMax(&mut self, max: *mut vec3_t) {
        if max.is_null() {
            VectorClear(&mut self.mMax);
        } else {
            VectorCopy(&*max, &mut self.mMax);
        }
    }

    pub unsafe fn SetFlags(&mut self, flags: c_int) {
        self.mFlags = flags as u32;
    }

    pub unsafe fn AddFlags(&mut self, flags: c_int) {
        self.mFlags |= flags as u32;
    }

    pub unsafe fn ClearFlags(&mut self, flags: c_int) {
        self.mFlags &= !(flags as u32);
    }

    pub unsafe fn SetOrigin1(&mut self, org: *mut vec3_t) {
        if org.is_null() {
            VectorClear(&mut self.mOrigin1);
        } else {
            VectorCopy(&*org, &mut self.mOrigin1);
        }
    }

    pub unsafe fn SetTimeStart(&mut self, time: c_int) {
        self.mTimeStart = time;
        if (self.mFlags & FX_SET_SHADER_TIME) != 0 {
            self.mRefEnt.shaderTime = time as c_float * 0.001;
        }
    }

    pub unsafe fn SetTimeEnd(&mut self, time: c_int) {
        self.mTimeEnd = time;
    }

    pub unsafe fn SetImpactFxID(&mut self, id: c_int) {
        self.mImpactFxID = id;
    }

    pub unsafe fn SetDeathFxID(&mut self, id: c_int) {
        self.mDeathFxID = id;
    }

    pub unsafe fn GetMatImpactFX(&self) -> EMatImpactEffect {
        self.mMatImpactFX
    }

    pub unsafe fn GetMatImpactParm(&self) -> c_int {
        self.mMatImpactParm
    }

    pub unsafe fn SetMatImpactFX(&mut self, matFX: EMatImpactEffect) {
        self.mMatImpactFX = matFX;
    }

    pub unsafe fn SetMatImpactParm(&mut self, matParm: c_int) {
        self.mMatImpactParm = matParm;
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CParticle {
    pub base: CEffect,
    pub mOrgOffset: vec3_t,
    pub mVel: vec3_t,
    pub mAccel: vec3_t,
    pub mSizeStart: c_float,
    pub mSizeEnd: c_float,
    pub mSizeParm: c_float,
    pub mRGBStart: vec3_t,
    pub mRGBEnd: vec3_t,
    pub mRGBParm: c_float,
    pub mAlphaStart: c_float,
    pub mAlphaEnd: c_float,
    pub mAlphaParm: c_float,
    pub mRotationDelta: c_float,
    pub mElasticity: c_float,
    pub mGhoul2: CGhoul2Info_v,
    pub mEntNum: c_short,
    pub mModelNum: c_char,
    pub mBoltNum: c_char,
}

impl Deref for CParticle {
    type Target = CEffect;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CParticle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Drop for CParticle {
    fn drop(&mut self) {
        self.mGhoul2.kill();
    }
}

impl CParticle {
    pub unsafe fn CParticle() -> Self {
        let mut this = Self::default();
        this.base.mRefEnt.reType = RT_SPRITE;
        this.mEntNum = -1;
        this.mModelNum = -1 as c_char;
        this.mBoltNum = -1 as c_char;
        this
    }

    pub unsafe fn SetBoltinfo(&mut self, iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int) {
        self.mGhoul2.mItem = iGhoul2;
        self.mEntNum = entNum as c_short;
        self.mModelNum = modelNum as c_char;
        self.mBoltNum = boltNum as c_char;
    }

    pub unsafe fn Init(&mut self) {
        todo!("CParticle::Init body is in FxPrimitives.cpp")
    }

    pub unsafe fn Die(&mut self) {
        todo!("CParticle::Die body is in FxPrimitives.cpp")
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CParticle::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn Cull(&mut self) -> bool {
        todo!("CParticle::Cull body is in FxPrimitives.cpp")
    }

    pub unsafe fn Draw(&mut self) {
        todo!("CParticle::Draw body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetShader(&mut self, sh: qhandle_t) {
        self.base.mRefEnt.customShader = sh;
    }

    pub unsafe fn SetOrgOffset(&mut self, o: *mut vec3_t) {
        if o.is_null() {
            VectorClear(&mut self.mOrgOffset);
        } else {
            VectorCopy(&*o, &mut self.mOrgOffset);
        }
    }

    pub unsafe fn SetVel(&mut self, vel: *mut vec3_t) {
        if vel.is_null() {
            VectorClear(&mut self.mVel);
        } else {
            VectorCopy(&*vel, &mut self.mVel);
        }
    }

    pub unsafe fn SetAccel(&mut self, ac: *mut vec3_t) {
        if ac.is_null() {
            VectorClear(&mut self.mAccel);
        } else {
            VectorCopy(&*ac, &mut self.mAccel);
        }
    }

    pub unsafe fn SetSizeStart(&mut self, sz: c_float) {
        self.mSizeStart = sz;
        self.base.mRefEnt.radius = sz;
    }

    pub unsafe fn SetSizeEnd(&mut self, sz: c_float) {
        self.mSizeEnd = sz;
    }

    pub unsafe fn SetSizeParm(&mut self, parm: c_float) {
        self.mSizeParm = parm;
    }

    pub unsafe fn SetRGBStart(&mut self, rgb: *mut vec3_t) {
        if rgb.is_null() {
            VectorClear(&mut self.mRGBStart);
        } else {
            VectorCopy(&*rgb, &mut self.mRGBStart);
        }
    }

    pub unsafe fn SetRGBEnd(&mut self, rgb: *mut vec3_t) {
        if rgb.is_null() {
            VectorClear(&mut self.mRGBEnd);
        } else {
            VectorCopy(&*rgb, &mut self.mRGBEnd);
        }
    }

    pub unsafe fn SetRGBParm(&mut self, parm: c_float) {
        self.mRGBParm = parm;
    }

    pub unsafe fn SetAlphaStart(&mut self, al: c_float) {
        self.mAlphaStart = al;
    }

    pub unsafe fn SetAlphaEnd(&mut self, al: c_float) {
        self.mAlphaEnd = al;
    }

    pub unsafe fn SetAlphaParm(&mut self, parm: c_float) {
        self.mAlphaParm = parm;
    }

    pub unsafe fn SetRotation(&mut self, rot: c_float) {
        self.base.mRefEnt.rotation = rot;
    }

    pub unsafe fn SetRotationDelta(&mut self, rot: c_float) {
        self.mRotationDelta = rot;
    }

    pub unsafe fn SetElasticity(&mut self, el: c_float) {
        self.mElasticity = el;
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CLine {
    pub base: CParticle,
    pub mOrigin2: vec3_t,
}

impl Deref for CLine {
    type Target = CParticle;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CLine {
    pub unsafe fn CLine() -> Self {
        todo!("CLine::CLine body is in FxPrimitives.cpp")
    }

    pub unsafe fn Die(&mut self) {}

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CLine::Update body is in FxPrimitives.cpp")
    }

    unsafe fn Draw(&mut self) {
        todo!("CLine::Draw body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetOrigin2(&mut self, org2: *mut vec3_t) {
        VectorCopy(&*org2, &mut self.mOrigin2);
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CBezier {
    pub base: CLine,
    pub mControl1: vec3_t,
    pub mControl1Vel: vec3_t,
    pub mControl2: vec3_t,
    pub mControl2Vel: vec3_t,
    pub mInit: bool,
}

impl Deref for CBezier {
    type Target = CLine;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CBezier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CBezier {
    pub unsafe fn CBezier() -> Self {
        let mut this = Self::default();
        this.mInit = false;
        this
    }

    pub unsafe fn Die(&mut self) {}

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CBezier::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn Cull(&mut self) -> bool {
        todo!("CBezier::Cull body is in FxPrimitives.cpp")
    }

    pub unsafe fn Draw(&mut self) {
        todo!("CBezier::Draw body is in FxPrimitives.cpp")
    }

    pub unsafe fn DrawSegment(
        &mut self,
        _start: *mut vec3_t,
        _end: *mut vec3_t,
        _texcoord1: c_float,
        _texcoord2: c_float,
        _segPercent: c_float,
        _lastSegPercent: c_float,
    ) {
        todo!("CBezier::DrawSegment body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetControlPoints(&mut self, ctrl1: *mut vec3_t, ctrl2: *mut vec3_t) {
        VectorCopy(&*ctrl1, &mut self.mControl1);
        VectorCopy(&*ctrl2, &mut self.mControl2);
    }

    pub unsafe fn SetControlVel(&mut self, ctrl1v: *mut vec3_t, ctrl2v: *mut vec3_t) {
        VectorCopy(&*ctrl1v, &mut self.mControl1Vel);
        VectorCopy(&*ctrl2v, &mut self.mControl2Vel);
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CElectricity {
    pub base: CLine,
    pub mChaos: c_float,
}

impl Deref for CElectricity {
    type Target = CLine;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CElectricity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CElectricity {
    pub unsafe fn CElectricity() -> Self {
        todo!("CElectricity::CElectricity body is in FxPrimitives.cpp")
    }

    pub unsafe fn Die(&mut self) {}

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CElectricity::Update body is in FxPrimitives.cpp")
    }

    unsafe fn Draw(&mut self) {
        todo!("CElectricity::Draw body is in FxPrimitives.cpp")
    }

    pub unsafe fn Initialize(&mut self) {
        todo!("CElectricity::Initialize body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetChaos(&mut self, chaos: c_float) {
        self.mChaos = chaos;
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CTail {
    pub base: CParticle,
    pub mOldOrigin: vec3_t,
    pub mLengthStart: c_float,
    pub mLengthEnd: c_float,
    pub mLengthParm: c_float,
    pub mLength: c_float,
}

impl Deref for CTail {
    type Target = CParticle;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CTail {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CTail {
    pub unsafe fn CTail() -> Self {
        todo!("CTail::CTail body is in FxPrimitives.cpp")
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CTail::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetLengthStart(&mut self, len: c_float) {
        self.mLengthStart = len;
    }

    pub unsafe fn SetLengthEnd(&mut self, len: c_float) {
        self.mLengthEnd = len;
    }

    pub unsafe fn SetLengthParm(&mut self, len: c_float) {
        self.mLengthParm = len;
    }

    unsafe fn UpdateLength(&mut self) {
        todo!("CTail::UpdateLength body is in FxPrimitives.cpp")
    }

    unsafe fn CalcNewEndpoint(&mut self) {
        todo!("CTail::CalcNewEndpoint body is in FxPrimitives.cpp")
    }

    unsafe fn Draw(&mut self) {
        todo!("CTail::Draw body is in FxPrimitives.cpp")
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CCylinder {
    pub base: CTail,
    pub mSize2Start: c_float,
    pub mSize2End: c_float,
    pub mSize2Parm: c_float,
    pub mTraceEnd: qboolean,
}

impl Deref for CCylinder {
    type Target = CTail;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CCylinder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CCylinder {
    pub unsafe fn CCylinder() -> Self {
        todo!("CCylinder::CCylinder body is in FxPrimitives.cpp")
    }

    pub unsafe fn Cull(&mut self) -> bool {
        todo!("CCylinder::Cull body is in FxPrimitives.cpp")
    }

    pub unsafe fn UpdateLength(&mut self) {
        todo!("CCylinder::UpdateLength body is in FxPrimitives.cpp")
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CCylinder::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetSize2Start(&mut self, sz: c_float) {
        self.mSize2Start = sz;
    }

    pub unsafe fn SetSize2End(&mut self, sz: c_float) {
        self.mSize2End = sz;
    }

    pub unsafe fn SetSize2Parm(&mut self, parm: c_float) {
        self.mSize2Parm = parm;
    }

    pub unsafe fn SetTraceEnd(&mut self, traceEnd: qboolean) {
        self.mTraceEnd = traceEnd;
    }

    pub unsafe fn SetNormal(&mut self, norm: *mut vec3_t) {
        VectorCopy(&*norm, &mut self.base.base.base.mRefEnt.axis[0]);
    }

    unsafe fn UpdateSize2(&mut self) {
        todo!("CCylinder::UpdateSize2 body is in FxPrimitives.cpp")
    }

    unsafe fn Draw(&mut self) {
        todo!("CCylinder::Draw body is in FxPrimitives.cpp")
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CEmitter {
    pub base: CParticle,
    pub mOldOrigin: vec3_t,
    pub mLastOrigin: vec3_t,
    pub mOldVelocity: vec3_t,
    pub mOldTime: c_int,
    pub mAngles: vec3_t,
    pub mAngleDelta: vec3_t,
    pub mEmitterFxID: c_int,
    pub mDensity: c_float,
    pub mVariance: c_float,
}

impl Deref for CEmitter {
    type Target = CParticle;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CEmitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Drop for CEmitter {
    fn drop(&mut self) {
        todo!("CEmitter::~CEmitter body is in FxPrimitives.cpp")
    }
}

impl CEmitter {
    pub unsafe fn CEmitter() -> Self {
        todo!("CEmitter::CEmitter body is in FxPrimitives.cpp")
    }

    pub unsafe fn Cull(&mut self) -> bool {
        false
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CEmitter::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetModel(&mut self, model: qhandle_t) {
        self.base.base.mRefEnt.hModel = model;
    }

    pub unsafe fn SetAngles(&mut self, ang: *mut vec3_t) {
        if ang.is_null() {
            VectorClear(&mut self.mAngles);
        } else {
            VectorCopy(&*ang, &mut self.mAngles);
        }
    }

    pub unsafe fn SetAngleDelta(&mut self, ang: *mut vec3_t) {
        if ang.is_null() {
            VectorClear(&mut self.mAngleDelta);
        } else {
            VectorCopy(&*ang, &mut self.mAngleDelta);
        }
    }

    pub unsafe fn SetEmitterFxID(&mut self, id: c_int) {
        self.mEmitterFxID = id;
    }

    pub unsafe fn SetDensity(&mut self, density: c_float) {
        self.mDensity = density;
    }

    pub unsafe fn SetVariance(&mut self, var: c_float) {
        self.mVariance = var;
    }

    pub unsafe fn SetOldTime(&mut self, time: c_int) {
        self.mOldTime = time;
    }

    pub unsafe fn SetLastOrg(&mut self, org: *mut vec3_t) {
        if org.is_null() {
            VectorClear(&mut self.mLastOrigin);
        } else {
            VectorCopy(&*org, &mut self.mLastOrigin);
        }
    }

    pub unsafe fn SetLastVel(&mut self, vel: *mut vec3_t) {
        if vel.is_null() {
            VectorClear(&mut self.mOldVelocity);
        } else {
            VectorCopy(&*vel, &mut self.mOldVelocity);
        }
    }

    unsafe fn UpdateAngles(&mut self) {
        todo!("CEmitter::UpdateAngles body is in FxPrimitives.cpp")
    }

    unsafe fn Draw(&mut self) {
        todo!("CEmitter::Draw body is in FxPrimitives.cpp")
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CLight {
    pub base: CEffect,
    pub mSizeStart: c_float,
    pub mSizeEnd: c_float,
    pub mSizeParm: c_float,
    pub mOrgOffset: vec3_t,
    pub mRGBStart: vec3_t,
    pub mRGBEnd: vec3_t,
    pub mRGBParm: c_float,
    pub mGhoul2: CGhoul2Info_v,
    pub mEntNum: c_short,
    pub mModelNum: c_char,
    pub mBoltNum: c_char,
}

impl Deref for CLight {
    type Target = CEffect;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CLight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Drop for CLight {
    fn drop(&mut self) {
        self.mGhoul2.kill();
    }
}

impl CLight {
    pub unsafe fn CLight() -> Self {
        let mut this = Self::default();
        this.mEntNum = -1;
        this.mModelNum = -1 as c_char;
        this.mBoltNum = -1 as c_char;
        this
    }

    pub unsafe fn SetBoltinfo(&mut self, iGhoul2: c_int, entNum: c_int, modelNum: c_int, boltNum: c_int) {
        self.mGhoul2.mItem = iGhoul2;
        self.mEntNum = entNum as c_short;
        self.mModelNum = modelNum as c_char;
        self.mBoltNum = boltNum as c_char;
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CLight::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetSizeStart(&mut self, sz: c_float) {
        self.mSizeStart = sz;
    }

    pub unsafe fn SetSizeEnd(&mut self, sz: c_float) {
        self.mSizeEnd = sz;
    }

    pub unsafe fn SetSizeParm(&mut self, parm: c_float) {
        self.mSizeParm = parm;
    }

    pub unsafe fn SetOrgOffset(&mut self, o: *mut vec3_t) {
        if o.is_null() {
            VectorClear(&mut self.mOrgOffset);
        } else {
            VectorCopy(&*o, &mut self.mOrgOffset);
        }
    }

    pub unsafe fn SetRGBStart(&mut self, rgb: *mut vec3_t) {
        if rgb.is_null() {
            VectorClear(&mut self.mRGBStart);
        } else {
            VectorCopy(&*rgb, &mut self.mRGBStart);
        }
    }

    pub unsafe fn SetRGBEnd(&mut self, rgb: *mut vec3_t) {
        if rgb.is_null() {
            VectorClear(&mut self.mRGBEnd);
        } else {
            VectorCopy(&*rgb, &mut self.mRGBEnd);
        }
    }

    pub unsafe fn SetRGBParm(&mut self, parm: c_float) {
        self.mRGBParm = parm;
    }

    unsafe fn UpdateSize(&mut self) {
        todo!("CLight::UpdateSize body is in FxPrimitives.cpp")
    }

    unsafe fn UpdateRGB(&mut self) {
        todo!("CLight::UpdateRGB body is in FxPrimitives.cpp")
    }

    unsafe fn Draw(&mut self) {
        todo!("CLight::Draw body is in FxPrimitives.cpp")
    }
}

#[repr(C)]
#[derive(Default)]
pub struct COrientedParticle {
    pub base: CParticle,
    pub mNormal: vec3_t,
}

impl Deref for COrientedParticle {
    type Target = CParticle;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for COrientedParticle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl COrientedParticle {
    pub unsafe fn COrientedParticle() -> Self {
        todo!("COrientedParticle::COrientedParticle body is in FxPrimitives.cpp")
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("COrientedParticle::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn Cull(&mut self) -> bool {
        todo!("COrientedParticle::Cull body is in FxPrimitives.cpp")
    }

    pub unsafe fn Draw(&mut self) {
        todo!("COrientedParticle::Draw body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetNormal(&mut self, norm: *mut vec3_t) {
        VectorCopy(&*norm, &mut self.mNormal);
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CPoly {
    pub base: CParticle,
    pub mCount: c_int,
    pub mRotDelta: vec3_t,
    pub mTimeStamp: c_int,
    pub mOrg: [vec3_t; MAX_CPOLY_VERTS],
    pub mST: [vec2_t; MAX_CPOLY_VERTS],
    pub mRot: [[c_float; 3]; 3],
    pub mLastFrameTime: c_int,
}

impl Deref for CPoly {
    type Target = CParticle;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CPoly {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CPoly {
    pub unsafe fn CPoly() -> Self {
        Self::default()
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CPoly::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn Cull(&mut self) -> bool {
        todo!("CPoly::Cull body is in FxPrimitives.cpp")
    }

    pub unsafe fn Draw(&mut self) {
        todo!("CPoly::Draw body is in FxPrimitives.cpp")
    }

    pub unsafe fn PolyInit(&mut self) {
        todo!("CPoly::PolyInit body is in FxPrimitives.cpp")
    }

    pub unsafe fn CalcRotateMatrix(&mut self) {
        todo!("CPoly::CalcRotateMatrix body is in FxPrimitives.cpp")
    }

    pub unsafe fn Rotate(&mut self) {
        todo!("CPoly::Rotate body is in FxPrimitives.cpp")
    }

    pub unsafe fn SetNumVerts(&mut self, c: c_int) {
        self.mCount = c;
    }

    pub unsafe fn SetRot(&mut self, r: *mut vec3_t) {
        if r.is_null() {
            VectorClear(&mut self.mRotDelta);
        } else {
            VectorCopy(&*r, &mut self.mRotDelta);
        }
    }

    pub unsafe fn SetMotionTimeStamp(&mut self, t: c_int) {
        self.mTimeStamp = unsafe { (*core::ptr::addr_of!(theFxHelper)).GetTime() + t };
    }

    pub unsafe fn GetMotionTimeStamp(&mut self) -> c_int {
        self.mTimeStamp
    }

    pub unsafe fn DrawSegment(
        &mut self,
        _start: *mut vec3_t,
        _end: *mut vec3_t,
        _texcoord1: c_float,
        _texcoord2: c_float,
        _segPercent: c_float,
        _lastSegPercent: c_float,
    ) {
        todo!("CPoly::DrawSegment body is in FxPrimitives.cpp")
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CFlash {
    pub base: CParticle,
    pub mScreenX: c_float,
    pub mScreenY: c_float,
    pub mRadiusModifier: c_float,
}

impl Deref for CFlash {
    type Target = CParticle;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CFlash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CFlash {
    pub unsafe fn CFlash() -> Self {
        let mut this = Self::default();
        this.mRadiusModifier = 1.0;
        this
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CFlash::Update body is in FxPrimitives.cpp")
    }

    pub unsafe fn Draw(&mut self) {
        todo!("CFlash::Draw body is in FxPrimitives.cpp")
    }

    pub unsafe fn Cull(&mut self) -> bool {
        false
    }

    pub unsafe fn Init(&mut self) {
        todo!("CFlash::Init body is in FxPrimitives.cpp")
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CTrail {
    pub base: CEffect,
    pub mVerts: [TVert; 4],
    pub mShader: qhandle_t,
}

impl Deref for CTrail {
    type Target = CEffect;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CTrail {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CTrail {
    pub unsafe fn CTrail() -> Self {
        Self::default()
    }

    pub unsafe fn Update(&mut self) -> bool {
        todo!("CTrail::Update body is in FxPrimitives.cpp")
    }

    unsafe fn Draw(&mut self) {
        todo!("CTrail::Draw body is in FxPrimitives.cpp")
    }
}
