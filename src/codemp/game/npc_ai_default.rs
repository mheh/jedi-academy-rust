//! Slice of `NPC_AI_Default.c` — the generic NPC AI behavior-state core. The
//! keystone behavior-state dispatchers that other NPC files lean on land here.
//!
//! Ported: `NPC_LostEnemyDecideChase` (NPC_AI_Default.c:18),
//! `NPC_StandIdle` (:42, empty — body is `#if 0`'d),
//! `NPC_StandTrackAndShoot` (:87), `NPC_BSIdle` (:161), `NPC_BSRun` (:179),
//! `NPC_BSStandGuard` (:191), `NPC_BSHuntAndKill` (:232),
//! `NPC_BSStandAndShoot` (:306), `NPC_BSRunAndShoot` (:394), `NPC_BSFace` (:490),
//! `NPC_BSPointShoot` (:505), `NPC_BSMove` (:616), `NPC_BSShoot` (:644),
//! `NPC_BSPatrol` (:664), `NPC_BSDefault` (:712).

#![allow(non_snake_case)] // C function names kept verbatim
#![allow(non_upper_case_globals)] // C `#define` constants kept verbatim

use core::ffi::c_int;
use core::ptr::addr_of;

use crate::codemp::game::anims::{
    BOTH_ATTACK1, BOTH_ATTACK2, BOTH_ATTACK3, BOTH_MELEE1, BOTH_MELEE2, TORSO_SURRENDER_START,
};
use crate::codemp::game::b_public_h::{
    BS_DEFAULT, BS_HUNT_AND_KILL, BS_SEARCH, BS_STAND_AND_SHOOT, BS_STAND_GUARD,
    SCF_FACE_MOVE_DIR, SCF_FIRE_WEAPON, SCF_FORCED_MARCH, SCF_IGNORE_ALERTS,
    SCF_LOOK_FOR_ENEMIES, SCF_RUNNING, SCF_WALKING, SPOT_HEAD, SPOT_WEAPON, VIS_PVS, VIS_SHOOT,
};
use crate::codemp::game::g_local::AEL_DISCOVERED;
use crate::codemp::game::bg_public::{SETANIM_FLAG_HOLD, SETANIM_TORSO, WEAPON_FIRING, WEAPON_READY};
use crate::codemp::game::bg_weapons_h::{WP_NONE, WP_SABER, WP_STUN_BATON};
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_nav::WAYPOINT_NONE;
use crate::codemp::game::teams_h::NPCTEAM_PLAYER;
use crate::codemp::game::g_main::level;
use crate::codemp::game::npc::{client, ucmd, NPC_SetAnim, NPC, NPCInfo};
use crate::codemp::game::npc_ai_stormtrooper::NPC_BSST_Attack;
use crate::codemp::game::npc_behavior::{NPC_BSFollowLeader, NPC_BSSearchStart};
use crate::codemp::game::npc_combat::{
    enemyVisibility, G_ClearEnemy, G_SetEnemy, IdealDistance, NPC_CheckCanAttack,
    NPC_CheckDefend, NPC_CheckEnemy, NPC_CheckGetNewWeapon, NPC_EnemyTooFar,
    NPC_MaxDistSquaredForWeapon, NPC_PickEnemy, WeaponThink,
};
use crate::codemp::game::npc_goal::{NPC_ClearGoal, UpdateGoal};
use crate::codemp::game::npc_move::{NPC_MoveToGoal, NPC_SlideMoveToGoal};
use crate::codemp::game::npc_senses::{NPC_CheckAlertEvents, NPC_CheckVisibility};
use crate::codemp::game::npc_utils::{CalcEntitySpot, NPC_SomeoneLookingAtMe, NPC_UpdateAngles};
use crate::codemp::game::q_math::{
    vectoangles, AngleDelta, AngleNormalize360, VectorLength, VectorScale, VectorSubtract,
};
use crate::codemp::game::q_shared_h::{
    vec3_t, BUTTON_ATTACK, BUTTON_WALKING, DEG2RAD, PITCH, YAW,
};
use crate::codemp::game::g_public_h::{TID_BSTATE, TID_MOVE_NAV};
use crate::ffi::types::{qboolean, QFALSE, QTRUE};
use crate::trap;

// `#define CHECK_FOV 4` (b_local.h:167) — not re-exported from npc_senses; the C
// `NPC_CheckVisibility` flag bits, redeclared locally to match.
const CHECK_FOV: c_int = 4;
/// `#define CHECK_SHOOT 8` (b_local.h:168).
const CHECK_SHOOT: c_int = 8;

/*
void NPC_LostEnemyDecideChase(void)

  We lost our enemy and want to drop him but see if we should chase him if we are in the proper bState
*/
pub unsafe fn NPC_LostEnemyDecideChase() {
    match (*NPCInfo).behaviorState {
        BS_HUNT_AND_KILL => {
            //We were chasing him and lost him, so try to find him
            if (*NPC).enemy == (*NPCInfo).goalEntity
                && (*(*NPC).enemy).lastWaypoint != WAYPOINT_NONE
            {
                //Remember his last valid Wp, then check it out
                //FIXME: Should we only do this if there's no other enemies or we've got LOCKED_ENEMY on?
                NPC_BSSearchStart((*(*NPC).enemy).lastWaypoint, BS_SEARCH);
            }
            //If he's not our goalEntity, we're running somewhere else, so lose him
        }
        _ => {}
    }
    G_ClearEnemy(NPC);
}

/*
-------------------------
NPC_StandIdle
-------------------------
*/
pub fn NPC_StandIdle() {
    /*
        //Must be done with any other animations
        if ( NPC->client->ps.legsAnimTimer != 0 )
            return;

        //Not ready to do another one
        if ( TIMER_Done( NPC, "idleAnim" ) == false )
            return;

        int anim = NPC->client->ps.legsAnim;

        if ( anim != BOTH_STAND1 && anim != BOTH_STAND2 )
            return;

        //FIXME: Account for STAND1 or STAND2 here and set the base anim accordingly
        int	baseSeq = ( anim == BOTH_STAND1 ) ? BOTH_STAND1_RANDOM1 : BOTH_STAND2_RANDOM1;

        //Must have at least one random idle animation
        //NOTENOTE: This relies on proper ordering of animations, which SHOULD be okay
        if ( PM_HasAnimation( NPC, baseSeq ) == false )
            return;

        int	newIdle = Q_irand( 0, MAX_IDLE_ANIMS-1 );

        //FIXME: Technically this could never complete.. but that's not really too likely
        while( 1 )
        {
            if ( PM_HasAnimation( NPC, baseSeq + newIdle ) )
                break;

            newIdle = Q_irand( 0, MAX_IDLE_ANIMS );
        }

        //Start that animation going
        NPC_SetAnim( NPC, SETANIM_BOTH, baseSeq + newIdle, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );

        int newTime = PM_AnimLength( NPC->client->clientInfo.animFileIndex, (animNumber_t) (baseSeq + newIdle) );

        //Don't do this again for a random amount of time
        TIMER_Set( NPC, "idleAnim", newTime + Q_irand( 2000, 10000 ) );
    */
}

pub unsafe fn NPC_StandTrackAndShoot(NPC_: *mut gentity_t, canDuck: qboolean) -> qboolean {
    let mut attack_ok: qboolean = QFALSE;
    let mut duck_ok: qboolean = QFALSE;
    let mut faced: qboolean = QFALSE;
    let attack_scale: f32 = 1.0;

    //First see if we're hurt bad- if so, duck
    //FIXME: if even when ducked, we can shoot someone, we should.
    //Maybe is can be shot even when ducked, we should run away to the nearest cover?
    if canDuck != QFALSE {
        if (*NPC_).health < 20 {
            //	if( NPC->svFlags&SVF_HEALING || random() )
            if crate::codemp::game::q_shared::random() != 0.0 {
                duck_ok = QTRUE;
            }
        } else if (*NPC_).health < 40 {
            //			if ( NPC->svFlags&SVF_HEALING )
            //			{//Medic is on the way, get down!
            //				duck_ok = qtrue;
            //			}
            // no more borg
            //			if ( NPC->client->playerTeam!= TEAM_BORG )
            //			{//Borg don't care if they're about to die
            //attack_scale will be a max of .66
            //				attack_scale = NPC->health/60;
            //			}
        }
    }

    //NPC_CheckEnemy( qtrue, qfalse, qtrue );

    if duck_ok == QFALSE {
        //made this whole part a function call
        attack_ok = NPC_CheckCanAttack(attack_scale, QTRUE);
        faced = QTRUE;
    }

    if canDuck != QFALSE
        && (duck_ok != QFALSE || (attack_ok == QFALSE && (*client).ps.weaponTime <= 0))
        && ucmd.upmove != -127
    {
        //if we didn't attack check to duck if we're not already
        if duck_ok == QFALSE {
            if !(*(*NPC_).enemy).client.is_null() {
                if (*(*NPC_).enemy).enemy == NPC_ {
                    if (*(*(*NPC_).enemy).client).buttons & BUTTON_ATTACK != 0 {
                        //FIXME: determine if enemy fire angles would hit me or get close
                        if NPC_CheckDefend(1.0) != QFALSE {
                            //FIXME: Check self-preservation?  Health?
                            duck_ok = QTRUE;
                        }
                    }
                }
            }
        }

        if duck_ok != QFALSE {
            //duck and don't shoot
            attack_ok = QFALSE;
            ucmd.upmove = -127;
            (*NPCInfo).duckDebounceTime = (*addr_of!(level)).time + 1000; //duck for a full second
        }
    }
    let _ = attack_ok;

    faced
}

pub unsafe fn NPC_BSIdle() {
    //FIXME if there is no nav data, we need to do something else
    // if we're stuck, try to move around it
    if !UpdateGoal().is_null() {
        NPC_MoveToGoal(QTRUE);
    }

    if (ucmd.forwardmove == 0) && (ucmd.rightmove == 0) && (ucmd.upmove == 0) {
        //		NPC_StandIdle();
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
    ucmd.buttons |= BUTTON_WALKING;
}

pub unsafe fn NPC_BSRun() {
    //FIXME if there is no nav data, we need to do something else
    // if we're stuck, try to move around it
    if !UpdateGoal().is_null() {
        NPC_MoveToGoal(QTRUE);
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

pub unsafe fn NPC_BSStandGuard() {
    //FIXME: Use Snapshot info
    if (*NPC).enemy.is_null() {
        //Possible to pick one up by being shot
        if crate::codemp::game::q_shared::random() < 0.5 {
            if (*(*NPC).client).enemyTeam != 0 {
                let newenemy: *mut gentity_t = NPC_PickEnemy(
                    NPC,
                    (*(*NPC).client).enemyTeam,
                    if (*NPC).cantHitEnemyCounter < 10 {
                        QTRUE
                    } else {
                        QFALSE
                    },
                    if (*(*NPC).client).enemyTeam == NPCTEAM_PLAYER {
                        QTRUE
                    } else {
                        QFALSE
                    },
                    QTRUE,
                );
                //only checks for vis if couldn't hit last enemy
                if !newenemy.is_null() {
                    G_SetEnemy(NPC, newenemy);
                }
            }
        }
    }

    if !(*NPC).enemy.is_null() {
        if (*NPCInfo).tempBehavior == BS_STAND_GUARD {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        }

        if (*NPCInfo).behaviorState == BS_STAND_GUARD {
            (*NPCInfo).behaviorState = BS_STAND_AND_SHOOT;
        }
    }

    NPC_UpdateAngles(QTRUE, QTRUE);
}

/*
-------------------------
NPC_BSHuntAndKill
-------------------------
*/
// Oracle calls `IdealDistance(NPC)`, but oracle `IdealDistance(gentity_t *self)`
// (NPC_combat.c) never reads `self` — it uses the file-static `NPC`/`NPCInfo` globals —
// so the parameterless `IdealDistance()` here is behaviorally identical.
pub unsafe fn NPC_BSHuntAndKill() {
    let mut turned: qboolean = QFALSE;
    let mut vec: vec3_t = [0.0; 3];
    let enemyDist: f32;
    let oEVis;
    let curAnim: c_int;

    NPC_CheckEnemy(
        if (*NPCInfo).tempBehavior != BS_HUNT_AND_KILL {
            QTRUE
        } else {
            QFALSE
        },
        QFALSE,
        QTRUE,
    ); //don't find new enemy if this is tempbehav

    if !(*NPC).enemy.is_null() {
        oEVis = NPC_CheckVisibility((*NPC).enemy, CHECK_FOV | CHECK_SHOOT); //CHECK_360|//CHECK_PVS|
        enemyVisibility = oEVis;
        if enemyVisibility > VIS_PVS {
            if NPC_EnemyTooFar((*NPC).enemy, 0.0, QTRUE) == QFALSE {
                //Enemy is close enough to shoot - FIXME: this next func does this also, but need to know here for info on whether ot not to turn later
                NPC_CheckCanAttack(1.0, QFALSE);
                turned = QTRUE;
            }
        }

        curAnim = (*(*NPC).client).ps.legsAnim;
        if curAnim != BOTH_ATTACK1
            && curAnim != BOTH_ATTACK2
            && curAnim != BOTH_ATTACK3
            && curAnim != BOTH_MELEE1
            && curAnim != BOTH_MELEE2
        {
            //Don't move toward enemy if we're in a full-body attack anim
            //FIXME, use IdealDistance to determin if we need to close distance
            VectorSubtract(
                &(*(*NPC).enemy).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                &mut vec,
            );
            enemyDist = VectorLength(&vec);
            if enemyDist > 48.0
                && ((enemyDist * 1.5) * (enemyDist * 1.5) >= NPC_MaxDistSquaredForWeapon()
                    || oEVis != VIS_SHOOT
                    // !(ucmd.buttons & BUTTON_ATTACK) ||
                    || enemyDist > IdealDistance() * 3.0)
            {
                //We should close in?
                (*NPCInfo).goalEntity = (*NPC).enemy;

                NPC_MoveToGoal(QTRUE);
            } else if enemyDist < IdealDistance() {
                //We should back off?
                //if(ucmd.buttons & BUTTON_ATTACK)
                {
                    (*NPCInfo).goalEntity = (*NPC).enemy;
                    (*NPCInfo).goalRadius = 12;
                    NPC_MoveToGoal(QTRUE);

                    ucmd.forwardmove = (ucmd.forwardmove as c_int * -1) as i8;
                    ucmd.rightmove = (ucmd.rightmove as c_int * -1) as i8;
                    let moveDir = (*(*NPC).client).ps.moveDir;
                    VectorScale(&moveDir, -1.0, &mut (*(*NPC).client).ps.moveDir);

                    ucmd.buttons |= BUTTON_WALKING;
                }
            } //otherwise, stay where we are
        }
    } else {
        //ok, stand guard until we find an enemy
        if (*NPCInfo).tempBehavior == BS_HUNT_AND_KILL {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        } else {
            (*NPCInfo).tempBehavior = BS_STAND_GUARD;
            NPC_BSStandGuard();
        }
        return;
    }

    if turned == QFALSE {
        NPC_UpdateAngles(QTRUE, QTRUE);
    }
}

pub unsafe fn NPC_BSStandAndShoot() {
    //FIXME:
    //When our numbers outnumber enemies 3 to 1, or only one of them,
    //go into hunt and kill mode

    //FIXME:
    //When they're all dead, go to some script or wander off to sickbay?

    if (*(*NPC).client).playerTeam != 0 && (*(*NPC).client).enemyTeam != 0 {
        //FIXME: don't realize this right away- or else enemies show up and we're standing around
        /*
        if( teamNumbers[NPC->enemyTeam] == 0 )
        {//ok, stand guard until we find another enemy
            //reset our rush counter
            teamCounter[NPC->playerTeam] = 0;
            NPCInfo->tempBehavior = BS_STAND_GUARD;
            NPC_BSStandGuard();
            return;
        }*/
        /*
        //FIXME: whether to do this or not should be settable
        else if( NPC->playerTeam != TEAM_BORG )//Borg don't rush
        ... (rush-logic block elided in original) ...
        */
    }

    NPC_CheckEnemy(QTRUE, QFALSE, QTRUE);

    if (*NPCInfo).duckDebounceTime > (*addr_of!(level)).time && (*(*NPC).client).ps.weapon != WP_SABER
    {
        ucmd.upmove = -127;
        if !(*NPC).enemy.is_null() {
            NPC_CheckCanAttack(1.0, QTRUE);
        }
        return;
    }

    if !(*NPC).enemy.is_null() {
        if NPC_StandTrackAndShoot(NPC, QTRUE) == QFALSE {
            //That func didn't update our angles
            (*NPCInfo).desiredYaw = (*(*NPC).client).ps.viewangles[YAW];
            (*NPCInfo).desiredPitch = (*(*NPC).client).ps.viewangles[PITCH];
            NPC_UpdateAngles(QTRUE, QTRUE);
        }
    } else {
        (*NPCInfo).desiredYaw = (*(*NPC).client).ps.viewangles[YAW];
        (*NPCInfo).desiredPitch = (*(*NPC).client).ps.viewangles[PITCH];
        NPC_UpdateAngles(QTRUE, QTRUE);
        //		NPC_BSIdle();//only moves if we have a goal
    }
}

pub unsafe fn NPC_BSRunAndShoot() {
    /*if(NPC->playerTeam && NPC->enemyTeam)
    {
        //FIXME: don't realize this right away- or else enemies show up and we're standing around
        if( teamNumbers[NPC->enemyTeam] == 0 )
        {//ok, stand guard until we find another enemy
            //reset our rush counter
            teamCounter[NPC->playerTeam] = 0;
            NPCInfo->tempBehavior = BS_STAND_GUARD;
            NPC_BSStandGuard();
            return;
        }
    }*/

    //NOTE: are we sure we want ALL run and shoot people to move this way?
    //Shouldn't it check to see if we have an enemy and our enemy is our goal?!
    //Moved that check into NPC_MoveToGoal
    //NPCInfo->combatMove = qtrue;

    NPC_CheckEnemy(QTRUE, QFALSE, QTRUE);

    if (*NPCInfo).duckDebounceTime > (*addr_of!(level)).time {
        // && NPCInfo->hidingGoal )
        ucmd.upmove = -127;
        if !(*NPC).enemy.is_null() {
            NPC_CheckCanAttack(1.0, QFALSE);
        }
        return;
    }

    if !(*NPC).enemy.is_null() {
        let monitor: c_int = (*NPC).cantHitEnemyCounter;
        NPC_StandTrackAndShoot(NPC, QFALSE); //(NPCInfo->hidingGoal != NULL) );

        if ucmd.buttons & BUTTON_ATTACK == 0
            && ucmd.upmove >= 0
            && (*NPC).cantHitEnemyCounter > monitor
        {
            //not crouching and not firing
            let mut vec: vec3_t = [0.0; 3];

            VectorSubtract(
                &(*(*NPC).enemy).r.currentOrigin,
                &(*NPC).r.currentOrigin,
                &mut vec,
            );
            vec[2] = 0.0;
            if VectorLength(&vec) > 128.0 || (*NPC).cantHitEnemyCounter >= 10 {
                //run at enemy if too far away
                //The cantHitEnemyCounter getting high has other repercussions
                //100 (10 seconds) will make you try to pick a new enemy...
                //But we're chasing, so we clamp it at 50 here
                if (*NPC).cantHitEnemyCounter > 60 {
                    (*NPC).cantHitEnemyCounter = 60;
                }

                if (*NPC).cantHitEnemyCounter >= ((*NPCInfo).stats.aggression + 1) * 10 {
                    NPC_LostEnemyDecideChase();
                }

                //chase and face
                ucmd.angles[YAW] = 0;
                ucmd.angles[PITCH] = 0;
                (*NPCInfo).goalEntity = (*NPC).enemy;
                (*NPCInfo).goalRadius = 12;
                //NAV_ClearLastRoute(NPC);
                NPC_MoveToGoal(QTRUE);
                NPC_UpdateAngles(QTRUE, QTRUE);
            } else {
                //FIXME: this could happen if they're just on the other side
                //of a thin wall or something else blocking out shot.  That
                //would make us just stand there and not go around it...
                //but maybe it's okay- might look like we're waiting for
                //him to come out...?
                //Current solution: runs around if cantHitEnemyCounter gets
                //to 10 (1 second).
            }
        } else {
            //Clear the can't hit enemy counter here
            (*NPC).cantHitEnemyCounter = 0;
        }
    } else {
        if (*NPCInfo).tempBehavior == BS_HUNT_AND_KILL {
            //lost him, go back to what we were doing before
            (*NPCInfo).tempBehavior = BS_DEFAULT;
            return;
        }

        //		NPC_BSRun();//only moves if we have a goal
    }
}

//Simply turn until facing desired angles
pub unsafe fn NPC_BSFace() {
    //FIXME: once you stop sending turning info, they reset to whatever their delta_angles was last????
    //Once this is over, it snaps back to what it was facing before- WHY???
    if NPC_UpdateAngles(QTRUE, QTRUE) != QFALSE {
        trap::ICARUS_TaskIDComplete(NPC, TID_BSTATE);

        (*NPCInfo).desiredYaw = (*client).ps.viewangles[YAW];
        (*NPCInfo).desiredPitch = (*client).ps.viewangles[PITCH];

        (*NPCInfo).aimTime = 0; //ok to turn normally now
    }
}

pub unsafe fn NPC_BSPointShoot(shoot: qboolean) {
    //FIXME: doesn't check for clear shot...
    let mut muzzle: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut org: vec3_t = [0.0; 3];

    if (*NPC).enemy.is_null()
        || (*(*NPC).enemy).inuse == QFALSE
        || (!(*(*NPC).enemy).NPC.is_null() && (*(*NPC).enemy).health <= 0)
    {
        //FIXME: should still keep shooting for a second or two after they actually die...
        trap::ICARUS_TaskIDComplete(NPC, TID_BSTATE);
        // goto finished;
        finished(dir);
        return;
    }

    CalcEntitySpot(NPC, SPOT_WEAPON, &mut muzzle);
    CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut org); //Was spot_org
                                                       //Head is a little high, so let's aim for the chest:
    if !(*(*NPC).enemy).client.is_null() {
        org[2] -= 12.0; //NOTE: is this enough?
    }

    VectorSubtract(&org, &muzzle, &mut dir);
    vectoangles(&dir, &mut angles);

    match (*(*NPC).client).ps.weapon {
        //	case WP_TRICORDER:
        WP_NONE | WP_STUN_BATON | WP_SABER => {
            //don't do any pitch change if not holding a firing weapon
        }
        _ => {
            (*NPCInfo).lockedDesiredPitch = AngleNormalize360(angles[PITCH]);
            (*NPCInfo).desiredPitch = (*NPCInfo).lockedDesiredPitch;
        }
    }

    (*NPCInfo).lockedDesiredYaw = AngleNormalize360(angles[YAW]);
    (*NPCInfo).desiredYaw = (*NPCInfo).lockedDesiredYaw;

    if NPC_UpdateAngles(QTRUE, QTRUE) != QFALSE {
        //FIXME: if angles clamped, this may never work!
        //NPCInfo->shotTime = NPC->attackDebounceTime = 0;

        if shoot != QFALSE {
            //FIXME: needs to hold this down if using a weapon that requires it, like phaser...
            ucmd.buttons |= BUTTON_ATTACK;
        }

        //if ( !shoot || !(NPC->svFlags & SVF_LOCKEDENEMY) )
        if true {
            //If locked_enemy is on, dont complete until it is destroyed...
            trap::ICARUS_TaskIDComplete(NPC, TID_BSTATE);
            // goto finished;
            finished(dir);
            return;
        }
    }
    //else if ( shoot && (NPC->svFlags & SVF_LOCKEDENEMY) )
    if false {
        //shooting them till their dead, not aiming right at them yet...
        /*
        qboolean movingTarget = qfalse;

        if ( NPC->enemy->client )
        {
            if ( VectorLengthSquared( NPC->enemy->client->ps.velocity ) )
            {
                movingTarget = qtrue;
            }
        }
        else if ( VectorLengthSquared( NPC->enemy->s.pos.trDelta ) )
        {
            movingTarget = qtrue;
        }

        if (movingTarget )
        */
        {
            let dist: f32 = VectorLength(&dir);
            let yawMiss: f32;
            let mut yawMissAllow: f32 = (*(*NPC).enemy).r.maxs[0];
            let pitchMiss: f32;
            let mut pitchMissAllow: f32 =
                ((*(*NPC).enemy).r.maxs[2] - (*(*NPC).enemy).r.mins[2]) / 2.0;

            if yawMissAllow < 8.0 {
                yawMissAllow = 8.0;
            }

            if pitchMissAllow < 8.0 {
                pitchMissAllow = 8.0;
            }

            yawMiss = (DEG2RAD(AngleDelta((*(*NPC).client).ps.viewangles[YAW], (*NPCInfo).desiredYaw))
                as f64)
                .tan() as f32
                * dist;
            pitchMiss = (DEG2RAD(AngleDelta(
                (*(*NPC).client).ps.viewangles[PITCH],
                (*NPCInfo).desiredPitch,
            )) as f64)
                .tan() as f32
                * dist;

            if yawMissAllow >= yawMiss && pitchMissAllow > pitchMiss {
                ucmd.buttons |= BUTTON_ATTACK;
            }
        }
    }

    return;

    // finished: — the C `goto finished` epilogue, hoisted into a local closure.
    unsafe fn finished(_dir: vec3_t) {
        (*NPCInfo).desiredYaw = (*client).ps.viewangles[YAW];
        (*NPCInfo).desiredPitch = (*client).ps.viewangles[PITCH];

        (*NPCInfo).aimTime = 0; //ok to turn normally now
    }
}

/*
void NPC_BSMove(void)
Move in a direction, face another
*/
pub unsafe fn NPC_BSMove() {
    let goal: *mut gentity_t;

    NPC_CheckEnemy(QTRUE, QFALSE, QTRUE);
    if !(*NPC).enemy.is_null() {
        NPC_CheckCanAttack(1.0, QFALSE);
    } else {
        NPC_UpdateAngles(QTRUE, QTRUE);
    }

    goal = UpdateGoal();
    if !goal.is_null() {
        //		NPCInfo->moveToGoalMod = 1.0;

        NPC_SlideMoveToGoal();
    }
}

/*
void NPC_BSShoot(void)
Move in a direction, face another
*/
pub unsafe fn NPC_BSShoot() {
    //	NPC_BSMove();

    enemyVisibility = VIS_SHOOT;

    if (*client).ps.weaponstate != WEAPON_READY && (*client).ps.weaponstate != WEAPON_FIRING {
        (*client).ps.weaponstate = WEAPON_READY;
    }

    WeaponThink(QTRUE);
}

/*
void NPC_BSPatrol( void )

  Same as idle, but you look for enemies every "vigilance"
  using your angles, HFOV, VFOV and visrange, and listen for sounds within earshot...
*/
pub unsafe fn NPC_BSPatrol() {
    //int	alertEventNum;

    if (*addr_of!(level)).time > (*NPCInfo).enemyCheckDebounceTime {
        (*NPCInfo).enemyCheckDebounceTime =
            (*addr_of!(level)).time + ((*NPCInfo).stats.vigilance * 1000.0) as c_int;
        NPC_CheckEnemy(QTRUE, QFALSE, QTRUE);
        if !(*NPC).enemy.is_null() {
            //FIXME: do anger script
            (*NPCInfo).behaviorState = BS_HUNT_AND_KILL;
            //NPC_AngerSound();
            return;
        }
    }

    //FIXME: Implement generic sound alerts
    /*
    alertEventNum = NPC_CheckAlertEvents( qtrue, qtrue );
    if( alertEventNum != -1 )
    {//If we heard something, see if we should check it out
        if ( NPC_CheckInvestigate( alertEventNum ) )
        {
            return;
        }
    }
    */

    (*NPCInfo).investigateSoundDebounceTime = 0;
    //FIXME if there is no nav data, we need to do something else
    // if we're stuck, try to move around it
    if !UpdateGoal().is_null() {
        NPC_MoveToGoal(QTRUE);
    }

    NPC_UpdateAngles(QTRUE, QTRUE);

    ucmd.buttons |= BUTTON_WALKING;
}

/*
void NPC_BSDefault(void)
	uses various scriptflags to determine how an npc should behave
*/
pub unsafe fn NPC_BSDefault() {
    //	vec3_t		enemyDir;
    //	float		enemyDist;
    //	float		shootDist;
    //	qboolean	enemyFOV = qfalse;
    //	qboolean	enemyShotFOV = qfalse;
    //	qboolean	enemyPVS = qfalse;
    //	vec3_t		enemyHead;
    //	vec3_t		muzzle;
    //	qboolean	enemyLOS = qfalse;
    //	qboolean	enemyCS = qfalse;
    let mut moveit: qboolean = QTRUE;
    //	qboolean	shoot = qfalse;

    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(QTRUE);
    }

    if (*NPCInfo).scriptFlags & SCF_FORCED_MARCH != 0 {
        //being forced to walk
        if (*(*NPC).client).ps.torsoAnim != TORSO_SURRENDER_START {
            NPC_SetAnim(NPC, SETANIM_TORSO, TORSO_SURRENDER_START, SETANIM_FLAG_HOLD);
        }
    }
    //look for a new enemy if don't have one and are allowed to look, validate current enemy if have one
    NPC_CheckEnemy(
        if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
            QTRUE
        } else {
            QFALSE
        },
        QFALSE,
        QTRUE,
    );
    if (*NPC).enemy.is_null() {
        //still don't have an enemy
        if (*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS == 0 {
            //check for alert events
            //FIXME: Check Alert events, see if we should investigate or just look at it
            let alertEvent: c_int = NPC_CheckAlertEvents(QTRUE, QTRUE, -1, QTRUE, AEL_DISCOVERED);

            //There is an event to look at
            if alertEvent >= 0
                && (*addr_of!(level)).alertEvents[alertEvent as usize].ID != (*NPCInfo).lastAlertID
            {
                //heard/saw something
                if (*addr_of!(level)).alertEvents[alertEvent as usize].level >= AEL_DISCOVERED
                    && (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0
                {
                    //was a big event
                    let owner = (*addr_of!(level)).alertEvents[alertEvent as usize].owner;
                    if !owner.is_null()
                        && !(*owner).client.is_null()
                        && (*owner).health >= 0
                        && (*(*owner).client).playerTeam == (*(*NPC).client).enemyTeam
                    {
                        //an enemy
                        G_SetEnemy(NPC, owner);
                    }
                } else {
                    //FIXME: investigate lesser events
                }
            }
            //FIXME: also check our allies' condition?
        }
    }

    if !(*NPC).enemy.is_null() && (*NPCInfo).scriptFlags & SCF_FORCED_MARCH == 0 {
        // just use the stormtrooper attack AI...
        NPC_CheckGetNewWeapon();
        if !(*(*NPC).client).leader.is_null()
            && (*NPCInfo).goalEntity == (*(*NPC).client).leader
            && trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE
        {
            NPC_ClearGoal();
        }
        NPC_BSST_Attack();
        return;
        /*
                //have an enemy
                //FIXME: if one of these fails, meaning we can't shoot, do we really need to do the rest?
                VectorSubtract( NPC->enemy->r.currentOrigin, NPC->r.currentOrigin, enemyDir );
                enemyDist = VectorNormalize( enemyDir );
                enemyDist *= enemyDist;
                shootDist = NPC_MaxDistSquaredForWeapon();

                enemyFOV = InFOV( NPC->enemy, NPC, NPCInfo->stats.hfov, NPCInfo->stats.vfov );
                enemyShotFOV = InFOV( NPC->enemy, NPC, 20, 20 );
                enemyPVS = gi.inPVS( NPC->enemy->r.currentOrigin, NPC->r.currentOrigin );

                if ( enemyPVS )
                {//in the pvs
                    trace_t	tr;

                    CalcEntitySpot( NPC->enemy, SPOT_HEAD, enemyHead );
                    enemyHead[2] -= Q_flrand( 0.0f, NPC->enemy->maxs[2]*0.5f );
                    CalcEntitySpot( NPC, SPOT_WEAPON, muzzle );
                    enemyLOS = NPC_ClearLOS( muzzle, enemyHead );

                    gi.trace ( &tr, muzzle, vec3_origin, vec3_origin, enemyHead, NPC->s.number, MASK_SHOT );
                    enemyCS = NPC_EvaluateShot( tr.entityNum, qtrue );
                }
                else
                {//skip thr 2 traces since they would have to fail
                    enemyLOS = qfalse;
                    enemyCS = qfalse;
                }

                if ( enemyCS && enemyShotFOV )
                {//can hit enemy if we want
                    NPC->cantHitEnemyCounter = 0;
                }
                else
                {//can't hit
                    NPC->cantHitEnemyCounter++;
                }

                if ( enemyCS && enemyShotFOV && enemyDist < shootDist )
                {//can shoot
                    shoot = qtrue;
                    if ( NPCInfo->goalEntity == NPC->enemy )
                    {//my goal is my enemy and I have a clear shot, no need to chase right now
                        move = qfalse;
                    }
                }
                else
                {//don't shoot yet, keep chasing
                    shoot = qfalse;
                    move = qtrue;
                }

                //shoot decision
                if ( !(NPCInfo->scriptFlags&SCF_DONT_FIRE) )
                {//try to shoot
                    if ( NPC->enemy )
                    {
                        if ( shoot )
                        {
                            if( !(NPCInfo->scriptFlags & SCF_FIRE_WEAPON) ) // we've already fired, no need to do it again here
                            {
                                WeaponThink( qtrue );
                            }
                        }
                    }
                }

                //chase decision
                if ( NPCInfo->scriptFlags & SCF_CHASE_ENEMIES )
                {//go after him
                    NPCInfo->goalEntity = NPC->enemy;
                    //FIXME: don't need to chase when have a clear shot and in range?
                    if ( !enemyCS && NPC->cantHitEnemyCounter > 60 )
                    {//haven't been able to shoot enemy for about 6 seconds, need to do something
                        //FIXME: combat points?  Just chase?
                        if ( enemyPVS )
                        {//in my PVS, just pick a combat point
                            //FIXME: implement
                        }
                        else
                        {//just chase him
                        }
                    }
                    //FIXME: in normal behavior, should we use combat Points?  Do we care?  Is anyone actually going to ever use this AI?
                }
                else if ( NPC->cantHitEnemyCounter > 60 )
                {//pick a new one
                    NPC_CheckEnemy( qtrue, qfalse, qtrue );
                }

                if ( enemyPVS && enemyLOS )//&& !enemyShotFOV )
                {//have a clear LOS to him//, but not looking at him
                    //Find the desired angles
                    vec3_t	angles;

                    GetAnglesForDirection( muzzle, enemyHead, angles );

                    NPCInfo->desiredYaw		= AngleNormalize180( angles[YAW] );
                    NPCInfo->desiredPitch	= AngleNormalize180( angles[PITCH] );
                }
                */
    }

    if UpdateGoal() != core::ptr::null_mut() {
        //have a goal
        if (*NPC).enemy.is_null()
            && !(*(*NPC).client).leader.is_null()
            && (*NPCInfo).goalEntity == (*(*NPC).client).leader
            && trap::ICARUS_TaskIDPending(NPC, TID_MOVE_NAV) == QFALSE
        {
            NPC_BSFollowLeader();
        } else {
            //set angles
            if (*NPCInfo).scriptFlags & SCF_FACE_MOVE_DIR != 0
                || (*NPCInfo).goalEntity != (*NPC).enemy
            {
                //face direction of movement, NOTE: default behavior when not chasing enemy
                (*NPCInfo).combatMove = QFALSE;
            } else {
                //face goal.. FIXME: what if have a navgoal but want to face enemy while moving?  Will this do that?
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];

                (*NPCInfo).combatMove = QFALSE;

                VectorSubtract(
                    &(*(*NPCInfo).goalEntity).r.currentOrigin,
                    &(*NPC).r.currentOrigin,
                    &mut dir,
                );
                vectoangles(&dir, &mut angles);
                (*NPCInfo).desiredYaw = angles[YAW];
                if (*NPCInfo).goalEntity == (*NPC).enemy {
                    (*NPCInfo).desiredPitch = angles[PITCH];
                }
            }

            //set movement
            //override default walk/run behavior
            //NOTE: redundant, done in NPC_ApplyScriptFlags
            if (*NPCInfo).scriptFlags & SCF_RUNNING != 0 {
                ucmd.buttons &= !BUTTON_WALKING;
            } else if (*NPCInfo).scriptFlags & SCF_WALKING != 0 {
                ucmd.buttons |= BUTTON_WALKING;
            } else if (*NPCInfo).goalEntity == (*NPC).enemy {
                ucmd.buttons &= !BUTTON_WALKING;
            } else {
                ucmd.buttons |= BUTTON_WALKING;
            }

            if (*NPCInfo).scriptFlags & SCF_FORCED_MARCH != 0 {
                //being forced to walk
                //if ( g_crosshairEntNum != NPC->s.number )
                if NPC_SomeoneLookingAtMe(NPC) == QFALSE {
                    //don't walk if player isn't aiming at me
                    moveit = QFALSE;
                }
            }

            if moveit != QFALSE {
                //move toward goal
                NPC_MoveToGoal(QTRUE);
            }
        }
    } else if (*NPC).enemy.is_null() && !(*(*NPC).client).leader.is_null() {
        NPC_BSFollowLeader();
    }

    //update angles
    NPC_UpdateAngles(QTRUE, QTRUE);
}
