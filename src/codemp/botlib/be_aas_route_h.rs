/*****************************************************************************
 * name:		be_aas_route.h
 *
 * desc:		AAS
 *
 * $Archive: /source/code/botlib/be_aas_route.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::c_int;

// Opaque type for aas_reachability struct — defined elsewhere
#[repr(C)]
pub struct aas_reachability_s {
    _unused: [u8; 0],
}

// Opaque type for aas_predictroute struct — defined elsewhere
#[repr(C)]
pub struct aas_predictroute_s {
    _unused: [u8; 0],
}

extern "C" {
    // Original C header had #ifdef AASINTERN guard — functions below were conditional.
    // In C they were:
    //initialize the AAS routing
    pub fn AAS_InitRouting();
    //free the AAS routing caches
    pub fn AAS_FreeRoutingCaches();
    //returns the travel time from start to end in the given area
    pub fn AAS_AreaTravelTime(areanum: c_int, start: *const f32, end: *const f32) -> u16;
    //
    pub fn AAS_CreateAllRoutingCache();
    pub fn AAS_WriteRouteCache();
    //
    pub fn AAS_RoutingInfo();

    //returns the travel flag for the given travel type
    pub fn AAS_TravelFlagForType(traveltype: c_int) -> c_int;
    //return the travel flag(s) for traveling through this area
    pub fn AAS_AreaContentsTravelFlags(areanum: c_int) -> c_int;
    //returns the index of the next reachability for the given area
    pub fn AAS_NextAreaReachability(areanum: c_int, reachnum: c_int) -> c_int;
    //returns the reachability with the given index
    pub fn AAS_ReachabilityFromNum(num: c_int, reach: *mut aas_reachability_s);
    //returns a random goal area and goal origin
    pub fn AAS_RandomGoalArea(
        areanum: c_int,
        travelflags: c_int,
        goalareanum: *mut c_int,
        goalorigin: *mut f32,
    ) -> c_int;
    //enable or disable an area for routing
    pub fn AAS_EnableRoutingArea(areanum: c_int, enable: c_int) -> c_int;
    //returns the travel time within the given area from start to end
    pub fn AAS_AreaTravelTime(areanum: c_int, start: *const f32, end: *const f32) -> u16;
    //returns the travel time from the area to the goal area using the given travel flags
    pub fn AAS_AreaTravelTimeToGoalArea(
        areanum: c_int,
        origin: *const f32,
        goalareanum: c_int,
        travelflags: c_int,
    ) -> c_int;
    //predict a route up to a stop event
    pub fn AAS_PredictRoute(
        route: *mut aas_predictroute_s,
        areanum: c_int,
        origin: *const f32,
        goalareanum: c_int,
        travelflags: c_int,
        maxareas: c_int,
        maxtime: c_int,
        stopevent: c_int,
        stopcontents: c_int,
        stoptfl: c_int,
        stopareanum: c_int,
    ) -> c_int;
}
