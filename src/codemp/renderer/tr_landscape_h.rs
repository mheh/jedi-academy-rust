#![allow(non_snake_case)]

// tr_landscape.h

use core::ffi::{c_int, c_char};
use core::ptr::{null_mut, addr_of_mut};

// Number of TriTreeNodes available
pub const POOL_SIZE: c_int = 50000;

pub const TEXTURE_ALPHA_TL: c_int = 0x000000ff;
pub const TEXTURE_ALPHA_TR: c_int = 0x0000ff00;
pub const TEXTURE_ALPHA_BL: c_int = 0x00ff0000;
pub const TEXTURE_ALPHA_BR: c_int = 0x000000ff;

pub const INDEX_TL: c_int = 0;
pub const INDEX_TR: c_int = 1;
pub const INDEX_BL: c_int = 2;
pub const INDEX_BR: c_int = 3;

pub const VARIANCE_MIN: f32 = 0.0f32;
pub const VARIANCE_MAX: f32 = 2000.0f32;
pub const SPLIT_VARIANCE_SIZE: c_int = 20;
pub const SPLIT_VARIANCE_STEP: f32 = VARIANCE_MAX / SPLIT_VARIANCE_SIZE as f32;

pub const PI_TOP: c_int = 1;
pub const PI_BOTTOM: c_int = 2;
pub const PI_BOTH: c_int = 3;

pub const HEIGHT_RESOLUTION: usize = 256;

// Type aliases
pub type vec3_t = [f32; 3];
pub type color4ub_t = [u8; 4];
pub type qhandle_t = c_int;
pub type thandle_t = c_int;
pub type vec3pair_t = [[f32; 3]; 2];
pub type ivec5_t = [c_int; 5];

// Forward declarations for opaque types
#[repr(C)]
pub struct CCMLandScape {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CCMPatch {
    _private: [u8; 0],
}

#[repr(C)]
pub struct shader_s {
    _private: [u8; 0],
}

pub type shader_t = *mut shader_s;

// CTerVert class
#[repr(C)]
pub struct CTerVert {
    pub coords: vec3_t,                 // real world coords of terxel
    pub normal: vec3_t,                 // required to calculate lighting and used in physics
    pub tint: color4ub_t,               // tint at this terxel
    pub tex: [f32; 2],                  // texture coordinates at this terxel
    pub height: c_int,                  // Copy of heightmap data
    pub tessIndex: c_int,               // Index of the vert in the tess array
    pub tessRegistration: c_int,        // ...... for the tess with this registration
}

impl CTerVert {
    #[inline]
    pub fn new() -> Self {
        CTerVert {
            coords: [0.0; 3],
            normal: [0.0; 3],
            tint: [0; 4],
            tex: [0.0; 2],
            height: 0,
            tessIndex: 0,
            tessRegistration: 0,
        }
    }
}

// CTRHeightDetails class
#[repr(C)]
pub struct CTRHeightDetails {
    mShader: qhandle_t,
}

impl CTRHeightDetails {
    #[inline]
    pub fn new() -> Self {
        CTRHeightDetails { mShader: 0 }
    }

    #[inline]
    pub fn GetShader(&self) -> qhandle_t {
        self.mShader
    }

    #[inline]
    pub fn SetShader(&mut self, shader: qhandle_t) {
        self.mShader = shader;
    }
}

// Information of each patch (tessellated area) of a CTRLandScape
#[repr(C)]
pub struct CTRPatch {
    owner: *mut CCMLandScape,
    localowner: *mut CTRLandScape,

    common: *mut CCMPatch,
    mCenter: vec3_t,                    // Real world center of the patch
    //	mNormal: [vec3_t; 2],
    //	mDistance: [f32; 2],

    mRenderMap: *mut CTerVert,          // Modulation value and texture coords per vertex
    mTLShader: shader_t,                // Dynamically created blended shader for the top left triangle
    mBRShader: shader_t,                // Dynamically created blended shader for the bottom right triangle

    misVisible: bool,                   // Is this patch visible in the current frame?
}

impl CTRPatch {
    #[inline]
    pub fn new() -> Self {
        CTRPatch {
            owner: null_mut(),
            localowner: null_mut(),
            common: null_mut(),
            mCenter: [0.0; 3],
            mRenderMap: null_mut(),
            mTLShader: null_mut(),
            mBRShader: null_mut(),
            misVisible: false,
        }
    }

    // Accessors
    // NOTE: These forward to C++ CCMPatch methods; implementations in C++ code
    pub fn GetWorld(&self) -> *const vec3_t;
    pub fn GetMins(&self) -> *const vec3_t;
    pub fn GetMaxs(&self) -> *const vec3_t;
    pub fn GetBounds(&self) -> *const vec3pair_t;

    #[inline]
    pub fn GetTLShader(&self) -> shader_t {
        self.mTLShader
    }

    #[inline]
    pub fn GetBRShader(&self) -> shader_t {
        self.mBRShader
    }

    #[inline]
    pub fn SetCommon(&mut self, in_common: *mut CCMPatch) {
        self.common = in_common;
    }

    #[inline]
    pub fn GetCommon(&self) -> *const CCMPatch {
        self.common as *const _
    }

    #[inline]
    pub fn isVisible(&self) -> bool {
        self.misVisible
    }

    pub fn SetTLShader(&mut self, in_handle: qhandle_t) {
        // SAFETY: R_GetShaderByHandle must be implemented externally
        unsafe {
            self.mTLShader = R_GetShaderByHandle(in_handle);
        }
    }

    pub fn SetBRShader(&mut self, in_handle: qhandle_t) {
        // SAFETY: R_GetShaderByHandle must be implemented externally
        unsafe {
            self.mBRShader = R_GetShaderByHandle(in_handle);
        }
    }

    #[inline]
    pub fn SetOwner(&mut self, in_owner: *mut CCMLandScape) {
        self.owner = in_owner;
    }

    #[inline]
    pub fn SetLocalOwner(&mut self, in_localowner: *mut CTRLandScape) {
        self.localowner = in_localowner;
    }

    #[inline]
    pub fn Clear(&mut self) {
        // SAFETY: self is valid
        unsafe {
            core::ptr::write_bytes(self as *mut _ as *mut u8, 0, core::mem::size_of::<Self>());
        }
    }

    pub fn SetCenter(&mut self);
    // NOTE: C++ implementation: VectorAverage(common->GetMins(), common->GetMaxs(), mCenter)

    pub fn CalcNormal(&mut self);

    // Prototypes
    pub fn SetVisibility(&mut self, visCheck: bool);
    pub fn RenderCorner(&mut self, corner: ivec5_t);
    pub fn Render(&mut self, Part: c_int);
    pub fn RecurseRender(&mut self, depth: c_int, left: ivec5_t, right: ivec5_t, apex: ivec5_t);
    pub fn SetRenderMap(&mut self, x: c_int, y: c_int);
    pub fn RenderWaterVert(&mut self, x: c_int, y: c_int) -> c_int;
    pub fn RenderWater(&mut self);
    pub fn HasWater(&self) -> bool;
}

#[repr(C)]
pub struct SPatchInfo {
    pub mPatch: *mut CTRPatch,
    pub mShader: shader_t,
    pub mPart: c_int,
}

pub type TPatchInfo = SPatchInfo;

// The master class used to define an area of terrain
#[repr(C)]
pub struct CTRLandScape {
    common: *const CCMLandScape,
    mTRPatches: *mut CTRPatch,          // Local patch info
    mSortedPatches: *mut TPatchInfo,

    mPatchMinx: c_int,
    mPatchMaxx: c_int,
    mPatchMiny: c_int,
    mPatchMaxy: c_int,
    mMaxNode: c_int,                    // terxels * terxels = exit condition for splitting
    mSortedCount: c_int,

    mPatchSize: f32,

    mShader: shader_t,                  // shader the terrain got its contents from

    mRenderMap: *mut CTerVert,          // modulation value and texture coords per vertex
    mTextureScale: f32,                 // Scale of texture mapped to terrain

    mScalarSize: f32,

    mWaterShader: shader_t,             // Water shader
    mFlatShader: qhandle_t,             // Flat ground shader

    mHeightDetails: [CTRHeightDetails; HEIGHT_RESOLUTION],  // Array of info specific to height
    // NOTE: mCycleCount is conditionally included in C++ (_DEBUG builds only),
    // but included unconditionally here to maintain struct layout compatibility
    mCycleCount: c_int,
}

impl CTRLandScape {
    pub fn new(configstring: *const c_char) -> Self;

    // Accessors - NOTE: These forward to C++ CCMLandScape methods; implementations in C++ code
    pub fn GetBlockWidth(&self) -> c_int;
    pub fn GetBlockHeight(&self) -> c_int;
    pub fn GetMins(&self) -> *const vec3_t;
    pub fn GetMaxs(&self) -> *const vec3_t;
    pub fn GetTerxelSize(&self) -> *const vec3_t;
    pub fn GetPatchSize(&self) -> *const vec3_t;
    pub fn GetWidth(&self) -> c_int;
    pub fn GetHeight(&self) -> c_int;
    pub fn GetRealWidth(&self) -> c_int;
    pub fn GetRealHeight(&self) -> c_int;

    #[inline]
    pub fn SetCommon(&mut self, landscape: *const CCMLandScape) {
        self.common = landscape;
    }

    #[inline]
    pub fn GetCommon(&self) -> *const CCMLandScape {
        self.common
    }

    pub fn GetCommonId(&self) -> thandle_t;
    // NOTE: C++ implementation calls common->GetTerrainId()

    #[inline]
    pub fn GetShader(&self) -> shader_t {
        self.mShader
    }

    pub fn GetRenderMap(&self, x: c_int, y: c_int) -> *mut CTerVert;
    // NOTE: C++ implementation: return mRenderMap + x + (y * common->GetRealWidth())

    pub fn GetPatch(&self, x: c_int, y: c_int) -> *mut CTRPatch;
    // NOTE: C++ implementation: return mTRPatches + (common->GetBlockWidth() * y) + x

    pub fn GetHeightDetail(&self, height: c_int) -> *const CTRHeightDetails {
        // SAFETY: height must be in bounds [0, HEIGHT_RESOLUTION)
        // Direct pointer arithmetic to match C++ behavior: mHeightDetails + height
        unsafe {
            self.mHeightDetails.as_ptr().add(height as usize)
        }
    }

    #[inline]
    pub fn GetScalarSize(&self) -> f32 {
        self.mScalarSize
    }

    #[inline]
    pub fn GetMaxNode(&self) -> c_int {
        self.mMaxNode
    }

    // Prototypes
    pub fn CalculateRegion(&mut self);
    pub fn Reset(&mut self, visCheck: bool);
    pub fn Render(&mut self);
    pub fn CalculateRealCoords(&mut self);
    pub fn CalculateNormals(&mut self);
    pub fn CalculateTextureCoords(&mut self);
    pub fn CalculateLighting(&mut self);
    pub fn CalculateShaders(&mut self);
    pub fn GetBlendedShader(&mut self, a: qhandle_t, b: qhandle_t, c: qhandle_t, surfaceSprites: bool) -> qhandle_t;
    pub fn LoadTerrainDef(&mut self, td: *const c_char);
    pub fn CopyHeightMap(&mut self);
    pub fn SetShaders(&mut self, height: c_int, shader: qhandle_t);
}

// External function declarations
extern "C" {
    pub fn R_GetShaderByHandle(handle: qhandle_t) -> shader_t;

    pub fn VectorAverage(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        out: *mut vec3_t,
    );

    pub fn R_CalcTerrainVisBounds(landscape: *mut CTRLandScape);
    pub fn R_AddTerrainSurfaces();
}
