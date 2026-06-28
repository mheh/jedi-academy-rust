//! Port of `g_nav.c` — the NPC navigation layer (the `NAV_*` suite): the navgoal-reached
//! test, per-frame blocked-info bookkeeping, the architecture/door-aware path-clearance traces
//! (`NAV_CheckAhead`/`NAV_TestBestNode`), the nearest-node wrapper, the entity collision-avoidance
//! gate (`NAV_AvoidCollision`) and the node-to-node mover (`NAV_MoveToGoal`).
//!
//! Landed as the keystone (cycle 78) and fully drained: the entire now-portable `NAV_*` suite,
//! including the deep collision-resolution family (`NAV_TestForBlocked`/`NAV_ResolveEntityCollision`/
//! `NAV_Bypass`/`NAV_StackedCanyon`/`NAV_TestBypass`/`NAV_ResolveBlock`/`NAV_TrueCollision`/
//! `NAV_MoveBlocker`) — its `NPC_Blocked`/`NPC_FaceEntity` callees were already ported, so the
//! initially-planned guarded stubs were never needed. Also lands `NPC_Blocked`, `NPC_SetMoveGoal`,
//! the `waypoint_testDirection`/`waypoint_getRadius` radius probes and `NAV_Shutdown`.
//! Out of scope and left missing: the spawn-time `SP_waypoint*` entity registration + the
//! `NAV_CalculatePaths`/`NAV_StoreWaypoint`/`Svcmd_Nav_f`/`NAV_ShowDebugInfo`/`NAV_FindPlayerWaypoint`
//! file-scope state machinery (waypointData_t / file-static waypoint table / NAVDEBUG_* globals +
//! the empty `G_Draw*`/`TAG_ShowTags` debug stubs), which depend on those file-scope statics /
//! spawn glue / debug-graphics subsystem.

#![allow(non_snake_case)] // C function names (`NAV_HitNavGoal`, …) kept verbatim
#![allow(non_camel_case_types)] // C type name (`navInfo_t`) kept verbatim
#![allow(non_upper_case_globals)] // C macro-constant names kept verbatim

use core::ffi::c_char;
use core::ptr::{addr_of, addr_of_mut};
use std::ffi::{CStr, CString};

/// `S_COLOR_RED` (q_shared.h) — console color escape.
const S_COLOR_RED: &str = "^1";

/// Owned NUL-terminated copy of a Rust `&str`, for passing to `*const c_char` C ports.
fn cstr(s: &str) -> CString {
    CString::new(s).unwrap_or_default()
}

use crate::codemp::game::b_public_h::NPCAI_BLOCKED;
use crate::codemp::game::b_public_h::NPCAI_NO_COLL_AVOID;
use crate::codemp::game::bg_misc::vectoyaw;
use crate::codemp::game::bg_public::{
    CROUCH_MAXS_2, DEFAULT_MAXS_2, DEFAULT_MINS_2, ET_ITEM, MASK_DEADSOLID, MASK_NPCSOLID, STEPSIZE,
};
use crate::codemp::game::bg_weapons_h::WP_SABER;
use crate::codemp::game::g_local::{gentity_t, waypointData_t, FL_NAVGOAL, FRAMETIME, RTF_NAVGOAL};
use crate::codemp::game::g_main::{g_entities, level, Com_Error, Com_Printf};
use crate::codemp::game::g_misc::TAG_Add;
use crate::codemp::game::g_mover::{
    G_EntIsBreakable, G_EntIsDoor, G_EntIsRemovableUsable, G_EntIsUnlockedDoor,
};
use crate::codemp::game::g_public_h::BSET_BLOCKED;
use crate::codemp::game::g_utils::{vtos, G_CheckInSolid, G_FreeEntity, G_SetOrigin, G_Spawn};
use crate::codemp::game::npc::{NPCInfo, NPC};
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::npc_goal::G_BoundsOverlap;
use crate::codemp::game::npc_utils::{G_ActivateBehavior, NPC_FaceEntity};
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleNormalize360, AngleVectors, CrossProduct, Distance,
    DistanceSquared, DotProduct, PerpendicularVector, VectorAdd, VectorClear, VectorCompare,
    VectorCopy, VectorLengthSquared, VectorMA, VectorNormalize, VectorNormalize2, VectorScale,
    VectorSet, VectorSubtract,
};
use crate::codemp::game::q_shared::random;
use crate::codemp::game::q_shared::{Com_sprintf, Q_stricmp, Q_strncpyz};
use crate::codemp::game::q_shared_h::{
    trace_t, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, ERR_DROP, MAX_QPATH, SOLID_BMODEL, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_BOTCLIP, CONTENTS_MONSTERCLIP, CONTENTS_SOLID, CONTENTS_TRIGGER,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// --- g_nav.h / b_local.h constants ---------------------------------------------------------------

/// `WAYPOINT_NONE` (g_nav.h:7).
pub const WAYPOINT_NONE: i32 = -1;
/// `MAX_COLL_AVOID_DIST` (g_nav.h:12).
pub const MAX_COLL_AVOID_DIST: f32 = 128.0;
/// `NAVGOAL_USE_RADIUS` (g_nav.h:13) — forces manually-radiused navgoals to do a DistanceSquared
/// check rather than just a bounds overlap.
pub const NAVGOAL_USE_RADIUS: i32 = 16384;
/// `MIN_STOP_DIST` (g_nav.h:15).
pub const MIN_STOP_DIST: f32 = 64.0;
/// `MIN_BLOCKED_SPEECH_TIME` (g_nav.h:16).
pub const MIN_BLOCKED_SPEECH_TIME: i32 = 4000;
/// `SHOVE_SPEED` (g_nav.h:19).
pub const SHOVE_SPEED: f32 = 200.0;
/// `SHOVE_LIFT` (g_nav.h:20).
pub const SHOVE_LIFT: f32 = 10.0;
/// `MAX_RADIUS_CHECK` (g_nav.h:21).
pub const MAX_RADIUS_CHECK: u32 = 1024;
/// `YAW_ITERATIONS` (g_nav.h:22).
pub const YAW_ITERATIONS: i32 = 16;
/// `MIN_DOOR_BLOCK_DIST` (g_nav.h:17).
pub const MIN_DOOR_BLOCK_DIST: f32 = 16.0;
/// `MIN_DOOR_BLOCK_DIST_SQR` (g_nav.h:18).
pub const MIN_DOOR_BLOCK_DIST_SQR: f32 = MIN_DOOR_BLOCK_DIST * MIN_DOOR_BLOCK_DIST;
/// `NF_CLEAR_PATH` (g_nav.h:37) — node flag.
pub const NF_CLEAR_PATH: i32 = 0x00000002;
/// `NODE_NONE` (g_nav.h:46).
pub const NODE_NONE: i32 = -1;
/// `NF_ANY` (g_nav.h:35) — node flag, no path requirements.
pub const NF_ANY: i32 = 0;
/// `MAX_STORED_WAYPOINTS` (g_nav.h:9).
pub const MAX_STORED_WAYPOINTS: usize = 512;
/// `NODE_START` (g_public.h:615) — debug-draw node type.
pub const NODE_START: i32 = 0;
/// `NODE_GOAL` (g_public.h:616) — debug-draw node type.
pub const NODE_GOAL: i32 = 2;

// --- JWEIER ADDITIONS: file-scope navigation state (g_nav.c:1606-1671) -----------------------------

/// `qboolean navCalculatePaths = qfalse;` (g_nav.c:1608) — set by the engine while a map's
/// nav data is being (re)built; gates the spawn-time waypoint registration in `SP_waypoint*`.
pub static mut navCalculatePaths: qboolean = QFALSE;

/// `NAVDEBUG_showNodes` (g_nav.c:1610).
pub static mut NAVDEBUG_showNodes: qboolean = QFALSE;
/// `NAVDEBUG_showRadius` (g_nav.c:1611).
pub static mut NAVDEBUG_showRadius: qboolean = QFALSE;
/// `NAVDEBUG_showEdges` (g_nav.c:1612).
pub static mut NAVDEBUG_showEdges: qboolean = QFALSE;
/// `NAVDEBUG_showTestPath` (g_nav.c:1613).
pub static mut NAVDEBUG_showTestPath: qboolean = QFALSE;
/// `NAVDEBUG_showEnemyPath` (g_nav.c:1614).
pub static mut NAVDEBUG_showEnemyPath: qboolean = QFALSE;
/// `NAVDEBUG_showCombatPoints` (g_nav.c:1615).
pub static mut NAVDEBUG_showCombatPoints: qboolean = QFALSE;
/// `NAVDEBUG_showNavGoals` (g_nav.c:1616).
pub static mut NAVDEBUG_showNavGoals: qboolean = QFALSE;
/// `NAVDEBUG_showCollision` (g_nav.c:1617).
pub static mut NAVDEBUG_showCollision: qboolean = QFALSE;
/// `int NAVDEBUG_curGoal = 0;` (g_nav.c:1618).
pub static mut NAVDEBUG_curGoal: i32 = 0;

// `#ifndef FINAL_BUILD` fatal-error accumulator for `NAV_WaypointsTooFar` (g_nav.c:1626-1628).
/// `int fatalErrors = 0;` (g_nav.c:1626). `extern`-referenced by `g_main.c::NAV_CheckCalcPaths`.
pub static mut fatalErrors: i32 = 0;
/// `char fatalErrorString[4096];` (g_nav.c:1628).
static mut fatalErrorString: [c_char; 4096] = [0; 4096];

/// `static int numStoredWaypoints = 0;` (g_nav.c:1669).
static mut numStoredWaypoints: i32 = 0;
/// `static waypointData_t tempWaypointList[MAX_STORED_WAYPOINTS];` (g_nav.c:1671) — "rwwFIXMEFIXME:
/// Need.. dynamic.. memory". Cleared each map by `NAV_ClearStoredWaypoints`.
static mut tempWaypointList: [waypointData_t; MAX_STORED_WAYPOINTS] = [waypointData_t {
    targetname: [0; MAX_QPATH],
    target: [0; MAX_QPATH],
    target2: [0; MAX_QPATH],
    target3: [0; MAX_QPATH],
    target4: [0; MAX_QPATH],
    nodeID: 0,
}; MAX_STORED_WAYPOINTS];

/// `NIF_NONE` (b_local.h:302).
pub const NIF_NONE: i32 = 0x00000000;
/// `NIF_FAILED` (b_local.h:303) — failed to find a way to the goal.
pub const NIF_FAILED: i32 = 0x00000001;
/// `NIF_MACRO_NAV` (b_local.h:304) — using macro navigation.
pub const NIF_MACRO_NAV: i32 = 0x00000002;
/// `NIF_COLLISION` (b_local.h:305) — resolving collision with an entity.
pub const NIF_COLLISION: i32 = 0x00000004;
/// `NIF_BLOCKED` (b_local.h:306) — blocked from moving.
pub const NIF_BLOCKED: i32 = 0x00000008;

/// `navInfo_t` (b_local.h:314) — per-frame navigation working set: the entity that blocked us, the
/// desired move/path directions, the move distance, the path-clearance trace and the `NIF_*` flags.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct navInfo_t {
    pub blocker: *mut gentity_t,
    pub direction: vec3_t,
    pub pathDirection: vec3_t,
    pub distance: f32,
    pub trace: trace_t,
    pub flags: i32,
}

// --- ported leaves -------------------------------------------------------------------------------

/*
-------------------------
NPC_Blocked
-------------------------
*/

/// `void NPC_Blocked( gentity_t *self, gentity_t *blocker )` (g_nav.c:68). No-oracle: NPC AI state +
/// `G_ActivateBehavior`/`G_SetEnemy`.
pub unsafe fn NPC_Blocked(self_: *mut gentity_t, blocker: *mut gentity_t) {
    if (*self_).NPC.is_null() {
        return;
    }

    //Don't do this too often
    if (*(*self_).NPC).blockedSpeechDebounceTime > level.time {
        return;
    }

    //Attempt to run any blocked scripts
    if G_ActivateBehavior(self_, BSET_BLOCKED) == QTRUE {
        return;
    }

    //If this is one of our enemies, then just attack him
    if !(*blocker).client.is_null()
        && (*(*blocker).client).playerTeam == (*(*self_).client).enemyTeam
    {
        G_SetEnemy(self_, blocker);
        return;
    }

    //Debug_Printf( debugNPCAI, DEBUG_LEVEL_WARNING, "%s: Excuse me, %s %s!\n", self->targetname, blocker->classname, blocker->targetname );

    //If we're being blocked by the player, say something to them
    if (*blocker).s.number == 0 && (*(*blocker).client).playerTeam == (*(*self_).client).playerTeam
    {
        //guys in formation are not trying to get to a critical point,
        //don't make them yell at the player (unless they have an enemy and
        //are in combat because BP thinks it sounds cool during battle)
        //NOTE: only imperials, misc crewmen and hazard team have these wav files now
        //G_AddVoiceEvent( self, Q_irand(EV_BLOCKED1, EV_BLOCKED3), 0 );
    }

    (*(*self_).NPC).blockedSpeechDebounceTime =
        level.time + MIN_BLOCKED_SPEECH_TIME + (random() * 4000.0) as i32;
    (*(*self_).NPC).blockingEntNum = (*blocker).s.number;
}

/*
-------------------------
NAV_HitNavGoal
-------------------------
*/

/// `qboolean NAV_HitNavGoal( vec3_t point, vec3_t mins, vec3_t maxs, vec3_t dest, int radius,
/// qboolean flying )` (g_nav.c:167). Pure vector math — oracle-tested.
pub fn NAV_HitNavGoal(
    point: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    dest: &vec3_t,
    mut radius: i32,
    flying: qboolean,
) -> qboolean {
    let mut dmins: vec3_t = [0.0; 3];
    let mut dmaxs: vec3_t = [0.0; 3];
    let mut pmins: vec3_t = [0.0; 3];
    let mut pmaxs: vec3_t = [0.0; 3];

    if (radius & NAVGOAL_USE_RADIUS) != 0 {
        radius &= !NAVGOAL_USE_RADIUS;
        //NOTE:  This needs to do a DistanceSquared on navgoals that had
        //			a radius manually set! We can't do the smaller navgoals against
        //			walls to get around this because player-sized traces to them
        //			from angles will not work... - MCG
        if flying == QFALSE {
            //Allow for a little z difference
            let mut diff: vec3_t = [0.0; 3];
            VectorSubtract(point, dest, &mut diff);
            if diff[2].abs() <= 24.0 {
                diff[2] = 0.0;
            }
            return (VectorLengthSquared(&diff) <= (radius * radius) as f32) as qboolean;
        } else {
            //must hit exactly
            return (DistanceSquared(dest, point) <= (radius * radius) as f32) as qboolean;
        }
        //There is probably a better way to do this, either by preserving the original
        //		mins and maxs of the navgoal and doing this check ONLY if the radius
        //		is non-zero (like the original implementation) or some boolean to
        //		tell us to do this check rather than the fake bbox overlap check...
    } else {
        //Construct a dummy bounding box from our radius value
        VectorSet(&mut dmins, -radius as f32, -radius as f32, -radius as f32);
        VectorSet(&mut dmaxs, radius as f32, radius as f32, radius as f32);

        //Translate it
        let dmins_in = dmins;
        VectorAdd(&dmins_in, dest, &mut dmins);
        let dmaxs_in = dmaxs;
        VectorAdd(&dmaxs_in, dest, &mut dmaxs);

        //Translate the starting box
        VectorAdd(point, mins, &mut pmins);
        VectorAdd(point, maxs, &mut pmaxs);

        //See if they overlap
        G_BoundsOverlap(&pmins, &pmaxs, &dmins, &dmaxs)
    }
}

/*
-------------------------
NAV_ClearBlockedInfo
-------------------------
*/

/// `void NAV_ClearBlockedInfo( gentity_t *self )` (g_nav.c:416). No-oracle: NPC-struct mutation.
pub unsafe fn NAV_ClearBlockedInfo(self_: *mut gentity_t) {
    (*(*self_).NPC).aiFlags &= !NPCAI_BLOCKED;
    (*(*self_).NPC).blockingEntNum = ENTITYNUM_WORLD;
}

/*
-------------------------
NAV_SetBlockedInfo
-------------------------
*/

/// `void NAV_SetBlockedInfo( gentity_t *self, int entId )` (g_nav.c:428). No-oracle: NPC-struct
/// mutation.
pub unsafe fn NAV_SetBlockedInfo(self_: *mut gentity_t, ent_id: i32) {
    (*(*self_).NPC).aiFlags |= NPCAI_BLOCKED;
    (*(*self_).NPC).blockingEntNum = ent_id;
}

/*
-------------------------
NAV_CheckAhead
-------------------------
*/

/// `qboolean NAV_CheckAhead( gentity_t *self, vec3_t end, trace_t *trace, int clipmask )`
/// (g_nav.c:494). No-oracle: `trap_Trace` + door predicates + entity state.
pub unsafe fn NAV_CheckAhead(
    self_: *mut gentity_t,
    end: &vec3_t,
    trace: &mut trace_t,
    mut clipmask: i32,
) -> qboolean {
    let mut mins: vec3_t = [0.0; 3];

    //Offset the step height
    VectorSet(
        &mut mins,
        (*self_).r.mins[0],
        (*self_).r.mins[1],
        (*self_).r.mins[2] + STEPSIZE as f32,
    );

    *trace = trap::Trace(
        &(*self_).r.currentOrigin,
        &mins,
        &(*self_).r.maxs,
        end,
        (*self_).s.number,
        clipmask,
    );

    if trace.startsolid != 0 && (trace.contents & CONTENTS_BOTCLIP) != 0 {
        //started inside do not enter, so ignore them
        clipmask &= !CONTENTS_BOTCLIP;
        *trace = trap::Trace(
            &(*self_).r.currentOrigin,
            &mins,
            &(*self_).r.maxs,
            end,
            (*self_).s.number,
            clipmask,
        );
    }
    //Do a simple check
    if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction == 1.0 {
        return QTRUE;
    }

    //See if we're too far above
    if ((*self_).r.currentOrigin[2] - end[2]).abs() > 48.0 {
        return QFALSE;
    }

    //This is a work around
    let radius = if (*self_).r.maxs[0] > (*self_).r.maxs[1] {
        (*self_).r.maxs[0]
    } else {
        (*self_).r.maxs[1]
    };
    let dist = Distance(&(*self_).r.currentOrigin, end);
    let t_frac = 1.0 - (radius / dist);

    if trace.fraction >= t_frac {
        return QTRUE;
    }

    //Do a special check for doors
    if (trace.entityNum as i32) < ENTITYNUM_WORLD {
        let blocker =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);

        if !(*blocker).classname.is_null() && *(*blocker).classname != 0 {
            if G_EntIsUnlockedDoor((*blocker).s.number) == QTRUE
            //if ( Q_stricmp( blocker->classname, "func_door" ) == 0 )
            {
                //We're too close, try and avoid the door (most likely stuck on a lip)
                if DistanceSquared(&(*self_).r.currentOrigin, &trace.endpos)
                    < MIN_DOOR_BLOCK_DIST_SQR
                {
                    return QFALSE;
                }

                return QTRUE;
            }
        }
    }

    QFALSE
}

/*
-------------------------
NAV_TestBestNode
-------------------------
*/

/// `int NAV_TestBestNode( gentity_t *self, int startID, int endID, qboolean failEdge )`
/// (g_nav.c:967) — check the path to a node against architecture only. No-oracle: `trap_Nav_*` +
/// `trap_Trace` + door/breakable/usable predicates.
pub unsafe fn NAV_TestBestNode(
    self_: *mut gentity_t,
    start_id: i32,
    end_id: i32,
    fail_edge: qboolean,
) -> i32 {
    //check only against architectrure
    let mut end: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut clipmask = ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP;

    //get the position for the test choice
    trap::Nav_GetNodePosition(end_id, &mut end);

    //Offset the step height
    VectorSet(
        &mut mins,
        (*self_).r.mins[0],
        (*self_).r.mins[1],
        (*self_).r.mins[2] + STEPSIZE as f32,
    );

    let mut trace = trap::Trace(
        &(*self_).r.currentOrigin,
        &mins,
        &(*self_).r.maxs,
        &end,
        (*self_).s.number,
        clipmask,
    );

    if trace.startsolid != 0 && (trace.contents & CONTENTS_BOTCLIP) != 0 {
        //started inside do not enter, so ignore them
        clipmask &= !CONTENTS_BOTCLIP;
        trace = trap::Trace(
            &(*self_).r.currentOrigin,
            &mins,
            &(*self_).r.maxs,
            &end,
            (*self_).s.number,
            clipmask,
        );
    }
    //Do a simple check
    if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction == 1.0 {
        //it's clear
        return end_id;
    }

    //See if we're too far above
    if (*self_).s.weapon != WP_SABER && ((*self_).r.currentOrigin[2] - end[2]).abs() > 48.0 {
    } else {
        //This is a work around
        let radius = if (*self_).r.maxs[0] > (*self_).r.maxs[1] {
            (*self_).r.maxs[0]
        } else {
            (*self_).r.maxs[1]
        };
        let dist = Distance(&(*self_).r.currentOrigin, &end);
        let t_frac = 1.0 - (radius / dist);

        if trace.fraction >= t_frac {
            //it's clear
            return end_id;
        }
    }

    //Do a special check for doors
    if (trace.entityNum as i32) < ENTITYNUM_WORLD {
        let blocker =
            (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(trace.entityNum as usize);

        if !(*blocker).classname.is_null() && *(*blocker).classname != 0 {
            //special case: doors are architecture, but are dynamic, like entitites
            if G_EntIsUnlockedDoor((*blocker).s.number) == QTRUE
            //if ( Q_stricmp( blocker->classname, "func_door" ) == 0 )
            {
                //it's unlocked, go for it
                //We're too close, try and avoid the door (most likely stuck on a lip)
                if DistanceSquared(&(*self_).r.currentOrigin, &trace.endpos)
                    < MIN_DOOR_BLOCK_DIST_SQR
                {
                    return start_id;
                }
                //we can keep heading to the door, it should open
                if (*self_).s.weapon != WP_SABER
                    && ((*self_).r.currentOrigin[2] - end[2]).abs() > 48.0
                {
                    //too far above
                } else {
                    return end_id;
                }
            } else if G_EntIsDoor((*blocker).s.number) == QTRUE {
                //a locked door!
                //path is blocked by a locked door, mark it as such if instructed to do so
                if fail_edge != QFALSE {
                    trap::Nav_AddFailedEdge((*self_).s.number, start_id, end_id);
                }
            } else if G_EntIsBreakable((*blocker).s.number) == QTRUE {
                //do same for breakable brushes/models/glass?
                //path is blocked by a breakable, mark it as such if instructed to do so
                if fail_edge != QFALSE {
                    trap::Nav_AddFailedEdge((*self_).s.number, start_id, end_id);
                }
            } else if G_EntIsRemovableUsable((*blocker).s.number) == QTRUE {
                //and removable usables
                //path is blocked by a removable usable, mark it as such if instructed to do so
                if fail_edge != QFALSE {
                    trap::Nav_AddFailedEdge((*self_).s.number, start_id, end_id);
                }
            } else if !(*blocker).targetname.is_null()
                && (*blocker).s.solid == SOLID_BMODEL
                && (((*blocker).r.contents & CONTENTS_MONSTERCLIP) != 0
                    || ((*blocker).r.contents & CONTENTS_BOTCLIP) != 0)
            {
                //some other kind of do not enter entity brush that will probably be removed
                //path is blocked by a removable brushent, mark it as such if instructed to do so
                if fail_edge != QFALSE {
                    trap::Nav_AddFailedEdge((*self_).s.number, start_id, end_id);
                }
            }
        }
    }
    //path is blocked
    //use the fallback choice
    start_id
}

/*
-------------------------
NAV_GetNearestNode
-------------------------
*/

/// `int NAV_GetNearestNode( gentity_t *self, int lastNode )` (g_nav.c:1079). Thin wrapper over the
/// `trap_Nav_GetNearestNode` syscall.
pub unsafe fn NAV_GetNearestNode(self_: *mut gentity_t, last_node: i32) -> i32 {
    trap::Nav_GetNearestNode(self_, last_node, NF_CLEAR_PATH, WAYPOINT_NONE)
}

/*
-------------------------
NPC_SetMoveGoal
-------------------------
*/

/// `void NPC_SetMoveGoal( gentity_t *ent, vec3_t point, int radius, qboolean isNavGoal,
/// int combatPoint, gentity_t *targetEnt )` (g_nav.c:112). No-oracle: NPC goal-entity setup ending
/// in `trap_LinkEntity`.
pub unsafe fn NPC_SetMoveGoal(
    ent: *mut gentity_t,
    point: &vec3_t,
    radius: i32,
    is_nav_goal: qboolean,
    combat_point: i32,
    target_ent: *mut gentity_t,
) {
    //Must be an NPC
    if (*ent).NPC.is_null() {
        return;
    }

    if (*(*ent).NPC).tempGoal.is_null() {
        //must still have a goal
        return;
    }

    //Copy the origin
    //VectorCopy( point, ent->NPC->goalPoint );	//FIXME: Make it use this, and this alone!
    VectorCopy(point, &mut (*(*(*ent).NPC).tempGoal).r.currentOrigin);

    //Copy the mins and maxs to the tempGoal
    VectorCopy(&(*ent).r.mins, &mut (*(*(*ent).NPC).tempGoal).r.mins);
    VectorCopy(&(*ent).r.mins, &mut (*(*(*ent).NPC).tempGoal).r.maxs);

    (*(*(*ent).NPC).tempGoal).target = core::ptr::null_mut();
    (*(*(*ent).NPC).tempGoal).clipmask = (*ent).clipmask;
    (*(*(*ent).NPC).tempGoal).flags &= !FL_NAVGOAL;
    if !target_ent.is_null() && (*target_ent).waypoint >= 0 {
        (*(*(*ent).NPC).tempGoal).waypoint = (*target_ent).waypoint;
    } else {
        (*(*(*ent).NPC).tempGoal).waypoint = WAYPOINT_NONE;
    }
    (*(*(*ent).NPC).tempGoal).noWaypointTime = 0;

    if is_nav_goal != QFALSE {
        debug_assert!(!(*(*(*ent).NPC).tempGoal).parent.is_null());
        (*(*(*ent).NPC).tempGoal).flags |= FL_NAVGOAL;
    }

    (*(*(*ent).NPC).tempGoal).combatPoint = combat_point;
    (*(*(*ent).NPC).tempGoal).enemy = target_ent;

    (*(*ent).NPC).goalEntity = (*(*ent).NPC).tempGoal;
    (*(*ent).NPC).goalRadius = radius;

    trap::LinkEntity((*(*ent).NPC).goalEntity);
}

/*
-------------------------
NAV_TestBypass
-------------------------
*/

/// `static qboolean NAV_TestBypass( gentity_t *self, float yaw, float blocked_dist,
/// vec3_t movedir )` (g_nav.c:555). No-oracle: `AngleVectors` + `NAV_CheckAhead` probe.
unsafe fn NAV_TestBypass(
    self_: *mut gentity_t,
    yaw: f32,
    blocked_dist: f32,
    movedir: &mut vec3_t,
) -> qboolean {
    let mut tr = trace_t::default();
    let mut avoid_angles: vec3_t = [0.0; 3];
    let mut block_test: vec3_t = [0.0; 3];
    let mut block_pos: vec3_t = [0.0; 3];

    VectorClear(&mut avoid_angles);
    avoid_angles[YAW as usize] = yaw;

    AngleVectors(&avoid_angles, Some(&mut block_test), None, None);
    VectorMA(
        &(*self_).r.currentOrigin,
        blocked_dist,
        &block_test,
        &mut block_pos,
    );

    //NAVDEBUG_showCollision debug graphics (no-op stubs in JKA) elided

    //See if we're clear to move in that direction
    if NAV_CheckAhead(
        self_,
        &block_pos,
        &mut tr,
        ((*self_).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
    ) == QTRUE
    {
        VectorCopy(&block_test, movedir);

        return QTRUE;
    }

    QFALSE
}

/*
-------------------------
NAV_Bypass
-------------------------
*/

/// `qboolean NAV_Bypass( gentity_t *self, gentity_t *blocker, vec3_t blocked_dir, float blocked_dist,
/// vec3_t movedir )` (g_nav.c:589). No-oracle: arc-tested `NAV_TestBypass`/`NAV_CheckAhead` probes.
pub unsafe fn NAV_Bypass(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: &vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
) -> qboolean {
    let mut dot: f32;
    let yaw: f32;
    let avoid_radius: f32;
    let mut arc_angle: f32;
    let mut right: vec3_t = [0.0; 3];

    //NAVDEBUG_showCollision debug graphics (no-op stubs in JKA) elided

    AngleVectors(&(*self_).r.currentAngles, None, Some(&mut right), None);

    //Get the blocked direction
    yaw = vectoyaw(blocked_dir);

    //Get the avoid radius
    avoid_radius = ((*blocker).r.maxs[0] * (*blocker).r.maxs[0]
        + (*blocker).r.maxs[1] * (*blocker).r.maxs[1])
        .sqrt()
        + ((*self_).r.maxs[0] * (*self_).r.maxs[0] + (*self_).r.maxs[1] * (*self_).r.maxs[1])
            .sqrt();

    //See if we're inside our avoidance radius
    arc_angle = if blocked_dist <= avoid_radius {
        135.0
    } else {
        (avoid_radius / blocked_dist) * 90.0
    };

    //FIXME: Although the below code will cause the NPC to take the "better" route, it can cause NPCs to become stuck on
    //		 one another in certain situations where both decide to take the same direction.

    //Check to see what dir the other guy is moving in (if any) and pick the opposite dir
    if !(*blocker).client.is_null()
        && VectorCompare(&(*(*blocker).client).ps.velocity, &vec3_origin) == QFALSE
    {
        let mut blocker_movedir: vec3_t = [0.0; 3];
        VectorNormalize2(&(*(*blocker).client).ps.velocity, &mut blocker_movedir);
        dot = DotProduct(&blocker_movedir, blocked_dir);
        if dot < 0.35 && dot > -0.35 {
            //he's moving to the side of me
            let mut block_pos: vec3_t = [0.0; 3];
            let mut tr = trace_t::default();
            let blocker_movedir_in = blocker_movedir;
            VectorScale(&blocker_movedir_in, -1.0, &mut blocker_movedir);
            VectorMA(
                &(*self_).r.currentOrigin,
                blocked_dist,
                &blocker_movedir,
                &mut block_pos,
            );
            if NAV_CheckAhead(
                self_,
                &block_pos,
                &mut tr,
                ((*self_).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
            ) == QTRUE
            {
                VectorCopy(&blocker_movedir, movedir);
                return QTRUE;
            }
        }
    }

    //FIXME: this makes NPCs stack up and ping-pong like crazy.
    //			Need to keep track of this and stop trying after a while
    dot = DotProduct(blocked_dir, &right);

    //Go right on the first try if that works better
    if dot < 0.0 {
        arc_angle *= -1.0;
    }

    //Test full, best position first
    if NAV_TestBypass(
        self_,
        AngleNormalize360(yaw + arc_angle),
        blocked_dist,
        movedir,
    ) == QTRUE
    {
        return QTRUE;
    }

    //Try a smaller arc
    if NAV_TestBypass(
        self_,
        AngleNormalize360(yaw + (arc_angle * 0.5)),
        blocked_dist,
        movedir,
    ) == QTRUE
    {
        return QTRUE;
    }

    //Try the other direction
    if NAV_TestBypass(
        self_,
        AngleNormalize360(yaw + (arc_angle * -1.0)),
        blocked_dist,
        movedir,
    ) == QTRUE
    {
        return QTRUE;
    }

    //Try the other direction more precisely
    if NAV_TestBypass(
        self_,
        AngleNormalize360(yaw + ((arc_angle * -1.0) * 0.5)),
        blocked_dist,
        movedir,
    ) == QTRUE
    {
        return QTRUE;
    }

    //Unable to go around
    QFALSE
}

/*
-------------------------
NAV_MoveBlocker
-------------------------
*/

/// `qboolean NAV_MoveBlocker( gentity_t *self, vec3_t shove_dir )` (g_nav.c:669). No-oracle: a
/// temporary method for shoving a blocker — applies a lifted, angled velocity to the client.
pub unsafe fn NAV_MoveBlocker(self_: *mut gentity_t, shove_dir: &vec3_t) -> qboolean {
    //FIXME: This is a temporary method for making blockers move
    //FIXME: This will, of course, push blockers off of cliffs, into walls and all over the place

    let mut temp_dir: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];

    vectoangles(shove_dir, &mut temp_dir);

    temp_dir[YAW as usize] += 45.0;
    AngleVectors(&temp_dir, Some(&mut forward), None, None);

    VectorScale(&forward, SHOVE_SPEED, &mut (*(*self_).client).ps.velocity);
    (*(*self_).client).ps.velocity[2] += SHOVE_LIFT;

    //self->NPC->shoveDebounce = level.time + 100;

    QTRUE
}

/*
-------------------------
NAV_ResolveBlock
-------------------------
*/

/// `qboolean NAV_ResolveBlock( gentity_t *self, gentity_t *blocker, vec3_t blocked_dir )`
/// (g_nav.c:696). No-oracle: NPC blocked-state + `NPC_Blocked`/`NPC_FaceEntity`.
pub unsafe fn NAV_ResolveBlock(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    _blocked_dir: &vec3_t,
) -> qboolean {
    //Stop double waiting
    if !(*blocker).NPC.is_null() && (*(*blocker).NPC).blockingEntNum == (*self_).s.number {
        return QTRUE;
    }

    //For now, just complain about it
    NPC_Blocked(self_, blocker);
    NPC_FaceEntity(blocker, QTRUE);

    QFALSE
}

/*
-------------------------
NAV_TrueCollision
-------------------------
*/

/// `qboolean NAV_TrueCollision( gentity_t *self, gentity_t *blocker, vec3_t movedir,
/// vec3_t blocked_dir )` (g_nav.c:715). No-oracle: player-velocity overlap predictor.
pub unsafe fn NAV_TrueCollision(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: &vec3_t,
    blocked_dir: &mut vec3_t,
) -> qboolean {
    let mut velocity_dir: vec3_t = [0.0; 3];
    let speed: f32;
    let dot: f32;
    let mut test_pos: vec3_t = [0.0; 3];
    let mut ptmins: vec3_t = [0.0; 3];
    let mut ptmaxs: vec3_t = [0.0; 3];
    let mut tmins: vec3_t = [0.0; 3];
    let mut tmaxs: vec3_t = [0.0; 3];

    //TODO: Handle all ents
    if (*blocker).client.is_null() {
        return QFALSE;
    }

    //Get the player's move direction and speed
    speed = VectorNormalize2(&(*(*self_).client).ps.velocity, &mut velocity_dir);

    //See if it's even feasible
    dot = DotProduct(movedir, &velocity_dir);

    if dot < 0.85 {
        return QFALSE;
    }

    VectorMA(
        &(*self_).r.currentOrigin,
        speed * FRAMETIME as f32,
        &velocity_dir,
        &mut test_pos,
    );

    VectorAdd(&(*blocker).r.currentOrigin, &(*blocker).r.mins, &mut tmins);
    VectorAdd(&(*blocker).r.currentOrigin, &(*blocker).r.maxs, &mut tmaxs);

    VectorAdd(&test_pos, &(*self_).r.mins, &mut ptmins);
    VectorAdd(&test_pos, &(*self_).r.maxs, &mut ptmaxs);

    if G_BoundsOverlap(&ptmins, &ptmaxs, &tmins, &tmaxs) == QTRUE {
        VectorCopy(&velocity_dir, blocked_dir);
        return QTRUE;
    }

    QFALSE
}

/*
-------------------------
NAV_StackedCanyon
-------------------------
*/

/// `qboolean NAV_StackedCanyon( gentity_t *self, gentity_t *blocker, vec3_t pathDir )`
/// (g_nav.c:758). No-oracle: perpendicular-offset `trap_Trace` solidity probes.
pub unsafe fn NAV_StackedCanyon(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    path_dir: &vec3_t,
) -> qboolean {
    let mut perp: vec3_t = [0.0; 3];
    let mut cross: vec3_t = [0.0; 3];
    let mut test: vec3_t = [0.0; 3];
    let avoid_radius: f32;
    let mut extra_clip = CONTENTS_BOTCLIP;

    PerpendicularVector(&mut perp, path_dir);
    CrossProduct(path_dir, &perp, &mut cross);

    avoid_radius = ((*blocker).r.maxs[0] * (*blocker).r.maxs[0]
        + (*blocker).r.maxs[1] * (*blocker).r.maxs[1])
        .sqrt()
        + ((*self_).r.maxs[0] * (*self_).r.maxs[0] + (*self_).r.maxs[1] * (*self_).r.maxs[1])
            .sqrt();

    VectorMA(&(*blocker).r.currentOrigin, avoid_radius, &cross, &mut test);

    let mut tr = trap::Trace(
        &test,
        &(*self_).r.mins,
        &(*self_).r.maxs,
        &test,
        (*self_).s.number,
        (*self_).clipmask | extra_clip,
    );
    if tr.startsolid != 0 && (tr.contents & CONTENTS_BOTCLIP) != 0 {
        //started inside do not enter, so ignore them
        extra_clip &= !CONTENTS_BOTCLIP;
        tr = trap::Trace(
            &test,
            &(*self_).r.mins,
            &(*self_).r.maxs,
            &test,
            (*self_).s.number,
            (*self_).clipmask | extra_clip,
        );
    }

    //NAVDEBUG_showCollision debug graphics (no-op stubs in JKA) elided

    if tr.startsolid == 0 && tr.allsolid == 0 {
        return QFALSE;
    }

    VectorMA(
        &(*blocker).r.currentOrigin,
        -avoid_radius,
        &cross,
        &mut test,
    );

    let mut tr = trap::Trace(
        &test,
        &(*self_).r.mins,
        &(*self_).r.maxs,
        &test,
        (*self_).s.number,
        (*self_).clipmask | extra_clip,
    );
    if tr.startsolid != 0 && (tr.contents & CONTENTS_BOTCLIP) != 0 {
        //started inside do not enter, so ignore them
        extra_clip &= !CONTENTS_BOTCLIP;
        tr = trap::Trace(
            &test,
            &(*self_).r.mins,
            &(*self_).r.maxs,
            &test,
            (*self_).s.number,
            (*self_).clipmask | extra_clip,
        );
    }

    if tr.startsolid == 0 && tr.allsolid == 0 {
        return QFALSE;
    }

    //NAVDEBUG_showCollision debug graphics (no-op stubs in JKA) elided

    QTRUE
}

/*
-------------------------
NAV_ResolveEntityCollision
-------------------------
*/

/// `qboolean NAV_ResolveEntityCollision( gentity_t *self, gentity_t *blocker, vec3_t movedir,
/// vec3_t pathDir )` (g_nav.c:824). No-oracle: door-aware bypass/block resolution.
pub unsafe fn NAV_ResolveEntityCollision(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: &mut vec3_t,
    path_dir: &vec3_t,
) -> qboolean {
    let mut blocked_dir: vec3_t = [0.0; 3];
    let blocked_dist: f32;

    //Doors are ignored
    if G_EntIsUnlockedDoor((*blocker).s.number) == QTRUE
    //if ( Q_stricmp( blocker->classname, "func_door" ) == 0 )
    {
        if DistanceSquared(&(*self_).r.currentOrigin, &(*blocker).r.currentOrigin)
            > MIN_DOOR_BLOCK_DIST_SQR
        {
            return QTRUE;
        }
    }

    VectorSubtract(
        &(*blocker).r.currentOrigin,
        &(*self_).r.currentOrigin,
        &mut blocked_dir,
    );
    blocked_dist = VectorNormalize(&mut blocked_dir);

    //Make sure an actual collision is going to happen
    //	if ( NAV_PredictCollision( self, blocker, movedir, blocked_dir ) == qfalse )
    //		return qtrue;

    //See if we can get around the blocker at all (only for player!)
    if (*blocker).s.number == 0 {
        if NAV_StackedCanyon(self_, blocker, path_dir) == QTRUE {
            NPC_Blocked(self_, blocker);
            NPC_FaceEntity(blocker, QTRUE);

            return QFALSE;
        }
    }

    //First, attempt to walk around the blocker
    if NAV_Bypass(self_, blocker, &blocked_dir, blocked_dist, movedir) == QTRUE {
        return QTRUE;
    }

    //Second, attempt to calculate a good move position for the blocker
    if NAV_ResolveBlock(self_, blocker, &blocked_dir) == QTRUE {
        return QTRUE;
    }

    QFALSE
}

/*
-------------------------
NAV_TestForBlocked
-------------------------
*/

/// `qboolean NAV_TestForBlocked( gentity_t *self, gentity_t *goal, gentity_t *blocker,
/// float distance, int *flags )` (g_nav.c:873). No-oracle: detect a blocker standing on our goal.
pub unsafe fn NAV_TestForBlocked(
    self_: *mut gentity_t,
    goal: *mut gentity_t,
    blocker: *mut gentity_t,
    distance: f32,
    flags: &mut i32,
) -> qboolean {
    if goal.is_null() {
        return QFALSE;
    }

    if (*blocker).s.eType == ET_ITEM {
        return QFALSE;
    }

    if NAV_HitNavGoal(
        &(*blocker).r.currentOrigin,
        &(*blocker).r.mins,
        &(*blocker).r.maxs,
        &(*goal).r.currentOrigin,
        12,
        QFALSE,
    ) == QTRUE
    {
        *flags |= NIF_BLOCKED;

        if distance <= MIN_STOP_DIST {
            NPC_Blocked(self_, blocker);
            NPC_FaceEntity(blocker, QTRUE);
            return QTRUE;
        }
    }

    QFALSE
}

/*
-------------------------
NAV_AvoidCollsion
-------------------------
*/

/// `qboolean NAV_AvoidCollision( gentity_t *self, gentity_t *goal, navInfo_t *info )`
/// (g_nav.c:902). No-oracle: `NAV_CheckAhead` + the guarded-stubbed collision family + NPC state.
pub unsafe fn NAV_AvoidCollision(
    self_: *mut gentity_t,
    goal: *mut gentity_t,
    info: &mut navInfo_t,
) -> qboolean {
    let mut movedir: vec3_t = [0.0; 3];
    let mut movepos: vec3_t = [0.0; 3];

    //Clear our block info for this frame
    NAV_ClearBlockedInfo(NPC);

    //Cap our distance
    if info.distance > MAX_COLL_AVOID_DIST {
        info.distance = MAX_COLL_AVOID_DIST;
    }

    //Get an end position
    VectorMA(
        &(*self_).r.currentOrigin,
        info.distance,
        &info.direction,
        &mut movepos,
    );
    VectorCopy(&info.direction, &mut movedir);

    if !self_.is_null()
        && !(*self_).NPC.is_null()
        && ((*(*self_).NPC).aiFlags & NPCAI_NO_COLL_AVOID) != 0
    {
        //pretend there's no-one in the way
        return QTRUE;
    }
    //Now test against entities
    if NAV_CheckAhead(self_, &movepos, &mut info.trace, CONTENTS_BODY) == QFALSE {
        //Get the blocker
        info.blocker = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>())
            .add(info.trace.entityNum as usize);
        info.flags |= NIF_COLLISION;

        //Ok to hit our goal entity
        if goal == info.blocker {
            return QTRUE;
        }

        //See if we're moving along with them
        //if ( NAV_TrueCollision( self, info.blocker, movedir, info.direction ) == qfalse )
        //	return qtrue;

        //Test for blocking by standing on goal
        if NAV_TestForBlocked(self_, goal, info.blocker, info.distance, &mut info.flags) == QTRUE {
            return QFALSE;
        }

        //If the above function said we're blocked, don't do the extra checks
        if (info.flags & NIF_BLOCKED) != 0 {
            return QTRUE;
        }

        //See if we can get that entity to move out of our way
        if NAV_ResolveEntityCollision(self_, info.blocker, &mut movedir, &info.pathDirection)
            == QFALSE
        {
            return QFALSE;
        }

        VectorCopy(&movedir, &mut info.direction);

        return QTRUE;
    }

    //Our path is clear, just move there
    //if ( NAVDEBUG_showCollision ) { G_DrawEdge(...) } -- debug graphics are no-op stubs in JKA

    QTRUE
}

/*
-------------------------
NAV_MoveToGoal
-------------------------
*/

/// `int NAV_MoveToGoal( gentity_t *self, navInfo_t *info )` (g_nav.c:1109). No-oracle: drives the
/// `trap_Nav_*` pathfinder + `NAV_GetNearestNode`/`NAV_TestBestNode`/`NAV_CheckAhead`.
pub unsafe fn NAV_MoveToGoal(self_: *mut gentity_t, info: &mut navInfo_t) -> i32 {
    let mut bestNode: i32;
    let mut origin: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    //Must have a goal entity to move there
    if (*(*self_).NPC).goalEntity.is_null() {
        return WAYPOINT_NONE;
    }

    //Check special player optimizations
    if (*(*(*self_).NPC).goalEntity).s.number == 0 {
        //If we couldn't find the point, then we won't be able to this turn
        if (*(*(*self_).NPC).goalEntity).waypoint == WAYPOINT_NONE {
            return WAYPOINT_NONE;
        }

        //NOTENOTE: Otherwise trust this waypoint for the whole frame (reduce all unnecessary calculations)
    } else {
        //Find the target's waypoint
        let wp = NAV_GetNearestNode(
            (*(*self_).NPC).goalEntity,
            (*(*(*self_).NPC).goalEntity).waypoint,
        );
        (*(*(*self_).NPC).goalEntity).waypoint = wp;
        if wp == WAYPOINT_NONE {
            return WAYPOINT_NONE;
        }
    }

    //Find our waypoint
    let my_wp = NAV_GetNearestNode(self_, (*self_).lastWaypoint);
    (*self_).waypoint = my_wp;
    if my_wp == WAYPOINT_NONE {
        return WAYPOINT_NONE;
    }

    bestNode = trap::Nav_GetBestNode(
        (*self_).waypoint,
        (*(*(*self_).NPC).goalEntity).waypoint,
        NODE_NONE,
    );

    if bestNode == WAYPOINT_NONE {
        //if ( NAVDEBUG_showEnemyPath ) { ... debug graphics no-op ... }
        return WAYPOINT_NONE;
    }

    //Check this node
    bestNode = NAV_TestBestNode(
        self_,
        bestNode,
        (*(*(*self_).NPC).goalEntity).waypoint,
        QFALSE,
    );

    //trace_t	trace;

    //Get this position
    trap::Nav_GetNodePosition(bestNode, &mut origin);
    trap::Nav_GetNodePosition((*self_).waypoint, &mut end);

    //Basically, see if the path we have isn't helping
    //if ( NAV_MicroError( origin, end ) )
    //	return WAYPOINT_NONE;

    //Test the path connection from our current position to the best node
    if NAV_CheckAhead(
        self_,
        &origin,
        &mut info.trace,
        ((*self_).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
    ) == QFALSE
    {
        //First attempt to move to the closest point on the line between the waypoints
        let origin_in = origin;
        crate::codemp::game::q_math::G_FindClosestPointOnLineSegment(
            &origin_in,
            &end,
            &(*self_).r.currentOrigin,
            &mut origin,
        );

        //See if we can go there
        if NAV_CheckAhead(
            self_,
            &origin,
            &mut info.trace,
            ((*self_).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
        ) == QFALSE
        {
            //Just move towards our current waypoint
            bestNode = (*self_).waypoint;
            trap::Nav_GetNodePosition(bestNode, &mut origin);
        }
    }

    //Setup our new move information
    VectorSubtract(&origin, &(*self_).r.currentOrigin, &mut info.direction);
    info.distance = VectorNormalize(&mut info.direction);

    VectorSubtract(&end, &origin, &mut info.pathDirection);
    VectorNormalize(&mut info.pathDirection);

    //Draw any debug info, if requested
    //if ( NAVDEBUG_showEnemyPath ) { ... debug graphics no-op ... }

    bestNode
}

/*
-------------------------
FlyingCreature
-------------------------
*/

/// `qboolean FlyingCreature( gentity_t *ent )` (g_nav.c:46) — true if the ent is a client whose
/// gravity has been zeroed (or made negative), i.e. it floats. No-oracle: entity-state read.
pub unsafe fn FlyingCreature(ent: *mut gentity_t) -> qboolean {
    if !(*ent).client.is_null() && (*(*ent).client).ps.gravity <= 0 {
        return QTRUE;
    }
    QFALSE
}

/*
-------------------------
NAV_ClearPathToPoint
-------------------------
*/

/// `qboolean NAV_ClearPathToPoint( gentity_t *self, vec3_t pmins, vec3_t pmaxs, vec3_t point,
/// int clipmask, int okToHitEntNum )` (g_nav.c:222). No-oracle: `trap_InPVS`/`trap_Trace` +
/// `NAV_HitNavGoal`/`FlyingCreature` + NPC/navgoal state.
pub unsafe fn NAV_ClearPathToPoint(
    self_: *mut gentity_t,
    pmins: &vec3_t,
    pmaxs: &vec3_t,
    point: &vec3_t,
    mut clipmask: i32,
    ok_to_hit_ent_num: i32,
) -> qboolean {
    //	trace_t	trace;
    //	return NAV_CheckAhead( self, point, trace, clipmask|CONTENTS_BOTCLIP );

    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    //Test if they're even conceivably close to one another
    if trap::InPVS(&(*self_).r.currentOrigin, point) == QFALSE {
        return QFALSE;
    }

    if ((*self_).flags & FL_NAVGOAL) != 0 {
        if (*self_).parent.is_null() {
            //SHOULD NEVER HAPPEN!!!
            return QFALSE;
        }
        VectorCopy(&(*(*self_).parent).r.mins, &mut mins);
        VectorCopy(&(*(*self_).parent).r.maxs, &mut maxs);
    } else {
        VectorCopy(pmins, &mut mins);
        VectorCopy(pmaxs, &mut maxs);
    }

    if !(*self_).client.is_null() || ((*self_).flags & FL_NAVGOAL) != 0 {
        //Clients can step up things, or if this is a navgoal check, a client will be using this info
        mins[2] += STEPSIZE as f32;

        //don't let box get inverted
        if mins[2] > maxs[2] {
            mins[2] = maxs[2];
        }
    }

    if ((*self_).flags & FL_NAVGOAL) != 0 {
        //Trace from point to navgoal
        let mut tr = trap::Trace(
            point,
            &mins,
            &maxs,
            &(*self_).r.currentOrigin,
            (*(*self_).parent).s.number,
            (clipmask | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP) & !CONTENTS_BODY,
        );
        if tr.startsolid != 0 && (tr.contents & CONTENTS_BOTCLIP) != 0 {
            //started inside do not enter, so ignore them
            clipmask &= !CONTENTS_BOTCLIP;
            tr = trap::Trace(
                point,
                &mins,
                &maxs,
                &(*self_).r.currentOrigin,
                (*(*self_).parent).s.number,
                (clipmask | CONTENTS_MONSTERCLIP) & !CONTENTS_BODY,
            );
        }

        if tr.startsolid != 0 || tr.allsolid != 0 {
            return QFALSE;
        }

        //Made it
        if tr.fraction == 1.0 {
            return QTRUE;
        }

        if ok_to_hit_ent_num != ENTITYNUM_NONE && tr.entityNum as i32 == ok_to_hit_ent_num {
            return QTRUE;
        }

        //Okay, didn't get all the way there, let's see if we got close enough:
        if NAV_HitNavGoal(
            &(*self_).r.currentOrigin,
            &(*(*self_).parent).r.mins,
            &(*(*self_).parent).r.maxs,
            &tr.endpos,
            (*NPCInfo).goalRadius,
            FlyingCreature((*self_).parent),
        ) == QTRUE
        {
            return QTRUE;
        }
        //else: NAVDEBUG_showCollision debug graphics (no-op stubs in JKA) elided
    } else {
        let mut tr = trap::Trace(
            &(*self_).r.currentOrigin,
            &mins,
            &maxs,
            point,
            (*self_).s.number,
            clipmask | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
        );
        if tr.startsolid != 0 && (tr.contents & CONTENTS_BOTCLIP) != 0 {
            //started inside do not enter, so ignore them
            clipmask &= !CONTENTS_BOTCLIP;
            tr = trap::Trace(
                &(*self_).r.currentOrigin,
                &mins,
                &maxs,
                point,
                (*self_).s.number,
                clipmask | CONTENTS_MONSTERCLIP,
            );
        }

        if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction == 1.0 {
            //FIXME: check for drops
            return QTRUE;
        }

        if ok_to_hit_ent_num != ENTITYNUM_NONE && tr.entityNum as i32 == ok_to_hit_ent_num {
            return QTRUE;
        }
        //NAVDEBUG_showCollision debug graphics (no-op stubs in JKA) elided
    }

    QFALSE
}

/*
-------------------------
NAV_FindClosestWaypointForEnt
-------------------------
*/

/// `int NAV_FindClosestWaypointForEnt( gentity_t *ent, int targWp )` (g_nav.c:352).
pub unsafe fn NAV_FindClosestWaypointForEnt(ent: *mut gentity_t, targ_wp: i32) -> i32 {
    //FIXME: Take the target into account
    trap::Nav_GetNearestNode(ent, (*ent).waypoint, NF_CLEAR_PATH, targ_wp)
}

/// `int NAV_FindClosestWaypointForPoint( gentity_t *ent, vec3_t point )` (g_nav.c:358).
pub unsafe fn NAV_FindClosestWaypointForPoint(ent: *mut gentity_t, point: &vec3_t) -> i32 {
    //FIXME: can we make this a static ent?
    let marker = G_Spawn();

    if marker.is_null() {
        return WAYPOINT_NONE;
    }

    G_SetOrigin(marker, point);

    VectorCopy(&(*ent).r.mins, &mut (*marker).r.mins); //stepsize?
    VectorCopy(&(*ent).r.mins, &mut (*marker).r.maxs); //crouching?

    (*marker).clipmask = (*ent).clipmask;
    (*marker).waypoint = WAYPOINT_NONE;

    let best_wp =
        trap::Nav_GetNearestNode(marker, (*marker).waypoint, NF_CLEAR_PATH, WAYPOINT_NONE);

    G_FreeEntity(marker);

    best_wp
}

/// `int NAV_FindClosestWaypointForPoint2( vec3_t point )` (g_nav.c:384).
pub unsafe fn NAV_FindClosestWaypointForPoint2(point: &vec3_t) -> i32 {
    //FIXME: can we make this a static ent?
    let marker = G_Spawn();

    if marker.is_null() {
        return WAYPOINT_NONE;
    }

    G_SetOrigin(marker, point);

    VectorSet(&mut (*marker).r.mins, -16.0, -16.0, -6.0); //includes stepsize
    VectorSet(&mut (*marker).r.maxs, 16.0, 16.0, 32.0);

    (*marker).clipmask = MASK_NPCSOLID;
    (*marker).waypoint = WAYPOINT_NONE;

    let best_wp =
        trap::Nav_GetNearestNode(marker, (*marker).waypoint, NF_CLEAR_PATH, WAYPOINT_NONE);

    G_FreeEntity(marker);

    best_wp
}

/*
-------------------------
NAV_Steer
-------------------------
*/

/// `int NAV_Steer( gentity_t *self, vec3_t dir, float distance )` (g_nav.c:440). No-oracle:
/// `AngleVectors` + `NAV_CheckAhead`. NOTE: the C returns the deviation yaw via an implicit
/// float→int truncation.
pub unsafe fn NAV_Steer(self_: *mut gentity_t, dir: &vec3_t, distance: f32) -> i32 {
    let mut right_test: vec3_t = [0.0; 3];
    let mut left_test: vec3_t = [0.0; 3];
    let mut deviation: vec3_t = [0.0; 3];
    let mut tr: trace_t;
    let right_push: f32;
    let left_push: f32;
    let right_ang = dir[YAW as usize] + 45.0;
    let left_ang = dir[YAW as usize] - 45.0;

    //Get the steering angles
    VectorCopy(dir, &mut deviation);
    deviation[YAW as usize] = right_ang;

    AngleVectors(&deviation, Some(&mut right_test), None, None);

    deviation[YAW as usize] = left_ang;

    AngleVectors(&deviation, Some(&mut left_test), None, None);

    //Find the end positions
    let right_in = right_test;
    VectorMA(
        &(*self_).r.currentOrigin,
        distance,
        &right_in,
        &mut right_test,
    );
    let left_in = left_test;
    VectorMA(
        &(*self_).r.currentOrigin,
        distance,
        &left_in,
        &mut left_test,
    );

    //NAVDEBUG_showCollision debug graphics (no-op stubs in JKA) elided

    //Find the right influence
    tr = trace_t::default();
    NAV_CheckAhead(
        self_,
        &right_test,
        &mut tr,
        (*self_).clipmask | CONTENTS_BOTCLIP,
    );

    right_push = -45.0 * (1.0 - tr.fraction);

    //Find the left influence
    NAV_CheckAhead(
        self_,
        &left_test,
        &mut tr,
        (*self_).clipmask | CONTENTS_BOTCLIP,
    );

    left_push = 45.0 * (1.0 - tr.fraction);

    //Influence the mover to respond to the steering
    VectorCopy(dir, &mut deviation);
    deviation[YAW as usize] += left_push + right_push;

    deviation[YAW as usize] as i32
}

/*
-------------------------
NAV_MicroError
-------------------------
*/

/// `qboolean NAV_MicroError( vec3_t start, vec3_t end )` (g_nav.c:1090). No-oracle: reads the `NPC`
/// AI-core global.
pub unsafe fn NAV_MicroError(start: &vec3_t, end: &vec3_t) -> qboolean {
    if VectorCompare(start, end) != QFALSE {
        if DistanceSquared(&(*NPC).r.currentOrigin, start) < (8 * 8) as f32 {
            return QTRUE;
        }
    }

    QFALSE
}

/*
-------------------------
waypoint_testDirection
-------------------------
*/

/// `unsigned int waypoint_testDirection( vec3_t origin, float yaw, unsigned int minDist )`
/// (g_nav.c:1216). No-oracle: a single `trap_Trace` along a yaw, scaled by the hit fraction.
pub fn waypoint_testDirection(origin: &vec3_t, yaw: f32, min_dist: u32) -> u32 {
    let mut trace_dir: vec3_t = [0.0; 3];
    let mut test_pos: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];

    //Setup the mins and max
    VectorSet(&mut maxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);
    VectorSet(
        &mut mins,
        -15.0,
        -15.0,
        DEFAULT_MINS_2 as f32 + STEPSIZE as f32,
    );

    //Get our test direction
    VectorSet(&mut angles, 0.0, yaw, 0.0);
    AngleVectors(&angles, Some(&mut trace_dir), None, None);

    //Move ahead
    //	VectorMA( origin, MAX_RADIUS_CHECK, trace_dir, test_pos );
    VectorMA(origin, min_dist as f32, &trace_dir, &mut test_pos);

    let tr = trap::Trace(
        origin,
        &mins,
        &maxs,
        &test_pos,
        ENTITYNUM_NONE,
        CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
    );

    //return (unsigned int) ( (float) MAX_RADIUS_CHECK * tr.fraction );
    (min_dist as f32 * tr.fraction) as u32
}

/*
-------------------------
waypoint_getRadius
-------------------------
*/

/// `unsigned int waypoint_getRadius( gentity_t *ent )` (g_nav.c:1247). No-oracle: minimum clear
/// distance over `YAW_ITERATIONS` directions.
pub unsafe fn waypoint_getRadius(ent: *mut gentity_t) -> u32 {
    let mut min_dist: u32 = MAX_RADIUS_CHECK + 1; // (unsigned int) -1;

    for i in 0..YAW_ITERATIONS {
        let dist = waypoint_testDirection(
            &(*ent).r.currentOrigin,
            (360.0 / YAW_ITERATIONS as f32) * i as f32,
            min_dist,
        );

        if dist < min_dist {
            min_dist = dist;
        }
    }

    min_dist
}

/*
-------------------------
NAV_Shutdown
-------------------------
*/

/// `void NAV_Shutdown( void )` (g_nav.c:1857). Thin wrapper over the `trap_Nav_Free` syscall.
pub fn NAV_Shutdown() {
    trap::Nav_Free();
}

// --- debug-graphics stubs (g_nav.c:9-44) ---------------------------------------------------------
//For debug graphics
//rwwFIXMEFIXME: Write these at some point for the sake of being able to debug visually

/// `void G_Line( vec3_t start, vec3_t end, vec3_t color, float alpha )` (g_nav.c:11). Empty debug
/// stub in the C original — no-oracle.
pub fn G_Line(_start: &vec3_t, _end: &vec3_t, _color: &vec3_t, _alpha: f32) {}

/// `void G_Cube( vec3_t mins, vec3_t maxs, vec3_t color, float alpha )` (g_nav.c:16). Empty debug
/// stub in the C original — no-oracle.
pub fn G_Cube(_mins: &vec3_t, _maxs: &vec3_t, _color: &vec3_t, _alpha: f32) {}

/// `void G_CubeOutline( vec3_t mins, vec3_t maxs, int time, unsigned int color, float alpha )`
/// (g_nav.c:21). Empty debug stub in the C original — no-oracle.
pub fn G_CubeOutline(_mins: &vec3_t, _maxs: &vec3_t, _time: i32, _color: u32, _alpha: f32) {}

/// `void G_DrawEdge( vec3_t start, vec3_t end, int type )` (g_nav.c:26). Empty debug stub in the
/// C original — no-oracle.
pub fn G_DrawEdge(_start: &vec3_t, _end: &vec3_t, _type: i32) {}

/// `void G_DrawNode( vec3_t origin, int type )` (g_nav.c:31). Empty debug stub in the C original —
/// no-oracle. (Distinct from `g_navnew.rs`'s file-local copy — this is g_nav.c's own.)
pub fn G_DrawNode(_origin: &vec3_t, _type: i32) {}

/// `void G_DrawCombatPoint( vec3_t origin, int type )` (g_nav.c:36). Empty debug stub in the C
/// original — no-oracle.
pub fn G_DrawCombatPoint(_origin: &vec3_t, _type: i32) {}

/// `void TAG_ShowTags( int flags )` (g_nav.c:41). Empty debug stub in the C original — no-oracle.
pub fn TAG_ShowTags(_flags: i32) {}

/*QUAKED waypoint  (0.7 0.7 0) (-16 -16 -24) (16 16 32) SOLID_OK
a place to go.

SOLID_OK - only use if placing inside solid is unavoidable in map, but may be clear in-game (ie: at the bottom of a tall, solid lift that starts at the top position)

radius is automatically calculated in-world.
*/
/// `void SP_waypoint( gentity_t *ent )` (g_nav.c:1271). No-oracle: spawn-time entity registration
/// (links the entity, adds a raw nav point, stores the waypoint, frees the entity).
pub unsafe fn SP_waypoint(ent: *mut gentity_t) {
    if navCalculatePaths != QFALSE {
        let radius: u32;

        VectorSet(&mut (*ent).r.mins, -15.0, -15.0, DEFAULT_MINS_2 as f32);
        VectorSet(&mut (*ent).r.maxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);

        (*ent).r.contents = CONTENTS_TRIGGER;
        (*ent).clipmask = MASK_DEADSOLID;

        trap::LinkEntity(ent);

        (*ent).count = -1;
        (*ent).classname = c"waypoint".as_ptr() as *mut c_char;

        if ((*ent).spawnflags & 1) == 0 && G_CheckInSolid(ent, QTRUE) != QFALSE {
            //if not SOLID_OK, and in solid
            (*ent).r.maxs[2] = CROUCH_MAXS_2 as f32;
            if G_CheckInSolid(ent, QTRUE) != QFALSE {
                Com_Printf(&format!(
                    "{S_COLOR_RED}ERROR: Waypoint {} at {} in solid!\n",
                    CStr::from_ptr((*ent).targetname).to_string_lossy(),
                    CStr::from_ptr(vtos(&(*ent).r.currentOrigin)).to_string_lossy(),
                ));
                debug_assert!(false, "Waypoint in solid!");
                G_FreeEntity(ent);
                return;
            }
        }

        radius = waypoint_getRadius(ent);

        (*ent).health =
            trap::Nav_AddRawPoint(&(*ent).r.currentOrigin, (*ent).spawnflags, radius as i32);
        NAV_StoreWaypoint(ent);
        G_FreeEntity(ent);
        return;
    }

    G_FreeEntity(ent);
}

/*QUAKED waypoint_small  (0.7 0.7 0) (-2 -2 -24) (2 2 32) SOLID_OK
SOLID_OK - only use if placing inside solid is unavoidable in map, but may be clear in-game (ie: at the bottom of a tall, solid lift that starts at the top position)
*/
/// `void SP_waypoint_small( gentity_t *ent )` (g_nav.c:1314). No-oracle: spawn-time entity
/// registration; identical to [`SP_waypoint`] but with a 2-unit bounds and a fixed radius of 2.
pub unsafe fn SP_waypoint_small(ent: *mut gentity_t) {
    if navCalculatePaths != QFALSE {
        VectorSet(&mut (*ent).r.mins, -2.0, -2.0, DEFAULT_MINS_2 as f32);
        VectorSet(&mut (*ent).r.maxs, 2.0, 2.0, DEFAULT_MAXS_2 as f32);

        (*ent).r.contents = CONTENTS_TRIGGER;
        (*ent).clipmask = MASK_DEADSOLID;

        trap::LinkEntity(ent);

        (*ent).count = -1;
        (*ent).classname = c"waypoint".as_ptr() as *mut c_char;

        if ((*ent).spawnflags & 1) == 0 && G_CheckInSolid(ent, QTRUE) != QFALSE {
            (*ent).r.maxs[2] = CROUCH_MAXS_2 as f32;
            if G_CheckInSolid(ent, QTRUE) != QFALSE {
                Com_Printf(&format!(
                    "{S_COLOR_RED}ERROR: Waypoint_small {} at {} in solid!\n",
                    CStr::from_ptr((*ent).targetname).to_string_lossy(),
                    CStr::from_ptr(vtos(&(*ent).r.currentOrigin)).to_string_lossy(),
                ));
                debug_assert!(false);
                G_FreeEntity(ent);
                return;
            }
        }

        (*ent).health = trap::Nav_AddRawPoint(&(*ent).r.currentOrigin, (*ent).spawnflags, 2);
        NAV_StoreWaypoint(ent);
        G_FreeEntity(ent);
        return;
    }

    G_FreeEntity(ent);
}

/*QUAKED waypoint_navgoal (0.3 1 0.3) (-16 -16 -24) (16 16 32) SOLID_OK
A waypoint for script navgoals
Not included in navigation data
...
*/
/// `void SP_waypoint_navgoal( gentity_t *ent )` (g_nav.c:1370). No-oracle: spawn-time navgoal
/// registration (sizes bounds, adds a `RTF_NAVGOAL` reference tag, frees the entity).
pub unsafe fn SP_waypoint_navgoal(ent: *mut gentity_t) {
    let radius: i32 = if (*ent).radius != 0.0 {
        ((*ent).radius as i32) | NAVGOAL_USE_RADIUS
    } else {
        12
    };

    VectorSet(&mut (*ent).r.mins, -16.0, -16.0, -24.0);
    VectorSet(&mut (*ent).r.maxs, 16.0, 16.0, 32.0);
    (*ent).s.origin[2] += 0.125;
    if ((*ent).spawnflags & 1) == 0 && G_CheckInSolid(ent, QFALSE) != QFALSE {
        Com_Printf(&format!(
            "{S_COLOR_RED}ERROR: Waypoint_navgoal {} at {} in solid!\n",
            CStr::from_ptr((*ent).targetname).to_string_lossy(),
            CStr::from_ptr(vtos(&(*ent).r.currentOrigin)).to_string_lossy(),
        ));
        debug_assert!(false);
    }
    TAG_Add(
        (*ent).targetname,
        core::ptr::null(),
        &(*ent).s.origin,
        &(*ent).s.angles,
        radius,
        RTF_NAVGOAL,
    );

    (*ent).classname = c"navgoal".as_ptr() as *mut c_char;
    G_FreeEntity(ent); //can't do this, they need to be found later by some functions, though those could be fixed, maybe?
}

/*QUAKED waypoint_navgoal_8 (0.3 1 0.3) (-8 -8 -24) (8 8 32) SOLID_OK */
/// `void SP_waypoint_navgoal_8( gentity_t *ent )` (g_nav.c:1402). No-oracle: 8x8 script navgoal,
/// touch-reach only (fixed radius 8, no `NAVGOAL_USE_RADIUS`).
pub unsafe fn SP_waypoint_navgoal_8(ent: *mut gentity_t) {
    VectorSet(&mut (*ent).r.mins, -8.0, -8.0, -24.0);
    VectorSet(&mut (*ent).r.maxs, 8.0, 8.0, 32.0);
    (*ent).s.origin[2] += 0.125;
    if ((*ent).spawnflags & 1) == 0 && G_CheckInSolid(ent, QFALSE) != QFALSE {
        Com_Printf(&format!(
            "{S_COLOR_RED}ERROR: Waypoint_navgoal_8 {} at {} in solid!\n",
            CStr::from_ptr((*ent).targetname).to_string_lossy(),
            CStr::from_ptr(vtos(&(*ent).r.currentOrigin)).to_string_lossy(),
        ));
        debug_assert!(false);
    }

    TAG_Add(
        (*ent).targetname,
        core::ptr::null(),
        &(*ent).s.origin,
        &(*ent).s.angles,
        8,
        RTF_NAVGOAL,
    );

    (*ent).classname = c"navgoal".as_ptr() as *mut c_char;
    G_FreeEntity(ent); //can't do this, they need to be found later by some functions, though those could be fixed, maybe?
}

/*QUAKED waypoint_navgoal_4 (0.3 1 0.3) (-4 -4 -24) (4 4 32) SOLID_OK */
/// `void SP_waypoint_navgoal_4( gentity_t *ent )` (g_nav.c:1433). No-oracle: 4x4 script navgoal,
/// touch-reach only (fixed radius 4, no `NAVGOAL_USE_RADIUS`).
pub unsafe fn SP_waypoint_navgoal_4(ent: *mut gentity_t) {
    VectorSet(&mut (*ent).r.mins, -4.0, -4.0, -24.0);
    VectorSet(&mut (*ent).r.maxs, 4.0, 4.0, 32.0);
    (*ent).s.origin[2] += 0.125;
    if ((*ent).spawnflags & 1) == 0 && G_CheckInSolid(ent, QFALSE) != QFALSE {
        Com_Printf(&format!(
            "{S_COLOR_RED}ERROR: Waypoint_navgoal_4 {} at {} in solid!\n",
            CStr::from_ptr((*ent).targetname).to_string_lossy(),
            CStr::from_ptr(vtos(&(*ent).r.currentOrigin)).to_string_lossy(),
        ));
        debug_assert!(false);
    }

    TAG_Add(
        (*ent).targetname,
        core::ptr::null(),
        &(*ent).s.origin,
        &(*ent).s.angles,
        4,
        RTF_NAVGOAL,
    );

    (*ent).classname = c"navgoal".as_ptr() as *mut c_char;
    G_FreeEntity(ent); //can't do this, they need to be found later by some functions, though those could be fixed, maybe?
}

/*QUAKED waypoint_navgoal_2 (0.3 1 0.3) (-2 -2 -24) (2 2 32) SOLID_OK */
/// `void SP_waypoint_navgoal_2( gentity_t *ent )` (g_nav.c:1464). No-oracle: 2x2 script navgoal,
/// touch-reach only (fixed radius 2, no `NAVGOAL_USE_RADIUS`).
pub unsafe fn SP_waypoint_navgoal_2(ent: *mut gentity_t) {
    VectorSet(&mut (*ent).r.mins, -2.0, -2.0, -24.0);
    VectorSet(&mut (*ent).r.maxs, 2.0, 2.0, 32.0);
    (*ent).s.origin[2] += 0.125;
    if ((*ent).spawnflags & 1) == 0 && G_CheckInSolid(ent, QFALSE) != QFALSE {
        Com_Printf(&format!(
            "{S_COLOR_RED}ERROR: Waypoint_navgoal_2 {} at {} in solid!\n",
            CStr::from_ptr((*ent).targetname).to_string_lossy(),
            CStr::from_ptr(vtos(&(*ent).r.currentOrigin)).to_string_lossy(),
        ));
        debug_assert!(false);
    }

    TAG_Add(
        (*ent).targetname,
        core::ptr::null(),
        &(*ent).s.origin,
        &(*ent).s.angles,
        2,
        RTF_NAVGOAL,
    );

    (*ent).classname = c"navgoal".as_ptr() as *mut c_char;
    G_FreeEntity(ent); //can't do this, they need to be found later by some functions, though those could be fixed, maybe?
}

/*QUAKED waypoint_navgoal_1 (0.3 1 0.3) (-1 -1 -24) (1 1 32) SOLID_OK */
/// `void SP_waypoint_navgoal_1( gentity_t *ent )` (g_nav.c:1495). No-oracle: 1x1 script navgoal,
/// touch-reach only (fixed radius 1, no `NAVGOAL_USE_RADIUS`).
pub unsafe fn SP_waypoint_navgoal_1(ent: *mut gentity_t) {
    VectorSet(&mut (*ent).r.mins, -1.0, -1.0, -24.0);
    VectorSet(&mut (*ent).r.maxs, 1.0, 1.0, 32.0);
    (*ent).s.origin[2] += 0.125;
    if ((*ent).spawnflags & 1) == 0 && G_CheckInSolid(ent, QFALSE) != QFALSE {
        Com_Printf(&format!(
            "{S_COLOR_RED}ERROR: Waypoint_navgoal_1 {} at {} in solid!\n",
            CStr::from_ptr((*ent).targetname).to_string_lossy(),
            CStr::from_ptr(vtos(&(*ent).r.currentOrigin)).to_string_lossy(),
        ));
        debug_assert!(false);
    }

    TAG_Add(
        (*ent).targetname,
        core::ptr::null(),
        &(*ent).s.origin,
        &(*ent).s.angles,
        1,
        RTF_NAVGOAL,
    );

    (*ent).classname = c"navgoal".as_ptr() as *mut c_char;
    G_FreeEntity(ent); //can't do this, they need to be found later by some functions, though those could be fixed, maybe?
}

/*
-------------------------
Svcmd_Nav_f
-------------------------
*/
/// `void Svcmd_Nav_f( void )` (g_nav.c:1529). No-oracle: server console command driving the
/// `NAVDEBUG_*` toggles and `trap_Nav_*` queries via `trap_Argv`.
pub unsafe fn Svcmd_Nav_f() {
    // char cmd[1024]; trap_Argv( 1, cmd, 1024 );
    let cmd = cstr(&trap::Argv(1));
    let cmd = cmd.as_ptr();

    if Q_stricmp(cmd, c"show".as_ptr()) == 0 {
        let cmd2 = cstr(&trap::Argv(2));
        let cmd = cmd2.as_ptr();

        if Q_stricmp(cmd, c"all".as_ptr()) == 0 {
            NAVDEBUG_showNodes = if NAVDEBUG_showNodes != QFALSE {
                QFALSE
            } else {
                QTRUE
            };

            //NOTENOTE: This causes the two states to sync up if they aren't already
            NAVDEBUG_showCollision = NAVDEBUG_showNodes;
            NAVDEBUG_showNavGoals = NAVDEBUG_showNodes;
            NAVDEBUG_showCombatPoints = NAVDEBUG_showNodes;
            NAVDEBUG_showEnemyPath = NAVDEBUG_showNodes;
            NAVDEBUG_showEdges = NAVDEBUG_showNodes;
            NAVDEBUG_showRadius = NAVDEBUG_showNodes;
        } else if Q_stricmp(cmd, c"nodes".as_ptr()) == 0 {
            NAVDEBUG_showNodes = if NAVDEBUG_showNodes != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        } else if Q_stricmp(cmd, c"radius".as_ptr()) == 0 {
            NAVDEBUG_showRadius = if NAVDEBUG_showRadius != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        } else if Q_stricmp(cmd, c"edges".as_ptr()) == 0 {
            NAVDEBUG_showEdges = if NAVDEBUG_showEdges != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        } else if Q_stricmp(cmd, c"testpath".as_ptr()) == 0 {
            NAVDEBUG_showTestPath = if NAVDEBUG_showTestPath != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        } else if Q_stricmp(cmd, c"enemypath".as_ptr()) == 0 {
            NAVDEBUG_showEnemyPath = if NAVDEBUG_showEnemyPath != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        } else if Q_stricmp(cmd, c"combatpoints".as_ptr()) == 0 {
            NAVDEBUG_showCombatPoints = if NAVDEBUG_showCombatPoints != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        } else if Q_stricmp(cmd, c"navgoals".as_ptr()) == 0 {
            NAVDEBUG_showNavGoals = if NAVDEBUG_showNavGoals != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        } else if Q_stricmp(cmd, c"collision".as_ptr()) == 0 {
            NAVDEBUG_showCollision = if NAVDEBUG_showCollision != QFALSE {
                QFALSE
            } else {
                QTRUE
            };
        }
    } else if Q_stricmp(cmd, c"set".as_ptr()) == 0 {
        let cmd2 = cstr(&trap::Argv(2));
        let cmd = cmd2.as_ptr();

        if Q_stricmp(cmd, c"testgoal".as_ptr()) == 0 {
            let ent0 = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0);
            NAVDEBUG_curGoal =
                trap::Nav_GetNearestNode(ent0, (*ent0).waypoint, NF_CLEAR_PATH, WAYPOINT_NONE);
        }
    } else if Q_stricmp(cmd, c"totals".as_ptr()) == 0 {
        Com_Printf("Navigation Totals:\n");
        Com_Printf("------------------\n");
        Com_Printf(&format!(
            "Total Nodes:         {}\n",
            trap::Nav_GetNumNodes()
        ));
        Com_Printf(&format!(
            "Total Combat Points: {}\n",
            (*addr_of!(level)).numCombatPoints
        ));
    } else {
        //Print the available commands
        Com_Printf("nav - valid commands\n---\n");
        Com_Printf("show\n - nodes\n - edges\n - testpath\n - enemypath\n - combatpoints\n - navgoals\n---\n");
        Com_Printf("set\n - testgoal\n---\n");
    }
}

/*
-------------------------
NAV_WaypointsTooFar
-------------------------
*/
/// `qboolean NAV_WaypointsTooFar( gentity_t *wp1, gentity_t *wp2 )` (g_nav.c:1629, `#ifndef
/// FINAL_BUILD`). No-oracle: accumulates over-1024 waypoint connections into a fatal-error
/// string and `Com_Error`s when the buffer fills. (Note: only ever called from the `#if
/// _HARD_CONNECT`-disabled path in `NAV_CalculatePaths`, and even there the call is commented out.)
pub unsafe fn NAV_WaypointsTooFar(wp1: *mut gentity_t, wp2: *mut gentity_t) -> qboolean {
    if Distance(&(*wp1).r.currentOrigin, &(*wp2).r.currentOrigin) > 1024.0 {
        let mut temp: [c_char; 1024] = [0; 1024];
        let len: i32;
        fatalErrors += 1;
        if (*wp1).targetname.is_null() && (*wp2).targetname.is_null() {
            Com_sprintf(
                temp.as_mut_ptr(),
                temp.len() as i32,
                format_args!(
                    "{S_COLOR_RED}Waypoint conn {}->{} > 1024\n",
                    CStr::from_ptr(vtos(&(*wp1).r.currentOrigin)).to_string_lossy(),
                    CStr::from_ptr(vtos(&(*wp2).r.currentOrigin)).to_string_lossy(),
                ),
            );
        } else if (*wp1).targetname.is_null() {
            Com_sprintf(
                temp.as_mut_ptr(),
                temp.len() as i32,
                format_args!(
                    "{S_COLOR_RED}Waypoint conn {}->{} > 1024\n",
                    CStr::from_ptr(vtos(&(*wp1).r.currentOrigin)).to_string_lossy(),
                    CStr::from_ptr((*wp2).targetname).to_string_lossy(),
                ),
            );
        } else if (*wp2).targetname.is_null() {
            Com_sprintf(
                temp.as_mut_ptr(),
                temp.len() as i32,
                format_args!(
                    "{S_COLOR_RED}Waypoint conn {}->{} > 1024\n",
                    CStr::from_ptr((*wp1).targetname).to_string_lossy(),
                    CStr::from_ptr(vtos(&(*wp2).r.currentOrigin)).to_string_lossy(),
                ),
            );
        } else {
            //they both have valid targetnames
            Com_sprintf(
                temp.as_mut_ptr(),
                temp.len() as i32,
                format_args!(
                    "{S_COLOR_RED}Waypoint conn {}->{} > 1024\n",
                    CStr::from_ptr((*wp1).targetname).to_string_lossy(),
                    CStr::from_ptr((*wp2).targetname).to_string_lossy(),
                ),
            );
        }
        len = CStr::from_ptr(temp.as_ptr()).to_bytes().len() as i32;
        let fatal_errors = fatalErrors;
        let fes_ptr = addr_of_mut!(fatalErrorString) as *mut c_char;
        // current write offset into fatalErrorString (C's fatalErrorPointer - fatalErrorString)
        let offset = CStr::from_ptr(fes_ptr).to_bytes().len() as i32;
        if offset + len >= 4096 {
            Com_Error(
                ERR_DROP,
                &format!(
                    "{}{}{}TOO MANY FATAL NAV ERRORS!!!\n",
                    CStr::from_ptr(fes_ptr).to_string_lossy(),
                    CStr::from_ptr(temp.as_ptr()).to_string_lossy(),
                    fatal_errors,
                ),
            );
        }
        // strcat( fatalErrorPointer, temp ); — append temp at the current offset.
        core::ptr::copy_nonoverlapping(
            temp.as_ptr(),
            fes_ptr.add(offset as usize),
            (len + 1) as usize,
        );
        QTRUE
    } else {
        QFALSE
    }
}

/*
-------------------------
NAV_ClearStoredWaypoints
-------------------------
*/
/// `void NAV_ClearStoredWaypoints( void )` (g_nav.c:1674). No-oracle: resets the file-static
/// stored-waypoint count.
pub unsafe fn NAV_ClearStoredWaypoints() {
    numStoredWaypoints = 0;
}

/// `void NAV_StoreWaypoint( gentity_t *ent )` (g_nav.c:1680). No-oracle: copies the entity's
/// target chain into the next `tempWaypointList` slot.
pub unsafe fn NAV_StoreWaypoint(ent: *mut gentity_t) {
    /*
    if ( !tempWaypointList )
    {
        //tempWaypointList = (waypointData_t *) gi.Malloc(sizeof(waypointData_t)*MAX_STORED_WAYPOINTS, TAG_TEMP_WORKSPACE, qtrue);
        int size = sizeof(waypointData_t)*MAX_STORED_WAYPOINTS;
        trap_TrueMalloc((void **)&tempWaypointList, size);
        memset(tempWaypointList, 0, size);
    }
    */

    if numStoredWaypoints as usize >= MAX_STORED_WAYPOINTS {
        //G_Error( "Too many waypoints!  (%d > %d)\n", numStoredWaypoints, MAX_STORED_WAYPOINTS );
        //rwwFIXMEFIXME: commented this out so I can load some of the SP levels.
        return;
    }
    let slot = numStoredWaypoints as usize;
    if !(*ent).targetname.is_null() {
        Q_strncpyz(
            tempWaypointList[slot].targetname.as_mut_ptr(),
            (*ent).targetname,
            MAX_QPATH as i32,
        );
    }
    if !(*ent).target.is_null() {
        Q_strncpyz(
            tempWaypointList[slot].target.as_mut_ptr(),
            (*ent).target,
            MAX_QPATH as i32,
        );
    }
    if !(*ent).target2.is_null() {
        Q_strncpyz(
            tempWaypointList[slot].target2.as_mut_ptr(),
            (*ent).target2,
            MAX_QPATH as i32,
        );
    }
    if !(*ent).target3.is_null() {
        Q_strncpyz(
            tempWaypointList[slot].target3.as_mut_ptr(),
            (*ent).target3,
            MAX_QPATH as i32,
        );
    }
    if !(*ent).target4.is_null() {
        Q_strncpyz(
            tempWaypointList[slot].target4.as_mut_ptr(),
            (*ent).target4,
            MAX_QPATH as i32,
        );
    }
    tempWaypointList[slot].nodeID = (*ent).health;

    numStoredWaypoints += 1;
}

/// `int NAV_GetStoredWaypoint( char *targetname )` (g_nav.c:1724). No-oracle: linear scan of the
/// stored-waypoint table for a matching targetname.
pub unsafe fn NAV_GetStoredWaypoint(targetname: *mut c_char) -> i32 {
    // !tempWaypointList is always false (static array), but keep the targetname guards.
    if targetname.is_null() || *targetname == 0 {
        return -1;
    }
    for i in 0..numStoredWaypoints as usize {
        if tempWaypointList[i].targetname[0] != 0
            && Q_stricmp(targetname, tempWaypointList[i].targetname.as_ptr()) == 0
        {
            return i as i32;
        }
    }
    -1
}

/// `void NAV_CalculatePaths( const char *filename, int checksum )` (g_nav.c:1745). No-oracle:
/// the `_HARD_CONNECT` hard-connection loop is `#if`-disabled (macro undefined), so this only
/// re-checks blocked edges and clears the paths-calculated flag. `FINAL_BUILD` is undefined so
/// the fatal-error reset/report blocks are included.
pub unsafe fn NAV_CalculatePaths(_filename: *const c_char, _checksum: i32) {
    // !tempWaypointList is always false (static array); fall through.
    // #ifndef FINAL_BUILD
    fatalErrors = 0;
    fatalErrorString = [0; 4096];
    // fatalErrorPointer = &fatalErrorString[0]; (tracked as offset 0)
    // #endif
    // #if _HARD_CONNECT — macro undefined, hard-connection loop excluded.

    //Remove all waypoints now that they're done
    //gi.Free(tempWaypointList);
    /*
    trap_TrueFree((void **)&tempWaypointList);
    tempWaypointList=0;
    */

    //Now check all blocked edges, mark failed ones
    trap::Nav_CheckBlockedEdges();

    trap::Nav_SetPathsCalculated(QFALSE);
    //navigator.pathsCalculated = qfalse;

    //Calculate the paths based on the supplied waypoints
    //trap_Nav_CalculatePaths();

    //Save the resulting information
    /*
    if ( trap_Nav_Save( filename, checksum ) == qfalse )
    {
        Com_Printf("Unable to save navigations data for map \"%s\" (checksum:%d)\n", filename, checksum );
    }
    */
    // #ifndef FINAL_BUILD
    if fatalErrors != 0 {
        //Com_Error( ERR_DROP, "%s%d FATAL NAV ERRORS\n", fatalErrorString, fatalErrors );
        let fatal_errors = fatalErrors;
        let fes_ptr = addr_of!(fatalErrorString) as *const c_char;
        Com_Printf(&format!(
            "{}{} FATAL NAV ERRORS\n",
            CStr::from_ptr(fes_ptr).to_string_lossy(),
            fatal_errors,
        ));
    }
    // #endif
}

/*
-------------------------
NAV_ShowDebugInfo
-------------------------
*/
/// `void NAV_ShowDebugInfo( void )` (g_nav.c:1868). No-oracle: per-frame debug visualization driven
/// by the `NAVDEBUG_*` toggles via `trap_Nav_*` + the (empty) `G_Draw*`/`TAG_ShowTags` stubs.
pub unsafe fn NAV_ShowDebugInfo() {
    if NAVDEBUG_showNodes != QFALSE {
        trap::Nav_ShowNodes();
    }

    if NAVDEBUG_showEdges != QFALSE {
        trap::Nav_ShowEdges();
    }

    if NAVDEBUG_showTestPath != QFALSE {
        let ent0 = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0);
        //Get the nearest node to the player
        let mut nearest_node =
            trap::Nav_GetNearestNode(ent0, (*ent0).waypoint, NF_ANY, WAYPOINT_NONE);
        let test_node = trap::Nav_GetBestNode(nearest_node, NAVDEBUG_curGoal, NODE_NONE);
        let mut dest: vec3_t = [0.0; 3];
        let mut start: vec3_t = [0.0; 3];

        nearest_node = NAV_TestBestNode(ent0, nearest_node, test_node, QFALSE);

        //Show the connection

        //Get the positions
        trap::Nav_GetNodePosition(NAVDEBUG_curGoal, &mut dest);
        trap::Nav_GetNodePosition(nearest_node, &mut start);

        G_DrawNode(&start, NODE_START);
        G_DrawNode(&dest, NODE_GOAL);
        trap::Nav_ShowPath(nearest_node, NAVDEBUG_curGoal);
    }

    if NAVDEBUG_showCombatPoints != QFALSE {
        for i in 0..(*addr_of!(level)).numCombatPoints as usize {
            G_DrawCombatPoint(&(*addr_of!(level)).combatPoints[i].origin, 0);
        }
    }

    if NAVDEBUG_showNavGoals != QFALSE {
        TAG_ShowTags(RTF_NAVGOAL);
    }
}

/*
-------------------------
NAV_FindPlayerWaypoint
-------------------------
*/
/// `void NAV_FindPlayerWaypoint( int clNum )` (g_nav.c:1922). No-oracle: caches the client's
/// nearest nav node into its `waypoint` field via `trap_Nav_GetNearestNode`.
pub unsafe fn NAV_FindPlayerWaypoint(cl_num: i32) {
    let ent = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(cl_num as usize);
    (*ent).waypoint =
        trap::Nav_GetNearestNode(ent, (*ent).lastWaypoint, NF_CLEAR_PATH, WAYPOINT_NONE);
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use crate::codemp::game::q_shared_h::vec3_t;
    use crate::ffi::types::qboolean;
    use crate::oracle::jka_NAV_HitNavGoal;

    use super::{NAV_HitNavGoal, NAVGOAL_USE_RADIUS};

    /// `NAV_HitNavGoal` checked bit-exact against the extracted C across both branches: the
    /// bounds-overlap path (radius without `NAVGOAL_USE_RADIUS`) and the DistanceSquared/
    /// VectorLengthSquared path (radius with the flag), flying and grounded.
    #[test]
    fn nav_hitnavgoal_matches_oracle() {
        let mins: vec3_t = [-16.0, -16.0, -24.0];
        let maxs: vec3_t = [16.0, 16.0, 32.0];

        let cases: &[(vec3_t, vec3_t, i32, qboolean)] = &[
            // bounds-overlap branch (no NAVGOAL_USE_RADIUS)
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 16, 0),
            ([0.0, 0.0, 0.0], [40.0, 0.0, 0.0], 16, 0),
            ([0.0, 0.0, 0.0], [200.0, 200.0, 0.0], 64, 0),
            ([10.0, 5.0, -3.0], [10.0, 5.0, 100.0], 8, 1),
            // radius branch, grounded (z fudge)
            (
                [0.0, 0.0, 0.0],
                [30.0, 0.0, 20.0],
                32 | NAVGOAL_USE_RADIUS,
                0,
            ),
            (
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 40.0],
                32 | NAVGOAL_USE_RADIUS,
                0,
            ),
            (
                [0.0, 0.0, 0.0],
                [50.0, 50.0, 0.0],
                32 | NAVGOAL_USE_RADIUS,
                0,
            ),
            // radius branch, flying (exact distance)
            (
                [0.0, 0.0, 0.0],
                [10.0, 10.0, 10.0],
                32 | NAVGOAL_USE_RADIUS,
                1,
            ),
            (
                [0.0, 0.0, 0.0],
                [40.0, 40.0, 40.0],
                32 | NAVGOAL_USE_RADIUS,
                1,
            ),
        ];

        for (i, (point, dest, radius, flying)) in cases.iter().enumerate() {
            let got = NAV_HitNavGoal(point, &mins, &maxs, dest, *radius, *flying);
            let want = unsafe {
                jka_NAV_HitNavGoal(
                    point.as_ptr(),
                    mins.as_ptr(),
                    maxs.as_ptr(),
                    dest.as_ptr(),
                    *radius,
                    *flying,
                )
            };
            assert_eq!(
                got, want,
                "case {i}: point {point:?} dest {dest:?} r {radius}"
            );
        }
    }
}
