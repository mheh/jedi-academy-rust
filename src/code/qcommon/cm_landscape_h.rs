// These are the root classes using data shared in both the server and the renderer.
// This common data is also available to physics

use core::ffi::c_int;
use std::collections::LinkedList;

// Trying to make a guess at the optimal step through the patches
// This is the average of 1 side and the diagonal presuming a square patch
pub const TERRAIN_STEP_MAGIC: f32 = 1.0f32 / 1.2071f32;

pub const HEIGHT_RESOLUTION: usize = 256;

pub const MIN_TERXELS: i32 = 2;
pub const MAX_TERXELS: i32 = 8;
// Defined as 1 << (sqrt(MAX_TERXELS) + 1)
pub const MAX_VARIANCE_SIZE: i32 = 16;

// Maximum number of instances to pick from an instance file
pub const MAX_INSTANCE_TYPES: i32 = 16;

// Types
pub type vec3_t = [f32; 3];
pub type vec3pair_t = [[f32; 3]; 2];
pub type byte = u8;
pub type thandle_t = c_int;

// Forward declarations for external types
pub struct cbrush_s;
pub struct cbrushside_s;
pub struct cplane_t;
pub struct traceWork_s;
pub struct trace_t;
pub struct cvar_t;
pub struct CRandomTerrain;

// Types of areas
#[repr(i32)]
#[derive(Copy, Clone, Debug)]
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

pub type areaList_t = Vec<*mut CArea>;
pub type areaIter_t = *mut CArea;

#[repr(C)]
pub struct CArea {
    mPosition: vec3_t,
    mRadius: f32,
    mAngle: f32,
    mAngleDiff: f32,
    mType: i32,
    mVillageID: i32,
}

impl CArea {
    pub fn new() -> Self {
        CArea {
            mPosition: [0.0; 3],
            mRadius: 0.0,
            mAngle: 0.0,
            mAngleDiff: 0.0,
            mType: 0,
            mVillageID: 0,
        }
    }

    pub fn Init(&mut self, pos: vec3_t, radius: f32, angle: f32, type_: i32, angleDiff: f32, villageID: i32) {
        self.mPosition = pos;
        self.mRadius = radius;
        self.mAngle = angle;
        self.mAngleDiff = angleDiff;
        self.mType = type_;
        self.mVillageID = villageID;
    }

    pub fn GetRadius(&self) -> f32 { self.mRadius }
    pub fn GetAngle(&self) -> f32 { self.mAngle }
    pub fn GetAngleDiff(&self) -> f32 { self.mAngleDiff }
    pub fn GetPosition(&mut self) -> &mut vec3_t { &mut self.mPosition }
    pub fn GetType(&self) -> i32 { self.mType }
    pub fn GetVillageID(&self) -> i32 { self.mVillageID }
}

#[repr(C)]
pub struct CCMHeightDetails {
    mContents: i32,
    mSurfaceFlags: i32,
}

impl CCMHeightDetails {
    pub fn new() -> Self {
        CCMHeightDetails {
            mContents: 0,
            mSurfaceFlags: 0,
        }
    }

    // Accessors
    pub fn GetSurfaceFlags(&self) -> i32 { self.mSurfaceFlags }
    pub fn GetContents(&self) -> i32 { self.mContents }
    pub fn SetFlags(&mut self, con: i32, sf: i32) { self.mContents = con; self.mSurfaceFlags = sf; }
}

#[repr(C)]
pub struct CCMPatch {
    owner: *mut CCMLandScape,                           // Owning landscape
    mHx: i32,
    mHy: i32,                                           // Terxel coords of patch
    mHeightMap: *mut byte,                              // Pointer to height map to use
    mCornerHeights: [byte; 4],                          // Heights at the corners of the patch
    mWorldCoords: vec3_t,                               // World coordinate offset of this patch.
    mBounds: vec3pair_t,                                // mins and maxs of the patch for culling
    mNumBrushes: i32,                                   // number of brushes to collide with in the patch
    mPatchBrushData: *mut cbrush_s,                     // List of brushes that make up the patch
    mSurfaceFlags: i32,                                 // surfaceflag of the heightshader
    mContentFlags: i32,                                 // contents of the heightshader
}

impl CCMPatch {
    pub fn new() -> Self {
        CCMPatch {
            owner: std::ptr::null_mut(),
            mHx: 0,
            mHy: 0,
            mHeightMap: std::ptr::null_mut(),
            mCornerHeights: [0; 4],
            mWorldCoords: [0.0; 3],
            mBounds: [[0.0; 3]; 2],
            mNumBrushes: 0,
            mPatchBrushData: std::ptr::null_mut(),
            mSurfaceFlags: 0,
            mContentFlags: 0,
        }
    }

    // Accessors
    pub fn GetWorld(&self) -> &vec3_t { &self.mWorldCoords }
    pub fn GetMins(&self) -> &vec3_t { &self.mBounds[0] }
    pub fn GetMaxs(&self) -> &vec3_t { &self.mBounds[1] }
    pub fn GetBounds(&self) -> &vec3pair_t { &self.mBounds }
    pub fn GetHeightMapX(&self) -> i32 { self.mHx }
    pub fn GetHeightMapY(&self) -> i32 { self.mHy }
    pub fn GetHeight(&self, corner: usize) -> byte { self.mCornerHeights[corner] }
    pub fn GetNumBrushes(&self) -> i32 { self.mNumBrushes }
    pub fn GetCollisionData(&self) -> *mut cbrush_s { self.mPatchBrushData }

    pub fn SetSurfaceFlags(&mut self, in_: i32) { self.mSurfaceFlags = in_; }
    pub fn GetSurfaceFlags(&self) -> i32 { self.mSurfaceFlags }
    pub fn SetContents(&mut self, in_: i32) { self.mContentFlags = in_; }
    pub fn GetContents(&self) -> i32 { self.mContentFlags }

    // Prototypes
    pub fn Init(&mut self, ls: *mut CCMLandScape, heightX: i32, heightY: i32, world: vec3_t, hMap: *mut byte, patchBrushData: *mut byte);
    pub fn InitPlane(&mut self, side: *mut cbrushside_s, plane: *mut cplane_t, p0: vec3_t, p1: vec3_t, p2: vec3_t);
    pub fn CreatePatchPlaneData(&mut self);

    pub fn GetAdjacentBrushX(&self, x: i32, y: i32) -> *mut c_void;
    pub fn GetAdjacentBrushY(&self, x: i32, y: i32) -> *mut c_void;
}

#[repr(C)]
pub struct CCMLandScape {
    mRefCount: i32,                                     // Number of times this class is referenced
    mTerrainHandle: thandle_t,
    mHeightMap: *mut byte,                              // Pointer to byte array of height samples
    mFlattenMap: *mut byte,                             // Pointer to byte array of flatten samples
    mWidth: i32,
    mHeight: i32,                                       // Width and height of heightMap excluding the 1 pixel edge
    mTerxels: i32,                                      // Number of terxels per patch side
    mTerxelSize: vec3_t,                                // Vector to scale heightMap samples to real world coords
    mBounds: vec3pair_t,                                // Real world bounds of terrain brush
    mSize: vec3_t,                                      // Size of terrain brush in real world coords excluding 1 patch edge
    mPatchSize: vec3_t,                                 // Size of each patch in the x and y directions only
    mPatchScalarSize: f32,                              // Horizontal size of the patch
    mBlockWidth: i32,
    mBlockHeight: i32,                                  // Width and height of heightfield on blocks
    mPatches: *mut CCMPatch,
    mPatchBrushData: *mut byte,                         // Base memory from which the patch brush data is taken
    mHasPhysics: bool,                                  // Set to true unless disabled
    mRandomTerrain: *mut CRandomTerrain,

    mBaseWaterHeight: i32,                              // Base water height in terxels
    mWaterHeight: f32,                                  // Real world height of the water
    mWaterContents: i32,                                // Contents of the water shader
    mWaterSurfaceFlags: i32,                            // Surface flags of the water shader

    holdrand: core::ffi::c_ulong,

    mAreas: LinkedList<*mut CArea>,                     // List of flattened areas on this landscape
    mAreasIt: *mut CArea,

    mHeightDetails: [CCMHeightDetails; HEIGHT_RESOLUTION], // Surfaceflags per height
    mCoords: *mut vec3_t,                               // Temp storage for real world coords
}

impl CCMLandScape {
    pub fn GetPatch(&self, x: i32, y: i32) -> *mut CCMPatch;

    // Prototypes
    pub fn PatchCollide(&mut self, tw: *mut traceWork_s, trace: &mut trace_t, start: vec3_t, end: vec3_t, checkcount: i32);
    pub fn TerrainPatchIterate(&self, IterateFunc: extern "C" fn(*mut CCMPatch, *mut c_void), userdata: *mut c_void);
    pub fn GetWorldHeight(&self, origin: vec3_t, bounds: vec3pair_t, aboveGround: bool) -> f32;
    pub fn WaterCollide(&self, begin: vec3_t, end: vec3_t, fraction: f32) -> f32;
    pub fn UpdatePatches(&mut self);
    pub fn GetTerxelLocalCoords(&self, x: i32, y: i32, coords: &mut [vec3_t; 8]);
    pub fn LoadTerrainDef(&mut self, td: *const u8);
    pub fn SetShaders(&mut self, height: i32, shader: *mut c_void);
    pub fn FlattenArea(&mut self, area: *mut CArea, height: i32, save: bool, forceHeight: bool, smooth: bool);
    pub fn CarveLine(&mut self, start: vec3_t, end: vec3_t, depth: i32, width: i32);
    pub fn CarveBezierCurve(&mut self, numCtlPoints: i32, ctlPoints: *mut vec3_t, steps: i32, depth: i32, size: i32);
    pub fn SaveArea(&mut self, area: *mut CArea);
    pub fn FractionBelowLevel(&self, area: *mut CArea, height: i32) -> f32;
    pub fn AreaCollision(&self, area: *mut CArea, areaTypes: *mut i32, areaTypeCount: i32) -> bool;
    pub fn GetFirstArea(&mut self) -> *mut CArea;
    pub fn GetFirstObjectiveArea(&mut self) -> *mut CArea;
    pub fn GetPlayerArea(&mut self) -> *mut CArea;
    pub fn GetNextArea(&mut self) -> *mut CArea;
    pub fn GetNextObjectiveArea(&mut self) -> *mut CArea;

    // Accessors
    pub fn GetRefCount(&self) -> i32 { self.mRefCount }
    pub fn IncreaseRefCount(&mut self) { self.mRefCount += 1; }
    pub fn DecreaseRefCount(&mut self) { self.mRefCount -= 1; }
    pub fn GetBounds(&self) -> &vec3pair_t { &self.mBounds }
    pub fn GetMins(&self) -> &vec3_t { &self.mBounds[0] }
    pub fn GetMaxs(&self) -> &vec3_t { &self.mBounds[1] }
    pub fn GetSize(&self) -> &vec3_t { &self.mSize }
    pub fn GetTerxelSize(&self) -> &vec3_t { &self.mTerxelSize }
    pub fn GetPatchSize(&self) -> &vec3_t { &self.mPatchSize }
    pub fn GetPatchWidth(&self) -> f32 { self.mPatchSize[0] }
    pub fn GetPatchHeight(&self) -> f32 { self.mPatchSize[1] }
    pub fn GetPatchScalarSize(&self) -> f32 { self.mPatchScalarSize }
    pub fn GetTerxels(&self) -> i32 { self.mTerxels }
    pub fn GetRealWidth(&self) -> i32 { self.mWidth + 1 }
    pub fn GetRealHeight(&self) -> i32 { self.mHeight + 1 }
    pub fn GetRealArea(&self) -> i32 { (self.mWidth + 1) * (self.mHeight + 1) }
    pub fn GetWidth(&self) -> i32 { self.mWidth }
    pub fn GetHeight(&self) -> i32 { self.mHeight }
    pub fn GetArea(&self) -> i32 { self.mWidth * self.mHeight }
    pub fn GetBlockWidth(&self) -> i32 { self.mBlockWidth }
    pub fn GetBlockHeight(&self) -> i32 { self.mBlockHeight }
    pub fn GetBlockCount(&self) -> i32 { self.mBlockWidth * self.mBlockHeight }
    pub fn GetHeightMap(&self) -> *mut byte { self.mHeightMap }
    pub fn GetFlattenMap(&self) -> *mut byte { self.mFlattenMap }
    pub fn GetTerrainId(&self) -> thandle_t { self.mTerrainHandle }
    pub fn SetTerrainId(&mut self, terrainId: thandle_t) { self.mTerrainHandle = terrainId; }
    pub fn CalcWorldHeight(&self, height: i32) -> f32 { (height as f32 * self.mTerxelSize[2]) + self.mBounds[0][2] }
    pub fn GetHasPhysics(&self) -> bool { self.mHasPhysics }
    pub fn GetIsRandom(&self) -> bool { !self.mRandomTerrain.is_null() }
    pub fn GetSurfaceFlags(&self, height: usize) -> i32 { self.mHeightDetails[height].GetSurfaceFlags() }
    pub fn GetContentFlags(&self, height: usize) -> i32 { self.mHeightDetails[height].GetContents() }
    pub fn CalcRealCoords(&mut self);
    pub fn GetCoords(&self) -> *mut vec3_t { self.mCoords }

    pub fn GetBaseWaterHeight(&self) -> i32 { self.mBaseWaterHeight }
    pub fn SetRealWaterHeight(&mut self, height: i32) { self.mWaterHeight = height as f32 * self.mTerxelSize[2]; }
    pub fn GetWaterHeight(&self) -> f32 { self.mWaterHeight }
    pub fn GetWaterContents(&self) -> i32 { self.mWaterContents }
    pub fn GetWaterSurfaceFlags(&self) -> i32 { self.mWaterSurfaceFlags }

    pub fn GetRandomTerrain(&self) -> *mut CRandomTerrain { self.mRandomTerrain }

    pub fn rand_seed(&mut self, seed: i32);
    pub fn get_rand_seed(&self) -> core::ffi::c_ulong { self.holdrand }

    pub fn flrand(&self, min: f32, max: f32) -> f32;
    pub fn irand(&self, min: i32, max: i32) -> i32;
}

pub extern "C" fn CM_TerrainPatchIterate(ls: *const CCMLandScape, IterateFunc: extern "C" fn(*mut CCMPatch, *mut c_void), userdata: *mut c_void);
pub extern "C" fn CM_InitTerrain(configstring: *const u8, terrainId: thandle_t, server: bool) -> *mut CCMLandScape;
pub extern "C" fn CM_GetWorldHeight(landscape: *const CCMLandScape, origin: vec3_t, bounds: vec3pair_t, aboveGround: bool) -> f32;
pub extern "C" fn CM_FlattenArea(landscape: *mut CCMLandScape, area: *mut CArea, height: i32, save: bool, forceHeight: bool, smooth: bool);
pub extern "C" fn CM_CarveBezierCurve(landscape: *mut CCMLandScape, numCtls: i32, ctls: *mut vec3_t, steps: i32, depth: i32, size: i32);
pub extern "C" fn CM_SaveArea(landscape: *mut CCMLandScape, area: *mut CArea);
pub extern "C" fn CM_FractionBelowLevel(landscape: *mut CCMLandScape, area: *mut CArea, height: i32) -> f32;
pub extern "C" fn CM_AreaCollision(landscape: *mut CCMLandScape, area: *mut CArea, areaTypes: *mut i32, areaTypeCount: i32) -> bool;
pub extern "C" fn CM_GetFirstArea(landscape: *mut CCMLandScape) -> *mut CArea;
pub extern "C" fn CM_GetFirstObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea;
pub extern "C" fn CM_GetPlayerArea(common: *mut CCMLandScape) -> *mut CArea;
pub extern "C" fn CM_GetNextArea(landscape: *mut CCMLandScape) -> *mut CArea;
pub extern "C" fn CM_GetNextObjectiveArea(landscape: *mut CCMLandScape) -> *mut CArea;
pub extern "C" fn CM_CircularIterate(data: *mut byte, width: i32, height: i32, xo: i32, yo: i32, insideRadius: i32, outsideRadius: i32, user: *mut i32, callback: extern "C" fn(*mut byte, f32, *mut i32));

pub extern "C" fn CreateRandomTerrain(config: *const u8, landscape: *mut CCMLandScape, heightmap: *mut byte, width: i32, height: i32) -> *mut CRandomTerrain;

pub extern "C" fn SV_LoadMissionDef(configstring: *const u8, landscape: *mut CCMLandScape);
pub extern "C" fn CL_CreateRandomTerrain(config: *const u8, landscape: *mut CCMLandScape, image: *mut byte, width: i32, height: i32);
pub extern "C" fn CL_LoadInstanceDef(configstring: *const u8, landscape: *mut CCMLandScape);
pub extern "C" fn CL_LoadMissionDef(configstring: *const u8, landscape: *mut CCMLandScape);

pub extern "C" {
    pub static mut com_terrainPhysics: *mut cvar_t;
}

// end
