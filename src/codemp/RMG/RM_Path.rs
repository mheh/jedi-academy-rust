// Anything above this #include will be ignored by the compiler

/************************************************************************************************
 *
 *	Copyright (C) 2001-2002 Raven Software
 *
 *  RM_Path.cpp
 *
 ************************************************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};

use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::RMG::RM_Headers_h::*;
// CArea, areaType_t, CCMLandScape, vec3pair_t — available in C++ through the
// exe_headers.h→qcommon.h→cm_landscape.h include chain.
use crate::codemp::qcommon::cm_landscape_h::*;

// PORT NOTE: CRMNode, CRMLoc, CRMCell, CRMPathManager, and their inline methods are defined
// in RM_Path.h, ported to RM_Path_h.rs with all fields pub.  All fields are accessible
// here via the glob import from RM_Headers_h.  This file adds only the non-inline methods
// (constructor, destructor, and the larger functions) that the .cpp defines.
// External types CRMManager, CCMLandScape, CRandomTerrain, CRMMission, CArea, vec3_t,
// TheRandomMissionManager, symmetry_t, DIR_*, HALF_DIR_MAX, VectorCopy, AT_NONE, etc.
// are all trusted to arrive via the glob imports above.

extern "C" {
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Com_Printf(msg: *const c_char, ...);
}

// #define max(a,b)    (((a) > (b)) ? (a) : (b))
#[inline]
fn max(a: c_int, b: c_int) -> c_int {
    if a > b { a } else { b }
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
            mName: String::new(),
            mFlattenHeight: 0,
            mPos: [0.0; 3],
            mPathID: [0; 8],
            mAreaPointPlaced: false,
        };

        node.mFlattenHeight = -1;

        node.mPos[0] = 0 as f32;
        node.mPos[1] = 0 as f32;
        node.mPos[2] = 0 as f32;

        // no paths
        i = 0;
        while i < DIR_MAX {
            node.mPathID[i as usize] = -1;
            i += 1;
        }

        node.mAreaPointPlaced = false;

        node
    }
}

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
    // CRMPathManager(CRandomTerrain* terrain)
    // : mXNodes(0), mYNodes(0), mPathCount(0), mRiverCount(0), mMaxDepth(0), mDepth(0),
    //   mPathPoints(10), mPathMinWidth(0.02f), mPathMaxWidth(0.04f), mPathDepth(0.3f),
    //   mPathDeviation(0.03f), mPathBreadth(5),
    //   mRiverDepth(5), mRiverPoints(10), mRiverMinWidth(0.01f), mRiverMaxWidth(0.02f),
    //   mRiverBedDepth(1), mRiverDeviation(0.01f), mRiverBreadth(7),
    //   mTerrain(terrain), mCrossed(false)
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
            mPathMinWidth: 0.02_f32,
            mPathMaxWidth: 0.04_f32,
            mPathDepth: 0.3_f32,
            mPathDeviation: 0.03_f32,
            mPathBreadth: 5_f32,
            mRiverDepth: 5,
            mRiverPoints: 10,
            mRiverMinWidth: 0.01_f32,
            mRiverMaxWidth: 0.02_f32,
            mRiverBedDepth: 1_f32,
            mRiverDeviation: 0.01_f32,
            mRiverBreadth: 7_f32,
            mTerrain: terrain,
            mCrossed: false,
            mRiverBridge: String::new(),
            mRiverPos: [0.0; 3],
        }
    }

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
            unsafe {
                if stricmp(name, (*self.mLocations[i as usize]).GetName()) == 0 {
                    (*self.mLocations[i as usize]).SetMinDepth(min_depth);
                    (*self.mLocations[i as usize]).SetMaxDepth(max_depth);
                    (*self.mLocations[i as usize]).SetMinPaths(min_paths);
                    (*self.mLocations[i as usize]).SetMaxPaths(max_paths);
                    return;
                }
            }
            i -= 1;
        }

        let p_loc = unsafe {
            Box::into_raw(Box::new(CRMLoc::new(name, min_depth, max_depth, min_paths, max_paths)))
        };
        self.mLocations.push(p_loc);
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
            self.mCells[(y * x_nodes) as usize].SetBorder(DIR_W);
            self.mCells[(y * x_nodes) as usize].SetBorder(DIR_SW);
            self.mCells[(y * x_nodes) as usize].SetBorder(DIR_NW);
            self.mCells[(y * x_nodes + x_nodes - 1) as usize].SetBorder(DIR_E);
            self.mCells[(y * x_nodes + x_nodes - 1) as usize].SetBorder(DIR_NE);
            self.mCells[(y * x_nodes + x_nodes - 1) as usize].SetBorder(DIR_SE);
            y += 1;
        }

        x = 0;
        while x < x_nodes {
            self.mCells[x as usize].SetBorder(DIR_N);
            self.mCells[x as usize].SetBorder(DIR_NE);
            self.mCells[x as usize].SetBorder(DIR_NW);
            self.mCells[((y_nodes - 1) * x_nodes + x) as usize].SetBorder(DIR_S);
            self.mCells[((y_nodes - 1) * x_nodes + x) as usize].SetBorder(DIR_SE);
            self.mCells[((y_nodes - 1) * x_nodes + x) as usize].SetBorder(DIR_SW);
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
                unsafe { let _ = Box::from_raw(self.mNodes[x as usize]); }
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
            let cell_x: f32 = (x + 1) as f32 / (self.mXNodes + 1) as f32;
            //		float cell_x = (x + 2.0f) / (mXNodes+3);

            y = 0;
            while y < self.mYNodes {
                let pos: vec3_t;
                let pnode: *mut CRMNode = Box::into_raw(Box::new(CRMNode::new()));
                self.mNodes[(x + y * self.mXNodes) as usize] = pnode;

                let cell_y: f32 = (y + 1) as f32 / (self.mYNodes + 1) as f32;
                //			float cell_y = (y + 2.0f) / (mYNodes+3);

                unsafe {
                    let landscape = (*TheRandomMissionManager).GetLandScape();
                    let px = (*landscape).flrand(cell_x - x_rnd, cell_x + x_rnd);
                    let py = (*landscape).flrand(cell_y - y_rnd, cell_y + y_rnd);
                    pos = [px, py, 0.0_f32];
                    self.SetNodePos(x, y, &pos);
                }

                y += 1;
            }
            x += 1;
        }

        self.ClearCells(self.mXNodes, self.mYNodes);

        return true;
    }
}

// neighbor offsets - easy way to turn a direction into the array position for a neighboring cell or node
// int CRMPathManager::neighbor_x[DIR_MAX] = { 0, 1, 1, 1, 0,-1,-1,-1};
// int CRMPathManager::neighbor_y[DIR_MAX] = {-1,-1, 0, 1, 1, 1, 0,-1};
// PORT NOTE: These shadow the placeholder neighbor_x/neighbor_y statics from RM_Path_h with
// the actual initialized values defined in this .cpp.
static neighbor_x: [c_int; 8] = [ 0, 1, 1, 1, 0,-1,-1,-1];
static neighbor_y: [c_int; 8] = [-1,-1, 0, 1, 1, 1, 0,-1];

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
        unsafe {
            if !(*self.Node(c_x, c_y)).IsLocation() {
                // not currently a location

                // how many paths lead to this cell?
                let mut count_paths: c_int = 0;
                let mut i: c_int;

                i = 0;
                while i < DIR_MAX {
                    if (*self.Node(c_x, c_y)).PathExist(i) != 0.0 {
                        count_paths += 1;
                    }
                    i += 1;
                }

                let mut deepest_depth: c_int = -1;
                let mut deepest_loc: c_int = -1;
                i = self.mLocations.len() as c_int - 1;
                while i >= 0 {
                    if !(*self.mLocations[i as usize]).Placed()                   // node has not been placed
                        && (*self.mLocations[i as usize]).MinDepth() <= self.mDepth    // our current depth is in the proper range
                        && (*self.mLocations[i as usize]).MaxDepth() >= self.mDepth
                        && (*self.mLocations[i as usize]).MinPaths() <= count_paths // our path count is in the proper range
                        && (*self.mLocations[i as usize]).MaxPaths() >= count_paths
                        && (*self.mLocations[i as usize]).MaxDepth() > deepest_depth   // and this is the deepest location of the ones that match
                    {
                        deepest_loc = i;
                        deepest_depth = (*self.mLocations[i as usize]).MaxDepth();
                    }
                    i -= 1;
                }

                if deepest_loc >= 0 && deepest_loc < self.mLocations.len() as c_int {
                    // found a location to place at this node / cell
                    let name: *const c_char = (*self.mLocations[deepest_loc as usize]).GetName();
                    (*self.Node(c_x, c_y)).SetName(name);
                    (*self.mLocations[deepest_loc as usize]).SetPlaced(true);

                    // need a new max depth
                    let mut max_depth: c_int = -1;
                    i = self.mLocations.len() as c_int - 1;
                    while i >= 0 {
                        // figure out new max depth based on the max depth of unplaced locations
                        if !(*self.mLocations[i as usize]).Placed()             // node has not been placed
                            && (*self.mLocations[i as usize]).MaxDepth() > max_depth   // and this is the deepest
                        {
                            max_depth = (*self.mLocations[i as usize]).MaxDepth();
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
        let off: c_int;

        // look at neighbors in random order
        off = unsafe { (*(*TheRandomMissionManager).GetLandScape()).irand(DIR_FIRST, DIR_MAX - 1) };

        self.mDepth += 1; // track our depth of recursion

        i = DIR_FIRST;
        while i < DIR_MAX && self.mDepth <= self.mMaxDepth {
            let d: c_int = (i + off) % DIR_MAX;
            if !self.Cell(c_x, c_y).Border_dir(d) {
                // we can move this way, since no border
                let new_c_x: c_int = c_x + neighbor_x[d as usize];
                let new_c_y: c_int = c_y + neighbor_y[d as usize];
                if self.Cell(new_c_x, new_c_y).Wall() == DIR_ALL {
                    // we have a new cell that has not been visited!
                    let new_dir: c_int;
                    // d is the direction relative to the current cell
                    // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                    if d < HALF_DIR_MAX {
                        new_dir = d + HALF_DIR_MAX;
                    } else {
                        new_dir = d - HALF_DIR_MAX;
                    }

                    // knock down walls
                    self.Cell(c_x, c_y).RemoveWall(d);
                    self.Cell(new_c_x, new_c_y).RemoveWall(new_dir); //DIR_MAX - d);

                    // set path id
                    let pc = self.mPathCount;
                    unsafe {
                        (*self.Node(c_x, c_y)).SetPath(d, pc);
                        (*self.Node(new_c_x, new_c_y)).SetPath(new_dir, pc); //DIR_MAX - d, mPathCount);
                    }

                    // create path between cells
                    let pos1: vec3_t = unsafe { *self.GetNodePos(c_x, c_y) };
                    let pos2: vec3_t = unsafe { *self.GetNodePos(new_c_x, new_c_y) };
                    unsafe {
                        (*self.mTerrain).CreatePath(
                            self.mPathCount,
                            -1,
                            0,
                            self.mPathPoints,
                            pos1[0],
                            pos1[1],
                            pos2[0],
                            pos2[1],
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
                    let flat_radius: f32;
                    unsafe {
                        let landscape = (*TheRandomMissionManager).GetLandScape();
                        let bounds = (*landscape).GetBounds();
                        flat_radius = self.mPathMaxWidth
                            * ((*bounds)[1][0] - (*bounds)[0][0]).abs();
                        let npos: vec3_t = *self.GetNodePos(c_x, c_y);
                        area.Init(&npos as *const vec3_t, flat_radius, 0.0_f32, areaType_t::AT_NONE as c_int, 0, 0);
                        (*landscape).FlattenArea(&mut area as *mut CArea, (255_f32 * self.mPathDepth) as c_int, false, true, true);
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
            && unsafe { (*(*TheRandomMissionManager).GetMission()).GetSymmetric() }
            && unsafe { (*(*TheRandomMissionManager).GetMission()).GetBackUpPath() }
        {
            self.mCrossed = true;

            let directionSet: [[c_int; 3]; 3] = [
                [DIR_NW, DIR_W, DIR_SW],
                [DIR_N,  -1,   DIR_S ],
                [DIR_NE, DIR_E, DIR_SE],
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
                // PORT NOTE: C++ accesses directionSet[x_delta][y_delta] where x_delta/y_delta
                // can be -1, which is undefined behaviour.  The intended mapping offsets by 1:
                // directionSet[x_delta+1][y_delta+1].
                let d: c_int = directionSet[(x_delta + 1) as usize][(y_delta + 1) as usize];
                let new_dir: c_int;
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
                self.Cell(ncx, ncy).RemoveWall(new_dir); //DIR_MAX - d);

                // set path id
                let pc = self.mPathCount;
                unsafe {
                    (*self.Node(c_x, c_y)).SetPath(d, pc);
                    (*self.Node(ncx, ncy)).SetPath(new_dir, pc); //DIR_MAX - d, mPathCount);
                }

                // create an artificial path that crosses over to connect the symmetric and non-symmetric map parts
                let pos1: vec3_t = unsafe { *self.GetNodePos(c_x, c_y) };
                let pos2: vec3_t = unsafe { *self.GetNodePos(ncx, ncy) };
                unsafe {
                    (*self.mTerrain).CreatePath(
                        self.mPathCount,
                        -1,
                        0,
                        self.mPathPoints,
                        pos1[0],
                        pos1[1],
                        pos2[0],
                        pos2[1],
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
            unsafe {
                if stricmp(name, (*self.mNodes[j as usize]).GetName()) == 0 {
                    return self.mNodes[j as usize];
                }
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
        self.mPathPoints    = points   ;
        self.mPathMinWidth  = minwidth ;
        self.mPathMaxWidth  = maxwidth ;
        self.mPathDepth     = depth    ;
        self.mPathDeviation = deviation;
        self.mPathBreadth   = breadth  ;
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
        bridge_name: String,
    ) {
        // save river style
        self.mRiverDepth     = depth    ;
        self.mRiverPoints    = points   ;
        self.mRiverMinWidth  = minwidth ;
        self.mRiverMaxWidth  = maxwidth ;
        self.mRiverBedDepth  = beddepth ;
        self.mRiverDeviation = deviation;
        self.mRiverBreadth   = breadth  ;
        self.mRiverBridge    = bridge_name;
    }

    pub fn GetRiverPos(&mut self, x: c_int, y: c_int) -> &mut vec3_t {
        self.mRiverPos[0] = (x + 1) as f32 / (self.mXNodes + 2) as f32;
        self.mRiverPos[1] = (y + 1) as f32 / (self.mYNodes + 2) as f32;
        &mut self.mRiverPos
    }

    pub fn RiverVisit(&mut self, c_x: c_int, c_y: c_int) {
        // does this cell have any neighbors with all walls intact?
        let mut i: c_int;
        let off: c_int;

        // look at neighbors in random order
        off = unsafe { (*(*TheRandomMissionManager).GetLandScape()).irand(DIR_FIRST, DIR_MAX - 1) };

        self.mDepth += 1; // track our depth of recursion

        i = DIR_FIRST;
        while i < DIR_MAX && self.mDepth <= self.mMaxDepth {
            let d: c_int = (i + off) % DIR_MAX;
            if !self.Cell(c_x, c_y).Border_dir(d) {
                // we can move this way, since no border
                let new_c_x: c_int = c_x + neighbor_x[d as usize];
                let new_c_y: c_int = c_y + neighbor_y[d as usize];
                if self.RiverCell(new_c_x, new_c_y).Wall() == DIR_ALL {
                    // we have a new cell that has not been visited!

                    let new_dir: c_int;
                    // d is the direction relative to the current cell
                    // new_dir is the direction relative to the next cell (N becomes S, NE becomes SW, etc...)
                    if d < HALF_DIR_MAX {
                        new_dir = d + HALF_DIR_MAX;
                    } else {
                        new_dir = d - HALF_DIR_MAX;
                    }
                    // knock down walls
                    self.RiverCell(c_x, c_y).RemoveWall(d);
                    self.RiverCell(new_c_x, new_c_y).RemoveWall(new_dir); //DIR_MAX - d);

                    // create river between cells
                    let rpos1: vec3_t = *self.GetRiverPos(c_x, c_y);
                    let rpos2: vec3_t = *self.GetRiverPos(new_c_x, new_c_y);
                    unsafe {
                        (*self.mTerrain).CreatePath(
                            self.mPathCount,
                            -1,
                            0,
                            self.mRiverPoints,
                            rpos1[0],
                            rpos1[1],
                            rpos2[0],
                            rpos2[1],
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
                    unsafe {
                        let landscape = (*TheRandomMissionManager).GetLandScape();
                        let bounds = (*landscape).GetBounds();
                        let flat_radius: f32 = self.mRiverMinWidth
                            * ((*bounds)[1][0] - (*bounds)[0][0]).abs();
                        let npos: vec3_t = *self.GetRiverPos(c_x, c_y);
                        area.Init(&npos as *const vec3_t, flat_radius, 0.0_f32, areaType_t::AT_NONE as c_int, 0, 0);
                        (*landscape).FlattenArea(&mut area as *mut CArea, (255_f32 * self.mRiverBedDepth) as c_int, false, true, true);
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
        if self.mRiverBedDepth == 1_f32 {
            // no rivers
            return;
        }

        self.mMaxDepth = self.mRiverDepth;
        self.mDepth = 0;

        let mut cell_x: c_int = 0;
        let mut cell_y: c_int = 0;

        // choose starting cell along an edge
        let edge: c_int = unsafe { (*(*TheRandomMissionManager).GetLandScape()).irand(0, 7) };
        match edge {
            0 => {
                cell_x = self.mXNodes / 2; cell_y = 0;
            }
            1 => {
                cell_x = self.mXNodes; cell_y = 0;
            }
            2 => {
                cell_x = self.mXNodes; cell_y = self.mYNodes / 2;
            }
            3 => {
                cell_x = self.mXNodes; cell_y = self.mYNodes;
            }
            4 => {
                cell_x = self.mXNodes / 2; cell_y = self.mYNodes;
            }
            5 => {
                cell_x = 0; cell_y = self.mYNodes;
            }
            6 => {
                cell_x = 0; cell_y = self.mYNodes / 2;
            }
            7 => {
                cell_x = 0; cell_y = 0;
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
        let mut cell_x: c_int = 0;
        let mut cell_y: c_int = 0;

        match symmetric {
            symmetry_t::SYMMETRY_TOPLEFT => {
                cell_x = self.mXNodes - 1;
                cell_y = 0;
            }

            symmetry_t::SYMMETRY_BOTTOMRIGHT => {
                cell_x = 0;
                cell_y = self.mYNodes - 1;
            }

            // default:
            _ => {
                // SYMMETRY_NONE — choose starting cell along an edge
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
                        // case 7 or default: both remain 0
                    }
                }
            }
        }

        // visit the first cell
        self.PathVisit(cell_x, cell_y);
    }
}

impl Drop for CRMPathManager {
    // ~CRMPathManager()
    fn drop(&mut self) {
        let mut i: c_int;
        let mut j: c_int;

        i = self.mLocations.len() as c_int - 1;
        while i >= 0 {
            if !self.mLocations[i as usize].is_null() {
                unsafe { let _ = Box::from_raw(self.mLocations[i as usize]); }
            }
            i -= 1;
        }
        self.mLocations.clear();

        j = self.mNodes.len() as c_int - 1;
        while j >= 0 {
            if !self.mNodes[j as usize].is_null() {
                unsafe { let _ = Box::from_raw(self.mNodes[j as usize]); }
            }
            j -= 1;
        }
        self.mNodes.clear();
    }
}
