/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Path.h
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

// Stubs for dependencies from qcommon/cm_randomterrain.h
// Full definitions are in cm_randomterrain module
pub type vec3_t = [f32; 3];

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum symmetry_t {
    SYMMETRY_NONE = 0,
}

pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

fn VectorCopy(src: &vec3_t, dst: &mut vec3_t) {
    *dst = *src;
}

// directions you can proceed from cells
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub const HALF_DIR_MAX: usize = 4; // DIR_MAX / 2

pub struct CRMNode {
    mName: String,                   // name of node - "" if not used yet
    mPos: vec3_t,                    // where node is
    mPathID: [i32; 8],               // path id's that lead from this node
    mAreaPointPlaced: bool,          // false if no area point here yet.

    mFlattenHeight: i32,
}

impl CRMNode {
    pub fn new() -> Self {
        CRMNode {
            mName: String::new(),
            mPos: [0.0; 3],
            mPathID: [-1; 8],
            mAreaPointPlaced: false,
            mFlattenHeight: 0,
        }
    }

    pub fn IsLocation(&self) -> bool {
        self.mName.len() > 0
    }

    pub fn GetName(&self) -> &str {
        &self.mName
    }

    pub fn GetPos(&mut self) -> &mut vec3_t {
        &mut self.mPos
    }

    pub fn PathExist(&self, dir: i32) -> f32 {
        if self.mPathID[(dir % 8) as usize] != -1 {
            1.0
        } else {
            0.0
        }
    }

    pub fn GetPath(&self, dir: i32) -> f32 {
        self.mPathID[(dir % 8) as usize] as f32
    }

    pub fn AreaPoint(&self) -> bool {
        self.mAreaPointPlaced
    }

    pub fn SetName(&mut self, name: &str) {
        self.mName = name.to_string();
    }

    pub fn SetPos(&mut self, v: &vec3_t) {
        VectorCopy(v, &mut self.mPos);
    }

    pub fn SetPath(&mut self, dir: i32, id: i32) {
        self.mPathID[(dir % 8) as usize] = id;
    }

    pub fn SetAreaPoint(&mut self, ap: bool) {
        self.mAreaPointPlaced = ap;
    }

    pub fn SetFlattenHeight(&mut self, flattenHeight: i32) {
        self.mFlattenHeight = flattenHeight;
    }

    pub fn GetFlattenHeight(&self) -> i32 {
        self.mFlattenHeight
    }
}

pub type rmNodeVector_t = Vec<*mut CRMNode>;

// named spots on the map, should be placed into nodes
pub struct CRMLoc {
    mName: String,       // name of location
    mMinDepth: i32,
    mMaxDepth: i32,
    mMinPaths: i32,
    mMaxPaths: i32,
    mPlaced: bool,       // location has been placed at a node
}

impl CRMLoc {
    pub fn new(name: &str, min_depth: i32, max_depth: i32, min_paths: i32, max_paths: i32) -> Self {
        CRMLoc {
            mName: name.to_string(),
            mMinDepth: min_depth,
            mMaxDepth: max_depth,
            mMinPaths: min_paths,
            mMaxPaths: max_paths,
            mPlaced: false,
        }
    }

    pub fn GetName(&self) -> &str {
        &self.mName
    }

    pub fn SetName(&mut self, name: &str) {
        self.mName = name.to_string();
    }

    pub fn MinDepth(&self) -> i32 {
        self.mMinDepth
    }

    pub fn SetMinDepth(&mut self, deep: i32) {
        self.mMinDepth = deep;
    }

    pub fn MaxDepth(&self) -> i32 {
        self.mMaxDepth
    }

    pub fn SetMaxDepth(&mut self, deep: i32) {
        self.mMaxDepth = deep;
    }

    pub fn MinPaths(&self) -> i32 {
        self.mMinPaths
    }

    pub fn SetMinPaths(&mut self, paths: i32) {
        self.mMinPaths = paths;
    }

    pub fn MaxPaths(&self) -> i32 {
        self.mMaxPaths
    }

    pub fn SetMaxPaths(&mut self, paths: i32) {
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
    border: i32,
    wall: i32,
}

impl CRMCell {
    pub fn new() -> Self {
        CRMCell {
            border: 0,
            wall: 255, // DIR_ALL
        }
    }

    pub fn Border(&self) -> i32 {
        self.border
    }

    pub fn Wall(&self) -> i32 {
        self.wall
    }

    pub fn Border_dir(&self, dir: i32) -> bool {
        (self.border & (1 << dir)) != 0
    }

    pub fn Wall_dir(&self, dir: i32) -> bool {
        (self.wall & (1 << dir)) != 0
    }

    pub fn SetBorder(&mut self, dir: i32) {
        self.border |= 1 << dir;
    }

    pub fn SetWall(&mut self, dir: i32) {
        self.wall |= 1 << dir;
    }

    pub fn RemoveWall(&mut self, dir: i32) {
        self.wall &= !(1 << dir);
    }
}

pub type rmCellVector_t = Vec<CRMCell>;

pub struct CRMPathManager {
    pub mXNodes: i32,    // number of nodes in the x dimension
    pub mYNodes: i32,    // number of nodes in the y dimension

    mLocations: rmLocVector_t,  // location, named spots to be placed at nodes
    mNodes: rmNodeVector_t,     // nodes, spots on map that *may* be connected by paths
    mCells: rmCellVector_t,     // array of cells for doing path generation

    mPathCount: i32,
    mRiverCount: i32,
    mMaxDepth: i32,     // deepest any location wants to be
    mDepth: i32,        // current depth

    mCrossed: bool,     // used to indicate if paths crossed the imaginary diagonal that cuts symmetric maps in half

    // path style
    mPathPoints: i32,
    mPathMinWidth: f32,
    mPathMaxWidth: f32,
    mPathDepth: f32,
    mPathDeviation: f32,
    mPathBreadth: f32,

    // river style
    mRiverDepth: i32,
    mRiverPoints: i32,
    mRiverMinWidth: f32,
    mRiverMaxWidth: f32,
    mRiverBedDepth: f32,
    mRiverDeviation: f32,
    mRiverBreadth: f32,
    mRiverBridge: String,
    mRiverPos: vec3_t,

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
            mRiverBridge: String::new(),
            mRiverPos: [0.0; 3],
            mTerrain: terrain,
        }
    }

    pub fn ClearCells(&mut self, x_nodes: i32, y_nodes: i32) {
        // Implementation defined in RM_Path.cpp
    }

    pub fn CreateArray(&mut self, x_nodes: i32, y_nodes: i32) -> bool {
        // Implementation defined in RM_Path.cpp
        false
    }

    pub fn FindNodeByName(&self, name: &str) -> Option<*mut CRMNode> {
        // Implementation defined in RM_Path.cpp
        None
    }

    pub fn Node(&self, x: i32, y: i32) -> Option<*mut CRMNode> {
        let idx = (x + y * self.mXNodes) as usize;
        self.mNodes.get(idx).copied()
    }

    pub fn CreateLocation(&mut self, name: &str, min_depth: i32, max_depth: i32, min_paths: i32, max_paths: i32) {
        // Implementation defined in RM_Path.cpp
    }

    pub fn GetNodePos(&mut self, x: i32, y: i32) -> Option<&mut vec3_t> {
        let idx = (x + y * self.mXNodes) as usize;
        // Note: This returns a reference into mNodes, but with Vec<*mut T>, we cannot safely return a reference
        // Implementation defined in RM_Path.cpp
        None
    }

    pub fn SetNodePos(&mut self, x: i32, y: i32, pos: &vec3_t) {
        // Implementation defined in RM_Path.cpp
    }

    pub fn GetPathCount(&self) -> i32 {
        self.mPathCount
    }

    pub fn GetRiverCount(&self) -> i32 {
        self.mRiverCount
    }

    pub fn GetRiverDepth(&self) -> f32 {
        self.mRiverBedDepth
    }

    pub fn GetPathDepth(&self) -> f32 {
        self.mPathDepth
    }

    pub fn GetBridgeName(&self) -> &str {
        &self.mRiverBridge
    }

    pub fn GetRiverPos(&mut self, x: i32, y: i32) -> &mut vec3_t {
        // Implementation defined in RM_Path.cpp
        &mut self.mRiverPos
    }

    pub fn Cell(&mut self, x: i32, y: i32) -> Option<&mut CRMCell> {
        let idx = (x + y * self.mXNodes) as usize;
        self.mCells.get_mut(idx)
    }

    pub fn RiverCell(&mut self, x: i32, y: i32) -> Option<&mut CRMCell> {
        let idx = (x + y * (self.mXNodes + 1)) as usize;
        self.mCells.get_mut(idx)
    }

    pub fn PlaceLocation(&mut self, x: i32, y: i32) {
        // Implementation defined in RM_Path.cpp
    }

    pub fn PathVisit(&mut self, x: i32, y: i32) {
        // Implementation defined in RM_Path.cpp
    }

    pub fn RiverVisit(&mut self, x: i32, y: i32) {
        // Implementation defined in RM_Path.cpp
    }

    pub fn SetPathStyle(&mut self, points: i32, minwidth: f32, maxwidth: f32, depth: f32, deviation: f32, breadth: f32) {
        // C++ had default values: points=10, minwidth=0.01f, maxwidth=0.05f, depth=0.3f, deviation=0.2f, breadth=5
        // Implementation defined in RM_Path.cpp
    }

    pub fn SetRiverStyle(&mut self, depth: i32, points: i32, minwidth: f32, maxwidth: f32, beddepth: f32, deviation: f32, breadth: f32, bridge_name: &str) {
        // C++ had default values: depth=5, points=10, minwidth=0.01, maxwidth=0.03, beddepth=0.0f, deviation=0.25f, breadth=7, bridge_name=""
        // Implementation defined in RM_Path.cpp
    }

    pub fn GeneratePaths(&mut self, symmetric: symmetry_t) {
        // C++ had default value: symmetric=SYMMETRY_NONE
        // Implementation defined in RM_Path.cpp
    }

    pub fn GenerateRivers(&mut self) {
        // Implementation defined in RM_Path.cpp
    }
}

// Static members of CRMPathManager
pub static mut neighbor_x: [i32; 8] = [0; 8];
pub static mut neighbor_y: [i32; 8] = [0; 8];
