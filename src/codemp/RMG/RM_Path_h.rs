/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Path.h
 *
 ************************************************************************************************/

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use core::ffi::{c_char, c_int};

use crate::codemp::qcommon::cm_randomterrain_h::*;

// class CRMPathManager; (forward declaration — not needed in Rust)

// directions you can proceed from cells
pub type ERMDir = c_int;
pub const DIR_FIRST: ERMDir = 0;
pub const DIR_N: ERMDir     = 0;
pub const DIR_NE: ERMDir    = 1;
pub const DIR_E: ERMDir     = 2;
pub const DIR_SE: ERMDir    = 3;
pub const DIR_S: ERMDir     = 4;
pub const DIR_SW: ERMDir    = 5;
pub const DIR_W: ERMDir     = 6;
pub const DIR_NW: ERMDir    = 7;
pub const DIR_MAX: ERMDir   = 8;
pub const DIR_ALL: ERMDir   = 255;

pub const HALF_DIR_MAX: ERMDir = DIR_MAX / 2; // #define HALF_DIR_MAX (DIR_MAX/2)

pub struct CRMNode {
    pub mName: String,                  // name of node - "" if not used yet
    pub mPos: vec3_t,                   // where node is
    pub mPathID: [c_int; 8],            // path id's that lead from this node [DIR_MAX]
    pub mAreaPointPlaced: bool,         // false if no area point here yet.

    pub mFlattenHeight: c_int,
}

// CRMNode() constructor declared here; defined in RM_Path.cpp
impl CRMNode {
    pub fn IsLocation(&self) -> bool {
        // strlen(mName.c_str())>0
        self.mName.len() > 0
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName.as_ptr() as *const c_char
    }

    pub fn GetPos(&mut self) -> &mut vec3_t {
        &mut self.mPos
    }

    pub fn PathExist(&self, dir: c_int) -> f32 {
        // return (mPathID[dir % DIR_MAX] != -1)  — bool implicitly converted to float
        (self.mPathID[(dir % DIR_MAX) as usize] != -1) as c_int as f32
    }

    pub fn GetPath(&self, dir: c_int) -> f32 {
        self.mPathID[(dir % DIR_MAX) as usize] as f32
    }

    pub fn AreaPoint(&self) -> bool {
        self.mAreaPointPlaced
    }

    pub unsafe fn SetName(&mut self, name: *const c_char) {
        self.mName = core::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
    }

    pub fn SetPos(&mut self, v: &vec3_t) {
        // VectorCopy ( v, mPos )
        VectorCopy(v, &mut self.mPos);
    }

    pub fn SetPath(&mut self, dir: c_int, id: c_int) {
        self.mPathID[(dir % DIR_MAX) as usize] = id;
    }

    pub fn SetAreaPoint(&mut self, ap: bool) {
        self.mAreaPointPlaced = ap;
    }

    pub fn SetFlattenHeight(&mut self, flattenHeight: c_int) {
        self.mFlattenHeight = flattenHeight;
    }

    pub fn GetFlattenHeight(&self) -> c_int {
        self.mFlattenHeight
    }
}

pub type rmNodeVector_t = Vec<*mut CRMNode>;

// named spots on the map, should be placed into nodes
pub struct CRMLoc {
    pub mName: String,          // name of location
    pub mMinDepth: c_int,
    pub mMaxDepth: c_int,
    pub mMinPaths: c_int,
    pub mMaxPaths: c_int,
    pub mPlaced: bool,          // location has been placed at a node
}

impl CRMLoc {
    // CRMLoc (const char *name, const int min_depth, const int max_depth,
    //         const int min_paths =1, const int max_paths=1 )
    //     : mMinDepth(min_depth), mMaxDepth(max_depth), mPlaced(false),
    //       mMinPaths(min_paths), mMaxPaths(max_paths)
    // { mName = name; };
    pub unsafe fn new(
        name: *const c_char,
        min_depth: c_int,
        max_depth: c_int,
        min_paths: c_int,
        max_paths: c_int,
    ) -> Self {
        let mut loc = CRMLoc {
            mName: String::new(),
            mMinDepth: min_depth,
            mMaxDepth: max_depth,
            mMinPaths: min_paths,
            mMaxPaths: max_paths,
            mPlaced: false,
        };
        loc.mName = core::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
        loc
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName.as_ptr() as *const c_char
    }

    pub unsafe fn SetName(&mut self, name: *const c_char) {
        self.mName = core::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
    }

    pub fn MinDepth(&self) -> c_int {
        self.mMinDepth
    }

    pub fn SetMinDepth(&mut self, deep: c_int) {
        self.mMinDepth = deep;
    }

    pub fn MaxDepth(&self) -> c_int {
        self.mMaxDepth
    }

    pub fn SetMaxDepth(&mut self, deep: c_int) {
        self.mMaxDepth = deep;
    }

    pub fn MinPaths(&self) -> c_int {
        self.mMinPaths
    }

    pub fn SetMinPaths(&mut self, paths: c_int) {
        self.mMinPaths = paths;
    }

    pub fn MaxPaths(&self) -> c_int {
        self.mMaxPaths
    }

    pub fn SetMaxPaths(&mut self, paths: c_int) {
        self.mMaxPaths = paths;
    }

    pub fn Placed(&self) -> bool {
        self.mPlaced
    }

    pub fn SetPlaced(&mut self, p: bool) {
        self.mPlaced = p;
    }
}

pub type rmLocVector_t = Vec<*mut CRMLoc>;


// cells are used for figuring out node connections / paths
#[repr(C)]
pub struct CRMCell {
    pub border: c_int,
    pub wall: c_int,
}

impl CRMCell {
    // CRMCell() { border = 0; wall = DIR_ALL; };
    pub fn new() -> Self {
        CRMCell { border: 0, wall: DIR_ALL }
    }

    pub fn Border(&self) -> c_int {
        self.border
    }

    pub fn Wall(&self) -> c_int {
        self.wall
    }

    // NOTE: Rust does not support method overloading; C++ Border(const int dir) and
    // Wall(const int dir) are renamed to Border_dir / Wall_dir to avoid collision
    // with the no-arg Border() and Wall() above.
    pub fn Border_dir(&self, dir: c_int) -> bool {
        (self.border & (1 << dir)) != 0
    }

    pub fn Wall_dir(&self, dir: c_int) -> bool {
        (self.wall & (1 << dir)) != 0
    }

    pub fn SetBorder(&mut self, dir: c_int) {
        self.border |= 1 << dir;
    }

    pub fn SetWall(&mut self, dir: c_int) {
        self.wall |= 1 << dir;
    }

    pub fn RemoveWall(&mut self, dir: c_int) {
        // wall &= ~(1<<dir)
        self.wall &= !(1 << dir);
    }
}

pub type rmCellVector_t = Vec<CRMCell>;


pub struct CRMPathManager {
    pub mXNodes: c_int,         // number of nodes in the x dimension
    pub mYNodes: c_int,         // number of nodes in the y dimension

    pub mLocations: rmLocVector_t,  // location, named spots to be placed at nodes
    pub mNodes: rmNodeVector_t,     // nodes, spots on map that *may* be connected by paths
    pub mCells: rmCellVector_t,     // array of cells for doing path generation

    pub mPathCount: c_int,
    pub mRiverCount: c_int,
    pub mMaxDepth: c_int,       // deepest any location wants to be
    pub mDepth: c_int,          // current depth

    pub mCrossed: bool,         // used to indicate if paths crossed the imaginary diagonal that cuts symmetric maps in half

    // path style
    pub mPathPoints: c_int,
    pub mPathMinWidth: f32,
    pub mPathMaxWidth: f32,
    pub mPathDepth: f32,
    pub mPathDeviation: f32,
    pub mPathBreadth: f32,

    // river style
    pub mRiverDepth: c_int,
    pub mRiverPoints: c_int,
    pub mRiverMinWidth: f32,
    pub mRiverMaxWidth: f32,
    pub mRiverBedDepth: f32,
    pub mRiverDeviation: f32,
    pub mRiverBreadth: f32,
    pub mRiverBridge: String,
    pub mRiverPos: vec3_t,

    pub mTerrain: *mut CRandomTerrain,
}

// static int neighbor_x[DIR_MAX]; — static member of CRMPathManager
pub static mut neighbor_x: [c_int; 8] = [0; 8]; // DIR_MAX = 8
// static int neighbor_y[DIR_MAX]; — static member of CRMPathManager
pub static mut neighbor_y: [c_int; 8] = [0; 8]; // DIR_MAX = 8

// CRMPathManager(terrain) constructor and ~CRMPathManager() destructor are declared
// here but defined in RM_Path.cpp; the remaining non-inline methods are also there.
impl CRMPathManager {
    pub fn Node(&self, x: c_int, y: c_int) -> *mut CRMNode {
        // return mNodes[x + y*mXNodes]
        self.mNodes[(x + y * self.mXNodes) as usize]
    }

    pub unsafe fn GetNodePos(&mut self, x: c_int, y: c_int) -> &mut vec3_t {
        // return mNodes[x + y*mXNodes]->GetPos()
        (*self.mNodes[(x + y * self.mXNodes) as usize]).GetPos()
    }

    pub unsafe fn SetNodePos(&mut self, x: c_int, y: c_int, pos: &vec3_t) {
        // mNodes[x + y*mXNodes]->SetPos(pos)
        (*self.mNodes[(x + y * self.mXNodes) as usize]).SetPos(pos)
    }

    pub fn GetPathCount(&self) -> c_int {
        self.mPathCount
    }

    pub fn GetRiverCount(&self) -> c_int {
        self.mRiverCount
    }

    pub fn GetRiverDepth(&self) -> f32 {
        self.mRiverBedDepth
    }

    pub fn GetPathDepth(&self) -> f32 {
        self.mPathDepth
    }

    pub fn GetBridgeName(&self) -> *const c_char {
        // return mRiverBridge.c_str()
        self.mRiverBridge.as_ptr() as *const c_char
    }

    pub fn Cell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        // return mCells[x + y*mXNodes]
        &mut self.mCells[(x + y * self.mXNodes) as usize]
    }

    pub fn RiverCell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        // return mCells[x + y*(mXNodes+1)]
        &mut self.mCells[(x + y * (self.mXNodes + 1)) as usize]
    }
}
