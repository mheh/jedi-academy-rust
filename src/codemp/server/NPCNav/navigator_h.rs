#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

// Defines
pub const __NEWCOLLECT: i32 = 1;

pub const _HARD_CONNECT: i32 = 1;

// Node flags
pub const NF_ANY: i32 = 0;
// #define	NF_CLEAR_LOS	0x00000001
pub const NF_CLEAR_PATH: i32 = 0x00000002;
pub const NF_RECALC: i32 = 0x00000004;

// Edge flags
pub const EFLAG_NONE: i32 = 0;
pub const EFLAG_BLOCKED: i32 = 0x00000001;
pub const EFLAG_FAILED: i32 = 0x00000002;

// Miscellaneous defines
pub const NODE_NONE: i32 = -1;
pub const NAV_HEADER_ID: [u8; 4] = [b'J', b'N', b'V', b'5'];
pub const NODE_HEADER_ID: [u8; 4] = [b'N', b'O', b'D', b'E'];

// Type aliases for STL containers
pub type EdgeMultimap = std::collections::BTreeMap<i32, i32>;
pub type EdgeMultimapIt = (); // Placeholder for iterator type

// Stubs for external types from server.h and q_shared.h
pub type vec3_t = [f32; 3];
pub type qboolean = i32;
pub type BYTE = u8;
pub type fileHandle_t = i32;

#[repr(C)]
pub struct sharedEntity_t {
    // Stub: actual fields defined in q_shared.h
    _dummy: c_int,
}

#[repr(C)]
pub struct failedEdge_t {
    // Stub: actual fields to be defined
    _dummy: c_int,
}

// External function stubs
extern "C" {
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
}

/*
-------------------------
CEdge
-------------------------
*/

pub struct CEdge {
    pub m_first: i32,
    pub m_second: i32,
    pub m_cost: i32,
}

impl CEdge {
    pub fn new() -> Self {
        CEdge {
            m_first: 0,
            m_second: 0,
            m_cost: 0,
        }
    }

    pub fn with_values(first: i32, second: i32, cost: i32) -> Self {
        CEdge {
            m_first: first,
            m_second: second,
            m_cost: cost,
        }
    }
}

/*
-------------------------
CNode
-------------------------
*/

#[repr(C)]
struct edge_t {
    ID: i32,
    cost: i32,
    flags: BYTE,
}

pub struct CNode {
    m_position: vec3_t,
    m_flags: i32,
    m_radius: i32,
    m_ID: i32,

    m_edges: Vec<edge_t>,

    m_ranks: *mut i32,
    m_numEdges: i32,
}

impl CNode {
    pub fn new() -> Self {
        CNode {
            m_position: [0.0; 3],
            m_flags: 0,
            m_radius: 0,
            m_ID: 0,
            m_edges: Vec::new(),
            m_ranks: std::ptr::null_mut(),
            m_numEdges: 0,
        }
    }

    pub fn Create(position: vec3_t, flags: i32, radius: i32, ID: i32) -> *mut CNode {
        // Stub: actual implementation would allocate and initialize
        std::ptr::null_mut()
    }

    pub fn Create_default() -> *mut CNode {
        // Stub: actual implementation would allocate default CNode
        std::ptr::null_mut()
    }

    pub fn AddEdge(&mut self, ID: i32, cost: i32, flags: i32) {
        // Stub: actual implementation would add edge to m_edges
    }

    pub fn AddRank(&mut self, ID: i32, rank: i32) {
        // Stub: actual implementation would manage ranks
    }

    pub fn Draw(&mut self, radius: qboolean) {
        // Stub: actual implementation would draw node
    }

    pub fn GetID(&self) -> i32 {
        self.m_ID
    }

    pub fn GetPosition(&self, position: *mut vec3_t) {
        if !position.is_null() {
            unsafe {
                VectorCopy(self.m_position.as_ptr(), (*position).as_mut_ptr());
            }
        }
    }

    pub fn GetNumEdges(&self) -> i32 {
        self.m_numEdges
    }

    pub fn GetEdgeNumToNode(&mut self, ID: i32) -> i32 {
        // Stub: actual implementation would search edges
        0
    }

    pub fn GetEdge(&self, edgeNum: i32) -> i32 {
        // Stub: actual implementation would return edge ID
        0
    }

    pub fn GetEdgeCost(&self, edgeNum: i32) -> i32 {
        // Stub: actual implementation would return cost
        0
    }

    pub fn GetEdgeFlags(&self, edgeNum: i32) -> BYTE {
        // Stub: actual implementation would return flags
        0
    }

    pub fn SetEdgeFlags(&mut self, edgeNum: i32, newFlags: i32) {
        // Stub: actual implementation would set flags
    }

    pub fn GetRadius(&self) -> i32 {
        self.m_radius
    }

    pub fn InitRanks(&mut self, size: i32) {
        // Stub: actual implementation would initialize ranks array
    }

    pub fn GetRank(&mut self, ID: i32) -> i32 {
        // Stub: actual implementation would return rank
        0
    }

    pub fn GetFlags(&self) -> i32 {
        self.m_flags
    }

    pub fn AddFlag(&mut self, newFlag: i32) {
        self.m_flags |= newFlag;
    }

    pub fn RemoveFlag(&mut self, oldFlag: i32) {
        self.m_flags &= !oldFlag;
    }

    pub fn Save(&mut self, numNodes: i32, file: fileHandle_t) -> i32 {
        // Stub: actual implementation would save node
        0
    }

    pub fn Load(&mut self, numNodes: i32, file: fileHandle_t) -> i32 {
        // Stub: actual implementation would load node
        0
    }
}

/*
-------------------------
CNavigator
-------------------------
*/
pub const MAX_FAILED_EDGES: usize = 32;

pub struct CNavigator {
    m_nodes: Vec<*mut CNode>,
    m_edgeLookupMap: EdgeMultimap,

    failedEdges: [failedEdge_t; MAX_FAILED_EDGES],

    m_ranks: *mut i32,
    m_numEdges: i32,

    pub pathsCalculated: qboolean,
}

impl CNavigator {
    pub fn new() -> Self {
        CNavigator {
            m_nodes: Vec::new(),
            m_edgeLookupMap: std::collections::BTreeMap::new(),
            failedEdges: [failedEdge_t { _dummy: 0 }; MAX_FAILED_EDGES],
            m_ranks: std::ptr::null_mut(),
            m_numEdges: 0,
            pathsCalculated: 0,
        }
    }

    pub fn Init(&mut self) {
        // Stub: actual implementation would initialize navigator
    }

    pub fn Free(&mut self) {
        // Stub: actual implementation would free resources
    }

    pub fn Load(&mut self, filename: *const i8, checksum: i32) -> bool {
        // Stub: actual implementation would load from file
        false
    }

    pub fn Save(&mut self, filename: *const i8, checksum: i32) -> bool {
        // Stub: actual implementation would save to file
        false
    }

    pub fn AddRawPoint(&mut self, point: vec3_t, flags: i32, radius: i32) -> i32 {
        // Stub: actual implementation would add point
        0
    }

    pub fn CalculatePaths(&mut self, recalc: qboolean) {
        // Stub: actual implementation would calculate paths
    }

    pub fn HardConnect(&mut self, first: i32, second: i32) {
        // Stub: actual implementation would hard-connect nodes
    }

    pub fn ShowNodes(&mut self) {
        // Stub: actual implementation would show nodes
    }

    pub fn ShowEdges(&mut self) {
        // Stub: actual implementation would show edges
    }

    pub fn ShowPath(&mut self, start: i32, end: i32) {
        // Stub: actual implementation would show path
    }

    pub fn GetNearestNode(&mut self, ent: *mut sharedEntity_t, lastID: i32, flags: i32, targetID: i32) -> i32 {
        // Stub: actual implementation would find nearest node
        0
    }

    pub fn GetBestNode(&mut self, startID: i32, endID: i32, rejectID: i32) -> i32 {
        // Stub: actual implementation would find best node
        0
    }

    pub fn GetNodePosition(&mut self, nodeID: i32, out: *mut vec3_t) -> i32 {
        // Stub: actual implementation would get position
        0
    }

    pub fn GetNodeNumEdges(&mut self, nodeID: i32) -> i32 {
        // Stub: actual implementation would get edge count
        0
    }

    pub fn GetNodeEdge(&mut self, nodeID: i32, edge: i32) -> i32 {
        // Stub: actual implementation would get edge
        0
    }

    pub fn GetNodeLeadDistance(&mut self, nodeID: i32) -> f32 {
        // Stub: actual implementation would get lead distance
        0.0
    }

    pub fn GetNumNodes(&self) -> usize {
        self.m_nodes.len()
    }

    pub fn Connected(&mut self, startID: i32, endID: i32) -> bool {
        // Stub: actual implementation would check connectivity
        false
    }

    pub fn GetPathCost(&mut self, startID: i32, endID: i32) -> u32 {
        // Stub: actual implementation would calculate path cost
        0
    }

    pub fn GetEdgeCost(&mut self, startID: i32, endID: i32) -> u32 {
        // Stub: actual implementation would get edge cost
        0
    }

    pub fn GetProjectedNode(&mut self, origin: vec3_t, nodeID: i32) -> i32 {
        // Stub: actual implementation would get projected node
        0
    }

    // MCG Added BEGIN
    pub fn CheckFailedNodes(&mut self, ent: *mut sharedEntity_t) {
        // Stub: actual implementation would check failed nodes
    }

    pub fn AddFailedNode(&mut self, ent: *mut sharedEntity_t, nodeID: i32) {
        // Stub: actual implementation would add failed node
    }

    pub fn NodeFailed(&mut self, ent: *mut sharedEntity_t, nodeID: i32) -> qboolean {
        // Stub: actual implementation would check if node failed
        0
    }

    pub fn NodesAreNeighbors(&mut self, startID: i32, endID: i32) -> qboolean {
        // Stub: actual implementation would check neighbors
        0
    }

    pub fn ClearFailedEdge(&mut self, failedEdge: *mut failedEdge_t) {
        // Stub: actual implementation would clear failed edge
    }

    pub fn ClearAllFailedEdges(&mut self) {
        // Stub: actual implementation would clear all failed edges
    }

    pub fn EdgeFailed(&mut self, startID: i32, endID: i32) -> i32 {
        // Stub: actual implementation would check if edge failed
        0
    }

    pub fn AddFailedEdge(&mut self, entID: i32, startID: i32, endID: i32) {
        // Stub: actual implementation would add failed edge
    }

    pub fn CheckFailedEdge(&mut self, failedEdge: *mut failedEdge_t) -> qboolean {
        // Stub: actual implementation would check failed edge
        0
    }

    pub fn CheckAllFailedEdges(&mut self) {
        // Stub: actual implementation would check all failed edges
    }

    pub fn RouteBlocked(&mut self, startID: i32, testEdgeID: i32, endID: i32, rejectRank: i32) -> qboolean {
        // Stub: actual implementation would check if route blocked
        0
    }

    pub fn GetBestNodeAltRoute_with_cost(&mut self, startID: i32, endID: i32, pathCost: *mut i32, rejectID: i32) -> i32 {
        // Stub: actual implementation would get best node alt route with cost
        0
    }

    pub fn GetBestNodeAltRoute(&mut self, startID: i32, endID: i32, rejectID: i32) -> i32 {
        // Stub: actual implementation would get best node alt route
        0
    }

    pub fn GetBestPathBetweenEnts(&mut self, ent: *mut sharedEntity_t, goal: *mut sharedEntity_t, flags: i32) -> i32 {
        // Stub: actual implementation would get best path between entities
        0
    }

    pub fn GetNodeRadius(&mut self, nodeID: i32) -> i32 {
        // Stub: actual implementation would get node radius
        0
    }

    pub fn CheckBlockedEdges(&mut self) {
        // Stub: actual implementation would check blocked edges
    }

    pub fn ClearCheckedNodes(&mut self) {
        // Stub: actual implementation would clear checked nodes
    }

    pub fn CheckedNode(&mut self, wayPoint: i32, ent: i32) -> BYTE {
        // Stub: actual implementation would get checked node value
        0
    }

    pub fn SetCheckedNode(&mut self, wayPoint: i32, ent: i32, value: BYTE) {
        // Stub: actual implementation would set checked node value
    }

    pub fn FlagAllNodes(&mut self, newFlag: i32) {
        // Stub: actual implementation would flag all nodes
    }
    // MCG Added END

    fn TestNodePath(&mut self, ent: *mut sharedEntity_t, okToHitEntNum: i32, position: vec3_t, includeEnts: qboolean) -> i32 {
        // Stub: protected method
        0
    }

    fn TestNodeLOS(&mut self, ent: *mut sharedEntity_t, position: vec3_t) -> i32 {
        // Stub: protected method
        0
    }

    fn TestBestFirst(&mut self, ent: *mut sharedEntity_t, lastID: i32, flags: i32) -> i32 {
        // Stub: protected method
        0
    }

    fn CollectNearestNodes(&mut self, origin: vec3_t, radius: i32, maxCollect: i32) -> i32 {
        // Stub: protected method
        // Original uses nodeChain_l which is a list<nodeList_t>
        0
    }

    fn GetChar(&mut self, file: fileHandle_t) -> i8 {
        // Stub: protected method
        0
    }

    fn GetInt(&mut self, file: fileHandle_t) -> i32 {
        // Stub: protected method
        0
    }

    fn GetFloat(&mut self, file: fileHandle_t) -> f32 {
        // Stub: protected method
        0.0
    }

    fn GetLong(&mut self, file: fileHandle_t) -> i64 {
        // Stub: protected method
        0
    }

    fn SetEdgeCost(&mut self, ID1: i32, ID2: i32, cost: i32) {
        // Stub: protected method
    }

    fn GetEdgeCost_private(&mut self, first: *mut CNode, second: *mut CNode) -> i32 {
        // Stub: protected method
        0
    }

    fn AddNodeEdges(&mut self, node: *mut CNode, addDist: i32, edgeList: &mut Vec<CEdge>, checkedNodes: *mut bool) {
        // Stub: protected method
    }

    fn CalculatePath(&mut self, node: *mut CNode) {
        // Stub: protected method
    }
}

//////////////////////////////////////////////////////////////////////
// class Priority Queue
//////////////////////////////////////////////////////////////////////
pub struct CPriorityQueue {
    mHeap: Vec<*mut CEdge>,
}

impl CPriorityQueue {
    pub fn new() -> Self {
        CPriorityQueue {
            mHeap: Vec::new(),
        }
    }

    pub fn Pop(&mut self) -> *mut CEdge {
        // Stub: actual implementation would pop from heap
        std::ptr::null_mut()
    }

    pub fn Find(&self, npNum: i32) -> *mut CEdge {
        // Stub: actual implementation would find edge
        std::ptr::null_mut()
    }

    pub fn Push(&mut self, theEdge: *mut CEdge) {
        // Stub: actual implementation would push to heap
    }

    pub fn Update(&mut self, edge: *mut CEdge) {
        // Stub: actual implementation would update edge in heap
    }

    pub fn Empty(&self) -> bool {
        // Stub: actual implementation would check if empty
        true
    }
}

pub static mut navigator: CNavigator = CNavigator {
    m_nodes: Vec::new(),
    m_edgeLookupMap: std::collections::BTreeMap::new(),
    failedEdges: [failedEdge_t { _dummy: 0 }; MAX_FAILED_EDGES],
    m_ranks: std::ptr::null_mut(),
    m_numEdges: 0,
    pathsCalculated: 0,
};
