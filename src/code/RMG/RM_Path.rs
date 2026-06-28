/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Path.cpp
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_int;
use std::ptr;

// porting stub: external types declared as opaque
#[repr(C)]
pub struct CRandomTerrain {
    _opaque: [u8; 0],
}

impl CRandomTerrain {
    pub unsafe fn CreatePath(
        &mut self,
        path_id: c_int,
        arg2: c_int,
        arg3: c_int,
        points: c_int,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        min_width: f32,
        max_width: f32,
        depth: f32,
        deviation: f32,
        breadth: f32,
    ) {
        // Stub: implementation provided by external code
    }
}

#[repr(C)]
pub struct CArea {
    _opaque: [u8; 0],
}

impl CArea {
    pub fn new() -> Self {
        CArea {
            _opaque: [],
        }
    }

    pub fn default() -> Self {
        CArea {
            _opaque: [],
        }
    }

    pub unsafe fn Init(
        &mut self,
        pos: *mut vec3_t,
        radius: f32,
        arg3: f32,
        arg4: c_int,
        arg5: c_int,
        arg6: c_int,
    ) {
        // Stub: implementation provided by external code
    }
}

#[repr(C)]
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

impl CCMLandScape {
    pub unsafe fn irand(&self, min: c_int, max: c_int) -> c_int {
        // Stub: implementation provided by external code
        0
    }

    pub unsafe fn flrand(&self, min: f32, max: f32) -> f32 {
        // Stub: implementation provided by external code
        0.0
    }

    pub unsafe fn GetBounds(&self) -> *mut [[f32; 3]; 2] {
        // Stub: implementation provided by external code
        ptr::null_mut()
    }

    pub unsafe fn FlattenArea(
        &self,
        area: *mut CArea,
        height: c_int,
        arg3: bool,
        arg4: bool,
        arg5: bool,
    ) {
        // Stub: implementation provided by external code
    }
}

#[repr(C)]
pub struct CRMMission {
    _opaque: [u8; 0],
}

impl CRMMission {
    pub unsafe fn GetSymmetric(&self) -> bool {
        // Stub: implementation provided by external code
        false
    }

    pub unsafe fn GetBackUpPath(&self) -> bool {
        // Stub: implementation provided by external code
        false
    }
}

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

pub const HALF_DIR_MAX: c_int = 4;

// symmetry_t enum
#[repr(C)]
#[derive(Copy, Clone)]
pub enum symmetry_t {
    SYMMETRY_NONE = 0,
    SYMMETRY_TOPLEFT = 1,
    SYMMETRY_BOTTOMRIGHT = 2,
}

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
                _data: ptr::null_mut(),
                _capacity: 0,
                _size: 0,
            },
            mPos: [0.0, 0.0, 0.0],
            mPathID: [-1; 8],
            mAreaPointPlaced: false,
            mFlattenHeight: -1,
        }
    }

    pub fn IsLocation(&self) -> bool {
        let len = unsafe {
            let mut len = 0usize;
            let mut ptr_char = self.mName.c_str();
            while !ptr_char.is_null() && *ptr_char != 0 {
                len += 1;
                ptr_char = ptr_char.add(1);
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
            if self.mPathID[idx] != -1 { 1 } else { 0 }
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
        self.mName._data = name as *mut core::ffi::c_char;
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
        self.mName._data = name as *mut core::ffi::c_char;
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

    pub fn Border(&self, dir: c_int) -> bool {
        (self.border & (1 << dir)) != 0
    }

    pub fn Wall(&self, dir: c_int) -> bool {
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

// neighbor offsets - easy way to turn a direction into the array position for a neighboring cell or node
pub static mut neighbor_x: [c_int; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
pub static mut neighbor_y: [c_int; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];

// porting stub: external functions
extern "C" {
    fn Com_Printf(format: *const core::ffi::c_char, ...);

    // Declare stricmp as an extern C function for case-insensitive comparison
    fn stricmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> c_int;
}

// Stub for global TheRandomMissionManager - accessed dynamically
pub static mut TheRandomMissionManager: *mut CRMManager = ptr::null_mut();

#[repr(C)]
pub struct CRMManager {
    _opaque: [u8; 0],
}

impl CRMManager {
    pub unsafe fn GetLandScape(&self) -> *mut CCMLandScape {
        // Stub: implementation provided by external code
        ptr::null_mut()
    }

    pub unsafe fn GetMission(&self) -> *mut CRMMission {
        // Stub: implementation provided by external code
        ptr::null_mut()
    }
}

// Helper macro equivalent for max
#[inline]
fn max(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
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
            mLocations: Vec::new(),
            mNodes: Vec::new(),
            mCells: Vec::new(),
            mPathCount: 0,
            mRiverCount: 0,
            mMaxDepth: 0,
            mDepth: 0,
            mCrossed: false,
            mPathPoints: 10,
            mPathMinWidth: 0.02f,
            mPathMaxWidth: 0.04f,
            mPathDepth: 0.3f,
            mPathDeviation: 0.03f,
            mPathBreadth: 5.0f,
            mRiverDepth: 5,
            mRiverPoints: 10,
            mRiverMinWidth: 0.01f,
            mRiverMaxWidth: 0.02f,
            mRiverBedDepth: 1.0f,
            mRiverDeviation: 0.01f,
            mRiverBreadth: 7.0f,
            mRiverBridge: string {
                _data: ptr::null_mut(),
                _capacity: 0,
                _size: 0,
            },
            mRiverPos: [0.0, 0.0, 0.0],
            mTerrain: terrain,
        }
    }

    /************************************************************************************************
     * CRMPathManager::~CRMPathManager
     *	destructor
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn drop(&mut self) {
        let mut i: c_int;

        i = self.mLocations.len() as c_int - 1;
        while i >= 0 {
            if !self.mLocations[i as usize].is_null() {
                let _ = unsafe { Box::from_raw(self.mLocations[i as usize]) };
            }
            i -= 1;
        }
        self.mLocations.clear();

        let mut j: c_int = self.mNodes.len() as c_int - 1;
        while j >= 0 {
            if !self.mNodes[j as usize].is_null() {
                let _ = unsafe { Box::from_raw(self.mNodes[j as usize]) };
            }
            j -= 1;
        }
        self.mNodes.clear();
    }

    pub fn CreateLocation(
        &mut self,
        name: *const core::ffi::c_char,
        min_depth: c_int,
        mut max_depth: c_int,
        min_paths: c_int,
        mut max_paths: c_int,
    ) {
        let mut i: c_int;

        // sanity checks -- dmv
        if max_paths < min_paths {
            unsafe {
                Com_Printf(
                    "[CreateLocation()] ERROR : max_paths < min_paths :: set max_paths = min_paths\n"
                        as *const core::ffi::c_char,
                );
            }
            max_paths = min_paths;
        }
        if max_depth < min_depth {
            unsafe {
                Com_Printf(
                    "[CreateLocation()] ERROR : max_depth < min_depth :: set max_depth = min_depth\n"
                        as *const core::ffi::c_char,
                );
            }
            max_depth = min_depth;
        }

        i = self.mLocations.len() as c_int - 1;
        while i >= 0 {
            if !self.mLocations[i as usize].is_null() {
                let loc = unsafe { &mut *self.mLocations[i as usize] };
                let loc_name = loc.GetName();
                if unsafe { stricmp(name, loc_name) } == 0 {
                    loc.SetMinDepth(min_depth);
                    loc.SetMaxDepth(max_depth);
                    loc.SetMinPaths(min_paths);
                    loc.SetMaxPaths(max_paths);
                    return;
                }
            }
            i -= 1;
        }

        let pLoc = unsafe { Box::new(CRMLoc::new(name, min_depth, max_depth, min_paths, max_paths)) };
        self.mLocations.push(Box::into_raw(pLoc));
        self.mMaxDepth = max(self.mMaxDepth, max_depth);
    }

    pub fn ClearCells(&mut self, x_nodes: c_int, y_nodes: c_int) {
        let mut x: c_int;
        let mut y: c_int;

        // clear cell array - used for generating paths
        let empty = CRMCell::new();
        x = 0;
        while x < x_nodes * y_nodes {
            if x >= self.mCells.len() as c_int {
                self.mCells.push(empty);
            } else {
                self.mCells[x as usize] = empty;
            }
            x += 1;
        }

        // set borders of world
        y = 0;
        while y < y_nodes {
            self.mCells[(y * x_nodes) as usize].SetBorder(6 /* DIR_W */);
            self.mCells[(y * x_nodes) as usize].SetBorder(5 /* DIR_SW */);
            self.mCells[(y * x_nodes) as usize].SetBorder(7 /* DIR_NW */);
            self.mCells[((y * x_nodes + x_nodes - 1) as usize)].SetBorder(2 /* DIR_E */);
            self.mCells[((y * x_nodes + x_nodes - 1) as usize)].SetBorder(3 /* DIR_NE */);
            self.mCells[((y * x_nodes + x_nodes - 1) as usize)].SetBorder(4 /* DIR_SE */);
            y += 1;
        }

        x = 0;
        while x < x_nodes {
            self.mCells[x as usize].SetBorder(0 /* DIR_N */);
            self.mCells[x as usize].SetBorder(1 /* DIR_NE */);
            self.mCells[x as usize].SetBorder(7 /* DIR_NW */);
            self.mCells[(((y_nodes - 1) * x_nodes + x) as usize)].SetBorder(4 /* DIR_S */);
            self.mCells[(((y_nodes - 1) * x_nodes + x) as usize)].SetBorder(3 /* DIR_SE */);
            self.mCells[(((y_nodes - 1) * x_nodes + x) as usize)].SetBorder(5 /* DIR_SW */);
            x += 1;
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

        // fill node array with positions that are spaced over the landscape
        let mut x: c_int;
        let mut y: c_int;

        // dump existing nodes
        x = self.mNodes.len() as c_int - 1;
        while x >= 0 {
            if !self.mNodes[x as usize].is_null() {
                let _ = unsafe { Box::from_raw(self.mNodes[x as usize]) };
            }
            x -= 1;
        }
        self.mNodes.clear();
        self.mNodes.resize(
            (self.mXNodes * self.mYNodes) as usize,
            ptr::null_mut(),
        );

        // add a small amount of random jitter to spots chosen
        let x_rnd = 0.2f / ((self.mXNodes + 1) as f32);
        let y_rnd = 0.2f / ((self.mYNodes + 1) as f32);

        x = 0;
        while x < self.mXNodes {
            let cell_x = (x as f32 + 1.0f) / ((self.mXNodes + 1) as f32);
            //		float cell_x = (x + 2.0f) / (mXNodes+3);

            y = 0;
            while y < self.mYNodes {
                let mut pos: vec3_t = [0.0; 3];
                let pnode = unsafe { Box::new(CRMNode::new()) };
                self.mNodes[(x + y * self.mXNodes) as usize] = Box::into_raw(pnode);

                let cell_y = (y as f32 + 1.0f) / ((self.mYNodes + 1) as f32);
                //			float cell_y = (y + 2.0f) / (mYNodes+3);

                unsafe {
                    let landscape = (*TheRandomMissionManager).GetLandScape();
                    if !landscape.is_null() {
                        pos[0] = (*landscape).flrand(cell_x - x_rnd, cell_x + x_rnd);
                        pos[1] = (*landscape).flrand(cell_y - y_rnd, cell_y + y_rnd);
                    }
                    pos[2] = 0.0;

                    self.SetNodePos(x, y, &pos);
                }

                y += 1;
            }

            x += 1;
        }

        self.ClearCells(self.mXNodes, self.mYNodes);

        true
    }

    /************************************************************************************************
     * CRMPathManager::PlaceLocation
     * This method is used to determine if a named location should be placed at this node.
     *
     * inputs:
     *  c_x, c_y - cell to examine
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn PlaceLocation(&mut self, c_x: c_int, c_y: c_int) {
        let node = self.Node(c_x, c_y);
        if !node.is_null() {
            let is_location = unsafe { (*node).IsLocation() };

            if !is_location {
                // not currently a location

                // how many paths lead to this cell?
                let mut count_paths = 0;
                let mut i: c_int;

                i = 0;
                while i < 8 {
                    let path_exist = unsafe { (*node).PathExist(i) };
                    if path_exist != 0 {
                        count_paths += 1;
                    }
                    i += 1;
                }

                let mut deepest_depth = -1;
                let mut deepest_loc = -1;
                i = self.mLocations.len() as c_int - 1;
                while i >= 0 {
                    if !self.mLocations[i as usize].is_null() {
                        let loc = unsafe { &*self.mLocations[i as usize] };
                        if !loc.Placed()
                            && loc.MinDepth() <= self.mDepth
                            && loc.MaxDepth() >= self.mDepth
                            && loc.MinPaths() <= count_paths
                            && loc.MaxPaths() >= count_paths
                            && loc.MaxDepth() > deepest_depth
                        {
                            deepest_loc = i;
                            deepest_depth = loc.MaxDepth();
                        }
                    }
                    i -= 1;
                }

                if deepest_loc >= 0 && (deepest_loc as usize) < self.mLocations.len() {
                    // found a location to place at this node / cell
                    let loc = unsafe { &mut *self.mLocations[deepest_loc as usize] };
                    let name = loc.GetName();
                    unsafe { (*node).SetName(name) };
                    loc.SetPlaced(true);

                    // need a new max depth
                    let mut max_depth = -1;
                    i = self.mLocations.len() as c_int - 1;
                    while i >= 0 {
                        if !self.mLocations[i as usize].is_null() {
                            let loc_iter = unsafe { &*self.mLocations[i as usize] };
                            // figure out new max depth based on the max depth of unplaced locations
                            if !loc_iter.Placed() && loc_iter.MaxDepth() > max_depth {
                                max_depth = loc_iter.MaxDepth();
                            }
                        }
                        i -= 1;
                    }
                    self.mMaxDepth = max_depth;
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
        let mut i: c_int;
        let mut off: c_int;

        // look at neighbors in random order
        unsafe {
            let landscape = (*TheRandomMissionManager).GetLandScape();
            if !landscape.is_null() {
                off = (*landscape).irand(0, 7);
            } else {
                off = 0;
            }
        }

        self.mDepth += 1; // track our depth of recursion

        i = 0;
        while i < 8 && self.mDepth <= self.mMaxDepth {
            let d = (i + off) % 8;
            if !self.Cell(c_x, c_y).Border(d) {
                // we can move this way, since no border
                let new_c_x = c_x + unsafe { neighbor_x[d as usize] };
                let new_c_y = c_y + unsafe { neighbor_y[d as usize] };
                if self.Cell(new_c_x, new_c_y).Wall_get() == 255 {
                    // we have a new cell that has not been visited!
                    let mut new_dir: c_int;
                    // d is the direction relative to the current cell
                    // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                    if d < 4 {
                        new_dir = d + 4;
                    } else {
                        new_dir = d - 4;
                    }

                    // knock down walls
                    self.Cell(c_x, c_y).RemoveWall(d);
                    self.Cell(new_c_x, new_c_y).RemoveWall(new_dir); //DIR_MAX - d);

                    // set path id
                    let node_cur = self.Node(c_x, c_y);
                    let node_new = self.Node(new_c_x, new_c_y);
                    if !node_cur.is_null() {
                        unsafe { (*node_cur).SetPath(d, self.mPathCount) };
                    }
                    if !node_new.is_null() {
                        unsafe { (*node_new).SetPath(new_dir, self.mPathCount) }; //DIR_MAX - d, mPathCount);
                    }

                    // create path between cells
                    let pos_cur = self.GetNodePos(c_x, c_y);
                    let pos_new = self.GetNodePos(new_c_x, new_c_y);
                    if !pos_cur.is_null() && !pos_new.is_null() && !self.mTerrain.is_null() {
                        unsafe {
                            (*self.mTerrain).CreatePath(
                                self.mPathCount,
                                -1,
                                0,
                                self.mPathPoints,
                                (*pos_cur)[0],
                                (*pos_cur)[1],
                                (*pos_new)[0],
                                (*pos_new)[1],
                                self.mPathMinWidth,
                                self.mPathMaxWidth,
                                self.mPathDepth,
                                self.mPathDeviation,
                                self.mPathBreadth,
                            );
                        }
                    }

                    self.mPathCount += 1;

                    // flatten a small spot
                    let mut area = CArea::new();
                    unsafe {
                        let landscape = (*TheRandomMissionManager).GetLandScape();
                        let flat_radius = if !landscape.is_null() {
                            let bounds = (*landscape).GetBounds();
                            if !bounds.is_null() {
                                self.mPathMaxWidth * ((*bounds)[1][0] - (*bounds)[0][0]).abs()
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };

                        area.Init(
                            pos_cur,
                            flat_radius,
                            0.0f,
                            0, /* AT_NONE */
                            0,
                            0,
                        );
                        if !landscape.is_null() {
                            (*landscape).FlattenArea(
                                &mut area,
                                (255.0 * self.mPathDepth) as c_int,
                                false,
                                true,
                                true,
                            );
                        }
                    }

                    // recurse
                    self.PathVisit(new_c_x, new_c_y);
                }
            }
            i += 1;
        }

        self.mDepth -= 1;

        // NOTE: *whoop* hack alert, the first time this is reached, it should be the very last placed node.
        unsafe {
            let manager = TheRandomMissionManager;
            if !manager.is_null() {
                let mission = (*manager).GetMission();
                if !self.mCrossed && !mission.is_null()
                    && (*mission).GetSymmetric()
                    && (*mission).GetBackUpPath()
                {
                    self.mCrossed = true;

                    let directionSet: [[c_int; 3]; 3] = [
                        [7 /* DIR_NW */, 6 /* DIR_W */, 5 /* DIR_SW */],
                        [0 /* DIR_N */, -1, 4 /* DIR_S */],
                        [1 /* DIR_NE */, 2 /* DIR_E */, 3 /* DIR_SE */],
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
                        let d = directionSet[(x_delta + 1) as usize][(y_delta + 1) as usize];
                        let mut new_dir: c_int;
                        // d is the direction relative to the current cell
                        // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                        if d < 4 {
                            new_dir = d + 4;
                        } else {
                            new_dir = d - 4;
                        }

                        //NOTE: Knocking down these walls will cause instances to be created on this new artificial path
                        // Since this path could span more than just the normal 1 cell, these walls being knocked down are not exactly correct... but get the job done

                        // knock down walls
                        self.Cell(c_x, c_y).RemoveWall(d);
                        self.Cell(ncx, ncy).RemoveWall(new_dir); //DIR_MAX - d);

                        // set path id
                        let node_cur = self.Node(c_x, c_y);
                        let node_mirror = self.Node(ncx, ncy);
                        if !node_cur.is_null() {
                            (*node_cur).SetPath(d, self.mPathCount);
                        }
                        if !node_mirror.is_null() {
                            (*node_mirror).SetPath(new_dir, self.mPathCount);
                        } //DIR_MAX - d, mPathCount);

                        // create an artificial path that crosses over to connect the symmetric and non-symmetric map parts
                        let pos_cur = self.GetNodePos(c_x, c_y);
                        let pos_mirror = self.GetNodePos(ncx, ncy);
                        if !pos_cur.is_null() && !pos_mirror.is_null() && !self.mTerrain.is_null() {
                            (*self.mTerrain).CreatePath(
                                self.mPathCount,
                                -1,
                                0,
                                self.mPathPoints,
                                (*pos_cur)[0],
                                (*pos_cur)[1],
                                (*pos_mirror)[0],
                                (*pos_mirror)[1],
                                self.mPathMinWidth,
                                self.mPathMaxWidth,
                                self.mPathDepth,
                                self.mPathDeviation,
                                self.mPathBreadth,
                            );
                        }

                        self.mPathCount += 1;
                    }
                }
            }
        }

        self.PlaceLocation(c_x, c_y);
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
    pub fn FindNodeByName(&self, name: *const core::ffi::c_char) -> *mut CRMNode {
        let mut j: c_int;

        j = self.mNodes.len() as c_int - 1;
        while j >= 0 {
            if !self.mNodes[j as usize].is_null() {
                let node = unsafe { &*self.mNodes[j as usize] };
                let node_name = node.GetName();
                if unsafe { stricmp(name, node_name) } == 0 {
                    return self.mNodes[j as usize];
                }
            }
            j -= 1;
        }
        ptr::null_mut()
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
        bridge_name: *const core::ffi::c_char,
    ) {
        // save river style
        self.mRiverDepth = depth;
        self.mRiverPoints = points;
        self.mRiverMinWidth = minwidth;
        self.mRiverMaxWidth = maxwidth;
        self.mRiverBedDepth = beddepth;
        self.mRiverDeviation = deviation;
        self.mRiverBreadth = breadth;
        self.mRiverBridge._data = bridge_name as *mut core::ffi::c_char;
    }

    pub fn GetRiverPos(&mut self, x: c_int, y: c_int) -> *mut vec3_t {
        self.mRiverPos[0] = (x as f32 + 1.0) / (self.mXNodes as f32 + 2.0);
        self.mRiverPos[1] = (y as f32 + 1.0) / (self.mYNodes as f32 + 2.0);
        &mut self.mRiverPos as *mut vec3_t
    }

    pub fn RiverVisit(&mut self, c_x: c_int, c_y: c_int) {
        // does this cell have any neighbors with all walls intact?
        let mut i: c_int;
        let mut off: c_int;

        // look at neighbors in random order
        unsafe {
            let landscape = (*TheRandomMissionManager).GetLandScape();
            if !landscape.is_null() {
                off = (*landscape).irand(0, 7);
            } else {
                off = 0;
            }
        }

        self.mDepth += 1; // track our depth of recursion

        i = 0;
        while i < 8 && self.mDepth <= self.mMaxDepth {
            let d = (i + off) % 8;
            if !self.Cell(c_x, c_y).Border(d) {
                // we can move this way, since no border
                let new_c_x = c_x + unsafe { neighbor_x[d as usize] };
                let new_c_y = c_y + unsafe { neighbor_y[d as usize] };
                if self.RiverCell(new_c_x, new_c_y).Wall_get() == 255 {
                    // we have a new cell that has not been visited!

                    let mut new_dir: c_int;
                    // d is the direction relative to the current cell
                    // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                    if d < 4 {
                        new_dir = d + 4;
                    } else {
                        new_dir = d - 4;
                    }
                    // knock down walls
                    self.RiverCell(c_x, c_y).RemoveWall(d);
                    self.RiverCell(new_c_x, new_c_y).RemoveWall(new_dir); //DIR_MAX - d);

                    // create river between cells
                    let river_pos_cur = self.GetRiverPos(c_x, c_y);
                    let river_pos_new = self.GetRiverPos(new_c_x, new_c_y);
                    if !river_pos_cur.is_null() && !river_pos_new.is_null() && !self.mTerrain.is_null() {
                        unsafe {
                            (*self.mTerrain).CreatePath(
                                self.mPathCount,
                                -1,
                                0,
                                self.mRiverPoints,
                                (*river_pos_cur)[0],
                                (*river_pos_cur)[1],
                                (*river_pos_new)[0],
                                (*river_pos_new)[1],
                                self.mRiverMinWidth,
                                self.mRiverMaxWidth,
                                self.mRiverBedDepth,
                                self.mRiverDeviation,
                                self.mRiverBreadth,
                            );
                        }
                    }

                    self.mPathCount += 1;

                    // flatten a small spot
                    let mut area = CArea::new();
                    unsafe {
                        let landscape = (*TheRandomMissionManager).GetLandScape();
                        let flat_radius = if !landscape.is_null() {
                            let bounds = (*landscape).GetBounds();
                            if !bounds.is_null() {
                                self.mRiverMinWidth * ((*bounds)[1][0] - (*bounds)[0][0]).abs()
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };

                        area.Init(
                            self.GetRiverPos(c_x, c_y),
                            flat_radius,
                            0.0f,
                            0, /* AT_NONE */
                            0,
                            0,
                        );
                        if !landscape.is_null() {
                            (*landscape).FlattenArea(
                                &mut area,
                                (255.0 * self.mRiverBedDepth) as c_int,
                                false,
                                true,
                                true,
                            );
                        }
                    }

                    // recurse
                    self.RiverVisit(new_c_x, new_c_y);
                }
            }
            i += 2;
        }

        //	--mDepth;
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
        let edge = unsafe {
            let landscape = (*TheRandomMissionManager).GetLandScape();
            if !landscape.is_null() {
                (*landscape).irand(0, 7)
            } else {
                0
            }
        };
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
            7 => {
                cell_x = 0;
                cell_y = 0;
            }
            _ => {}
        }

        self.ClearCells(self.mXNodes + 1, self.mYNodes + 1);

        self.mRiverCount = self.mPathCount;

        // visit the first cell
        self.RiverVisit(cell_x, cell_y);

        self.mRiverCount = self.mPathCount - self.mRiverCount;

        return;
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

            _ => {
                // SYMMETRY_NONE
                // choose starting cell along an edge
                let rand_val = unsafe {
                    let landscape = (*TheRandomMissionManager).GetLandScape();
                    if !landscape.is_null() {
                        (*landscape).irand(0, 7)
                    } else {
                        0
                    }
                };
                match rand_val {
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
                    7 => {
                        // default: no change
                    }
                    _ => {}
                }
            }
        }

        // visit the first cell
        self.PathVisit(cell_x, cell_y);
    }

    // Helper methods for accessing private fields
    fn Node(&mut self, x: c_int, y: c_int) -> *mut CRMNode {
        let idx = (x + y * self.mXNodes) as usize;
        if idx < self.mNodes.len() {
            self.mNodes[idx]
        } else {
            ptr::null_mut()
        }
    }

    fn GetNodePos(&self, x: c_int, y: c_int) -> *mut vec3_t {
        let idx = (x + y * self.mXNodes) as usize;
        if idx < self.mNodes.len() {
            let node = self.mNodes[idx];
            if !node.is_null() {
                unsafe { &mut (*node).mPos as *mut vec3_t }
            } else {
                ptr::null_mut()
            }
        } else {
            ptr::null_mut()
        }
    }

    fn SetNodePos(&mut self, x: c_int, y: c_int, pos: &vec3_t) {
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

    fn Cell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        let idx = (x + y * self.mXNodes) as usize;
        if idx < self.mCells.len() {
            &mut self.mCells[idx]
        } else {
            panic!("Cell index out of bounds");
        }
    }

    fn RiverCell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        let idx = (x + y * (self.mXNodes + 1)) as usize;
        if idx < self.mCells.len() {
            &mut self.mCells[idx]
        } else {
            panic!("RiverCell index out of bounds");
        }
    }
}
