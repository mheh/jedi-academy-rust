// Preserve original header guard logic
// #if !defined(FX_SYSTEM_H_INC)
//	#include "FxSystem.h"
// #endif

// (FxSystem.h would be imported via module dependencies, not explicitly here)

// #ifndef FX_PRIMITIVES_H_INC
// #define FX_PRIMITIVES_H_INC

use core::ffi::{c_int, c_char};

pub const MAX_EFFECTS: usize = 1200;

// Generic group flags, used by parser, then get converted to the appropriate specific flags
pub const FX_PARM_MASK: u32 = 0xC;		// use this to mask off any transition types that use a parm
pub const FX_GENERIC_MASK: u32 = 0xF;
pub const FX_LINEAR: u32 = 0x1;
pub const FX_RAND: u32 = 0x2;
pub const FX_NONLINEAR: u32 = 0x4;
pub const FX_WAVE: u32 = 0x8;
pub const FX_CLAMP: u32 = 0xC;

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

// Feature flags
pub const FX_DEPTH_HACK: u32 = 0x00100000;
pub const FX_RELATIVE: u32 = 0x00200000;
pub const FX_SET_SHADER_TIME: u32 = 0x00400000;		// by having the effects system set the shader time, we can make animating textures start at the correct time
pub const FX_EXPENSIVE_PHYSICS: u32 = 0x00800000;

// rww - g2-related flags (these can slow things down significantly, use sparingly)
// These should be used only with particles/decals as they steal flags used by cylinders.
pub const FX_GHOUL2_TRACE: u32 = 0x00020000;		// use in conjunction with particles - actually do full ghoul2 traces for physics collision against entities with a ghoul2 instance
									// shared FX_SIZE2_RAND (used only with cylinders)
pub const FX_GHOUL2_DECALS: u32 = 0x00040000;		// use in conjunction with decals - can project decal as a ghoul2 gore skin object onto ghoul2 models
									// shared FX_SIZE2_NONLINEAR (used only with cylinders)

pub const FX_ATTACHED_MODEL: u32 = 0x01000000;

pub const FX_APPLY_PHYSICS: u32 = 0x02000000;
pub const FX_USE_BBOX: u32 = 0x04000000;		// can make physics more accurate at the expense of speed

pub const FX_USE_ALPHA: u32 = 0x08000000;		// the FX system actually uses RGB to do fades, but this will override that
													// and cause it to fill in the alpha.

pub const FX_EMIT_FX: u32 = 0x10000000;		// emitters technically don't have to emit stuff, but when they do
													// this flag needs to be set
pub const FX_DEATH_RUNS_FX: u32 = 0x20000000;		// Normal death triggers effect, but not kill_on_impact
pub const FX_KILL_ON_IMPACT: u32 = 0x40000000;		// works just like it says, but only when physics are on.
pub const FX_IMPACT_RUNS_FX: u32 = 0x80000000;		// an effect can call another effect when it hits something.

// Lightning flags, duplicates of existing flags, but lightning doesn't use those flags in that context...and nothing will ever use these in this context..so we are safe.
pub const FX_TAPER: u32 = 0x01000000;		// tapers as it moves towards its endpoint
pub const FX_BRANCH: u32 = 0x02000000;		// enables lightning branching
pub const FX_GROW: u32 = 0x04000000;		// lightning grows from start point to end point over the course of its life

//------------------------------
// Base CEffect class - represents a single effect in the system
pub struct CEffect {
	// Protected members (not directly exposed in Rust public API)
	pub mOrigin1: [f32; 3],			// vec3_t

	pub mTimeStart: c_int,
	pub mTimeEnd: c_int,

	pub mFlags: u32,

	// Size of our object, useful for things that have physics
	pub mMin: [f32; 3],				// vec3_t
	pub mMax: [f32; 3],				// vec3_t

	pub mImpactFxID: c_int,			// if we have an impact event, we may have to call an effect
	pub mDeathFxID: c_int,				// if we have a death event, we may have to call an effect

	pub mRefEnt: refEntity_t,
}

// Type stub for refEntity_t - would be defined in renderer module
#[repr(C)]
pub struct refEntity_t {
	// Placeholder structure - actual fields would come from renderer module
	_marker: std::marker::PhantomData<()>,
}

impl CEffect {
	pub fn new() -> Self {
		// C++ constructor: memset( &mRefEnt, 0, sizeof( refEntity_t ));
		// Rust default-initializes to zeros
		CEffect {
			mOrigin1: [0.0; 3],
			mTimeStart: 0,
			mTimeEnd: 0,
			mFlags: 0,
			mMin: [0.0; 3],
			mMax: [0.0; 3],
			mImpactFxID: 0,
			mDeathFxID: 0,
			mRefEnt: refEntity_t {
				_marker: std::marker::PhantomData,
			},
		}
	}

	pub fn Die(&mut self) {
		// Virtual method in C++; no-op in base class
	}

	pub fn Update(&mut self) -> bool {
		// Game pausing can cause dumb time things to happen, so kill the effect in this instance
		if self.mTimeStart > unsafe { crate::theFxHelper.mTime } {
			return false;
		}
		true
	}

	// inline void SetSTScale(float s,float t)	{ mRefEnt.shaderTexCoord[0]=s;mRefEnt.shaderTexCoord[1]=t;}
	pub fn SetSTScale(&mut self, s: f32, t: f32) {
		// Stub: shaderTexCoord access would depend on refEntity_t definition
		// self.mRefEnt.shaderTexCoord[0] = s;
		// self.mRefEnt.shaderTexCoord[1] = t;
	}

	// inline void SetMin( const vec3_t min )		{ if(min){VectorCopy(min,mMin);}else{VectorClear(mMin);}			}
	pub fn SetMin(&mut self, min: Option<&[f32; 3]>) {
		if let Some(m) = min {
			self.mMin = *m;
		} else {
			self.mMin = [0.0; 3];
		}
	}

	// inline void SetMax( const vec3_t max )		{ if(max){VectorCopy(max,mMax);}else{VectorClear(mMax);}			}
	pub fn SetMax(&mut self, max: Option<&[f32; 3]>) {
		if let Some(m) = max {
			self.mMax = *m;
		} else {
			self.mMax = [0.0; 3];
		}
	}

	// inline void SetFlags( int flags )		{ mFlags = flags;				}
	pub fn SetFlags(&mut self, flags: u32) {
		self.mFlags = flags;
	}

	// inline void AddFlags( int flags )		{ mFlags |= flags;				}
	pub fn AddFlags(&mut self, flags: u32) {
		self.mFlags |= flags;
	}

	// inline void ClearFlags( int flags )		{ mFlags &= ~flags;				}
	pub fn ClearFlags(&mut self, flags: u32) {
		self.mFlags &= !flags;
	}

	// inline void SetOrigin1( const vec3_t org )	{ if(org){VectorCopy(org,mOrigin1);}else{VectorClear(mOrigin1);}	}
	pub fn SetOrigin1(&mut self, org: Option<&[f32; 3]>) {
		if let Some(o) = org {
			self.mOrigin1 = *o;
		} else {
			self.mOrigin1 = [0.0; 3];
		}
	}

	// inline void SetTimeStart( int time )	{ mTimeStart = time; if (mFlags&FX_SET_SHADER_TIME) { mRefEnt.shaderTime = cg.time * 0.001f; }}
	pub fn SetTimeStart(&mut self, time: c_int) {
		self.mTimeStart = time;
		if (self.mFlags & FX_SET_SHADER_TIME) != 0 {
			// mRefEnt.shaderTime = cg.time * 0.001f; -- depends on cg and refEntity_t definition
		}
	}

	// inline void	SetTimeEnd( int time )		{ mTimeEnd = time;				}
	pub fn SetTimeEnd(&mut self, time: c_int) {
		self.mTimeEnd = time;
	}

	// inline void SetImpactFxID( int id )		{ mImpactFxID = id;				}
	pub fn SetImpactFxID(&mut self, id: c_int) {
		self.mImpactFxID = id;
	}

	// inline void SetDeathFxID( int id )		{ mDeathFxID = id;				}
	pub fn SetDeathFxID(&mut self, id: c_int) {
		self.mDeathFxID = id;
	}
}

//---------------------------------------------------
// This class is kind of an exception to the "rule".
//	For now it exists only for allowing an easy way
//	to get the saber slash trails rendered.
//---------------------------------------------------
pub struct CTrail {
	// Inherits from CEffect
	pub base: CEffect,

	// This is such a specific case thing, just grant public access to the goods.
	pub mVerts: [TVert; 4],
	pub mShader: qhandle_t,
}

pub type qhandle_t = c_int;

pub struct TVert {
	pub origin: [f32; 3],				// vec3_t

	// very specifc case, we can modulate the color and the alpha
	pub rgb: [f32; 3],					// vec3_t
	pub destrgb: [f32; 3],				// vec3_t
	pub curRGB: [f32; 3],				// vec3_t

	pub alpha: f32,
	pub destAlpha: f32,
	pub curAlpha: f32,

	// this is a very specific case thing...allow interpolating the st coords so we can map the texture
	//	properly as this segement progresses through it's life
	pub ST: [f32; 2],					// float[2]
	pub destST: [f32; 2],				// float[2]
	pub curST: [f32; 2],				// float[2]
}

impl CTrail {
	pub fn new() -> Self {
		CTrail {
			base: CEffect::new(),
			mVerts: [TVert::new(); 4],
			mShader: 0,
		}
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}
}

impl TVert {
	pub fn new() -> Self {
		TVert {
			origin: [0.0; 3],
			rgb: [0.0; 3],
			destrgb: [0.0; 3],
			curRGB: [0.0; 3],
			alpha: 0.0,
			destAlpha: 0.0,
			curAlpha: 0.0,
			ST: [0.0; 2],
			destST: [0.0; 2],
			curST: [0.0; 2],
		}
	}
}

//------------------------------
pub struct CLight {
	// Inherits from CEffect
	pub base: CEffect,

	pub mSizeStart: f32,
	pub mSizeEnd: f32,
	pub mSizeParm: f32,

	pub mRGBStart: [f32; 3],			// vec3_t
	pub mRGBEnd: [f32; 3],				// vec3_t
	pub mRGBParm: f32,
}

impl CLight {
	pub fn new() -> Self {
		CLight {
			base: CEffect::new(),
			mSizeStart: 0.0,
			mSizeEnd: 0.0,
			mSizeParm: 0.0,
			mRGBStart: [0.0; 3],
			mRGBEnd: [0.0; 3],
			mRGBParm: 0.0,
		}
	}

	fn UpdateSize(&mut self) {
		// Protected method stub
	}

	fn UpdateRGB(&mut self) {
		// Protected method stub
	}

	fn Draw(&self) {
		// theFxHelper.AddLightToScene( mOrigin1, mRefEnt.radius,
		// 	mRefEnt.lightingOrigin[0], mRefEnt.lightingOrigin[1], mRefEnt.lightingOrigin[2] );
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetSizeStart( float sz )	{ mSizeStart = sz;			}
	pub fn SetSizeStart(&mut self, sz: f32) {
		self.mSizeStart = sz;
	}

	// inline void SetSizeEnd( float sz )		{ mSizeEnd = sz;			}
	pub fn SetSizeEnd(&mut self, sz: f32) {
		self.mSizeEnd = sz;
	}

	// inline void SetSizeParm( float parm )	{ mSizeParm = parm;			}
	pub fn SetSizeParm(&mut self, parm: f32) {
		self.mSizeParm = parm;
	}

	// inline void SetRGBStart( vec3_t rgb )	{ if(rgb){VectorCopy(rgb,mRGBStart);}else{VectorClear(mRGBStart);}	}
	pub fn SetRGBStart(&mut self, rgb: Option<&[f32; 3]>) {
		if let Some(r) = rgb {
			self.mRGBStart = *r;
		} else {
			self.mRGBStart = [0.0; 3];
		}
	}

	// inline void SetRGBEnd( vec3_t rgb )		{ if(rgb){VectorCopy(rgb,mRGBEnd);}else{VectorClear(mRGBEnd);}		}
	pub fn SetRGBEnd(&mut self, rgb: Option<&[f32; 3]>) {
		if let Some(r) = rgb {
			self.mRGBEnd = *r;
		} else {
			self.mRGBEnd = [0.0; 3];
		}
	}

	// inline void SetRGBParm( float parm )	{ mRGBParm = parm;			}
	pub fn SetRGBParm(&mut self, parm: f32) {
		self.mRGBParm = parm;
	}
}

//------------------------------
pub struct CFlash {
	// Inherits from CLight
	pub base: CLight,
}

impl CFlash {
	pub fn new() -> Self {
		CFlash {
			base: CLight::new(),
		}
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetShader( qhandle_t sh )
	// {	assert(sh);
	// 	mRefEnt.customShader = sh;
	// }
	pub fn SetShader(&mut self, sh: qhandle_t) {
		assert!(sh != 0);
		// mRefEnt.customShader = sh; -- depends on refEntity_t definition
	}

	pub fn Init(&mut self) {
		// Method stub
	}
}

//------------------------------
pub struct CParticle {
	// Inherits from CEffect
	pub base: CEffect,

	pub mOrgOffset: [f32; 3],			// vec3_t

	pub mVel: [f32; 3],					// vec3_t
	pub mAccel: [f32; 3],				// vec3_t
	pub mGravity: f32,

	pub mSizeStart: f32,
	pub mSizeEnd: f32,
	pub mSizeParm: f32,

	pub mRGBStart: [f32; 3],			// vec3_t
	pub mRGBEnd: [f32; 3],				// vec3_t
	pub mRGBParm: f32,

	pub mAlphaStart: f32,
	pub mAlphaEnd: f32,
	pub mAlphaParm: f32,

	pub mRotationDelta: f32,
	pub mElasticity: f32,

	pub mClientID: i16,					// short
	pub mModelNum: c_char,
	pub mBoltNum: c_char,
}

impl CParticle {
	pub fn new() -> Self {
		let mut p = CParticle {
			base: CEffect::new(),
			mOrgOffset: [0.0; 3],
			mVel: [0.0; 3],
			mAccel: [0.0; 3],
			mGravity: 0.0,
			mSizeStart: 0.0,
			mSizeEnd: 0.0,
			mSizeParm: 0.0,
			mRGBStart: [0.0; 3],
			mRGBEnd: [0.0; 3],
			mRGBParm: 0.0,
			mAlphaStart: 0.0,
			mAlphaEnd: 0.0,
			mAlphaParm: 0.0,
			mRotationDelta: 0.0,
			mElasticity: 0.0,
			mClientID: -1,
			mModelNum: -1 as c_char,
			mBoltNum: -1 as c_char,
		};
		// mRefEnt.reType = RT_SPRITE;
		p
	}

	fn UpdateOrigin(&mut self) -> bool {
		// Protected method stub
		true
	}

	// void		UpdateVelocity() {VectorMA( mVel, theFxHelper.mFloatFrameTime, mAccel, mVel ); }
	fn UpdateVelocity(&mut self) {
		// VectorMA( mVel, theFxHelper.mFloatFrameTime, mAccel, mVel );
	}

	fn UpdateSize(&mut self) {
		// Protected method stub
	}

	fn UpdateRGB(&mut self) {
		// Protected method stub
	}

	fn UpdateAlpha(&mut self) {
		// Protected method stub
	}

	// void		UpdateRotation() { mRefEnt.rotation += theFxHelper.mFrameTime * 0.01f * mRotationDelta; }
	fn UpdateRotation(&mut self) {
		// mRefEnt.rotation += theFxHelper.mFrameTime * 0.01f * mRotationDelta;
	}

	fn Cull(&self) -> bool {
		// Protected method stub
		false
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Die(&mut self) {
		// Virtual method override
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetShader( qhandle_t sh )		{ mRefEnt.customShader = sh;}
	pub fn SetShader(&mut self, sh: qhandle_t) {
		// mRefEnt.customShader = sh; -- depends on refEntity_t definition
	}

	// inline void SetOrgOffset( const vec3_t o )	{ if(o){VectorCopy(o,mOrgOffset);}else{VectorClear(mOrgOffset);}}
	pub fn SetOrgOffset(&mut self, o: Option<&[f32; 3]>) {
		if let Some(org) = o {
			self.mOrgOffset = *org;
		} else {
			self.mOrgOffset = [0.0; 3];
		}
	}

	// inline void SetVel( const vec3_t vel )		{ if(vel){VectorCopy(vel,mVel);}else{VectorClear(mVel);}	}
	pub fn SetVel(&mut self, vel: Option<&[f32; 3]>) {
		if let Some(v) = vel {
			self.mVel = *v;
		} else {
			self.mVel = [0.0; 3];
		}
	}

	// inline void SetAccel( const vec3_t ac )		{ if(ac){VectorCopy(ac,mAccel);}else{VectorClear(mAccel);}	}
	pub fn SetAccel(&mut self, ac: Option<&[f32; 3]>) {
		if let Some(a) = ac {
			self.mAccel = *a;
		} else {
			self.mAccel = [0.0; 3];
		}
	}

	// inline void SetGravity( float grav )		{ mGravity = grav;			}
	pub fn SetGravity(&mut self, grav: f32) {
		self.mGravity = grav;
	}

	// inline void SetSizeStart( float sz )		{ mSizeStart = sz;			}
	pub fn SetSizeStart(&mut self, sz: f32) {
		self.mSizeStart = sz;
	}

	// inline void SetSizeEnd( float sz )			{ mSizeEnd = sz;			}
	pub fn SetSizeEnd(&mut self, sz: f32) {
		self.mSizeEnd = sz;
	}

	// inline void SetSizeParm( float parm )		{ mSizeParm = parm;			}
	pub fn SetSizeParm(&mut self, parm: f32) {
		self.mSizeParm = parm;
	}

	// inline void SetRGBStart( const vec3_t rgb )	{ if(rgb){VectorCopy(rgb,mRGBStart);}else{VectorClear(mRGBStart);}	}
	pub fn SetRGBStart(&mut self, rgb: Option<&[f32; 3]>) {
		if let Some(r) = rgb {
			self.mRGBStart = *r;
		} else {
			self.mRGBStart = [0.0; 3];
		}
	}

	// inline void SetRGBEnd( const vec3_t rgb )	{ if(rgb){VectorCopy(rgb,mRGBEnd);}else{VectorClear(mRGBEnd);}		}
	pub fn SetRGBEnd(&mut self, rgb: Option<&[f32; 3]>) {
		if let Some(r) = rgb {
			self.mRGBEnd = *r;
		} else {
			self.mRGBEnd = [0.0; 3];
		}
	}

	// inline void SetRGBParm( float parm )		{ mRGBParm = parm;			}
	pub fn SetRGBParm(&mut self, parm: f32) {
		self.mRGBParm = parm;
	}

	// inline void SetAlphaStart( float al )		{ mAlphaStart = al;			}
	pub fn SetAlphaStart(&mut self, al: f32) {
		self.mAlphaStart = al;
	}

	// inline void SetAlphaEnd( float al )			{ mAlphaEnd = al;			}
	pub fn SetAlphaEnd(&mut self, al: f32) {
		self.mAlphaEnd = al;
	}

	// inline void SetAlphaParm( float parm )		{ mAlphaParm = parm;		}
	pub fn SetAlphaParm(&mut self, parm: f32) {
		self.mAlphaParm = parm;
	}

	// inline void SetRotation( float rot )		{ mRefEnt.rotation = rot;	}
	pub fn SetRotation(&mut self, rot: f32) {
		// mRefEnt.rotation = rot; -- depends on refEntity_t definition
	}

	// inline void SetRotationDelta( float rot )	{ mRotationDelta = rot;		}
	pub fn SetRotationDelta(&mut self, rot: f32) {
		self.mRotationDelta = rot;
	}

	// inline void SetElasticity( float el )		{ mElasticity = el;			}
	pub fn SetElasticity(&mut self, el: f32) {
		self.mElasticity = el;
	}

	// inline void SetClient( int clientID,  int modelNum = -1, int boltNum = -1 )	{mClientID = clientID;	mModelNum = modelNum; mBoltNum = boltNum; }
	pub fn SetClient(&mut self, clientID: i16, modelNum: Option<c_char>, boltNum: Option<c_char>) {
		self.mClientID = clientID;
		self.mModelNum = modelNum.unwrap_or(-1 as c_char);
		self.mBoltNum = boltNum.unwrap_or(-1 as c_char);
	}
}

//------------------------------
pub struct CLine {
	// Inherits from CParticle
	pub base: CParticle,

	pub mOrigin2: [f32; 3],				// vec3_t
}

impl CLine {
	pub fn new() -> Self {
		CLine {
			base: CParticle::new(),
			mOrigin2: [0.0; 3],
		}
		// mRefEnt.reType = RT_LINE;
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Die(&mut self) {
		// Virtual method override
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetOrigin2( const vec3_t org2 )	{ VectorCopy( org2, mOrigin2 ); }
	pub fn SetOrigin2(&mut self, org2: &[f32; 3]) {
		self.mOrigin2 = *org2;
	}
}

//------------------------------
pub struct CBezier {
	// Inherits from CLine
	pub base: CLine,

	pub mControl1: [f32; 3],			// vec3_t
	pub mControl1Vel: [f32; 3],			// vec3_t

	pub mControl2: [f32; 3],			// vec3_t
	pub mControl2Vel: [f32; 3],			// vec3_t

	pub mInit: bool,
}

impl CBezier {
	pub fn new() -> Self {
		CBezier {
			base: CLine::new(),
			mControl1: [0.0; 3],
			mControl1Vel: [0.0; 3],
			mControl2: [0.0; 3],
			mControl2Vel: [0.0; 3],
			mInit: false,
		}
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Die(&mut self) {
		// Virtual method override
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// void DrawSegment( vec3_t start, vec3_t end, float texcoord1, float texcoord2 );
	pub fn DrawSegment(&self, start: &[f32; 3], end: &[f32; 3], texcoord1: f32, texcoord2: f32) {
		// Method stub
	}

	// inline void SetControlPoints( const vec3_t ctrl1, const vec3_t ctrl2 )	{ VectorCopy( ctrl1, mControl1 ); VectorCopy( ctrl2, mControl2 ); }
	pub fn SetControlPoints(&mut self, ctrl1: &[f32; 3], ctrl2: &[f32; 3]) {
		self.mControl1 = *ctrl1;
		self.mControl2 = *ctrl2;
	}

	// inline void SetControlVel( const vec3_t ctrl1v, const vec3_t ctrl2v )	{ VectorCopy( ctrl1v, mControl1Vel ); VectorCopy( ctrl2v, mControl2Vel ); }
	pub fn SetControlVel(&mut self, ctrl1v: &[f32; 3], ctrl2v: &[f32; 3]) {
		self.mControl1Vel = *ctrl1v;
		self.mControl2Vel = *ctrl2v;
	}
}

//------------------------------
pub struct CElectricity {
	// Inherits from CLine
	pub base: CLine,

	pub mChaos: f32,
}

impl CElectricity {
	pub fn new() -> Self {
		CElectricity {
			base: CLine::new(),
			mChaos: 0.0,
		}
		// mRefEnt.reType = RT_ELECTRICITY;
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Die(&mut self) {
		// Virtual method override
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	pub fn Initialize(&mut self) {
		// Method stub
	}

	// inline void SetChaos( float chaos )		{ mChaos = chaos; }
	pub fn SetChaos(&mut self, chaos: f32) {
		self.mChaos = chaos;
	}
}

// Oriented quad
//------------------------------
pub struct COrientedParticle {
	// Inherits from CParticle
	pub base: CParticle,

	pub mNormal: [f32; 3],				// vec3_t
	pub mNormalOffset: [f32; 3],		// vec3_t
}

impl COrientedParticle {
	pub fn new() -> Self {
		COrientedParticle {
			base: CParticle::new(),
			mNormal: [0.0; 3],
			mNormalOffset: [0.0; 3],
		}
		// mRefEnt.reType = RT_ORIENTED_QUAD;
	}

	fn Cull(&self) -> bool {
		// Protected method stub
		false
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetNormal( const vec3_t norm )	{ VectorCopy( norm, mNormal );	}
	pub fn SetNormal(&mut self, norm: &[f32; 3]) {
		self.mNormal = *norm;
	}

	// inline void SetNormalOffset( const vec3_t norm )	{ VectorCopy( norm, mNormalOffset );	}
	pub fn SetNormalOffset(&mut self, norm: &[f32; 3]) {
		self.mNormalOffset = *norm;
	}
}

//------------------------------
pub struct CTail {
	// Inherits from CParticle
	pub base: CParticle,

	pub mOldOrigin: [f32; 3],			// vec3_t

	pub mLengthStart: f32,
	pub mLengthEnd: f32,
	pub mLengthParm: f32,

	pub mLength: f32,
}

impl CTail {
	pub fn new() -> Self {
		CTail {
			base: CParticle::new(),
			mOldOrigin: [0.0; 3],
			mLengthStart: 0.0,
			mLengthEnd: 0.0,
			mLengthParm: 0.0,
			mLength: 0.0,
		}
		// mRefEnt.reType = RT_LINE;
	}

	fn UpdateLength(&mut self) {
		// Protected method stub
	}

	fn CalcNewEndpoint(&mut self) {
		// Protected method stub
	}

	fn Draw(&self) {
		// Protected method stub
	}

	fn Cull(&self) -> bool {
		// Protected method stub
		false
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetLengthStart( float len )	{ mLengthStart = len;	}
	pub fn SetLengthStart(&mut self, len: f32) {
		self.mLengthStart = len;
	}

	// inline void SetLengthEnd( float len )	{ mLengthEnd = len;	}
	pub fn SetLengthEnd(&mut self, len: f32) {
		self.mLengthEnd = len;
	}

	// inline void SetLengthParm( float len )	{ mLengthParm = len;	}
	pub fn SetLengthParm(&mut self, len: f32) {
		self.mLengthParm = len;
	}
}

//------------------------------
pub struct CCylinder {
	// Inherits from CTail
	pub base: CTail,

	pub mSize2Start: f32,
	pub mSize2End: f32,
	pub mSize2Parm: f32,
}

impl CCylinder {
	pub fn new() -> Self {
		CCylinder {
			base: CTail::new(),
			mSize2Start: 0.0,
			mSize2End: 0.0,
			mSize2Parm: 0.0,
		}
		// mRefEnt.reType = RT_CYLINDER;
	}

	fn UpdateSize2(&mut self) {
		// Protected method stub
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetSize2Start( float sz )	{ mSize2Start = sz;			}
	pub fn SetSize2Start(&mut self, sz: f32) {
		self.mSize2Start = sz;
	}

	// inline void SetSize2End( float sz )		{ mSize2End = sz;			}
	pub fn SetSize2End(&mut self, sz: f32) {
		self.mSize2End = sz;
	}

	// inline void SetSize2Parm( float parm )	{ mSize2Parm = parm;		}
	pub fn SetSize2Parm(&mut self, parm: f32) {
		self.mSize2Parm = parm;
	}

	// inline void SetNormal( const vec3_t norm )	{ VectorCopy( norm, mRefEnt.axis[0] ); }
	pub fn SetNormal(&mut self, norm: &[f32; 3]) {
		// VectorCopy( norm, mRefEnt.axis[0] ); -- depends on refEntity_t definition
	}
}

//------------------------------
// Emitters are derived from particles because, although they don't draw, any effect called
//	from them can borrow an initial or ending value from the emitters current alpha, rgb, etc..
pub struct CEmitter {
	// Inherits from CParticle
	pub base: CParticle,

	pub mOldOrigin: [f32; 3],			// vec3_t		// we use these to do some nice
	pub mLastOrigin: [f32; 3],			// vec3_t		//	tricks...
	pub mOldVelocity: [f32; 3],			// vec3_t		//
	pub mOldTime: c_int,

	pub mAngles: [f32; 3],				// vec3_t		// for a rotating thing, using a delta
	pub mAngleDelta: [f32; 3],			// vec3_t		//	as opposed to an end angle is probably much easier

	pub mEmitterFxID: c_int,			// if we have emitter fx, this is our id

	pub mDensity: f32,					// controls how often emitter chucks an effect
	pub mVariance: f32,					// density sloppiness
}

impl CEmitter {
	pub fn new() -> Self {
		let mut e = CEmitter {
			base: CParticle::new(),
			mOldOrigin: [0.0; 3],
			mLastOrigin: [0.0; 3],
			mOldVelocity: [0.0; 3],
			mOldTime: 0,
			mAngles: [0.0; 3],
			mAngleDelta: [0.0; 3],
			mEmitterFxID: 0,
			mDensity: 0.0,
			mVariance: 0.0,
		};
		// There may or may not be a model, but if there isn't one,
		//	we just won't bother adding the refEnt in our Draw func
		// mRefEnt.reType = RT_MODEL;
		e
	}

	fn UpdateAngles(&mut self) {
		// Protected method stub
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	// inline void SetModel( qhandle_t model )		{ mRefEnt.hModel = model;	}
	pub fn SetModel(&mut self, model: qhandle_t) {
		// mRefEnt.hModel = model; -- depends on refEntity_t definition
	}

	// inline void SetAngles( const vec3_t ang )	{ if(ang){VectorCopy(ang,mAngles);}else{VectorClear(mAngles);}			}
	pub fn SetAngles(&mut self, ang: Option<&[f32; 3]>) {
		if let Some(a) = ang {
			self.mAngles = *a;
		} else {
			self.mAngles = [0.0; 3];
		}
	}

	// inline void SetAngleDelta( const vec3_t ang){ if(ang){VectorCopy(ang,mAngleDelta);}else{VectorClear(mAngleDelta);}	}
	pub fn SetAngleDelta(&mut self, ang: Option<&[f32; 3]>) {
		if let Some(a) = ang {
			self.mAngleDelta = *a;
		} else {
			self.mAngleDelta = [0.0; 3];
		}
	}

	// inline void SetEmitterFxID( int id )		{ mEmitterFxID = id;		}
	pub fn SetEmitterFxID(&mut self, id: c_int) {
		self.mEmitterFxID = id;
	}

	// inline void SetDensity( float density )		{ mDensity = density;		}
	pub fn SetDensity(&mut self, density: f32) {
		self.mDensity = density;
	}

	// inline void SetVariance( float var )		{ mVariance = var;			}
	pub fn SetVariance(&mut self, var: f32) {
		self.mVariance = var;
	}

	// inline void SetOldTime( int time )			{ mOldTime = time;			}
	pub fn SetOldTime(&mut self, time: c_int) {
		self.mOldTime = time;
	}

	// inline void SetLastOrg( const vec3_t org )	{ if(org){VectorCopy(org,mLastOrigin);}else{VectorClear(mLastOrigin);}	}
	pub fn SetLastOrg(&mut self, org: Option<&[f32; 3]>) {
		if let Some(o) = org {
			self.mLastOrigin = *o;
		} else {
			self.mLastOrigin = [0.0; 3];
		}
	}

	// inline void SetLastVel( const vec3_t vel )	{ if(vel){VectorCopy(vel,mOldVelocity);}else{VectorClear(mOldVelocity);}}
	pub fn SetLastVel(&mut self, vel: Option<&[f32; 3]>) {
		if let Some(v) = vel {
			self.mOldVelocity = *v;
		} else {
			self.mOldVelocity = [0.0; 3];
		}
	}
}

// We're getting pretty low level here, not the kind of thing to abuse considering how much overhead this
//	adds to a SINGLE triangle or quad....
// The editor doesn't need to see or do anything with this
//------------------------------
pub const MAX_CPOLY_VERTS: usize = 5;

pub struct CPoly {
	// Inherits from CParticle
	pub base: CParticle,

	pub mCount: c_int,
	pub mRotDelta: [f32; 3],			// vec3_t
	pub mTimeStamp: c_int,

	pub mOrg: [[f32; 3]; MAX_CPOLY_VERTS],		// vec3_t[MAX_CPOLY_VERTS]
	pub mST: [[f32; 2]; MAX_CPOLY_VERTS],		// vec2_t[MAX_CPOLY_VERTS]

	pub mRot: [[f32; 3]; 3],			// float[3][3]
	pub mLastFrameTime: c_int,
}

impl CPoly {
	pub fn new() -> Self {
		CPoly {
			base: CParticle::new(),
			mCount: 0,
			mRotDelta: [0.0; 3],
			mTimeStamp: 0,
			mOrg: [[0.0; 3]; MAX_CPOLY_VERTS],
			mST: [[0.0; 2]; MAX_CPOLY_VERTS],
			mRot: [[0.0; 3]; 3],
			mLastFrameTime: 0,
		}
	}

	fn Cull(&self) -> bool {
		// Protected method stub
		false
	}

	fn Draw(&self) {
		// Protected method stub
	}

	pub fn Update(&mut self) -> bool {
		// Virtual method override
		self.base.Update()
	}

	pub fn PolyInit(&mut self) {
		// Method stub
	}

	pub fn CalcRotateMatrix(&mut self) {
		// Method stub
	}

	pub fn Rotate(&mut self) {
		// Method stub
	}

	// inline void SetNumVerts( int c )					{ mCount = c;			}
	pub fn SetNumVerts(&mut self, c: c_int) {
		self.mCount = c;
	}

	// inline void SetRot( vec3_t r )						{ if(r){VectorCopy(r,mRotDelta);}else{VectorClear(mRotDelta);}}
	pub fn SetRot(&mut self, r: Option<&[f32; 3]>) {
		if let Some(rot) = r {
			self.mRotDelta = *rot;
		} else {
			self.mRotDelta = [0.0; 3];
		}
	}

	// inline void SetMotionTimeStamp( int t )				{ mTimeStamp = theFxHelper.mTime + t; }
	pub fn SetMotionTimeStamp(&mut self, t: c_int) {
		// mTimeStamp = theFxHelper.mTime + t; -- depends on theFxHelper definition
	}

	// inline int	GetMotionTimeStamp()					{ return mTimeStamp; }
	pub fn GetMotionTimeStamp(&self) -> c_int {
		self.mTimeStamp
	}
}

// Stub for theFxHelper - would be a global in the actual module
pub struct FxHelper {
	pub mTime: c_int,
	pub mFloatFrameTime: f32,
	pub mFrameTime: c_int,
}

pub static mut theFxHelper: FxHelper = FxHelper {
	mTime: 0,
	mFloatFrameTime: 0.0,
	mFrameTime: 0,
};

// Stub for cg - would be a global in the actual module
pub struct CGameState {
	pub time: c_int,
}

pub static mut cg: CGameState = CGameState {
	time: 0,
};

// #endif //FX_PRIMITIVES_H_INC
