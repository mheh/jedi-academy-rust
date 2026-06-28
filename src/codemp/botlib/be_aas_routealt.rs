// name:		be_aas_routealt.c
//
// desc:		AAS
//
// $Archive: /MissionPack/code/botlib/be_aas_routealt.c $
// $Author: Zaphod $
// $Revision: 5 $
// $Modtime: 11/22/00 8:47a $
// $Date: 11/22/00 8:55a $

// Stub imports - full module definitions would come from elsewhere
// These represent external engine/botlib functions and globals
mod stubs {
    use core::ffi::c_int;

    extern "C" {
        // External AAS functions
        pub fn AAS_AreaTravelTimeToGoalArea(
            areanum: c_int,
            origin: *const f32,
            goalareanum: c_int,
            travelflags: c_int,
        ) -> c_int;
        pub fn AAS_AreaReachability(areanum: c_int) -> c_int;
        pub fn Log_Write(fmt: *const u8, ...) -> ();
        pub fn Sys_MilliSeconds() -> c_int;
        pub fn AAS_ShowAreaPolygons(areanum: c_int, time: c_int, showback: c_int) -> ();
        pub fn Com_Memset(dest: *mut u8, c: c_int, count: usize) -> *mut u8;
        pub fn GetMemory(size: usize) -> *mut u8;
        pub fn FreeMemory(ptr: *mut u8) -> ();

        // External global: botimport
        pub static mut botimport: BotImportT;
    }

    #[repr(C)]
    pub struct BotImportT {
        pub Print: unsafe extern "C" fn(type_: c_int, fmt: *const u8, ...) -> (),
        // ... other fields would go here
    }
}

use core::ffi::c_int;
use core::mem;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct midrangearea_s {
    pub valid: c_int,
    pub starttime: u16,
    pub goaltime: u16,
}

pub type midrangearea_t = midrangearea_s;

pub static mut midrangeareas: *mut midrangearea_t = core::ptr::null_mut();
pub static mut clusterareas: *mut c_int = core::ptr::null_mut();
pub static mut numclusterareas: c_int = 0;

// Stub types for external engine structures
// These would be defined more completely in their respective modules
#[repr(C)]
pub struct aas_area_t {
    pub numfaces: c_int,
    pub firstface: c_int,
    pub center: [f32; 3],
    // ... other fields omitted
}

#[repr(C)]
pub struct aas_face_t {
    pub frontarea: c_int,
    pub backarea: c_int,
    // ... other fields omitted
}

#[repr(C)]
pub struct aas_areasettings_t {
    pub contents: c_int,
    // ... other fields omitted
}

#[repr(C)]
pub struct aasworld_s {
    pub areas: *mut aas_area_t,
    pub faces: *mut aas_face_t,
    pub faceindex: *mut c_int,
    pub numareas: c_int,
    pub areasettings: *mut aas_areasettings_t,
    // ... other fields omitted
}

#[repr(C)]
pub struct aas_altroutegoal_t {
    pub origin: [f32; 3],
    pub areanum: c_int,
    pub starttraveltime: c_int,
    pub goaltraveltime: c_int,
    pub extratraveltime: c_int,
}

extern "C" {
    pub static mut aasworld: aasworld_s;
}

// Constants
const ALTROUTEGOAL_ALL: c_int = 1;
const ALTROUTEGOAL_CLUSTERPORTALS: c_int = 2;
const ALTROUTEGOAL_VIEWPORTALS: c_int = 4;
const AREACONTENTS_CLUSTERPORTAL: c_int = 0x80;
const AREACONTENTS_VIEWPORTAL: c_int = 0x100;
const PRT_MESSAGE: c_int = 1;

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
fn AAS_AltRoutingFloodCluster_r(areanum: c_int) {
    let mut i: c_int;
    let mut otherareanum: c_int;
    let area: *mut aas_area_t;
    let face: *mut aas_face_t;

    unsafe {
        //add the current area to the areas of the current cluster
        *clusterareas.add(numclusterareas as usize) = areanum;
        numclusterareas += 1;
        //remove the area from the mid range areas
        (*midrangeareas.add(areanum as usize)).valid = 0; // qfalse
        //flood to other areas through the faces of this area
        area = &mut *(*core::ptr::addr_of_mut!(aasworld))
            .areas
            .add(areanum as usize);
        i = 0;
        while i < (*area).numfaces {
            face = &mut *(*core::ptr::addr_of_mut!(aasworld))
                .faces
                .add(((*(*core::ptr::addr_of_mut!(aasworld))
                    .faceindex
                    .add(((*area).firstface + i) as usize)) as i32)
                    .abs() as usize);
            //get the area at the other side of the face
            if (*face).frontarea == areanum {
                otherareanum = (*face).backarea;
            } else {
                otherareanum = (*face).frontarea;
            }
            //if there is an area at the other side of this face
            if otherareanum == 0 {
                i += 1;
                continue;
            }
            //if the other area is not a midrange area
            if (*midrangeareas.add(otherareanum as usize)).valid == 0 {
                i += 1;
                continue;
            }
            //
            AAS_AltRoutingFloodCluster_r(otherareanum);
            i += 1;
        } //end for
    }
} //end of the function AAS_AltRoutingFloodCluster_r

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "enable_altrouting")]
pub fn AAS_AlternativeRouteGoals(
    start: *const f32,
    startareanum: c_int,
    goal: *const f32,
    goalareanum: c_int,
    travelflags: c_int,
    altroutegoals: *mut aas_altroutegoal_t,
    maxaltroutegoals: c_int,
    type_: c_int,
) -> c_int {
    unsafe {
        let mut i: c_int;
        let mut j: c_int;
        let mut bestareanum: c_int;
        let mut numaltroutegoals: c_int;
        let mut nummidrangeareas: c_int;
        let mut starttime: c_int;
        let mut goaltime: c_int;
        let mut goaltraveltime: c_int;
        let mut dist: f32;
        let mut bestdist: f32;
        let mut mid: [f32; 3];
        let mut dir: [f32; 3];

        #[cfg(feature = "altroute_debug")]
        let mut startmillisecs: c_int;

        #[cfg(feature = "altroute_debug")]
        {
            startmillisecs = stubs::Sys_MilliSeconds();
        }

        if startareanum == 0 || goalareanum == 0 {
            return 0;
        }
        //travel time towards the goal area
        goaltraveltime =
            stubs::AAS_AreaTravelTimeToGoalArea(startareanum, start, goalareanum, travelflags);
        //clear the midrange areas
        stubs::Com_Memset(
            midrangeareas as *mut u8,
            0,
            (*core::ptr::addr_of_mut!(aasworld)).numareas as usize * mem::size_of::<midrangearea_t>(),
        );
        numaltroutegoals = 0;
        //
        nummidrangeareas = 0;
        //
        i = 1;
        while i < (*core::ptr::addr_of_mut!(aasworld)).numareas {
            //
            if (type_ & ALTROUTEGOAL_ALL) == 0 {
                if (type_ & ALTROUTEGOAL_CLUSTERPORTALS) == 0
                    || ((*(*core::ptr::addr_of_mut!(aasworld))
                        .areasettings
                        .add(i as usize))
                        .contents
                        & AREACONTENTS_CLUSTERPORTAL)
                        == 0
                {
                    if (type_ & ALTROUTEGOAL_VIEWPORTALS) == 0
                        || ((*(*core::ptr::addr_of_mut!(aasworld))
                            .areasettings
                            .add(i as usize))
                            .contents
                            & AREACONTENTS_VIEWPORTAL)
                            == 0
                    {
                        i += 1;
                        continue;
                    } //end if
                } //end if
            } //end if
            //if the area has no reachabilities
            if stubs::AAS_AreaReachability(i) == 0 {
                i += 1;
                continue;
            }
            //tavel time from the area to the start area
            starttime = stubs::AAS_AreaTravelTimeToGoalArea(startareanum, start, i, travelflags);
            if starttime == 0 {
                i += 1;
                continue;
            }
            //if the travel time from the start to the area is greater than the shortest goal travel time
            if starttime > (1.1 * goaltraveltime as f32) as c_int {
                i += 1;
                continue;
            }
            //travel time from the area to the goal area
            goaltime = stubs::AAS_AreaTravelTimeToGoalArea(i, core::ptr::null(), goalareanum, travelflags);
            if goaltime == 0 {
                i += 1;
                continue;
            }
            //if the travel time from the area to the goal is greater than the shortest goal travel time
            if goaltime > (0.8 * goaltraveltime as f32) as c_int {
                i += 1;
                continue;
            }
            //this is a mid range area
            (*midrangeareas.add(i as usize)).valid = 1; // qtrue
            (*midrangeareas.add(i as usize)).starttime = starttime as u16;
            (*midrangeareas.add(i as usize)).goaltime = goaltime as u16;
            stubs::Log_Write(b"%d midrange area %d\0".as_ptr(), nummidrangeareas, i);
            nummidrangeareas += 1;
            i += 1;
        } //end for
        //
        i = 1;
        while i < (*core::ptr::addr_of_mut!(aasworld)).numareas {
            if (*midrangeareas.add(i as usize)).valid == 0 {
                i += 1;
                continue;
            }
            //get the areas in one cluster
            numclusterareas = 0;
            AAS_AltRoutingFloodCluster_r(i);
            //now we've got a cluster with areas through which an alternative route could go
            //get the 'center' of the cluster
            mid[0] = 0.0;
            mid[1] = 0.0;
            mid[2] = 0.0; // VectorClear(mid)
            j = 0;
            while j < numclusterareas {
                // VectorAdd
                mid[0] += (*(*core::ptr::addr_of_mut!(aasworld))
                    .areas
                    .add(*clusterareas.add(j as usize) as usize))
                    .center[0];
                mid[1] += (*(*core::ptr::addr_of_mut!(aasworld))
                    .areas
                    .add(*clusterareas.add(j as usize) as usize))
                    .center[1];
                mid[2] += (*(*core::ptr::addr_of_mut!(aasworld))
                    .areas
                    .add(*clusterareas.add(j as usize) as usize))
                    .center[2];
                j += 1;
            } //end for
            // VectorScale(mid, 1.0 / numclusterareas, mid)
            let scale = 1.0 / numclusterareas as f32;
            mid[0] *= scale;
            mid[1] *= scale;
            mid[2] *= scale;

            //get the area closest to the center of the cluster
            bestdist = 999999.0;
            bestareanum = 0;
            j = 0;
            while j < numclusterareas {
                // VectorSubtract
                dir[0] = mid[0]
                    - (*(*core::ptr::addr_of_mut!(aasworld))
                        .areas
                        .add(*clusterareas.add(j as usize) as usize))
                    .center[0];
                dir[1] = mid[1]
                    - (*(*core::ptr::addr_of_mut!(aasworld))
                        .areas
                        .add(*clusterareas.add(j as usize) as usize))
                    .center[1];
                dir[2] = mid[2]
                    - (*(*core::ptr::addr_of_mut!(aasworld))
                        .areas
                        .add(*clusterareas.add(j as usize) as usize))
                    .center[2];
                // VectorLength
                dist = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                if dist < bestdist {
                    bestdist = dist;
                    bestareanum = *clusterareas.add(j as usize);
                } //end if
                j += 1;
            } //end for
            //now we've got an area for an alternative route
            //FIXME: add alternative goal origin
            // VectorCopy
            (*altroutegoals.add(numaltroutegoals as usize)).origin[0] = (*(*core::ptr::addr_of_mut!(aasworld))
                .areas
                .add(bestareanum as usize))
                .center[0];
            (*altroutegoals.add(numaltroutegoals as usize)).origin[1] = (*(*core::ptr::addr_of_mut!(aasworld))
                .areas
                .add(bestareanum as usize))
                .center[1];
            (*altroutegoals.add(numaltroutegoals as usize)).origin[2] = (*(*core::ptr::addr_of_mut!(aasworld))
                .areas
                .add(bestareanum as usize))
                .center[2];

            (*altroutegoals.add(numaltroutegoals as usize)).areanum = bestareanum;
            (*altroutegoals.add(numaltroutegoals as usize)).starttraveltime =
                (*midrangeareas.add(bestareanum as usize)).starttime as c_int;
            (*altroutegoals.add(numaltroutegoals as usize)).goaltraveltime =
                (*midrangeareas.add(bestareanum as usize)).goaltime as c_int;
            (*altroutegoals.add(numaltroutegoals as usize)).extratraveltime = ((*midrangeareas.add(bestareanum as usize)).starttime as c_int
                + (*midrangeareas.add(bestareanum as usize)).goaltime as c_int)
                - goaltraveltime;
            numaltroutegoals += 1;
            //
            #[cfg(feature = "altroute_debug")]
            {
                stubs::AAS_ShowAreaPolygons(bestareanum, 1, 1); // qtrue
            }
            //don't return more than the maximum alternative route goals
            if numaltroutegoals >= maxaltroutegoals {
                break;
            }
            i += 1;
        } //end for
        #[cfg(feature = "altroute_debug")]
        {
            ((*core::ptr::addr_of_mut!(stubs::botimport)).Print)(
                PRT_MESSAGE,
                b"alternative route goals in %d msec\n\0".as_ptr(),
                stubs::Sys_MilliSeconds() - startmillisecs,
            );
        }
        numaltroutegoals
    }
} //end of the function AAS_AlternativeRouteGoals

#[cfg(not(feature = "enable_altrouting"))]
pub fn AAS_AlternativeRouteGoals(
    _start: *const f32,
    _startareanum: c_int,
    _goal: *const f32,
    _goalareanum: c_int,
    _travelflags: c_int,
    _altroutegoals: *mut aas_altroutegoal_t,
    _maxaltroutegoals: c_int,
    _type_: c_int,
) -> c_int {
    0
}

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "enable_altrouting")]
pub fn AAS_InitAlternativeRouting() {
    unsafe {
        if !midrangeareas.is_null() {
            stubs::FreeMemory(midrangeareas as *mut u8);
        }
        midrangeareas = stubs::GetMemory(
            (*core::ptr::addr_of_mut!(aasworld)).numareas as usize * mem::size_of::<midrangearea_t>(),
        ) as *mut midrangearea_t;
        if !clusterareas.is_null() {
            stubs::FreeMemory(clusterareas as *mut u8);
        }
        clusterareas =
            stubs::GetMemory((*core::ptr::addr_of_mut!(aasworld)).numareas as usize * mem::size_of::<c_int>())
                as *mut c_int;
    }
} //end of the function AAS_InitAlternativeRouting

#[cfg(not(feature = "enable_altrouting"))]
pub fn AAS_InitAlternativeRouting() {}

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
#[cfg(feature = "enable_altrouting")]
pub fn AAS_ShutdownAlternativeRouting() {
    unsafe {
        if !midrangeareas.is_null() {
            stubs::FreeMemory(midrangeareas as *mut u8);
        }
        midrangeareas = core::ptr::null_mut();
        if !clusterareas.is_null() {
            stubs::FreeMemory(clusterareas as *mut u8);
        }
        clusterareas = core::ptr::null_mut();
        numclusterareas = 0;
    }
} //end of the function AAS_ShutdownAlternativeRouting

#[cfg(not(feature = "enable_altrouting"))]
pub fn AAS_ShutdownAlternativeRouting() {}
