////////////////////////////////////////////////////////////////////////////////////////
// RAVEN SOFTWARE - STAR WARS: JK II
//  (c) 2002 Activision
//
//
//
// NAVIGATOR
// ---------
// This file provides an interface to two actor related systems:
//  - Path Finding
//  - Steering
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

use core::ffi::c_int;

// Define the USENEWNAVSYSTEM constant
pub const USENEWNAVSYSTEM: c_int = 1;

// Forward declarations of external types
// These are defined in other modules but needed here for function signatures
pub type gentity_t = core::ffi::c_void;  // Placeholder for actual entity type
pub type vec3_t = [f32; 3];              // Vector type
pub type CVec3 = [f32; 3];               // Vector type (alternative name)
pub type qboolean = c_int;               // Boolean type
pub type usercmd_t = core::ffi::c_void;  // Placeholder for user command type

// Constant for entity number
pub const ENTITYNUM_NONE: c_int = -1;

////////////////////////////////////////////////////////////////////////////////////////
// The NAV Namespace
//
// This namespace provides the public interface to the NPC Navigation and Pathfinding
// system.  This system is a bidirectional graph of nodes and weighted edges.  Finding
// a path from one node to another is accomplished with A*, and cached internally for
// each actor who requests a path.
////////////////////////////////////////////////////////////////////////////////////////
pub mod NAV {
    use core::ffi::c_int;
    use super::{gentity_t, vec3_t, CVec3, qboolean};

    pub type TNodeHandle = c_int;
    pub type TEdgeHandle = c_int;

    ////////////////////////////////////////////////////////////////////////////////////
    //
    //
    ////////////////////////////////////////////////////////////////////////////////////
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub enum EPointType {
        PT_NONE = 0,

        PT_WAYNODE = 1,
        PT_COMBATNODE = 2,
        PT_GOALNODE = 3,

        PT_MAX = 4,
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Save, Load, Construct
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn LoadFromFile(filename: *const u8, checksum: c_int) -> bool;
        pub fn TestEdge(NodeA: TNodeHandle, NodeB: TNodeHandle, IsDebugEdge: qboolean) -> bool;
        pub fn LoadFromEntitiesAndSaveToFile(filename: *const u8, checksum: c_int) -> bool;
        pub fn SpawnedPoint(ent: *mut gentity_t, point_type: EPointType);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Finding Nav Points
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn GetNearestNode(ent: *mut gentity_t, forceRecalcNow: bool, goal: TNodeHandle) -> TNodeHandle;
        pub fn GetNearestNode_2(position: *const vec3_t, previous: TNodeHandle, goal: TNodeHandle, ignoreEnt: c_int, allowZOffset: bool) -> TNodeHandle;

        pub fn ChooseRandomNeighbor(NodeHandle: TNodeHandle) -> TNodeHandle;
        pub fn ChooseRandomNeighbor_2(NodeHandle: TNodeHandle, position: *const vec3_t, maxDistance: f32) -> TNodeHandle;
        pub fn ChooseClosestNeighbor(NodeHandle: TNodeHandle, position: *const vec3_t) -> TNodeHandle;
        pub fn ChooseFarthestNeighbor(NodeHandle: TNodeHandle, position: *const vec3_t) -> TNodeHandle;
        pub fn ChooseFarthestNeighbor_2(actor: *mut gentity_t, target: *const vec3_t, maxSafeDot: f32) -> TNodeHandle;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get The Location Of A Given Node Handle
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn GetNodePosition(NodeHandle: TNodeHandle) -> *const vec3_t;
        pub fn GetNodePosition_2(NodeHandle: TNodeHandle, position: *mut vec3_t);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Testing Nearness
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn EstimateCostToGoal(position: *const vec3_t, Goal: TNodeHandle) -> f32;
        pub fn EstimateCostToGoal_2(Start: TNodeHandle, Goal: TNodeHandle) -> f32;

        pub fn OnSamePoint(actor: *mut gentity_t, target: *mut gentity_t) -> bool;
        pub fn OnNeighboringPoints(A: TNodeHandle, B: TNodeHandle) -> bool;
        pub fn OnNeighboringPoints_2(actor: *mut gentity_t, target: *mut gentity_t) -> bool;
        pub fn OnNeighboringPoints_3(actor: *mut gentity_t, position: *const vec3_t) -> bool;
        pub fn InSameRegion(actor: *mut gentity_t, target: *mut gentity_t) -> bool;
        pub fn InSameRegion_2(actor: *mut gentity_t, position: *const vec3_t) -> bool;
        pub fn InSameRegion_3(A: TNodeHandle, B: TNodeHandle) -> bool;

        pub fn InSafeRadius(at: CVec3, atNode: TNodeHandle, targetNode: TNodeHandle) -> bool;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Finding A Path
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn GoTo(actor: *mut gentity_t, target: TNodeHandle, MaxDangerLevel: f32) -> bool;
        pub fn GoTo_2(actor: *mut gentity_t, target: *mut gentity_t, MaxDangerLevel: f32) -> bool;
        pub fn GoTo_3(actor: *mut gentity_t, position: *const vec3_t, MaxDangerLevel: f32) -> bool;

        pub fn FindPath(actor: *mut gentity_t, target: TNodeHandle, MaxDangerLevel: f32) -> bool;
        pub fn FindPath_2(actor: *mut gentity_t, target: *mut gentity_t, MaxDangerLevel: f32) -> bool;
        pub fn FindPath_3(actor: *mut gentity_t, position: *const vec3_t, MaxDangerLevel: f32) -> bool;

        pub fn SafePathExists(start: CVec3, stop: CVec3, danger: CVec3, dangerDistSq: f32) -> bool;

        pub fn HasPath(actor: *mut gentity_t, target: TNodeHandle) -> bool;
        pub fn ClearPath(actor: *mut gentity_t);
        pub fn UpdatePath(actor: *mut gentity_t, target: TNodeHandle, MaxDangerLevel: f32) -> bool;
        pub fn PathDangerLevel(actor: *mut gentity_t) -> f32;
        pub fn PathNodesRemaining(actor: *mut gentity_t) -> c_int;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn NextPosition(actor: *mut gentity_t) -> *const vec3_t;
        pub fn NextPosition_2(actor: *mut gentity_t, Position: *mut CVec3) -> bool;
        pub fn NextPosition_3(actor: *mut gentity_t, Position: *mut CVec3, SlowingRadius: *mut f32, Fly: *mut bool, Jump: *mut bool) -> bool;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Update One Or More Edges As A Result Of An Entity Getting Removed
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn WayEdgesNowClear(ent: *mut gentity_t);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // How Big Is The Given Ent
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn ClassifyEntSize(ent: *mut gentity_t) -> u32;
        pub fn RegisterDangerSense(actor: *mut gentity_t, alertEventIndex: c_int);
        pub fn DecayDangerSenses();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Debugging Information
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn ShowDebugInfo(PlayerPosition: *const vec3_t, PlayerWaypoint: TNodeHandle);
        pub fn ShowStats();

        pub fn TeleportTo(actor: *mut gentity_t, pointName: *const u8);
        pub fn TeleportTo_2(actor: *mut gentity_t, pointNum: c_int);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The STEER Namespace
//
// These functions allow access to the steering system.
//
// The Reset() and Finalize() functions MUST be called before and after any other steering
// operations.  Beyond that, all other steering operations can be called in any order
// and any number of times.  Once Finalize() is called, the results of all these
// operations will be summed up and applied as accelleration to the actor's velocity.
////////////////////////////////////////////////////////////////////////////////////////
pub mod STEER {
    use core::ffi::c_int;
    use super::{gentity_t, vec3_t, CVec3, usercmd_t};

    ////////////////////////////////////////////////////////////////////////////////////
    // Reset & Finalize
    //
    // Call these two operations before and after all other STEER operations.  They
    // clear out and setup the thrust vector for use by the entity.
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Activate(actor: *mut gentity_t);
        pub fn DeActivate(actor: *mut gentity_t, ucmd: *mut usercmd_t);
        pub fn Active(actor: *mut gentity_t) -> bool;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Master Functions
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn GoTo(actor: *mut gentity_t, target: *mut gentity_t, reachedRadius: f32, avoidCollisions: bool) -> bool;
        pub fn GoTo_2(actor: *mut gentity_t, position: *const vec3_t, reachedRadius: f32, avoidCollisions: bool) -> bool;

        pub fn SafeToGoTo(actor: *mut gentity_t, targetPosition: *const vec3_t, targetNode: c_int) -> bool;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Stop
    //
    // Slow down and come to a stop.
    //
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Stop(actor: *mut gentity_t, weight: f32) -> f32;
        pub fn MatchSpeed(actor: *mut gentity_t, speed: f32, weight: f32) -> f32;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Seek & Flee
    //
    // These two operations form the root of all steering.  They do simple
    // vector operations and add to the thrust vector.
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Seek(actor: *mut gentity_t, pos: CVec3, slowingDistance: f32, weight: f32, desiredSpeed: f32) -> f32;
        pub fn Flee(actor: *mut gentity_t, pos: CVec3, weight: f32) -> f32;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Persue & Evade
    //
    // Slightly more complicated than Seek & Flee, these operations predict the position
    // of the target entitiy.
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Persue(actor: *mut gentity_t, target: *mut gentity_t, slowingDistance: f32) -> f32;
        pub fn Persue_2(actor: *mut gentity_t, target: *mut gentity_t, slowingDistance: f32, offsetForward: f32, offsetRight: f32, offsetUp: f32, relativeToTargetFacing: bool) -> f32;
        pub fn Evade(actor: *mut gentity_t, target: *mut gentity_t) -> f32;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Separation, Alignment, Cohesion
    //
    // These standard steering operations will apply thrust to achieve a group oriented
    // position or direction.
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Separation(actor: *mut gentity_t, Scale: f32) -> f32;
        pub fn Alignment(actor: *mut gentity_t, Scale: f32) -> f32;
        pub fn Cohesion(actor: *mut gentity_t, Scale: f32) -> f32;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Wander & Path
    //
    // By far the most common way to alter a character's thrust, path maintaines motion
    // along a navigational path (see NAV namespace), and a random wander path.
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Path(actor: *mut gentity_t) -> f32;
        pub fn Wander(actor: *mut gentity_t) -> f32;
        pub fn FollowLeader(actor: *mut gentity_t, leader: *mut gentity_t, dist: f32) -> f32;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Collision Avoidance
    //
    // Usually the last steering operation to call before finialization, this operation
    // attempts to avoid collisions with nearby entities and architecture by thrusing
    // away from them.
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn AvoidCollisions(actor: *mut gentity_t, leader: *mut gentity_t) -> f32;
        pub fn SelectLeader(actor: *mut gentity_t) -> *mut gentity_t;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Blocked
    //
    // This function records whether AI is blocked while the steering is active
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Blocked(actor: *mut gentity_t, target: *mut gentity_t);
        pub fn Blocked_2(actor: *mut gentity_t, target: *const vec3_t);
        pub fn HasBeenBlockedFor(actor: *mut gentity_t, duration: c_int) -> bool;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Reached
    //
    // A quick function to see if a target location has been reached by an actor
    ////////////////////////////////////////////////////////////////////////////////////
    extern "C" {
        pub fn Reached(actor: *mut gentity_t, target: *mut gentity_t, targetRadius: f32, flying: bool) -> bool;
        pub fn Reached_2(actor: *mut gentity_t, target: super::NAV::TNodeHandle, targetRadius: f32, flying: bool) -> bool;
        pub fn Reached_3(actor: *mut gentity_t, target: *const vec3_t, targetRadius: f32, flying: bool) -> bool;
    }
}
