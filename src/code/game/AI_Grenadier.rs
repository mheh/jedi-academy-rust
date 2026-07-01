// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
#![allow(
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_mut,
    unused_assignments,
    unused_imports,
    non_camel_case_types,
    unused_unsafe,
)]
use crate::code::game::g_headers_h::*;

use crate::code::game::b_local_h::*;
use crate::code::game::g_nav_h::*;
use crate::code::game::anims_h::*;
use crate::code::game::g_navigator_h::*;

use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, addr_of_mut, null_mut};

extern "C" {
    fn CG_DrawAlert(origin: vec3_t, rating: f32);
    fn G_AddVoiceEvent(self_: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    fn NPC_TempLookTarget(
        self_: *mut gentity_t,
        lookEntNum: c_int,
        minLookTime: c_int,
        maxLookTime: c_int,
    );
    fn G_ExpandPointToBBox(
        point: vec3_t,
        mins: *const f32,
        maxs: *const f32,
        ignore: c_int,
        clipmask: c_int,
    ) -> qboolean;
    fn NPC_AimAdjust(change: c_int);
    fn FlyingCreature(ent: *mut gentity_t) -> qboolean;
}


const MAX_VIEW_DIST: c_int = 1024;
const MAX_VIEW_SPEED: c_int = 250;
const MAX_LIGHT_INTENSITY: c_int = 255;
const MIN_LIGHT_THRESHOLD: f32 = 0.1;

const DISTANCE_SCALE: f32 = 0.25;
const DISTANCE_THRESHOLD: f32 = 0.075;
const SPEED_SCALE: f32 = 0.25;
const FOV_SCALE: f32 = 0.5;
const LIGHT_SCALE: f32 = 0.25;

const REALIZE_THRESHOLD: f32 = 0.6;
const CAUTIOUS_THRESHOLD: f32 = REALIZE_THRESHOLD * 0.75;

// porting note: `qboolean NPC_CheckPlayerTeamStealth( void );` here in the original .cpp is only
// a forward declaration (no body in this file). It is already declared
// `pub fn NPC_CheckPlayerTeamStealth() -> qboolean;` in ai_h.rs, which is pulled in transitively
// via the `b_local_h` glob import (b_local.h includes AI.h). Deduped rather than re-declared here.

static mut enemyLOS: qboolean = qfalse;
static mut enemyCS: qboolean = qfalse;
static mut faceEnemy: qboolean = qfalse;
static mut move_: qboolean = qfalse;
static mut shoot: qboolean = qfalse;
static mut enemyDist: f32 = 0.0;

//Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_UNDERFIRE: c_int = 1;
const LSTATE_INVESTIGATE: c_int = 2;

pub unsafe extern "C" fn Grenadier_ClearTimers(ent: *mut gentity_t) {
    TIMER_Set(ent, b"chatter\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"duck\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"stand\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"shuffleTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"sleepTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"enemyLastVisible\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"roamTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"hideTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"attackDelay\0".as_ptr() as *const c_char, 0); //FIXME: Slant for difficulty levels
    TIMER_Set(ent, b"stick\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"scoutTime\0".as_ptr() as *const c_char, 0);
    TIMER_Set(ent, b"flee\0".as_ptr() as *const c_char, 0);
}

pub unsafe extern "C" fn NPC_Grenadier_PlayConfusionSound(self_: *mut gentity_t) {
    //FIXME: make this a custom sound in sound set
    if (*self_).health > 0 {
        G_AddVoiceEvent(self_, Q_irand(EV_CONFUSE1, EV_CONFUSE3), 2000);
    }
    //reset him to be totally unaware again
    TIMER_Set(self_, b"enemyLastVisible\0".as_ptr() as *const c_char, 0);
    TIMER_Set(self_, b"flee\0".as_ptr() as *const c_char, 0);
    (*(*self_).NPC).squadState = SQUAD_IDLE;
    (*(*self_).NPC).tempBehavior = BS_DEFAULT;

    //self->NPC->behaviorState = BS_PATROL;
    G_ClearEnemy(self_); //FIXME: or just self->enemy = NULL;?

    (*(*self_).NPC).investigateCount = 0;
}


/*
-------------------------
NPC_ST_Pain
-------------------------
*/

pub unsafe extern "C" fn NPC_Grenadier_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: vec3_t,
    damage: c_int,
    mod_: c_int,
) {
    (*(*self_).NPC).localState = LSTATE_UNDERFIRE;

    TIMER_Set(self_, b"duck\0".as_ptr() as *const c_char, -1);
    TIMER_Set(self_, b"stand\0".as_ptr() as *const c_char, 2000);

    NPC_Pain(self_, inflictor, other, point, damage, mod_);

    if damage == 0 && (*self_).health > 0 {
        //FIXME: better way to know I was pushed
        G_AddVoiceEvent(self_, Q_irand(EV_PUSHED1, EV_PUSHED3), 2000);
    }
}

/*
-------------------------
ST_HoldPosition
-------------------------
*/

unsafe fn Grenadier_HoldPosition() {
    NPC_FreeCombatPoint((*NPCInfo).combatPoint, qtrue);
    (*NPCInfo).goalEntity = null_mut();

    /*if ( TIMER_Done( NPC, "stand" ) )
    {//FIXME: what if can't shoot from this pos?
        TIMER_Set( NPC, "duck", Q_irand( 2000, 4000 ) );
    }
    */
}

/*
-------------------------
ST_Move
-------------------------
*/

unsafe fn Grenadier_Move() -> qboolean {
    (*NPCInfo).combatMove = qtrue; //always move straight toward our goal

    let moved: qboolean = NPC_MoveToGoal(qtrue);
    //	navInfo_t	info;

    //Get the move info
    //	NAV_GetLastMove( info );

    //FIXME: if we bump into another one of our guys and can't get around him, just stop!
    //If we hit our target, then stop and fire!
    //	if ( info.flags & NIF_COLLISION )
    //	{
    //		if ( info.blocker == NPC->enemy )
    //		{
    //			Grenadier_HoldPosition();
    //		}
    //	}

    //If our move failed, then reset
    if moved == qfalse {
        //couldn't get to enemy
        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0
            && (*(*NPC).client).ps.weapon == WP_THERMAL
            && !(*NPCInfo).goalEntity.is_null()
            && (*NPCInfo).goalEntity == (*NPC).enemy
        {
            //we were running after enemy
            //Try to find a combat point that can hit the enemy
            let mut cpFlags: c_int = CP_CLEAR | CP_HAS_ROUTE;
            if ((*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST) != 0 {
                cpFlags &= !(CP_FLANK | CP_APPROACH_ENEMY | CP_CLOSEST);
                cpFlags |= CP_NEAREST;
            }
            let mut cp: c_int = NPC_FindCombatPoint(
                (*NPC).currentOrigin,
                (*NPC).currentOrigin,
                (*NPC).currentOrigin,
                cpFlags,
                32,
            );
            if cp == -1 && ((*NPCInfo).scriptFlags & SCF_USE_CP_NEAREST) == 0 {
                //okay, try one by the enemy
                cp = NPC_FindCombatPoint(
                    (*NPC).currentOrigin,
                    (*NPC).currentOrigin,
                    (*(*NPC).enemy).currentOrigin,
                    CP_CLEAR | CP_HAS_ROUTE | CP_HORZ_DIST_COLL,
                    32,
                );
            }
            //NOTE: there may be a perfectly valid one, just not one within CP_COLLECT_RADIUS of either me or him...
            if cp != -1 {
                //found a combat point that has a clear shot to enemy
                NPC_SetCombatPoint(cp);
                NPC_SetMoveGoal(
                    NPC,
                    (*addr_of!(level)).combatPoints[cp as usize].origin,
                    8,
                    qtrue,
                    cp,
                );
                return moved;
            }
        }
        //just hang here
        Grenadier_HoldPosition();
    }

    return moved;
}

/*
-------------------------
NPC_BSGrenadier_Patrol
-------------------------
*/

pub unsafe extern "C" fn NPC_BSGrenadier_Patrol() {
    //FIXME: pick up on bodies of dead buddies?
    if (*NPCInfo).confusionTime < (*addr_of!(level)).time {
        //Look for any enemies
        if ((*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
            if NPC_CheckPlayerTeamStealth() != 0 {
                //NPCInfo->behaviorState = BS_HUNT_AND_KILL;//should be automatic now
                //NPC_AngerSound();
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            }
        }

        if ((*NPCInfo).scriptFlags & SCF_IGNORE_ALERTS) == 0 {
            //Is there danger nearby
            let alertEvent: c_int = NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_SUSPICIOUS);
            if NPC_CheckForDanger(alertEvent) != 0 {
                NPC_UpdateAngles(qtrue, qtrue);
                return;
            } else {
                //check for other alert events
                //There is an event to look at
                if alertEvent >= 0
                //&& level.alertEvents[alertEvent].ID != NPCInfo->lastAlertID )
                {
                    //NPCInfo->lastAlertID = level.alertEvents[alertEvent].ID;
                    if (*addr_of!(level)).alertEvents[alertEvent as usize].level == AEL_DISCOVERED
                    {
                        if !(*addr_of!(level)).alertEvents[alertEvent as usize]
                            .owner
                            .is_null()
                            && !(*(*addr_of!(level)).alertEvents[alertEvent as usize].owner)
                                .client
                                .is_null()
                            && (*(*addr_of!(level)).alertEvents[alertEvent as usize].owner).health
                                >= 0
                            && (*(*(*addr_of!(level)).alertEvents[alertEvent as usize].owner)
                                .client)
                                .playerTeam
                                == (*(*NPC).client).enemyTeam
                        {
                            //an enemy
                            G_SetEnemy(
                                NPC,
                                (*addr_of!(level)).alertEvents[alertEvent as usize].owner,
                            );
                            //NPCInfo->enemyLastSeenTime = level.time;
                            TIMER_Set(
                                NPC,
                                b"attackDelay\0".as_ptr() as *const c_char,
                                Q_irand(500, 2500),
                            );
                        }
                    } else {
                        //FIXME: get more suspicious over time?
                        //Save the position for movement (if necessary)
                        VectorCopy(
                            (*addr_of!(level)).alertEvents[alertEvent as usize].position,
                            (*NPCInfo).investigateGoal,
                        );
                        (*NPCInfo).investigateDebounceTime =
                            (*addr_of!(level)).time + Q_irand(500, 1000);
                        if (*addr_of!(level)).alertEvents[alertEvent as usize].level
                            == AEL_SUSPICIOUS
                        {
                            //suspicious looks longer
                            (*NPCInfo).investigateDebounceTime += Q_irand(500, 2500);
                        }
                    }
                }
            }

            if (*NPCInfo).investigateDebounceTime > (*addr_of!(level)).time {
                //FIXME: walk over to it, maybe?  Not if not chase enemies
                //NOTE: stops walking or doing anything else below
                let mut dir: vec3_t = [0.0; 3];
                let mut angles: vec3_t = [0.0; 3];
                let o_yaw: f32;
                let o_pitch: f32;

                VectorSubtract(
                    (*NPCInfo).investigateGoal,
                    (*(*NPC).client).renderInfo.eyePoint,
                    dir,
                );
                vectoangles(dir, angles);

                o_yaw = (*NPCInfo).desiredYaw;
                o_pitch = (*NPCInfo).desiredPitch;
                (*NPCInfo).desiredYaw = angles[YAW as usize];
                (*NPCInfo).desiredPitch = angles[PITCH as usize];

                NPC_UpdateAngles(qtrue, qtrue);

                (*NPCInfo).desiredYaw = o_yaw;
                (*NPCInfo).desiredPitch = o_pitch;
                return;
            }
        }
    }

    //If we have somewhere to go, then do that
    if UpdateGoal() != 0 {
        (*addr_of_mut!(ucmd)).buttons |= BUTTON_WALKING;
        NPC_MoveToGoal(qtrue);
    }

    NPC_UpdateAngles(qtrue, qtrue);
}

/*
-------------------------
NPC_BSGrenadier_Idle
-------------------------
*/
/*
void NPC_BSGrenadier_Idle( void )
{
	//FIXME: check for other alert events?

	//Is there danger nearby?
	if ( NPC_CheckForDanger( NPC_CheckAlertEvents( qtrue, qtrue, -1, qfalse, AEL_DANGER ) ) )
	{
		NPC_UpdateAngles( qtrue, qtrue );
		return;
	}

	TIMER_Set( NPC, "roamTime", 2000 + Q_irand( 1000, 2000 ) );

	NPC_UpdateAngles( qtrue, qtrue );
}
*/
/*
-------------------------
ST_CheckMoveState
-------------------------
*/

unsafe fn Grenadier_CheckMoveState() {
    //See if we're a scout
    if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0
    //behaviorState == BS_STAND_AND_SHOOT )
    {
        if (*NPCInfo).goalEntity == (*NPC).enemy {
            move_ = qfalse;
            return;
        }
    }
    //See if we're running away
    else if (*NPCInfo).squadState == SQUAD_RETREAT {
        if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) != 0 {
            (*NPCInfo).squadState = SQUAD_IDLE;
        } else {
            faceEnemy = qfalse;
        }
    }
    /*
    else if ( NPCInfo->squadState == SQUAD_IDLE )
    {
        if ( !NPCInfo->goalEntity )
        {
            move = qfalse;
            return;
        }
        //Should keep moving toward player when we're out of range... right?
    }
    */

    //See if we're moving towards a goal, not the enemy
    if (*NPCInfo).goalEntity != (*NPC).enemy && !(*NPCInfo).goalEntity.is_null() {
        //Did we make it?
        if STEER::Reached(NPC, (*NPCInfo).goalEntity, 16.0, FlyingCreature(NPC) != 0)
            || ((*NPCInfo).squadState == SQUAD_SCOUT && enemyLOS != 0 && enemyDist <= 10000.0)
        {
            let mut newSquadState: c_int = SQUAD_STAND_AND_SHOOT;
            //we got where we wanted to go, set timers based on why we were running
            match (*NPCInfo).squadState {
                SQUAD_RETREAT => {
                    //was running away
                    TIMER_Set(
                        NPC,
                        b"duck\0".as_ptr() as *const c_char,
                        ((*NPC).max_health - (*NPC).health) * 100,
                    );
                    TIMER_Set(
                        NPC,
                        b"hideTime\0".as_ptr() as *const c_char,
                        Q_irand(3000, 7000),
                    );
                    newSquadState = SQUAD_COVER;
                }
                SQUAD_TRANSITION => {
                    //was heading for a combat point
                    TIMER_Set(
                        NPC,
                        b"hideTime\0".as_ptr() as *const c_char,
                        Q_irand(2000, 4000),
                    );
                }
                SQUAD_SCOUT => {
                    //was running after player
                }
                _ => {}
            }
            NPC_ReachedGoal();
            //don't attack right away
            TIMER_Set(
                NPC,
                b"attackDelay\0".as_ptr() as *const c_char,
                Q_irand(250, 500),
            ); //FIXME: Slant for difficulty levels
               //don't do something else just yet
            TIMER_Set(
                NPC,
                b"roamTime\0".as_ptr() as *const c_char,
                Q_irand(1000, 4000),
            );
            //stop fleeing
            if (*NPCInfo).squadState == SQUAD_RETREAT {
                TIMER_Set(
                    NPC,
                    b"flee\0".as_ptr() as *const c_char,
                    -(*addr_of!(level)).time,
                );
                (*NPCInfo).squadState = SQUAD_IDLE;
            }
            return;
        }

        //keep going, hold of roamTimer until we get there
        TIMER_Set(
            NPC,
            b"roamTime\0".as_ptr() as *const c_char,
            Q_irand(4000, 8000),
        );
    }

    if (*NPCInfo).goalEntity.is_null() {
        if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) != 0 {
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = (*NPC).maxs[0] * 1.5;
        }
    }
}

/*
-------------------------
ST_CheckFireState
-------------------------
*/

unsafe fn Grenadier_CheckFireState() {
    if enemyCS != 0 {
        //if have a clear shot, always try
        return;
    }

    if (*NPCInfo).squadState == SQUAD_RETREAT
        || (*NPCInfo).squadState == SQUAD_TRANSITION
        || (*NPCInfo).squadState == SQUAD_SCOUT
    {
        //runners never try to fire at the last pos
        return;
    }

    if VectorCompare((*(*NPC).client).ps.velocity, vec3_origin) == 0 {
        //if moving at all, don't do this
        return;
    }

    //continue to fire on their last position
    /*
    if ( !Q_irand( 0, 1 ) && NPCInfo->enemyLastSeenTime && level.time - NPCInfo->enemyLastSeenTime < 4000 )
    {
        //Fire on the last known position
        vec3_t	muzzle, dir, angles;

        CalcEntitySpot( NPC, SPOT_WEAPON, muzzle );
        VectorSubtract( NPCInfo->enemyLastSeenLocation, muzzle, dir );

        VectorNormalize( dir );

        vectoangles( dir, angles );

        NPCInfo->desiredYaw		= angles[YAW];
        NPCInfo->desiredPitch	= angles[PITCH];
        //FIXME: they always throw toward enemy, so this will be very odd...
        shoot = qtrue;
        faceEnemy = qfalse;

        return;
    }
    */
}

pub unsafe extern "C" fn Grenadier_EvaluateShot(hit: c_int) -> qboolean {
    if (*NPC).enemy.is_null() {
        return qfalse;
    }

    if hit == (*(*NPC).enemy).s.number
        || (!addr_of!(g_entities[hit as usize]).is_null()
            && (g_entities[hit as usize].svFlags & SVF_GLASS_BRUSH) != 0)
    {
        //can hit enemy or will hit glass, so shoot anyway
        return qtrue;
    }
    return qfalse;
}

/*
-------------------------
NPC_BSGrenadier_Attack
-------------------------
*/

pub unsafe extern "C" fn NPC_BSGrenadier_Attack() {
    //Don't do anything if we're hurt
    if (*NPC).painDebounceTime > (*addr_of!(level)).time {
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    //NPC_CheckEnemy( qtrue, qfalse );
    //If we don't have an enemy, just idle
    if NPC_CheckEnemyExt() == qfalse
    // !NPC->enemy )//
    {
        NPC_BSGrenadier_Patrol(); //FIXME: or patrol?
        return;
    }

    if TIMER_Done(NPC, b"flee\0".as_ptr() as *const c_char) != 0
        && NPC_CheckForDanger(NPC_CheckAlertEvents(qtrue, qtrue, -1, qfalse, AEL_DANGER)) != 0
    {
        //going to run
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    if (*NPC).enemy.is_null() {
        //WTF?  somehow we lost our enemy?
        NPC_BSGrenadier_Patrol(); //FIXME: or patrol?
        return;
    }

    enemyLOS = qfalse;
    enemyCS = qfalse;
    move_ = qtrue;
    faceEnemy = qfalse;
    shoot = qfalse;
    enemyDist = DistanceSquared((*(*NPC).enemy).currentOrigin, (*NPC).currentOrigin);

    //See if we should switch to melee attack
    if enemyDist < 16384.0
        && ((*(*NPC).enemy).client.is_null()
            || (*(*(*NPC).enemy).client).ps.weapon != WP_SABER
            || (*(*(*NPC).enemy).client).ps.SaberActive() == 0)
    //128
    {
        //enemy is close and not using saber
        if (*(*NPC).client).ps.weapon == WP_THERMAL {
            //grenadier
            let mut trace: trace_t = core::mem::zeroed();
            ((*addr_of!(gi)).trace)(
                addr_of_mut!(trace),
                (*NPC).currentOrigin,
                (*(*NPC).enemy).mins,
                (*(*NPC).enemy).maxs,
                (*(*NPC).enemy).currentOrigin,
                (*NPC).s.number,
                (*(*NPC).enemy).clipmask,
            );
            if trace.allsolid == 0
                && trace.startsolid == 0
                && (trace.fraction == 1.0 || trace.entityNum == (*(*NPC).enemy).s.number)
            {
                //I can get right to him
                //reset fire-timing variables
                NPC_ChangeWeapon(WP_MELEE);
                if ((*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES) == 0
                //NPCInfo->behaviorState == BS_STAND_AND_SHOOT )
                {
                    //FIXME: should we be overriding scriptFlags?
                    (*NPCInfo).scriptFlags |= SCF_CHASE_ENEMIES; //NPCInfo->behaviorState = BS_HUNT_AND_KILL;
                }
            }
        }
    } else if enemyDist > 65536.0
        || (!(*(*NPC).enemy).client.is_null()
            && (*(*(*NPC).enemy).client).ps.weapon == WP_SABER
            && (*(*(*NPC).enemy).client).ps.SaberActive() != 0)
    //256
    {
        //enemy is far or using saber
        if (*(*NPC).client).ps.weapon == WP_MELEE
            && ((*(*NPC).client).ps.stats[STAT_WEAPONS as usize] & (1 << WP_THERMAL)) != 0
        {
            //fisticuffs, make switch to thermal if have it
            //reset fire-timing variables
            NPC_ChangeWeapon(WP_THERMAL);
        }
    }

    //can we see our target?
    if NPC_ClearLOS((*NPC).enemy) != 0 {
        (*NPCInfo).enemyLastSeenTime = (*addr_of!(level)).time;
        enemyLOS = qtrue;

        if (*(*NPC).client).ps.weapon == WP_MELEE {
            if enemyDist <= 4096.0
                && InFOV(
                    (*(*NPC).enemy).currentOrigin,
                    (*NPC).currentOrigin,
                    (*(*NPC).client).ps.viewangles,
                    90,
                    45,
                ) != 0
            //within 64 & infront
            {
                VectorCopy(
                    (*(*NPC).enemy).currentOrigin,
                    (*NPCInfo).enemyLastSeenLocation,
                );
                enemyCS = qtrue;
            }
        } else if InFOV(
            (*(*NPC).enemy).currentOrigin,
            (*NPC).currentOrigin,
            (*(*NPC).client).ps.viewangles,
            45,
            90,
        ) != 0
        {
            //in front of me
            //can we shoot our target?
            //FIXME: how accurate/necessary is this check?
            let hit: c_int = NPC_ShotEntity((*NPC).enemy);
            let hitEnt: *mut gentity_t = addr_of_mut!(g_entities[hit as usize]);
            if hit == (*(*NPC).enemy).s.number
                || (!hitEnt.is_null()
                    && !(*hitEnt).client.is_null()
                    && (*(*hitEnt).client).playerTeam == (*(*NPC).client).enemyTeam)
            {
                VectorCopy(
                    (*(*NPC).enemy).currentOrigin,
                    (*NPCInfo).enemyLastSeenLocation,
                );
                let enemyHorzDist: f32 = DistanceHorizontalSquared(
                    (*(*NPC).enemy).currentOrigin,
                    (*NPC).currentOrigin,
                );
                if enemyHorzDist < 1048576.0 {
                    //within 1024
                    enemyCS = qtrue;
                    NPC_AimAdjust(2); //adjust aim better longer we have clear shot at enemy
                } else {
                    NPC_AimAdjust(1); //adjust aim better longer we can see enemy
                }
            }
        }
    } else {
        NPC_AimAdjust(-1); //adjust aim worse longer we cannot see enemy
    }
    /*
    else if ( gi.inPVS( NPC->enemy->currentOrigin, NPC->currentOrigin ) )
    {
        NPCInfo->enemyLastSeenTime = level.time;
        faceEnemy = qtrue;
    }
    */

    if enemyLOS != 0 {
        //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
        faceEnemy = qtrue;
    }

    if enemyCS != 0 {
        shoot = qtrue;
        if (*(*NPC).client).ps.weapon == WP_THERMAL {
            //don't chase and throw
            move_ = qfalse;
        } else if (*(*NPC).client).ps.weapon == WP_MELEE
            && enemyDist
                < ((*NPC).maxs[0] + (*(*NPC).enemy).maxs[0] + 16.0)
                    * ((*NPC).maxs[0] + (*(*NPC).enemy).maxs[0] + 16.0)
        {
            //close enough
            move_ = qfalse;
        }
    } //this should make him chase enemy when out of range...?

    //Check for movement to take care of
    Grenadier_CheckMoveState();

    //See if we should override shooting decision with any special considerations
    Grenadier_CheckFireState();

    if move_ != 0 {
        //move toward goal
        if !(*NPCInfo).goalEntity.is_null()
        //&& ( NPCInfo->goalEntity != NPC->enemy || enemyDist > 10000 ) )//100 squared
        {
            move_ = Grenadier_Move();
        } else {
            move_ = qfalse;
        }
    }

    if move_ == 0 {
        if TIMER_Done(NPC, b"duck\0".as_ptr() as *const c_char) == 0 {
            (*addr_of_mut!(ucmd)).upmove = -127;
        }
        //FIXME: what about leaning?
    } else {
        //stop ducking!
        TIMER_Set(NPC, b"duck\0".as_ptr() as *const c_char, -1);
    }

    if faceEnemy == 0 {
        //we want to face in the dir we're running
        if move_ != 0 {
            //don't run away and shoot
            (*NPCInfo).desiredYaw = (*NPCInfo).lastPathAngles[YAW as usize];
            (*NPCInfo).desiredPitch = 0.0;
            shoot = qfalse;
        }
        NPC_UpdateAngles(qtrue, qtrue);
    } else {
        // if ( faceEnemy )
        //face the enemy
        NPC_FaceEnemy();
    }

    if ((*NPCInfo).scriptFlags & SCF_DONT_FIRE) != 0 {
        shoot = qfalse;
    }

    //FIXME: don't shoot right away!
    if shoot != 0 {
        //try to shoot if it's time
        if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != 0 {
            if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                // we've already fired, no need to do it again here
                WeaponThink(qtrue);
                TIMER_Set(
                    NPC,
                    b"attackDelay\0".as_ptr() as *const c_char,
                    (*NPCInfo).shotTime - (*addr_of!(level)).time,
                );
            }
        }
    }
}

pub unsafe extern "C" fn NPC_BSGrenadier_Default() {
    if ((*NPCInfo).scriptFlags & SCF_FIRE_WEAPON) != 0 {
        WeaponThink(qtrue);
    }

    if (*NPC).enemy.is_null() {
        //don't have an enemy, look for one
        NPC_BSGrenadier_Patrol();
    } else {
        //if ( NPC->enemy )
        //have an enemy
        NPC_BSGrenadier_Attack();
    }
}
