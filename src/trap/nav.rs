//! NPC navigation syscall wrappers — `trap_Nav_*` (`G_NAV_*` family).
//! 1:1 with `refs/raven-jediacademy/codemp/game/g_syscalls.c`; faithful thin thunks.
//! Types: g_public_h::failedEdge_t (already ported).

use super::cstr;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_public_h::failedEdge_t;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport::*;

/// `trap_Nav_Init`.
pub fn Nav_Init() {
    unsafe {
        syscall!(G_NAV_INIT);
    }
}

/// `trap_Nav_Free`.
pub fn Nav_Free() {
    unsafe {
        syscall!(G_NAV_FREE);
    }
}

/// `trap_Nav_Load`.
pub fn Nav_Load(filename: &str, checksum: i32) -> qboolean {
    let f = cstr(filename);
    unsafe { syscall!(G_NAV_LOAD, f.as_ptr(), checksum) as qboolean }
}

/// `trap_Nav_Save`.
pub fn Nav_Save(filename: &str, checksum: i32) -> qboolean {
    let f = cstr(filename);
    unsafe { syscall!(G_NAV_SAVE, f.as_ptr(), checksum) as qboolean }
}

/// `trap_Nav_AddRawPoint`.
pub fn Nav_AddRawPoint(point: &vec3_t, flags: i32, radius: i32) -> i32 {
    unsafe { syscall!(G_NAV_ADDRAWPOINT, point.as_ptr(), flags, radius) as i32 }
}

/// `trap_Nav_CalculatePaths` (`recalc = qfalse`).
pub fn Nav_CalculatePaths(recalc: qboolean) {
    unsafe {
        syscall!(G_NAV_CALCULATEPATHS, recalc);
    }
}

/// `trap_Nav_HardConnect`.
pub fn Nav_HardConnect(first: i32, second: i32) {
    unsafe {
        syscall!(G_NAV_HARDCONNECT, first, second);
    }
}

/// `trap_Nav_ShowNodes`.
pub fn Nav_ShowNodes() {
    unsafe {
        syscall!(G_NAV_SHOWNODES);
    }
}

/// `trap_Nav_ShowEdges`.
pub fn Nav_ShowEdges() {
    unsafe {
        syscall!(G_NAV_SHOWEDGES);
    }
}

/// `trap_Nav_ShowPath`.
pub fn Nav_ShowPath(start: i32, end: i32) {
    unsafe {
        syscall!(G_NAV_SHOWPATH, start, end);
    }
}

/// `trap_Nav_GetNearestNode`.
pub fn Nav_GetNearestNode(ent: *mut gentity_t, last_id: i32, flags: i32, target_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETNEARESTNODE, ent, last_id, flags, target_id) as i32 }
}

/// `trap_Nav_GetBestNode` (`rejectID = NODE_NONE`).
pub fn Nav_GetBestNode(start_id: i32, end_id: i32, reject_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETBESTNODE, start_id, end_id, reject_id) as i32 }
}

/// `trap_Nav_GetNodePosition` — fills `out` with the node's world position.
pub fn Nav_GetNodePosition(node_id: i32, out: &mut vec3_t) -> i32 {
    unsafe { syscall!(G_NAV_GETNODEPOSITION, node_id, out.as_mut_ptr()) as i32 }
}

/// `trap_Nav_GetNodeNumEdges`.
pub fn Nav_GetNodeNumEdges(node_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETNODENUMEDGES, node_id) as i32 }
}

/// `trap_Nav_GetNodeEdge`.
pub fn Nav_GetNodeEdge(node_id: i32, edge: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETNODEEDGE, node_id, edge) as i32 }
}

/// `trap_Nav_GetNumNodes`.
pub fn Nav_GetNumNodes() -> i32 {
    unsafe { syscall!(G_NAV_GETNUMNODES) as i32 }
}

/// `trap_Nav_Connected`.
pub fn Nav_Connected(start_id: i32, end_id: i32) -> qboolean {
    unsafe { syscall!(G_NAV_CONNECTED, start_id, end_id) as qboolean }
}

/// `trap_Nav_GetPathCost`.
pub fn Nav_GetPathCost(start_id: i32, end_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETPATHCOST, start_id, end_id) as i32 }
}

/// `trap_Nav_GetEdgeCost`.
pub fn Nav_GetEdgeCost(start_id: i32, end_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETEDGECOST, start_id, end_id) as i32 }
}

/// `trap_Nav_GetProjectedNode`.
pub fn Nav_GetProjectedNode(origin: &vec3_t, node_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETPROJECTEDNODE, origin.as_ptr(), node_id) as i32 }
}

/// `trap_Nav_CheckFailedNodes`.
pub fn Nav_CheckFailedNodes(ent: *mut gentity_t) {
    unsafe {
        syscall!(G_NAV_CHECKFAILEDNODES, ent);
    }
}

/// `trap_Nav_AddFailedNode`.
pub fn Nav_AddFailedNode(ent: *mut gentity_t, node_id: i32) {
    unsafe {
        syscall!(G_NAV_ADDFAILEDNODE, ent, node_id);
    }
}

/// `trap_Nav_NodeFailed`.
pub fn Nav_NodeFailed(ent: *mut gentity_t, node_id: i32) -> qboolean {
    unsafe { syscall!(G_NAV_NODEFAILED, ent, node_id) as qboolean }
}

/// `trap_Nav_NodesAreNeighbors`.
pub fn Nav_NodesAreNeighbors(start_id: i32, end_id: i32) -> qboolean {
    unsafe { syscall!(G_NAV_NODESARENEIGHBORS, start_id, end_id) as qboolean }
}

/// `trap_Nav_ClearFailedEdge`.
pub fn Nav_ClearFailedEdge(failed_edge: &mut failedEdge_t) {
    unsafe {
        syscall!(G_NAV_CLEARFAILEDEDGE, failed_edge as *mut failedEdge_t);
    }
}

/// `trap_Nav_ClearAllFailedEdges`.
pub fn Nav_ClearAllFailedEdges() {
    unsafe {
        syscall!(G_NAV_CLEARALLFAILEDEDGES);
    }
}

/// `trap_Nav_EdgeFailed`.
pub fn Nav_EdgeFailed(start_id: i32, end_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_EDGEFAILED, start_id, end_id) as i32 }
}

/// `trap_Nav_AddFailedEdge`.
pub fn Nav_AddFailedEdge(ent_id: i32, start_id: i32, end_id: i32) {
    unsafe {
        syscall!(G_NAV_ADDFAILEDEDGE, ent_id, start_id, end_id);
    }
}

/// `trap_Nav_CheckFailedEdge`.
pub fn Nav_CheckFailedEdge(failed_edge: &mut failedEdge_t) -> qboolean {
    unsafe { syscall!(G_NAV_CHECKFAILEDEDGE, failed_edge as *mut failedEdge_t) as qboolean }
}

/// `trap_Nav_CheckAllFailedEdges`.
pub fn Nav_CheckAllFailedEdges() {
    unsafe {
        syscall!(G_NAV_CHECKALLFAILEDEDGES);
    }
}

/// `trap_Nav_RouteBlocked`.
pub fn Nav_RouteBlocked(
    start_id: i32,
    test_edge_id: i32,
    end_id: i32,
    reject_rank: i32,
) -> qboolean {
    unsafe { syscall!(G_NAV_ROUTEBLOCKED, start_id, test_edge_id, end_id, reject_rank) as qboolean }
}

/// `trap_Nav_GetBestNodeAltRoute` (`rejectID = NODE_NONE`).
pub fn Nav_GetBestNodeAltRoute(
    start_id: i32,
    end_id: i32,
    path_cost: &mut i32,
    reject_id: i32,
) -> i32 {
    unsafe {
        syscall!(
            G_NAV_GETBESTNODEALTROUTE,
            start_id,
            end_id,
            path_cost as *mut i32,
            reject_id
        ) as i32
    }
}

/// `trap_Nav_GetBestNodeAltRoute2` (`rejectID = NODE_NONE`).
pub fn Nav_GetBestNodeAltRoute2(start_id: i32, end_id: i32, reject_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETBESTNODEALT2, start_id, end_id, reject_id) as i32 }
}

/// `trap_Nav_GetBestPathBetweenEnts`.
pub fn Nav_GetBestPathBetweenEnts(ent: *mut gentity_t, goal: *mut gentity_t, flags: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETBESTPATHBETWEENENTS, ent, goal, flags) as i32 }
}

/// `trap_Nav_GetNodeRadius`.
pub fn Nav_GetNodeRadius(node_id: i32) -> i32 {
    unsafe { syscall!(G_NAV_GETNODERADIUS, node_id) as i32 }
}

/// `trap_Nav_CheckBlockedEdges`.
pub fn Nav_CheckBlockedEdges() {
    unsafe {
        syscall!(G_NAV_CHECKBLOCKEDEDGES);
    }
}

/// `trap_Nav_ClearCheckedNodes`.
pub fn Nav_ClearCheckedNodes() {
    unsafe {
        syscall!(G_NAV_CLEARCHECKEDNODES);
    }
}

/// `trap_Nav_CheckedNode` (return int was byte).
pub fn Nav_CheckedNode(way_point: i32, ent: i32) -> i32 {
    unsafe { syscall!(G_NAV_CHECKEDNODE, way_point, ent) as i32 }
}

/// `trap_Nav_SetCheckedNode` (int value was byte value).
pub fn Nav_SetCheckedNode(way_point: i32, ent: i32, value: i32) {
    unsafe {
        syscall!(G_NAV_SETCHECKEDNODE, way_point, ent, value);
    }
}

/// `trap_Nav_FlagAllNodes`.
pub fn Nav_FlagAllNodes(new_flag: i32) {
    unsafe {
        syscall!(G_NAV_FLAGALLNODES, new_flag);
    }
}

/// `trap_Nav_GetPathsCalculated`.
pub fn Nav_GetPathsCalculated() -> qboolean {
    unsafe { syscall!(G_NAV_GETPATHSCALCULATED) as qboolean }
}

/// `trap_Nav_SetPathsCalculated`.
pub fn Nav_SetPathsCalculated(new_val: qboolean) {
    unsafe {
        syscall!(G_NAV_SETPATHSCALCULATED, new_val);
    }
}
