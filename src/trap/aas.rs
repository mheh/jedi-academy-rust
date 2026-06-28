//! AAS (Area Awareness System) syscall wrappers — `trap_AAS_*` (`BOTLIB_AAS_*` family).
//! 1:1 with `refs/raven-jediacademy/codemp/game/g_syscalls.c`; faithful thin thunks.
//! Per the faithful-types decision, the C `void * /* struct aas_*_s */` out-params
//! take their real Rust struct type and are cast to the raw pointer at the call.

use core::ffi::{c_char, c_int};

use crate::codemp::game::be_aas_h::{
    aas_altroutegoal_t, aas_areainfo_t, aas_clientmove_t, aas_entityinfo_t, aas_predictroute_t,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport::*;

/// `trap_AAS_EntityInfo`.
pub fn AAS_EntityInfo(entnum: i32, info: &mut aas_entityinfo_t) {
    unsafe {
        syscall!(
            BOTLIB_AAS_ENTITY_INFO,
            entnum,
            info as *mut aas_entityinfo_t
        );
    }
}

/// `trap_AAS_Initialized`.
pub fn AAS_Initialized() -> i32 {
    unsafe { syscall!(BOTLIB_AAS_INITIALIZED) as i32 }
}

/// `trap_AAS_PresenceTypeBoundingBox`.
pub fn AAS_PresenceTypeBoundingBox(presencetype: i32, mins: &mut vec3_t, maxs: &mut vec3_t) {
    unsafe {
        syscall!(
            BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX,
            presencetype,
            mins.as_mut_ptr(),
            maxs.as_mut_ptr()
        );
    }
}

/// `trap_AAS_Time`.
pub fn AAS_Time() -> f32 {
    let temp = unsafe { syscall!(BOTLIB_AAS_TIME) } as i32;
    f32::from_bits(temp as u32)
}

/// `trap_AAS_PointAreaNum`.
pub fn AAS_PointAreaNum(point: &vec3_t) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_POINT_AREA_NUM, point.as_ptr()) as i32 }
}

/// `trap_AAS_PointReachabilityAreaIndex`.
pub fn AAS_PointReachabilityAreaIndex(point: &vec3_t) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX, point.as_ptr()) as i32 }
}

/// `trap_AAS_TraceAreas`.
pub fn AAS_TraceAreas(
    start: &vec3_t,
    end: &vec3_t,
    areas: &mut [c_int],
    points: &mut [vec3_t],
    maxareas: i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AAS_TRACE_AREAS,
            start.as_ptr(),
            end.as_ptr(),
            areas.as_mut_ptr(),
            points.as_mut_ptr(),
            maxareas
        ) as i32
    }
}

/// `trap_AAS_BBoxAreas`.
pub fn AAS_BBoxAreas(
    absmins: &vec3_t,
    absmaxs: &vec3_t,
    areas: &mut [c_int],
    maxareas: i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AAS_BBOX_AREAS,
            absmins.as_ptr(),
            absmaxs.as_ptr(),
            areas.as_mut_ptr(),
            maxareas
        ) as i32
    }
}

/// `trap_AAS_AreaInfo`.
pub fn AAS_AreaInfo(areanum: i32, info: &mut aas_areainfo_t) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_AREA_INFO, areanum, info as *mut aas_areainfo_t) as i32 }
}

/// `trap_AAS_PointContents`.
pub fn AAS_PointContents(point: &vec3_t) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_POINT_CONTENTS, point.as_ptr()) as i32 }
}

/// `trap_AAS_NextBSPEntity`.
pub fn AAS_NextBSPEntity(ent: i32) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_NEXT_BSP_ENTITY, ent) as i32 }
}

/// `trap_AAS_ValueForBSPEpairKey`.
pub fn AAS_ValueForBSPEpairKey(ent: i32, key: &str, value: &mut [c_char], size: i32) -> i32 {
    let k = super::cstr(key);
    unsafe {
        syscall!(
            BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY,
            ent,
            k.as_ptr(),
            value.as_mut_ptr(),
            size
        ) as i32
    }
}

/// `trap_AAS_VectorForBSPEpairKey`.
pub fn AAS_VectorForBSPEpairKey(ent: i32, key: &str, v: &mut vec3_t) -> i32 {
    let k = super::cstr(key);
    unsafe {
        syscall!(
            BOTLIB_AAS_VECTOR_FOR_BSP_EPAIR_KEY,
            ent,
            k.as_ptr(),
            v.as_mut_ptr()
        ) as i32
    }
}

/// `trap_AAS_FloatForBSPEpairKey`.
pub fn AAS_FloatForBSPEpairKey(ent: i32, key: &str, value: &mut f32) -> i32 {
    let k = super::cstr(key);
    unsafe {
        syscall!(
            BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY,
            ent,
            k.as_ptr(),
            value as *mut f32
        ) as i32
    }
}

/// `trap_AAS_IntForBSPEpairKey`.
pub fn AAS_IntForBSPEpairKey(ent: i32, key: &str, value: &mut c_int) -> i32 {
    let k = super::cstr(key);
    unsafe {
        syscall!(
            BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY,
            ent,
            k.as_ptr(),
            value as *mut c_int
        ) as i32
    }
}

/// `trap_AAS_AreaReachability`.
pub fn AAS_AreaReachability(areanum: i32) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_AREA_REACHABILITY, areanum) as i32 }
}

/// `trap_AAS_AreaTravelTimeToGoalArea`.
pub fn AAS_AreaTravelTimeToGoalArea(
    areanum: i32,
    origin: &vec3_t,
    goalareanum: i32,
    travelflags: i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AAS_AREA_TRAVEL_TIME_TO_GOAL_AREA,
            areanum,
            origin.as_ptr(),
            goalareanum,
            travelflags
        ) as i32
    }
}

/// `trap_AAS_EnableRoutingArea`.
pub fn AAS_EnableRoutingArea(areanum: i32, enable: i32) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_ENABLE_ROUTING_AREA, areanum, enable) as i32 }
}

/// `trap_AAS_PredictRoute`.
#[allow(clippy::too_many_arguments)]
pub fn AAS_PredictRoute(
    route: &mut aas_predictroute_t,
    areanum: i32,
    origin: &vec3_t,
    goalareanum: i32,
    travelflags: i32,
    maxareas: i32,
    maxtime: i32,
    stopevent: i32,
    stopcontents: i32,
    stoptfl: i32,
    stopareanum: i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AAS_PREDICT_ROUTE,
            route as *mut aas_predictroute_t,
            areanum,
            origin.as_ptr(),
            goalareanum,
            travelflags,
            maxareas,
            maxtime,
            stopevent,
            stopcontents,
            stoptfl,
            stopareanum
        ) as i32
    }
}

/// `trap_AAS_AlternativeRouteGoals`.
#[allow(clippy::too_many_arguments)]
pub fn AAS_AlternativeRouteGoals(
    start: &vec3_t,
    startareanum: i32,
    goal: &vec3_t,
    goalareanum: i32,
    travelflags: i32,
    altroutegoals: &mut [aas_altroutegoal_t],
    maxaltroutegoals: i32,
    r#type: i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AAS_ALTERNATIVE_ROUTE_GOAL,
            start.as_ptr(),
            startareanum,
            goal.as_ptr(),
            goalareanum,
            travelflags,
            altroutegoals.as_mut_ptr(),
            maxaltroutegoals,
            r#type
        ) as i32
    }
}

/// `trap_AAS_Swimming`.
pub fn AAS_Swimming(origin: &vec3_t) -> i32 {
    unsafe { syscall!(BOTLIB_AAS_SWIMMING, origin.as_ptr()) as i32 }
}

/// `trap_AAS_PredictClientMovement`.
#[allow(clippy::too_many_arguments)]
pub fn AAS_PredictClientMovement(
    movement: &mut aas_clientmove_t,
    entnum: i32,
    origin: &vec3_t,
    presencetype: i32,
    onground: i32,
    velocity: &vec3_t,
    cmdmove: &vec3_t,
    cmdframes: i32,
    maxframes: i32,
    frametime: f32,
    stopevent: i32,
    stopareanum: i32,
    visualize: i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT,
            movement as *mut aas_clientmove_t,
            entnum,
            origin.as_ptr(),
            presencetype,
            onground,
            velocity.as_ptr(),
            cmdmove.as_ptr(),
            cmdframes,
            maxframes,
            pass_float(frametime),
            stopevent,
            stopareanum,
            visualize
        ) as i32
    }
}
