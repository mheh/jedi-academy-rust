//
// NPC_move.rs
//

// leave this line at the top for all NPC_xxxx.rs files...

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// Type aliases matching C definitions
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// Forward declarations for opaque types from the engine
pub struct gclient_t;

#[repr(C)]
pub struct playerState_t {
    pub velocity: vec3_t,
    pub gravity: f32,
    pub pm_flags: c_int,
    pub groundEntityNum: c_int,
    pub forceJumpZStart: f32,
    pub weaponTime: c_int,
    pub torsoAnimTimer: c_int,
    pub legsAnim: c_int,
    pub legsAnimTimer: c_int,
    pub moveDir: vec3_t,
    pub viewangles: vec3_t,
    pub forcePowersActive: c_int,
    _padding: [u8; 512], // Placeholder for additional fields
}

#[repr(C)]
pub struct npcInfo_t {
    pub scriptFlags: c_int,
    pub jumpBackupTime: c_int,
    pub jumpNextCheckTime: c_int,
    pub jumpTime: c_int,
    pub jumpDest: vec3_t,
    pub jumpTarget: *mut gentity_t,
    pub jumpMaxXYDist: f32,
    pub jumpMazZDist: f32,
    pub jumpSide: c_int,
    pub goalEntity: *mut gentity_t,
    pub combatMove: c_int,
    pub watchTarget: *mut gentity_t,
    pub blockedTargetPosition: vec3_t,
    pub goalRadius: f32,
    pub desiredYaw: f32,
    pub rank: c_int,
    _padding: [u8; 256], // Placeholder for additional fields
}

#[repr(C)]
pub struct usercmd_t {
    pub forwardmove: c_int,
    pub rightmove: c_int,
    pub upmove: c_int,
}

#[repr(C)]
pub struct cvar_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eFlags: c_int,
    pub weapon: c_int,
    _padding: [u8; 512], // Placeholder for additional fields
}

#[repr(C)]
pub struct gentity_t {
    pub currentOrigin: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub s: entityState_t,
    pub client: *mut gclient_t,
    pub clipmask: c_int,
    pub currentAngles: vec3_t,
    pub watertype: c_int,
    pub health: c_int,
    pub lastOrigin: vec3_t,
    pub enemy: *mut gentity_t,
    _padding: [u8; 512], // Placeholder for additional fields
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
    pub NPC_class: c_int,
    _padding: [u8; 256], // Placeholder for additional fields
}

#[repr(C)]
pub struct level_t {
    pub time: c_int,
    // Additional fields from the engine are not fully defined here
}

#[repr(C)]
pub struct navInfo_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct plane_t {
    pub normal: vec3_t,
    pub dist: f32,
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub endpos: vec3_t,
    pub plane: plane_t,
    pub entityNum: c_int,
    pub contents: c_int,
}

#[repr(C)]
pub struct trajectory_t {
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
    pub trType: c_int,
    pub trTime: c_int,
}

// Extern functions and variables
extern "C" {
    pub fn NPC_ClearPathToGoal(dir: *mut vec3_t, goal: *mut gentity_t) -> qboolean;
    pub fn NAV_MoveDirSafe(self_: *mut gentity_t, cmd: *mut usercmd_t, distScale: f32) -> qboolean;
    pub fn CG_Cylinder(start: *mut vec3_t, end: *mut vec3_t, radius: f32, color: *mut vec3_t);
    pub fn GetTime(lastTime: c_int) -> c_int;
    pub fn FlyingCreature(ent: *mut gentity_t) -> qboolean;
    pub fn PM_InKnockDown(ps: *mut playerState_t) -> qboolean;
    pub fn Boba_Flying(self_: *mut gentity_t) -> qboolean;
    pub fn PM_InRoll(ps: *mut playerState_t) -> qboolean;
    pub fn JET_FlyStart(actor: *mut gentity_t);
    pub fn CG_DrawEdge(start: *mut vec3_t, end: *mut vec3_t, color: c_int);
    pub fn TIMER_Done(self_: *mut gentity_t, label: *const c_char) -> qboolean;
    pub fn TIMER_Set(self_: *mut gentity_t, label: *const c_char, duration: c_int);
    pub fn VectorMA(v: *const vec3_t, scale: f32, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorCopy(src: *const vec3_t, dest: *mut vec3_t);
    pub fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn VectorClear(v: *mut vec3_t);
    pub fn VectorCompare(a: *const vec3_t, b: *const vec3_t) -> qboolean;
    pub fn EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: *mut vec3_t);
    pub fn DistanceSquared(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn DistanceHorizontal(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    pub fn NPC_SetAnim(ent: *mut gentity_t, setAnimType: c_int, anim: c_int, flags: c_int);
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, sound: *const c_char);
    pub fn NPC_FaceEntity(ent: *mut gentity_t, doPitch: qboolean);
    pub fn NPC_FacePosition(position: *const vec3_t, doPitch: qboolean);
    pub fn NPC_UpdateAngles(usePitch: qboolean, useYaw: qboolean);
    pub fn PlayerStateToEntityState(ps: *const playerState_t, s: *mut entityState_t);
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn floor(x: f64) -> f64;
    pub fn fabsf(x: f32) -> f32;
    pub fn sqrt(x: f64) -> f64;
    pub fn G_BoundsOverlap(mins1: *const vec3_t, maxs1: *const vec3_t, mins2: *const vec3_t, maxs2: *const vec3_t) -> qboolean;

    // STEER and NAV namespace functions (C++ mangled names might differ; adjust as needed)
    pub fn STEER_Activate(self_: *mut gentity_t);
    pub fn STEER_Flee(self_: *mut gentity_t, dest: *const vec3_t);
    pub fn STEER_DeActivate(self_: *mut gentity_t, cmd: *mut usercmd_t);
    pub fn STEER_GoTo(self_: *mut gentity_t, goal: *mut gentity_t, goalRadius: f32) -> qboolean;
    pub fn STEER_Stop(self_: *mut gentity_t);
    pub fn NAV_GoTo(self_: *mut gentity_t, goal: *mut gentity_t) -> qboolean;

    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut npcInfo_t;
    pub static mut level: level_t;
    pub static mut ucmd: usercmd_t;
    pub static g_navSafetyChecks: *mut cvar_t;
    pub static vec3_origin: vec3_t;
}

// Globals defined in this module
pub static mut frameNavInfo: navInfo_t = unsafe { core::mem::zeroed() };
pub static mut mJumpTrace: trace_t = unsafe { core::mem::zeroed() };

// Macros translated to constants
const APEX_HEIGHT: f32 = 200.0f32;
const PARA_WIDTH: f32 = 28.284271f32; // sqrt(200) + sqrt(200) ≈ 28.284
const JUMP_SPEED: f32 = 200.0f32;
const NPC_JUMP_PREP_BACKUP_DIST: f32 = 34.0f32;

// Constants (likely defined in other headers, declared here as stubs)
const Q3_INFINITE: f32 = 16777216.0f32;
const NAVDEBUG_showCollision: bool = false;

// Animation constants (stubs)
const BOTH_JUMP1: c_int = 0;
const BOTH_FORCEJUMP1: c_int = 1;
const BOTH_FLIP_F: c_int = 2;
const BOTH_ALORA_FLIP_1: c_int = 3;
const BOTH_ALORA_FLIP_3: c_int = 5;
const BOTH_PAIN1: c_int = 100;
const BOTH_PAIN18: c_int = 117;

// Class constants (stubs)
const CLASS_BOBAFETT: c_int = 1;
const CLASS_REBORN: c_int = 2;
const CLASS_ROCKETTROOPER: c_int = 3;
const CLASS_HOWLER: c_int = 4;
const CLASS_ALORA: c_int = 5;

// Rank constants (stubs)
const RANK_CREWMAN: c_int = 0;
const RANK_LT_JG: c_int = 1;

// Weapon constants (stubs)
const WP_SABER: c_int = 1;

// Entity number constants (stubs)
const ENTITYNUM_NONE: c_int = 2047;

// Force power constants (stubs)
const FP_LEVITATION: c_int = 2;

// Player movement flag constants (stubs)
const PMF_JUMPING: c_int = 1;
const PMF_TRIGGER_PUSHED: c_int = 2;

// Entity flag constants (stubs)
const EF_LOCKED_TO_WEAPON: c_int = 0x00000100;
const EF_HELD_BY_RANCOR: c_int = 0x00000200;
const EF_HELD_BY_WAMPA: c_int = 0x00000400;
const EF_HELD_BY_SAND_CREATURE: c_int = 0x00000800;

// Contents constants (stubs)
const CONTENTS_BOTCLIP: c_int = 0x00040000;
const CONTENTS_LADDER: c_int = 0x00020000;

// Edge drawing constants (stubs)
const EDGE_RED_TWOSECOND: c_int = 1;
const EDGE_WHITE_TWOSECOND: c_int = 2;

// Sound channel constants (stubs)
const CHAN_BODY: c_int = 1;

// Angle index constants (stubs)
const YAW: usize = 1;

// Trajectory type constants (stubs)
const TR_GRAVITY: c_int = 1;

// Animation set constants (stubs)
const SETANIM_BOTH: c_int = 0;
const SETANIM_FLAG_OVERRIDE: c_int = 1;
const SETANIM_FLAG_HOLD: c_int = 2;

// Forward declaration
fn NPC_TryJump() -> qboolean;

// NPC_Jump - attempts to make NPC jump to destination
//FIXME: if land on enemy, knock him down & jump off again
unsafe fn NPC_Jump(dest: *const vec3_t, goalEntNum: c_int) -> qboolean {
    let mut targetDist: f32;
    let mut travelTime: f32;
    let mut impactDist: f32;
    let mut bestImpactDist: f32 = Q3_INFINITE;
    let mut originalShotSpeed: f32;
    let mut shotSpeed: f32;
    let speedStep: f32 = 50.0f32;
    let minShotSpeed: f32 = 30.0f32;
    let maxShotSpeed: f32 = 500.0f32;
    let mut belowBlocked: qboolean = 0;
    let mut aboveBlocked: qboolean = 0;
    let mut targetDir: vec3_t = [0.0f32; 3];
    let mut shotVel: vec3_t = [0.0f32; 3];
    let mut failCase: vec3_t = [0.0f32; 3];
    let mut trace: trace_t;
    let mut tr: trajectory_t = core::mem::zeroed();
    let mut blocked: qboolean;
    let mut elapsedTime: c_int;
    let timeStep: c_int = 250;
    let mut hitCount: c_int = 0;
    let mut aboveTries: c_int = 0;
    let mut belowTries: c_int = 0;
    let maxHits: c_int = 10;
    let mut lastPos: vec3_t = [0.0f32; 3];
    let mut testPos: vec3_t = [0.0f32; 3];
    let mut bottom: vec3_t = [0.0f32; 3];

    VectorSubtract(dest, (*NPC).currentOrigin, &mut targetDir);
    targetDist = VectorNormalize(&mut targetDir);
    //make our shotSpeed reliant on the distance
    originalShotSpeed = targetDist;
    if originalShotSpeed > maxShotSpeed {
        originalShotSpeed = maxShotSpeed;
    } else if originalShotSpeed < minShotSpeed {
        originalShotSpeed = minShotSpeed;
    }
    shotSpeed = originalShotSpeed;

    while hitCount < maxHits {
        VectorScale(&targetDir, shotSpeed, &mut shotVel);
        travelTime = targetDist / shotSpeed;
        shotVel[2] += travelTime * 0.5f32 * (*(*NPC).client).ps.gravity;

        if hitCount == 0 {
            //save the first one as the worst case scenario
            VectorCopy(&shotVel, &mut failCase);
        }

        if true {
            //do a rough trace of the path
            blocked = 0;

            VectorCopy((*NPC).currentOrigin, &mut tr.trBase);
            VectorCopy(&shotVel, &mut tr.trDelta);
            tr.trType = TR_GRAVITY;
            tr.trTime = level.time;
            travelTime *= 1000.0f32;
            VectorCopy((*NPC).currentOrigin, &mut lastPos);

            //This may be kind of wasteful, especially on long throws... use larger steps?  Divide the travelTime into a certain hard number of slices?  Trace just to apex and down?
            elapsedTime = timeStep;
            while elapsedTime < (floor(travelTime as f64) as c_int) + timeStep {
                if (elapsedTime as f32) > travelTime {
                    //cap it
                    elapsedTime = floor(travelTime as f64) as c_int;
                }
                EvaluateTrajectory(&tr, level.time + elapsedTime, &mut testPos);
                //FUCK IT, always check for do not enter...
                gi_trace(
                    &mut trace,
                    lastPos,
                    (*NPC).mins,
                    (*NPC).maxs,
                    testPos,
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
                        CG_DrawEdge(&mut lastPos, &mut trace.endpos, EDGE_RED_TWOSECOND);
                    }
                    return 0; //you're hosed, dude
                }
                if trace.fraction < 1.0f32 {
                    //hit something
                    if NAVDEBUG_showCollision {
                        CG_DrawEdge(&mut lastPos, &mut trace.endpos, EDGE_RED_TWOSECOND); // TryJump
                    }
                    if trace.entityNum == goalEntNum {
                        //hit the enemy, that's bad!
                        blocked = 1;
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
                        if (trace.contents & CONTENTS_BOTCLIP) != 0 {
                            //hit a do-not-enter brush
                            blocked = 1;
                            break;
                        }
                        if trace.plane.normal[2] > 0.7f32
                            && DistanceSquared(&trace.endpos, dest) < 4096.0f32
                        {
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
                            blocked = 1;
                            break;
                        }
                    }
                } else {
                    if NAVDEBUG_showCollision {
                        CG_DrawEdge(&mut lastPos, &mut testPos, EDGE_WHITE_TWOSECOND); // TryJump
                    }
                }
                if elapsedTime == floor(travelTime as f64) as c_int {
                    //reached end, all clear
                    if trace.fraction >= 1.0f32 {
                        //hmm, make sure we'll land on the ground...
                        //FIXME: do we care how far below ourselves or our dest we'll land?
                        VectorCopy(&trace.endpos, &mut bottom);
                        bottom[2] -= 128.0f32;
                        gi_trace(
                            &mut trace,
                            trace.endpos,
                            (*NPC).mins,
                            (*NPC).maxs,
                            bottom,
                            (*NPC).s.number,
                            (*NPC).clipmask,
                        );
                        if trace.fraction >= 1.0f32 {
                            //would fall too far
                            blocked = 1;
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
                if (hitCount % 2) != 0 && belowBlocked == 0 {
                    //odd
                    belowTries += 1;
                    shotSpeed = originalShotSpeed - (belowTries as f32) * speedStep;
                } else if aboveBlocked == 0 {
                    //even
                    aboveTries += 1;
                    shotSpeed = originalShotSpeed + (aboveTries as f32) * speedStep;
                } else {
                    //can't go any higher or lower
                    hitCount = maxHits;
                    break;
                }
                if shotSpeed > maxShotSpeed {
                    shotSpeed = maxShotSpeed;
                    aboveBlocked = 1;
                } else if shotSpeed < minShotSpeed {
                    shotSpeed = minShotSpeed;
                    belowBlocked = 1;
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
        return 0;
        //NOTE: or try failcase?
        //VectorCopy( failCase, NPC->client->ps.velocity );
        //return qtrue;
    }
    VectorCopy(&shotVel, &mut (*(*NPC).client).ps.velocity);
    return 1;
}

pub fn NPC_CanTryJump() -> qboolean {
    unsafe {
        if ((*NPCInfo).scriptFlags & 0x00000001) == 0
            || ((*NPCInfo).scriptFlags & 0x00000040) != 0
            || level.time < (*NPCInfo).jumpBackupTime
            || level.time < (*NPCInfo).jumpNextCheckTime
            || (*NPCInfo).jumpTime != 0
            || PM_InKnockDown(&mut (*(*NPC).client).ps) != 0
            || PM_InRoll(&mut (*(*NPC).client).ps) != 0
            || (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE
        {
            return 0;
        }
        return 1;
    }
}

// NPC_TryJump with position parameter
pub unsafe fn NPC_TryJump_pos(pos: *const vec3_t, max_xy_dist: f32, max_z_diff: f32) -> qboolean {
    if NPC_CanTryJump() != 0 {
        (*NPCInfo).jumpNextCheckTime = level.time + Q_irand(1000, 2000);

        VectorCopy(pos, &mut (*NPCInfo).jumpDest);

        // Can't Try To Jump At A Point In The Air
        //-----------------------------------------
        {
            let mut groundTest: vec3_t = [0.0f32; 3];
            VectorCopy(pos, &mut groundTest);
            groundTest[2] += (*NPC).mins[2] * 3.0f32;
            gi_trace(
                &mut mJumpTrace,
                (*NPCInfo).jumpDest,
                vec3_origin,
                vec3_origin,
                groundTest,
                (*NPC).s.number,
                (*NPC).clipmask,
            );
            if mJumpTrace.fraction >= 1.0f32 {
                return 0; //no ground = no jump
            }
        }
        (*NPCInfo).jumpTarget = core::ptr::null_mut();
        (*NPCInfo).jumpMaxXYDist = if max_xy_dist != 0.0f32 {
            max_xy_dist
        } else {
            if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
                1200.0f32
            } else {
                750.0f32
            }
        };
        (*NPCInfo).jumpMazZDist = if max_z_diff != 0.0f32 {
            max_z_diff
        } else {
            if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
                -1000.0f32
            } else {
                -450.0f32
            }
        };
        (*NPCInfo).jumpTime = 0;
        (*NPCInfo).jumpBackupTime = 0;
        return NPC_TryJump();
    }
    return 0;
}

// NPC_TryJump with entity parameter
pub unsafe fn NPC_TryJump_entity(goal: *mut gentity_t, max_xy_dist: f32, max_z_diff: f32) -> qboolean {
    if NPC_CanTryJump() != 0 {
        (*NPCInfo).jumpNextCheckTime = level.time + Q_irand(1000, 3000);

        // Can't Jump At Targets In The Air
        //---------------------------------
        if !goal.is_null() && !(*goal).client.is_null() && (*(*goal).client).ps.groundEntityNum == ENTITYNUM_NONE {
            return 0;
        }
        VectorCopy((*goal).currentOrigin, &mut (*NPCInfo).jumpDest);
        (*NPCInfo).jumpTarget = goal;
        (*NPCInfo).jumpMaxXYDist = if max_xy_dist != 0.0f32 {
            max_xy_dist
        } else {
            if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
                1200.0f32
            } else {
                750.0f32
            }
        };
        (*NPCInfo).jumpMazZDist = if max_z_diff != 0.0f32 {
            max_z_diff
        } else {
            if (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER {
                -1000.0f32
            } else {
                -400.0f32
            }
        };
        (*NPCInfo).jumpTime = 0;
        (*NPCInfo).jumpBackupTime = 0;
        return NPC_TryJump();
    }
    return 0;
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
    );
}

pub unsafe fn NPC_JumpSound() {
    if (*(*NPC).client).NPC_class == CLASS_HOWLER {
        //FIXME: can I delay the actual jump so that it matches the anim...?
    } else if (*(*NPC).client).NPC_class == CLASS_BOBAFETT
        || (*(*NPC).client).NPC_class == CLASS_ROCKETTROOPER
    {
        // does this really need to be here?
        JET_FlyStart(NPC);
    } else {
        G_SoundOnEnt(NPC, CHAN_BODY, c"sound/weapons/force/jump.wav".as_ptr());
    }
}

// NPC_TryJump with no parameters - main implementation
pub unsafe fn NPC_TryJump() -> qboolean {
    let mut targetDirection: vec3_t = [0.0f32; 3];
    let mut targetDistanceXY: f32;
    let mut targetDistanceZ: f32;

    // Get The Direction And Distances To The Target
    //-----------------------------------------------
    VectorSubtract(
        (*NPCInfo).jumpDest,
        (*NPC).currentOrigin,
        &mut targetDirection,
    );
    targetDirection[2] = 0.0f32;
    targetDistanceXY = VectorNormalize(&mut targetDirection);
    targetDistanceZ = (*NPCInfo).jumpDest[2] - (*NPC).currentOrigin[2];

    if targetDistanceXY > (*NPCInfo).jumpMaxXYDist || targetDistanceZ < (*NPCInfo).jumpMazZDist {
        return 0;
    }

    // Test To See If There Is A Wall Directly In Front Of Actor, If So, Backup Some
    //-------------------------------------------------------------------------------
    if TIMER_Done(NPC, c"jumpBackupDebounce".as_ptr()) != 0 {
        let mut actorProjectedTowardTarget: vec3_t = [0.0f32; 3];
        VectorMA(
            (*NPC).currentOrigin,
            NPC_JUMP_PREP_BACKUP_DIST,
            &targetDirection,
            &mut actorProjectedTowardTarget,
        );
        gi_trace(
            &mut mJumpTrace,
            (*NPC).currentOrigin,
            vec3_origin,
            vec3_origin,
            actorProjectedTowardTarget,
            (*NPC).s.number,
            (*NPC).clipmask,
        );
        if mJumpTrace.fraction < 1.0f32 || mJumpTrace.allsolid != 0 || mJumpTrace.startsolid != 0 {
            if NAVDEBUG_showCollision {
                CG_DrawEdge(
                    &mut (*NPC).currentOrigin,
                    &mut actorProjectedTowardTarget,
                    EDGE_RED_TWOSECOND,
                ); // TryJump
            }

            // TODO: We may want to test to see if it is safe to back up here?
            (*NPCInfo).jumpBackupTime = level.time + 1000;
            TIMER_Set(NPC, c"jumpBackupDebounce".as_ptr(), 5000);
            return 1;
        }
    }

    //	bool	Wounded					= (NPC->health < 150);
    //	bool	OnLowerLedge			= ((targetDistanceZ<-80.0f) && (targetDistanceZ>-200.0f));
    //	bool	WithinNormalJumpRange	= ((targetDistanceZ<32.0f)  && (targetDistanceXY<200.0f));
    let WithinForceJumpRange: bool = (fabsf(targetDistanceZ) > 0.0f32) || (targetDistanceXY > 128.0f32);

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
        return 0;
    }

    // If There Is Any Chance That This Jump Will Land On An Enemy, Try 8 Different Traces Around The Target
    //-------------------------------------------------------------------------------------------------------
    if !(*NPCInfo).jumpTarget.is_null() {
        let minSafeRadius: f32 = ((*NPC).maxs[0] * 1.5f32) + ((*(*NPCInfo).jumpTarget).maxs[0] * 1.5f32);
        let minSafeRadiusSq: f32 = minSafeRadius * minSafeRadius;

        if DistanceSquared(
            (*NPCInfo).jumpDest,
            (*(*NPCInfo).jumpTarget).currentOrigin,
        ) < minSafeRadiusSq
        {
            let mut startPos: vec3_t = [0.0f32; 3];
            let mut floorPos: vec3_t = [0.0f32; 3];
            VectorCopy((*NPCInfo).jumpDest, &mut startPos);

            floorPos[2] = (*NPCInfo).jumpDest[2] + ((*NPC).mins[2] - 32.0f32);

            let mut sideTryCount: c_int = 0;
            while sideTryCount < 8 {
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
                        startPos[1] -= minSafeRadius;
                        (*NPCInfo).jumpDest[1] = startPos[1];
                    }
                    _ => {}
                }

                floorPos[0] = (*NPCInfo).jumpDest[0];
                floorPos[1] = (*NPCInfo).jumpDest[1];

                gi_trace(
                    &mut mJumpTrace,
                    (*NPCInfo).jumpDest,
                    (*NPC).mins,
                    (*NPC).maxs,
                    floorPos,
                    if !(*NPCInfo).jumpTarget.is_null() {
                        (*(*NPCInfo).jumpTarget).s.number
                    } else {
                        (*NPC).s.number
                    },
                    (*NPC).clipmask | CONTENTS_BOTCLIP,
                );
                if mJumpTrace.fraction < 1.0f32
                    && mJumpTrace.allsolid == 0
                    && mJumpTrace.startsolid == 0
                {
                    break;
                }

                if NAVDEBUG_showCollision {
                    CG_DrawEdge(&mut (*NPCInfo).jumpDest, &mut floorPos, EDGE_RED_TWOSECOND);
                }

                sideTryCount += 1;
            }

            // If All Traces Failed, Just Try Going Right Back At The Target Location
            //------------------------------------------------------------------------
            if mJumpTrace.fraction >= 1.0f32 || mJumpTrace.allsolid != 0 || mJumpTrace.startsolid != 0 {
                VectorCopy(&startPos, &mut (*NPCInfo).jumpDest);
            }
        }
    }

    // Now, Actually Try The Jump To The Dest Target
    //-----------------------------------------------
    if NPC_Jump(
        (*NPCInfo).jumpDest,
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
        ucmd.forwardmove = 0;
        (*NPCInfo).jumpTime = 1;

        VectorClear(&mut (*(*NPC).client).ps.moveDir);
        TIMER_Set(NPC, c"duck".as_ptr(), -level.time);

        return 1;
    }
    return 0;
}

pub unsafe fn NPC_Jumping() -> qboolean {
    if (*NPCInfo).jumpTime != 0 {
        if ((*(*NPC).client).ps.pm_flags & PMF_JUMPING) == 0
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
                NPC_FacePosition((*NPCInfo).jumpDest, 1);
            }
            return 1;
        }
    }
    return 0;
}

pub unsafe fn NPC_JumpBackingUp() -> qboolean {
    if (*NPCInfo).jumpBackupTime != 0 {
        if level.time < (*NPCInfo).jumpBackupTime {
            STEER_Activate(NPC);
            STEER_Flee(NPC, (*NPCInfo).jumpDest);
            STEER_DeActivate(NPC, &mut ucmd);
            NPC_FacePosition((*NPCInfo).jumpDest, 1);
            NPC_UpdateAngles(0, 1);
            return 1;
        }

        (*NPCInfo).jumpBackupTime = 0;
        return NPC_TryJump();
    }
    return 0;
}

/*
-------------------------
NPC_CheckCombatMove
-------------------------
*/

#[inline]
pub unsafe fn NPC_CheckCombatMove() -> qboolean {
    //return NPCInfo->combatMove;
    if ((!(*NPCInfo).goalEntity.is_null()
        && !(*NPC).enemy.is_null()
        && (*NPCInfo).goalEntity == (*NPC).enemy)
        || (*NPCInfo).combatMove != 0)
    {
        return 1;
    }

    if !(*NPCInfo).goalEntity.is_null() && !(*NPCInfo).watchTarget.is_null() {
        if (*NPCInfo).goalEntity != (*NPCInfo).watchTarget {
            return 1;
        }
    }

    return 0;
}

/*
-------------------------
NPC_LadderMove
-------------------------
*/

unsafe fn NPC_LadderMove(dir: *const vec3_t) {
    //FIXME: this doesn't guarantee we're facing ladder
    //ALSO: Need to be able to get off at top
    //ALSO: Need to play an anim
    //ALSO: Need transitionary anims?

    if (*dir)[2] > 0.0f32
        || ((*dir)[2] < 0.0f32 && (*(*NPC).client).ps.groundEntityNum == ENTITYNUM_NONE)
    {
        //Set our movement direction
        ucmd.upmove = if (*dir)[2] > 0.0f32 { 127 } else { -127 };

        //Don't move around on XY
        ucmd.forwardmove = 0;
        ucmd.rightmove = 0;
    }
}

/*
-------------------------
NPC_GetMoveInformation
-------------------------
*/

#[inline]
unsafe fn NPC_GetMoveInformation(dir: *mut vec3_t, distance: *mut f32) -> qboolean {
    //NOTENOTE: Use path stacks!

    //Make sure we have somewhere to go
    if (*NPCInfo).goalEntity.is_null() {
        return 0;
    }

    //Get our move info
    VectorSubtract(
        (*(*NPCInfo).goalEntity).currentOrigin,
        (*NPC).currentOrigin,
        dir,
    );
    *distance = VectorNormalize(dir);

    VectorCopy(
        (*(*NPCInfo).goalEntity).currentOrigin,
        &mut (*NPCInfo).blockedTargetPosition,
    );

    return 1;
}

/*
-------------------------
NAV_GetLastMove
-------------------------
*/

pub unsafe fn NAV_GetLastMove(info: *mut navInfo_t) {
    *info = frameNavInfo;
}

pub unsafe fn G_UcmdMoveForDir(self_: *mut gentity_t, cmd: *mut usercmd_t, dir: *mut vec3_t) {
    let mut forward: vec3_t = [0.0f32; 3];
    let mut right: vec3_t = [0.0f32; 3];

    AngleVectors((*self_).currentAngles, &mut forward, &mut right, core::ptr::null_mut());

    (*dir)[2] = 0.0f32;
    VectorNormalize(dir);
    //NPCs cheat and store this directly because converting movement into a ucmd loses precision
    VectorCopy(dir, &mut (*(*self_).client).ps.moveDir);

    let mut fDot: f32 = DotProduct(&forward, dir) * 127.0f32;
    let mut rDot: f32 = DotProduct(&right, dir) * 127.0f32;
    //Must clamp this because DotProduct is not guaranteed to return a number within -1 to 1, and that would be bad when we're shoving this into a signed byte
    if fDot > 127.0f32 {
        fDot = 127.0f32;
    }
    if fDot < -127.0f32 {
        fDot = -127.0f32;
    }
    if rDot > 127.0f32 {
        rDot = 127.0f32;
    }
    if rDot < -127.0f32 {
        rDot = -127.0f32;
    }
    (*cmd).forwardmove = floor(fDot as f64) as c_int;
    (*cmd).rightmove = floor(rDot as f64) as c_int;

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

#[cfg(feature = "AI_TIMERS")]
extern "C" {
    pub static mut navTime: c_int;
}

pub unsafe fn NPC_MoveToGoal(tryStraight: qboolean) -> qboolean {
    //FIXME: tryStraight not even used!  Stop passing it
    #[cfg(feature = "AI_TIMERS")]
    let startTime: c_int = GetTime(0);

    if PM_InKnockDown(&mut (*(*NPC).client).ps) != 0
        || (((*(*NPC).client).ps.legsAnim >= BOTH_PAIN1)
            && ((*(*NPC).client).ps.legsAnim <= BOTH_PAIN18)
            && (*(*NPC).client).ps.legsAnimTimer > 0)
    {
        //If taking full body pain, don't move
        return 1;
    }

    if ((*NPC).s.eFlags & EF_LOCKED_TO_WEAPON) != 0 {
        //If in an emplaced gun, never try to navigate!
        return 1;
    }

    if ((*NPC).s.eFlags & EF_HELD_BY_RANCOR) != 0 {
        //If in a rancor's hand, never try to navigate!
        return 1;
    }
    if ((*NPC).s.eFlags & EF_HELD_BY_WAMPA) != 0 {
        //If in a wampa's hand, never try to navigate!
        return 1;
    }
    if ((*NPC).s.eFlags & EF_HELD_BY_SAND_CREATURE) != 0 {
        //If in a worm's mouth, never try to navigate!
        return 1;
    }

    if ((*NPC).watertype & CONTENTS_LADDER) != 0 {
        //Do we still want to do this?
        let mut dir: vec3_t = [0.0f32; 3];
        VectorSubtract(
            (*(*NPCInfo).goalEntity).currentOrigin,
            (*NPC).currentOrigin,
            &mut dir,
        );
        VectorNormalize(&mut dir);
        NPC_LadderMove(&dir);
    }

    let mut moveSuccess: bool = true;
    STEER_Activate(NPC);
    {
        // Attempt To Steer Directly To Our Goal
        //---------------------------------------
        moveSuccess = STEER_GoTo(
            NPC,
            (*NPCInfo).goalEntity,
            (*NPCInfo).goalRadius,
        ) != 0;

        // Perhaps Not Close Enough?  Try To Use The Navigation Grid
        //-----------------------------------------------------------
        if !moveSuccess {
            moveSuccess = NAV_GoTo(NPC, (*NPCInfo).goalEntity) != 0;
            if !moveSuccess {
                STEER_Stop(NPC);
            }
        }
    }
    STEER_DeActivate(NPC, &mut ucmd);

    #[cfg(feature = "AI_TIMERS")]
    {
        navTime += GetTime(startTime);
    }
    return if moveSuccess { 1 } else { 0 };
}

/*
-------------------------
void NPC_SlideMoveToGoal( void )

  Now assumes goal is goalEntity, if want to use tempGoal, you set that before calling the func
-------------------------
*/
pub unsafe fn NPC_SlideMoveToGoal() -> qboolean {
    let saveYaw: f32 = (*(*NPC).client).ps.viewangles[YAW];

    (*NPCInfo).combatMove = 1;

    let ret: qboolean = NPC_MoveToGoal(1);

    (*NPCInfo).desiredYaw = saveYaw;

    return ret;
}

/*
-------------------------
NPC_ApplyRoff
-------------------------
*/

pub unsafe fn NPC_ApplyRoff() {
    PlayerStateToEntityState(&(*(*NPC).client).ps, &mut (*NPC).s);
    VectorCopy((*NPC).currentOrigin, &mut (*NPC).lastOrigin);

    // use the precise origin for linking
    gi_linkentity(NPC);
}

// Stub implementations for gi functions (these would normally come from the engine)
// In a real port, these would be extern "C" functions from the actual engine

unsafe fn gi_trace(
    result: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    passent: c_int,
    contentmask: c_int,
) {
    // This is a stub; the real implementation comes from the engine
    // For now, initialize the trace result to a sensible default
    (*result).allsolid = 0;
    (*result).startsolid = 0;
    (*result).fraction = 1.0f32;
}

unsafe fn gi_linkentity(ent: *mut gentity_t) {
    // This is a stub; the real implementation comes from the engine
}
