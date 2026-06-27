//! Slice of `NPC_AI_Utils.c` — NPC AI group-management helpers
//! (`AIGroupInfo_t` collection/sort/validate logic).
//!
//! These utilities are meant for strictly non-player, non-team NPCs. These functions
//! are in their own file because they are only intended for use with NPCs who's logic
//! has been overriden from the original AI code (the `AI_`-prefixed files).
//!
//! Ported: `AI_GetGroupSize` (NPC_AI_Utils.c:23), `AI_GetGroupSize2` (:71),
//! `AI_ClosestGroupEntityNumToPoint` (:80), `AI_SetClosestBuddy` (:112),
//! `AI_SortGroupByPathCostToEnemy` (:134), `AI_FindSelfInPreviousGroup` (:212),
//! `AI_InsertGroupMember` (:232), `AI_TryJoinPreviousGroup` (:259),
//! `AI_GetNextEmptyGroup` (:279), `AI_ValidateNoEnemyGroupMember` (:310),
//! `AI_ValidateGroupMember` (:342), `AI_GetGroup` (:443),
//! `AI_SetNewGroupCommander` (:553), `AI_DeleteGroupMember` (:570),
//! `AI_DeleteSelfFromGroup` (:602), `AI_GroupUpdateEnemyLastSeen` (:699),
//! `AI_GroupUpdateClearShotTime` (:709), `AI_GroupUpdateSquadstates` (:718),
//! `AI_RefreshGroup` (:740), `AI_UpdateGroups` (:964),
//! `AI_GroupContainsEntNum` (:982), `AI_CheckEnemyCollision` (:1030),
//! `AI_DistributeAttack` (:1065), `AI_GroupMemberKilled` (:620).
//!
//! The commented-out `AI_GetGroup( AIGroupInfo_t &group, ... )` reference-param overload
//! (:1002) is `/* */`'d-out in the retail source — only the single `gentity_t*` form exists.

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_int;
use core::ptr::{addr_of, null_mut};

use crate::codemp::game::ai_h::{
    AIGroupInfo_t, AIGroupMember_t, MAX_FRAME_GROUPS, MAX_GROUP_MEMBERS, NUM_SQUAD_STATES,
};
use crate::codemp::game::ai_wpnav::G_TestLine;
use crate::codemp::game::b_public_h::{RANK_ENSIGN, SCF_NO_GROUPS};
use crate::codemp::game::bg_public::team_t;
use crate::codemp::game::bg_weapons_h::{
    WP_BRYAR_PISTOL, WP_DET_PACK, WP_DISRUPTOR, WP_EMPLACED_GUN, WP_FLECHETTE, WP_REPEATER,
    WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use crate::codemp::game::g_local::{gentity_t, AEL_DANGER_GREAT, FRAMETIME};
use crate::codemp::game::g_main::{d_noGroupAI, debugNPCAI, g_entities, level};
use crate::codemp::game::g_nav::{
    NAV_FindClosestWaypointForEnt, NAV_FindClosestWaypointForPoint, WAYPOINT_NONE,
};
use crate::codemp::game::g_public_h::Q3_INFINITE;
use crate::codemp::game::g_timer::TIMER_Done;
use crate::codemp::game::npc_ai_stormtrooper::{ST_AggressionAdjust, ST_MarkToCover, ST_StartFlee};
use crate::codemp::game::npc_combat::G_SetEnemy;
use crate::codemp::game::npc_move::NAV_GetLastMove;
use crate::codemp::game::q_math::{DistanceSquared, VectorCopy};
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_shared_h::{vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD};
use crate::codemp::game::teams_h::{
    CLASS_ATST, CLASS_HOWLER, CLASS_INTERROGATOR, CLASS_MARK1, CLASS_MARK2, CLASS_MINEMONSTER,
    CLASS_PROBE, CLASS_REMOTE, CLASS_SEEKER, CLASS_SENTRY,
};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// #define MAX_RADIUS_ENTS 128 (NPC_AI_Utils.c:9)
const MAX_RADIUS_ENTS: usize = 128;

/// `int AI_GetGroupSize( vec3_t origin, int radius, team_t playerTeam, gentity_t *avoid )`
/// (NPC_AI_Utils.c:23).
///
/// Count the living same-team clients (other than `avoid`) whose bbox overlaps the
/// `radius`-cube around `origin`. No oracle — drives `trap_EntitiesInBox` and reads the
/// `g_entities` global.
///
/// # Safety
/// `origin` must point to a valid `vec3_t`; `avoid` may be null. Reads the `g_entities`
/// global, which must be set up.
pub unsafe fn AI_GetGroupSize(
    origin: &vec3_t,
    radius: c_int,
    playerTeam: team_t,
    avoid: *mut gentity_t,
) -> c_int {
    let mut radiusEnts: [c_int; MAX_RADIUS_ENTS] = [0; MAX_RADIUS_ENTS];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let numEnts: c_int;
    let mut realCount: c_int = 0;

    //Setup the bbox to search in
    for i in 0..3 {
        mins[i] = origin[i] - radius as f32;
        maxs[i] = origin[i] + radius as f32;
    }

    //Get the number of entities in a given space
    numEnts = trap::EntitiesInBox(&mins, &maxs, &mut radiusEnts);

    //Cull this list
    let mut j = 0;
    while j < numEnts {
        let check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(radiusEnts[j as usize] as usize);

        //Validate clients
        if (*check).client.is_null() {
            j += 1;
            continue;
        }

        //Skip the requested avoid ent if present
        if !avoid.is_null() && check == avoid {
            j += 1;
            continue;
        }

        //Must be on the same team
        if (*(*check).client).playerTeam != playerTeam {
            j += 1;
            continue;
        }

        //Must be alive
        if (*check).health <= 0 {
            j += 1;
            continue;
        }

        realCount += 1;
        j += 1;
    }

    realCount
}

//Overload

/// `int AI_GetGroupSize2( gentity_t *ent, int radius )` (NPC_AI_Utils.c:71).
///
/// Overload of [`AI_GetGroupSize`] keyed off `ent`. Returns `-1` for a null/clientless
/// `ent`. No oracle.
///
/// # Safety
/// `ent` may be null (checked).
pub unsafe fn AI_GetGroupSize2(ent: *mut gentity_t, radius: c_int) -> c_int {
    if ent.is_null() || (*ent).client.is_null() {
        return -1;
    }

    AI_GetGroupSize(
        &(*ent).r.currentOrigin,
        radius,
        (*(*ent).client).playerTeam,
        ent,
    )
}

/// `int AI_ClosestGroupEntityNumToPoint( AIGroupInfo_t *group, vec3_t point )`
/// (NPC_AI_Utils.c:80).
///
/// Entity number of the group member with the cheapest nav path to `point`. Returns
/// `ENTITYNUM_NONE` for an empty group or when no waypoint covers `point`. No oracle —
/// drives `NAV_FindClosestWaypointForPoint` and `trap_Nav_GetPathCost`.
///
/// # Safety
/// `group` may be null (checked); `point` must point to a valid `vec3_t`.
pub unsafe fn AI_ClosestGroupEntityNumToPoint(group: *mut AIGroupInfo_t, point: &vec3_t) -> c_int {
    let markerWP: c_int;
    let mut bestCost: c_int = Q3_INFINITE;
    let mut closest: c_int = ENTITYNUM_NONE;

    if group.is_null() || (*group).numGroup <= 0 {
        return ENTITYNUM_NONE;
    }

    markerWP = NAV_FindClosestWaypointForPoint(
        (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[0].number as usize),
        point,
    );

    if markerWP == WAYPOINT_NONE {
        return ENTITYNUM_NONE;
    }

    for i in 0..(*group).numGroup {
        let cost = trap::Nav_GetPathCost((*group).member[i as usize].waypoint, markerWP);
        if cost < bestCost {
            bestCost = cost;
            closest = (*group).member[i as usize].number;
        }
    }

    closest
}

/// `void AI_SetClosestBuddy( AIGroupInfo_t *group )` (NPC_AI_Utils.c:112).
///
/// For each member, record the nearest fellow member's entity number in
/// `member.closestBuddy`. No oracle — pure geometry over the `g_entities` origins.
///
/// # Safety
/// `group` must point to a valid `AIGroupInfo_t`; reads the `g_entities` global.
pub unsafe fn AI_SetClosestBuddy(group: *mut AIGroupInfo_t) {
    for i in 0..(*group).numGroup {
        (*group).member[i as usize].closestBuddy = ENTITYNUM_NONE;

        let mut bestDist = Q3_INFINITE;
        for j in 0..(*group).numGroup {
            let dist = DistanceSquared(
                &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[i as usize].number as usize))
                    .r
                    .currentOrigin,
                &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[j as usize].number as usize))
                    .r
                    .currentOrigin,
            ) as c_int;
            if dist < bestDist {
                bestDist = dist;
                (*group).member[i as usize].closestBuddy = (*group).member[j as usize].number;
            }
        }
    }
}

/// `void AI_SortGroupByPathCostToEnemy( AIGroupInfo_t *group )` (NPC_AI_Utils.c:134).
///
/// Order the group's members by ascending nav path-cost to the group enemy's closest
/// waypoint, when at least one member has a route. No oracle — drives
/// `NAV_FindClosestWaypointForEnt` and `trap_Nav_GetPathCost`.
///
/// NOTE: the C insertion-sort bump loop is `for ( k = group->numGroup; k > j; k++ )` —
/// `k++` (not `k--`) is a latent upstream bug that, if that branch is ever reached, runs
/// off the end of `bestMembers` and loops without terminating. Ported verbatim via a raw
/// pointer (mirroring C's unchecked array access) so the behaviour is identical to the
/// retail module.
///
/// # Safety
/// `group` must point to a valid `AIGroupInfo_t`.
pub unsafe fn AI_SortGroupByPathCostToEnemy(group: *mut AIGroupInfo_t) {
    let mut bestMembers: [AIGroupMember_t; MAX_GROUP_MEMBERS] = [AIGroupMember_t::default(); MAX_GROUP_MEMBERS];
    let mut sort: qboolean = QFALSE;

    if !(*group).enemy.is_null() {
        //FIXME: just use enemy->waypoint?
        (*group).enemyWP = NAV_FindClosestWaypointForEnt((*group).enemy, WAYPOINT_NONE);
    } else {
        (*group).enemyWP = WAYPOINT_NONE;
    }

    for i in 0..(*group).numGroup {
        let i = i as usize;
        if (*group).enemyWP == WAYPOINT_NONE {
            //FIXME: just use member->waypoint?
            (*group).member[i].waypoint = WAYPOINT_NONE;
            (*group).member[i].pathCostToEnemy = Q3_INFINITE;
        } else {
            //FIXME: just use member->waypoint?
            (*group).member[i].waypoint =
                NAV_FindClosestWaypointForEnt((*group).enemy, WAYPOINT_NONE);
            if (*group).member[i].waypoint != WAYPOINT_NONE {
                (*group).member[i].pathCostToEnemy =
                    trap::Nav_GetPathCost((*group).member[i].waypoint, (*group).enemyWP);
                //at least one of us has a path, so do sorting
                sort = QTRUE;
            } else {
                (*group).member[i].pathCostToEnemy = Q3_INFINITE;
            }
        }
    }
    //Now sort
    if sort != QFALSE {
        let best = bestMembers.as_mut_ptr();
        //initialize bestMembers data
        for j in 0..(*group).numGroup {
            (*best.add(j as usize)).number = ENTITYNUM_NONE;
        }

        for i in 0..(*group).numGroup {
            for j in 0..(*group).numGroup {
                if (*best.add(j as usize)).number != ENTITYNUM_NONE {
                    //slot occupied
                    if (*group).member[i as usize].pathCostToEnemy
                        < (*best.add(j as usize)).pathCostToEnemy
                    {
                        //this guy has a shorter path than the one currenly in this spot, bump him and put myself in here
                        let mut k = (*group).numGroup;
                        while k > j {
                            *best.add(k as usize) = *best.add((k - 1) as usize);
                            k += 1; // NOTE: verbatim C `k++` (latent upstream bug)
                        }
                        *best.add(j as usize) = (*group).member[i as usize];
                        break;
                    }
                } else {
                    //slot unoccupied, reached end of list, throw self in here
                    *best.add(j as usize) = (*group).member[i as usize];
                    break;
                }
            }
        }

        //Okay, now bestMembers is a sorted list, just copy it into group->members
        for i in 0..(*group).numGroup {
            (*group).member[i as usize] = *best.add(i as usize);
        }
    }
}

/// `qboolean AI_FindSelfInPreviousGroup( gentity_t *self )` (NPC_AI_Utils.c:212).
///
/// Go through other groups made this frame and see if any of those contain me already;
/// if so, point `self->NPC->group` at it. No oracle — walks `level.groups`.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` whose `NPC` is non-null.
pub unsafe fn AI_FindSelfInPreviousGroup(self_: *mut gentity_t) -> qboolean {
    for i in 0..MAX_FRAME_GROUPS {
        if level.groups[i].numGroup != 0
        /*&& level.groups[i].enemy != NULL */
        {
            //check this one
            for j in 0..level.groups[i].numGroup {
                if level.groups[i].member[j as usize].number == (*self_).s.number {
                    (*(*self_).NPC).group = addr_of!(level.groups[i]) as *mut AIGroupInfo_t;
                    return QTRUE;
                }
            }
        }
    }
    QFALSE
}

/// `void AI_InsertGroupMember( AIGroupInfo_t *group, gentity_t *member )`
/// (NPC_AI_Utils.c:232).
///
/// Add `member` to `group` (unless already present), tally its squad-state, track the
/// highest-ranked member as commander, and back-link `member->NPC->group`. No oracle.
///
/// # Safety
/// `group` and `member` must be valid; `member->NPC` must be non-null.
pub unsafe fn AI_InsertGroupMember(group: *mut AIGroupInfo_t, member: *mut gentity_t) {
    let mut i: c_int = 0;

    //okay, you know what?  Check this damn group and make sure we're not already in here!
    while i < (*group).numGroup {
        if (*group).member[i as usize].number == (*member).s.number {
            //already in here
            break;
        }
        i += 1;
    }
    if i < (*group).numGroup {
        //found him in group already
    } else {
        //add him in
        let idx = (*group).numGroup;
        (*group).numGroup += 1;
        (*group).member[idx as usize].number = (*member).s.number;
        (*group).numState[(*(*member).NPC).squadState as usize] += 1;
    }
    if (*group).commander.is_null()
        || (*(*member).NPC).rank > (*(*(*group).commander).NPC).rank
    {
        //keep track of highest rank
        (*group).commander = member;
    }
    (*(*member).NPC).group = group;
}

/// `qboolean AI_TryJoinPreviousGroup( gentity_t *self )` (NPC_AI_Utils.c:259).
///
/// Go through other groups made this frame and see if any of those have the same enemy
/// as me; if so (and I'm a valid member), add me in. No oracle — walks `level.groups`.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` whose `NPC` is non-null.
pub unsafe fn AI_TryJoinPreviousGroup(self_: *mut gentity_t) -> qboolean {
    for i in 0..MAX_FRAME_GROUPS {
        if level.groups[i].numGroup != 0
            && level.groups[i].numGroup < (MAX_GROUP_MEMBERS as c_int - 1)
            //&& level.groups[i].enemy != NULL
            && level.groups[i].enemy == (*self_).enemy
        {
            //has members, not full and has my enemy
            let grp = addr_of!(level.groups[i]) as *mut AIGroupInfo_t;
            if AI_ValidateGroupMember(grp, self_) != QFALSE {
                //I am a valid member for this group
                AI_InsertGroupMember(grp, self_);
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// `qboolean AI_GetNextEmptyGroup( gentity_t *self )` (NPC_AI_Utils.c:279).
///
/// Find (or reuse) a group slot for `self`: first try an existing group I'm already in
/// or can join, otherwise grab an empty `level.groups` slot. Returns `qtrue` only when a
/// fresh group was allocated for me (the caller then initializes it). No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` whose `NPC` is non-null.
pub unsafe fn AI_GetNextEmptyGroup(self_: *mut gentity_t) -> qboolean {
    if AI_FindSelfInPreviousGroup(self_) != QFALSE {
        //already in one, no need to make a new one
        return QFALSE;
    }

    if AI_TryJoinPreviousGroup(self_) != QFALSE {
        //try to just put us in one that already exists
        return QFALSE;
    }

    //okay, make a whole new one, then
    for i in 0..MAX_FRAME_GROUPS {
        if level.groups[i].numGroup == 0 {
            //make a new one
            (*(*self_).NPC).group = addr_of!(level.groups[i]) as *mut AIGroupInfo_t;
            return QTRUE;
        }
    }

    //if ( i >= MAX_FRAME_GROUPS )
    {
        //WTF?  Out of groups!
        (*(*self_).NPC).group = null_mut();
        QFALSE
    }
}

/// `qboolean AI_ValidateNoEnemyGroupMember( AIGroupInfo_t *group, gentity_t *member )`
/// (NPC_AI_Utils.c:310).
///
/// For an enemy-less (patrol) group, accept `member` only if it is within 384u and the
/// PVS of the group's center (the commander, or failing that the first member). No
/// oracle — drives `trap_InPVS`.
///
/// # Safety
/// `group` may be null (checked); `member` must point to a valid `gentity_t`.
pub unsafe fn AI_ValidateNoEnemyGroupMember(
    group: *mut AIGroupInfo_t,
    member: *mut gentity_t,
) -> qboolean {
    let mut center: vec3_t = [0.0; 3];

    if group.is_null() {
        return QFALSE;
    }
    if !(*group).commander.is_null() {
        VectorCopy(&(*(*group).commander).r.currentOrigin, &mut center);
    } else {
        //hmm, just pick the first member
        if (*group).member[0].number < 0 || (*group).member[0].number >= ENTITYNUM_WORLD {
            return QFALSE;
        }
        VectorCopy(
            &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[0].number as usize))
                .r
                .currentOrigin,
            &mut center,
        );
    }
    //FIXME: maybe it should be based on the center of the mass of the group, not the commander?
    if DistanceSquared(&center, &(*member).r.currentOrigin) > 147456.0
    /*384*384*/
    {
        return QFALSE;
    }
    if trap::InPVS(&(*member).r.currentOrigin, &center) == QFALSE {
        //not within PVS of the group enemy
        return QFALSE;
    }
    QTRUE
}

/// `qboolean AI_ValidateGroupMember( AIGroupInfo_t *group, gentity_t *member )`
/// (NPC_AI_Utils.c:342).
///
/// Decide whether `member` may belong to `group`: an aware, alive, same-team, group-
/// allowed squad-type NPC not already in another group, with a compatible weapon/class,
/// and sharing the group's enemy (or being within range for an enemy-less group). No
/// oracle — entity/NPC/client-field logic + `trap_InPVS` (via the no-enemy helper) +
/// `TIMER_Done`.
///
/// # Safety
/// `group` and `member` may be null/null-fielded (checked). Reads the `level` global.
pub unsafe fn AI_ValidateGroupMember(
    group: *mut AIGroupInfo_t,
    member: *mut gentity_t,
) -> qboolean {
    //Validate ents
    if member.is_null() {
        return QFALSE;
    }

    //Validate clients
    if (*member).client.is_null() {
        return QFALSE;
    }

    //Validate NPCs
    if (*member).NPC.is_null() {
        return QFALSE;
    }

    //must be aware
    if (*(*member).NPC).confusionTime > level.time {
        return QFALSE;
    }

    //must be allowed to join groups
    if (*(*member).NPC).scriptFlags & SCF_NO_GROUPS != 0 {
        return QFALSE;
    }

    //Must not be in another group
    if !(*(*member).NPC).group.is_null() && (*(*member).NPC).group != group {
        //FIXME: if that group's enemy is mine, why not absorb that group into mine?
        return QFALSE;
    }

    //Must be alive
    if (*member).health <= 0 {
        return QFALSE;
    }

    //can't be in an emplaced gun
    //	if( member->s.eFlags & EF_LOCKED_TO_WEAPON )
    //		return qfalse;
    //rwwFIXMEFIXME: support this flag

    //Must be on the same team
    if (*(*member).client).playerTeam != (*group).team {
        return QFALSE;
    }

    if (*(*member).client).ps.weapon == WP_SABER //	!= self->s.weapon )
		|| (*(*member).client).ps.weapon == WP_THERMAL
		|| (*(*member).client).ps.weapon == WP_DISRUPTOR
		|| (*(*member).client).ps.weapon == WP_EMPLACED_GUN
//		|| member->client->ps.weapon == WP_BOT_LASER		// Probe droid	- Laser blast
		|| (*(*member).client).ps.weapon == WP_STUN_BATON
		|| (*(*member).client).ps.weapon == WP_TURRET
    /*||			// turret guns
		member->client->ps.weapon == WP_ATST_MAIN ||
		member->client->ps.weapon == WP_ATST_SIDE ||
		member->client->ps.weapon == WP_TIE_FIGHTER*/
    {
        //not really a squad-type guy
        return QFALSE;
    }

    if (*(*member).client).NPC_class == CLASS_ATST
        || (*(*member).client).NPC_class == CLASS_PROBE
        || (*(*member).client).NPC_class == CLASS_SEEKER
        || (*(*member).client).NPC_class == CLASS_REMOTE
        || (*(*member).client).NPC_class == CLASS_SENTRY
        || (*(*member).client).NPC_class == CLASS_INTERROGATOR
        || (*(*member).client).NPC_class == CLASS_MINEMONSTER
        || (*(*member).client).NPC_class == CLASS_HOWLER
        || (*(*member).client).NPC_class == CLASS_MARK1
        || (*(*member).client).NPC_class == CLASS_MARK2
    {
        //these kinds of enemies don't actually use this group AI
        return QFALSE;
    }

    //should have same enemy
    if (*member).enemy != (*group).enemy {
        if !(*member).enemy.is_null() {
            //he's fighting someone else, leave him out
            return QFALSE;
        }
        if trap::InPVS(
            &(*member).r.currentOrigin,
            &(*(*group).enemy).r.currentOrigin,
        ) == QFALSE
        {
            //not within PVS of the group enemy
            return QFALSE;
        }
    } else if (*group).enemy.is_null() {
        //if the group is a patrol group, only take those within the room and radius
        if AI_ValidateNoEnemyGroupMember(group, member) == QFALSE {
            return QFALSE;
        }
    }
    //must be actually in combat mode
    if TIMER_Done(member, c"interrogating".as_ptr()) == QFALSE {
        return QFALSE;
    }
    //FIXME: need to have a route to enemy and/or clear shot?
    QTRUE
}

/*
-------------------------
AI_GetGroup
-------------------------
*/
//#define MAX_WAITERS	128
/// `void AI_GetGroup( gentity_t *self )` (NPC_AI_Utils.c:443).
///
/// Build (or rejoin) `self`'s combat group: bail for non-NPCs, group-banned NPCs, stale
/// enemies, or when `d_noGroupAI` is set, otherwise allocate a fresh group, seed it from
/// `self`, sweep `g_entities` for valid members, then sort by path-cost and assign
/// closest-buddies. No oracle — mutates `level.groups`/entity state and drives the
/// validate/sort callees.
///
/// # Safety
/// `self_` may be null (checked); otherwise must point to a valid `gentity_t`. Reads the
/// `level`/`g_entities`/`d_noGroupAI` globals.
pub unsafe fn AI_GetGroup(self_: *mut gentity_t) {
    let mut member: *mut gentity_t;

    if self_.is_null() || (*self_).NPC.is_null() {
        return;
    }

    if (*addr_of!(d_noGroupAI)).integer != 0 {
        (*(*self_).NPC).group = null_mut();
        return;
    }

    if (*self_).client.is_null() {
        (*(*self_).NPC).group = null_mut();
        return;
    }

    if (*(*self_).NPC).scriptFlags & SCF_NO_GROUPS != 0 {
        (*(*self_).NPC).group = null_mut();
        return;
    }

    if !(*self_).enemy.is_null()
        && ((*(*self_).enemy).client.is_null()
            || (level.time - (*(*self_).NPC).enemyLastSeenTime > 7000))
    {
        (*(*self_).NPC).group = null_mut();
        return;
    }

    if AI_GetNextEmptyGroup(self_) == QFALSE {
        //either no more groups left or we're already in a group built earlier
        return;
    }

    //create a new one
    core::ptr::write_bytes((*(*self_).NPC).group, 0, 1);

    let group = (*(*self_).NPC).group;
    (*group).enemy = (*self_).enemy;
    (*group).team = (*(*self_).client).playerTeam;
    (*group).processed = QFALSE;
    (*group).commander = self_;
    (*group).memberValidateTime = level.time + 2000;
    (*group).activeMemberNum = 0;

    if !(*group).enemy.is_null() {
        (*group).lastSeenEnemyTime = level.time;
        (*group).lastClearShotTime = level.time;
        VectorCopy(
            &(*(*group).enemy).r.currentOrigin,
            &mut (*group).enemyLastSeenPos,
        );
    }

    //	for ( i = 0, member = &g_entities[0]; i < globals.num_entities ; i++, member++)
    for i in 0..level.num_entities {
        member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(i as usize);

        if (*member).inuse == QFALSE {
            continue;
        }

        if AI_ValidateGroupMember((*(*self_).NPC).group, member) == QFALSE {
            //FIXME: keep track of those who aren't angry yet and see if we should wake them after we assemble the core group
            continue;
        }

        //store it
        AI_InsertGroupMember((*(*self_).NPC).group, member);

        if (*(*(*self_).NPC).group).numGroup >= (MAX_GROUP_MEMBERS as c_int - 1) {
            //full
            break;
        }
    }

    /*
    //now go through waiters and see if any should join the group
    //NOTE:  Some should hang back and probably not attack, so we can ambush
    //NOTE: only do this if calling for reinforcements?
    for ( i = 0; i < numWaiters; i++ )
    {
        waiter = &g_entities[waiters[i]];

        for ( j = 0; j < self->NPC->group->numGroup; j++ )
        {
            member = &g_entities[self->NPC->group->member[j];

            if ( trap_InPVS( waiter->r.currentOrigin, member->r.currentOrigin ) )
            {//this waiter is within PVS of a current member
            }
        }
    }
    */

    if (*(*(*self_).NPC).group).numGroup <= 0 {
        //none in group
        (*(*self_).NPC).group = null_mut();
        return;
    }

    AI_SortGroupByPathCostToEnemy((*(*self_).NPC).group);
    AI_SetClosestBuddy((*(*self_).NPC).group);
}

/// `void AI_SetNewGroupCommander( AIGroupInfo_t *group )` (NPC_AI_Utils.c:553).
///
/// Re-elect the group's commander as the highest-ranked surviving member. No oracle.
///
/// # Safety
/// `group` must point to a valid `AIGroupInfo_t`; reads the `g_entities` global.
pub unsafe fn AI_SetNewGroupCommander(group: *mut AIGroupInfo_t) {
    let mut member: *mut gentity_t;

    (*group).commander = null_mut();
    for i in 0..(*group).numGroup {
        member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[i as usize].number as usize);

        if (*group).commander.is_null()
            || (!member.is_null()
                && !(*member).NPC.is_null()
                && !(*(*group).commander).NPC.is_null()
                && (*(*member).NPC).rank > (*(*(*group).commander).NPC).rank)
        {
            //keep track of highest rank
            (*group).commander = member;
        }
    }
}

/// `void AI_DeleteGroupMember( AIGroupInfo_t *group, int memberNum )` (NPC_AI_Utils.c:570).
///
/// Remove slot `memberNum` from `group`: clear its NPC back-link, shift the tail up,
/// fix up `activeMemberNum`/`numGroup`, and re-elect a commander. No oracle.
///
/// # Safety
/// `group` must point to a valid `AIGroupInfo_t`; `memberNum` must be a valid slot.
pub unsafe fn AI_DeleteGroupMember(group: *mut AIGroupInfo_t, memberNum: c_int) {
    if !(*group).commander.is_null()
        && (*(*group).commander).s.number == (*group).member[memberNum as usize].number
    {
        (*group).commander = null_mut();
    }
    if !(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[memberNum as usize].number as usize))
        .NPC
        .is_null()
    {
        (*(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[memberNum as usize].number as usize)).NPC)
            .group = null_mut();
    }
    for i in memberNum..((*group).numGroup - 1) {
        (*group).member[i as usize] = (*group).member[(i + 1) as usize];
    }
    if memberNum < (*group).activeMemberNum {
        (*group).activeMemberNum -= 1;
        if (*group).activeMemberNum < 0 {
            (*group).activeMemberNum = 0;
        }
    }
    (*group).numGroup -= 1;
    if (*group).numGroup < 0 {
        (*group).numGroup = 0;
    }
    AI_SetNewGroupCommander(group);
}

/// `void AI_DeleteSelfFromGroup( gentity_t *self )` (NPC_AI_Utils.c:602).
///
/// Find `self` in its own group and delete that membership. No oracle.
///
/// # Safety
/// `self_` must point to a valid `gentity_t` whose `NPC->group` is non-null.
pub unsafe fn AI_DeleteSelfFromGroup(self_: *mut gentity_t) {
    //FIXME: if killed, keep track of how many in group killed?  To affect morale?
    for i in 0..(*(*(*self_).NPC).group).numGroup {
        if (*(*(*self_).NPC).group).member[i as usize].number == (*self_).s.number {
            AI_DeleteGroupMember((*(*self_).NPC).group, i);
            return;
        }
    }
}

/// `void AI_GroupMemberKilled( gentity_t *self )` (NPC_AI_Utils.c:620).
///
/// React to a group member's death: temporarily drop group morale, lower teammates'
/// aggression/aim, and — if the dead NPC was the commander and no senior officer remains —
/// make the grunts flee or take cover (those near the enemy or the commander run; the rest
/// flee on a rank-weighted random chance). Officers (`rank >= RANK_ENSIGN`) never panic.
/// No oracle — group/entity-state mutation driven by `Q_irand` over the process-global
/// statics, plus the `ST_*` stormtrooper helpers.
///
/// # Safety
/// `self_` may be null (checked); otherwise must point to a valid `gentity_t` whose `NPC`
/// is non-null. Reads the `g_entities` global.
pub unsafe fn AI_GroupMemberKilled(self_: *mut gentity_t) {
    let group: *mut AIGroupInfo_t = (*(*self_).NPC).group;
    let mut member: *mut gentity_t;
    let mut noflee: qboolean = QFALSE;

    if group.is_null() {
        //what group?
        return;
    }
    if self_.is_null() || (*self_).NPC.is_null() || (*(*self_).NPC).rank < RANK_ENSIGN {
        //I'm not an officer, let's not really care for now
        return;
    }
    //temporarily drop group morale for a few seconds
    (*group).moraleAdjust -= (*(*self_).NPC).rank;
    //go through and drop aggression on my teammates (more cover, worse aim)
    for i in 0..(*group).numGroup {
        member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[i as usize].number as usize);
        if member == self_ {
            continue;
        }
        if (*(*member).NPC).rank > RANK_ENSIGN {
            //officers do not panic
            noflee = QTRUE;
        } else {
            ST_AggressionAdjust(member, -1);
            (*(*member).NPC).currentAim -= Q_irand(0, 10); //Q_irand( 0, 2);//drop their aim accuracy
        }
    }
    //okay, if I'm the group commander, make everyone else flee
    if (*group).commander != self_ {
        //I'm not the commander... hmm, should maybe a couple flee... maybe those near me?
        return;
    }
    //now see if there is another of sufficient rank to keep them from fleeing
    if noflee == QFALSE {
        (*(*(*self_).NPC).group).speechDebounceTime = 0;
        for i in 0..(*group).numGroup {
            member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[i as usize].number as usize);
            if member == self_ {
                continue;
            }
            if (*(*member).NPC).rank < RANK_ENSIGN {
                //grunt
                if !(*group).enemy.is_null()
                    && DistanceSquared(
                        &(*member).r.currentOrigin,
                        &(*(*group).enemy).r.currentOrigin,
                    ) < 65536.0
                /*256*256*/
                {
                    //those close to enemy run away!
                    ST_StartFlee(
                        member,
                        (*group).enemy,
                        &(*member).r.currentOrigin,
                        AEL_DANGER_GREAT,
                        3000,
                        5000,
                    );
                } else if DistanceSquared(
                    &(*member).r.currentOrigin,
                    &(*self_).r.currentOrigin,
                ) < 65536.0
                /*256*256*/
                {
                    //those close to me run away!
                    ST_StartFlee(
                        member,
                        (*group).enemy,
                        &(*member).r.currentOrigin,
                        AEL_DANGER_GREAT,
                        3000,
                        5000,
                    );
                } else {
                    //else, maybe just a random chance
                    if Q_irand(0, (*(*self_).NPC).rank) > (*(*member).NPC).rank {
                        //lower rank they are, higher rank I am, more likely they are to flee
                        ST_StartFlee(
                            member,
                            (*group).enemy,
                            &(*member).r.currentOrigin,
                            AEL_DANGER_GREAT,
                            3000,
                            5000,
                        );
                    } else {
                        ST_MarkToCover(member);
                    }
                }
                (*(*member).NPC).currentAim -= Q_irand(1, 15); //Q_irand( 1, 3 );//drop their aim accuracy even more
            }
            (*(*member).NPC).currentAim -= Q_irand(1, 15); //Q_irand( 1, 3 );//drop their aim accuracy even more
        }
    }
}

/// `void AI_GroupUpdateEnemyLastSeen( AIGroupInfo_t *group, vec3_t spot )`
/// (NPC_AI_Utils.c:699).
///
/// Stamp the group's last-seen-enemy time/position. No oracle.
///
/// # Safety
/// `group` may be null (checked); `spot` must point to a valid `vec3_t`.
pub unsafe fn AI_GroupUpdateEnemyLastSeen(group: *mut AIGroupInfo_t, spot: &vec3_t) {
    if group.is_null() {
        return;
    }
    (*group).lastSeenEnemyTime = level.time;
    VectorCopy(spot, &mut (*group).enemyLastSeenPos);
}

/// `void AI_GroupUpdateClearShotTime( AIGroupInfo_t *group )` (NPC_AI_Utils.c:709).
///
/// Stamp the group's last-clear-shot time. No oracle.
///
/// # Safety
/// `group` may be null (checked).
pub unsafe fn AI_GroupUpdateClearShotTime(group: *mut AIGroupInfo_t) {
    if group.is_null() {
        return;
    }
    (*group).lastClearShotTime = level.time;
}

/// `void AI_GroupUpdateSquadstates( AIGroupInfo_t *group, gentity_t *member, int newSquadState )`
/// (NPC_AI_Utils.c:718).
///
/// Move `member` to `newSquadState`, keeping the group's per-state tally in sync (or just
/// setting the NPC field directly when there is no group). No oracle.
///
/// # Safety
/// `group` may be null (checked); `member` must point to a valid `gentity_t` whose `NPC`
/// is non-null.
pub unsafe fn AI_GroupUpdateSquadstates(
    group: *mut AIGroupInfo_t,
    member: *mut gentity_t,
    newSquadState: c_int,
) {
    if group.is_null() {
        (*(*member).NPC).squadState = newSquadState;
        return;
    }

    for i in 0..(*group).numGroup {
        if (*group).member[i as usize].number == (*member).s.number {
            (*group).numState[(*(*member).NPC).squadState as usize] -= 1;
            (*(*member).NPC).squadState = newSquadState;
            (*group).numState[(*(*member).NPC).squadState as usize] += 1;
            return;
        }
    }
}

/// `qboolean AI_RefreshGroup( AIGroupInfo_t *group )` (NPC_AI_Utils.c:740).
///
/// Re-validate and recompute a group for this frame: merge with any earlier group sharing
/// our enemy when the combined members fit, clear and re-tally squad-states, drop dead/
/// invalid members, re-elect the highest-ranked commander, recompute morale (member ranks,
/// enemy health/weapon, slow moraleAdjust decay), and mark the group un-processed. Returns
/// `qtrue` while the group still has members. No oracle — group/entity-state mutation over
/// `level.groups`/`g_entities`, driven by the validate helpers + `Q_irand` + the
/// `G_TestLine` debug-draw.
///
/// # Safety
/// `group` must point to a valid `AIGroupInfo_t` inside `level.groups`; reads the
/// `g_entities`/`level`/`debugNPCAI` globals.
pub unsafe fn AI_RefreshGroup(group: *mut AIGroupInfo_t) -> qboolean {
    let mut member: *mut gentity_t;
    let mut i: c_int; //, j;

    //see if we should merge with another group
    i = 0;
    while i < MAX_FRAME_GROUPS as c_int {
        if addr_of!(level.groups[i as usize]) as *mut AIGroupInfo_t == group {
            break;
        } else if level.groups[i as usize].enemy == (*group).enemy {
            //2 groups with same enemy
            if level.groups[i as usize].numGroup + (*group).numGroup
                < (MAX_GROUP_MEMBERS as c_int - 1)
            {
                //combining the members would fit in one group
                let mut deleteWhenDone: qboolean = QTRUE;

                //combine the members of mine into theirs
                let mut j: c_int = 0;
                while j < (*group).numGroup {
                    member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[j as usize].number as usize);
                    if level.groups[i as usize].enemy.is_null() {
                        //special case for groups without enemies, must be in range
                        if AI_ValidateNoEnemyGroupMember(
                            addr_of!(level.groups[i as usize]) as *mut AIGroupInfo_t,
                            member,
                        ) == QFALSE
                        {
                            deleteWhenDone = QFALSE;
                            j += 1;
                            continue;
                        }
                    }
                    //remove this member from this group
                    AI_DeleteGroupMember(group, j);
                    //keep marker at same place since we deleted this guy and shifted everyone up one
                    j -= 1;
                    //add them to the earlier group
                    AI_InsertGroupMember(
                        addr_of!(level.groups[i as usize]) as *mut AIGroupInfo_t,
                        member,
                    );
                    j += 1;
                }
                //return and delete this group
                if deleteWhenDone != QFALSE {
                    return QFALSE;
                }
            }
        }
        i += 1;
    }
    //clear numStates
    for i in 0..NUM_SQUAD_STATES {
        (*group).numState[i as usize] = 0;
    }

    //go through group and validate each membership
    (*group).commander = null_mut();
    i = 0;
    while i < (*group).numGroup {
        /*
        //this checks for duplicate copies of one member in a group
        for ( j = 0; j < group->numGroup; j++ )
        {
            if ( i != j )
            {
                if ( group->member[i].number == group->member[j].number )
                {
                    break;
                }
            }
        }
        if ( j < group->numGroup )
        {//found a dupe!
            gi.Printf( S_COLOR_RED"ERROR: member %s(%d) a duplicate group member!!!\n", g_entities[group->member[i].number].targetname, group->member[i].number );
            AI_DeleteGroupMember( group, i );
            i--;
            continue;
        }
        */
        member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[i as usize].number as usize);

        //Must be alive
        if (*member).health <= 0 {
            AI_DeleteGroupMember(group, i);
            //keep marker at same place since we deleted this guy and shifted everyone up one
            i -= 1;
        } else if (*group).memberValidateTime < level.time
            && AI_ValidateGroupMember(group, member) == QFALSE
        {
            //remove this one from the group
            AI_DeleteGroupMember(group, i);
            //keep marker at same place since we deleted this guy and shifted everyone up one
            i -= 1;
        } else {
            //membership is valid
            //keep track of squadStates
            (*group).numState[(*(*member).NPC).squadState as usize] += 1;
            if (*group).commander.is_null()
                || (*(*member).NPC).rank > (*(*(*group).commander).NPC).rank
            {
                //keep track of highest rank
                (*group).commander = member;
            }
        }
        i += 1;
    }
    if (*group).memberValidateTime < level.time {
        (*group).memberValidateTime = level.time + Q_irand(500, 2500);
    }
    //Now add any new guys as long as we're not full
    /*
    for ( i = 0, member = &g_entities[0]; i < globals.num_entities && group->numGroup < (MAX_GROUP_MEMBERS - 1); i++, member++)
    {
        if ( !AI_ValidateGroupMember( group, member ) )
        {//FIXME: keep track of those who aren't angry yet and see if we should wake them after we assemble the core group
            continue;
        }
        if ( member->NPC->group == group )
        {//DOH, already in our group
            continue;
        }

        //store it
        AI_InsertGroupMember( group, member );
    }
    */

    //calc the morale of this group
    (*group).morale = (*group).moraleAdjust;
    for i in 0..(*group).numGroup {
        member = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add((*group).member[i as usize].number as usize);
        if (*(*member).NPC).rank < RANK_ENSIGN {
            //grunts
            (*group).morale += 1;
        } else {
            (*group).morale += (*(*member).NPC).rank;
        }
        if !(*group).commander.is_null() && (*addr_of!(debugNPCAI)).integer != 0 {
            //G_DebugLine( group->commander->r.currentOrigin, member->r.currentOrigin, FRAMETIME, 0x00ff00ff, qtrue );
            G_TestLine(
                &(*(*group).commander).r.currentOrigin,
                &(*member).r.currentOrigin,
                0x00000ff,
                FRAMETIME,
            );
        }
    }
    if !(*group).enemy.is_null() {
        //modify morale based on enemy health and weapon
        if (*(*group).enemy).health < 10 {
            (*group).morale += 10;
        } else if (*(*group).enemy).health < 25 {
            (*group).morale += 5;
        } else if (*(*group).enemy).health < 50 {
            (*group).morale += 2;
        }
        match (*(*group).enemy).s.weapon {
            WP_SABER => {
                (*group).morale -= 5;
            }
            WP_BRYAR_PISTOL => {
                (*group).morale += 3;
            }
            WP_DISRUPTOR => {
                (*group).morale += 2;
            }
            WP_REPEATER => {
                (*group).morale -= 1;
            }
            WP_FLECHETTE => {
                (*group).morale -= 2;
            }
            WP_ROCKET_LAUNCHER => {
                (*group).morale -= 10;
            }
            WP_THERMAL => {
                (*group).morale -= 5;
            }
            WP_TRIP_MINE => {
                (*group).morale -= 3;
            }
            WP_DET_PACK => {
                (*group).morale -= 10;
            }
            //		case WP_MELEE:			// Any ol' melee attack
            //			group->morale += 20;
            //			break;
            WP_STUN_BATON => {
                (*group).morale += 10;
            }
            WP_EMPLACED_GUN => {
                (*group).morale -= 8;
            }
            //		case WP_ATST_MAIN:
            //			group->morale -= 8;
            //			break;
            //		case WP_ATST_SIDE:
            //			group->morale -= 20;
            //			break;
            _ => {}
        }
    }
    if (*group).moraleDebounce < level.time {
        //slowly degrade whatever moraleAdjusters we may have
        if (*group).moraleAdjust > 0 {
            (*group).moraleAdjust -= 1;
        } else if (*group).moraleAdjust < 0 {
            (*group).moraleAdjust += 1;
        }
        (*group).moraleDebounce = level.time + 1000; //FIXME: define?
    }
    //mark this group as not having been run this frame
    (*group).processed = QFALSE;

    if (*group).numGroup > 0 {
        QTRUE
    } else {
        QFALSE
    }
}

/// `void AI_UpdateGroups( void )` (NPC_AI_Utils.c:964).
///
/// Once per frame, refresh every active group and zero any that are empty or that
/// [`AI_RefreshGroup`] reports as defunct. Gated off the `d_noGroupAI` cvar. No oracle.
///
/// # Safety
/// Reads/writes the `level.groups` global and the `d_noGroupAI` cvar.
pub unsafe fn AI_UpdateGroups() {
    if (*addr_of!(d_noGroupAI)).integer != 0 {
        return;
    }
    //Clear all Groups
    for i in 0..MAX_FRAME_GROUPS {
        if level.groups[i].numGroup == 0
            || AI_RefreshGroup(addr_of!(level.groups[i]) as *mut AIGroupInfo_t) == QFALSE
        //level.groups[i].enemy == NULL ||
        {
            core::ptr::write_bytes(addr_of!(level.groups[i]) as *mut AIGroupInfo_t, 0, 1);
        }
    }
}

/// `qboolean AI_GroupContainsEntNum( AIGroupInfo_t *group, int entNum )`
/// (NPC_AI_Utils.c:982).
///
/// Whether `group` lists entity number `entNum`. No oracle.
///
/// # Safety
/// `group` may be null (checked).
pub unsafe fn AI_GroupContainsEntNum(group: *mut AIGroupInfo_t, entNum: c_int) -> qboolean {
    if group.is_null() {
        return QFALSE;
    }
    for i in 0..(*group).numGroup {
        if (*group).member[i as usize].number == entNum {
            return QTRUE;
        }
    }
    QFALSE
}

//Overload

/*
void AI_GetGroup( AIGroupInfo_t &group, gentity_t *ent, int radius )
{
	if ( ent->client == NULL )
		return;

	vec3_t	temp, angles;

	//FIXME: This is specialized code.. move?
	if ( ent->enemy )
	{
		VectorSubtract( ent->enemy->r.currentOrigin, ent->r.currentOrigin, temp );
		VectorNormalize( temp );	//FIXME: Needed?
		vectoangles( temp, angles );
	}
	else
	{
		VectorCopy( ent->currentAngles, angles );
	}

	AI_GetGroup( group, ent->r.currentOrigin, ent->currentAngles, DEFAULT_RADIUS, radius, ent->client->playerTeam, ent, ent->enemy );
}
*/
/*
-------------------------
AI_CheckEnemyCollision
-------------------------
*/

/// `qboolean AI_CheckEnemyCollision( gentity_t *ent, qboolean takeEnemy )`
/// (NPC_AI_Utils.c:1030).
///
/// After a move, if we've blocked on an enemy-team client that isn't already our enemy,
/// optionally take it as our enemy. No oracle — drives `NAV_GetLastMove`/`G_SetEnemy`.
///
/// # Safety
/// `ent` may be null (checked); otherwise must point to a valid `gentity_t` with a client.
pub unsafe fn AI_CheckEnemyCollision(ent: *mut gentity_t, takeEnemy: qboolean) -> qboolean {
    let mut info = core::mem::zeroed();

    if ent.is_null() {
        return QFALSE;
    }

    //	if ( ent->svFlags & SVF_LOCKEDENEMY )
    //		return qfalse;

    NAV_GetLastMove(&mut info);

    //See if we've hit something
    if !info.blocker.is_null() && info.blocker != (*ent).enemy {
        if !(*info.blocker).client.is_null()
            && (*(*info.blocker).client).playerTeam == (*(*ent).client).enemyTeam
        {
            if takeEnemy != QFALSE {
                G_SetEnemy(ent, info.blocker);
            }

            return QTRUE;
        }
    }

    QFALSE
}

/*
-------------------------
AI_DistributeAttack
-------------------------
*/

/// `gentity_t *AI_DistributeAttack( gentity_t *attacker, gentity_t *enemy, team_t team, int threshold )`
/// (NPC_AI_Utils.c:1065).
///
/// Spread out attackers: if too many of `team` already surround `enemy`, pick a less-
/// pressed nearby same-team target (preferring the player when they're under-threatened).
/// Returns the chosen target, or `NULL`. No oracle — drives `AI_GetGroupSize` /
/// `trap_EntitiesInBox`.
///
/// # Safety
/// `attacker`/`enemy` must point to valid `gentity_t`s; reads the `g_entities` global.
pub unsafe fn AI_DistributeAttack(
    attacker: *mut gentity_t,
    enemy: *mut gentity_t,
    team: team_t,
    threshold: c_int,
) -> *mut gentity_t {
    let mut radiusEnts: [c_int; MAX_RADIUS_ENTS] = [0; MAX_RADIUS_ENTS];
    let numEnts: c_int;
    let numSurrounding: c_int;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    //Don't take new targets
    //	if ( NPC->svFlags & SVF_LOCKEDENEMY )
    //		return enemy;

    numSurrounding = AI_GetGroupSize(&(*enemy).r.currentOrigin, 48, team, attacker);

    //First, see if we should look for the player
    if enemy != (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0) {
        //rwwFIXMEFIXME: care about all clients not just 0
        let aroundPlayer = AI_GetGroupSize(
            &(*(core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0)).r.currentOrigin,
            48,
            team,
            attacker,
        );

        //See if we're above our threshold
        if aroundPlayer < threshold {
            return (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(0);
        }
    }

    //See if our current enemy is still ok
    if numSurrounding < threshold {
        return enemy;
    }

    //Otherwise we need to take a new enemy if possible

    //Setup the bbox to search in
    for i in 0..3 {
        mins[i] = (*enemy).r.currentOrigin[i] - 512.0;
        maxs[i] = (*enemy).r.currentOrigin[i] + 512.0;
    }

    //Get the number of entities in a given space
    numEnts = trap::EntitiesInBox(&mins, &maxs, &mut radiusEnts);

    //Cull this list
    for j in 0..numEnts {
        let check = (core::ptr::addr_of_mut!(g_entities).cast::<gentity_t>()).add(radiusEnts[j as usize] as usize);

        //Validate clients
        if (*check).client.is_null() {
            continue;
        }

        //Skip the requested avoid ent if present
        if check == enemy {
            continue;
        }

        //Must be on the same team
        if (*(*check).client).playerTeam != (*(*enemy).client).playerTeam {
            continue;
        }

        //Must be alive
        if (*check).health <= 0 {
            continue;
        }

        //Must not be overwhelmed
        if AI_GetGroupSize(&(*check).r.currentOrigin, 48, team, attacker) > threshold {
            continue;
        }

        return check;
    }

    null_mut()
}
