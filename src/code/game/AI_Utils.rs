// These utilities are meant for strictly non-player, non-team NPCs.
// These functions are in their own file because they are only intended
// for use with NPCs who's logic has been overriden from the original
// AI code, and who's code resides in files with the AI_ prefix.

// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// #include "g_headers.h"

use core::ffi::c_int;

// Stub declarations for external types and functions
// These would normally come from the included headers

extern "C" {
    pub static mut d_noGroupAI: *mut cvar_t;
    pub static mut debugNPCAI: *mut cvar_t;
    pub static mut g_entities: [gentity_t; 2048];
    pub static mut level: level_locals_t;
    pub static mut globals: game_globals_t;

    pub fn gi_EntitiesInBox(
        mins: *const vec3_t,
        maxs: *const vec3_t,
        list: *mut *mut gentity_t,
        maxcount: c_int,
    ) -> c_int;
    pub fn gi_inPVS(p1: *const vec3_t, p2: *const vec3_t) -> qboolean;
    pub fn G_DebugLine(
        start: *const vec3_t,
        end: *const vec3_t,
        duration: c_int,
        color: u32,
        bold: qboolean,
    );
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn DistanceSquared(p1: *const vec3_t, p2: *const vec3_t) -> c_int;
    pub fn TIMER_Done(ent: *const gentity_t, timer: *const u8) -> qboolean;
    pub fn PInUse(entnum: c_int) -> qboolean;
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
}

pub unsafe fn AI_ValidateGroupMember(group: *mut AIGroupInfo_t, member: *mut gentity_t) -> qboolean {
    AI_ValidateGroupMember_impl(group, member)
}

const MAX_RADIUS_ENTS: usize = 128;
const DEFAULT_RADIUS: c_int = 45;

//extern	CNavigator	navigator;

/*
-------------------------
AI_GetGroupSize
-------------------------
*/

pub unsafe fn AI_GetGroupSize(
    mut origin: vec3_t,
    mut radius: c_int,
    playerTeam: team_t,
    avoid: *mut gentity_t,
) -> c_int {
    let mut radiusEnts: [*mut gentity_t; MAX_RADIUS_ENTS] = [core::ptr::null_mut(); MAX_RADIUS_ENTS];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut numEnts: c_int;
    let mut realCount: c_int = 0;

    //Setup the bbox to search in
    for i in 0..3 {
        mins[i] = origin[i] - radius as f32;
        maxs[i] = origin[i] + radius as f32;
    }

    //Get the number of entities in a given space
    numEnts = gi_EntitiesInBox(
        &mins,
        &maxs,
        radiusEnts.as_mut_ptr(),
        MAX_RADIUS_ENTS as c_int,
    );

    //Cull this list
    for j in 0..(numEnts as usize) {
        //Validate clients
        if (*radiusEnts[j]).client.is_null() {
            continue;
        }

        //Skip the requested avoid ent if present
        if !avoid.is_null() && radiusEnts[j] == avoid {
            continue;
        }

        //Must be on the same team
        if (*(*radiusEnts[j]).client).playerTeam != playerTeam {
            continue;
        }

        //Must be alive
        if (*radiusEnts[j]).health <= 0 {
            continue;
        }

        realCount += 1;
    }

    realCount
}

//Overload

pub unsafe fn AI_GetGroupSize_ent(ent: *mut gentity_t, radius: c_int) -> c_int {
    if ent.is_null() || (*ent).client.is_null() {
        return -1;
    }

    AI_GetGroupSize(
        (*ent).currentOrigin,
        radius,
        (*(*ent).client).playerTeam,
        ent,
    )
}

pub unsafe fn AI_SetClosestBuddy(group: *mut AIGroupInfo_t) {
    let mut i: usize;
    let mut j: usize;
    let mut dist: c_int;
    let mut bestDist: c_int;

    i = 0;
    while i < (*group).numGroup as usize {
        (*group).member[i].closestBuddy = ENTITYNUM_NONE;

        bestDist = Q3_INFINITE;
        j = 0;
        while j < (*group).numGroup as usize {
            dist = DistanceSquared(
                &g_entities[(*group).member[i].number as usize].currentOrigin,
                &g_entities[(*group).member[j].number as usize].currentOrigin,
            );
            if dist < bestDist {
                bestDist = dist;
                (*group).member[i].closestBuddy = (*group).member[j].number;
            }
            j += 1;
        }
        i += 1;
    }
}

pub unsafe fn AI_SortGroupByPathCostToEnemy(group: *mut AIGroupInfo_t) {
    let mut bestMembers: [AIGroupMember_t; 16] = core::mem::zeroed(); // MAX_GROUP_MEMBERS
    let mut i: usize;
    let mut j: usize;
    let mut k: usize;
    let mut sort: qboolean = 0;

    if !(*group).enemy.is_null() {
        //FIXME: just use enemy->waypoint?
        (*group).enemyWP = NAV_GetNearestNode((*group).enemy);
    } else {
        (*group).enemyWP = WAYPOINT_NONE;
    }

    i = 0;
    while i < (*group).numGroup as usize {
        if (*group).enemyWP == WAYPOINT_NONE {
            //FIXME: just use member->waypoint?
            (*group).member[i].waypoint = WAYPOINT_NONE;
            (*group).member[i].pathCostToEnemy = Q3_INFINITE;
        } else {
            //FIXME: just use member->waypoint?
            (*group).member[i].waypoint = NAV_GetNearestNode((*group).enemy);
            if (*group).member[i].waypoint != WAYPOINT_NONE {
                (*group).member[i].pathCostToEnemy =
                    NAV_EstimateCostToGoal((*group).member[i].waypoint, (*group).enemyWP);
                //at least one of us has a path, so do sorting
                sort = 1; // qtrue
            } else {
                (*group).member[i].pathCostToEnemy = Q3_INFINITE;
            }
        }
        i += 1;
    }
    //Now sort
    if sort != 0 {
        //initialize bestMembers data
        j = 0;
        while j < (*group).numGroup as usize {
            bestMembers[j].number = ENTITYNUM_NONE;
            j += 1;
        }

        i = 0;
        while i < (*group).numGroup as usize {
            j = 0;
            while j < (*group).numGroup as usize {
                if bestMembers[j].number != ENTITYNUM_NONE {
                    //slot occupied
                    if (*group).member[i].pathCostToEnemy < bestMembers[j].pathCostToEnemy {
                        //this guy has a shorter path than the one currenly in this spot, bump him and put myself in here
                        k = (*group).numGroup as usize;
                        while k > j {
                            core::ptr::copy_nonoverlapping(
                                &bestMembers[k - 1],
                                &mut bestMembers[k],
                                1,
                            );
                            k -= 1;
                        }
                        core::ptr::copy_nonoverlapping(
                            &(*group).member[i],
                            &mut bestMembers[j],
                            1,
                        );
                        break;
                    }
                } else {
                    //slot unoccupied, reached end of list, throw self in here
                    core::ptr::copy_nonoverlapping(
                        &(*group).member[i],
                        &mut bestMembers[j],
                        1,
                    );
                    break;
                }
                j += 1;
            }
            i += 1;
        }

        //Okay, now bestMembers is a sorted list, just copy it into group->members
        i = 0;
        while i < (*group).numGroup as usize {
            core::ptr::copy_nonoverlapping(&bestMembers[i], &mut (*group).member[i], 1);
            i += 1;
        }
    }
}

pub unsafe fn AI_FindSelfInPreviousGroup(self_: *mut gentity_t) -> qboolean {
    //go through other groups made this frame and see if any of those contain me already
    let mut i: usize;
    let mut j: usize;
    i = 0;
    while i < MAX_FRAME_GROUPS as usize {
        if level.groups[i].numGroup != 0 {
            //check this one
            j = 0;
            while j < level.groups[i].numGroup as usize {
                if level.groups[i].member[j].number == (*self_).s.number {
                    (*(*self_).NPC).group = &mut level.groups[i];
                    return 1; // qtrue
                }
                j += 1;
            }
        }
        i += 1;
    }
    0 // qfalse
}

pub unsafe fn AI_InsertGroupMember(group: *mut AIGroupInfo_t, member: *mut gentity_t) {
    //okay, you know what?  Check this damn group and make sure we're not already in here!
    let mut i: usize = 0;
    while i < (*group).numGroup as usize {
        if (*group).member[i].number == (*member).s.number {
            //already in here
            break;
        }
        i += 1;
    }
    if i < (*group).numGroup as usize {
        //found him in group already
    } else {
        //add him in
        (*group).member[(*group).numGroup as usize].number = (*member).s.number;
        let squad_state = (*(*member).NPC).squadState as usize;
        (*group).numState[squad_state] += 1;
        (*group).numGroup += 1;
    }
    if (*group).commander.is_null()
        || (*member).NPC.is_null()
        || ((*(*(*group).commander).NPC).rank < (*(*member).NPC).rank)
    {
        //keep track of highest rank
        (*group).commander = member;
    }
    (*(*member).NPC).group = group;
}

pub unsafe fn AI_TryJoinPreviousGroup(self_: *mut gentity_t) -> qboolean {
    //go through other groups made this frame and see if any of those have the same enemy as me... if so, add me in!
    let mut i: usize;
    i = 0;
    while i < MAX_FRAME_GROUPS as usize {
        if level.groups[i].numGroup != 0
            && level.groups[i].numGroup < (MAX_GROUP_MEMBERS - 1) as i32
            && level.groups[i].enemy == (*self_).enemy
        {
            //has members, not full and has my enemy
            if AI_ValidateGroupMember(&mut level.groups[i], self_) != 0 {
                //I am a valid member for this group
                AI_InsertGroupMember(&mut level.groups[i], self_);
                return 1; // qtrue
            }
        }
        i += 1;
    }
    0 // qfalse
}

pub unsafe fn AI_GetNextEmptyGroup(self_: *mut gentity_t) -> qboolean {
    if AI_FindSelfInPreviousGroup(self_) != 0 {
        //already in one, no need to make a new one
        return 0; // qfalse
    }

    if AI_TryJoinPreviousGroup(self_) != 0 {
        //try to just put us in one that already exists
        return 0; // qfalse
    }

    //okay, make a whole new one, then
    let mut i: usize = 0;
    while i < MAX_FRAME_GROUPS as usize {
        if level.groups[i].numGroup == 0 {
            //make a new one
            (*(*self_).NPC).group = &mut level.groups[i];
            return 1; // qtrue
        }
        i += 1;
    }

    //if ( i >= MAX_FRAME_GROUPS )
    {
        //WTF?  Out of groups!
        (*(*self_).NPC).group = core::ptr::null_mut();
        return 0; // qfalse
    }
}

pub unsafe fn AI_ValidateNoEnemyGroupMember(
    group: *mut AIGroupInfo_t,
    member: *mut gentity_t,
) -> qboolean {
    if group.is_null() {
        return 0; // qfalse
    }
    let mut center: vec3_t = [0.0; 3];
    if !(*group).commander.is_null() {
        VectorCopy(&(*(*group).commander).currentOrigin, &mut center);
    } else {
        //hmm, just pick the first member
        if (*group).member[0].number < 0 || (*group).member[0].number >= ENTITYNUM_WORLD {
            return 0; // qfalse
        }
        VectorCopy(
            &g_entities[(*group).member[0].number as usize].currentOrigin,
            &mut center,
        );
    }
    //FIXME: maybe it should be based on the center of the mass of the group, not the commander?
    if DistanceSquared(&center, &(*member).currentOrigin) > 147456 /*384*384*/
    {
        return 0; // qfalse
    }
    if gi_inPVS(&(*member).currentOrigin, &center) == 0 {
        //not within PVS of the group enemy
        return 0; // qfalse
    }
    1 // qtrue
}

pub unsafe fn AI_ValidateGroupMember_impl(
    group: *mut AIGroupInfo_t,
    member: *mut gentity_t,
) -> qboolean {
    //Validate ents
    if member.is_null() {
        return 0; // qfalse
    }

    //Validate clients
    if (*member).client.is_null() {
        return 0; // qfalse
    }

    //Validate NPCs
    if (*member).NPC.is_null() {
        return 0; // qfalse
    }

    //must be aware
    if (*(*member).NPC).confusionTime > level.time {
        return 0; // qfalse
    }

    //must be allowed to join groups
    if ((*(*member).NPC).scriptFlags & SCF_NO_GROUPS) != 0 {
        return 0; // qfalse
    }

    //Must not be in another group
    if !(*(*member).NPC).group.is_null()
        && (*(*member).NPC).group != group
    {
        //FIXME: if that group's enemy is mine, why not absorb that group into mine?
        return 0; // qfalse
    }

    //Must be alive
    if (*member).health <= 0 {
        return 0; // qfalse
    }

    //can't be in an emplaced gun
    if ((*member).s.eFlags & EF_LOCKED_TO_WEAPON) != 0 {
        return 0; // qfalse
    }

    if ((*member).s.eFlags & EF_HELD_BY_RANCOR) != 0 {
        return 0; // qfalse
    }

    if ((*member).s.eFlags & EF_HELD_BY_SAND_CREATURE) != 0 {
        return 0; // qfalse
    }

    if ((*member).s.eFlags & EF_HELD_BY_WAMPA) != 0 {
        return 0; // qfalse
    }

    //Must be on the same team
    if (*(*member).client).playerTeam != (*group).team {
        return 0; // qfalse
    }

    if (*(*member).client).ps.weapon == WP_SABER
        || //!= self->s.weapon )
        (*(*member).client).ps.weapon == WP_THERMAL
        || (*(*member).client).ps.weapon == WP_DISRUPTOR
        || (*(*member).client).ps.weapon == WP_EMPLACED_GUN
        || (*(*member).client).ps.weapon == WP_BOT_LASER /* Probe droid	- Laser blast*/
        || (*(*member).client).ps.weapon == WP_MELEE
        || (*(*member).client).ps.weapon == WP_TURRET /* turret guns */
        || (*(*member).client).ps.weapon == WP_ATST_MAIN
        || (*(*member).client).ps.weapon == WP_ATST_SIDE
        || (*(*member).client).ps.weapon == WP_TIE_FIGHTER
    {
        //not really a squad-type guy
        return 0; // qfalse
    }

    if (*(*member).client).NPC_class == CLASS_ATST
        || (*(*member).client).NPC_class == CLASS_PROBE
        || (*(*member).client).NPC_class == CLASS_SEEKER
        || (*(*member).client).NPC_class == CLASS_REMOTE
        || (*(*member).client).NPC_class == CLASS_SENTRY
        || (*(*member).client).NPC_class == CLASS_INTERROGATOR
        || (*(*member).client).NPC_class == CLASS_MINEMONSTER
        || (*(*member).client).NPC_class == CLASS_HOWLER
        || (*(*member).client).NPC_class == CLASS_RANCOR
        || (*(*member).client).NPC_class == CLASS_MARK1
        || (*(*member).client).NPC_class == CLASS_MARK2
    {
        //these kinds of enemies don't actually use this group AI
        return 0; // qfalse
    }

    //should have same enemy
    if (*member).enemy != (*group).enemy {
        if !(*member).enemy.is_null() {
            //he's fighting someone else, leave him out
            return 0; // qfalse
        }
        if gi_inPVS(&(*member).currentOrigin, &(*(*group).enemy).currentOrigin) == 0 {
            //not within PVS of the group enemy
            return 0; // qfalse
        }
    } else if (*group).enemy.is_null() {
        //if the group is a patrol group, only take those within the room and radius
        if AI_ValidateNoEnemyGroupMember(group, member) == 0 {
            return 0; // qfalse
        }
    }
    //must be actually in combat mode
    if TIMER_Done(member, b"interrogating\0".as_ptr()) == 0 {
        return 0; // qfalse
    }
    //FIXME: need to have a route to enemy and/or clear shot?
    1 // qtrue
}

/*
-------------------------
AI_GetGroup
-------------------------
*/
//#define MAX_WAITERS	128
pub unsafe fn AI_GetGroup(self_: *mut gentity_t) {
    let mut i: usize;
    let mut member: *mut gentity_t;
    //int	waiters[MAX_WAITERS];

    if self_.is_null() || (*self_).NPC.is_null() {
        return;
    }

    if !d_noGroupAI.is_null() && (*d_noGroupAI).integer != 0 {
        (*(*self_).NPC).group = core::ptr::null_mut();
        return;
    }

    if (*self_).client.is_null() {
        (*(*self_).NPC).group = core::ptr::null_mut();
        return;
    }

    if ((*(*self_).NPC).scriptFlags & SCF_NO_GROUPS) != 0 {
        (*(*self_).NPC).group = core::ptr::null_mut();
        return;
    }

    if !(*self_).enemy.is_null()
        && ((*(*self_).enemy).client.is_null()
            || (level.time - (*(*self_).NPC).enemyLastSeenTime > 7000))
    {
        (*(*self_).NPC).group = core::ptr::null_mut();
        return;
    }

    if AI_GetNextEmptyGroup(self_) == 0 {
        //either no more groups left or we're already in a group built earlier
        return;
    }

    //create a new one
    core::ptr::write_bytes((*(*self_).NPC).group, 0, 1);

    (*(*(*self_).NPC).group).enemy = (*self_).enemy;
    (*(*(*self_).NPC).group).team = (*(*self_).client).playerTeam;
    (*(*(*self_).NPC).group).processed = 0; // qfalse
    (*(*(*self_).NPC).group).commander = self_;
    (*(*(*self_).NPC).group).memberValidateTime = level.time + 2000;
    (*(*(*self_).NPC).group).activeMemberNum = 0;

    if !(*(*(*self_).NPC).group).enemy.is_null() {
        (*(*(*self_).NPC).group).lastSeenEnemyTime = level.time;
        (*(*(*self_).NPC).group).lastClearShotTime = level.time;
        VectorCopy(
            &(*(*(*(*self_).NPC).group).enemy).currentOrigin,
            &mut (*(*(*self_).NPC).group).enemyLastSeenPos,
        );
    }

    //	for ( i = 0, member = &g_entities[0]; i < globals.num_entities ; i++, member++)
    i = 0;
    while i < globals.num_entities as usize {
        if PInUse(i as c_int) == 0 {
            i += 1;
            continue;
        }
        member = &mut g_entities[i];

        if AI_ValidateGroupMember((*(*self_).NPC).group, member) == 0 {
            //FIXME: keep track of those who aren't angry yet and see if we should wake them after we assemble the core group
            i += 1;
            continue;
        }

        //store it
        AI_InsertGroupMember((*(*self_).NPC).group, member);

        if (*(*(*self_).NPC).group).numGroup >= (MAX_GROUP_MEMBERS - 1) as i32 {
            //full
            break;
        }
        i += 1;
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

            if ( gi.inPVS( waiter->currentOrigin, member->currentOrigin ) )
            {//this waiter is within PVS of a current member
            }
        }
    }
    */

    if (*(*(*self_).NPC).group).numGroup <= 0 {
        //none in group
        (*(*self_).NPC).group = core::ptr::null_mut();
        return;
    }

    AI_SortGroupByPathCostToEnemy((*(*self_).NPC).group);
    AI_SetClosestBuddy((*(*self_).NPC).group);
}

pub unsafe fn AI_SetNewGroupCommander(group: *mut AIGroupInfo_t) {
    let mut member: *mut gentity_t = core::ptr::null_mut();
    (*group).commander = core::ptr::null_mut();
    let mut i: usize = 0;
    while i < (*group).numGroup as usize {
        member = &mut g_entities[(*group).member[i].number as usize];

        if (*group).commander.is_null()
            || (!member.is_null()
                && !(*member).NPC.is_null()
                && !(*group).commander.is_null()
                && !(*(*group).commander).NPC.is_null()
                && (*(*member).NPC).rank > (*(*(*group).commander).NPC).rank)
        {
            //keep track of highest rank
            (*group).commander = member;
        }
        i += 1;
    }
}

pub unsafe fn AI_DeleteGroupMember(group: *mut AIGroupInfo_t, memberNum: usize) {
    if !(*group).commander.is_null()
        && (*(*group).commander).s.number == (*group).member[memberNum].number
    {
        (*group).commander = core::ptr::null_mut();
    }
    if !g_entities[(*group).member[memberNum].number as usize]
        .NPC
        .is_null()
    {
        g_entities[(*group).member[memberNum].number as usize].NPC =
            core::ptr::null_mut();
    }
    let mut i: usize = memberNum;
    while i < ((*group).numGroup - 1) as usize {
        core::ptr::copy_nonoverlapping(
            &(*group).member[i + 1],
            &mut (*group).member[i],
            1,
        );
        i += 1;
    }
    if (memberNum as i32) < (*group).activeMemberNum {
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

pub unsafe fn AI_DeleteSelfFromGroup(self_: *mut gentity_t) {
    //FIXME: if killed, keep track of how many in group killed?  To affect morale?
    if (*(*self_).NPC).group.is_null() {
        return;
    }
    let mut i: usize = 0;
    while i < (*(*(*self_).NPC).group).numGroup as usize {
        if (*(*(*self_).NPC).group).member[i].number == (*self_).s.number {
            AI_DeleteGroupMember((*(*self_).NPC).group, i);
            return;
        }
        i += 1;
    }
}

extern "C" {
    pub fn ST_AggressionAdjust(self_: *mut gentity_t, change: c_int);
    pub fn ST_MarkToCover(self_: *mut gentity_t);
    pub fn ST_StartFlee(
        self_: *mut gentity_t,
        enemy: *mut gentity_t,
        dangerPoint: *const vec3_t,
        dangerLevel: c_int,
        minTime: c_int,
        maxTime: c_int,
    );
}

pub unsafe fn AI_GroupMemberKilled(self_: *mut gentity_t) {
    /*	AIGroupInfo_t *group = self->NPC->group;
        gentity_t	*member;
        qboolean	noflee = qfalse;

        if ( !group )
        {//what group?
            return;
        }
        if ( !self || !self->NPC || self->NPC->rank < RANK_ENSIGN )
        {//I'm not an officer, let's not really care for now
            return;
        }
        //temporarily drop group morale for a few seconds
        group->moraleAdjust -= self->NPC->rank;
        //go through and drop aggression on my teammates (more cover, worse aim)
        for ( int i = 0; i < group->numGroup; i++ )
        {
            member = &g_entities[group->member[i].number];
            if ( member == self )
            {
                continue;
            }
            if ( member->NPC->rank > RANK_ENSIGN )
            {//officers do not panic
                noflee = qtrue;
            }
            else
            {
                ST_AggressionAdjust( member, -1 );
                member->NPC->currentAim -= Q_irand( 0, 10 );//Q_irand( 0, 2);//drop their aim accuracy
            }
        }
        //okay, if I'm the group commander, make everyone else flee
        if ( group->commander != self )
        {//I'm not the commander... hmm, should maybe a couple flee... maybe those near me?
            return;
        }
        //now see if there is another of sufficient rank to keep them from fleeing
        if ( !noflee )
        {
            self->NPC->group->speechDebounceTime = 0;
            for ( int i = 0; i < group->numGroup; i++ )
            {
                member = &g_entities[group->member[i].number];
                if ( member == self )
                {
                    continue;
                }
                if ( member->NPC->rank < RANK_ENSIGN )
                {//grunt
                    if ( group->enemy && DistanceSquared( member->currentOrigin, group->enemy->currentOrigin ) < 65536 )//256*256
                    {//those close to enemy run away!
                        ST_StartFlee( member, group->enemy, member->currentOrigin, AEL_DANGER_GREAT, 3000, 5000 );
                    }
                    else if ( DistanceSquared( member->currentOrigin, self->currentOrigin ) < 65536 )
                    {//those close to me run away!
                        ST_StartFlee( member, group->enemy, member->currentOrigin, AEL_DANGER_GREAT, 3000, 5000 );
                    }
                    else
                    {//else, maybe just a random chance
                        if ( Q_irand( 0, self->NPC->rank ) > member->NPC->rank )
                        {//lower rank they are, higher rank I am, more likely they are to flee
                            ST_StartFlee( member, group->enemy, member->currentOrigin, AEL_DANGER_GREAT, 3000, 5000 );
                        }
                        else
                        {
                            ST_MarkToCover( member );
                        }
                    }
                    member->NPC->currentAim -= Q_irand( 1, 15 ); //Q_irand( 1, 3 );//drop their aim accuracy even more
                }
                member->NPC->currentAim -= Q_irand( 1, 15 ); //Q_irand( 1, 3 );//drop their aim accuracy even more
            }
        }*/
}

pub unsafe fn AI_GroupUpdateEnemyLastSeen(group: *mut AIGroupInfo_t, spot: *const vec3_t) {
    if group.is_null() {
        return;
    }
    (*group).lastSeenEnemyTime = level.time;
    VectorCopy(spot, &mut (*group).enemyLastSeenPos);
}

pub unsafe fn AI_GroupUpdateClearShotTime(group: *mut AIGroupInfo_t) {
    if group.is_null() {
        return;
    }
    (*group).lastClearShotTime = level.time;
}

pub unsafe fn AI_GroupUpdateSquadstates(
    group: *mut AIGroupInfo_t,
    member: *mut gentity_t,
    newSquadState: c_int,
) {
    if group.is_null() {
        (*(*member).NPC).squadState = newSquadState;
        return;
    }

    let mut i: usize = 0;
    while i < (*group).numGroup as usize {
        if (*group).member[i].number == (*member).s.number {
            let old_squad_state = (*(*member).NPC).squadState as usize;
            (*group).numState[old_squad_state] -= 1;
            (*(*member).NPC).squadState = newSquadState;
            let new_squad_state = newSquadState as usize;
            (*group).numState[new_squad_state] += 1;
            return;
        }
        i += 1;
    }
}

pub unsafe fn AI_RefreshGroup(group: *mut AIGroupInfo_t) -> qboolean {
    let mut member: *mut gentity_t;
    let mut i: usize; //, j;

    //see if we should merge with another group
    i = 0;
    while i < MAX_FRAME_GROUPS as usize {
        if &level.groups[i] as *const _ == group as *const _ {
            break;
        } else {
            if level.groups[i].enemy == (*group).enemy {
                //2 groups with same enemy
                if level.groups[i].numGroup + (*group).numGroup
                    < (MAX_GROUP_MEMBERS - 1) as i32
                {
                    //combining the members would fit in one group
                    let mut deleteWhenDone: qboolean = 1; // qtrue
                    //combine the members of mine into theirs
                    let mut j: usize = 0;
                    while j < (*group).numGroup as usize {
                        member = &mut g_entities[(*group).member[j].number as usize];
                        if level.groups[i].enemy.is_null() {
                            //special case for groups without enemies, must be in range
                            if AI_ValidateNoEnemyGroupMember(&mut level.groups[i], member) == 0 {
                                deleteWhenDone = 0; // qfalse
                            } else {
                                j += 1;
                                continue;
                            }
                        }
                        //remove this member from this group
                        AI_DeleteGroupMember(group, j);
                        //keep marker at same place since we deleted this guy and shifted everyone up one
                        j = j.saturating_sub(1);
                        //add them to the earlier group
                        AI_InsertGroupMember(&mut level.groups[i], member);
                        j += 1;
                    }
                    //return and delete this group
                    if deleteWhenDone != 0 {
                        return 0; // qfalse
                    }
                }
            }
        }
        i += 1;
    }
    //clear numStates
    i = 0;
    while i < NUM_SQUAD_STATES as usize {
        (*group).numState[i] = 0;
        i += 1;
    }

    //go through group and validate each membership
    (*group).commander = core::ptr::null_mut();
    i = 0;
    while i < (*group).numGroup as usize {
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
        member = &mut g_entities[(*group).member[i].number as usize];

        //Must be alive
        if (*member).health <= 0 {
            AI_DeleteGroupMember(group, i);
            //keep marker at same place since we deleted this guy and shifted everyone up one
            i = i.saturating_sub(1);
        } else if (*group).memberValidateTime < level.time
            && AI_ValidateGroupMember(group, member) == 0
        {
            //remove this one from the group
            AI_DeleteGroupMember(group, i);
            //keep marker at same place since we deleted this guy and shifted everyone up one
            i = i.saturating_sub(1);
        } else {
            //membership is valid
            //keep track of squadStates
            let squad_state = (*(*member).NPC).squadState as usize;
            (*group).numState[squad_state] += 1;
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
    i = 0;
    while i < (*group).numGroup as usize {
        member = &mut g_entities[(*group).member[i].number as usize];
        if (*(*member).NPC).rank < RANK_ENSIGN {
            //grunts
            (*group).morale += 1;
        } else {
            (*group).morale += (*(*member).NPC).rank;
        }
        if !(*group).commander.is_null() && !debugNPCAI.is_null() && (*debugNPCAI).integer != 0 {
            G_DebugLine(
                &(*(*group).commander).currentOrigin,
                &(*member).currentOrigin,
                16, // FRAMETIME
                0x00ff00ff,
                1, // qtrue
            );
        }
        i += 1;
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
            WP_BRYAR_PISTOL | WP_BLASTER_PISTOL => {
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
            WP_CONCUSSION => {
                (*group).morale -= 12;
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
            WP_MELEE => {
                // Any ol' melee attack
                (*group).morale += 20;
            }
            WP_STUN_BATON => {
                (*group).morale += 10;
            }
            WP_EMPLACED_GUN => {
                (*group).morale -= 8;
            }
            WP_ATST_MAIN => {
                (*group).morale -= 8;
            }
            WP_ATST_SIDE => {
                (*group).morale -= 20;
            }
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
    (*group).processed = 0; // qfalse

    if (*group).numGroup > 0 {
        1 // qtrue
    } else {
        0 // qfalse
    }
}

pub unsafe fn AI_UpdateGroups() {
    if !d_noGroupAI.is_null() && (*d_noGroupAI).integer != 0 {
        return;
    }
    //Clear all Groups
    let mut i: usize = 0;
    while i < MAX_FRAME_GROUPS as usize {
        if level.groups[i].numGroup == 0 || AI_RefreshGroup(&mut level.groups[i]) == 0 {
            //level.groups[i].enemy == NULL ||
            core::ptr::write_bytes(&mut level.groups[i], 0, 1);
        }
        i += 1;
    }
}

pub unsafe fn AI_GroupContainsEntNum(group: *mut AIGroupInfo_t, entNum: c_int) -> qboolean {
    if group.is_null() {
        return 0; // qfalse
    }
    let mut i: usize = 0;
    while i < (*group).numGroup as usize {
        if (*group).member[i].number == entNum {
            return 1; // qtrue
        }
        i += 1;
    }
    0 // qfalse
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
        VectorSubtract( ent->enemy->currentOrigin, ent->currentOrigin, temp );
        VectorNormalize( temp );	//FIXME: Needed?
        vectoangles( temp, angles );
    }
    else
    {
        VectorCopy( ent->currentAngles, angles );
    }

    AI_GetGroup( group, ent->currentOrigin, ent->currentAngles, DEFAULT_RADIUS, radius, ent->client->playerTeam, ent, ent->enemy );
}
*/

/*
-------------------------
AI_DistributeAttack
-------------------------
*/

pub unsafe fn AI_DistributeAttack(
    attacker: *mut gentity_t,
    enemy: *mut gentity_t,
    team: team_t,
    threshold: c_int,
) -> *mut gentity_t {
    //Don't take new targets
    if ((*attacker).svFlags & SVF_LOCKEDENEMY) != 0 {
        return enemy;
    }

    let numSurrounding: c_int = AI_GetGroupSize((*enemy).currentOrigin, 48, team, attacker);

    //First, see if we should look for the player
    if enemy != addr_of_mut!(g_entities[0]) {
        let aroundPlayer: c_int = AI_GetGroupSize(g_entities[0].currentOrigin, 48, team, attacker);

        //See if we're above our threshold
        if aroundPlayer < threshold {
            return addr_of_mut!(g_entities[0]);
        }
    }

    //See if our current enemy is still ok
    if numSurrounding < threshold {
        return enemy;
    }

    //Otherwise we need to take a new enemy if possible
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];

    //Setup the bbox to search in
    for i in 0..3 {
        mins[i] = (*enemy).currentOrigin[i] - 512.0;
        maxs[i] = (*enemy).currentOrigin[i] + 512.0;
    }

    //Get the number of entities in a given space
    let mut radiusEnts: [*mut gentity_t; MAX_RADIUS_ENTS] = [core::ptr::null_mut(); MAX_RADIUS_ENTS];

    let numEnts: c_int =
        gi_EntitiesInBox(&mins, &maxs, radiusEnts.as_mut_ptr(), MAX_RADIUS_ENTS as c_int);

    //Cull this list
    for j in 0..(numEnts as usize) {
        //Validate clients
        if (*radiusEnts[j]).client.is_null() {
            continue;
        }

        //Skip the requested avoid ent if present
        if radiusEnts[j] == enemy {
            continue;
        }

        //Must be on the same team
        if (*(*radiusEnts[j]).client).playerTeam != (*(*enemy).client).playerTeam {
            continue;
        }

        //Must be alive
        if (*radiusEnts[j]).health <= 0 {
            continue;
        }

        //Must not be overwhelmed
        if AI_GetGroupSize(
            (*radiusEnts[j]).currentOrigin,
            48,
            team,
            attacker,
        ) > threshold
        {
            continue;
        }

        return radiusEnts[j];
    }

    core::ptr::null_mut()
}

// Type stubs for external types used but not defined here
#[repr(C)]
pub struct cvar_t {
    pub name: *const u8,
    pub string: *const u8,
    pub latched_string: *const u8,
    pub integer: c_int,
    pub value: f32,
    pub modified: c_int,
}

pub type vec3_t = [f32; 3];
pub type qboolean = c_int;
pub type team_t = u32;

#[repr(C)]
pub struct gentity_s;
pub type gentity_t = gentity_s;

#[repr(C)]
pub struct AIGroupMember_t {
    pub number: c_int,
    pub closestBuddy: c_int,
    pub waypoint: c_int,
    pub pathCostToEnemy: c_int,
}

#[repr(C)]
pub struct AIGroupInfo_t {
    pub member: [AIGroupMember_t; 16], // MAX_GROUP_MEMBERS
    pub numGroup: i32,
    pub commander: *mut gentity_t,
    pub enemy: *mut gentity_t,
    pub enemyWP: c_int,
    pub team: team_t,
    pub processed: c_int,
    pub numState: [c_int; 8], // NUM_SQUAD_STATES
    pub lastSeenEnemyTime: c_int,
    pub lastClearShotTime: c_int,
    pub enemyLastSeenPos: vec3_t,
    pub memberValidateTime: c_int,
    pub morale: c_int,
    pub moraleAdjust: c_int,
    pub moraleDebounce: c_int,
    pub activeMemberNum: c_int,
    pub speechDebounceTime: c_int,
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
    pub groups: [AIGroupInfo_t; 8], // MAX_FRAME_GROUPS
    // ... other fields omitted for this stub
}

#[repr(C)]
pub struct game_globals_t {
    pub num_entities: c_int,
    // ... other fields omitted for this stub
}

#[repr(C)]
pub struct entityState_s {
    pub number: c_int,
    pub weapon: c_int,
    pub eFlags: u32,
}

#[repr(C)]
pub struct gclient_t {
    pub playerTeam: team_t,
    pub NPC_class: c_int,
    pub ps: playerState_t,
}

#[repr(C)]
pub struct playerState_t {
    pub weapon: c_int,
}

#[repr(C)]
pub struct gNPC_t {
    pub group: *mut AIGroupInfo_t,
    pub squadState: c_int,
    pub rank: c_int,
    pub scriptFlags: u32,
    pub confusionTime: c_int,
    pub enemyLastSeenTime: c_int,
    pub currentAim: c_int,
}

// Constants
const MAX_FRAME_GROUPS: i32 = 8;
const MAX_GROUP_MEMBERS: i32 = 16;
const NUM_SQUAD_STATES: i32 = 8;
const Q3_INFINITE: c_int = 2147483647;
const ENTITYNUM_NONE: c_int = 2047;
const ENTITYNUM_WORLD: c_int = 2048;
const WAYPOINT_NONE: c_int = -1;

// Weapon constants
const WP_SABER: c_int = 0;
const WP_BRYAR_PISTOL: c_int = 1;
const WP_BLASTER_PISTOL: c_int = 2;
const WP_DISRUPTOR: c_int = 3;
const WP_THERMAL: c_int = 4;
const WP_TRIP_MINE: c_int = 5;
const WP_DET_PACK: c_int = 6;
const WP_REPEATER: c_int = 7;
const WP_FLECHETTE: c_int = 8;
const WP_ROCKET_LAUNCHER: c_int = 9;
const WP_CONCUSSION: c_int = 10;
const WP_MELEE: c_int = 11;
const WP_STUN_BATON: c_int = 12;
const WP_EMPLACED_GUN: c_int = 13;
const WP_BOT_LASER: c_int = 14;
const WP_TURRET: c_int = 15;
const WP_ATST_MAIN: c_int = 16;
const WP_ATST_SIDE: c_int = 17;
const WP_TIE_FIGHTER: c_int = 18;

// Class constants
const CLASS_ATST: c_int = 0;
const CLASS_PROBE: c_int = 1;
const CLASS_SEEKER: c_int = 2;
const CLASS_REMOTE: c_int = 3;
const CLASS_SENTRY: c_int = 4;
const CLASS_INTERROGATOR: c_int = 5;
const CLASS_MINEMONSTER: c_int = 6;
const CLASS_HOWLER: c_int = 7;
const CLASS_RANCOR: c_int = 8;
const CLASS_MARK1: c_int = 9;
const CLASS_MARK2: c_int = 10;

// Rank constants
const RANK_ENSIGN: c_int = 1;

// Script flags
const SCF_NO_GROUPS: u32 = 0x00000200;

// Entity flags
const EF_LOCKED_TO_WEAPON: u32 = 0x00000001;
const EF_HELD_BY_RANCOR: u32 = 0x00000002;
const EF_HELD_BY_SAND_CREATURE: u32 = 0x00000004;
const EF_HELD_BY_WAMPA: u32 = 0x00000008;

// Server flags
const SVF_LOCKEDENEMY: u32 = 0x00000001;

// NAV namespace functions
pub mod NAV {
    use super::*;

    pub unsafe fn GetNearestNode(ent: *mut gentity_t) -> c_int {
        WAYPOINT_NONE
    }

    pub unsafe fn EstimateCostToGoal(from: c_int, to: c_int) -> c_int {
        Q3_INFINITE
    }
}

pub use core::ptr::{addr_of, addr_of_mut};

// Placeholder function for the declaration
pub unsafe fn NAV_GetNearestNode(ent: *mut gentity_t) -> c_int {
    WAYPOINT_NONE
}

pub unsafe fn NAV_EstimateCostToGoal(from: c_int, to: c_int) -> c_int {
    Q3_INFINITE
}
