#![allow(non_snake_case)]

use core::ptr;

// Number of TriTreeNodes available
pub const POOL_SIZE: usize = 50000;

pub const TEXTURE_ALPHA_TL: u32 = 0x000000ff;
pub const TEXTURE_ALPHA_TR: u32 = 0x0000ff00;
pub const TEXTURE_ALPHA_BL: u32 = 0x00ff0000;
pub const TEXTURE_ALPHA_BR: u32 = 0x000000ff;

pub const INDEX_TL: i32 = 0;
pub const INDEX_TR: i32 = 1;
pub const INDEX_BL: i32 = 2;
pub const INDEX_BR: i32 = 3;

pub const VARIANCE_MIN: f32 = 0.0f32;
pub const VARIANCE_MAX: f32 = 2000.0f32;
pub const SPLIT_VARIANCE_SIZE: usize = 20;
pub const SPLIT_VARIANCE_STEP: f32 = VARIANCE_MAX / (SPLIT_VARIANCE_SIZE as f32);

// Macro-like function to compute vector average
#[inline]
pub fn VectorAverage(a: &[f32; 3], b: &[f32; 3], c: &mut [f32; 3]) {
    c[0] = (a[0] + b[0]) * 0.5f32;
    c[1] = (a[1] + b[1]) * 0.5f32;
    c[2] = (a[2] + b[2]) * 0.5f32;
}

// Type aliases for C types
pub type vec3_t = [f32; 3];
pub type vec3pair_t = [vec3_t; 2];
pub type color4ub_t = [u8; 4];
pub type qhandle_t = i32;
pub type thandle_t = i32;
pub type ivec5_t = [i32; 5];

// Forward declarations for external types
#[repr(C)]
pub struct shader_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CCMPatch {
    _opaque: [u8; 0],
}

// External function declarations
extern "C" {
    pub fn R_GetShaderByHandle(handle: qhandle_t) -> *mut shader_t;
}

#[repr(C)]
pub struct CTerVert {
    pub coords: vec3_t,       // real world coords of terxel
    pub normal: vec3_t,       // required to calculate lighting and used in physics
    pub tint: color4ub_t,     // tint at this terxel
    pub tex: [f32; 2],        // texture coordinates at this terxel
    pub height: i32,          // Copy of heightmap data
    pub tessIndex: i32,       // Index of the vert in the tess array
    pub tessRegistration: i32, // ...... for the tess with this registration
}

impl CTerVert {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

#[repr(C)]
pub struct CTRHeightDetails {
    mShader: qhandle_t,
}

impl CTRHeightDetails {
    pub fn new() -> Self {
        CTRHeightDetails { mShader: 0 }
    }

    pub const fn GetShader(&self) -> qhandle_t {
        self.mShader
    }

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
    mCenter: vec3_t,  // Real world center of the patch
    // vec3_t mNormal[2];
    // float mDistance[2];

    mRenderMap: *mut CTerVert,   // Modulation value and texture coords per vertex
    mTLShader: *mut shader_t,    // Dynamically created blended shader for the top left triangle
    mBRShader: *mut shader_t,    // Dynamically created blended shader for the bottom right triangle

    misVisible: bool, // Is this patch visible in the current frame?
}

impl CTRPatch {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    // Accessors
    pub fn GetWorld(&self) -> &vec3_t {
        // return(common->GetWorld());
        unimplemented!("GetWorld requires external CCMPatch method")
    }

    pub fn GetMins(&self) -> &vec3_t {
        // return(common->GetMins());
        unimplemented!("GetMins requires external CCMPatch method")
    }

    pub fn GetMaxs(&self) -> &vec3_t {
        // return(common->GetMaxs());
        unimplemented!("GetMaxs requires external CCMPatch method")
    }

    pub fn GetBounds(&self) -> &vec3pair_t {
        // return(common->GetBounds());
        unimplemented!("GetBounds requires external CCMPatch method")
    }

    pub fn GetTLShader(&mut self) -> *mut shader_t {
        self.mTLShader
    }

    pub fn GetBRShader(&mut self) -> *mut shader_t {
        self.mBRShader
    }

    pub fn SetCommon(&mut self, input: *mut CCMPatch) {
        self.common = input;
    }

    pub fn GetCommon(&self) -> *const CCMPatch {
        self.common
    }

    pub fn isVisible(&self) -> bool {
        self.misVisible
    }

    pub fn SetTLShader(&mut self, input: qhandle_t) {
        self.mTLShader = unsafe { R_GetShaderByHandle(input) };
    }

    pub fn SetBRShader(&mut self, input: qhandle_t) {
        self.mBRShader = unsafe { R_GetShaderByHandle(input) };
    }

    pub fn SetOwner(&mut self, input: *mut CCMLandScape) {
        self.owner = input;
    }

    pub fn SetLocalOwner(&mut self, input: *mut CTRLandScape) {
        self.localowner = input;
    }

    pub fn Clear(&mut self) {
        unsafe {
            ptr::write_bytes(self as *mut Self as *mut u8, 0, core::mem::size_of::<Self>());
        }
    }

    pub fn SetCenter(&mut self) {
        // VectorAverage(common->GetMins(), common->GetMaxs(), mCenter);
        unimplemented!("SetCenter requires external CCMPatch method")
    }

    pub fn CalcNormal(&mut self) {
        unimplemented!("CalcNormal implementation in tr_landscape.cpp")
    }

    // Prototypes
    pub fn SetVisibility(&mut self, _visCheck: bool) {
        unimplemented!("SetVisibility implementation in tr_landscape.cpp")
    }

    pub fn RenderCorner(&mut self, _corner: ivec5_t) {
        unimplemented!("RenderCorner implementation in tr_landscape.cpp")
    }

    pub fn Render(&mut self, _Part: i32) {
        unimplemented!("Render implementation in tr_landscape.cpp")
    }

    pub fn RecurseRender(&mut self, _depth: i32, _left: ivec5_t, _right: ivec5_t, _apex: ivec5_t) {
        unimplemented!("RecurseRender implementation in tr_landscape.cpp")
    }

    pub fn SetRenderMap(&mut self, _x: i32, _y: i32) {
        unimplemented!("SetRenderMap implementation in tr_landscape.cpp")
    }

    pub fn RenderWaterVert(&mut self, _x: i32, _y: i32) -> i32 {
        unimplemented!("RenderWaterVert implementation in tr_landscape.cpp")
    }

    pub fn RenderWater(&mut self) {
        unimplemented!("RenderWater implementation in tr_landscape.cpp")
    }

    pub fn HasWater(&self) -> bool {
        unimplemented!("HasWater implementation in tr_landscape.cpp")
    }
}

pub const PI_TOP: i32 = 1;
pub const PI_BOTTOM: i32 = 2;
pub const PI_BOTH: i32 = 3;

#[repr(C)]
pub struct TPatchInfo {
    pub mPatch: *mut CTRPatch,
    pub mShader: *mut shader_t,
    pub mPart: i32,
}

// The master class used to define an area of terrain
#[repr(C)]
pub struct CTRLandScape {
    common: *const CCMLandScape,
    mTRPatches: *mut CTRPatch,         // Local patch info
    mSortedPatches: *mut TPatchInfo,

    mPatchMinx: i32,
    mPatchMaxx: i32,
    mPatchMiny: i32,
    mPatchMaxy: i32,
    mMaxNode: i32,            // terxels * terxels = exit condition for splitting
    mSortedCount: i32,

    mPatchSize: f32,

    mShader: *mut shader_t, // shader the terrain got its contents from

    mRenderMap: *mut CTerVert,  // modulation value and texture coords per vertex
    mTextureScale: f32,         // Scale of texture mapped to terrain

    mScalarSize: f32,

    mWaterShader: *mut shader_t, // Water shader
    mFlatShader: qhandle_t,      // Flat ground shader

    mHeightDetails: [CTRHeightDetails; 256], // Array of info specific to height (HEIGHT_RESOLUTION = 256)
    #[cfg(debug_assertions)]
    mCycleCount: i32,
}

impl CTRLandScape {
    pub fn new(_configstring: *const i8) -> Self {
        unsafe { core::mem::zeroed() }
    }

    // Accessors
    pub fn GetBlockWidth(&self) -> i32 {
        // return(common->GetBlockWidth());
        unimplemented!("GetBlockWidth requires external CCMLandScape method")
    }

    pub fn GetBlockHeight(&self) -> i32 {
        // return(common->GetBlockHeight());
        unimplemented!("GetBlockHeight requires external CCMLandScape method")
    }

    pub fn GetMins(&self) -> &vec3_t {
        // return(common->GetMins());
        unimplemented!("GetMins requires external CCMLandScape method")
    }

    pub fn GetMaxs(&self) -> &vec3_t {
        // return(common->GetMaxs());
        unimplemented!("GetMaxs requires external CCMLandScape method")
    }

    pub fn GetTerxelSize(&self) -> &vec3_t {
        // return(common->GetTerxelSize());
        unimplemented!("GetTerxelSize requires external CCMLandScape method")
    }

    pub fn GetPatchSize(&self) -> &vec3_t {
        // return(common->GetPatchSize());
        unimplemented!("GetPatchSize requires external CCMLandScape method")
    }

    pub fn GetWidth(&self) -> i32 {
        // return(common->GetWidth());
        unimplemented!("GetWidth requires external CCMLandScape method")
    }

    pub fn GetHeight(&self) -> i32 {
        // return(common->GetHeight());
        unimplemented!("GetHeight requires external CCMLandScape method")
    }

    pub fn GetRealWidth(&self) -> i32 {
        // return(common->GetRealWidth());
        unimplemented!("GetRealWidth requires external CCMLandScape method")
    }

    pub fn GetRealHeight(&self) -> i32 {
        // return(common->GetRealHeight());
        unimplemented!("GetRealHeight requires external CCMLandScape method")
    }

    pub fn SetCommon(&mut self, landscape: *const CCMLandScape) {
        self.common = landscape;
    }

    pub fn GetCommon(&self) -> *const CCMLandScape {
        self.common
    }

    pub fn GetCommonId(&self) -> thandle_t {
        // return(common->GetTerrainId());
        unimplemented!("GetCommonId requires external CCMLandScape method")
    }

    pub fn GetShader(&self) -> *mut shader_t {
        self.mShader
    }

    pub fn GetRenderMap(&self, x: i32, y: i32) -> *mut CTerVert {
        // return(mRenderMap + x + (y * common->GetRealWidth()));
        unsafe { self.mRenderMap.add(x as usize + (y as usize * self.GetRealWidth() as usize)) }
    }

    pub fn GetPatch(&self, x: i32, y: i32) -> *mut CTRPatch {
        // return(mTRPatches + (common->GetBlockWidth() * y) + x);
        unsafe {
            self.mTRPatches.add((self.GetBlockWidth() as usize * y as usize) + x as usize)
        }
    }

    pub fn GetHeightDetail(&self, height: i32) -> *const CTRHeightDetails {
        unsafe { ptr::addr_of!(self.mHeightDetails[height as usize]) }
    }

    pub fn GetScalarSize(&self) -> f32 {
        self.mScalarSize
    }

    pub fn GetMaxNode(&self) -> i32 {
        self.mMaxNode
    }

    // Prototypes
    pub fn CalculateRegion(&mut self) {
        unimplemented!("CalculateRegion implementation in tr_landscape.cpp")
    }

    pub fn Reset(&mut self, _visCheck: bool) {
        unimplemented!("Reset implementation in tr_landscape.cpp")
    }

    pub fn Render(&mut self) {
        unimplemented!("Render implementation in tr_landscape.cpp")
    }

    pub fn CalculateRealCoords(&mut self) {
        unimplemented!("CalculateRealCoords implementation in tr_landscape.cpp")
    }

    pub fn CalculateNormals(&mut self) {
        unimplemented!("CalculateNormals implementation in tr_landscape.cpp")
    }

    pub fn CalculateTextureCoords(&mut self) {
        unimplemented!("CalculateTextureCoords implementation in tr_landscape.cpp")
    }

    pub fn CalculateLighting(&mut self) {
        unimplemented!("CalculateLighting implementation in tr_landscape.cpp")
    }

    pub fn CalculateShaders(&mut self) {
        unimplemented!("CalculateShaders implementation in tr_landscape.cpp")
    }

    pub fn GetBlendedShader(
        &mut self,
        _a: qhandle_t,
        _b: qhandle_t,
        _c: qhandle_t,
        _surfaceSprites: bool,
    ) -> qhandle_t {
        unimplemented!("GetBlendedShader implementation in tr_landscape.cpp")
    }

    pub fn LoadTerrainDef(&mut self, _td: *const i8) {
        unimplemented!("LoadTerrainDef implementation in tr_landscape.cpp")
    }

    pub fn CopyHeightMap(&mut self) {
        unimplemented!("CopyHeightMap implementation in tr_landscape.cpp")
    }

    pub fn SetShaders(&mut self, _height: i32, _shader: qhandle_t) {
        unimplemented!("SetShaders implementation in tr_landscape.cpp")
    }
}

// External function declarations
extern "C" {
    pub fn R_CalcTerrainVisBounds(landscape: *mut CTRLandScape);
    pub fn R_AddTerrainSurfaces();
}
