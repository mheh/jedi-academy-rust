// cm_landscape.h — common terrain landscape types shared by server and renderer.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports)]

use core::ffi::{c_char, c_int, c_ulong, c_void};
use std::collections::LinkedList;

// Porting note: thandle_t imported from q_shared_h per TRIAGE CAUTIONS (oracle uses via value, no #include in header)
use crate::codemp::qcommon::q_shared_h::*;
// Porting note: cbrush_s, cbrushside_s, CCMShader imported from cm_local_h per TRIAGE CAUTIONS
// (oracle forward-declares and uses via pointer; no #include in header)
use crate::codemp::qcommon::cm_local_h::*;

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

// Types of areas

#[repr(C)]
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

pub use areaType_t::*;

#[repr(C)]
pub struct CArea {
    // private:
    mPosition: vec3_t,
    mRadius: f32,
    mAngle: f32,
    mAngleDiff: f32,
    mType: c_int,
    mVillageID: c_int,
}

impl CArea {
    // CArea(void) {}
    pub unsafe fn new() -> Self {
        core::mem::zeroed()
    }

    // ~CArea(void) {}

    pub unsafe fn Init(
        &mut self,
        pos: vec3_t,
        radius: f32,
        angle: f32,
        type_: c_int,
        angleDiff: f32,
        villageID: c_int,
    ) {
        self.mPosition = pos; // VectorCopy(pos, mPosition)
        self.mRadius = radius;
        self.mAngle = angle;
        self.mAngleDiff = angleDiff;
        self.mType = type_;
        self.mVillageID = villageID;
    }

    #[inline]
    pub fn GetRadius(&self) -> f32 { self.mRadius }
    #[inline]
    pub fn GetAngle(&self) -> f32 { self.mAngle }
    #[inline]
    pub fn GetAngleDiff(&self) -> f32 { self.mAngleDiff }
    #[inline]
    pub fn GetPosition(&mut self) -> &mut vec3_t { &mut self.mPosition }
    #[inline]
    pub fn GetType(&self) -> c_int { self.mType }
    #[inline]
    pub fn GetVillageID(&self) -> c_int { self.mVillageID }
}

// typedef list<CArea*>              areaList_t;
pub type areaList_t = LinkedList<*mut CArea>;
// typedef list<CArea*>::iterator    areaIter_t;
// Porting note: C++ std::list<CArea*>::iterator; translated as IterMut<'static, *mut CArea>.
// The stored mAreasIt field in CCMLandScape is a self-referential pattern in C++;
// requires unsafe access in Rust and cannot be safely default-initialized.
pub type areaIter_t = std::collections::linked_list::IterMut<'static, *mut CArea>;

#[repr(C)]
pub struct CCMHeightDetails {
    // private:
    mContents: c_int,
    mSurfaceFlags: c_int,
}

impl CCMHeightDetails {
    // CCMHeightDetails(void) {}
    pub unsafe fn new() -> Self {
        core::mem::zeroed()
    }

    // ~CCMHeightDetails(void) {}

    // Accessors
    #[inline]
    pub fn GetSurfaceFlags(&self) -> c_int { self.mSurfaceFlags }
    #[inline]
    pub fn GetContents(&self) -> c_int { self.mContents }
    #[inline]
    pub fn SetFlags(&mut self, con: c_int, sf: c_int) {
        self.mContents = con;
        self.mSurfaceFlags = sf;
    }
}

// class CRandomTerrain; — local forward declaration; type defined elsewhere in the codebase
// Porting note: unit struct stands in for the C++ forward declaration; only used via raw pointer.
pub struct CRandomTerrain;

#[repr(C)]
pub struct CCMPatch {
    // protected:
    owner: *mut CCMLandScape,              // Owning landscape
    mHx: c_int,
    mHy: c_int,                           // Terxel coords of patch
    mHeightMap: *mut byte,                // Pointer to height map to use
    mCornerHeights: [byte; 4],            // Heights at the corners of the patch
    mWorldCoords: vec3_t,                 // World coordinate offset of this patch.
    mBounds: vec3pair_t,                  // mins and maxs of the patch for culling
    mNumBrushes: c_int,                   // number of brushes to collide with in the patch
    mPatchBrushData: *mut cbrush_s,       // List of brushes that make up the patch
    mSurfaceFlags: c_int,                 // surfaceflag of the heightshader
    mContentFlags: c_int,                 // contents of the heightshader
}

impl CCMPatch {
    // CCMPatch(void) {}
    pub unsafe fn new() -> Self {
        core::mem::zeroed()
    }

    // Accessors
    #[inline]
    pub fn GetWorld(&self) -> &vec3_t { &self.mWorldCoords }
    #[inline]
    pub fn GetMins(&self) -> &vec3_t { &self.mBounds[0] }
    #[inline]
    pub fn GetMaxs(&self) -> &vec3_t { &self.mBounds[1] }
    #[inline]
    pub fn GetBounds(&self) -> &vec3pair_t { &self.mBounds }
    #[inline]
    pub fn GetHeightMapX(&self) -> c_int { self.mHx }
    #[inline]
    pub fn GetHeightMapY(&self) -> c_int { self.mHy }
    #[inline]
    pub fn GetHeight(&self, corner: c_int) -> c_int { self.mCornerHeights[corner as usize] as c_int }
    #[inline]
    pub fn GetNumBrushes(&self) -> c_int { self.mNumBrushes }
    #[inline]
    pub fn GetCollisionData(&self) -> *mut cbrush_s { self.mPatchBrushData }

    #[inline]
    pub fn SetSurfaceFlags(&mut self, in_: c_int) { self.mSurfaceFlags = in_; }
    #[inline]
    pub fn GetSurfaceFlags(&self) -> c_int { self.mSurfaceFlags }
    #[inline]
    pub fn SetContents(&mut self, in_: c_int) { self.mContentFlags = in_; }
    #[inline]
    pub fn GetContents(&self) -> c_int { self.mContentFlags }

    // Prototypes
    pub unsafe fn Init(
        &mut self,
        ls: *mut CCMLandScape,
        heightX: c_int,
        heightY: c_int,
        world: vec3_t,
        hMap: *mut byte,
        patchBrushData: *mut byte,
    ) {
        unimplemented!()
    }

    pub unsafe fn InitPlane(
        &mut self,
        side: *mut cbrushside_s,
        plane: *mut cplane_t,
        p0: vec3_t,
        p1: vec3_t,
        p2: vec3_t,
    ) {
        unimplemented!()
    }

    pub unsafe fn CreatePatchPlaneData(&mut self) {
        unimplemented!()
    }

    pub unsafe fn GetAdjacentBrushX(&mut self, x: c_int, y: c_int) -> *mut c_void {
        unimplemented!()
    }

    pub unsafe fn GetAdjacentBrushY(&mut self, x: c_int, y: c_int) -> *mut c_void {
        unimplemented!()
    }
}

impl Drop for CCMPatch {
    // ~CCMPatch(void) — non-trivial destructor; body in cm_landscape.cpp
    fn drop(&mut self) {
        unimplemented!()
    }
}

// Porting note: CCMLandScape contains std::collections::LinkedList (C++ std::list) fields;
// cannot be #[repr(C)] because LinkedList is not a C-layout type.
pub struct CCMLandScape {
    // private:
    mRefCount: c_int,                           // Number of times this class is referenced
    mTerrainHandle: thandle_t,
    mHeightMap: *mut byte,                      // Pointer to byte array of height samples
    mFlattenMap: *mut byte,                     // Pointer to byte array of flatten samples
    mWidth: c_int,
    mHeight: c_int,                             // Width and height of heightMap excluding the 1 pixel edge
    mTerxels: c_int,                            // Number of terxels per patch side
    mTerxelSize: vec3_t,                        // Vector to scale heightMap samples to real world coords
    mBounds: vec3pair_t,                        // Real world bounds of terrain brush
    mSize: vec3_t,                              // Size of terrain brush in real world coords excluding 1 patch edge
    mPatchSize: vec3_t,                         // Size of each patch in the x and y directions only
    mPatchScalarSize: f32,                      // Horizontal size of the patch
    mBlockWidth: c_int,
    mBlockHeight: c_int,                        // Width and height of heightfield on blocks
    mPatches: *mut CCMPatch,
    mPatchBrushData: *mut byte,                 // Base memory from which the patch brush data is taken
    mHasPhysics: bool,                          // Set to true unless disabled
    mRandomTerrain: *mut CRandomTerrain,

    mBaseWaterHeight: c_int,                    // Base water height in terxels
    mWaterHeight: f32,                          // Real world height of the water
    mWaterContents: c_int,                      // Contents of the water shader
    mWaterSurfaceFlags: c_int,                  // Surface flags of the water shader

    holdrand: c_ulong,

    mAreas: LinkedList<*mut CArea>,             // List of flattened areas on this landscape
    // Porting note: original is list<CArea*>::iterator; see areaIter_t note above.
    mAreasIt: areaIter_t,

    mHeightDetails: [CCMHeightDetails; HEIGHT_RESOLUTION], // Surfaceflags per height
    mCoords: *mut vec3_t,                       // Temp storage for real world coords
}

impl CCMLandScape {
    // CCMLandScape(const char *configstring, bool server)
    pub unsafe fn new(configstring: *const c_char, server: bool) -> Self {
        unimplemented!()
    }

    pub unsafe fn GetPatch(&mut self, x: c_int, y: c_int) -> *mut CCMPatch {
        unimplemented!()
    }

    // Prototypes
    pub unsafe fn PatchCollide(
        &mut self,
        tw: *mut traceWork_s,
        trace: &mut trace_t,
        start: vec3_t,
        end: vec3_t,
        checkcount: c_int,
    ) {
        unimplemented!()
    }

    pub unsafe fn TerrainPatchIterate(
        &self,
        IterateFunc: Option<unsafe extern "C" fn(*mut CCMPatch, *mut c_void)>,
        userdata: *mut c_void,
    ) {
        unimplemented!()
    }

    pub unsafe fn GetWorldHeight(
        &self,
        origin: vec3_t,
        bounds: vec3pair_t,
        aboveGround: bool,
    ) -> f32 {
        unimplemented!()
    }

    pub unsafe fn WaterCollide(
        &self,
        begin: vec3_t,
        end: vec3_t,
        fraction: f32,
    ) -> f32 {
        unimplemented!()
    }

    pub unsafe fn UpdatePatches(&mut self) {
        unimplemented!()
    }

    pub unsafe fn GetTerxelLocalCoords(&mut self, x: c_int, y: c_int, coords: *mut vec3_t) {
        unimplemented!()
    }

    pub unsafe fn LoadTerrainDef(&mut self, td: *const c_char) {
        unimplemented!()
    }

    pub unsafe fn SetShaders(&mut self, height: c_int, shader: *mut CCMShader) {
        unimplemented!()
    }

    pub unsafe fn FlattenArea(
        &mut self,
        area: *mut CArea,
        height: c_int,
        save: bool,
        forceHeight: bool,
        smooth: bool,
    ) {
        unimplemented!()
    }

    pub unsafe fn CarveLine(
        &mut self,
        start: vec3_t,
        end: vec3_t,
        depth: c_int,
        width: c_int,
    ) {
        unimplemented!()
    }

    pub unsafe fn CarveBezierCurve(
        &mut self,
        numCtlPoints: c_int,
        ctlPoints: *mut vec3_t,
        steps: c_int,
        depth: c_int,
        size: c_int,
    ) {
        unimplemented!()
    }

    pub unsafe fn SaveArea(&mut self, area: *mut CArea) {
        unimplemented!()
    }

    pub unsafe fn FractionBelowLevel(&mut self, area: *mut CArea, height: c_int) -> f32 {
        unimplemented!()
    }

    pub unsafe fn AreaCollision(
        &mut self,
        area: *mut CArea,
        areaTypes: *mut c_int,
        areaTypeCount: c_int,
    ) -> bool {
        unimplemented!()
    }

    pub unsafe fn GetFirstArea(&mut self) -> *mut CArea {
        unimplemented!()
    }

    pub unsafe fn GetFirstObjectiveArea(&mut self) -> *mut CArea {
        unimplemented!()
    }

    pub unsafe fn GetPlayerArea(&mut self) -> *mut CArea {
        unimplemented!()
    }

    pub unsafe fn GetNextArea(&mut self) -> *mut CArea {
        unimplemented!()
    }

    pub unsafe fn GetNextObjectiveArea(&mut self) -> *mut CArea {
        unimplemented!()
    }

    // Accessors
    #[inline]
    pub fn GetRefCount(&self) -> c_int { self.mRefCount }
    #[inline]
    pub fn IncreaseRefCount(&mut self) { self.mRefCount += 1; }
    #[inline]
    pub fn DecreaseRefCount(&mut self) { self.mRefCount -= 1; }
    #[inline]
    pub fn GetBounds(&self) -> &vec3pair_t { &self.mBounds }
    #[inline]
    pub fn GetMins(&self) -> &vec3_t { &self.mBounds[0] }
    #[inline]
    pub fn GetMaxs(&self) -> &vec3_t { &self.mBounds[1] }
    #[inline]
    pub fn GetSize(&self) -> &vec3_t { &self.mSize }
    #[inline]
    pub fn GetTerxelSize(&self) -> &vec3_t { &self.mTerxelSize }
    #[inline]
    pub fn GetPatchSize(&self) -> &vec3_t { &self.mPatchSize }
    #[inline]
    pub fn GetPatchWidth(&self) -> f32 { self.mPatchSize[0] }
    #[inline]
    pub fn GetPatchHeight(&self) -> f32 { self.mPatchSize[1] }
    #[inline]
    pub fn GetPatchScalarSize(&self) -> f32 { self.mPatchScalarSize }
    #[inline]
    pub fn GetTerxels(&self) -> c_int { self.mTerxels }
    #[inline]
    pub fn GetRealWidth(&self) -> c_int { self.mWidth + 1 }
    #[inline]
    pub fn GetRealHeight(&self) -> c_int { self.mHeight + 1 }
    #[inline]
    pub fn GetRealArea(&self) -> c_int { (self.mWidth + 1) * (self.mHeight + 1) }
    #[inline]
    pub fn GetWidth(&self) -> c_int { self.mWidth }
    #[inline]
    pub fn GetHeight(&self) -> c_int { self.mHeight }
    #[inline]
    pub fn GetArea(&self) -> c_int { self.mWidth * self.mHeight }
    #[inline]
    pub fn GetBlockWidth(&self) -> c_int { self.mBlockWidth }
    #[inline]
    pub fn GetBlockHeight(&self) -> c_int { self.mBlockHeight }
    #[inline]
    pub fn GetBlockCount(&self) -> c_int { self.mBlockWidth * self.mBlockHeight }
    #[inline]
    pub fn GetHeightMap(&self) -> *mut byte { self.mHeightMap }
    #[inline]
    pub fn GetFlattenMap(&self) -> *mut byte { self.mFlattenMap }
    #[inline]
    pub fn GetTerrainId(&self) -> thandle_t { self.mTerrainHandle }
    #[inline]
    pub fn SetTerrainId(&mut self, terrainId: thandle_t) { self.mTerrainHandle = terrainId; }
    #[inline]
    pub fn CalcWorldHeight(&self, height: c_int) -> f32 {
        (height as f32 * self.mTerxelSize[2]) + self.mBounds[0][2]
    }
    #[inline]
    pub fn GetHasPhysics(&self) -> bool { self.mHasPhysics }
    #[inline]
    pub fn GetIsRandom(&self) -> bool { !self.mRandomTerrain.is_null() }
    #[inline]
    pub fn GetSurfaceFlags(&self, height: c_int) -> c_int {
        self.mHeightDetails[height as usize].GetSurfaceFlags()
    }
    #[inline]
    pub fn GetContentFlags(&self, height: c_int) -> c_int {
        self.mHeightDetails[height as usize].GetContents()
    }
    pub unsafe fn CalcRealCoords(&mut self) {
        unimplemented!()
    }
    #[inline]
    pub fn GetCoords(&self) -> *mut vec3_t { self.mCoords }

    #[inline]
    pub fn GetBaseWaterHeight(&self) -> c_int { self.mBaseWaterHeight }
    #[inline]
    pub fn SetRealWaterHeight(&mut self, height: c_int) {
        self.mWaterHeight = height as f32 * self.mTerxelSize[2];
    }
    #[inline]
    pub fn GetWaterHeight(&self) -> f32 { self.mWaterHeight }
    #[inline]
    pub fn GetWaterContents(&self) -> c_int { self.mWaterContents }
    #[inline]
    pub fn GetWaterSurfaceFlags(&self) -> c_int { self.mWaterSurfaceFlags }

    #[inline]
    pub fn GetRandomTerrain(&mut self) -> *mut CRandomTerrain { self.mRandomTerrain }

    pub unsafe fn rand_seed(&mut self, seed: c_int) {
        unimplemented!()
    }
    #[inline]
    pub fn get_rand_seed(&self) -> c_ulong { self.holdrand }

    pub unsafe fn flrand(&mut self, min: f32, max: f32) -> f32 {
        unimplemented!()
    }
    pub unsafe fn irand(&mut self, min: c_int, max: c_int) -> c_int {
        unimplemented!()
    }
}

impl Drop for CCMLandScape {
    // ~CCMLandScape(void) — non-trivial destructor; body in cm_landscape.cpp
    fn drop(&mut self) {
        unimplemented!()
    }
}

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
        origin: vec3_t,
        bounds: vec3pair_t,
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
