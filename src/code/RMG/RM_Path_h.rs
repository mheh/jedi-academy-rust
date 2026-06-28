/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Path.h
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_int;

// Forward declaration for types referenced but not fully defined in this header
#[repr(C)]
pub struct CRandomTerrain;

// Type aliases for C compatibility
pub type vec3_t = [f32; 3];

// String type wrapper (approximating C++ std::string for ABI stability)
#[repr(C)]
pub struct string {
    _data: *mut core::ffi::c_char,
    _capacity: usize,
    _size: usize,
}

impl string {
    pub fn c_str(&self) -> *const core::ffi::c_char {
        self._data
    }
}

// directions you can proceed from cells
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum ERMDir {
    DIR_FIRST = 0,
    DIR_N = 0,
    DIR_NE = 1,
    DIR_E = 2,
    DIR_SE = 3,
    DIR_S = 4,
    DIR_SW = 5,
    DIR_W = 6,
    DIR_NW = 7,
    DIR_MAX = 8,
    DIR_ALL = 255,
}

pub const HALF_DIR_MAX: c_int = 8 / 2;

#[repr(C)]
pub struct CRMNode {
    // name of node - "" if not used yet
    mName: string,
    // where node is
    mPos: vec3_t,
    // path id's that lead from this node
    mPathID: [c_int; 8],
    // false if no area point here yet.
    mAreaPointPlaced: bool,

    mFlattenHeight: c_int,
}

impl CRMNode {
    pub fn new() -> Self {
        CRMNode {
            mName: string {
                _data: core::ptr::null_mut(),
                _capacity: 0,
                _size: 0,
            },
            mPos: [0.0, 0.0, 0.0],
            mPathID: [-1; 8],
            mAreaPointPlaced: false,
            mFlattenHeight: 0,
        }
    }

    pub fn IsLocation(&self) -> bool {
        let len = unsafe {
            let mut len = 0usize;
            let mut ptr = self.mName.c_str();
            while !ptr.is_null() && *ptr != 0 {
                len += 1;
                ptr = ptr.add(1);
            }
            len
        };
        len > 0
    }

    pub fn GetName(&self) -> *const core::ffi::c_char {
        self.mName.c_str()
    }

    pub fn GetPos(&mut self) -> &mut vec3_t {
        &mut self.mPos
    }

    pub fn PathExist(&self, dir: c_int) -> c_int {
        let idx = (dir % 8) as usize;
        if idx < self.mPathID.len() {
            if self.mPathID[idx] != -1 {
                1
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn GetPath(&self, dir: c_int) -> c_int {
        let idx = (dir % 8) as usize;
        if idx < self.mPathID.len() {
            self.mPathID[idx]
        } else {
            -1
        }
    }

    pub fn AreaPoint(&self) -> bool {
        self.mAreaPointPlaced
    }

    pub fn SetName(&mut self, name: *const core::ffi::c_char) {
        // Stub: string assignment not included in header
    }

    pub fn SetPos(&mut self, v: &vec3_t) {
        // VectorCopy equivalent
        self.mPos[0] = v[0];
        self.mPos[1] = v[1];
        self.mPos[2] = v[2];
    }

    pub fn SetPath(&mut self, dir: c_int, id: c_int) {
        let idx = (dir % 8) as usize;
        if idx < self.mPathID.len() {
            self.mPathID[idx] = id;
        }
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
#[repr(C)]
pub struct CRMLoc {
    // name of location
    mName: string,
    mMinDepth: c_int,
    mMaxDepth: c_int,
    mMinPaths: c_int,
    mMaxPaths: c_int,
    // location has been placed at a node
    mPlaced: bool,
}

impl CRMLoc {
    pub fn new(
        name: *const core::ffi::c_char,
        min_depth: c_int,
        max_depth: c_int,
        min_paths: c_int,
        max_paths: c_int,
    ) -> Self {
        CRMLoc {
            mName: string {
                _data: name as *mut core::ffi::c_char,
                _capacity: 0,
                _size: 0,
            },
            mMinDepth: min_depth,
            mMaxDepth: max_depth,
            mPlaced: false,
            mMinPaths: min_paths,
            mMaxPaths: max_paths,
        }
    }

    pub fn GetName(&self) -> *const core::ffi::c_char {
        self.mName.c_str()
    }

    pub fn SetName(&mut self, name: *const core::ffi::c_char) {
        // Stub: string assignment not included in header
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
    border: c_int,
    wall: c_int,
}

impl CRMCell {
    pub fn new() -> Self {
        CRMCell {
            border: 0,
            wall: 255, // DIR_ALL
        }
    }

    pub fn Border_get(&self) -> c_int {
        self.border
    }

    pub fn Wall_get(&self) -> c_int {
        self.wall
    }

    pub fn Border_has(&self, dir: c_int) -> bool {
        (self.border & (1 << dir)) != 0
    }

    pub fn Wall_has(&self, dir: c_int) -> bool {
        (self.wall & (1 << dir)) != 0
    }

    pub fn SetBorder(&mut self, dir: c_int) {
        self.border |= 1 << dir;
    }

    pub fn SetWall(&mut self, dir: c_int) {
        self.wall |= 1 << dir;
    }

    pub fn RemoveWall(&mut self, dir: c_int) {
        self.wall &= !(1 << dir);
    }
}

pub type rmCellVector_t = Vec<CRMCell>;

#[repr(C)]
pub struct CRMPathManager {
    pub mXNodes: c_int,     // number of nodes in the x dimension
    pub mYNodes: c_int,     // number of nodes in the y dimension

    // private members follow
    mLocations: rmLocVector_t,  // location, named spots to be placed at nodes
    mNodes: rmNodeVector_t,     // nodes, spots on map that *may* be connected by paths
    mCells: rmCellVector_t,     // array of cells for doing path generation

    mPathCount: c_int,
    mRiverCount: c_int,
    mMaxDepth: c_int,           // deepest any location wants to be
    mDepth: c_int,              // current depth

    mCrossed: bool,             // used to indicate if paths crossed the imaginary diagonal that cuts symmetric maps in half

    // path style
    mPathPoints: c_int,
    mPathMinWidth: f32,
    mPathMaxWidth: f32,
    mPathDepth: f32,
    mPathDeviation: f32,
    mPathBreadth: f32,

    // river style
    mRiverDepth: c_int,
    mRiverPoints: c_int,
    mRiverMinWidth: f32,
    mRiverMaxWidth: f32,
    mRiverBedDepth: f32,
    mRiverDeviation: f32,
    mRiverBreadth: f32,
    mRiverBridge: string,
    mRiverPos: vec3_t,

    // static members: neighbor_x, neighbor_y
    mTerrain: *mut CRandomTerrain,
}

impl CRMPathManager {
    pub fn new(terrain: *mut CRandomTerrain) -> Self {
        CRMPathManager {
            mXNodes: 0,
            mYNodes: 0,
            mLocations: Vec::new(),
            mNodes: Vec::new(),
            mCells: Vec::new(),
            mPathCount: 0,
            mRiverCount: 0,
            mMaxDepth: 0,
            mDepth: 0,
            mCrossed: false,
            mPathPoints: 0,
            mPathMinWidth: 0.0,
            mPathMaxWidth: 0.0,
            mPathDepth: 0.0,
            mPathDeviation: 0.0,
            mPathBreadth: 0.0,
            mRiverDepth: 0,
            mRiverPoints: 0,
            mRiverMinWidth: 0.0,
            mRiverMaxWidth: 0.0,
            mRiverBedDepth: 0.0,
            mRiverDeviation: 0.0,
            mRiverBreadth: 0.0,
            mRiverBridge: string {
                _data: core::ptr::null_mut(),
                _capacity: 0,
                _size: 0,
            },
            mRiverPos: [0.0, 0.0, 0.0],
            mTerrain: terrain,
        }
    }

    pub fn drop(&mut self) {
        // Stub: destructor implementation not included in header
    }

    pub fn ClearCells(&mut self, x_nodes: c_int, y_nodes: c_int) {
        // Stub: implementation not included in header
    }

    pub fn CreateArray(&mut self, x_nodes: c_int, y_nodes: c_int) -> bool {
        // Stub: implementation not included in header
        false
    }

    pub fn FindNodeByName(&mut self, name: *const core::ffi::c_char) -> *mut CRMNode {
        // Stub: implementation not included in header
        core::ptr::null_mut()
    }

    pub fn Node(&self, x: c_int, y: c_int) -> *mut CRMNode {
        let idx = (x + y * self.mXNodes) as usize;
        if idx < self.mNodes.len() {
            self.mNodes[idx]
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn CreateLocation(
        &mut self,
        name: *const core::ffi::c_char,
        min_depth: c_int,
        max_depth: c_int,
        min_paths: c_int,
        max_paths: c_int,
    ) {
        // Stub: implementation not included in header
    }

    pub fn GetNodePos(&self, x: c_int, y: c_int) -> *mut vec3_t {
        let idx = (x + y * self.mXNodes) as usize;
        if idx < self.mNodes.len() {
            let node = self.mNodes[idx];
            if !node.is_null() {
                unsafe { &mut (*node).mPos as *mut vec3_t }
            } else {
                core::ptr::null_mut()
            }
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn SetNodePos(&mut self, x: c_int, y: c_int, pos: &vec3_t) {
        let idx = (x + y * self.mXNodes) as usize;
        if idx < self.mNodes.len() {
            let node = self.mNodes[idx];
            if !node.is_null() {
                unsafe {
                    (*node).SetPos(pos);
                }
            }
        }
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

    pub fn GetBridgeName(&self) -> *const core::ffi::c_char {
        self.mRiverBridge.c_str()
    }

    pub fn GetRiverPos(&self, x: c_int, y: c_int) -> *mut vec3_t {
        let idx = (x + y * (self.mXNodes + 1)) as usize;
        if idx < self.mCells.len() {
            // Note: This is a pointer into the mCells array, which contains CRMCell structs.
            // The original returns a reference to the river position data.
            // Stub: actual implementation not fully defined in header
            core::ptr::null_mut()
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn Cell(&mut self, x: c_int, y: c_int) -> *mut CRMCell {
        let idx = (x + y * self.mXNodes) as usize;
        if idx < self.mCells.len() {
            &mut self.mCells[idx] as *mut CRMCell
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn RiverCell(&mut self, x: c_int, y: c_int) -> *mut CRMCell {
        let idx = (x + y * (self.mXNodes + 1)) as usize;
        if idx < self.mCells.len() {
            &mut self.mCells[idx] as *mut CRMCell
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn PlaceLocation(&mut self, x: c_int, y: c_int) {
        // Stub: implementation not included in header
    }

    pub fn PathVisit(&mut self, x: c_int, y: c_int) {
        // Stub: implementation not included in header
    }

    pub fn RiverVisit(&mut self, x: c_int, y: c_int) {
        // Stub: implementation not included in header
    }

    pub fn SetPathStyle(
        &mut self,
        points: c_int,
        minwidth: f32,
        maxwidth: f32,
        depth: f32,
        deviation: f32,
        breadth: f32,
    ) {
        // Stub: implementation not included in header
    }

    pub fn SetRiverStyle(
        &mut self,
        depth: c_int,
        points: c_int,
        minwidth: f32,
        maxwidth: f32,
        beddepth: f32,
        deviation: f32,
        breadth: f32,
        bridge_name: *const core::ffi::c_char,
    ) {
        // Stub: implementation not included in header
    }

    pub fn GeneratePaths(&mut self, symmetric: c_int) {
        // Stub: implementation not included in header
    }

    pub fn GenerateRivers(&mut self) {
        // Stub: implementation not included in header
    }
}

// static members of CRMPathManager
pub static neighbor_x: [c_int; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
pub static neighbor_y: [c_int; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];
