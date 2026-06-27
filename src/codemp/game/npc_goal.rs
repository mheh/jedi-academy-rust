//! Slice of `NPC_goal.c` (`b_goal.cpp`) — the NPC goal-entity bookkeeping layer:
//! the small set/clear helpers that push and pop `NPCInfo->goalEntity`, plus the
//! pure `G_BoundsOverlap` AABB test.
//!
//! `NPC_ReachedGoal` (:117) also lands here — it pops the goal stack and reports
//! the move-nav task complete via the ICARUS task bridge. The goal-navigation core
//! `ReachedGoal` (:136) and `UpdateGoal` (:234) round the file out — both funnel into
//! the now-ported NAV subsystem (`NAV_HitNavGoal`) and the cross-file `FlyingCreature`.
//!
//! Ported here so far: `SetGoal` (NPC_goal.c:10), `NPC_SetGoal` (:31),
//! `NPC_ClearGoal` (:64), `G_BoundsOverlap` (:94), `NPC_ReachedGoal` (:117),
//! `ReachedGoal` (:136), `UpdateGoal` (:234).

#![allow(non_snake_case)] // C function names kept verbatim

use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::level;
use crate::codemp::game::npc::{ucmd, NPCInfo, NPC};

use crate::codemp::game::b_public_h::{NPCAI_MOVING, NPCAI_TOUCHED_GOAL};
use crate::codemp::game::bg_public::EF_NODRAW;
use crate::codemp::game::g_nav::{FlyingCreature, NAV_HitNavGoal};
use crate::codemp::game::g_public_h::TID_MOVE_NAV;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

use core::ptr::addr_of;

/// `void SetGoal( gentity_t *goal, float rating )` (NPC_goal.c:10).
///
/// Sets the current NPC's `goalEntity`/`goalTime`. The `rating` (would-be
/// `goalEntityNeed`), the `NAV_ClearLastRoute` route reset, and the
/// `Debug_NPCPrintf` logging are all commented out in the original C, so this is a
/// pure two-field write. No oracle (mutates the process-global `NPCInfo`/`level`).
pub unsafe fn SetGoal(goal: *mut gentity_t, _rating: f32) {
    (*NPCInfo).goalEntity = goal;
    //	NPCInfo->goalEntityNeed = rating;
    (*NPCInfo).goalTime = (*addr_of!(level)).time;
    //	NAV_ClearLastRoute(NPC);
    if !goal.is_null() {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_SetGoal: %s @ %s (%f)\n", goal->classname, vtos( goal->currentOrigin), rating );
    } else {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_SetGoal: NONE\n" );
    }
}

/// `void NPC_SetGoal( gentity_t *goal, float rating )` (NPC_goal.c:31).
///
/// Pushes a new goal entity: no-ops if `goal` is already the current goal, NULL, or
/// a client; otherwise saves the existing goal as `lastGoalEntity` and delegates to
/// `SetGoal`. All `Debug_NPCPrintf` calls are commented out in the original C. No
/// oracle (mutates the process-global `NPCInfo`).
pub unsafe fn NPC_SetGoal(goal: *mut gentity_t, rating: f32) {
    if goal == (*NPCInfo).goalEntity {
        return;
    }

    if goal.is_null() {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_ERROR, "NPC_SetGoal: NULL goal\n" );
        return;
    }

    if !(*goal).client.is_null() {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_ERROR, "NPC_SetGoal: goal is a client\n" );
        return;
    }

    if !(*NPCInfo).goalEntity.is_null() {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_SetGoal: push %s\n", NPCInfo->goalEntity->classname );
        (*NPCInfo).lastGoalEntity = (*NPCInfo).goalEntity;
        //		NPCInfo->lastGoalEntityNeed = NPCInfo->goalEntityNeed;
    }

    SetGoal(goal, rating);
}

/// `void NPC_ClearGoal( void )` (NPC_goal.c:64).
///
/// Pops the goal stack: if there is no `lastGoalEntity`, clears to NULL; otherwise
/// restores `lastGoalEntity` as the goal (if it is still in use and drawn) and nulls
/// `lastGoalEntity`. The `NAV_ClearLastRoute` and `Debug_NPCPrintf` calls are
/// commented out in the original C. No oracle (mutates the process-global `NPCInfo`).
pub unsafe fn NPC_ClearGoal() {
    let goal: *mut gentity_t;

    if (*NPCInfo).lastGoalEntity.is_null() {
        SetGoal(core::ptr::null_mut(), 0.0);
        return;
    }

    goal = (*NPCInfo).lastGoalEntity;
    (*NPCInfo).lastGoalEntity = core::ptr::null_mut();
    //	NAV_ClearLastRoute(NPC);
    if (*goal).inuse == QTRUE && (*goal).s.eFlags & EF_NODRAW == 0 {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_ClearGoal: pop %s\n", goal->classname );
        SetGoal(goal, 0.0); //, NPCInfo->lastGoalEntityNeed
        return;
    }

    SetGoal(core::ptr::null_mut(), 0.0);
}

/// `qboolean G_BoundsOverlap( const vec3_t mins1, const vec3_t maxs1, const vec3_t
/// mins2, const vec3_t maxs2 )` (NPC_goal.c:94).
///
/// Axis-aligned bounding-box overlap test. NOTE: flush up against counts as
/// overlapping. Pure float comparisons — oracle-tested.
pub fn G_BoundsOverlap(
    mins1: &vec3_t,
    maxs1: &vec3_t,
    mins2: &vec3_t,
    maxs2: &vec3_t,
) -> qboolean {
    //NOTE: flush up against counts as overlapping
    if mins1[0] > maxs2[0] {
        return QFALSE;
    }

    if mins1[1] > maxs2[1] {
        return QFALSE;
    }

    if mins1[2] > maxs2[2] {
        return QFALSE;
    }

    if maxs1[0] < mins2[0] {
        return QFALSE;
    }

    if maxs1[1] < mins2[1] {
        return QFALSE;
    }

    if maxs1[2] < mins2[2] {
        return QFALSE;
    }

    QTRUE
}

/// `void NPC_ReachedGoal( void )` (NPC_goal.c:117).
///
/// Called when the current NPC has reached its goal entity: pops the goal stack,
/// timestamps `goalTime`, clears the `NPCAI_MOVING` flag, zeroes the forward move,
/// and reports the `TID_MOVE_NAV` task complete through the ICARUS bridge. The
/// leading `Debug_NPCPrintf` is commented out in the original C. No oracle (mutates
/// the process-global `NPCInfo`/`ucmd` and calls a `trap_*`).
pub unsafe fn NPC_ReachedGoal() {
    //	Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "UpdateGoal: reached goal entity\n" );
    NPC_ClearGoal();
    (*NPCInfo).goalTime = (*addr_of!(level)).time;

    //MCG - Begin
    (*NPCInfo).aiFlags &= !NPCAI_MOVING;
    ucmd.forwardmove = 0;
    //Return that the goal was reached
    trap::ICARUS_TaskIDComplete(NPC, TID_MOVE_NAV);
    //MCG - End
}

/// `qboolean ReachedGoal( gentity_t *goal )` (NPC_goal.c:136).
///
/// Tests whether the current NPC has reached `goal`. id removed the old waypoint /
/// surface checks (the large `FIXME` block is commented out in the original C), so the
/// live body is two cases: an explicit `NPCAI_TOUCHED_GOAL` latch (consumed and reported
/// as reached), otherwise a `NAV_HitNavGoal` bounds test against the goal origin at the
/// NPC's `goalRadius`. No oracle (reads/mutates the process-global `NPC`/`NPCInfo`).
pub unsafe fn ReachedGoal(goal: *mut gentity_t) -> qboolean {
    //FIXME: For script waypoints, need a special check (id's waypoint/surface checks
    //       are all commented out in the original C — see NPC_goal.c).
    if (*NPCInfo).aiFlags & NPCAI_TOUCHED_GOAL != 0 {
        (*NPCInfo).aiFlags &= !NPCAI_TOUCHED_GOAL;
        return QTRUE;
    }

    NAV_HitNavGoal(
        &(*NPC).r.currentOrigin,
        &(*NPC).r.mins,
        &(*NPC).r.maxs,
        &(*goal).r.currentOrigin,
        (*NPCInfo).goalRadius,
        FlyingCreature(NPC),
    )
}

/// `gentity_t *UpdateGoal( void )` (NPC_goal.c:234).
///
/// Returns the NPC's current goal entity, or NULL. Bails on no goal; if the goal was
/// somehow freed without being cleared, calls `NPC_ClearGoal` and returns NULL; if the
/// goal has been reached, pops it via `NPC_ReachedGoal` and returns NULL so the NPC
/// stops trying to move to it. (Per the original C, goal is ALWAYS `goalEntity` now —
/// id removed the waypoint plumbing.) No oracle (reads/mutates process-global `NPCInfo`).
pub unsafe fn UpdateGoal() -> *mut gentity_t {
    let mut goal: *mut gentity_t;

    if (*NPCInfo).goalEntity.is_null() {
        return core::ptr::null_mut();
    }

    if (*(*NPCInfo).goalEntity).inuse == QFALSE {
        //Somehow freed it, but didn't clear it
        NPC_ClearGoal();
        return core::ptr::null_mut();
    }

    goal = (*NPCInfo).goalEntity;

    if ReachedGoal(goal) != QFALSE {
        NPC_ReachedGoal();
        goal = core::ptr::null_mut(); //so they don't keep trying to move to it
    } //else if fail, need to tell script so?

    goal
}

#[cfg(all(test, feature = "oracle"))]
mod oracle_tests {
    use super::*;
    use crate::oracle;
    use core::ffi::c_int;

    /// `G_BoundsOverlap` over overlapping, disjoint, and flush-up-against box pairs
    /// (the last category exercises the inclusive `>`/`<` boundary).
    #[test]
    fn g_boundsoverlap_matches_oracle() {
        let cases: &[(vec3_t, vec3_t, vec3_t, vec3_t)] = &[
            // clearly overlapping
            ([-10.0, -10.0, -10.0], [10.0, 10.0, 10.0], [0.0, 0.0, 0.0], [5.0, 5.0, 5.0]),
            // disjoint in x
            ([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [20.0, 0.0, 0.0], [30.0, 10.0, 10.0]),
            // disjoint in y
            ([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [0.0, 20.0, 0.0], [10.0, 30.0, 10.0]),
            // disjoint in z
            ([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [0.0, 0.0, 20.0], [10.0, 10.0, 30.0]),
            // flush up against in x (counts as overlapping)
            ([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]),
            // flush up against in y
            ([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [0.0, 10.0, 0.0], [10.0, 20.0, 10.0]),
            // flush up against in z
            ([0.0, 0.0, 0.0], [10.0, 10.0, 10.0], [0.0, 0.0, 10.0], [10.0, 10.0, 20.0]),
            // one fully inside the other
            ([-5.0, -5.0, -5.0], [5.0, 5.0, 5.0], [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
            // touching only at a single corner
            ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]),
        ];
        for (mins1, maxs1, mins2, maxs2) in cases {
            let got = G_BoundsOverlap(mins1, maxs1, mins2, maxs2);
            let want = unsafe {
                oracle::jka_G_BoundsOverlap(
                    mins1.as_ptr(),
                    maxs1.as_ptr(),
                    mins2.as_ptr(),
                    maxs2.as_ptr(),
                )
            };
            assert_eq!(got as c_int, want, "G_BoundsOverlap mismatch for {mins1:?} {maxs1:?} {mins2:?} {maxs2:?}");
        }
    }
}
