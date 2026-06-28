/*****************************************************************************
 * name:		be_aas_reach.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_reach.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::c_int;
use core::ffi::c_float;

// Local stubs for types from other modules - needed for function signatures
#[repr(C)]
pub struct aas_link_t;
pub type vec3_t = [c_float; 3];

#[cfg(feature = "aasintern")]
extern "C" {
    //initialize calculating the reachabilities
    pub fn AAS_InitReachability();
    //continue calculating the reachabilities
    pub fn AAS_ContinueInitReachability(time: c_float) -> c_int;
    //
    pub fn AAS_BestReachableLinkArea(areas: *mut aas_link_t) -> c_int;
}

extern "C" {
    //returns true if the are has reachabilities to other areas
    pub fn AAS_AreaReachability(areanum: c_int) -> c_int;
    //returns the best reachable area and goal origin for a bounding box at the given origin
    pub fn AAS_BestReachableArea(origin: vec3_t, mins: vec3_t, maxs: vec3_t, goalorigin: vec3_t) -> c_int;
    //returns the best jumppad area from which the bbox at origin is reachable
    pub fn AAS_BestReachableFromJumpPadArea(origin: vec3_t, mins: vec3_t, maxs: vec3_t) -> c_int;
    //returns the next reachability using the given model
    pub fn AAS_NextModelReachability(num: c_int, modelnum: c_int) -> c_int;
    //returns the total area of the ground faces of the given area
    pub fn AAS_AreaGroundFaceArea(areanum: c_int) -> c_float;
    //returns true if the area is crouch only
    pub fn AAS_AreaCrouch(areanum: c_int) -> c_int;
    //returns true if a player can swim in this area
    pub fn AAS_AreaSwim(areanum: c_int) -> c_int;
    //returns true if the area is filled with a liquid
    pub fn AAS_AreaLiquid(areanum: c_int) -> c_int;
    //returns true if the area contains lava
    pub fn AAS_AreaLava(areanum: c_int) -> c_int;
    //returns true if the area contains slime
    pub fn AAS_AreaSlime(areanum: c_int) -> c_int;
    //returns true if the area has one or more ground faces
    pub fn AAS_AreaGrounded(areanum: c_int) -> c_int;
    //returns true if the area has one or more ladder faces
    pub fn AAS_AreaLadder(areanum: c_int) -> c_int;
    //returns true if the area is a jump pad
    pub fn AAS_AreaJumpPad(areanum: c_int) -> c_int;
    //returns true if the area is donotenter
    pub fn AAS_AreaDoNotEnter(areanum: c_int) -> c_int;
}
