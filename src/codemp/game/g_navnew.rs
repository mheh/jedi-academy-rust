//! Port of `g_navnew.c` — the "new" NPC collision-avoidance / steering layer that
//! sits on top of the `g_nav.c` waypoint graph. This is the NAV keystone's
//! entity-collision half: given a desired move direction it tries to walk around,
//! shove, or wait out a blocking entity (`NAVNEW_Bypass` /
//! `NAVNEW_SidestepBlocker` / `NAVNEW_PushBlocker` / `NAVNEW_DanceWithBlocker`),
//! resolves door/double-block special cases (`NAVNEW_ResolveEntityCollision` /
//! `NAVNEW_CheckDoubleBlock`), and answers raw "is the path clear" trace queries
//! (`NAVNEW_ClearPathBetweenPoints` / `NAVNEW_TestNodeConnectionBlocked`).
//!
//! The two top-level drivers `NAVNEW_AvoidCollision` and `NAVNEW_MoveToGoal` sit on
//! top of the intra-file leaves and the `g_nav.c` waypoint graph
//! (`NAV_CheckAhead` / `NAV_TestForBlocked` / `NAV_TestBestNode`, now all in
//! g_nav.rs) plus the `trap_Nav_*` wrapper family. The debug-draw visualization
//! helpers (`G_DrawEdge` / `G_DrawNode`) and the `NAVDEBUG_*` toggle globals live in
//! `g_nav.c` where they are empty/zero stubs; they are guarded-stubbed here too —
//! see the `// REVISIT` markers.

#![allow(non_snake_case)] // C function names (`NAVNEW_PushBlocker`, …) kept verbatim
#![allow(non_camel_case_types)] // C type name (`navInfo_t`) kept verbatim
#![allow(non_upper_case_globals)] // C global names (`NAVDEBUG_showCollision`, …) kept verbatim

use core::ptr::addr_of;

use crate::trap;

use crate::codemp::game::b_public_h::NPCAI_BLOCKED;
use crate::codemp::game::bg_misc::vectoyaw;
use crate::codemp::game::bg_public::{DEFAULT_MAXS_2, DEFAULT_MINS_2, MASK_NPCSOLID, STEPSIZE};
use crate::codemp::game::bg_weapons_h::WP_SABER;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{d_altRoutes, d_patched, g_entities, level};
use crate::codemp::game::g_mover::CalcTeamDoorCenter;
use crate::codemp::game::g_nav::{NAV_CheckAhead, NAV_TestBestNode, NAV_TestForBlocked};
use crate::codemp::game::g_public_h::MAX_FAILED_NODES;
use crate::codemp::game::npc::NPCInfo;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{
    vec3_origin, vectoangles, AngleNormalize360, AngleVectors, DistanceSquared, DotProduct,
    VectorClear, VectorCompare, VectorCopy, VectorMA, VectorNormalize, VectorScale, VectorSet,
    VectorSubtract,
};
use crate::codemp::game::q_shared::{random, Q_stricmp};
use crate::codemp::game::q_shared_h::{
    qboolean, trace_t, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, QFALSE, QTRUE, YAW,
};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_BODY, CONTENTS_BOTCLIP, CONTENTS_MONSTERCLIP, CONTENTS_SOLID,
};

// ---------------------------------------------------------------------------
// navInfo_t (b_local.h:302-322), the NIF_* nav-info flags, and the g_nav.h nav
// constants are the canonical NAV-layer types/consts — defined once in g_nav.rs
// (g_nav.c) and re-used here. (g_navnew.c relies on g_nav.h / b_local.h for the
// same declarations, so this single-definition import mirrors the C headers.)
// ---------------------------------------------------------------------------
use crate::codemp::game::g_nav::{
    navInfo_t, MAX_COLL_AVOID_DIST, MIN_BLOCKED_SPEECH_TIME, MIN_DOOR_BLOCK_DIST_SQR,
    NF_CLEAR_PATH, NIF_COLLISION, NODE_NONE, WAYPOINT_NONE,
};

// `EDGE_*` / `NODE_*` debug-draw enums (g_public.h:603-618) — only consumed by the
// guarded-stub visualization helpers below.
const EDGE_NORMAL: i32 = 0;
#[allow(dead_code)]
const EDGE_PATH: i32 = 1;
#[allow(dead_code)]
const EDGE_MOVEDIR: i32 = 4;

// ---------------------------------------------------------------------------
// Visualization helpers + NAVDEBUG_* toggles — these live in g_nav.c (g_nav.rs).
// Guarded-stubbed so the collision consumers below can land this cycle.
// ---------------------------------------------------------------------------

/// `NAVDEBUG_showCollision` (g_nav.h:60).
// REVISIT: guarded stub; lives in g_nav.c — un-stub when g_nav.rs integrates.
const NAVDEBUG_showCollision: qboolean = QFALSE;

/// `void G_Line( vec3_t start, vec3_t end, vec3_t color, float alpha )`
/// (g_navnew.c:7) — debug visualization, defined in g_nav.c.
// REVISIT: guarded stub; lives in g_nav.c — un-stub when g_nav.rs integrates.
#[allow(unused_variables)]
unsafe fn G_DrawEdge(_start: &vec3_t, _end: &vec3_t, _edge_type: i32) {}

/// `int NAVNEW_ClearPathBetweenPoints(...)` is the only fn in this file that needs
/// the world entity number; `NAV_CheckNodeFailedForEnt` is the only failed-node
/// helper. Both are pure leaves below.

// ---------------------------------------------------------------------------
// Leaves.
// ---------------------------------------------------------------------------

/// `g_navnew.c:15` — `NAV_CheckNodeFailedForEnt`. Whether `ent` has already
/// recorded `nodeNum` in its `failedWaypoints` list (stored +1 because 0 is a
/// valid nodeNum but also the default empty value).
///
/// No-oracle: reads the `gentity_t::failedWaypoints` array through a raw pointer.
pub unsafe fn NAV_CheckNodeFailedForEnt(ent: *mut gentity_t, nodeNum: i32) -> qboolean {
    let mut j: i32;

    //FIXME: must be a better way to do this
    j = 0;
    while j < MAX_FAILED_NODES as i32 {
        if (*ent).failedWaypoints[j as usize] == nodeNum + 1
        //+1 because 0 is a valid nodeNum, but also the default
        {
            //we failed against this node
            return QTRUE;
        }
        j += 1;
    }
    QFALSE
}

/*
-------------------------
NPC_UnBlocked
-------------------------
*/
/// `g_navnew.c:34` — `NPC_ClearBlocked`. Clear the NPC's "blocked by entity"
/// record.
///
/// No-oracle: mutates `gNPC_t` state through a raw `gentity_t` pointer.
pub unsafe fn NPC_ClearBlocked(self_: *mut gentity_t) {
    if (*self_).NPC.is_null() {
        return;
    }

    //self->NPC->aiFlags &= ~NPCAI_BLOCKED;
    (*(*self_).NPC).blockingEntNum = ENTITYNUM_NONE;
}

/// `g_navnew.c:43` — `NPC_SetBlocked`. Record `blocker` as the entity blocking
/// `self`, arming the blocked-speech debounce.
///
/// No-oracle: mutates `gNPC_t` state through raw `gentity_t` pointers; uses
/// `level.time` and `random()`.
pub unsafe fn NPC_SetBlocked(self_: *mut gentity_t, blocker: *mut gentity_t) {
    if (*self_).NPC.is_null() {
        return;
    }

    //self->NPC->aiFlags |= NPCAI_BLOCKED;
    (*(*self_).NPC).blockedSpeechDebounceTime =
        level.time + MIN_BLOCKED_SPEECH_TIME + (random() * 4000.0) as i32;
    (*(*self_).NPC).blockingEntNum = (*blocker).s.number;
}

/*
-------------------------
NAVNEW_ClearPathBetweenPoints
-------------------------
*/
/// `g_navnew.c:58` — `NAVNEW_ClearPathBetweenPoints`. Trace a box between two
/// points and return the entity number hit (or `ENTITYNUM_WORLD` if the two
/// points aren't even in the same PVS).
///
/// No-oracle: issues `trap_InPVS` / `trap_Trace` syscalls.
pub unsafe fn NAVNEW_ClearPathBetweenPoints(
    start: &vec3_t,
    end: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    ignore: i32,
    clipmask: i32,
) -> i32 {
    //Test if they're even conceivably close to one another
    if trap::InPVS(start, end) == QFALSE {
        return ENTITYNUM_WORLD;
    }

    let trace: trace_t = trap::Trace(start, mins, maxs, end, ignore, clipmask);

    //if( ( ( trace.startsolid == false ) && ( trace.allsolid == false ) ) && ( trace.fraction < 1.0f ) )
    //{//FIXME: check for drops?
    //FIXME: if startsolid or allsolid, then the path isn't clear... but returning ENTITYNUM_NONE indicates to CheckFailedEdge that is is clear...?
    trace.entityNum as i32
    //}

    //return ENTITYNUM_NONE;
}

/*
-------------------------
NAVNEW_PushBlocker
-------------------------
*/
/// `g_navnew.c:84` — `NAVNEW_PushBlocker`. Try shoving the blocking client to one
/// side (favoring left) by writing a `pushVec` onto its `gclient_t`.
///
/// No-oracle: traces + mutates client/NPC state through raw pointers.
pub unsafe fn NAVNEW_PushBlocker(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    right: &vec3_t,
    setBlockedInfo: qboolean,
) {
    //try pushing blocker to one side
    let mut tr: trace_t;
    let mut mins: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let rightSucc: f32;
    let leftSucc: f32;
    let moveamt: f32;

    if (*(*self_).NPC).shoveCount > 30 {
        //don't push for more than 3 seconds;
        return;
    }

    if (*blocker).s.number == 0 {
        //never push the player
        return;
    }

    if (*blocker).client.is_null()
        || VectorCompare(&(*(*blocker).client).pushVec, &vec3_origin) == 0
    {
        //someone else is pushing him, wait until they give up?
        return;
    }

    VectorCopy(&(*blocker).r.mins, &mut mins);
    mins[2] += STEPSIZE as f32;

    moveamt = ((*self_).r.maxs[1] + (*blocker).r.maxs[1]) * 1.2; //yes, magic number

    VectorMA(&(*blocker).r.currentOrigin, -moveamt, right, &mut end);
    tr = trap::Trace(
        &(*blocker).r.currentOrigin,
        &mins,
        &(*blocker).r.maxs,
        &end,
        (*blocker).s.number,
        (*blocker).clipmask | CONTENTS_BOTCLIP,
    );
    if tr.startsolid == 0 && tr.allsolid == 0 {
        leftSucc = tr.fraction;
    } else {
        leftSucc = 0.0;
    }

    if leftSucc >= 1.0 {
        //it's clear, shove him that way
        VectorScale(right, -moveamt, &mut (*(*blocker).client).pushVec);
        (*(*blocker).client).pushVecTime = level.time + 2000;
    } else {
        VectorMA(&(*blocker).r.currentOrigin, moveamt, right, &mut end);
        tr = trap::Trace(
            &(*blocker).r.currentOrigin,
            &mins,
            &(*blocker).r.maxs,
            &end,
            (*blocker).s.number,
            (*blocker).clipmask | CONTENTS_BOTCLIP,
        );
        if tr.startsolid == 0 && tr.allsolid == 0 {
            rightSucc = tr.fraction;
        } else {
            rightSucc = 0.0;
        }

        if leftSucc == 0.0 && rightSucc == 0.0 {
            //both sides failed
            if (*addr_of!(d_patched)).integer != 0 {
                //use patch-style navigation
                (*(*blocker).client).pushVecTime = 0;
            }
            return;
        }

        if rightSucc >= 1.0 {
            //it's clear, shove him that way
            VectorScale(right, moveamt, &mut (*(*blocker).client).pushVec);
            (*(*blocker).client).pushVecTime = level.time + 2000;
        }
        //if neither are enough, we probably can't get around him, but keep trying
        else if leftSucc >= rightSucc {
            //favor the left, all things being equal
            VectorScale(right, -moveamt, &mut (*(*blocker).client).pushVec);
            (*(*blocker).client).pushVecTime = level.time + 2000;
        } else {
            VectorScale(right, moveamt, &mut (*(*blocker).client).pushVec);
            (*(*blocker).client).pushVecTime = level.time + 2000;
        }
    }

    if setBlockedInfo != QFALSE {
        //we tried pushing
        (*(*self_).NPC).shoveCount += 1;
    }
}

/*
-------------------------
NAVNEW_DanceWithBlocker
-------------------------
*/
/// `g_navnew.c:178` — `NAVNEW_DanceWithBlocker`. If the blocker has lateral
/// movement, steer to the opposite side of it (modifying `movedir` in place).
///
/// No-oracle: reads client velocity through raw pointers.
pub unsafe fn NAVNEW_DanceWithBlocker(
    _self: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: &mut vec3_t,
    right: &vec3_t,
) -> qboolean {
    //sees if blocker has any lateral movement
    if !(*blocker).client.is_null()
        && VectorCompare(&(*(*blocker).client).ps.velocity, &vec3_origin) == 0
    {
        let mut blocker_movedir: vec3_t = [0.0; 3];
        let dot: f32;

        VectorCopy(&(*(*blocker).client).ps.velocity, &mut blocker_movedir);
        blocker_movedir[2] = 0.0; //cancel any vertical motion
        dot = DotProduct(&blocker_movedir, right);
        if dot > 50.0 {
            //he's moving to the right of me at a relatively good speed
            //go to my left
            let movedir_copy = *movedir;
            VectorMA(&movedir_copy, -1.0, right, movedir);
            VectorNormalize(movedir);
            return QTRUE;
        } else if dot > -50.0 {
            //he's moving to the left of me at a relatively good speed
            //go to my right
            let movedir_copy = *movedir;
            VectorAdd_local(right, &movedir_copy, movedir);
            VectorNormalize(movedir);
            return QTRUE;
        }
        /*
        vec3_t	block_pos;
        trace_t	tr;
        VectorScale( blocker_movedir, -1, blocker_movedir );
        VectorMA( self->r.currentOrigin, blocked_dist, blocker_movedir, block_pos );
        if ( NAVNEW_CheckAhead( self, block_pos, tr, ( self->clipmask & ~CONTENTS_BODY )|CONTENTS_BOTCLIP ) )
        {
            VectorCopy( blocker_movedir, movedir );
            return qtrue;
        }
        */
    }
    QFALSE
}

// `q_math::VectorAdd` borrows both inputs immutably and the output mutably; the C
// call `VectorAdd( right, movedir, movedir )` aliases `movedir` as both an input
// and the output, so the caller snapshots it first.
#[inline]
unsafe fn VectorAdd_local(a: &vec3_t, b: &vec3_t, c: &mut vec3_t) {
    crate::codemp::game::q_math::VectorAdd(a, b, c);
}

/*
-------------------------
NAVNEW_SidestepBlocker
-------------------------
*/
/// `g_navnew.c:222` — `NAVNEW_SidestepBlocker`. Trace to either side of the
/// blocker (with a ping-pong debounce) and pick a clear arc to sidestep through,
/// writing the chosen direction into `movedir`.
///
/// No-oracle: traces + mutates NPC sidestep state through raw pointers.
pub unsafe fn NAVNEW_SidestepBlocker(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: &vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
    _right: &vec3_t, // C param `right` is unused in the body (kept for signature fidelity)
) -> qboolean {
    //trace to sides of blocker and see if either is clear
    let mut tr: trace_t;
    let mut avoidAngles: vec3_t = [0.0; 3];
    let mut avoidRight_dir: vec3_t = [0.0; 3];
    let mut avoidLeft_dir: vec3_t = [0.0; 3];
    let mut block_pos: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let rightSucc: f32;
    let leftSucc: f32;
    let yaw: f32;
    let avoidRadius: f32;
    let mut arcAngle: f32;

    VectorCopy(&(*self_).r.mins, &mut mins);
    mins[2] += STEPSIZE as f32;

    //Get the blocked direction
    yaw = vectoyaw(blocked_dir);

    //Get the avoid radius
    avoidRadius = (((*blocker).r.maxs[0] * (*blocker).r.maxs[0])
        + ((*blocker).r.maxs[1] * (*blocker).r.maxs[1]))
        .sqrt()
        + (((*self_).r.maxs[0] * (*self_).r.maxs[0]) + ((*self_).r.maxs[1] * (*self_).r.maxs[1]))
            .sqrt();

    //See if we're inside our avoidance radius
    arcAngle = if blocked_dist <= avoidRadius {
        135.0
    } else {
        (avoidRadius / blocked_dist) * 90.0
    };

    /*
    float dot = DotProduct( blocked_dir, right );

    //Go right on the first try if that works better
    if ( dot < 0.0f )
        arcAngle *= -1;
    */

    VectorClear(&mut avoidAngles);

    //need to stop it from ping-ponging, so we have a bit of a debounce time on which side you try
    if (*(*self_).NPC).sideStepHoldTime > level.time {
        if (*(*self_).NPC).lastSideStepSide == -1
        //left
        {
            arcAngle *= -1.0;
        } //else right
        avoidAngles[YAW] = AngleNormalize360(yaw + arcAngle);
        AngleVectors(&avoidAngles, Some(movedir), None, None);
        VectorMA(
            &(*self_).r.currentOrigin,
            blocked_dist,
            movedir,
            &mut block_pos,
        );
        tr = trap::Trace(
            &(*self_).r.currentOrigin,
            &mins,
            &(*self_).r.maxs,
            &block_pos,
            (*self_).s.number,
            (*self_).clipmask | CONTENTS_BOTCLIP,
        );
        return ((tr.fraction == 1.0 && tr.allsolid == 0 && tr.startsolid == 0) as i32) as qboolean;
    }

    //test right
    avoidAngles[YAW] = AngleNormalize360(yaw + arcAngle);
    AngleVectors(&avoidAngles, Some(&mut avoidRight_dir), None, None);

    VectorMA(
        &(*self_).r.currentOrigin,
        blocked_dist,
        &avoidRight_dir,
        &mut block_pos,
    );

    tr = trap::Trace(
        &(*self_).r.currentOrigin,
        &mins,
        &(*self_).r.maxs,
        &block_pos,
        (*self_).s.number,
        (*self_).clipmask | CONTENTS_BOTCLIP,
    );

    if tr.allsolid == 0 && tr.startsolid == 0 {
        if tr.fraction >= 1.0 {
            //all clear, go for it (favor the right if both are equal)
            VectorCopy(&avoidRight_dir, movedir);
            (*(*self_).NPC).lastSideStepSide = 1;
            (*(*self_).NPC).sideStepHoldTime = level.time + 2000;
            return QTRUE;
        }
        rightSucc = tr.fraction;
    } else {
        rightSucc = 0.0;
    }

    //now test left
    arcAngle *= -1.0;

    avoidAngles[YAW] = AngleNormalize360(yaw + arcAngle);
    AngleVectors(&avoidAngles, Some(&mut avoidLeft_dir), None, None);

    VectorMA(
        &(*self_).r.currentOrigin,
        blocked_dist,
        &avoidLeft_dir,
        &mut block_pos,
    );

    tr = trap::Trace(
        &(*self_).r.currentOrigin,
        &mins,
        &(*self_).r.maxs,
        &block_pos,
        (*self_).s.number,
        (*self_).clipmask | CONTENTS_BOTCLIP,
    );

    if tr.allsolid == 0 && tr.startsolid == 0 {
        if tr.fraction >= 1.0 {
            //all clear, go for it (right side would have already succeeded if as good as this)
            VectorCopy(&avoidLeft_dir, movedir);
            (*(*self_).NPC).lastSideStepSide = -1;
            (*(*self_).NPC).sideStepHoldTime = level.time + 2000;
            return QTRUE;
        }
        leftSucc = tr.fraction;
    } else {
        leftSucc = 0.0;
    }

    if leftSucc == 0.0 && rightSucc == 0.0 {
        //both sides failed
        return QFALSE;
    }

    if rightSucc * blocked_dist >= avoidRadius || leftSucc * blocked_dist >= avoidRadius {
        //the traces hit something, but got a relatively good distance
        if rightSucc >= leftSucc {
            //favor the right, all things being equal
            VectorCopy(&avoidRight_dir, movedir);
            (*(*self_).NPC).lastSideStepSide = 1;
            (*(*self_).NPC).sideStepHoldTime = level.time + 2000;
        } else {
            VectorCopy(&avoidLeft_dir, movedir);
            (*(*self_).NPC).lastSideStepSide = -1;
            (*(*self_).NPC).sideStepHoldTime = level.time + 2000;
        }
        return QTRUE;
    }

    //if neither are enough, we probably can't get around him
    QFALSE
}

/*
-------------------------
NAVNEW_Bypass
-------------------------
*/
/// `g_navnew.c:347` — `NAVNEW_Bypass`. Try to get around `blocker`: first dance to
/// its opposite, then sidestep, and as a last resort shove it aside.
///
/// No-oracle: drives the trace-based steering helpers through raw pointers.
pub unsafe fn NAVNEW_Bypass(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: &vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    let mut moveangles: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];

    //Draw debug info if requested
    if NAVDEBUG_showCollision != QFALSE {
        G_DrawEdge(
            &(*self_).r.currentOrigin,
            &(*blocker).r.currentOrigin,
            EDGE_NORMAL,
        );
    }

    vectoangles(movedir, &mut moveangles);
    moveangles[2] = 0.0;
    AngleVectors(&moveangles, None, Some(&mut right), None);

    //Check to see what dir the other guy is moving in (if any) and pick the opposite dir
    if NAVNEW_DanceWithBlocker(self_, blocker, movedir, &right) != QFALSE {
        return QTRUE;
    }

    //Okay, so he's not moving to my side, see which side of him is most clear
    if NAVNEW_SidestepBlocker(self_, blocker, blocked_dir, blocked_dist, movedir, &right) != QFALSE
    {
        return QTRUE;
    }

    //Neither side is clear, tell him to step aside
    NAVNEW_PushBlocker(self_, blocker, &right, setBlockedInfo);

    QFALSE
}

/*
-------------------------
NAVNEW_CheckDoubleBlock
-------------------------
*/
/// `g_navnew.c:384` — `NAVNEW_CheckDoubleBlock`. Whether `blocker` is in turn
/// blocked by `self` (a mutual stand-off) — stops both from waiting forever.
///
/// No-oracle: reads `gNPC_t` state through raw pointers.
pub unsafe fn NAVNEW_CheckDoubleBlock(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    _blocked_dir: &vec3_t,
) -> qboolean {
    //Stop double waiting
    if !(*blocker).NPC.is_null() && (*(*blocker).NPC).blockingEntNum == (*self_).s.number {
        return QTRUE;
    }

    QFALSE
}

/*
-------------------------
NAVNEW_ResolveEntityCollision
-------------------------
*/
/// `g_navnew.c:399` — `NAVNEW_ResolveEntityCollision`. Resolve an imminent
/// collision with `blocker`: ignore far-off doors, try to bypass/shove, detect a
/// double-block, else record being blocked.
///
/// No-oracle: drives the steering helpers + `CalcTeamDoorCenter` through raw
/// pointers; classname compare via `Q_stricmp`.
pub unsafe fn NAVNEW_ResolveEntityCollision(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: &mut vec3_t,
    _pathDir: &vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    let mut blocked_dir: vec3_t = [0.0; 3];
    let blocked_dist: f32;

    //Doors are ignored
    if Q_stricmp((*blocker).classname, c"func_door".as_ptr()) == 0 {
        let mut center: vec3_t = [0.0; 3];
        CalcTeamDoorCenter(blocker, &mut center);
        if DistanceSquared(&(*self_).r.currentOrigin, &center) > MIN_DOOR_BLOCK_DIST_SQR {
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
    //	if ( NAVNEW_PredictCollision( self, blocker, movedir, blocked_dir ) == qfalse )
    //		return qtrue;

    //First, attempt to walk around the blocker or shove him out of the way
    if NAVNEW_Bypass(
        self_,
        blocker,
        &blocked_dir,
        blocked_dist,
        movedir,
        setBlockedInfo,
    ) != QFALSE
    {
        return QTRUE;
    }

    //Can't get around him... see if I'm blocking him too... if so, I need to just keep moving?
    if NAVNEW_CheckDoubleBlock(self_, blocker, &blocked_dir) != QFALSE {
        return QTRUE;
    }

    if setBlockedInfo != QFALSE {
        //Complain about it if we can
        NPC_SetBlocked(self_, blocker);
    }

    QFALSE
}

/// `g_navnew.c:520` — `NAVNEW_TestNodeConnectionBlocked`. Trace the direct line
/// between two nav nodes and report whether architecture/an entity (other than
/// the goal) blocks it.
///
/// No-oracle: issues `trap_Nav_GetNodePosition` / `trap_Trace` syscalls; reads
/// `gentity_t` bounds through a raw pointer.
pub unsafe fn NAVNEW_TestNodeConnectionBlocked(
    wp1: i32,
    wp2: i32,
    ignoreEnt: *mut gentity_t,
    goalEntNum: i32,
    checkWorld: qboolean,
    checkEnts: qboolean,
) -> qboolean {
    //see if the direct path between 2 nodes is blocked by architecture or an ent
    let mut pos1: vec3_t = [0.0; 3];
    let mut pos2: vec3_t = [0.0; 3];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let trace: trace_t;
    let mut clipmask: i32 = MASK_NPCSOLID | CONTENTS_BOTCLIP;
    let ignoreEntNum: i32;
    let mut playerMins: vec3_t = [0.0; 3];
    let mut playerMaxs: vec3_t = [0.0; 3];

    if checkWorld == QFALSE && checkEnts == QFALSE {
        //duh, nothing to trace against
        return QFALSE;
    }
    VectorSet(&mut playerMins, -15.0, -15.0, DEFAULT_MINS_2 as f32);
    VectorSet(&mut playerMaxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);

    trap::nav::Nav_GetNodePosition(wp1, &mut pos1);
    trap::nav::Nav_GetNodePosition(wp2, &mut pos2);

    if checkWorld == QFALSE {
        clipmask &= !(CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP);
    }
    if checkEnts == QFALSE {
        clipmask &= !CONTENTS_BODY;
    }
    if !ignoreEnt.is_null() {
        VectorCopy(&(*ignoreEnt).r.mins, &mut mins);
        VectorCopy(&(*ignoreEnt).r.maxs, &mut maxs);
        ignoreEntNum = (*ignoreEnt).s.number;
    } else {
        VectorCopy(&playerMins, &mut mins);
        VectorCopy(&playerMaxs, &mut mins); // NOTE: C bug preserved — copies into `mins`, not `maxs`
        ignoreEntNum = ENTITYNUM_NONE;
    }
    mins[2] += STEPSIZE as f32;
    //don't let box get inverted
    if mins[2] > maxs[2] {
        mins[2] = maxs[2];
    }

    trace = trap::Trace(&pos1, &mins, &maxs, &pos2, ignoreEntNum, clipmask);
    if trace.fraction >= 1.0 || trace.entityNum as i32 == goalEntNum {
        //clear or hit goal
        return QFALSE;
    }
    //hit something we weren't supposed to
    QTRUE
}

/*
-------------------------
NAVNEW_AvoidCollision
-------------------------
*/
/// `g_navnew.c:442` — `NAVNEW_AvoidCollision`. Cap the desired move distance, test
/// the path ahead against entities, and if blocked try to resolve the collision
/// (bypass/shove the blocker) — modifying `info->direction` to whatever direction
/// avoids the obstacle.
///
/// No-oracle: entity-state — traces + drives the NPC steering helpers through raw
/// `gentity_t` / `navInfo_t` pointers.
pub unsafe fn NAVNEW_AvoidCollision(
    self_: *mut gentity_t,
    goal: *mut gentity_t,
    info: &mut navInfo_t,
    setBlockedInfo: qboolean,
    blockedMovesLimit: i32,
) -> qboolean {
    let mut movedir: vec3_t = [0.0; 3];
    let mut movepos: vec3_t = [0.0; 3];

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

        if setBlockedInfo != QFALSE {
            if (*(*self_).NPC).consecutiveBlockedMoves > blockedMovesLimit {
                if (*addr_of!(d_patched)).integer != 0 {
                    //use patch-style navigation
                    (*(*self_).NPC).consecutiveBlockedMoves += 1;
                }
                NPC_SetBlocked(self_, info.blocker);
                return QFALSE;
            }
            (*(*self_).NPC).consecutiveBlockedMoves += 1;
        }
        //See if we're moving along with them
        //if ( NAVNEW_TrueCollision( self, info->blocker, movedir, info->direction ) == qfalse )
        //	return qtrue;

        //Test for blocking by standing on goal
        if NAV_TestForBlocked(self_, goal, info.blocker, info.distance, &mut info.flags) == QTRUE {
            return QFALSE;
        }

        //If the above function said we're blocked, don't do the extra checks
        /*
        if ( info->flags & NIF_BLOCKED )
            return qtrue;
        */

        //See if we can get that entity to move out of our way
        if NAVNEW_ResolveEntityCollision(
            self_,
            info.blocker,
            &mut movedir,
            &info.pathDirection,
            setBlockedInfo,
        ) == QFALSE
        {
            return QFALSE;
        }

        VectorCopy(&movedir, &mut info.direction);

        QTRUE
    } else {
        if setBlockedInfo != QFALSE {
            (*(*self_).NPC).consecutiveBlockedMoves = 0;
        }

        //Our path is clear, just move there
        if NAVDEBUG_showCollision != QFALSE {
            G_DrawEdge(&(*self_).r.currentOrigin, &movepos, EDGE_MOVEDIR);
        }

        QTRUE
    }
}

/// `void G_DrawNode( vec3_t origin, int type )` (g_nav.c:31) — debug visualization,
/// defined as an empty stub in g_nav.c too.
// REVISIT: real def lives in g_nav.c, stubbed empty there too.
#[allow(unused_variables)]
unsafe fn G_DrawNode(_origin: &vec3_t, _type: i32) {}

// `NODE_*` debug-draw node types (g_public.h) + `NAVDEBUG_showEnemyPath` (g_nav.h)
// — only consumed by the guarded-stub visualization in NAVNEW_MoveToGoal below.
#[allow(dead_code)]
const NODE_START: i32 = 0;
#[allow(dead_code)]
const NODE_NAVGOAL: i32 = 1;
#[allow(dead_code)]
const NODE_GOAL: i32 = 2;
/// `NAVDEBUG_showEnemyPath` (g_nav.h).
// REVISIT: guarded stub; lives in g_nav.c — un-stub when the debug toggles land.
const NAVDEBUG_showEnemyPath: qboolean = QFALSE;

/*
-------------------------
NAVNEW_MoveToGoal
-------------------------
*/
/// `g_navnew.c:578` — `NAVNEW_MoveToGoal`. The top-level macro-nav driver: pick the
/// best route node toward the NPC's goal entity, then loop trying to reach it while
/// avoiding entity collisions, falling back along alternate routes / failed-node
/// marks until a clear path is found or all options are exhausted.
///
/// No-oracle: entity-state — drives the `trap_Nav_*` graph + steering helpers
/// through raw `gentity_t` / `navInfo_t` pointers.
// `unused_assignments`: the final-iteration rewrites of `tempInfo` and the
// `inGoalWP` / `goalWPFailed` flag writes are only read by the (faithfully-
// preserved) commented-out blocks in the C original, so the live path leaves those
// last stores unread — kept to mirror the C control flow verbatim.
#[allow(unused_assignments, unused_variables)]
pub unsafe fn NAVNEW_MoveToGoal(self_: *mut gentity_t, info: &mut navInfo_t) -> i32 {
    let mut bestNode: i32 = WAYPOINT_NONE;
    let mut foundClearPath: qboolean = QFALSE;
    let mut origin: vec3_t = [0.0; 3];
    let mut tempInfo: navInfo_t;
    let mut setBlockedInfo: qboolean = QTRUE;
    let mut inGoalWP: qboolean = QFALSE;
    let mut goalWPFailed: qboolean = QFALSE;
    let mut numTries: i32 = 0;

    tempInfo = *info;

    //Must have a goal entity to move there
    if (*(*self_).NPC).goalEntity.is_null() {
        return WAYPOINT_NONE;
    }

    if (*self_).waypoint == WAYPOINT_NONE && (*self_).noWaypointTime > level.time {
        //didn't have a valid one in about the past second, don't look again just yet
        return WAYPOINT_NONE;
    }
    if (*(*(*self_).NPC).goalEntity).waypoint == WAYPOINT_NONE
        && (*(*(*self_).NPC).goalEntity).noWaypointTime > level.time
    {
        //didn't have a valid one in about the past second, don't look again just yet
        return WAYPOINT_NONE;
    }
    if (*self_).noWaypointTime > level.time
        && (*(*(*self_).NPC).goalEntity).noWaypointTime > level.time
    {
        //just use current waypoints
        bestNode = trap::Nav_GetBestNodeAltRoute2(
            (*self_).waypoint,
            (*(*(*self_).NPC).goalEntity).waypoint,
            bestNode,
        );
    }
    //FIXME!!!!: this is making them wiggle back and forth between waypoints
    else if {
        bestNode =
            trap::Nav_GetBestPathBetweenEnts(self_, (*(*self_).NPC).goalEntity, NF_CLEAR_PATH);
        bestNode == NODE_NONE
    } {
        // !NAVNEW_GetWaypoints( self, qtrue ) )
        //one of us didn't have a valid waypoint!
        if (*self_).waypoint == NODE_NONE {
            //don't even try to find one again for a bit
            (*self_).noWaypointTime = level.time + Q_irand(500, 1500);
        }
        if (*(*(*self_).NPC).goalEntity).waypoint == NODE_NONE {
            //don't even try to find one again for a bit
            (*(*(*self_).NPC).goalEntity).noWaypointTime = level.time + Q_irand(500, 1500);
        }
        return WAYPOINT_NONE;
    } else if (*(*(*self_).NPC).goalEntity).noWaypointTime < level.time {
        (*(*(*self_).NPC).goalEntity).noWaypointTime = level.time + Q_irand(500, 1500);
    }

    while foundClearPath == QFALSE {
        inGoalWP = QFALSE;
        /*
        bestNode = trap_Nav_GetBestNodeAltRoute( self->waypoint, self->NPC->goalEntity->waypoint, bestNode );
        */

        if bestNode == WAYPOINT_NONE {
            return navnew_movetogoal_failed(self_, &mut origin);
        }

        //see if we can get directly to the next node off bestNode en route to goal's node...
        //NOTE: shouldn't be necc. now
        /*
        int oldBestNode = bestNode;
        bestNode = NAV_TestBestNode( self, self->waypoint, bestNode, qtrue );//, self->NPC->goalEntity->waypoint );//
        //NOTE: Guaranteed to return something
        if ( bestNode != oldBestNode )
        {//we were blocked somehow
            if ( setBlockedInfo )
            {
                self->NPC->aiFlags |= NPCAI_BLOCKED;
                trap_Nav_GetNodePosition( oldBestNode, NPCInfo->blockedDest );
            }
        }
        */
        trap::Nav_GetNodePosition(bestNode, &mut origin);
        /*
        if ( !goalWPFailed )
        {//we haven't already tried to go straight to goal or goal's wp
            if ( bestNode == self->NPC->goalEntity->waypoint )
            {//our bestNode is the goal's wp
                if ( NAV_HitNavGoal( self->r.currentOrigin, self->r.mins, self->r.maxs, origin, trap_Nav_GetNodeRadius( bestNode ), FlyingCreature( self ) ) )
                {//we're in the goal's wp
                    inGoalWP = qtrue;
                    //we're in the goalEntity's waypoint already
                    //so head for the goalEntity since we know it's clear of architecture
                    //FIXME: this is pretty stupid because the NPCs try to go straight
                    //		towards their goal before then even try macro_nav...
                    VectorCopy( self->NPC->goalEntity->r.currentOrigin, origin );
                }
            }
        }
        */
        if inGoalWP == QFALSE {
            //not heading straight for goal
            if bestNode == (*self_).waypoint {
                //we know it's clear or architecture
                //trap_Nav_GetNodePosition( self->waypoint, origin );
                /*
                if ( NAV_HitNavGoal( self->r.currentOrigin, self->r.mins, self->r.maxs, origin, trap_Nav_GetNodeRadius( bestNode ), FlyingCreature( self ) ) )
                {//we're in the wp we're heading for already
                    inBestWP = qtrue;
                }
                */
            } else {
                //heading to an edge off our confirmed clear waypoint... make sure it's clear
                //it it's not, bestNode will fall back to our waypoint
                let oldBestNode = bestNode;
                bestNode = NAV_TestBestNode(self_, (*self_).waypoint, bestNode, QTRUE);
                if bestNode == (*self_).waypoint {
                    //we fell back to our waypoint, reset the origin
                    (*(*self_).NPC).aiFlags |= NPCAI_BLOCKED;
                    trap::Nav_GetNodePosition(oldBestNode, &mut (*NPCInfo).blockedDest);
                    trap::Nav_GetNodePosition(bestNode, &mut origin);
                }
            }
        }
        //Com_Printf( "goalwp = %d, mywp = %d, node = %d, origin = %s\n", self->NPC->goalEntity->waypoint, self->waypoint, bestNode, vtos(origin) );

        tempInfo = *info;
        VectorSubtract(&origin, &(*self_).r.currentOrigin, &mut tempInfo.direction);
        VectorNormalize(&mut tempInfo.direction);

        //NOTE: One very important thing NAVNEW_AvoidCollision does is
        //		it actually CHANGES the value of "direction" - it changes it to
        //		whatever dir you need to go in to avoid the obstacle...
        foundClearPath = NAVNEW_AvoidCollision(
            self_,
            (*(*self_).NPC).goalEntity,
            &mut tempInfo,
            setBlockedInfo,
            5,
        );

        if foundClearPath == QFALSE {
            //blocked by an ent
            if inGoalWP != QFALSE {
                //we were heading straight for the goal, head for the goal's wp instead
                trap::Nav_GetNodePosition(bestNode, &mut origin);
                foundClearPath = NAVNEW_AvoidCollision(
                    self_,
                    (*(*self_).NPC).goalEntity,
                    &mut tempInfo,
                    setBlockedInfo,
                    5,
                );
            }
        }

        if foundClearPath != QFALSE {
            //clear!
            //If we got set to blocked, clear it
            NPC_ClearBlocked(self_);
            //Take the dir
            *info = tempInfo;
            if (*self_).s.weapon == WP_SABER {
                //jedi
                if info.direction[2] * info.distance > 64.0 {
                    (*(*self_).NPC).aiFlags |= NPCAI_BLOCKED;
                    VectorCopy(&origin, &mut (*NPCInfo).blockedDest);
                    return navnew_movetogoal_failed(self_, &mut origin);
                }
            }
        } else {
            //blocked by ent!
            if setBlockedInfo != QFALSE {
                (*(*self_).NPC).aiFlags |= NPCAI_BLOCKED;
                trap::Nav_GetNodePosition(bestNode, &mut (*NPCInfo).blockedDest);
            }
            //Only set blocked info first time
            setBlockedInfo = QFALSE;

            if inGoalWP != QFALSE {
                //we headed for our goal and failed and our goal's WP and failed
                if (*self_).waypoint == (*(*(*self_).NPC).goalEntity).waypoint {
                    //our waypoint is our goal's waypoint, nothing we can do
                    //remember that this node is blocked
                    trap::Nav_AddFailedNode(self_, (*self_).waypoint);
                    return navnew_movetogoal_failed(self_, &mut origin);
                } else {
                    //try going for our waypoint this time
                    goalWPFailed = QTRUE;
                    inGoalWP = QFALSE;
                }
            } else if bestNode != (*self_).waypoint {
                //we headed toward our next waypoint (instead of our waypoint) and failed
                if (*addr_of!(d_altRoutes)).integer != 0 {
                    //mark this edge failed and try our waypoint
                    //NOTE: don't assume there is something blocking the direct path
                    //			between my waypoint and the bestNode... I could be off
                    //			that path because of collision avoidance...
                    if (*addr_of!(d_patched)).integer != 0
                        && (trap::Nav_NodesAreNeighbors((*self_).waypoint, bestNode) == QFALSE
                            || NAVNEW_TestNodeConnectionBlocked(
                                (*self_).waypoint,
                                bestNode,
                                self_,
                                (*(*(*self_).NPC).goalEntity).s.number,
                                QFALSE,
                                QTRUE,
                            ) != QFALSE)
                    {
                        //use patch-style navigation
                        //the direct path between these 2 nodes is blocked by an ent
                        trap::Nav_AddFailedEdge((*self_).s.number, (*self_).waypoint, bestNode);
                    }
                    bestNode = (*self_).waypoint;
                } else {
                    //we should stop
                    return navnew_movetogoal_failed(self_, &mut origin);
                }
            } else {
                //we headed for *our* waypoint and couldn't get to it
                if (*addr_of!(d_altRoutes)).integer != 0 {
                    //remember that this node is blocked
                    trap::Nav_AddFailedNode(self_, (*self_).waypoint);
                    //Now we should get our waypoints again
                    //FIXME: cache the trace-data for subsequent calls as only the route info would have changed
                    //if ( (bestNode = trap_Nav_GetBestPathBetweenEnts( self, self->NPC->goalEntity, NF_CLEAR_PATH )) == NODE_NONE )//!NAVNEW_GetWaypoints( self, qfalse ) )
                    {
                        //one of our waypoints is WAYPOINT_NONE now
                        return navnew_movetogoal_failed(self_, &mut origin);
                    }
                } else {
                    //we should stop
                    return navnew_movetogoal_failed(self_, &mut origin);
                }
            }

            numTries += 1;
            if numTries >= 10 {
                return navnew_movetogoal_failed(self_, &mut origin);
            }
        }
    }

    //finish:
    //Draw any debug info, if requested
    if NAVDEBUG_showEnemyPath != QFALSE {
        let mut dest: vec3_t = [0.0; 3];
        let mut start: vec3_t = [0.0; 3];

        //Get the positions
        trap::Nav_GetNodePosition((*(*(*self_).NPC).goalEntity).waypoint, &mut dest);
        trap::Nav_GetNodePosition(bestNode, &mut start);

        //Draw the route
        G_DrawNode(&start, NODE_START);
        if bestNode != (*self_).waypoint {
            let mut wpPos: vec3_t = [0.0; 3];
            trap::Nav_GetNodePosition((*self_).waypoint, &mut wpPos);
            G_DrawNode(&wpPos, NODE_NAVGOAL);
        }
        G_DrawNode(&dest, NODE_GOAL);
        G_DrawEdge(
            &dest,
            &(*(*(*self_).NPC).goalEntity).r.currentOrigin,
            EDGE_PATH,
        );
        G_DrawNode(&(*(*(*self_).NPC).goalEntity).r.currentOrigin, NODE_GOAL);
        trap::Nav_ShowPath(bestNode, (*(*(*self_).NPC).goalEntity).waypoint);
    }

    (*(*self_).NPC).shoveCount = 0;

    //let me keep this waypoint for a while
    if (*self_).noWaypointTime < level.time {
        (*self_).noWaypointTime = level.time + Q_irand(500, 1500);
    }

    bestNode
}

// `failed:` label of `NAVNEW_MoveToGoal` (g_navnew.c:839) — C `goto failed` jumps
// here from every dead-end above; the active path always returns WAYPOINT_NONE (the
// alternate-route retry below is dead in the original).
unsafe fn navnew_movetogoal_failed(self_: *mut gentity_t, origin: &mut vec3_t) -> i32 {
    //FIXME: What we should really do here is have a list of the goal's and our
    //		closest clearpath waypoints, ranked.  If the first set fails, try the rest
    //		until there are no alternatives.

    trap::Nav_GetNodePosition((*self_).waypoint, origin);

    //do this to avoid ping-ponging?
    WAYPOINT_NONE
    /*
    //this was causing ping-ponging
    if ( DistanceSquared( origin, self->r.currentOrigin ) < 16 )//woo, magic number
    {//We're right up on our waypoint, so that won't help, return none
        //Or maybe find the nextbest here?
        return WAYPOINT_NONE;
    }
    else
    {//Try going to our waypoint
        bestNode = self->waypoint;

        VectorSubtract( origin, self->r.currentOrigin, info.direction );
        VectorNormalize( info.direction );
    }

    goto finish;
    */
}
