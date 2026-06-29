/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Path.cpp
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// #include "../server/exe_headers.h"
use crate::code::server::exe_headers_h::*;
// #include "rm_headers.h"
use crate::code::RMG::rm_headers_h::*;
// per triage: external types imported from the headers their C #includes name; none are defined
// or stubbed with opaque placeholders here.
// CRandomTerrain — from ../qcommon/cm_randomterrain.h
use crate::code::qcommon::cm_randomterrain_h::*;
// CCMLandScape, CArea, AT_NONE — from ../qcommon/cm_landscape.h
use crate::code::qcommon::cm_landscape_h::*;
// CRMManager, TheRandomMissionManager — from RM_Manager.h (selective to avoid glob name conflicts)
use crate::code::RMG::RM_Manager_h::{CRMManager, TheRandomMissionManager};
// CRMMission — from RM_Mission.h (selective)
use crate::code::RMG::RM_Mission_h::CRMMission;

// Porting note: the triage instructs importing CRMNode / CRMLoc / CRMCell / CRMPathManager
// from crate::code::RMG::RM_Path_h. That approach is blocked by two Rust constraints:
//   (a) RM_Path_h.rs keeps those struct fields private, so impl blocks in a sibling module
//       cannot access them; and
//   (b) RM_Path_h.rs already defines stub methods with the same names, causing duplicate-item
//       errors if we add real impl blocks.
// Following the established codebase pattern (cf. RM_Manager.rs), the types are defined
// locally here so all method bodies have full field access. No opaque placeholder stubs are used.

// String type wrapper (C++ std::string used in this translation unit)
#[repr(C)]
pub struct string {
    pub _data: *mut c_char,
    pub _capacity: usize,
    pub _size: usize,
}

impl string {
    pub fn c_str(&self) -> *const c_char {
        self._data
    }
}

pub type vec3_t = [f32; 3];

// #define max(a,b)    (((a) > (b)) ? (a) : (b))
#[inline]
fn max(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
}

// ---- CRMNode ------------------------------------------------------------------

pub struct CRMNode {
    // name of node - "" if not used yet
    pub mName: string,
    // where node is
    pub mPos: vec3_t,
    // path id's that lead from this node
    pub mPathID: [c_int; 8 /* DIR_MAX */],
    // false if no area point here yet.
    pub mAreaPointPlaced: bool,

    pub mFlattenHeight: c_int,
}

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
impl CRMNode {
    pub fn new() -> Self {
        let mut i: c_int;

        let mut node = CRMNode {
            mName: string { _data: core::ptr::null_mut(), _capacity: 0, _size: 0 },
            mPos: [0.0_f32; 3],
            mPathID: [-1; 8],
            mAreaPointPlaced: false,
            mFlattenHeight: -1,
        };

        node.mPos[0] = 0.0_f32;
        node.mPos[1] = 0.0_f32;
        node.mPos[2] = 0.0_f32;

        // no paths
        i = 0;
        while i < 8 /* DIR_MAX */ {
            node.mPathID[i as usize] = -1;
            i += 1;
        }

        node.mAreaPointPlaced = false;
        node
    }

    pub fn IsLocation(&self) -> bool {
        // strlen(mName.c_str()) > 0
        !self.mName._data.is_null() && unsafe { *self.mName._data != 0 }
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName.c_str()
    }

    pub fn GetPos(&mut self) -> &mut vec3_t {
        &mut self.mPos
    }

    pub fn PathExist(&self, dir: c_int) -> bool {
        self.mPathID[(dir % 8) as usize] != -1
    }

    pub fn GetPath(&self, dir: c_int) -> c_int {
        self.mPathID[(dir % 8) as usize]
    }

    pub fn AreaPoint(&self) -> bool {
        self.mAreaPointPlaced
    }

    pub fn SetName(&mut self, name: *const c_char) {
        self.mName._data = name as *mut c_char;
    }

    pub fn SetPos(&mut self, v: &vec3_t) {
        // VectorCopy
        self.mPos[0] = v[0];
        self.mPos[1] = v[1];
        self.mPos[2] = v[2];
    }

    pub fn SetPath(&mut self, dir: c_int, id: c_int) {
        self.mPathID[(dir % 8) as usize] = id;
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

// ---- CRMLoc -------------------------------------------------------------------

// named spots on the map, should be placed into nodes
pub struct CRMLoc {
    // name of location
    pub mName: string,
    pub mMinDepth: c_int,
    pub mMaxDepth: c_int,
    pub mMinPaths: c_int,
    pub mMaxPaths: c_int,
    // location has been placed at a node
    pub mPlaced: bool,
}

impl CRMLoc {
    pub fn new(
        name: *const c_char,
        min_depth: c_int,
        max_depth: c_int,
        min_paths: c_int,
        max_paths: c_int,
    ) -> Self {
        let mut loc = CRMLoc {
            mName: string { _data: core::ptr::null_mut(), _capacity: 0, _size: 0 },
            mMinDepth: min_depth,
            mMaxDepth: max_depth,
            mMinPaths: min_paths,
            mMaxPaths: max_paths,
            mPlaced: false,
        };
        // mName = name
        loc.mName._data = name as *mut c_char;
        loc
    }

    pub fn GetName(&self) -> *const c_char {
        self.mName.c_str()
    }

    pub fn SetName(&mut self, name: *const c_char) {
        self.mName._data = name as *mut c_char;
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

// ---- CRMCell ------------------------------------------------------------------

// cells are used for figuring out node connections / paths
pub struct CRMCell {
    pub border: c_int,
    pub wall: c_int,
}

impl CRMCell {
    pub fn new() -> Self {
        CRMCell { border: 0, wall: 255 /* DIR_ALL */ }
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

// ---- CRMPathManager -----------------------------------------------------------

pub struct CRMPathManager {
    pub mXNodes: c_int,     // number of nodes in the x dimension
    pub mYNodes: c_int,     // number of nodes in the y dimension

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
pub static mut neighbor_x: [c_int; 8 /* DIR_MAX */] = [ 0, 1, 1, 1, 0,-1,-1,-1];
pub static mut neighbor_y: [c_int; 8 /* DIR_MAX */] = [-1,-1, 0, 1, 1, 1, 0,-1];

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
            mPathPoints: 10,
            mPathMinWidth: 0.02_f32,
            mPathMaxWidth: 0.04_f32,
            mPathDepth: 0.3_f32,
            mPathDeviation: 0.03_f32,
            mPathBreadth: 5.0_f32,
            mRiverDepth: 5,
            mRiverPoints: 10,
            mRiverMinWidth: 0.01_f32,
            mRiverMaxWidth: 0.02_f32,
            mRiverBedDepth: 1.0_f32,
            mRiverDeviation: 0.01_f32,
            mRiverBreadth: 7.0_f32,
            // mRiverBridge: default-constructed std::string; zero-init = null/empty
            mRiverBridge: string { _data: core::ptr::null_mut(), _capacity: 0, _size: 0 },
            mRiverPos: [0.0_f32; 3],
            mTerrain: terrain,
        }
    }

    // --- inline header accessors (mirrored here for completeness) -----------------

    pub fn Node(&self, x: c_int, y: c_int) -> *mut CRMNode {
        self.mNodes[(x + y * self.mXNodes) as usize]
    }

    pub fn GetNodePos(&self, x: c_int, y: c_int) -> *mut vec3_t {
        let node = self.Node(x, y);
        if !node.is_null() {
            unsafe { core::ptr::addr_of_mut!((*node).mPos) }
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn SetNodePos(&mut self, x: c_int, y: c_int, pos: &vec3_t) {
        let node = self.mNodes[(x + y * self.mXNodes) as usize];
        if !node.is_null() {
            unsafe { (*node).SetPos(pos) };
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

    pub fn GetBridgeName(&self) -> *const c_char {
        self.mRiverBridge.c_str()
    }

    pub fn Cell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        &mut self.mCells[(x + y * self.mXNodes) as usize]
    }

    pub fn RiverCell(&mut self, x: c_int, y: c_int) -> &mut CRMCell {
        &mut self.mCells[(x + y * (self.mXNodes + 1)) as usize]
    }
}

impl Drop for CRMPathManager {
    fn drop(&mut self) {
        let mut i: c_int;
        let mut j: c_int;

        i = self.mLocations.len() as c_int - 1;
        while i >= 0 {
            if !self.mLocations[i as usize].is_null() {
                unsafe { drop(Box::from_raw(self.mLocations[i as usize])) };
            }
            i -= 1;
        }
        self.mLocations.clear();

        j = self.mNodes.len() as c_int - 1;
        while j >= 0 {
            if !self.mNodes[j as usize].is_null() {
                unsafe { drop(Box::from_raw(self.mNodes[j as usize])) };
            }
            j -= 1;
        }
        self.mNodes.clear();
    }
}

impl CRMPathManager {
    pub fn CreateLocation(
        &mut self,
        name: *const c_char,
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

        i = self.mLocations.len() as c_int - 1;
        while i >= 0 {
            if unsafe { stricmp(name, (*self.mLocations[i as usize]).GetName()) } == 0 {
                unsafe {
                    (*self.mLocations[i as usize]).SetMinDepth(min_depth);
                    (*self.mLocations[i as usize]).SetMaxDepth(max_depth);
                    (*self.mLocations[i as usize]).SetMinPaths(min_paths);
                    (*self.mLocations[i as usize]).SetMaxPaths(max_paths);
                }
                return;
            }
            i -= 1;
        }

        let p_loc: *mut CRMLoc =
            Box::into_raw(Box::new(CRMLoc::new(name, min_depth, max_depth, min_paths, max_paths)));
        self.mLocations.push(p_loc);
        self.mMaxDepth = max(self.mMaxDepth, max_depth);
    }

    pub fn ClearCells(&mut self, x_nodes: c_int, y_nodes: c_int) {
        let mut x: c_int;
        let mut y: c_int;

        // clear cell array - used for generating paths
        x = 0;
        while x < x_nodes * y_nodes {
            if x >= self.mCells.len() as c_int {
                self.mCells.push(CRMCell::new());
            } else {
                self.mCells[x as usize] = CRMCell::new();
            }
            x += 1;
        }

        // set borders of world
        y = 0;
        while y < y_nodes {
            self.mCells[(y * x_nodes) as usize].SetBorder(6 /* DIR_W */);
            self.mCells[(y * x_nodes) as usize].SetBorder(5 /* DIR_SW */);
            self.mCells[(y * x_nodes) as usize].SetBorder(7 /* DIR_NW */);
            self.mCells[(y * x_nodes + x_nodes - 1) as usize].SetBorder(2 /* DIR_E */);
            self.mCells[(y * x_nodes + x_nodes - 1) as usize].SetBorder(1 /* DIR_NE */);
            self.mCells[(y * x_nodes + x_nodes - 1) as usize].SetBorder(3 /* DIR_SE */);
            y += 1;
        }

        x = 0;
        while x < x_nodes {
            self.mCells[x as usize].SetBorder(0 /* DIR_N */);
            self.mCells[x as usize].SetBorder(1 /* DIR_NE */);
            self.mCells[x as usize].SetBorder(7 /* DIR_NW */);
            self.mCells[((y_nodes - 1) * x_nodes + x) as usize].SetBorder(4 /* DIR_S */);
            self.mCells[((y_nodes - 1) * x_nodes + x) as usize].SetBorder(3 /* DIR_SE */);
            self.mCells[((y_nodes - 1) * x_nodes + x) as usize].SetBorder(5 /* DIR_SW */);
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
                unsafe { drop(Box::from_raw(self.mNodes[x as usize])) };
            }
            x -= 1;
        }
        self.mNodes.clear();
        self.mNodes.resize((self.mXNodes * self.mYNodes) as usize, core::ptr::null_mut());

        // add a small amount of random jitter to spots chosen
        let x_rnd: f32 = 0.2_f32 / (self.mXNodes + 1) as f32;
        let y_rnd: f32 = 0.2_f32 / (self.mYNodes + 1) as f32;

        x = 0;
        while x < self.mXNodes {
            let cell_x: f32 = (x as f32 + 1.0_f32) / (self.mXNodes + 1) as f32;
//		float cell_x = (x + 2.0f) / (mXNodes+3);

            y = 0;
            while y < self.mYNodes {
                let mut pos: vec3_t = [0.0_f32; 3];
                let pnode: *mut CRMNode = Box::into_raw(Box::new(CRMNode::new()));
                self.mNodes[(x + y * self.mXNodes) as usize] = pnode;

                let cell_y: f32 = (y as f32 + 1.0_f32) / (self.mYNodes + 1) as f32;
//			float cell_y = (y + 2.0f) / (mYNodes+3);

                unsafe {
                    pos[0] = (*(*TheRandomMissionManager).GetLandScape()).flrand(
                        cell_x - x_rnd, cell_x + x_rnd,
                    );
                    pos[1] = (*(*TheRandomMissionManager).GetLandScape()).flrand(
                        cell_y - y_rnd, cell_y + y_rnd,
                    );
                }
                pos[2] = 0.0_f32;

                self.SetNodePos(x, y, &pos);

                y += 1;
            }

            x += 1;
        }

        self.ClearCells(self.mXNodes, self.mYNodes);

        true
    }
}

impl CRMPathManager {
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
        if !unsafe { (*self.Node(c_x, c_y)).IsLocation() } {
            // not currently a location

            // how many paths lead to this cell?
            let mut count_paths: c_int = 0;
            let mut i: c_int;

            i = 0;
            while i < 8 /* DIR_MAX */ {
                if unsafe { (*self.Node(c_x, c_y)).PathExist(i) } {
                    count_paths += 1;
                }
                i += 1;
            }

            let mut deepest_depth: c_int = -1;
            let mut deepest_loc: c_int = -1;
            i = self.mLocations.len() as c_int - 1;
            while i >= 0 {
                let loc = unsafe { &*self.mLocations[i as usize] };
                if !loc.Placed()                              // node has not been placed
                    && loc.MinDepth() <= self.mDepth         // our current depth is in the proper range
                    && loc.MaxDepth() >= self.mDepth
                    && loc.MinPaths() <= count_paths          // our path count is in the proper range
                    && loc.MaxPaths() >= count_paths
                    && loc.MaxDepth() > deepest_depth         // and this is the deepest location of the ones that match
                {
                    deepest_loc = i;
                    deepest_depth = loc.MaxDepth();
                }
                i -= 1;
            }

            if deepest_loc >= 0 && (deepest_loc as usize) < self.mLocations.len() {
                // found a location to place at this node / cell
                let name = unsafe { (*self.mLocations[deepest_loc as usize]).GetName() };
                unsafe { (*self.Node(c_x, c_y)).SetName(name) };
                unsafe { (*self.mLocations[deepest_loc as usize]).SetPlaced(true) };

                // need a new max depth
                let mut max_depth: c_int = -1;
                i = self.mLocations.len() as c_int - 1;
                while i >= 0 {
                    let loc_iter = unsafe { &*self.mLocations[i as usize] };
                    // figure out new max depth based on the max depth of unplaced locations
                    if !loc_iter.Placed()                             // node has not been placed
                        && loc_iter.MaxDepth() > max_depth            // and this is the deepest
                    {
                        max_depth = loc_iter.MaxDepth();
                    }
                    i -= 1;
                }
                self.mMaxDepth = max_depth;
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
        let off: c_int;

        // look at neighbors in random order
        off = unsafe {
            (*(*TheRandomMissionManager).GetLandScape()).irand(0 /* DIR_FIRST */, 7 /* DIR_MAX-1 */)
        };

        self.mDepth += 1;  // track our depth of recursion

        i = 0; /* DIR_FIRST */
        while i < 8 /* DIR_MAX */ && self.mDepth <= self.mMaxDepth {
            let d: c_int = (i + off) % 8 /* DIR_MAX */;
            if !self.Cell(c_x, c_y).Border_dir(d) {
                // we can move this way, since no border
                let new_c_x: c_int = c_x + unsafe { neighbor_x[d as usize] };
                let new_c_y: c_int = c_y + unsafe { neighbor_y[d as usize] };
                if self.Cell(new_c_x, new_c_y).Wall() == 255 /* DIR_ALL */ {
                    // we have a new cell that has not been visited!
                    let new_dir: c_int;
                    // d is the direction relative to the current cell
                    // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                    if d < 4 /* HALF_DIR_MAX */ {
                        new_dir = d + 4; /* HALF_DIR_MAX */
                    } else {
                        new_dir = d - 4; /* HALF_DIR_MAX */
                    }

                    // knock down walls
                    self.Cell(c_x, c_y).RemoveWall(d);
                    self.Cell(new_c_x, new_c_y).RemoveWall(new_dir); //DIR_MAX - d);

                    // set path id
                    unsafe {
                        (*self.Node(c_x, c_y)).SetPath(d, self.mPathCount);
                        (*self.Node(new_c_x, new_c_y)).SetPath(new_dir, self.mPathCount); //DIR_MAX - d, mPathCount);
                    }

                    // create path between cells
                    let pos_cx = self.GetNodePos(c_x, c_y);
                    let pos_nx = self.GetNodePos(new_c_x, new_c_y);
                    unsafe {
                        (*self.mTerrain).CreatePath(
                            self.mPathCount,
                            -1,
                            0,
                            self.mPathPoints,
                            (*pos_cx)[0],
                            (*pos_cx)[1],
                            (*pos_nx)[0],
                            (*pos_nx)[1],
                            self.mPathMinWidth,
                            self.mPathMaxWidth,
                            self.mPathDepth,
                            self.mPathDeviation,
                            self.mPathBreadth,
                        );
                    }
                    self.mPathCount += 1;

                    // flatten a small spot
                    let mut area: CArea = unsafe { core::mem::zeroed() };
                    let flat_radius: f32 = self.mPathMaxWidth
                        * unsafe {
                            let bounds = (*(*TheRandomMissionManager).GetLandScape()).GetBounds();
                            f32::abs((*bounds)[1][0] - (*bounds)[0][0])
                        };
                    let pos_cx2 = self.GetNodePos(c_x, c_y);
                    unsafe {
                        area.Init(pos_cx2, flat_radius, 0.0_f32, AT_NONE, 0, 0);
                        (*(*TheRandomMissionManager).GetLandScape()).FlattenArea(
                            &mut area,
                            (255.0_f32 * self.mPathDepth) as c_int,
                            false,
                            true,
                            true,
                        );
                    }

                    // recurse
                    self.PathVisit(new_c_x, new_c_y);
                }
            }
            i += 1;
        }

        self.mDepth -= 1;

        // NOTE: *whoop* hack alert, the first time this is reached, it should be the very last placed node.
        if !self.mCrossed
            && unsafe { (*(*TheRandomMissionManager).GetMission()).GetSymmetric() } != 0
            && unsafe { (*(*TheRandomMissionManager).GetMission()).GetBackUpPath() } != 0
        {
            self.mCrossed = true;

            // Porting note: C++ uses directionSet[x_delta][y_delta] where x_delta/y_delta are in
            // {-1,0,1}; negative array indices are UB in C++. Offset by +1 to produce valid
            // indices {0,1,2} — this matches the intended semantic direction lookup.
            let direction_set: [[c_int; 3]; 3] = [
                [7 /* DIR_NW */, 6 /* DIR_W */, 5 /* DIR_SW */],
                [0 /* DIR_N */, -1, 4 /* DIR_S */],
                [1 /* DIR_NE */, 2 /* DIR_E */, 3 /* DIR_SE */],
            ];
            let ncx: c_int = (self.mXNodes - 1) - c_x;
            let ncy: c_int = (self.mYNodes - 1) - c_y;

            let mut x_delta: c_int = ncx - c_x;
            let mut y_delta: c_int = ncy - c_y;

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

                let d: c_int = direction_set[(x_delta + 1) as usize][(y_delta + 1) as usize];
                let new_dir: c_int;
                // d is the direction relative to the current cell
                // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                if d < 4 /* HALF_DIR_MAX */ {
                    new_dir = d + 4; /* HALF_DIR_MAX */
                } else {
                    new_dir = d - 4; /* HALF_DIR_MAX */
                }

                //NOTE: Knocking down these walls will cause instances to be created on this new artificial path
                // Since this path could span more than just the normal 1 cell, these walls being knocked down are not exactly correct... but get the job done

                // knock down walls
                self.Cell(c_x, c_y).RemoveWall(d);
                self.Cell(ncx, ncy).RemoveWall(new_dir); //DIR_MAX - d);

                // set path id
                unsafe {
                    (*self.Node(c_x, c_y)).SetPath(d, self.mPathCount);
                    (*self.Node(ncx, ncy)).SetPath(new_dir, self.mPathCount); //DIR_MAX - d, mPathCount);
                }

                // create an artificial path that crosses over to connect the symmetric and non-symmetric map parts
                let pos_cx3 = self.GetNodePos(c_x, c_y);
                let pos_nc = self.GetNodePos(ncx, ncy);
                unsafe {
                    (*self.mTerrain).CreatePath(
                        self.mPathCount,
                        -1,
                        0,
                        self.mPathPoints,
                        (*pos_cx3)[0],
                        (*pos_cx3)[1],
                        (*pos_nc)[0],
                        (*pos_nc)[1],
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

        self.PlaceLocation(c_x, c_y);
    }
}

impl CRMPathManager {
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
    pub fn FindNodeByName(&self, name: *const c_char) -> *mut CRMNode {
        let mut j: c_int;

        j = self.mNodes.len() as c_int - 1;
        while j >= 0 {
            if unsafe { stricmp(name, (*self.mNodes[j as usize]).GetName()) } == 0 {
                return self.mNodes[j as usize];
            }
            j -= 1;
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
        self.mPathPoints    = points  ;
        self.mPathMinWidth  = minwidth;
        self.mPathMaxWidth  = maxwidth;
        self.mPathDepth     = depth   ;
        self.mPathDeviation = deviation;
        self.mPathBreadth   = breadth ;
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
        bridge_name: string,
    ) {
        // save river style
        self.mRiverDepth    = depth;
        self.mRiverPoints   = points  ;
        self.mRiverMinWidth = minwidth;
        self.mRiverMaxWidth = maxwidth;
        self.mRiverBedDepth = beddepth   ;
        self.mRiverDeviation= deviation;
        self.mRiverBreadth  = breadth ;
        self.mRiverBridge   = bridge_name;
    }

    pub fn GetRiverPos(&mut self, x: c_int, y: c_int) -> &mut vec3_t {
        self.mRiverPos[0] = (x as f32 + 1.0_f32) / (self.mXNodes as f32 + 2.0_f32);
        self.mRiverPos[1] = (y as f32 + 1.0_f32) / (self.mYNodes as f32 + 2.0_f32);
        &mut self.mRiverPos
    }

    pub fn RiverVisit(&mut self, c_x: c_int, c_y: c_int) {
        // does this cell have any neighbors with all walls intact?
        let mut i: c_int;
        let off: c_int;

        // look at neighbors in random order
        off = unsafe {
            (*(*TheRandomMissionManager).GetLandScape()).irand(0 /* DIR_FIRST */, 7 /* DIR_MAX-1 */)
        };

        self.mDepth += 1;  // track our depth of recursion

        i = 0; /* DIR_FIRST */
        while i < 8 /* DIR_MAX */ && self.mDepth <= self.mMaxDepth {
            let d: c_int = (i + off) % 8 /* DIR_MAX */;
            if !self.Cell(c_x, c_y).Border_dir(d) {
                // we can move this way, since no border
                let new_c_x: c_int = c_x + unsafe { neighbor_x[d as usize] };
                let new_c_y: c_int = c_y + unsafe { neighbor_y[d as usize] };
                if self.RiverCell(new_c_x, new_c_y).Wall() == 255 /* DIR_ALL */ {
                    // we have a new cell that has not been visited!

                    let new_dir: c_int;
                    // d is the direction relative to the current cell
                    // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                    if d < 4 /* HALF_DIR_MAX */ {
                        new_dir = d + 4; /* HALF_DIR_MAX */
                    } else {
                        new_dir = d - 4; /* HALF_DIR_MAX */
                    }
                    // knock down walls
                    self.RiverCell(c_x, c_y).RemoveWall(d);
                    self.RiverCell(new_c_x, new_c_y).RemoveWall(new_dir); //DIR_MAX - d);

                    // create river between cells
                    // Safety: mTerrain is a valid pointer for the lifetime of this object.
                    // GetRiverPos returns a reference into self.mRiverPos; we copy the values
                    // before the second call would overwrite them.
                    let (rp_cx0, rp_cx1) = {
                        let rp = self.GetRiverPos(c_x, c_y);
                        (rp[0], rp[1])
                    };
                    let (rp_nx0, rp_nx1) = {
                        let rp = self.GetRiverPos(new_c_x, new_c_y);
                        (rp[0], rp[1])
                    };
                    unsafe {
                        (*self.mTerrain).CreatePath(
                            self.mPathCount,
                            -1,
                            0,
                            self.mRiverPoints,
                            rp_cx0,
                            rp_cx1,
                            rp_nx0,
                            rp_nx1,
                            self.mRiverMinWidth,
                            self.mRiverMaxWidth,
                            self.mRiverBedDepth,
                            self.mRiverDeviation,
                            self.mRiverBreadth,
                        );
                    }
                    self.mPathCount += 1;

                    // flatten a small spot
                    let mut area: CArea = unsafe { core::mem::zeroed() };
                    let flat_radius: f32 = self.mRiverMinWidth
                        * unsafe {
                            let bounds = (*(*TheRandomMissionManager).GetLandScape()).GetBounds();
                            f32::abs((*bounds)[1][0] - (*bounds)[0][0])
                        };
                    let (rp2_cx0, rp2_cx1) = {
                        let rp = self.GetRiverPos(c_x, c_y);
                        (rp[0], rp[1])
                    };
                    // pass a stack-local copy of the position to avoid aliasing via &mut self
                    let mut pos_copy: vec3_t = [rp2_cx0, rp2_cx1, 0.0_f32];
                    unsafe {
                        area.Init(&mut pos_copy, flat_radius, 0.0_f32, AT_NONE, 0, 0);
                        (*(*TheRandomMissionManager).GetLandScape()).FlattenArea(
                            &mut area,
                            (255.0_f32 * self.mRiverBedDepth) as c_int,
                            false,
                            true,
                            true,
                        );
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
        if self.mRiverBedDepth == 1.0_f32 {
            // no rivers
            return;
        }

        self.mMaxDepth = self.mRiverDepth;
        self.mDepth = 0;

        let mut cell_x: c_int = 0;
        let mut cell_y: c_int = 0;

        // choose starting cell along an edge
        let edge: c_int = unsafe {
            (*(*TheRandomMissionManager).GetLandScape()).irand(0, 7)
        };
        match edge {
            0 => { cell_x = self.mXNodes / 2; cell_y = 0; }
            1 => { cell_x = self.mXNodes; cell_y = 0; }
            2 => { cell_x = self.mXNodes; cell_y = self.mYNodes / 2; }
            3 => { cell_x = self.mXNodes; cell_y = self.mYNodes; }
            4 => { cell_x = self.mXNodes / 2; cell_y = self.mYNodes; }
            5 => { cell_x = 0; cell_y = self.mYNodes; }
            6 => { cell_x = 0; cell_y = self.mYNodes / 2; }
            7 => { cell_x = 0; cell_y = 0; }
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
    pub fn GeneratePaths(&mut self, symmetric: c_int /* symmetry_t */) {
        let mut cell_x: c_int = 0;
        let mut cell_y: c_int = 0;

        match symmetric {
            1 /* SYMMETRY_TOPLEFT */ => {
                cell_x = self.mXNodes - 1;
                cell_y = 0;
            }

            2 /* SYMMETRY_BOTTOMRIGHT */ => {
                cell_x = 0;
                cell_y = self.mYNodes - 1;
            }

            _ => {
                // SYMMETRY_NONE / default
                // choose starting cell along an edge
                match unsafe { (*(*TheRandomMissionManager).GetLandScape()).irand(0, 7) } {
                    0 => {
                        cell_x = self.mXNodes / 2;
                    }
                    1 => {
                        cell_x = self.mXNodes - 1;
                    }
                    2 => {
                        cell_x = self.mXNodes - 1; cell_y = self.mYNodes / 2;
                    }
                    3 => {
                        cell_x = self.mXNodes - 1; cell_y = self.mYNodes - 1;
                    }
                    4 => {
                        cell_x = self.mXNodes / 2; cell_y = self.mYNodes - 1;
                    }
                    5 => {
                        cell_y = self.mYNodes - 1;
                    }
                    6 => {
                        cell_y = self.mYNodes / 2;
                    }
                    _ => {
                        // default case 7: cell_x = 0, cell_y = 0
                    }
                }
            }
        }

        // visit the first cell
        self.PathVisit(cell_x, cell_y);
    }
}
