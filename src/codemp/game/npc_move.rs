//! Slice of `NPC_move.c` — the NPC locomotion layer (path-to-goal, steering,
//! ladder/jump movement). Opened at the one self-contained leaf that does not
//! funnel into NAV: [`G_UcmdMoveForDir`], which converts a world-space move
//! direction into a `usercmd_t`'s `forwardmove`/`rightmove` bytes (and stashes
//! the exact `moveDir` on the player state to dodge the precision loss of the
//! byte round-trip).
//!
//! The bulk of `NPC_move.c` (`NPC_MoveToGoal`, `NPC_GetMoveInformation`,
//! `NPC_ClearPathToGoal`, `NavTest`, the `NPC_TryJump*`/`NPC_LadderMove`
//! family) leans on the NAV subsystem (`g_nav.c`, `frameNavInfo`,
//! `NAV_CheckAhead`/`NAV_Steer`), the `gNPC_t` goal/move fields, and the
//! `NPC`/`NPCInfo` AI-core globals — all still not yet ported.

#![allow(non_snake_case)] // C function names kept verbatim

use crate::codemp::game::anims::{BOTH_PAIN1, BOTH_PAIN18};
use crate::codemp::game::b_public_h::NPCAI_BLOCKED;
use crate::codemp::game::bg_misc::BG_PlayerStateToEntityState;
use crate::codemp::game::bg_panimate::PM_InKnockDown;
use crate::codemp::game::bg_public::EF2_FLYING;
use crate::codemp::game::g_local::{gentity_t, FL_NAVGOAL};
use crate::codemp::game::g_main::d_altRoutes;
use crate::codemp::game::g_nav::{
    navInfo_t, FlyingCreature, NAV_AvoidCollision, NAV_CheckAhead, NAV_HitNavGoal, NAV_MoveToGoal,
    NIF_MACRO_NAV, WAYPOINT_NONE,
};
use crate::codemp::game::g_navnew::{NAVNEW_AvoidCollision, NAVNEW_MoveToGoal};
use crate::codemp::game::npc::{ucmd, NPC, NPCInfo};
use crate::codemp::game::q_shared_h::ENTITYNUM_NONE;
use crate::codemp::game::q_math::{
    AngleNormalize360, AngleVectors, Distance, DotProduct, VectorCopy, VectorNormalize,
    VectorSubtract, vectoangles,
};
use crate::codemp::game::q_shared_h::{
    qboolean, usercmd_t, vec3_t, BUTTON_WALKING, PITCH, QFALSE, QTRUE, YAW,
};
use crate::codemp::game::surfaceflags_h::{CONTENTS_BODY, CONTENTS_BOTCLIP, CONTENTS_LADDER};
use crate::trap;
use core::ptr::{addr_of, addr_of_mut};

/// `NPC_move.c:14` — `navInfo_t frameNavInfo;`, the file-scope per-frame
/// navigation working set shared by `NPC_GetMoveDirection*`/`NAV_GetLastMove`.
/// C zero-initializes it implicitly (file-scope global); mirror that with a
/// zeroed const initializer.
#[allow(non_upper_case_globals)] // C global name kept verbatim
pub static mut frameNavInfo: navInfo_t = unsafe { core::mem::zeroed() };

/// `NPC_move.c:324` — `G_UcmdMoveForDir`. Turn a world-space direction `dir`
/// into a `usercmd_t`'s `forwardmove`/`rightmove` signed-byte fields, projecting
/// the (flattened, normalized) direction onto the entity's facing `forward`/
/// `right` vectors. NPCs also cheat and stash the exact `dir` directly on
/// `ps.moveDir`, because converting movement into a ucmd loses precision.
///
/// No-oracle: entity-state — mutates `cmd->forwardmove/rightmove` and
/// `self->client->ps.moveDir` through raw `gentity_t`/`usercmd_t` pointers.
pub unsafe fn G_UcmdMoveForDir(self_: *mut gentity_t, cmd: *mut usercmd_t, dir: *mut vec3_t) {
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut fDot: f32;
    let mut rDot: f32;

    AngleVectors(
        &(*self_).r.currentAngles,
        Some(&mut forward),
        Some(&mut right),
        None,
    );

    (*dir)[2] = 0.0;
    VectorNormalize(&mut *dir);
    //NPCs cheat and store this directly because converting movement into a ucmd loses precision
    VectorCopy(&*dir, &mut (*(*self_).client).ps.moveDir);

    fDot = DotProduct(&forward, &*dir) * 127.0;
    rDot = DotProduct(&right, &*dir) * 127.0;
    //Must clamp this because DotProduct is not guaranteed to return a number within -1 to 1, and that would be bad when we're shoving this into a signed byte
    if fDot > 127.0 {
        fDot = 127.0;
    }
    if fDot < -127.0 {
        fDot = -127.0;
    }
    if rDot > 127.0 {
        rDot = 127.0;
    }
    if rDot < -127.0 {
        rDot = -127.0;
    }
    (*cmd).forwardmove = (fDot as f64).floor() as i8;
    (*cmd).rightmove = (rDot as f64).floor() as i8;

    /*
    vec3_t	wishvel;
    for ( int i = 0 ; i < 3 ; i++ )
    {
        wishvel[i] = forward[i]*cmd->forwardmove + right[i]*cmd->rightmove;
    }
    VectorNormalize( wishvel );
    if ( !VectorCompare( wishvel, dir ) )
    {
        Com_Printf( "PRECISION LOSS: %s != %s\n", vtos(wishvel), vtos(dir) );
    }
    */
}

/// `NPC_move.c:103` — `NPC_LadderMove` (`static`). When climbing/descending a
/// ladder, drive the move into the vertical `upmove` (clamped to +/-127) and
/// zero the horizontal move so the NPC sticks to the ladder.
///
/// No-oracle: reads the `NPC` AI-core global and mutates the `ucmd` global.
unsafe fn NPC_LadderMove(dir: &vec3_t) {
    //FIXME: this doesn't guarantee we're facing ladder
    //ALSO: Need to be able to get off at top
    //ALSO: Need to play an anim
    //ALSO: Need transitionary anims?

    if (dir[2] > 0.0)
        || (dir[2] < 0.0 && (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE)
    {
        //Set our movement direction
        ucmd.upmove = if dir[2] > 0.0 { 127 } else { -127 };

        //Don't move around on XY
        ucmd.forwardmove = 0;
        ucmd.rightmove = 0;
    }
}

/// `NPC_move.c:126` — `NPC_GetMoveInformation` (`ID_INLINE`). Fill `dir` with
/// the normalized vector from the NPC to its goal entity and return the distance
/// in `*distance`; caches the goal origin as the blocked-destination. Returns
/// false when there is no goal.
///
/// No-oracle: reads the `NPC`/`NPCInfo` AI-core globals through raw pointers.
pub unsafe fn NPC_GetMoveInformation(dir: *mut vec3_t, distance: *mut f32) -> qboolean {
    //NOTENOTE: Use path stacks!

    //Make sure we have somewhere to go
    if (*NPCInfo).goalEntity.is_null() {
        return QFALSE;
    }

    //Get our move info
    VectorSubtract(
        &(*(*NPCInfo).goalEntity).r.currentOrigin,
        &(*NPC).r.currentOrigin,
        &mut *dir,
    );
    *distance = VectorNormalize(&mut *dir);

    VectorCopy(
        &(*(*NPCInfo).goalEntity).r.currentOrigin,
        &mut (*NPCInfo).blockedDest,
    );

    QTRUE
}

/// `NPC_move.c:78` — `NPC_CheckCombatMove` (`ID_INLINE`). Decide whether this
/// NPC should move while keeping its combat facing: true when its goal is its
/// enemy (or `combatMove` is forced), or when watching a target distinct from
/// its goal.
///
/// No-oracle: reads the `NPC`/`NPCInfo` AI-core globals through raw pointers.
pub unsafe fn NPC_CheckCombatMove() -> qboolean {
    //return NPCInfo->combatMove;
    if (!(*NPCInfo).goalEntity.is_null()
        && !(*NPC).enemy.is_null()
        && (*NPCInfo).goalEntity == (*NPC).enemy)
        || ((*NPCInfo).combatMove != QFALSE)
    {
        return QTRUE;
    }

    if !(*NPCInfo).goalEntity.is_null() && !(*NPCInfo).watchTarget.is_null() {
        if (*NPCInfo).goalEntity != (*NPCInfo).watchTarget {
            return QTRUE;
        }
    }

    QFALSE
}

/// `NPC_move.c:149` — `NAV_GetLastMove`. Copy the file-scope `frameNavInfo`
/// working set out to the caller's `info`.
///
/// No-oracle: reads the `frameNavInfo` AI-core global.
pub unsafe fn NAV_GetLastMove(info: *mut navInfo_t) {
    *info = *addr_of!(frameNavInfo);
}

/// `NPC_move.c:497` — `NPC_ApplyRoff`. Push the NPC's player-state into its
/// shared entity-state (so a ROFF-driven playback is networked), then re-link
/// the entity using the precise origin.
///
/// No-oracle: entity-state — copies `NPC->client->ps` into `NPC->s` through the
/// `NPC` AI-core global pointer and ends in `trap_LinkEntity`.
pub unsafe fn NPC_ApplyRoff() {
    BG_PlayerStateToEntityState(&mut (*(*NPC).client).ps, &mut (*NPC).s, QFALSE);
    //VectorCopy ( NPC->r.currentOrigin, NPC->lastOrigin );
    //rwwFIXMEFIXME: Any significance to this?

    // use the precise origin for linking
    trap::LinkEntity(NPC);
}

/// `NPC_move.c:27` — `NPC_ClearPathToGoal`. Look ahead from the `NPC` toward
/// `goal` and decide whether the straight path is clear: a clean `NAV_CheckAhead`
/// passes immediately, otherwise (for non-flyers) reject if the goal is too far
/// above/below, and finally accept if the trace got close enough — within the
/// NPC's radius, or (for navgoals) within the goal radius via `NAV_HitNavGoal`.
///
/// No-oracle: operates on `NPC`/`NPCInfo` AI-core globals and a `trace_t` filled
/// by `NAV_CheckAhead`'s `trap_Trace`.
pub unsafe fn NPC_ClearPathToGoal(_dir: &mut vec3_t, goal: *mut gentity_t) -> qboolean {
    let mut trace = crate::codemp::game::q_shared_h::trace_t::default();
    let radius: f32;
    let dist: f32;
    let tFrac: f32;

    //FIXME: What does do about area portals?  THIS IS BROKEN
    //if ( gi.inPVS( NPC->r.currentOrigin, goal->r.currentOrigin ) == qfalse )
    //	return qfalse;

    //Look ahead and see if we're clear to move to our goal position
    if NAV_CheckAhead(
        NPC,
        &(*goal).r.currentOrigin,
        &mut trace,
        ((*NPC).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
    ) != QFALSE
    {
        //VectorSubtract( goal->r.currentOrigin, NPC->r.currentOrigin, dir );
        return QTRUE;
    }

    if FlyingCreature(NPC) == QFALSE {
        //See if we're too far above
        if ((*NPC).r.currentOrigin[2] - (*goal).r.currentOrigin[2]).abs() > 48.0 {
            return QFALSE;
        }
    }

    //This is a work around
    radius = if (*NPC).r.maxs[0] > (*NPC).r.maxs[1] {
        (*NPC).r.maxs[0]
    } else {
        (*NPC).r.maxs[1]
    };
    dist = Distance(&(*NPC).r.currentOrigin, &(*goal).r.currentOrigin);
    tFrac = 1.0 - (radius / dist);

    if trace.fraction >= tFrac {
        return QTRUE;
    }

    //See if we're looking for a navgoal
    if ((*goal).flags & FL_NAVGOAL) != 0 {
        //Okay, didn't get all the way there, let's see if we got close enough:
        if NAV_HitNavGoal(
            &trace.endpos,
            &(*NPC).r.mins,
            &(*NPC).r.maxs,
            &(*goal).r.currentOrigin,
            (*NPCInfo).goalRadius,
            FlyingCreature(NPC),
        ) != QFALSE
        {
            //VectorSubtract(goal->r.currentOrigin, NPC->r.currentOrigin, dir);
            return QTRUE;
        }
    }

    QFALSE
}

/// `NPC_move.c:160` — `NPC_GetMoveDirection`. The straight-move planner: clear
/// `frameNavInfo`, fetch the move toward the goal, then resolve a direction —
/// ladder special-case, a direct clear path, falling back to macro navigation
/// (`NAV_MoveToGoal`) when stuck, and finally collision avoidance
/// (`NAV_AvoidCollision`), itself falling back to macro nav. On total failure it
/// just faces the goal (sets `desiredYaw`) and returns false.
///
/// No-oracle: drives the `NPC`/`NPCInfo` AI-core globals and the `frameNavInfo`
/// global through the NAV subsystem.
pub unsafe fn NPC_GetMoveDirection(out: *mut vec3_t, distance: *mut f32) -> qboolean {
    let mut angles: vec3_t = [0.0; 3];
    // Access the file-scope `frameNavInfo` global through a raw pointer.
    let fni = addr_of_mut!(frameNavInfo);

    //Clear the struct
    *fni = core::mem::zeroed();

    //Get our movement, if any
    if NPC_GetMoveInformation(&mut (*fni).direction, &mut (*fni).distance) == QFALSE {
        return QFALSE;
    }

    //Setup the return value
    *distance = (*fni).distance;

    //For starters
    VectorCopy(&(*fni).direction, &mut (*fni).pathDirection);

    //If on a ladder, move appropriately
    if ((*NPC).watertype & CONTENTS_LADDER) != 0 {
        NPC_LadderMove(&(*fni).direction);
        return QTRUE;
    }

    //Attempt a straight move to goal
    if NPC_ClearPathToGoal(&mut (*fni).direction, (*NPCInfo).goalEntity) == QFALSE {
        //See if we're just stuck
        if NAV_MoveToGoal(NPC, &mut *fni) == WAYPOINT_NONE {
            //Can't reach goal, just face
            vectoangles(&(*fni).direction, &mut angles);
            (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
            VectorCopy(&(*fni).direction, &mut *out);
            *distance = (*fni).distance;
            return QFALSE;
        }

        (*fni).flags |= NIF_MACRO_NAV;
    }

    //Avoid any collisions on the way
    if NAV_AvoidCollision(NPC, (*NPCInfo).goalEntity, &mut *fni) == QFALSE {
        //FIXME: Emit a warning, this is a worst case scenario
        //FIXME: if we have a clear path to our goal (exluding bodies), but then this
        //			check (against bodies only) fails, shouldn't we fall back
        //			to macro navigation?  Like so:
        if ((*fni).flags & NIF_MACRO_NAV) == 0 {
            //we had a clear path to goal and didn't try macro nav, but can't avoid collision so try macro nav here
            //See if we're just stuck
            if NAV_MoveToGoal(NPC, &mut *fni) == WAYPOINT_NONE {
                //Can't reach goal, just face
                vectoangles(&(*fni).direction, &mut angles);
                (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
                VectorCopy(&(*fni).direction, &mut *out);
                *distance = (*fni).distance;
                return QFALSE;
            }

            (*fni).flags |= NIF_MACRO_NAV;
        }
    }

    //Setup the return values
    VectorCopy(&(*fni).direction, &mut *out);
    *distance = (*fni).distance;

    QTRUE
}

/// `NPC_move.c:239` — `NPC_GetMoveDirectionAltRoute`. The alt-route move planner
/// (the NAVNEW counterpart of `NPC_GetMoveDirection`): clear `frameNavInfo`,
/// fetch the move toward the goal, then resolve a direction — ladder
/// special-case, a direct clear path, falling back to NAVNEW macro navigation
/// (`NAVNEW_MoveToGoal`) when blocked. On a clear path it tries collision
/// avoidance (`NAVNEW_AvoidCollision`), either via the `d_altRoutes` macro-nav
/// path (working on a temp copy) or, when that cvar is off, just giving up if it
/// can't avoid. On total failure it just faces the goal (sets `desiredYaw`) and
/// returns false.
///
/// No-oracle: drives the `NPC`/`NPCInfo` AI-core globals and the `frameNavInfo`
/// global through the NAVNEW subsystem.
pub unsafe fn NPC_GetMoveDirectionAltRoute(
    out: *mut vec3_t,
    distance: *mut f32,
    tryStraight: qboolean,
) -> qboolean {
    let mut angles: vec3_t = [0.0; 3];
    // Access the file-scope `frameNavInfo` global through a raw pointer.
    let fni = addr_of_mut!(frameNavInfo);

    (*NPCInfo).aiFlags &= !NPCAI_BLOCKED;

    //Clear the struct
    *fni = core::mem::zeroed();

    //Get our movement, if any
    if NPC_GetMoveInformation(&mut (*fni).direction, &mut (*fni).distance) == QFALSE {
        return QFALSE;
    }

    //Setup the return value
    *distance = (*fni).distance;

    //For starters
    VectorCopy(&(*fni).direction, &mut (*fni).pathDirection);

    //If on a ladder, move appropriately
    if ((*NPC).watertype & CONTENTS_LADDER) != 0 {
        NPC_LadderMove(&(*fni).direction);
        return QTRUE;
    }

    //Attempt a straight move to goal
    if tryStraight == QFALSE
        || NPC_ClearPathToGoal(&mut (*fni).direction, (*NPCInfo).goalEntity) == QFALSE
    {
        //blocked
        //Can't get straight to goal, use macro nav
        if NAVNEW_MoveToGoal(NPC, &mut *fni) == WAYPOINT_NONE {
            //Can't reach goal, just face
            vectoangles(&(*fni).direction, &mut angles);
            (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
            VectorCopy(&(*fni).direction, &mut *out);
            *distance = (*fni).distance;
            return QFALSE;
        }
        //else we are on our way
        (*fni).flags |= NIF_MACRO_NAV;
    } else {
        //we have no architectural problems, see if there are ents inthe way and try to go around them
        //not blocked
        if (*addr_of!(d_altRoutes)).integer != 0 {
            //try macro nav
            let mut tempInfo: navInfo_t = *fni;
            if NAVNEW_AvoidCollision(NPC, (*NPCInfo).goalEntity, &mut tempInfo, QTRUE, 5) == QFALSE
            {
                //revert to macro nav
                //Can't get straight to goal, dump tempInfo and use macro nav
                if NAVNEW_MoveToGoal(NPC, &mut *fni) == WAYPOINT_NONE {
                    //Can't reach goal, just face
                    vectoangles(&(*fni).direction, &mut angles);
                    (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
                    VectorCopy(&(*fni).direction, &mut *out);
                    *distance = (*fni).distance;
                    return QFALSE;
                }
                //else we are on our way
                (*fni).flags |= NIF_MACRO_NAV;
            } else {
                //otherwise, either clear or can avoid
                *fni = tempInfo;
            }
        } else {
            //OR: just give up
            if NAVNEW_AvoidCollision(NPC, (*NPCInfo).goalEntity, &mut *fni, QTRUE, 30) == QFALSE {
                //give up
                return QFALSE;
            }
        }
    }

    //Setup the return values
    VectorCopy(&(*fni).direction, &mut *out);
    *distance = (*fni).distance;

    QTRUE
}

/// `NPC_move.c:382` — `NPC_MoveToGoal`. Top-level navigation tick: if taking
/// full-body pain (knockdown or a `BOTH_PAIN*` legs anim) don't move; otherwise
/// resolve a move direction via [`NPC_GetMoveDirectionAltRoute`], cache the
/// distance, and convert the move into facing. In a combat move it keeps facing
/// and drives the move straight at the goal via [`G_UcmdMoveForDir`]; otherwise
/// it faces the goal (`desiredYaw`/`desiredPitch`, plus pitch + vertical velocity
/// when flying) and walks forward at full `forwardmove`.
///
/// No-oracle: drives the `NPC`/`NPCInfo` AI-core globals and the `ucmd` global.
pub unsafe fn NPC_MoveToGoal(tryStraight: qboolean) -> qboolean {
    let mut distance: f32 = 0.0;
    let mut dir: vec3_t = [0.0; 3];

    //If taking full body pain, don't move
    if PM_InKnockDown(&mut (*(*NPC).client).ps) != QFALSE
        || (((*NPC).s.legsAnim >= BOTH_PAIN1) && ((*NPC).s.legsAnim <= BOTH_PAIN18))
    {
        return QTRUE;
    }

    /*
    if( NPC->s.eFlags & EF_LOCKED_TO_WEAPON )
    {//If in an emplaced gun, never try to navigate!
        return qtrue;
    }
    */
    //rwwFIXMEFIXME: emplaced support

    //FIXME: if can't get to goal & goal is a target (enemy), try to find a waypoint that has line of sight to target, at least?
    //Get our movement direction
    if NPC_GetMoveDirectionAltRoute(&mut dir, &mut distance, tryStraight) == QFALSE {
        return QFALSE;
    }

    (*NPCInfo).distToGoal = distance;

    //Convert the move to angles
    vectoangles(&dir, &mut (*NPCInfo).lastPathAngles);
    if (ucmd.buttons & BUTTON_WALKING) != 0 {
        (*(*NPC).client).ps.speed = (*NPCInfo).stats.walkSpeed as f32;
    } else {
        (*(*NPC).client).ps.speed = (*NPCInfo).stats.runSpeed as f32;
    }

    //FIXME: still getting ping-ponging in certain cases... !!!  Nav/avoidance error?  WTF???!!!
    //If in combat move, then move directly towards our goal
    if NPC_CheckCombatMove() != QFALSE {
        //keep current facing
        G_UcmdMoveForDir(NPC, addr_of_mut!(ucmd), &mut dir);
    } else {
        //face our goal
        //FIXME: strafe instead of turn if change in dir is small and temporary
        (*NPCInfo).desiredPitch = 0.0;
        (*NPCInfo).desiredYaw = AngleNormalize360((*NPCInfo).lastPathAngles[YAW]);

        //Pitch towards the goal and also update if flying or swimming
        if ((*(*NPC).client).ps.eFlags2 & EF2_FLYING) != 0
        //moveType == MT_FLYSWIM )
        {
            (*NPCInfo).desiredPitch = AngleNormalize360((*NPCInfo).lastPathAngles[PITCH]);

            if dir[2] != 0.0 {
                let mut scale = dir[2] * distance;
                if scale > 64.0 {
                    scale = 64.0;
                } else if scale < -64.0 {
                    scale = -64.0;
                }
                (*(*NPC).client).ps.velocity[2] = scale;
                //NPC->client->ps.velocity[2] = (dir[2] > 0) ? 64 : -64;
            }
        }

        //Set any final info
        ucmd.forwardmove = 127;
    }

    QTRUE
}

/// `NPC_move.c:476` — `NPC_SlideMoveToGoal`. A combat-move flavour of
/// [`NPC_MoveToGoal`]: force `combatMove`, run the move (now assumes goal is
/// goalEntity), then restore the saved view yaw into `desiredYaw`.
///
/// No-oracle: reads/mutates the `NPC`/`NPCInfo` AI-core globals.
pub unsafe fn NPC_SlideMoveToGoal() -> qboolean {
    let saveYaw = (*(*NPC).client).ps.viewangles[YAW];
    let ret: qboolean;

    (*NPCInfo).combatMove = QTRUE;

    ret = NPC_MoveToGoal(QTRUE);

    (*NPCInfo).desiredYaw = saveYaw;

    ret
}
