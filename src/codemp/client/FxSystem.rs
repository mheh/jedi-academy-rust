#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};
use core::mem;
use core::ptr::addr_of_mut;

//Anything above this #include will be ignored by the compiler
// this include must remain at the top of every CPP file

// ==================== Type Stubs ====================
// Types used from client.h, FxScheduler.h, and other headers

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // ... other fields from cvar_t, truncated for brevity
}

#[repr(C)]
pub struct refdef_t;

#[repr(C)]
pub struct TCGCameraShake {
    pub mOrigin: [f32; 3],
    pub mIntensity: f32,
    pub mRadius: c_int,
    pub mTime: c_int,
}

#[repr(C)]
pub struct TCGGetBoltData {
    pub mEntityNum: c_int,
    pub mAngles: [f32; 3],
    pub mOrigin: [f32; 3],
    pub mScale: [f32; 3],
}

#[repr(C)]
pub struct CGhoul2Info_v;

#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

pub type qboolean = c_int;
pub type vec3_t = [f32; 3];

#[repr(C)]
pub struct clientActive_t {
    pub mSharedMemory: *mut core::ffi::c_void,
}

pub type vmHandle_t = c_int;

// ==================== External Functions ====================

extern "C" {
    pub fn G2API_GetBoltMatrix(
        ghoul2: CGhoul2Info_v,
        modelNum: c_int,
        boltNum: c_int,
        boltMatrix: *mut mdxaBone_t,
        angles: *const f32,
        origin: *const f32,
        frameTime: c_int,
        unk: c_int,
        scale: *const f32,
    ) -> qboolean;

    pub fn VM_Call(handle: vmHandle_t, call: c_int) -> c_int;
    pub fn Com_DPrintf(msg: *const c_char);
}

// ==================== Global Variables ====================

pub static mut fx_debug: *mut cvar_t = core::ptr::null_mut();
#[cfg(feature = "_SOF2DEV_")]
pub static mut fx_freeze: *mut cvar_t = core::ptr::null_mut();
pub static mut fx_countScale: *mut cvar_t = core::ptr::null_mut();
pub static mut fx_nearCull: *mut cvar_t = core::ptr::null_mut();

const DEFAULT_EXPLOSION_RADIUS: c_int = 512;

pub static mut cl: clientActive_t = clientActive_t {
    mSharedMemory: core::ptr::null_mut(),
};
pub static mut cgvm: vmHandle_t = 0;

// TODO: get actual values from FxScheduler.h or cg_public.h
pub const CG_FX_CAMERASHAKE: c_int = 0;
pub const CG_GET_LERP_DATA: c_int = 0;

// Stuff for the FxHelper
//------------------------------------------------------

#[repr(C)]
pub struct SFxHelper {
    pub mTime: c_int,
    pub mOldTime: c_int,
    pub mFrameTime: c_int,
    pub mTimeFrozen: bool,
    pub mRealTime: f32,
    pub refdef: *mut refdef_t,
}

impl SFxHelper {
    pub fn new() -> Self {
        SFxHelper {
            mTime: 0,
            mOldTime: 0,
            mFrameTime: 0,
            mTimeFrozen: false,
            mRealTime: 0.0,
            refdef: core::ptr::null_mut(),
        }
    }

    pub fn ReInit(&mut self, pRefdef: *mut refdef_t) {
        self.mTime = 0;
        self.mOldTime = 0;
        self.mFrameTime = 0;
        self.mTimeFrozen = false;
        self.refdef = pRefdef;
    }

    //------------------------------------------------------
    pub fn Print(&self, msg: *const c_char) {
        unsafe {
            Com_DPrintf(msg);
        }
    }

    //------------------------------------------------------
    pub fn AdjustTime(&mut self, frametime: c_int) {
        #[cfg(feature = "_SOF2DEV_")]
        let should_freeze = unsafe { (*fx_freeze).integer != 0 } || frametime <= 0;
        #[cfg(not(feature = "_SOF2DEV_"))]
        let should_freeze = frametime <= 0;

        if should_freeze {
            // Allow no time progression when we are paused.
            self.mFrameTime = 0;
            self.mRealTime = 0.0;
        } else {
            self.mOldTime = self.mTime;
            self.mTime = frametime;
            self.mFrameTime = self.mTime - self.mOldTime;

            self.mRealTime = (self.mFrameTime as f32) * 0.001;

            /*		mFrameTime = frametime;
            mTime += mFrameTime;
            mRealTime = mFrameTime * 0.001f;*/

            //		mHalfRealTimeSq = mRealTime * mRealTime * 0.5f;
        }
    }

    //------------------------------------------------------
    pub fn CameraShake(&self, origin: vec3_t, intensity: f32, radius: c_int, time: c_int) {
        unsafe {
            let data = (*addr_of_mut!(cl)).mSharedMemory as *mut TCGCameraShake;

            // VectorCopy(origin, data->mOrigin);
            (*data).mOrigin = origin;
            (*data).mIntensity = intensity;
            (*data).mRadius = radius;
            (*data).mTime = time;

            VM_Call(cgvm, CG_FX_CAMERASHAKE);
        }
    }

    //------------------------------------------------------
    pub fn GetOriginAxisFromBolt(
        &self,
        pGhoul2: *mut CGhoul2Info_v,
        mEntNum: c_int,
        modelNum: c_int,
        boltNum: c_int,
        origin: *mut vec3_t,
        axis: *mut [vec3_t; 3],
    ) -> qboolean {
        unsafe {
            let mut doesBoltExist: qboolean;
            let mut boltMatrix: mdxaBone_t = mem::zeroed();
            let data = (*addr_of_mut!(cl)).mSharedMemory as *mut TCGGetBoltData;
            (*data).mEntityNum = mEntNum;
            VM_Call(cgvm, CG_GET_LERP_DATA); //this func will zero out pitch and roll for players, and ridable vehicles

            //Fixme: optimize these VM calls away by storing

            // go away and get me the bolt position for this frame please
            doesBoltExist = G2API_GetBoltMatrix(
                core::ptr::read(pGhoul2),
                modelNum,
                boltNum,
                &mut boltMatrix,
                (*data).mAngles.as_ptr(),
                (*data).mOrigin.as_ptr(),
                (*addr_of_mut!(theFxHelper)).mOldTime,
                0,
                (*data).mScale.as_ptr(),
            );

            if doesBoltExist != 0 {
                // set up the axis and origin we need for the actual effect spawning
                (*origin)[0] = boltMatrix.matrix[0][3];
                (*origin)[1] = boltMatrix.matrix[1][3];
                (*origin)[2] = boltMatrix.matrix[2][3];

                (*axis)[1][0] = boltMatrix.matrix[0][0];
                (*axis)[1][1] = boltMatrix.matrix[1][0];
                (*axis)[1][2] = boltMatrix.matrix[2][0];

                (*axis)[0][0] = boltMatrix.matrix[0][1];
                (*axis)[0][1] = boltMatrix.matrix[1][1];
                (*axis)[0][2] = boltMatrix.matrix[2][1];

                (*axis)[2][0] = boltMatrix.matrix[0][2];
                (*axis)[2][1] = boltMatrix.matrix[1][2];
                (*axis)[2][2] = boltMatrix.matrix[2][2];
            }
            doesBoltExist
        }
    }
}

pub static mut theFxHelper: SFxHelper = SFxHelper {
    mTime: 0,
    mOldTime: 0,
    mFrameTime: 0,
    mTimeFrozen: false,
    mRealTime: 0.0,
    refdef: core::ptr::null_mut(),
};
