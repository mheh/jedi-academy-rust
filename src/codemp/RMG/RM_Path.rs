//! Mechanical port of `codemp/RMG/RM_Path.cpp`.
//!
//! Implements path and river generation for random missions.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};

// ============================================================================
// LOCAL STUBS for unported types
// ============================================================================

/// Stub for unported `class CRMManager` (RM_Manager.h).
pub struct CRMManager {
    _opaque: [u8; 0],
}

impl CRMManager {
    /// Stub for `CRMMission* CRMManager::GetMission()`.
    pub fn GetMission(&self) -> *mut CRMMission {
        core::ptr::null_mut()
    }

    /// Stub for `CCMLandScape* CRMManager::GetLandScape()`.
    pub fn GetLandScape(&self) -> *mut CCMLandScape {
        core::ptr::null_mut()
    }
}

/// Stub for unported `class CCMLandScape` (cm_landscape.h).
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

impl CCMLandScape {
    /// Stub for `float CCMLandScape::flrand(float, float)`.
    pub fn flrand(&self, _min: f32, _max: f32) -> f32 {
        0.0
    }

    /// Stub for `int CCMLandScape::irand(int, int)`.
    pub fn irand(&self, _min: c_int, _max: c_int) -> c_int {
        0
    }

    /// Stub for `vec3pair_t* CCMLandScape::GetBounds()`.
    pub fn GetBounds(&self) -> *const [[f32; 3]; 2] {
        core::ptr::null()
    }

    /// Stub for `void CCMLandScape::FlattenArea(CArea*, float, bool, bool, bool)`.
    pub fn FlattenArea(
        &mut self,
        _area: *mut CArea,
        _depth: f32,
        _is_flag1: bool,
        _is_flag2: bool,
        _is_flag3: bool,
    ) {
    }
}

/// Stub for unported `class CRandomTerrain` (RM_Terrain.h).
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

impl CRandomTerrain {
    /// Stub for `void CRandomTerrain::CreatePath(...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn CreatePath(
        &mut self,
        _pathcount: c_int,
        _arg2: c_int,
        _arg3: c_int,
        _points: c_int,
        _x1: f32,
        _y1: f32,
        _x2: f32,
        _y2: f32,
        _minwidth: f32,
        _maxwidth: f32,
        _depth: f32,
        _deviation: f32,
        _breadth: f32,
    ) {
    }
}

/// Stub for unported `class CRMMission` (RM_Mission.h).
pub struct CRMMission {
    _opaque: [u8; 0],
}

impl CRMMission {
    /// Stub for `bool CRMMission::GetSymmetric()`.
    pub fn GetSymmetric(&self) -> bool {
        false
    }

    /// Stub for `bool CRMMission::GetBackUpPath()`.
    pub fn GetBackUpPath(&self) -> bool {
        false
    }
}

/// Stub for unported `class CArea` (RM_Area.h).
pub struct CArea {
    _opaque: [u8; 0],
}

impl CArea {
    /// Stub for `void CArea::Init(...)`.
    pub fn Init(
        &mut self,
        _pos: *const [f32; 3],
        _radius: f32,
        _float_arg3: f32,
        _type_enum: c_int,
        _arg5: c_int,
        _arg6: c_int,
    ) {
    }
}

// Type definitions (expected from common headers)
pub type vec3_t = [f32; 3];

// ============================================================================
// extern "C" functions
// ============================================================================

extern "C" {
    /// Global instance of the random mission manager.
    pub static mut TheRandomMissionManager: *mut CRMManager;

    /// `int stricmp(const char*, const char*)` — case-insensitive string compare.
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// `void Com_Printf(const char*, ...)` — print to console.
    fn Com_Printf(msg: *const c_char, ...);
}

// ============================================================================
// Inline max() macro
// ============================================================================

#[inline]
fn max(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

// ============================================================================
// CRMNode class
// ============================================================================

/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Path.cpp
 *
 ************************************************************************************************/

pub struct CRMNode {
    mName: String,                   // name of node - "" if not used yet
    mPos: vec3_t,                    // where node is
    mPathID: [c_int; 8],             // path id's that lead from this node
    mAreaPointPlaced: bool,          // false if no area point here yet.
    mFlattenHeight: c_int,
}

impl CRMNode {
    /************************************************************************************************
     * CRMNode::CRMNode
     *	constructor
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new() -> Self {
        let mut node = CRMNode {
            mName: String::new(),
            mPos: [0.0; 3],
            mPathID: [-1; 8],
            mAreaPointPlaced: false,
            mFlattenHeight: 0,
        };

        node.mFlattenHeight = -1;

        node.mPos[0] = 0.0;
        node.mPos[1] = 0.0;
        node.mPos[2] = 0.0;

        // no paths
        for i in 0..8 {
            node.mPathID[i] = -1;
        }

        node.mAreaPointPlaced = false;

        node
    }

    pub fn IsLocation(&self) -> bool {
        self.mName.len() > 0
    }

    pub fn GetName(&self) -> &str {
        &self.mName
    }

    pub fn GetPos(&self) -> &vec3_t {
        &self.mPos
    }

    pub fn PathExist(&self, dir: c_int) -> bool {
        let idx = (dir % 8) as usize;
        self.mPathID[idx] != -1
    }

    pub fn GetPath(&self, dir: c_int) -> c_int {
        let idx = (dir % 8) as usize;
        self.mPathID[idx]
    }

    pub fn AreaPoint(&self) -> bool {
        self.mAreaPointPlaced
    }

    pub fn SetName(&mut self, name: &str) {
        self.mName = name.to_string();
    }

    pub fn SetPos(&mut self, v: &vec3_t) {
        self.mPos = *v;
    }

    pub fn SetPath(&mut self, dir: c_int, id: c_int) {
        let idx = (dir % 8) as usize;
        self.mPathID[idx] = id;
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

// ============================================================================
// CRMLoc class
// ============================================================================

// named spots on the map, should be placed into nodes
pub struct CRMLoc {
    mName: String,       // name of location
    mMinDepth: c_int,
    mMaxDepth: c_int,
    mMinPaths: c_int,
    mMaxPaths: c_int,
    mPlaced: bool,       // location has been placed at a node
}

impl CRMLoc {
    pub fn new(name: &str, min_depth: c_int, max_depth: c_int, min_paths: c_int, max_paths: c_int) -> Self {
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

// ============================================================================
// CRMCell class
// ============================================================================

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

    pub fn Border(&self) -> c_int {
        self.border
    }

    pub fn Wall(&self) -> c_int {
        self.wall
    }

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
        self.wall &= !(1 << dir);
    }
}

pub type rmCellVector_t = Vec<CRMCell>;

// Direction enums
pub const DIR_FIRST: c_int = 0;
pub const DIR_N: c_int = 0;
pub const DIR_NE: c_int = 1;
pub const DIR_E: c_int = 2;
pub const DIR_SE: c_int = 3;
pub const DIR_S: c_int = 4;
pub const DIR_SW: c_int = 5;
pub const DIR_W: c_int = 6;
pub const DIR_NW: c_int = 7;
pub const DIR_MAX: c_int = 8;
pub const DIR_ALL: c_int = 255;

pub const HALF_DIR_MAX: c_int = 4; // DIR_MAX / 2

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum symmetry_t {
    SYMMETRY_NONE = 0,
    SYMMETRY_TOPLEFT = 1,
    SYMMETRY_BOTTOMRIGHT = 2,
}

// ============================================================================
// CRMPathManager class
// ============================================================================

pub struct CRMPathManager {
    pub mXNodes: c_int,    // number of nodes in the x dimension
    pub mYNodes: c_int,    // number of nodes in the y dimension

    mLocations: rmLocVector_t,  // location, named spots to be placed at nodes
    mNodes: rmNodeVector_t,     // nodes, spots on map that *may* be connected by paths
    mCells: rmCellVector_t,     // array of cells for doing path generation

    mPathCount: c_int,
    mRiverCount: c_int,
    mMaxDepth: c_int,     // deepest any location wants to be
    mDepth: c_int,        // current depth

    mCrossed: bool,       // used to indicate if paths crossed the imaginary diagonal that cuts symmetric maps in half

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
    mRiverBridge: String,
    mRiverPos: vec3_t,

    mTerrain: *mut CRandomTerrain,
}

impl CRMPathManager {
    /************************************************************************************************
     * CRMPathManager::CRMPathManager
     *	constructor
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn new(terrain: *mut CRandomTerrain) -> Self {
        CRMPathManager {
            mXNodes: 0,
            mYNodes: 0,
            mPathCount: 0,
            mRiverCount: 0,
            mMaxDepth: 0,
            mDepth: 0,
            mLocations: Vec::new(),
            mNodes: Vec::new(),
            mCells: Vec::new(),
            mPathPoints: 10,
            mPathMinWidth: 0.02,
            mPathMaxWidth: 0.04,
            mPathDepth: 0.3,
            mPathDeviation: 0.03,
            mPathBreadth: 5.0,
            mRiverDepth: 5,
            mRiverPoints: 10,
            mRiverMinWidth: 0.01,
            mRiverMaxWidth: 0.02,
            mRiverBedDepth: 1.0,
            mRiverDeviation: 0.01,
            mRiverBreadth: 7.0,
            mTerrain: terrain,
            mCrossed: false,
            mRiverBridge: String::new(),
            mRiverPos: [0.0; 3],
        }
    }

    pub fn drop(&mut self) {
        let i_len = self.mLocations.len();
        for i in (0..i_len).rev() {
            if !self.mLocations[i].is_null() {
                let ptr = self.mLocations[i];
                unsafe {
                    let _ = Box::from_raw(ptr);
                }
            }
        }
        self.mLocations.clear();

        let j_len = self.mNodes.len();
        for j in (0..j_len).rev() {
            if !self.mNodes[j].is_null() {
                let ptr = self.mNodes[j];
                unsafe {
                    let _ = Box::from_raw(ptr);
                }
            }
        }
        self.mNodes.clear();
    }

    /************************************************************************************************
     * CRMPathManager::CreateLocation
     *	Create a location and add to list
     *
     * inputs:
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn CreateLocation(
        &mut self,
        name: &str,
        min_depth: c_int,
        mut max_depth: c_int,
        min_paths: c_int,
        mut max_paths: c_int,
    ) {
        // sanity checks -- dmv
        if max_paths < min_paths {
            unsafe {
                Com_Printf(
                    b"[CreateLocation()] ERROR : max_paths < min_paths :: set max_paths = min_paths\n\0"
                        .as_ptr() as *const c_char,
                );
            }
            max_paths = min_paths;
        }
        if max_depth < min_depth {
            unsafe {
                Com_Printf(
                    b"[CreateLocation()] ERROR : max_depth < min_depth :: set max_depth = min_depth\n\0"
                        .as_ptr() as *const c_char,
                );
            }
            max_depth = min_depth;
        }

        let locations_len = self.mLocations.len();
        for i in (0..locations_len).rev() {
            let loc_ptr = self.mLocations[i];
            if !loc_ptr.is_null() {
                unsafe {
                    if stricmp(
                        name.as_ptr() as *const c_char,
                        (*loc_ptr).GetName().as_ptr() as *const c_char,
                    ) == 0
                    {
                        (*loc_ptr).SetMinDepth(min_depth);
                        (*loc_ptr).SetMaxDepth(max_depth);
                        (*loc_ptr).SetMinPaths(min_paths);
                        (*loc_ptr).SetMaxPaths(max_paths);
                        return;
                    }
                }
            }
        }

        let p_loc = Box::into_raw(Box::new(CRMLoc::new(name, min_depth, max_depth, min_paths, max_paths)));
        self.mLocations.push(p_loc);
        self.mMaxDepth = max(self.mMaxDepth, max_depth);
    }

    pub fn ClearCells(&mut self, x_nodes: c_int, y_nodes: c_int) {
        // clear cell array - used for generating paths
        let empty = CRMCell::new();
        let total_cells = (x_nodes * y_nodes) as usize;
        for x in 0..total_cells {
            if x >= self.mCells.len() {
                self.mCells.push(empty);
            } else {
                self.mCells[x] = empty;
            }
        }

        // set borders of world
        for y in 0..(y_nodes as usize) {
            let idx_w = (y as c_int) * x_nodes;
            let idx_e = (y as c_int) * x_nodes + x_nodes - 1;

            self.mCells[idx_w as usize].SetBorder(DIR_W);
            self.mCells[idx_w as usize].SetBorder(DIR_SW);
            self.mCells[idx_w as usize].SetBorder(DIR_NW);

            self.mCells[idx_e as usize].SetBorder(DIR_E);
            self.mCells[idx_e as usize].SetBorder(DIR_NE);
            self.mCells[idx_e as usize].SetBorder(DIR_SE);
        }

        for x in 0..(x_nodes as usize) {
            let x_val = x as c_int;
            self.mCells[x].SetBorder(DIR_N);
            self.mCells[x].SetBorder(DIR_NE);
            self.mCells[x].SetBorder(DIR_NW);

            let idx_s = (y_nodes - 1) * x_nodes + x_val;
            self.mCells[idx_s as usize].SetBorder(DIR_S);
            self.mCells[idx_s as usize].SetBorder(DIR_SE);
            self.mCells[idx_s as usize].SetBorder(DIR_SW);
        }
    }

    /************************************************************************************************
     * CRMPathManager::CreateArray
     *	Create array of nodes that are spaced over the landscape.
     *	Create array of cells, which is used to determine how nodes are connected.
     *
     * inputs:
     *  x_nodes, y_nodes - how many nodes in each dimension to layout
     *
     * return:
     *	true if the node array was created, false if we have a problem
     *
     ************************************************************************************************/
    pub fn CreateArray(&mut self, x_nodes: c_int, y_nodes: c_int) -> bool {
        self.mXNodes = x_nodes;
        self.mYNodes = y_nodes;

        // dump existing nodes
        let nodes_len = self.mNodes.len();
        for x in (0..nodes_len).rev() {
            if !self.mNodes[x].is_null() {
                let ptr = self.mNodes[x];
                unsafe {
                    let _ = Box::from_raw(ptr);
                }
            }
        }
        self.mNodes.clear();

        let total_nodes = (x_nodes * y_nodes) as usize;
        self.mNodes.resize(total_nodes, core::ptr::null_mut());

        // add a small amount of random jitter to spots chosen
        let x_rnd = 0.2 / (x_nodes + 1) as f32;
        let y_rnd = 0.2 / (y_nodes + 1) as f32;

        for x in 0..(x_nodes as usize) {
            let cell_x = (x as f32 + 1.0) / (x_nodes + 1) as f32;

            for y in 0..(y_nodes as usize) {
                let pnode = Box::into_raw(Box::new(CRMNode::new()));
                self.mNodes[x + y * (x_nodes as usize)] = pnode;

                let cell_y = (y as f32 + 1.0) / (y_nodes + 1) as f32;

                unsafe {
                    let landscape = (*TheRandomMissionManager).GetLandScape();
                    let pos_x = (*landscape).flrand(cell_x - x_rnd, cell_x + x_rnd);
                    let pos_y = (*landscape).flrand(cell_y - y_rnd, cell_y + y_rnd);

                    let mut pos: vec3_t = [pos_x, pos_y, 0.0];

                    self.SetNodePos(x as c_int, y as c_int, &pos);
                }
            }
        }

        self.ClearCells(x_nodes, y_nodes);

        true
    }

    /************************************************************************************************
     * CRMPathManager::neighbor_x and neighbor_y
     * neighbor offsets - easy way to turn a direction into the array position for a neighboring cell or node
     ************************************************************************************************/
    // These are static members, defined below after impl block

    pub fn PlaceLocation(&mut self, c_x: c_int, c_y: c_int) {
        if !self.Node(c_x, c_y).IsLocation() {
            // not currently a location

            // how many paths lead to this cell?
            let mut count_paths = 0;

            for i in 0..DIR_MAX {
                if self.Node(c_x, c_y).PathExist(i) {
                    count_paths += 1;
                }
            }

            let mut deepest_depth = -1;
            let mut deepest_loc = -1;

            let locations_len = self.mLocations.len();
            for i in (0..locations_len).rev() {
                let loc_ptr = self.mLocations[i];
                if !loc_ptr.is_null() {
                    unsafe {
                        if !(*loc_ptr).Placed()
                            && (*loc_ptr).MinDepth() <= self.mDepth
                            && (*loc_ptr).MaxDepth() >= self.mDepth
                            && (*loc_ptr).MinPaths() <= count_paths
                            && (*loc_ptr).MaxPaths() >= count_paths
                            && (*loc_ptr).MaxDepth() > deepest_depth
                        {
                            deepest_loc = i as c_int;
                            deepest_depth = (*loc_ptr).MaxDepth();
                        }
                    }
                }
            }

            if deepest_loc >= 0 && (deepest_loc as usize) < self.mLocations.len() {
                // found a location to place at this node / cell
                let loc_ptr = self.mLocations[deepest_loc as usize];
                if !loc_ptr.is_null() {
                    unsafe {
                        let name = (*loc_ptr).GetName();
                        self.Node(c_x, c_y).SetName(name);
                        (*loc_ptr).SetPlaced(true);

                        // need a new max depth
                        let mut max_depth = -1;
                        let locations_len = self.mLocations.len();
                        for i in (0..locations_len).rev() {
                            let loc_ptr2 = self.mLocations[i];
                            if !loc_ptr2.is_null() {
                                // figure out new max depth based on the max depth of unplaced locations
                                if !(*loc_ptr2).Placed() && (*loc_ptr2).MaxDepth() > max_depth {
                                    max_depth = (*loc_ptr2).MaxDepth();
                                }
                            }
                        }
                        self.mMaxDepth = max_depth;
                    }
                }
            }
        }
    }

    /************************************************************************************************
     * CRMPathManager::PathVisit
     * This method is called recursively to create a network of nodes connected with paths.
     *
     * inputs:
     *  c_x, c_y - cell to visit
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn PathVisit(&mut self, c_x: c_int, c_y: c_int) {
        // does this cell have any neighbors with all walls intact?
        // look at neighbors in random order
        unsafe {
            let off = (*TheRandomMissionManager)
                .GetLandScape()
                .irand(DIR_FIRST, DIR_MAX - 1);

            self.mDepth += 1; // track our depth of recursion

            for i in DIR_FIRST..DIR_MAX {
                if self.mDepth > self.mMaxDepth {
                    break;
                }

                let d = (i + off) % DIR_MAX;
                if !self.Cell(c_x, c_y).Border_dir(d) {
                    // we can move this way, since no border
                    let new_c_x = c_x + NEIGHBOR_X[d as usize];
                    let new_c_y = c_y + NEIGHBOR_Y[d as usize];

                    if self.Cell(new_c_x, new_c_y).Wall() == DIR_ALL {
                        // we have a new cell that has not been visited!
                        let new_dir;
                        // d is the direction relative to the current cell
                        // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                        if d < HALF_DIR_MAX {
                            new_dir = d + HALF_DIR_MAX;
                        } else {
                            new_dir = d - HALF_DIR_MAX;
                        }

                        // knock down walls
                        self.Cell(c_x, c_y).RemoveWall(d);
                        self.Cell(new_c_x, new_c_y).RemoveWall(new_dir);

                        // set path id
                        self.Node(c_x, c_y).SetPath(d, self.mPathCount);
                        self.Node(new_c_x, new_c_y)
                            .SetPath(new_dir, self.mPathCount);

                        // create path between cells
                        let node_pos_1 = self.GetNodePos(c_x, c_y);
                        let node_pos_2 = self.GetNodePos(new_c_x, new_c_y);

                        (*self.mTerrain).CreatePath(
                            self.mPathCount,
                            -1,
                            0,
                            self.mPathPoints,
                            node_pos_1[0],
                            node_pos_1[1],
                            node_pos_2[0],
                            node_pos_2[1],
                            self.mPathMinWidth,
                            self.mPathMaxWidth,
                            self.mPathDepth,
                            self.mPathDeviation,
                            self.mPathBreadth,
                        );

                        self.mPathCount += 1;

                        // flatten a small spot
                        let mut area = CArea {};
                        let bounds = (*(*TheRandomMissionManager).GetLandScape()).GetBounds();
                        let flat_radius = self.mPathMaxWidth
                            * ((*bounds)[1][0] - (*bounds)[0][0]).abs();
                        area.Init(
                            self.GetNodePos(c_x, c_y).as_ptr(),
                            flat_radius,
                            0.0,
                            0, // AT_NONE
                            0,
                            0,
                        );
                        (*(*TheRandomMissionManager).GetLandScape())
                            .FlattenArea(&mut area, 255.0 * self.mPathDepth, false, true, true);

                        // recurse
                        self.PathVisit(new_c_x, new_c_y);
                    }
                }
            }

            self.mDepth -= 1;

            // NOTE: *whoop* hack alert, the first time this is reached, it should be the very last placed node.
            if !self.mCrossed
                && (*(*TheRandomMissionManager).GetMission()).GetSymmetric()
                && (*(*TheRandomMissionManager).GetMission()).GetBackUpPath()
            {
                self.mCrossed = true;

                let directionSet: [[c_int; 3]; 3] = [
                    [DIR_NW, DIR_W, DIR_SW],
                    [DIR_N, -1, DIR_S],
                    [DIR_NE, DIR_E, DIR_SE],
                ];

                let ncx = (self.mXNodes - 1) - c_x;
                let ncy = (self.mYNodes - 1) - c_y;

                let mut x_delta = ncx - c_x;
                let mut y_delta = ncy - c_y;

                if x_delta < -1 {
                    x_delta = -1;
                } else if x_delta > 1 {
                    x_delta = 1;
                }

                if y_delta < -1 {
                    y_delta = -1;
                } else if y_delta > 1 {
                    y_delta = 1;
                }

                // make sure the mirror is actually in a different position than then un-mirrored node
                if x_delta != 0 || y_delta != 0 {
                    let x_idx = (x_delta + 1) as usize;
                    let y_idx = (y_delta + 1) as usize;
                    let d = directionSet[y_idx][x_idx];
                    let new_dir;
                    // d is the direction relative to the current cell
                    // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                    if d < HALF_DIR_MAX {
                        new_dir = d + HALF_DIR_MAX;
                    } else {
                        new_dir = d - HALF_DIR_MAX;
                    }

                    //NOTE: Knocking down these walls will cause instances to be created on this new artificial path
                    // Since this path could span more than just the normal 1 cell, these walls being knocked down are not exactly correct... but get the job done

                    // knock down walls
                    self.Cell(c_x, c_y).RemoveWall(d);
                    self.Cell(ncx, ncy).RemoveWall(new_dir);

                    // set path id
                    self.Node(c_x, c_y).SetPath(d, self.mPathCount);
                    self.Node(ncx, ncy).SetPath(new_dir, self.mPathCount);

                    // create an artificial path that crosses over to connect the symmetric and non-symmetric map parts
                    let node_pos_1 = self.GetNodePos(c_x, c_y);
                    let node_pos_2 = self.GetNodePos(ncx, ncy);

                    (*self.mTerrain).CreatePath(
                        self.mPathCount,
                        -1,
                        0,
                        self.mPathPoints,
                        node_pos_1[0],
                        node_pos_1[1],
                        node_pos_2[0],
                        node_pos_2[1],
                        self.mPathMinWidth,
                        self.mPathMaxWidth,
                        self.mPathDepth,
                        self.mPathDeviation,
                        self.mPathBreadth,
                    );

                    self.mPathCount += 1;
                }
            }

            self.PlaceLocation(c_x, c_y);
        }
    }

    /************************************************************************************************
     * CRMPathManager::FindNodeByName
     *	Finds the managed node with the matching case-insensivity name
     *
     * inputs:
     *  name - name of the node to find
     *
     * return:
     *	a pointer to the found node or NULL if the node couldn't be found
     *
     ************************************************************************************************/
    pub fn FindNodeByName(&self, name: &str) -> *mut CRMNode {
        let j_len = self.mNodes.len();
        for j in (0..j_len).rev() {
            if !self.mNodes[j].is_null() {
                unsafe {
                    if stricmp(
                        name.as_ptr() as *const c_char,
                        (*self.mNodes[j]).GetName().as_ptr() as *const c_char,
                    ) == 0
                    {
                        return self.mNodes[j];
                    }
                }
            }
        }
        core::ptr::null_mut()
    }

    /************************************************************************************************
     * CRMPathManager::SetPathStyle
     *	sets style for all paths
     *
     * inputs:
     *  settings for paths that are created
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn SetPathStyle(
        &mut self,
        points: c_int,
        minwidth: f32,
        maxwidth: f32,
        depth: f32,
        deviation: f32,
        breadth: f32,
    ) {
        // save path style
        self.mPathPoints = points;
        self.mPathMinWidth = minwidth;
        self.mPathMaxWidth = maxwidth;
        self.mPathDepth = depth;
        self.mPathDeviation = deviation;
        self.mPathBreadth = breadth;
    }

    /************************************************************************************************
     * CRMPathManager::SetRiverStyle
     *	sets style for all rivers
     *
     * inputs:
     *  settings for river paths that are created
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn SetRiverStyle(
        &mut self,
        depth: c_int,
        points: c_int,
        minwidth: f32,
        maxwidth: f32,
        beddepth: f32,
        deviation: f32,
        breadth: f32,
        bridge_name: &str,
    ) {
        // save river style
        self.mRiverDepth = depth;
        self.mRiverPoints = points;
        self.mRiverMinWidth = minwidth;
        self.mRiverMaxWidth = maxwidth;
        self.mRiverBedDepth = beddepth;
        self.mRiverDeviation = deviation;
        self.mRiverBreadth = breadth;
        self.mRiverBridge = bridge_name.to_string();
    }

    pub fn GetRiverPos(&mut self, x: c_int, y: c_int) -> &mut vec3_t {
        self.mRiverPos[0] = (x as f32 + 1.0) / (self.mXNodes + 2) as f32;
        self.mRiverPos[1] = (y as f32 + 1.0) / (self.mYNodes + 2) as f32;
        &mut self.mRiverPos
    }

    pub fn RiverVisit(&mut self, c_x: c_int, c_y: c_int) {
        // does this cell have any neighbors with all walls intact?
        unsafe {
            // look at neighbors in random order
            let off = (*TheRandomMissionManager)
                .GetLandScape()
                .irand(DIR_FIRST, DIR_MAX - 1);

            self.mDepth += 1; // track our depth of recursion

            for i in (DIR_FIRST..DIR_MAX).step_by(2) {
                if self.mDepth > self.mMaxDepth {
                    break;
                }

                let d = (i + off) % DIR_MAX;
                if !self.Cell(c_x, c_y).Border_dir(d) {
                    // we can move this way, since no border
                    let new_c_x = c_x + NEIGHBOR_X[d as usize];
                    let new_c_y = c_y + NEIGHBOR_Y[d as usize];

                    if self.RiverCell(new_c_x, new_c_y).Wall() == DIR_ALL {
                        // we have a new cell that has not been visited!

                        let new_dir;
                        // d is the direction relative to the current cell
                        // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                        if d < HALF_DIR_MAX {
                            new_dir = d + HALF_DIR_MAX;
                        } else {
                            new_dir = d - HALF_DIR_MAX;
                        }

                        // knock down walls
                        self.RiverCell(c_x, c_y).RemoveWall(d);
                        self.RiverCell(new_c_x, new_c_y).RemoveWall(new_dir);

                        // create river between cells
                        let river_pos_1 = [
                            self.mRiverPos[0],
                            self.mRiverPos[1],
                            self.mRiverPos[2],
                        ];
                        let river_pos_2_arr = self.GetRiverPos(new_c_x, new_c_y);
                        let river_pos_2 = [
                            river_pos_2_arr[0],
                            river_pos_2_arr[1],
                            river_pos_2_arr[2],
                        ];

                        self.GetRiverPos(c_x, c_y);

                        (*self.mTerrain).CreatePath(
                            self.mPathCount,
                            -1,
                            0,
                            self.mRiverPoints,
                            self.mRiverPos[0],
                            self.mRiverPos[1],
                            river_pos_2[0],
                            river_pos_2[1],
                            self.mRiverMinWidth,
                            self.mRiverMaxWidth,
                            self.mRiverBedDepth,
                            self.mRiverDeviation,
                            self.mRiverBreadth,
                        );

                        self.mPathCount += 1;

                        // flatten a small spot
                        let mut area = CArea {};
                        let bounds = (*(*TheRandomMissionManager).GetLandScape()).GetBounds();
                        let flat_radius = self.mRiverMinWidth
                            * ((*bounds)[1][0] - (*bounds)[0][0]).abs();
                        area.Init(
                            self.GetRiverPos(c_x, c_y).as_ptr(),
                            flat_radius,
                            0.0,
                            0, // AT_NONE
                            0,
                            0,
                        );
                        (*(*TheRandomMissionManager).GetLandScape()).FlattenArea(
                            &mut area,
                            255.0 * self.mRiverBedDepth,
                            false,
                            true,
                            true,
                        );

                        // recurse
                        self.RiverVisit(new_c_x, new_c_y);
                    }
                }
            }
        }

        // NOTE: --mDepth commented out in original C++ at line 591
    }

    /************************************************************************************************
     * CRMPathManager::GenerateRivers
     *	Creates a river which intersects the main path
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn GenerateRivers(&mut self) {
        if self.mRiverBedDepth == 1.0 {
            // no rivers
            return;
        }

        self.mMaxDepth = self.mRiverDepth;
        self.mDepth = 0;

        let mut cell_x = 0;
        let mut cell_y = 0;

        // choose starting cell along an edge
        unsafe {
            let edge = (*TheRandomMissionManager)
                .GetLandScape()
                .irand(0, 7);

            match edge {
                0 => {
                    cell_x = self.mXNodes / 2;
                    cell_y = 0;
                }
                1 => {
                    cell_x = self.mXNodes;
                    cell_y = 0;
                }
                2 => {
                    cell_x = self.mXNodes;
                    cell_y = self.mYNodes / 2;
                }
                3 => {
                    cell_x = self.mXNodes;
                    cell_y = self.mYNodes;
                }
                4 => {
                    cell_x = self.mXNodes / 2;
                    cell_y = self.mYNodes;
                }
                5 => {
                    cell_x = 0;
                    cell_y = self.mYNodes;
                }
                6 => {
                    cell_x = 0;
                    cell_y = self.mYNodes / 2;
                }
                _ => {
                    cell_x = 0;
                    cell_y = 0;
                }
            }
        }

        self.ClearCells(self.mXNodes + 1, self.mYNodes + 1);

        self.mRiverCount = self.mPathCount;

        // visit the first cell
        self.RiverVisit(cell_x, cell_y);

        self.mRiverCount = self.mPathCount - self.mRiverCount;
    }

    /************************************************************************************************
     * CRMPathManager::GeneratePaths
     *	Creates all paths
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn GeneratePaths(&mut self, symmetric: symmetry_t) {
        let mut cell_x = 0;
        let mut cell_y = 0;

        match symmetric {
            symmetry_t::SYMMETRY_TOPLEFT => {
                cell_x = self.mXNodes - 1;
                cell_y = 0;
            }

            symmetry_t::SYMMETRY_BOTTOMRIGHT => {
                cell_x = 0;
                cell_y = self.mYNodes - 1;
            }

            symmetry_t::SYMMETRY_NONE => {
                // choose starting cell along an edge
                unsafe {
                    match (*TheRandomMissionManager)
                        .GetLandScape()
                        .irand(0, 7)
                    {
                        0 => {
                            cell_x = self.mXNodes / 2;
                        }
                        1 => {
                            cell_x = self.mXNodes - 1;
                        }
                        2 => {
                            cell_x = self.mXNodes - 1;
                            cell_y = self.mYNodes / 2;
                        }
                        3 => {
                            cell_x = self.mXNodes - 1;
                            cell_y = self.mYNodes - 1;
                        }
                        4 => {
                            cell_x = self.mXNodes / 2;
                            cell_y = self.mYNodes - 1;
                        }
                        5 => {
                            cell_y = self.mYNodes - 1;
                        }
                        6 => {
                            cell_y = self.mYNodes / 2;
                        }
                        _ => {
                            // case 7 or default: both remain 0
                        }
                    }
                }
            }
        }

        // visit the first cell
        self.PathVisit(cell_x, cell_y);
    }

    // Helper methods
    fn Node(&mut self, x: c_int, y: c_int) -> &mut CRMNode {
        let idx = (x + y * self.mXNodes) as usize;
        unsafe { &mut *self.mNodes[idx] }
    }

    fn Cell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        let idx = (x + y * self.mXNodes) as usize;
        &mut self.mCells[idx]
    }

    fn RiverCell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        let idx = (x + y * (self.mXNodes + 1)) as usize;
        &mut self.mCells[idx]
    }

    fn GetNodePos(&self, x: c_int, y: c_int) -> &vec3_t {
        let idx = (x + y * self.mXNodes) as usize;
        unsafe { (*self.mNodes[idx]).GetPos() }
    }

    fn SetNodePos(&mut self, x: c_int, y: c_int, pos: &vec3_t) {
        let idx = (x + y * self.mXNodes) as usize;
        unsafe {
            (*self.mNodes[idx]).SetPos(pos);
        }
    }
}

// ============================================================================
// Static member arrays
// ============================================================================

// neighbor offsets - easy way to turn a direction into the array position for a neighboring cell or node
pub const NEIGHBOR_X: [c_int; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
pub const NEIGHBOR_Y: [c_int; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];
