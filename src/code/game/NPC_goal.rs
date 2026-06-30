#![allow(non_snake_case)]
// b_goal.cpp
// leave this line at the top for all NPC_xxxx.cpp files...

use core::ptr;

use crate::code::game::g_headers::*; // g_headers.h
use crate::code::game::b_local_h::*; // b_local.h
use crate::code::game::Q3_Interface_h::*; // Q3_Interface.h

/*
SetGoal
*/

pub unsafe fn SetGoal(goal: *mut gentity_t, rating: f32) {
    (*NPCInfo).goalEntity = goal;
    //	(*NPCInfo).goalEntityNeed = rating;
    (*NPCInfo).goalTime = (*ptr::addr_of!(level)).time;
    if !goal.is_null() {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_SetGoal: %s @ %s (%f)\n", goal->classname, vtos( goal->currentOrigin), rating );
    } else {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_SetGoal: NONE\n" );
    }
}

/*
NPC_SetGoal
*/

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
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_SetGoal: push %s\n", (*NPCInfo).goalEntity->classname );
        (*NPCInfo).lastGoalEntity = (*NPCInfo).goalEntity;
        //		(*NPCInfo).lastGoalEntityNeed = (*NPCInfo).goalEntityNeed;
    }

    SetGoal(goal, rating);
}

/*
NPC_ClearGoal
*/

pub unsafe fn NPC_ClearGoal() {
    let mut goal: *mut gentity_t;

    if (*NPCInfo).lastGoalEntity.is_null() {
        SetGoal(ptr::null_mut(), 0.0);
        return;
    }

    goal = (*NPCInfo).lastGoalEntity;
    (*NPCInfo).lastGoalEntity = ptr::null_mut();
    if (*goal).inuse != 0 && ((*goal).s.eFlags & EF_NODRAW) == 0 {
        //		Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "NPC_ClearGoal: pop %s\n", goal->classname );
        SetGoal(goal, 0.0); //,  (*NPCInfo).lastGoalEntityNeed
        return;
    }

    SetGoal(ptr::null_mut(), 0.0);
}

/*
-------------------------
G_BoundsOverlap
-------------------------
*/

pub fn G_BoundsOverlap(
    mins1: &vec3_t,
    maxs1: &vec3_t,
    mins2: &vec3_t,
    maxs2: &vec3_t,
) -> qboolean {
    //NOTE: flush up against counts as overlapping
    if mins1[0] > maxs2[0] {
        return qfalse;
    }

    if mins1[1] > maxs2[1] {
        return qfalse;
    }

    if mins1[2] > maxs2[2] {
        return qfalse;
    }

    if maxs1[0] < mins2[0] {
        return qfalse;
    }

    if maxs1[1] < mins2[1] {
        return qfalse;
    }

    if maxs1[2] < mins2[2] {
        return qfalse;
    }

    return qtrue;
}

pub unsafe fn NPC_ReachedGoal() {
    //	Debug_NPCPrintf( NPC, debugNPCAI, DEBUG_LEVEL_INFO, "UpdateGoal: reached goal entity\n" );
    NPC_ClearGoal();
    (*NPCInfo).goalTime = (*ptr::addr_of!(level)).time;

    //MCG - Begin
    (*NPCInfo).aiFlags &= !NPCAI_MOVING;
    (*ptr::addr_of_mut!(ucmd)).forwardmove = 0;
    //Return that the goal was reached
    Q3_TaskIDComplete(*ptr::addr_of!(NPC), TID_MOVE_NAV);
    //MCG - End
}

/*
ReachedGoal

id removed checks against waypoints and is now checking surfaces
*/
pub unsafe fn ReachedGoal(goal: *mut gentity_t) -> qboolean {
    if ((*NPCInfo).aiFlags & NPCAI_TOUCHED_GOAL) != 0 {
        (*NPCInfo).aiFlags &= !NPCAI_TOUCHED_GOAL;
        return qtrue;
    }
    return STEER_Reached(
        *ptr::addr_of!(NPC),
        goal,
        (*NPCInfo).goalRadius,
        if FlyingCreature(*ptr::addr_of!(NPC)) != 0 { 1 } else { 0 },
    );
}

/*
static gentity_t *UpdateGoal( void )

Id removed a lot of shit here... doesn't seem to handle waypoints independantly of goalentity

In fact, doesn't seem to be any waypoint info on entities at all any more?

MCG - Since goal is ALWAYS goalEntity, took out a lot of sending goal entity pointers around for no reason
*/

pub unsafe fn UpdateGoal() -> *mut gentity_t {
    //FIXME: CREED should look at this
    //		this func doesn't seem to be working correctly for the sand creature

    let mut goal: *mut gentity_t;

    if (*NPCInfo).goalEntity.is_null() {
        return ptr::null_mut();
    }

    if (*(*NPCInfo).goalEntity).inuse == 0 {
        //Somehow freed it, but didn't clear it
        NPC_ClearGoal();
        return ptr::null_mut();
    }

    goal = (*NPCInfo).goalEntity;

    if ReachedGoal(goal) != 0 {
        NPC_ReachedGoal();
        goal = ptr::null_mut(); //so they don't keep trying to move to it
    } //else if fail, need to tell script so?

    return goal;
}
