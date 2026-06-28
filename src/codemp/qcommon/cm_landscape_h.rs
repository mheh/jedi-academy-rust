//! `cm_landscape.h` — common terrain landscape declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, vec3_t, vec3pair_t};
use crate::codemp::qcommon::files_h::cvar_t;
use core::ffi::{c_char, c_int, c_ulong, c_void};

// These are the root classes using data shared in both the server and the renderer.
// This common data is also available to physics

pub const HEIGHT_RESOLUTION: usize = 256;

// Trying to make a guess at the optimal step through the patches
// This is the average of 1 side and the diagonal presuming a square patch
pub const TERRAIN_STEP_MAGIC: f32 = 1.0f32 / 1.2071f32;

pub const MIN_TERXELS: c_int = 2;
pub const MAX_TERXELS: c_int = 8;
// Defined as 1 << (sqrt(MAX_TERXELS) + 1)
pub const MAX_VARIANCE_SIZE: c_int = 16;

// Maximum number of instances to pick from an instance file
pub const MAX_INSTANCE_TYPES: c_int = 16;

// Header-local stubs for q_shared.h/qcommon terrain symbols not yet ported here.
pub type thandle_t = c_int;

#[repr(C)]
pub struct cbrush_s {
    _private: [u8; 0],
}

#[repr(C)]
pub struct cbrushside_s {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CCMShader {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CRandomTerrain {
    _private: [u8; 0],
}

// Opaque storage stubs for the C++ STL list members. The original ABI layout is
// implementation-specific; these keep the declarations structurally visible only.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct areaList_t {
    pub _opaque: [usize; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct areaIter_t {
    pub _opaque: *mut c_void,
}

// Types of areas

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum areaType_t {
    AT_NONE,
    AT_FLAT,
    AT_BSP,
    AT_NPC,
    AT_GROUP,
    AT_RIVER,
    AT_OBJECTIVE,
    AT_PLAYER,
}

#[repr(C)]
pub struct CArea {
    pub mPosition: vec3_t,
    pub mRadius: f32,
    pub mAngle: f32,
    pub mAngleDiff: f32,
    pub mType: c_int,
    pub mVillageID: c_int,
}

impl CArea {
    // C++ member declarations, in class order:
    // public: CArea, ~CArea, Init, GetRadius, GetAngle, GetAngleDiff,
    // GetPosition, GetType, GetVillageID

    pub unsafe fn Init(
        &mut self,
        pos: *const vec3_t,
        radius: f32,
        angle: f32,
        type_: c_int,
        angleDiff: f32,
        villageID: c_int,
    ) {
        self.mPosition = *pos;
        self.mRadius = radius;
        self.mAngle = angle;
        self.mAngleDiff = angleDiff;
        self.mType = type_;
        self.mVillageID = villageID;
    }

    #[inline]
    pub fn GetRadius(&self) -> f32 {
        self.mRadius
    }

    #[inline]
    pub fn GetAngle(&self) -> f32 {
        self.mAngle
    }

    #[inline]
    pub fn GetAngleDiff(&self) -> f32 {
        self.mAngleDiff
    }

    #[inline]
    pub fn GetPosition(&mut self) -> *mut vec3_t {
        &mut self.mPosition
    }

    #[inline]
    pub fn GetType(&self) -> c_int {
        self.mType
    }

    #[inline]
    pub fn GetVillageID(&self) -> c_int {
        self.mVillageID
    }
}

#[repr(C)]
pub struct CCMHeightDetails {
    pub mContents: c_int,
    pub mSurfaceFlags: c_int,
}

impl CCMHeightDetails {
    // C++ member declarations, in class order:
    // public: CCMHeightDetails, ~CCMHeightDetails, GetSurfaceFlags,
    // GetContents, SetFlags

    // Accessors
    #[inline]
    pub fn GetSurfaceFlags(&self) -> c_int {
        self.mSurfaceFlags
    }

    #[inline]
    pub fn GetContents(&self) -> c_int {
        self.mContents
    }

    #[inline]
    pub fn SetFlags(&mut self, con: c_int, sf: c_int) {
        self.mContents = con;
        self.mSurfaceFlags = sf;
    }
}

#[repr(C)]
pub struct CCMPatch {
    pub owner: *mut CCMLandScape,             // Owning landscape
    pub mHx: c_int,
    pub mHy: c_int,                          // Terxel coords of patch
    pub mHeightMap: *mut byte,               // Pointer to height map to use
    pub mCornerHeights: [byte; 4],           // Heights at the corners of the patch
    pub mWorldCoords: vec3_t,                // World coordinate offset of this patch.
    pub mBounds: vec3pair_t,                 // mins and maxs of the patch for culling
    pub mNumBrushes: c_int,                  // number of brushes to collide with in the patch
    pub mPatchBrushData: *mut cbrush_s,      // List of brushes that make up the patch
    pub mSurfaceFlags: c_int,                // surfaceflag of the heightshader
    pub mContentFlags: c_int,                // contents of the heightshader
}

impl CCMPatch {
    // C++ member declarations, in class order:
    // public: CCMPatch, ~CCMPatch, GetWorld, GetMins, GetMaxs, GetBounds,
    // GetHeightMapX, GetHeightMapY, GetHeight, GetNumBrushes,
    // GetCollisionData, SetSurfaceFlags, GetSurfaceFlags, SetContents,
    // GetContents, Init, InitPlane, CreatePatchPlaneData,
    // GetAdjacentBrushX, GetAdjacentBrushY

    // Accessors
    #[inline]
    pub fn GetWorld(&self) -> *const vec3_t {
        &self.mWorldCoords
    }

    #[inline]
    pub fn GetMins(&self) -> *const vec3_t {
        &self.mBounds[0]
    }

    #[inline]
    pub fn GetMaxs(&self) -> *const vec3_t {
        &self.mBounds[1]
    }

    #[inline]
    pub fn GetBounds(&self) -> *const vec3pair_t {
        &self.mBounds
    }

    #[inline]
    pub fn GetHeightMapX(&self) -> c_int {
        self.mHx
    }

    #[inline]
    pub fn GetHeightMapY(&self) -> c_int {
        self.mHy
    }

    #[inline]
    pub unsafe fn GetHeight(&self, corner: c_int) -> c_int {
        *self.mCornerHeights.as_ptr().add(corner as usize) as c_int
    }

    #[inline]
    pub fn GetNumBrushes(&self) -> c_int {
        self.mNumBrushes
    }

    #[inline]
    pub fn GetCollisionData(&self) -> *mut cbrush_s {
        self.mPatchBrushData
    }

    #[inline]
    pub fn SetSurfaceFlags(&mut self, in_: c_int) {
        self.mSurfaceFlags = in_;
    }

    #[inline]
    pub fn GetSurfaceFlags(&self) -> c_int {
        self.mSurfaceFlags
    }

    #[inline]
    pub fn SetContents(&mut self, in_: c_int) {
        self.mContentFlags = in_;
    }

    #[inline]
    pub fn GetContents(&self) -> c_int {
        self.mContentFlags
    }
}

#[repr(C)]
pub struct CCMLandScape {
    pub mRefCount: c_int,                            // Number of times this class is referenced
    pub mTerrainHandle: thandle_t,
    pub mHeightMap: *mut byte,                       // Pointer to byte array of height samples
    pub mFlattenMap: *mut byte,                      // Pointer to byte array of flatten samples
    pub mWidth: c_int,
    pub mHeight: c_int,                              // Width and height of heightMap excluding the 1 pixel edge
    pub mTerxels: c_int,                             // Number of terxels per patch side
    pub mTerxelSize: vec3_t,                         // Vector to scale heightMap samples to real world coords
    pub mBounds: vec3pair_t,                         // Real world bounds of terrain brush
    pub mSize: vec3_t,                               // Size of terrain brush in real world coords excluding 1 patch edge
    pub mPatchSize: vec3_t,                          // Size of each patch in the x and y directions only
    pub mPatchScalarSize: f32,                       // Horizontal size of the patch
    pub mBlockWidth: c_int,
    pub mBlockHeight: c_int,                         // Width and height of heightfield on blocks
    pub mPatches: *mut CCMPatch,
    pub mPatchBrushData: *mut byte,                  // Base memory from which the patch brush data is taken
    pub mHasPhysics: bool,                           // Set to true unless disabled
    pub mRandomTerrain: *mut CRandomTerrain,

    pub mBaseWaterHeight: c_int,                     // Base water height in terxels
    pub mWaterHeight: f32,                           // Real world height of the water
    pub mWaterContents: c_int,                       // Contents of the water shader
    pub mWaterSurfaceFlags: c_int,                   // Surface flags of the water shader

    pub holdrand: c_ulong,

    pub mAreas: areaList_t,                          // List of flattened areas on this landscape
    pub mAreasIt: areaIter_t,

    pub mHeightDetails: [CCMHeightDetails; HEIGHT_RESOLUTION], // Surfaceflags per height
    pub mCoords: *mut vec3_t,                        // Temp storage for real world coords
}

impl CCMLandScape {
    // Accessors
    #[inline]
    pub fn GetRefCount(&self) -> c_int {
        self.mRefCount
    }

    #[inline]
    pub fn IncreaseRefCount(&mut self) {
        self.mRefCount += 1;
    }

    #[inline]
    pub fn DecreaseRefCount(&mut self) {
        self.mRefCount -= 1;
    }

    #[inline]
    pub fn GetBounds(&self) -> *const vec3pair_t {
        &self.mBounds
    }

    #[inline]
    pub fn GetMins(&self) -> *const vec3_t {
        &self.mBounds[0]
    }

    #[inline]
    pub fn GetMaxs(&self) -> *const vec3_t {
        &self.mBounds[1]
    }

    #[inline]
    pub fn GetSize(&self) -> *const vec3_t {
        &self.mSize
    }

    #[inline]
    pub fn GetTerxelSize(&self) -> *const vec3_t {
        &self.mTerxelSize
    }

    #[inline]
    pub fn GetPatchSize(&self) -> *const vec3_t {
        &self.mPatchSize
    }

    #[inline]
    pub fn GetPatchWidth(&self) -> f32 {
        self.mPatchSize[0]
    }

    #[inline]
    pub fn GetPatchHeight(&self) -> f32 {
        self.mPatchSize[1]
    }

    #[inline]
    pub fn GetPatchScalarSize(&self) -> f32 {
        self.mPatchScalarSize
    }

    #[inline]
    pub fn GetTerxels(&self) -> c_int {
        self.mTerxels
    }

    #[inline]
    pub fn GetRealWidth(&self) -> c_int {
        self.mWidth + 1
    }

    #[inline]
    pub fn GetRealHeight(&self) -> c_int {
        self.mHeight + 1
    }

    #[inline]
    pub fn GetRealArea(&self) -> c_int {
        (self.mWidth + 1) * (self.mHeight + 1)
    }

    #[inline]
    pub fn GetWidth(&self) -> c_int {
        self.mWidth
    }

    #[inline]
    pub fn GetHeight(&self) -> c_int {
        self.mHeight
    }

    #[inline]
    pub fn GetArea(&self) -> c_int {
        self.mWidth * self.mHeight
    }

    #[inline]
    pub fn GetBlockWidth(&self) -> c_int {
        self.mBlockWidth
    }

    #[inline]
    pub fn GetBlockHeight(&self) -> c_int {
        self.mBlockHeight
    }

    #[inline]
    pub fn GetBlockCount(&self) -> c_int {
        self.mBlockWidth * self.mBlockHeight
    }

    #[inline]
    pub fn GetHeightMap(&self) -> *mut byte {
        self.mHeightMap
    }

    #[inline]
    pub fn GetFlattenMap(&self) -> *mut byte {
        self.mFlattenMap
    }

    #[inline]
    pub fn GetTerrainId(&self) -> thandle_t {
        self.mTerrainHandle
    }

    #[inline]
    pub fn SetTerrainId(&mut self, terrainId: thandle_t) {
        self.mTerrainHandle = terrainId;
    }

    #[inline]
    pub fn CalcWorldHeight(&self, height: c_int) -> f32 {
        (height as f32 * self.mTerxelSize[2]) + self.mBounds[0][2]
    }

    #[inline]
    pub fn GetHasPhysics(&self) -> bool {
        self.mHasPhysics
    }

    #[inline]
    pub fn GetIsRandom(&self) -> bool {
        !self.mRandomTerrain.is_null()
    }

    #[inline]
    pub unsafe fn GetSurfaceFlags(&self, height: c_int) -> c_int {
        (*self.mHeightDetails.as_ptr().add(height as usize)).GetSurfaceFlags()
    }

    #[inline]
    pub unsafe fn GetContentFlags(&self, height: c_int) -> c_int {
        (*self.mHeightDetails.as_ptr().add(height as usize)).GetContents()
    }

    #[inline]
    pub fn GetCoords(&self) -> *mut vec3_t {
        self.mCoords
    }

    #[inline]
    pub fn GetBaseWaterHeight(&self) -> c_int {
        self.mBaseWaterHeight
    }

    #[inline]
    pub fn SetRealWaterHeight(&mut self, height: c_int) {
        self.mWaterHeight = height as f32 * self.mTerxelSize[2];
    }

    #[inline]
    pub fn GetWaterHeight(&self) -> f32 {
        self.mWaterHeight
    }

    #[inline]
    pub fn GetWaterContents(&self) -> c_int {
        self.mWaterContents
    }

    #[inline]
    pub fn GetWaterSurfaceFlags(&self) -> c_int {
        self.mWaterSurfaceFlags
    }

    #[inline]
    pub fn GetRandomTerrain(&mut self) -> *mut CRandomTerrain {
        self.mRandomTerrain
    }

    #[inline]
    pub fn get_rand_seed(&self) -> c_ulong {
        self.holdrand
    }
}

// CCMLandScape C++ member declarations, in class order:
// public: CCMLandScape, ~CCMLandScape, GetPatch, PatchCollide,
// TerrainPatchIterate, GetWorldHeight, WaterCollide, UpdatePatches,
// GetTerxelLocalCoords, LoadTerrainDef, SetShaders, FlattenArea, CarveLine,
// CarveBezierCurve, SaveArea, FractionBelowLevel, AreaCollision, GetFirstArea,
// GetFirstObjectiveArea, GetPlayerArea, GetNextArea, GetNextObjectiveArea,
// GetRefCount, IncreaseRefCount, DecreaseRefCount, GetBounds, GetMins, GetMaxs,
// GetSize, GetTerxelSize, GetPatchSize, GetPatchWidth, GetPatchHeight,
// GetPatchScalarSize, GetTerxels, GetRealWidth, GetRealHeight, GetRealArea,
// GetWidth, GetHeight, GetArea, GetBlockWidth, GetBlockHeight, GetBlockCount,
// GetHeightMap, GetFlattenMap, GetTerrainId, SetTerrainId, CalcWorldHeight,
// GetHasPhysics, GetIsRandom, GetSurfaceFlags, GetContentFlags, CalcRealCoords,
// GetCoords, GetBaseWaterHeight, SetRealWaterHeight, GetWaterHeight,
// GetWaterContents, GetWaterSurfaceFlags, GetRandomTerrain, rand_seed,
// get_rand_seed, flrand, irand

unsafe extern "C" {
    pub fn CM_TerrainPatchIterate(
        ls: *const CCMLandScape,
        IterateFunc: Option<unsafe extern "C" fn(*mut CCMPatch, *mut c_void)>,
        userdata: *mut c_void,
    );
    pub fn CM_InitTerrain(
        configstring: *const c_char,
        terrainId: thandle_t,
        server: bool,
    ) -> *mut CCMLandScape;
    pub fn CM_GetWorldHeight(
        landscape: *const CCMLandScape,
        origin: *mut vec3_t,
        bounds: *const vec3pair_t,
        aboveGround: bool,
    ) -> f32;
    pub fn CM_FlattenArea(
        landscape: *mut CCMLandScape,
        area: *mut CArea,
        height: c_int,
        save: bool,
        forceHeight: bool,
        smooth: bool,
    );
    pub fn CM_CarveBezierCurve(
        landscape: *mut CCMLandScape,
        numCtls: c_int,
        ctls: *mut vec3_t,
        steps: c_int,
        depth: c_int,
        size: c_int,
    );
    pub fn CM_SaveArea(landscape: *mut CCMLandScape, area: *mut CArea);
    pub fn CM_FractionBelowLevel(
        landscape: *mut CCMLandScape,
        area: *mut CArea,
        height: c_int,
    ) -> f32;
    pub fn CM_AreaCollision(
        landscape: *mut CCMLandScape,
        area: *mut CArea,
        areaTypes: *mut c_int,
        areaTypeCount: c_int,
    ) -> bool;
    pub fn CM_GetFirstArea(landscape: *mut CCMLandScape) -> *mut CArea;
    pub fn CM_GetFirstObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea;
    pub fn CM_GetPlayerArea(common: *mut CCMLandScape) -> *mut CArea;
    pub fn CM_GetNextArea(landscape: *mut CCMLandScape) -> *mut CArea;
    pub fn CM_GetNextObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea;
    pub fn CM_CircularIterate(
        data: *mut byte,
        width: c_int,
        height: c_int,
        xo: c_int,
        yo: c_int,
        insideRadius: c_int,
        outsideRadius: c_int,
        user: *mut c_int,
        callback: Option<unsafe extern "C" fn(*mut byte, f32, *mut c_int)>,
    );

    pub fn CreateRandomTerrain(
        config: *const c_char,
        landscape: *mut CCMLandScape,
        heightmap: *mut byte,
        width: c_int,
        height: c_int,
    ) -> *mut CRandomTerrain;

    pub fn SV_LoadMissionDef(configstring: *const c_char, landscape: *mut CCMLandScape);
    pub fn CL_CreateRandomTerrain(
        config: *const c_char,
        landscape: *mut CCMLandScape,
        image: *mut byte,
        width: c_int,
        height: c_int,
    );
    pub fn CL_LoadInstanceDef(configstring: *const c_char, landscape: *mut CCMLandScape);
    pub fn CL_LoadMissionDef(configstring: *const c_char, landscape: *mut CCMLandScape);

    pub static mut com_terrainPhysics: *mut cvar_t;
}

// end
