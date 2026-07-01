//
// NPC_move.rs
//

// leave this line at the top for all NPC_xxxx.rs files...

#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    unused_variables,
    unused_imports,
    unused_mut,
    unused_assignments,
    clippy::all
)]

// #include "g_headers.h"
use crate::code::game::g_headers_h::*;
// #include "b_local.h"
use crate::code::game::b_local_h::*;
// #include "g_nav.h"
use crate::code::game::g_nav_h::*;
// #include "anims.h"
use crate::code::game::anims_h::*;

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    fn NPC_ClearPathToGoal(dir: *mut vec3_t, goal: *mut gentity_t) -> qboolean;
    fn NAV_MoveDirSafe(self_: *mut gentity_t, cmd: *mut usercmd_t, distScale: f32) -> qboolean;
}

extern "C" {
    fn CG_Cylinder(start: *mut vec3_t, end: *mut vec3_t, radius: f32, color: *mut vec3_t);
}

extern "C" {
    fn G_BoundsOverlap(
        mins1: *const vec3_t,
        maxs1: *const vec3_t,
        mins2: *const vec3_t,
        maxs2: *const vec3_t,
    ) -> qboolean;
    fn GetTime(lastTime: c_int) -> c_int;
}

pub static mut frameNavInfo: navInfo_t = unsafe { core::mem::zeroed() };

extern "C" {
    fn FlyingCreature(ent: *mut gentity_t) -> qboolean;
}

extern "C" {
    fn PM_InKnockDown(ps: *mut playerState_t) -> qboolean;
}

extern "C" {
    static mut g_navSafetyChecks: *mut cvar_t;
}

extern "C" {
    fn Boba_Flying(self_: *mut gentity_t) -> qboolean;
    fn PM_InRoll(ps: *mut playerState_t) -> qboolean;
}

// Porting note: the following are trusted-import externs for functions/globals that are
// defined elsewhere (q_shared.h / bg_public.h, brought in transitively by the includes
// above) but are not explicitly forward-declared in NPC_move.cpp itself. Each translated
// .rs file in this codebase re-declares these locally (mirroring the established
// convention), since the vector-math headers have not been ported to their own modules yet.
extern "C" {
    fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    fn VectorNormalize(v: *mut vec3_t) -> f32;
    fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
    fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    fn VectorMA(veca: *const vec3_t, scale: f32, vecb: *const vec3_t, vecc: *mut vec3_t);
    fn VectorClear(v: *mut vec3_t);
    fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32;
    fn DistanceSquared(p1: *const vec3_t, p2: *const vec3_t) -> f32;
    fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    fn Q_irand(min: c_int, max: c_int) -> c_int;

    fn EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: *mut vec3_t);
    fn PlayerStateToEntityState(ps: *mut playerState_t, s: *mut entityState_t);

    // gi.trace() / gi.linkentity() flattened per established convention
    fn gi_trace(
        trace: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passent: c_int,
        contentmask: c_int,
    );
    fn gi_linkentity(ent: *mut gentity_t);

    // math.h
    fn floor(x: f64) -> f64;
    fn fabsf(x: f32) -> f32;
    fn sqrt(x: f64) -> f64;
}

const APEX_HEIGHT: f32 = 200.0;
// #define PARA_WIDTH (sqrt(APEX_HEIGHT)+sqrt(APEX_HEIGHT))
// Porting note: macro used a function call, so it's translated as a small inline fn
// (unused in this file, preserved for fidelity).
#[inline]
unsafe fn PARA_WIDTH() -> f64 {
    sqrt(APEX_HEIGHT as f64) + sqrt(APEX_HEIGHT as f64)
}
const JUMP_SPEED: f32 = 200.0;




// Porting note: `static qboolean NPC_TryJump();` forward declaration from the original
// source is not needed in Rust (item order doesn't matter within a module); the
// no-argument overload is translated below as the plain (non-pub) `NPC_TryJump`, since
// it was declared `static` (file-local linkage) in the original.




unsafe fn NPC_Jump(dest: *const vec3_t, goalEntNum: c_int) -> qboolean {
    //FIXME: if land on enemy, knock him down & jump off again
    let mut targetDist: f32;
    let mut travelTime: f32;
    let mut impactDist: f32;
    let mut bestImpactDist: f32 = Q3_INFINITE as f32; //fireSpeed,
    let mut originalShotSpeed: f32;
    let mut shotSpeed: f32;
    let speedStep: f32 = 50.0;
    let minShotSpeed: f32 = 30.0;
    let maxShotSpeed: f32 = 500.0;
    let mut belowBlocked: qboolean = qfalse;
    let mut aboveBlocked: qboolean = qfalse;
    let mut targetDir: vec3_t = [0.0; 3];
    let mut shotVel: vec3_t = [0.0; 3];
    let mut failCase: vec3_t = [0.0; 3];
    let mut trace: trace_t = core::mem::zeroed();
    let mut tr: trajectory_t = core::mem::zeroed();
    let mut blocked: qboolean;
    let mut elapsedTime: c_int;
    let timeStep: c_int = 250;
    let mut hitCount: c_int = 0;
    let mut aboveTries: c_int = 0;
    let mut belowTries: c_int = 0;
    let maxHits: c_int = 10;
    let mut lastPos: vec3_t = [0.0; 3];
    let mut testPos: vec3_t = [0.0; 3];
    let mut bottom: vec3_t = [0.0; 3];

    VectorSubtract(dest, &(*NPC).currentOrigin, &mut targetDir);
    targetDist = VectorNormalize(&mut targetDir);
    //make our shotSpeed reliant on the distance
    originalShotSpeed = targetDist; //DistanceHorizontal( dest, NPC->currentOrigin )/2.0f;
    if originalShotSpeed > maxShotSpeed {
        originalShotSpeed = maxShotSpeed;
    } else if originalShotSpeed < minShotSpeed {
        originalShotSpeed = minShotSpeed;
    }
    shotSpeed = originalShotSpeed;

    while hitCount < maxHits {
        VectorScale(&targetDir, shotSpeed, &mut shotVel);
        travelTime = targetDist / shotSpeed;
        shotVel[2] += travelTime * 0.5 * (*(*NPC).client).ps.gravity;

        if hitCount == 0 {
            //save the first one as the worst case scenario
            VectorCopy(&shotVel, &mut failCase);
        }

        if 1 != 0 {
            //tracePath )
            //do a rough trace of the path
            blocked = qfalse;

            VectorCopy(&(*NPC).currentOrigin, &mut tr.trBase);
            VectorCopy(&shotVel, &mut tr.trDelta);
            tr.trType = TR_GRAVITY;
            tr.trTime = (*addr_of!(level)).time;
            travelTime *= 1000.0;
            VectorCopy(&(*NPC).currentOrigin, &mut lastPos);

            //This may be kind of wasteful, especially on long throws... use larger steps?  Divide the travelTime into a certain hard number of slices?  Trace just to apex and down?
            elapsedTime = timeStep;
            while (elapsedTime as f64) < floor(travelTime as f64) + (timeStep as f64) {
                if elapsedTime as f32 > travelTime {
                    //cap it
                    elapsedTime = floor(travelTime as f64) as c_int;
                }
                EvaluateTrajectory(&tr, (*addr_of!(level)).time + elapsedTime, &mut testPos);
                //FUCK IT, always check for do not enter...
                gi_trace(
                    &mut trace,
                    &lastPos,
                    &(*NPC).mins,
                    &(*NPC).maxs,
                    &testPos,
                    (*NPC).s.number,
                    (*NPC).clipmask | CONTENTS_BOTCLIP,
                );
                /*
                if ( testPos[2] < lastPos[2]
                    && elapsedTime < floor( travelTime ) )
                {//going down, haven't reached end, ignore botclip
                    gi.trace( &trace, lastPos, NPC->mins, NPC->maxs, testPos, NPC->s.number, NPC->clipmask );
                }
                else
                {//going up, check for botclip
                    gi.trace( &trace, lastPos, NPC->mins, NPC->maxs, testPos, NPC->s.number, NPC->clipmask|CONTENTS_BOTCLIP );
                }
                */

                if trace.allsolid != 0 || trace.startsolid != 0 {
                    //started in solid
                    if NAVDEBUG_showCollision {
                        CG_DrawEdge(&lastPos, &trace.endpos, EDGE_RED_TWOSECOND);
                    }
                    return qfalse; //you're hosed, dude
                }
                if trace.fraction < 1.0 {
                    //hit something
                    if NAVDEBUG_showCollision {
                        CG_DrawEdge(&lastPos, &trace.endpos, EDGE_RED_TWOSECOND); // TryJump
                    }
                    if trace.entityNum == goalEntNum {
                        //hit the enemy, that's bad!
                        blocked = qtrue;
                        /*
                        if ( g_entities[goalEntNum].client && g_entities[goalEntNum].client->ps.groundEntityNum == ENTITYNUM_NONE )
                        {//bah, would collide in mid-air, no good
                            blocked = qtrue;
                        }
                        else
                        {//he's on the ground, good enough, I guess
                            //Hmm, don't want to land on him, though...?
                        }
                        */
                        break;
                    } else {
                        if trace.contents & CONTENTS_BOTCLIP != 0 {
                            //hit a do-not-enter brush
                            blocked = qtrue;
                            break;
                        }
                        if trace.plane.normal[2] > 0.7 && DistanceSquared(&trace.endpos, dest) < 4096.0 {
                            //hit within 64 of desired location, should be okay
                            //close enough!
                            break;
                        } else {
                            //FIXME: maybe find the extents of this brush and go above or below it on next try somehow?
                            impactDist = DistanceSquared(&trace.endpos, dest);
                            if impactDist < bestImpactDist {
                                bestImpactDist = impactDist;
                                VectorCopy(&shotVel, &mut failCase);
                            }
                            blocked = qtrue;
                            break;
                        }
                    }
                } else {
                    if NAVDEBUG_showCollision {
                        CG_DrawEdge(&lastPos, &testPos, EDGE_WHITE_TWOSECOND); // TryJump
                    }
                }
                if (elapsedTime as f64) == floor(travelTime as f64) {
                    //reached end, all clear
                    if trace.fraction >= 1.0 {
                        //hmm, make sure we'll land on the ground...
                        //FIXME: do we care how far below ourselves or our dest we'll land?
                        VectorCopy(&trace.endpos, &mut bottom);
                        bottom[2] -= 128.0;
                        gi_trace(
                            &mut trace,
                            &trace.endpos,
                            &(*NPC).mins,
                            &(*NPC).maxs,
                            &bottom,
                            (*NPC).s.number,
                            (*NPC).clipmask,
                        );
                        if trace.fraction >= 1.0 {
                            //would fall too far
                            blocked = qtrue;
                        }
                    }
                    break;
                } else {
                    //all clear, try next slice
                    VectorCopy(&testPos, &mut lastPos);
                }
                elapsedTime += timeStep;
            }
            if blocked != 0 {
                //hit something, adjust speed (which will change arc)
                hitCount += 1;
                //alternate back and forth between trying an arc slightly above or below the ideal
                if (hitCount % 2) != 0 && belowBlocked == qfalse {
                    //odd
                    belowTries += 1;
                    shotSpeed = originalShotSpeed - (belowTries as f32 * speedStep);
                } else if aboveBlocked == qfalse {
                    //even
                    aboveTries += 1;
                    shotSpeed = originalShotSpeed + (aboveTries as f32 * speedStep);
                } else {
                    //can't go any higher or lower
                    hitCount = maxHits;
                    break;
                }
                if shotSpeed > maxShotSpeed {
                    shotSpeed = maxShotSpeed;
                    aboveBlocked = qtrue;
                } else if shotSpeed < minShotSpeed {
                    shotSpeed = minShotSpeed;
                    belowBlocked = qtrue;
                }
            } else {
                //made it!
                break;
            }
        } else {
            //no need to check the path, go with first calc
            break;
        }
    }

    if hitCount >= maxHits {
        //NOTE: worst case scenario, use the one that impacted closest to the target (or just use the first try...?)
        return qfalse;
        //NOTE: or try failcase?
        //VectorCopy( failCase, NPC->client->ps.velocity );
        //return qtrue;
    }
    VectorCopy(&shotVel, &mut (*(*NPC).client).ps.velocity);
    qtrue
}

const NPC_JUMP_PREP_BACKUP_DIST: f32 = 34.0;

pub static mut mJumpTrace: trace_t = unsafe { core::mem::zeroed() };

pub unsafe fn NPC_CanTryJump() -> qboolean {
    if ((*NPCInfo).scriptFlags & SCF_NAV_CAN_JUMP) == 0 // Can't Jump
        || ((*NPCInfo).scriptFlags & SCF_NO_ACROBATICS) != 0 // If Can't Jump At All
        || (*addr_of!(level)).time < (*NPCInfo).jumpBackupTime // If Backing Up, Don't Try The Jump Again
        || (*addr_of!(level)).time < (*NPCInfo).jumpNextCheckTime // Don't Even Try To Jump Again For This Amount Of Time
        || (*NPCInfo).jumpTime != 0 // Don't Jump If Already Going
        || PM_InKnockDown(&mut (*(*NPC).client).ps) != 0 // Don't Jump If In Knockdown
        || PM_InRoll(&mut (*(*NPC).client).ps) != 0 // ... Or Roll
        || (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE // ... Or In The Air
    {
        return qfalse;
    }
    qtrue
}

pub unsafe fn NPC_TryJump_2(pos: *const vec3_t, max_xy_dist: f32, max_z_diff: f32) -> qboolean {
    if NPC_CanTryJump() != 0 {
        (*NPCInfo).jumpNextCheckTime = (*addr_of!(level)).time + Q_irand(1000, 2000);

        VectorCopy(pos, &mut (*NPCInfo).jumpDest);

        // Can't Try To Jump At A Point In The Air
        //-----------------------------------------
        {
            let mut groundTest: vec3_t = [0.0; 3];
            VectorCopy(pos, &mut groundTest);
            groundTest[2] += (*NPC).mins[2] * 3.0;
            gi_trace(
                addr_of_mut!(mJumpTrace),
                &(*NPCInfo).jumpDest,
                &vec3_origin,
                &vec3_origin,
                &groundTest,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if (*addr_of!(mJumpTrace)).fraction >= 1.0 {
                return qfalse; //no ground = no jump
            }
        }
        (*NPCInfo).jumpTarget = core::ptr::null_mut();
        (*NPCInfo).jumpMaxXYDist = if max_xy_dist != 0.0 {
            max_xy_dist
        } else if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
            1200.0
        } else {
            750.0
        };
        (*NPCInfo).jumpMazZDist = if max_z_diff != 0.0 {
            max_z_diff
        } else if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
            -1000.0
        } else {
            -450.0
        };
        (*NPCInfo).jumpTime = 0;
        (*NPCInfo).jumpBackupTime = 0;
        return NPC_TryJump();
    }
    qfalse
}

pub unsafe fn NPC_TryJump_3(goal: *mut gentity_t, max_xy_dist: f32, max_z_diff: f32) -> qboolean {
    if NPC_CanTryJump() != 0 {
        (*NPCInfo).jumpNextCheckTime = (*addr_of!(level)).time + Q_irand(1000, 3000);

        // Can't Jump At Targets In The Air
        //---------------------------------
        if !(*goal).client.is_null() && (*(*goal).client).ps.groundEntityNum == ENTITYNUM_NONE {
            return qfalse;
        }
        VectorCopy(&(*goal).currentOrigin, &mut (*NPCInfo).jumpDest);
        (*NPCInfo).jumpTarget = goal;
        (*NPCInfo).jumpMaxXYDist = if max_xy_dist != 0.0 {
            max_xy_dist
        } else if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
            1200.0
        } else {
            750.0
        };
        (*NPCInfo).jumpMazZDist = if max_z_diff != 0.0 {
            max_z_diff
        } else if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
            -1000.0
        } else {
            -400.0
        };
        (*NPCInfo).jumpTime = 0;
        (*NPCInfo).jumpBackupTime = 0;
        return NPC_TryJump();
    }
    qfalse
}

pub unsafe fn NPC_JumpAnimation() {
    let mut jumpAnim: c_int = BOTH_JUMP1;

    if (*(*NPC).client).NPC_class == CLASS_BOBAFETT
        || ((*(*NPC).client).NPC_class == CLASS_REBORN && (*NPC).s.weapon != WP_SABER)
        || (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
        || ((*NPCInfo).rank != RANK_CREWMAN && (*NPCInfo).rank <= RANK_LT_JG)
    {
        //can't do acrobatics
        jumpAnim = BOTH_FORCEJUMP1;
    } else if (*(*NPC).client).NPC_class != CLASS_HOWLER {
        if (*(*NPC).client).NPC_class == CLASS_ALORA && Q_irand(0, 3) != 0 {
            jumpAnim = Q_irand(BOTH_ALORA_FLIP_1, BOTH_ALORA_FLIP_3);
        } else {
            jumpAnim = BOTH_FLIP_F;
        }
    }
    NPC_SetAnim(
        NPC,
        SETANIM_BOTH,
        jumpAnim,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        SETANIM_BLEND_DEFAULT,
    );
}

extern "C" {
    fn JET_FlyStart(actor: *mut gentity_t);
}

pub unsafe fn NPC_JumpSound() {
    if (*(*NPC).client).NPC_class == CLASS_HOWLER {
        //FIXME: can I delay the actual jump so that it matches the anim...?
    } else if (*(*NPC).client).NPC_class == CLASS_BOBAFETT || (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
        // does this really need to be here?
        JET_FlyStart(NPC);
    } else {
        G_SoundOnEnt(NPC, CHAN_BODY, "sound/weapons/force/jump.wav" as *const c_char);
    }
}

unsafe fn NPC_TryJump() -> qboolean {
    let mut targetDirection: vec3_t = [0.0; 3];
    let targetDistanceXY: f32;
    let targetDistanceZ: f32;

    // Get The Direction And Distances To The Target
    //-----------------------------------------------
    VectorSubtract(&(*NPCInfo).jumpDest, &(*NPC).currentOrigin, &mut targetDirection);
    targetDirection[2] = 0.0;
    targetDistanceXY = VectorNormalize(&mut targetDirection);
    targetDistanceZ = (*NPCInfo).jumpDest[2] - (*NPC).currentOrigin[2];

    if targetDistanceXY > (*NPCInfo).jumpMaxXYDist || targetDistanceZ < (*NPCInfo).jumpMazZDist {
        return qfalse;
    }

    // Test To See If There Is A Wall Directly In Front Of Actor, If So, Backup Some
    //-------------------------------------------------------------------------------
    if TIMER_Done(NPC, "jumpBackupDebounce" as *const c_char) != 0 {
        let mut actorProjectedTowardTarget: vec3_t = [0.0; 3];
        VectorMA(
            &(*NPC).currentOrigin,
            NPC_JUMP_PREP_BACKUP_DIST,
            &targetDirection,
            &mut actorProjectedTowardTarget,
        );
        gi_trace(
            addr_of_mut!(mJumpTrace),
            &(*NPC).currentOrigin,
            &vec3_origin,
            &vec3_origin,
            &actorProjectedTowardTarget,
            (*NPC).s.number,
            (*NPC).clipmask,
        );
        if (*addr_of!(mJumpTrace)).fraction < 1.0
            || (*addr_of!(mJumpTrace)).allsolid != 0
            || (*addr_of!(mJumpTrace)).startsolid != 0
        {
            if NAVDEBUG_showCollision {
                CG_DrawEdge(&(*NPC).currentOrigin, &actorProjectedTowardTarget, EDGE_RED_TWOSECOND); // TryJump
            }

            // TODO: We may want to test to see if it is safe to back up here?
            (*NPCInfo).jumpBackupTime = (*addr_of!(level)).time + 1000;
            TIMER_Set(NPC, "jumpBackupDebounce" as *const c_char, 5000);
            return qtrue;
        }
    }

    //	bool	Wounded					= (NPC->health < 150);
    //	bool	OnLowerLedge			= ((targetDistanceZ<-80.0f) && (targetDistanceZ>-200.0f));
    //	bool	WithinNormalJumpRange	= ((targetDistanceZ<32.0f)  && (targetDistanceXY<200.0f));
    let WithinForceJumpRange: bool = fabsf(targetDistanceZ) > 0.0 || targetDistanceXY > 128.0;

    /*	if (Wounded && OnLowerLedge)
    	{
    		ucmd.forwardmove	= 127;
    		VectorClear(NPC->client->ps.moveDir);
    		TIMER_Set(NPC, "duck", -level.time);
    		return qtrue;
    	}

    	if (WithinNormalJumpRange)
    	{
    		ucmd.upmove			= 127;
    		ucmd.forwardmove	= 127;
    		VectorClear(NPC->client->ps.moveDir);
    		TIMER_Set(NPC, "duck", -level.time);
    		return qtrue;
    	}
    */

    if !WithinForceJumpRange {
        return qfalse;
    }

    // If There Is Any Chance That This Jump Will Land On An Enemy, Try 8 Different Traces Around The Target
    //-------------------------------------------------------------------------------------------------------
    if !(*NPCInfo).jumpTarget.is_null() {
        let minSafeRadius: f32 = ((*NPC).maxs[0] * 1.5) + ((*(*NPCInfo).jumpTarget).maxs[0] * 1.5);
        let minSafeRadiusSq: f32 = minSafeRadius * minSafeRadius;

        if DistanceSquared(&(*NPCInfo).jumpDest, &(*(*NPCInfo).jumpTarget).currentOrigin) < minSafeRadiusSq {
            let mut startPos: vec3_t = [0.0; 3];
            let mut floorPos: vec3_t = [0.0; 3];
            VectorCopy(&(*NPCInfo).jumpDest, &mut startPos);

            floorPos[2] = (*NPCInfo).jumpDest[2] + ((*NPC).mins[2] - 32.0);

            for sideTryCount in 0..8 {
                (*NPCInfo).jumpSide += 1;
                if (*NPCInfo).jumpSide > 7 {
                    (*NPCInfo).jumpSide = 0;
                }

                match (*NPCInfo).jumpSide {
                    0 => {
                        (*NPCInfo).jumpDest[0] = startPos[0] + minSafeRadius;
                        (*NPCInfo).jumpDest[1] = startPos[1];
                    }
                    1 => {
                        (*NPCInfo).jumpDest[0] = startPos[0] + minSafeRadius;
                        (*NPCInfo).jumpDest[1] = startPos[1] + minSafeRadius;
                    }
                    2 => {
                        (*NPCInfo).jumpDest[0] = startPos[0];
                        (*NPCInfo).jumpDest[1] = startPos[1] + minSafeRadius;
                    }
                    3 => {
                        (*NPCInfo).jumpDest[0] = startPos[0] - minSafeRadius;
                        (*NPCInfo).jumpDest[1] = startPos[1] + minSafeRadius;
                    }
                    4 => {
                        (*NPCInfo).jumpDest[0] = startPos[0] - minSafeRadius;
                        (*NPCInfo).jumpDest[1] = startPos[1];
                    }
                    5 => {
                        (*NPCInfo).jumpDest[0] = startPos[0] - minSafeRadius;
                        (*NPCInfo).jumpDest[1] = startPos[1] - minSafeRadius;
                    }
                    6 => {
                        (*NPCInfo).jumpDest[0] = startPos[0];
                        (*NPCInfo).jumpDest[1] = startPos[1] - minSafeRadius;
                    }
                    7 => {
                        (*NPCInfo).jumpDest[0] = startPos[0] + minSafeRadius;
                        // Porting note: original C is `jumpDest[1] = startPos[1] -= minSafeRadius;`
                        // i.e. it both mutates startPos[1] in place AND assigns the resulting
                        // value into jumpDest[1] (compound-assignment-as-expression). Rust's
                        // `-=` yields `()`, so this is split into the two equivalent steps.
                        startPos[1] -= minSafeRadius;
                        (*NPCInfo).jumpDest[1] = startPos[1];
                    }
                    _ => {}
                }

                floorPos[0] = (*NPCInfo).jumpDest[0];
                floorPos[1] = (*NPCInfo).jumpDest[1];

                gi_trace(
                    addr_of_mut!(mJumpTrace),
                    &(*NPCInfo).jumpDest,
                    &(*NPC).mins,
                    &(*NPC).maxs,
                    &floorPos,
                    if !(*NPCInfo).jumpTarget.is_null() {
                        (*(*NPCInfo).jumpTarget).s.number
                    } else {
                        (*NPC).s.number
                    },
                    (*NPC).clipmask | CONTENTS_BOTCLIP,
                );
                if (*addr_of!(mJumpTrace)).fraction < 1.0
                    && (*addr_of!(mJumpTrace)).allsolid == 0
                    && (*addr_of!(mJumpTrace)).startsolid == 0
                {
                    break;
                }

                if NAVDEBUG_showCollision {
                    CG_DrawEdge(&(*NPCInfo).jumpDest, &floorPos, EDGE_RED_TWOSECOND);
                }
            }

            // If All Traces Failed, Just Try Going Right Back At The Target Location
            //------------------------------------------------------------------------
            if (*addr_of!(mJumpTrace)).fraction >= 1.0
                || (*addr_of!(mJumpTrace)).allsolid != 0
                || (*addr_of!(mJumpTrace)).startsolid != 0
            {
                VectorCopy(&startPos, &mut (*NPCInfo).jumpDest);
            }
        }
    }

    // Now, Actually Try The Jump To The Dest Target
    //-----------------------------------------------
    if NPC_Jump(
        &(*NPCInfo).jumpDest,
        if !(*NPCInfo).jumpTarget.is_null() {
            (*(*NPCInfo).jumpTarget).s.number
        } else {
            (*NPC).s.number
        },
    ) != 0
    {
        // We Made IT!
        //-------------
        NPC_JumpAnimation();
        NPC_JumpSound();

        (*(*NPC).client).ps.forceJumpZStart = (*NPC).currentOrigin[2];
        (*(*NPC).client).ps.pm_flags |= PMF_JUMPING;
        (*(*NPC).client).ps.weaponTime = (*(*NPC).client).ps.torsoAnimTimer;
        (*(*NPC).client).ps.forcePowersActive |= 1 << FP_LEVITATION;
        (*addr_of_mut!(ucmd)).forwardmove = 0;
        (*NPCInfo).jumpTime = 1;

        VectorClear(&mut (*(*NPC).client).ps.moveDir);
        TIMER_Set(NPC, "duck" as *const c_char, -(*addr_of!(level)).time);

        return qtrue;
    }
    qfalse
}

pub unsafe fn NPC_Jumping() -> qboolean {
    if (*NPCInfo).jumpTime != 0 {
        if ((*(*NPC).client).ps.pm_flags & PMF_JUMPING) == 0
            //forceJumpZStart )
            && ((*(*NPC).client).ps.pm_flags & PMF_TRIGGER_PUSHED) == 0
        {
            //landed
            (*NPCInfo).jumpTime = 0;
        } else {
            //	if (NPCInfo->jumpTarget)
            //	{
            //		NPC_FaceEntity(NPCInfo->jumpTarget, qtrue);
            //	}
            //	else
            {
                NPC_FacePosition(&mut (*NPCInfo).jumpDest, qtrue);
            }
            return qtrue;
        }
    }
    qfalse
}

pub unsafe fn NPC_JumpBackingUp() -> qboolean {
    if (*NPCInfo).jumpBackupTime != 0 {
        if (*addr_of!(level)).time < (*NPCInfo).jumpBackupTime {
            STEER::Activate(NPC);
            STEER::Flee(NPC, (*NPCInfo).jumpDest, 1.0);
            STEER::DeActivate(NPC, addr_of_mut!(ucmd));
            NPC_FacePosition(&mut (*NPCInfo).jumpDest, qtrue);
            NPC_UpdateAngles(qfalse, qtrue);
            return qtrue;
        }

        (*NPCInfo).jumpBackupTime = 0;
        return NPC_TryJump();
    }
    false as qboolean
}

/*
-------------------------
NPC_CheckCombatMove
-------------------------
*/

pub unsafe fn NPC_CheckCombatMove() -> qboolean {
    //return NPCInfo->combatMove;
    if (!(*NPCInfo).goalEntity.is_null() && !(*NPC).enemy.is_null() && (*NPCInfo).goalEntity == (*NPC).enemy)
        || (*NPCInfo).combatMove != 0
    {
        return qtrue;
    }

    if !(*NPCInfo).goalEntity.is_null() && !(*NPCInfo).watchTarget.is_null() {
        if (*NPCInfo).goalEntity != (*NPCInfo).watchTarget {
            return qtrue;
        }
    }

    qfalse
}

/*
-------------------------
NPC_LadderMove
-------------------------
*/

unsafe fn NPC_LadderMove(dir: *mut vec3_t) {
    //FIXME: this doesn't guarantee we're facing ladder
    //ALSO: Need to be able to get off at top
    //ALSO: Need to play an anim
    //ALSO: Need transitionary anims?

    if (*dir)[2] > 0.0 || ((*dir)[2] < 0.0 && (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE) {
        //Set our movement direction
        (*addr_of_mut!(ucmd)).upmove = if (*dir)[2] > 0.0 { 127 } else { -127 };

        //Don't move around on XY
        (*addr_of_mut!(ucmd)).forwardmove = 0;
        (*addr_of_mut!(ucmd)).rightmove = 0;
    }
}

/*
-------------------------
NPC_GetMoveInformation
-------------------------
*/

pub unsafe fn NPC_GetMoveInformation(dir: *mut vec3_t, distance: *mut f32) -> qboolean {
    //NOTENOTE: Use path stacks!

    //Make sure we have somewhere to go
    if (*NPCInfo).goalEntity.is_null() {
        return qfalse;
    }

    //Get our move info
    VectorSubtract(&(*(*NPCInfo).goalEntity).currentOrigin, &(*NPC).currentOrigin, dir);
    *distance = VectorNormalize(dir);

    VectorCopy(&(*(*NPCInfo).goalEntity).currentOrigin, &mut (*NPCInfo).blockedTargetPosition);

    qtrue
}

/*
-------------------------
NAV_GetLastMove
-------------------------
*/

pub unsafe fn NAV_GetLastMove(info: *mut navInfo_t) {
    core::ptr::copy_nonoverlapping(addr_of!(frameNavInfo), info, 1);
}

pub unsafe fn G_UcmdMoveForDir(self_: *mut gentity_t, cmd: *mut usercmd_t, dir: *mut vec3_t) {
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];

    AngleVectors(&(*self_).currentAngles, &mut forward, &mut right, core::ptr::null_mut());

    (*dir)[2] = 0.0;
    VectorNormalize(dir);
    //NPCs cheat and store this directly because converting movement into a ucmd loses precision
    VectorCopy(dir, &mut (*(*self_).client).ps.moveDir);

    let mut fDot: f32 = DotProduct(&forward, dir) * 127.0;
    let mut rDot: f32 = DotProduct(&right, dir) * 127.0;
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
    (*cmd).forwardmove = floor(fDot as f64) as i8;
    (*cmd).rightmove = floor(rDot as f64) as i8;

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

/*
-------------------------
NPC_MoveToGoal

  Now assumes goal is goalEntity, was no reason for it to be otherwise
-------------------------
*/

// #if	AI_TIMERS
extern "C" {
    static mut navTime: c_int;
}
// #endif//	AI_TIMERS
pub unsafe fn NPC_MoveToGoal(tryStraight: qboolean) -> qboolean {
    //FIXME: tryStraight not even used!  Stop passing it
    let mut startTime: c_int = 0;
    if AI_TIMERS != 0 {
        startTime = GetTime(0);
    }

    if PM_InKnockDown(&mut (*(*NPC).client).ps) != 0
        || ((*(*NPC).client).ps.legsAnim >= BOTH_PAIN1
            && (*(*NPC).client).ps.legsAnim <= BOTH_PAIN18
            && (*(*NPC).client).ps.legsAnimTimer > 0)
    {
        //If taking full body pain, don't move
        return qtrue;
    }

    if (*NPC).s.eFlags & EF_LOCKED_TO_WEAPON != 0 {
        //If in an emplaced gun, never try to navigate!
        return qtrue;
    }

    if (*NPC).s.eFlags & EF_HELD_BY_RANCOR != 0 {
        //If in a rancor's hand, never try to navigate!
        return qtrue;
    }
    if (*NPC).s.eFlags & EF_HELD_BY_WAMPA != 0 {
        //If in a wampa's hand, never try to navigate!
        return qtrue;
    }
    if (*NPC).s.eFlags & EF_HELD_BY_SAND_CREATURE != 0 {
        //If in a worm's mouth, never try to navigate!
        return qtrue;
    }

    if (*NPC).watertype & CONTENTS_LADDER != 0 {
        //Do we still want to do this?
        let mut dir: vec3_t = [0.0; 3];
        VectorSubtract(&(*(*NPCInfo).goalEntity).currentOrigin, &(*NPC).currentOrigin, &mut dir);
        VectorNormalize(&mut dir);
        NPC_LadderMove(&mut dir);
    }

    let mut moveSuccess: bool = true;
    STEER::Activate(NPC);
    {
        // Attempt To Steer Directly To Our Goal
        //---------------------------------------
        moveSuccess = STEER::GoTo(NPC, (*NPCInfo).goalEntity, (*NPCInfo).goalRadius, true);

        // Perhaps Not Close Enough?  Try To Use The Navigation Grid
        //-----------------------------------------------------------
        if !moveSuccess {
            moveSuccess = NAV::GoTo_2(NPC, (*NPCInfo).goalEntity, 1.0);
            if !moveSuccess {
                STEER::Stop(NPC, 1.0);
            }
        }
    }
    STEER::DeActivate(NPC, addr_of_mut!(ucmd));

    if AI_TIMERS != 0 {
        navTime += GetTime(startTime);
    }
    moveSuccess as qboolean
}

/*
-------------------------
void NPC_SlideMoveToGoal( void )

  Now assumes goal is goalEntity, if want to use tempGoal, you set that before calling the func
-------------------------
*/
pub unsafe fn NPC_SlideMoveToGoal() -> qboolean {
    let saveYaw: f32 = (*(*NPC).client).ps.viewangles[YAW as usize];

    (*NPCInfo).combatMove = qtrue;

    let ret: qboolean = NPC_MoveToGoal(qtrue);

    (*NPCInfo).desiredYaw = saveYaw;

    ret
}

/*
-------------------------
NPC_ApplyRoff
-------------------------
*/

pub unsafe fn NPC_ApplyRoff() {
    PlayerStateToEntityState(&mut (*(*NPC).client).ps, &mut (*NPC).s);
    VectorCopy(&(*NPC).currentOrigin, &mut (*NPC).lastOrigin);

    // use the precise origin for linking
    gi_linkentity(NPC);
}
