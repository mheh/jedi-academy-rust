#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// Extern declarations for engine types and functions
extern "C" {
    pub static mut NPC: *mut gentity_t;
    pub static mut client: *mut gclient_t;
    pub static mut NPCInfo: *mut gNPC_t;
    pub static mut ucmd: usercmd_t;
    pub static mut level: level_locals_t;
    pub static g_crosshairEntNum: c_int;

    // Functions from other modules
    pub fn NPC_LostEnemyDecideChase();
    pub fn NPC_CheckEnemy(check_alerts: qboolean, find_new: qboolean);
    pub fn NPC_CheckVisibility(ent: *mut gentity_t, flags: c_int) -> visibility_t;
    pub fn NPC_EnemyTooFar(ent: *mut gentity_t, distance: c_int, check_line: qboolean) -> qboolean;
    pub fn NPC_CheckCanAttack(scale: f32, use_ammo: qboolean) -> qboolean;
    pub fn NPC_CheckDefend(damage: f32) -> qboolean;
    pub fn NPC_PickEnemy(
        ent: *mut gentity_t,
        enemy_team: c_int,
        check_los: qboolean,
        player_team_check: qboolean,
        check_unk: qboolean,
    ) -> *mut gentity_t;
    pub fn G_SetEnemy(npc: *mut gentity_t, enemy: *mut gentity_t);
    pub fn G_ClearEnemy(npc: *mut gentity_t);
    pub fn NPC_BSSearchStart(waypoint: c_int, behavior_state: c_int);
    pub fn NPC_MoveToGoal(retreat: qboolean);
    pub fn NPC_UpdateAngles(do_pitch: qboolean, do_yaw: qboolean) -> qboolean;
    pub fn UpdateGoal() -> *mut gentity_t;
    pub fn NPC_SlideMoveToGoal();
    pub fn NPC_SetAnim(
        ent: *mut gentity_t,
        set_anim: c_int,
        anim: c_int,
        flags: c_int,
    );
    pub fn NPC_ClearGoal();
    pub fn NPC_CheckGetNewWeapon();
    pub fn NPC_BSST_Attack();
    pub fn NPC_CheckAlertEvents(
        check_sounds: qboolean,
        check_sights: qboolean,
        alert_ent_num: c_int,
        check_alerts: qboolean,
        min_level: c_int,
    ) -> c_int;
    pub fn NPC_BSFollowLeader();
    pub fn WeaponThink(ai_level: qboolean);
    pub fn CalcEntitySpot(ent: *mut gentity_t, spot: c_int, spot_origin: *mut [f32; 3]);
    pub fn VectorSubtract(vec_a: *const [f32; 3], vec_b: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorLength(vec: *const [f32; 3]) -> f32;
    pub fn VectorScale(vec_in: *const [f32; 3], scale: f32, vec_out: *mut [f32; 3]);
    pub fn vectoangles(vec: *const [f32; 3], angles: *mut [f32; 3]);
    pub fn VectorLengthSquared(vec: *const [f32; 3]) -> f32;
    pub fn AngleNormalize360(angle: f32) -> f32;
    pub fn AngleDelta(angle1: f32, angle2: f32) -> f32;
    pub fn Q3_TaskIDComplete(ent: *mut gentity_t, task_id: c_int);
    pub fn Q3_TaskIDPending(ent: *mut gentity_t, task_id: c_int) -> qboolean;
    pub fn random() -> f32;
    pub fn InFOV(
        ent: *mut gentity_t,
        npc: *mut gentity_t,
        hfov: f32,
        vfov: f32,
    ) -> qboolean;
    pub fn NPC_ClearLOS(start: *const [f32; 3], end: *const [f32; 3]) -> qboolean;
    pub fn NPC_EvaluateShot(entity_num: c_int, test: qboolean) -> qboolean;
    pub fn IdealDistance(npc: *mut gentity_t) -> f32;
    pub fn NPC_MaxDistSquaredForWeapon() -> f32;
    pub fn GetAnglesForDirection(start: *const [f32; 3], end: *const [f32; 3], angles: *mut [f32; 3]);

    // Math functions
    pub fn tan(x: f32) -> f32;
}

// Type stubs and constants
pub type qboolean = c_int;
pub type gentity_t = c_void;
pub type gclient_t = c_void;
pub type gNPC_t = c_void;
pub type visibility_t = c_int;
pub type usercmd_t = c_void;
pub type level_locals_t = c_void;

// Constants - Behavior States
const BS_HUNT_AND_KILL: c_int = 2;
const BS_SEARCH: c_int = 3;
const BS_DEFAULT: c_int = 0;
const BS_STAND_GUARD: c_int = 1;
const BS_STAND_AND_SHOOT: c_int = 4;
const BS_RUN_AND_SHOOT: c_int = 5;
const BS_FACE: c_int = 6;
const BS_POINT_SHOOT: c_int = 7;
const BS_MOVE: c_int = 8;
const BS_SHOOT: c_int = 9;
const BS_PATROL: c_int = 10;
const BS_IDLE: c_int = 11;
const BS_RUN: c_int = 12;

// Constants - Visibility
const VIS_PVS: c_int = 1;
const VIS_SHOOT: c_int = 2;

// Constants - Flags
const CHECK_FOV: c_int = 1;
const CHECK_SHOOT: c_int = 2;
const CHECK_360: c_int = 4;
const CHECK_PVS: c_int = 8;

// Animation constants
const BOTH_ATTACK1: c_int = 0;
const BOTH_ATTACK2: c_int = 1;
const BOTH_ATTACK3: c_int = 2;
const BOTH_MELEE1: c_int = 3;
const BOTH_MELEE2: c_int = 4;
const BOTH_STAND1: c_int = 5;
const BOTH_STAND2: c_int = 6;
const BOTH_STAND1_RANDOM1: c_int = 7;
const BOTH_STAND2_RANDOM1: c_int = 15;
const TORSO_SURRENDER_START: c_int = 100;

// Weapon constants
const WP_NONE: c_int = 0;
const WP_MELEE: c_int = 1;
const WP_TUSKEN_STAFF: c_int = 2;
const WP_SABER: c_int = 3;

// Team constants
const TEAM_PLAYER: c_int = 0;
const TEAM_BORG: c_int = 1;

// Alert event constants
const AEL_DISCOVERED: c_int = 1;

// Waypoint constants
const WAYPOINT_NONE: c_int = -1;

// Animation flags
const SETANIM_BOTH: c_int = 0;
const SETANIM_TORSO: c_int = 1;
const SETANIM_FLAG_OVERRIDE: c_int = 1;
const SETANIM_FLAG_HOLD: c_int = 2;

// Command flags
const BUTTON_ATTACK: c_int = 1;
const BUTTON_WALKING: c_int = 2;

// Weapon states
const WEAPON_READY: c_int = 0;
const WEAPON_FIRING: c_int = 1;

// Entity spot types
const SPOT_WEAPON: c_int = 0;
const SPOT_HEAD: c_int = 1;
const SPOT_ORG: c_int = 2;

// Server flags
const SVF_HEALING: c_int = 1;
const SVF_LOCKEDENEMY: c_int = 2;

// Script flags
const SCF_FIRE_WEAPON: c_int = 1;
const SCF_FORCED_MARCH: c_int = 2;
const SCF_LOOK_FOR_ENEMIES: c_int = 4;
const SCF_IGNORE_ALERTS: c_int = 8;
const SCF_CHASE_ENEMIES: c_int = 16;
const SCF_DONT_FIRE: c_int = 32;
const SCF_WALKING: c_int = 64;
const SCF_RUNNING: c_int = 128;
const SCF_FACE_MOVE_DIR: c_int = 256;

// Task IDs
const TID_BSTATE: c_int = 0;
const TID_MOVE_NAV: c_int = 1;

// Angle indices
const YAW: usize = 0;
const PITCH: usize = 1;
const ROLL: usize = 2;

// Macros - converted to inline functions
#[inline]
fn DEG2RAD(x: f32) -> f32 {
    x * std::f32::consts::PI / 180.0
}

#[inline]
fn Q_irand(min: c_int, max: c_int) -> c_int {
    min + ((random() * ((max - min + 1) as f32)) as c_int)
}

const MAX_IDLE_ANIMS: c_int = 8;

/*
void NPC_LostEnemyDecideChase(void)

  We lost our enemy and want to drop him but see if we should chase him if we are in the proper bState
*/

pub unsafe fn NPC_LostEnemyDecideChase_impl() {
    match (*NPCInfo).behaviorState {
        BS_HUNT_AND_KILL => {
            //We were chasing him and lost him, so try to find him
            if (*NPC).enemy == (*NPCInfo).goalEntity && (*(*NPC).enemy).lastWaypoint != WAYPOINT_NONE
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

pub unsafe fn NPC_StandIdle() {
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

pub unsafe fn NPC_StandTrackAndShoot(npc: *mut gentity_t, can_duck: qboolean) -> qboolean {
    let mut attack_ok: qboolean = 0; // qfalse
    let mut duck_ok: qboolean = 0; // qfalse
    let mut faced: qboolean = 0; // qfalse
    let mut attack_scale: f32 = 1.0;

    //First see if we're hurt bad- if so, duck
    //FIXME: if even when ducked, we can shoot someone, we should.
    //Maybe is can be shot even when ducked, we should run away to the nearest cover?
    if can_duck != 0 {
        if (*npc).health < 20 {
            //	if( NPC->svFlags&SVF_HEALING || random() )
            if random() != 0.0 {
                duck_ok = 1; // qtrue
            }
        } else if (*npc).health < 40 {
            //			if ( NPC->svFlags&SVF_HEALING )
            //			{//Medic is on the way, get down!
            //				duck_ok = qtrue;
            //			}
            // no more borg
            ///			if ( NPC->client->playerTeam!= TEAM_BORG )
            //			{//Borg don't care if they're about to die
            //attack_scale will be a max of .66
            //				attack_scale = NPC->health/60;
            //			}
        }
    }

    //NPC_CheckEnemy( qtrue, qfalse );

    if duck_ok == 0 {
        //made this whole part a function call
        attack_ok = NPC_CheckCanAttack(attack_scale, 1); // qtrue
        faced = 1; // qtrue
    }

    if can_duck != 0
        && (duck_ok != 0
            || (attack_ok == 0 && (*client).fireDelay == 0))
        && ucmd.upmove != -127
    {
        //if we didn't attack check to duck if we're not already
        if duck_ok == 0 {
            if (*(*npc).enemy).client != core::ptr::null_mut() {
                if (*(*(*npc).enemy).client).enemy == npc {
                    if (*(*(*npc).enemy).client).buttons & BUTTON_ATTACK != 0 {
                        //FIXME: determine if enemy fire angles would hit me or get close
                        if NPC_CheckDefend(1.0) != 0 {
                            //FIXME: Check self-preservation?  Health?
                            duck_ok = 1; // qtrue
                        }
                    }
                }
            }
        }

        if duck_ok != 0 {
            //duck and don't shoot
            attack_ok = 0; // qfalse
            ucmd.upmove = -127;
            (*NPCInfo).duckDebounceTime = (*level).time + 1000; //duck for a full second
        }
    }

    return faced;
}

pub unsafe fn NPC_BSIdle() {
    //FIXME if there is no nav data, we need to do something else
    // if we're stuck, try to move around it
    if UpdateGoal() != core::ptr::null_mut() {
        NPC_MoveToGoal(1); // qtrue
    }

    if (ucmd.forwardmove == 0) && (ucmd.rightmove == 0) && (ucmd.upmove == 0) {
        //		NPC_StandIdle();
    }

    NPC_UpdateAngles(1, 1); // qtrue, qtrue
    ucmd.buttons |= BUTTON_WALKING;
}

pub unsafe fn NPC_BSRun() {
    //FIXME if there is no nav data, we need to do something else
    // if we're stuck, try to move around it
    if UpdateGoal() != core::ptr::null_mut() {
        NPC_MoveToGoal(1); // qtrue
    }

    NPC_UpdateAngles(1, 1); // qtrue, qtrue
}

pub unsafe fn NPC_BSStandGuard() {
    //FIXME: Use Snapshot info
    if (*NPC).enemy == core::ptr::null_mut() {
        //Possible to pick one up by being shot
        if random() < 0.5 {
            if (*(*NPC).client).enemyTeam != 0 {
                let newenemy: *mut gentity_t = NPC_PickEnemy(
                    NPC,
                    (*(*NPC).client).enemyTeam,
                    if (*NPC).cantHitEnemyCounter < 10 { 1 } else { 0 }, // qboolean
                    if (*(*NPC).client).enemyTeam == TEAM_PLAYER { 1 } else { 0 }, // qboolean
                    1, // qtrue
                );
                //only checks for vis if couldn't hit last enemy
                if newenemy != core::ptr::null_mut() {
                    G_SetEnemy(NPC, newenemy);
                }
            }
        }
    }

    if (*NPC).enemy != core::ptr::null_mut() {
        if (*NPCInfo).tempBehavior == BS_STAND_GUARD {
            (*NPCInfo).tempBehavior = BS_DEFAULT;
        }

        if (*NPCInfo).behaviorState == BS_STAND_GUARD {
            (*NPCInfo).behaviorState = BS_STAND_AND_SHOOT;
        }
    }

    NPC_UpdateAngles(1, 1); // qtrue, qtrue
}

/*
-------------------------
NPC_BSHuntAndKill
-------------------------
*/

pub unsafe fn NPC_BSHuntAndKill() {
    let mut turned: qboolean = 0; // qfalse
    let mut vec: [f32; 3] = [0.0; 3];
    let mut enemy_dist: f32;
    let mut o_evis: visibility_t;
    let mut cur_anim: c_int;

    NPC_CheckEnemy(
        if (*NPCInfo).tempBehavior != BS_HUNT_AND_KILL {
            1
        } else {
            0
        }, // don't find new enemy if this is tempbehav
        0, // qfalse
    );

    if (*NPC).enemy != core::ptr::null_mut() {
        o_evis = NPC_CheckVisibility((*NPC).enemy, CHECK_FOV | CHECK_SHOOT); //CHECK_360|//CHECK_PVS|
        if o_evis > VIS_PVS {
            if !NPC_EnemyTooFar((*NPC).enemy, 0, 1) != 0 {
                //Enemy is close enough to shoot - FIXME: this next func does this also, but need to know here for info on whether ot not to turn later
                NPC_CheckCanAttack(1.0, 0); // qfalse
                turned = 1; // qtrue
            }
        }

        cur_anim = (*(*NPC).client).ps.legsAnim;
        if cur_anim != BOTH_ATTACK1
            && cur_anim != BOTH_ATTACK2
            && cur_anim != BOTH_ATTACK3
            && cur_anim != BOTH_MELEE1
            && cur_anim != BOTH_MELEE2
        {
            //Don't move toward enemy if we're in a full-body attack anim
            //FIXME, use IdealDistance to determin if we need to close distance
            VectorSubtract(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin, &mut vec);
            enemy_dist = VectorLength(&vec);
            if enemy_dist > 48.0
                && ((enemy_dist * 1.5) * (enemy_dist * 1.5) >= NPC_MaxDistSquaredForWeapon()
                    || o_evis != VIS_SHOOT
                    || //!(ucmd.buttons & BUTTON_ATTACK) ||
                    enemy_dist > IdealDistance(NPC) * 3.0)
            {
                //We should close in?
                (*NPCInfo).goalEntity = (*NPC).enemy;

                NPC_MoveToGoal(1); // qtrue
            } else if enemy_dist < IdealDistance(NPC) {
                //We should back off?
                //if(ucmd.buttons & BUTTON_ATTACK)
                {
                    (*NPCInfo).goalEntity = (*NPC).enemy;
                    (*NPCInfo).goalRadius = 12;
                    NPC_MoveToGoal(1); // qtrue

                    ucmd.forwardmove *= -1;
                    ucmd.rightmove *= -1;
                    VectorScale(&(*(*NPC).client).ps.moveDir, -1.0, &mut (*(*NPC).client).ps.moveDir);

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

    if turned == 0 {
        NPC_UpdateAngles(1, 1); // qtrue, qtrue
    }
}

pub unsafe fn NPC_BSStandAndShoot() {
    //FIXME:
    //When our numbers outnumber enemies 3 to 1, or only one of them,
    //go into hunt and kill mode

    //FIXME:
    //When they're all dead, go to some script or wander off to sickbay?

    if (*NPC).client != core::ptr::null_mut() && (*(*NPC).client).playerTeam != 0 && (*(*NPC).client).enemyTeam != 0
    {
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
        {
        //FIXME: In case reinforcements show up, we should wait a few seconds
        //and keep checking before rushing!
        //Also: what if not everyone on our team is going after playerTeam?
        //Also: our team count includes medics!
            if(NPC->health > 25)
            {//Can we rush the enemy?
                if(teamNumbers[NPC->enemyTeam] == 1 ||
                    teamNumbers[NPC->playerTeam] >= teamNumbers[NPC->enemyTeam]*3)
                {//Only one of them or we outnumber 3 to 1
                    if(teamStrength[NPC->playerTeam] >= 75 ||
                        (teamStrength[NPC->playerTeam] >= 50 && teamStrength[NPC->playerTeam] > teamStrength[NPC->enemyTeam]))
                    {//Our team is strong enough to rush
                        teamCounter[NPC->playerTeam]++;
                        if(teamNumbers[NPC->playerTeam] * 17 <= teamCounter[NPC->playerTeam])
                        {//ok, we waited 1.7 think cycles on average and everyone is go, let's do it!
                            //FIXME: Should we do this to everyone on our team?
                            NPCInfo->behaviorState = BS_HUNT_AND_KILL;
                            //FIXME: if the tide changes, we should retreat!
                            //FIXME: when do we reset the counter?
                            NPC_BSHuntAndKill ();
                            return;
                        }
                    }
                    else//Oops!  Something's wrong, reset the counter to rush
                        teamCounter[NPC->playerTeam] = 0;
                }
                else//Oops!  Something's wrong, reset the counter to rush
                    teamCounter[NPC->playerTeam] = 0;
            }
        }
        */
    }

    NPC_CheckEnemy(1, 0); // qtrue, qfalse

    if (*NPCInfo).duckDebounceTime > (*level).time && (*(*NPC).client).ps.weapon != WP_SABER {
        ucmd.upmove = -127;
        if (*NPC).enemy != core::ptr::null_mut() {
            NPC_CheckCanAttack(1.0, 1); // qtrue
        }
        return;
    }

    if (*NPC).enemy != core::ptr::null_mut() {
        if NPC_StandTrackAndShoot(NPC, 1) == 0 {
            //That func didn't update our angles
            (*NPCInfo).desiredYaw = (*(*NPC).client).ps.viewangles[YAW];
            (*NPCInfo).desiredPitch = (*(*NPC).client).ps.viewangles[PITCH];
            NPC_UpdateAngles(1, 1); // qtrue, qtrue
        }
    } else {
        (*NPCInfo).desiredYaw = (*(*NPC).client).ps.viewangles[YAW];
        (*NPCInfo).desiredPitch = (*(*NPC).client).ps.viewangles[PITCH];
        NPC_UpdateAngles(1, 1); // qtrue, qtrue
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

    NPC_CheckEnemy(1, 0); // qtrue, qfalse

    if (*NPCInfo).duckDebounceTime > (*level).time {
        // && NPCInfo->hidingGoal )
        ucmd.upmove = -127;
        if (*NPC).enemy != core::ptr::null_mut() {
            NPC_CheckCanAttack(1.0, 0); // qfalse
        }
        return;
    }

    if (*NPC).enemy != core::ptr::null_mut() {
        let monitor: c_int = (*NPC).cantHitEnemyCounter;
        NPC_StandTrackAndShoot(NPC, 0); //(NPCInfo->hidingGoal != NULL) );

        if (ucmd.buttons & BUTTON_ATTACK) == 0 && ucmd.upmove >= 0 && (*NPC).cantHitEnemyCounter > monitor
        {
            //not crouching and not firing
            let mut vec: [f32; 3] = [0.0; 3];

            VectorSubtract(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin, &mut vec);
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
                NPC_MoveToGoal(1); // qtrue
                NPC_UpdateAngles(1, 1); // qtrue, qtrue
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
    if NPC_UpdateAngles(1, 1) != 0 {
        // qtrue, qtrue
        Q3_TaskIDComplete(NPC, TID_BSTATE);

        (*NPCInfo).desiredYaw = (*client).ps.viewangles[YAW];
        (*NPCInfo).desiredPitch = (*client).ps.viewangles[PITCH];

        (*NPCInfo).aimTime = 0; //ok to turn normally now
    }
}

pub unsafe fn NPC_BSPointShoot(shoot: qboolean) {
    //FIXME: doesn't check for clear shot...
    let mut muzzle: [f32; 3] = [0.0; 3];
    let mut dir: [f32; 3] = [0.0; 3];
    let mut angles: [f32; 3] = [0.0; 3];
    let mut org: [f32; 3] = [0.0; 3];

    if (*NPC).enemy == core::ptr::null_mut()
        || (*(*NPC).enemy).inuse == 0
        || ((*(*NPC).enemy).NPC != 0 && (*(*NPC).enemy).health <= 0)
    {
        //FIXME: should still keep shooting for a second or two after they actually die...
        Q3_TaskIDComplete(NPC, TID_BSTATE);
        return;
    }

    CalcEntitySpot(NPC, SPOT_WEAPON, &mut muzzle);
    CalcEntitySpot((*NPC).enemy, SPOT_HEAD, &mut org); //Was spot_org
    //Head is a little high, so let's aim for the chest:
    if (*(*NPC).enemy).client != core::ptr::null_mut() {
        org[2] -= 12.0; //NOTE: is this enough?
    }

    VectorSubtract(&org, &muzzle, &mut dir);
    vectoangles(&dir, &mut angles);

    match (*(*NPC).client).ps.weapon {
        WP_NONE => {
            //	case WP_TRICORDER:
            //don't do any pitch change if not holding a firing weapon
        }
        WP_MELEE | WP_TUSKEN_STAFF | WP_SABER => {
            //don't do any pitch change if not holding a firing weapon
        }
        _ => {
            (*NPCInfo).desiredPitch = AngleNormalize360(angles[PITCH]);
            (*NPCInfo).lockedDesiredPitch = (*NPCInfo).desiredPitch;
        }
    }

    (*NPCInfo).desiredYaw = AngleNormalize360(angles[YAW]);
    (*NPCInfo).lockedDesiredYaw = (*NPCInfo).desiredYaw;

    if NPC_UpdateAngles(1, 1) != 0 {
        // qtrue, qtrue
        //FIXME: if angles clamped, this may never work!
        //NPCInfo->shotTime = NPC->attackDebounceTime = 0;

        if shoot != 0 {
            //FIXME: needs to hold this down if using a weapon that requires it, like phaser...
            ucmd.buttons |= BUTTON_ATTACK;
        }

        if shoot == 0 || ((*NPC).svFlags & SVF_LOCKEDENEMY) == 0 {
            //If locked_enemy is on, dont complete until it is destroyed...
            Q3_TaskIDComplete(NPC, TID_BSTATE);
            return;
        }
    } else if shoot != 0 && ((*NPC).svFlags & SVF_LOCKEDENEMY) != 0 {
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
            let mut yaw_miss: f32;
            let mut yaw_miss_allow: f32 = (*(*NPC).enemy).maxs[0];
            let mut pitch_miss: f32;
            let pitch_miss_allow: f32 = ((*(*NPC).enemy).maxs[2] - (*(*NPC).enemy).mins[2]) / 2.0;

            if yaw_miss_allow < 8.0 {
                yaw_miss_allow = 8.0;
            }

            if pitch_miss_allow < 8.0 {
                // pitch_miss_allow = 8.0;  // Note: pitch_miss_allow is const
            }

            yaw_miss = tan(DEG2RAD(AngleDelta(
                (*(*NPC).client).ps.viewangles[YAW],
                (*NPCInfo).desiredYaw,
            ))) * dist;
            pitch_miss = tan(DEG2RAD(AngleDelta(
                (*(*NPC).client).ps.viewangles[PITCH],
                (*NPCInfo).desiredPitch,
            ))) * dist;

            if yaw_miss_allow >= yaw_miss && pitch_miss_allow > pitch_miss {
                ucmd.buttons |= BUTTON_ATTACK;
            }
        }
    }
}

/*
void NPC_BSMove(void)
Move in a direction, face another
*/
pub unsafe fn NPC_BSMove() {
    let mut goal: *mut gentity_t = core::ptr::null_mut();

    NPC_CheckEnemy(1, 0); // qtrue, qfalse
    if (*NPC).enemy != core::ptr::null_mut() {
        NPC_CheckCanAttack(1.0, 0); // qfalse
    } else {
        NPC_UpdateAngles(1, 1); // qtrue, qtrue
    }

    goal = UpdateGoal();
    if goal != core::ptr::null_mut() {
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

    // enemyVisibility = VIS_SHOOT;  // Note: enemyVisibility is a global that must be set

    if (*client).ps.weaponstate != WEAPON_READY && (*client).ps.weaponstate != WEAPON_FIRING {
        (*client).ps.weaponstate = WEAPON_READY;
    }

    WeaponThink(1); // qtrue
}

/*
void NPC_BSPatrol( void )

  Same as idle, but you look for enemies every "vigilance"
  using your angles, HFOV, VFOV and visrange, and listen for sounds within earshot...
*/
pub unsafe fn NPC_BSPatrol() {
    //int	alertEventNum;

    if (*level).time > (*NPCInfo).enemyCheckDebounceTime {
        (*NPCInfo).enemyCheckDebounceTime =
            (*level).time + ((*NPCInfo).stats.vigilance as c_int * 1000);
        NPC_CheckEnemy(1, 0); // qtrue, qfalse
        if (*NPC).enemy != core::ptr::null_mut() {
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
    if UpdateGoal() != core::ptr::null_mut() {
        NPC_MoveToGoal(1); // qtrue
    }

    NPC_UpdateAngles(1, 1); // qtrue, qtrue

    ucmd.buttons |= BUTTON_WALKING;
}

/*
void NPC_BSDefault(void)
    uses various scriptflags to determine how an npc should behave
*/
extern "C" {
    pub fn NPC_CheckGetNewWeapon();
    pub fn NPC_BSST_Attack();
}

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
    let mut move_val: qboolean = 1; // qtrue
                                     //	qboolean	shoot = qfalse;

    if (*NPCInfo).scriptFlags & SCF_FIRE_WEAPON != 0 {
        WeaponThink(1); // qtrue
    }

    if (*NPCInfo).scriptFlags & SCF_FORCED_MARCH != 0 {
        //being forced to walk
        if (*(*NPC).client).ps.torsoAnim != TORSO_SURRENDER_START {
            NPC_SetAnim(NPC, SETANIM_TORSO, TORSO_SURRENDER_START, SETANIM_FLAG_HOLD);
        }
    }
    //look for a new enemy if don't have one and are allowed to look, validate current enemy if have one
    NPC_CheckEnemy(
        if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            1
        } else {
            0
        },
        0, // qfalse
    );
    if (*NPC).enemy == core::ptr::null_mut() {
        //still don't have an enemy
        if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            //check for alert events
            //FIXME: Check Alert events, see if we should investigate or just look at it
            let alert_event: c_int = NPC_CheckAlertEvents(1, 1, -1, 1, AEL_DISCOVERED); // qtrue, qtrue, qtrue

            //There is an event to look at
            if alert_event >= 0 {
                //&& level.alertEvents[alertEvent].ID != NPCInfo->lastAlertID )
                //heard/saw something
                if (*level).alertEvents[alert_event as usize].level >= AEL_DISCOVERED
                    && ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0
                {
                    //was a big event
                    if (*level).alertEvents[alert_event as usize].owner != core::ptr::null_mut()
                        && (*level).alertEvents[alert_event as usize].owner != NPC
                        && (*(*level).alertEvents[alert_event as usize].owner).client != core::ptr::null_mut()
                        && (*(*level).alertEvents[alert_event as usize].owner).health >= 0
                        && (*(*(*level).alertEvents[alert_event as usize].owner).client).playerTeam == (*(*NPC).client).enemyTeam
                    {
                        //an enemy
                        G_SetEnemy(NPC, (*level).alertEvents[alert_event as usize].owner);
                    }
                } else {
                    //FIXME: investigate lesser events
                }
            }
            //FIXME: also check our allies' condition?
        }
    }

    if (*NPC).enemy != core::ptr::null_mut() && ((*NPCInfo).scriptFlags & SCF_FORCED_MARCH) == 0 {
        // just use the stormtrooper attack AI...
        NPC_CheckGetNewWeapon();
        if (*NPC).client != core::ptr::null_mut()
            && (*(*NPC).client).leader != core::ptr::null_mut()
            && (*NPCInfo).goalEntity == (*(*NPC).client).leader
            && Q3_TaskIDPending(NPC, TID_MOVE_NAV) == 0
        {
            NPC_ClearGoal();
        }
        NPC_BSST_Attack();
        return;
        /*
        //have an enemy
        //FIXME: if one of these fails, meaning we can't shoot, do we really need to do the rest?
        VectorSubtract( NPC->enemy->currentOrigin, NPC->currentOrigin, enemyDir );
        enemyDist = VectorNormalize( enemyDir );
        enemyDist *= enemyDist;
        shootDist = NPC_MaxDistSquaredForWeapon();

        enemyFOV = InFOV( NPC->enemy, NPC, NPCInfo->stats.hfov, NPCInfo->stats.vfov );
        enemyShotFOV = InFOV( NPC->enemy, NPC, 20, 20 );
        enemyPVS = gi.inPVS( NPC->enemy->currentOrigin, NPC->currentOrigin );

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
            NPC_CheckEnemy( qtrue, qfalse );
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
        if (*NPC).enemy == core::ptr::null_mut()
            && (*NPC).client != core::ptr::null_mut()
            && (*(*NPC).client).leader != core::ptr::null_mut()
            && (*NPCInfo).goalEntity == (*(*NPC).client).leader
            && Q3_TaskIDPending(NPC, TID_MOVE_NAV) == 0
        {
            NPC_BSFollowLeader();
        } else {
            //set angles
            if ((*NPCInfo).scriptFlags & SCF_FACE_MOVE_DIR) != 0
                || (*NPCInfo).goalEntity != (*NPC).enemy
            {
                //face direction of movement, NOTE: default behavior when not chasing enemy
                (*NPCInfo).combatMove = 0; // qfalse
            } else {
                //face goal.. FIXME: what if have a navgoal but want to face enemy while moving?  Will this do that?
                let mut dir: [f32; 3] = [0.0; 3];
                let mut angles: [f32; 3] = [0.0; 3];

                (*NPCInfo).combatMove = 0; // qfalse

                VectorSubtract(&(*(*NPCInfo).goalEntity).currentOrigin, &(*NPC).currentOrigin, &mut dir);
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
                if g_crosshairEntNum != (*NPC).s.number {
                    //don't walk if player isn't aiming at me
                    move_val = 0; // qfalse
                }
            }

            if move_val != 0 {
                //move toward goal
                NPC_MoveToGoal(1); // qtrue
            }
        }
    } else if (*NPC).enemy == core::ptr::null_mut()
        && (*NPC).client != core::ptr::null_mut()
        && (*(*NPC).client).leader != core::ptr::null_mut()
    {
        NPC_BSFollowLeader();
    }

    //update angles
    NPC_UpdateAngles(1, 1); // qtrue, qtrue
}
