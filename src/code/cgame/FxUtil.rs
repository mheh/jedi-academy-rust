// this include must remain at the top of every CPP file
// Translated from: oracle/code/cgame/FxUtil.cpp

use core::ffi::{c_int, c_void};

// Types from common_headers.h and FxScheduler.h
pub type vec3_t = [f32; 3];
pub type vec2_t = [f32; 2];
pub type qhandle_t = c_int;
pub type qboolean = c_int;

const QTRUE: qboolean = 1;
const QFALSE: qboolean = 0;

pub const WHITE: vec3_t = [1.0f32, 1.0f32, 1.0f32];

// Opaque type stubs for classes from other modules
pub struct CEffect;
pub struct CParticle;
pub struct CLine;
pub struct CElectricity;
pub struct CTail;
pub struct CCylinder;
pub struct CEmitter;
pub struct CLight;
pub struct COrientedParticle;
pub struct CPoly;
pub struct CBezier;
pub struct CFlash;

pub struct SFxHelper;
pub struct SFxScheduler;

#[repr(C)]
pub struct SEffectList {
    pub mEffect: *mut CEffect,
    pub mKillTime: c_int,
    pub mPortal: bool,
}

const PI: f32 = 3.14159f32;

const MAX_EFFECTS: usize = 32;

static mut effectList: [SEffectList; MAX_EFFECTS] = [SEffectList {
    mEffect: core::ptr::null_mut(),
    mKillTime: 0,
    mPortal: false,
}; MAX_EFFECTS];

static mut nextValidEffect: *mut SEffectList = core::ptr::null_mut();
static mut theFxHelper: SFxHelper = unsafe { core::mem::zeroed() };

static mut activeFx: c_int = 0;
static mut mMax: c_int = 0;
static mut mMaxTime: c_int = 0;
static mut drawnFx: c_int = 0;
static mut mParticles: c_int = 0;
static mut mOParticles: c_int = 0;
static mut mLines: c_int = 0;
static mut mTails: c_int = 0;
static mut fxInitialized: qboolean = QFALSE;

// Extern declarations for globals and functions
extern "C" {
    static mut gEffectsInPortal: bool;
    static mut fx_debug: cvar_t;
    static mut theFxScheduler: SFxScheduler;
}

// Cvar type stub
#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

// Extern function declarations for C++ class operations
// These would normally be defined in class headers
extern "C" {
    fn FX_CopeWithAnyLoadedSaveGames();
}

// Local stub functions for vector operations
// These would come from common_headers.h
#[inline]
unsafe fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

#[inline]
unsafe fn Vector2Copy(src: &vec2_t, dst: &mut vec2_t) {
    dst[0] = src[0];
    dst[1] = src[1];
}

const vec3_origin: vec3_t = [0.0f32, 0.0f32, 0.0f32];

// Constants that would come from FxScheduler.h or related headers
const FX_KILL_ON_IMPACT: c_int = 0x00000004;
const FX_RELATIVE: c_int = 0x00000001;
const FX_RGB_PARM_MASK: c_int = 0x00007000;
const FX_RGB_WAVE: c_int = 0x00001000;
const FX_ALPHA_PARM_MASK: c_int = 0x00070000;
const FX_ALPHA_WAVE: c_int = 0x00010000;
const FX_SIZE_PARM_MASK: c_int = 0x00000700;
const FX_SIZE_WAVE: c_int = 0x00000100;
const FX_LENGTH_PARM_MASK: c_int = 0x00700000;
const FX_LENGTH_WAVE: c_int = 0x00100000;
const FX_SIZE2_PARM_MASK: c_int = 0x00007000;
const FX_SIZE2_WAVE: c_int = 0x00001000;
const FX_ALPHA_LINEAR: c_int = 0x00000000;

//-------------------------
// FX_Free
//
// Frees all FX
//-------------------------
pub unsafe fn FX_Free() -> bool {
    for i in 0..MAX_EFFECTS {
        if !(*addr_of_mut!(effectList[i])).mEffect.is_null() {
            // PORTING: C++ delete would call destructor; preserving pattern
            let _p = (*addr_of_mut!(effectList[i])).mEffect;
            // delete effectList[i].mEffect;
        }

        (*addr_of_mut!(effectList[i])).mEffect = core::ptr::null_mut();
    }

    *addr_of_mut!(activeFx) = 0;

    (*addr_of_mut!(theFxScheduler)).Clean();
    true
}

//-------------------------
// FX_Stop
//
// Frees all active FX but leaves the templates
//-------------------------
pub unsafe fn FX_Stop() {
    for i in 0..MAX_EFFECTS {
        if !(*addr_of_mut!(effectList[i])).mEffect.is_null() {
            // PORTING: C++ delete would call destructor; preserving pattern
            let _p = (*addr_of_mut!(effectList[i])).mEffect;
            // delete effectList[i].mEffect;
        }

        (*addr_of_mut!(effectList[i])).mEffect = core::ptr::null_mut();
    }

    *addr_of_mut!(activeFx) = 0;

    (*addr_of_mut!(theFxScheduler)).Clean_with_arg(false);
}

//-------------------------
// FX_Init
//
// Preps system for use
//-------------------------
pub unsafe fn FX_Init() -> c_int {
    if *addr_of!(fxInitialized) == QFALSE {
        *addr_of_mut!(fxInitialized) = QTRUE;

        for i in 0..MAX_EFFECTS {
            (*addr_of_mut!(effectList[i])).mEffect = core::ptr::null_mut();
        }
    }

    FX_Free();

    *addr_of_mut!(mMax) = 0;
    *addr_of_mut!(mMaxTime) = 0;

    *addr_of_mut!(nextValidEffect) = addr_of_mut!(effectList[0]);
    (*addr_of_mut!(theFxHelper)).Init();

    // ( nothing to see here, go away )
    //
    FX_CopeWithAnyLoadedSaveGames();

    true as c_int
}

//-------------------------
// FX_FreeMember
//-------------------------
unsafe fn FX_FreeMember(obj: *mut SEffectList) {
    // obj->mEffect->Die();
    // delete obj->mEffect;
    (*obj).mEffect = core::ptr::null_mut();

    // May as well mark this to be used next
    *addr_of_mut!(nextValidEffect) = obj;

    *addr_of_mut!(activeFx) -= 1;
}

//-------------------------
// FX_GetValidEffect
//
// Finds an unused effect slot
//
// Note - in the editor, this function may return NULL, indicating that all
// effects are being stopped.
//-------------------------
unsafe fn FX_GetValidEffect() -> *mut SEffectList {
    if (*(*addr_of!(nextValidEffect))).mEffect.is_null() {
        return *addr_of!(nextValidEffect);
    }

    let mut i: c_int;
    let mut ef: *mut SEffectList;

    // Blah..plow through the list till we find something that is currently untainted
    i = 0;
    ef = addr_of_mut!(effectList[0]);
    while i < (MAX_EFFECTS as c_int) {
        if (*ef).mEffect.is_null() {
            return ef;
        }
        i += 1;
        ef = ef.offset(1);
    }

    // report the error.
    #[cfg(not(feature = "FINAL_BUILD"))]
    {
        // theFxHelper.Print( "FX system out of effects\n" );
    }

    // Hmmm.. just trashing the first effect in the list is a poor approach
    FX_FreeMember(addr_of_mut!(effectList[0]));

    // Recursive call
    *addr_of!(nextValidEffect)
}

//-------------------------
// FX_ActiveFx
//
// Returns whether these are any active or scheduled effects
//-------------------------
pub unsafe fn FX_ActiveFx() -> bool {
    ((*addr_of!(activeFx)) > 0) || ((*addr_of_mut!(theFxScheduler)).NumScheduledFx() > 0)
}

//-------------------------
// FX_Add
//
// Adds all fx to the view
//-------------------------
pub unsafe fn FX_Add(portal: bool) {
    let mut i: c_int;
    let mut ef: *mut SEffectList;

    *addr_of_mut!(drawnFx) = 0;
    *addr_of_mut!(mParticles) = 0;
    *addr_of_mut!(mOParticles) = 0;
    *addr_of_mut!(mLines) = 0;
    *addr_of_mut!(mTails) = 0;

    let mut numFx = *addr_of!(activeFx); // but stop when there can't be any more left!
    i = 0;
    ef = addr_of_mut!(effectList[0]);
    while i < (MAX_EFFECTS as c_int) && numFx > 0 {
        if !(*ef).mEffect.is_null() {
            numFx -= 1;
            if portal != (*ef).mPortal {
                i += 1;
                ef = ef.offset(1);
                continue; // this one does not render in this scene
            }
            // Effect is active
            if *addr_of!((*addr_of_mut!(theFxHelper)).mTime) > (*ef).mKillTime {
                // Clean up old effects, calling any death effects as needed
                // this flag just has to be cleared otherwise death effects might not happen correctly
                // ef->mEffect->ClearFlags( FX_KILL_ON_IMPACT );
                FX_FreeMember(ef);
            } else {
                // if ( ef->mEffect->Update() == false )
                // {
                //     // We've been marked for death
                //     FX_FreeMember( ef );
                //     continue;
                // }
            }
        }
        i += 1;
        ef = ef.offset(1);
    }
    if *addr_of!(fx_debug.integer) == 2 && !portal {
        if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) > 100
            || *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 5
        {
            // theFxHelper.Print( "theFxHelper.mFrameTime = %i\n", theFxHelper.mFrameTime );
        }
    }
    if *addr_of!(fx_debug.integer) == 1 && !portal {
        if *addr_of!((*addr_of_mut!(theFxHelper)).mTime) > *addr_of!(mMaxTime) {
            // decay pretty harshly when we do it
            *addr_of_mut!(mMax) = (*addr_of!(mMax) as f32 * 0.9f32) as c_int;
            *addr_of_mut!(mMaxTime) = *addr_of!((*addr_of_mut!(theFxHelper)).mTime) + 200; // decay 5 times a second if we haven't set a new max
        }
        if *addr_of!(activeFx) > *addr_of!(mMax) {
            // but we can never be less that the current activeFx count
            *addr_of_mut!(mMax) = *addr_of!(activeFx);
            *addr_of_mut!(mMaxTime) = *addr_of!((*addr_of_mut!(theFxHelper)).mTime) + 4000; // since we just increased the max, hold it for at least 4 seconds
        }

        // Particles
        if *addr_of!(mParticles) > 500 {
            // theFxHelper.Print( ">Particles  ^1%4i  ", mParticles );
        } else if *addr_of!(mParticles) > 250 {
            // theFxHelper.Print( ">Particles  ^3%4i  ", mParticles );
        } else {
            // theFxHelper.Print( ">Particles  %4i  ", mParticles );
        }

        // Lines
        if *addr_of!(mLines) > 500 {
            // theFxHelper.Print( ">Lines ^1%4i\n", mLines );
        } else if *addr_of!(mLines) > 250 {
            // theFxHelper.Print( ">Lines ^3%4i\n", mLines );
        } else {
            // theFxHelper.Print( ">Lines %4i\n", mLines );
        }

        // OParticles
        if *addr_of!(mOParticles) > 500 {
            // theFxHelper.Print( ">OParticles ^1%4i  ", mOParticles );
        } else if *addr_of!(mOParticles) > 250 {
            // theFxHelper.Print( ">OParticles ^3%4i  ", mOParticles );
        } else {
            // theFxHelper.Print( ">OParticles %4i  ", mOParticles );
        }

        // Tails
        if *addr_of!(mTails) > 400 {
            // theFxHelper.Print( ">Tails ^1%4i\n", mTails );
        } else if *addr_of!(mTails) > 200 {
            // theFxHelper.Print( ">Tails ^3%4i\n", mTails );
        } else {
            // theFxHelper.Print( ">Tails %4i\n", mTails );
        }

        // Active
        if *addr_of!(activeFx) > 600 {
            // theFxHelper.Print( ">Active     ^1%4i  ", activeFx );
        } else if *addr_of!(activeFx) > 400 {
            // theFxHelper.Print( ">Active     ^3%4i  ", activeFx );
        } else {
            // theFxHelper.Print( ">Active     %4i  ", activeFx );
        }

        // Drawn
        if *addr_of!(drawnFx) > 600 {
            // theFxHelper.Print( ">Drawn ^1%4i  ", drawnFx );
        } else if *addr_of!(drawnFx) > 400 {
            // theFxHelper.Print( ">Drawn ^3%4i  ", drawnFx );
        } else {
            // theFxHelper.Print( ">Drawn %4i  ", drawnFx );
        }

        // Max
        if *addr_of!(mMax) > 600 {
            // theFxHelper.Print( ">Max ^1%4i  ", mMax );
        } else if *addr_of!(mMax) > 400 {
            // theFxHelper.Print( ">Max ^3%4i  ", mMax );
        } else {
            // theFxHelper.Print( ">Max %4i  ", mMax );
        }

        // Scheduled
        if (*addr_of_mut!(theFxScheduler)).NumScheduledFx() > 100 {
            // theFxHelper.Print( ">Scheduled ^1%4i\n", theFxScheduler.NumScheduledFx() );
        } else if (*addr_of_mut!(theFxScheduler)).NumScheduledFx() > 50 {
            // theFxHelper.Print( ">Scheduled ^3%4i\n", theFxScheduler.NumScheduledFx() );
        } else {
            // theFxHelper.Print( ">Scheduled %4i\n", theFxScheduler.NumScheduledFx() );
        }
    }
}

//-------------------------
// FX_AddPrimitive
//
// Note - in the editor, this function may change *pEffect to NULL, indicating that
// all effects are being stopped.
//-------------------------
pub unsafe fn FX_AddPrimitive(pEffect: *mut *mut CEffect, killTime: c_int) {
    let item = FX_GetValidEffect();

    (*item).mEffect = *pEffect;
    (*item).mKillTime = *addr_of!((*addr_of_mut!(theFxHelper)).mTime) + killTime;
    (*item).mPortal = gEffectsInPortal; // global set in AddScheduledEffects

    *addr_of_mut!(activeFx) += 1;

    // Stash these in the primitive so it has easy access to the vals
    // (*pEffect)->SetTimeStart( theFxHelper.mTime );
    // (*pEffect)->SetTimeEnd( theFxHelper.mTime + killTime );
}

//-------------------------
//  FX_AddParticle
//-------------------------
pub unsafe fn FX_AddParticle(
    clientID: c_int,
    org: &vec3_t,
    vel: &vec3_t,
    accel: &vec3_t,
    gravity: f32,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rgbParm: f32,
    rotation: f32,
    rotationDelta: f32,
    min: &vec3_t,
    max: &vec3_t,
    elasticity: f32,
    deathID: c_int,
    impactID: c_int,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CParticle {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding effects when the system is paused
        return core::ptr::null_mut();
    }

    // let mut fx = Box::new(CParticle::default());
    let mut fx: *mut CParticle = core::ptr::null_mut(); // PORTING: would be new CParticle

    if !fx.is_null() {
        if (flags & FX_RELATIVE) != 0 && clientID >= 0 {
            // fx->SetOrigin1( NULL );
            // fx->SetOrgOffset( org );
            // fx->SetClient( clientID, modelNum, boltNum );
        } else {
            // fx->SetOrigin1( org );
        }
        // fx->SetVel( vel );
        // fx->SetAccel( accel );
        // fx->SetGravity( gravity );

        // RGB----------------
        // fx->SetRGBStart( sRGB );
        // fx->SetRGBEnd( eRGB );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetFlags( flags );
        // fx->SetShader( shader );
        // fx->SetRotation( rotation );
        // fx->SetRotationDelta( rotationDelta );
        // fx->SetElasticity( elasticity );
        // fx->SetMin( min );
        // fx->SetMax( max );
        // fx->SetDeathFxID( deathID );
        // fx->SetImpactFxID( impactID );

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL
    }

    fx
}

//-------------------------
//  FX_AddLine
//-------------------------
pub unsafe fn FX_AddLine(
    clientID: c_int,
    start: &vec3_t,
    end: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rgbParm: f32,
    killTime: c_int,
    shader: qhandle_t,
    impactFX_id: c_int,
    flags: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CLine {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding new effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CLine = core::ptr::null_mut(); // PORTING: would be new CLine

    if !fx.is_null() {
        if (flags & FX_RELATIVE) != 0 && clientID >= 0 {
            // fx->SetOrigin1( NULL );
            // fx->SetOrgOffset( start ); //offset from bolt pos
            // fx->SetVel( end );	//vel is the vector offset from bolt+orgOffset
            // fx->SetClient( clientID, modelNum, boltNum );
        } else {
            // fx->SetOrigin1( start );
            // fx->SetOrigin2( end );
        }
        // RGB----------------
        // fx->SetRGBStart( sRGB );
        // fx->SetRGBEnd( eRGB );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetShader( shader );
        // fx->SetFlags( flags );

        // fx->SetSTScale( 1.0f, 1.0f );
        // fx->SetImpactFxID( impactFX_id );

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL
    }

    fx
}

//-------------------------
//  FX_AddElectricity
//-------------------------
pub unsafe fn FX_AddElectricity(
    clientID: c_int,
    start: &vec3_t,
    end: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rgbParm: f32,
    chaos: f32,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CElectricity {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding new effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CElectricity = core::ptr::null_mut(); // PORTING: would be new CElectricity

    if !fx.is_null() {
        if (flags & FX_RELATIVE) != 0 && clientID >= 0 {
            // fx->SetOrigin1( NULL );
            // fx->SetOrgOffset( start );//offset
            // fx->SetVel( end );	//vel is the vector offset from bolt+orgOffset
            // fx->SetClient( clientID, modelNum, boltNum );
        } else {
            // fx->SetOrigin1( start );
            // fx->SetOrigin2( end );
        }

        // RGB----------------
        // fx->SetRGBStart( sRGB );
        // fx->SetRGBEnd( eRGB );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetShader( shader );
        // fx->SetFlags( flags );
        // fx->SetChaos( chaos );

        // fx->SetSTScale( 1.0f, 1.0f );

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL?
        if !fx.is_null() {
            // fx->Initialize();
        }
    }

    fx
}

//-------------------------
//  FX_AddTail
//-------------------------
pub unsafe fn FX_AddTail(
    clientID: c_int,
    org: &vec3_t,
    vel: &vec3_t,
    accel: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    length1: f32,
    length2: f32,
    lengthParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rgbParm: f32,
    min: &vec3_t,
    max: &vec3_t,
    elasticity: f32,
    deathID: c_int,
    impactID: c_int,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CTail {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CTail = core::ptr::null_mut(); // PORTING: would be new CTail

    if !fx.is_null() {
        if (flags & FX_RELATIVE) != 0 && clientID >= 0 {
            // fx->SetOrigin1( NULL );
            // fx->SetOrgOffset( org );
            // fx->SetClient( clientID, modelNum, boltNum );
        } else {
            // fx->SetOrigin1( org );
        }
        // fx->SetVel( vel );
        // fx->SetAccel( accel );
        // RGB----------------
        // fx->SetRGBStart( sRGB );
        // fx->SetRGBEnd( eRGB );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Length----------------
        // fx->SetLengthStart( length1 );
        // fx->SetLengthEnd( length2 );

        if (flags & FX_LENGTH_PARM_MASK) == FX_LENGTH_WAVE {
            // fx->SetLengthParm( lengthParm * PI * 0.001f );
        } else if (flags & FX_LENGTH_PARM_MASK) != 0 {
            // fx->SetLengthParm( lengthParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetFlags( flags );
        // fx->SetShader( shader );
        // fx->SetElasticity( elasticity );
        // fx->SetMin( min );
        // fx->SetMax( max );
        // fx->SetSTScale( 1.0f, 1.0f );
        // fx->SetDeathFxID( deathID );
        // fx->SetImpactFxID( impactID );

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL
    }

    fx
}

//-------------------------
//  FX_AddCylinder
//-------------------------
pub unsafe fn FX_AddCylinder(
    clientID: c_int,
    start: &vec3_t,
    normal: &vec3_t,
    size1s: f32,
    size1e: f32,
    sizeParm: f32,
    size2s: f32,
    size2e: f32,
    size2Parm: f32,
    length1: f32,
    length2: f32,
    lengthParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    rgb1: &vec3_t,
    rgb2: &vec3_t,
    rgbParm: f32,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut CCylinder {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding new effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CCylinder = core::ptr::null_mut(); // PORTING: would be new CCylinder

    if !fx.is_null() {
        if (flags & FX_RELATIVE) != 0 && clientID >= 0 {
            // fx->SetOrigin1( NULL );
            // fx->SetOrgOffset( start );//offset
            // NOTE: relative version doesn't ever use normal!
            // fx->SetNormal( normal );
            // fx->SetClient( clientID, modelNum, boltNum );
        } else {
            // fx->SetOrigin1( start );
            // fx->SetNormal( normal );
        }

        // RGB----------------
        // fx->SetRGBStart( rgb1 );
        // fx->SetRGBEnd( rgb2 );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size1----------------
        // fx->SetSizeStart( size1s );
        // fx->SetSizeEnd( size1e );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size2----------------
        // fx->SetSize2Start( size2s );
        // fx->SetSize2End( size2e );

        if (flags & FX_SIZE2_PARM_MASK) == FX_SIZE2_WAVE {
            // fx->SetSize2Parm( size2Parm * PI * 0.001f );
        } else if (flags & FX_SIZE2_PARM_MASK) != 0 {
            // fx->SetSize2Parm( size2Parm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Length1---------------
        // fx->SetLengthStart( length1 );
        // fx->SetLengthEnd( length2 );

        if (flags & FX_LENGTH_PARM_MASK) == FX_LENGTH_WAVE {
            // fx->SetLengthParm( lengthParm * PI * 0.001f );
        } else if (flags & FX_LENGTH_PARM_MASK) != 0 {
            // fx->SetLengthParm( lengthParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetShader( shader );
        // fx->SetFlags( flags );

        FX_AddPrimitive(&mut fx, killTime);
    }

    fx
}

//-------------------------
//  FX_AddEmitter
//-------------------------
pub unsafe fn FX_AddEmitter(
    org: &vec3_t,
    vel: &vec3_t,
    accel: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    rgb1: &vec3_t,
    rgb2: &vec3_t,
    rgbParm: f32,
    angs: &vec3_t,
    deltaAngs: &vec3_t,
    min: &vec3_t,
    max: &vec3_t,
    elasticity: f32,
    deathID: c_int,
    impactID: c_int,
    emitterID: c_int,
    density: f32,
    variance: f32,
    killTime: c_int,
    model: qhandle_t,
    flags: c_int,
) -> *mut CEmitter {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CEmitter = core::ptr::null_mut(); // PORTING: would be new CEmitter

    if !fx.is_null() {
        // fx->SetOrigin1( org );
        // fx->SetVel( vel );
        // fx->SetAccel( accel );

        // RGB----------------
        // fx->SetRGBStart( rgb1 );
        // fx->SetRGBEnd( rgb2 );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetAngles( angs );
        // fx->SetAngleDelta( deltaAngs );
        // fx->SetFlags( flags );
        // fx->SetModel( model );
        // fx->SetElasticity( elasticity );
        // fx->SetMin( min );
        // fx->SetMax( max );
        // fx->SetDeathFxID( deathID );
        // fx->SetImpactFxID( impactID );
        // fx->SetEmitterFxID( emitterID );
        // fx->SetDensity( density );
        // fx->SetVariance( variance );
        // fx->SetOldTime( theFxHelper.mTime );

        // fx->SetLastOrg( org );
        // fx->SetLastVel( vel );

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL
    }

    fx
}

//-------------------------
//  FX_AddLight
//-------------------------
pub unsafe fn FX_AddLight(
    org: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    rgb1: &vec3_t,
    rgb2: &vec3_t,
    rgbParm: f32,
    killTime: c_int,
    flags: c_int,
) -> *mut CLight {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CLight = core::ptr::null_mut(); // PORTING: would be new CLight

    if !fx.is_null() {
        // fx->SetOrigin1( org );

        // RGB----------------
        // fx->SetRGBStart( rgb1 );
        // fx->SetRGBEnd( rgb2 );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetFlags( flags );

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL
    }

    fx
}

//-------------------------
//  FX_AddOrientedParticle
//-------------------------
pub unsafe fn FX_AddOrientedParticle(
    clientID: c_int,
    org: &vec3_t,
    norm: &vec3_t,
    vel: &vec3_t,
    accel: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    rgb1: &vec3_t,
    rgb2: &vec3_t,
    rgbParm: f32,
    rotation: f32,
    rotationDelta: f32,
    min: &vec3_t,
    max: &vec3_t,
    bounce: f32,
    deathID: c_int,
    impactID: c_int,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
    modelNum: c_int,
    boltNum: c_int,
) -> *mut COrientedParticle {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut COrientedParticle = core::ptr::null_mut(); // PORTING: would be new COrientedParticle

    if !fx.is_null() {
        if (flags & FX_RELATIVE) != 0 && clientID >= 0 {
            // fx->SetOrigin1( NULL );
            // fx->SetOrgOffset( org );//offset
            // fx->SetNormalOffset( norm );
            // fx->SetClient( clientID, modelNum, boltNum );
        } else {
            // fx->SetOrigin1( org );
            // fx->SetNormal( norm );
        }
        // fx->SetVel( vel );
        // fx->SetAccel( accel );

        // RGB----------------
        // fx->SetRGBStart( rgb1 );
        // fx->SetRGBEnd( rgb2 );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetFlags( flags );
        // fx->SetShader( shader );
        // fx->SetRotation( rotation );
        // fx->SetRotationDelta( rotationDelta );
        // fx->SetElasticity( bounce );
        // fx->SetMin( min );
        // fx->SetMax( max );
        // fx->SetDeathFxID( deathID );
        // fx->SetImpactFxID( impactID );

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL
    }

    fx
}

//-------------------------
//  FX_AddPoly
//-------------------------
pub unsafe fn FX_AddPoly(
    verts: *const vec3_t,
    st: *const vec2_t,
    numVerts: c_int,
    vel: &vec3_t,
    accel: &vec3_t,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    rgb1: &vec3_t,
    rgb2: &vec3_t,
    rgbParm: f32,
    rotationDelta: &vec3_t,
    bounce: f32,
    motionDelay: c_int,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
) -> *mut CPoly {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 || verts.is_null() {
        // disallow adding effects when the system is paused or the user doesn't pass in a vert array
        return core::ptr::null_mut();
    }

    let mut fx: *mut CPoly = core::ptr::null_mut(); // PORTING: would be new CPoly

    if !fx.is_null() {
        // Do a cheesy copy of the verts and texture coords into our own structure
        for i in 0..numVerts {
            // VectorCopy( verts[i], fx->mOrg[i] );
            // Vector2Copy( st[i], fx->mST[i] );
        }

        // fx->SetVel( vel );
        // fx->SetAccel( accel );

        // RGB----------------
        // fx->SetRGBStart( rgb1 );
        // fx->SetRGBEnd( rgb2 );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetFlags( flags );
        // fx->SetShader( shader );
        // fx->SetRot( rotationDelta );
        // fx->SetElasticity( bounce );
        // fx->SetMotionTimeStamp( motionDelay );
        // fx->SetNumVerts( numVerts );

        // Now that we've set our data up, let's process it into a useful format
        // fx->PolyInit();

        FX_AddPrimitive(&mut fx, killTime);
        // in the editor, fx may now be NULL
    }

    fx
}

//-------------------------
//  FX_AddBezier
//-------------------------
pub unsafe fn FX_AddBezier(
    start: &vec3_t,
    end: &vec3_t,
    control1: &vec3_t,
    control1Vel: &vec3_t,
    control2: &vec3_t,
    control2Vel: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rgbParm: f32,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
) -> *mut CBezier {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding new effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CBezier = core::ptr::null_mut(); // PORTING: would be new CBezier

    if !fx.is_null() {
        // fx->SetOrigin1( start );
        // fx->SetOrigin2( end );

        // fx->SetControlPoints( control1, control2 );
        // fx->SetControlVel( control1Vel, control2Vel );

        // RGB----------------
        // fx->SetRGBStart( sRGB );
        // fx->SetRGBEnd( eRGB );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Alpha----------------
        // fx->SetAlphaStart( alpha1 );
        // fx->SetAlphaEnd( alpha2 );

        if (flags & FX_ALPHA_PARM_MASK) == FX_ALPHA_WAVE {
            // fx->SetAlphaParm( alphaParm * PI * 0.001f );
        } else if (flags & FX_ALPHA_PARM_MASK) != 0 {
            // fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        // fx->SetSizeStart( size1 );
        // fx->SetSizeEnd( size2 );

        if (flags & FX_SIZE_PARM_MASK) == FX_SIZE_WAVE {
            // fx->SetSizeParm( sizeParm * PI * 0.001f );
        } else if (flags & FX_SIZE_PARM_MASK) != 0 {
            // fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // fx->SetShader( shader );
        // fx->SetFlags( flags );

        // fx->SetSTScale( 1.0f, 1.0f );

        FX_AddPrimitive(&mut fx, killTime);
    }

    fx
}

//-------------------------
//  FX_AddFlash
//-------------------------
pub unsafe fn FX_AddFlash(
    origin: &vec3_t,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rgbParm: f32,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
) -> *mut CFlash {
    if *addr_of!((*addr_of_mut!(theFxHelper)).mFrameTime) < 1 {
        // disallow adding new effects when the system is paused
        return core::ptr::null_mut();
    }

    let mut fx: *mut CFlash = core::ptr::null_mut(); // PORTING: would be new CFlash

    if !fx.is_null() {
        // fx->SetOrigin1( origin );

        // RGB----------------
        // fx->SetRGBStart( sRGB );
        // fx->SetRGBEnd( eRGB );

        if (flags & FX_RGB_PARM_MASK) == FX_RGB_WAVE {
            // fx->SetRGBParm( rgbParm * PI * 0.001f );
        } else if (flags & FX_RGB_PARM_MASK) != 0 {
            // rgbParm should be a value from 0-100..
            // fx->SetRGBParm( rgbParm * 0.01f * killTime + theFxHelper.mTime );
        }

        /*		// Alpha----------------
        fx->SetAlphaStart( alpha1 );
        fx->SetAlphaEnd( alpha2 );

        if (( flags & FX_ALPHA_PARM_MASK ) == FX_ALPHA_WAVE )
        {
            fx->SetAlphaParm( alphaParm * PI * 0.001f );
        }
        else if ( flags & FX_ALPHA_PARM_MASK )
        {
            fx->SetAlphaParm( alphaParm * 0.01f * killTime + theFxHelper.mTime );
        }

        // Size----------------
        fx->SetSizeStart( size1 );
        fx->SetSizeEnd( size2 );

        if (( flags & FX_SIZE_PARM_MASK ) == FX_SIZE_WAVE )
        {
            fx->SetSizeParm( sizeParm * PI * 0.001f );
        }
        else if ( flags & FX_SIZE_PARM_MASK )
        {
            fx->SetSizeParm( sizeParm * 0.01f * killTime + theFxHelper.mTime );
        }
        */

        // fx->SetShader( shader );
        // fx->SetFlags( flags );

        // fx->SetSTScale( 1.0f, 1.0f );

        // fx->Init();

        FX_AddPrimitive(&mut fx, killTime);
    }

    fx
}

//-------------------------------------------------------
// Functions for limited backward compatibility with EF.
//	These calls can be used for simple programmatic
//	effects, temp effects or debug graphics.
// Note that this is not an all-inclusive list of
//	fx add functions from EF, nor are the calls guaranteed
//	to produce the exact same result.
//-------------------------------------------------------

//---------------------------------------------------
pub unsafe fn FX_AddSprite(
    origin: &vec3_t,
    vel: &vec3_t,
    accel: &vec3_t,
    scale: f32,
    _dscale: f32,
    sAlpha: f32,
    eAlpha: f32,
    rotation: f32,
    bounce: f32,
    life: c_int,
    shader: qhandle_t,
    flags: c_int,
) {
    FX_AddParticle(
        -1, origin, vel, accel, 0.0f, scale, scale, 0.0f, sAlpha, eAlpha, FX_ALPHA_LINEAR,
        &WHITE, &WHITE, 0.0f, rotation, 0.0f, &vec3_origin, &vec3_origin, bounce, 0, 0, life,
        shader, flags, 0, 0,
    );
}

//---------------------------------------------------
pub unsafe fn FX_AddSprite_WithRGB(
    origin: &vec3_t,
    vel: &vec3_t,
    accel: &vec3_t,
    scale: f32,
    _dscale: f32,
    sAlpha: f32,
    eAlpha: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rotation: f32,
    bounce: f32,
    life: c_int,
    shader: qhandle_t,
    flags: c_int,
) {
    FX_AddParticle(
        -1, origin, vel, accel, 0.0f, scale, scale, 0.0f, sAlpha, eAlpha, FX_ALPHA_LINEAR,
        sRGB, eRGB, 0.0f, rotation, 0.0f, &vec3_origin, &vec3_origin, bounce, 0, 0, life,
        shader, flags, 0, 0,
    );
}

//---------------------------------------------------
pub unsafe fn FX_AddLine_Simple(
    start: &vec3_t,
    end: &vec3_t,
    _stScale: f32,
    width: f32,
    _dwidth: f32,
    sAlpha: f32,
    eAlpha: f32,
    life: c_int,
    shader: qhandle_t,
    flags: c_int,
) {
    FX_AddLine(
        -1, start, end, width, width, 0.0f, sAlpha, eAlpha, FX_ALPHA_LINEAR, &WHITE, &WHITE,
        0.0f, life, shader, 0, flags, 0, 0,
    );
}

//---------------------------------------------------
pub unsafe fn FX_AddLine_WithRGB(
    start: &vec3_t,
    end: &vec3_t,
    _stScale: f32,
    width: f32,
    _dwidth: f32,
    sAlpha: f32,
    eAlpha: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    life: c_int,
    shader: qhandle_t,
    flags: c_int,
) {
    FX_AddLine(
        -1, start, end, width, width, 0.0f, sAlpha, eAlpha, FX_ALPHA_LINEAR, sRGB, eRGB, 0.0f,
        life, shader, 0, flags, 0, 0,
    );
}

//---------------------------------------------------
pub unsafe fn FX_AddQuad(
    origin: &vec3_t,
    normal: &vec3_t,
    vel: &vec3_t,
    accel: &vec3_t,
    sradius: f32,
    eradius: f32,
    salpha: f32,
    ealpha: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rotation: f32,
    life: c_int,
    shader: qhandle_t,
    flags: c_int,
) {
    FX_AddOrientedParticle(
        -1, origin, normal, vel, accel, sradius, eradius, 0.0f, salpha, ealpha, 0.0f, sRGB,
        eRGB, 0.0f, rotation, 0.0f, core::ptr::null(), core::ptr::null(), 0.0f, 0, 0, life,
        shader, flags, 0, 0,
    );
}
